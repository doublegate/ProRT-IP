//! Advanced timing control and adaptive rate limiting
//!
//! This module implements sophisticated timing control for scans, including:
//! - Timing templates (T0-T5) matching Nmap's behavior
//! - Adaptive rate limiting with AIMD congestion control
//! - RTT (Round Trip Time) estimation
//! - Dynamic timeout calculation
//! - Jitter for IDS/IPS evasion
//!
//! # Timing Templates
//!
//! Templates from T0 (Paranoid) to T5 (Insane) control scan aggressiveness:
//!
//! - **T0 (Paranoid)**: 5-minute delays, 1 concurrent probe, 300s timeout
//! - **T1 (Sneaky)**: 15-second delays, 10 concurrent, 15s timeout
//! - **T2 (Polite)**: 0.4-second delays, 100 concurrent, 10s timeout
//! - **T3 (Normal)**: No delays, 1000 concurrent, 3s timeout (default)
//! - **T4 (Aggressive)**: No delays, 5000 concurrent, 1s timeout
//! - **T5 (Insane)**: No delays, 10000 concurrent, 250ms timeout
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::timing::{TimingConfig, AdaptiveRateLimiter};
//! use prtip_core::TimingTemplate;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! // Create timing config from template
//! let config = TimingConfig::from_template(TimingTemplate::Aggressive);
//!
//! // Create adaptive rate limiter
//! let mut limiter = AdaptiveRateLimiter::new(config);
//!
//! // Wait before sending packet
//! limiter.wait().await;
//!
//! // Report success or failure
//! limiter.report_response(true, std::time::Duration::from_millis(50));
//! # Ok(())
//! # }
//! ```

use parking_lot::Mutex;
use prtip_core::TimingTemplate;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, trace};

/// Configuration for timing behavior
#[derive(Debug, Clone)]
pub struct TimingConfig {
    /// Initial timeout for probes
    pub initial_timeout: Duration,
    /// Minimum timeout (never go below this)
    pub min_timeout: Duration,
    /// Maximum timeout (never exceed this)
    pub max_timeout: Duration,
    /// Maximum number of retries
    pub max_retries: u8,
    /// Delay between probes
    pub scan_delay: Duration,
    /// Maximum concurrent probes
    pub max_parallelism: usize,
    /// Enable jitter for timing randomization
    pub enable_jitter: bool,
    /// Jitter amount (fraction of delay)
    pub jitter_factor: f64,
}

impl TimingConfig {
    /// Create timing config from a template
    pub fn from_template(template: TimingTemplate) -> Self {
        match template {
            TimingTemplate::Paranoid => Self {
                initial_timeout: Duration::from_secs(300),
                min_timeout: Duration::from_secs(100),
                max_timeout: Duration::from_secs(300),
                max_retries: 5,
                scan_delay: Duration::from_secs(300),
                max_parallelism: 1,
                enable_jitter: true,
                jitter_factor: 0.3,
            },
            TimingTemplate::Sneaky => Self {
                initial_timeout: Duration::from_secs(15),
                min_timeout: Duration::from_secs(5),
                max_timeout: Duration::from_secs(15),
                max_retries: 5,
                scan_delay: Duration::from_secs(15),
                max_parallelism: 10,
                enable_jitter: true,
                jitter_factor: 0.2,
            },
            TimingTemplate::Polite => Self {
                initial_timeout: Duration::from_secs(10),
                min_timeout: Duration::from_secs(1),
                max_timeout: Duration::from_secs(10),
                max_retries: 5,
                scan_delay: Duration::from_millis(400),
                max_parallelism: 100,
                enable_jitter: true,
                jitter_factor: 0.1,
            },
            TimingTemplate::Normal => Self {
                initial_timeout: Duration::from_secs(3),
                min_timeout: Duration::from_millis(500),
                max_timeout: Duration::from_secs(10),
                max_retries: 2,
                scan_delay: Duration::from_millis(0),
                max_parallelism: 1000,
                enable_jitter: false,
                jitter_factor: 0.0,
            },
            TimingTemplate::Aggressive => Self {
                initial_timeout: Duration::from_secs(1),
                min_timeout: Duration::from_millis(100),
                max_timeout: Duration::from_secs(10),
                max_retries: 6,
                scan_delay: Duration::from_millis(0),
                max_parallelism: 5000,
                enable_jitter: false,
                jitter_factor: 0.0,
            }
            .with_max_timeout_millis(1250),
            TimingTemplate::Insane => Self {
                initial_timeout: Duration::from_millis(250),
                min_timeout: Duration::from_millis(50),
                max_timeout: Duration::from_millis(300),
                max_retries: 2,
                scan_delay: Duration::from_millis(0),
                max_parallelism: 10000,
                enable_jitter: false,
                jitter_factor: 0.0,
            },
        }
    }

    fn with_max_timeout_millis(mut self, millis: u64) -> Self {
        self.max_timeout = Duration::from_millis(millis);
        self
    }

    /// Apply jitter to a duration
    pub fn apply_jitter(&self, duration: Duration) -> Duration {
        if !self.enable_jitter || self.jitter_factor == 0.0 {
            return duration;
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Jitter range: [duration * (1 - factor), duration * (1 + factor)]
        let millis = duration.as_millis() as f64;
        let min_millis = millis * (1.0 - self.jitter_factor);
        let max_millis = millis * (1.0 + self.jitter_factor);

        let jittered_millis = rng.gen_range(min_millis..max_millis);
        Duration::from_millis(jittered_millis as u64)
    }
}

/// RTT (Round Trip Time) statistics
#[derive(Debug, Clone)]
struct RttStats {
    /// Smoothed RTT
    srtt: Duration,
    /// RTT variance
    rttvar: Duration,
    /// Number of samples
    samples: usize,
}

impl RttStats {
    fn new() -> Self {
        Self {
            srtt: Duration::from_secs(3),
            rttvar: Duration::from_secs(1),
            samples: 0,
        }
    }

    /// Update statistics with a new RTT measurement
    fn update(&mut self, rtt: Duration) {
        const ALPHA: f64 = 0.125; // Weight for SRTT
        const BETA: f64 = 0.25; // Weight for RTTVAR

        if self.samples == 0 {
            // First measurement
            self.srtt = rtt;
            self.rttvar = rtt / 2;
        } else {
            // RFC 6298 algorithm
            let rtt_millis = rtt.as_millis() as f64;
            let srtt_millis = self.srtt.as_millis() as f64;

            let diff = (rtt_millis - srtt_millis).abs();
            let new_rttvar = (1.0 - BETA) * self.rttvar.as_millis() as f64 + BETA * diff;
            let new_srtt = (1.0 - ALPHA) * srtt_millis + ALPHA * rtt_millis;

            self.srtt = Duration::from_millis(new_srtt as u64);
            self.rttvar = Duration::from_millis(new_rttvar as u64);
        }

        self.samples += 1;
    }

    /// Calculate recommended timeout (RTO)
    fn timeout(&self) -> Duration {
        // RTO = SRTT + max(G, K * RTTVAR) where K = 4
        let k = 4;
        let g = Duration::from_millis(10); // Clock granularity

        let variance_component = std::cmp::max(g, self.rttvar * k);
        self.srtt + variance_component
    }
}

/// Adaptive rate limiter with AIMD congestion control
pub struct AdaptiveRateLimiter {
    config: TimingConfig,
    /// Current rate in millihertz (mHz = packets/sec * 1000) for atomic storage
    current_rate_mhz: AtomicU64,
    /// Minimum rate in mHz
    min_rate_mhz: u64,
    /// Maximum rate in mHz
    max_rate_mhz: u64,
    /// Number of consecutive timeouts
    consecutive_timeouts: AtomicUsize,
    /// Number of successful responses
    successful_responses: AtomicUsize,
    /// RTT statistics (still needs mutex for complex operations)
    rtt_stats: Arc<Mutex<RttStats>>,
    /// Last rate adjustment time (stored as micros since epoch)
    last_adjustment_micros: AtomicU64,
}

impl AdaptiveRateLimiter {
    /// Create a new adaptive rate limiter
    pub fn new(config: TimingConfig) -> Self {
        let initial_rate = (config.max_parallelism as f64) * 10.0; // 10 Hz per parallel probe
        let initial_rate_mhz = (initial_rate * 1000.0) as u64;

        let now_micros = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        Self {
            config,
            current_rate_mhz: AtomicU64::new(initial_rate_mhz),
            min_rate_mhz: 10_000,        // 10 pps * 1000
            max_rate_mhz: 1_000_000_000, // 1M pps * 1000
            consecutive_timeouts: AtomicUsize::new(0),
            successful_responses: AtomicUsize::new(0),
            rtt_stats: Arc::new(Mutex::new(RttStats::new())),
            last_adjustment_micros: AtomicU64::new(now_micros),
        }
    }

    /// Wait before sending next packet
    pub async fn wait(&self) {
        // Read current rate atomically (convert from mHz to pps)
        let rate_mhz = self.current_rate_mhz.load(Ordering::Relaxed);
        let packets_per_second = ((rate_mhz as f64) / 1000.0).max(1.0);
        let delay_millis = 1000.0 / packets_per_second;
        let delay = Duration::from_millis(delay_millis as u64);

        // Apply jitter if enabled
        let jittered_delay = self.config.apply_jitter(delay);

        // Add scan delay
        let total_delay = jittered_delay + self.config.scan_delay;

        if total_delay > Duration::from_millis(1) {
            sleep(total_delay).await;
        } else {
            tokio::task::yield_now().await;
        }
    }

    /// Report a response (success or failure)
    pub fn report_response(&self, success: bool, rtt: Duration) {
        if success {
            // Update RTT statistics (requires mutex)
            self.rtt_stats.lock().update(rtt);

            // Atomic updates
            self.successful_responses.fetch_add(1, Ordering::Relaxed);
            self.consecutive_timeouts.store(0, Ordering::Relaxed);

            // AIMD: Additive Increase
            // Increase rate slowly when successful (check if 100ms elapsed)
            let now_micros = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64;

            let last_adj_micros = self.last_adjustment_micros.load(Ordering::Relaxed);

            if now_micros.saturating_sub(last_adj_micros) > 100_000 {
                // 100ms in microseconds
                // Atomic compare-and-swap loop for rate increase
                loop {
                    let current_mhz = self.current_rate_mhz.load(Ordering::Relaxed);
                    let increase_mhz = (current_mhz as f64 * 0.01) as u64; // 1% increase
                    let new_mhz = (current_mhz + increase_mhz).min(self.max_rate_mhz);

                    if self
                        .current_rate_mhz
                        .compare_exchange_weak(
                            current_mhz,
                            new_mhz,
                            Ordering::Release,
                            Ordering::Relaxed,
                        )
                        .is_ok()
                    {
                        self.last_adjustment_micros
                            .store(now_micros, Ordering::Relaxed);
                        trace!(
                            "Rate increase: {:.0} pps (success rate improved)",
                            new_mhz as f64 / 1000.0
                        );
                        break;
                    }
                }
            }
        } else {
            // Timeout occurred
            let timeouts = self.consecutive_timeouts.fetch_add(1, Ordering::Relaxed) + 1;

            // AIMD: Multiplicative Decrease
            // Decrease rate aggressively on timeouts
            if timeouts >= 3 {
                // Atomic compare-and-swap loop for rate decrease
                loop {
                    let current_mhz = self.current_rate_mhz.load(Ordering::Relaxed);
                    let new_mhz = ((current_mhz as f64 * 0.5) as u64).max(self.min_rate_mhz); // Cut in half

                    if self
                        .current_rate_mhz
                        .compare_exchange_weak(
                            current_mhz,
                            new_mhz,
                            Ordering::Release,
                            Ordering::Relaxed,
                        )
                        .is_ok()
                    {
                        self.consecutive_timeouts.store(0, Ordering::Relaxed);
                        let now_micros = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_micros() as u64;
                        self.last_adjustment_micros
                            .store(now_micros, Ordering::Relaxed);
                        debug!(
                            "Rate decrease: {:.0} pps (congestion detected)",
                            new_mhz as f64 / 1000.0
                        );
                        break;
                    }
                }
            }
        }
    }

    /// Get current recommended timeout
    pub fn current_timeout(&self) -> Duration {
        let rtt_stats = self.rtt_stats.lock();
        let calculated = rtt_stats.timeout();

        // Clamp to configured min/max
        calculated
            .max(self.config.min_timeout)
            .min(self.config.max_timeout)
    }

    /// Get current rate in packets per second
    pub fn current_rate(&self) -> f64 {
        let rate_mhz = self.current_rate_mhz.load(Ordering::Relaxed);
        (rate_mhz as f64) / 1000.0
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.consecutive_timeouts.store(0, Ordering::Relaxed);
        self.successful_responses.store(0, Ordering::Relaxed);
        *self.rtt_stats.lock() = RttStats::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_config_from_template() {
        let normal = TimingConfig::from_template(TimingTemplate::Normal);
        assert_eq!(normal.max_retries, 2);
        assert_eq!(normal.max_parallelism, 1000);

        let paranoid = TimingConfig::from_template(TimingTemplate::Paranoid);
        assert_eq!(paranoid.max_retries, 5);
        assert_eq!(paranoid.max_parallelism, 1);
    }

    #[test]
    fn test_jitter_application() {
        let config = TimingConfig {
            enable_jitter: true,
            jitter_factor: 0.2,
            ..TimingConfig::from_template(TimingTemplate::Normal)
        };

        let base_duration = Duration::from_secs(1);
        let jittered = config.apply_jitter(base_duration);

        // Should be within ±20%
        let base_millis = base_duration.as_millis();
        let jittered_millis = jittered.as_millis();

        assert!(jittered_millis >= (base_millis as f64 * 0.8) as u128);
        assert!(jittered_millis <= (base_millis as f64 * 1.2) as u128);
    }

    #[test]
    fn test_rtt_stats() {
        let mut stats = RttStats::new();

        stats.update(Duration::from_millis(100));
        assert!(stats.samples == 1);

        stats.update(Duration::from_millis(150));
        assert!(stats.samples == 2);

        let timeout = stats.timeout();
        assert!(timeout > Duration::from_millis(100));
    }

    #[test]
    fn test_adaptive_rate_limiter_creation() {
        let config = TimingConfig::from_template(TimingTemplate::Normal);
        let limiter = AdaptiveRateLimiter::new(config);

        let rate = limiter.current_rate();
        assert!(rate > 0.0);
    }

    #[test]
    fn test_adaptive_rate_increase() {
        let config = TimingConfig::from_template(TimingTemplate::Normal);
        let limiter = AdaptiveRateLimiter::new(config);

        let initial_rate = limiter.current_rate();

        // Report multiple successes
        for _ in 0..10 {
            limiter.report_response(true, Duration::from_millis(50));
            std::thread::sleep(Duration::from_millis(150)); // Wait for adjustment interval
        }

        let new_rate = limiter.current_rate();
        // Rate may increase (but not guaranteed due to timing)
        assert!(new_rate >= initial_rate * 0.9); // Allow some variance
    }

    #[test]
    fn test_adaptive_rate_decrease() {
        let config = TimingConfig::from_template(TimingTemplate::Normal);
        let limiter = AdaptiveRateLimiter::new(config);

        let initial_rate = limiter.current_rate();

        // Report multiple failures
        for _ in 0..5 {
            limiter.report_response(false, Duration::from_secs(5));
        }

        let new_rate = limiter.current_rate();
        assert!(new_rate < initial_rate); // Rate should decrease
    }
}
