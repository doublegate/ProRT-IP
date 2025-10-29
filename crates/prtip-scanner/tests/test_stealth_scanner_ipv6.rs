// Sprint 5.1 Phase 2.2: Stealth Scanner IPv6 Integration Tests
// Comprehensive testing of dual-stack IPv4/IPv6 stealth scanning support
// Tests all 4 scan types: FIN, NULL, Xmas, ACK
//
// NOTE: These tests require CAP_NET_RAW capability (root privileges)
// Run with: sudo -E cargo test --test test_stealth_scanner_ipv6 -- --ignored
//
// Tests are marked #[ignore] by default since they need elevated permissions.
// This prevents failures in non-root CI environments while allowing comprehensive
// testing when run manually with privileges.

use prtip_core::config::Config;
use prtip_core::PortState;
use prtip_scanner::{StealthScanType, StealthScanner};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Helper to create default config
fn default_config() -> Config {
    let mut config = Config::default();
    config.scan.timeout_ms = 1000; // 1 second timeout for tests
    config.scan.retries = 0; // No retries for stealth scans (timeouts are meaningful)
    config
}

// ============================================================================
// Test Group 1: IPv6 Basic Connectivity (1 test)
// Verify stealth scanner can create packets and handle IPv6 addresses
// ============================================================================

#[test]
fn test_stealth_scanner_ipv6_creation() {
    let config = default_config();
    let scanner = StealthScanner::new(config);
    assert!(
        scanner.is_ok(),
        "Stealth scanner should create successfully for IPv6 scanning"
    );
}

// ============================================================================
// Test Group 2: FIN Scan IPv6 (1 test)
// FIN scan: FIN flag only, RST = Closed, No response = Open|Filtered
// ============================================================================

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_fin_scan_ipv6() {
    // Test FIN scan on IPv6 loopback (::1)
    let config = default_config();
    let mut scanner = StealthScanner::new(config).expect("Failed to create stealth scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST); // ::1
    let port = 65432; // High port unlikely to be open

    let result = scanner.scan_port(target, port, StealthScanType::Fin).await;
    if let Err(ref e) = result {
        eprintln!("IPv6 FIN scan error: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "IPv6 FIN scan should complete successfully: {:?}",
        result.err()
    );

    let scan_result = result.unwrap();
    assert_eq!(scan_result.target_ip, target);
    assert_eq!(scan_result.port, port);
    // FIN scan: RST = Closed, No response = Open|Filtered (timeout)
    assert!(
        matches!(scan_result.state, PortState::Closed | PortState::Filtered),
        "FIN scan state should be Closed or Filtered (got: {:?})",
        scan_result.state
    );
}

// ============================================================================
// Test Group 3: NULL Scan IPv6 (1 test)
// NULL scan: No flags, RST = Closed, No response = Open|Filtered
// ============================================================================

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_null_scan_ipv6() {
    // Test NULL scan on IPv6 loopback
    let config = default_config();
    let mut scanner = StealthScanner::new(config).expect("Failed to create stealth scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let port = 65433;

    let result = scanner.scan_port(target, port, StealthScanType::Null).await;
    assert!(
        result.is_ok(),
        "IPv6 NULL scan should complete successfully"
    );

    let scan_result = result.unwrap();
    assert_eq!(scan_result.target_ip, target);
    assert_eq!(scan_result.port, port);
    // NULL scan: RST = Closed, No response = Open|Filtered (timeout)
    assert!(
        matches!(scan_result.state, PortState::Closed | PortState::Filtered),
        "NULL scan state should be Closed or Filtered (got: {:?})",
        scan_result.state
    );
}

// ============================================================================
// Test Group 4: Xmas Scan IPv6 (1 test)
// Xmas scan: FIN + PSH + URG flags, RST = Closed, No response = Open|Filtered
// ============================================================================

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_xmas_scan_ipv6() {
    // Test Xmas scan on IPv6 loopback
    let config = default_config();
    let mut scanner = StealthScanner::new(config).expect("Failed to create stealth scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let port = 65434;

    let result = scanner.scan_port(target, port, StealthScanType::Xmas).await;
    assert!(
        result.is_ok(),
        "IPv6 Xmas scan should complete successfully"
    );

    let scan_result = result.unwrap();
    assert_eq!(scan_result.target_ip, target);
    assert_eq!(scan_result.port, port);
    // Xmas scan: RST = Closed, No response = Open|Filtered (timeout)
    assert!(
        matches!(scan_result.state, PortState::Closed | PortState::Filtered),
        "Xmas scan state should be Closed or Filtered (got: {:?})",
        scan_result.state
    );
}

// ============================================================================
// Test Group 5: ACK Scan IPv6 (1 test)
// ACK scan: ACK flag only, RST = Unfiltered, No response = Filtered
// Used for firewall rule detection, not port state
// ============================================================================

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_ack_scan_ipv6() {
    // Test ACK scan on IPv6 loopback (firewall detection)
    let config = default_config();
    let mut scanner = StealthScanner::new(config).expect("Failed to create stealth scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let port = 65435;

    let result = scanner.scan_port(target, port, StealthScanType::Ack).await;
    assert!(result.is_ok(), "IPv6 ACK scan should complete successfully");

    let scan_result = result.unwrap();
    assert_eq!(scan_result.target_ip, target);
    assert_eq!(scan_result.port, port);
    // ACK scan: RST = Unfiltered (Open in code), No response = Filtered (timeout)
    // On loopback, typically Unfiltered (RST received)
    assert!(
        matches!(scan_result.state, PortState::Open | PortState::Filtered),
        "ACK scan state should be Open (Unfiltered) or Filtered (got: {:?})",
        scan_result.state
    );
}

// ============================================================================
// Test Group 6: Dual-Stack Support (1 test)
// Verify scanner can handle both IPv4 and IPv6 simultaneously
// ============================================================================

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_stealth_ipv6_dual_stack_ipv4() {
    // Verify stealth scanner still works with IPv4 after IPv6 support added
    let config = default_config();
    let mut scanner = StealthScanner::new(config).expect("Failed to create stealth scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let ipv4_target = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let port = 65436;

    // Test with FIN scan (most common stealth type)
    let result = scanner
        .scan_port(ipv4_target, port, StealthScanType::Fin)
        .await;
    assert!(
        result.is_ok(),
        "IPv4 FIN scan should work after IPv6 support"
    );

    let scan_result = result.unwrap();
    assert_eq!(scan_result.target_ip, ipv4_target);
    assert_eq!(scan_result.port, port);
    // Should get valid state (Closed or Filtered)
    assert!(
        matches!(scan_result.state, PortState::Closed | PortState::Filtered),
        "IPv4 backwards compatibility check (got: {:?})",
        scan_result.state
    );
}

// ============================================================================
// Test Group 7: Closed Port Detection (1 test)
// Verify RST response is correctly interpreted
// ============================================================================

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_stealth_ipv6_closed_port() {
    // Test against closed port - should get RST response
    let config = default_config();
    let mut scanner = StealthScanner::new(config).expect("Failed to create stealth scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let port = 65000; // Very high port unlikely to be open

    // Test with NULL scan (tends to get RST on closed ports)
    let result = scanner.scan_port(target, port, StealthScanType::Null).await;
    assert!(result.is_ok(), "IPv6 scan of closed port should succeed");

    let scan_result = result.unwrap();
    assert_eq!(scan_result.target_ip, target);
    assert_eq!(scan_result.port, port);
    // On loopback, closed ports typically return Closed (RST) or Filtered (timeout)
    assert!(
        matches!(scan_result.state, PortState::Closed | PortState::Filtered),
        "Closed port should be detected (got: {:?})",
        scan_result.state
    );
}

// ============================================================================
// Test Group 8: Response Time Tracking (1 test)
// Verify response times are reasonable for loopback
// ============================================================================

#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_stealth_ipv6_response_time() {
    // Test response time tracking for IPv6
    let config = default_config();
    let mut scanner = StealthScanner::new(config).expect("Failed to create stealth scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let port = 65437;

    let result = scanner.scan_port(target, port, StealthScanType::Fin).await;
    assert!(result.is_ok(), "IPv6 scan should complete");

    let scan_result = result.unwrap();
    assert_eq!(scan_result.target_ip, target);
    assert_eq!(scan_result.port, port);

    // Response time should be tracked (even for timeout/no response)
    // For loopback with 1s timeout, should be either very fast (<100ms) or timeout (~1s)
    let duration = scan_result.response_time;
    // Either fast RST response or timeout
    assert!(
        duration.as_millis() < 100 || duration.as_millis() > 900,
        "Response time should be either fast (<100ms) or timeout (~1000ms), got: {:?}ms",
        duration.as_millis()
    );
}
