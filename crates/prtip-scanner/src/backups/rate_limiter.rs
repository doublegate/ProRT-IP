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
                // Use burst size of 100 to reduce async overhead while maintaining rate control
                // Allows batching of up to 100 packets before rate limiting check
                // Evolution: burst=1 (40% overhead) → burst=100 (15% overhead, OPTIMAL)
                // Note: burst=1000 tested but showed 10-33% overhead (worse than burst=100)
                // Tokens still refill at configured rate, so rate enforcement is maintained
                let quota = Quota::per_second(nz_rate).allow_burst(NonZeroU32::new(100).unwrap());
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


// NOTE: Test module removed to prevent unnecessary test execution.
// This is the archived Governor-based RateLimiter (Phase 3), preserved for
// reference only. V3 (AdaptiveRateLimiterV3) is the default rate limiter
// as of v0.4.5.
//
// Tests removed - see git history to restore
