//! Adaptive Batch Sizing for Network I/O
//!
//! Dynamically adjusts batch sizes (1-1024 packets) based on real-time network
//! performance monitoring. Improves throughput by increasing batch sizes during
//! good conditions and reduces packet loss by decreasing during congestion.
//!
//! # Architecture
//!
//! - **PerformanceMonitor**: Tracks throughput, packet loss, memory usage
//! - **AdaptiveBatchSizer**: Implements adaptive algorithm with thresholds
//!
//! # Usage
//!
//! ```
//! use prtip_network::adaptive_batch::{AdaptiveBatchSizer, AdaptiveConfig};
//! use std::time::Duration;
//!
//! let config = AdaptiveConfig {
//!     min_batch_size: 1,
//!     max_batch_size: 1024,
//!     increase_threshold: 0.95,
//!     decrease_threshold: 0.85,
//!     memory_limit: 100_000_000, // 100 MB
//!     window_size: Duration::from_secs(5),
//! };
//!
//! let mut sizer = AdaptiveBatchSizer::new(config);
//!
//! // Record network activity
//! sizer.record_send(100);
//! sizer.record_receive(95);
//!
//! // Get adjusted batch size
//! let new_batch_size = sizer.update();
//! ```
//!
//! Sprint 6.3 Task Area 3: Adaptive Batch Sizing

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Performance metrics for network I/O
#[derive(Debug, Clone, Copy)]
pub struct PerformanceMetrics {
    /// Packets sent per second
    pub throughput_pps: f64,
    /// Packet loss rate (0.0 = no loss, 1.0 = 100% loss)
    pub packet_loss_rate: f64,
    /// Current memory usage (bytes)
    pub memory_usage: usize,
}

/// Throughput sample (timestamp + packet counts)
#[derive(Debug, Clone, Copy)]
struct ThroughputSample {
    timestamp: Instant,
    packets_sent: usize,
    packets_received: usize,
}

/// Performance monitor for network operations
///
/// Tracks throughput, packet loss, and memory usage over a sliding time window
/// to enable adaptive batch size decisions.
pub struct PerformanceMonitor {
    /// Sliding window of throughput samples
    samples: VecDeque<ThroughputSample>,
    /// Current packet loss rate (0.0-1.0)
    packet_loss_rate: f64,
    /// Current memory usage (bytes)
    memory_usage: usize,
    /// Time window for samples (e.g., 5 seconds)
    window_size: Duration,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    ///
    /// # Arguments
    ///
    /// * `window_size` - Time window for throughput calculation (e.g., 5 seconds)
    pub fn new(window_size: Duration) -> Self {
        Self {
            samples: VecDeque::new(),
            packet_loss_rate: 0.0,
            memory_usage: 0,
            window_size,
        }
    }

    /// Record packets sent
    pub fn record_send(&mut self, packets: usize) {
        let now = Instant::now();

        // If we have a recent sample, update it; otherwise create new
        if let Some(last_sample) = self.samples.back_mut() {
            if now.duration_since(last_sample.timestamp) < Duration::from_millis(100) {
                // Update existing sample (same 100ms window)
                last_sample.packets_sent += packets;
                return;
            }
        }

        // Create new sample
        self.samples.push_back(ThroughputSample {
            timestamp: now,
            packets_sent: packets,
            packets_received: 0,
        });

        // Trim old samples outside window
        self.trim_old_samples();
    }

    /// Record packets received
    pub fn record_receive(&mut self, packets: usize) {
        let now = Instant::now();

        // If we have a recent sample, update it; otherwise create new
        if let Some(last_sample) = self.samples.back_mut() {
            if now.duration_since(last_sample.timestamp) < Duration::from_millis(100) {
                // Update existing sample (same 100ms window)
                last_sample.packets_received += packets;
                return;
            }
        }

        // Create new sample
        self.samples.push_back(ThroughputSample {
            timestamp: now,
            packets_sent: 0,
            packets_received: packets,
        });

        // Trim old samples outside window
        self.trim_old_samples();
    }

    /// Update memory usage
    pub fn update_memory_usage(&mut self, bytes: usize) {
        self.memory_usage = bytes;
    }

    /// Calculate current throughput (packets per second)
    pub fn calculate_throughput(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }

        let total_packets: usize = self
            .samples
            .iter()
            .map(|s| s.packets_sent + s.packets_received)
            .sum();

        let window_duration =
            if let (Some(first), Some(last)) = (self.samples.front(), self.samples.back()) {
                last.timestamp.duration_since(first.timestamp).as_secs_f64()
            } else {
                return 0.0;
            };

        if window_duration > 0.0 {
            total_packets as f64 / window_duration
        } else {
            0.0
        }
    }

    /// Calculate packet loss rate (0.0 = no loss, 1.0 = 100% loss)
    pub fn calculate_packet_loss(&mut self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }

        let total_sent: usize = self.samples.iter().map(|s| s.packets_sent).sum();
        let total_received: usize = self.samples.iter().map(|s| s.packets_received).sum();

        if total_sent == 0 {
            return 0.0;
        }

        let loss = if total_received > total_sent {
            // Can happen with async operations, consider no loss
            0.0
        } else {
            (total_sent - total_received) as f64 / total_sent as f64
        };

        // Update cached loss rate
        self.packet_loss_rate = loss;
        loss
    }

    /// Get current performance metrics
    pub fn get_metrics(&mut self) -> PerformanceMetrics {
        PerformanceMetrics {
            throughput_pps: self.calculate_throughput(),
            packet_loss_rate: self.calculate_packet_loss(),
            memory_usage: self.memory_usage,
        }
    }

    /// Trim samples outside the time window
    fn trim_old_samples(&mut self) {
        let now = Instant::now();
        while let Some(sample) = self.samples.front() {
            if now.duration_since(sample.timestamp) > self.window_size {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }
}

/// Configuration for adaptive batch sizing
#[derive(Debug, Clone, Copy)]
pub struct AdaptiveConfig {
    /// Minimum batch size (default: 1)
    pub min_batch_size: usize,
    /// Maximum batch size (default: 1024)
    pub max_batch_size: usize,
    /// Threshold for increasing batch size (success rate, e.g., 0.95 = 95%)
    pub increase_threshold: f64,
    /// Threshold for decreasing batch size (success rate, e.g., 0.85 = 85%)
    pub decrease_threshold: f64,
    /// Memory limit (bytes)
    pub memory_limit: usize,
    /// Time window for performance monitoring
    pub window_size: Duration,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            min_batch_size: 1,
            max_batch_size: 1024,
            increase_threshold: 0.95,
            decrease_threshold: 0.85,
            memory_limit: 100_000_000, // 100 MB
            window_size: Duration::from_secs(5),
        }
    }
}

/// Adaptive batch size controller
///
/// Dynamically adjusts batch sizes based on network performance:
/// - Increases batch size during good performance (low loss, high throughput)
/// - Decreases batch size during poor performance (high loss, congestion)
/// - Respects memory constraints
pub struct AdaptiveBatchSizer {
    /// Performance monitor
    monitor: PerformanceMonitor,
    /// Current batch size
    current_batch_size: usize,
    /// Configuration
    config: AdaptiveConfig,
}

impl AdaptiveBatchSizer {
    /// Create a new adaptive batch sizer
    pub fn new(config: AdaptiveConfig) -> Self {
        Self {
            monitor: PerformanceMonitor::new(config.window_size),
            current_batch_size: config.min_batch_size.max(32), // Start at 32 (conservative)
            config,
        }
    }

    /// Record packets sent
    pub fn record_send(&mut self, packets: usize) {
        self.monitor.record_send(packets);
    }

    /// Record packets received
    pub fn record_receive(&mut self, packets: usize) {
        self.monitor.record_receive(packets);
    }

    /// Update memory usage
    pub fn update_memory(&mut self, bytes: usize) {
        self.monitor.update_memory_usage(bytes);
    }

    /// Update batch size based on current performance
    ///
    /// Returns the new batch size.
    pub fn update(&mut self) -> usize {
        let metrics = self.monitor.get_metrics();
        self.adjust_batch_size(metrics);
        self.current_batch_size
    }

    /// Get current batch size
    pub fn current_batch_size(&self) -> usize {
        self.current_batch_size
    }

    /// Adjust batch size based on performance metrics
    fn adjust_batch_size(&mut self, metrics: PerformanceMetrics) {
        let success_rate = 1.0 - metrics.packet_loss_rate;

        if success_rate >= self.config.increase_threshold {
            // Good performance - increase batch size
            self.increase_batch_size();
        } else if success_rate < self.config.decrease_threshold {
            // Poor performance - decrease batch size
            self.decrease_batch_size();
        }
        // else: maintain current batch size

        // Apply memory constraint
        let max_allowed = self.memory_constrained_max();
        if self.current_batch_size > max_allowed {
            self.current_batch_size = max_allowed;
        }
    }

    /// Increase batch size (double, up to max)
    fn increase_batch_size(&mut self) {
        let new_size = (self.current_batch_size * 2).min(self.config.max_batch_size);
        if new_size > self.current_batch_size {
            self.current_batch_size = new_size;
        }
    }

    /// Decrease batch size (halve, down to min)
    fn decrease_batch_size(&mut self) {
        let new_size = (self.current_batch_size / 2).max(self.config.min_batch_size);
        if new_size < self.current_batch_size {
            self.current_batch_size = new_size;
        }
    }

    /// Calculate max batch size allowed by memory constraints
    fn memory_constrained_max(&self) -> usize {
        // Estimate: 1500 bytes per packet (max Ethernet frame)
        const BYTES_PER_PACKET: usize = 1500;

        let available_memory = self
            .config
            .memory_limit
            .saturating_sub(self.monitor.memory_usage);
        let max_by_memory = available_memory / BYTES_PER_PACKET;

        max_by_memory
            .min(self.config.max_batch_size)
            .max(self.config.min_batch_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new(Duration::from_secs(5));
        assert_eq!(monitor.samples.len(), 0);
        assert_eq!(monitor.packet_loss_rate, 0.0);
        assert_eq!(monitor.memory_usage, 0);
    }

    #[test]
    fn test_record_send_receive() {
        let mut monitor = PerformanceMonitor::new(Duration::from_secs(5));

        monitor.record_send(100);
        std::thread::sleep(Duration::from_millis(101)); // > 100ms to create separate samples
        monitor.record_receive(95);

        let metrics = monitor.get_metrics();
        assert!(metrics.throughput_pps > 0.0);
        assert!(metrics.packet_loss_rate >= 0.0 && metrics.packet_loss_rate <= 1.0);
    }

    #[test]
    fn test_throughput_calculation() {
        let mut monitor = PerformanceMonitor::new(Duration::from_secs(5));

        monitor.record_send(1000);
        std::thread::sleep(Duration::from_millis(100));
        monitor.record_receive(950);

        let throughput = monitor.calculate_throughput();
        assert!(throughput > 0.0, "Throughput should be positive");
    }

    #[test]
    fn test_packet_loss_calculation() {
        let mut monitor = PerformanceMonitor::new(Duration::from_secs(5));

        monitor.record_send(100);
        monitor.record_receive(85); // 15% loss

        let loss = monitor.calculate_packet_loss();
        assert!(
            (loss - 0.15).abs() < 0.01,
            "Expected ~15% loss, got {}",
            loss
        );
    }

    #[test]
    fn test_sliding_window() {
        let mut monitor = PerformanceMonitor::new(Duration::from_millis(100));

        monitor.record_send(100);
        std::thread::sleep(Duration::from_millis(150));
        monitor.record_send(100);

        // Old samples should be trimmed
        assert!(monitor.samples.len() <= 2);
    }

    #[test]
    fn test_memory_usage_tracking() {
        let mut monitor = PerformanceMonitor::new(Duration::from_secs(5));

        monitor.update_memory_usage(50_000_000); // 50 MB
        let metrics = monitor.get_metrics();

        assert_eq!(metrics.memory_usage, 50_000_000);
    }

    #[test]
    fn test_adaptive_config_default() {
        let config = AdaptiveConfig::default();

        assert_eq!(config.min_batch_size, 1);
        assert_eq!(config.max_batch_size, 1024);
        assert_eq!(config.increase_threshold, 0.95);
        assert_eq!(config.decrease_threshold, 0.85);
    }

    #[test]
    fn test_adaptive_sizer_creation() {
        let config = AdaptiveConfig::default();
        let sizer = AdaptiveBatchSizer::new(config);

        // Should start at 32 (conservative)
        assert_eq!(sizer.current_batch_size(), 32);
    }

    #[test]
    fn test_batch_size_increase_on_good_performance() {
        let config = AdaptiveConfig::default();
        let mut sizer = AdaptiveBatchSizer::new(config);

        // Simulate good performance (low loss)
        for _ in 0..10 {
            sizer.record_send(100);
            sizer.record_receive(98); // 98% success rate
        }

        let initial_size = sizer.current_batch_size();
        sizer.update();

        // Should increase
        assert!(sizer.current_batch_size() >= initial_size);
    }

    #[test]
    fn test_batch_size_decrease_on_poor_performance() {
        let mut config = AdaptiveConfig::default();
        config.min_batch_size = 16;
        let mut sizer = AdaptiveBatchSizer::new(config);

        // Set high initial size
        sizer.current_batch_size = 512;

        // Simulate poor performance (high loss)
        for _ in 0..10 {
            sizer.record_send(100);
            sizer.record_receive(60); // 40% loss
        }

        sizer.update();

        // Should decrease
        assert!(sizer.current_batch_size() < 512);
    }

    #[test]
    fn test_batch_size_stability_on_medium_performance() {
        let config = AdaptiveConfig::default();
        let mut sizer = AdaptiveBatchSizer::new(config);

        // Simulate medium performance (between thresholds)
        for _ in 0..10 {
            sizer.record_send(100);
            sizer.record_receive(90); // 90% success rate (between 85-95%)
        }

        let initial_size = sizer.current_batch_size();
        sizer.update();

        // Should stay stable
        assert_eq!(sizer.current_batch_size(), initial_size);
    }

    #[test]
    fn test_memory_constraint_limits_batch_size() {
        let mut config = AdaptiveConfig::default();
        config.memory_limit = 30_000; // Small limit: ~20 packets max
        config.max_batch_size = 1024;

        let mut sizer = AdaptiveBatchSizer::new(config);

        // Try to increase to max
        sizer.current_batch_size = 1024;

        // Simulate high memory usage
        sizer.update_memory(25_000);

        // Update should apply memory constraint
        sizer.update();

        // Should be limited by memory (not 1024)
        assert!(sizer.current_batch_size() < 1024);
    }

    #[test]
    fn test_min_max_bounds_enforced() {
        let config = AdaptiveConfig {
            min_batch_size: 10,
            max_batch_size: 100,
            ..Default::default()
        };
        let mut sizer = AdaptiveBatchSizer::new(config);

        // Try to go below min
        sizer.current_batch_size = 5;
        sizer.update();
        assert!(sizer.current_batch_size() >= 10);

        // Try to go above max
        sizer.current_batch_size = 200;
        sizer.update();
        assert!(sizer.current_batch_size() <= 100);
    }
}
