//! Zombie Host Discovery for Idle Scan
//!
//! This module provides functionality for discovering suitable "zombie" hosts that can
//! be used for idle scanning. A good zombie must have predictable IPID increments and
//! minimal network traffic.
//!
//! # Zombie Requirements
//!
//! 1. **Idle**: Minimal background network activity (predictable IPID)
//! 2. **Sequential IPID**: Increments by 1 per packet (+1, +2, +3, ...)
//! 3. **Reachable**: Responds to SYN/ACK probes with RST
//! 4. **No firewall**: Allows outbound RST packets
//!
//! # Good Zombie Candidates
//!
//! - Printers and IoT devices (simple network stacks)
//! - Legacy systems (Linux <4.18, Windows XP/2003)
//! - Idle workstations
//! - Kiosks and information displays
//!
//! Modern systems (Linux 4.18+, Windows 10+) use random IPID for security.
//!
//! # See Also
//!
//! - [Idle Scan Guide](../../../docs/25-IDLE-SCAN-GUIDE.md) - Zombie selection criteria and techniques
//! - [`IdleScanner`](super::idle_scanner::IdleScanner) - Using discovered zombies for scanning

use crate::idle::ipid_tracker::{IPIDPattern, IPIDTracker};
use ipnetwork::IpNetwork;
use prtip_core::{Error, Result};
use std::net::IpAddr;
use std::time::Instant;

/// Zombie host candidate with quality metrics
#[derive(Debug, Clone)]
pub struct ZombieCandidate {
    /// IP address of the zombie host
    pub ip: IpAddr,

    /// IPID increment pattern (Sequential/Random/PerHost/Broken256)
    pub pattern: IPIDPattern,

    /// Quality score: 0.0 (bad) to 1.0 (perfect)
    /// Based on pattern consistency, latency, and responsiveness
    pub quality_score: f32,

    /// Round-trip time to zombie in milliseconds
    pub latency_ms: u64,

    /// When this candidate was last tested
    pub last_tested: Instant,
}

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Number of IPID probes per host (default: 3)
    pub probes_per_host: usize,

    /// Time between probes in milliseconds (default: 1000)
    pub probe_interval_ms: u64,

    /// Minimum acceptable quality score (default: 0.7)
    pub min_quality_score: f32,

    /// Maximum number of candidates to return (default: 10)
    pub max_candidates: usize,

    /// Timeout for host responsiveness check (default: 5000ms)
    pub host_timeout_ms: u64,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            probes_per_host: 3,
            probe_interval_ms: 1000,
            min_quality_score: 0.7,
            max_candidates: 10,
            host_timeout_ms: 5000,
        }
    }
}

/// Zombie discovery scanner
///
/// Scans a subnet to find hosts with predictable IPID patterns suitable
/// for use as zombies in idle scanning.
pub struct ZombieDiscovery {
    /// Subnet to scan (e.g., "192.168.1.0/24")
    subnet: String,

    /// Configuration
    config: DiscoveryConfig,
}

impl ZombieDiscovery {
    /// Create new zombie discovery scanner for subnet
    ///
    /// # Arguments
    /// * `subnet` - CIDR subnet notation (e.g., "192.168.1.0/24")
    ///
    /// # Example
    /// ```
    /// use prtip_scanner::idle::ZombieDiscovery;
    ///
    /// let discovery = ZombieDiscovery::new("192.168.1.0/24".to_string());
    /// ```
    pub fn new(subnet: String) -> Self {
        Self {
            subnet,
            config: DiscoveryConfig::default(),
        }
    }

    /// Create zombie discovery with custom configuration
    ///
    /// # Arguments
    /// * `subnet` - CIDR subnet notation
    /// * `config` - Custom discovery configuration
    pub fn with_config(subnet: String, config: DiscoveryConfig) -> Self {
        Self { subnet, config }
    }

    /// Scan subnet for potential zombies
    ///
    /// Tests each host in the subnet and returns a sorted list of candidates
    /// (best first) that meet the minimum quality threshold.
    ///
    /// # Returns
    /// * `Result<Vec<ZombieCandidate>>` - Sorted list of suitable zombies
    ///
    /// # Errors
    /// * Network errors during scanning
    /// * Invalid subnet notation
    pub async fn find_zombies(&self) -> Result<Vec<ZombieCandidate>> {
        // Parse subnet
        let network = self
            .subnet
            .parse::<IpNetwork>()
            .map_err(|e| Error::Parse(format!("Invalid subnet '{}': {}", self.subnet, e)))?;

        let mut candidates = Vec::new();

        // Test each host in subnet
        for ip in network.iter() {
            // Skip network and broadcast addresses
            if ip == network.network() || ip == network.broadcast() {
                continue;
            }

            // Test candidate
            match self.test_candidate(ip).await {
                Ok(candidate) => {
                    // Only keep if meets minimum quality and has good pattern
                    if candidate.quality_score >= self.config.min_quality_score
                        && (candidate.pattern == IPIDPattern::Sequential
                            || candidate.pattern == IPIDPattern::Broken256)
                    {
                        candidates.push(candidate);

                        // Stop if we have enough candidates
                        if candidates.len() >= self.config.max_candidates {
                            break;
                        }
                    }
                }
                Err(_) => {
                    // Skip hosts that fail (unresponsive, filtered, etc.)
                    continue;
                }
            }
        }

        // Sort by quality score (highest first)
        candidates.sort_by(|a, b| {
            b.quality_score
                .partial_cmp(&a.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(candidates)
    }

    /// Test single host as zombie candidate
    ///
    /// Sends multiple IPID probes to classify the host's pattern and
    /// calculate a quality score.
    ///
    /// # Arguments
    /// * `ip` - IP address to test
    ///
    /// # Returns
    /// * `Result<ZombieCandidate>` - Candidate with metrics
    async fn test_candidate(&self, ip: IpAddr) -> Result<ZombieCandidate> {
        let start_time = Instant::now();

        // Create IPID tracker
        let mut tracker = IPIDTracker::new(ip)?;

        // Classify IPID pattern with multiple probes
        let pattern = tracker
            .classify_pattern(self.config.probes_per_host)
            .await?;

        // Calculate latency
        let latency_ms =
            start_time.elapsed().as_millis() as u64 / self.config.probes_per_host as u64;

        // Calculate quality score
        let quality_score = self.calculate_quality(&tracker, pattern, latency_ms);

        Ok(ZombieCandidate {
            ip,
            pattern,
            quality_score,
            latency_ms,
            last_tested: Instant::now(),
        })
    }

    /// Calculate zombie quality score (0.0-1.0)
    ///
    /// Scoring factors:
    /// - Pattern type (0.0-0.5): Sequential = 0.5, Broken256 = 0.4, others = 0.0
    /// - Consistency (0.0-0.3): Based on increment variance
    /// - Latency (0.0-0.2): Lower latency = higher score
    ///
    /// # Arguments
    /// * `tracker` - IPID tracker with measurements
    /// * `pattern` - Classified IPID pattern
    /// * `latency_ms` - Average round-trip time
    ///
    /// # Returns
    /// * `f32` - Quality score 0.0-1.0
    fn calculate_quality(
        &self,
        tracker: &IPIDTracker,
        pattern: IPIDPattern,
        latency_ms: u64,
    ) -> f32 {
        let mut score = 0.0;

        // Pattern quality (0.0-0.5)
        score += match pattern {
            IPIDPattern::Sequential => 0.5, // Perfect
            IPIDPattern::Broken256 => 0.4,  // Windows bug, still usable
            IPIDPattern::PerHost => 0.0,    // Not suitable
            IPIDPattern::Random => 0.0,     // Not suitable
        };

        // Consistency (0.0-0.3)
        let increments = tracker.calculate_increments();
        if !increments.is_empty() {
            // Check if all increments are the same
            let first_inc = increments[0];
            let all_same = increments.iter().all(|&inc| inc == first_inc);

            if all_same {
                score += 0.3; // Perfect consistency
            } else {
                // Calculate variance penalty
                let variance = tracker.calculate_variance();
                let consistency = (1.0 - (variance / 10.0).min(1.0)) * 0.3;
                score += consistency;
            }
        }

        // Latency (0.0-0.2)
        let latency_score = match latency_ms {
            0..=50 => 0.2,      // Excellent (<50ms)
            51..=200 => 0.15,   // Good (50-200ms)
            201..=500 => 0.1,   // Acceptable (200-500ms)
            501..=1000 => 0.05, // Poor (500-1000ms)
            _ => 0.0,           // Very poor (>1000ms)
        };
        score += latency_score;

        // Clamp to 0.0-1.0
        score.clamp(0.0, 1.0)
    }

    /// Check if host is responsive (basic connectivity test)
    ///
    /// # Arguments
    /// * `ip` - IP address to test
    ///
    /// # Returns
    /// * `bool` - True if host responds to probe
    pub async fn is_responsive(&self, ip: IpAddr) -> bool {
        let mut tracker = match IPIDTracker::new(ip) {
            Ok(t) => t,
            Err(_) => return false,
        };

        // Try single probe with timeout
        matches!(
            tokio::time::timeout(
                std::time::Duration::from_millis(self.config.host_timeout_ms),
                tracker.probe()
            )
            .await,
            Ok(Ok(_))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();

        assert_eq!(config.probes_per_host, 3);
        assert_eq!(config.probe_interval_ms, 1000);
        assert_eq!(config.min_quality_score, 0.7);
        assert_eq!(config.max_candidates, 10);
        assert_eq!(config.host_timeout_ms, 5000);
    }

    #[test]
    fn test_zombie_discovery_creation() {
        let discovery = ZombieDiscovery::new("192.168.1.0/24".to_string());
        assert_eq!(discovery.subnet, "192.168.1.0/24");
    }

    #[test]
    fn test_zombie_discovery_with_config() {
        let config = DiscoveryConfig {
            probes_per_host: 5,
            probe_interval_ms: 500,
            min_quality_score: 0.8,
            max_candidates: 5,
            host_timeout_ms: 3000,
        };

        let discovery = ZombieDiscovery::with_config("10.0.0.0/24".to_string(), config.clone());
        assert_eq!(discovery.config.probes_per_host, 5);
        assert_eq!(discovery.config.min_quality_score, 0.8);
    }

    #[test]
    fn test_quality_score_sequential() {
        let discovery = ZombieDiscovery::new("192.168.1.0/24".to_string());
        let ip: IpAddr = "192.168.1.100".parse().unwrap();
        let mut tracker = IPIDTracker::new(ip).unwrap();

        // Simulate perfect sequential IPID
        use crate::idle::ipid_tracker::IPIDMeasurement;
        tracker.set_measurements_for_test(vec![
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
        ]);

        let score = discovery.calculate_quality(&tracker, IPIDPattern::Sequential, 50);

        // Should get: 0.5 (pattern) + 0.3 (consistency) + 0.2 (latency) = 1.0
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_quality_score_broken256() {
        let discovery = ZombieDiscovery::new("192.168.1.0/24".to_string());
        let ip: IpAddr = "192.168.1.100".parse().unwrap();
        let mut tracker = IPIDTracker::new(ip).unwrap();

        // Simulate Windows byte-order bug
        use crate::idle::ipid_tracker::IPIDMeasurement;
        tracker.set_measurements_for_test(vec![
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
        ]);

        let score = discovery.calculate_quality(&tracker, IPIDPattern::Broken256, 100);

        // Should get: 0.4 (pattern) + 0.3 (consistency) + 0.15 (latency) = 0.85
        assert_eq!(score, 0.85);
    }

    #[test]
    fn test_quality_score_random() {
        let discovery = ZombieDiscovery::new("192.168.1.0/24".to_string());
        let ip: IpAddr = "192.168.1.100".parse().unwrap();
        let tracker = IPIDTracker::new(ip).unwrap();

        let score = discovery.calculate_quality(&tracker, IPIDPattern::Random, 50);

        // Random pattern gets 0.0 for pattern score
        // With no measurements, gets 0.0 for consistency
        // Good latency gets 0.2
        assert!(score <= 0.2);
    }

    #[test]
    fn test_quality_score_latency_tiers() {
        let discovery = ZombieDiscovery::new("192.168.1.0/24".to_string());
        let ip: IpAddr = "192.168.1.100".parse().unwrap();
        let tracker = IPIDTracker::new(ip).unwrap();

        // Test different latency tiers
        let score_10ms = discovery.calculate_quality(&tracker, IPIDPattern::Sequential, 10);
        let score_100ms = discovery.calculate_quality(&tracker, IPIDPattern::Sequential, 100);
        let score_300ms = discovery.calculate_quality(&tracker, IPIDPattern::Sequential, 300);
        let score_800ms = discovery.calculate_quality(&tracker, IPIDPattern::Sequential, 800);
        let score_2000ms = discovery.calculate_quality(&tracker, IPIDPattern::Sequential, 2000);

        // Lower latency should have higher score
        assert!(score_10ms > score_100ms);
        assert!(score_100ms > score_300ms);
        assert!(score_300ms > score_800ms);
        assert!(score_800ms > score_2000ms);
    }

    #[test]
    fn test_zombie_candidate_creation() {
        let candidate = ZombieCandidate {
            ip: "192.168.1.100".parse().unwrap(),
            pattern: IPIDPattern::Sequential,
            quality_score: 0.95,
            latency_ms: 45,
            last_tested: Instant::now(),
        };

        assert_eq!(candidate.quality_score, 0.95);
        assert_eq!(candidate.latency_ms, 45);
        assert_eq!(candidate.pattern, IPIDPattern::Sequential);
    }
}
