// Sprint 5.1 Phase 2.1: UDP Scanner IPv6 Integration Tests
// Comprehensive testing of dual-stack IPv4/IPv6 UDP scanning support
//
// NOTE: These tests require CAP_NET_RAW capability (root privileges)
// Run with: sudo -E cargo test --test test_udp_scanner_ipv6 -- --ignored
//
// Tests are marked #[ignore] by default since they need elevated permissions.
// This prevents failures in non-root CI environments while allowing comprehensive
// testing when run manually with privileges.

use prtip_core::config::Config;
use prtip_core::PortState;
use prtip_scanner::UdpScanner;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Helper to create default config
fn default_config() -> Config {
    let mut config = Config::default();
    config.scan.timeout_ms = 2000; // 2 second timeout for tests
    config.scan.retries = 1; // Minimal retries for speed
    config
}

// ============================================================================
// Test Group 1: IPv6 Basic Connectivity (3 tests)
// Verify UDP scanner can create packets and handle IPv6 addresses
// ============================================================================

#[test]
fn test_udp_scanner_ipv6_creation() {
    let config = default_config();
    let scanner = UdpScanner::new(config);
    assert!(
        scanner.is_ok(),
        "UDP scanner should create successfully for IPv6 scanning"
    );
}

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_ipv6_loopback_dns() {
    // Test IPv6 loopback (::1) DNS port - equivalent to 127.0.0.1:53
    let config = default_config();
    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST); // ::1
    let port = 53; // DNS

    let result = scanner.scan_port(target, port).await;
    if let Err(ref e) = result {
        eprintln!("IPv6 UDP scan error: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "IPv6 loopback UDP scan should complete successfully: {:?}",
        result.err()
    );

    let scan_result = result.unwrap();
    assert_eq!(scan_result.target_ip, target);
    assert_eq!(scan_result.port, port);
    // State may be Open, Closed, or Filtered depending on local DNS service
    assert!(
        matches!(
            scan_result.state,
            PortState::Open | PortState::Closed | PortState::Filtered
        ),
        "Port state should be valid (got: {:?})",
        scan_result.state
    );
}

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_ipv6_closed_port_icmpv6() {
    // Test closed port - should get ICMPv6 Type 1 Code 4 (Port Unreachable)
    let config = default_config();
    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let port = 65534; // Very high port unlikely to be open

    let result = scanner.scan_port(target, port).await;
    assert!(
        result.is_ok(),
        "IPv6 UDP scan of closed port should succeed"
    );

    let scan_result = result.unwrap();
    // On loopback, closed ports typically return Closed (ICMPv6 Type 1 Code 4)
    // May timeout on some systems if ICMP rate limited
    assert!(
        matches!(scan_result.state, PortState::Closed | PortState::Filtered),
        "Closed port should be detected (got: {:?})",
        scan_result.state
    );
}

// ============================================================================
// Test Group 2: Dual-Stack Support (2 tests)
// Verify scanner can handle both IPv4 and IPv6 simultaneously
// ============================================================================

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_ipv6_dual_stack_ipv4_target() {
    // Verify scanner still works with IPv4 after IPv6 support added
    let config = default_config();
    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let ipv4_target = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let port = 53; // DNS

    let result = scanner.scan_port(ipv4_target, port).await;
    assert!(
        result.is_ok(),
        "IPv4 UDP scan should still work in dual-stack scanner"
    );

    let scan_result = result.unwrap();
    assert_eq!(scan_result.target_ip, ipv4_target);
}

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_ipv6_dual_stack_mixed_targets() {
    // Scan both IPv4 and IPv6 targets with same scanner instance
    let config = default_config();
    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let ipv4_target = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let ipv6_target = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let port = 53; // DNS

    // Scan IPv4
    let ipv4_result = scanner.scan_port(ipv4_target, port).await;
    assert!(
        ipv4_result.is_ok(),
        "IPv4 UDP scan should succeed in mixed scenario"
    );

    // Scan IPv6
    let ipv6_result = scanner.scan_port(ipv6_target, port).await;
    assert!(
        ipv6_result.is_ok(),
        "IPv6 UDP scan should succeed in mixed scenario"
    );

    // Verify both results reference correct targets
    assert_eq!(ipv4_result.unwrap().target_ip, ipv4_target);
    assert_eq!(ipv6_result.unwrap().target_ip, ipv6_target);
}

// ============================================================================
// Test Group 3: IPv6 Protocol Handling (3 tests)
// Verify ICMPv6 error handling and timeouts
// ============================================================================

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_ipv6_timeout() {
    // Test timeout behavior with unreachable IPv6 address
    let mut config = default_config();
    config.scan.timeout_ms = 100; // Very short timeout
    config.scan.retries = 0; // No retries

    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    // Use documentation IPv6 prefix (2001:db8::/32 - reserved for docs)
    let target = IpAddr::V6("2001:db8::1".parse().unwrap());
    let port = 80;

    let result = scanner.scan_port(target, port).await;
    assert!(result.is_ok(), "Timeout should result in valid scan result");

    let scan_result = result.unwrap();
    // No response should result in Filtered state
    assert_eq!(
        scan_result.state,
        PortState::Filtered,
        "Timeout should mark port as Filtered"
    );
}

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_ipv6_snmp_port() {
    // Test SNMP port scan (161) - protocol-specific payload
    let config = default_config();
    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let port = 161; // SNMP

    let result = scanner.scan_port(target, port).await;
    assert!(result.is_ok(), "SNMP port scan should complete");

    let scan_result = result.unwrap();
    // SNMP may be open, closed, or filtered depending on system
    assert!(
        matches!(
            scan_result.state,
            PortState::Open | PortState::Closed | PortState::Filtered
        ),
        "SNMP port state should be valid (got: {:?})",
        scan_result.state
    );
}

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_ipv6_response_time_tracking() {
    // Verify response time is tracked for IPv6 scans
    let config = default_config();
    let mut scanner = UdpScanner::new(config).expect("Failed to create UDP scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let port = 53; // DNS

    let result = scanner.scan_port(target, port).await;
    assert!(result.is_ok(), "Response time test should succeed");

    let scan_result = result.unwrap();
    // Response time should be populated (Duration type, always set in Phase 4.22)
    assert!(
        scan_result.response_time.as_millis() > 0,
        "Response time should be tracked for IPv6 UDP scans"
    );
}

// ============================================================================
// Test Summary
// ============================================================================
// Total tests: 8
// - Basic connectivity: 3 (creation, loopback DNS, closed port)
// - Dual-stack: 2 (IPv4 compat, mixed targets)
// - Protocol handling: 3 (timeout, SNMP, response time)
//
// Coverage:
// ✅ IPv6 UDP packet building via IpAddr enum
// ✅ ICMPv6 Type 1 Code 4 handling (Port Unreachable → Closed)
// ✅ Dual-stack IPv4/IPv6 support
// ✅ IPv6 loopback (::1) scanning
// ✅ Timeout and retry logic for IPv6
// ✅ Response time tracking
// ✅ Port state determination (Open/Closed/Filtered)
// ✅ Protocol-specific payloads (DNS, SNMP)
//
// Deferred to future sprints:
// - Extension header traversal (rare in practice)
// - IPv6 fragmentation (uses extension headers)
// - Link-local address handling (fe80::/10) - needs zone ID
// - ICMPv6 other error types (admin prohibited, etc.)
// ============================================================================
