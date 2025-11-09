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
//! use std::net::{IpAddr, Ipv4Addr};
//!
//! # async fn example() -> prtip_core::Result<()> {
//! let config = Config::default();
//! let scanner = SynScanner::new(config)?;
//!
//! let target = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
//! let result = scanner.scan_port(target, 80).await?;
//!
//! println!("Port 80 state: {:?}", result.state);
//! # Ok(())
//! # }
//! ```

use crate::{AdaptiveRateLimiterV2, HostgroupLimiter};
use dashmap::DashMap;
use parking_lot::Mutex;
use prtip_core::{Config, EventBus, PortState, Protocol, Result, ScanEvent, ScanResult, ScanStage, ScanType};
use prtip_network::{
    create_capture, packet_buffer::with_buffer, PacketCapture, TcpFlags, TcpOption,
    TcpPacketBuilder,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::{debug, trace, warn};
use uuid::Uuid;

// PCAPNG packet capture support
use crate::pcapng::{Direction, PcapngWriter};
use std::sync::Mutex as StdMutex;

/// Connection state for tracking SYN scan responses
#[derive(Debug, Clone)]
struct ConnectionState {
    /// Target IP address (IPv4 or IPv6)
    target_ip: IpAddr,
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
/// Sprint 5.1: Updated to IpAddr for dual-stack IPv4/IPv6 support
type ConnectionTable = Arc<DashMap<(IpAddr, u16, u16), ConnectionState>>;

/// SYN scanner with raw packet support
/// Sprint 5.1: Enhanced with dual-stack IPv4/IPv6 support
///
/// # Rate Limiting (Sprint 5.4 Phase 1)
///
/// Supports optional hostgroup and adaptive rate limiting:
/// - Hostgroup limiter controls concurrent targets
/// - Adaptive limiter provides per-target ICMP backoff
/// - AdaptiveV3 provides <5% overhead two-tier architecture (experimental)
pub struct SynScanner {
    config: Config,
    capture: Arc<Mutex<Option<Box<dyn PacketCapture>>>>,
    connections: ConnectionTable,
    /// Local IPv4 address for IPv4 scans
    local_ipv4: Ipv4Addr,
    /// Local IPv6 address for IPv6 scans (if available)
    local_ipv6: Option<Ipv6Addr>,
    /// Optional hostgroup limiter (controls concurrent targets)
    hostgroup_limiter: Option<Arc<HostgroupLimiter>>,
    /// Optional adaptive rate limiter (ICMP-aware throttling)
    adaptive_limiter: Option<Arc<AdaptiveRateLimiterV2>>,
    /// Optional AdaptiveV3 rate limiter (<5% overhead target, experimental)
    adaptive_v3: Option<Arc<AdaptiveRateLimiterV2>>,
    /// Optional event bus for real-time progress updates
    event_bus: Option<Arc<EventBus>>,
}

impl SynScanner {
    /// Create a new SYN scanner
    /// Sprint 5.1: Enhanced to detect both IPv4 and IPv6 local addresses
    pub fn new(config: Config) -> Result<Self> {
        // Get local IP addresses (simplified - in production would detect interface)
        let local_ipv4 = Self::detect_local_ipv4()?;
        let local_ipv6 = Self::detect_local_ipv6();

        Ok(Self {
            config,
            capture: Arc::new(Mutex::new(None)),
            connections: Arc::new(DashMap::new()),
            local_ipv4,
            local_ipv6,
            hostgroup_limiter: None,
            adaptive_limiter: None,
            adaptive_v3: None,
            event_bus: None,
        })
    }

    /// Enable hostgroup limiting (concurrent target control)
    pub fn with_hostgroup_limiter(mut self, limiter: Arc<HostgroupLimiter>) -> Self {
        self.hostgroup_limiter = Some(limiter);
        self
    }

    /// Enable adaptive rate limiting (ICMP-aware throttling)
    pub fn with_adaptive_limiter(mut self, limiter: Arc<AdaptiveRateLimiterV2>) -> Self {
        self.adaptive_limiter = Some(limiter);
        self
    }

    /// Enable AdaptiveV3 rate limiting (<5% overhead, experimental)
    pub fn with_adaptive_v3(mut self, limiter: Arc<AdaptiveRateLimiterV2>) -> Self {
        self.adaptive_v3 = Some(limiter);
        self
    }

    /// Attach an event bus for real-time scan events
    ///
    /// # Arguments
    ///
    /// * `bus` - Event bus to emit scan events to
    pub fn with_event_bus(mut self, bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(bus);
        self
    }

    /// Initialize packet capture
    pub async fn initialize(&mut self) -> Result<()> {
        let mut capture = create_capture()?;
        capture.open(None)?; // Auto-detect interface
        *self.capture.lock() = Some(capture);
        Ok(())
    }

    /// Detect local IPv4 address for the interface
    fn detect_local_ipv4() -> Result<Ipv4Addr> {
        // Simplified detection - in production would use interface detection
        // For now, use a placeholder
        Ok(Ipv4Addr::new(192, 168, 1, 100))
    }

    /// Detect local IPv6 address for the interface
    /// Returns None if no IPv6 address is available
    fn detect_local_ipv6() -> Option<Ipv6Addr> {
        // Simplified detection - in production would use interface detection
        // Use link-local placeholder (fe80::1)
        Some(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1))
    }

    /// Get appropriate local IP address for target
    /// Returns IPv4 address for IPv4 targets, IPv6 address for IPv6 targets
    fn get_local_ip_for_target(&self, target: IpAddr) -> Result<IpAddr> {
        match target {
            IpAddr::V4(_) => Ok(IpAddr::V4(self.local_ipv4)),
            IpAddr::V6(_) => self.local_ipv6.map(IpAddr::V6).ok_or_else(|| {
                prtip_core::Error::Config("No IPv6 address available for IPv6 scan".to_string())
            }),
        }
    }

    /// Scan a single port
    /// Sprint 5.1: Updated to accept IpAddr for dual-stack support
    pub async fn scan_port(&self, target: IpAddr, port: u16) -> Result<ScanResult> {
        self.scan_port_with_pcapng(target, port, None).await
    }

    /// Scan a single port with optional PCAPNG packet capture
    /// Sprint 5.1: Updated to accept IpAddr for dual-stack IPv4/IPv6 support
    pub async fn scan_port_with_pcapng(
        &self,
        target: IpAddr,
        port: u16,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<ScanResult> {
        let start_time = Instant::now();

        // Use configured source port or generate random
        use rand::Rng;
        let src_port: u16 = self
            .config
            .network
            .source_port
            .unwrap_or_else(|| rand::thread_rng().gen_range(1024..65535));

        // Send initial SYN
        let sequence = self
            .send_syn(target, port, src_port, 0, pcapng_writer.clone())
            .await?;

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

                    return Ok(
                        ScanResult::new(target, port, state).with_response_time(response_time)
                    );
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
                        self.send_syn(
                            target,
                            port,
                            src_port,
                            (retry + 1) as u8,
                            pcapng_writer.clone(),
                        )
                        .await?;
                    }
                }
            }
        }

        // No response after all retries - port is filtered
        self.connections.remove(&(target, port, src_port));
        let response_time = start_time.elapsed();

        Ok(ScanResult::new(target, port, PortState::Filtered).with_response_time(response_time))
    }

    /// Send a SYN packet
    ///
    /// Sprint 4.17 Phase 3: Integrated zero-copy packet building.
    /// Uses `with_buffer()` closure and `build_ip_packet_with_buffer()` to
    /// eliminate heap allocations in packet crafting hot path.
    ///
    /// Sprint 4.18: Added optional PCAPNG packet capture.
    /// Sprint 5.1: Enhanced with dual-stack IPv4/IPv6 support
    async fn send_syn(
        &self,
        target: IpAddr,
        port: u16,
        src_port: u16,
        retry: u8,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<u32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Generate sequence number (for stateless, could use SipHash)
        let sequence: u32 = rng.gen();

        // Get appropriate local IP for target
        let local_ip = self.get_local_ip_for_target(target)?;

        // Build and send SYN packet (dispatch based on IP version)
        match (local_ip, target) {
            (IpAddr::V4(src_ipv4), IpAddr::V4(dst_ipv4)) => {
                // IPv4 SYN packet
                with_buffer(|pool| {
                    let mut builder = TcpPacketBuilder::new()
                        .source_ip(src_ipv4)
                        .dest_ip(dst_ipv4)
                        .source_port(src_port)
                        .dest_port(port)
                        .sequence(sequence)
                        .flags(TcpFlags::SYN)
                        .window(65535)
                        .add_option(TcpOption::Mss(1460))
                        .add_option(TcpOption::WindowScale(7))
                        .add_option(TcpOption::SackPermitted);

                    // Apply TTL if configured (Sprint 4.20: Evasion features)
                    if let Some(ttl) = self.config.evasion.ttl {
                        builder = builder.ttl(ttl);
                    }

                    // Apply bad checksum if configured (Sprint 4.20 Phase 6: Bad checksum)
                    if self.config.evasion.bad_checksums {
                        builder = builder.bad_checksum(true);
                    }

                    let packet = builder.build_ip_packet_with_buffer(pool)?;

                    // Sprint 4.20: Check if packet fragmentation is enabled
                    let packets_to_send: Vec<Vec<u8>> = if self.config.evasion.fragment_packets {
                        // Fragment the packet using configured MTU
                        use prtip_network::fragment_tcp_packet;
                        let mtu = self.config.evasion.mtu.unwrap_or(1500);
                        let packet_data = packet.to_vec(); // Copy from pool for fragmentation
                        fragment_tcp_packet(&packet_data, mtu).map_err(|e| {
                            prtip_core::Error::Network(format!("Fragmentation failed: {}", e))
                        })?
                    } else {
                        // No fragmentation - send as single packet
                        vec![packet.to_vec()]
                    };

                    // Capture packets to PCAPNG if writer is provided
                    if let Some(ref writer) = pcapng_writer {
                        for packet_data in &packets_to_send {
                            if let Ok(guard) = writer.lock() {
                                if let Err(e) = guard.write_packet(packet_data, Direction::Sent) {
                                    // Log error but don't fail scan (PCAPNG is optional)
                                    warn!("PCAPNG write error (SYN packet): {}", e);
                                }
                            }
                        }
                    }

                    // Send packet(s) (fragmented or whole)
                    if let Some(ref mut capture) = *self.capture.lock() {
                        for fragment in &packets_to_send {
                            capture.send_packet(fragment).map_err(|e| {
                                prtip_core::Error::Network(format!("Failed to send SYN: {}", e))
                            })?;
                        }

                        if self.config.evasion.fragment_packets {
                            trace!(
                                "Sent {} fragmented SYN packets to {}:{} (src_port={}, seq={}, retry={})",
                                packets_to_send.len(),
                                target,
                                port,
                                src_port,
                                sequence,
                                retry
                            );
                        } else {
                            trace!(
                                "Sent SYN to {}:{} (src_port={}, seq={}, retry={})",
                                target,
                                port,
                                src_port,
                                sequence,
                                retry
                            );
                        }
                    } else {
                        return Err(prtip_core::Error::Config(
                            "Packet capture not initialized".to_string(),
                        ));
                    }

                    // Reset buffer for reuse
                    pool.reset();
                    Ok::<_, prtip_core::Error>(())
                })?;
            }
            (IpAddr::V6(src_ipv6), IpAddr::V6(dst_ipv6)) => {
                // IPv6 SYN packet - Sprint 5.1
                let mut builder = TcpPacketBuilder::new()
                    .source_port(src_port)
                    .dest_port(port)
                    .sequence(sequence)
                    .flags(TcpFlags::SYN)
                    .window(65535)
                    .add_option(TcpOption::Mss(1460))
                    .add_option(TcpOption::WindowScale(7))
                    .add_option(TcpOption::SackPermitted);

                // Apply hop limit if configured (IPv6 equivalent of TTL)
                if let Some(ttl) = self.config.evasion.ttl {
                    builder = builder.ttl(ttl);
                }

                // Apply bad checksum if configured
                if self.config.evasion.bad_checksums {
                    builder = builder.bad_checksum(true);
                }

                // Build IPv6+TCP packet (no fragmentation support for IPv6 yet - Sprint 5.1 Phase 1)
                let packet = builder.build_ipv6_packet(src_ipv6, dst_ipv6)?;

                // Capture packet to PCAPNG if writer is provided
                if let Some(ref writer) = pcapng_writer {
                    if let Ok(guard) = writer.lock() {
                        if let Err(e) = guard.write_packet(&packet, Direction::Sent) {
                            warn!("PCAPNG write error (IPv6 SYN packet): {}", e);
                        }
                    }
                }

                // Send packet
                if let Some(ref mut capture) = *self.capture.lock() {
                    capture.send_packet(&packet).map_err(|e| {
                        prtip_core::Error::Network(format!("Failed to send IPv6 SYN: {}", e))
                    })?;

                    trace!(
                        "Sent IPv6 SYN to {}:{} (src_port={}, seq={}, retry={})",
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
            }
            _ => {
                return Err(prtip_core::Error::Config(format!(
                    "IP version mismatch: local {} vs target {}",
                    local_ip, target
                )));
            }
        }

        Ok(sequence)
    }

    /// Send a RST packet to close the connection
    ///
    /// Sprint 4.17 Phase 3: Integrated zero-copy packet building.
    /// Sprint 5.1: Updated for dual-stack IPv4/IPv6 support
    async fn send_rst(
        &self,
        target: IpAddr,
        port: u16,
        src_port: u16,
        sequence: u32,
    ) -> Result<()> {
        // Get appropriate local IP for target
        let local_ip = self.get_local_ip_for_target(target)?;

        match (local_ip, target) {
            (IpAddr::V4(src_ipv4), IpAddr::V4(dst_ipv4)) => {
                // Build and send IPv4 RST packet using zero-copy API
                with_buffer(|pool| {
                    let mut builder = TcpPacketBuilder::new()
                        .source_ip(src_ipv4)
                        .dest_ip(dst_ipv4)
                        .source_port(src_port)
                        .dest_port(port)
                        .sequence(sequence)
                        .flags(TcpFlags::RST)
                        .window(0);

                    // Apply TTL if configured
                    if let Some(ttl) = self.config.evasion.ttl {
                        builder = builder.ttl(ttl);
                    }

                    // Apply bad checksum if configured
                    if self.config.evasion.bad_checksums {
                        builder = builder.bad_checksum(true);
                    }

                    let packet = builder.build_ip_packet_with_buffer(pool)?;

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
            }
            (IpAddr::V6(src_ipv6), IpAddr::V6(dst_ipv6)) => {
                // Build and send IPv6 RST packet
                let mut builder = TcpPacketBuilder::new()
                    .source_port(src_port)
                    .dest_port(port)
                    .sequence(sequence)
                    .flags(TcpFlags::RST)
                    .window(0);

                // Apply hop limit if configured
                if let Some(ttl) = self.config.evasion.ttl {
                    builder = builder.ttl(ttl);
                }

                // Apply bad checksum if configured
                if self.config.evasion.bad_checksums {
                    builder = builder.bad_checksum(true);
                }

                let packet = builder.build_ipv6_packet(src_ipv6, dst_ipv6)?;

                if let Some(ref mut capture) = *self.capture.lock() {
                    capture.send_packet(&packet).map_err(|e| {
                        prtip_core::Error::Network(format!("Failed to send IPv6 RST: {}", e))
                    })?;

                    trace!(
                        "Sent IPv6 RST to {}:{} (src_port={})",
                        target,
                        port,
                        src_port
                    );
                }
            }
            _ => {
                return Err(prtip_core::Error::Config(format!(
                    "IP version mismatch: local {} vs target {}",
                    local_ip, target
                )));
            }
        }

        Ok(())
    }

    /// Wait for response (SYN/ACK, RST, or ICMP) with optional PCAPNG capture
    /// Sprint 5.1: Updated for dual-stack IPv4/IPv6 support
    async fn wait_for_response(
        &self,
        target: IpAddr,
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
    /// Sprint 5.1: Updated for dual-stack IPv4/IPv6 support
    fn parse_response(
        &self,
        packet: &[u8],
        target: IpAddr,
        port: u16,
        src_port: u16,
    ) -> Result<Option<PortState>> {
        use pnet::packet::{
            ethernet::EthernetPacket, ipv4::Ipv4Packet, ipv6::Ipv6Packet, tcp::TcpPacket, Packet,
        };

        // Parse Ethernet frame
        let eth_packet = match EthernetPacket::new(packet) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Match on target IP version and parse accordingly
        match target {
            IpAddr::V4(target_ipv4) => {
                // Parse IPv4 packet
                let ipv4_packet = match Ipv4Packet::new(eth_packet.payload()) {
                    Some(p) => p,
                    None => return Ok(None),
                };

                // Check if it's from our target
                if ipv4_packet.get_source() != target_ipv4 {
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
            IpAddr::V6(target_ipv6) => {
                // Parse IPv6 packet
                let ipv6_packet = match Ipv6Packet::new(eth_packet.payload()) {
                    Some(p) => p,
                    None => return Ok(None),
                };

                // Check if it's from our target
                if ipv6_packet.get_source() != target_ipv6 {
                    return Ok(None);
                }

                // Sprint 5.1 Phase 1: Skip extension headers to find TCP
                // Note: This is a simplified implementation - production should handle all extension header types
                let tcp_payload = ipv6_packet.payload();
                let next_header = ipv6_packet.get_next_header();

                // Check if next header is TCP (protocol 6)
                if next_header.0 != 6 {
                    // TODO Sprint 5.1 Phase 1.5: Handle extension headers (Fragment, Hop-by-Hop, Routing, Destination Options)
                    // For now, only support direct TCP (no extension headers)
                    trace!("IPv6 packet with non-TCP next header: {}", next_header.0);
                    return Ok(None);
                }

                // Parse TCP packet
                let tcp_packet = match TcpPacket::new(tcp_payload) {
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
                    debug!("Received IPv6 SYN/ACK from {}:{} - OPEN", target, port);
                    return Ok(Some(PortState::Open));
                }

                // RST = closed
                if (flags & 0x04) == 0x04 {
                    debug!("Received IPv6 RST from {}:{} - CLOSED", target, port);
                    return Ok(Some(PortState::Closed));
                }

                // Unknown response
                Ok(None)
            }
        }
    }

    /// Scan multiple ports in parallel
    /// Sprint 5.1: Updated for dual-stack IPv4/IPv6 support
    pub async fn scan_ports(&self, target: IpAddr, ports: Vec<u16>) -> Result<Vec<ScanResult>> {
        // Generate scan ID for event tracking
        let scan_id = Uuid::new_v4();
        let scan_start = Instant::now();

        // Emit ScanStarted event
        if let Some(bus) = &self.event_bus {
            bus.publish(ScanEvent::ScanStarted {
                scan_id,
                scan_type: ScanType::Syn,
                target_count: 1,
                port_count: ports.len(),
                timestamp: SystemTime::now(),
            })
            .await;

            // Emit stage change to ScanningPorts
            bus.publish(ScanEvent::StageChanged {
                scan_id,
                from_stage: ScanStage::ResolvingTargets,
                to_stage: ScanStage::ScanningPorts,
                timestamp: SystemTime::now(),
            })
            .await;
        }

        // 1. Acquire hostgroup permit (if rate limiting enabled)
        let _permit = if let Some(limiter) = &self.hostgroup_limiter {
            Some(limiter.acquire_target().await)
        } else {
            None
        };

        // 2. Check ICMP backoff (if adaptive rate limiting enabled)
        if let Some(limiter) = &self.adaptive_limiter {
            if limiter.is_target_backed_off(target) {
                debug!("Skipping {} (ICMP backoff active)", target);

                // Emit scan completion with no results
                if let Some(bus) = &self.event_bus {
                    bus.publish(ScanEvent::ScanCompleted {
                        scan_id,
                        duration: scan_start.elapsed(),
                        total_targets: 1,
                        open_ports: 0,
                        closed_ports: 0,
                        filtered_ports: 0,
                        detected_services: 0,
                        timestamp: SystemTime::now(),
                    })
                    .await;
                }

                return Ok(Vec::new());
            }
        }

        let (tx, mut rx) = mpsc::channel(1000);
        let mut tasks = Vec::new();

        // Clone event bus for tasks
        let event_bus = self.event_bus.clone();

        // Spawn scan tasks for each port
        for port in ports {
            let tx = tx.clone();
            let scanner = self.clone_for_task();
            let bus = event_bus.clone();

            let task = tokio::spawn(async move {
                match scanner.scan_port(target, port).await {
                    Ok(result) => {
                        // Emit PortFound event for open ports
                        if result.state == PortState::Open {
                            if let Some(bus) = &bus {
                                bus.publish(ScanEvent::PortFound {
                                    scan_id,
                                    ip: target,
                                    port,
                                    state: result.state,
                                    protocol: Protocol::Tcp,
                                    scan_type: ScanType::Syn,
                                    timestamp: SystemTime::now(),
                                })
                                .await;
                            }
                        }
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

        // Calculate final statistics
        let open_count = results.iter().filter(|r| r.state == PortState::Open).count();
        let closed_count = results.iter().filter(|r| r.state == PortState::Closed).count();
        let filtered_count = results.iter().filter(|r| r.state == PortState::Filtered).count();

        // Emit ScanCompleted event
        if let Some(bus) = &self.event_bus {
            bus.publish(ScanEvent::ScanCompleted {
                scan_id,
                duration: scan_start.elapsed(),
                total_targets: 1,
                open_ports: open_count,
                closed_ports: closed_count,
                filtered_ports: filtered_count,
                detected_services: 0, // SYN scan doesn't do service detection
                timestamp: SystemTime::now(),
            })
            .await;
        }

        Ok(results)
    }

    /// Clone scanner for task spawning (shares connection table and capture)
    /// Sprint 5.1: Updated for dual-stack IPv4/IPv6 support
    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            capture: Arc::clone(&self.capture),
            connections: Arc::clone(&self.connections),
            local_ipv4: self.local_ipv4,
            local_ipv6: self.local_ipv6,
            hostgroup_limiter: self.hostgroup_limiter.clone(),
            adaptive_limiter: self.adaptive_limiter.clone(),
            adaptive_v3: self.adaptive_v3.clone(),
            event_bus: self.event_bus.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_creation() {
        let state = ConnectionState {
            target_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
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
