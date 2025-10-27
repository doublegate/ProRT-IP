//! Resource monitoring with adaptive degradation
//!
//! This module provides system resource monitoring (memory, CPU) with
//! automatic degradation of scan parameters when resources are constrained.
//! Benefits:
//! - Prevents out-of-memory crashes
//! - Maintains system responsiveness under load
//! - Automatic recovery when resources become available
//! - Configurable thresholds per use case
//!
//! Sprint 4.22 Phase 4.3: Resource Monitor

use std::time::{Duration, Instant};
use sysinfo::System;

/// Resource status indicating system constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceStatus {
    /// Resources are plentiful, no constraints
    Normal,

    /// Memory is constrained (below threshold)
    MemoryConstrained,

    /// CPU is constrained (above threshold)
    CpuConstrained,

    /// Both memory and CPU are constrained
    Constrained,
}

impl ResourceStatus {
    /// Returns whether this status indicates any constraint
    pub fn is_constrained(&self) -> bool {
        !matches!(self, Self::Normal)
    }

    /// Returns whether memory is constrained
    pub fn is_memory_constrained(&self) -> bool {
        matches!(self, Self::MemoryConstrained | Self::Constrained)
    }

    /// Returns whether CPU is constrained
    pub fn is_cpu_constrained(&self) -> bool {
        matches!(self, Self::CpuConstrained | Self::Constrained)
    }
}

/// Resource monitoring configuration
#[derive(Debug, Clone)]
pub struct ResourceMonitorConfig {
    /// Available memory threshold (bytes) - warn when below this
    pub memory_threshold: u64,

    /// CPU usage threshold (percentage 0-100) - warn when above this
    pub cpu_threshold: f32,

    /// How often to check resources (avoid excessive polling)
    pub check_interval: Duration,
}

impl Default for ResourceMonitorConfig {
    fn default() -> Self {
        Self {
            memory_threshold: 512 * 1024 * 1024, // 512 MB
            cpu_threshold: 80.0,                   // 80%
            check_interval: Duration::from_secs(5),
        }
    }
}

impl ResourceMonitorConfig {
    /// Conservative configuration (more sensitive to constraints)
    ///
    /// Use when running on resource-limited systems or when stability
    /// is more important than performance.
    pub fn conservative() -> Self {
        Self {
            memory_threshold: 1024 * 1024 * 1024, // 1 GB
            cpu_threshold: 70.0,                    // 70%
            check_interval: Duration::from_secs(3),
        }
    }

    /// Aggressive configuration (less sensitive to constraints)
    ///
    /// Use when running on powerful systems where maximum throughput
    /// is desired and some resource pressure is acceptable.
    pub fn aggressive() -> Self {
        Self {
            memory_threshold: 256 * 1024 * 1024, // 256 MB
            cpu_threshold: 90.0,                  // 90%
            check_interval: Duration::from_secs(10),
        }
    }
}

/// Resource monitor with cached status
///
/// Monitors system memory and CPU usage, checking at configured intervals
/// to avoid excessive overhead. Provides adaptive recommendations for
/// adjusting scan parameters based on resource availability.
///
/// # Example
///
/// ```no_run
/// use prtip_core::resource_monitor::{ResourceMonitor, ResourceMonitorConfig};
///
/// let mut monitor = ResourceMonitor::new(ResourceMonitorConfig::default());
///
/// // Check periodically during scan
/// match monitor.check_resources() {
///     ResourceStatus::Normal => {
///         // Full speed ahead
///     }
///     ResourceStatus::MemoryConstrained => {
///         // Reduce batch sizes
///     }
///     ResourceStatus::CpuConstrained => {
///         // Reduce parallelism
///     }
///     ResourceStatus::Constrained => {
///         // Reduce both
///     }
/// }
/// ```
pub struct ResourceMonitor {
    system: System,
    config: ResourceMonitorConfig,
    last_check: Instant,
    last_status: ResourceStatus,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(config: ResourceMonitorConfig) -> Self {
        Self {
            system: System::new_all(),
            config,
            last_check: Instant::now(),
            last_status: ResourceStatus::Normal,
        }
    }

    /// Check resource status
    ///
    /// Returns the current resource status. If the check interval hasn't
    /// passed since the last check, returns the cached status to avoid
    /// excessive system calls.
    pub fn check_resources(&mut self) -> ResourceStatus {
        // Only check if interval has passed
        if self.last_check.elapsed() < self.config.check_interval {
            return self.last_status;
        }

        // Refresh system information
        self.system.refresh_memory();
        self.system.refresh_cpu();

        let available_memory = self.system.available_memory();
        let cpu_usage = self.system.global_cpu_info().cpu_usage();

        let memory_ok = available_memory >= self.config.memory_threshold;
        let cpu_ok = cpu_usage <= self.config.cpu_threshold;

        let status = match (memory_ok, cpu_ok) {
            (true, true) => ResourceStatus::Normal,
            (false, true) => ResourceStatus::MemoryConstrained,
            (true, false) => ResourceStatus::CpuConstrained,
            (false, false) => ResourceStatus::Constrained,
        };

        self.last_check = Instant::now();
        self.last_status = status;

        status
    }

    /// Get memory usage information (available, total) in bytes
    pub fn memory_info(&self) -> (u64, u64) {
        (self.system.available_memory(), self.system.total_memory())
    }

    /// Get CPU usage percentage (0-100)
    pub fn cpu_usage(&self) -> f32 {
        self.system.global_cpu_info().cpu_usage()
    }

    /// Get the last checked status without refreshing
    pub fn last_status(&self) -> ResourceStatus {
        self.last_status
    }

    /// Force a fresh resource check (ignoring check interval)
    pub fn force_check(&mut self) -> ResourceStatus {
        self.last_check = Instant::now() - self.config.check_interval;
        self.check_resources()
    }
}

/// Adaptive configuration for scan parameters based on resource status
///
/// Automatically adjusts parallelism and batch size based on system
/// resource availability to maintain stability while maximizing throughput.
#[derive(Debug, Clone)]
pub struct AdaptiveConfig {
    base_parallelism: usize,
    base_batch_size: usize,
}

impl AdaptiveConfig {
    /// Create adaptive configuration with base values
    pub fn new(base_parallelism: usize, base_batch_size: usize) -> Self {
        Self {
            base_parallelism,
            base_batch_size,
        }
    }

    /// Adapt configuration based on resource status
    ///
    /// Returns (parallelism, batch_size) adjusted for current resources:
    /// - Normal: Use base values
    /// - MemoryConstrained: Halve batch size (reduce memory footprint)
    /// - CpuConstrained: Halve parallelism (reduce CPU load)
    /// - Constrained: Halve both
    pub fn adapt(&self, status: ResourceStatus) -> (usize, usize) {
        match status {
            ResourceStatus::Normal => (self.base_parallelism, self.base_batch_size),
            ResourceStatus::MemoryConstrained => {
                // Reduce batch size to lower memory footprint
                (self.base_parallelism, self.base_batch_size / 2)
            }
            ResourceStatus::CpuConstrained => {
                // Reduce parallelism to lower CPU usage
                (self.base_parallelism / 2, self.base_batch_size)
            }
            ResourceStatus::Constrained => {
                // Reduce both
                (self.base_parallelism / 2, self.base_batch_size / 2)
            }
        }
    }

    /// Get base parallelism value
    pub fn base_parallelism(&self) -> usize {
        self.base_parallelism
    }

    /// Get base batch size value
    pub fn base_batch_size(&self) -> usize {
        self.base_batch_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_monitor_creation() {
        let monitor = ResourceMonitor::new(ResourceMonitorConfig::default());
        assert_eq!(monitor.last_status(), ResourceStatus::Normal);
    }

    #[test]
    fn test_resource_monitor_check() {
        let mut monitor = ResourceMonitor::new(ResourceMonitorConfig::default());
        let status = monitor.check_resources();
        // Should return some status (actual value depends on system)
        assert!(matches!(
            status,
            ResourceStatus::Normal
                | ResourceStatus::MemoryConstrained
                | ResourceStatus::CpuConstrained
                | ResourceStatus::Constrained
        ));
    }

    #[test]
    fn test_check_interval_respected() {
        let mut monitor = ResourceMonitor::new(ResourceMonitorConfig {
            check_interval: Duration::from_secs(10),
            ..Default::default()
        });

        // First check
        let status1 = monitor.check_resources();

        // Immediate second check should return cached value
        let start = Instant::now();
        let status2 = monitor.check_resources();
        let elapsed = start.elapsed();

        // Should be nearly instant (< 1ms) because it's cached
        assert!(elapsed < Duration::from_millis(10));
        assert_eq!(status1, status2);
    }

    #[test]
    fn test_memory_info() {
        let monitor = ResourceMonitor::new(ResourceMonitorConfig::default());
        let (available, total) = monitor.memory_info();

        // System should have some memory
        assert!(total > 0);
        assert!(available <= total);
    }

    #[test]
    fn test_cpu_usage() {
        let monitor = ResourceMonitor::new(ResourceMonitorConfig::default());
        let cpu = monitor.cpu_usage();

        // CPU usage should be between 0 and 100
        assert!(cpu >= 0.0);
        assert!(cpu <= 100.0);
    }

    #[test]
    fn test_force_check() {
        let mut monitor = ResourceMonitor::new(ResourceMonitorConfig {
            check_interval: Duration::from_secs(10),
            ..Default::default()
        });

        // First check
        monitor.check_resources();

        // Force check should update immediately
        let status = monitor.force_check();
        assert!(matches!(
            status,
            ResourceStatus::Normal
                | ResourceStatus::MemoryConstrained
                | ResourceStatus::CpuConstrained
                | ResourceStatus::Constrained
        ));
    }

    #[test]
    fn test_resource_status_is_constrained() {
        assert!(!ResourceStatus::Normal.is_constrained());
        assert!(ResourceStatus::MemoryConstrained.is_constrained());
        assert!(ResourceStatus::CpuConstrained.is_constrained());
        assert!(ResourceStatus::Constrained.is_constrained());
    }

    #[test]
    fn test_resource_status_is_memory_constrained() {
        assert!(!ResourceStatus::Normal.is_memory_constrained());
        assert!(ResourceStatus::MemoryConstrained.is_memory_constrained());
        assert!(!ResourceStatus::CpuConstrained.is_memory_constrained());
        assert!(ResourceStatus::Constrained.is_memory_constrained());
    }

    #[test]
    fn test_resource_status_is_cpu_constrained() {
        assert!(!ResourceStatus::Normal.is_cpu_constrained());
        assert!(!ResourceStatus::MemoryConstrained.is_cpu_constrained());
        assert!(ResourceStatus::CpuConstrained.is_cpu_constrained());
        assert!(ResourceStatus::Constrained.is_cpu_constrained());
    }

    #[test]
    fn test_adaptive_config_normal() {
        let config = AdaptiveConfig::new(100, 1000);
        let (parallelism, batch_size) = config.adapt(ResourceStatus::Normal);

        assert_eq!(parallelism, 100);
        assert_eq!(batch_size, 1000);
    }

    #[test]
    fn test_adaptive_config_memory_constrained() {
        let config = AdaptiveConfig::new(100, 1000);
        let (parallelism, batch_size) = config.adapt(ResourceStatus::MemoryConstrained);

        assert_eq!(parallelism, 100); // Unchanged
        assert_eq!(batch_size, 500); // Halved
    }

    #[test]
    fn test_adaptive_config_cpu_constrained() {
        let config = AdaptiveConfig::new(100, 1000);
        let (parallelism, batch_size) = config.adapt(ResourceStatus::CpuConstrained);

        assert_eq!(parallelism, 50); // Halved
        assert_eq!(batch_size, 1000); // Unchanged
    }

    #[test]
    fn test_adaptive_config_constrained() {
        let config = AdaptiveConfig::new(100, 1000);
        let (parallelism, batch_size) = config.adapt(ResourceStatus::Constrained);

        assert_eq!(parallelism, 50); // Halved
        assert_eq!(batch_size, 500); // Halved
    }

    #[test]
    fn test_resource_monitor_config_default() {
        let config = ResourceMonitorConfig::default();
        assert_eq!(config.memory_threshold, 512 * 1024 * 1024);
        assert_eq!(config.cpu_threshold, 80.0);
        assert_eq!(config.check_interval, Duration::from_secs(5));
    }

    #[test]
    fn test_resource_monitor_config_conservative() {
        let config = ResourceMonitorConfig::conservative();
        assert_eq!(config.memory_threshold, 1024 * 1024 * 1024);
        assert_eq!(config.cpu_threshold, 70.0);
        assert_eq!(config.check_interval, Duration::from_secs(3));
    }

    #[test]
    fn test_resource_monitor_config_aggressive() {
        let config = ResourceMonitorConfig::aggressive();
        assert_eq!(config.memory_threshold, 256 * 1024 * 1024);
        assert_eq!(config.cpu_threshold, 90.0);
        assert_eq!(config.check_interval, Duration::from_secs(10));
    }
}
