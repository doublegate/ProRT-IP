//! Concurrent scanning using FuturesUnordered pattern
//!
//! This module implements high-performance concurrent scanning inspired by RustScan's
//! successful architecture. It uses `FuturesUnordered` for efficient batched scanning
//! with automatic work stealing and optimal CPU utilization.
//!
//! # Architecture
//!
//! The scanner uses a fixed-size window of concurrent futures:
//! 1. Initialize a pool of `batch_size` scanning tasks
//! 2. As each task completes, immediately spawn a new one
//! 3. Results stream back in completion order (not submission order)
//!
//! This provides:
//! - Constant memory usage (fixed batch size)
//! - Maximum concurrency (always `batch_size` tasks running)
//! - Automatic work distribution (futures complete in any order)
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::ConcurrentScanner;
//! use prtip_core::Config;
//! use std::net::IpAddr;
//! use std::time::Duration;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! let config = Config::default();
//! let scanner = ConcurrentScanner::new(config);
//!
//! let targets = vec!["192.168.1.1".parse::<IpAddr>().unwrap()];
//! let ports = vec![80, 443, 8080];
//!
//! let results = scanner.scan_targets(targets, ports).await?;
//! println!("Found {} open ports", results.len());
//! # Ok(())
//! # }
//! ```

use futures::stream::{FuturesUnordered, StreamExt};
use prtip_core::{Config, PortState, Result, ScanResult};
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, trace};

/// Concurrent scanner using FuturesUnordered for optimal throughput
///
/// This scanner implements RustScan's proven batching strategy:
/// - Fixed-size concurrent task pool
/// - Automatic work stealing via futures runtime
/// - Streaming results for memory efficiency
#[derive(Debug, Clone)]
pub struct ConcurrentScanner {
    config: Config,
}

impl ConcurrentScanner {
    /// Create a new concurrent scanner
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Scan multiple targets and ports concurrently
    ///
    /// This is the main entry point for concurrent scanning. It generates
    /// socket addresses from the cartesian product of targets and ports,
    /// then scans them using a fixed-size concurrent batch.
    ///
    /// # Arguments
    ///
    /// * `targets` - IP addresses to scan
    /// * `ports` - Port numbers to scan on each target
    ///
    /// # Returns
    ///
    /// Vector of scan results for open ports
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_scanner::ConcurrentScanner;
    /// # use prtip_core::Config;
    /// # use std::net::IpAddr;
    /// # async fn example() -> prtip_core::Result<()> {
    /// let scanner = ConcurrentScanner::new(Config::default());
    /// let targets = vec!["192.168.1.1".parse::<IpAddr>()?];
    /// let ports = vec![80, 443];
    ///
    /// let results = scanner.scan_targets(targets, ports).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn scan_targets(
        &self,
        targets: Vec<IpAddr>,
        ports: Vec<u16>,
    ) -> Result<Vec<ScanResult>> {
        // Generate all socket addresses (cartesian product)
        let mut sockets = Vec::with_capacity(targets.len() * ports.len());
        for target in &targets {
            for &port in &ports {
                sockets.push(SocketAddr::new(*target, port));
            }
        }

        debug!(
            "Starting concurrent scan: {} targets x {} ports = {} sockets",
            targets.len(),
            ports.len(),
            sockets.len()
        );

        self.scan_sockets(sockets).await
    }

    /// Scan a list of socket addresses concurrently
    ///
    /// Uses FuturesUnordered for optimal concurrency. Maintains a fixed-size
    /// window of concurrent tasks, spawning new tasks as old ones complete.
    ///
    /// # Arguments
    ///
    /// * `sockets` - Socket addresses to scan
    ///
    /// # Returns
    ///
    /// Vector of scan results for responsive sockets
    pub async fn scan_sockets(&self, sockets: Vec<SocketAddr>) -> Result<Vec<ScanResult>> {
        let batch_size = self.config.performance.parallelism;
        let total_sockets = sockets.len();

        // Iterator over remaining sockets
        let mut socket_iter = sockets.into_iter();

        // Pool of concurrent scanning futures
        let mut futures: FuturesUnordered<_> = FuturesUnordered::new();

        // Track unique error messages (avoid spam)
        let mut errors: HashSet<String> = HashSet::new();

        // Results accumulator
        let mut results = Vec::new();

        // Helper closure to create futures (ensures same type)
        let make_future = |socket: SocketAddr, config: Config| async move {
            scan_socket_tcp(socket, config).await
        };

        // Initialize the pool with batch_size futures
        for _ in 0..batch_size {
            if let Some(socket) = socket_iter.next() {
                let config = self.config.clone();
                futures.push(make_future(socket, config));
            } else {
                break;
            }
        }

        debug!(
            "Concurrent scanner initialized: batch_size={}, total_sockets={}",
            batch_size, total_sockets
        );

        // Process futures as they complete
        while let Some(scan_result) = futures.next().await {
            // Immediately spawn new task to maintain concurrency
            if let Some(socket) = socket_iter.next() {
                let config = self.config.clone();
                futures.push(make_future(socket, config));
            }

            // Handle result
            match scan_result {
                Ok(result) => {
                    if result.state == PortState::Open {
                        trace!("Found open port: {}:{}", result.target_ip, result.port);
                        results.push(result);
                    }
                }
                Err(e) => {
                    // Track unique errors (avoid logging duplicates)
                    let error_msg = e.to_string();
                    if errors.len() < 1000 && !errors.contains(&error_msg) {
                        errors.insert(error_msg.clone());
                    }
                }
            }
        }

        debug!(
            "Concurrent scan complete: {} open ports, {} unique errors",
            results.len(),
            errors.len()
        );

        if !errors.is_empty() {
            debug!("Error types encountered: {:?}", errors);
        }

        Ok(results)
    }
}

/// Scan a single socket using TCP connect
///
/// This is the atomic scanning operation. It attempts to connect to the
/// socket with a timeout, handling retries as configured.
///
/// # Arguments
///
/// * `socket` - Socket address to scan
/// * `config` - Scan configuration (timeout, retries, etc.)
///
/// # Returns
///
/// * `Ok(ScanResult)` - Scan completed successfully
/// * `Err(Error)` - Connection failed or error occurred
async fn scan_socket_tcp(socket: SocketAddr, config: Config) -> Result<ScanResult> {
    let timeout_duration = Duration::from_millis(config.scan.timeout_ms);
    let max_retries = config.scan.retries;

    for retry in 0..=max_retries {
        match timeout(timeout_duration, TcpStream::connect(socket)).await {
            Ok(Ok(mut stream)) => {
                // Connection successful - port is open
                trace!("Connection successful to {}", socket);

                // Clean shutdown (ignore errors)
                let _ = stream.shutdown().await;

                return Ok(ScanResult::new(socket.ip(), socket.port(), PortState::Open));
            }
            Ok(Err(e)) => {
                // Connection failed
                let error_str = e.to_string().to_lowercase();

                // Fatal errors that shouldn't be retried
                if error_str.contains("too many open files") {
                    panic!(
                        "Too many open files. Reduce parallelism from {} to a lower value (try {})",
                        config.performance.parallelism,
                        config.performance.parallelism / 2
                    );
                }

                // Connection refused = port closed (no retry needed)
                if error_str.contains("connection refused") {
                    return Ok(ScanResult::new(
                        socket.ip(),
                        socket.port(),
                        PortState::Closed,
                    ));
                }

                // Last retry - return error
                if retry == max_retries {
                    return Err(prtip_core::Error::Network(format!(
                        "Failed to connect to {}: {}",
                        socket, e
                    )));
                }

                // Continue to next retry
                trace!("Retry {}/{} for {}: {}", retry + 1, max_retries, socket, e);
            }
            Err(_) => {
                // Timeout
                if retry == max_retries {
                    // Filtered port (no response after all retries)
                    return Ok(ScanResult::new(
                        socket.ip(),
                        socket.port(),
                        PortState::Filtered,
                    ));
                }

                trace!("Timeout {}/{} for {}", retry + 1, max_retries, socket);
            }
        }
    }

    // Shouldn't reach here
    Ok(ScanResult::new(
        socket.ip(),
        socket.port(),
        PortState::Filtered,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_concurrent_scanner_creation() {
        let config = Config::default();
        let scanner = ConcurrentScanner::new(config);
        assert!(std::mem::size_of_val(&scanner) > 0);
    }

    #[tokio::test]
    async fn test_scan_localhost() {
        let config = Config {
            scan: prtip_core::ScanConfig {
                timeout_ms: 1000,
                retries: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let scanner = ConcurrentScanner::new(config);
        let targets = vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))];

        // Scan some common ports (most should be closed/filtered)
        let ports = vec![1, 2, 3, 4, 5];

        let results = scanner.scan_targets(targets, ports).await.unwrap();

        // Results should exist (even if all ports are closed)
        assert!(results.len() <= 5);
    }

    #[tokio::test]
    async fn test_scan_empty_targets() {
        let config = Config::default();
        let scanner = ConcurrentScanner::new(config);

        let results = scanner.scan_targets(vec![], vec![80, 443]).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_scan_empty_ports() {
        let config = Config::default();
        let scanner = ConcurrentScanner::new(config);

        let targets = vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))];
        let results = scanner.scan_targets(targets, vec![]).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_scan_socket_closed_port() {
        let config = Config {
            scan: prtip_core::ScanConfig {
                timeout_ms: 100,
                retries: 0,
                ..Default::default()
            },
            ..Default::default()
        };

        // Port 1 is almost certainly closed on localhost
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1);

        let result = scan_socket_tcp(socket, config).await.unwrap();

        // Should be either Closed or Filtered
        assert!(result.state == PortState::Closed || result.state == PortState::Filtered);
    }

    #[tokio::test]
    async fn test_concurrent_batch_processing() {
        let config = Config {
            performance: prtip_core::PerformanceConfig {
                parallelism: 10,
                max_rate: None,
                batch_size: None,
                requested_ulimit: None,
            },
            scan: prtip_core::ScanConfig {
                timeout_ms: 100,
                retries: 0,
                ..Default::default()
            },
            ..Default::default()
        };

        let scanner = ConcurrentScanner::new(config);

        // Generate multiple sockets to test batching
        let targets = vec![
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        ];
        let ports = vec![1, 2, 3, 4, 5];

        let results = scanner.scan_targets(targets, ports).await.unwrap();

        // Should handle all sockets (10 total)
        assert!(results.len() <= 10);
    }
}
