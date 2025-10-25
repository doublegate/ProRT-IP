//! Integration tests for CLI argument parsing
//!
//! Tests nmap-compatible flags and mixed syntax support.

#[path = "common/mod.rs"]
mod common;

use common::{init, run_prtip};

#[test]
fn test_help_flag() {
    init();
    let output = run_prtip(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ProRT-IP"));
    assert!(stdout.contains("Usage:"));
}

#[test]
fn test_version_flag() {
    init();
    let output = run_prtip(&["--version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("prtip") || stdout.contains("0.3"));
}

#[test]
fn test_nmap_syn_scan_flag() {
    init();
    // -sS requires privileges, so this should either work or give permission error
    let output = run_prtip(&["-sS", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either succeed or fail with permission error (not parsing error)
    if !output.status.success() {
        assert!(
            stderr.contains("privilege")
                || stderr.contains("permission")
                || stderr.contains("root")
                || stderr.contains("CAP_NET_RAW"),
            "Expected privilege error, got: {}",
            stderr
        );
    }
}

#[test]
fn test_nmap_connect_scan_flag() {
    init();
    // -sT should work without privileges
    let output = run_prtip(&["-sT", "-p", "80", "127.0.0.1"]);

    // Should succeed (connect scan doesn't require privileges)
    // Note: might fail if port is actually in use, but that's okay
    assert!(
        output.status.success()
            || String::from_utf8_lossy(&output.stderr).contains("timeout")
            || String::from_utf8_lossy(&output.stderr).contains("refused"),
        "Connect scan should work or fail with connection error, not parsing error"
    );
}

#[test]
fn test_port_single() {
    init();
    let output = run_prtip(&["-sT", "-p", "80", "127.0.0.1"]);
    // Should not error on parsing
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "Port parsing failed: {}",
            stderr
        );
    }
}

#[test]
fn test_port_range() {
    init();
    let output = run_prtip(&["-sT", "-p", "80-85", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "Port range parsing failed: {}",
            stderr
        );
    }
}

#[test]
fn test_port_list() {
    init();
    let output = run_prtip(&["-sT", "-p", "22,80,443", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "Port list parsing failed: {}",
            stderr
        );
    }
}

#[test]
fn test_fast_scan_flag() {
    init();
    // -F should scan top 100 ports
    let output = run_prtip(&["-sT", "-F", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "Fast scan flag parsing failed: {}",
            stderr
        );
    }
}

#[test]
fn test_timing_templates() {
    init();
    // Test each timing template
    for timing in &["-T0", "-T1", "-T2", "-T3", "-T4", "-T5"] {
        let output = run_prtip(&["-sT", timing, "-p", "80", "127.0.0.1"]);
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !stderr.contains("parse") && !stderr.contains("invalid"),
                "Timing template {} parsing failed: {}",
                timing,
                stderr
            );
        }
    }
}

#[test]
fn test_output_normal_flag() {
    init();
    let temp_dir = common::create_temp_dir("test_output");
    let output_file = temp_dir.join("scan.txt");

    let output = run_prtip(&[
        "-sT",
        "-p",
        "80",
        "-oN",
        output_file.to_str().unwrap(),
        "127.0.0.1",
    ]);

    // Check if output file was created (if scan succeeded)
    if output.status.success() {
        assert!(output_file.exists(), "Output file was not created");
    }

    common::cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_mixed_syntax() {
    init();
    // Mix nmap and ProRT-IP flags
    let output = run_prtip(&[
        "-sT", // nmap flag
        "--timeout",
        "1000", // ProRT-IP flag
        "-p",
        "80", // nmap flag
        "127.0.0.1",
    ]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse")
                && !stderr.contains("invalid")
                && !stderr.contains("conflict"),
            "Mixed syntax failed: {}",
            stderr
        );
    }
}

#[test]
fn test_invalid_port() {
    init();
    let output = run_prtip(&["-sT", "-p", "99999", "127.0.0.1"]);
    assert!(!output.status.success(), "Should fail on invalid port");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("port") || stderr.contains("invalid") || stderr.contains("range"),
        "Should report port error"
    );
}

#[test]
fn test_invalid_ip() {
    init();
    let output = run_prtip(&["-sT", "-p", "80", "999.999.999.999"]);
    assert!(!output.status.success(), "Should fail on invalid IP");
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Windows and Unix may have different DNS resolution error messages
    // Check for various error indicators across platforms
    assert!(
        stderr.contains("IP")
            || stderr.contains("address")
            || stderr.contains("invalid")
            || stderr.contains("Invalid")  // Capital I variant
            || stderr.contains("target")   // "Invalid target specification"
            || stderr.contains("resolve")  // "Failed to resolve"
            || stderr.contains("lookup")   // DNS lookup errors
            || stderr.contains("Error"), // Generic error indicator
        "Should report IP/target error, got: {}",
        stderr
    );
}

#[test]
fn test_verbose_flag() {
    init();
    let output = run_prtip(&["-sT", "-p", "80", "-v", "127.0.0.1"]);
    // Verbose flag should be accepted
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("unknown") && !stderr.contains("unrecognized"),
            "Verbose flag not recognized: {}",
            stderr
        );
    }
}

// ============================================================================
// TTL (Time-To-Live) Flag Tests - Sprint 4.20 Phase 3
// ============================================================================

#[test]
fn test_ttl_flag_minimum_value() {
    init();
    // Test TTL=1 (minimum valid value)
    let output = run_prtip(&["--ttl", "1", "-sT", "-p", "80", "127.0.0.1"]);
    // Should not fail on parsing (may fail on network/permission)
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "TTL minimum value parsing failed: {}",
            stderr
        );
    }
}

#[test]
fn test_ttl_flag_linux_default() {
    init();
    // Test TTL=64 (Linux/Unix default)
    let output = run_prtip(&["--ttl", "64", "-sT", "-p", "80", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "TTL Linux default parsing failed: {}",
            stderr
        );
    }
}

#[test]
fn test_ttl_flag_windows_default() {
    init();
    // Test TTL=128 (Windows default)
    let output = run_prtip(&["--ttl", "128", "-sT", "-p", "80", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "TTL Windows default parsing failed: {}",
            stderr
        );
    }
}

#[test]
fn test_ttl_flag_maximum_value() {
    init();
    // Test TTL=255 (maximum valid value)
    let output = run_prtip(&["--ttl", "255", "-sT", "-p", "80", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "TTL maximum value parsing failed: {}",
            stderr
        );
    }
}

#[test]
fn test_ttl_flag_custom_value() {
    init();
    // Test TTL=32 (arbitrary mid-range value)
    let output = run_prtip(&["--ttl", "32", "-sT", "-p", "80", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "TTL custom value parsing failed: {}",
            stderr
        );
    }
}

#[test]
fn test_ttl_flag_overflow_value() {
    init();
    // Test TTL=256 (above maximum for u8)
    let output = run_prtip(&["--ttl", "256", "-sT", "-p", "80", "127.0.0.1"]);
    assert!(!output.status.success(), "Should fail on TTL overflow");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ttl")
            || stderr.contains("256")
            || stderr.contains("invalid")
            || stderr.contains("range"),
        "Should report TTL overflow error: {}",
        stderr
    );
}

#[test]
fn test_ttl_flag_negative_value() {
    init();
    // Test TTL=-1 (negative value, invalid for u8)
    let output = run_prtip(&["--ttl", "-1", "-sT", "-p", "80", "127.0.0.1"]);
    assert!(!output.status.success(), "Should fail on negative TTL");
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Clap treats -1 as a flag, resulting in "unexpected argument" error
    assert!(
        stderr.contains("ttl")
            || stderr.contains("invalid")
            || stderr.contains("negative")
            || stderr.contains("parse")
            || stderr.contains("unexpected argument"),
        "Should report TTL negative value error: {}",
        stderr
    );
}

#[test]
fn test_ttl_flag_non_numeric() {
    init();
    // Test TTL=abc (non-numeric input)
    let output = run_prtip(&["--ttl", "abc", "-sT", "-p", "80", "127.0.0.1"]);
    assert!(!output.status.success(), "Should fail on non-numeric TTL");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ttl") || stderr.contains("invalid") || stderr.contains("parse"),
        "Should report TTL non-numeric error: {}",
        stderr
    );
}

#[test]
fn test_ttl_with_syn_scan() {
    init();
    // Test TTL combined with SYN scan
    let output = run_prtip(&["--ttl", "32", "-sS", "-p", "80", "127.0.0.1"]);
    // Should succeed or fail with privilege error (not parsing error)
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        assert!(
            stderr.contains("privilege")
                || stderr.contains("permission")
                || stderr.contains("root")
                || stderr.contains("CAP_NET_RAW")
                || !stderr.contains("parse"),
            "TTL with SYN scan should not have parsing errors: {}",
            stderr
        );
    }
}

#[test]
fn test_ttl_with_fragmentation() {
    init();
    // Test TTL combined with fragmentation flag
    let output = run_prtip(&["--ttl", "64", "-f", "-sT", "-p", "80", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse")
                && !stderr.contains("invalid")
                && !stderr.contains("conflict"),
            "TTL with fragmentation should not have parsing/conflict errors: {}",
            stderr
        );
    }
}

#[test]
fn test_ttl_with_timing_template() {
    init();
    // Test TTL combined with timing template
    let output = run_prtip(&["--ttl", "128", "-T3", "-sT", "-p", "80", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse")
                && !stderr.contains("invalid")
                && !stderr.contains("conflict"),
            "TTL with timing template should not have parsing/conflict errors: {}",
            stderr
        );
    }
}

#[test]
fn test_ttl_flag_full_scan() {
    init();
    // Integration test: full scan with TTL flag
    let output = run_prtip(&["--ttl", "64", "-sT", "-p", "80,443", "127.0.0.1"]);
    // Should complete successfully or fail with connection error (not parsing error)
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "TTL full scan should not have parsing errors: {}",
            stderr
        );
    }
}

// ============================================================================
// Bad Checksum (--badsum) Flag Tests - Sprint 4.20 Phase 7
// ============================================================================

#[test]
fn test_badsum_flag_with_syn_scan() {
    init();
    // Test --badsum combined with SYN scan
    let output = run_prtip(&["--badsum", "-sS", "-p", "80", "127.0.0.1"]);
    // Should succeed or fail with privilege error (not parsing error)
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        assert!(
            stderr.contains("privilege")
                || stderr.contains("permission")
                || stderr.contains("root")
                || stderr.contains("CAP_NET_RAW")
                || !stderr.contains("parse"),
            "--badsum with SYN scan should not have parsing errors: {}",
            stderr
        );
    }
}

#[test]
fn test_badsum_flag_with_udp_scan() {
    init();
    // Test --badsum combined with UDP scan
    let output = run_prtip(&["--badsum", "-sU", "-p", "53", "127.0.0.1"]);
    // Should succeed or fail with privilege/connection error (not parsing error)
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        assert!(
            stderr.contains("privilege")
                || stderr.contains("permission")
                || stderr.contains("timeout")
                || !stderr.contains("parse"),
            "--badsum with UDP scan should not have parsing errors: {}",
            stderr
        );
    }
}

#[test]
fn test_badsum_flag_with_stealth_scan() {
    init();
    // Test --badsum combined with FIN scan (stealth)
    let output = run_prtip(&["--badsum", "--scan-type", "fin", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        assert!(
            stderr.contains("privilege")
                || stderr.contains("permission")
                || !stderr.contains("parse"),
            "--badsum with stealth scan should not have parsing errors: {}",
            stderr
        );
    }
}

#[test]
fn test_badsum_flag_with_connect_scan() {
    init();
    // Test --badsum combined with TCP connect scan (no privileges required)
    let output = run_prtip(&["--badsum", "-sT", "-p", "80", "127.0.0.1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Connect scan should work (though --badsum has no effect on connect scans)
    if !output.status.success() {
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "--badsum with connect scan should not have parsing errors: {}",
            stderr
        );
    }
}

#[test]
fn test_badsum_with_fragmentation() {
    init();
    // Test --badsum combined with fragmentation flag
    let output = run_prtip(&["--badsum", "-f", "-sT", "-p", "80", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse")
                && !stderr.contains("invalid")
                && !stderr.contains("conflict"),
            "--badsum with fragmentation should not have parsing/conflict errors: {}",
            stderr
        );
    }
}

#[test]
fn test_badsum_with_ttl() {
    init();
    // Test --badsum combined with TTL flag
    let output = run_prtip(&["--badsum", "--ttl", "64", "-sT", "-p", "80", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse")
                && !stderr.contains("invalid")
                && !stderr.contains("conflict"),
            "--badsum with TTL should not have parsing/conflict errors: {}",
            stderr
        );
    }
}

#[test]
fn test_badsum_with_timing() {
    init();
    // Test --badsum combined with timing template
    let output = run_prtip(&["--badsum", "-T3", "-sT", "-p", "80", "127.0.0.1"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse")
                && !stderr.contains("invalid")
                && !stderr.contains("conflict"),
            "--badsum with timing template should not have parsing/conflict errors: {}",
            stderr
        );
    }
}

#[test]
fn test_badsum_all_evasion_flags() {
    init();
    // Test --badsum + -f + --ttl combined (all evasion techniques)
    let output = run_prtip(&[
        "--badsum",
        "-f",
        "--ttl",
        "32",
        "-sT",
        "-p",
        "80",
        "127.0.0.1",
    ]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse")
                && !stderr.contains("invalid")
                && !stderr.contains("conflict"),
            "All evasion flags combined should not have parsing/conflict errors: {}",
            stderr
        );
    }
}

#[test]
fn test_badsum_flag_full_scan() {
    init();
    // Integration test: full scan with --badsum flag
    let output = run_prtip(&["--badsum", "-sT", "-p", "80,443", "127.0.0.1"]);
    // Should complete successfully or fail with connection error (not parsing error)
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid"),
            "--badsum full scan should not have parsing errors: {}",
            stderr
        );
    }
}
