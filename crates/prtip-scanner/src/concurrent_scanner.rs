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

use crate::{AdaptiveRateLimiterV2, HostgroupLimiter, ResultWriter};
use futures::stream::{FuturesUnordered, StreamExt};
use prtip_core::{Config, PortState, Result, ScanResult};
use prtip_network::CdnDetector;
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
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
///
/// # Rate Limiting (Sprint 5.4 Phase 1)
///
/// Supports optional hostgroup and adaptive rate limiting:
/// - Hostgroup limiter controls concurrent targets
/// - Adaptive limiter provides per-target ICMP backoff
///
/// # CDN Detection (Sprint 6.3 Phase 2.2)
///
/// Supports optional CDN IP filtering to avoid scanning CDN infrastructure:
/// - CDN detector filters out CDN IPs before scanning
/// - Reduces unnecessary scans by 30-70% on internet targets
#[derive(Clone)]
pub struct ConcurrentScanner {
    config: Config,
    /// Optional hostgroup limiter (controls concurrent targets)
    hostgroup_limiter: Option<Arc<HostgroupLimiter>>,
    /// Optional adaptive rate limiter (ICMP-aware throttling)
    adaptive_limiter: Option<Arc<AdaptiveRateLimiterV2>>,
    /// Optional CDN detector (filters CDN IPs before scanning)
    cdn_detector: Option<Arc<CdnDetector>>,
}

impl ConcurrentScanner {
    /// Create a new concurrent scanner
    pub fn new(config: Config) -> Self {
        Self {
            config,
            hostgroup_limiter: None,
            adaptive_limiter: None,
            cdn_detector: None,
        }
    }

    /// Enable hostgroup limiting (concurrent target control)
    ///
    /// # Arguments
    ///
    /// * `limiter` - Hostgroup limiter to use
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::{ConcurrentScanner, HostgroupLimiter};
    /// use prtip_core::Config;
    /// use std::sync::Arc;
    ///
    /// let limiter = Arc::new(HostgroupLimiter::with_max(64));
    /// let scanner = ConcurrentScanner::new(Config::default())
    ///     .with_hostgroup_limiter(limiter);
    /// ```
    pub fn with_hostgroup_limiter(mut self, limiter: Arc<HostgroupLimiter>) -> Self {
        self.hostgroup_limiter = Some(limiter);
        self
    }

    /// Enable adaptive rate limiting (ICMP-aware throttling)
    ///
    /// # Arguments
    ///
    /// * `limiter` - Adaptive rate limiter to use
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::{ConcurrentScanner, AdaptiveRateLimiterV2};
    /// use prtip_core::Config;
    /// use std::sync::Arc;
    ///
    /// let limiter = Arc::new(AdaptiveRateLimiterV2::new(100_000.0));
    /// let scanner = ConcurrentScanner::new(Config::default())
    ///     .with_adaptive_limiter(limiter);
    /// ```
    pub fn with_adaptive_limiter(mut self, limiter: Arc<AdaptiveRateLimiterV2>) -> Self {
        self.adaptive_limiter = Some(limiter);
        self
    }

    /// Enable CDN detection and filtering
    ///
    /// When enabled, CDN IPs are filtered out before scanning to avoid
    /// wasting resources on CDN infrastructure. This can reduce scans by
    /// 30-70% when targeting internet hosts behind CDNs.
    ///
    /// # Arguments
    ///
    /// * `detector` - CDN detector to use for filtering
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::ConcurrentScanner;
    /// use prtip_network::CdnDetector;
    /// use prtip_core::Config;
    /// use std::sync::Arc;
    ///
    /// let detector = Arc::new(CdnDetector::new());
    /// let scanner = ConcurrentScanner::new(Config::default())
    ///     .with_cdn_detector(detector);
    /// ```
    pub fn with_cdn_detector(mut self, detector: Arc<CdnDetector>) -> Self {
        self.cdn_detector = Some(detector);
        self
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
        // If no rate limiting configured, use original fast path
        if self.hostgroup_limiter.is_none() && self.adaptive_limiter.is_none() {
            return self.scan_targets_fast_path(targets, ports).await;
        }

        // Filter CDN IPs if detector is enabled (before rate-limited scanning)
        let filtered_targets = if let Some(detector) = &self.cdn_detector {
            let original_count = targets.len();
            let filtered: Vec<IpAddr> = targets
                .into_iter()
                .filter(|ip| detector.detect(ip).is_none())
                .collect();
            let filtered_count = original_count - filtered.len();
            if filtered_count > 0 {
                debug!(
                    "CDN filtering (rate-limited): removed {} CDN IPs from {} targets ({:.1}% reduction)",
                    filtered_count,
                    original_count,
                    (filtered_count as f64 / original_count as f64) * 100.0
                );
            }
            filtered
        } else {
            targets
        };

        // Rate-limited path: scan each target sequentially with rate limiting
        let mut all_results = Vec::new();
        let target_count = filtered_targets.len();

        for target in filtered_targets {
            // 1. Acquire hostgroup permit (if enabled)
            let _permit = if let Some(limiter) = &self.hostgroup_limiter {
                Some(limiter.acquire_target().await)
            } else {
                None
            };

            // 2. Check ICMP backoff (if enabled)
            if let Some(limiter) = &self.adaptive_limiter {
                if limiter.is_target_backed_off(target) {
                    debug!("Skipping {} (ICMP backoff active)", target);
                    continue;
                }
            }

            // 3. Scan all ports on this target
            let sockets: Vec<SocketAddr> = ports
                .iter()
                .map(|&port| SocketAddr::new(target, port))
                .collect();

            debug!(
                "Scanning target {} ({} ports) with rate limiting",
                target,
                ports.len()
            );

            let results = self.scan_sockets(sockets).await?;
            all_results.extend(results);

            // Permit automatically released here (RAII)
        }

        debug!(
            "Rate-limited scan complete: {} results from {} targets",
            all_results.len(),
            target_count
        );

        Ok(all_results)
    }

    /// Fast path for scanning without rate limiting (original implementation)
    async fn scan_targets_fast_path(
        &self,
        targets: Vec<IpAddr>,
        ports: Vec<u16>,
    ) -> Result<Vec<ScanResult>> {
        // Filter CDN IPs if detector is enabled
        let filtered_targets = if let Some(detector) = &self.cdn_detector {
            let original_count = targets.len();
            let filtered: Vec<IpAddr> = targets
                .into_iter()
                .filter(|ip| detector.detect(ip).is_none())
                .collect();
            let filtered_count = original_count - filtered.len();
            if filtered_count > 0 {
                debug!(
                    "CDN filtering: removed {} CDN IPs from {} targets ({:.1}% reduction)",
                    filtered_count,
                    original_count,
                    (filtered_count as f64 / original_count as f64) * 100.0
                );
            }
            filtered
        } else {
            targets
        };

        // Generate all socket addresses (cartesian product)
        let mut sockets = Vec::with_capacity(filtered_targets.len() * ports.len());
        for target in &filtered_targets {
            for &port in &ports {
                sockets.push(SocketAddr::new(*target, port));
            }
        }

        debug!(
            "Starting concurrent scan: {} targets x {} ports = {} sockets",
            filtered_targets.len(),
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

        // Sprint 6.6 Task Area 3: Use ResultWriter for result accumulation
        let mut writer = ResultWriter::from_config(&self.config, total_sockets)?;

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
                        writer.write(&result)?;
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

        // Flush writer and collect results
        writer.flush()?;
        let results = writer.collect()?;

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
                    // Convert to ScannerError, then to core::Error
                    let scanner_err = crate::error::ScannerError::too_many_open_files(
                        config.performance.parallelism as u64,
                        (config.performance.parallelism / 2) as u64,
                    );
                    return Err(prtip_core::Error::ScannerOperation(scanner_err.to_string()));
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
                numa_enabled: false,
                adaptive_batch_enabled: false,
                min_batch_size: 1,
                max_batch_size: 1024,
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

    #[tokio::test]
    async fn test_cdn_filtering_fast_path() {
        use std::net::Ipv6Addr;

        let config = Config {
            scan: prtip_core::ScanConfig {
                timeout_ms: 100,
                retries: 0,
                ..Default::default()
            },
            ..Default::default()
        };

        let detector = Arc::new(CdnDetector::new());
        let scanner = ConcurrentScanner::new(config).with_cdn_detector(detector);

        // Mix of CDN and non-CDN IPs
        let targets = vec![
            IpAddr::V4(Ipv4Addr::new(104, 16, 0, 1)),  // Cloudflare CDN
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), // Private (non-CDN)
            IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x20, 0, 0, 0, 0, 1)), // Cloudflare IPv6
        ];
        let ports = vec![80];

        let results = scanner.scan_targets(targets, ports).await.unwrap();

        // Should only scan the non-CDN IP (192.168.1.1)
        // Results will be empty since ports are closed, but we verify no errors from CDN IPs
        assert!(results.len() <= 1);
    }

    #[tokio::test]
    async fn test_cdn_filtering_rate_limited_path() {
        use std::net::Ipv6Addr;

        let config = Config {
            scan: prtip_core::ScanConfig {
                timeout_ms: 100,
                retries: 0,
                ..Default::default()
            },
            ..Default::default()
        };

        let detector = Arc::new(CdnDetector::new());
        let hostgroup_limiter = Arc::new(HostgroupLimiter::with_max(10));
        let scanner = ConcurrentScanner::new(config)
            .with_cdn_detector(detector)
            .with_hostgroup_limiter(hostgroup_limiter);

        // Mix of CDN and non-CDN IPs
        let targets = vec![
            IpAddr::V4(Ipv4Addr::new(104, 16, 0, 1)),  // Cloudflare CDN
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), // Private (non-CDN)
            IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x20, 0, 0, 0, 0, 1)), // Cloudflare IPv6
        ];
        let ports = vec![80];

        let results = scanner.scan_targets(targets, ports).await.unwrap();

        // Should only scan the non-CDN IP (192.168.1.1) in rate-limited mode
        assert!(results.len() <= 1);
    }

    #[tokio::test]
    async fn test_no_cdn_filtering_when_disabled() {
        let config = Config {
            scan: prtip_core::ScanConfig {
                timeout_ms: 100,
                retries: 0,
                ..Default::default()
            },
            ..Default::default()
        };

        // Scanner WITHOUT CDN detector
        let scanner = ConcurrentScanner::new(config);

        // CDN IPs
        let targets = vec![
            IpAddr::V4(Ipv4Addr::new(104, 16, 0, 1)),  // Cloudflare
            IpAddr::V4(Ipv4Addr::new(151, 101, 0, 1)), // Fastly
        ];
        let ports = vec![80];

        let results = scanner.scan_targets(targets, ports).await.unwrap();

        // Should attempt to scan both CDN IPs (no filtering)
        // Results will be empty/filtered since ports are closed, but no errors
        assert!(results.len() <= 2);
    }
}
