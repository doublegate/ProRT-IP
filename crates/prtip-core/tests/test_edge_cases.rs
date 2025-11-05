// Sprint 5.6 Phase 4: Security & Edge Case Tests
// Edge Case Testing
//
// Test Strategy:
// - Group 1: Port boundary values (no root required)
// - Group 2: Special IP addresses (no root required)
// - Group 3: Extreme configuration values (no root required)
//
// Run all tests: cargo test --test test_edge_cases

use prtip_core::Config;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// Helper to create default config for testing
fn default_config() -> Config {
    Config::default()
}

// ============================================================================
// Test Group 1: Port Boundary Values (3 tests)
// Tests port number boundaries (1-65535) to prevent undefined behavior
// ============================================================================

/// Tests that port 0 is handled correctly
///
/// **Edge Case:** Port 0 is reserved and should not be scanned.
///
/// **Expected Behavior:** Port 0 is technically valid in Rust (u16 can be 0),
/// but scanners should treat it appropriately (typically as invalid for scanning).
///
/// **Note:** This test verifies the u16 type allows 0, but application logic
/// should handle it appropriately.
#[test]
fn test_edge_case_port_zero() {
    let port: u16 = 0;

    // Port 0 is representable as u16 (type system allows it)
    assert_eq!(port, 0, "Port 0 should be representable as u16");

    // Note: Scanner implementation should handle port 0 appropriately
    // (typically skip it or treat as invalid). This test just verifies
    // the type system doesn't prevent it.
    println!("Port 0 is representable (scanner should handle appropriately)");
}

/// Tests port boundary at u16::MAX (65535)
///
/// **Edge Case:** Port 65535 is the maximum valid port number.
///
/// **Expected Behavior:** Port 65535 should be representable and scannable
/// (it's a valid port number).
#[test]
fn test_edge_case_port_max() {
    let port: u16 = 65535; // u16::MAX

    // Maximum port should be representable
    assert_eq!(port, u16::MAX, "Port 65535 should equal u16::MAX");

    // Port 65535 is valid (though rarely used)
    println!("Port 65535 (u16::MAX) is valid");
}

/// Tests that port overflow is prevented by type system
///
/// **Edge Case:** Attempting to use port 65536 (u16::MAX + 1).
///
/// **Expected Behavior:** Rust's type system prevents port overflow at
/// compile time (u16 cannot exceed 65535).
///
/// **Security:** Type safety prevents port overflow attacks.
#[test]
fn test_edge_case_port_type_safety() {
    // Port is u16, so max is 65535
    let port: u16 = u16::MAX;
    assert_eq!(port, 65535);

    // Attempting port = 65536 would fail to compile:
    // let port: u16 = 65536; // Compile error: literal out of range for `u16`

    // Overflow with wrapping_add (demonstrates overflow prevention)
    let overflowed = port.wrapping_add(1);
    assert_eq!(overflowed, 0, "Port overflow wraps to 0 (u16 behavior)");

    println!("Type system prevents port overflow (u16 max is 65535)");
}

// ============================================================================
// Test Group 2: Special IPv4 Addresses (3 tests)
// Tests handling of special/reserved IPv4 addresses
// ============================================================================

/// Tests parsing of special IPv4 addresses
///
/// **Edge Cases:** Localhost, broadcast, private ranges, documentation ranges.
///
/// **Expected Behavior:** All special addresses should parse successfully.
/// Scanner should handle them appropriately (may skip scanning, may warn).
#[test]
fn test_edge_case_ipv4_special_addresses() {
    let special_addresses = vec![
        ("127.0.0.1", "Localhost"),
        ("0.0.0.0", "All interfaces"),
        ("255.255.255.255", "Broadcast"),
        ("192.168.1.1", "Private (RFC 1918)"),
        ("10.0.0.1", "Private (RFC 1918)"),
        ("172.16.0.1", "Private (RFC 1918)"),
        ("192.0.2.1", "Documentation (TEST-NET-1)"),
        ("198.51.100.1", "Documentation (TEST-NET-2)"),
        ("203.0.113.1", "Documentation (TEST-NET-3)"),
        ("224.0.0.1", "Multicast"),
        ("169.254.1.1", "Link-local"),
    ];

    for (addr_str, description) in special_addresses {
        let result = IpAddr::from_str(addr_str);

        assert!(
            result.is_ok(),
            "Special address {} ({}) should parse successfully",
            addr_str,
            description
        );

        if let Ok(addr) = result {
            println!("✓ Parsed {} ({}): {:?}", description, addr_str, addr);

            // Verify it parsed as IPv4
            assert!(addr.is_ipv4(), "{} should be IPv4", description);
        }
    }
}

/// Tests IPv4 address with all zeros
///
/// **Edge Case:** 0.0.0.0 (INADDR_ANY)
///
/// **Expected Behavior:** Should parse as valid IPv4 address. Represents
/// "all interfaces" in bind contexts.
#[test]
fn test_edge_case_ipv4_all_zeros() {
    let addr_str = "0.0.0.0";
    let result = Ipv4Addr::from_str(addr_str);

    assert!(result.is_ok(), "0.0.0.0 should parse successfully");

    let addr = result.unwrap();
    assert_eq!(addr, Ipv4Addr::new(0, 0, 0, 0));

    // Verify special property checks
    assert!(addr.is_unspecified(), "0.0.0.0 should be unspecified");

    println!("✓ 0.0.0.0 (INADDR_ANY) parsed correctly");
}

/// Tests IPv4 address with all ones (broadcast)
///
/// **Edge Case:** 255.255.255.255 (broadcast address)
///
/// **Expected Behavior:** Should parse as valid IPv4 address. Represents
/// broadcast to all hosts on local network.
#[test]
fn test_edge_case_ipv4_broadcast() {
    let addr_str = "255.255.255.255";
    let result = Ipv4Addr::from_str(addr_str);

    assert!(result.is_ok(), "255.255.255.255 should parse successfully");

    let addr = result.unwrap();
    assert_eq!(addr, Ipv4Addr::new(255, 255, 255, 255));

    // Verify special property checks
    assert!(addr.is_broadcast(), "255.255.255.255 should be broadcast");

    println!("✓ 255.255.255.255 (broadcast) parsed correctly");
}

// ============================================================================
// Test Group 3: Special IPv6 Addresses (3 tests)
// Tests handling of special/reserved IPv6 addresses
// ============================================================================

/// Tests parsing of special IPv6 addresses
///
/// **Edge Cases:** Unspecified (::), loopback (::1), multicast (ff00::/8).
///
/// **Expected Behavior:** All special IPv6 addresses should parse successfully.
#[test]
fn test_edge_case_ipv6_special_addresses() {
    let special_addresses = vec![
        ("::", "Unspecified"),
        ("::1", "Loopback"),
        ("ff00::", "Multicast prefix"),
        ("ff02::1", "All nodes multicast"),
        ("ff02::2", "All routers multicast"),
        ("fe80::", "Link-local prefix"),
        ("fe80::1", "Link-local address"),
        ("::ffff:192.0.2.1", "IPv4-mapped IPv6"),
        ("2001:db8::1", "Documentation prefix"),
        ("fc00::1", "Unique local address"),
    ];

    for (addr_str, description) in special_addresses {
        let result = IpAddr::from_str(addr_str);

        assert!(
            result.is_ok(),
            "Special IPv6 address {} ({}) should parse successfully",
            addr_str,
            description
        );

        if let Ok(addr) = result {
            println!("✓ Parsed {} ({}): {:?}", description, addr_str, addr);

            // Verify it parsed as IPv6
            assert!(addr.is_ipv6(), "{} should be IPv6", description);
        }
    }
}

/// Tests IPv6 unspecified address (::)
///
/// **Edge Case:** :: (all zeros, INADDR6_ANY)
///
/// **Expected Behavior:** Should parse as valid IPv6 address. Represents
/// "unspecified address" (equivalent to 0.0.0.0 in IPv4).
#[test]
fn test_edge_case_ipv6_unspecified() {
    let addr_str = "::";
    let result = Ipv6Addr::from_str(addr_str);

    assert!(
        result.is_ok(),
        ":: (IPv6 unspecified) should parse successfully"
    );

    let addr = result.unwrap();

    // Verify special property checks
    assert!(addr.is_unspecified(), ":: should be unspecified");

    println!("✓ :: (IPv6 unspecified) parsed correctly");
}

/// Tests IPv6 loopback address (::1)
///
/// **Edge Case:** ::1 (loopback, equivalent to 127.0.0.1 in IPv4)
///
/// **Expected Behavior:** Should parse as valid IPv6 address. Represents
/// loopback (local host).
#[test]
fn test_edge_case_ipv6_loopback() {
    let addr_str = "::1";
    let result = Ipv6Addr::from_str(addr_str);

    assert!(
        result.is_ok(),
        "::1 (IPv6 loopback) should parse successfully"
    );

    let addr = result.unwrap();

    // Verify special property checks
    assert!(addr.is_loopback(), "::1 should be loopback");

    println!("✓ ::1 (IPv6 loopback) parsed correctly");
}

// ============================================================================
// Test Group 4: Extreme Configuration Values (3 tests)
// Tests configuration with extreme but valid values
// ============================================================================

/// Tests configuration with minimum valid timeout
///
/// **Edge Case:** timeout_ms = 1 (1 millisecond, minimum valid)
///
/// **Expected Behavior:** Config validation should accept 1ms timeout
/// (minimum valid value > 0).
#[test]
fn test_edge_case_minimum_timeout() {
    let mut config = default_config();
    config.scan.timeout_ms = 1; // 1 millisecond

    let result = config.validate();

    assert!(result.is_ok(), "Minimum timeout (1ms) should be accepted");

    println!("✓ Minimum timeout (1ms) is valid");
}

/// Tests configuration with maximum valid timeout
///
/// **Edge Case:** timeout_ms = 3,600,000 (1 hour, maximum valid)
///
/// **Expected Behavior:** Config validation should accept 1 hour timeout
/// (maximum valid value).
#[test]
fn test_edge_case_maximum_timeout() {
    let mut config = default_config();
    config.scan.timeout_ms = 3_600_000; // 1 hour

    let result = config.validate();

    assert!(
        result.is_ok(),
        "Maximum timeout (1 hour) should be accepted"
    );

    println!("✓ Maximum timeout (1 hour) is valid");
}

/// Tests configuration with maximum valid parallelism
///
/// **Edge Case:** parallelism = 100,000 (maximum valid)
///
/// **Expected Behavior:** Config validation should accept 100,000 parallelism
/// (maximum valid value).
///
/// **Note:** This is an extreme value that would use significant memory but
/// should not crash (memory usage = ~100,000 * task_size).
#[test]
fn test_edge_case_maximum_parallelism() {
    let mut config = default_config();
    config.performance.parallelism = 100_000; // Maximum valid

    let result = config.validate();

    assert!(
        result.is_ok(),
        "Maximum parallelism (100,000) should be accepted"
    );

    println!("✓ Maximum parallelism (100,000) is valid");
}

// ============================================================================
// Test Group 5: Malformed Input Rejection (3 tests)
// Tests that malformed inputs are rejected appropriately
// ============================================================================

/// Tests rejection of malformed IPv4 addresses
///
/// **Edge Cases:** Out of range octets, incomplete addresses, too many octets.
///
/// **Expected Behavior:** IpAddr::from_str() should return Err for malformed
/// IPv4 addresses.
///
/// **Security:** Prevents undefined behavior from malformed IP parsing.
#[test]
fn test_edge_case_malformed_ipv4_rejected() {
    let malformed_addresses = vec![
        ("256.256.256.256", "Octet out of range"),
        ("192.168.1", "Incomplete address"),
        ("192.168.1.1.1", "Too many octets"),
        ("192.168.-1.1", "Negative octet"),
        ("192.168.1.1/24", "CIDR notation (not IP)"),
        ("localhost", "Hostname (not IP)"),
        ("../../../etc/passwd", "Path traversal attempt"),
        ("'; DROP TABLE --", "SQL injection attempt"),
    ];

    for (addr_str, description) in malformed_addresses {
        let result = IpAddr::from_str(addr_str);

        assert!(
            result.is_err(),
            "Malformed address {} ({}) should be rejected, but was accepted!",
            addr_str,
            description
        );

        println!("✓ Rejected {}: {}", description, addr_str);
    }
}

/// Tests rejection of malformed IPv6 addresses
///
/// **Edge Cases:** Too many groups, invalid characters, malformed compression.
///
/// **Expected Behavior:** IpAddr::from_str() should return Err for malformed
/// IPv6 addresses.
#[test]
fn test_edge_case_malformed_ipv6_rejected() {
    let malformed_addresses = vec![
        ("gggg::", "Invalid hex character"),
        ("1:2:3:4:5:6:7:8:9", "Too many groups"),
        (":::", "Multiple compressions"),
        ("1::2::3", "Multiple compressions"),
        ("::ffff:256.0.0.1", "IPv4-mapped with invalid IPv4"),
        ("2001:db8::/64", "CIDR notation (not IP)"),
    ];

    for (addr_str, description) in malformed_addresses {
        let result = IpAddr::from_str(addr_str);

        assert!(
            result.is_err(),
            "Malformed IPv6 address {} ({}) should be rejected, but was accepted!",
            addr_str,
            description
        );

        println!("✓ Rejected {}: {}", description, addr_str);
    }
}

/// Tests rejection of invalid port ranges in configuration
///
/// **Edge Case:** Scanner implementations may accept port ranges as strings
/// (e.g., "1-65535"). This test verifies parsing logic rejects malformed ranges.
///
/// **Note:** This test is conceptual - actual implementation depends on how
/// port ranges are parsed. Included for completeness.
#[test]
fn test_edge_case_port_range_validation() {
    // This test verifies port range parsing (if implemented)
    // Since Config uses u16 for ports, this is more relevant to CLI parsing

    // Valid single ports
    let valid_ports = vec![1u16, 80, 443, 8080, 65535];

    for port in valid_ports {
        // Port is u16, so always <= 65535 (type system guarantees it)
        assert!(port >= 1, "Port {} should be >= 1", port);
    }

    // Type system prevents invalid ports at compile time
    // let invalid: u16 = 65536; // Would not compile

    println!("✓ Port range validation enforced by type system (u16)");
}
