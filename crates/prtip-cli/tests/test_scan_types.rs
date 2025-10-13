//! Integration tests for different scan types
//!
//! Tests TCP Connect, SYN, UDP, and stealth scans.

#[path = "common/mod.rs"]
mod common;

use common::{init, run_prtip, has_elevated_privileges};

#[test]
fn test_tcp_connect_scan() {
    init();
    // TCP Connect scan (-sT) should work without privileges
    let output = run_prtip(&["-sT", "-p", "80", "127.0.0.1"]);

    // Should complete (may find port open, closed, or filtered)
    // As long as it doesn't crash or error on privileges
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Should not fail due to privileges
        assert!(
            !stderr.contains("privilege") &&
            !stderr.contains("permission") &&
            !stderr.contains("root"),
            "TCP Connect should not require privileges: {}",
            stderr
        );
    }
}

#[test]
fn test_tcp_connect_multiple_ports() {
    init();
    let output = run_prtip(&["-sT", "-p", "22,80,443", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("privilege"),
            "Multiple port scan should not require privileges: {}",
            stderr
        );
    }
}

#[test]
fn test_tcp_syn_scan_without_privileges() {
    init();

    if has_elevated_privileges() {
        // Skip this test if running as root
        eprintln!("Skipping test (running with elevated privileges)");
        return;
    }

    // SYN scan should fail with permission error (unless CAP_NET_RAW is set)
    let output = run_prtip(&["-sS", "-p", "80", "127.0.0.1"]);

    // If it succeeds, binary might have CAP_NET_RAW capability
    if output.status.success() {
        eprintln!("Note: SYN scan succeeded without root (binary may have CAP_NET_RAW capability)");
        return;
    }

    // Should fail with permission error
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("privilege") ||
        stderr.contains("permission") ||
        stderr.contains("root") ||
        stderr.contains("CAP_NET_RAW"),
        "SYN scan without privileges should fail with permission error: {}",
        stderr
    );
}

#[test]
fn test_tcp_syn_scan_with_privileges() {
    init();

    if !has_elevated_privileges() {
        eprintln!("Skipping test (requires elevated privileges)");
        return;
    }

    // SYN scan should work with privileges
    let output = run_prtip(&["-sS", "-p", "80", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Should not fail due to privileges
        assert!(
            !stderr.contains("privilege") && !stderr.contains("permission"),
            "SYN scan with privileges should work: {}",
            stderr
        );
    }
}

#[test]
fn test_udp_scan() {
    init();

    // UDP scan may require privileges on some systems
    let output = run_prtip(&["-sU", "-p", "53", "127.0.0.1"]);

    // Should complete or fail with permission error (not crash)
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Acceptable to fail on privileges or timeout
        assert!(
            stderr.contains("privilege") ||
            stderr.contains("permission") ||
            stderr.contains("timeout") ||
            stderr.contains("filtered"),
            "UDP scan should fail gracefully: {}",
            stderr
        );
    }
}

#[test]
fn test_fin_scan() {
    init();

    if !has_elevated_privileges() {
        eprintln!("Skipping FIN scan test (requires elevated privileges)");
        return;
    }

    let output = run_prtip(&["-sF", "-p", "80", "127.0.0.1"]);

    // FIN scan is a stealth scan
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("unrecognized") && !stderr.contains("unknown"),
            "FIN scan flag should be recognized: {}",
            stderr
        );
    }
}

#[test]
fn test_null_scan() {
    init();

    if !has_elevated_privileges() {
        eprintln!("Skipping NULL scan test (requires elevated privileges)");
        return;
    }

    let output = run_prtip(&["-sN", "-p", "80", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("unrecognized") && !stderr.contains("unknown"),
            "NULL scan flag should be recognized: {}",
            stderr
        );
    }
}

#[test]
fn test_xmas_scan() {
    init();

    if !has_elevated_privileges() {
        eprintln!("Skipping Xmas scan test (requires elevated privileges)");
        return;
    }

    let output = run_prtip(&["-sX", "-p", "80", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("unrecognized") && !stderr.contains("unknown"),
            "Xmas scan flag should be recognized: {}",
            stderr
        );
    }
}

#[test]
fn test_ack_scan() {
    init();

    if !has_elevated_privileges() {
        eprintln!("Skipping ACK scan test (requires elevated privileges)");
        return;
    }

    let output = run_prtip(&["-sA", "-p", "80", "127.0.0.1"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("unrecognized") && !stderr.contains("unknown"),
            "ACK scan flag should be recognized: {}",
            stderr
        );
    }
}

#[test]
fn test_scan_localhost_common_ports() {
    init();

    // Scan some common ports on localhost
    let output = run_prtip(&["-sT", "-p", "22,80,443,3306,5432", "127.0.0.1"]);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should contain target IP
        assert!(
            stdout.contains("127.0.0.1") || stdout.contains("localhost"),
            "Output should contain target information"
        );
    }
}

#[test]
fn test_fast_scan() {
    init();

    // Fast scan (-F) scans top 100 ports
    let output = run_prtip(&["-sT", "-F", "127.0.0.1"]);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should scan multiple ports
        assert!(
            !stdout.is_empty(),
            "Fast scan should produce output"
        );
    }
}

#[test]
fn test_scan_with_timeout() {
    init();

    // Test with short timeout
    let output = run_prtip(&[
        "-sT",
        "-p", "80",
        "--timeout", "100",
        "127.0.0.1"
    ]);

    // Should complete (may timeout, but shouldn't crash)
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Acceptable to timeout
        assert!(
            !stderr.contains("panic") && !stderr.contains("crashed"),
            "Should not crash on timeout: {}",
            stderr
        );
    }
}

#[test]
fn test_scan_rate_limiting() {
    init();

    // Test with rate limit
    let output = run_prtip(&[
        "-sT",
        "-p", "80-85",
        "--rate", "10",
        "127.0.0.1"
    ]);

    // Should complete with rate limiting
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("unrecognized"),
            "Rate limit flag should be recognized: {}",
            stderr
        );
    }
}
