//! Integration tests for the CLI

#![allow(clippy::needless_borrows_for_generic_args)]

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

// ============================================================================
// DATABASE COMMAND TESTS (Sprint 4.18.1)
// ============================================================================

/// Helper to create a temporary test database with scan data
fn create_test_db_with_scan(temp_dir: &TempDir) -> (String, String) {
    let db_path = temp_dir
        .path()
        .join("test.db")
        .to_str()
        .unwrap()
        .to_string();
    let target = "127.0.0.1";

    // Run a scan with database storage
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(&["-p", "80,443", target, "--with-db", "--database", &db_path]);

    let output = cmd.output().unwrap();

    // Print debug output if failed
    if !output.status.success() {
        eprintln!("Scan stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Scan stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Scan command failed");
    }

    (db_path, target.to_string())
}

#[test]
fn test_db_list() {
    let temp_dir = TempDir::new().unwrap();
    let (db_path, _target) = create_test_db_with_scan(&temp_dir);

    // Test `prtip db list`
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(&["db", "list", &db_path]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scans in Database").or(predicate::str::contains("scan")));
}

#[test]
fn test_db_query_scan_id() {
    let temp_dir = TempDir::new().unwrap();
    let (db_path, _target) = create_test_db_with_scan(&temp_dir);

    // Query scan ID 1 (first scan)
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(&["db", "query", &db_path, "--scan-id", "1"]);

    cmd.assert().success().stdout(
        predicate::str::contains("Results for Scan").or(predicate::str::contains("127.0.0.1")),
    );
}

#[test]
fn test_db_query_target() {
    let temp_dir = TempDir::new().unwrap();
    let (db_path, target) = create_test_db_with_scan(&temp_dir);

    // Query by target IP
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(&["db", "query", &db_path, "--target", &target]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Open Ports").or(predicate::str::contains(&target)));
}

#[test]
fn test_db_query_port() {
    let temp_dir = TempDir::new().unwrap();
    let (db_path, _target) = create_test_db_with_scan(&temp_dir);

    // Query by port
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(&["db", "query", &db_path, "--port", "80"]);

    cmd.assert().success(); // May or may not find port 80, but should not error
}

#[test]
fn test_db_export_json() {
    let temp_dir = TempDir::new().unwrap();
    let (db_path, _target) = create_test_db_with_scan(&temp_dir);
    let export_path = temp_dir
        .path()
        .join("export.json")
        .to_str()
        .unwrap()
        .to_string();

    // Export to JSON
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(&[
        "db",
        "export",
        &db_path,
        "--scan-id",
        "1",
        "--format",
        "json",
        "-o",
        &export_path,
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Exported"));

    // Verify file exists and is valid JSON
    let content = fs::read_to_string(&export_path).unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("[") || content.contains("target_ip")); // Should be JSON
}

#[test]
fn test_db_export_csv() {
    let temp_dir = TempDir::new().unwrap();
    let (db_path, _target) = create_test_db_with_scan(&temp_dir);
    let export_path = temp_dir
        .path()
        .join("export.csv")
        .to_str()
        .unwrap()
        .to_string();

    // Export to CSV
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(&[
        "db",
        "export",
        &db_path,
        "--scan-id",
        "1",
        "--format",
        "csv",
        "-o",
        &export_path,
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Exported"));

    // Verify file exists and has CSV header
    let content = fs::read_to_string(&export_path).unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("Target IP") || content.contains("Port")); // CSV header
}

#[test]
fn test_db_compare_scans() {
    let temp_dir = TempDir::new().unwrap();

    // Create first scan
    let db_path = temp_dir
        .path()
        .join("test.db")
        .to_str()
        .unwrap()
        .to_string();
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(&["-p", "80", "127.0.0.1", "--with-db", "--database", &db_path]);
    cmd.output().unwrap();

    // Create second scan (different ports)
    let mut cmd2 = Command::cargo_bin("prtip").unwrap();
    cmd2.args(&[
        "-p",
        "443",
        "127.0.0.1",
        "--with-db",
        "--database",
        &db_path,
    ]);
    cmd2.output().unwrap();

    // Compare scans
    let mut cmd3 = Command::cargo_bin("prtip").unwrap();
    cmd3.args(&["db", "compare", &db_path, "1", "2"]);

    cmd3.assert()
        .success()
        .stdout(predicate::str::contains("Comparing Scan").or(predicate::str::contains("Summary")));
}

#[test]
fn test_db_query_no_filter_error() {
    let temp_dir = TempDir::new().unwrap();
    let (db_path, _target) = create_test_db_with_scan(&temp_dir);

    // Query without any filter should error
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.args(&["db", "query", &db_path]);

    cmd.assert().failure().stderr(
        predicate::str::contains("At least one filter").or(predicate::str::contains("required")),
    );
}
