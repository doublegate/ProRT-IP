//! UDP scan implementation
//!
//! UDP scanning is more complex than TCP scanning because UDP is connectionless.
//! We must rely on ICMP port unreachable messages to determine if ports are closed.
//!
//! ## State determination
//!
//! - **Open**: Receive UDP response from target
//! - **Closed**: Receive ICMP port unreachable (type 3, code 3)
//! - **Open|Filtered**: No response (could be open or filtered by firewall)
//!
//! ## Protocol-specific probes
//!
//! For well-known services, we send protocol-specific payloads to elicit responses:
//! - DNS (53): DNS query
//! - SNMP (161): GetRequest
//! - NTP (123): Version query
//! - NetBIOS (137): Name query
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::UdpScanner;
//! use prtip_core::Config;
//! use std::net::Ipv4Addr;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! let config = Config::default();
//! let scanner = UdpScanner::new(config)?;
//!
//! let target = Ipv4Addr::new(192, 168, 1, 1);
//! let result = scanner.scan_port(target, 53).await?;
//!
//! println!("UDP port 53 state: {:?}", result.state);
//! # Ok(())
//! # }
//! ```

use parking_lot::Mutex;
use prtip_core::{Config, PortState, Result, ScanResult};
use prtip_network::{
    create_capture, get_udp_payload, with_buffer, PacketCapture, UdpPacketBuilder,
};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, trace, warn};

/// UDP scanner
pub struct UdpScanner {
    config: Config,
    capture: Arc<Mutex<Option<Box<dyn PacketCapture>>>>,
    local_ip: Ipv4Addr,
}

impl UdpScanner {
    /// Create a new UDP scanner
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

    /// Scan a single UDP port
    pub async fn scan_port(&self, target: Ipv4Addr, port: u16) -> Result<ScanResult> {
        let start_time = Instant::now();

        // Generate random source port
        use rand::Rng;
        let src_port: u16 = rand::thread_rng().gen_range(1024..65535);

        // Get protocol-specific payload if available
        let payload = get_udp_payload(port).unwrap_or_default();

        // Send UDP probe
        self.send_udp_probe(target, port, src_port, &payload)
            .await?;

        // Wait for response
        let timeout_ms = self.config.scan.timeout_ms;
        let wait_duration = Duration::from_millis(timeout_ms);

        match timeout(
            wait_duration,
            self.wait_for_response(target, port, src_port),
        )
        .await
        {
            Ok(Ok(state)) => {
                let response_time = start_time.elapsed();
                Ok(ScanResult::new(IpAddr::V4(target), port, state)
                    .with_response_time(response_time))
            }
            Ok(Err(e)) => {
                warn!("Error waiting for UDP response: {}", e);
                let response_time = start_time.elapsed();
                Ok(
                    ScanResult::new(IpAddr::V4(target), port, PortState::Unknown)
                        .with_response_time(response_time),
                )
            }
            Err(_) => {
                // Timeout - port is open|filtered
                debug!("No response from {}:{} - OPEN|FILTERED", target, port);
                let response_time = start_time.elapsed();
                Ok(
                    ScanResult::new(IpAddr::V4(target), port, PortState::Filtered)
                        .with_response_time(response_time),
                )
            }
        }
    }

    /// Send a UDP probe packet (zero-copy)
    async fn send_udp_probe(
        &self,
        target: Ipv4Addr,
        port: u16,
        src_port: u16,
        payload: &[u8],
    ) -> Result<()> {
        if let Some(ref mut capture) = *self.capture.lock() {
            // Use thread-local zero-copy buffer
            // Build and send within closure to satisfy lifetime requirements
            with_buffer(|buffer_pool| {
                // Build packet in buffer
                let packet_slice = UdpPacketBuilder::new()
                    .source_ip(self.local_ip)
                    .dest_ip(target)
                    .source_port(src_port)
                    .dest_port(port)
                    .payload(payload.to_vec())
                    .build_ip_packet_with_buffer(buffer_pool)?;

                // Send immediately (buffer ref valid within closure)
                capture.send_packet(packet_slice).map_err(|e| {
                    prtip_core::Error::Network(format!("Failed to send UDP: {}", e))
                })?;

                trace!(
                    "Sent UDP to {}:{} (src_port={}, payload_len={}) [zero-copy]",
                    target,
                    port,
                    src_port,
                    payload.len()
                );

                // Reset buffer for next packet
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

    /// Wait for UDP or ICMP response
    async fn wait_for_response(
        &self,
        target: Ipv4Addr,
        port: u16,
        src_port: u16,
    ) -> Result<PortState> {
        loop {
            if let Some(ref mut capture) = *self.capture.lock() {
                if let Some(packet) = capture.receive_packet(100)? {
                    if let Some(state) = self.parse_response(&packet, target, port, src_port)? {
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
    ) -> Result<Option<PortState>> {
        use pnet::packet::{
            ethernet::EthernetPacket, icmp::IcmpPacket, ipv4::Ipv4Packet, udp::UdpPacket, Packet,
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
            17 => {
                // UDP response = port open
                if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                    if udp_packet.get_source() == port && udp_packet.get_destination() == src_port {
                        debug!("Received UDP response from {}:{} - OPEN", target, port);
                        return Ok(Some(PortState::Open));
                    }
                }
            }
            1 => {
                // ICMP message
                if let Some(icmp_packet) = IcmpPacket::new(ipv4_packet.payload()) {
                    let icmp_type = icmp_packet.get_icmp_type().0;
                    let icmp_code = icmp_packet.get_icmp_code().0;

                    // ICMP type 3 code 3 = Port Unreachable
                    if icmp_type == 3 && icmp_code == 3 {
                        debug!(
                            "Received ICMP port unreachable from {}:{} - CLOSED",
                            target, port
                        );
                        return Ok(Some(PortState::Closed));
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
    fn test_udp_scanner_creation() {
        let config = Config::default();
        let scanner = UdpScanner::new(config);
        assert!(scanner.is_ok());
    }
}
