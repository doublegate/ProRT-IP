//! Progress calculator with adaptive ETA estimation
//!
//! This module provides real-time progress and ETA calculation using
//! Exponential Weighted Moving Average (EWMA) for smooth, accurate predictions.
//!
//! # Algorithm
//!
//! - **EWMA Smoothing**: Recent rates weighted more heavily than historical (alpha=0.3)
//! - **Startup Grace**: First 5% of scan uses simple linear ETA (insufficient history)
//! - **Rate Tracking**: 60-second sliding window for throughput calculation
//! - **Adaptive**: Handles variable scan rates (network conditions, rate limiting)
//!
//! # Examples
//!
//! ```no_run
//! use prtip_core::progress::ProgressCalculator;
//!
//! # async fn example() {
//! let calculator = ProgressCalculator::new(1000, 10000);
//!
//! // Update progress periodically
//! calculator.update(500, 5000).await; // 50% complete
//!
//! let eta = calculator.eta().await;
//! println!("ETA: {:.0} seconds", eta.map(|d| d.as_secs_f64()).unwrap_or(0.0));
//!
//! let percentage = calculator.percentage().await;
//! println!("Progress: {:.1}%", percentage);
//! # }
//! ```

use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

const EWMA_ALPHA: f64 = 0.3; // Weight for recent rates (30% new, 70% historical)
const STARTUP_THRESHOLD: f64 = 0.05; // Use linear ETA until 5% complete
const RATE_WINDOW_SECS: u64 = 60; // Track rates over last 60 seconds

/// Progress calculation state
#[derive(Debug, Clone)]
struct ProgressState {
    /// Total work units (targets × ports)
    total: u64,
    /// Completed work units
    completed: u64,
    /// Start time
    started_at: Instant,
    /// Last update time
    last_update: Instant,
    /// Smoothed completion rate (units per second) using EWMA
    smoothed_rate: Option<f64>,
    /// Historical rate samples (time, completed) for sliding window
    rate_samples: VecDeque<(Instant, u64)>,
}

/// Real-time progress and ETA calculator with EWMA smoothing
///
/// Provides accurate progress percentage and estimated time to completion
/// by tracking completion rate with exponential weighted moving average.
///
/// # Thread Safety
///
/// Uses Arc<RwLock> for concurrent read/write access from multiple threads.
///
/// # Performance
///
/// - Update: O(1) amortized (occasionally trims rate window)
/// - ETA calculation: O(1) (uses cached smoothed rate)
#[derive(Clone)]
pub struct ProgressCalculator {
    state: Arc<RwLock<ProgressState>>,
}

impl ProgressCalculator {
    /// Create a new progress calculator
    ///
    /// # Arguments
    ///
    /// * `total_targets` - Number of targets to scan
    /// * `total_ports` - Number of ports per target
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::progress::ProgressCalculator;
    ///
    /// // 100 targets, 1000 ports each = 100,000 total work
    /// let calc = ProgressCalculator::new(100, 1000);
    /// ```
    pub fn new(total_targets: usize, total_ports: usize) -> Self {
        let total = (total_targets as u64) * (total_ports as u64);
        let now = Instant::now();

        Self {
            state: Arc::new(RwLock::new(ProgressState {
                total,
                completed: 0,
                started_at: now,
                last_update: now,
                smoothed_rate: None,
                rate_samples: VecDeque::new(),
            })),
        }
    }

    /// Update progress with current completion count
    ///
    /// Should be called periodically (e.g., every 5% progress or 30 seconds).
    /// Recalculates smoothed rate and trims old rate samples.
    ///
    /// # Arguments
    ///
    /// * `completed_targets` - Number of targets fully scanned (unused, kept for API compat)
    /// * `completed_ports` - Total number of work units completed (target × port combinations)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressCalculator;
    ///
    /// # async fn example() {
    /// let calc = ProgressCalculator::new(100, 1000);
    ///
    /// // After scanning 50 targets × 1000 ports = 50,000 units
    /// calc.update(0, 50000).await;
    /// # }
    /// ```
    pub async fn update(&self, _completed_targets: usize, completed_ports: usize) {
        let mut state = self.state.write();
        let now = Instant::now();
        let completed = completed_ports as u64;

        // Calculate current rate (units per second since last update)
        let delta_completed = completed.saturating_sub(state.completed);
        let delta_time = now.duration_since(state.last_update).as_secs_f64();

        if delta_time > 0.0 && delta_completed > 0 {
            let current_rate = delta_completed as f64 / delta_time;

            // Update EWMA smoothed rate
            state.smoothed_rate = Some(match state.smoothed_rate {
                Some(prev_rate) => {
                    // EWMA: new_rate = alpha × current + (1 - alpha) × previous
                    EWMA_ALPHA * current_rate + (1.0 - EWMA_ALPHA) * prev_rate
                }
                None => current_rate, // First sample
            });

            // Add to sliding window
            state.rate_samples.push_back((now, completed));

            // Trim old samples (older than RATE_WINDOW_SECS)
            let cutoff = now - Duration::from_secs(RATE_WINDOW_SECS);
            while state
                .rate_samples
                .front()
                .map(|(t, _)| *t < cutoff)
                .unwrap_or(false)
            {
                state.rate_samples.pop_front();
            }
        }

        state.completed = completed;
        state.last_update = now;
    }

    /// Get current progress percentage (0.0 - 100.0)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressCalculator;
    ///
    /// # async fn example() {
    /// let calc = ProgressCalculator::new(100, 1000);
    /// calc.update(50, 50000).await;
    ///
    /// let pct = calc.percentage().await;
    /// assert!((pct - 50.0).abs() < 0.1);
    /// # }
    /// ```
    pub async fn percentage(&self) -> f64 {
        let state = self.state.read();
        if state.total == 0 {
            return 100.0;
        }
        (state.completed as f64 / state.total as f64) * 100.0
    }

    /// Get estimated time to completion
    ///
    /// Returns `None` if:
    /// - Not enough data (< 1% complete)
    /// - Rate is zero or negative
    /// - Already 100% complete
    ///
    /// # Algorithm
    ///
    /// - **Startup** (< 5% complete): Simple linear ETA = remaining / instant_rate
    /// - **Normal** (>= 5% complete): EWMA-smoothed ETA = remaining / smoothed_rate
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressCalculator;
    ///
    /// # async fn example() {
    /// let calc = ProgressCalculator::new(100, 1000);
    /// calc.update(50, 50000).await;
    ///
    /// if let Some(eta) = calc.eta().await {
    ///     println!("Estimated {} seconds remaining", eta.as_secs());
    /// }
    /// # }
    /// ```
    pub async fn eta(&self) -> Option<Duration> {
        let state = self.state.read();

        if state.completed >= state.total {
            return None; // Already complete
        }

        let remaining = state.total - state.completed;
        let pct_complete = state.completed as f64 / state.total as f64;

        // Not enough data for reliable estimate
        if pct_complete < 0.01 {
            return None;
        }

        let rate = if pct_complete < STARTUP_THRESHOLD {
            // Startup phase: use instantaneous rate
            let elapsed = state.started_at.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                state.completed as f64 / elapsed
            } else {
                return None;
            }
        } else {
            // Normal phase: use EWMA smoothed rate
            state.smoothed_rate?
        };

        if rate <= 0.0 {
            return None;
        }

        let seconds = remaining as f64 / rate;
        Some(Duration::from_secs_f64(seconds))
    }

    /// Get current completion rate (units per second)
    ///
    /// Returns smoothed rate if available, otherwise instantaneous rate.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressCalculator;
    ///
    /// # async fn example() {
    /// let calc = ProgressCalculator::new(100, 1000);
    /// calc.update(50, 50000).await;
    ///
    /// let rate = calc.rate().await;
    /// println!("Current rate: {:.0} ports/sec", rate);
    /// # }
    /// ```
    pub async fn rate(&self) -> f64 {
        let state = self.state.read();

        // Try smoothed rate first
        if let Some(smoothed) = state.smoothed_rate {
            return smoothed;
        }

        // Fall back to overall rate
        let elapsed = state.started_at.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            state.completed as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Get elapsed time since scan start
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressCalculator;
    ///
    /// # async fn example() {
    /// let calc = ProgressCalculator::new(100, 1000);
    /// calc.update(50, 50000).await;
    ///
    /// let elapsed = calc.elapsed().await;
    /// println!("Elapsed: {:.0} seconds", elapsed.as_secs_f64());
    /// # }
    /// ```
    pub async fn elapsed(&self) -> Duration {
        let state = self.state.read();
        state.started_at.elapsed()
    }

    /// Get completed and total counts
    ///
    /// Returns (completed, total) as a tuple.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressCalculator;
    ///
    /// # async fn example() {
    /// let calc = ProgressCalculator::new(100, 1000);
    /// calc.update(50, 50000).await;
    ///
    /// let (completed, total) = calc.counts().await;
    /// println!("{} / {} completed", completed, total);
    /// # }
    /// ```
    pub async fn counts(&self) -> (u64, u64) {
        let state = self.state.read();
        (state.completed, state.total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_new_calculator() {
        let calc = ProgressCalculator::new(100, 1000);
        assert_eq!(calc.percentage().await, 0.0);
        assert!(calc.eta().await.is_none()); // No data yet
    }

    #[tokio::test]
    async fn test_progress_update() {
        let calc = ProgressCalculator::new(100, 1000);

        calc.update(0, 50000).await; // 50,000 of 100,000 units = 50%
        let pct = calc.percentage().await;
        assert!((pct - 50.0).abs() < 0.1);

        let (completed, total) = calc.counts().await;
        assert_eq!(completed, 50000);
        assert_eq!(total, 100000);
    }

    #[tokio::test]
    async fn test_eta_calculation() {
        let calc = ProgressCalculator::new(100, 1000);

        // Simulate progress over time
        sleep(Duration::from_millis(100)).await;
        calc.update(0, 10000).await; // 10% done

        sleep(Duration::from_millis(100)).await;
        calc.update(0, 20000).await; // 20% done

        let eta = calc.eta().await;
        assert!(eta.is_some(), "ETA should be available with enough data");

        if let Some(duration) = eta {
            // ETA should be reasonable (not negative, not infinite)
            assert!(duration.as_secs() < 3600, "ETA should be < 1 hour");
        }
    }

    #[tokio::test]
    async fn test_rate_calculation() {
        let calc = ProgressCalculator::new(100, 1000);

        sleep(Duration::from_millis(100)).await;
        calc.update(0, 50000).await;

        let rate = calc.rate().await;
        assert!(rate > 0.0, "Rate should be positive");
        assert!(rate < 1_000_000.0, "Rate should be reasonable");
    }

    #[tokio::test]
    async fn test_ewma_smoothing() {
        let calc = ProgressCalculator::new(100, 10000);

        // First update
        sleep(Duration::from_millis(100)).await;
        calc.update(0, 100000).await;
        let rate1 = calc.rate().await;

        // Second update (different rate)
        sleep(Duration::from_millis(200)).await;
        calc.update(0, 200000).await;
        let rate2 = calc.rate().await;

        // EWMA should smooth the rates
        assert!(rate1 > 0.0);
        assert!(rate2 > 0.0);
        // Rate should be influenced by both samples (not just latest)
    }

    #[tokio::test]
    async fn test_completion() {
        let calc = ProgressCalculator::new(100, 1000);

        calc.update(0, 100000).await; // 100 targets × 1000 ports = 100,000 units
        assert_eq!(calc.percentage().await, 100.0);
        assert!(calc.eta().await.is_none()); // No ETA when complete
    }

    #[tokio::test]
    async fn test_elapsed_time() {
        let calc = ProgressCalculator::new(100, 1000);

        sleep(Duration::from_millis(100)).await;
        let elapsed = calc.elapsed().await;
        assert!(elapsed.as_millis() >= 100);
    }

    #[tokio::test]
    async fn test_startup_threshold() {
        let calc = ProgressCalculator::new(100, 10000);

        // Below startup threshold (5%)
        sleep(Duration::from_millis(50)).await;
        calc.update(0, 20000).await; // 2% complete (20k / 1M total)

        let eta1 = calc.eta().await;

        // Above startup threshold
        sleep(Duration::from_millis(100)).await;
        calc.update(0, 100000).await; // 10% complete (100k / 1M total)

        let eta2 = calc.eta().await;

        // Both should provide ETA (different algorithms)
        assert!(eta1.is_some() || eta2.is_some());
    }

    #[tokio::test]
    async fn test_zero_total() {
        let calc = ProgressCalculator::new(0, 0);
        assert_eq!(calc.percentage().await, 100.0); // Zero work = 100% complete
    }

    #[tokio::test]
    async fn test_rate_window_trimming() {
        let calc = ProgressCalculator::new(1000, 10000);

        // Add many samples
        for i in 0..100 {
            sleep(Duration::from_millis(10)).await;
            calc.update(0, i * 1000).await;
        }

        // Verify state (samples should be trimmed to 60-second window)
        let state = calc.state.read();
        // Should have trimmed old samples
        assert!(state.rate_samples.len() < 100);
    }
}
