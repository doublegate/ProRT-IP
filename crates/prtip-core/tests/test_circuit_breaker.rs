//! Circuit breaker integration tests
//!
//! Tests the circuit breaker pattern implementation for failing targets.
//! Complements the unit tests in circuit_breaker.rs with integration scenarios.
//!
//! Sprint 4.22 Phase 7: Comprehensive Testing - Subtask 2

use prtip_core::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use std::net::IpAddr;
use std::time::Duration;
use tokio::time::sleep;

// ========================================================================
// Test 1-4: State Transition Tests
// ========================================================================

#[tokio::test]
async fn test_state_transition_closed_to_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "192.168.1.10".parse().unwrap();

    // Initially CLOSED
    let stats = breaker.get_stats(target).await;
    assert!(stats.is_none() || stats.unwrap().state == CircuitState::Closed);

    // Record exactly 4 failures - should stay CLOSED
    for i in 0..4 {
        breaker.record_failure(target).await;
        let stats = breaker.get_stats(target).await.unwrap();
        assert_eq!(
            stats.state,
            CircuitState::Closed,
            "Circuit opened prematurely after {} failures",
            i + 1
        );
    }

    // 5th failure should open circuit
    breaker.record_failure(target).await;
    let stats = breaker.get_stats(target).await.unwrap();
    assert_eq!(stats.state, CircuitState::Open);
    assert_eq!(stats.failure_count, 5);
    assert!(stats.opened_at.is_some());
}

#[tokio::test]
async fn test_state_transition_open_to_half_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_millis(100),
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "10.0.0.1".parse().unwrap();

    // Open circuit
    for _ in 0..3 {
        breaker.record_failure(target).await;
    }
    assert_eq!(
        breaker.get_stats(target).await.unwrap().state,
        CircuitState::Open
    );

    // Before timeout expires - should stay OPEN
    assert!(!breaker.should_attempt(target).await);
    assert_eq!(
        breaker.get_stats(target).await.unwrap().state,
        CircuitState::Open
    );

    // After timeout expires - should transition to HALF_OPEN
    sleep(Duration::from_millis(120)).await;
    assert!(breaker.should_attempt(target).await);
    let stats = breaker.get_stats(target).await.unwrap();
    assert_eq!(stats.state, CircuitState::HalfOpen);
    assert_eq!(stats.success_count, 0);
    assert_eq!(stats.failure_count, 0); // Counters reset
}

#[tokio::test]
async fn test_state_transition_half_open_to_closed() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "172.16.0.1".parse().unwrap();

    // Open circuit
    breaker.record_failure(target).await;
    breaker.record_failure(target).await;

    // Transition to HALF_OPEN
    sleep(Duration::from_millis(60)).await;
    breaker.should_attempt(target).await;

    // First success - should stay HALF_OPEN
    breaker.record_success(target).await;
    assert_eq!(
        breaker.get_stats(target).await.unwrap().state,
        CircuitState::HalfOpen
    );
    assert_eq!(breaker.get_stats(target).await.unwrap().success_count, 1);

    // Second success - should close circuit
    breaker.record_success(target).await;
    let stats = breaker.get_stats(target).await.unwrap();
    assert_eq!(stats.state, CircuitState::Closed);
    assert_eq!(stats.success_count, 2);
    assert!(stats.opened_at.is_none());
}

#[tokio::test]
async fn test_state_transition_half_open_to_open_on_failure() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "10.1.1.1".parse().unwrap();

    // Open circuit
    breaker.record_failure(target).await;
    breaker.record_failure(target).await;
    let first_opened_at = breaker.get_stats(target).await.unwrap().opened_at;

    // Transition to HALF_OPEN
    sleep(Duration::from_millis(60)).await;
    breaker.should_attempt(target).await;
    assert_eq!(
        breaker.get_stats(target).await.unwrap().state,
        CircuitState::HalfOpen
    );

    // Failure in HALF_OPEN should re-open circuit
    breaker.record_failure(target).await;
    let stats = breaker.get_stats(target).await.unwrap();
    assert_eq!(stats.state, CircuitState::Open);
    assert_eq!(stats.failure_count, 1); // Reset to 1
    assert!(stats.opened_at.is_some());
    assert!(stats.opened_at > first_opened_at); // New timestamp
}

// ========================================================================
// Test 5-7: Threshold Detection Tests
// ========================================================================

#[tokio::test]
async fn test_exactly_threshold_failures_opens_circuit() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "192.168.2.100".parse().unwrap();

    // Exactly 5 failures
    for i in 0..5 {
        breaker.record_failure(target).await;
        let stats = breaker.get_stats(target).await.unwrap();
        if i < 4 {
            assert_eq!(stats.state, CircuitState::Closed);
        } else {
            assert_eq!(stats.state, CircuitState::Open);
        }
    }

    assert!(!breaker.should_attempt(target).await);
}

#[tokio::test]
async fn test_one_less_than_threshold_keeps_closed() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "192.168.2.101".parse().unwrap();

    // 4 failures (one less than threshold)
    for _ in 0..4 {
        breaker.record_failure(target).await;
    }

    let stats = breaker.get_stats(target).await.unwrap();
    assert_eq!(stats.state, CircuitState::Closed);
    assert_eq!(stats.failure_count, 4);
    assert!(breaker.should_attempt(target).await);
}

#[tokio::test]
async fn test_custom_threshold_configuration() {
    let config = CircuitBreakerConfig {
        failure_threshold: 10, // Custom threshold
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "10.2.2.2".parse().unwrap();

    // 9 failures - should stay CLOSED
    for _ in 0..9 {
        breaker.record_failure(target).await;
    }
    assert_eq!(
        breaker.get_stats(target).await.unwrap().state,
        CircuitState::Closed
    );

    // 10th failure - should OPEN
    breaker.record_failure(target).await;
    assert_eq!(
        breaker.get_stats(target).await.unwrap().state,
        CircuitState::Open
    );
}

// ========================================================================
// Test 8-10: Cooldown Timing Tests
// ========================================================================

#[tokio::test]
async fn test_cooldown_before_expiry_stays_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        timeout: Duration::from_millis(200),
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "192.168.3.1".parse().unwrap();

    // Open circuit
    breaker.record_failure(target).await;
    breaker.record_failure(target).await;

    // Check immediately - should be OPEN
    assert!(!breaker.should_attempt(target).await);

    // Check at 100ms (halfway through cooldown) - should still be OPEN
    sleep(Duration::from_millis(100)).await;
    assert!(!breaker.should_attempt(target).await);
    assert_eq!(
        breaker.get_stats(target).await.unwrap().state,
        CircuitState::Open
    );
}

#[tokio::test]
async fn test_cooldown_after_expiry_transitions_half_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        timeout: Duration::from_millis(100),
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "192.168.3.2".parse().unwrap();

    // Open circuit
    breaker.record_failure(target).await;
    breaker.record_failure(target).await;

    // Wait for cooldown to expire
    sleep(Duration::from_millis(120)).await;

    // Should transition to HALF_OPEN
    assert!(breaker.should_attempt(target).await);
    assert_eq!(
        breaker.get_stats(target).await.unwrap().state,
        CircuitState::HalfOpen
    );
}

#[tokio::test]
async fn test_custom_cooldown_duration() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        timeout: Duration::from_millis(500), // Long cooldown
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "10.3.3.3".parse().unwrap();

    // Open circuit
    breaker.record_failure(target).await;
    breaker.record_failure(target).await;

    // 250ms - should still be OPEN
    sleep(Duration::from_millis(250)).await;
    assert!(!breaker.should_attempt(target).await);

    // 550ms - should be HALF_OPEN
    sleep(Duration::from_millis(300)).await;
    assert!(breaker.should_attempt(target).await);
}

// ========================================================================
// Test 11-12: Success Counter Tests
// ========================================================================

#[tokio::test]
async fn test_success_in_half_open_increments_counter() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 3,
        timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "172.20.0.1".parse().unwrap();

    // Open circuit and transition to HALF_OPEN
    breaker.record_failure(target).await;
    breaker.record_failure(target).await;
    sleep(Duration::from_millis(60)).await;
    breaker.should_attempt(target).await;

    // Record successes and verify counter increments
    assert_eq!(breaker.get_stats(target).await.unwrap().success_count, 0);

    breaker.record_success(target).await;
    assert_eq!(breaker.get_stats(target).await.unwrap().success_count, 1);

    breaker.record_success(target).await;
    assert_eq!(breaker.get_stats(target).await.unwrap().success_count, 2);
}

#[tokio::test]
async fn test_multiple_successes_required_to_close() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 3, // Require 3 successes
        timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "172.20.0.2".parse().unwrap();

    // Open and transition to HALF_OPEN
    breaker.record_failure(target).await;
    breaker.record_failure(target).await;
    sleep(Duration::from_millis(60)).await;
    breaker.should_attempt(target).await;

    // 1 success - should stay HALF_OPEN
    breaker.record_success(target).await;
    assert_eq!(
        breaker.get_stats(target).await.unwrap().state,
        CircuitState::HalfOpen
    );

    // 2 successes - should still be HALF_OPEN
    breaker.record_success(target).await;
    assert_eq!(
        breaker.get_stats(target).await.unwrap().state,
        CircuitState::HalfOpen
    );

    // 3rd success - should close
    breaker.record_success(target).await;
    assert_eq!(
        breaker.get_stats(target).await.unwrap().state,
        CircuitState::Closed
    );
}

// ========================================================================
// Test 13: Concurrent Access Test
// ========================================================================

#[tokio::test]
async fn test_concurrent_access_no_data_races() {
    let breaker = std::sync::Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 10,
        ..Default::default()
    }));
    let target: IpAddr = "192.168.100.1".parse().unwrap();

    // Spawn 10 tasks that each record 5 failures concurrently
    let mut handles = vec![];
    for _ in 0..10 {
        let breaker_clone = breaker.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..5 {
                breaker_clone.record_failure(target).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Should have 50 total failures (10 tasks × 5 failures)
    // Circuit should be OPEN (threshold 10)
    let stats = breaker.get_stats(target).await.unwrap();
    assert_eq!(stats.state, CircuitState::Open);
    assert!(stats.failure_count >= 10); // At least threshold reached
}

// ========================================================================
// Test 14: Per-Target Tracking Test
// ========================================================================

#[tokio::test]
async fn test_per_target_independent_circuits() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target1: IpAddr = "192.168.1.1".parse().unwrap();
    let target2: IpAddr = "192.168.1.2".parse().unwrap();
    let target3: IpAddr = "192.168.1.3".parse().unwrap();

    // Open target1 circuit
    breaker.record_failure(target1).await;
    breaker.record_failure(target1).await;
    breaker.record_failure(target1).await;

    // Record 2 failures for target2 (not enough to open)
    breaker.record_failure(target2).await;
    breaker.record_failure(target2).await;

    // target3 has no failures

    // Verify independent states
    assert_eq!(
        breaker.get_stats(target1).await.unwrap().state,
        CircuitState::Open
    );
    assert_eq!(
        breaker.get_stats(target2).await.unwrap().state,
        CircuitState::Closed
    );
    assert!(
        breaker.get_stats(target3).await.is_none()
            || breaker.get_stats(target3).await.unwrap().state == CircuitState::Closed
    );

    // Verify independent counters
    assert_eq!(breaker.get_stats(target1).await.unwrap().failure_count, 3);
    assert_eq!(breaker.get_stats(target2).await.unwrap().failure_count, 2);

    // Verify should_attempt returns correct values
    assert!(!breaker.should_attempt(target1).await);
    assert!(breaker.should_attempt(target2).await);
    assert!(breaker.should_attempt(target3).await);
}

// ========================================================================
// Test 15: Metrics Accuracy Tests
// ========================================================================

#[tokio::test]
async fn test_failure_count_accurate() {
    let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
    let target: IpAddr = "10.10.10.10".parse().unwrap();

    for i in 1..=4 {
        breaker.record_failure(target).await;
        let stats = breaker.get_stats(target).await.unwrap();
        assert_eq!(
            stats.failure_count, i,
            "Failure count mismatch at iteration {}",
            i
        );
    }
}

#[tokio::test]
async fn test_success_count_accurate() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 5,
        timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "10.10.10.11".parse().unwrap();

    // Open and transition to HALF_OPEN
    breaker.record_failure(target).await;
    breaker.record_failure(target).await;
    sleep(Duration::from_millis(60)).await;
    breaker.should_attempt(target).await;

    // Record successes and verify count
    for i in 1..=4 {
        breaker.record_success(target).await;
        let stats = breaker.get_stats(target).await.unwrap();
        assert_eq!(
            stats.success_count, i,
            "Success count mismatch at iteration {}",
            i
        );
    }
}

#[tokio::test]
async fn test_last_failure_timestamp_updated() {
    let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
    let target: IpAddr = "10.10.10.12".parse().unwrap();

    // No failures yet
    assert!(breaker.get_stats(target).await.is_none());

    // First failure
    breaker.record_failure(target).await;
    let first_timestamp = breaker.get_stats(target).await.unwrap().last_failure;
    assert!(first_timestamp.is_some());

    // Wait and record another failure
    sleep(Duration::from_millis(10)).await;
    breaker.record_failure(target).await;
    let second_timestamp = breaker.get_stats(target).await.unwrap().last_failure;
    assert!(second_timestamp.is_some());
    assert!(
        second_timestamp > first_timestamp,
        "Timestamp should be updated"
    );
}

#[tokio::test]
async fn test_opened_at_timestamp_set() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    let target: IpAddr = "10.10.10.13".parse().unwrap();

    // Before opening - no timestamp
    breaker.record_failure(target).await;
    assert!(breaker.get_stats(target).await.unwrap().opened_at.is_none());

    // After opening - timestamp set
    breaker.record_failure(target).await;
    let stats = breaker.get_stats(target).await.unwrap();
    assert_eq!(stats.state, CircuitState::Open);
    assert!(stats.opened_at.is_some());
}
