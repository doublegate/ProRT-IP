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
    AdaptiveRateLimiterV3, BannerGrabber, DiscoveryEngine, DiscoveryMethod, LockFreeAggregator,
    ResultWriter, ScanProgressBar, ServiceDetector, TcpConnectScanner, UdpScanner,
};
use prtip_core::event_bus::EventBus;
use prtip_core::events::{ScanEvent, ScanStage, Throughput};
use prtip_core::{
    Config, PortRange, PortState, Result, ScanResult, ScanTarget, ScanType, ServiceProbeDb,
};
use prtip_network::{CdnDetector, CdnProvider};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[cfg(feature = "numa")]
use prtip_network::numa::{NumaManager, NumaTopology};

/// Helper for tracking and publishing scan progress to TUI
struct ProgressTracker {
    scan_id: Uuid,
    total: u64,
    completed: u64,
    last_publish: Instant,
    start_time: Instant,
    publish_interval: Duration,
}

impl ProgressTracker {
    /// Create a new progress tracker
    fn new(scan_id: Uuid, total: u64) -> Self {
        Self {
            scan_id,
            total,
            completed: 0,
            last_publish: Instant::now(),
            start_time: Instant::now(),
            publish_interval: Duration::from_millis(250), // Publish every 250ms
        }
    }

    /// Increment completed count and publish if interval elapsed
    async fn increment(&mut self, event_bus: &Option<Arc<EventBus>>) {
        self.completed += 1;

        // Publish at regular intervals or on completion
        if self.last_publish.elapsed() >= self.publish_interval || self.completed >= self.total {
            self.publish(event_bus).await;
            self.last_publish = Instant::now();
        }
    }

    /// Publish a ProgressUpdate event
    async fn publish(&self, event_bus: &Option<Arc<EventBus>>) {
        if let Some(ref bus) = event_bus {
            let percentage = if self.total > 0 {
                (self.completed as f32 / self.total as f32) * 100.0
            } else {
                0.0
            };

            // Calculate ETA based on current progress rate
            let elapsed = self.start_time.elapsed();
            let eta = if self.completed > 0 && self.completed < self.total {
                let rate = self.completed as f64 / elapsed.as_secs_f64();
                let remaining = (self.total - self.completed) as f64;
                if rate > 0.0 {
                    Some(Duration::from_secs_f64(remaining / rate))
                } else {
                    None
                }
            } else {
                None
            };

            // Calculate throughput
            let pps = if elapsed.as_secs_f64() > 0.0 {
                self.completed as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };

            bus.publish(ScanEvent::ProgressUpdate {
                scan_id: self.scan_id,
                stage: ScanStage::ScanningPorts,
                percentage,
                completed: self.completed,
                total: self.total,
                throughput: Throughput {
                    packets_per_second: pps,
                    hosts_per_minute: pps * 60.0 / 1000.0, // Rough estimate
                    bandwidth_mbps: 0.0,                   // Not tracked at this level
                },
                eta,
                timestamp: SystemTime::now(),
            })
            .await;
        }
    }
}

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
/// let results = scheduler.execute_scan(targets, None).await?;
///
/// println!("Scan found {} results", results.len());
/// # Ok(())
/// # }
/// ```
pub struct ScanScheduler {
    config: Config,
    tcp_scanner: Arc<TcpConnectScanner>,
    discovery: Arc<DiscoveryEngine>,
    rate_limiter: Option<Arc<AdaptiveRateLimiterV3>>,
    storage_backend: Arc<StorageBackend>,
    cdn_detector: Option<CdnDetector>,
    #[cfg(feature = "numa")]
    numa_manager: Option<Arc<NumaManager>>,
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

        // Create TCP scanner with EventBus attached (if available)
        let mut tcp_scanner = TcpConnectScanner::new(timeout, config.scan.retries);
        if let Some(ref event_bus) = config.scan.event_bus {
            tcp_scanner = tcp_scanner.with_event_bus(event_bus.clone());
            debug!("Attached EventBus to TCP scanner for real-time PortFound events");
        }
        let tcp_scanner = Arc::new(tcp_scanner);

        // Create discovery engine
        // For Phase 1, we only support TCP SYN ping
        let discovery = Arc::new(DiscoveryEngine::new(timeout, DiscoveryMethod::TcpSyn));

        // Create rate limiter (V3 is now the default and only rate limiter)
        let rate_limiter = config.performance.max_rate.map(|rate| {
            // Convert u32 to Option<u64> for V3
            let rate_u64 = Some(rate as u64);
            AdaptiveRateLimiterV3::new(rate_u64)
        });

        // Create CDN detector if skip_cdn is enabled
        let cdn_detector = if config.network.skip_cdn {
            // Helper function to parse provider names
            fn parse_provider(name: &str) -> Option<CdnProvider> {
                match name.to_lowercase().as_str() {
                    "cloudflare" | "cf" => Some(CdnProvider::Cloudflare),
                    "aws" | "cloudfront" | "amazon" => Some(CdnProvider::AwsCloudFront),
                    "azure" | "microsoft" | "ms" => Some(CdnProvider::AzureCdn),
                    "akamai" => Some(CdnProvider::Akamai),
                    "fastly" => Some(CdnProvider::Fastly),
                    "google" | "gcp" | "gcloud" => Some(CdnProvider::GoogleCloud),
                    _ => None,
                }
            }

            let detector = if let Some(ref whitelist) = config.network.cdn_whitelist {
                // Parse whitelist providers
                let providers: Vec<CdnProvider> = whitelist
                    .iter()
                    .filter_map(|name| parse_provider(name))
                    .collect();
                info!("CDN detection enabled with whitelist: {:?}", providers);
                CdnDetector::with_whitelist(providers)
            } else if let Some(ref blacklist) = config.network.cdn_blacklist {
                // Parse blacklist providers
                let providers: Vec<CdnProvider> = blacklist
                    .iter()
                    .filter_map(|name| parse_provider(name))
                    .collect();
                info!("CDN detection enabled with blacklist: {:?}", providers);
                CdnDetector::with_blacklist(providers)
            } else {
                // Default: skip all CDN providers
                info!("CDN detection enabled (all providers)");
                CdnDetector::new()
            };
            Some(detector)
        } else {
            None
        };

        // Initialize NUMA manager if enabled
        #[cfg(feature = "numa")]
        let numa_manager = if config.performance.numa_enabled {
            match NumaTopology::detect() {
                Ok(topo) if topo.is_multi_node() => {
                    match NumaManager::new(topo) {
                        Ok(manager) => {
                            info!("NUMA optimization enabled ({} nodes)", manager.node_count());

                            // Pin main scheduler thread (TX thread) to core on NUMA node 0
                            // This reduces latency for packet transmission by placing the thread
                            // near the NIC (typically on node 0)
                            match manager.allocate_core(Some(0)) {
                                Ok(tx_core) => {
                                    if let Err(e) = manager.pin_current_thread(tx_core) {
                                        warn!("Failed to pin scheduler thread: {}", e);
                                    } else {
                                        info!(
                                            "Scheduler thread pinned to core {} (node 0)",
                                            tx_core
                                        );
                                    }
                                }
                                Err(e) => warn!("Failed to allocate core for scheduler: {}", e),
                            }

                            Some(Arc::new(manager))
                        }
                        Err(e) => {
                            warn!(
                                "NUMA initialization failed: {}, falling back to non-NUMA mode",
                                e
                            );
                            None
                        }
                    }
                }
                Ok(_) => {
                    info!("Single-node system detected, NUMA optimization disabled");
                    None
                }
                Err(e) => {
                    warn!(
                        "NUMA detection failed: {}, falling back to non-NUMA mode",
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            config,
            tcp_scanner,
            discovery,
            rate_limiter,
            storage_backend,
            cdn_detector,
            #[cfg(feature = "numa")]
            numa_manager,
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
    /// let results = scheduler.execute_scan(targets, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_scan(
        &self,
        targets: Vec<ScanTarget>,
        pcapng_writer: Option<Arc<std::sync::Mutex<crate::pcapng::PcapngWriter>>>,
    ) -> Result<Vec<ScanResult>> {
        info!("Starting scan of {} targets", targets.len());

        let mut all_results = Vec::new();

        // Scan each target
        for target in targets {
            debug!("Scanning target: {:?}", target);

            match self.scan_target(&target, pcapng_writer.clone()).await {
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
    async fn scan_target(
        &self,
        target: &ScanTarget,
        pcapng_writer: Option<Arc<std::sync::Mutex<crate::pcapng::PcapngWriter>>>,
    ) -> Result<Vec<ScanResult>> {
        // Expand target into individual IPs
        let original_hosts = target.expand_hosts();
        debug!("Target expanded to {} hosts", original_hosts.len());

        // Filter CDN IPs if enabled
        let hosts = if let Some(ref detector) = self.cdn_detector {
            let mut filtered = Vec::new();
            let mut skipped = 0;
            let mut provider_counts: std::collections::HashMap<CdnProvider, usize> =
                std::collections::HashMap::new();

            for host in original_hosts {
                if let Some(provider) = detector.detect(&host) {
                    // Track statistics
                    *provider_counts.entry(provider).or_insert(0) += 1;
                    skipped += 1;
                    debug!("Skipping CDN IP {}: {:?}", host, provider);
                } else {
                    filtered.push(host);
                }
            }

            // Log statistics
            if skipped > 0 {
                let total = filtered.len() + skipped;
                let reduction_pct = (skipped * 100) / total;
                info!(
                    "Filtered {} CDN IPs ({}% reduction): {:?}",
                    skipped, reduction_pct, provider_counts
                );
            }

            // Check if all hosts were filtered
            if filtered.is_empty() {
                debug!("All hosts filtered (CDN detection), skipping scan");
                return Ok(Vec::new());
            }

            debug!("Scanning {} hosts after CDN filtering", filtered.len());
            filtered
        } else {
            original_hosts
        };

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

        // Create progress tracker for TUI updates
        let total_work = (hosts.len() * ports.len()) as u64;
        let scan_id = Uuid::new_v4();
        let mut progress_tracker = ProgressTracker::new(scan_id, total_work);

        for host in hosts {
            // Rate limiting (V3 is now the default and only rate limiter)
            if let Some(ref limiter) = self.rate_limiter {
                limiter.acquire().await?;
            }

            // Perform scan based on configured scan type
            let scan_result = match self.config.scan.scan_type {
                ScanType::Connect => {
                    // TCP Connect scan (already exists, no PCAPNG support yet - TASK-2)
                    let results = self
                        .tcp_scanner
                        .scan_ports(host, ports.clone(), parallelism)
                        .await;

                    // Update progress tracker for TUI (per port scanned)
                    for _ in 0..ports.len() {
                        progress_tracker
                            .increment(&self.config.scan.event_bus)
                            .await;
                    }

                    results
                }
                ScanType::Syn => {
                    // SYN scan (has PCAPNG support now!)
                    use crate::SynScanner;
                    let mut syn_scanner = SynScanner::new(self.config.clone())?;
                    syn_scanner.initialize().await?;

                    // Scan each port individually (SYN scanner scans one port at a time)
                    // Sprint 5.1: Now supports both IPv4 and IPv6
                    // Sprint 6.6 Task Area 3: Use ResultWriter for result accumulation
                    let mut writer = ResultWriter::from_config(&self.config, ports.len())?;
                    for port in &ports {
                        match syn_scanner
                            .scan_port_with_pcapng(
                                host, // Pass IpAddr directly (Sprint 5.1: dual-stack support)
                                *port,
                                pcapng_writer.clone(),
                            )
                            .await
                        {
                            Ok(result) => writer.write(&result)?,
                            Err(e) => warn!("Error scanning SYN {}:{}: {}", host, port, e),
                        }
                        // Update progress tracker for TUI
                        progress_tracker
                            .increment(&self.config.scan.event_bus)
                            .await;
                    }
                    writer.flush()?;
                    writer.collect()
                }
                ScanType::Udp => {
                    // UDP scan (has PCAPNG support already!)
                    let mut udp_scanner = UdpScanner::new(self.config.clone())?;
                    udp_scanner.initialize().await?;

                    // Scan each port individually (Sprint 5.1: UDP now supports dual-stack IPv4/IPv6)
                    // Sprint 6.6 Task Area 3: Use ResultWriter for result accumulation
                    let mut writer = ResultWriter::from_config(&self.config, ports.len())?;
                    for port in &ports {
                        match udp_scanner
                            .scan_port_with_pcapng(
                                host, // Pass IpAddr directly (dual-stack support)
                                *port,
                                pcapng_writer.clone(),
                            )
                            .await
                        {
                            Ok(result) => writer.write(&result)?,
                            Err(e) => warn!("Error scanning UDP {}:{}: {}", host, port, e),
                        }
                        // Update progress tracker for TUI
                        progress_tracker
                            .increment(&self.config.scan.event_bus)
                            .await;
                    }
                    writer.flush()?;
                    writer.collect()
                }
                ScanType::Fin | ScanType::Null | ScanType::Xmas | ScanType::Ack => {
                    // Stealth scans (have PCAPNG support now!)
                    use crate::{StealthScanType, StealthScanner};

                    // Determine stealth scan type
                    let stealth_type = match self.config.scan.scan_type {
                        ScanType::Fin => StealthScanType::Fin,
                        ScanType::Null => StealthScanType::Null,
                        ScanType::Xmas => StealthScanType::Xmas,
                        ScanType::Ack => StealthScanType::Ack,
                        _ => unreachable!("Already matched stealth scan type"),
                    };

                    let mut stealth_scanner = StealthScanner::new(self.config.clone())?;
                    stealth_scanner.initialize().await?;

                    // Scan each port individually
                    // Sprint 5.1 Phase 2.2: Stealth scanner now supports IPv6
                    // Sprint 6.6 Task Area 3: Use ResultWriter for result accumulation
                    let mut writer = ResultWriter::from_config(&self.config, ports.len())?;
                    for port in &ports {
                        match stealth_scanner
                            .scan_port_with_pcapng(
                                host, // Pass IpAddr directly - dual-stack support
                                *port,
                                stealth_type,
                                pcapng_writer.clone(),
                            )
                            .await
                        {
                            Ok(result) => writer.write(&result)?,
                            Err(e) => warn!(
                                "Error scanning {} {}:{}: {}",
                                stealth_type.name(),
                                host,
                                port,
                                e
                            ),
                        }
                        // Update progress tracker for TUI
                        progress_tracker
                            .increment(&self.config.scan.event_bus)
                            .await;
                    }
                    writer.flush()?;
                    writer.collect()
                }
                ScanType::Idle => {
                    // Idle scan (Phase 5 feature)
                    return Err(prtip_core::Error::Config(
                        "Idle scan not yet implemented".to_string(),
                    ));
                }
            };

            match scan_result {
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
    /// * `pcapng_writer` - Optional PCAPNG writer for packet capture
    ///
    /// # Returns
    ///
    /// Vector of scan results for live hosts only.
    pub async fn execute_scan_with_discovery(
        &self,
        targets: Vec<ScanTarget>,
        pcapng_writer: Option<Arc<std::sync::Mutex<crate::pcapng::PcapngWriter>>>,
    ) -> Result<Vec<ScanResult>> {
        info!("Starting scan with host discovery");

        // Publish ScanStarted event for TUI
        if let Some(ref event_bus) = self.config.scan.event_bus {
            use prtip_core::events::{ScanEvent, ScanStage};
            use std::time::SystemTime;
            use uuid::Uuid;

            let scan_id = Uuid::new_v4();
            let timestamp = SystemTime::now();

            event_bus
                .publish(ScanEvent::ScanStarted {
                    scan_id,
                    scan_type: self.config.scan.scan_type,
                    target_count: targets.len(),
                    port_count: 0, // Will be determined after discovery
                    timestamp,
                })
                .await;

            // Transition to DiscoveringHosts stage
            event_bus
                .publish(ScanEvent::StageChanged {
                    scan_id,
                    timestamp,
                    from_stage: ScanStage::Initializing,
                    to_stage: ScanStage::DiscoveringHosts,
                })
                .await;

            debug!("Published ScanStarted and StageChanged (DiscoveringHosts) events");
        }

        // Expand all targets to individual IPs
        let mut original_ips = Vec::new();
        for target in &targets {
            original_ips.extend(target.expand_hosts());
        }

        // Filter CDN IPs if enabled
        let all_ips = if let Some(ref detector) = self.cdn_detector {
            let mut filtered = Vec::new();
            let mut skipped = 0;
            let mut provider_counts: std::collections::HashMap<CdnProvider, usize> =
                std::collections::HashMap::new();

            for ip in original_ips {
                if let Some(provider) = detector.detect(&ip) {
                    // Track statistics
                    *provider_counts.entry(provider).or_insert(0) += 1;
                    skipped += 1;
                    debug!("Skipping CDN IP {}: {:?}", ip, provider);
                } else {
                    filtered.push(ip);
                }
            }

            // Log statistics
            if skipped > 0 {
                let total = filtered.len() + skipped;
                let reduction_pct = (skipped * 100) / total;
                info!(
                    "Filtered {} CDN IPs before discovery ({}% reduction): {:?}",
                    skipped, reduction_pct, provider_counts
                );
            }

            // Check if all hosts were filtered
            if filtered.is_empty() {
                info!("All hosts filtered (CDN detection), skipping scan");
                return Ok(Vec::new());
            }

            filtered
        } else {
            original_ips
        };

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

        // Transition to ScanningPorts stage after discovery
        if let Some(ref event_bus) = self.config.scan.event_bus {
            use prtip_core::events::{ScanEvent, ScanStage};
            use std::time::SystemTime;
            use uuid::Uuid;

            let scan_id = Uuid::new_v4();
            let timestamp = SystemTime::now();

            event_bus
                .publish(ScanEvent::StageChanged {
                    scan_id,
                    timestamp,
                    from_stage: ScanStage::DiscoveringHosts,
                    to_stage: ScanStage::ScanningPorts,
                })
                .await;

            debug!("Published StageChanged (ScanningPorts) event after discovery");
        }

        // Execute normal scan on live hosts
        self.execute_scan(live_targets, pcapng_writer).await
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
        // Store target count before vector is consumed
        let target_count = targets.len();

        info!(
            "Starting port scan: {} targets, {} ports",
            target_count,
            ports.count()
        );

        // Generate scan_id for this scan (used by events and progress tracker)
        let scan_id = Uuid::new_v4();

        // Publish ScanStarted event for TUI
        if let Some(ref event_bus) = self.config.scan.event_bus {
            use prtip_core::events::{ScanEvent, ScanStage};
            use std::time::SystemTime;

            let timestamp = SystemTime::now();

            event_bus
                .publish(ScanEvent::ScanStarted {
                    scan_id,
                    scan_type: self.config.scan.scan_type,
                    target_count,
                    port_count: ports.count(),
                    timestamp,
                })
                .await;

            // Transition to ScanningPorts stage
            event_bus
                .publish(ScanEvent::StageChanged {
                    scan_id,
                    timestamp,
                    from_stage: ScanStage::Initializing,
                    to_stage: ScanStage::ScanningPorts,
                })
                .await;

            debug!("Published ScanStarted and StageChanged events");
        }

        let ports_vec: Vec<u16> = ports.iter().collect();

        // Calculate estimated hosts for progress bar and buffer sizing
        let estimated_hosts: usize = targets.iter().map(|t| t.expand_hosts().len()).sum();

        // Create progress bar for real-time feedback
        let total_ports = (estimated_hosts * ports_vec.len()) as u64;
        let progress = Arc::new(ScanProgressBar::new(total_ports, self.config.scan.progress));
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

        // Create progress tracker for TUI updates (Sprint 6.8 fix)
        // Uses same scan_id as ScanStarted event
        let mut progress_tracker = ProgressTracker::new(scan_id, total_ports);

        for target in targets {
            let original_hosts = target.expand_hosts();

            // Filter CDN IPs if enabled
            let hosts = if let Some(ref detector) = self.cdn_detector {
                let mut filtered = Vec::new();
                let mut skipped = 0;
                let mut provider_counts: std::collections::HashMap<CdnProvider, usize> =
                    std::collections::HashMap::new();

                for host in original_hosts {
                    if let Some(provider) = detector.detect(&host) {
                        // Track statistics
                        *provider_counts.entry(provider).or_insert(0) += 1;
                        skipped += 1;
                        debug!("Skipping CDN IP {}: {:?}", host, provider);
                    } else {
                        filtered.push(host);
                    }
                }

                // Log statistics
                if skipped > 0 {
                    let total = filtered.len() + skipped;
                    let reduction_pct = (skipped * 100) / total;
                    info!(
                        "Filtered {} CDN IPs ({}% reduction): {:?}",
                        skipped, reduction_pct, provider_counts
                    );
                }

                // Check if all hosts were filtered
                if filtered.is_empty() {
                    debug!("All hosts filtered (CDN detection), continuing to next target");
                    continue;
                }

                debug!("Scanning {} hosts after CDN filtering", filtered.len());
                filtered
            } else {
                original_hosts
            };

            for (host_idx, host) in hosts.iter().enumerate() {
                // Rate limiter
                if let Some(ref limiter) = self.rate_limiter {
                    limiter.acquire().await?;
                }

                // Create a progress tracker for this host's ports
                let host_progress = Arc::new(prtip_core::ScanProgress::new(ports_vec.len()));

                // Spawn a task to bridge host progress to main progress bar
                // This updates the progress bar in real-time as each port completes,
                // rather than jumping to 100% after all ports are done
                let progress_clone = Arc::clone(&progress);
                let host_progress_clone = Arc::clone(&host_progress);
                let total_ports = ports_vec.len();

                // Adaptive polling interval based on TOTAL SCAN PORTS (hosts × ports):
                // - Tiny scans (< 1K ports): 1ms - still responsive, 5x less overhead than 200µs
                // - Small scans (< 10K ports): 2ms - rapid updates for fast scans
                // - Medium scans (< 100K ports): 5ms - balance responsiveness and CPU
                // - Large scans (< 1M ports): 10ms - reduces overhead for network scans
                // - Huge scans (≥ 1M ports): 20ms - minimal overhead for massive scans
                //
                // This prevents catastrophic polling overhead on large scans:
                // Example: 256 hosts × 10K ports = 2.56M total
                //   - Old (1ms): 7.2M polls over 2 hours = 2,160s overhead (30%!)
                //   - New (20ms): 360K polls = 108s overhead (1.5%, acceptable)
                let poll_interval = if total_scan_ports < 1_000 {
                    Duration::from_millis(1) // 1ms - tiny scans (1 host × 1K ports)
                } else if total_scan_ports < 10_000 {
                    Duration::from_millis(2) // 2ms - small scans (1 host × 10K ports)
                } else if total_scan_ports < 100_000 {
                    Duration::from_millis(5) // 5ms - medium scans (10 hosts × 10K ports)
                } else if total_scan_ports < 1_000_000 {
                    Duration::from_millis(10) // 10ms - large scans (100 hosts × 10K ports)
                } else {
                    Duration::from_millis(20) // 20ms - huge scans (256+ hosts × 10K ports)
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

                #[cfg(feature = "numa")]
                let numa_ref = self.numa_manager.as_ref();
                #[cfg(not(feature = "numa"))]
                let numa_ref: Option<&()> = None;

                match self
                    .tcp_scanner
                    .scan_ports_with_progress(
                        *host,
                        ports_vec.clone(),
                        parallelism,
                        Some(&host_progress),
                        numa_ref,
                    )
                    .await
                {
                    Ok(results) => {
                        // Wait for progress bridge to finish
                        let _ = bridge_handle.await;

                        // Update progress tracker for TUI (Sprint 6.8 fix)
                        for _ in 0..ports_vec.len() {
                            progress_tracker
                                .increment(&self.config.scan.event_bus)
                                .await;
                        }

                        // Push results to lock-free aggregator (zero contention!)
                        for result in results.iter() {
                            // Try to push result
                            if let Err(e) = aggregator.push(result.clone()) {
                                warn!("Failed to push result to aggregator: {}", e);
                                // If queue is full, drain a batch to storage immediately
                                let batch = aggregator.drain_batch(10000);

                                if !batch.is_empty() {
                                    self.storage_backend.add_results_batch(batch)?;
                                }
                                // Retry push (result not moved because we cloned above)
                                aggregator.push(result.clone())?;
                            }
                        }

                        // Apply host delay if configured (helps avoid network rate limiting)
                        if self.config.scan.host_delay_ms > 0 {
                            debug!(
                                "Applying host delay: {}ms (host {}/{})",
                                self.config.scan.host_delay_ms,
                                host_idx + 1,
                                hosts.len()
                            );
                            tokio::time::sleep(Duration::from_millis(
                                self.config.scan.host_delay_ms,
                            ))
                            .await;
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
                let probe_db = if let Some(path) = &self.config.scan.service_detection.probe_db_path
                {
                    ServiceProbeDb::load_from_file(path)?
                } else {
                    ServiceProbeDb::default()
                };
                let detector = ServiceDetector::with_options(
                    probe_db,
                    self.config.scan.service_detection.intensity,
                    self.config.scan.service_detection.enable_tls,
                    self.config.scan.service_detection.capture_raw,
                );
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
                                    result.raw_response = service_info.raw_response;

                                    // Combine product and version
                                    let version_string =
                                        match (&service_info.product, &service_info.version) {
                                            (Some(product), Some(version)) => {
                                                Some(format!("{} {}", product, version))
                                            }
                                            (Some(product), None) => Some(product.clone()),
                                            (None, Some(version)) => Some(version.clone()),
                                            (None, None) => None,
                                        };
                                    result.version = version_string.clone();

                                    debug!(
                                        "Detected service on {}:{}: {} {}",
                                        result.target_ip,
                                        result.port,
                                        result.service.as_ref().unwrap_or(&"unknown".to_string()),
                                        result.version.as_ref().unwrap_or(&"".to_string())
                                    );

                                    // Publish ServiceDetected event for TUI
                                    if let Some(ref event_bus) = self.config.scan.event_bus {
                                        // Calculate confidence based on detection method
                                        // High confidence (0.9) if product and version detected
                                        // Medium confidence (0.75) if only product detected
                                        // Low confidence (0.5) for service name only
                                        let confidence = if service_info.product.is_some()
                                            && service_info.version.is_some()
                                        {
                                            0.9
                                        } else if service_info.product.is_some() {
                                            0.75
                                        } else {
                                            0.5
                                        };

                                        event_bus
                                            .publish(ScanEvent::ServiceDetected {
                                                scan_id: Uuid::new_v4(),
                                                ip: result.target_ip,
                                                port: result.port,
                                                service_name: service_info.service.clone(),
                                                service_version: version_string,
                                                confidence,
                                                timestamp: SystemTime::now(),
                                            })
                                            .await;
                                    }

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

        // Publish final ProgressUpdate event to ensure 100% completion (Sprint 6.8 fix)
        progress_tracker.publish(&self.config.scan.event_bus).await;

        // Publish ScanCompleted event for TUI
        if let Some(ref event_bus) = self.config.scan.event_bus {
            use prtip_core::events::{ScanEvent, ScanStage};
            use std::time::{Duration, SystemTime};

            // Use same scan_id as ScanStarted/ProgressUpdate events
            let timestamp = SystemTime::now();

            // Calculate port counts
            let open_count = all_results
                .iter()
                .filter(|r| r.state == PortState::Open)
                .count();
            let closed_count = all_results
                .iter()
                .filter(|r| r.state == PortState::Closed)
                .count();
            let filtered_count = all_results
                .iter()
                .filter(|r| r.state == PortState::Filtered)
                .count();
            let detected_services = all_results.iter().filter(|r| r.service.is_some()).count();

            // Transition to Completed stage
            event_bus
                .publish(ScanEvent::StageChanged {
                    scan_id,
                    timestamp,
                    from_stage: ScanStage::ScanningPorts,
                    to_stage: ScanStage::Completed,
                })
                .await;

            event_bus
                .publish(ScanEvent::ScanCompleted {
                    scan_id,
                    duration: Duration::from_secs(0), // TODO: Track actual scan duration
                    total_targets: target_count,
                    open_ports: open_count,
                    closed_ports: closed_count,
                    filtered_ports: filtered_count,
                    detected_services,
                    timestamp,
                })
                .await;

            debug!("Published ScanCompleted event");
        }

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
                host_delay_ms: 0,
                service_detection: Default::default(),
                progress: false,
                event_bus: None,
            },
            network: NetworkConfig {
                interface: None,
                source_port: None,
                skip_cdn: false,
                cdn_whitelist: None,
                cdn_blacklist: None,
            },
            output: OutputConfig {
                format: OutputFormat::Json,
                file: None,
                verbose: 0,
                use_mmap: false,
                mmap_output_path: None,
            },
            performance: PerformanceConfig {
                max_rate: Some(100),
                parallelism: 10,
                batch_size: None,
                requested_ulimit: None,
                numa_enabled: false,
                adaptive_batch_enabled: false,
                min_batch_size: 1,
                max_batch_size: 1024,
            },
            evasion: Default::default(),
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

        let results = scheduler.execute_scan(vec![], None).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_scan_localhost() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(1000));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        let results = scheduler.execute_scan(targets, None).await.unwrap();

        // Should have some results (even if all filtered/closed)
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_execute_scan_localhost_memory_backend() {
        let config = create_test_config().await;
        let storage_backend = Arc::new(StorageBackend::memory(10000));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        let results = scheduler.execute_scan(targets, None).await.unwrap();

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
        let results = scheduler.scan_target(&target, None).await.unwrap();

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
            .execute_scan_with_discovery(targets, None)
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
        let results = scheduler.execute_scan(targets, None).await.unwrap();

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

        let results = scheduler.execute_scan(targets, None).await.unwrap();

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
        let results = scheduler.execute_scan(targets, None).await.unwrap();

        // Check that results were stored
        let stored_results = storage_backend.get_results().await.unwrap();
        assert_eq!(stored_results.len(), results.len());
        assert!(!stored_results.is_empty());
    }

    #[tokio::test]
    async fn test_scheduler_with_numa_enabled() {
        // Test that scheduler works with NUMA enabled (graceful fallback if unavailable)
        let mut config = create_test_config().await;
        config.performance.numa_enabled = true; // Request NUMA

        let storage_backend = Arc::new(StorageBackend::memory(1000));
        let scheduler = ScanScheduler::new(config, storage_backend).await.unwrap();

        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        let ports = PortRange::parse("9999").unwrap();

        // Should work even if NUMA unavailable or single-socket (graceful fallback)
        let results = scheduler.execute_scan_ports(targets, &ports).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].port, 9999);
    }

    #[tokio::test]
    async fn test_scheduler_numa_graceful_fallback() {
        // Test that NUMA gracefully falls back on non-NUMA systems
        let mut config = create_test_config().await;
        config.performance.numa_enabled = true; // Request NUMA

        let storage_backend = Arc::new(StorageBackend::memory(100));

        // Scheduler creation should not fail even if NUMA unavailable
        let scheduler = ScanScheduler::new(config, storage_backend).await;
        assert!(
            scheduler.is_ok(),
            "Scheduler should create successfully even if NUMA unavailable"
        );

        let scheduler = scheduler.unwrap();
        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        let results = scheduler.execute_scan(targets, None).await.unwrap();

        // Should have results (NUMA is optional optimization, not blocking)
        assert!(!results.is_empty());
    }
}
