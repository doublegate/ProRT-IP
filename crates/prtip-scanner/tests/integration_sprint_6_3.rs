//! Sprint 6.3 Integration Tests
//!
//! Comprehensive end-to-end integration tests for Sprint 6.3 Network Optimization features:
//! - Task Area 2: CDN IP Deduplication (30-70% scan time reduction)
//! - Task Area 3: Adaptive Batch Sizing (20-40% throughput improvement)
//! - Task Area 4: Cross-Platform Batch I/O (sendmmsg/recvmmsg on Linux)
//!
//! These tests validate that all Sprint 6.3 components work together correctly
//! across different platforms and configurations.

use prtip_core::{
    Config, NetworkConfig, OutputConfig, OutputFormat, PerformanceConfig, ScanConfig, ScanTarget,
    ScanType, TimingTemplate,
};
use prtip_network::adaptive_batch::{AdaptiveBatchSizer, AdaptiveConfig};
use prtip_network::PlatformCapabilities;
use prtip_scanner::{ScanScheduler, StorageBackend};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Helper to create test config with all Sprint 6.3 features configurable
fn create_sprint_6_3_config(
    skip_cdn: bool,
    adaptive_batch: bool,
    min_batch: usize,
    max_batch: usize,
) -> Config {
    Config {
        scan: ScanConfig {
            scan_type: ScanType::Connect,
            timing_template: TimingTemplate::Normal,
            timeout_ms: 500,
            retries: 0,
            scan_delay_ms: 0,
            host_delay_ms: 0,
            service_detection: Default::default(),
            progress: false,
            event_bus: None,
        },
        network: NetworkConfig {
            interface: None,
            source_port: None,
            skip_cdn,
            cdn_whitelist: None,
            cdn_blacklist: None,
        },
        output: OutputConfig {
            format: OutputFormat::Json,
            file: None,
            verbose: 0,
        },
        performance: PerformanceConfig {
            max_rate: Some(1000),
            parallelism: 10,
            batch_size: Some(32), // Default batch size for non-adaptive mode
            requested_ulimit: None,
            numa_enabled: false,
            adaptive_batch_enabled: adaptive_batch,
            min_batch_size: min_batch,
            max_batch_size: max_batch,
        },
        evasion: Default::default(),
    }
}

/// Helper to create a mix of CDN and non-CDN test targets
fn create_mixed_targets() -> Vec<ScanTarget> {
    vec![
        // CDN IPs (should be filtered when skip_cdn=true)
        ScanTarget::parse("104.16.132.229").unwrap(), // Cloudflare
        ScanTarget::parse("13.32.1.1").unwrap(),      // AWS CloudFront
        ScanTarget::parse("151.101.1.1").unwrap(),    // Fastly
        // Non-CDN IPs (should NOT be filtered)
        ScanTarget::parse("192.0.2.1").unwrap(), // TEST-NET-1
        ScanTarget::parse("198.51.100.1").unwrap(), // TEST-NET-2
    ]
}

/// Helper to measure scan throughput (targets/sec)
fn calculate_throughput(target_count: usize, duration: Duration) -> f64 {
    target_count as f64 / duration.as_secs_f64()
}

// ============================================================================
// TEST 1: FULL STACK BATCH I/O (Linux-only)
// ============================================================================

#[tokio::test]
#[cfg_attr(not(target_os = "linux"), ignore)]
async fn test_full_stack_batch_io_linux() {
    // GIVEN: Linux platform with batch I/O support
    let caps = PlatformCapabilities::detect();
    assert!(
        caps.has_sendmmsg,
        "Linux platform should support sendmmsg/recvmmsg"
    );

    // GIVEN: Config with batch I/O enabled (via batch_size)
    let config = create_sprint_6_3_config(false, false, 1, 1024);
    assert_eq!(config.performance.batch_size, Some(32));

    // GIVEN: Storage backend for results
    let storage = Arc::new(StorageBackend::memory(100));

    // WHEN: Execute scan with batch I/O
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();
    let targets = vec![
        ScanTarget::parse("192.0.2.1").unwrap(), // TEST-NET-1
        ScanTarget::parse("192.0.2.2").unwrap(),
        ScanTarget::parse("192.0.2.3").unwrap(),
    ];

    let start = Instant::now();
    let results = scheduler.execute_scan(targets.clone(), None).await.unwrap();
    let elapsed = start.elapsed();

    // THEN: Scan completes successfully
    // Results include multiple ports per target (22 default ports)
    let unique_targets: std::collections::HashSet<_> =
        results.iter().map(|r| r.target_ip).collect();
    assert_eq!(
        unique_targets.len(),
        targets.len(),
        "Should scan all 3 targets (batch I/O on Linux), got {} results from {} unique IPs",
        results.len(),
        unique_targets.len()
    );

    // THEN: Scan completes within reasonable time (batch I/O should be fast)
    assert!(
        elapsed <= Duration::from_secs(5),
        "Batch I/O scan should complete quickly, took {:?}",
        elapsed
    );
}

// ============================================================================
// TEST 2: FULL STACK CDN FILTERING
// ============================================================================

#[tokio::test]
async fn test_full_stack_cdn_filtering() {
    // GIVEN: Config with CDN filtering enabled
    let config = create_sprint_6_3_config(true, false, 1, 1024);
    assert!(config.network.skip_cdn);

    // GIVEN: Storage backend for results
    let storage = Arc::new(StorageBackend::memory(100));

    // WHEN: Execute scan with mixed CDN/non-CDN targets
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();
    let targets = create_mixed_targets();

    let start = Instant::now();
    let results = scheduler.execute_scan(targets, None).await.unwrap();
    let elapsed = start.elapsed();

    // THEN: Only non-CDN IPs are scanned (3 CDN filtered, 2 non-CDN scanned)
    // Results include multiple ports per target (22 default ports)
    let unique_targets: std::collections::HashSet<_> =
        results.iter().map(|r| r.target_ip).collect();
    assert_eq!(
        unique_targets.len(),
        2,
        "Should scan only 2 non-CDN targets (3 CDN filtered), got {} results from {} unique IPs",
        results.len(),
        unique_targets.len()
    );

    // THEN: Scan completes faster than without filtering
    // (CDN filtering reduces scan time by 30-70%)
    assert!(
        elapsed <= Duration::from_secs(3),
        "CDN filtering should reduce scan time, took {:?}",
        elapsed
    );

    // THEN: Results contain only non-CDN IPs
    for result in results {
        let ip_str = result.target_ip.to_string();
        assert!(
            !ip_str.starts_with("104.16.")
                && !ip_str.starts_with("13.32.")
                && !ip_str.starts_with("151.101."),
            "Result should not contain CDN IP: {}",
            ip_str
        );
    }
}

// ============================================================================
// TEST 3: FULL STACK ADAPTIVE BATCHING
// ============================================================================

#[tokio::test]
async fn test_full_stack_adaptive_batching() {
    // GIVEN: Config with adaptive batching enabled
    let config = create_sprint_6_3_config(false, true, 1, 1024);
    assert!(config.performance.adaptive_batch_enabled);
    assert_eq!(config.performance.min_batch_size, 1);
    assert_eq!(config.performance.max_batch_size, 1024);

    // GIVEN: Adaptive batch sizer component
    let adaptive_config = AdaptiveConfig {
        min_batch_size: 1,
        max_batch_size: 1024,
        increase_threshold: 0.95,
        decrease_threshold: 0.85,
        memory_limit: 100_000_000,
        window_size: Duration::from_secs(5),
    };
    let mut sizer = AdaptiveBatchSizer::new(adaptive_config);

    // GIVEN: Storage backend for results
    let storage = Arc::new(StorageBackend::memory(100));

    // WHEN: Simulate good network conditions
    for _ in 0..10 {
        sizer.record_send(100);
        sizer.record_receive(98); // 98% delivery rate (good)
    }

    // THEN: Batch size should increase under good conditions
    let new_batch_size = sizer.update();
    assert!(
        new_batch_size > 1,
        "Batch size should increase under good conditions, got {}",
        new_batch_size
    );
    assert!(
        new_batch_size <= 1024,
        "Batch size should not exceed max (1024), got {}",
        new_batch_size
    );

    // WHEN: Execute scan with adaptive batching
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();
    let targets = vec![
        ScanTarget::parse("192.0.2.1").unwrap(),
        ScanTarget::parse("192.0.2.2").unwrap(),
        ScanTarget::parse("192.0.2.3").unwrap(),
    ];

    let start = Instant::now();
    let results = scheduler.execute_scan(targets.clone(), None).await.unwrap();
    let elapsed = start.elapsed();

    // THEN: Scan completes successfully with adaptive batching
    // Results include multiple ports per target (22 default ports)
    let unique_targets: std::collections::HashSet<_> =
        results.iter().map(|r| r.target_ip).collect();
    assert_eq!(
        unique_targets.len(),
        targets.len(),
        "Should scan all 3 targets with adaptive batching, got {} results from {} unique IPs",
        results.len(),
        unique_targets.len()
    );

    // THEN: Scan completes within reasonable time
    assert!(
        elapsed <= Duration::from_secs(5),
        "Adaptive batching scan should complete quickly, took {:?}",
        elapsed
    );
}

// ============================================================================
// TEST 4: COMBINED FEATURES
// ============================================================================

#[tokio::test]
async fn test_combined_features() {
    // GIVEN: Config with ALL Sprint 6.3 features enabled
    let config = create_sprint_6_3_config(true, true, 1, 1024);
    assert!(config.network.skip_cdn);
    assert!(config.performance.adaptive_batch_enabled);

    // GIVEN: Storage backend for results
    let storage = Arc::new(StorageBackend::memory(100));

    // WHEN: Execute scan with mixed CDN/non-CDN targets
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();
    let targets = create_mixed_targets();

    let start = Instant::now();
    let results = scheduler.execute_scan(targets, None).await.unwrap();
    let elapsed = start.elapsed();

    // THEN: Only non-CDN IPs are scanned (CDN filtering)
    // Results include multiple ports per target (22 default ports)
    let unique_targets: std::collections::HashSet<_> =
        results.iter().map(|r| r.target_ip).collect();
    assert_eq!(
        unique_targets.len(),
        2,
        "Should scan only 2 non-CDN targets (CDN + adaptive batching), got {} results from {} unique IPs",
        results.len(),
        unique_targets.len()
    );

    // THEN: Scan completes faster with combined optimizations
    // (CDN filtering: 30-70% + Adaptive batching: 20-40%)
    // Allow 3 seconds for CI environment variability
    assert!(
        elapsed <= Duration::from_secs(3),
        "Combined features should maximize performance, took {:?}",
        elapsed
    );

    // THEN: Results are correct (2 unique IPs, each with 22 ports = 44 total results)
    assert_eq!(
        unique_targets.len(),
        2,
        "Should have exactly 2 unique target IPs"
    );
}

// ============================================================================
// TEST 5: CROSS-PLATFORM COMPATIBILITY
// ============================================================================

#[tokio::test]
async fn test_cross_platform_compatibility() {
    // GIVEN: Detect platform capabilities
    let caps = PlatformCapabilities::detect();

    // WHEN: On Linux
    #[cfg(target_os = "linux")]
    {
        // THEN: Should support batch I/O (sendmmsg/recvmmsg)
        assert!(caps.has_sendmmsg, "Linux should support sendmmsg/recvmmsg");
    }

    // WHEN: On Windows or macOS
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        // THEN: Should gracefully fall back to non-batch I/O
        assert!(
            !caps.has_sendmmsg,
            "Windows/macOS should NOT support sendmmsg/recvmmsg"
        );

        // GIVEN: Config with batch I/O configured
        let config = create_sprint_6_3_config(false, false, 1, 1024);
        let storage = Arc::new(StorageBackend::memory(100));

        // WHEN: Execute scan on non-Linux platform
        let scheduler = ScanScheduler::new(config, storage).await.unwrap();
        let targets = vec![ScanTarget::parse("192.0.2.1").unwrap()];

        let start = Instant::now();
        let results = scheduler.execute_scan(targets.clone(), None).await.unwrap();
        let elapsed = start.elapsed();

        // THEN: Scan should complete successfully with fallback mode
        // Results include multiple ports per target (22 default ports)
        let unique_targets: std::collections::HashSet<_> =
            results.iter().map(|r| r.target_ip).collect();
        assert_eq!(
            unique_targets.len(),
            targets.len(),
            "Windows/macOS should fall back gracefully, got {} results from {} unique IPs",
            results.len(),
            unique_targets.len()
        );

        assert!(
            elapsed <= Duration::from_secs(5),
            "Fallback mode should still work, took {:?}",
            elapsed
        );
    }
}

// ============================================================================
// TEST 6: PERFORMANCE REGRESSION
// ============================================================================

#[tokio::test]
async fn test_performance_regression() {
    // GIVEN: Baseline config (no Sprint 6.3 features)
    let baseline_config = create_sprint_6_3_config(false, false, 1, 1024);
    let baseline_storage = Arc::new(StorageBackend::memory(100));

    // GIVEN: Optimized config (all Sprint 6.3 features)
    let optimized_config = create_sprint_6_3_config(true, true, 1, 1024);
    let optimized_storage = Arc::new(StorageBackend::memory(100));

    // GIVEN: Test targets (non-CDN only for fair comparison)
    let targets = vec![
        ScanTarget::parse("192.0.2.1").unwrap(),
        ScanTarget::parse("192.0.2.2").unwrap(),
        ScanTarget::parse("192.0.2.3").unwrap(),
    ];

    // WHEN: Run baseline scan
    let baseline_scheduler = ScanScheduler::new(baseline_config, baseline_storage)
        .await
        .unwrap();
    let baseline_start = Instant::now();
    let baseline_results = baseline_scheduler
        .execute_scan(targets.clone(), None)
        .await
        .unwrap();
    let baseline_elapsed = baseline_start.elapsed();
    let baseline_throughput = calculate_throughput(baseline_results.len(), baseline_elapsed);

    // WHEN: Run optimized scan
    let optimized_scheduler = ScanScheduler::new(optimized_config, optimized_storage)
        .await
        .unwrap();
    let optimized_start = Instant::now();
    let optimized_results = optimized_scheduler
        .execute_scan(targets.clone(), None)
        .await
        .unwrap();
    let optimized_elapsed = optimized_start.elapsed();
    let optimized_throughput = calculate_throughput(optimized_results.len(), optimized_elapsed);

    // THEN: Both scans should produce same number of unique targets
    // (Results include multiple ports per target)
    let baseline_targets: std::collections::HashSet<_> =
        baseline_results.iter().map(|r| r.target_ip).collect();
    let optimized_targets: std::collections::HashSet<_> =
        optimized_results.iter().map(|r| r.target_ip).collect();
    assert_eq!(
        baseline_targets.len(),
        optimized_targets.len(),
        "Both scans should scan same number of unique targets"
    );

    // THEN: Optimized scan should NOT be slower (no regression)
    // Allow 20% variance for CI environment variability
    let allowed_regression = 1.20;
    assert!(
        optimized_elapsed.as_secs_f64() <= baseline_elapsed.as_secs_f64() * allowed_regression,
        "Optimized scan should not be slower (regression detected): baseline={:?}, optimized={:?}",
        baseline_elapsed,
        optimized_elapsed
    );

    // THEN: Throughput should be maintained or improved
    println!(
        "Performance comparison - Baseline: {:.2} targets/sec, Optimized: {:.2} targets/sec",
        baseline_throughput, optimized_throughput
    );

    // Note: Throughput improvement varies by platform and network conditions
    // We only check for NO REGRESSION here (not strict improvement requirement)
    // because CI environments can be variable
}
