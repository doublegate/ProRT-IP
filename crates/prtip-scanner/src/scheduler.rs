//! Scan Scheduler
//!
//! Orchestrates the complete scanning workflow by coordinating all scanner components:
//! - Host discovery
//! - Port scanning
//! - Rate limiting
//! - Result storage
//!
//! The scheduler manages the scan lifecycle from initialization through completion.

use crate::{DiscoveryEngine, DiscoveryMethod, RateLimiter, ScanStorage, TcpConnectScanner};
use prtip_core::{Config, Error, PortRange, Result, ScanResult, ScanTarget};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Scan scheduler that orchestrates the scanning process
///
/// The scheduler coordinates between different scan components, manages
/// resource limits, and provides a high-level interface for executing scans.
///
/// # Examples
///
/// ```no_run
/// use prtip_scanner::{ScanScheduler, ScanStorage};
/// use prtip_core::{Config, ScanTarget};
///
/// # async fn example() -> prtip_core::Result<()> {
/// let config = Config::default();
/// let storage = ScanStorage::new("results.db").await?;
/// let scheduler = ScanScheduler::new(config, storage).await?;
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
    storage: Arc<RwLock<ScanStorage>>,
}

impl ScanScheduler {
    /// Create a new scan scheduler
    ///
    /// # Arguments
    ///
    /// * `config` - Scan configuration
    /// * `storage` - Storage backend for results
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub async fn new(config: Config, storage: ScanStorage) -> Result<Self> {
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
            storage: Arc::new(RwLock::new(storage)),
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

        // Create scan record
        let config_json = serde_json::to_string(&self.config)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;

        let scan_id = {
            let storage = self.storage.write().await;
            storage.create_scan(&config_json).await?
        };

        info!("Created scan ID: {}", scan_id);

        let mut all_results = Vec::new();

        // Scan each target
        for target in targets {
            debug!("Scanning target: {:?}", target);

            match self.scan_target(&target, scan_id).await {
                Ok(results) => {
                    all_results.extend(results);
                }
                Err(e) => {
                    error!("Error scanning target {:?}: {}", target, e);
                    // Continue with other targets
                }
            }
        }

        // Complete scan
        {
            let storage = self.storage.write().await;
            storage.complete_scan(scan_id).await?;
        }

        info!("Scan complete: {} results", all_results.len());
        Ok(all_results)
    }

    /// Scan a single target
    async fn scan_target(&self, target: &ScanTarget, scan_id: i64) -> Result<Vec<ScanResult>> {
        // Expand target into individual IPs
        let hosts = target.expand_hosts();
        debug!("Target expanded to {} hosts", hosts.len());

        let mut all_results = Vec::new();

        // Parse port range from config
        // For Phase 1, we'll scan common ports if none specified
        let ports = self.get_scan_ports()?;
        debug!("Scanning {} ports", ports.len());

        for host in hosts {
            // Rate limiting
            self.rate_limiter.acquire().await?;

            // Perform TCP connect scan
            match self
                .tcp_scanner
                .scan_ports(host, ports.clone(), self.config.performance.parallelism)
                .await
            {
                Ok(results) => {
                    // Store results immediately (streaming to disk)
                    {
                        let storage = self.storage.write().await;
                        if let Err(e) = storage.store_results_batch(scan_id, &results).await {
                            warn!("Failed to store results for {}: {}", host, e);
                        }
                    }

                    all_results.extend(results);
                }
                Err(e) => {
                    warn!("Error scanning {}: {}", host, e);
                    // Continue with other hosts
                }
            }
        }

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

        // Discover live hosts
        let live_hosts = self
            .discovery
            .discover_hosts(all_ips.clone(), self.config.performance.parallelism)
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

        let config_json = serde_json::to_string(&self.config)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;

        let scan_id = {
            let storage = self.storage.write().await;
            storage.create_scan(&config_json).await?
        };

        let ports_vec: Vec<u16> = ports.iter().collect();
        let mut all_results = Vec::new();

        for target in targets {
            let hosts = target.expand_hosts();

            for host in hosts {
                self.rate_limiter.acquire().await?;

                match self
                    .tcp_scanner
                    .scan_ports(host, ports_vec.clone(), self.config.performance.parallelism)
                    .await
                {
                    Ok(results) => {
                        {
                            let storage = self.storage.write().await;
                            storage.store_results_batch(scan_id, &results).await?;
                        }
                        all_results.extend(results);
                    }
                    Err(e) => {
                        warn!("Error scanning {}: {}", host, e);
                    }
                }
            }
        }

        {
            let storage = self.storage.write().await;
            storage.complete_scan(scan_id).await?;
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
        NetworkConfig, OutputConfig, OutputFormat, PerformanceConfig, ScanConfig, ScanType, TimingTemplate,
    };

    async fn create_test_config() -> Config {
        Config {
            scan: ScanConfig {
                scan_type: ScanType::Connect,
                timing_template: TimingTemplate::Normal,
                timeout_ms: 1000,
                retries: 0,
                scan_delay_ms: 0,
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
            },
        }
    }

    #[tokio::test]
    async fn test_create_scheduler() {
        let config = create_test_config().await;
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scheduler = ScanScheduler::new(config, storage).await.unwrap();

        assert!(scheduler.config().performance.max_rate.is_some());
    }

    #[tokio::test]
    async fn test_execute_scan_empty() {
        let config = create_test_config().await;
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scheduler = ScanScheduler::new(config, storage).await.unwrap();

        let results = scheduler.execute_scan(vec![]).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_scan_localhost() {
        let config = create_test_config().await;
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scheduler = ScanScheduler::new(config, storage).await.unwrap();

        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        let results = scheduler.execute_scan(targets).await.unwrap();

        // Should have some results (even if all filtered/closed)
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_execute_scan_ports() {
        let config = create_test_config().await;
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scheduler = ScanScheduler::new(config, storage).await.unwrap();

        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        let ports = PortRange::parse("9999").unwrap();

        let results = scheduler.execute_scan_ports(targets, &ports).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].port, 9999);
    }

    #[tokio::test]
    async fn test_scan_target_single_host() {
        let config = create_test_config().await;
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scan_id = storage.create_scan(r#"{"test": true}"#).await.unwrap();

        let scheduler = ScanScheduler::new(config, storage).await.unwrap();

        let target = ScanTarget::parse("127.0.0.1").unwrap();
        let results = scheduler.scan_target(&target, scan_id).await.unwrap();

        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_config() {
        let mut config = create_test_config().await;
        config.scan.timeout_ms = 0; // Invalid

        let storage = ScanStorage::new(":memory:").await.unwrap();
        let result = ScanScheduler::new(config, storage).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_scan_ports() {
        let config = create_test_config().await;
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scheduler = ScanScheduler::new(config, storage).await.unwrap();

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
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scheduler = ScanScheduler::new(config, storage).await.unwrap();

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
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scheduler = ScanScheduler::new(config, storage).await.unwrap();

        // TEST-NET-1 (should be unreachable)
        let targets = vec![ScanTarget::parse("192.0.2.1").unwrap()];
        let results = scheduler.execute_scan(targets).await.unwrap();

        // May have results (filtered ports), just verify no panic
        let _ = results;
    }

    #[tokio::test]
    async fn test_multiple_targets() {
        let config = create_test_config().await;
        let storage = ScanStorage::new(":memory:").await.unwrap();
        let scheduler = ScanScheduler::new(config, storage).await.unwrap();

        let targets = vec![
            ScanTarget::parse("127.0.0.1").unwrap(),
            ScanTarget::parse("127.0.0.2").unwrap(),
        ];

        let results = scheduler.execute_scan(targets).await.unwrap();

        // Should have results for both targets
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_results_stored_in_database() {
        let config = create_test_config().await;
        let storage = ScanStorage::new(":memory:").await.unwrap();

        let scheduler = ScanScheduler::new(config, storage).await.unwrap();

        let targets = vec![ScanTarget::parse("127.0.0.1").unwrap()];
        scheduler.execute_scan(targets).await.unwrap();

        // Check that results were stored
        let scan_count = {
            let storage = scheduler.storage.read().await;
            storage.get_scan_count().await.unwrap()
        };

        assert!(scan_count >= 1);
    }
}
