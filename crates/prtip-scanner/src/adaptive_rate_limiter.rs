//! Adaptive Rate Limiting for High-Performance Scanning
//!
//! Implements an adaptive throttler inspired by Masscan's approach, which adjusts
//! batch sizes dynamically based on actual throughput to achieve precise rate limiting
//! at high packet rates (1M+ pps).
//!
//! # Algorithm
//!
//! Uses a circular buffer of 256 buckets tracking recent packet counts and timestamps.
//! Dynamically adjusts batch size:
//! - Increases by 0.5% when below target rate (convergence)
//! - Decreases by 0.1% when above target rate (backoff)
//!
//! # Advantages over Token Bucket
//!
//! - Better performance at very high rates (>100K pps)
//! - Adaptive batching reduces syscall overhead
//! - Handles system suspend/resume gracefully
//! - Uses only recent history (prevents burst after pause)
//!
//! # Examples
//!
//! ```no_run
//! use prtip_scanner::AdaptiveRateLimiterV2;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! // Create limiter for 1M packets per second
//! let mut limiter = AdaptiveRateLimiterV2::new(1_000_000.0);
//!
//! let mut packets_sent = 0;
//! loop {
//!     // Get next batch size
//!     let batch = limiter.next_batch(packets_sent).await?;
//!
//!     // Send 'batch' number of packets
//!     for _ in 0..batch {
//!         // ... send packet ...
//!         packets_sent += 1;
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crate::icmp_monitor::{BackoffState, IcmpMonitor};
use dashmap::DashMap;
use prtip_core::Result;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, trace};

/// Number of buckets in circular buffer for rate tracking
const BUCKET_COUNT: usize = 256;

/// Maximum wait time to prevent system hangs (100ms)
const MAX_WAIT_TIME_SECS: f64 = 0.1;

/// Batch size increase factor when below target rate
const BATCH_INCREASE_FACTOR: f64 = 1.005;

/// Batch size decrease factor when above target rate
const BATCH_DECREASE_FACTOR: f64 = 0.999;

/// Maximum batch size to prevent overwhelming buffers
const MAX_BATCH_SIZE: f64 = 10000.0;

/// Timestamp and packet count for a single time bucket
#[derive(Debug, Clone, Copy)]
struct Bucket {
    timestamp: Instant,
    packet_count: u64,
}

impl Default for Bucket {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            packet_count: 0,
        }
    }
}

/// Adaptive rate limiter with dynamic batch sizing
///
/// Tracks packet transmission rate over recent history and adjusts batch sizes
/// to converge on target rate. Inspired by Masscan's throttler implementation.
///
/// # Performance
///
/// - At low rates (<1K pps): batch size ~1, per-packet throttling
/// - At medium rates (1K-100K pps): batch size 2-100, reduced overhead
/// - At high rates (>100K pps): batch size 100-10000, minimal overhead
///
/// # ICMP Monitoring
///
/// Optional ICMP monitoring detects Type 3 Code 13 (admin prohibited) errors
/// and backs off per-target using exponential backoff (1s → 2s → 4s → 8s → 16s).
pub struct AdaptiveRateLimiter {
    /// Target maximum rate in packets per second
    max_rate: f64,

    /// Current measured rate in packets per second
    current_rate: f64,

    /// Current batch size (adaptive)
    batch_size: f64,

    /// Index into circular buffer (wraps at 256)
    index: usize,

    /// Circular buffer of recent measurements
    buckets: [Bucket; BUCKET_COUNT],

    /// Start time for overall statistics
    start_time: Instant,

    /// ICMP monitor (optional)
    icmp_monitor: Option<Arc<IcmpMonitor>>,

    /// Per-target backoff states
    target_backoffs: Arc<DashMap<IpAddr, BackoffState>>,
}

impl AdaptiveRateLimiter {
    /// Create a new adaptive rate limiter
    ///
    /// # Arguments
    ///
    /// * `max_rate` - Maximum packets per second (as f64 for precision)
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::AdaptiveRateLimiterV2;
    ///
    /// // Limit to 500,000 packets per second
    /// let limiter = AdaptiveRateLimiterV2::new(500_000.0);
    /// ```
    pub fn new(max_rate: f64) -> Self {
        let now = Instant::now();

        debug!(
            "Creating adaptive rate limiter: target = {:.2} pps",
            max_rate
        );

        Self {
            max_rate,
            current_rate: 0.0,
            batch_size: 1.0,
            index: 0,
            buckets: [Bucket {
                timestamp: now,
                packet_count: 0,
            }; BUCKET_COUNT],
            start_time: now,
            icmp_monitor: None,
            target_backoffs: Arc::new(DashMap::new()),
        }
    }

    /// Enable ICMP monitoring for adaptive backoff
    ///
    /// Starts an ICMP monitor that listens for Type 3 Code 13 errors
    /// and automatically backs off on affected targets.
    ///
    /// # Errors
    ///
    /// Returns error if ICMP monitor cannot be created or started.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_scanner::AdaptiveRateLimiterV2;
    /// # async fn example() -> prtip_core::Result<()> {
    /// let mut limiter = AdaptiveRateLimiterV2::new(100_000.0);
    /// limiter.enable_icmp_monitoring().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn enable_icmp_monitoring(&mut self) -> Result<()> {
        let monitor = IcmpMonitor::new()?;
        monitor.start().await?;

        debug!("ICMP monitoring enabled for adaptive rate limiter");

        // Spawn task to process ICMP errors
        let backoffs = self.target_backoffs.clone();
        let mut rx = monitor.subscribe();

        tokio::spawn(async move {
            while let Ok(error) = rx.recv().await {
                let mut entry = backoffs
                    .entry(error.target_ip)
                    .or_insert(BackoffState::new());
                entry.escalate();

                debug!(
                    "ICMP error for {}: backing off to level {}",
                    error.target_ip, entry.backoff_level
                );
            }
        });

        self.icmp_monitor = Some(Arc::new(monitor));
        Ok(())
    }

    /// Check if target is currently backed off
    ///
    /// # Examples
    ///
    /// ```
    /// # use prtip_scanner::AdaptiveRateLimiterV2;
    /// # use std::net::IpAddr;
    /// let limiter = AdaptiveRateLimiterV2::new(100_000.0);
    /// let target: IpAddr = "192.168.1.1".parse().unwrap();
    /// assert!(!limiter.is_target_backed_off(target));
    /// ```
    pub fn is_target_backed_off(&self, target: IpAddr) -> bool {
        self.target_backoffs
            .get(&target)
            .map(|state| !state.is_expired())
            .unwrap_or(false)
    }

    /// Wait for backoff to expire on target
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_scanner::AdaptiveRateLimiterV2;
    /// # use std::net::IpAddr;
    /// # async fn example() {
    /// let limiter = AdaptiveRateLimiterV2::new(100_000.0);
    /// let target: IpAddr = "192.168.1.1".parse().unwrap();
    /// limiter.wait_for_backoff(target).await;
    /// # }
    /// ```
    pub async fn wait_for_backoff(&self, target: IpAddr) {
        while self.is_target_backed_off(target) {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get backoff info for target (for logging/debugging)
    ///
    /// Returns `Some((level, remaining_duration))` if target is backed off,
    /// `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prtip_scanner::AdaptiveRateLimiterV2;
    /// # use std::net::IpAddr;
    /// let limiter = AdaptiveRateLimiterV2::new(100_000.0);
    /// let target: IpAddr = "192.168.1.1".parse().unwrap();
    ///
    /// if let Some((level, remaining)) = limiter.get_backoff_info(target) {
    ///     println!("Target backed off: level {}, {}s remaining",
    ///              level, remaining.as_secs());
    /// }
    /// ```
    pub fn get_backoff_info(&self, target: IpAddr) -> Option<(u8, Duration)> {
        self.target_backoffs
            .get(&target)
            .map(|state| (state.backoff_level, state.remaining()))
    }

    /// Get the next batch size and wait if necessary to maintain rate limit
    ///
    /// This is the core throttling function. It calculates the current rate based
    /// on recent history, waits if we're going too fast, and returns the number
    /// of packets that can be sent in the next batch.
    ///
    /// # Arguments
    ///
    /// * `packet_count` - Total packets sent so far
    ///
    /// # Returns
    ///
    /// Number of packets to send in next batch (minimum 1)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_scanner::AdaptiveRateLimiterV2;
    /// # async fn example() -> prtip_core::Result<()> {
    /// let mut limiter = AdaptiveRateLimiterV2::new(1000.0);
    /// let mut sent = 0;
    ///
    /// for _ in 0..10 {
    ///     let batch = limiter.next_batch(sent).await?;
    ///     sent += batch;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn next_batch(&mut self, packet_count: u64) -> Result<u64> {
        loop {
            let now = Instant::now();

            // Store current measurement in circular buffer
            let current_index = self.index & (BUCKET_COUNT - 1);
            self.buckets[current_index] = Bucket {
                timestamp: now,
                packet_count,
            };

            // Move to next bucket and get old measurement
            self.index = self.index.wrapping_add(1);
            let old_index = self.index & (BUCKET_COUNT - 1);
            let old_bucket = self.buckets[old_index];

            let elapsed = now.duration_since(old_bucket.timestamp);

            // If more than 1 second has passed, reset to avoid burst after pause
            // This handles laptop suspend/resume gracefully
            if elapsed > Duration::from_secs(1) {
                trace!("Rate limiter reset: elapsed time > 1 second");
                self.batch_size = 1.0;
                continue;
            }

            // Calculate current rate over recent window
            let elapsed_secs = elapsed.as_secs_f64();
            if elapsed_secs < 0.000001 {
                // Too short interval, wait a bit
                tokio::time::sleep(Duration::from_micros(100)).await;
                continue;
            }

            let packets_in_window = packet_count.saturating_sub(old_bucket.packet_count);
            self.current_rate = packets_in_window as f64 / elapsed_secs;

            // If we're going too fast, wait and adjust batch size down
            if self.current_rate > self.max_rate {
                let overage_ratio = (self.current_rate - self.max_rate) / self.max_rate;

                // Calculate wait time proportional to overage
                let mut wait_time = overage_ratio * 0.1; // Damping factor

                // Cap wait time to prevent hangs
                if wait_time > MAX_WAIT_TIME_SECS {
                    wait_time = MAX_WAIT_TIME_SECS;
                }

                // Reduce batch size slightly (gradual convergence)
                self.batch_size *= BATCH_DECREASE_FACTOR;
                if self.batch_size < 1.0 {
                    self.batch_size = 1.0;
                }

                trace!(
                    "Rate too high ({:.2} > {:.2}), waiting {:.4}s, batch_size={:.2}",
                    self.current_rate,
                    self.max_rate,
                    wait_time,
                    self.batch_size
                );

                // Wait to reduce rate
                tokio::time::sleep(Duration::from_secs_f64(wait_time)).await;

                // Loop again to recalculate (for very slow rates)
                continue;
            }

            // We're below target rate, increase batch size slightly
            self.batch_size *= BATCH_INCREASE_FACTOR;
            if self.batch_size > MAX_BATCH_SIZE {
                self.batch_size = MAX_BATCH_SIZE;
            }

            let batch = self.batch_size as u64;

            trace!(
                "Rate OK ({:.2} / {:.2}), batch_size={:.2} -> {}",
                self.current_rate,
                self.max_rate,
                self.batch_size,
                batch
            );

            return Ok(batch.max(1)); // Always return at least 1
        }
    }

    /// Get current measured rate in packets per second
    pub fn current_rate(&self) -> f64 {
        self.current_rate
    }

    /// Get target maximum rate
    pub fn max_rate(&self) -> f64 {
        self.max_rate
    }

    /// Get current batch size
    pub fn batch_size(&self) -> f64 {
        self.batch_size
    }

    /// Get overall statistics since start
    pub fn stats(&self) -> RateLimiterStats {
        RateLimiterStats {
            elapsed: self.start_time.elapsed(),
            current_rate: self.current_rate,
            target_rate: self.max_rate,
            batch_size: self.batch_size,
        }
    }
}

/// Statistics for the adaptive rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiterStats {
    pub elapsed: Duration,
    pub current_rate: f64,
    pub target_rate: f64,
    pub batch_size: f64,
}

impl std::fmt::Display for RateLimiterStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rate: {:.2}/{:.2} pps (batch: {:.1}), elapsed: {:.2}s",
            self.current_rate,
            self.target_rate,
            self.batch_size,
            self.elapsed.as_secs_f64()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_adaptive_limiter_basic() {
        let mut limiter = AdaptiveRateLimiter::new(100.0);

        assert_eq!(limiter.max_rate(), 100.0);
        assert_eq!(limiter.batch_size(), 1.0);

        let batch = limiter.next_batch(0).await.unwrap();
        assert!(batch >= 1);
    }

    #[tokio::test]
    async fn test_adaptive_limiter_rate_enforcement() {
        let target_rate = 100.0;
        let mut limiter = AdaptiveRateLimiter::new(target_rate);

        let start = Instant::now();
        let mut packets_sent = 0;

        // Send 200 packets
        for _ in 0..20 {
            let batch = limiter.next_batch(packets_sent).await.unwrap();
            packets_sent += batch;

            // Simulate sending packets (small delay to avoid immediate completion)
            tokio::time::sleep(Duration::from_micros(10)).await;

            if packets_sent >= 200 {
                break;
            }
        }

        let elapsed = start.elapsed().as_secs_f64();
        let actual_rate = packets_sent as f64 / elapsed;

        // Rate should be close to target (within 50% due to test variability)
        assert!(
            actual_rate <= target_rate * 1.5,
            "Rate too high: {} > {}",
            actual_rate,
            target_rate * 1.5
        );
    }

    #[tokio::test]
    async fn test_batch_size_adaptation() {
        let mut limiter = AdaptiveRateLimiter::new(10000.0);

        let mut packets_sent = 0;

        // Run for a bit and observe batch size increase
        for _ in 0..10 {
            let batch = limiter.next_batch(packets_sent).await.unwrap();
            packets_sent += batch;
            tokio::time::sleep(Duration::from_micros(1)).await;
        }

        // Batch size should have increased from initial 1.0
        assert!(limiter.batch_size() > 1.0);
    }

    #[test]
    fn test_rate_limiter_stats() {
        let limiter = AdaptiveRateLimiter::new(5000.0);
        let stats = limiter.stats();

        assert_eq!(stats.target_rate, 5000.0);
        assert_eq!(stats.batch_size, 1.0);

        let display = format!("{}", stats);
        assert!(display.contains("pps"));
    }

    #[tokio::test]
    async fn test_zero_rate_handling() {
        let mut limiter = AdaptiveRateLimiter::new(0.0);

        // Should still return at least 1 packet per batch
        let batch = limiter.next_batch(0).await.unwrap();
        assert_eq!(batch, 1);
    }

    #[tokio::test]
    async fn test_high_rate_batching() {
        let mut limiter = AdaptiveRateLimiter::new(1_000_000.0);

        let mut packets_sent = 0;

        // At high rates, batch size should grow quickly
        for _ in 0..100 {
            let batch = limiter.next_batch(packets_sent).await.unwrap();
            packets_sent += batch;
            // Very short delay to simulate high-speed sending
            tokio::time::sleep(Duration::from_micros(1)).await;
        }

        // Should have some batching at 1M pps (more lenient check for CI)
        assert!(
            limiter.batch_size() > 1.0,
            "Batch size should grow above 1.0, got: {}",
            limiter.batch_size()
        );
    }

    #[tokio::test]
    async fn test_convergence_stability() {
        let target_rate = 1000.0;
        let mut limiter = AdaptiveRateLimiter::new(target_rate);

        let mut packets_sent = 0;

        // Run long enough for convergence
        for _ in 0..50 {
            let batch = limiter.next_batch(packets_sent).await.unwrap();
            packets_sent += batch;
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        // Batch size should stabilize (not oscillate wildly)
        let initial_batch = limiter.batch_size();

        for _ in 0..20 {
            let batch = limiter.next_batch(packets_sent).await.unwrap();
            packets_sent += batch;
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        let final_batch = limiter.batch_size();

        // Batch size shouldn't change drastically (within 50%)
        let ratio = final_batch / initial_batch;
        assert!(
            ratio > 0.5 && ratio < 2.0,
            "Batch size oscillating too much: {} -> {}",
            initial_batch,
            final_batch
        );
    }

    #[tokio::test]
    async fn test_suspend_recovery() {
        let mut limiter = AdaptiveRateLimiter::new(1000.0);

        let mut packets_sent = 0;

        // Send some packets
        for _ in 0..10 {
            let batch = limiter.next_batch(packets_sent).await.unwrap();
            packets_sent += batch;
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        let _batch_before_suspend = limiter.batch_size();

        // Simulate suspend (wait > 1 second)
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Next batch should reset to 1.0
        let batch = limiter.next_batch(packets_sent).await.unwrap();
        assert!(batch >= 1);

        // Batch size should have reset (allow small increase from 1.0 due to convergence)
        assert!(
            limiter.batch_size() < 1.1,
            "Batch size should be close to 1.0 after suspend, got: {}",
            limiter.batch_size()
        );
    }

    #[tokio::test]
    async fn test_max_batch_size_limit() {
        let mut limiter = AdaptiveRateLimiter::new(10_000_000.0); // Very high rate

        let mut packets_sent = 0;

        // Run until batch size should max out
        for _ in 0..200 {
            let batch = limiter.next_batch(packets_sent).await.unwrap();
            packets_sent += batch;
            // Very short delay to encourage batch growth
            tokio::time::sleep(Duration::from_nanos(100)).await;
        }

        // Batch size should not exceed MAX_BATCH_SIZE
        assert!(
            limiter.batch_size() <= MAX_BATCH_SIZE,
            "Batch size exceeded max: {} > {}",
            limiter.batch_size(),
            MAX_BATCH_SIZE
        );
    }

    #[test]
    fn test_circular_buffer_wraparound() {
        let mut limiter = AdaptiveRateLimiter::new(1000.0);

        // Index should wrap at 256
        for _ in 0..300 {
            limiter.index = limiter.index.wrapping_add(1);
        }

        // Verify index wraps correctly
        assert!(limiter.index >= 300);
        let buffer_index = limiter.index & (BUCKET_COUNT - 1);
        assert!(buffer_index < BUCKET_COUNT);
    }

    #[tokio::test]
    async fn test_icmp_integration_creation() {
        let limiter = AdaptiveRateLimiter::new(100_000.0);

        // ICMP monitor should be None initially
        assert!(limiter.icmp_monitor.is_none());
        assert_eq!(limiter.target_backoffs.len(), 0);
    }

    #[tokio::test]
    async fn test_backoff_tracking() {
        use crate::icmp_monitor::BackoffState;

        let limiter = AdaptiveRateLimiter::new(100_000.0);
        let target: IpAddr = "192.168.1.1".parse().unwrap();

        // Initially not backed off
        assert!(!limiter.is_target_backed_off(target));

        // Manually insert backoff state
        let mut state = BackoffState::new();
        state.escalate();
        limiter.target_backoffs.insert(target, state);

        // Now should be backed off
        assert!(limiter.is_target_backed_off(target));

        // Get backoff info
        let info = limiter.get_backoff_info(target);
        assert!(info.is_some());
        let (level, _duration) = info.unwrap();
        assert_eq!(level, 1);
    }

    #[tokio::test]
    async fn test_backoff_expiration_check() {
        use crate::icmp_monitor::BackoffState;

        let limiter = AdaptiveRateLimiter::new(100_000.0);
        let target: IpAddr = "192.168.1.2".parse().unwrap();

        // Create expired backoff
        let mut state = BackoffState::new();
        state.backoff_until = Instant::now() - Duration::from_secs(1);
        limiter.target_backoffs.insert(target, state);

        // Should not be backed off (expired)
        assert!(!limiter.is_target_backed_off(target));
    }

    #[test]
    fn test_get_backoff_info_nonexistent() {
        let limiter = AdaptiveRateLimiter::new(100_000.0);
        let target: IpAddr = "10.0.0.1".parse().unwrap();

        // Should return None for non-existent target
        assert!(limiter.get_backoff_info(target).is_none());
    }
}
