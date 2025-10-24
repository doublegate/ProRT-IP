//! TCP SYN scan implementation
//!
//! SYN scanning (also known as half-open scanning) is a stealthy port scanning technique
//! that doesn't complete the full TCP 3-way handshake.
//!
//! ## How it works
//!
//! 1. Send SYN packet to target port
//! 2. Wait for response:
//!    - SYN/ACK = port open
//!    - RST = port closed
//!    - No response = port filtered
//! 3. Send RST to tear down connection (stealth)
//!
//! ## Advantages
//!
//! - Faster than connect scans (no full handshake)
//! - Stealthier (many IDS don't log half-open connections)
//! - Requires raw socket privileges
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::SynScanner;
//! use prtip_core::{Config, ScanTarget};
//! use std::net::Ipv4Addr;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! let config = Config::default();
//! let scanner = SynScanner::new(config)?;
//!
//! let target = Ipv4Addr::new(192, 168, 1, 1);
//! let result = scanner.scan_port(target, 80).await?;
//!
//! println!("Port 80 state: {:?}", result.state);
//! # Ok(())
//! # }
//! ```

use dashmap::DashMap;
use parking_lot::Mutex;
use prtip_core::{Config, PortState, Result, ScanResult};
use prtip_network::{
    create_capture, packet_buffer::with_buffer, PacketCapture, TcpFlags, TcpOption,
    TcpPacketBuilder,
};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::{debug, trace, warn};

// PCAPNG packet capture support
use crate::pcapng::{Direction, PcapngWriter};
use std::sync::Mutex as StdMutex;

/// Connection state for tracking SYN scan responses
#[derive(Debug, Clone)]
struct ConnectionState {
    /// Target IP address
    target_ip: Ipv4Addr,
    /// Target port
    target_port: u16,
    /// Source port used
    source_port: u16,
    /// Sequence number sent
    sequence: u32,
    /// Time the SYN was sent
    sent_time: Instant,
    /// Number of retries attempted
    retries: u8,
}

/// Connection tracking table (lock-free with DashMap for Phase 4 performance)
type ConnectionTable = Arc<DashMap<(Ipv4Addr, u16, u16), ConnectionState>>;

/// SYN scanner with raw packet support
pub struct SynScanner {
    config: Config,
    capture: Arc<Mutex<Option<Box<dyn PacketCapture>>>>,
    connections: ConnectionTable,
    local_ip: Ipv4Addr,
}

impl SynScanner {
    /// Create a new SYN scanner
    pub fn new(config: Config) -> Result<Self> {
        // Get local IP address (simplified - in production would detect interface)
        let local_ip = Self::detect_local_ip()?;

        Ok(Self {
            config,
            capture: Arc::new(Mutex::new(None)),
            connections: Arc::new(DashMap::new()),
            local_ip,
        })
    }

    /// Initialize packet capture
    pub async fn initialize(&mut self) -> Result<()> {
        let mut capture = create_capture()?;
        capture.open(None)?; // Auto-detect interface
        *self.capture.lock() = Some(capture);
        Ok(())
    }

    /// Detect local IP address for the interface
    fn detect_local_ip() -> Result<Ipv4Addr> {
        // Simplified detection - in production would use interface detection
        // For now, use a placeholder
        Ok(Ipv4Addr::new(192, 168, 1, 100))
    }

    /// Scan a single port
    pub async fn scan_port(&self, target: Ipv4Addr, port: u16) -> Result<ScanResult> {
        self.scan_port_with_pcapng(target, port, None).await
    }

    /// Scan a single port with optional PCAPNG packet capture
    pub async fn scan_port_with_pcapng(
        &self,
        target: Ipv4Addr,
        port: u16,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<ScanResult> {
        let start_time = Instant::now();

        // Generate random source port
        use rand::Rng;
        let src_port: u16 = rand::thread_rng().gen_range(1024..65535);

        // Send initial SYN
        let sequence = self.send_syn(target, port, src_port, 0, pcapng_writer.clone()).await?;

        // Track connection
        let conn_state = ConnectionState {
            target_ip: target,
            target_port: port,
            source_port: src_port,
            sequence,
            sent_time: start_time,
            retries: 0,
        };

        self.connections
            .insert((target, port, src_port), conn_state.clone());

        // Wait for response with retries
        let max_retries = self.config.scan.retries;
        let timeout_ms = self.config.scan.timeout_ms;

        for retry in 0..=max_retries {
            let wait_duration = Duration::from_millis(timeout_ms);

            // Update retry count in connection state
            if let Some(mut conn) = self.connections.get_mut(&(target, port, src_port)) {
                conn.retries = retry as u8;
            }

            match timeout(
                wait_duration,
                self.wait_for_response(target, port, src_port, pcapng_writer.clone()),
            )
            .await
            {
                Ok(Ok(state)) => {
                    // Cleanup connection tracking (DashMap returns (key, value) tuple)
                    let conn_state = self
                        .connections
                        .remove(&(target, port, src_port))
                        .map(|(_, v)| v);

                    // Send RST to close connection if it was open
                    if state == PortState::Open {
                        let _ = self.send_rst(target, port, src_port, sequence + 1).await;
                    }

                    // Calculate response time from tracked sent_time
                    let response_time = if let Some(conn) = conn_state {
                        conn.sent_time.elapsed()
                    } else {
                        start_time.elapsed()
                    };

                    return Ok(ScanResult::new(IpAddr::V4(target), port, state)
                        .with_response_time(response_time));
                }
                Ok(Err(e)) => {
                    warn!("Error waiting for response: {}", e);
                    break;
                }
                Err(_) => {
                    // Timeout - retry if we haven't exceeded max retries
                    if retry < max_retries {
                        // Get connection state for detailed logging
                        let conn_info =
                            self.connections.get(&(target, port, src_port)).map(|conn| {
                                format!(
                                    "{}:{} -> src_port={}, seq={:#x}, elapsed={:?}, retries={}",
                                    conn.target_ip,
                                    conn.target_port,
                                    conn.source_port,
                                    conn.sequence,
                                    conn.sent_time.elapsed(),
                                    conn.retries
                                )
                            });

                        debug!(
                            "Timeout waiting for connection, retry {}/{}: {}",
                            retry + 1,
                            max_retries,
                            conn_info.unwrap_or_else(|| format!("{}:{} (no state)", target, port))
                        );

                        // Exponential backoff
                        let backoff = Duration::from_millis(timeout_ms * (1 << retry));
                        tokio::time::sleep(backoff).await;

                        // Resend SYN
                        self.send_syn(target, port, src_port, (retry + 1) as u8, pcapng_writer.clone())
                            .await?;
                    }
                }
            }
        }

        // No response after all retries - port is filtered
        self.connections.remove(&(target, port, src_port));
        let response_time = start_time.elapsed();

        Ok(
            ScanResult::new(IpAddr::V4(target), port, PortState::Filtered)
                .with_response_time(response_time),
        )
    }

    /// Send a SYN packet
    ///
    /// Sprint 4.17 Phase 3: Integrated zero-copy packet building.
    /// Uses `with_buffer()` closure and `build_ip_packet_with_buffer()` to
    /// eliminate heap allocations in packet crafting hot path.
    ///
    /// Sprint 4.18: Added optional PCAPNG packet capture.
    async fn send_syn(
        &self,
        target: Ipv4Addr,
        port: u16,
        src_port: u16,
        retry: u8,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<u32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Generate sequence number (for stateless, could use SipHash)
        let sequence: u32 = rng.gen();

        // Build and send SYN packet using zero-copy API
        with_buffer(|pool| {
            // Build SYN packet (zero allocations)
            let packet = TcpPacketBuilder::new()
                .source_ip(self.local_ip)
                .dest_ip(target)
                .source_port(src_port)
                .dest_port(port)
                .sequence(sequence)
                .flags(TcpFlags::SYN)
                .window(65535)
                .add_option(TcpOption::Mss(1460))
                .add_option(TcpOption::WindowScale(7))
                .add_option(TcpOption::SackPermitted)
                .build_ip_packet_with_buffer(pool)?;

            // Capture packet to PCAPNG if writer is provided
            if let Some(ref writer) = pcapng_writer {
                // Clone packet data before sending (PCAPNG needs owned copy)
                let packet_data = packet.to_vec();
                if let Ok(guard) = writer.lock() {
                    if let Err(e) = guard.write_packet(&packet_data, Direction::Sent) {
                        // Log error but don't fail scan (PCAPNG is optional)
                        warn!("PCAPNG write error (SYN packet): {}", e);
                    }
                }
            }

            // Send packet (while borrowed from pool)
            if let Some(ref mut capture) = *self.capture.lock() {
                capture.send_packet(packet).map_err(|e| {
                    prtip_core::Error::Network(format!("Failed to send SYN: {}", e))
                })?;

                trace!(
                    "Sent SYN to {}:{} (src_port={}, seq={}, retry={})",
                    target,
                    port,
                    src_port,
                    sequence,
                    retry
                );
            } else {
                return Err(prtip_core::Error::Config(
                    "Packet capture not initialized".to_string(),
                ));
            }

            // Reset buffer for reuse
            pool.reset();
            Ok::<_, prtip_core::Error>(())
        })?;

        Ok(sequence)
    }

    /// Send a RST packet to close the connection
    ///
    /// Sprint 4.17 Phase 3: Integrated zero-copy packet building.
    async fn send_rst(
        &self,
        target: Ipv4Addr,
        port: u16,
        src_port: u16,
        sequence: u32,
    ) -> Result<()> {
        // Build and send RST packet using zero-copy API
        with_buffer(|pool| {
            let packet = TcpPacketBuilder::new()
                .source_ip(self.local_ip)
                .dest_ip(target)
                .source_port(src_port)
                .dest_port(port)
                .sequence(sequence)
                .flags(TcpFlags::RST)
                .window(0)
                .build_ip_packet_with_buffer(pool)?;

            if let Some(ref mut capture) = *self.capture.lock() {
                capture.send_packet(packet).map_err(|e| {
                    prtip_core::Error::Network(format!("Failed to send RST: {}", e))
                })?;

                trace!("Sent RST to {}:{} (src_port={})", target, port, src_port);
            }

            // Reset buffer for reuse
            pool.reset();
            Ok::<_, prtip_core::Error>(())
        })?;

        Ok(())
    }

    /// Wait for response (SYN/ACK, RST, or ICMP) with optional PCAPNG capture
    async fn wait_for_response(
        &self,
        target: Ipv4Addr,
        port: u16,
        src_port: u16,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<PortState> {
        // In a real implementation, this would:
        // 1. Set up a BPF filter to capture only relevant packets
        // 2. Parse incoming packets using pnet
        // 3. Match based on source IP, source port, destination port
        // 4. Determine state based on TCP flags

        // Simplified implementation for now - would need packet parsing
        // This is a placeholder that shows the structure

        loop {
            if let Some(ref mut capture) = *self.capture.lock() {
                if let Some(packet) = capture.receive_packet(100)? {
                    // Capture received packet to PCAPNG if writer is provided
                    if let Some(ref writer) = pcapng_writer {
                        if let Ok(guard) = writer.lock() {
                            if let Err(e) = guard.write_packet(&packet, Direction::Received) {
                                // Log error but don't fail scan (PCAPNG is optional)
                                warn!("PCAPNG write error (SYN response): {}", e);
                            }
                        }
                    }

                    // Parse packet and check if it matches our connection
                    if let Some(state) = self.parse_response(&packet, target, port, src_port)? {
                        return Ok(state);
                    }
                }
            }

            // Yield to allow other tasks to run
            tokio::task::yield_now().await;
        }
    }

    /// Parse a received packet and determine port state
    fn parse_response(
        &self,
        packet: &[u8],
        target: Ipv4Addr,
        port: u16,
        src_port: u16,
    ) -> Result<Option<PortState>> {
        use pnet::packet::{ethernet::EthernetPacket, ipv4::Ipv4Packet, tcp::TcpPacket, Packet};

        // Parse Ethernet frame
        let eth_packet = match EthernetPacket::new(packet) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Parse IPv4 packet
        let ipv4_packet = match Ipv4Packet::new(eth_packet.payload()) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Check if it's from our target
        if ipv4_packet.get_source() != target {
            return Ok(None);
        }

        // Parse TCP packet
        let tcp_packet = match TcpPacket::new(ipv4_packet.payload()) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Check if it matches our connection
        if tcp_packet.get_source() != port || tcp_packet.get_destination() != src_port {
            return Ok(None);
        }

        // Validate sequence number against stored connection state
        if let Some(conn) = self.connections.get(&(target, port, src_port)) {
            let ack_num = tcp_packet.get_acknowledgement();
            // For SYN/ACK, the ACK should be our sequence + 1
            if ack_num != conn.sequence.wrapping_add(1) {
                trace!(
                    "Sequence mismatch: expected {}, got {}",
                    conn.sequence.wrapping_add(1),
                    ack_num
                );
                return Ok(None);
            }
        }

        // Determine state based on flags
        let flags = tcp_packet.get_flags();

        // SYN/ACK = open
        if (flags & 0x12) == 0x12 {
            debug!("Received SYN/ACK from {}:{} - OPEN", target, port);
            return Ok(Some(PortState::Open));
        }

        // RST = closed
        if (flags & 0x04) == 0x04 {
            debug!("Received RST from {}:{} - CLOSED", target, port);
            return Ok(Some(PortState::Closed));
        }

        // Unknown response
        Ok(None)
    }

    /// Scan multiple ports in parallel
    pub async fn scan_ports(&self, target: Ipv4Addr, ports: Vec<u16>) -> Result<Vec<ScanResult>> {
        let (tx, mut rx) = mpsc::channel(1000);
        let mut tasks = Vec::new();

        // Spawn scan tasks for each port
        for port in ports {
            let tx = tx.clone();
            let scanner = self.clone_for_task();

            let task = tokio::spawn(async move {
                match scanner.scan_port(target, port).await {
                    Ok(result) => {
                        let _ = tx.send(result).await;
                    }
                    Err(e) => {
                        warn!("Error scanning {}:{}: {}", target, port, e);
                    }
                }
            });

            tasks.push(task);
        }

        // Drop the sender so receiver knows when all tasks are done
        drop(tx);

        // Collect results
        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result);
        }

        // Wait for all tasks to complete
        for task in tasks {
            let _ = task.await;
        }

        Ok(results)
    }

    /// Clone scanner for task spawning (shares connection table and capture)
    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            capture: Arc::clone(&self.capture),
            connections: Arc::clone(&self.connections),
            local_ip: self.local_ip,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_creation() {
        let state = ConnectionState {
            target_ip: Ipv4Addr::new(192, 168, 1, 1),
            target_port: 80,
            source_port: 12345,
            sequence: 0x12345678,
            sent_time: Instant::now(),
            retries: 0,
        };

        assert_eq!(state.target_port, 80);
        assert_eq!(state.source_port, 12345);
        assert_eq!(state.retries, 0);
    }

    #[test]
    fn test_scanner_creation() {
        let config = Config::default();
        let scanner = SynScanner::new(config);
        assert!(scanner.is_ok());
    }
}
