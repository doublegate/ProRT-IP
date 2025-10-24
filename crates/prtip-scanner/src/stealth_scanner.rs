//! Stealth TCP scan techniques
//!
//! This module implements various stealth scanning techniques that exploit
//! different interpretations of the TCP RFC to determine port states.
//!
//! ## Scan Types
//!
//! ### FIN Scan (-sF)
//! Sends packets with only the FIN flag set. Per RFC 793:
//! - **Open ports**: No response
//! - **Closed ports**: RST response
//! - **Filtered**: ICMP unreachable
//!
//! ### NULL Scan (-sN)
//! Sends packets with no flags set:
//! - **Open ports**: No response
//! - **Closed ports**: RST response
//! - **Filtered**: ICMP unreachable
//!
//! ### Xmas Scan (-sX)
//! Sends packets with FIN, PSH, and URG flags ("lit up like a Christmas tree"):
//! - **Open ports**: No response
//! - **Closed ports**: RST response
//! - **Filtered**: ICMP unreachable
//!
//! ### ACK Scan (-sA)
//! Used for firewall rule detection, not port state:
//! - **Unfiltered**: RST response
//! - **Filtered**: No response or ICMP unreachable
//!
//! ## Limitations
//!
//! - **Windows**: Doesn't work (sends RST regardless of port state)
//! - **Cisco**: Many devices send RST for all ports
//! - **Stateful Firewalls**: May detect these unusual flag combinations
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::{StealthScanner, StealthScanType};
//! use prtip_core::Config;
//! use std::net::Ipv4Addr;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! let config = Config::default();
//! let scanner = StealthScanner::new(config)?;
//!
//! let target = Ipv4Addr::new(192, 168, 1, 1);
//! let result = scanner.scan_port(target, 80, StealthScanType::Fin).await?;
//!
//! println!("FIN scan result: {:?}", result.state);
//! # Ok(())
//! # }
//! ```

use parking_lot::Mutex;
use prtip_core::{Config, PortState, Result, ScanResult};
use prtip_network::{create_capture, with_buffer, PacketCapture, TcpFlags, TcpPacketBuilder};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, trace, warn};

// PCAPNG packet capture support
use crate::pcapng::{Direction, PcapngWriter};
use std::sync::Mutex as StdMutex;

/// Type of stealth scan to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StealthScanType {
    /// FIN scan - only FIN flag set
    Fin,
    /// NULL scan - no flags set
    Null,
    /// Xmas scan - FIN, PSH, URG flags set
    Xmas,
    /// ACK scan - only ACK flag set (for firewall detection)
    Ack,
}

impl StealthScanType {
    /// Get the TCP flags for this scan type
    pub fn flags(&self) -> TcpFlags {
        match self {
            StealthScanType::Fin => TcpFlags::FIN,
            StealthScanType::Null => TcpFlags::empty(),
            StealthScanType::Xmas => TcpFlags::FIN.combine(TcpFlags::PSH).combine(TcpFlags::URG),
            StealthScanType::Ack => TcpFlags::ACK,
        }
    }

    /// Get the name of this scan type
    pub fn name(&self) -> &'static str {
        match self {
            StealthScanType::Fin => "FIN",
            StealthScanType::Null => "NULL",
            StealthScanType::Xmas => "Xmas",
            StealthScanType::Ack => "ACK",
        }
    }
}

/// Stealth scanner
pub struct StealthScanner {
    config: Config,
    capture: Arc<Mutex<Option<Box<dyn PacketCapture>>>>,
    local_ip: Ipv4Addr,
}

impl StealthScanner {
    /// Create a new stealth scanner
    pub fn new(config: Config) -> Result<Self> {
        let local_ip = Self::detect_local_ip()?;

        Ok(Self {
            config,
            capture: Arc::new(Mutex::new(None)),
            local_ip,
        })
    }

    /// Initialize packet capture
    pub async fn initialize(&mut self) -> Result<()> {
        let mut capture = create_capture()?;
        capture.open(None)?;
        *self.capture.lock() = Some(capture);
        Ok(())
    }

    /// Detect local IP address
    fn detect_local_ip() -> Result<Ipv4Addr> {
        Ok(Ipv4Addr::new(192, 168, 1, 100))
    }

    /// Scan a single port with specified stealth technique
    pub async fn scan_port(
        &self,
        target: Ipv4Addr,
        port: u16,
        scan_type: StealthScanType,
    ) -> Result<ScanResult> {
        self.scan_port_with_pcapng(target, port, scan_type, None)
            .await
    }

    /// Scan a single port with specified stealth technique and optional PCAPNG capture
    pub async fn scan_port_with_pcapng(
        &self,
        target: Ipv4Addr,
        port: u16,
        scan_type: StealthScanType,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<ScanResult> {
        let start_time = Instant::now();

        // Generate random source port
        use rand::Rng;
        let src_port: u16 = rand::thread_rng().gen_range(1024..65535);

        // Send probe
        self.send_probe(target, port, src_port, scan_type, pcapng_writer.clone())
            .await?;

        // Wait for response
        let timeout_ms = self.config.scan.timeout_ms;
        let wait_duration = Duration::from_millis(timeout_ms);

        match timeout(
            wait_duration,
            self.wait_for_response(target, port, src_port, scan_type, pcapng_writer),
        )
        .await
        {
            Ok(Ok(state)) => {
                let response_time = start_time.elapsed();
                Ok(ScanResult::new(IpAddr::V4(target), port, state)
                    .with_response_time(response_time))
            }
            Ok(Err(e)) => {
                warn!("Error waiting for response: {}", e);
                let response_time = start_time.elapsed();
                Ok(
                    ScanResult::new(IpAddr::V4(target), port, PortState::Unknown)
                        .with_response_time(response_time),
                )
            }
            Err(_) => {
                // Timeout - interpretation depends on scan type
                let state = match scan_type {
                    StealthScanType::Ack => {
                        // For ACK scan, no response = filtered
                        debug!("No response from {}:{} (ACK scan) - FILTERED", target, port);
                        PortState::Filtered
                    }
                    _ => {
                        // For FIN/NULL/Xmas, no response = open|filtered
                        debug!(
                            "No response from {}:{} ({} scan) - OPEN|FILTERED",
                            target,
                            port,
                            scan_type.name()
                        );
                        PortState::Filtered // Could also be open
                    }
                };

                let response_time = start_time.elapsed();
                Ok(ScanResult::new(IpAddr::V4(target), port, state)
                    .with_response_time(response_time))
            }
        }
    }

    /// Send a stealth probe packet (zero-copy with optional PCAPNG capture)
    async fn send_probe(
        &self,
        target: Ipv4Addr,
        port: u16,
        src_port: u16,
        scan_type: StealthScanType,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<()> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let sequence: u32 = rng.gen();

        if let Some(ref mut capture) = *self.capture.lock() {
            // Use thread-local zero-copy buffer
            with_buffer(|buffer_pool| {
                // Build packet in buffer
                let packet_slice = TcpPacketBuilder::new()
                    .source_ip(self.local_ip)
                    .dest_ip(target)
                    .source_port(src_port)
                    .dest_port(port)
                    .sequence(sequence)
                    .flags(scan_type.flags())
                    .window(1024)
                    .build_ip_packet_with_buffer(buffer_pool)?;

                // Capture packet to PCAPNG if writer is provided
                if let Some(ref writer) = pcapng_writer {
                    // Clone packet data before sending (PCAPNG needs owned copy)
                    let packet_data = packet_slice.to_vec();
                    if let Ok(guard) = writer.lock() {
                        if let Err(e) = guard.write_packet(&packet_data, Direction::Sent) {
                            // Log error but don't fail scan (PCAPNG is optional)
                            warn!("PCAPNG write error ({} probe): {}", scan_type.name(), e);
                        }
                    }
                }

                // Send immediately
                capture.send_packet(packet_slice).map_err(|e| {
                    prtip_core::Error::Network(format!(
                        "Failed to send {} probe: {}",
                        scan_type.name(),
                        e
                    ))
                })?;

                trace!(
                    "Sent {} probe to {}:{} (src_port={}) [zero-copy]",
                    scan_type.name(),
                    target,
                    port,
                    src_port
                );

                // Reset buffer
                buffer_pool.reset();

                Ok::<(), prtip_core::Error>(())
            })?;
        } else {
            return Err(prtip_core::Error::Config(
                "Packet capture not initialized".to_string(),
            ));
        }

        Ok(())
    }

    /// Wait for response with optional PCAPNG capture
    async fn wait_for_response(
        &self,
        target: Ipv4Addr,
        port: u16,
        src_port: u16,
        scan_type: StealthScanType,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<PortState> {
        loop {
            if let Some(ref mut capture) = *self.capture.lock() {
                if let Some(packet) = capture.receive_packet(100)? {
                    // Capture received packet to PCAPNG if writer is provided
                    if let Some(ref writer) = pcapng_writer {
                        if let Ok(guard) = writer.lock() {
                            if let Err(e) = guard.write_packet(&packet, Direction::Received) {
                                // Log error but don't fail scan (PCAPNG is optional)
                                warn!("PCAPNG write error ({} response): {}", scan_type.name(), e);
                            }
                        }
                    }

                    if let Some(state) =
                        self.parse_response(&packet, target, port, src_port, scan_type)?
                    {
                        return Ok(state);
                    }
                }
            }

            tokio::task::yield_now().await;
        }
    }

    /// Parse received packet
    fn parse_response(
        &self,
        packet: &[u8],
        target: Ipv4Addr,
        port: u16,
        src_port: u16,
        scan_type: StealthScanType,
    ) -> Result<Option<PortState>> {
        use pnet::packet::{
            ethernet::EthernetPacket, icmp::IcmpPacket, ipv4::Ipv4Packet, tcp::TcpPacket, Packet,
        };

        let eth_packet = match EthernetPacket::new(packet) {
            Some(p) => p,
            None => return Ok(None),
        };

        let ipv4_packet = match Ipv4Packet::new(eth_packet.payload()) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Check if it's from our target
        if ipv4_packet.get_source() != target {
            return Ok(None);
        }

        // Check protocol
        match ipv4_packet.get_next_level_protocol().0 {
            6 => {
                // TCP response
                if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                    if tcp_packet.get_source() != port || tcp_packet.get_destination() != src_port {
                        return Ok(None);
                    }

                    let flags = tcp_packet.get_flags();

                    // RST response
                    if (flags & 0x04) == 0x04 {
                        match scan_type {
                            StealthScanType::Ack => {
                                // For ACK scan, RST = unfiltered
                                debug!(
                                    "Received RST from {}:{} (ACK scan) - UNFILTERED",
                                    target, port
                                );
                                return Ok(Some(PortState::Open)); // Using "Open" to mean "Unfiltered"
                            }
                            _ => {
                                // For FIN/NULL/Xmas, RST = closed
                                debug!(
                                    "Received RST from {}:{} ({} scan) - CLOSED",
                                    target,
                                    port,
                                    scan_type.name()
                                );
                                return Ok(Some(PortState::Closed));
                            }
                        }
                    }
                }
            }
            1 => {
                // ICMP response
                if let Some(icmp_packet) = IcmpPacket::new(ipv4_packet.payload()) {
                    let icmp_type = icmp_packet.get_icmp_type().0;
                    let _icmp_code = icmp_packet.get_icmp_code().0;

                    // ICMP type 3 = Destination Unreachable
                    if icmp_type == 3 {
                        debug!(
                            "Received ICMP unreachable from {}:{} ({} scan) - FILTERED",
                            target,
                            port,
                            scan_type.name()
                        );
                        return Ok(Some(PortState::Filtered));
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_type_flags() {
        assert_eq!(StealthScanType::Fin.flags().0, TcpFlags::FIN.0);
        assert_eq!(StealthScanType::Null.flags().0, 0);
        assert_eq!(StealthScanType::Ack.flags().0, TcpFlags::ACK.0);

        // Xmas should have FIN + PSH + URG
        let xmas_flags = StealthScanType::Xmas.flags();
        assert!(xmas_flags.has(TcpFlags::FIN));
        assert!(xmas_flags.has(TcpFlags::PSH));
        assert!(xmas_flags.has(TcpFlags::URG));
    }

    #[test]
    fn test_scan_type_names() {
        assert_eq!(StealthScanType::Fin.name(), "FIN");
        assert_eq!(StealthScanType::Null.name(), "NULL");
        assert_eq!(StealthScanType::Xmas.name(), "Xmas");
        assert_eq!(StealthScanType::Ack.name(), "ACK");
    }

    #[test]
    fn test_stealth_scanner_creation() {
        let config = Config::default();
        let scanner = StealthScanner::new(config);
        assert!(scanner.is_ok());
    }
}
