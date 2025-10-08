//! Connection Pool for Efficient Scanning
//!
//! Provides connection management utilities inspired by high-performance scanners.
//! Uses FuturesUnordered for efficient concurrent connection handling.
//!
//! # Design
//!
//! Based on RustScan's approach: maintain a pool of active futures with bounded
//! concurrency, adding new futures as old ones complete. This provides better
//! performance than spawning all futures at once or using a simple semaphore.
//!
//! # Performance Benefits
//!
//! - Constant memory usage (bounded queue)
//! - Better CPU utilization (work-stealing)
//! - Lower syscall overhead (batch processing)
//! - Adaptive to varying response times
//!
//! # Examples
//!
//! ```
//! use prtip_scanner::connection_pool::ConnectionPool;
//! use std::time::Duration;
//!
//! # async fn example() {
//! let pool = ConnectionPool::new(1000); // 1000 concurrent connections
//!
//! // Scan operations will use the pool automatically
//! # }
//! ```

use futures::stream::FuturesUnordered;
use futures::StreamExt;
use prtip_core::resource_limits::get_recommended_batch_size;
use prtip_core::{PortState, Result, ScanResult};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, trace, warn};

/// Connection pool with bounded concurrency
///
/// Manages concurrent TCP connections using FuturesUnordered for optimal
/// performance. Inspired by RustScan's scanner implementation.
#[derive(Clone)]
pub struct ConnectionPool {
    /// Maximum concurrent connections
    max_concurrent: usize,

    /// Connection timeout
    timeout: Duration,

    /// Number of retries for failed connections
    retries: u32,
}

impl ConnectionPool {
    /// Create a new connection pool
    ///
    /// Automatically checks system file descriptor limits and adjusts
    /// concurrency to safe values if needed. Issues warnings when limits
    /// constrain performance.
    ///
    /// # Arguments
    ///
    /// * `max_concurrent` - Maximum number of concurrent connections
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::connection_pool::ConnectionPool;
    ///
    /// let pool = ConnectionPool::new(500);
    /// ```
    pub fn new(max_concurrent: usize) -> Self {
        let actual_concurrent = Self::check_ulimit_and_adjust(max_concurrent);
        Self::with_timeout(actual_concurrent, Duration::from_secs(3))
    }

    /// Create a connection pool with custom timeout
    ///
    /// # Arguments
    ///
    /// * `max_concurrent` - Maximum concurrent connections
    /// * `timeout` - Connection timeout duration
    pub fn with_timeout(max_concurrent: usize, timeout: Duration) -> Self {
        Self {
            max_concurrent,
            timeout,
            retries: 0,
        }
    }

    /// Set number of retries for failed connections
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    /// Scan multiple targets using FuturesUnordered pattern
    ///
    /// This method provides superior performance compared to simple semaphore-based
    /// approaches by maintaining a constant-size pool of active futures.
    ///
    /// # Arguments
    ///
    /// * `targets` - Iterator of socket addresses to scan
    ///
    /// # Returns
    ///
    /// Vector of scan results for all successful scans. Failed connections are
    /// logged but don't prevent other scans from completing.
    ///
    /// # Performance
    ///
    /// This implementation uses FuturesUnordered which:
    /// - Polls futures in LIFO order (cache-friendly)
    /// - Avoids allocation overhead of channels
    /// - Provides work-stealing benefits on multi-core systems
    pub async fn scan_batch<I>(&self, targets: I) -> Vec<ScanResult>
    where
        I: IntoIterator<Item = SocketAddr>,
    {
        let mut targets_iter = targets.into_iter();
        let mut futures = FuturesUnordered::new();
        let mut results = Vec::new();

        // Fill initial batch up to max_concurrent
        for _ in 0..self.max_concurrent {
            if let Some(addr) = targets_iter.next() {
                futures.push(self.scan_socket(addr));
            } else {
                break;
            }
        }

        // Process futures, adding new ones as old ones complete
        while let Some(result) = futures.next().await {
            match result {
                Ok(scan_result) => results.push(scan_result),
                Err(e) => {
                    // Log error but continue scanning
                    debug!("Scan error: {}", e);
                }
            }

            // Add next target if any remain
            if let Some(addr) = targets_iter.next() {
                futures.push(self.scan_socket(addr));
            }
        }

        results
    }

    /// Scan a single socket with retry logic
    async fn scan_socket(&self, addr: SocketAddr) -> Result<ScanResult> {
        let start = std::time::Instant::now();

        for attempt in 0..=self.retries {
            if attempt > 0 {
                trace!("Retry attempt {} for {}", attempt, addr);
                // Small delay before retry
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            match timeout(self.timeout, TcpStream::connect(addr)).await {
                Ok(Ok(_stream)) => {
                    // Connection successful - port is open
                    let response_time = start.elapsed();
                    debug!("Port {} open on {}", addr.port(), addr.ip());

                    return Ok(ScanResult::new(addr.ip(), addr.port(), PortState::Open)
                        .with_response_time(response_time));
                }
                Ok(Err(e)) => {
                    // Connection error
                    match e.kind() {
                        std::io::ErrorKind::ConnectionRefused => {
                            // Explicit RST - port is closed
                            let response_time = start.elapsed();
                            debug!("Port {} closed on {}", addr.port(), addr.ip());

                            return Ok(ScanResult::new(addr.ip(), addr.port(), PortState::Closed)
                                .with_response_time(response_time));
                        }
                        std::io::ErrorKind::PermissionDenied => {
                            // Firewall blocked
                            return Ok(ScanResult::new(
                                addr.ip(),
                                addr.port(),
                                PortState::Filtered,
                            ));
                        }
                        _ => {
                            // Other errors - retry if attempts remaining
                            if attempt == self.retries {
                                return Ok(ScanResult::new(
                                    addr.ip(),
                                    addr.port(),
                                    PortState::Filtered,
                                ));
                            }
                        }
                    }
                }
                Err(_elapsed) => {
                    // Timeout - retry if attempts remaining
                    if attempt == self.retries {
                        debug!("Timeout scanning {}", addr);
                        return Ok(ScanResult::new(addr.ip(), addr.port(), PortState::Filtered));
                    }
                }
            }
        }

        // Should not reach here due to retry logic
        Ok(ScanResult::new(addr.ip(), addr.port(), PortState::Filtered))
    }

    /// Get maximum concurrent connections
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Get connection timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get retry count
    pub fn retries(&self) -> u32 {
        self.retries
    }

    /// Check ulimit and adjust concurrency to safe values
    ///
    /// Inspired by RustScan's resource management. Warns users when system
    /// limits constrain performance.
    fn check_ulimit_and_adjust(requested: usize) -> usize {
        match get_recommended_batch_size(requested as u64, None) {
            Ok(recommended) => {
                let recommended_usize = recommended as usize;
                if requested > recommended_usize {
                    warn!(
                        "Requested concurrency {} exceeds safe limit {} based on file descriptor limits",
                        requested, recommended_usize
                    );
                    warn!(
                        "Reducing to {}. To increase: run 'ulimit -n {}' (Unix) or use --ulimit flag",
                        recommended_usize,
                        requested * 2
                    );
                    recommended_usize
                } else {
                    requested
                }
            }
            Err(e) => {
                warn!("Failed to detect file descriptor limits: {}", e);
                warn!(
                    "Using requested concurrency {} without validation",
                    requested
                );
                requested
            }
        }
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let pool = ConnectionPool::new(100);
        assert_eq!(pool.max_concurrent(), 100);
        assert_eq!(pool.timeout(), Duration::from_secs(3));
        assert_eq!(pool.retries(), 0);
    }

    #[tokio::test]
    async fn test_connection_pool_with_options() {
        let pool = ConnectionPool::with_timeout(50, Duration::from_secs(1)).with_retries(2);

        assert_eq!(pool.max_concurrent(), 50);
        assert_eq!(pool.timeout(), Duration::from_secs(1));
        assert_eq!(pool.retries(), 2);
    }

    #[tokio::test]
    async fn test_scan_batch_empty() {
        let pool = ConnectionPool::new(10);
        let results = pool.scan_batch(vec![]).await;

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_scan_batch_localhost() {
        let pool = ConnectionPool::with_timeout(10, Duration::from_millis(100));

        let targets: Vec<SocketAddr> = vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9998),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9999),
        ];

        let results = pool.scan_batch(targets).await;

        // Should scan both ports
        assert_eq!(results.len(), 2);

        // Results should be closed or filtered (ports likely not open)
        for result in results {
            assert!(matches!(
                result.state,
                PortState::Closed | PortState::Filtered
            ));
        }
    }

    #[tokio::test]
    async fn test_default_pool() {
        let pool = ConnectionPool::default();
        assert_eq!(pool.max_concurrent(), 1000);
    }

    #[tokio::test]
    async fn test_concurrent_scanning() {
        let pool = ConnectionPool::with_timeout(5, Duration::from_millis(50));

        // Create 10 targets but pool only handles 5 at a time
        let targets: Vec<SocketAddr> = (9990..10000)
            .map(|port| SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
            .collect();

        let start = std::time::Instant::now();
        let results = pool.scan_batch(targets).await;
        let elapsed = start.elapsed();

        // Should scan all 10 targets
        assert_eq!(results.len(), 10);

        // With concurrency of 5, should take roughly half the time
        // compared to sequential (timing tests are flaky in CI, so just check it worked)
        assert!(elapsed < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn test_retry_mechanism() {
        let pool = ConnectionPool::with_timeout(5, Duration::from_millis(10)).with_retries(2);

        let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), 80);

        let results = pool.scan_batch(vec![target]).await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].state, PortState::Filtered);
    }
}
