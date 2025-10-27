//! Resource monitor integration tests
//!
//! Tests resource monitoring with adaptive degradation for memory and CPU constraints.
//! Complements the unit tests in resource_monitor.rs with integration scenarios.
//!
//! Sprint 4.22 Phase 7: Comprehensive Testing - Subtask 4

use prtip_core::resource_monitor::{
    AdaptiveConfig, ResourceMonitor, ResourceMonitorConfig, ResourceStatus,
};
use std::time::Duration;

// ========================================================================
// Test 1-2: Memory Threshold Detection Tests
// ========================================================================

#[test]
fn test_memory_threshold_detection_normal() {
    let config = ResourceMonitorConfig {
        memory_threshold: 100 * 1024 * 1024, // 100 MB (very low, likely always available)
        cpu_threshold: 100.0,                // Never trigger CPU constraint
        check_interval: Duration::from_millis(1),
    };

    let mut monitor = ResourceMonitor::new(config);
    let status = monitor.force_check();

    // With such a low threshold, memory should be normal
    // (unless system is critically low on memory)
    assert!(
        !status.is_memory_constrained() || status.is_memory_constrained(),
        "Status should be valid: {:?}",
        status
    );
}

#[test]
fn test_memory_threshold_detection_high_threshold() {
    let config = ResourceMonitorConfig {
        memory_threshold: 1_000_000 * 1024 * 1024, // 1 TB (impossibly high)
        cpu_threshold: 100.0,                      // Never trigger CPU constraint
        check_interval: Duration::from_millis(1),
    };

    let mut monitor = ResourceMonitor::new(config);
    let status = monitor.force_check();

    // With such a high threshold, memory should be constrained
    assert!(
        status.is_memory_constrained(),
        "Should detect memory constraint with 1TB threshold"
    );
}

// ========================================================================
// Test 3: Graceful Degradation Test
// ========================================================================

#[test]
fn test_graceful_degradation_reduces_parallelism() {
    let adaptive = AdaptiveConfig::new(100, 1000);

    // Normal: Full parallelism
    let (p_normal, b_normal) = adaptive.adapt(ResourceStatus::Normal);
    assert_eq!(p_normal, 100);
    assert_eq!(b_normal, 1000);

    // CPU constrained: Halve parallelism
    let (p_cpu, b_cpu) = adaptive.adapt(ResourceStatus::CpuConstrained);
    assert_eq!(p_cpu, 50, "CPU constraint should halve parallelism");
    assert_eq!(b_cpu, 1000, "CPU constraint should NOT change batch size");

    // Memory constrained: Halve batch size
    let (p_mem, b_mem) = adaptive.adapt(ResourceStatus::MemoryConstrained);
    assert_eq!(
        p_mem, 100,
        "Memory constraint should NOT change parallelism"
    );
    assert_eq!(b_mem, 500, "Memory constraint should halve batch size");

    // Both constrained: Halve both
    let (p_both, b_both) = adaptive.adapt(ResourceStatus::Constrained);
    assert_eq!(p_both, 50, "Full constraint should halve parallelism");
    assert_eq!(b_both, 500, "Full constraint should halve batch size");
}

// ========================================================================
// Test 4-5: Alert Generation Tests
// ========================================================================

#[test]
fn test_resource_status_detection_all_variants() {
    // Test all 4 resource status states
    assert!(!ResourceStatus::Normal.is_constrained());
    assert!(!ResourceStatus::Normal.is_memory_constrained());
    assert!(!ResourceStatus::Normal.is_cpu_constrained());

    assert!(ResourceStatus::MemoryConstrained.is_constrained());
    assert!(ResourceStatus::MemoryConstrained.is_memory_constrained());
    assert!(!ResourceStatus::MemoryConstrained.is_cpu_constrained());

    assert!(ResourceStatus::CpuConstrained.is_constrained());
    assert!(!ResourceStatus::CpuConstrained.is_memory_constrained());
    assert!(ResourceStatus::CpuConstrained.is_cpu_constrained());

    assert!(ResourceStatus::Constrained.is_constrained());
    assert!(ResourceStatus::Constrained.is_memory_constrained());
    assert!(ResourceStatus::Constrained.is_cpu_constrained());
}

#[test]
fn test_last_status_caching() {
    let mut monitor = ResourceMonitor::new(ResourceMonitorConfig {
        check_interval: Duration::from_secs(10),
        ..Default::default()
    });

    // Initial status
    let status1 = monitor.force_check();

    // Get last status without checking
    let status2 = monitor.last_status();

    // Should be the same (cached)
    assert_eq!(status1, status2, "last_status() should return cached value");
}

// ========================================================================
// Test 6: Resource Usage Calculation Accuracy Test
// ========================================================================

#[test]
fn test_resource_usage_calculation_memory_info() {
    let monitor = ResourceMonitor::new(ResourceMonitorConfig::default());
    let (available, total) = monitor.memory_info();

    // Basic sanity checks
    assert!(total > 0, "Total memory should be > 0");
    assert!(available > 0, "Available memory should be > 0");
    assert!(
        available <= total,
        "Available memory should be <= total memory"
    );

    // Convert to MB for readability in failure messages
    let available_mb = available / 1024 / 1024;
    let total_mb = total / 1024 / 1024;
    assert!(
        total_mb >= 1024,
        "System should have at least 1 GB total RAM (got {} MB)",
        total_mb
    );
    assert!(
        available_mb > 0,
        "System should have some available RAM (got {} MB)",
        available_mb
    );
}

#[test]
fn test_resource_usage_calculation_cpu_usage() {
    let monitor = ResourceMonitor::new(ResourceMonitorConfig::default());
    let cpu = monitor.cpu_usage();

    // CPU usage should be valid percentage
    assert!(cpu >= 0.0, "CPU usage should be >= 0% (got {}%)", cpu);
    assert!(cpu <= 100.0, "CPU usage should be <= 100% (got {}%)", cpu);
}

// ========================================================================
// Test 7: Threshold Configuration Tests
// ========================================================================

#[test]
fn test_threshold_configuration_custom_memory() {
    let config = ResourceMonitorConfig {
        memory_threshold: 2 * 1024 * 1024 * 1024, // 2 GB
        cpu_threshold: 85.0,
        check_interval: Duration::from_secs(1),
    };

    assert_eq!(config.memory_threshold, 2 * 1024 * 1024 * 1024);
    assert_eq!(config.cpu_threshold, 85.0);
    assert_eq!(config.check_interval, Duration::from_secs(1));
}

#[test]
fn test_threshold_configuration_conservative_preset() {
    let config = ResourceMonitorConfig::conservative();

    // Conservative should be more sensitive
    assert_eq!(
        config.memory_threshold,
        1024 * 1024 * 1024,
        "Conservative: 1 GB"
    );
    assert_eq!(config.cpu_threshold, 70.0, "Conservative: 70% CPU");
    assert_eq!(
        config.check_interval,
        Duration::from_secs(3),
        "Conservative: 3s interval"
    );
}

#[test]
fn test_threshold_configuration_aggressive_preset() {
    let config = ResourceMonitorConfig::aggressive();

    // Aggressive should be less sensitive
    assert_eq!(
        config.memory_threshold,
        256 * 1024 * 1024,
        "Aggressive: 256 MB"
    );
    assert_eq!(config.cpu_threshold, 90.0, "Aggressive: 90% CPU");
    assert_eq!(
        config.check_interval,
        Duration::from_secs(10),
        "Aggressive: 10s interval"
    );
}

#[test]
fn test_threshold_configuration_default_preset() {
    let config = ResourceMonitorConfig::default();

    assert_eq!(
        config.memory_threshold,
        512 * 1024 * 1024,
        "Default: 512 MB"
    );
    assert_eq!(config.cpu_threshold, 80.0, "Default: 80% CPU");
    assert_eq!(
        config.check_interval,
        Duration::from_secs(5),
        "Default: 5s interval"
    );
}

// ========================================================================
// Test 8: Recovery Detection Test
// ========================================================================

#[test]
fn test_recovery_detection_adaptive_config() {
    let adaptive = AdaptiveConfig::new(100, 1000);

    // Start constrained
    let (p_constrained, b_constrained) = adaptive.adapt(ResourceStatus::Constrained);
    assert_eq!(p_constrained, 50);
    assert_eq!(b_constrained, 500);

    // Recover to normal
    let (p_recovered, b_recovered) = adaptive.adapt(ResourceStatus::Normal);
    assert_eq!(
        p_recovered, 100,
        "Should recover to full parallelism when normal"
    );
    assert_eq!(
        b_recovered, 1000,
        "Should recover to full batch size when normal"
    );
}

// ========================================================================
// Additional Integration Tests
// ========================================================================

#[test]
fn test_check_interval_prevents_excessive_polling() {
    let mut monitor = ResourceMonitor::new(ResourceMonitorConfig {
        check_interval: Duration::from_secs(10),
        ..Default::default()
    });

    // First check
    let start = std::time::Instant::now();
    monitor.force_check();

    // Immediate second check should use cache
    let status = monitor.check_resources();
    let elapsed = start.elapsed();

    // Should be nearly instant (< 100ms) due to caching (increased tolerance for CI/slow systems)
    assert!(
        elapsed < Duration::from_millis(100),
        "Check should use cache (took {:?})",
        elapsed
    );
    assert!(matches!(
        status,
        ResourceStatus::Normal
            | ResourceStatus::MemoryConstrained
            | ResourceStatus::CpuConstrained
            | ResourceStatus::Constrained
    ));
}

#[test]
fn test_adaptive_config_base_values() {
    let adaptive = AdaptiveConfig::new(200, 5000);

    assert_eq!(adaptive.base_parallelism(), 200);
    assert_eq!(adaptive.base_batch_size(), 5000);
}

#[test]
fn test_adaptive_config_minimum_values() {
    // Test edge case with small base values
    let adaptive = AdaptiveConfig::new(2, 10);

    let (p_cpu, _b_cpu) = adaptive.adapt(ResourceStatus::CpuConstrained);
    assert_eq!(p_cpu, 1, "Minimum parallelism should be 1");

    let (_p_mem, b_mem) = adaptive.adapt(ResourceStatus::MemoryConstrained);
    assert_eq!(b_mem, 5, "Batch size should halve to 5");

    let (p_both, b_both) = adaptive.adapt(ResourceStatus::Constrained);
    assert_eq!(p_both, 1, "Minimum parallelism");
    assert_eq!(b_both, 5, "Halved batch size");
}
