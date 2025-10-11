//! Scan Scheduler
//!
//! Orchestrates the complete scanning workflow by coordinating all scanner components:
//! - Host discovery
//! - Port scanning
//! - Rate limiting
//! - Result storage
//!
//! The scheduler manages the scan lifecycle from initialization through completion.

use crate::adaptive_parallelism::calculate_parallelism;
use crate::storage_backend::StorageBackend;
use crate::{
    BannerGrabber, DiscoveryEngine, DiscoveryMethod, LockFreeAggregator, RateLimiter,
    ScanProgressBar, ServiceDetector, TcpConnectScanner,
};
use prtip_core::{Config, PortRange, PortState, Result, ScanResult, ScanTarget, ServiceProbeDb};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Scan scheduler that orchestrates the scanning process
///
/// The scheduler coordinates between different scan components, manages
/// resource limits, and provides a high-level interface for executing scans.
///
/// # Examples
///
/// ```no_run
/// use prtip_scanner::{ScanScheduler, StorageBackend};
/// use prtip_core::{Config, ScanTarget};
/// use std::sync::Arc;
///
/// # async fn example() -> prtip_core::Result<()> {
/// let config = Config::default();
/// let storage_backend = Arc::new(StorageBackend::memory(10000));
/// let scheduler = ScanScheduler::new(config, storage_backend).await?;
///
/// let targets = vec![ScanTarget::parse("192.168.1.1")?];
/// let results = scheduler.execute_scan(targets).await?;
///
/// println!("Scan found {} results", results.len());
/// # Ok(())
/// # }
/// ```
pub struct ScanScheduler {
    config: Config,
    tcp_scanner: Arc<TcpConnectScanner>,
    discovery: Arc<DiscoveryEngine>,
    rate_limiter: Arc<RateLimiter>,
    storage_backend: Arc<StorageBackend>,
}

impl ScanScheduler {
    /// Create a new scan scheduler
    ///
    /// # Arguments
    ///
    /// * `config` - Scan configuration
    /// * `storage_backend` - Storage backend for results (Memory or AsyncDatabase)
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub async fn new(config: Config, storage_backend: Arc<StorageBackend>) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        let timeout = Duration::from_millis(config.scan.timeout_ms);

        // Create TCP scanner
        let tcp_scanner = Arc::new(TcpConnectScanner::new(timeout, config.scan.retries));

        // Create discovery engine
        // For Phase 1, we only support TCP SYN ping
        let discovery = Arc::new(DiscoveryEngine::new(timeout, DiscoveryMethod::TcpSyn));

        // Create rate limiter
        let rate_limiter = Arc::new(RateLimiter::new(config.performance.max_rate));

        Ok(Self {
            config,
            tcp_scanner,
            discovery,
            rate_limiter,
            storage_backend,
        })
    }

    /// Execute a complete scan
    ///
    /// This method performs the full scan workflow:
    /// 1. Creates a scan record in the database
    /// 2. Optionally performs host discovery
    /// 3. Scans ports on discovered hosts
    /// 4. Stores results in the database
    /// 5. Marks the scan as complete
    ///
    /// # Arguments
    ///
    /// * `targets` - Vector of scan targets
    ///
    /// # Returns
    ///
    /// Vector of all scan results.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use prtip_scanner::ScanScheduler;
    /// # use prtip_core::ScanTarget;
    /// # async fn example(scheduler: ScanScheduler) -> prtip_core::Result<()> {
    /// let targets = vec![
    ///     ScanTarget::parse("192.168.1.0/24")?,
    ///     ScanTarget::parse("10.0.0.1")?,
    /// ];
    ///
    /// let results = scheduler.execute_scan(targets).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_scan(&self, targets: Vec<ScanTarget>) -> Result<Vec<ScanResult>> {
        info!("Starting scan of {} targets", targets.len());

        let mut all_results = Vec::new();

        // Scan each target
        for target in targets {
            debug!("Scanning target: {:?}", target);

            match self.scan_target(&target).await {
                Ok(results) => {
                    // Store results via storage backend (non-blocking for async!)
                    self.storage_backend.add_results_batch(results.clone())?;
                    all_results.extend(results);
                }
                Err(e) => {
                    error!("Error scanning target {:?}: {}", target, e);
                    // Continue with other targets
                }
            }
        }

        // Flush all pending writes
        self.storage_backend.flush().await?;

        info!("Scan complete: {} results", all_results.len());
        Ok(all_results)
    }

    /// Scan a single target
    async fn scan_target(&self, target: &ScanTarget) -> Result<Vec<ScanResult>> {
        // Expand target into individual IPs
        let hosts = target.expand_hosts();
        debug!("Target expanded to {} hosts", hosts.len());

        // Parse port range from config
        // For Phase 1, we'll scan common ports if none specified
        let ports = self.get_scan_ports()?;
        debug!("Scanning {} ports", ports.len());

        // Create lock-free aggregator for this target
        let estimated_results = hosts.len().saturating_mul(ports.len()).saturating_mul(2);
        let max_buffer = estimated_results.min(100_000); // Cap at 100K per target
        let aggregator = Arc::new(LockFreeAggregator::new(max_buffer));

        // Calculate adaptive parallelism based on port count
        // parallelism == 0 means use adaptive, otherwise use configured value
        let user_override = if self.config.performance.parallelism > 0 {
            Some(self.config.performance.parallelism)
        } else {
            None
        };
        let parallelism = calculate_parallelism(
            ports.len(),
            user_override,
            self.config.performance.requested_ulimit,
        );

        for host in hosts {
            // Rate limiting
            self.rate_limiter.acquire().await?;

            // Perform TCP connect scan with adaptive parallelism
            match self
                .tcp_scanner
                .scan_ports(host, ports.clone(), parallelism)
                .await
            {
                Ok(results) => {
                    // Push results to lock-free aggregator (zero contention!)
                    for result in results {
                        // Try to push result
                        if let Err(e) = aggregator.push(result.clone()) {
                            warn!("Failed to push result to aggregator: {}", e);
                            // If queue is full, drain and store batch
                            let batch = aggregator.drain_batch(5000);
                            if !batch.is_empty() {
                                self.storage_backend.add_results_batch(batch)?;
                            }
                            // Retry push (result not moved because we cloned above)
                            aggregator.push(result)?;
                        }
                    }
                }
                Err(e) => {
                    warn!("Error scanning {}: {}", host, e);
                    // Continue with other hosts
                }
            }
        }

        // Drain all results from aggregator
        let all_results = aggregator.drain_all();

        Ok(all_results)
    }

    /// Execute scan with host discovery first
    ///
    /// Performs host discovery before port scanning to reduce scan time
    /// by avoiding probes to unreachable hosts.
    ///
    /// # Arguments
    ///
    /// * `targets` - Vector of scan targets
    ///
    /// # Returns
    ///
    /// Vector of scan results for live hosts only.
    pub async fn execute_scan_with_discovery(
        &self,
        targets: Vec<ScanTarget>,
    ) -> Result<Vec<ScanResult>> {
        info!("Starting scan with host discovery");

        // Expand all targets to individual IPs
        let mut all_ips = Vec::new();
        for target in &targets {
            all_ips.extend(target.expand_hosts());
        }

        info!("Discovering live hosts among {} addresses", all_ips.len());

        // Calculate parallelism for host discovery
        // parallelism == 0 means use adaptive, otherwise use configured value
        let user_override = if self.config.performance.parallelism > 0 {
            Some(self.config.performance.parallelism)
        } else {
            None
        };
        let discovery_parallelism = calculate_parallelism(
            all_ips.len(),
            user_override,
            self.config.performance.requested_ulimit,
        );

        // Discover live hosts
        let live_hosts = self
            .discovery
            .discover_hosts(all_ips.clone(), discovery_parallelism)
            .await?;

        info!("Found {} live hosts", live_hosts.len());

        if live_hosts.is_empty() {
            warn!("No live hosts found, skipping port scan");
            return Ok(Vec::new());
        }

        // Create new targets with only live hosts
        let live_targets: Vec<ScanTarget> = live_hosts
            .iter()
            .filter_map(|ip| ScanTarget::parse(&ip.to_string()).ok())
            .collect();

        // Execute normal scan on live hosts
        self.execute_scan(live_targets).await
    }

    /// Get ports to scan based on configuration
    ///
    /// For Phase 1, returns a default set of common ports.
    /// In later phases, this will parse port specifications from config.
    fn get_scan_ports(&self) -> Result<Vec<u16>> {
        // Default common ports for Phase 1
        // Later phases will parse from config
        Ok(vec![
            21, 22, 23, 25, 53, 80, 110, 111, 135, 139, 143, 443, 445, 993, 995, 1723, 3306, 3389,
            5432, 5900, 8080, 8443,
        ])
    }

    /// Scan specific ports
    ///
    /// Execute a scan targeting specific ports across all targets.
    ///
    /// # Arguments
    ///
    /// * `targets` - Vector of scan targets
    /// * `ports` - Port range to scan
    ///
    /// # Returns
    ///
    /// Vector of scan results.
    pub async fn execute_scan_ports(
        &self,
        targets: Vec<ScanTarget>,
        ports: &PortRange,
    ) -> Result<Vec<ScanResult>> {
        info!(
            "Starting port scan: {} targets, {} ports",
            targets.len(),
            ports.count()
        );

        let ports_vec: Vec<u16> = ports.iter().collect();

        // Calculate estimated hosts for progress bar and buffer sizing
        let estimated_hosts: usize = targets.iter().map(|t| t.expand_hosts().len()).sum();

        // Create progress bar for real-time feedback
        let total_ports = (estimated_hosts * ports_vec.len()) as u64;
        let progress = Arc::new(ScanProgressBar::new(
            total_ports,
            self.config.scan.progress,
        ));
        progress.set_message("Port scanning...");

        // Create lock-free aggregator to collect results without contention
        // Buffer size = estimated total results (hosts * ports) with 2x safety margin
        let estimated_results = estimated_hosts
            .saturating_mul(ports_vec.len())
            .saturating_mul(2);
        let max_buffer = estimated_results.min(1_000_000); // Cap at 1M results
        let aggregator = Arc::new(LockFreeAggregator::new(max_buffer));

        info!(
            "Using lock-free aggregator with buffer size: {} (estimated {} results)",
            max_buffer,
            estimated_results / 2
        );

        // Calculate adaptive parallelism based on port count
        // parallelism == 0 means use adaptive, otherwise use configured value
        let user_override = if self.config.performance.parallelism > 0 {
            Some(self.config.performance.parallelism)
        } else {
            None
        };
        let parallelism = calculate_parallelism(
            ports_vec.len(),
            user_override,
            self.config.performance.requested_ulimit,
        );

        // Capture total scan ports for adaptive polling interval calculation
        // (must be before the loop where total_ports gets shadowed)
        let total_scan_ports = total_ports;

        for target in targets {
            let hosts = target.expand_hosts();

            for host in hosts {
                self.rate_limiter.acquire().await?;

                // Create a progress tracker for this host's ports
                let host_progress = Arc::new(prtip_core::ScanProgress::new(ports_vec.len()));

                // Spawn a task to bridge host progress to main progress bar
                // This updates the progress bar in real-time as each port completes,
                // rather than jumping to 100% after all ports are done
                let progress_clone = Arc::clone(&progress);
                let host_progress_clone = Arc::clone(&host_progress);
                let total_ports = ports_vec.len();

                // Adaptive polling interval based on TOTAL SCAN PORTS (hosts × ports):
                // - Tiny scans (< 1K ports): 0.2ms - catches ultra-fast localhost scans
                // - Small scans (< 10K ports): 0.5ms - rapid updates for fast scans
                // - Medium scans (< 100K ports): 1ms - balance responsiveness and CPU
                // - Large scans (< 1M ports): 5ms - reduces overhead for network scans
                // - Huge scans (≥ 1M ports): 10ms - minimal overhead for massive scans
                //
                // This prevents catastrophic polling overhead on large scans:
                // Example: 256 hosts × 10K ports = 2.56M total
                //   - Old (1ms): 7.2M polls over 2 hours = 2,160s overhead (30%!)
                //   - New (10ms): 720K polls = 216s overhead (3%, acceptable)
                let poll_interval = if total_scan_ports < 1_000 {
                    Duration::from_micros(200)   // 0.2ms - tiny scans (1 host × 1K ports)
                } else if total_scan_ports < 10_000 {
                    Duration::from_micros(500)   // 0.5ms - small scans (1 host × 10K ports)
                } else if total_scan_ports < 100_000 {
                    Duration::from_millis(1)     // 1ms - medium scans (10 hosts × 10K ports)
                } else if total_scan_ports < 1_000_000 {
                    Duration::from_millis(5)     // 5ms - large scans (100 hosts × 10K ports)
                } else {
                    Duration::from_millis(10)    // 10ms - huge scans (256+ hosts × 10K ports)
                };

                let bridge_handle = tokio::spawn(async move {
                    let mut last_completed = 0;
                    loop {
                        tokio::time::sleep(poll_interval).await;
                        let current_completed = host_progress_clone.completed();
                        if current_completed > last_completed {
                            let delta = current_completed - last_completed;
                            progress_clone.inc(delta as u64);
                            last_completed = current_completed;
                        }
                        if current_completed >= total_ports {
                            break;
                        }
                    }
                });

                match self
                    .tcp_scanner
                    .scan_ports_with_progress(host, ports_vec.clone(), parallelism, Some(&host_progress))
                    .await
                {
                    Ok(results) => {
                        // Wait for progress bridge to finish
                        let _ = bridge_handle.await;

                        // Push results to lock-free aggregator (zero contention!)
                        for result in results {
                            // Try to push result
                            if let Err(e) = aggregator.push(result.clone()) {
                                warn!("Failed to push result to aggregator: {}", e);
                                // If queue is full, drain a batch to storage immediately
                                let batch = aggregator.drain_batch(10000);
                                if !batch.is_empty() {
                                    self.storage_backend.add_results_batch(batch)?;
                                }
                                // Retry push (result not moved because we cloned above)
                                aggregator.push(result)?;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Error scanning {}: {}", host, e);
                        // Cancel bridge task
                        bridge_handle.abort();
                    }
                }
            }
        }

        // Drain all results from aggregator
        let mut all_results = aggregator.drain_all();
        info!(
            "Collected {} results from lock-free aggregator",
            all_results.len()
        );

        // Perform service detection if enabled
        if self.config.scan.service_detection.enabled {
            progress.set_message("Service detection...");
            info!("Starting service detection on open ports");
            let open_count = all_results
                .iter()
                .filter(|r| r.state == PortState::Open)
                .count();

            if open_count > 0 {
                // Create service detector and banner grabber
                let probe_db = if let Some(path) = &self.config.scan.service_detection.probe_db_path {
                    ServiceProbeDb::load_from_file(path)?
                } else {
                    ServiceProbeDb::default()
                };
                let detector =
                    ServiceDetector::new(probe_db, self.config.scan.service_detection.intensity);
                let grabber = BannerGrabber::new();

                debug!("Detecting services on {} open ports", open_count);

                // Process each open port
                for result in all_results.iter_mut() {
                    if result.state == PortState::Open {
                        let target = SocketAddr::new(result.target_ip, result.port);

                        // Try service detection first
                        match detector.detect_service(target).await {
                            Ok(service_info) => {
                                if service_info.service != "unknown" {
                                    result.service = Some(service_info.service.clone());

                                    // Combine product and version
                                    if let Some(product) = service_info.product {
                                        if let Some(version) = service_info.version {
                                            result.version =
                                                Some(format!("{} {}", product, version));
                                        } else {
                                            result.version = Some(product);
                                        }
                                    } else if let Some(version) = service_info.version {
                                        result.version = Some(version);
                                    }

                                    debug!(
                                        "Detected service on {}:{}: {} {}",
                                        result.target_ip,
                                        result.port,
                                        result.service.as_ref().unwrap_or(&"unknown".to_string()),
                                        result.version.as_ref().unwrap_or(&"".to_string())
                                    );
                                    continue;
                                }
                            }
                            Err(e) => {
                                debug!(
                                    "Service detection failed for {}:{}: {}",
                                    result.target_ip, result.port, e
                                );
                            }
                        }

                        // If service detection failed and banner grabbing is enabled, try that
                        if self.config.scan.service_detection.banner_grab {
                            match grabber.grab_banner(target).await {
                                Ok(banner) => {
                                    if !banner.is_empty() {
                                        result.banner = Some(banner);
                                        debug!(
                                            "Grabbed banner from {}:{}",
                                            result.target_ip, result.port
                                        );
                                    }
                                }
                                Err(e) => {
                                    debug!(
                                        "Banner grab failed for {}:{}: {}",
                                        result.target_ip, result.port, e
                                    );
                                }
                            }
                        }
                    }
                }

                let detected = all_results
                    .iter()
                    .filter(|r| r.service.is_some() || r.banner.is_some())
                    .count();
                info!(
                    "Service detection complete: {}/{} services identified",
                    detected, open_count
                );
            }
        }

        // Store all results via storage backend (non-blocking for async!)
        if !all_results.is_empty() {
            self.storage_backend
                .add_results_batch(all_results.clone())?;
        }

        // Flush all pending writes
        self.storage_backend.flush().await?;

        // Complete progress bar
        progress.finish("Scan complete");

        info!("Port scan complete: {} results", all_results.len());
        Ok(all_results)
    }

    /// Get configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prtip_core::{
        NetworkConfig, OutputConfig, OutputFormat, PerformanceConfig, ScanConfig, ScanType,
        TimingTemplate,
    };

    async fn create_test_config() -> Config {
        Config {
            scan: ScanConfig {
                scan_type: ScanType::Connect,
                timing_template: TimingTemplate::Normal,
                timeout_ms: 1000,
                retries: 0,
                scan_delay_ms: 0,
                service_detection: Default::default(),
                progress: false,
            },
            network: NetworkConfig {
                interface: None,
                source_port: None,
            },
            output: OutputConfig {
                format: OutputFormat::Json,
                file: None,
                verbose: 0,
            },
            performance: PerformanceConfig {
                max_rate: Some(100),
                parallelism: 10,
                batch_size: None,
                requested_ulimit: None,
            },
        }
    }

    #[tokio::test]
    async fn test_create_scheduler() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(1000));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        assert!(scheduler.config().performance.max_rate.is_some());
    }

    #[tokio::test]
    async fn test_create_scheduler_memory_backend() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(10000));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        assert!(scheduler.config().performance.max_rate.is_some());
    }

    #[tokio::test]
    async fn test_execute_scan_empty() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(100));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        let results = scheduler.execute_scan(vec![]).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_scan_localhost() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(1000));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        let results = scheduler.execute_scan(targets).await.unwrap();

        // Should have some results (even if all filtered/closed)
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_execute_scan_localhost_memory_backend() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(10000));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        let results = scheduler.execute_scan(targets).await.unwrap();

        // Should have some results (even if all filtered/closed)
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_execute_scan_ports() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(100));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        let ports = PortRange::parse("9999").unwrap();

        let results = scheduler.execute_scan_ports(targets, &ports).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].port, 9999);
    }

    #[tokio::test]
    async fn test_scan_target_single_host() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(1000));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        let target = ScanTarget::parse("127.0.0.1").unwrap();
        let results = scheduler.scan_target(&target).await.unwrap();

        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_config() {
        let mut config = create_test_config().await;
        config.scan.timeout_ms = 0; // Invalid

        let storage_backend = Arc::new(StorageBackend::memory(100));
        let result = ScanScheduler::new(config, storage_backend).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_scan_ports() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(100));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        let ports = scheduler.get_scan_ports().unwrap();

        // Should return common ports
        assert!(!ports.is_empty());
        assert!(ports.contains(&80));
        assert!(ports.contains(&443));
        assert!(ports.contains(&22));
    }

    #[tokio::test]
    async fn test_execute_scan_with_discovery() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(1000));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        let results = scheduler
            .execute_scan_with_discovery(targets)
            .await
            .unwrap();

        // Localhost should be discovered and scanned
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_scan_nonexistent_host() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(1000));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        // TEST-NET-1 (should be unreachable)
        let targets = vec![ScanTarget::parse("192.0.2.1").unwrap()];
        let results = scheduler.execute_scan(targets).await.unwrap();

        // May have results (filtered ports), just verify no panic
        let _ = results;
    }

    #[tokio::test]
    async fn test_multiple_targets() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(5000));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        let targets = vec![
            ScanTarget::parse("127.0.0.1").unwrap(),
            ScanTarget::parse("127.0.0.2").unwrap(),
        ];

        let results = scheduler.execute_scan(targets).await.unwrap();

        // Should have results for both targets
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_results_stored_in_memory_backend() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(5000));

        let scheduler = ScanScheduler::new(config, Arc::clone(&storage_backend))
            .await
            .unwrap();

        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        let results = scheduler.execute_scan(targets).await.unwrap();

        // Check that results were stored
        let stored_results = storage_backend.get_results().await.unwrap();
        assert_eq!(stored_results.len(), results.len());
        assert!(!stored_results.is_empty());
    }
}
