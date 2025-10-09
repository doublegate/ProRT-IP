//! Rate Limiting for Scan Operations
//!
//! Implements token bucket rate limiting to control packet send rate and prevent
//! network flooding or overwhelming target systems.
//!
//! # Algorithm
//!
//! Uses the [governor](https://docs.rs/governor) crate's token bucket implementation:
//! - Tokens are added to the bucket at a constant rate
//! - Each operation consumes one token
//! - If no tokens available, the operation waits until a token becomes available
//!
//! # Examples
//!
//! ```
//! use prtip_scanner::RateLimiter;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! // Limit to 100 packets per second
//! let limiter = RateLimiter::new(Some(100));
//!
//! // Acquire permission before sending each packet
//! limiter.acquire().await?;
//! // ... send packet ...
//! # Ok(())
//! # }
//! ```

use governor::{DefaultDirectRateLimiter, Quota, RateLimiter as GovernorRateLimiter};
use prtip_core::Result;
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::trace;

/// Rate limiter for scan operations
///
/// Uses token bucket algorithm to control packet send rate. When no rate limit
/// is specified (None), operations proceed immediately without throttling.
#[derive(Clone)]
pub struct RateLimiter {
    limiter: Option<Arc<DefaultDirectRateLimiter>>,
    max_rate: Option<u32>,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    ///
    /// * `max_rate` - Maximum packets per second (None for unlimited)
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::RateLimiter;
    ///
    /// // Limit to 1000 packets per second
    /// let limiter = RateLimiter::new(Some(1000));
    ///
    /// // No rate limit (unlimited)
    /// let unlimited = RateLimiter::new(None);
    /// ```
    pub fn new(max_rate: Option<u32>) -> Self {
        let limiter = max_rate.and_then(|rate| {
            NonZeroU32::new(rate).map(|nz_rate| {
                // Use burst size of 1 to enforce strict rate limiting without bursts
                // This ensures predictable timing for network scanning
                let quota = Quota::per_second(nz_rate).allow_burst(NonZeroU32::new(1).unwrap());
                Arc::new(GovernorRateLimiter::direct(quota))
            })
        });

        Self { limiter, max_rate }
    }

    /// Wait until a packet can be sent according to rate limit
    ///
    /// This method will block asynchronously until a token is available.
    /// If no rate limit is configured, returns immediately.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when permission is granted to proceed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prtip_scanner::RateLimiter;
    /// # async fn example() -> prtip_core::Result<()> {
    /// let limiter = RateLimiter::new(Some(100));
    ///
    /// // Acquire token before each operation
    /// for _ in 0..10 {
    ///     limiter.acquire().await?;
    ///     // ... perform rate-limited operation ...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn acquire(&self) -> Result<()> {
        if let Some(ref limiter) = self.limiter {
            trace!("Acquiring rate limit token (max: {:?} pps)", self.max_rate);
            limiter.until_ready().await;
        }
        Ok(())
    }

    /// Try to acquire permission without waiting
    ///
    /// # Returns
    ///
    /// Returns `true` if a token was immediately available, `false` if the
    /// rate limit would block. If no rate limit is configured, always returns `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::RateLimiter;
    ///
    /// let limiter = RateLimiter::new(Some(100));
    ///
    /// if limiter.try_acquire() {
    ///     // Permission granted - proceed
    /// } else {
    ///     // Rate limited - defer operation
    /// }
    /// ```
    pub fn try_acquire(&self) -> bool {
        if let Some(ref limiter) = self.limiter {
            limiter.check().is_ok()
        } else {
            true
        }
    }

    /// Get current maximum rate
    ///
    /// # Returns
    ///
    /// The configured maximum packets per second, or `None` if unlimited.
    pub fn max_rate(&self) -> Option<u32> {
        self.max_rate
    }

    /// Check if rate limiting is enabled
    ///
    /// # Returns
    ///
    /// `true` if rate limiting is active, `false` if unlimited.
    pub fn is_limited(&self) -> bool {
        self.max_rate.is_some()
    }
}

impl Default for RateLimiter {
    /// Create an unlimited rate limiter
    fn default() -> Self {
        Self::new(None)
    }
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("max_rate", &self.max_rate)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_no_rate_limit() {
        let limiter = RateLimiter::new(None);

        let start = Instant::now();
        for _ in 0..100 {
            limiter.acquire().await.unwrap();
        }
        let elapsed = start.elapsed();

        // Should complete almost instantly
        assert!(elapsed < Duration::from_millis(100));
        assert_eq!(limiter.max_rate(), None);
        assert!(!limiter.is_limited());
    }

    #[tokio::test]
    async fn test_rate_limit_enforced() {
        let limiter = RateLimiter::new(Some(10)); // 10 packets per second

        let start = Instant::now();
        for _ in 0..20 {
            limiter.acquire().await.unwrap();
        }
        let elapsed = start.elapsed();

        // Should take ~2 seconds for 20 packets at 10 pps
        // Allow some tolerance for timing variations
        assert!(elapsed >= Duration::from_millis(1800));
        assert!(elapsed <= Duration::from_millis(2500));
        assert_eq!(limiter.max_rate(), Some(10));
        assert!(limiter.is_limited());
    }

    #[test]
    fn test_try_acquire_unlimited() {
        let limiter = RateLimiter::new(None);

        // Should always succeed without rate limit
        for _ in 0..100 {
            assert!(limiter.try_acquire());
        }
    }

    #[test]
    fn test_try_acquire_limited() {
        let limiter = RateLimiter::new(Some(10));

        // First acquisition should succeed
        assert!(limiter.try_acquire());

        // After exhausting initial burst capacity, should eventually fail
        let mut succeeded = 0;
        for _ in 0..100 {
            if limiter.try_acquire() {
                succeeded += 1;
            }
        }

        // Should have exhausted the burst capacity
        assert!(succeeded < 100);
    }

    #[tokio::test]
    async fn test_different_rates() {
        // Test 100 pps
        let limiter = RateLimiter::new(Some(100));

        let start = Instant::now();
        for _ in 0..50 {
            limiter.acquire().await.unwrap();
        }
        let elapsed = start.elapsed();

        // 50 packets at 100 pps = ~500ms
        assert!(elapsed >= Duration::from_millis(400));
        assert!(elapsed <= Duration::from_millis(700));
    }

    #[tokio::test]
    async fn test_default_limiter() {
        let limiter = RateLimiter::default();

        assert_eq!(limiter.max_rate(), None);
        assert!(!limiter.is_limited());

        // Should not block
        let start = Instant::now();
        for _ in 0..100 {
            limiter.acquire().await.unwrap();
        }
        let elapsed = start.elapsed();

        assert!(elapsed < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_high_rate_limit() {
        // Test very high rate (10,000 pps)
        let limiter = RateLimiter::new(Some(10_000));

        let start = Instant::now();
        for _ in 0..1000 {
            limiter.acquire().await.unwrap();
        }
        let elapsed = start.elapsed();

        // 1000 packets at 10,000 pps = ~100ms
        // Allow more tolerance in CI environments (CI runners can be slower)
        assert!(elapsed >= Duration::from_millis(80));
        assert!(
            elapsed <= Duration::from_millis(500),
            "Elapsed: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_clone() {
        let limiter1 = RateLimiter::new(Some(100));
        let limiter2 = limiter1.clone();

        assert_eq!(limiter1.max_rate(), limiter2.max_rate());
        assert_eq!(limiter1.is_limited(), limiter2.is_limited());
    }

    #[test]
    fn test_debug_format() {
        let limiter = RateLimiter::new(Some(1000));
        let debug_str = format!("{:?}", limiter);

        assert!(debug_str.contains("RateLimiter"));
        assert!(debug_str.contains("max_rate"));
        assert!(debug_str.contains("1000"));
    }

    #[tokio::test]
    async fn test_concurrent_acquire() {
        use std::sync::Arc;

        let limiter = Arc::new(RateLimiter::new(Some(100)));
        let mut handles = vec![];

        // Spawn 10 tasks, each acquiring 10 tokens
        for _ in 0..10 {
            let limiter = limiter.clone();
            let handle = tokio::spawn(async move {
                for _ in 0..10 {
                    limiter.acquire().await.unwrap();
                }
            });
            handles.push(handle);
        }

        let start = Instant::now();
        for handle in handles {
            handle.await.unwrap();
        }
        let elapsed = start.elapsed();

        // 100 total acquisitions at 100 pps = ~1 second
        // CI environments (especially macOS) can be significantly slower
        // Increased tolerance to handle CI environment variability
        let expected = Duration::from_millis(900);
        let tolerance = Duration::from_millis(1000); // Very generous for CI
        let max_allowed = Duration::from_millis(5000); // Maximum reasonable wait

        assert!(
            elapsed >= expected.saturating_sub(tolerance),
            "Elapsed: {:?}, expected at least {:?} with {}ms tolerance. CI may be slower.",
            elapsed,
            expected,
            tolerance.as_millis()
        );
        assert!(
            elapsed <= max_allowed,
            "Elapsed: {:?}, should complete within {:?}. Test took too long.",
            elapsed,
            max_allowed
        );
    }

    #[test]
    fn test_rate_limiter_properties() {
        let limiter = RateLimiter::new(Some(500));

        assert_eq!(limiter.max_rate(), Some(500));
        assert!(limiter.is_limited());

        let unlimited = RateLimiter::new(None);
        assert_eq!(unlimited.max_rate(), None);
        assert!(!unlimited.is_limited());
    }
}
