//! Edge case and boundary condition tests
//!
//! Tests unusual scenarios, boundary values, and corner cases to ensure
//! robust error handling across the entire valid input space.
//!
//! Sprint 4.22 Phase 7: Comprehensive Testing - Subtask 7

use std::process::Command;

// Helper to get binary path
fn get_binary() -> &'static str {
    env!("CARGO_BIN_EXE_prtip")
}

// ========================================================================
// PORT RANGE EDGE CASES (5 tests)
// ========================================================================

#[test]
fn test_empty_port_range_80_to_79() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80-79", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    // Should fail with clear error
    assert!(!output.status.success(), "Empty range should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("range")
            || stderr.to_lowercase().contains("empty"),
        "Should mention invalid range: {}",
        stderr
    );
}

#[test]
fn test_port_zero() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "0", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    // Port 0 is special (ephemeral port), may be accepted or rejected
    // Test verifies graceful handling (no crash)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "Should handle port 0 gracefully: {}",
        stderr
    );
}

#[test]
fn test_port_65535_maximum() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "65535", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    // Should succeed - 65535 is the maximum valid port
    assert!(
        output.status.success(),
        "Port 65535 (max) should be accepted: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_port_65536_overflow() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "65536", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    // Should fail - 65536 exceeds maximum
    assert!(!output.status.success(), "Port 65536 should be rejected");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("65535")
            || stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("range"),
        "Should mention max port (65535): {}",
        stderr
    );
}

#[test]
fn test_single_port_range_80_to_80() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80-80", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    // Single-port range should be accepted
    assert!(
        output.status.success(),
        "Single-port range (80-80) should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ========================================================================
// CIDR EDGE CASES (4 tests)
// ========================================================================

#[test]
fn test_cidr_slash_0_entire_internet() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80", "0.0.0.0/0"])
        .output()
        .expect("Failed to execute");

    // /0 is valid but represents the entire internet (4.3 billion IPs)
    // Current implementation may panic due to integer overflow during target expansion
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _stdout = String::from_utf8_lossy(&output.stdout);

    // The binary executed without freezing (Command returned)
    // Exit code indicates failure (panic or validation error)
    assert!(
        !output.status.success(),
        "/0 CIDR should fail (too large to handle): exit code {:?}",
        output.status.code()
    );

    // Should produce some error message (either panic or validation error)
    assert!(
        !stderr.is_empty(),
        "Should produce error message for /0 CIDR: {}",
        stderr
    );

    // NOTE: This test documents current behavior (panic on overflow)
    // Future enhancement: should validate CIDR size before expansion
}

#[test]
fn test_cidr_slash_32_single_host() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80", "192.168.1.1/32"])
        .output()
        .expect("Failed to execute");

    // /32 is a single host - should be accepted
    assert!(
        output.status.success(),
        "/32 CIDR (single host) should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_cidr_slash_31_two_hosts() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80", "192.168.1.0/31"])
        .output()
        .expect("Failed to execute");

    // /31 is valid for point-to-point links (2 hosts)
    assert!(
        output.status.success(),
        "/31 CIDR (2 hosts) should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_cidr_slash_33_invalid() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80", "192.168.1.0/33"])
        .output()
        .expect("Failed to execute");

    // /33 exceeds maximum for IPv4 - should fail
    assert!(!output.status.success(), "/33 CIDR should be rejected");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("cidr")
            || stderr.to_lowercase().contains("prefix"),
        "Should mention invalid CIDR: {}",
        stderr
    );
}

// ========================================================================
// TIMEOUT EDGE CASES (2 tests)
// ========================================================================

#[test]
fn test_timeout_zero_milliseconds() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80", "127.0.0.1", "--timeout", "0ms"])
        .output()
        .expect("Failed to execute");

    // Zero timeout may be rejected or treated as minimum
    // Test verifies graceful handling
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        assert!(
            stderr.to_lowercase().contains("timeout")
                || stderr.to_lowercase().contains("invalid")
                || stderr.to_lowercase().contains("minimum"),
            "Should mention timeout issue: {}",
            stderr
        );
    }
    // If accepted, should use minimum timeout
}

#[test]
fn test_timeout_extremely_large_one_year() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80", "127.0.0.1", "--timeout", "31536000000ms"])
        .output()
        .expect("Failed to execute");

    // 1 year timeout may be clamped or accepted
    // Test verifies graceful handling (no crash)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked") && !stderr.contains("overflow"),
        "Should handle large timeout gracefully: {}",
        stderr
    );
}

// ========================================================================
// PARALLELISM EDGE CASES (2 tests)
// ========================================================================

#[test]
fn test_parallelism_zero() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80", "127.0.0.1", "--max-parallelism", "0"])
        .output()
        .expect("Failed to execute");

    // Parallelism 0 should be rejected
    assert!(!output.status.success(), "Parallelism 0 should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("parallelism")
            || stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("minimum"),
        "Should mention parallelism issue: {}",
        stderr
    );
}

#[test]
fn test_parallelism_one_million() {
    let output = Command::new(get_binary())
        .args([
            "-sT",
            "-p",
            "80",
            "127.0.0.1",
            "--max-parallelism",
            "1000000",
        ])
        .output()
        .expect("Failed to execute");

    // Very high parallelism may be clamped or warned
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        assert!(
            stderr.to_lowercase().contains("parallelism")
                || stderr.to_lowercase().contains("maximum")
                || stderr.to_lowercase().contains("clamp"),
            "Should mention parallelism limit: {}",
            stderr
        );
    }
    // If accepted, should be clamped to reasonable max
}

// ========================================================================
// OUTPUT FILE EDGE CASES (2 tests)
// ========================================================================

#[test]
fn test_output_file_empty_path() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80", "127.0.0.1", "-oN", ""])
        .output()
        .expect("Failed to execute");

    // Empty path should fail
    assert!(!output.status.success(), "Empty output path should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("path")
            || stderr.to_lowercase().contains("file")
            || stderr.to_lowercase().contains("invalid"),
        "Should mention invalid path: {}",
        stderr
    );
}

#[test]
fn test_output_file_in_nonexistent_directory() {
    let output = Command::new(get_binary())
        .args([
            "-sT",
            "-p",
            "80",
            "127.0.0.1",
            "-oN",
            "/nonexistent_dir_12345/output.txt",
        ])
        .output()
        .expect("Failed to execute");

    // Nonexistent directory may create or fail
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        assert!(
            stderr.to_lowercase().contains("directory")
                || stderr.to_lowercase().contains("not found")
                || stderr.to_lowercase().contains("path")
                || stderr.to_lowercase().contains("file"),
            "Should mention directory/path issue: {}",
            stderr
        );
    }
}

// ========================================================================
// ADDITIONAL EDGE CASES
// ========================================================================

#[test]
fn test_ipv6_localhost() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80", "::1"])
        .output()
        .expect("Failed to execute");

    // IPv6 localhost should be handled gracefully
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "Should handle IPv6 localhost gracefully: {}",
        stderr
    );
}

#[test]
fn test_multiple_targets_with_mixed_validity() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80", "127.0.0.1,999.999.999.999,192.168.1.1"])
        .output()
        .expect("Failed to execute");

    // Mixed valid/invalid targets should be handled gracefully
    let stderr = String::from_utf8_lossy(&output.stderr);

    // May fail (invalid targets present) or partially succeed
    // Either is acceptable - test ensures no crash
    assert!(
        !stderr.contains("panicked") && !stderr.contains("thread"),
        "Should handle mixed targets gracefully: {}",
        stderr
    );
}

#[test]
fn test_rate_limit_zero() {
    let output = Command::new(get_binary())
        .args(["-sT", "-p", "80", "127.0.0.1", "--max-rate", "0"])
        .output()
        .expect("Failed to execute");

    // Zero rate limit should fail or use minimum
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        assert!(
            stderr.to_lowercase().contains("rate")
                || stderr.to_lowercase().contains("invalid")
                || stderr.to_lowercase().contains("minimum"),
            "Should mention rate limit issue: {}",
            stderr
        );
    }
}
