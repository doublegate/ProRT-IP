//! Retry logic with exponential backoff
//!
//! This module provides retry mechanisms for transient failures with:
//! - Exponential backoff to avoid overwhelming failing services
//! - Jitter to prevent thundering herd problems
//! - Timing templates matching Nmap's T0-T5 aggressiveness levels
//! - Integration with error types for automatic retriability detection
//!
//! Sprint 4.22 Phase 4: Recovery Mechanisms

use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

/// Retry configuration with exponential backoff
#[derive(Debug, Clone, PartialEq)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial delay before first retry
    pub initial_delay: Duration,

    /// Maximum delay between retries
    pub max_delay: Duration,

    /// Multiplier for exponential backoff (typically 2.0)
    pub multiplier: f64,

    /// Whether to add jitter to delay (reduces thundering herd)
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::normal()
    }
}

impl RetryConfig {
    /// Paranoid retry (T0): Very slow, many retries
    ///
    /// Use for stealthy scans where detection avoidance is critical.
    /// - 5 retry attempts
    /// - 1 second initial delay
    /// - Up to 30 seconds maximum delay
    pub fn paranoid() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }

    /// Sneaky retry (T1): Slow, multiple retries
    ///
    /// Use for cautious scans with moderate stealth requirements.
    /// - 4 retry attempts
    /// - 500ms initial delay
    /// - Up to 15 seconds maximum delay
    pub fn sneaky() -> Self {
        Self {
            max_attempts: 4,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(15),
            multiplier: 2.0,
            jitter: true,
        }
    }

    /// Normal retry (T3): Balanced (default)
    ///
    /// Use for most scanning scenarios.
    /// - 3 retry attempts
    /// - 100ms initial delay
    /// - Up to 5 seconds maximum delay
    pub fn normal() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            multiplier: 2.0,
            jitter: true,
        }
    }

    /// Aggressive retry (T4): Fast, fewer retries
    ///
    /// Use for fast scans where speed is more important than reliability.
    /// - 2 retry attempts
    /// - 50ms initial delay
    /// - Up to 2 seconds maximum delay
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(2),
            multiplier: 2.0,
            jitter: false, // No jitter for speed
        }
    }

    /// Insane retry (T5): No retries (fail fast)
    ///
    /// Use for maximum speed scans where failures are acceptable.
    /// - 1 attempt only (no retries)
    /// - No delays
    pub fn insane() -> Self {
        Self {
            max_attempts: 1,
            initial_delay: Duration::from_millis(0),
            max_delay: Duration::from_millis(0),
            multiplier: 1.0,
            jitter: false,
        }
    }
}

/// Retry a fallible async operation with exponential backoff
///
/// # Arguments
///
/// * `operation` - The operation to retry (closure returning Future<Result<T, E>>)
/// * `config` - Retry configuration (use timing templates for Nmap compatibility)
/// * `is_retriable` - Function to determine if error should be retried
///
/// # Examples
///
/// ```no_run
/// use prtip_core::retry::{retry_with_backoff, RetryConfig};
/// use std::io;
///
/// async fn connect_to_target() -> Result<(), io::Error> {
///     // ... connection logic
///     Ok(())
/// }
///
/// # async fn example() -> Result<(), io::Error> {
/// let result = retry_with_backoff(
///     || connect_to_target(),
///     RetryConfig::normal(),
///     |err| err.kind() == io::ErrorKind::TimedOut,
/// ).await;
/// # Ok(())
/// # }
/// ```
///
/// # Behavior
///
/// - Attempts operation up to `config.max_attempts` times
/// - On failure, checks if error is retriable via `is_retriable()`
/// - If retriable, waits with exponential backoff before retrying
/// - If not retriable or max attempts reached, returns error
/// - Logs retry attempts at debug level
pub async fn retry_with_backoff<T, E, F, Fut, R>(
    mut operation: F,
    config: RetryConfig,
    is_retriable: R,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    R: Fn(&E) -> bool,
{
    let mut delay = config.initial_delay;
    let mut attempt = 0;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!("Operation succeeded after {} attempts", attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                // Check if we should retry
                let should_retry = attempt < config.max_attempts && is_retriable(&e);

                if !should_retry {
                    if attempt >= config.max_attempts {
                        debug!(
                            "Max retry attempts ({}) exceeded for operation",
                            config.max_attempts
                        );
                    } else {
                        debug!(
                            "Error is not retriable, giving up after attempt {}",
                            attempt
                        );
                    }
                    return Err(e);
                }

                // Log retry attempt
                debug!(
                    "Attempt {}/{} failed: {}. Retrying in {:?}...",
                    attempt, config.max_attempts, e, delay
                );

                // Wait before retry
                if delay.as_millis() > 0 {
                    let actual_delay = if config.jitter {
                        add_jitter(delay)
                    } else {
                        delay
                    };
                    sleep(actual_delay).await;
                }

                // Calculate next delay (exponential backoff)
                delay = std::cmp::min(
                    Duration::from_secs_f64(delay.as_secs_f64() * config.multiplier),
                    config.max_delay,
                );
            }
        }
    }
}

/// Add random jitter to delay (±25%)
///
/// This prevents thundering herd problems where many clients retry simultaneously.
/// The jitter factor is randomly chosen between 0.75 and 1.25, giving a ±25% variation.
fn add_jitter(delay: Duration) -> Duration {
    use rand::Rng;
    let jitter_factor = rand::thread_rng().gen_range(0.75..=1.25);
    Duration::from_secs_f64(delay.as_secs_f64() * jitter_factor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let result = retry_with_backoff(
            || async { Ok::<_, String>(42) },
            RetryConfig::default(),
            |_| true,
        )
        .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = retry_with_backoff(
            || {
                let attempts = attempts_clone.clone();
                async move {
                    let count = attempts.fetch_add(1, Ordering::SeqCst);
                    if count == 0 {
                        Err("first attempt fails")
                    } else {
                        Ok(42)
                    }
                }
            },
            RetryConfig::default(),
            |_| true,
        )
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_max_attempts_exceeded() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = retry_with_backoff(
            || {
                let attempts = attempts_clone.clone();
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>("always fails")
                }
            },
            RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(1),
                max_delay: Duration::from_millis(10),
                multiplier: 2.0,
                jitter: false,
            },
            |_| true,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_non_retriable_error() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = retry_with_backoff(
            || {
                let attempts = attempts_clone.clone();
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>("non-retriable")
                }
            },
            RetryConfig::default(),
            |_| false, // Not retriable
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1); // Only one attempt
    }

    #[tokio::test]
    async fn test_retry_exponential_backoff() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let start = std::time::Instant::now();
        let _result = retry_with_backoff(
            || {
                let attempts = attempts_clone.clone();
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>("always fails")
                }
            },
            RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(100),
                multiplier: 2.0,
                jitter: false,
            },
            |_| true,
        )
        .await;
        let elapsed = start.elapsed();

        // Should wait: 10ms + 20ms = 30ms minimum (no jitter)
        assert!(elapsed >= Duration::from_millis(30));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_retry_config_paranoid() {
        let config = RetryConfig::paranoid();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.multiplier, 2.0);
        assert!(config.jitter);
    }

    #[test]
    fn test_retry_config_sneaky() {
        let config = RetryConfig::sneaky();
        assert_eq!(config.max_attempts, 4);
        assert_eq!(config.initial_delay, Duration::from_millis(500));
        assert_eq!(config.max_delay, Duration::from_secs(15));
    }

    #[test]
    fn test_retry_config_normal() {
        let config = RetryConfig::normal();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(5));
        assert_eq!(config, RetryConfig::default());
    }

    #[test]
    fn test_retry_config_aggressive() {
        let config = RetryConfig::aggressive();
        assert_eq!(config.max_attempts, 2);
        assert_eq!(config.initial_delay, Duration::from_millis(50));
        assert_eq!(config.max_delay, Duration::from_secs(2));
        assert!(!config.jitter); // No jitter for speed
    }

    #[test]
    fn test_retry_config_insane() {
        let config = RetryConfig::insane();
        assert_eq!(config.max_attempts, 1); // No retries
        assert_eq!(config.initial_delay, Duration::from_millis(0));
        assert!(!config.jitter);
    }

    #[test]
    fn test_add_jitter_range() {
        let delay = Duration::from_secs(1);
        for _ in 0..100 {
            let jittered = add_jitter(delay);
            // Should be within ±25% (0.75s to 1.25s)
            assert!(jittered >= Duration::from_millis(750));
            assert!(jittered <= Duration::from_millis(1250));
        }
    }

    #[tokio::test]
    async fn test_retry_with_jitter() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let _result = retry_with_backoff(
            || {
                let attempts = attempts_clone.clone();
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>("always fails")
                }
            },
            RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(100),
                multiplier: 2.0,
                jitter: true, // Enable jitter
            },
            |_| true,
        )
        .await;

        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }
}
