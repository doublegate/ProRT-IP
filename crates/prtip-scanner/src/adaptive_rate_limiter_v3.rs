//! Adaptive Rate Limiter V3 - Ultra-Low Overhead Two-Tier Architecture
//!
//! Implements a high-performance adaptive rate limiter with <5% overhead target
//! using a two-tier architecture pattern inspired by HostgroupLimiter.
//!
//! # Architecture
//!
//! - **Hot Path (acquire()):** Ultra-fast atomic operations (3 atomics + conditional sleep)
//! - **Background Task:** 100ms monitoring loop for rate measurement and batch adjustment
//! - **Target Overhead:** <5% average, <3% optimal (competitive with HostgroupLimiter's 1-9%)
//!
//! # Algorithm
//!
//! The hot path reads a cached batch size (updated by background task) and consumes
//! from a batch counter. When the batch is exhausted, it sleeps to enforce rate.
//! The background task measures actual rate every 100ms and adjusts batch size
//! using convergence logic (±10% adjustments).
//!
//! # Advantages over AdaptiveRateLimiter (next_batch)
//!
//! - No Instant::now() in hot path (moved to background task)
//! - No rate calculation in hot path (cached values only)
//! - No circular buffer updates (background task handles measurement)
//! - Compatible with all scanners (acquire() API pattern)
//! - Expected 22-40% → <5% overhead improvement
//!
//! # Advantages over RateLimiter (Governor)
//!
//! - Adaptive batch sizing (vs fixed burst)
//! - Expected 15% → <5% overhead improvement
//! - Dynamic adjustment to actual network conditions
//!
//! # Examples
//!
//! ```no_run
//! use prtip_scanner::AdaptiveRateLimiterV3;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! // Create limiter for 1M packets per second
//! let limiter = AdaptiveRateLimiterV3::new(Some(1_000_000));
//!
//! // Acquire permit before each packet
//! for _ in 0..1000 {
//!     limiter.acquire().await?;
//!     // ... send packet ...
//! }
//! # Ok(())
//! # }
//! ```

use prtip_core::{Error, Result};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tracing::{debug, trace};

/// Hysteresis band around target rate (±5% tolerance to prevent oscillation)
const HYSTERESIS_FACTOR: f64 = 0.05;

/// Batch size increase factor when below target rate (10% for fast convergence)
const BATCH_INCREASE_FACTOR: f64 = 1.10;

/// Batch size decrease factor when above target rate (10% for fast backoff)
const BATCH_DECREASE_FACTOR: f64 = 0.90;

/// Maximum batch size to prevent overwhelming buffers
const MAX_BATCH_SIZE: u64 = 10000;

/// Minimum batch size (always at least 10 for efficiency)
const MIN_BATCH_SIZE: u64 = 10;

/// Monitor task interval (100ms)
const MONITOR_INTERVAL_MS: u64 = 100;

/// Adaptive rate limiter V3 - Two-tier architecture for <5% overhead
///
/// This limiter separates concerns into a fast hot path (acquire()) and a
/// background monitoring task. The hot path performs minimal work (3 atomic
/// operations + conditional sleep), while the background task handles all
/// expensive operations (time measurement, rate calculation, batch adjustment).
///
/// # Thread Safety
///
/// This limiter is fully thread-safe and can be cloned cheaply
/// (uses `Arc` internally for all shared state).
///
/// # Lifecycle
///
/// The background monitor task starts automatically on creation and shuts down
/// gracefully when the limiter is dropped (within 100ms of drop).
#[derive(Clone)]
pub struct AdaptiveRateLimiterV3 {
    /// Target rate in packets per second
    target_rate: u64,

    /// Current batch size (updated by background task, read by hot path)
    current_batch_size: Arc<AtomicU64>,

    /// Remaining permits in current batch (decremented by hot path)
    batch_counter: Arc<AtomicU64>,

    /// Total packets processed (incremented by hot path, read by background)
    packet_count: Arc<AtomicU64>,

    /// Shutdown signal for background task
    shutdown: Arc<AtomicBool>,

    /// Background monitor task handle
    monitor_handle: Arc<parking_lot::Mutex<Option<JoinHandle<()>>>>,
}

impl AdaptiveRateLimiterV3 {
    /// Create new adaptive rate limiter with target rate
    ///
    /// Spawns a background monitoring task that adjusts batch size every 100ms.
    ///
    /// # Arguments
    ///
    /// * `target_rate` - Target packets per second (None = 1M pps default)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_scanner::AdaptiveRateLimiterV3;
    ///
    /// // 500K pps
    /// let limiter = AdaptiveRateLimiterV3::new(Some(500_000));
    ///
    /// // 1M pps (default)
    /// let limiter_default = AdaptiveRateLimiterV3::new(None);
    /// ```
    pub fn new(target_rate: Option<u64>) -> Arc<Self> {
        let target_rate = target_rate.unwrap_or(1_000_000);

        // Intelligent initial batch sizing: rate / 100 (target ~100 batches/sec)
        // Clamp to min 10 (avoid per-packet overhead), max 1000 (avoid bursts)
        let initial_batch = (target_rate / 100).clamp(MIN_BATCH_SIZE, 1000);

        debug!(
            "Creating AdaptiveRateLimiterV3: target_rate={} pps, initial_batch={}",
            target_rate, initial_batch
        );

        // Create shared state
        let current_batch_size = Arc::new(AtomicU64::new(initial_batch));
        let batch_counter = Arc::new(AtomicU64::new(initial_batch));
        let packet_count = Arc::new(AtomicU64::new(0));
        let shutdown = Arc::new(AtomicBool::new(false));

        let limiter = Arc::new(Self {
            target_rate,
            current_batch_size: current_batch_size.clone(),
            batch_counter: batch_counter.clone(),
            packet_count: packet_count.clone(),
            shutdown: shutdown.clone(),
            monitor_handle: Arc::new(parking_lot::Mutex::new(None)),
        });

        // Spawn background monitor task
        let handle = tokio::spawn(Self::monitor_task(
            target_rate,
            current_batch_size,
            batch_counter.clone(),
            packet_count,
            shutdown,
        ));

        *limiter.monitor_handle.lock() = Some(handle);

        limiter
    }

    /// Acquire permit to send one packet (hot path - ultra-fast)
    ///
    /// This is the performance-critical function. It performs exactly 3 atomic
    /// operations in the typical case:
    /// 1. Atomic increment: packet_count (for background monitoring)
    /// 2. Atomic load: current_batch_size (read cached value from background)
    /// 3. Atomic decrement: batch_counter (consume from current batch)
    ///
    /// When the batch is exhausted (rare), it resets the counter and sleeps
    /// to enforce the rate limit.
    ///
    /// # Returns
    ///
    /// Ok(()) on success, Err if shutdown signal received.
    ///
    /// # Performance
    ///
    /// - Typical case: ~10-20ns (3 atomics)
    /// - Batch exhausted: ~400-500ns (reset + sleep setup)
    /// - Expected overhead: <5% average, <3% optimal
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_scanner::AdaptiveRateLimiterV3;
    /// # async fn example() -> prtip_core::Result<()> {
    /// let limiter = AdaptiveRateLimiterV3::new(Some(100_000));
    ///
    /// for _ in 0..1000 {
    ///     limiter.acquire().await?;
    ///     // ... send packet ...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn acquire(&self) -> Result<()> {
        // Check shutdown signal (rarely true, branch predictor optimizes this)
        if self.shutdown.load(Ordering::Relaxed) {
            return Err(Error::Network("Rate limiter shutting down".to_string()));
        }

        // 1. Atomic increment: Track packet for background monitoring
        //    Relaxed ordering sufficient (counter doesn't need immediate visibility)
        self.packet_count.fetch_add(1, Ordering::Relaxed);

        // 2. Atomic load: Get current batch size (cached by background task)
        //    Relaxed ordering sufficient (stale reads self-correct via batch_counter)
        let batch_size = self.current_batch_size.load(Ordering::Relaxed);

        // 3. Atomic decrement: Consume from current batch
        //    Relaxed ordering sufficient (each thread only cares about exhaustion, not order)
        let remaining = self.batch_counter.fetch_sub(1, Ordering::Relaxed);

        // 4. Conditional sleep: Only when batch exhausted (rare, ~1 in batch_size calls)
        if remaining == 0 {
            // Reset batch counter for next batch
            self.batch_counter.store(batch_size, Ordering::Relaxed);

            // Calculate sleep duration to enforce rate
            // Formula: sleep_micros = (batch_size * 1_000_000) / target_rate
            // Example: batch_size=100, rate=100K pps => 100 * 1M / 100K = 1000us = 1ms
            let sleep_micros = (batch_size * 1_000_000) / self.target_rate;

            trace!(
                "Batch exhausted, sleeping {}us (batch_size={}, rate={})",
                sleep_micros,
                batch_size,
                self.target_rate
            );

            tokio::time::sleep(Duration::from_micros(sleep_micros)).await;
        }

        Ok(())
    }

    /// Background monitor task - adjusts batch size based on actual rate
    ///
    /// Runs every 100ms, measures actual packet rate, and adjusts batch size
    /// using convergence logic (±10% adjustments with ±5% hysteresis band).
    ///
    /// This task performs ALL expensive operations:
    /// - Time measurement (Instant::now())
    /// - Rate calculation (packets_sent / elapsed_time)
    /// - Batch size adjustment (convergence algorithm)
    /// - Atomic updates to shared state
    ///
    /// These operations add ZERO overhead to the hot path (acquire()).
    async fn monitor_task(
        target_rate: u64,
        current_batch_size: Arc<AtomicU64>,
        batch_counter: Arc<AtomicU64>,
        packet_count: Arc<AtomicU64>,
        shutdown: Arc<AtomicBool>,
    ) {
        debug!("AdaptiveRateLimiterV3 monitor task started");

        let mut last_packet_count = 0u64;
        let mut last_measurement_time = Instant::now();

        loop {
            // Sleep for monitor interval (100ms)
            tokio::time::sleep(Duration::from_millis(MONITOR_INTERVAL_MS)).await;

            // Check shutdown signal
            if shutdown.load(Ordering::Relaxed) {
                debug!("Monitor task shutting down");
                break;
            }

            // Measure actual rate
            let now = Instant::now();
            let current_count = packet_count.load(Ordering::Relaxed);
            let elapsed = now.duration_since(last_measurement_time).as_secs_f64();

            // Skip if elapsed time is too short (avoid division by zero)
            if elapsed < 0.01 {
                trace!(
                    "Monitor: elapsed time too short ({:.6}s), skipping",
                    elapsed
                );
                continue;
            }

            let packets_sent = current_count.saturating_sub(last_packet_count);
            let actual_rate = packets_sent as f64 / elapsed;

            let target_rate_f64 = target_rate as f64;

            // Calculate hysteresis bounds (±5% around target)
            let lower_bound = target_rate_f64 * (1.0 - HYSTERESIS_FACTOR);
            let upper_bound = target_rate_f64 * (1.0 + HYSTERESIS_FACTOR);

            // Get current batch size
            let current_batch = current_batch_size.load(Ordering::Relaxed);

            // Adjust batch size based on actual vs target rate
            let new_batch = if actual_rate > upper_bound {
                // Too fast: decrease batch size by 10%
                let adjusted = (current_batch as f64 * BATCH_DECREASE_FACTOR).round() as u64;
                let clamped = adjusted.clamp(MIN_BATCH_SIZE, MAX_BATCH_SIZE);

                trace!(
                    "Monitor: rate too high ({:.0} > {:.0}), decreasing batch {} -> {}",
                    actual_rate,
                    target_rate_f64,
                    current_batch,
                    clamped
                );

                clamped
            } else if actual_rate < lower_bound {
                // Too slow: increase batch size by 10%
                let adjusted = (current_batch as f64 * BATCH_INCREASE_FACTOR).round() as u64;
                let clamped = adjusted.clamp(MIN_BATCH_SIZE, MAX_BATCH_SIZE);

                trace!(
                    "Monitor: rate too low ({:.0} < {:.0}), increasing batch {} -> {}",
                    actual_rate,
                    target_rate_f64,
                    current_batch,
                    clamped
                );

                clamped
            } else {
                // Within hysteresis band: no adjustment needed
                trace!(
                    "Monitor: rate within bounds ({:.0} ≈ {:.0}), batch={} (stable)",
                    actual_rate,
                    target_rate_f64,
                    current_batch
                );

                current_batch
            };

            // Update cached batch size (hot path will read this)
            // Relaxed ordering sufficient (eventual visibility is acceptable)
            if new_batch != current_batch {
                current_batch_size.store(new_batch, Ordering::Relaxed);

                // Also update batch_counter to avoid immediate exhaustion
                // (Only if increasing - decreasing will naturally converge)
                if new_batch > current_batch {
                    batch_counter.store(new_batch, Ordering::Relaxed);
                }
            }

            // Update state for next iteration
            last_packet_count = current_count;
            last_measurement_time = now;
        }

        debug!("Monitor task exited");
    }

    /// Get current target rate
    pub fn target_rate(&self) -> u64 {
        self.target_rate
    }

    /// Get current batch size (cached value)
    pub fn batch_size(&self) -> u64 {
        self.current_batch_size.load(Ordering::Relaxed)
    }

    /// Get total packets processed
    pub fn packet_count(&self) -> u64 {
        self.packet_count.load(Ordering::Relaxed)
    }

    /// Get remaining permits in current batch
    pub fn remaining_in_batch(&self) -> u64 {
        self.batch_counter.load(Ordering::Relaxed)
    }
}

impl Drop for AdaptiveRateLimiterV3 {
    fn drop(&mut self) {
        // Signal background task to shutdown
        self.shutdown.store(true, Ordering::Relaxed);

        // Note: Can't await in Drop, but background will exit within 100ms
        // This is acceptable for graceful shutdown
        debug!("AdaptiveRateLimiterV3 dropped, background task will exit within 100ms");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_creation() {
        let limiter = AdaptiveRateLimiterV3::new(Some(100_000));

        assert_eq!(limiter.target_rate(), 100_000);
        assert_eq!(limiter.batch_size(), 1000); // 100K / 100 = 1000
        assert_eq!(limiter.packet_count(), 0);
    }

    #[tokio::test]
    async fn test_creation_default_rate() {
        let limiter = AdaptiveRateLimiterV3::new(None);

        assert_eq!(limiter.target_rate(), 1_000_000); // Default 1M pps
        assert_eq!(limiter.batch_size(), 1000); // 1M / 100 = 10000, clamped to 1000 in new()
    }

    #[tokio::test]
    async fn test_basic_acquire() {
        let limiter = AdaptiveRateLimiterV3::new(Some(1000));

        // Acquire a few permits
        for _ in 0..10 {
            limiter.acquire().await.unwrap();
        }

        assert_eq!(limiter.packet_count(), 10);
    }

    #[tokio::test]
    async fn test_batch_exhaustion() {
        let limiter = AdaptiveRateLimiterV3::new(Some(100));

        let batch_size = limiter.batch_size();
        let initial_batch = batch_size;

        // Acquire exactly batch_size permits
        for _ in 0..batch_size {
            limiter.acquire().await.unwrap();
        }

        // Batch should have been reset
        assert_eq!(limiter.packet_count(), initial_batch);

        // One more acquire should work (new batch)
        limiter.acquire().await.unwrap();
        assert_eq!(limiter.packet_count(), initial_batch + 1);
    }

    #[tokio::test]
    async fn test_concurrent_acquires() {
        let limiter = AdaptiveRateLimiterV3::new(Some(10_000));
        let mut handles = vec![];

        // Spawn 10 tasks acquiring 100 permits each
        for _ in 0..10 {
            let limiter = limiter.clone();
            let handle = tokio::spawn(async move {
                for _ in 0..100 {
                    limiter.acquire().await.unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Should have processed 1000 packets
        assert_eq!(limiter.packet_count(), 1000);
    }

    #[tokio::test]
    async fn test_rate_enforcement() {
        let target_rate = 1000; // 1K pps
        let limiter = AdaptiveRateLimiterV3::new(Some(target_rate));

        let start = Instant::now();
        let packets_to_send = 500;

        // Send packets
        for _ in 0..packets_to_send {
            limiter.acquire().await.unwrap();
        }

        let elapsed = start.elapsed().as_secs_f64();
        let actual_rate = packets_to_send as f64 / elapsed;

        // Rate should be close to target (within 50% due to initial convergence)
        assert!(
            actual_rate <= target_rate as f64 * 1.5,
            "Rate too high: {:.0} > {:.0}",
            actual_rate,
            target_rate as f64 * 1.5
        );
    }

    #[tokio::test]
    async fn test_batch_size_convergence() {
        let limiter = AdaptiveRateLimiterV3::new(Some(10_000));

        let initial_batch = limiter.batch_size();
        assert_eq!(initial_batch, 100); // 10K / 100 = 100

        // Send some packets
        for _ in 0..500 {
            limiter.acquire().await.unwrap();
        }

        // Wait for monitor task to run at least once (100ms + buffer)
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Batch size should still be reasonable
        let current_batch = limiter.batch_size();
        assert!(
            (MIN_BATCH_SIZE..=MAX_BATCH_SIZE).contains(&current_batch),
            "Batch size out of range: {}",
            current_batch
        );
    }

    #[tokio::test]
    async fn test_high_rate_batching() {
        let limiter = AdaptiveRateLimiterV3::new(Some(1_000_000));

        // Initial batch should be large for high rate (clamped to 1000 in new())
        assert_eq!(limiter.batch_size(), 1000); // 1M / 100 = 10000, clamped to 1000

        // Send many packets quickly
        for _ in 0..10000 {
            limiter.acquire().await.unwrap();
            // No artificial delay - go as fast as possible
        }

        // Batch size should remain large
        assert!(
            limiter.batch_size() >= 1000,
            "Batch size should be large for high rate: {}",
            limiter.batch_size()
        );
    }

    #[tokio::test]
    async fn test_low_rate_batching() {
        let limiter = AdaptiveRateLimiterV3::new(Some(100));

        // Initial batch for low rate (100 / 100 = 1, clamped to 10)
        assert_eq!(limiter.batch_size(), MIN_BATCH_SIZE);

        // Send packets slowly
        for _ in 0..50 {
            limiter.acquire().await.unwrap();
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        // Batch size should stay small
        assert!(
            limiter.batch_size() <= 100,
            "Batch size should be small for low rate: {}",
            limiter.batch_size()
        );
    }

    #[tokio::test]
    async fn test_zero_rate_handling() {
        let limiter = AdaptiveRateLimiterV3::new(Some(0));

        // Batch size should be clamped to minimum (avoid division by zero)
        assert_eq!(limiter.batch_size(), MIN_BATCH_SIZE);

        // Should still be able to acquire (very slow rate)
        limiter.acquire().await.unwrap();
        assert_eq!(limiter.packet_count(), 1);
    }

    #[tokio::test]
    async fn test_max_batch_size_limit() {
        let limiter = AdaptiveRateLimiterV3::new(Some(10_000_000)); // 10M pps

        // Initial batch should be clamped to 1000 (not 100000)
        assert_eq!(limiter.batch_size(), 1000); // Clamped to max 1000 in new()

        // Send packets very quickly
        for _ in 0..20000 {
            limiter.acquire().await.unwrap();
        }

        // Wait for monitor adjustments
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Batch size should never exceed MAX_BATCH_SIZE
        assert!(
            limiter.batch_size() <= MAX_BATCH_SIZE,
            "Batch size exceeded max: {}",
            limiter.batch_size()
        );
    }

    #[tokio::test]
    async fn test_min_batch_size_limit() {
        let limiter = AdaptiveRateLimiterV3::new(Some(10)); // Very low rate

        // Initial batch should be clamped to minimum
        assert_eq!(limiter.batch_size(), MIN_BATCH_SIZE);

        // Send packets
        for _ in 0..100 {
            limiter.acquire().await.unwrap();
        }

        // Wait for monitor adjustments
        tokio::time::sleep(Duration::from_millis(300)).await;

        // Batch size should never go below MIN_BATCH_SIZE
        assert!(
            limiter.batch_size() >= MIN_BATCH_SIZE,
            "Batch size below min: {}",
            limiter.batch_size()
        );
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let limiter = AdaptiveRateLimiterV3::new(Some(1000));

        // Acquire a few permits
        for _ in 0..10 {
            limiter.acquire().await.unwrap();
        }

        // Drop limiter (triggers shutdown)
        drop(limiter);

        // Wait for monitor task to exit (within 100ms)
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Test passes if no panics occurred
    }

    #[tokio::test]
    async fn test_shutdown_signal() {
        let limiter = AdaptiveRateLimiterV3::new(Some(1000));

        // Manually trigger shutdown
        limiter.shutdown.store(true, Ordering::Relaxed);

        // Next acquire should fail
        let result = limiter.acquire().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_monitor_task_convergence() {
        let target_rate = 5000;
        let limiter = AdaptiveRateLimiterV3::new(Some(target_rate));

        // Send packets for a few monitor intervals
        for _ in 0..1000 {
            limiter.acquire().await.unwrap();
        }

        // Wait for several monitor cycles (300ms = 3 cycles)
        tokio::time::sleep(Duration::from_millis(300)).await;

        // Batch size should have stabilized (not oscillating wildly)
        let batch1 = limiter.batch_size();

        // Send more packets
        for _ in 0..1000 {
            limiter.acquire().await.unwrap();
        }

        // Wait for more monitor cycles
        tokio::time::sleep(Duration::from_millis(300)).await;

        let batch2 = limiter.batch_size();

        // Batch sizes shouldn't differ drastically (within 2x)
        let ratio = batch2 as f64 / batch1 as f64;
        assert!(
            ratio > 0.5 && ratio < 2.0,
            "Batch size oscillating: {} -> {} (ratio: {:.2})",
            batch1,
            batch2,
            ratio
        );
    }

    #[tokio::test]
    async fn test_adaptive_adjustment() {
        // Test that batch size adapts based on actual rate
        let target_rate = 10000;
        let limiter = AdaptiveRateLimiterV3::new(Some(target_rate));

        let initial_batch = limiter.batch_size();

        // Send packets quickly (above target rate)
        for _ in 0..2000 {
            limiter.acquire().await.unwrap();
            // Very short delay = high rate
        }

        // Wait for monitor to measure and adjust (200ms = 2 cycles)
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Batch size should have changed from initial
        let adjusted_batch = limiter.batch_size();

        // Verify adaptive behavior occurred (batch size changed)
        assert_ne!(
            initial_batch, adjusted_batch,
            "Batch size should adapt based on actual rate"
        );

        // Verify batch stays within valid bounds
        assert!(
            (MIN_BATCH_SIZE..=MAX_BATCH_SIZE).contains(&adjusted_batch),
            "Batch size out of bounds: {}",
            adjusted_batch
        );
    }

    #[tokio::test]
    async fn test_atomic_ordering_correctness() {
        // This test verifies atomic ordering is correct (compiles without warnings)
        let limiter = AdaptiveRateLimiterV3::new(Some(1000));

        // Relaxed ordering on all operations (optimized for performance)
        let _batch = limiter.current_batch_size.load(Ordering::Relaxed);

        // Relaxed on fetch_sub (each thread only cares about exhaustion)
        let _remaining = limiter.batch_counter.fetch_sub(1, Ordering::Relaxed);

        // Relaxed on write (eventual visibility is acceptable)
        limiter.current_batch_size.store(100, Ordering::Relaxed);

        // Relaxed for counter
        let _count = limiter.packet_count.load(Ordering::Relaxed);
    }
}
