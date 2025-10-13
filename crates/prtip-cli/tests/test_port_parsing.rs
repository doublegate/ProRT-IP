//! Integration tests for port specification parsing
//!
//! Tests various port formats: single, range, list, all, top-N.

#[path = "common/mod.rs"]
mod common;

use common::{init, run_prtip};

#[test]
fn test_single_port() {
    init();
    let output = run_prtip(&["-sT", "-p", "80", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid port"),
            "Single port should parse correctly: {}",
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
            !stderr.contains("parse") && !stderr.contains("invalid port"),
            "Port range should parse correctly: {}",
            stderr
        );
    }
}

#[test]
fn test_port_range_reversed() {
    init();
    // Reversed range should be handled gracefully
    let output = run_prtip(&["-sT", "-p", "85-80", "127.0.0.1"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should either work (auto-reverse) or fail gracefully
    if !output.status.success() {
        assert!(
            stderr.contains("range") || stderr.contains("invalid"),
            "Reversed range should fail gracefully: {}",
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
            !stderr.contains("parse") && !stderr.contains("invalid port"),
            "Port list should parse correctly: {}",
            stderr
        );
    }
}

#[test]
fn test_port_list_with_spaces() {
    init();
    // Some users might try spaces
    let output = run_prtip(&["-sT", "-p", "22, 80, 443", "127.0.0.1"]);

    // Should either work or fail gracefully
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Acceptable to not support spaces
        assert!(
            stderr.contains("parse") || stderr.contains("invalid"),
            "Port list with spaces should fail gracefully: {}",
            stderr
        );
    }
}

#[test]
fn test_port_mixed_range_and_list() {
    init();
    let output = run_prtip(&["-sT", "-p", "22,80-85,443", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid port"),
            "Mixed port range and list should parse: {}",
            stderr
        );
    }
}

#[test]
fn test_all_ports() {
    init();
    // -p- means all ports (1-65535)
    // This will be slow, so use with timeout
    let output = run_prtip(&[
        "-sT",
        "-p-",
        "--timeout",
        "100",
        "--rate",
        "1000",
        "127.0.0.1",
    ]);

    // Should at least start scanning
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Acceptable to timeout, but should recognize -p-
        assert!(
            !stderr.contains("unrecognized") && !stderr.contains("unknown flag"),
            "All ports flag should be recognized: {}",
            stderr
        );
    }
}

#[test]
fn test_top_ports_100() {
    init();
    // -F is shorthand for top 100 ports
    let output = run_prtip(&["-sT", "-F", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("unrecognized"),
            "Fast scan flag should be recognized: {}",
            stderr
        );
    }
}

#[test]
fn test_top_ports_1000() {
    init();
    let output = run_prtip(&["-sT", "--top-ports", "1000", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("unrecognized"),
            "Top ports flag should be recognized: {}",
            stderr
        );
    }
}

#[test]
fn test_invalid_port_zero() {
    init();
    let output = run_prtip(&["-sT", "-p", "0", "127.0.0.1"]);

    // Port 0 is invalid
    assert!(!output.status.success(), "Port 0 should be invalid");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("port") || stderr.contains("invalid") || stderr.contains("range"),
        "Should report port error: {}",
        stderr
    );
}

#[test]
fn test_invalid_port_too_high() {
    init();
    let output = run_prtip(&["-sT", "-p", "65536", "127.0.0.1"]);

    // Port 65536 is invalid
    assert!(!output.status.success(), "Port 65536 should be invalid");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("port") || stderr.contains("invalid") || stderr.contains("range"),
        "Should report port error: {}",
        stderr
    );
}

#[test]
fn test_invalid_port_negative() {
    init();
    let output = run_prtip(&["-sT", "-p", "-1", "127.0.0.1"]);

    // Negative port is invalid
    assert!(!output.status.success(), "Negative port should be invalid");
}

#[test]
fn test_invalid_port_non_numeric() {
    init();
    let output = run_prtip(&["-sT", "-p", "http", "127.0.0.1"]);

    // Non-numeric port should fail (unless service name resolution is supported)
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        assert!(
            stderr.contains("port") || stderr.contains("invalid") || stderr.contains("parse"),
            "Non-numeric port should fail or be resolved: {}",
            stderr
        );
    }
}

#[test]
fn test_port_boundary_values() {
    init();

    // Test boundary values
    for port in &["1", "80", "443", "8080", "65535"] {
        let output = run_prtip(&["-sT", "-p", port, "127.0.0.1"]);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !stderr.contains("parse") && !stderr.contains("invalid port"),
                "Valid port {} should parse: {}",
                port,
                stderr
            );
        }
    }
}

#[test]
fn test_empty_port_list() {
    init();
    let output = run_prtip(&["-sT", "-p", "", "127.0.0.1"]);

    // Empty port list should fail
    assert!(!output.status.success(), "Empty port list should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("port") || stderr.contains("empty") || stderr.contains("missing"),
        "Should report port error: {}",
        stderr
    );
}

#[test]
fn test_port_range_large() {
    init();
    // Test large range
    let output = run_prtip(&["-sT", "-p", "1-1000", "--timeout", "100", "127.0.0.1"]);

    // Should at least start scanning
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Acceptable to timeout, but should parse correctly
        assert!(
            !stderr.contains("parse") && !stderr.contains("invalid port"),
            "Large port range should parse: {}",
            stderr
        );
    }
}
