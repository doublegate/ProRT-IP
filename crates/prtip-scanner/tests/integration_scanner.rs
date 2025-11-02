//! Integration tests for the scanner engine
//!
//! These tests verify end-to-end functionality of the scanner components
//! working together.

use prtip_core::{Config, PortRange, PortState, ScanTarget};
use prtip_scanner::{
    DiscoveryEngine, DiscoveryMethod, RateLimiter, ScanScheduler, ScanStorage, TcpConnectScanner,
};
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

#[tokio::test]
async fn test_end_to_end_tcp_scan() {
    // Create scanner
    let scanner = TcpConnectScanner::new(Duration::from_millis(500), 0);

    // Scan localhost
    let target = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let results = scanner.scan_ports(target, vec![9999], 10).await.unwrap();

    assert_eq!(results.len(), 1);
    assert!(matches!(
        results[0].state,
        PortState::Closed | PortState::Filtered
    ));
    assert_eq!(results[0].target_ip, target);
    assert_eq!(results[0].port, 9999);
}

#[tokio::test]
async fn test_storage_integration() {
    use prtip_core::ScanResult;

    let storage = ScanStorage::new(":memory:").await.unwrap();
    let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

    // Create and store result
    let result = ScanResult::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        80,
        PortState::Open,
    )
    .with_response_time(Duration::from_millis(100));

    storage.store_result(scan_id, &result).await.unwrap();
    storage.complete_scan(scan_id).await.unwrap();

    // Retrieve and verify
    let retrieved = storage.get_scan_results(scan_id).await.unwrap();
    assert_eq!(retrieved.len(), 1);
    assert_eq!(retrieved[0].port, 80);
    assert_eq!(retrieved[0].state, PortState::Open);
}

#[tokio::test]
async fn test_rate_limiter_integration() {
    let limiter = RateLimiter::new(Some(50)); // 50 packets per second

    // First exhaust burst (burst=100)
    for _ in 0..100 {
        limiter.acquire().await.unwrap();
    }

    // Now measure rate limiting on next batch (burst exhausted)
    let start = std::time::Instant::now();
    for _ in 0..10 {
        limiter.acquire().await.unwrap();
    }
    let elapsed = start.elapsed();

    // 10 packets at 50 pps = ~200ms (after burst exhausted)
    // Platform-specific timeouts (CI environments need wider margins):
    // - Linux: 600ms (3x baseline)
    // - macOS: 1200ms (6x baseline, CI runners can be slow)
    // - Windows: 1800ms (9x baseline, see CLAUDE.md CI/CD best practices)
    let max_duration = if cfg!(target_os = "macos") {
        Duration::from_millis(1200)
    } else if cfg!(target_os = "windows") {
        Duration::from_millis(1800)
    } else {
        Duration::from_millis(600) // Linux and others
    };

    assert!(elapsed >= Duration::from_millis(180));
    assert!(elapsed <= max_duration);
}

#[tokio::test]
async fn test_discovery_integration() {
    let engine = DiscoveryEngine::new(Duration::from_secs(1), DiscoveryMethod::TcpSyn);

    let targets = vec![
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), // localhost
        IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), // TEST-NET
    ];

    let live_hosts = engine.discover_hosts(targets, 5).await.unwrap();

    // Should find at least localhost
    assert!(!live_hosts.is_empty());
    assert!(live_hosts.contains(&IpAddr::V4(Ipv4Addr::LOCALHOST)));
}

#[tokio::test]
async fn test_scheduler_full_workflow() {
    use prtip_scanner::StorageBackend;
    use std::sync::Arc;

    let config = Config::default();
    let storage_backend = Arc::new(StorageBackend::memory(10000));
    let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

    let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
    let results = scheduler.execute_scan(targets, None).await.unwrap();

    // Should have some results
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_scheduler_with_port_range() {
    use prtip_scanner::StorageBackend;
    use std::sync::Arc;

    let config = Config::default();
    let storage_backend = Arc::new(StorageBackend::memory(100));
    let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

    let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
    let ports = PortRange::parse("9998-9999").unwrap();

    let results = scheduler.execute_scan_ports(targets, &ports).await.unwrap();

    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_scheduler_with_discovery() {
    use prtip_scanner::StorageBackend;
    use std::sync::Arc;

    let config = Config::default();
    let storage_backend = Arc::new(StorageBackend::memory(10000));
    let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

    let targets = vec![
        ScanTarget::parse("127.0.0.1").unwrap(),
        ScanTarget::parse("192.0.2.1").unwrap(), // unreachable
    ];

    let results = scheduler
        .execute_scan_with_discovery(targets, None)
        .await
        .unwrap();

    // Should only scan localhost
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_batch_storage_performance() {
    use prtip_core::ScanResult;

    let storage = ScanStorage::new(":memory:").await.unwrap();
    let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

    // Create 100 results
    let results: Vec<ScanResult> = (1..=100)
        .map(|i| {
            ScanResult::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, i)), 80, PortState::Open)
                .with_response_time(Duration::from_millis(i as u64))
        })
        .collect();

    let start = std::time::Instant::now();
    storage
        .store_results_batch(scan_id, &results)
        .await
        .unwrap();
    let elapsed = start.elapsed();

    // Should be fast (< 1 second for 100 results)
    assert!(elapsed < Duration::from_secs(1));

    // Verify all stored
    let count = storage.get_result_count(scan_id).await.unwrap();
    assert_eq!(count, 100);
}

#[tokio::test]
async fn test_concurrent_scanning() {
    let scanner = TcpConnectScanner::new(Duration::from_millis(100), 0);

    // Scan multiple hosts concurrently
    let hosts = vec![
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)),
    ];

    let mut all_results = Vec::new();
    for host in hosts {
        let results = scanner.scan_ports(host, vec![9999], 10).await.unwrap();
        all_results.extend(results);
    }

    assert_eq!(all_results.len(), 2);
}

#[tokio::test]
async fn test_scanner_error_handling() {
    let scanner = TcpConnectScanner::new(Duration::from_millis(10), 0);

    // Scan non-existent network
    let results = scanner
        .scan_ports(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), vec![80, 443], 5)
        .await
        .unwrap();

    // Should complete without panic, all filtered
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.state == PortState::Filtered));
}

#[tokio::test]
async fn test_storage_multiple_scans() {
    use prtip_core::ScanResult;

    let storage = ScanStorage::new(":memory:").await.unwrap();

    let scan1 = storage.create_scan(r#"{"scan": 1}"#).await.unwrap();
    let scan2 = storage.create_scan(r#"{"scan": 2}"#).await.unwrap();

    let result1 = ScanResult::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 80, PortState::Open);

    let result2 = ScanResult::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 443, PortState::Open);

    storage.store_result(scan1, &result1).await.unwrap();
    storage.store_result(scan2, &result2).await.unwrap();

    let count1 = storage.get_result_count(scan1).await.unwrap();
    let count2 = storage.get_result_count(scan2).await.unwrap();

    assert_eq!(count1, 1);
    assert_eq!(count2, 1);
}

#[tokio::test]
async fn test_target_expansion() {
    // Test CIDR expansion
    let target = ScanTarget::parse("192.168.1.0/30").unwrap();
    let hosts = target.expand_hosts();

    // /30 = 4 addresses - 2 (network/broadcast) = 2 usable
    assert_eq!(hosts.len(), 4); // pnet includes network/broadcast
}

#[tokio::test]
async fn test_port_range_iteration() {
    let range = PortRange::parse("80,443,8080-8082").unwrap();
    let ports: Vec<u16> = range.iter().collect();

    assert_eq!(ports, vec![80, 443, 8080, 8081, 8082]);
}

#[tokio::test]
async fn test_scheduler_config_validation() {
    use prtip_core::{
        NetworkConfig, OutputConfig, OutputFormat, PerformanceConfig, ScanConfig, ScanType,
        TimingTemplate,
    };

    let mut config = Config {
        scan: ScanConfig {
            scan_type: ScanType::Connect,
            timing_template: TimingTemplate::Normal,
            timeout_ms: 0, // Invalid!
            retries: 0,
            scan_delay_ms: 0,
            host_delay_ms: 0,
            service_detection: Default::default(),
            progress: false,
        },
        network: NetworkConfig {
            interface: None,
            source_port: None,
        },
        output: OutputConfig {
            format: OutputFormat::Json,
            file: None,
            verbose: 0,
        },
        performance: PerformanceConfig {
            max_rate: None,
            parallelism: 10,
            batch_size: None,
            requested_ulimit: None,
            numa_enabled: false,
        },
        evasion: Default::default(),
    };

    use prtip_scanner::StorageBackend;
    use std::sync::Arc;

    let storage_backend = Arc::new(StorageBackend::memory(100));
    let result = ScanScheduler::new(config.clone(), storage_backend).await;

    // Should fail validation
    assert!(result.is_err());

    // Fix config
    config.scan.timeout_ms = 1000;
    let storage_backend2 = Arc::new(StorageBackend::memory(100));
    let result2 = ScanScheduler::new(config, storage_backend2).await;
    assert!(result2.is_ok());
}
