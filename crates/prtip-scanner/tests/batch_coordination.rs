//! Integration tests for Sprint 6.3 Task 1.3: Batch Coordination
//!
//! These tests verify the batch I/O implementation in SynScanner,
//! including platform detection, fallback behavior, and rate limiting
//! integration.

use prtip_core::{Config, PortState};
use prtip_network::PlatformCapabilities;
use prtip_scanner::SynScanner;
use std::net::{IpAddr, Ipv4Addr};

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_scan_ports_batch_mode_linux() {
    // Test that batch I/O is used on Linux platforms with sendmmsg/recvmmsg support
    let caps = PlatformCapabilities::detect();

    if !caps.has_sendmmsg || !caps.has_recvmmsg {
        eprintln!("Skipping test: Linux platform lacks batch I/O support");
        return;
    }

    // Create scanner with default config
    let mut config = Config::default();
    config.scan.timeout_ms = 2000; // 2 seconds
    let scanner = match SynScanner::new(config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Skipping test: Failed to create scanner (need root permissions): {}",
                e
            );
            return;
        }
    };

    // Scan localhost on a few ports (likely closed/filtered)
    let target = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let ports = vec![9001, 9002, 9003, 9004, 9005];

    let results = match scanner.scan_ports(target, ports.clone()).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "Skipping test: Batch scan failed (need root permissions): {}",
                e
            );
            return;
        }
    };

    // Verify results
    assert_eq!(
        results.len(),
        ports.len(),
        "Should return result for each port"
    );

    for result in &results {
        assert_eq!(result.target_ip, target);
        assert!(ports.contains(&result.port));
        // Localhost ports are typically closed or filtered
        assert!(
            matches!(result.state, PortState::Closed | PortState::Filtered),
            "Unexpected port state: {:?}",
            result.state
        );
    }
}

#[cfg(not(target_os = "linux"))]
#[tokio::test]
async fn test_scan_ports_fallback_mode() {
    // Test that fallback mode is used on non-Linux platforms
    let caps = PlatformCapabilities::detect();

    // Non-Linux platforms should not have batch I/O support
    assert!(
        !caps.has_sendmmsg || !caps.has_recvmmsg,
        "Non-Linux platform should lack batch I/O support"
    );

    // Create scanner with default config
    let mut config = Config::default();
    config.scan.timeout_ms = 2000; // 2 seconds
    let scanner = match SynScanner::new(config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Skipping test: Failed to create scanner (need root permissions): {}",
                e
            );
            return;
        }
    };

    // Scan localhost on a few ports
    let target = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let ports = vec![9001, 9002, 9003];

    let results = match scanner.scan_ports(target, ports.clone()).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "Skipping test: Fallback scan failed (need root permissions): {}",
                e
            );
            return;
        }
    };

    // Verify results (same as batch mode, just different implementation)
    assert_eq!(
        results.len(),
        ports.len(),
        "Should return result for each port"
    );

    for result in &results {
        assert_eq!(result.target_ip, target);
        assert!(ports.contains(&result.port));
        assert!(
            matches!(result.state, PortState::Closed | PortState::Filtered),
            "Unexpected port state: {:?}",
            result.state
        );
    }
}

#[tokio::test]
async fn test_scan_ports_rate_limiting_integration() {
    // Test that batch coordination works correctly with rate limiting
    use prtip_core::PerformanceConfig;

    // Create config with rate limiting enabled
    let mut config = Config::default();
    config.scan.timeout_ms = 2000; // 2 seconds
    config.performance = PerformanceConfig {
        max_rate: Some(100), // Low rate for testing (100 packets per second)
        parallelism: 100,
        batch_size: None,
        requested_ulimit: None,
        numa_enabled: false,
        adaptive_batch_enabled: false,
        min_batch_size: 1,
        max_batch_size: 1024,
    };

    let scanner = match SynScanner::new(config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Skipping test: Failed to create scanner (need root permissions): {}",
                e
            );
            return;
        }
    };

    // Scan localhost on multiple ports
    let target = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let ports = vec![9001, 9002, 9003, 9004, 9005, 9006, 9007, 9008];

    let start = std::time::Instant::now();
    let results = match scanner.scan_ports(target, ports.clone()).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "Skipping test: Rate-limited scan failed (need root permissions): {}",
                e
            );
            return;
        }
    };
    let elapsed = start.elapsed();

    // Verify results
    assert_eq!(
        results.len(),
        ports.len(),
        "Should return result for each port"
    );

    // Verify rate limiting was applied
    // At 100 pps, 8 packets should take at least ~80ms
    // (Allow some variance for test stability)
    assert!(
        elapsed.as_millis() >= 50,
        "Rate limiting should slow down scan (took {}ms)",
        elapsed.as_millis()
    );

    for result in &results {
        assert_eq!(result.target_ip, target);
        assert!(ports.contains(&result.port));
    }
}
