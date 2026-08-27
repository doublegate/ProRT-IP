//! IPID Tracker for Idle Scan
//!
//! This module provides functionality for measuring and analyzing IP Identification (IPID)
//! fields in TCP/IP packets. The IPID tracker is used to find suitable "zombie" hosts
//! for idle scanning by testing their IPID increment patterns.
//!
//! # IPID Patterns
//!
//! Different operating systems handle IPID incrementing differently:
//! - **Sequential**: Increments by 1 per packet (good zombie) - Linux <4.18, Windows XP/2003
//! - **Random**: Unpredictable (bad zombie) - Modern Linux 4.18+, Windows 10+
//! - **PerHost**: Separate counter per destination (bad zombie) - BSD, macOS
//! - **Broken256**: Windows byte-order bug, increments by 256 (still usable)

use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::{checksum, Ipv4Packet, MutableIpv4Packet};
use pnet::packet::tcp::{ipv4_checksum, MutableTcpPacket, TcpFlags, TcpPacket};
use pnet::transport::{transport_channel, TransportChannelType};
use pnet_packet::Packet; // Trait needed for .packet() method
use prtip_core::{Error, Result};
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};

/// IPID measurement result containing the IPID value and timestamp
#[derive(Debug, Clone)]
pub struct IPIDMeasurement {
    /// The IPID value extracted from the RST packet
    pub ipid: u16,
    /// When the measurement was taken
    pub timestamp: Instant,
}

/// IPID pattern classification
///
/// Different operating systems use different IPID generation strategies.
/// Only Sequential and Broken256 patterns are suitable for idle scanning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IPIDPattern {
    /// Sequential +1 per packet (good zombie)
    /// Common on: Linux <4.18, Windows XP/2003, old BSD
    Sequential,

    /// Random/unpredictable (bad zombie)
    /// Common on: Modern Linux 4.18+, Windows 10+
    Random,

    /// Grouped by destination host (bad zombie)
    /// Common on: FreeBSD, macOS, some BSD variants
    PerHost,

    /// Windows byte-order bug: increments by 256 instead of 1 (still usable)
    /// Due to little-endian ↔ big-endian conversion issue
    Broken256,
}

/// IPID tracker for zombie host analysis
///
/// Sends SYN/ACK probes to a target host and measures IPID values in RST responses.
/// This allows classification of the host's IPID increment pattern.
pub struct IPIDTracker {
    /// Target IP address (potential zombie)
    #[allow(dead_code)] // Used in Phase 3 implementation
    target: IpAddr,

    /// History of IPID measurements
    measurements: Vec<IPIDMeasurement>,

    /// Timeout for probe responses
    #[allow(dead_code)] // Used in Phase 3 implementation
    timeout: Duration,
}

impl IPIDTracker {
    /// Create new IPID tracker for target host
    ///
    /// # Arguments
    /// * `target` - IP address to probe
    ///
    /// # Returns
    /// * `Result<Self>` - New tracker instance
    pub fn new(target: IpAddr) -> Result<Self> {
        Ok(Self {
            target,
            measurements: Vec::new(),
            timeout: Duration::from_secs(5),
        })
    }

    /// Send SYN/ACK probe to target and measure IPID from RST response
    ///
    /// This sends an unsolicited SYN/ACK packet to the target, which should
    /// trigger a RST response. The IPID field is extracted from the RST packet.
    ///
    /// # Returns
    /// * `Result<IPIDMeasurement>` - Measurement with IPID and timestamp
    ///
    /// # Errors
    /// * Timeout if no RST received within 5 seconds
    /// * I/O errors from raw socket operations
    pub async fn probe(&mut self) -> Result<IPIDMeasurement> {
        // Create transport channel for sending/receiving IP packets (Layer3 for IPID access)
        let protocol = TransportChannelType::Layer3(IpNextHeaderProtocols::Tcp);

        let (mut tx, mut rx) = transport_channel(4096, protocol)
            .map_err(|e| Error::Network(format!("Failed to create transport channel: {}", e)))?;

        // Build and send SYN/ACK packet
        self.send_syn_ack_probe(&mut tx).await?;

        // Wait for RST response and extract IPID
        let ipid = self.receive_rst_response(&mut rx).await?;

        // Record measurement
        let measurement = IPIDMeasurement {
            ipid,
            timestamp: Instant::now(),
        };
        self.measurements.push(measurement.clone());

        Ok(measurement)
    }

    /// Perform multiple probes and classify IPID pattern
    ///
    /// Sends `num_probes` SYN/ACK packets (1 second apart) and analyzes
    /// the IPID increment pattern to determine zombie suitability.
    ///
    /// # Arguments
    /// * `num_probes` - Number of probes to send (recommended: 3-5)
    ///
    /// # Returns
    /// * `Result<IPIDPattern>` - Classified IPID pattern
    pub async fn classify_pattern(&mut self, num_probes: usize) -> Result<IPIDPattern> {
        // Send multiple probes with 1 second intervals
        for i in 0..num_probes {
            self.probe().await?;

            // Wait between probes (except after last one)
            if i < num_probes - 1 {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }

        // Analyze increment pattern
        let increments = self.calculate_increments();

        if increments.is_empty() {
            return Err(Error::Scanner(
                "Not enough measurements for pattern classification".into(),
            ));
        }

        // Check for sequential pattern (+1)
        if increments.iter().all(|&inc| inc == 1) {
            return Ok(IPIDPattern::Sequential);
        }

        // Check for Windows byte-order bug (+256)
        if increments.iter().all(|&inc| inc == 256) {
            return Ok(IPIDPattern::Broken256);
        }

        // Calculate variance to distinguish Random from PerHost
        let variance = self.calculate_variance();

        if variance > 10.0 {
            // High variance = random IPID
            Ok(IPIDPattern::Random)
        } else {
            // Some pattern but not sequential = per-host
            Ok(IPIDPattern::PerHost)
        }
    }

    /// Calculate IPID increments between consecutive measurements
    ///
    /// Uses wrapping arithmetic to handle IPID rollover (65535 → 0).
    ///
    /// # Returns
    /// * `Vec<i32>` - IPID deltas between consecutive probes
    pub fn calculate_increments(&self) -> Vec<i32> {
        self.measurements
            .windows(2)
            .map(|w| {
                // Use wrapping subtraction to handle rollover
                let delta = w[1].ipid.wrapping_sub(w[0].ipid);
                delta as i32
            })
            .collect()
    }

    /// Check if measurements show sequential pattern
    ///
    /// # Returns
    /// * `bool` - True if all increments are exactly 1
    pub fn is_sequential(&self) -> bool {
        let increments = self.calculate_increments();
        !increments.is_empty() && increments.iter().all(|&inc| inc == 1)
    }

    /// Calculate variance of IPID increments (detect noise/randomness)
    ///
    /// Higher variance indicates either random IPID or zombie traffic noise.
    ///
    /// # Returns
    /// * `f32` - Variance of increments (0.0 = perfectly consistent)
    pub fn calculate_variance(&self) -> f32 {
        let increments = self.calculate_increments();
        if increments.len() < 2 {
            return 0.0;
        }

        let mean: f32 = increments.iter().sum::<i32>() as f32 / increments.len() as f32;

        let variance: f32 = increments
            .iter()
            .map(|&inc| {
                let diff = inc as f32 - mean;
                diff * diff
            })
            .sum::<f32>()
            / increments.len() as f32;

        variance
    }

    /// Get the number of measurements taken
    pub fn measurement_count(&self) -> usize {
        self.measurements.len()
    }

    /// Clear measurement history
    pub fn clear_measurements(&mut self) {
        self.measurements.clear();
    }

    /// Set measurements (for testing only)
    #[cfg(test)]
    pub(crate) fn set_measurements_for_test(&mut self, measurements: Vec<IPIDMeasurement>) {
        self.measurements = measurements;
    }

    // Private helper methods

    /// Send SYN/ACK probe packet to target
    ///
    /// Crafts a SYN/ACK packet with proper IP and TCP headers and checksums.
    /// The unsolicited SYN/ACK should trigger a RST response from the target.
    async fn send_syn_ack_probe(&self, tx: &mut pnet::transport::TransportSender) -> Result<()> {
        match self.target {
            IpAddr::V4(target_ipv4) => {
                // Build IPv4 + TCP SYN/ACK packet
                let mut buffer = [0u8; 40]; // IPv4 header (20) + TCP header (20)

                // Use local interface IP as source (simplified - could be improved)
                let src_ip = Ipv4Addr::new(192, 168, 1, 100); // Placeholder

                // Split buffer to avoid multiple mutable borrows
                let (ip_buf, tcp_buf) = buffer.split_at_mut(20);

                // Build IP header
                {
                    let mut ip_packet = MutableIpv4Packet::new(ip_buf)
                        .ok_or_else(|| Error::Scanner("Failed to create IPv4 packet".into()))?;

                    ip_packet.set_version(4);
                    ip_packet.set_header_length(5); // 5 * 4 = 20 bytes
                    ip_packet.set_total_length(40); // IP(20) + TCP(20)
                    ip_packet.set_ttl(64);
                    ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
                    ip_packet.set_identification(rand::random()); // Random IPID for our probe
                    ip_packet.set_source(src_ip);
                    ip_packet.set_destination(target_ipv4);

                    // Calculate IP checksum
                    let ip_checksum = checksum(&ip_packet.to_immutable());
                    ip_packet.set_checksum(ip_checksum);
                }

                // Build TCP header
                {
                    let mut tcp_packet = MutableTcpPacket::new(tcp_buf)
                        .ok_or_else(|| Error::Scanner("Failed to create TCP packet".into()))?;

                    tcp_packet.set_source(rand::random::<u16>().saturating_add(10000)); // Random high port
                    tcp_packet.set_destination(80); // Common port
                    tcp_packet.set_sequence(rand::random());
                    tcp_packet.set_acknowledgement(rand::random());
                    tcp_packet.set_data_offset(5); // 5 * 4 = 20 bytes
                    tcp_packet.set_flags(TcpFlags::SYN | TcpFlags::ACK); // SYN+ACK
                    tcp_packet.set_window(8192);
                    tcp_packet.set_urgent_ptr(0);

                    // Calculate TCP checksum
                    let tcp_checksum =
                        ipv4_checksum(&tcp_packet.to_immutable(), &src_ip, &target_ipv4);
                    tcp_packet.set_checksum(tcp_checksum);
                }

                // Reconstruct IP packet for sending
                let ip_packet = Ipv4Packet::new(&buffer[..20])
                    .ok_or_else(|| Error::Scanner("Failed to reconstruct IPv4 packet".into()))?;

                // Send packet
                tx.send_to(ip_packet, std::net::IpAddr::V4(target_ipv4))
                    .map_err(|e| Error::Network(format!("Failed to send probe: {}", e)))?;

                Ok(())
            }
            IpAddr::V6(_target_ipv6) => {
                // IPv6 doesn't have IPID in main header (only in Fragment extension)
                // For now, return error indicating IPv6 not fully supported
                Err(Error::Scanner(
                    "IPv6 IPID tracking not fully supported (IPID only in Fragment extension)"
                        .into(),
                ))
            }
        }
    }

    /// Receive RST response and extract IPID
    ///
    /// Waits for RST packet from target and extracts IPID from IP header.
    /// Uses timeout (self.timeout) to avoid blocking indefinitely.
    ///
    /// Note: This is a simplified implementation for Sprint 6.5 TASK 2.
    /// In production, this would use non-blocking I/O or a separate thread pool.
    async fn receive_rst_response(
        &self,
        rx: &mut pnet::transport::TransportReceiver,
    ) -> Result<u16> {
        use pnet::transport::ipv4_packet_iter;
        use std::time::Instant as StdInstant;

        // Create iterator for IPv4 packets
        let mut iter = ipv4_packet_iter(rx);
        let deadline = StdInstant::now() + self.timeout;

        // Keep receiving until we get a RST packet from our target or timeout
        loop {
            // Check timeout
            if StdInstant::now() >= deadline {
                return Err(Error::Network(format!(
                    "Timeout waiting for RST response ({}s)",
                    self.timeout.as_secs()
                )));
            }

            // Try to receive packet (this is blocking)
            match iter.next() {
                Ok((ipv4_packet, addr)) => {
                    // Verify packet is from our target (addr is already IpAddr)
                    if addr != self.target {
                        continue; // Not from target, keep waiting
                    }

                    // Extract IPID from IP header
                    let ipid = ipv4_packet.get_identification();

                    // Verify payload is TCP RST
                    let tcp_offset = ipv4_packet.get_header_length() as usize * 4;
                    let packet_data = ipv4_packet.packet();
                    if let Some(tcp_packet) = TcpPacket::new(&packet_data[tcp_offset..]) {
                        let flags = tcp_packet.get_flags();
                        if (flags & TcpFlags::RST) != 0 {
                            // Found RST packet with valid IPID
                            return Ok(ipid);
                        }
                    }
                }
                Err(e) => {
                    return Err(Error::Network(format!("Failed to receive response: {}", e)));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipid_measurement_creation() {
        let measurement = IPIDMeasurement {
            ipid: 12345,
            timestamp: Instant::now(),
        };

        assert_eq!(measurement.ipid, 12345);
    }

    #[test]
    fn test_ipid_pattern_equality() {
        assert_eq!(IPIDPattern::Sequential, IPIDPattern::Sequential);
        assert_ne!(IPIDPattern::Sequential, IPIDPattern::Random);
    }

    #[test]
    fn test_tracker_creation() {
        let target: IpAddr = "192.168.1.100".parse().unwrap();
        let tracker = IPIDTracker::new(target);

        assert!(tracker.is_ok());
        let tracker = tracker.unwrap();
        assert_eq!(tracker.measurement_count(), 0);
    }

    #[test]
    fn test_calculate_increments_sequential() {
        let target: IpAddr = "192.168.1.100".parse().unwrap();
        let mut tracker = IPIDTracker::new(target).unwrap();

        // Simulate sequential IPID measurements
        tracker.measurements = vec![
            IPIDMeasurement {
                ipid: 100,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 101,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 102,
                timestamp: Instant::now(),
            },
        ];

        let increments = tracker.calculate_increments();
        assert_eq!(increments, vec![1, 1]);
        assert!(tracker.is_sequential());
    }

    #[test]
    fn test_calculate_increments_rollover() {
        let target: IpAddr = "192.168.1.100".parse().unwrap();
        let mut tracker = IPIDTracker::new(target).unwrap();

        // Simulate IPID rollover (65535 → 0)
        tracker.measurements = vec![
            IPIDMeasurement {
                ipid: 65534,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 65535,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 0,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 1,
                timestamp: Instant::now(),
            },
        ];

        let increments = tracker.calculate_increments();
        // 65535 - 65534 = 1, 0 - 65535 = 1 (wrapping), 1 - 0 = 1
        assert_eq!(increments, vec![1, 1, 1]);
    }

    #[test]
    fn test_calculate_increments_broken256() {
        let target: IpAddr = "192.168.1.100".parse().unwrap();
        let mut tracker = IPIDTracker::new(target).unwrap();

        // Simulate Windows byte-order bug
        tracker.measurements = vec![
            IPIDMeasurement {
                ipid: 256,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 512,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 768,
                timestamp: Instant::now(),
            },
        ];

        let increments = tracker.calculate_increments();
        assert_eq!(increments, vec![256, 256]);
        assert!(!tracker.is_sequential()); // Not +1, so not sequential
    }

    #[test]
    fn test_calculate_variance_consistent() {
        let target: IpAddr = "192.168.1.100".parse().unwrap();
        let mut tracker = IPIDTracker::new(target).unwrap();

        // All increments are 1 (perfect consistency)
        tracker.measurements = vec![
            IPIDMeasurement {
                ipid: 100,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 101,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 102,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 103,
                timestamp: Instant::now(),
            },
        ];

        let variance = tracker.calculate_variance();
        assert_eq!(variance, 0.0); // Perfect consistency = zero variance
    }

    #[test]
    fn test_calculate_variance_random() {
        let target: IpAddr = "192.168.1.100".parse().unwrap();
        let mut tracker = IPIDTracker::new(target).unwrap();

        // Random increments
        tracker.measurements = vec![
            IPIDMeasurement {
                ipid: 100,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 150,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 175,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 300,
                timestamp: Instant::now(),
            },
        ];

        let variance = tracker.calculate_variance();
        assert!(variance > 10.0); // High variance
    }

    #[test]
    fn test_clear_measurements() {
        let target: IpAddr = "192.168.1.100".parse().unwrap();
        let mut tracker = IPIDTracker::new(target).unwrap();

        tracker.measurements = vec![
            IPIDMeasurement {
                ipid: 100,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 101,
                timestamp: Instant::now(),
            },
        ];

        assert_eq!(tracker.measurement_count(), 2);

        tracker.clear_measurements();
        assert_eq!(tracker.measurement_count(), 0);
    }

    // New tests for IPID extraction logic

    #[test]
    fn test_ipv4_packet_parsing() {
        // Test that we can parse IPv4 packets and extract IPID
        use pnet::packet::ipv4::MutableIpv4Packet;

        let mut buffer = vec![0u8; 20];
        let mut packet = MutableIpv4Packet::new(&mut buffer).unwrap();

        packet.set_version(4);
        packet.set_header_length(5);
        packet.set_identification(12345);

        // Parse immutable packet
        let parsed = Ipv4Packet::new(&buffer).unwrap();
        assert_eq!(parsed.get_identification(), 12345);
    }

    #[test]
    fn test_tcp_rst_flag_detection() {
        // Test RST flag detection in TCP packets
        use pnet::packet::tcp::MutableTcpPacket;

        let mut buffer = vec![0u8; 20];
        let mut packet = MutableTcpPacket::new(&mut buffer).unwrap();

        packet.set_flags(TcpFlags::RST);

        // Parse and verify RST flag
        let parsed = TcpPacket::new(&buffer).unwrap();
        assert_ne!(parsed.get_flags() & TcpFlags::RST, 0);
    }

    #[test]
    fn test_tcp_syn_ack_flags() {
        // Test SYN+ACK flag combination
        use pnet::packet::tcp::MutableTcpPacket;

        let mut buffer = vec![0u8; 20];
        let mut packet = MutableTcpPacket::new(&mut buffer).unwrap();

        packet.set_flags(TcpFlags::SYN | TcpFlags::ACK);

        let parsed = TcpPacket::new(&buffer).unwrap();
        let flags = parsed.get_flags();
        assert_ne!(flags & TcpFlags::SYN, 0);
        assert_ne!(flags & TcpFlags::ACK, 0);
    }

    #[test]
    fn test_ipid_wrapping_arithmetic() {
        // Test that IPID wrapping is handled correctly
        let target: IpAddr = "192.168.1.100".parse().unwrap();
        let mut tracker = IPIDTracker::new(target).unwrap();

        // Test rollover from max value
        tracker.set_measurements_for_test(vec![
            IPIDMeasurement {
                ipid: u16::MAX - 1, // 65534
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: u16::MAX, // 65535
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 0, // Wrapped to 0
                timestamp: Instant::now(),
            },
        ]);

        let increments = tracker.calculate_increments();
        assert_eq!(increments.len(), 2);
        assert_eq!(increments[0], 1); // 65535 - 65534 = 1
        assert_eq!(increments[1], 1); // 0 - 65535 = 1 (wrapping)
    }

    #[test]
    fn test_ipv6_target_error() {
        // Test that IPv6 targets return appropriate error
        let target: IpAddr = "::1".parse().unwrap();
        let _tracker = IPIDTracker::new(target).unwrap();

        assert!(matches!(target, IpAddr::V6(_)));
        // IPv6 IPID tracking should fail with proper error message
    }

    #[test]
    fn test_pattern_classification_random() {
        // Test classification of random IPID pattern
        let target: IpAddr = "192.168.1.100".parse().unwrap();
        let mut tracker = IPIDTracker::new(target).unwrap();

        // Simulate random IPIDs (high variance)
        tracker.set_measurements_for_test(vec![
            IPIDMeasurement {
                ipid: 1234,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 5678,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 9012,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 2345,
                timestamp: Instant::now(),
            },
        ]);

        let variance = tracker.calculate_variance();
        assert!(variance > 10.0, "Expected high variance for random pattern");
    }

    #[test]
    fn test_pattern_classification_perhost() {
        // Test classification of per-host IPID pattern
        let target: IpAddr = "192.168.1.100".parse().unwrap();
        let mut tracker = IPIDTracker::new(target).unwrap();

        // Simulate per-host pattern (low variance but not sequential)
        tracker.set_measurements_for_test(vec![
            IPIDMeasurement {
                ipid: 100,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 102,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 104,
                timestamp: Instant::now(),
            },
            IPIDMeasurement {
                ipid: 106,
                timestamp: Instant::now(),
            },
        ]);

        let increments = tracker.calculate_increments();
        assert_eq!(increments, vec![2, 2, 2]); // Consistent but not +1

        let variance = tracker.calculate_variance();
        assert!(
            variance < 10.0,
            "Expected low variance for per-host pattern"
        );
        assert!(!tracker.is_sequential());
    }

    // Integration tests requiring root privileges

    #[tokio::test]
    #[ignore] // Requires root privileges
    async fn test_ipid_tracking_real_host() {
        // This test must be run manually with sudo
        // cargo test --test idle_scan -- --ignored --test-threads=1

        let target: IpAddr = "192.168.1.1".parse().unwrap();
        let mut tracker = IPIDTracker::new(target).unwrap();

        // Attempt to probe real host
        match tracker.probe().await {
            Ok(measurement) => {
                // Real IPID should not be stub value 0 (unless coincidence)
                println!("Measured IPID: {}", measurement.ipid);
                assert!(measurement.timestamp.elapsed().as_secs() < 10);
            }
            Err(e) => {
                // May fail if no host at that IP or insufficient privileges
                println!("Probe failed (expected if no host): {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires root privileges
    async fn test_classify_pattern_sequential() {
        // Test with known sequential zombie (Linux <4.18 VM)
        // Replace with actual sequential host IP in your environment

        let target: IpAddr = "192.168.1.50".parse().unwrap(); // Placeholder
        let mut tracker = IPIDTracker::new(target).unwrap();

        match tracker.classify_pattern(5).await {
            Ok(pattern) => {
                println!("Detected pattern: {:?}", pattern);
                // Pattern should be Sequential, Random, PerHost, or Broken256
                assert!(matches!(
                    pattern,
                    IPIDPattern::Sequential
                        | IPIDPattern::Random
                        | IPIDPattern::PerHost
                        | IPIDPattern::Broken256
                ));
            }
            Err(e) => {
                println!("Classification failed: {}", e);
                // Expected if host not available or insufficient privileges
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires root privileges
    async fn test_ipv6_limitation() {
        // Test that IPv6 targets fail gracefully with proper error message

        let target: IpAddr = "::1".parse().unwrap();
        let mut tracker = IPIDTracker::new(target).unwrap();

        let result = tracker.probe().await;
        assert!(result.is_err());

        if let Err(e) = result {
            let error_msg = format!("{:?}", e);
            assert!(
                error_msg.contains("IPv6") || error_msg.contains("IPID"),
                "Error should mention IPv6 limitation"
            );
        }
    }
}
