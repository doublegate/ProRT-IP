//! CLI integration tests for rate limiting flags
//!
//! Tests CLI argument parsing for --max-hostgroup, --min-hostgroup,
//! --max-parallelism, and --adaptive-rate flags.

use assert_cmd::Command;

#[test]
fn test_max_hostgroup_flag() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.arg("--max-hostgroup")
        .arg("32")
        .arg("-sT") // TCP Connect (works without raw sockets)
        .arg("-p")
        .arg("80")
        .arg("127.0.0.1");

    // Should parse successfully (may not complete scan without permissions)
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Flag should parse (not be "invalid" or "unexpected")
    assert!(
        !stderr.contains("invalid") && !stderr.contains("unexpected"),
        "Unexpected error parsing --max-hostgroup: {}",
        stderr
    );
}

#[test]
fn test_min_hostgroup_flag() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.arg("--min-hostgroup")
        .arg("1")
        .arg("-sT")
        .arg("-p")
        .arg("80")
        .arg("127.0.0.1");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("invalid") && !stderr.contains("unexpected"),
        "Unexpected error parsing --min-hostgroup: {}",
        stderr
    );
}

#[test]
fn test_max_parallelism_alias() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.arg("--max-parallelism")
        .arg("16")
        .arg("-sT")
        .arg("-p")
        .arg("80")
        .arg("127.0.0.1");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("invalid") && !stderr.contains("unexpected"),
        "Unexpected error parsing --max-parallelism: {}",
        stderr
    );
}

#[test]
fn test_adaptive_rate_flag() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.arg("--adaptive-rate")
        .arg("-sT")
        .arg("-p")
        .arg("80")
        .arg("127.0.0.1");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // May fail due to raw socket permissions, but flag should parse
    assert!(
        !stderr.contains("invalid") && !stderr.contains("unexpected"),
        "Unexpected error parsing --adaptive-rate: {}",
        stderr
    );
}

#[test]
fn test_combined_rate_limiting_flags() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.arg("--adaptive-rate")
        .arg("--max-hostgroup")
        .arg("32")
        .arg("--min-hostgroup")
        .arg("1")
        .arg("-sT")
        .arg("-p")
        .arg("80")
        .arg("127.0.0.1");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Flags should parse correctly together (no conflicts)
    assert!(
        !stderr.contains("conflict"),
        "Unexpected flag conflict: {}",
        stderr
    );
}

#[test]
fn test_max_hostgroup_validation_zero() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.arg("--max-hostgroup")
        .arg("0")
        .arg("-sT")
        .arg("-p")
        .arg("80")
        .arg("127.0.0.1");

    let output = cmd.output().unwrap();
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Max hostgroup must be greater than 0"),
        "Expected validation error, got: {}",
        stderr
    );
}

#[test]
fn test_min_hostgroup_exceeds_max() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.arg("--min-hostgroup")
        .arg("100")
        .arg("--max-hostgroup")
        .arg("10")
        .arg("-sT")
        .arg("-p")
        .arg("80")
        .arg("127.0.0.1");

    let output = cmd.output().unwrap();
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Min hostgroup") && stderr.contains("cannot exceed max hostgroup"),
        "Expected validation error, got: {}",
        stderr
    );
}

#[test]
fn test_max_hostgroup_excessive() {
    let mut cmd = Command::cargo_bin("prtip").unwrap();
    cmd.arg("--max-hostgroup")
        .arg("20000") // Exceeds 10,000 limit
        .arg("-sT")
        .arg("-p")
        .arg("80")
        .arg("127.0.0.1");

    let output = cmd.output().unwrap();
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Max hostgroup cannot exceed"),
        "Expected validation error, got: {}",
        stderr
    );
}
