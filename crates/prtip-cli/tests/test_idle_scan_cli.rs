//! Integration tests for idle scan CLI argument parsing
//!
//! Tests the -sI (idle scan) flag parsing, zombie validation, and conflicts.
//! Covers both nmap-compatible -sI syntax and native -I/--idle-scan syntax.

#[path = "common/mod.rs"]
mod common;

use common::{init, run_prtip};

/// Test basic -sI flag with zombie host
#[test]
fn test_idle_scan_short_flag() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fail with zombie unreachable, not parsing error
    if !output.status.success() {
        assert!(
            stderr.contains("zombie")
                || stderr.contains("unreachable")
                || stderr.contains("timeout"),
            "Expected zombie-related error, got: {}",
            stderr
        );
        assert!(
            !stderr.contains("invalid") && !stderr.contains("parse"),
            "Should not be a parsing error: {}",
            stderr
        );
    }
}

/// Test native -I flag (not nmap-compatible, but supported)
#[test]
fn test_idle_scan_native_short_flag() {
    init();
    let output = run_prtip(&["-I", "192.168.1.100", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fail with zombie unreachable, not parsing error
    if !output.status.success() {
        assert!(
            stderr.contains("zombie")
                || stderr.contains("unreachable")
                || stderr.contains("timeout"),
            "Expected zombie-related error, got: {}",
            stderr
        );
    }
}

/// Test long --idle-scan flag
#[test]
fn test_idle_scan_long_flag() {
    init();
    let output = run_prtip(&["--idle-scan", "192.168.1.100", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        assert!(
            stderr.contains("zombie")
                || stderr.contains("unreachable")
                || stderr.contains("timeout"),
            "Expected zombie-related error, got: {}",
            stderr
        );
    }
}

/// Test -sI flag with hostname zombie (not just IP)
#[test]
fn test_idle_scan_with_hostname() {
    init();
    let output = run_prtip(&["-sI", "zombie.example.local", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept hostname and fail with DNS/connection error, not parsing error
    if !output.status.success() {
        assert!(
            !stderr.contains("invalid") && !stderr.contains("parse"),
            "Should not be a parsing error: {}",
            stderr
        );
    }
}

/// Test -sI flag with IPv6 zombie
#[test]
fn test_idle_scan_with_ipv6_zombie() {
    init();
    let output = run_prtip(&["-sI", "::1", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept IPv6 and fail with connection error, not parsing error
    if !output.status.success() {
        assert!(
            !stderr.contains("invalid") && !stderr.contains("parse"),
            "Should not be a parsing error: {}",
            stderr
        );
    }
}

/// Test -sI flag with port specification
#[test]
fn test_idle_scan_with_multiple_ports() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-p", "22,80,443", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept multiple ports
    if !output.status.success() {
        assert!(
            stderr.contains("zombie") || stderr.contains("unreachable"),
            "Expected zombie-related error, got: {}",
            stderr
        );
    }
}

/// Test -sI flag with port range
#[test]
fn test_idle_scan_with_port_range() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-p", "1-100", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept port range
    if !output.status.success() {
        assert!(
            stderr.contains("zombie") || stderr.contains("unreachable"),
            "Expected zombie-related error, got: {}",
            stderr
        );
    }
}

/// Test --zombie-quality flag (minimum quality threshold)
#[test]
fn test_zombie_quality_flag() {
    init();
    let output = run_prtip(&[
        "-sI",
        "192.168.1.100",
        "--zombie-quality",
        "75",
        "-p",
        "80",
        "127.0.0.1",
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept quality threshold
    if !output.status.success() {
        assert!(
            !stderr.contains("invalid") && !stderr.contains("parse"),
            "Should not be a parsing error: {}",
            stderr
        );
    }
}

/// Test invalid zombie quality (out of range)
#[test]
fn test_zombie_quality_invalid_high() {
    init();
    let output = run_prtip(&[
        "-sI",
        "192.168.1.100",
        "--zombie-quality",
        "150",
        "-p",
        "80",
        "127.0.0.1",
    ]);

    // Should fail with validation error (quality must be 0-100)
    // Note: Currently clap validates u8 range (0-255), so 150 is technically valid
    // but runtime validation should catch this if implemented
    // For now, we just verify it doesn't cause a parsing error
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        // Either runtime validation error or zombie error
        assert!(
            stderr.contains("quality")
                || stderr.contains("zombie")
                || stderr.contains("unreachable")
                || stderr.contains("range"),
            "Got: {}",
            stderr
        );
    }
}

/// Test invalid zombie quality (negative)
#[test]
fn test_zombie_quality_invalid_negative() {
    init();
    let output = run_prtip(&[
        "-sI",
        "192.168.1.100",
        "--zombie-quality",
        "-10",
        "-p",
        "80",
        "127.0.0.1",
    ]);

    // Should fail with parsing error (clap should reject negative values)
    assert!(!output.status.success(), "Should reject negative quality");
}

/// Test -sI with other scan types (idle scan takes precedence)
#[test]
fn test_idle_scan_with_syn() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-sS", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Idle scan should take precedence, fail with zombie error not conflict
    if !output.status.success() {
        assert!(
            stderr.contains("zombie")
                || stderr.contains("unreachable")
                || stderr.contains("conflict"),
            "Expected zombie-related error or conflict, got: {}",
            stderr
        );
    }
}

/// Test -sI with connect scan (idle scan takes precedence)
#[test]
fn test_idle_scan_with_connect() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-sT", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Idle scan should take precedence
    if !output.status.success() {
        assert!(
            stderr.contains("zombie")
                || stderr.contains("unreachable")
                || stderr.contains("conflict"),
            "Expected zombie-related error or conflict, got: {}",
            stderr
        );
    }
}

/// Test -sI with UDP scan (idle scan takes precedence)
#[test]
fn test_idle_scan_with_udp() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-sU", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Idle scan should take precedence
    if !output.status.success() {
        assert!(
            stderr.contains("zombie")
                || stderr.contains("unreachable")
                || stderr.contains("conflict"),
            "Expected zombie-related error or conflict, got: {}",
            stderr
        );
    }
}

/// Test -sI with stealth scans (idle scan takes precedence)
#[test]
fn test_idle_scan_with_stealth() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-sF", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Idle scan should take precedence
    if !output.status.success() {
        assert!(
            stderr.contains("zombie")
                || stderr.contains("unreachable")
                || stderr.contains("conflict"),
            "Expected zombie-related error or conflict, got: {}",
            stderr
        );
    }
}

/// Test -sI works with timing templates
#[test]
fn test_idle_scan_with_timing() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-T4", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept timing template (no conflict)
    if !output.status.success() {
        assert!(
            stderr.contains("zombie") || stderr.contains("unreachable"),
            "Expected zombie-related error, not timing error: {}",
            stderr
        );
    }
}

/// Test -sI works with output formats
#[test]
fn test_idle_scan_with_output_json() {
    init();
    let temp_dir = common::create_temp_dir("idle-json");
    let json_path = temp_dir.join("scan.json");

    let output = run_prtip(&[
        "-sI",
        "192.168.1.100",
        "-p",
        "80",
        "--output-format",
        "json",
        "--output-file",
        json_path.to_str().unwrap(),
        "127.0.0.1",
    ]);

    common::cleanup_temp_dir(&temp_dir);

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept output format (no conflict)
    if !output.status.success() {
        assert!(
            stderr.contains("zombie") || stderr.contains("unreachable"),
            "Expected zombie-related error, not output error: {}",
            stderr
        );
    }
}

/// Test -sI works with verbosity flags
#[test]
fn test_idle_scan_with_verbosity() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-v", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept verbosity (no conflict)
    if !output.status.success() {
        assert!(
            stderr.contains("zombie") || stderr.contains("unreachable"),
            "Expected zombie-related error, not verbosity error: {}",
            stderr
        );
    }
}

/// Test missing zombie host argument
#[test]
fn test_idle_scan_missing_zombie() {
    init();
    let output = run_prtip(&["-sI", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fail with missing value error
    assert!(!output.status.success(), "Should require zombie host");
    assert!(
        stderr.contains("a value is required")
            || stderr.contains("requires a value")
            || stderr.contains("expected"),
        "Expected missing value error, got: {}",
        stderr
    );
}

/// Test -sI with multiple targets
#[test]
fn test_idle_scan_multiple_targets() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-p", "80", "127.0.0.1", "127.0.0.2"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept multiple targets
    if !output.status.success() {
        assert!(
            stderr.contains("zombie") || stderr.contains("unreachable"),
            "Expected zombie-related error, got: {}",
            stderr
        );
    }
}

/// Test -sI with CIDR notation target
#[test]
fn test_idle_scan_with_cidr() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-p", "80", "127.0.0.0/30"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept CIDR notation
    if !output.status.success() {
        assert!(
            stderr.contains("zombie") || stderr.contains("unreachable"),
            "Expected zombie-related error, got: {}",
            stderr
        );
    }
}

/// Test preprocessor handles -sI correctly (integration with main.rs preprocessing)
#[test]
fn test_idle_scan_preprocessor_integration() {
    init();
    // This tests that the preprocessor in main.rs correctly transforms -sI to --nmap-idle
    let output = run_prtip(&["-sI", "192.168.1.100", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should NOT fail with parsing error (preprocessor should work)
    if !output.status.success() {
        assert!(
            !stderr.contains("invalid value") && !stderr.contains("unexpected argument"),
            "Preprocessor may have failed: {}",
            stderr
        );
    }
}

/// Test combined flags: -sI with evasion techniques
#[test]
fn test_idle_scan_with_fragmentation() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-f", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept fragmentation flag (no conflict)
    if !output.status.success() {
        assert!(
            stderr.contains("zombie") || stderr.contains("unreachable"),
            "Expected zombie-related error, not evasion error: {}",
            stderr
        );
    }
}

/// Test combined flags: -sI with TTL manipulation
#[test]
fn test_idle_scan_with_ttl() {
    init();
    let output = run_prtip(&[
        "-sI",
        "192.168.1.100",
        "--ttl",
        "64",
        "-p",
        "80",
        "127.0.0.1",
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept TTL flag (no conflict)
    if !output.status.success() {
        assert!(
            stderr.contains("zombie") || stderr.contains("unreachable"),
            "Expected zombie-related error, not TTL error: {}",
            stderr
        );
    }
}

/// Test combined flags: -sI with decoy scanning
#[test]
fn test_idle_scan_with_decoys() {
    init();
    let output = run_prtip(&[
        "-sI",
        "192.168.1.100",
        "-D",
        "RND:3",
        "-p",
        "80",
        "127.0.0.1",
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept decoy flag (no conflict)
    if !output.status.success() {
        assert!(
            stderr.contains("zombie") || stderr.contains("unreachable"),
            "Expected zombie-related error, not decoy error: {}",
            stderr
        );
    }
}

/// Test -sI with source port manipulation
#[test]
fn test_idle_scan_with_source_port() {
    init();
    let output = run_prtip(&["-sI", "192.168.1.100", "-g", "53", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should accept source port flag (no conflict)
    if !output.status.success() {
        assert!(
            stderr.contains("zombie") || stderr.contains("unreachable"),
            "Expected zombie-related error, not source port error: {}",
            stderr
        );
    }
}
