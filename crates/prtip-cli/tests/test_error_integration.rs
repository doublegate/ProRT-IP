//! End-to-end CLI error handling integration tests
//!
//! Tests complete error scenarios from CLI invocation through error display.
//! Uses actual binary execution to ensure user-facing behavior is correct.
//!
//! Sprint 4.22 Phase 7: Comprehensive Testing - Subtask 6

use std::process::Command;

// Helper to get binary path
fn get_binary() -> &'static str {
    env!("CARGO_BIN_EXE_prtip")
}

// Helper to create a command with history disabled (prevents race conditions in tests)
fn create_test_command() -> Command {
    let mut cmd = Command::new(get_binary());
    cmd.env("PRTIP_DISABLE_HISTORY", "1");
    cmd
}

// ========================================================================
// INPUT VALIDATION (5 tests)
// ========================================================================

#[test]
fn test_invalid_ip_address_999() {
    let output = create_test_command()
        .args(["-sT", "-p", "80", "999.999.999.999"])
        .output()
        .expect("Failed to execute");

    // Should fail
    assert!(!output.status.success(), "Invalid IP should fail");
    assert_eq!(output.status.code(), Some(1), "Should exit with code 1");

    // Error message should be clear
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("invalid") || stderr.to_lowercase().contains("parse"),
        "Should mention invalid input: {}",
        stderr
    );

    // Should NOT contain stack traces
    assert!(!stderr.contains("backtrace"), "No stack trace");
    assert!(!stderr.contains("panicked at"), "No panic message");
}

#[test]
fn test_invalid_port_range_reversed() {
    let output = create_test_command()
        .args(["-sT", "-p", "80-79", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success(), "Invalid port range should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Clap might catch this early or parser catches it
    assert!(
        stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("range")
            || stderr.to_lowercase().contains("error"),
        "Should mention invalid range: {}",
        stderr
    );
}

#[test]
fn test_port_overflow_65537() {
    let output = create_test_command()
        .args(["-sT", "-p", "65537", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success(), "Port overflow should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should mention the max port or overflow
    assert!(
        stderr.contains("65535")
            || stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("range"),
        "Should mention valid port range: {}",
        stderr
    );
}

#[test]
fn test_invalid_cidr_prefix_33() {
    let output = create_test_command()
        .args(["-sT", "-p", "80", "192.168.1.0/33"])
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success(), "Invalid CIDR should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("cidr")
            || stderr.to_lowercase().contains("parse"),
        "Should mention invalid CIDR: {}",
        stderr
    );
}

#[test]
fn test_empty_port_specification() {
    let output = create_test_command()
        .args(["-sT", "-p", "", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success(), "Empty port should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("required")
            || stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("empty"),
        "Should mention required/invalid: {}",
        stderr
    );
}

// ========================================================================
// PERMISSION ERRORS (3 tests)
// ========================================================================

#[test]
#[cfg(unix)]
fn test_syn_scan_without_root() {
    // Skip if running as root
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("Skipping: running as root");
        return;
    }

    let output = create_test_command()
        .args(["-sS", "-p", "80", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    // May fail OR warn in stderr (implementation-dependent)
    let stderr = String::from_utf8_lossy(&output.stderr);

    // If it fails, should mention permission
    if !output.status.success() {
        assert!(
            stderr.to_lowercase().contains("permission")
                || stderr.to_lowercase().contains("privilege")
                || stderr.to_lowercase().contains("root")
                || stderr.to_lowercase().contains("cap_net_raw")
                || stderr.to_lowercase().contains("administrator")
                || stderr.to_lowercase().contains("error"),
            "Should mention permission issue if failing: {}",
            stderr
        );
    }
    // If it succeeds, it might use fallback or elevated privileges
    // Test just verifies graceful handling (no panic)
}

#[test]
#[cfg(unix)]
fn test_fin_scan_without_root() {
    // Skip if running as root
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("Skipping: running as root");
        return;
    }

    let output = create_test_command()
        .args(["-sF", "-p", "80", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    // May fail OR warn in stderr (implementation-dependent)
    let stderr = String::from_utf8_lossy(&output.stderr);

    // If it fails, should mention permission
    if !output.status.success() {
        assert!(
            stderr.to_lowercase().contains("permission")
                || stderr.to_lowercase().contains("privilege")
                || stderr.to_lowercase().contains("root")
                || stderr.to_lowercase().contains("error"),
            "Should mention permission issue if failing: {}",
            stderr
        );
    }
    // Test just verifies graceful handling (no panic)
}

#[test]
fn test_write_to_readonly_directory() {
    #[cfg(unix)]
    let readonly_path = "/dev/null/output.txt"; // /dev/null is not a directory
    #[cfg(windows)]
    let readonly_path = "C:\\Windows\\System32\\output.txt"; // Typically restricted

    let output = create_test_command()
        .args(["-sT", "-p", "80", "127.0.0.1", "-oN", readonly_path])
        .output()
        .expect("Failed to execute");

    // May fail during arg parsing or during write attempt
    // We expect failure in either case
    assert!(
        !output.status.success() || output.status.code() == Some(1),
        "Write to readonly should fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Might fail at different stages, so check for any error indication
    assert!(
        stderr.to_lowercase().contains("permission")
            || stderr.to_lowercase().contains("denied")
            || stderr.to_lowercase().contains("directory")
            || stderr.to_lowercase().contains("error"),
        "Should mention error: {}",
        stderr
    );
}

// ========================================================================
// NETWORK FAILURES (4 tests)
// ========================================================================

#[test]
fn test_scan_unreachable_network() {
    // TEST-NET-1 (RFC 5737) - reserved for documentation, should be unreachable
    let output = create_test_command()
        .args(["-sT", "-p", "80", "192.0.2.1", "--timeout", "100ms"])
        .output()
        .expect("Failed to execute");

    // May succeed with 0 results or fail with error
    // Either is acceptable for unreachable network
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should not panic or crash
    assert!(
        !stderr.contains("panicked"),
        "Should handle unreachable network gracefully"
    );
}

#[test]
fn test_connection_timeout_short_timeout() {
    let output = create_test_command()
        .args([
            "-sT",
            "-p",
            "80",
            "192.0.2.1", // TEST-NET-1
            "--timeout",
            "1ms", // Very short timeout
        ])
        .output()
        .expect("Failed to execute");

    // Should complete (may timeout)
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should handle timeouts gracefully (not crash)
    assert!(
        !stderr.contains("thread")
            && !stderr.contains("panicked")
            && !stderr.contains("RUST_BACKTRACE"),
        "Should handle timeout gracefully"
    );
}

#[test]
fn test_localhost_scan_succeeds() {
    let output = create_test_command()
        .args(["-sT", "-p", "1-10", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    // Should succeed (exit code 0)
    assert!(
        output.status.success(),
        "Localhost scan should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Should produce output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.is_empty() || !output.stderr.is_empty(),
        "Should produce output"
    );
}

#[test]
fn test_invalid_hostname_resolution() {
    let output = create_test_command()
        .args([
            "-sT",
            "-p",
            "80",
            "this-hostname-does-not-exist-12345.invalid",
        ])
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success(), "Invalid hostname should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("resolve")
            || stderr.to_lowercase().contains("dns")
            || stderr.to_lowercase().contains("name")
            || stderr.to_lowercase().contains("not found")
            || stderr.to_lowercase().contains("invalid"),
        "Should mention DNS/resolution failure: {}",
        stderr
    );
}

// ========================================================================
// CONFIGURATION ERRORS (3 tests)
// ========================================================================

#[test]
fn test_conflicting_scan_types() {
    let output = create_test_command()
        .args(["-sS", "-sT", "-p", "80", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    // Implementation may accept last flag or warn
    // Test verifies no crash/panic
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "Should handle multiple scan types gracefully (no panic)"
    );

    // Exit code may vary (success if accepts last flag, error if conflicts)
    // Either is acceptable behavior
}

#[test]
fn test_invalid_timing_template_t6() {
    let output = create_test_command()
        .args(["-sT", "-p", "80", "-T6", "127.0.0.1"])
        .output()
        .expect("Failed to execute");

    assert!(
        !output.status.success(),
        "Invalid timing template should fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("T0")
            || stderr.contains("T5")
            || stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("0-5"),
        "Should mention valid templates (T0-T5): {}",
        stderr
    );
}

#[test]
fn test_invalid_output_format() {
    let output = create_test_command()
        .args([
            "-sT",
            "-p",
            "80",
            "127.0.0.1",
            "--output-format",
            "INVALID_FORMAT",
        ])
        .output()
        .expect("Failed to execute");

    assert!(
        !output.status.success(),
        "Invalid output format should fail"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("format")
            || stderr.to_lowercase().contains("possible values"),
        "Should mention invalid format: {}",
        stderr
    );
}
