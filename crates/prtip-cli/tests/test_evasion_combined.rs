//! Integration tests for combined evasion techniques
//!
//! Tests interaction between fragmentation, TTL, and bad checksums.

#[path = "common/mod.rs"]
mod common;

use common::{init, run_prtip};

#[test]
fn test_fragmentation_with_bad_checksum_default_mtu() {
    init();
    // Test fragmentation (-f) with bad checksums (--badsum)
    // -f uses default MTU of 28 bytes (nmap compatible)
    let output = run_prtip(&["-f", "--badsum", "-sT", "-p", "80", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse")
                && !stderr.contains("invalid")
                && !stderr.contains("conflict"),
            "Fragmentation + bad checksum should not conflict: {}",
            stderr
        );
    }
}

#[test]
fn test_fragmentation_with_bad_checksum_custom_mtu() {
    init();
    // Test custom MTU (--mtu 200) with bad checksums (--badsum)
    let output = run_prtip(&["--mtu", "200", "--badsum", "-sT", "-p", "80", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse")
                && !stderr.contains("invalid")
                && !stderr.contains("conflict"),
            "Custom MTU + bad checksum should not conflict: {}",
            stderr
        );
    }
}

#[test]
fn test_ttl_with_bad_checksum_low_ttl() {
    init();
    // Test low TTL (--ttl 16) with bad checksums (--badsum)
    let output = run_prtip(&["--ttl", "16", "--badsum", "-sT", "-p", "80", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse")
                && !stderr.contains("invalid")
                && !stderr.contains("conflict"),
            "Low TTL + bad checksum should not conflict: {}",
            stderr
        );
    }
}

#[test]
fn test_ttl_with_bad_checksum_high_ttl() {
    init();
    // Test high TTL (--ttl 128) with bad checksums (--badsum)
    let output = run_prtip(&["--ttl", "128", "--badsum", "-sT", "-p", "80", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("parse")
                && !stderr.contains("invalid")
                && !stderr.contains("conflict"),
            "High TTL + bad checksum should not conflict: {}",
            stderr
        );
    }
}

#[test]
fn test_all_three_evasion_techniques() {
    init();
    // Test all three evasion techniques combined
    // Fragmentation (-f), TTL (--ttl 32), Bad Checksums (--badsum)
    let output = run_prtip(&[
        "-f",
        "--ttl",
        "32",
        "--badsum",
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
            "All three evasion techniques should work together: {}",
            stderr
        );
    }
}

#[test]
fn test_all_evasion_with_timing() {
    init();
    // Test all evasion techniques with timing template
    let output = run_prtip(&[
        "-f",
        "--ttl",
        "32",
        "--badsum",
        "-T3",
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
            "All evasion + timing should work together: {}",
            stderr
        );
    }
}
