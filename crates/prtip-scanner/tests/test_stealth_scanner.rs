// Stealth Scanner Tests
// Tests for FIN, NULL, and Xmas scan techniques

use prtip_core::Config;
use prtip_scanner::{StealthScanType, StealthScanner};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Test FIN scan of a single port
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
#[cfg(unix)] // FIN scans require raw sockets
async fn test_fin_scan_single_port() {
    let config = Config::default();
    let mut scanner = StealthScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V4(Ipv4Addr::LOCALHOST);

    let result = scanner.scan_port(target, 80, StealthScanType::Fin).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.port, 80);
    assert_eq!(result.target_ip, target);
}

/// Test NULL scan of multiple ports
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
#[cfg(unix)] // NULL scans require raw sockets
async fn test_null_scan_multiple_ports() {
    let config = Config::default();
    let mut scanner = StealthScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let ports = vec![22, 80, 443];

    for port in ports {
        let result = scanner.scan_port(target, port, StealthScanType::Null).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().port, port);
    }
}

/// Test Xmas scan
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
#[cfg(unix)] // Xmas scans require raw sockets
async fn test_xmas_scan() {
    let config = Config::default();
    let mut scanner = StealthScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let ports = vec![80, 443, 8080];

    for port in ports {
        let result = scanner.scan_port(target, port, StealthScanType::Xmas).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().port, port);
    }
}

/// Test ACK scan for firewall detection
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
#[cfg(unix)]
async fn test_ack_scan() {
    let config = Config::default();
    let mut scanner = StealthScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V4(Ipv4Addr::LOCALHOST);

    let result = scanner.scan_port(target, 80, StealthScanType::Ack).await;

    assert!(result.is_ok());
    // ACK scan is for firewall detection, not port state
}

/// Test FIN scan with IPv6
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
#[cfg(unix)]
async fn test_fin_scan_ipv6() {
    let config = Config::default();
    let mut scanner = StealthScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);

    let result = scanner.scan_port(target, 80, StealthScanType::Fin).await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.port, 80);
    assert_eq!(result.target_ip, target);
}

/// Test NULL scan with IPv6
#[tokio::test]
#[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
#[cfg(unix)]
async fn test_null_scan_ipv6() {
    let config = Config::default();
    let mut scanner = StealthScanner::new(config).expect("Failed to create scanner");
    scanner
        .initialize()
        .await
        .expect("Failed to initialize scanner");
    let target = IpAddr::V6(Ipv6Addr::LOCALHOST);
    let ports = vec![80, 443];

    for port in ports {
        let result = scanner.scan_port(target, port, StealthScanType::Null).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().port, port);
    }
}

// Unit tests for stealth scanner components

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_stealth_scanner_creation() {
        let config = Config::default();
        let scanner = StealthScanner::new(config);
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_stealth_scan_type_fin() {
        let scan_type = StealthScanType::Fin;
        assert_eq!(scan_type.name(), "FIN");
    }

    #[test]
    fn test_stealth_scan_type_null() {
        let scan_type = StealthScanType::Null;
        assert_eq!(scan_type.name(), "NULL");
    }

    #[test]
    fn test_stealth_scan_type_xmas() {
        let scan_type = StealthScanType::Xmas;
        assert_eq!(scan_type.name(), "Xmas");
    }

    #[test]
    fn test_stealth_scan_type_ack() {
        let scan_type = StealthScanType::Ack;
        assert_eq!(scan_type.name(), "ACK");
    }

    #[test]
    fn test_config_with_custom_timeout() {
        let mut config = Config::default();
        config.scan.timeout_ms = 500;
        let scanner = StealthScanner::new(config);
        assert!(scanner.is_ok());
    }
}

// Integration tests for real-world scenarios

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test that all stealth scan types can be executed
    #[tokio::test]
    #[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
    #[cfg(unix)]
    async fn test_all_scan_types() {
        let config = Config::default();
        let mut scanner = StealthScanner::new(config).expect("Failed to create scanner");
        scanner
            .initialize()
            .await
            .expect("Failed to initialize scanner");
        let target = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let port = 80;

        // Test FIN scan
        let fin_result = scanner.scan_port(target, port, StealthScanType::Fin).await;
        assert!(fin_result.is_ok());

        // Test NULL scan
        let null_result = scanner.scan_port(target, port, StealthScanType::Null).await;
        assert!(null_result.is_ok());

        // Test Xmas scan
        let xmas_result = scanner.scan_port(target, port, StealthScanType::Xmas).await;
        assert!(xmas_result.is_ok());

        // Test ACK scan
        let ack_result = scanner.scan_port(target, port, StealthScanType::Ack).await;
        assert!(ack_result.is_ok());
    }

    /// Test stealth scan against likely closed ports
    #[tokio::test]
    #[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
    #[cfg(unix)]
    async fn test_closed_port_detection() {
        let config = Config::default();
        let mut scanner = StealthScanner::new(config).expect("Failed to create scanner");
        scanner
            .initialize()
            .await
            .expect("Failed to initialize scanner");
        let target = IpAddr::V4(Ipv4Addr::LOCALHOST);

        // Use high ports unlikely to be listening
        let ports = vec![54321, 54322, 54323];

        for port in ports {
            let result = scanner.scan_port(target, port, StealthScanType::Null).await;
            assert!(result.is_ok());
            // Closed ports should respond with RST
        }
    }

    /// Test stealth scanning with different scan types on same target
    #[tokio::test]
    #[ignore] // Requires CAP_NET_RAW (root): sudo -E cargo test -- --ignored
    #[cfg(unix)]
    async fn test_mixed_scan_types_same_target() {
        let config = Config::default();
        let mut scanner = StealthScanner::new(config).expect("Failed to create scanner");
        scanner
            .initialize()
            .await
            .expect("Failed to initialize scanner");
        let target = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let ports = vec![80, 443];

        for port in &ports {
            // Try each scan type on each port
            let _ = scanner.scan_port(target, *port, StealthScanType::Fin).await;
            let _ = scanner
                .scan_port(target, *port, StealthScanType::Null)
                .await;
            let _ = scanner
                .scan_port(target, *port, StealthScanType::Xmas)
                .await;
        }
    }
}
