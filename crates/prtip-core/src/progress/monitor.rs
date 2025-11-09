//! Throughput monitoring with sliding window metrics
//!
//! This module provides real-time throughput tracking for network scanning operations,
//! calculating packets per second (pps), hosts per minute (hpm), and bandwidth (Mbps)
//! using a sliding window approach for accurate, responsive metrics.
//!
//! # Metrics
//!
//! - **PPS (Packets Per Second)**: Total packets sent/received per second
//! - **HPM (Hosts Per Minute)**: Targets scanned per minute
//! - **Mbps (Megabits Per Second)**: Network bandwidth utilization
//!
//! # Algorithm
//!
//! Uses a 60-second sliding window with 1-second buckets for granular tracking.
//! Old buckets are automatically pruned to maintain constant memory usage.
//!
//! # Examples
//!
//! ```no_run
//! use prtip_core::progress::ThroughputMonitor;
//!
//! # async fn example() {
//! let monitor = ThroughputMonitor::new();
//!
//! // Record activity
//! monitor.record_packets(100).await; // 100 packets sent
//! monitor.record_bytes(15000).await; // 15KB transferred
//! monitor.record_host_completed().await; // 1 host scanned
//!
//! // Get current throughput
//! let throughput = monitor.current_throughput().await;
//! println!("PPS: {:.0}", throughput.packets_per_second);
//! println!("HPM: {:.0}", throughput.hosts_per_minute);
//! println!("Mbps: {:.2}", throughput.bandwidth_mbps);
//! # }
//! ```

use crate::events::Throughput;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

const WINDOW_DURATION_SECS: u64 = 60; // Track last 60 seconds
const BUCKET_DURATION_SECS: u64 = 1; // 1-second granularity

/// Throughput data for a time bucket
#[derive(Debug, Clone, Copy, Default)]
struct Bucket {
    /// When this bucket started
    timestamp: Option<Instant>,
    /// Total packets in this bucket
    packets: u64,
    /// Total bytes in this bucket
    bytes: u64,
    /// Hosts completed in this bucket
    hosts: u64,
}

/// Throughput monitor state
#[derive(Debug)]
struct MonitorState {
    /// Sliding window of buckets (60 buckets = 60 seconds)
    buckets: VecDeque<Bucket>,
    /// Maximum buckets to retain
    max_buckets: usize,
    /// Current bucket (accumulating)
    current_bucket: Bucket,
    /// Last update time
    last_update: Instant,
}

/// Real-time throughput monitor with sliding window
///
/// Tracks packets per second (pps), hosts per minute (hpm), and
/// bandwidth (Mbps) using a 60-second sliding window with 1-second granularity.
///
/// # Thread Safety
///
/// Uses Arc<RwLock> for concurrent access from multiple threads.
///
/// # Performance
///
/// - Record: O(1) (updates current bucket)
/// - Query: O(n) where n = window size (60 buckets max)
/// - Memory: O(1) (fixed-size window)
#[derive(Clone)]
pub struct ThroughputMonitor {
    state: Arc<RwLock<MonitorState>>,
}

impl ThroughputMonitor {
    /// Create a new throughput monitor
    ///
    /// Uses a 60-second sliding window by default.
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::progress::ThroughputMonitor;
    ///
    /// let monitor = ThroughputMonitor::new();
    /// ```
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            state: Arc::new(RwLock::new(MonitorState {
                buckets: VecDeque::new(),
                max_buckets: WINDOW_DURATION_SECS as usize / BUCKET_DURATION_SECS as usize,
                current_bucket: Bucket {
                    timestamp: Some(now),
                    packets: 0,
                    bytes: 0,
                    hosts: 0,
                },
                last_update: now,
            })),
        }
    }

    /// Record packets sent/received
    ///
    /// # Arguments
    ///
    /// * `count` - Number of packets
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ThroughputMonitor;
    ///
    /// # async fn example() {
    /// let monitor = ThroughputMonitor::new();
    /// monitor.record_packets(100).await;
    /// # }
    /// ```
    pub async fn record_packets(&self, count: u64) {
        self.rotate_bucket_if_needed().await;
        let mut state = self.state.write();
        state.current_bucket.packets += count;
        state.last_update = Instant::now();
    }

    /// Record bytes transferred
    ///
    /// # Arguments
    ///
    /// * `bytes` - Number of bytes
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ThroughputMonitor;
    ///
    /// # async fn example() {
    /// let monitor = ThroughputMonitor::new();
    /// monitor.record_bytes(15000).await; // 15KB
    /// # }
    /// ```
    pub async fn record_bytes(&self, bytes: u64) {
        self.rotate_bucket_if_needed().await;
        let mut state = self.state.write();
        state.current_bucket.bytes += bytes;
        state.last_update = Instant::now();
    }

    /// Record host completion
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ThroughputMonitor;
    ///
    /// # async fn example() {
    /// let monitor = ThroughputMonitor::new();
    /// monitor.record_host_completed().await;
    /// # }
    /// ```
    pub async fn record_host_completed(&self) {
        self.rotate_bucket_if_needed().await;
        let mut state = self.state.write();
        state.current_bucket.hosts += 1;
        state.last_update = Instant::now();
    }

    /// Get current throughput metrics
    ///
    /// Calculates throughput across the entire sliding window.
    ///
    /// # Returns
    ///
    /// [`Throughput`] struct with pps, hpm, and Mbps
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ThroughputMonitor;
    ///
    /// # async fn example() {
    /// let monitor = ThroughputMonitor::new();
    /// monitor.record_packets(1000).await;
    ///
    /// let tp = monitor.current_throughput().await;
    /// println!("PPS: {:.0}", tp.packets_per_second);
    /// # }
    /// ```
    pub async fn current_throughput(&self) -> Throughput {
        self.rotate_bucket_if_needed().await;
        let state = self.state.read();

        // Sum all buckets in window
        let mut total_packets = 0u64;
        let mut total_bytes = 0u64;
        let mut total_hosts = 0u64;

        for bucket in &state.buckets {
            total_packets += bucket.packets;
            total_bytes += bucket.bytes;
            total_hosts += bucket.hosts;
        }

        // Include current bucket
        total_packets += state.current_bucket.packets;
        total_bytes += state.current_bucket.bytes;
        total_hosts += state.current_bucket.hosts;

        // Calculate rates
        let window_secs = WINDOW_DURATION_SECS as f64;

        let packets_per_second = total_packets as f64 / window_secs;
        let hosts_per_minute = (total_hosts as f64 / window_secs) * 60.0;
        let bandwidth_mbps = (total_bytes as f64 * 8.0) / (window_secs * 1_000_000.0);

        Throughput {
            packets_per_second,
            hosts_per_minute,
            bandwidth_mbps,
        }
    }

    /// Get instantaneous throughput (last second only)
    ///
    /// Returns metrics for the current bucket, useful for detecting
    /// short-term rate changes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ThroughputMonitor;
    ///
    /// # async fn example() {
    /// let monitor = ThroughputMonitor::new();
    /// let instant = monitor.instant_throughput().await;
    /// println!("Current second: {:.0} pps", instant.packets_per_second);
    /// # }
    /// ```
    pub async fn instant_throughput(&self) -> Throughput {
        let state = self.state.read();

        let packets_per_second = state.current_bucket.packets as f64;
        let hosts_per_minute = (state.current_bucket.hosts as f64) * 60.0;
        let bandwidth_mbps = (state.current_bucket.bytes as f64 * 8.0) / 1_000_000.0;

        Throughput {
            packets_per_second,
            hosts_per_minute,
            bandwidth_mbps,
        }
    }

    /// Rotate to new bucket if current bucket is old enough
    async fn rotate_bucket_if_needed(&self) {
        let now = Instant::now();
        let should_rotate = {
            let state = self.state.read();
            if let Some(timestamp) = state.current_bucket.timestamp {
                now.duration_since(timestamp).as_secs() >= BUCKET_DURATION_SECS
            } else {
                false
            }
        };

        if should_rotate {
            let mut state = self.state.write();

            // Move current bucket to history (clone to avoid borrow conflict)
            let bucket_to_save = state.current_bucket;
            state.buckets.push_back(bucket_to_save);

            // Trim old buckets
            while state.buckets.len() > state.max_buckets {
                state.buckets.pop_front();
            }

            // Start new bucket
            state.current_bucket = Bucket {
                timestamp: Some(now),
                packets: 0,
                bytes: 0,
                hosts: 0,
            };
        }
    }

    /// Reset all throughput metrics
    ///
    /// Clears the sliding window and starts fresh.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ThroughputMonitor;
    ///
    /// # async fn example() {
    /// let monitor = ThroughputMonitor::new();
    /// monitor.reset().await;
    /// # }
    /// ```
    pub async fn reset(&self) {
        let now = Instant::now();
        let mut state = self.state.write();
        state.buckets.clear();
        state.current_bucket = Bucket {
            timestamp: Some(now),
            packets: 0,
            bytes: 0,
            hosts: 0,
        };
        state.last_update = now;
    }
}

impl Default for ThroughputMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_new_monitor() {
        let monitor = ThroughputMonitor::new();
        let tp = monitor.current_throughput().await;

        assert_eq!(tp.packets_per_second, 0.0);
        assert_eq!(tp.hosts_per_minute, 0.0);
        assert_eq!(tp.bandwidth_mbps, 0.0);
    }

    #[tokio::test]
    async fn test_record_packets() {
        let monitor = ThroughputMonitor::new();
        monitor.record_packets(1000).await;

        let tp = monitor.current_throughput().await;
        assert!(tp.packets_per_second > 0.0);
    }

    #[tokio::test]
    async fn test_record_bytes() {
        let monitor = ThroughputMonitor::new();
        monitor.record_bytes(100_000).await; // 100KB

        let tp = monitor.current_throughput().await;
        assert!(tp.bandwidth_mbps > 0.0);
    }

    #[tokio::test]
    async fn test_record_hosts() {
        let monitor = ThroughputMonitor::new();
        monitor.record_host_completed().await;
        monitor.record_host_completed().await;
        monitor.record_host_completed().await;

        let tp = monitor.current_throughput().await;
        assert!(tp.hosts_per_minute > 0.0);
    }

    #[tokio::test]
    async fn test_instant_throughput() {
        let monitor = ThroughputMonitor::new();
        monitor.record_packets(500).await;

        let instant = monitor.instant_throughput().await;
        assert_eq!(instant.packets_per_second, 500.0);
    }

    #[tokio::test]
    async fn test_bucket_rotation() {
        let monitor = ThroughputMonitor::new();

        // Record in first bucket
        monitor.record_packets(100).await;

        // Wait for bucket rotation
        sleep(Duration::from_secs(2)).await;

        // Record in new bucket
        monitor.record_packets(200).await;

        // Should have both buckets in window
        let tp = monitor.current_throughput().await;
        assert!(tp.packets_per_second > 0.0);
    }

    #[tokio::test]
    async fn test_window_sliding() {
        let monitor = ThroughputMonitor::new();

        // Fill window with packets
        for _ in 0..5 {
            monitor.record_packets(100).await;
            sleep(Duration::from_secs(1)).await;
        }

        let tp1 = monitor.current_throughput().await;
        assert!(tp1.packets_per_second > 0.0);

        // Verify window is sliding (old buckets dropped)
        let state = monitor.state.read();
        assert!(state.buckets.len() <= state.max_buckets);
    }

    #[tokio::test]
    async fn test_throughput_calculation() {
        let monitor = ThroughputMonitor::new();

        // Record 1000 packets
        monitor.record_packets(1000).await;

        let tp = monitor.current_throughput().await;

        // Over 60-second window: 1000 / 60 ≈ 16.67 pps
        assert!(tp.packets_per_second >= 15.0 && tp.packets_per_second <= 18.0);
    }

    #[tokio::test]
    async fn test_bandwidth_calculation() {
        let monitor = ThroughputMonitor::new();

        // 1MB of data
        monitor.record_bytes(1_000_000).await;

        let tp = monitor.current_throughput().await;

        // (1,000,000 bytes × 8 bits) / (60 seconds × 1,000,000) = 0.133 Mbps
        assert!(tp.bandwidth_mbps > 0.1 && tp.bandwidth_mbps < 0.2);
    }

    #[tokio::test]
    async fn test_hosts_per_minute() {
        let monitor = ThroughputMonitor::new();

        // 10 hosts completed
        for _ in 0..10 {
            monitor.record_host_completed().await;
        }

        let tp = monitor.current_throughput().await;

        // 10 hosts / 60 seconds = 0.167 hosts/sec × 60 = 10 hpm
        assert!(tp.hosts_per_minute >= 9.0 && tp.hosts_per_minute <= 11.0);
    }

    #[tokio::test]
    async fn test_reset() {
        let monitor = ThroughputMonitor::new();

        monitor.record_packets(1000).await;
        monitor.record_bytes(100_000).await;
        monitor.record_host_completed().await;

        monitor.reset().await;

        let tp = monitor.current_throughput().await;
        assert_eq!(tp.packets_per_second, 0.0);
        assert_eq!(tp.hosts_per_minute, 0.0);
        assert_eq!(tp.bandwidth_mbps, 0.0);
    }

    #[tokio::test]
    async fn test_concurrent_recording() {
        use std::sync::Arc as StdArc;
        let monitor = StdArc::new(ThroughputMonitor::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let monitor_clone = StdArc::clone(&monitor);
            let handle = tokio::spawn(async move {
                for _ in 0..100 {
                    monitor_clone.record_packets(1).await;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let tp = monitor.current_throughput().await;
        // Should have recorded 1000 packets total
        assert!(tp.packets_per_second > 0.0);
    }
}
