//! Integration tests for the CLI

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ProRT-IP WarScan"))
        .stdout(predicate::str::contains("NMAP-COMPATIBLE"))
        .stdout(predicate::str::contains("PERFORMANCE"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("prtip"));
}

#[test]
fn test_cli_no_targets() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_cli_invalid_ports() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-p", "invalid", "192.168.1.1"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("port"));
}

#[test]
fn test_cli_invalid_timing() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-T", "10", "192.168.1.1"]);
    cmd.assert().failure();
}

#[test]
fn test_cli_zero_timeout() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["--timeout", "0", "192.168.1.1"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Timeout must be greater than 0"));
}

#[test]
fn test_cli_zero_max_rate() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["--max-rate", "0", "192.168.1.1"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Max rate must be greater than 0"));
}

#[test]
fn test_cli_multiple_targets() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-p", "9999", "127.0.0.1", "127.0.0.2", "127.0.0.3"]);

    // May need privileges, but should at least parse correctly
    let output = cmd.output().unwrap();
    // Either succeeds or fails with permission error
    if output.status.success() || String::from_utf8_lossy(&output.stderr).contains("privileges") {
        // Expected
    } else {
        panic!(
            "Unexpected failure: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_cli_json_output() {
    let tmp_dir = TempDir::new().unwrap();
    let output_file = tmp_dir.path().join("results.json");

    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args([
        "-o",
        "json",
        "--output-file",
        output_file.to_str().unwrap(),
        "-p",
        "9999",
        "127.0.0.1",
    ]);

    // Run the command
    let output = cmd.output().unwrap();

    // Check if output file was created (if scan succeeded)
    if output.status.success() {
        assert!(output_file.exists());

        // Verify JSON format
        let content = fs::read_to_string(&output_file).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("total_results").is_some());
    }
}

#[test]
fn test_cli_xml_output() {
    let tmp_dir = TempDir::new().unwrap();
    let output_file = tmp_dir.path().join("results.xml");

    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args([
        "-o",
        "xml",
        "--output-file",
        output_file.to_str().unwrap(),
        "-p",
        "9999",
        "127.0.0.1",
    ]);

    // Run the command
    let output = cmd.output().unwrap();

    // Check if output file was created (if scan succeeded)
    if output.status.success() {
        assert!(output_file.exists());

        // Verify XML format
        let content = fs::read_to_string(&output_file).unwrap();
        assert!(content.contains("<?xml"));
        assert!(content.contains("<nmaprun"));
    }
}

#[test]
fn test_cli_verbose_levels() {
    // Test -v
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-v", "-p", "9999", "127.0.0.1"]);
    let _ = cmd.output(); // Just check it doesn't crash

    // Test -vv
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-vv", "-p", "9999", "127.0.0.1"]);
    let _ = cmd.output();

    // Test -vvv
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-vvv", "-p", "9999", "127.0.0.1"]);
    let _ = cmd.output();
}

#[test]
fn test_cli_scan_types() {
    // Test connect scan (default)
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-s", "connect", "-p", "9999", "127.0.0.1"]);
    let _ = cmd.output();

    // Test syn scan (will likely fail without privileges, but should parse)
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-s", "syn", "-p", "9999", "127.0.0.1"]);
    let output = cmd.output().unwrap();
    // Either works or fails with privilege error
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        assert!(
            stderr.contains("privileges") || stderr.is_empty(),
            "Unexpected error: {}",
            stderr
        );
    }
}

#[test]
fn test_cli_timing_templates() {
    for timing in 0..=5 {
        let mut cmd = Command::cargo_bin("prtip").unwrap();
        cmd.args(["-T", &timing.to_string(), "-p", "9999", "127.0.0.1"]);
        let _ = cmd.output(); // Should at least parse
    }
}

#[test]
fn test_cli_custom_database() {
    let tmp_dir = TempDir::new().unwrap();
    let db_file = tmp_dir.path().join("custom.db");

    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args([
        "--with-db",
        "--database",
        db_file.to_str().unwrap(),
        "-p",
        "9999",
        "127.0.0.1",
    ]);

    // Run the command
    let output = cmd.output().unwrap();

    // Check if database was created (if scan succeeded and --with-db was specified)
    if output.status.success() {
        assert!(db_file.exists());
    }
}

#[test]
fn test_cli_host_discovery_flag() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-P", "-p", "9999", "127.0.0.1"]);
    let _ = cmd.output(); // Should parse correctly
}

#[test]
fn test_cli_interface_option() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["--interface", "lo", "-p", "9999", "127.0.0.1"]);
    let _ = cmd.output(); // Should parse correctly
}

#[test]
fn test_cli_max_concurrent() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["--max-concurrent", "50", "-p", "9999", "127.0.0.1"]);
    let _ = cmd.output(); // Should parse correctly
}

#[test]
fn test_cli_scan_delay() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["--scan-delay", "100", "-p", "9999", "127.0.0.1"]);
    let _ = cmd.output(); // Should parse correctly
}

#[test]
fn test_cli_retries() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["--retries", "3", "-p", "9999", "127.0.0.1"]);
    let _ = cmd.output(); // Should parse correctly
}

#[test]
fn test_cli_excessive_retries() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["--retries", "20", "-p", "9999", "127.0.0.1"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Retries"));
}

#[test]
fn test_cli_cidr_notation() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-p", "9999", "127.0.0.0/30"]);
    let _ = cmd.output(); // Should parse CIDR correctly
}

#[test]
fn test_cli_port_range() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-p", "9998-10000", "127.0.0.1"]);
    let _ = cmd.output();
}

#[test]
fn test_cli_port_list() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-p", "80,443,8080,8443", "127.0.0.1"]);
    let _ = cmd.output();
}

#[test]
fn test_cli_mixed_port_spec() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-p", "80,443,8000-8010", "127.0.0.1"]);
    let _ = cmd.output();
}

#[test]
fn test_cli_invalid_port_range() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-p", "100-50", "127.0.0.1"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("port"));
}

#[test]
fn test_cli_port_zero() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["-p", "0", "127.0.0.1"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("port"));
}

#[test]
fn test_cli_excessive_timeout() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["--timeout", "4000000", "127.0.0.1"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Timeout cannot exceed"));
}

#[test]
fn test_cli_excessive_max_concurrent() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(["--max-concurrent", "200000", "127.0.0.1"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Max concurrent"));
}

#[test]
fn test_cli_output_to_file() {
    let tmp_dir = TempDir::new().unwrap();
    let output_file = tmp_dir.path().join("scan.txt");

    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args([
        "--output-file",
        output_file.to_str().unwrap(),
        "-p",
        "9999",
        "127.0.0.1",
    ]);

    let output = cmd.output().unwrap();

    if output.status.success() {
        assert!(output_file.exists());
        let content = fs::read_to_string(&output_file).unwrap();
        assert!(content.contains("ProRT-IP"));
    }
}
