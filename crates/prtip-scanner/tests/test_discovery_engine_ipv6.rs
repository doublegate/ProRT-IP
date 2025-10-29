//! Integration tests for Discovery Engine IPv6 support
//!
//! These tests verify ICMPv6 Echo Request/Reply and NDP (Neighbor Discovery Protocol)
//! functionality for IPv6 host discovery.
//!
//! **Requirements:** CAP_NET_RAW capability (root/sudo)
//!
//! Run with: `sudo -E cargo test --test test_discovery_engine_ipv6 -- --ignored`

use prtip_scanner::discovery::{DiscoveryEngine, DiscoveryMethod};
use std::net::{IpAddr, Ipv6Addr};
use std::time::Duration;

/// Test 1: Discovery engine creation with ICMPv6 Echo method
#[test]
fn test_discovery_engine_icmpv6_creation() {
    let engine = DiscoveryEngine::new(Duration::from_secs(2), DiscoveryMethod::IcmpEcho);
    assert!(
        matches!(engine.method(), DiscoveryMethod::IcmpEcho),
        "Discovery method should be IcmpEcho"
    );
    assert_eq!(
        engine.timeout(),
        Duration::from_secs(2),
        "Timeout should be 2 seconds"
    );
}

/// Test 2: ICMPv6 Echo Request/Reply to IPv6 loopback (::1)
#[test]
#[ignore] // Requires CAP_NET_RAW
fn test_icmpv6_echo_loopback() {
    let engine = DiscoveryEngine::new(Duration::from_secs(2), DiscoveryMethod::IcmpEcho);

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST); // ::1

    // IPv6 loopback should respond to ICMP Echo
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { engine.is_host_alive(target).await });

    assert!(result.is_ok(), "ICMPv6 Echo failed: {:?}", result.err());
    assert!(
        result.unwrap(),
        "IPv6 loopback should respond to ICMPv6 Echo"
    );
}

/// Test 3: ICMPv6 Echo timeout for unreachable address
#[test]
#[ignore] // Requires CAP_NET_RAW
fn test_icmpv6_echo_timeout() {
    let engine = DiscoveryEngine::new(Duration::from_millis(500), DiscoveryMethod::IcmpEcho);

    // Use documentation prefix (2001:db8::/32) - reserved, non-routable
    let target = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { engine.is_host_alive(target).await });

    assert!(
        result.is_ok(),
        "ICMPv6 Echo should handle timeout gracefully"
    );
    assert!(
        !result.unwrap(),
        "Unreachable address should timeout (return false)"
    );
}

/// Test 4: Dual-stack - IPv4 backwards compatibility (verify no regression)
#[test]
#[ignore] // Requires CAP_NET_RAW
fn test_discovery_dual_stack_ipv4_compat() {
    let engine = DiscoveryEngine::new(Duration::from_secs(2), DiscoveryMethod::IcmpEcho);

    let ipv4_target = "127.0.0.1".parse::<IpAddr>().unwrap();

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { engine.is_host_alive(ipv4_target).await });

    assert!(result.is_ok(), "IPv4 ICMP Echo should still work");
    assert!(result.unwrap(), "IPv4 loopback should respond");
}

/// Test 5: NDP Neighbor Discovery (requires link-local or on-link address)
#[test]
#[ignore] // Requires CAP_NET_RAW + network configuration
fn test_ndp_neighbor_discovery() {
    let engine = DiscoveryEngine::new(Duration::from_secs(3), DiscoveryMethod::Arp);

    // Use IPv6 link-local address (fe80::1) - common gateway address
    let target = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { engine.is_host_alive(target).await });

    // NDP may succeed or timeout depending on network configuration
    // Test validates that NDP runs without panic/error
    assert!(
        result.is_ok(),
        "NDP should handle link-local addresses gracefully"
    );
}

/// Test 6: NDP fallback to ICMPv6 Echo for loopback
#[test]
#[ignore] // Requires CAP_NET_RAW
fn test_ndp_loopback_fallback() {
    let engine = DiscoveryEngine::new(Duration::from_secs(2), DiscoveryMethod::Arp);

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST); // ::1

    // NDP should fall back to ICMPv6 Echo for loopback
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { engine.is_host_alive(target).await });

    assert!(
        result.is_ok(),
        "NDP loopback fallback failed: {:?}",
        result.err()
    );
    assert!(
        result.unwrap(),
        "Loopback should respond via ICMPv6 Echo fallback"
    );
}

/// Test 7: Response time tracking (verify Duration is measured)
#[test]
#[ignore] // Requires CAP_NET_RAW
fn test_icmpv6_response_time_tracking() {
    use std::time::Instant;

    let engine = DiscoveryEngine::new(Duration::from_secs(2), DiscoveryMethod::IcmpEcho);

    let target = IpAddr::V6(Ipv6Addr::LOCALHOST); // ::1

    let start = Instant::now();
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { engine.is_host_alive(target).await });
    let duration = start.elapsed();

    assert!(result.is_ok(), "ICMPv6 Echo failed");
    assert!(result.unwrap(), "Loopback should respond");

    // Loopback response should be very fast (< 100ms)
    assert!(
        duration < Duration::from_millis(100),
        "ICMPv6 Echo to loopback took too long: {:?}",
        duration
    );
}
