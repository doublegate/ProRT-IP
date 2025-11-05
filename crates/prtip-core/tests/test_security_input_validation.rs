// Sprint 5.6 Phase 4: Security & Edge Case Tests
// Input Validation Testing
//
// Test Strategy:
// - Group 1: Configuration boundary tests (no root required)
// - Group 2: TOML parsing security (no root required)
// - Group 3: Overflow and underflow prevention (no root required)
//
// Run all tests: cargo test --test test_security_input_validation

use prtip_core::{Config, Error};

/// Helper to create default config for testing
fn default_config() -> Config {
    Config::default()
}

// ============================================================================
// Test Group 1: Timeout Validation (3 tests)
// Tests timeout boundaries to prevent DoS and overflow attacks
// ============================================================================

/// Tests that zero timeout is rejected
///
/// **Attack Scenario:** Attacker sets timeout_ms = 0 to cause division by zero
/// or infinite wait loops.
///
/// **Expected Behavior:** Config validation rejects zero timeout with clear error.
///
/// **Failure Impact:** HIGH - Could cause crashes (division by zero) or hangs
/// (infinite loops waiting for timeout).
///
/// **Mitigation:** Config::validate() explicitly checks timeout_ms > 0.
#[test]
fn test_security_validate_timeout_zero() {
    let mut config = default_config();
    config.scan.timeout_ms = 0;

    let result = config.validate();
    assert!(
        result.is_err(),
        "Should reject zero timeout (prevents division by zero)"
    );

    if let Err(Error::Config(msg)) = result {
        assert!(
            msg.contains("timeout_ms") && msg.contains("greater than 0"),
            "Error should mention timeout_ms requirement, got: {}",
            msg
        );
    } else {
        panic!("Expected Error::Config, got: {:?}", result);
    }
}

/// Tests that extremely large timeout is rejected
///
/// **Attack Scenario:** Attacker sets timeout_ms = u64::MAX to cause integer
/// overflow in timeout calculations or force indefinite hangs.
///
/// **Expected Behavior:** Config validation rejects timeout > 1 hour (3,600,000 ms).
///
/// **Failure Impact:** HIGH - Could cause resource exhaustion by holding
/// connections open indefinitely.
///
/// **Mitigation:** Config::validate() enforces max timeout of 1 hour.
#[test]
fn test_security_validate_timeout_overflow() {
    let mut config = default_config();
    config.scan.timeout_ms = u64::MAX; // Maximum possible value

    let result = config.validate();
    assert!(
        result.is_err(),
        "Should reject timeout overflow (prevents indefinite hangs)"
    );

    if let Err(Error::Config(msg)) = result {
        assert!(
            msg.contains("timeout_ms") && msg.contains("cannot exceed"),
            "Error should mention timeout_ms limit, got: {}",
            msg
        );
    } else {
        panic!("Expected Error::Config, got: {:?}", result);
    }
}

/// Tests that timeout at upper bound is rejected
///
/// **Attack Scenario:** Attacker sets timeout_ms = 3,600,001 (just over 1 hour).
///
/// **Expected Behavior:** Config validation rejects timeout > 1 hour.
///
/// **Failure Impact:** MEDIUM - Not as severe as u64::MAX but still unreasonable.
///
/// **Mitigation:** Config::validate() enforces max timeout of 1 hour.
#[test]
fn test_security_validate_timeout_upper_bound() {
    let mut config = default_config();
    config.scan.timeout_ms = 3_600_001; // 1 hour + 1 millisecond

    let result = config.validate();
    assert!(
        result.is_err(),
        "Should reject timeout > 1 hour (prevents resource exhaustion)"
    );
}

// ============================================================================
// Test Group 2: Retry Validation (1 test)
// Tests retry boundaries to prevent resource exhaustion
// ============================================================================

/// Tests that excessive retries are rejected
///
/// **Attack Scenario:** Attacker sets retries = u32::MAX to cause infinite
/// retry loops and resource exhaustion.
///
/// **Expected Behavior:** Config validation rejects retries > 10.
///
/// **Failure Impact:** HIGH - Could cause indefinite scanning, resource
/// exhaustion, and DoS of target hosts.
///
/// **Mitigation:** Config::validate() enforces max retries of 10.
#[test]
fn test_security_validate_retries_overflow() {
    let mut config = default_config();
    config.scan.retries = u32::MAX; // Maximum possible value

    let result = config.validate();
    assert!(
        result.is_err(),
        "Should reject retry overflow (prevents infinite retries)"
    );

    if let Err(Error::Config(msg)) = result {
        assert!(
            msg.contains("retries") && msg.contains("cannot exceed"),
            "Error should mention retries limit, got: {}",
            msg
        );
    } else {
        panic!("Expected Error::Config, got: {:?}", result);
    }
}

// ============================================================================
// Test Group 3: Performance Parameter Validation (2 tests)
// Tests parallelism and rate limiting boundaries
// ============================================================================

/// Tests that excessive parallelism is rejected
///
/// **Attack Scenario:** Attacker sets parallelism = 1,000,000 to cause
/// memory exhaustion by spawning millions of threads/tasks.
///
/// **Expected Behavior:** Config validation rejects parallelism > 100,000.
///
/// **Failure Impact:** CRITICAL - Could cause OOM crashes by allocating
/// memory for millions of concurrent tasks.
///
/// **Mitigation:** Config::validate() enforces max parallelism of 100,000.
#[test]
fn test_security_validate_parallelism_overflow() {
    let mut config = default_config();
    config.performance.parallelism = 1_000_000; // 1 million concurrent tasks

    let result = config.validate();
    assert!(
        result.is_err(),
        "Should reject parallelism overflow (prevents memory exhaustion)"
    );

    if let Err(Error::Config(msg)) = result {
        assert!(
            msg.contains("parallelism") && msg.contains("cannot exceed"),
            "Error should mention parallelism limit, got: {}",
            msg
        );
    } else {
        panic!("Expected Error::Config, got: {:?}", result);
    }
}

/// Tests that zero max_rate is rejected
///
/// **Attack Scenario:** Attacker sets max_rate = 0 to cause division by zero
/// in rate limiter calculations.
///
/// **Expected Behavior:** Config validation rejects max_rate = 0.
///
/// **Failure Impact:** HIGH - Could cause crashes (division by zero) or
/// incorrect rate limiting.
///
/// **Mitigation:** Config::validate() explicitly checks max_rate > 0 if set.
#[test]
fn test_security_validate_max_rate_zero() {
    let mut config = default_config();
    config.performance.max_rate = Some(0); // Zero rate (invalid)

    let result = config.validate();
    assert!(
        result.is_err(),
        "Should reject zero max_rate (prevents division by zero)"
    );

    if let Err(Error::Config(msg)) = result {
        assert!(
            msg.contains("max_rate") && msg.contains("greater than 0"),
            "Error should mention max_rate requirement, got: {}",
            msg
        );
    } else {
        panic!("Expected Error::Config, got: {:?}", result);
    }
}

/// Tests that excessive max_rate is rejected
///
/// **Attack Scenario:** Attacker sets max_rate = u64::MAX to cause integer
/// overflow in rate calculations or unrealistic packet rates.
///
/// **Expected Behavior:** Config validation rejects max_rate > 100M pps.
///
/// **Failure Impact:** MEDIUM - Could cause overflow in rate calculations or
/// attempt impossible packet rates.
///
/// **Mitigation:** Config::validate() enforces max rate of 100M pps.
#[test]
fn test_security_validate_max_rate_overflow() {
    let mut config = default_config();
    config.performance.max_rate = Some(u32::MAX); // Maximum possible value

    let result = config.validate();
    assert!(
        result.is_err(),
        "Should reject max_rate overflow (prevents calculation errors)"
    );

    if let Err(Error::Config(msg)) = result {
        assert!(
            msg.contains("max_rate") && msg.contains("cannot exceed"),
            "Error should mention max_rate limit, got: {}",
            msg
        );
    } else {
        panic!("Expected Error::Config, got: {:?}", result);
    }
}

// ============================================================================
// Test Group 4: Valid Edge Cases (Positive Tests) (1 test)
// Verifies that valid boundary values are accepted
// ============================================================================

/// Tests that valid boundary values are accepted
///
/// **Purpose:** Ensure validation doesn't reject valid configurations.
///
/// **Test Cases:**
/// - timeout_ms = 1 (minimum valid)
/// - timeout_ms = 3,600,000 (maximum valid, exactly 1 hour)
/// - retries = 0 (minimum valid, no retries)
/// - retries = 10 (maximum valid)
/// - parallelism = 0 (valid, means adaptive)
/// - parallelism = 100,000 (maximum valid)
///
/// **Expected Behavior:** All valid boundary values should pass validation.
#[test]
fn test_security_validate_valid_boundaries() {
    // Test minimum timeout (1ms)
    let mut config = default_config();
    config.scan.timeout_ms = 1;
    assert!(
        config.validate().is_ok(),
        "Should accept minimum timeout (1ms)"
    );

    // Test maximum timeout (1 hour exactly)
    let mut config = default_config();
    config.scan.timeout_ms = 3_600_000;
    assert!(
        config.validate().is_ok(),
        "Should accept maximum timeout (1 hour)"
    );

    // Test minimum retries (0)
    let mut config = default_config();
    config.scan.retries = 0;
    assert!(config.validate().is_ok(), "Should accept zero retries");

    // Test maximum retries (10)
    let mut config = default_config();
    config.scan.retries = 10;
    assert!(
        config.validate().is_ok(),
        "Should accept maximum retries (10)"
    );

    // Test adaptive parallelism (0 = auto-detect)
    let mut config = default_config();
    config.performance.parallelism = 0;
    assert!(
        config.validate().is_ok(),
        "Should accept parallelism = 0 (adaptive)"
    );

    // Test maximum parallelism (100,000)
    let mut config = default_config();
    config.performance.parallelism = 100_000;
    assert!(
        config.validate().is_ok(),
        "Should accept maximum parallelism (100,000)"
    );

    // Test minimum max_rate (1 pps)
    let mut config = default_config();
    config.performance.max_rate = Some(1);
    assert!(
        config.validate().is_ok(),
        "Should accept minimum max_rate (1 pps)"
    );

    // Test maximum max_rate (100M pps)
    let mut config = default_config();
    config.performance.max_rate = Some(100_000_000);
    assert!(
        config.validate().is_ok(),
        "Should accept maximum max_rate (100M pps)"
    );
}

// ============================================================================
// Test Group 5: Combined Validation (1 test)
// Tests that multiple invalid values are all reported
// ============================================================================

/// Tests that validation catches multiple errors
///
/// **Purpose:** Ensure comprehensive validation when multiple fields are invalid.
///
/// **Test Case:** Config with multiple invalid values (zero timeout, excessive
/// retries, overflow parallelism).
///
/// **Expected Behavior:** Validation should fail on first error encountered
/// (fail-fast approach).
#[test]
fn test_security_validate_multiple_errors_fail_fast() {
    let mut config = default_config();
    config.scan.timeout_ms = 0; // Invalid
    config.scan.retries = 100; // Invalid
    config.performance.parallelism = 1_000_000; // Invalid

    let result = config.validate();
    assert!(
        result.is_err(),
        "Should reject config with multiple invalid values"
    );

    // Should fail on first error (timeout_ms = 0)
    if let Err(Error::Config(msg)) = result {
        assert!(
            msg.contains("timeout_ms"),
            "Should fail fast on first error (timeout_ms), got: {}",
            msg
        );
    } else {
        panic!("Expected Error::Config, got: {:?}", result);
    }
}
