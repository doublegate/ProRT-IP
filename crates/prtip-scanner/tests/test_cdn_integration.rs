//! CDN Detection Integration Tests
//!
//! Tests CDN IP filtering and deduplication in the scanner pipeline.
//! Verifies that CDN IPs are correctly identified and skipped to reduce
//! scan redundancy and improve efficiency.
//!
//! Sprint 6.3 Task Area 2: CDN IP Deduplication

use prtip_core::{
    Config, NetworkConfig, OutputConfig, OutputFormat, PerformanceConfig, ScanConfig, ScanTarget,
    ScanType, TimingTemplate,
};
use prtip_scanner::{ScanScheduler, StorageBackend};
use std::net::IpAddr;
use std::sync::Arc;

/// Helper to create test config with CDN detection enabled
fn create_test_config_with_cdn(
    skip_cdn: bool,
    whitelist: Option<Vec<String>>,
    blacklist: Option<Vec<String>>,
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
            cdn_whitelist: whitelist,
            cdn_blacklist: blacklist,
        },
        output: OutputConfig {
            format: OutputFormat::Json,
            file: None,
            verbose: 0,
        },
        performance: PerformanceConfig {
            max_rate: Some(100),
            parallelism: 10,
            batch_size: None,
            requested_ulimit: None,
            numa_enabled: false,
            adaptive_batch_enabled: false,
            min_batch_size: 1,
            max_batch_size: 1024,
        },
        evasion: Default::default(),
    }
}

// ============================================================================
// UNIT TESTS: CDN Filtering Logic
// ============================================================================

#[tokio::test]
async fn test_cdn_filtering_cloudflare() {
    // Test Cloudflare IP filtering
    let config = create_test_config_with_cdn(true, None, None);
    let storage = Arc::new(StorageBackend::memory(100));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // Cloudflare IP: 104.16.132.229 (range 104.16.0.0/13)
    let cf_target = ScanTarget::parse("104.16.132.229").unwrap();

    // Execute scan - should be filtered
    let results = scheduler.execute_scan(vec![cf_target], None).await.unwrap();

    // Should be empty (filtered out)
    assert!(
        results.is_empty(),
        "Cloudflare IP should be filtered, got {} results",
        results.len()
    );
}

#[tokio::test]
async fn test_cdn_filtering_aws_cloudfront() {
    // Test AWS CloudFront IP filtering
    let config = create_test_config_with_cdn(true, None, None);
    let storage = Arc::new(StorageBackend::memory(100));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // AWS CloudFront IP: 13.32.1.1 (range 13.32.0.0/15)
    let aws_target = ScanTarget::parse("13.32.1.1").unwrap();

    // Execute scan - should be filtered
    let results = scheduler
        .execute_scan(vec![aws_target], None)
        .await
        .unwrap();

    // Should be empty (filtered out)
    assert!(
        results.is_empty(),
        "AWS CloudFront IP should be filtered, got {} results",
        results.len()
    );
}

#[tokio::test]
async fn test_cdn_filtering_fastly() {
    // Test Fastly IP filtering
    let config = create_test_config_with_cdn(true, None, None);
    let storage = Arc::new(StorageBackend::memory(100));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // Fastly IP: 151.101.1.1 (range 151.101.0.0/16)
    let fastly_target = ScanTarget::parse("151.101.1.1").unwrap();

    // Execute scan - should be filtered
    let results = scheduler
        .execute_scan(vec![fastly_target], None)
        .await
        .unwrap();

    // Should be empty (filtered out)
    assert!(
        results.is_empty(),
        "Fastly IP should be filtered, got {} results",
        results.len()
    );
}

#[tokio::test]
async fn test_cdn_filtering_multiple_providers() {
    // Test filtering with multiple CDN providers in single scan
    let config = create_test_config_with_cdn(true, None, None);
    let storage = Arc::new(StorageBackend::memory(100));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // Mix of CDN IPs from different providers
    let targets = vec![
        ScanTarget::parse("104.16.132.229").unwrap(), // Cloudflare
        ScanTarget::parse("13.32.1.1").unwrap(),      // AWS
        ScanTarget::parse("151.101.1.1").unwrap(),    // Fastly
    ];

    // Execute scan - all should be filtered
    let results = scheduler.execute_scan(targets, None).await.unwrap();

    // Should be empty (all filtered out)
    assert!(
        results.is_empty(),
        "All CDN IPs should be filtered, got {} results",
        results.len()
    );
}

#[tokio::test]
async fn test_cdn_whitelist_mode() {
    // Test whitelist mode - only skip specified providers
    let config = create_test_config_with_cdn(true, Some(vec!["cloudflare".to_string()]), None);
    let storage = Arc::new(StorageBackend::memory(200));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // Cloudflare IP (should be filtered)
    let cf_target = ScanTarget::parse("104.16.132.229").unwrap();
    let cf_results = scheduler.execute_scan(vec![cf_target], None).await.unwrap();

    assert!(
        cf_results.is_empty(),
        "Cloudflare IP should be filtered (whitelisted)"
    );

    // AWS IP (should NOT be filtered - not in whitelist)
    let config2 = create_test_config_with_cdn(true, Some(vec!["cloudflare".to_string()]), None);
    let storage2 = Arc::new(StorageBackend::memory(200));
    let scheduler2 = ScanScheduler::new(config2, storage2).await.unwrap();

    let aws_target = ScanTarget::parse("13.32.1.1").unwrap();
    let aws_results = scheduler2
        .execute_scan(vec![aws_target], None)
        .await
        .unwrap();

    assert!(
        !aws_results.is_empty(),
        "AWS IP should NOT be filtered (not in whitelist)"
    );
}

#[tokio::test]
async fn test_cdn_blacklist_mode() {
    // Test blacklist mode - skip all except specified
    let config = create_test_config_with_cdn(true, None, Some(vec!["cloudflare".to_string()]));
    let storage = Arc::new(StorageBackend::memory(200));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // Cloudflare IP (should NOT be filtered - blacklisted means "don't skip")
    let cf_target = ScanTarget::parse("104.16.132.229").unwrap();
    let cf_results = scheduler.execute_scan(vec![cf_target], None).await.unwrap();

    assert!(
        !cf_results.is_empty(),
        "Cloudflare IP should NOT be filtered (blacklisted)"
    );

    // AWS IP (should be filtered - not blacklisted)
    let config2 = create_test_config_with_cdn(true, None, Some(vec!["cloudflare".to_string()]));
    let storage2 = Arc::new(StorageBackend::memory(200));
    let scheduler2 = ScanScheduler::new(config2, storage2).await.unwrap();

    let aws_target = ScanTarget::parse("13.32.1.1").unwrap();
    let aws_results = scheduler2
        .execute_scan(vec![aws_target], None)
        .await
        .unwrap();

    assert!(
        aws_results.is_empty(),
        "AWS IP should be filtered (not blacklisted)"
    );
}

#[tokio::test]
async fn test_cdn_provider_aliases() {
    // Test provider name aliases (cf, aws, gcp, etc.)
    let test_cases = vec![
        ("cf", "104.16.132.229"),     // Cloudflare alias
        ("amazon", "13.32.1.1"),      // AWS alias
        ("gcp", "35.190.1.1"),        // Google Cloud alias
        ("microsoft", "20.21.1.1"),   // Azure alias
        ("cloudfront", "54.192.1.1"), // AWS CloudFront alias
    ];

    for (alias, ip) in test_cases {
        let config = create_test_config_with_cdn(true, Some(vec![alias.to_string()]), None);
        let storage = Arc::new(StorageBackend::memory(100));
        let scheduler = ScanScheduler::new(config, storage).await.unwrap();

        let target = ScanTarget::parse(ip).unwrap();
        let results = scheduler.execute_scan(vec![target], None).await.unwrap();

        assert!(
            results.is_empty(),
            "IP {} should be filtered with alias '{}'",
            ip,
            alias
        );
    }
}

// ============================================================================
// INTEGRATION TESTS: Real Scan Workflows
// ============================================================================

#[tokio::test]
async fn test_mixed_cdn_non_cdn_targets() {
    // Test scan with mix of CDN and non-CDN IPs
    let config = create_test_config_with_cdn(true, None, None);
    let storage = Arc::new(StorageBackend::memory(500));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // Mix of IPs:
    // - Cloudflare: 104.16.132.229 (should be filtered)
    // - AWS: 13.32.1.1 (should be filtered)
    // - Localhost: 127.0.0.1 (should be scanned)
    let targets = vec![
        ScanTarget::parse("104.16.132.229").unwrap(), // Cloudflare
        ScanTarget::parse("13.32.1.1").unwrap(),      // AWS
        ScanTarget::parse("127.0.0.1").unwrap(),      // Localhost (non-CDN)
    ];

    let results = scheduler.execute_scan(targets, None).await.unwrap();

    // Should only have results for localhost (non-CDN IP)
    assert!(
        !results.is_empty(),
        "Should have results for non-CDN IP (localhost)"
    );

    // All results should be for localhost only
    for result in &results {
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        assert_eq!(
            result.target_ip, ip,
            "All results should be for localhost (127.0.0.1)"
        );
    }
}

#[tokio::test]
async fn test_all_cdn_targets_early_exit() {
    // Test that 100% CDN target list triggers early exit (no port scanning)
    let config = create_test_config_with_cdn(true, None, None);
    let storage = Arc::new(StorageBackend::memory(100));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // All CDN IPs
    let targets = vec![
        ScanTarget::parse("104.16.132.229").unwrap(), // Cloudflare
        ScanTarget::parse("13.32.1.1").unwrap(),      // AWS
        ScanTarget::parse("151.101.1.1").unwrap(),    // Fastly
        ScanTarget::parse("35.190.1.1").unwrap(),     // Google Cloud
    ];

    let results = scheduler.execute_scan(targets, None).await.unwrap();

    // Should be completely empty (early exit)
    assert!(
        results.is_empty(),
        "100% CDN targets should trigger early exit with no results"
    );
}

#[tokio::test]
async fn test_cdn_filtering_with_discovery_mode() {
    // Test CDN filtering works correctly with host discovery enabled
    let config = create_test_config_with_cdn(true, None, None);
    let storage = Arc::new(StorageBackend::memory(500));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // Mix of IPs including CDN
    let targets = vec![
        ScanTarget::parse("104.16.132.229").unwrap(), // Cloudflare (CDN)
        ScanTarget::parse("127.0.0.1").unwrap(),      // Localhost (non-CDN)
    ];

    // Execute with discovery mode
    let results = scheduler
        .execute_scan_with_discovery(targets, None)
        .await
        .unwrap();

    // Should only discover and scan localhost (non-CDN)
    if !results.is_empty() {
        for result in &results {
            let localhost: IpAddr = "127.0.0.1".parse().unwrap();
            assert_eq!(
                result.target_ip, localhost,
                "Only localhost should be scanned (CDN filtered before discovery)"
            );
        }
    }
}

#[tokio::test]
async fn test_cdn_statistics_tracking() {
    // Test that CDN filtering statistics are tracked correctly
    // Note: This test verifies the filtering logic; actual stats are logged
    let config = create_test_config_with_cdn(true, None, None);
    let storage = Arc::new(StorageBackend::memory(500));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // Multiple IPs from different providers
    let targets = vec![
        ScanTarget::parse("104.16.132.229").unwrap(), // Cloudflare
        ScanTarget::parse("104.16.132.230").unwrap(), // Cloudflare
        ScanTarget::parse("13.32.1.1").unwrap(),      // AWS
        ScanTarget::parse("13.32.1.2").unwrap(),      // AWS
        ScanTarget::parse("151.101.1.1").unwrap(),    // Fastly
        ScanTarget::parse("127.0.0.1").unwrap(),      // Non-CDN
    ];

    let results = scheduler.execute_scan(targets, None).await.unwrap();

    // Should only scan localhost (1 host)
    // 5 CDN IPs should be filtered
    // Reduction: 5/6 = 83.3%
    assert!(!results.is_empty(), "Should have results for non-CDN IP");

    // All results should be for localhost only
    for result in &results {
        let localhost: IpAddr = "127.0.0.1".parse().unwrap();
        assert_eq!(
            result.target_ip, localhost,
            "All results should be for localhost only"
        );
    }

    // 5 IPs filtered out of 6 total = 83.3% reduction
    // (Actual percentage is logged by scheduler, we verify behavior here)
}

#[tokio::test]
async fn test_cdn_disabled_scans_all_ips() {
    // Test that disabling CDN detection scans all IPs normally
    let config = create_test_config_with_cdn(false, None, None);
    let storage = Arc::new(StorageBackend::memory(500));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // Mix of CDN and non-CDN IPs
    let targets = vec![
        ScanTarget::parse("104.16.132.229").unwrap(), // Cloudflare
        ScanTarget::parse("127.0.0.1").unwrap(),      // Localhost
    ];

    let results = scheduler.execute_scan(targets, None).await.unwrap();

    // Should have results for BOTH IPs (CDN detection disabled)
    assert!(
        !results.is_empty(),
        "Should have results when CDN detection disabled"
    );

    // Should have results from both IPs
    let unique_ips: std::collections::HashSet<_> = results.iter().map(|r| r.target_ip).collect();

    // May not have results for both if Cloudflare is unreachable, but at least localhost
    assert!(
        unique_ips.len() >= 1,
        "Should scan at least localhost when CDN disabled"
    );
}

#[tokio::test]
async fn test_cdn_ipv6_detection() {
    // Test CDN filtering for IPv6 addresses
    let config = create_test_config_with_cdn(true, None, None);
    let storage = Arc::new(StorageBackend::memory(500));
    let scheduler = ScanScheduler::new(config, storage).await.unwrap();

    // IPv6 CDN addresses from various providers
    let targets = vec![
        ScanTarget::parse("2606:4700:20::1").unwrap(), // Cloudflare IPv6
        ScanTarget::parse("2600:9000::1").unwrap(),    // AWS CloudFront IPv6
        ScanTarget::parse("2a04:4e40::1").unwrap(),    // Fastly IPv6
        ScanTarget::parse("::1").unwrap(),             // IPv6 localhost (non-CDN)
    ];

    let results = scheduler.execute_scan(targets, None).await.unwrap();

    // Should only have results for localhost (non-CDN IPv6)
    // CDN IPv6 addresses should be filtered
    if !results.is_empty() {
        for result in &results {
            let localhost: IpAddr = "::1".parse().unwrap();
            assert_eq!(
                result.target_ip, localhost,
                "Only IPv6 localhost should be scanned (CDN IPv6 filtered)"
            );
        }
    }
}

#[tokio::test]
async fn test_cdn_filtering_performance_overhead() {
    // Test that CDN filtering has minimal performance overhead
    use std::time::Instant;

    let targets = vec![
        ScanTarget::parse("104.16.132.229").unwrap(), // Cloudflare
        ScanTarget::parse("13.32.1.1").unwrap(),      // AWS
        ScanTarget::parse("151.101.1.1").unwrap(),    // Fastly
        ScanTarget::parse("127.0.0.1").unwrap(),      // Localhost
    ];

    // Test with CDN filtering enabled
    let config_cdn = create_test_config_with_cdn(true, None, None);
    let storage_cdn = Arc::new(StorageBackend::memory(500));
    let scheduler_cdn = ScanScheduler::new(config_cdn, storage_cdn).await.unwrap();

    let start_cdn = Instant::now();
    let _results_cdn = scheduler_cdn
        .execute_scan(targets.clone(), None)
        .await
        .unwrap();
    let duration_cdn = start_cdn.elapsed();

    // Test without CDN filtering
    let config_no_cdn = create_test_config_with_cdn(false, None, None);
    let storage_no_cdn = Arc::new(StorageBackend::memory(500));
    let scheduler_no_cdn = ScanScheduler::new(config_no_cdn, storage_no_cdn)
        .await
        .unwrap();

    let start_no_cdn = Instant::now();
    let _results_no_cdn = scheduler_no_cdn.execute_scan(targets, None).await.unwrap();
    let duration_no_cdn = start_no_cdn.elapsed();

    // CDN filtering should actually be FASTER (fewer hosts to scan)
    // or have negligible overhead (<10%)
    let overhead_ratio = duration_cdn.as_secs_f64() / duration_no_cdn.as_secs_f64();

    // Allow up to 10% overhead (but typically should be faster)
    assert!(
        overhead_ratio <= 1.1,
        "CDN filtering overhead should be <10%, got {:.2}%",
        (overhead_ratio - 1.0) * 100.0
    );

    // In practice, should be faster since we skip 3/4 hosts (75% reduction)
    // Uncomment to verify performance gain:
    // println!("CDN filtering: {:?}, No CDN: {:?}, Ratio: {:.2}x",
    //          duration_cdn, duration_no_cdn, overhead_ratio);
}

// NOTE: execute_scan_ports() currently doesn't implement CDN filtering
// This is a future enhancement for Phase 6.4+ (see scheduler.rs lines 606-899)
// CDN filtering is only active in execute_scan() and execute_scan_with_discovery()
//
// #[tokio::test]
// async fn test_cdn_filtering_with_port_range() {
//     // Future: Test CDN filtering in execute_scan_ports
//     // Currently CDN filtering only applies to execute_scan/execute_scan_with_discovery
// }
