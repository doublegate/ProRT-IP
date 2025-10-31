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
use pnet::transport::{transport_channel, TransportChannelType, TransportProtocol};
use prtip_core::{Error, Result};
use std::net::IpAddr;
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
        // Create transport channel for sending/receiving TCP packets
        let protocol =
            TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp));

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
    async fn send_syn_ack_probe(&self, _tx: &mut pnet::transport::TransportSender) -> Result<()> {
        // TODO: Implement SYN/ACK packet building and sending
        // This will be implemented in Phase 3 with proper packet construction
        Ok(())
    }

    /// Receive RST response and extract IPID
    async fn receive_rst_response(
        &self,
        _rx: &mut pnet::transport::TransportReceiver,
    ) -> Result<u16> {
        // TODO: Implement RST packet reception and IPID extraction
        // This will be implemented in Phase 3 with proper packet parsing

        // Placeholder: Return a mock IPID for now
        Ok(0)
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
}
