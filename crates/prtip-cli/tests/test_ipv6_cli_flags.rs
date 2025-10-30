//! Tests for IPv6 CLI flags (-6, -4, --dual-stack)
//!
//! Sprint 5.1 Phase 4.1: CLI Integration for IPv6
//!
//! Tests validate:
//! - Flag parsing (-6, -4, --dual-stack)
//! - Mutual exclusivity enforcement
//! - Target protocol validation
//! - Error messages for protocol mismatches
//! - Default behavior (dual-stack)

use clap::Parser;
use prtip_cli::args::{Args, IpVersion};

// ============================================================================
// Unit Tests: Flag Behavior
// ============================================================================

#[test]
fn test_ipv6_flag_forces_v6_only() {
    let args = Args::parse_from(["prtip", "-6", "-p", "80", "::1"]);
    assert!(args.ipv6);
    assert!(!args.ipv4);
    assert!(!args.dual_stack);
    assert_eq!(args.get_ip_version(), IpVersion::V6Only);
}

#[test]
fn test_ipv4_flag_forces_v4_only() {
    let args = Args::parse_from(["prtip", "-4", "-p", "80", "127.0.0.1"]);
    assert!(!args.ipv6);
    assert!(args.ipv4);
    assert!(!args.dual_stack);
    assert_eq!(args.get_ip_version(), IpVersion::V4Only);
}

#[test]
fn test_dual_stack_allows_both() {
    let args = Args::parse_from(["prtip", "--dual-stack", "-p", "80", "192.168.1.1"]);
    assert!(!args.ipv6);
    assert!(!args.ipv4);
    assert!(args.dual_stack);
    assert_eq!(args.get_ip_version(), IpVersion::DualStack);
}

#[test]
fn test_default_is_dual_stack() {
    // No -4, -6, or --dual-stack flags specified
    let args = Args::parse_from(["prtip", "-p", "80", "192.168.1.1"]);
    assert!(!args.ipv6);
    assert!(!args.ipv4);
    assert!(!args.dual_stack);
    // get_ip_version() should return DualStack by default
    assert_eq!(args.get_ip_version(), IpVersion::DualStack);
}

#[test]
fn test_ipv6_long_form() {
    let args = Args::parse_from(["prtip", "--ipv6", "-p", "80", "::1"]);
    assert!(args.ipv6);
    assert_eq!(args.get_ip_version(), IpVersion::V6Only);
}

#[test]
fn test_ipv4_long_form() {
    let args = Args::parse_from(["prtip", "--ipv4", "-p", "80", "127.0.0.1"]);
    assert!(args.ipv4);
    assert_eq!(args.get_ip_version(), IpVersion::V4Only);
}

// ============================================================================
// Unit Tests: Mutual Exclusivity
// ============================================================================

#[test]
fn test_mutual_exclusivity_v4_v6() {
    // -4 and -6 together should fail due to ArgGroup
    let result = Args::try_parse_from(["prtip", "-4", "-6", "-p", "80", "127.0.0.1"]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("cannot be used with") || err.contains("mutually exclusive"),
        "Expected mutual exclusivity error, got: {}",
        err
    );
}

#[test]
fn test_mutual_exclusivity_dual_stack_v6() {
    // --dual-stack and -6 together should fail due to ArgGroup
    let result = Args::try_parse_from(["prtip", "--dual-stack", "-6", "-p", "80", "::1"]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("cannot be used with") || err.contains("mutually exclusive"),
        "Expected mutual exclusivity error, got: {}",
        err
    );
}

#[test]
fn test_mutual_exclusivity_dual_stack_v4() {
    // --dual-stack and -4 together should fail due to ArgGroup
    let result = Args::try_parse_from(["prtip", "--dual-stack", "-4", "-p", "80", "127.0.0.1"]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("cannot be used with") || err.contains("mutually exclusive"),
        "Expected mutual exclusivity error, got: {}",
        err
    );
}

// ============================================================================
// Unit Tests: Target Validation
// ============================================================================

#[test]
fn test_v6_rejects_ipv4_targets() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-6", "-p", "80", "192.168.1.1"]);

    // Parse target (will be IPv4)
    let target = ScanTarget::parse("192.168.1.1").expect("Valid IPv4 target");
    let targets = vec![target];

    // Validate should fail
    let result = args.validate_target_protocols(&targets);
    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("IPv4 target") && err.contains("-6"),
        "Expected IPv4/IPv6 mismatch error, got: {}",
        err
    );
}

#[test]
fn test_v4_rejects_ipv6_targets() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-4", "-p", "80", "::1"]);

    // Parse target (will be IPv6)
    let target = ScanTarget::parse("::1").expect("Valid IPv6 target");
    let targets = vec![target];

    // Validate should fail
    let result = args.validate_target_protocols(&targets);
    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("IPv6 target") && err.contains("-4"),
        "Expected IPv6/IPv4 mismatch error, got: {}",
        err
    );
}

#[test]
fn test_v6_accepts_ipv6_targets() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-6", "-p", "80", "::1"]);

    let target = ScanTarget::parse("::1").expect("Valid IPv6 target");
    let targets = vec![target];

    // Validate should succeed
    let result = args.validate_target_protocols(&targets);
    assert!(
        result.is_ok(),
        "IPv6 target should be accepted with -6 flag"
    );
}

#[test]
fn test_v4_accepts_ipv4_targets() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-4", "-p", "80", "192.168.1.1"]);

    let target = ScanTarget::parse("192.168.1.1").expect("Valid IPv4 target");
    let targets = vec![target];

    // Validate should succeed
    let result = args.validate_target_protocols(&targets);
    assert!(
        result.is_ok(),
        "IPv4 target should be accepted with -4 flag"
    );
}

#[test]
fn test_dual_stack_accepts_ipv4() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "--dual-stack", "-p", "80", "192.168.1.1"]);

    let target = ScanTarget::parse("192.168.1.1").expect("Valid IPv4 target");
    let targets = vec![target];

    let result = args.validate_target_protocols(&targets);
    assert!(result.is_ok(), "Dual-stack should accept IPv4 targets");
}

#[test]
fn test_dual_stack_accepts_ipv6() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "--dual-stack", "-p", "80", "::1"]);

    let target = ScanTarget::parse("::1").expect("Valid IPv6 target");
    let targets = vec![target];

    let result = args.validate_target_protocols(&targets);
    assert!(result.is_ok(), "Dual-stack should accept IPv6 targets");
}

#[test]
fn test_dual_stack_accepts_mixed_targets() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "--dual-stack", "-p", "80", "192.168.1.1", "::1"]);

    let target1 = ScanTarget::parse("192.168.1.1").expect("Valid IPv4 target");
    let target2 = ScanTarget::parse("::1").expect("Valid IPv6 target");
    let targets = vec![target1, target2];

    let result = args.validate_target_protocols(&targets);
    assert!(
        result.is_ok(),
        "Dual-stack should accept mixed IPv4/IPv6 targets"
    );
}

#[test]
fn test_default_accepts_mixed_targets() {
    use prtip_core::ScanTarget;

    // No -4, -6, or --dual-stack (default behavior is dual-stack)
    let args = Args::parse_from(["prtip", "-p", "80", "192.168.1.1", "::1"]);

    let target1 = ScanTarget::parse("192.168.1.1").expect("Valid IPv4 target");
    let target2 = ScanTarget::parse("::1").expect("Valid IPv6 target");
    let targets = vec![target1, target2];

    let result = args.validate_target_protocols(&targets);
    assert!(
        result.is_ok(),
        "Default (dual-stack) should accept mixed targets"
    );
}

// ============================================================================
// Unit Tests: Error Message Quality
// ============================================================================

#[test]
fn test_error_messages_clear_ipv4_with_v6_flag() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-6", "-p", "80", "192.168.1.1"]);
    let target = ScanTarget::parse("192.168.1.1").unwrap();
    let targets = vec![target];

    let result = args.validate_target_protocols(&targets);
    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    // Should contain helpful error message with hints
    assert!(
        err.contains("192.168.1.1"),
        "Error should mention the target IP"
    );
    assert!(
        err.contains("-6") || err.contains("IPv6-only"),
        "Error should mention -6 flag"
    );
    assert!(
        err.contains("Hint") || err.contains("dual-stack"),
        "Error should provide hint"
    );
}

#[test]
fn test_error_messages_clear_ipv6_with_v4_flag() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-4", "-p", "80", "::1"]);
    let target = ScanTarget::parse("::1").unwrap();
    let targets = vec![target];

    let result = args.validate_target_protocols(&targets);
    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    assert!(err.contains("::1"), "Error should mention the target IP");
    assert!(
        err.contains("-4") || err.contains("IPv4-only"),
        "Error should mention -4 flag"
    );
    assert!(
        err.contains("Hint") || err.contains("dual-stack"),
        "Error should provide hint"
    );
}

// ============================================================================
// Unit Tests: IP Version Combinations
// ============================================================================

#[test]
fn test_ipv6_with_link_local() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-6", "-p", "80", "fe80::1"]);
    let target = ScanTarget::parse("fe80::1").expect("Valid link-local IPv6");
    let targets = vec![target];

    let result = args.validate_target_protocols(&targets);
    assert!(
        result.is_ok(),
        "Link-local IPv6 should be accepted with -6 flag"
    );
}

#[test]
fn test_ipv6_with_ula() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-6", "-p", "80", "fd00::1"]);
    let target = ScanTarget::parse("fd00::1").expect("Valid ULA IPv6");
    let targets = vec![target];

    let result = args.validate_target_protocols(&targets);
    assert!(result.is_ok(), "ULA IPv6 should be accepted with -6 flag");
}

#[test]
fn test_ipv6_with_global() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-6", "-p", "80", "2001:db8::1"]);
    let target = ScanTarget::parse("2001:db8::1").expect("Valid global IPv6");
    let targets = vec![target];

    let result = args.validate_target_protocols(&targets);
    assert!(
        result.is_ok(),
        "Global IPv6 should be accepted with -6 flag"
    );
}

// ============================================================================
// Integration Tests: Full Command Line
// ============================================================================

#[test]
fn test_cli_help_includes_ipv6_flags() {
    // Verify help text includes IPv6 OPTIONS section
    let result = Args::try_parse_from(["prtip", "--help"]);
    assert!(result.is_err()); // --help returns Err with help text

    let help_text = result.unwrap_err().to_string();
    assert!(
        help_text.contains("IPv6 OPTIONS")
            || help_text.contains("ipv6")
            || help_text.contains("-6"),
        "Help text should mention IPv6 options"
    );
}

#[test]
fn test_cli_ipv6_with_scan_type() {
    let args = Args::parse_from(["prtip", "-6", "--nmap-syn", "-p", "80,443", "::1"]);
    assert_eq!(args.get_ip_version(), IpVersion::V6Only);
    assert!(args.nmap_syn);
}

#[test]
fn test_cli_ipv4_with_scan_type() {
    let args = Args::parse_from(["prtip", "-4", "--nmap-connect", "-p", "80,443", "127.0.0.1"]);
    assert_eq!(args.get_ip_version(), IpVersion::V4Only);
    assert!(args.nmap_connect);
}

#[test]
fn test_cli_dual_stack_with_multiple_targets() {
    let args = Args::parse_from([
        "prtip",
        "--dual-stack",
        "-p",
        "80",
        "192.168.1.1",
        "::1",
        "10.0.0.1",
        "fe80::1",
    ]);
    assert_eq!(args.get_ip_version(), IpVersion::DualStack);
    assert_eq!(args.targets.len(), 4);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_multiple_ipv6_targets_with_v6_flag() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-6", "-p", "80", "::1", "::2", "::3"]);

    let targets: Vec<_> = ["::1", "::2", "::3"]
        .iter()
        .map(|s| ScanTarget::parse(s).unwrap())
        .collect();

    let result = args.validate_target_protocols(&targets);
    assert!(
        result.is_ok(),
        "Multiple IPv6 targets should be accepted with -6 flag"
    );
}

#[test]
fn test_cidr_ipv4_with_v4_flag() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-4", "-p", "80", "192.168.1.0/24"]);

    let target = ScanTarget::parse("192.168.1.0/24").expect("Valid IPv4 CIDR");
    let targets = vec![target];

    let result = args.validate_target_protocols(&targets);
    assert!(result.is_ok(), "IPv4 CIDR should be accepted with -4 flag");
}

#[test]
fn test_cidr_ipv6_with_v6_flag() {
    use prtip_core::ScanTarget;

    let args = Args::parse_from(["prtip", "-6", "-p", "80", "2001:db8::/32"]);

    let target = ScanTarget::parse("2001:db8::/32").expect("Valid IPv6 CIDR");
    let targets = vec![target];

    let result = args.validate_target_protocols(&targets);
    assert!(result.is_ok(), "IPv6 CIDR should be accepted with -6 flag");
}
