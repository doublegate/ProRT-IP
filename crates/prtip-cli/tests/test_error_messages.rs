//! Error message validation tests
//!
//! Tests that user-facing error messages are clear, actionable, and free of technical jargon.
//! Ensures NO stack traces, debug formatting, or internal details leak to users.
//!
//! Sprint 4.22 Phase 7: Comprehensive Testing - Subtask 5

use prtip_scanner::ScannerError;
use std::net::SocketAddr;
use std::time::Duration;

// ========================================================================
// NETWORK ERRORS (5 tests)
// ========================================================================

#[test]
fn test_network_unreachable_error_format() {
    let target: SocketAddr = "192.168.1.1:80".parse().unwrap();
    let err = ScannerError::TargetUnreachable {
        target,
        reason: "Network unreachable".to_string(),
    };

    let display = err.to_string();

    // Required components
    assert!(display.contains("192.168.1.1"), "Should contain target IP");
    assert!(
        display.contains("unreachable"),
        "Should mention unreachable"
    );

    // NO internal details
    assert!(!display.contains("backtrace"), "No stack trace");
    assert!(!display.contains("at src/"), "No source file paths");
    assert!(!display.contains("Debug"), "No debug formatting");
    assert!(!display.contains("{:?}"), "No debug format markers");
}

#[test]
fn test_timeout_error_contains_context() {
    let target: SocketAddr = "10.0.0.1:443".parse().unwrap();
    let err = ScannerError::Timeout {
        target,
        duration: Duration::from_secs(5),
        retriable: true,
    };

    let display = err.to_string();

    // Required components
    assert!(display.contains("10.0.0.1"), "Should contain target IP");
    assert!(
        display.contains("443") || display.to_lowercase().contains("timeout"),
        "Should contain port or timeout"
    );

    // Recovery suggestion
    let suggestion = err.recovery_suggestion();
    assert!(
        suggestion.is_some(),
        "Timeout should have recovery suggestion"
    );
    let suggestion_text = suggestion.unwrap();
    assert!(
        suggestion_text.contains("timeout")
            || suggestion_text.contains("--timeout")
            || suggestion_text.contains("-T"),
        "Suggestion should mention timeout configuration: {}",
        suggestion_text
    );

    // NO internal details
    assert!(!display.contains("backtrace"));
    assert!(!display.contains("thread"));
    assert!(!display.contains("panic"));
}

#[test]
fn test_connection_refused_error_contains_suggestion() {
    let target: SocketAddr = "192.168.100.50:22".parse().unwrap();
    let err = ScannerError::ConnectionFailed {
        target,
        reason: "Connection refused".to_string(),
        retriable: false,
    };

    let display = err.to_string();

    // Required components
    assert!(
        display.contains("192.168.100.50"),
        "Should contain target IP"
    );
    assert!(display.contains("22"), "Should contain port");
    assert!(
        display.contains("refused") || display.contains("failed"),
        "Should mention failure"
    );

    // Recovery suggestion
    let suggestion = err.recovery_suggestion();
    assert!(suggestion.is_some(), "Should have recovery suggestion");
    assert!(
        suggestion.unwrap().to_lowercase().contains("down")
            || suggestion.unwrap().to_lowercase().contains("unreachable"),
        "Suggestion should mention target may be down"
    );

    // NO internal details
    assert!(!display.contains("Error:"), "Should not use error label");
    assert!(!display.contains("backtrace"));
}

#[test]
fn test_host_unreachable_contains_routing_hint() {
    let target: SocketAddr = "172.16.0.1:80".parse().unwrap();
    let err = ScannerError::TargetUnreachable {
        target,
        reason: "No route to host".to_string(),
    };

    let display = err.to_string();

    // Required components
    assert!(display.contains("172.16.0.1"), "Should contain target IP");
    assert!(
        display.to_lowercase().contains("unreachable") || display.to_lowercase().contains("route"),
        "Should mention routing/unreachability"
    );

    // Should NOT be retriable (permanent error)
    assert!(
        !err.is_retriable(),
        "Host unreachable should not be retriable"
    );

    // NO internal details
    assert!(!display.contains("src/"), "No source paths");
    assert!(!display.contains("::"), "No Rust module paths");
}

#[test]
fn test_connection_reset_has_retriability_indicator() {
    let target: SocketAddr = "10.1.1.1:8080".parse().unwrap();
    let err = ScannerError::ConnectionFailed {
        target,
        reason: "Connection reset by peer".to_string(),
        retriable: true,
    };

    let display = err.to_string();

    // Required components
    assert!(display.contains("10.1.1.1"), "Should contain target IP");
    assert!(display.contains("8080"), "Should contain port");
    assert!(
        display.to_lowercase().contains("reset") || display.to_lowercase().contains("failed"),
        "Should mention failure type"
    );

    // Should be retriable (transient error)
    assert!(err.is_retriable(), "Connection reset should be retriable");

    // NO internal details
    assert!(!display.contains("Debug"));
    assert!(!display.contains("stack"));
}

// ========================================================================
// PERMISSION ERRORS (3 tests)
// ========================================================================

#[test]
fn test_raw_socket_permission_denied_mentions_sudo() {
    let err = ScannerError::InsufficientPrivileges {
        scan_type: "SYN".to_string(),
        suggestion: "Run as root or use sudo".to_string(),
    };

    let display = err.to_string();

    // Required components
    assert!(display.contains("SYN"), "Should mention scan type");
    assert!(
        display.to_lowercase().contains("privilege")
            || display.to_lowercase().contains("permission"),
        "Should mention permission issue"
    );

    // Recovery suggestion
    let suggestion = err.recovery_suggestion();
    assert!(suggestion.is_some(), "Should have recovery suggestion");
    let suggestion_text = suggestion.unwrap();
    assert!(
        suggestion_text.contains("sudo")
            || suggestion_text.contains("root")
            || suggestion_text.contains("CAP_NET_RAW"),
        "Suggestion should mention sudo/root/capabilities: {}",
        suggestion_text
    );

    // NO internal details
    assert!(!display.contains("EACCES"));
    assert!(!display.contains("errno"));
}

#[test]
fn test_insufficient_privileges_contains_scan_type() {
    let err = ScannerError::InsufficientPrivileges {
        scan_type: "FIN".to_string(),
        suggestion: "Use TCP connect scan (-sT) or run with elevated privileges".to_string(),
    };

    let display = err.to_string();

    assert!(display.contains("FIN"), "Should contain scan type");
    assert!(
        display.to_lowercase().contains("privilege")
            || display.to_lowercase().contains("permission"),
        "Should mention privilege requirement"
    );

    let suggestion = err.recovery_suggestion().unwrap();
    assert!(
        suggestion.contains("-sT") || suggestion.contains("connect"),
        "Should suggest alternative scan type"
    );
}

#[test]
fn test_resource_exhausted_contains_limits() {
    let err = ScannerError::ResourceExhausted {
        resource: "file descriptors".to_string(),
        current: 1020,
        limit: 1024,
        suggestion: "Reduce parallelism with --max-parallelism or increase ulimit".to_string(),
    };

    let display = err.to_string();

    // Required components
    assert!(
        display.contains("file descriptors"),
        "Should mention resource type"
    );
    assert!(display.contains("1020"), "Should contain current value");
    assert!(display.contains("1024"), "Should contain limit");

    // Recovery suggestion
    let suggestion = err.recovery_suggestion().unwrap();
    assert!(
        suggestion.contains("parallelism") || suggestion.contains("ulimit"),
        "Should suggest reducing parallelism or increasing limit: {}",
        suggestion
    );

    // Should be retriable (can free resources)
    assert!(
        err.is_retriable(),
        "Resource exhaustion should be retriable"
    );
}

// ========================================================================
// INPUT VALIDATION ERRORS (5 tests)
// ========================================================================

#[test]
fn test_invalid_configuration_clear_message() {
    let err =
        ScannerError::InvalidConfiguration("Port range 80-79 is invalid (start > end)".to_string());

    let display = err.to_string();

    // Should contain the specific error
    assert!(display.contains("80-79"), "Should contain invalid input");
    assert!(display.contains("invalid"), "Should mention invalidity");

    // Should NOT be retriable (permanent configuration error)
    assert!(
        !err.is_retriable(),
        "Configuration errors should not be retriable"
    );

    // NO internal details
    assert!(!display.contains("parse error"));
    assert!(!display.contains("Expected"));
}

#[test]
fn test_rate_limit_error_contains_values() {
    let err = ScannerError::RateLimitExceeded {
        current_rate: 150_000,
        max_rate: 100_000,
    };

    let display = err.to_string();

    // Required components
    assert!(
        display.contains("150"),
        "Should contain current rate (in thousands)"
    );
    assert!(
        display.contains("100"),
        "Should contain max rate (in thousands)"
    );
    assert!(
        display.to_lowercase().contains("rate") || display.to_lowercase().contains("limit"),
        "Should mention rate limiting"
    );

    // Recovery suggestion
    let suggestion = err.recovery_suggestion();
    assert!(suggestion.is_some(), "Should have recovery suggestion");
    let suggestion_text = suggestion.unwrap();
    assert!(
        suggestion_text.contains("--max-rate") || suggestion_text.contains("-T"),
        "Should suggest rate limiting options: {}",
        suggestion_text
    );

    // Should be retriable (transient constraint)
    assert!(err.is_retriable(), "Rate limit should be retriable");
}

#[test]
fn test_probe_failed_error_contains_target_and_reason() {
    let target: SocketAddr = "192.168.1.100:80".parse().unwrap();
    let err = ScannerError::ProbeFailed {
        target,
        probe_type: "HTTP".to_string(),
        reason: "Response malformed".to_string(),
        retriable: false,
    };

    let display = err.to_string();

    // Required components
    assert!(
        display.contains("192.168.1.100"),
        "Should contain target IP"
    );
    assert!(
        display.contains("malformed") || display.to_lowercase().contains("failed"),
        "Should mention failure reason"
    );
    assert!(
        display.to_lowercase().contains("probe"),
        "Should mention probe failure"
    );

    // NO internal details
    assert!(!display.contains("unwrap"));
    assert!(!display.contains("expect"));
}

#[test]
fn test_scan_cancelled_clear_reason() {
    let err = ScannerError::Cancelled {
        reason: "User interrupted with Ctrl+C".to_string(),
    };

    let display = err.to_string();

    assert!(
        display.contains("cancelled") || display.contains("Cancelled"),
        "Should mention cancellation"
    );
    assert!(
        display.contains("Ctrl+C") || display.contains("interrupted"),
        "Should mention reason"
    );

    // Should NOT be retriable (intentional cancellation)
    assert!(
        !err.is_retriable(),
        "Cancelled scan should not be retriable"
    );
}

#[test]
fn test_error_display_no_debug_formatting() {
    let target: SocketAddr = "10.0.0.1:443".parse().unwrap();
    let err = ScannerError::Timeout {
        target,
        duration: Duration::from_secs(5),
        retriable: true,
    };

    let display = err.to_string();

    // Should NOT contain debug formatting markers
    assert!(!display.contains("{:?}"), "Should not contain debug format");
    assert!(!display.contains("Some("), "Should not show Option wrapper");
    assert!(!display.contains("Ok("), "Should not show Result wrapper");
    assert!(!display.contains("Err("), "Should not show Error wrapper");
    assert!(
        !display.contains("ScannerError::"),
        "Should not show enum variant name"
    );
}

// ========================================================================
// CONFIGURATION ERRORS (4 tests)
// ========================================================================

#[test]
fn test_invalid_configuration_no_stack_traces() {
    let err = ScannerError::InvalidConfiguration(
        "Conflicting options: --syn and --connect cannot be used together".to_string(),
    );

    let display = err.to_string();

    assert!(display.contains("Conflicting"), "Should mention conflict");
    assert!(
        display.contains("--syn") && display.contains("--connect"),
        "Should list conflicting options"
    );

    // NO stack traces or internal details
    assert!(!display.contains("at crates/"));
    assert!(!display.contains("thread 'main'"));
    assert!(!display.contains("panicked at"));
    assert!(!display.contains("stack backtrace:"));
}

#[test]
fn test_invalid_configuration_actionable_message() {
    let err =
        ScannerError::InvalidConfiguration("Timing template must be T0-T5, got 'T6'".to_string());

    let display = err.to_string();

    assert!(display.contains("T0-T5"), "Should show valid range");
    assert!(display.contains("T6"), "Should show invalid input");
    assert!(
        display.to_lowercase().contains("must") || display.to_lowercase().contains("invalid"),
        "Should clearly indicate the problem"
    );
}

#[test]
fn test_connection_failed_has_target_context() {
    let target: SocketAddr = "[2001:db8::1]:80".parse().unwrap();
    let err = ScannerError::ConnectionFailed {
        target,
        reason: "Connection timeout".to_string(),
        retriable: true,
    };

    let display = err.to_string();

    // Should handle IPv6 addresses
    assert!(
        display.contains("2001:db8::1"),
        "Should contain IPv6 address"
    );
    assert!(display.contains("80"), "Should contain port");
}

#[test]
fn test_target_unreachable_has_reason() {
    let target: SocketAddr = "192.168.255.255:80".parse().unwrap();
    let err = ScannerError::TargetUnreachable {
        target,
        reason: "Destination host prohibited".to_string(),
    };

    let display = err.to_string();

    assert!(display.contains("192.168.255.255"), "Should contain target");
    assert!(
        display.contains("prohibited") || display.to_lowercase().contains("unreachable"),
        "Should mention the reason"
    );
}

// ========================================================================
// ERROR CATEGORY TESTS (3 tests)
// ========================================================================

#[test]
fn test_error_categories_correct() {
    use prtip_scanner::ErrorCategory;

    let target: SocketAddr = "127.0.0.1:80".parse().unwrap();

    let connection_err = ScannerError::ConnectionFailed {
        target,
        reason: "test".to_string(),
        retriable: false,
    };
    assert!(matches!(
        connection_err.category(),
        ErrorCategory::ConnectionFailed
    ));

    let timeout_err = ScannerError::Timeout {
        target,
        duration: Duration::from_secs(1),
        retriable: true,
    };
    assert!(matches!(timeout_err.category(), ErrorCategory::Timeout));

    let rate_err = ScannerError::RateLimitExceeded {
        current_rate: 100,
        max_rate: 50,
    };
    assert!(matches!(rate_err.category(), ErrorCategory::RateLimit));
}

#[test]
fn test_retriability_classification() {
    let target: SocketAddr = "127.0.0.1:80".parse().unwrap();

    // Retriable errors
    assert!(ScannerError::Timeout {
        target,
        duration: Duration::from_secs(1),
        retriable: true,
    }
    .is_retriable());
    assert!(ScannerError::RateLimitExceeded {
        current_rate: 100,
        max_rate: 50,
    }
    .is_retriable());
    assert!(ScannerError::ResourceExhausted {
        resource: "memory".to_string(),
        current: 100,
        limit: 50,
        suggestion: "".to_string(),
    }
    .is_retriable());

    // Non-retriable errors
    assert!(!ScannerError::InvalidConfiguration("test".to_string()).is_retriable());
    assert!(!ScannerError::InsufficientPrivileges {
        scan_type: "SYN".to_string(),
        suggestion: "".to_string(),
    }
    .is_retriable());
    assert!(!ScannerError::Cancelled {
        reason: "test".to_string(),
    }
    .is_retriable());
}

#[test]
fn test_from_io_error_conversion() {
    use std::io::{Error as IoError, ErrorKind};

    let target: SocketAddr = "127.0.0.1:80".parse().unwrap();

    // Timeout
    let io_err = IoError::new(ErrorKind::TimedOut, "timeout");
    let scanner_err = ScannerError::from_io_error(io_err, target);
    assert!(scanner_err.is_retriable());
    assert!(scanner_err.to_string().to_lowercase().contains("timeout"));

    // Connection refused
    let io_err = IoError::new(ErrorKind::ConnectionRefused, "refused");
    let scanner_err = ScannerError::from_io_error(io_err, target);
    assert!(!scanner_err.is_retriable());
    assert!(scanner_err.to_string().to_lowercase().contains("refused"));

    // Connection reset (retriable)
    let io_err = IoError::new(ErrorKind::ConnectionReset, "reset");
    let scanner_err = ScannerError::from_io_error(io_err, target);
    assert!(scanner_err.is_retriable());
}
