//! Hostgroup Limiter - Controls concurrent target scanning
//!
//! Implements Nmap-compatible --max-hostgroup and --min-hostgroup
//! using tokio::sync::Semaphore for async concurrency control.
//!
//! # Algorithm
//!
//! Uses a semaphore to limit concurrent target scanning, preventing
//! overwhelming of network resources or rate-limited systems.
//!
//! # Examples
//!
//! ```no_run
//! use prtip_scanner::HostgroupLimiter;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! // Limit to 64 concurrent targets (Nmap default)
//! let limiter = HostgroupLimiter::with_max(64);
//!
//! // Acquire permit before scanning each target
//! let _permit = limiter.acquire_target().await;
//! // ... scan target ...
//! // Permit automatically released when dropped
//! # Ok(())
//! # }
//! ```

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{Semaphore, SemaphorePermit};
use tracing::{debug, warn};

/// Hostgroup limiter configuration
#[derive(Debug, Clone)]
pub struct HostgroupConfig {
    /// Maximum number of concurrent targets
    pub max_hostgroup: usize,
    /// Minimum recommended concurrent targets
    pub min_hostgroup: usize,
}

impl Default for HostgroupConfig {
    fn default() -> Self {
        Self {
            max_hostgroup: 64, // Nmap default
            min_hostgroup: 1,
        }
    }
}

/// Hostgroup limiter - controls concurrent target scanning
///
/// Limits the number of targets being scanned simultaneously using
/// a semaphore. This prevents overwhelming network resources and
/// provides better control over scan parallelism.
///
/// # Thread Safety
///
/// This limiter is fully thread-safe and can be cloned cheaply
/// (uses `Arc` internally).
#[derive(Clone)]
pub struct HostgroupLimiter {
    config: HostgroupConfig,
    semaphore: Arc<Semaphore>,
    active_targets: Arc<AtomicUsize>,
}

impl HostgroupLimiter {
    /// Create new hostgroup limiter with configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Hostgroup configuration (max/min)
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::{HostgroupConfig, HostgroupLimiter};
    ///
    /// let config = HostgroupConfig {
    ///     max_hostgroup: 32,
    ///     min_hostgroup: 4,
    /// };
    /// let limiter = HostgroupLimiter::new(config);
    /// ```
    pub fn new(config: HostgroupConfig) -> Self {
        debug!(
            "Creating hostgroup limiter: max={}, min={}",
            config.max_hostgroup, config.min_hostgroup
        );

        Self {
            semaphore: Arc::new(Semaphore::new(config.max_hostgroup)),
            active_targets: Arc::new(AtomicUsize::new(0)),
            config,
        }
    }

    /// Create hostgroup limiter with maximum only (min=1)
    ///
    /// # Arguments
    ///
    /// * `max_hostgroup` - Maximum concurrent targets
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::HostgroupLimiter;
    ///
    /// let limiter = HostgroupLimiter::with_max(100);
    /// ```
    pub fn with_max(max_hostgroup: usize) -> Self {
        Self::new(HostgroupConfig {
            max_hostgroup,
            min_hostgroup: 1,
        })
    }

    /// Acquire permit to scan target (blocks if at limit)
    ///
    /// Returns a `TargetPermit` that automatically releases when dropped.
    /// If the number of active targets is below `min_hostgroup`, a warning
    /// is logged (performance may be suboptimal).
    ///
    /// # Returns
    ///
    /// A `TargetPermit` that must be held while scanning the target.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_scanner::HostgroupLimiter;
    /// # async fn example() {
    /// let limiter = HostgroupLimiter::with_max(64);
    ///
    /// {
    ///     let _permit = limiter.acquire_target().await;
    ///     // ... scan target ...
    /// } // Permit automatically released here
    /// # }
    /// ```
    pub async fn acquire_target(&self) -> TargetPermit<'_> {
        let permit = self
            .semaphore
            .acquire()
            .await
            .expect("Semaphore should not be closed");

        let count = self.active_targets.fetch_add(1, Ordering::Relaxed) + 1;

        // Warn if below min_hostgroup (performance concern)
        if count < self.config.min_hostgroup {
            warn!(
                "Active targets ({}) below min_hostgroup ({}), consider increasing parallelism",
                count, self.config.min_hostgroup
            );
        }

        debug!(
            "Target permit acquired: {}/{} active",
            count, self.config.max_hostgroup
        );

        TargetPermit {
            _permit: permit,
            limiter: self,
        }
    }

    /// Try to acquire permit without waiting
    ///
    /// # Returns
    ///
    /// `Some(TargetPermit)` if a permit was immediately available, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::HostgroupLimiter;
    ///
    /// let limiter = HostgroupLimiter::with_max(64);
    ///
    /// if let Some(_permit) = limiter.try_acquire_target() {
    ///     // Permission granted - scan target
    /// }; // <- Semicolon to drop permit before limiter
    /// ```
    pub fn try_acquire_target(&self) -> Option<TargetPermit<'_>> {
        self.semaphore.try_acquire().ok().map(|permit| {
            let count = self.active_targets.fetch_add(1, Ordering::Relaxed) + 1;

            debug!(
                "Target permit acquired (try): {}/{} active",
                count, self.config.max_hostgroup
            );

            TargetPermit {
                _permit: permit,
                limiter: self,
            }
        })
    }

    /// Get current active target count
    ///
    /// # Returns
    ///
    /// Number of targets currently being scanned.
    pub fn current_active(&self) -> usize {
        self.active_targets.load(Ordering::Relaxed)
    }

    /// Get maximum hostgroup configuration
    pub fn max_hostgroup(&self) -> usize {
        self.config.max_hostgroup
    }

    /// Get minimum hostgroup configuration
    pub fn min_hostgroup(&self) -> usize {
        self.config.min_hostgroup
    }
}

/// RAII permit - automatically releases when dropped
///
/// Holds a semaphore permit and tracks active target count.
/// Automatically decrements count when dropped (RAII pattern).
pub struct TargetPermit<'a> {
    _permit: SemaphorePermit<'a>,
    limiter: &'a HostgroupLimiter,
}

impl Drop for TargetPermit<'_> {
    fn drop(&mut self) {
        let count = self.limiter.active_targets.fetch_sub(1, Ordering::Relaxed) - 1;

        debug!(
            "Target permit released: {}/{} active",
            count,
            self.limiter.max_hostgroup()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_hostgroup_creation() {
        let config = HostgroupConfig::default();
        let limiter = HostgroupLimiter::new(config);

        assert_eq!(limiter.current_active(), 0);
        assert_eq!(limiter.max_hostgroup(), 64);
        assert_eq!(limiter.min_hostgroup(), 1);
    }

    #[tokio::test]
    async fn test_acquire_release() {
        let limiter = HostgroupLimiter::with_max(64);

        assert_eq!(limiter.current_active(), 0);

        {
            let _permit = limiter.acquire_target().await;
            assert_eq!(limiter.current_active(), 1);
        }

        // Permit dropped, count should decrease
        assert_eq!(limiter.current_active(), 0);
    }

    #[tokio::test]
    async fn test_max_limit_enforced() {
        let config = HostgroupConfig {
            max_hostgroup: 2,
            min_hostgroup: 1,
        };
        let limiter = Arc::new(HostgroupLimiter::new(config));

        let _p1 = limiter.acquire_target().await;
        let _p2 = limiter.acquire_target().await;
        assert_eq!(limiter.current_active(), 2);

        // Third acquire should block (timeout test)
        let limiter_clone = limiter.clone();
        let task = tokio::spawn(async move {
            let _p3 = limiter_clone.acquire_target().await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!task.is_finished()); // Should still be blocked
    }

    #[tokio::test]
    async fn test_concurrent_acquires() {
        let limiter = Arc::new(HostgroupLimiter::with_max(10));
        let mut handles = vec![];

        // Spawn 10 tasks (at limit)
        for _ in 0..10 {
            let limiter = limiter.clone();
            let handle = tokio::spawn(async move {
                let _permit = limiter.acquire_target().await;
                tokio::time::sleep(Duration::from_millis(50)).await;
            });
            handles.push(handle);
        }

        // Wait a bit for tasks to acquire permits
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should have 10 active
        assert!(limiter.current_active() <= 10);

        // Wait for all to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // All permits released
        assert_eq!(limiter.current_active(), 0);
    }

    #[test]
    fn test_config_defaults() {
        let config = HostgroupConfig::default();

        assert_eq!(config.max_hostgroup, 64);
        assert_eq!(config.min_hostgroup, 1);
    }

    #[test]
    fn test_config_custom() {
        let config = HostgroupConfig {
            max_hostgroup: 128,
            min_hostgroup: 8,
        };

        assert_eq!(config.max_hostgroup, 128);
        assert_eq!(config.min_hostgroup, 8);
    }

    #[tokio::test]
    async fn test_try_acquire_success() {
        let limiter = HostgroupLimiter::with_max(2);

        // First should succeed
        let _permit1 = limiter.try_acquire_target();
        assert!(_permit1.is_some());
        assert_eq!(limiter.current_active(), 1);

        // Second should succeed
        let _permit2 = limiter.try_acquire_target();
        assert!(_permit2.is_some());
        assert_eq!(limiter.current_active(), 2);
    }

    #[tokio::test]
    async fn test_try_acquire_failure() {
        let limiter = HostgroupLimiter::with_max(1);

        let _permit1 = limiter.try_acquire_target();
        assert!(_permit1.is_some());

        // Should fail (at limit)
        let permit2 = limiter.try_acquire_target();
        assert!(permit2.is_none());
    }

    #[tokio::test]
    async fn test_permit_auto_release() {
        let limiter = HostgroupLimiter::with_max(64);

        for _ in 0..5 {
            {
                let _permit = limiter.acquire_target().await;
                assert_eq!(limiter.current_active(), 1);
            }
            // Auto-released
            assert_eq!(limiter.current_active(), 0);
        }
    }

    #[tokio::test]
    async fn test_multiple_permits_same_task() {
        let limiter = HostgroupLimiter::with_max(64);

        let _p1 = limiter.acquire_target().await;
        let _p2 = limiter.acquire_target().await;
        let _p3 = limiter.acquire_target().await;

        assert_eq!(limiter.current_active(), 3);
    }
}
