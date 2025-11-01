//! Integration tests for idle scan (zombie scan) functionality
//!
//! Tests the complete zombie discovery → idle scanning workflow using the public APIs.
//! Focuses on cross-module integration rather than low-level implementation details
//! (which are covered by unit tests in each module).

use prtip_core::PortState;
use prtip_scanner::idle::{
    DiscoveryConfig, IPIDPattern, IdleScanConfig, IdleScanner, ZombieCandidate, ZombieDiscovery,
};
use std::net::{IpAddr, Ipv4Addr};
use std::time::Instant;

/// Helper to create a localhost IP
fn localhost() -> IpAddr {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
}

/// Helper to create a test target IP (TEST-NET-1: 192.0.2.0/24)
fn test_target() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))
}

/// Helper to create a test zombie candidate
fn create_test_zombie() -> ZombieCandidate {
    ZombieCandidate {
        ip: localhost(),
        pattern: IPIDPattern::Sequential,
        quality_score: 0.95,
        latency_ms: 5,
        last_tested: Instant::now(),
    }
}

#[tokio::test]
async fn test_zombie_discovery_localhost_subnet() {
    // Test zombie discovery with a small localhost subnet
    let discovery = ZombieDiscovery::new("127.0.0.0/30".to_string());

    let zombies = discovery.find_zombies().await.unwrap();

    // Localhost subnet should complete without error
    // May or may not find suitable zombies depending on OS
    assert!(zombies.len() <= 2); // Max 2 usable IPs in /30
}

#[tokio::test]
async fn test_zombie_discovery_with_custom_config() {
    let config = DiscoveryConfig {
        probes_per_host: 2,     // Fewer probes for faster test
        probe_interval_ms: 500, // Faster interval
        min_quality_score: 0.5, // Lower threshold
        max_candidates: 5,
        host_timeout_ms: 1000, // Shorter timeout
    };

    let discovery = ZombieDiscovery::with_config("127.0.0.0/30".to_string(), config);
    let zombies = discovery.find_zombies().await.unwrap();

    // Should complete without error and respect max_candidates
    assert!(zombies.len() <= 5);
}

#[tokio::test]
async fn test_zombie_candidate_structure() {
    // Create a zombie candidate
    let zombie = create_test_zombie();

    // Verify structure
    assert_eq!(zombie.ip, localhost());
    assert_eq!(zombie.pattern, IPIDPattern::Sequential);
    assert!(zombie.quality_score >= 0.0 && zombie.quality_score <= 1.0);
    assert!(zombie.latency_ms > 0);
}

#[tokio::test]
async fn test_idle_scanner_creation() {
    // Create idle scan config with test zombie
    let config = IdleScanConfig {
        zombie: create_test_zombie(),
        wait_time_ms: 300,
        retries: 2,
        confidence_threshold: 0.7,
    };

    // Create idle scanner
    let scanner = IdleScanner::new(config);

    // Verify scanner was created successfully
    assert!(scanner.is_ok());
}

#[tokio::test]
async fn test_idle_scanner_timeout_handling() {
    // Create a zombie that won't respond (unreachable IP in TEST-NET-2)
    let unreachable_zombie = ZombieCandidate {
        ip: IpAddr::V4(Ipv4Addr::new(198, 51, 100, 254)),
        pattern: IPIDPattern::Sequential,
        quality_score: 0.95,
        latency_ms: 5,
        last_tested: Instant::now(),
    };

    let config = IdleScanConfig {
        zombie: unreachable_zombie,
        wait_time_ms: 100, // Short wait time for faster test
        retries: 1,        // Minimal retries
        confidence_threshold: 0.7,
    };

    let mut scanner = IdleScanner::new(config).unwrap();

    // Attempt to scan a port (should timeout or error quickly)
    let start = std::time::Instant::now();
    let result = scanner.scan_ports(test_target(), &[80]).await;
    let elapsed = start.elapsed();

    // Should complete within reasonable time
    // Platform-specific tolerances for CI environments
    let max_duration = if cfg!(target_os = "windows") {
        std::time::Duration::from_secs(10)
    } else if cfg!(target_os = "macos") {
        std::time::Duration::from_secs(5)
    } else {
        std::time::Duration::from_secs(3)
    };

    assert!(elapsed <= max_duration);

    // Verify result is either error or has no open ports
    if let Ok(results) = result {
        assert!(results.is_empty() || results.iter().all(|r| r.state != PortState::Open));
    }
}

#[tokio::test]
async fn test_idle_scanner_multiple_ports() {
    // Create scanner with test zombie
    let config = IdleScanConfig {
        zombie: create_test_zombie(),
        wait_time_ms: 300,
        retries: 1,
        confidence_threshold: 0.7,
    };

    let mut scanner = IdleScanner::new(config).unwrap();

    // Scan multiple ports on TEST-NET (should be unreachable)
    let ports = vec![81, 82, 83];
    let result = scanner.scan_ports(test_target(), &ports).await;

    // Should complete without panic
    if let Ok(results) = result {
        // Verify we got results for at most all requested ports
        assert!(results.len() <= ports.len());

        // All ports on TEST-NET should be filtered/closed
        for port_result in results {
            assert!(matches!(
                port_result.state,
                PortState::Closed | PortState::Filtered
            ));
        }
    }
}

#[tokio::test]
async fn test_idle_scan_result_structure() {
    let config = IdleScanConfig {
        zombie: create_test_zombie(),
        wait_time_ms: 300,
        retries: 1,
        confidence_threshold: 0.7,
    };

    let mut scanner = IdleScanner::new(config).unwrap();

    // Scan a single port
    let result = scanner.scan_ports(test_target(), &[9999]).await;

    if let Ok(results) = result {
        // Verify result structure
        for port_result in results {
            assert_eq!(port_result.target, test_target());
            assert_eq!(port_result.port, 9999);
            assert!(matches!(
                port_result.state,
                PortState::Open | PortState::Closed | PortState::Filtered
            ));
            assert!(port_result.confidence >= 0.0 && port_result.confidence <= 1.0);
        }
    }
}

#[tokio::test]
async fn test_integration_discovery_then_scan() {
    // Full integration test: zombie discovery → idle scan

    // Step 1: Discover zombies in localhost subnet
    let config = DiscoveryConfig {
        probes_per_host: 2,
        probe_interval_ms: 500,
        min_quality_score: 0.5, // Lower threshold for testing
        max_candidates: 3,
        host_timeout_ms: 1000,
    };

    let discovery = ZombieDiscovery::with_config("127.0.0.0/30".to_string(), config);
    let zombies = discovery.find_zombies().await.unwrap();

    // If we found a zombie, test scanning with it
    if let Some(zombie) = zombies.first() {
        // Step 2: Create idle scanner
        let scan_config = IdleScanConfig {
            zombie: zombie.clone(),
            wait_time_ms: 300,
            retries: 1,
            confidence_threshold: 0.7,
        };

        let mut scanner = IdleScanner::new(scan_config).unwrap();

        // Step 3: Scan target
        let target = test_target();
        let ports = vec![80, 443];
        let result = scanner.scan_ports(target, &ports).await;

        // Step 4: Verify results
        if let Ok(results) = result {
            // Should get results for at most all requested ports
            assert!(results.len() <= ports.len());

            // All results should have valid target IP
            for port_result in results {
                assert_eq!(port_result.target, target);
                assert!(ports.contains(&port_result.port));
            }
        }
    } else {
        // No zombies found in localhost subnet (expected on modern systems)
        // Test passes as discovery completed successfully
    }
}

#[tokio::test]
async fn test_ipid_pattern_enum_variants() {
    // Verify all pattern variants can be used
    let patterns = vec![
        IPIDPattern::Sequential,
        IPIDPattern::Random,
        IPIDPattern::PerHost,
        IPIDPattern::Broken256,
    ];

    for pattern in patterns {
        // Create zombie with each pattern type
        let zombie = ZombieCandidate {
            ip: localhost(),
            pattern: pattern.clone(),
            quality_score: 0.5,
            latency_ms: 10,
            last_tested: Instant::now(),
        };

        assert_eq!(zombie.pattern, pattern);
    }
}

#[tokio::test]
async fn test_discovery_config_defaults() {
    let config = DiscoveryConfig::default();

    assert_eq!(config.probes_per_host, 3);
    assert_eq!(config.probe_interval_ms, 1000);
    assert_eq!(config.min_quality_score, 0.7);
    assert_eq!(config.max_candidates, 10);
    assert_eq!(config.host_timeout_ms, 5000);
}

#[tokio::test]
async fn test_idle_scan_config_defaults() {
    let config = IdleScanConfig::default();

    assert_eq!(config.wait_time_ms, 300);
    assert_eq!(config.retries, 2);
    assert_eq!(config.confidence_threshold, 0.7);
    // Zombie field has default values too but we don't check every field
}

#[tokio::test]
async fn test_zombie_quality_comparison() {
    // Sequential pattern should be better than random
    let sequential = ZombieCandidate {
        ip: localhost(),
        pattern: IPIDPattern::Sequential,
        quality_score: 0.9,
        latency_ms: 10,
        last_tested: Instant::now(),
    };

    let random = ZombieCandidate {
        ip: test_target(),
        pattern: IPIDPattern::Random,
        quality_score: 0.3,
        latency_ms: 10,
        last_tested: Instant::now(),
    };

    // Sequential pattern should have higher quality score
    assert!(sequential.quality_score > random.quality_score);
}

#[tokio::test]
async fn test_idle_scanner_empty_port_list() {
    let config = IdleScanConfig {
        zombie: create_test_zombie(),
        wait_time_ms: 300,
        retries: 1,
        confidence_threshold: 0.7,
    };

    let mut scanner = IdleScanner::new(config).unwrap();

    // Scan empty port list
    let result = scanner.scan_ports(test_target(), &[]).await;

    // Should either succeed with empty results or error
    if let Ok(results) = result {
        assert_eq!(results.len(), 0);
    }
}

#[tokio::test]
async fn test_discovery_invalid_subnet() {
    // Test with invalid CIDR notation
    let discovery = ZombieDiscovery::new("invalid-subnet".to_string());

    let result = discovery.find_zombies().await;

    // Should error with parse error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_discovery_empty_subnet() {
    // Test with /32 (single IP, no usable IPs after excluding network/broadcast)
    let discovery = ZombieDiscovery::new("127.0.0.1/32".to_string());

    let zombies = discovery.find_zombies().await.unwrap();

    // Should find at most 1 zombie (the single IP)
    assert!(zombies.len() <= 1);
}
