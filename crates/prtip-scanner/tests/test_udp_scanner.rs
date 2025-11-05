// UDP Scanner Tests
// Tests for UDP port scanning functionality

use prtip_core::Config;
use prtip_scanner::UdpScanner;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Test basic UDP scan of a single port
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_scan_single_port() {
    let config = Config::default();
    let mut scanner = UdpScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V4(Ipv4Addr::LOCALHOST);

    let result = scanner.scan_port(target, 53).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.port, 53);
    assert_eq!(result.target_ip, target);
}

/// Test UDP scan of multiple ports sequentially
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_scan_multiple_ports() {
    let config = Config::default();
    let mut scanner = UdpScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let ports = vec![53, 123, 161, 137]; // DNS, NTP, SNMP, NetBIOS

    for port in ports {
        let result = scanner.scan_port(target, port).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().port, port);
    }
}

/// Test UDP scan with protocol-specific payloads (DNS)
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_scan_dns_payload() {
    let config = Config::default();
    let mut scanner = UdpScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V4(Ipv4Addr::LOCALHOST);

    // Test DNS query payload (port 53)
    let result = scanner.scan_port(target, 53).await;

    assert!(result.is_ok());
    // The scanner should have sent a DNS query payload
    // Response handling depends on whether a DNS server is running
}

/// Test UDP scan ICMP port unreachable handling
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
#[cfg(unix)] // Requires root on Unix systems to receive ICMP
async fn test_udp_scan_icmp_unreachable() {
    let config = Config::default();
    let mut scanner = UdpScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V4(Ipv4Addr::LOCALHOST);

    // Use high port unlikely to be listening
    let result = scanner.scan_port(target, 54321).await;

    assert!(result.is_ok());
    // Port should be marked as closed if ICMP unreachable received
    // Otherwise marked as open|filtered
}

/// Test UDP scan with IPv6 target
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_scan_ipv6() {
    let config = Config::default();
    let mut scanner = UdpScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);

    let result = scanner.scan_port(target, 53).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.port, 53);
    assert_eq!(result.target_ip, target);
}

/// Test UDP scanner with multiple IPv6 ports
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
async fn test_udp_scan_ipv6_multiple() {
    let config = Config::default();
    let mut scanner = UdpScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let ports = vec![53, 123];

    for port in ports {
        let result = scanner.scan_port(target, port).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().port, port);
    }
}

// Unit tests for internal functionality

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_udp_scanner_creation() {
        let config = Config::default();
        let scanner = UdpScanner::new(config);
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_udp_scanner_with_custom_timeout() {
        let mut config = Config::default();
        config.scan.timeout_ms = 100;
        let scanner = UdpScanner::new(config);
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        // Verify default configuration is valid for UDP scanning
        assert!(config.scan.timeout_ms > 0);
    }
}
