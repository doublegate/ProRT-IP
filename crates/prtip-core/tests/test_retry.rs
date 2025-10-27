//! Retry logic integration tests
//!
//! Tests exponential backoff retry mechanisms for transient failures.
//! Complements the unit tests in retry.rs with integration scenarios.
//!
//! Sprint 4.22 Phase 7: Comprehensive Testing - Subtask 3

use prtip_core::retry::{retry_with_backoff, RetryConfig};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ========================================================================
// Test 1-2: Max Retry Limit Tests
// ========================================================================

#[tokio::test]
async fn test_max_retry_limit_exactly_3_attempts() {
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        multiplier: 2.0,
        jitter: false,
    };

    let result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("permanent failure")
            }
        },
        config,
        |_| true, // Always retriable
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        3,
        "Should attempt exactly 3 times"
    );
}

#[tokio::test]
async fn test_max_retry_limit_custom_5_attempts() {
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        multiplier: 2.0,
        jitter: false,
    };

    let result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("always fails")
            }
        },
        config,
        |_| true,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        5,
        "Should attempt exactly 5 times"
    );
}

// ========================================================================
// Test 3-5: Exponential Backoff Tests
// ========================================================================

#[tokio::test]
async fn test_exponential_backoff_timing_1s_2s_4s() {
    let attempts = Arc::new(AtomicU32::new(0));
    let attempt_times = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let attempts_clone = attempts.clone();
    let times_clone = attempt_times.clone();

    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(10),
        multiplier: 2.0,
        jitter: false,
    };

    let start = Instant::now();
    let _result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            let times = times_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                times.lock().await.push(start.elapsed());
                Err::<(), _>("always fails")
            }
        },
        config,
        |_| true,
    )
    .await;

    let times = attempt_times.lock().await;
    assert_eq!(times.len(), 3);

    // Attempt 1: Immediate (0ms)
    assert!(
        times[0] < Duration::from_millis(100),
        "First attempt should be immediate"
    );

    // Attempt 2: After ~1s (initial_delay)
    assert!(
        times[1] >= Duration::from_millis(900),
        "Second attempt should wait ~1s"
    );
    assert!(
        times[1] <= Duration::from_millis(1200),
        "Second attempt within tolerance"
    );

    // Attempt 3: After ~3s total (1s + 2s backoff)
    assert!(
        times[2] >= Duration::from_millis(2900),
        "Third attempt should wait ~3s total"
    );
    assert!(
        times[2] <= Duration::from_millis(3200),
        "Third attempt within tolerance"
    );
}

#[tokio::test]
async fn test_exponential_backoff_respects_max_delay() {
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(200), // Cap at 200ms
        multiplier: 2.0,
        jitter: false,
    };

    let start = Instant::now();
    let _result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("always fails")
            }
        },
        config,
        |_| true,
    )
    .await;

    let elapsed = start.elapsed();

    // Expected delays:
    // Attempt 1: 0ms
    // Attempt 2: 100ms (initial)
    // Attempt 3: 200ms (min(100*2, 200) = 200)
    // Attempt 4: 200ms (min(200*2, 200) = 200, capped)
    // Attempt 5: 200ms (capped)
    // Total: 100 + 200 + 200 + 200 = 700ms
    assert!(
        elapsed >= Duration::from_millis(650),
        "Should respect max_delay cap"
    );
    assert!(elapsed <= Duration::from_millis(850), "Within tolerance");
}

#[tokio::test]
async fn test_backoff_multiplier_effects() {
    let config_2x = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        multiplier: 2.0,
        jitter: false,
    };

    let config_3x = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        multiplier: 3.0,
        jitter: false,
    };

    let start_2x = Instant::now();
    let _result_2x =
        retry_with_backoff(|| async { Err::<(), _>("fail") }, config_2x, |_| true).await;
    let elapsed_2x = start_2x.elapsed();

    let start_3x = Instant::now();
    let _result_3x =
        retry_with_backoff(|| async { Err::<(), _>("fail") }, config_3x, |_| true).await;
    let elapsed_3x = start_3x.elapsed();

    // 2x: 10ms + 20ms = 30ms
    // 3x: 10ms + 30ms = 40ms
    assert!(elapsed_3x > elapsed_2x, "3x multiplier should take longer");
}

// ========================================================================
// Test 6-7: Transient vs Permanent Error Tests
// ========================================================================

#[tokio::test]
async fn test_retry_only_on_transient_errors() {
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        multiplier: 2.0,
        jitter: false,
    };

    let result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("timeout") // Simulate transient timeout
            }
        },
        config,
        |e| e.contains("timeout"), // Only retry timeouts
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        5,
        "Transient error should retry max times"
    );
}

#[tokio::test]
async fn test_no_retry_on_permanent_errors() {
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        multiplier: 2.0,
        jitter: false,
    };

    let result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("connection refused") // Permanent error
            }
        },
        config,
        |e| e.contains("timeout"), // Only retry timeouts
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "Permanent error should fail immediately"
    );
}

// ========================================================================
// Test 8-9: Success Cases
// ========================================================================

#[tokio::test]
async fn test_eventual_success_after_retries() {
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        multiplier: 2.0,
        jitter: false,
    };

    let result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst);
                if count < 3 {
                    Err("not yet") // Fail 3 times
                } else {
                    Ok(42) // Succeed on 4th attempt
                }
            }
        },
        config,
        |_| true,
    )
    .await;

    assert_eq!(result.unwrap(), 42, "Should eventually succeed");
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        4,
        "Should have tried 4 times"
    );
}

#[tokio::test]
async fn test_eventual_failure_after_exhausting_retries() {
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        multiplier: 2.0,
        jitter: false,
    };

    let result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("persistent failure")
            }
        },
        config,
        |_| true,
    )
    .await;

    assert!(result.is_err(), "Should fail after exhausting retries");
    assert_eq!(result.unwrap_err(), "persistent failure");
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        3,
        "Should have tried max times"
    );
}

// ========================================================================
// Test 10: Retry Counter Tests
// ========================================================================

#[tokio::test]
async fn test_retry_counter_increments_correctly() {
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig {
        max_attempts: 4,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        multiplier: 2.0,
        jitter: false,
    };

    let result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst);
                if count == 3 {
                    Ok(count) // Return the count to verify
                } else {
                    Err("not yet")
                }
            }
        },
        config,
        |_| true,
    )
    .await;

    assert_eq!(result.unwrap(), 3, "Counter should be 3 (0-indexed)");
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        4,
        "Total attempts should be 4"
    );
}

// ========================================================================
// Test 11-12: Timing Template Tests
// ========================================================================

#[tokio::test]
async fn test_paranoid_template_slow_retries() {
    let config = RetryConfig::paranoid();
    assert_eq!(config.max_attempts, 5, "Paranoid should have 5 attempts");
    assert_eq!(config.initial_delay, Duration::from_secs(1));
    assert_eq!(config.max_delay, Duration::from_secs(30));
    assert!(config.jitter, "Paranoid should use jitter");
}

#[tokio::test]
async fn test_insane_template_no_retries() {
    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig::insane(); // No retries

    let result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("fail")
            }
        },
        config,
        |_| true,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "Insane template should not retry"
    );
}

#[tokio::test]
async fn test_normal_template_balanced() {
    let config = RetryConfig::normal();
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.initial_delay, Duration::from_millis(100));
    assert_eq!(config.max_delay, Duration::from_secs(5));
}

#[tokio::test]
async fn test_aggressive_template_no_jitter() {
    let config = RetryConfig::aggressive();
    assert_eq!(config.max_attempts, 2);
    assert!(!config.jitter, "Aggressive should not use jitter for speed");
}
