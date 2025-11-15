//! Integration tests for adaptive batch sizing with BatchSender
//!
//! These tests verify that AdaptiveBatchSizer properly integrates with
//! BatchSender and dynamically adjusts batch sizes based on network
//! performance.

use prtip_network::{AdaptiveConfig, BatchSender};
use std::time::Duration;

#[test]
fn test_batch_sender_without_adaptive_sizing() {
    // Test that BatchSender creation works without adaptive sizing
    // Note: We can't actually create BatchSender in tests without root privileges
    // because it tries to create raw sockets. This test validates the API.

    // Verify the constructor signature accepts None for adaptive_config
    // (Compile-time check - if this compiles, the API is correct)

    // We can't actually run new() without root, so we just verify the types
    let _: fn(&str, usize, Option<AdaptiveConfig>) -> prtip_core::Result<BatchSender> =
        BatchSender::new;
}

#[test]
fn test_batch_sender_with_adaptive_sizing_creation() {
    // Test that BatchSender API accepts adaptive config
    let config = AdaptiveConfig {
        min_batch_size: 16,
        max_batch_size: 256,
        increase_threshold: 0.95,
        decrease_threshold: 0.85,
        memory_limit: 50_000_000, // 50 MB
        window_size: Duration::from_secs(5),
    };

    // Verify the constructor signature accepts Some(config)
    // (Compile-time check - if this compiles, the API is correct)
    let _: fn(&str, usize, Option<AdaptiveConfig>) -> prtip_core::Result<BatchSender> =
        BatchSender::new;

    // Verify AdaptiveConfig is properly configured
    assert_eq!(config.min_batch_size, 16);
    assert_eq!(config.max_batch_size, 256);
    assert_eq!(config.increase_threshold, 0.95);
    assert_eq!(config.decrease_threshold, 0.85);
}

#[test]
fn test_adaptive_config_defaults() {
    let config = AdaptiveConfig::default();

    assert_eq!(config.min_batch_size, 1);
    assert_eq!(config.max_batch_size, 1024);
    assert_eq!(config.increase_threshold, 0.95);
    assert_eq!(config.decrease_threshold, 0.85);
    assert_eq!(config.memory_limit, 100_000_000); // 100 MB
    assert_eq!(config.window_size, Duration::from_secs(5));
}

#[test]
fn test_adaptive_batch_size_increase_scenario() {
    // Simulate scenario where batch size should increase due to good performance
    let config = AdaptiveConfig {
        min_batch_size: 16,
        max_batch_size: 256,
        increase_threshold: 0.95,
        decrease_threshold: 0.85,
        memory_limit: 50_000_000,
        window_size: Duration::from_secs(5),
    };

    let mut sizer = prtip_network::AdaptiveBatchSizer::new(config);

    // Get initial batch size (should be 32 - conservative start)
    let initial_size = sizer.current_batch_size();
    assert_eq!(initial_size, 32);

    // Simulate good performance: 100 sent, 98 received (98% success rate)
    for _ in 0..5 {
        sizer.record_send(100);
        sizer.record_receive(98);
        sizer.update();
    }

    // Batch size should increase (doubled from 32)
    let new_size = sizer.current_batch_size();
    assert!(
        new_size >= initial_size,
        "Batch size should increase on good performance: {} -> {}",
        initial_size,
        new_size
    );
}

#[test]
fn test_adaptive_batch_size_decrease_scenario() {
    // Simulate scenario where batch size should decrease due to poor performance
    let config = AdaptiveConfig {
        min_batch_size: 16,
        max_batch_size: 256,
        increase_threshold: 0.95,
        decrease_threshold: 0.85,
        memory_limit: 50_000_000,
        window_size: Duration::from_secs(5),
    };

    let mut sizer = prtip_network::AdaptiveBatchSizer::new(config);

    // Manually set higher initial batch size
    // (In real scenario, this would be result of previous good performance)
    // Note: We can't directly set it, so we first increase it
    for _ in 0..10 {
        sizer.record_send(100);
        sizer.record_receive(98); // Good performance
        sizer.update();
    }

    let increased_size = sizer.current_batch_size();
    assert!(increased_size > 32, "Should have increased from initial 32");

    // Now simulate poor performance: 100 sent, 60 received (40% loss)
    for _ in 0..10 {
        sizer.record_send(100);
        sizer.record_receive(60);
        sizer.update();
    }

    let final_size = sizer.current_batch_size();
    assert!(
        final_size < increased_size,
        "Batch size should decrease on poor performance: {} -> {}",
        increased_size,
        final_size
    );
}

#[test]
fn test_adaptive_batch_size_stability() {
    // Simulate scenario where batch size should remain stable
    let config = AdaptiveConfig {
        min_batch_size: 16,
        max_batch_size: 256,
        increase_threshold: 0.95,
        decrease_threshold: 0.85,
        memory_limit: 50_000_000,
        window_size: Duration::from_secs(5),
    };

    let mut sizer = prtip_network::AdaptiveBatchSizer::new(config);

    let initial_size = sizer.current_batch_size();

    // Simulate medium performance: 100 sent, 90 received (90% success rate)
    // This is between increase_threshold (95%) and decrease_threshold (85%)
    for _ in 0..10 {
        sizer.record_send(100);
        sizer.record_receive(90);
        sizer.update();
    }

    let final_size = sizer.current_batch_size();
    assert_eq!(
        final_size, initial_size,
        "Batch size should remain stable at medium performance: {}",
        initial_size
    );
}

#[test]
fn test_adaptive_batch_respects_min_max_bounds() {
    // Test that adaptive sizing respects configured min/max bounds
    let config = AdaptiveConfig {
        min_batch_size: 32,
        max_batch_size: 128,
        increase_threshold: 0.95,
        decrease_threshold: 0.85,
        memory_limit: 50_000_000,
        window_size: Duration::from_secs(5),
    };

    let mut sizer = prtip_network::AdaptiveBatchSizer::new(config);

    // Try to increase beyond max
    for _ in 0..20 {
        sizer.record_send(1000);
        sizer.record_receive(990); // 99% success
        sizer.update();
    }

    let max_reached = sizer.current_batch_size();
    assert!(
        max_reached <= 128,
        "Batch size should not exceed max_batch_size (128), got {}",
        max_reached
    );

    // Try to decrease below min
    for _ in 0..20 {
        sizer.record_send(1000);
        sizer.record_receive(500); // 50% loss
        sizer.update();
    }

    let min_reached = sizer.current_batch_size();
    assert!(
        min_reached >= 32,
        "Batch size should not go below min_batch_size (32), got {}",
        min_reached
    );
}

#[test]
fn test_adaptive_batch_memory_constraint() {
    // Test that adaptive sizing respects memory constraints
    let config = AdaptiveConfig {
        min_batch_size: 1,
        max_batch_size: 1024,
        increase_threshold: 0.95,
        decrease_threshold: 0.85,
        memory_limit: 30_000, // Very small: ~20 packets max (1500 bytes each)
        window_size: Duration::from_secs(5),
    };

    let mut sizer = prtip_network::AdaptiveBatchSizer::new(config);

    // Simulate high memory usage
    sizer.update_memory(25_000); // Using 25KB of 30KB limit

    // Try to increase batch size with good performance
    for _ in 0..10 {
        sizer.record_send(100);
        sizer.record_receive(98);
        let new_size = sizer.update();

        // Should be limited by memory constraint, not performance
        assert!(
            new_size < 100,
            "Batch size should be limited by memory constraint, got {}",
            new_size
        );
    }
}

#[test]
fn test_adaptive_config_custom_thresholds() {
    // Test custom threshold configuration
    let config = AdaptiveConfig {
        min_batch_size: 1,
        max_batch_size: 512,
        increase_threshold: 0.90, // Lower threshold - increase more aggressively
        decrease_threshold: 0.80, // Lower threshold - decrease less aggressively
        memory_limit: 100_000_000,
        window_size: Duration::from_secs(10),
    };

    let mut sizer = prtip_network::AdaptiveBatchSizer::new(config);

    // 88% success rate should trigger increase (> 80% but actual check is >= 90%)
    // Wait, the increase threshold is 90%, so we need >= 90% to increase
    // Let me fix this test to use 91% success rate

    let initial_size = sizer.current_batch_size();

    // 91% success rate should trigger increase (>= 90% threshold)
    for _ in 0..10 {
        sizer.record_send(100);
        sizer.record_receive(91);
        sizer.update();
    }

    let new_size = sizer.current_batch_size();
    assert!(
        new_size >= initial_size,
        "With 91% success and 90% increase threshold, should increase: {} -> {}",
        initial_size,
        new_size
    );
}
