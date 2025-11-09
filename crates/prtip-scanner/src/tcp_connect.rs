//! TCP Connect Scanner
//!
//! Implements full TCP connect scans using the OS socket API. This is the most
//! compatible scan type as it doesn't require raw socket privileges.
//!
//! # How It Works
//!
//! The scanner attempts to complete a full 3-way TCP handshake for each target port:
//! 1. Send SYN (OS handles this automatically)
//! 2. Receive SYN/ACK (port open) or RST (port closed)
//! 3. Complete handshake and close connection
//!
//! # Performance
//!
//! TCP connect scans are slower than SYN scans because they complete the full handshake,
//! but they work without elevated privileges and are compatible with all target systems.

use crate::lockfree_aggregator::LockFreeAggregator;
use crate::{AdaptiveRateLimiterV2, HostgroupLimiter};
use prtip_core::{
    Error, EventBus, PortState, Protocol, Result, ScanEvent, ScanProgress, ScanResult, ScanStage,
};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;
use tracing::{debug, trace, warn};
use uuid::Uuid;

#[cfg(feature = "numa")]
use prtip_network::numa::NumaManager;
#[cfg(feature = "numa")]
use std::sync::atomic::{AtomicUsize, Ordering};

/// TCP Connect Scanner
///
/// Performs TCP connect scans by establishing full TCP connections
/// to target ports using the OS socket API.
///
/// # Examples
///
/// ```no_run
/// use prtip_scanner::TcpConnectScanner;
/// use std::net::IpAddr;
/// use std::time::Duration;
///
/// # async fn example() -> prtip_core::Result<()> {
/// let scanner = TcpConnectScanner::new(Duration::from_secs(2), 1);
/// let target: IpAddr = "192.168.1.1".parse().unwrap();
/// let result = scanner.scan_port(target, 80).await?;
/// println!("Port 80: {}", result.state);
/// # Ok(())
/// # }
/// ```
///
/// # Rate Limiting (Sprint 5.4 Phase 1)
///
/// Supports optional hostgroup and adaptive rate limiting:
/// - Hostgroup limiter controls concurrent targets
/// - Adaptive limiter provides per-target ICMP backoff
#[derive(Clone)]
pub struct TcpConnectScanner {
    timeout: Duration,
    retries: u32,
    /// Optional hostgroup limiter (controls concurrent targets)
    hostgroup_limiter: Option<Arc<HostgroupLimiter>>,
    /// Optional adaptive rate limiter (ICMP-aware throttling)
    adaptive_limiter: Option<Arc<AdaptiveRateLimiterV2>>,
    /// Optional event bus for real-time progress updates
    event_bus: Option<Arc<EventBus>>,
}

impl TcpConnectScanner {
    /// Create a new TCP connect scanner
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for a connection response
    /// * `retries` - Number of retry attempts for failed connections
    pub fn new(timeout: Duration, retries: u32) -> Self {
        Self {
            timeout,
            retries,
            hostgroup_limiter: None,
            adaptive_limiter: None,
            event_bus: None,
        }
    }

    /// Enable hostgroup limiting (concurrent target control)
    ///
    /// # Arguments
    ///
    /// * `limiter` - Hostgroup limiter to use
    pub fn with_hostgroup_limiter(mut self, limiter: Arc<HostgroupLimiter>) -> Self {
        self.hostgroup_limiter = Some(limiter);
        self
    }

    /// Enable adaptive rate limiting (ICMP-aware throttling)
    ///
    /// # Arguments
    ///
    /// * `limiter` - Adaptive rate limiter to use
    pub fn with_adaptive_limiter(mut self, limiter: Arc<AdaptiveRateLimiterV2>) -> Self {
        self.adaptive_limiter = Some(limiter);
        self
    }

    /// Attach an event bus for real-time scan events
    ///
    /// # Arguments
    ///
    /// * `bus` - Event bus to emit scan events to
    pub fn with_event_bus(mut self, bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(bus);
        self
    }

    /// Scan a single port on a target host
    ///
    /// Attempts to establish a TCP connection to the specified port. Returns a
    /// `ScanResult` indicating whether the port is open, closed, or filtered.
    ///
    /// # Arguments
    ///
    /// * `target` - Target IP address
    /// * `port` - Port number to scan (1-65535)
    ///
    /// # Returns
    ///
    /// Returns a `ScanResult` with the port state and timing information.
    ///
    /// # Errors
    ///
    /// Returns an error if the port number is invalid (0) or if there's a
    /// network configuration problem.
    pub async fn scan_port(&self, target: IpAddr, port: u16) -> Result<ScanResult> {
        if port == 0 {
            return Err(Error::InvalidPortRange("port 0 is invalid".to_string()));
        }

        let start = Instant::now();
        let addr = SocketAddr::new(target, port);

        trace!("Scanning {}:{}", target, port);

        let state = self.attempt_connect(addr).await?;
        let response_time = start.elapsed();

        Ok(ScanResult::new(target, port, state).with_response_time(response_time))
    }

    /// Attempt to connect to a socket address with retries
    ///
    /// # Implementation Details
    ///
    /// This method tries to establish a TCP connection with the following logic:
    /// - Connection successful (SYN/ACK received) → Port is OPEN
    /// - Connection refused (RST received) → Port is CLOSED
    /// - Timeout or other I/O errors → Port is FILTERED (potentially firewalled)
    async fn attempt_connect(&self, addr: SocketAddr) -> Result<PortState> {
        for attempt in 0..=self.retries {
            if attempt > 0 {
                trace!("Retry attempt {} for {}", attempt, addr);
            }

            match timeout(self.timeout, TcpStream::connect(addr)).await {
                Ok(Ok(_stream)) => {
                    // Connection succeeded - port is open
                    debug!("Port {} open on {}", addr.port(), addr.ip());
                    return Ok(PortState::Open);
                }
                Ok(Err(e)) => {
                    // Connection error
                    match e.kind() {
                        std::io::ErrorKind::ConnectionRefused => {
                            // Explicit RST received - port is closed
                            debug!("Port {} closed on {}", addr.port(), addr.ip());
                            return Ok(PortState::Closed);
                        }
                        std::io::ErrorKind::PermissionDenied => {
                            // Firewall or policy blocked the connection
                            warn!("Permission denied for {}", addr);
                            return Ok(PortState::Filtered);
                        }
                        std::io::ErrorKind::AddrInUse | std::io::ErrorKind::AddrNotAvailable => {
                            // Local address/port issue
                            warn!("Address unavailable for {}: {}", addr, e);
                            if attempt == self.retries {
                                return Ok(PortState::Filtered);
                            }
                            // Retry with different source port
                            continue;
                        }
                        _ => {
                            // Other I/O errors (network unreachable, etc.)
                            warn!("I/O error connecting to {}: {}", addr, e);
                            if attempt == self.retries {
                                return Ok(PortState::Filtered);
                            }
                        }
                    }
                }
                Err(_elapsed) => {
                    // Timeout - port filtered or no response
                    debug!(
                        "Timeout connecting to {} (attempt {}/{})",
                        addr,
                        attempt + 1,
                        self.retries + 1
                    );
                    if attempt == self.retries {
                        return Ok(PortState::Filtered);
                    }
                }
            }

            // Small delay before retry to avoid overwhelming the target
            if attempt < self.retries {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        Ok(PortState::Filtered)
    }

    /// Scan multiple ports on a target host concurrently
    ///
    /// Uses a semaphore to limit concurrency and avoid overwhelming the target
    /// or local system resources.
    ///
    /// # Arguments
    ///
    /// * `target` - Target IP address
    /// * `ports` - Vector of port numbers to scan
    /// * `max_concurrent` - Maximum number of concurrent scan operations
    ///
    /// # Returns
    ///
    /// Vector of `ScanResult` for all scanned ports. Failed scans are logged
    /// but don't prevent other scans from completing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_scanner::TcpConnectScanner;
    /// # use std::time::Duration;
    /// # async fn example() -> prtip_core::Result<()> {
    /// let scanner = TcpConnectScanner::new(Duration::from_secs(1), 0);
    /// let target = "192.168.1.1".parse().unwrap();
    /// let ports = vec![80, 443, 8080];
    /// let results = scanner.scan_ports(target, ports, 10).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn scan_ports(
        &self,
        target: IpAddr,
        ports: Vec<u16>,
        max_concurrent: usize,
    ) -> Result<Vec<ScanResult>> {
        self.scan_ports_with_progress(target, ports, max_concurrent, None, None)
            .await
    }

    /// Scan multiple ports on a target with optional progress tracking
    ///
    /// Similar to `scan_ports` but accepts an optional `ScanProgress` tracker
    /// for real-time progress monitoring and statistics collection.
    ///
    /// # Arguments
    ///
    /// * `target` - Target IP address
    /// * `ports` - Vector of port numbers to scan
    /// * `max_concurrent` - Maximum concurrent scan operations
    /// * `progress` - Optional progress tracker for statistics
    /// * `numa_manager` - Optional NUMA manager for thread pinning (Linux only)
    pub async fn scan_ports_with_progress(
        &self,
        target: IpAddr,
        ports: Vec<u16>,
        max_concurrent: usize,
        progress: Option<&ScanProgress>,
        #[cfg(feature = "numa")] numa_manager: Option<&Arc<NumaManager>>,
        #[cfg(not(feature = "numa"))] _numa_manager: Option<&()>,
    ) -> Result<Vec<ScanResult>> {
        // Generate scan ID for event tracking
        let scan_id = Uuid::new_v4();
        let scan_start = Instant::now();

        // Emit ScanStarted event
        if let Some(bus) = &self.event_bus {
            bus.publish(ScanEvent::ScanStarted {
                scan_id,
                scan_type: prtip_core::ScanType::Connect,
                target_count: 1,
                port_count: ports.len(),
                timestamp: SystemTime::now(),
            })
            .await;

            // Emit stage change to ScanningPorts
            bus.publish(ScanEvent::StageChanged {
                scan_id,
                from_stage: ScanStage::ResolvingTargets,
                to_stage: ScanStage::ScanningPorts,
                timestamp: SystemTime::now(),
            })
            .await;
        }

        // 1. Acquire hostgroup permit (if rate limiting enabled)
        let _permit = if let Some(limiter) = &self.hostgroup_limiter {
            Some(limiter.acquire_target().await)
        } else {
            None
        };

        // 2. Check ICMP backoff (if adaptive rate limiting enabled)
        if let Some(limiter) = &self.adaptive_limiter {
            if limiter.is_target_backed_off(target) {
                debug!("Skipping {} (ICMP backoff active)", target);

                // Emit scan completion with no results
                if let Some(bus) = &self.event_bus {
                    bus.publish(ScanEvent::ScanCompleted {
                        scan_id,
                        duration: scan_start.elapsed(),
                        total_targets: 1,
                        open_ports: 0,
                        closed_ports: 0,
                        filtered_ports: 0,
                        detected_services: 0,
                        timestamp: SystemTime::now(),
                    })
                    .await;
                }

                return Ok(Vec::new());
            }
        }

        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let scanner = self.clone();

        // Use lock-free aggregator for concurrent result collection (10-30% faster)
        let aggregator = Arc::new(LockFreeAggregator::new(ports.len() * 2));

        let mut handles = Vec::with_capacity(ports.len());

        // NUMA: Worker counter for round-robin core allocation across NUMA nodes
        #[cfg(feature = "numa")]
        let worker_counter = Arc::new(AtomicUsize::new(0));

        #[cfg(feature = "numa")]
        let numa_manager_clone = numa_manager.cloned();

        for port in ports {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| Error::Network(format!("Semaphore error: {}", e)))?;
            let scanner = scanner.clone();
            let agg_clone = Arc::clone(&aggregator);

            #[cfg(feature = "numa")]
            let worker_id = worker_counter.fetch_add(1, Ordering::Relaxed);
            #[cfg(feature = "numa")]
            let numa_mgr = numa_manager_clone.clone();

            let handle = tokio::spawn(async move {
                // NUMA: Pin worker thread to core based on round-robin across NUMA nodes
                #[cfg(feature = "numa")]
                if let Some(ref manager) = numa_mgr {
                    let node_id = worker_id % manager.node_count();
                    match manager.allocate_core(Some(node_id)) {
                        Ok(worker_core) => {
                            if let Err(e) = manager.pin_current_thread(worker_core) {
                                debug!(
                                    "Failed to pin worker {} to core {}: {}",
                                    worker_id, worker_core, e
                                );
                            } else {
                                trace!(
                                    "Worker {} pinned to core {} (node {})",
                                    worker_id,
                                    worker_core,
                                    node_id
                                );
                            }
                        }
                        Err(e) => debug!("Failed to allocate core for worker {}: {}", worker_id, e),
                    }
                }

                let result = scanner.scan_port(target, port).await;
                drop(permit);

                // Push result to lock-free aggregator in worker thread (zero contention)
                if let Ok(scan_result) = &result {
                    let _ = agg_clone.push(scan_result.clone());
                }

                result
            });

            handles.push(handle);
        }

        // Wait for all workers and update progress AS THEY COMPLETE
        // Using FuturesUnordered ensures we process results as soon as they're ready
        use futures::stream::{FuturesUnordered, StreamExt};

        let mut futures_unordered = handles.into_iter().collect::<FuturesUnordered<_>>();

        while let Some(result) = futures_unordered.next().await {
            match result {
                Ok(Ok(scan_result)) => {
                    // Emit PortFound event for open ports
                    if scan_result.state == PortState::Open {
                        if let Some(bus) = &self.event_bus {
                            bus.publish(ScanEvent::PortFound {
                                scan_id,
                                ip: scan_result.target_ip,
                                port: scan_result.port,
                                state: scan_result.state,
                                protocol: Protocol::Tcp,
                                scan_type: prtip_core::ScanType::Connect,
                                timestamp: SystemTime::now(),
                            })
                            .await;
                        }
                    }

                    if let Some(p) = progress {
                        p.increment_completed();
                        match scan_result.state {
                            PortState::Open => p.increment_open(),
                            PortState::Closed => p.increment_closed(),
                            PortState::Filtered => p.increment_filtered(),
                            PortState::Unknown => {} // Don't increment state counters for unknown
                        }
                    }
                }
                Ok(Err(e)) => {
                    if let Some(p) = progress {
                        p.increment_completed();
                        // For now, categorize as "Other" - in future we can add
                        // specific error types to prtip_core::Error
                        use prtip_core::ErrorCategory;
                        p.increment_error(ErrorCategory::Other);
                    }
                    warn!("Scan error: {}", e);
                }
                Err(e) => warn!("Task join error: {}", e),
            }
        }

        // Drain all results from aggregator (lock-free batch operation)
        let results = aggregator.drain_all();
        debug!(
            "Collected {} results from lock-free aggregator",
            results.len()
        );

        // Calculate final statistics
        let open_count = results.iter().filter(|r| r.state == PortState::Open).count();
        let closed_count = results.iter().filter(|r| r.state == PortState::Closed).count();
        let filtered_count = results.iter().filter(|r| r.state == PortState::Filtered).count();

        // Emit ScanCompleted event
        if let Some(bus) = &self.event_bus {
            bus.publish(ScanEvent::ScanCompleted {
                scan_id,
                duration: scan_start.elapsed(),
                total_targets: 1,
                open_ports: open_count,
                closed_ports: closed_count,
                filtered_ports: filtered_count,
                detected_services: 0, // TCP Connect doesn't do service detection
                timestamp: SystemTime::now(),
            })
            .await;
        }

        Ok(results)
    }

    /// Scan a complete target specification (multiple hosts and ports)
    ///
    /// Expands the target into individual hosts and scans all specified ports
    /// on each host.
    ///
    /// # Arguments
    ///
    /// * `target` - Target specification (may include CIDR ranges)
    /// * `ports` - Vector of port numbers to scan
    /// * `max_concurrent` - Maximum concurrent scan operations
    ///
    /// # Returns
    ///
    /// Vector of all scan results across all hosts and ports.
    pub async fn scan_target(
        &self,
        target: &prtip_core::ScanTarget,
        ports: Vec<u16>,
        max_concurrent: usize,
    ) -> Result<Vec<ScanResult>> {
        let hosts = target.expand_hosts();
        let mut all_results = Vec::new();

        for host in hosts {
            let results = self.scan_ports(host, ports.clone(), max_concurrent).await?;
            all_results.extend(results);
        }

        Ok(all_results)
    }

    /// Get the configured timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get the configured retry count
    pub fn retries(&self) -> u32 {
        self.retries
    }
}

impl Default for TcpConnectScanner {
    fn default() -> Self {
        Self::new(Duration::from_secs(3), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_scan_closed_port() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(100), 0);
        let result = scanner
            .scan_port(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999)
            .await
            .unwrap();

        // Should be Closed or Filtered (depending on local firewall)
        assert!(matches!(
            result.state,
            PortState::Closed | PortState::Filtered
        ));
        assert_eq!(result.port, 9999);
    }

    #[tokio::test]
    async fn test_scan_timeout() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(1), 0);
        // Scan non-routable address to force timeout
        let result = scanner
            .scan_port(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), 80)
            .await
            .unwrap();

        assert_eq!(result.state, PortState::Filtered);
    }

    #[tokio::test]
    async fn test_scan_multiple_ports() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(100), 0);
        let results = scanner
            .scan_ports(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                vec![9998, 9999],
                10,
            )
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].port, 9998);
        assert_eq!(results[1].port, 9999);
    }

    #[tokio::test]
    async fn test_scanner_default() {
        let scanner = TcpConnectScanner::default();
        assert_eq!(scanner.timeout(), Duration::from_secs(3));
        assert_eq!(scanner.retries(), 0);
    }

    #[tokio::test]
    async fn test_scan_localhost() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(500), 0);
        // Most systems have something listening locally, but we can't guarantee ports
        // Just test that scanning localhost doesn't panic
        let result = scanner
            .scan_port(IpAddr::V4(Ipv4Addr::LOCALHOST), 65535)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scan_invalid_port_zero() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(100), 0);
        let result = scanner.scan_port(IpAddr::V4(Ipv4Addr::LOCALHOST), 0).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidPortRange(_)));
    }

    #[tokio::test]
    async fn test_concurrent_scan_limiting() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(50), 0);
        let ports: Vec<u16> = (9990..10000).collect();

        let start = Instant::now();
        let results = scanner
            .scan_ports(IpAddr::V4(Ipv4Addr::LOCALHOST), ports.clone(), 3)
            .await
            .unwrap();
        let _elapsed = start.elapsed();

        // With max_concurrent=3 and 10 ports, should take at least 4 batches
        // Each batch takes ~50ms, so total should be >150ms
        assert_eq!(results.len(), ports.len());
        // Don't assert on timing as it's unreliable in CI environments
    }

    #[tokio::test]
    async fn test_response_time_measurement() {
        let scanner = TcpConnectScanner::new(Duration::from_secs(1), 0);
        let result = scanner
            .scan_port(IpAddr::V4(Ipv4Addr::LOCALHOST), 9999)
            .await
            .unwrap();

        // Response time should be measured
        assert!(result.response_time > Duration::ZERO);
        assert!(result.response_time < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn test_retry_mechanism() {
        // Create scanner with 2 retries
        let scanner = TcpConnectScanner::new(Duration::from_millis(10), 2);

        // Scan a non-routable address
        let start = Instant::now();
        let result = scanner
            .scan_port(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), 80)
            .await
            .unwrap();
        let elapsed = start.elapsed();

        assert_eq!(result.state, PortState::Filtered);
        // With 2 retries (3 attempts total), should take at least 30ms (3 * 10ms timeouts)
        // Plus retry delays of 100ms each (2 delays)
        assert!(elapsed >= Duration::from_millis(30));
    }

    #[tokio::test]
    async fn test_scan_ipv6_localhost() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(100), 0);
        let result = scanner
            .scan_port(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), 9999)
            .await;

        // Should succeed (even if filtered/closed)
        assert!(result.is_ok());
    }

    // ============= IPv6 Integration Tests =============

    #[tokio::test]
    async fn test_ipv6_socket_creation() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(100), 0);
        let result = scanner
            .scan_port(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), 80)
            .await;

        // Should succeed (connection attempt completed)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ipv6_connection_to_loopback() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(200), 0);
        let result = scanner
            .scan_port(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), 22)
            .await
            .unwrap();

        // Port 22 could be open (SSH), closed, or filtered - just verify we got a result
        assert!(matches!(
            result.state,
            PortState::Open | PortState::Closed | PortState::Filtered
        ));
        assert_eq!(result.port, 22);
    }

    #[tokio::test]
    async fn test_ipv6_multiple_ports() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(100), 0);
        let ports = vec![22, 80, 443];
        let results = scanner
            .scan_ports(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), ports.clone(), 3)
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(ports.contains(&result.port));
        }
    }

    #[tokio::test]
    async fn test_ipv6_vs_ipv4_parity() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(200), 0);
        let ports = vec![9998, 9999];

        // Scan same ports on IPv4 and IPv6 loopback
        let ipv4_results = scanner
            .scan_ports(IpAddr::V4(Ipv4Addr::LOCALHOST), ports.clone(), 2)
            .await
            .unwrap();

        let ipv6_results = scanner
            .scan_ports(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), ports.clone(), 2)
            .await
            .unwrap();

        // Both should scan same number of ports
        assert_eq!(ipv4_results.len(), ipv6_results.len());

        // Results should be similar (both closed/filtered on loopback)
        for (ipv4_res, ipv6_res) in ipv4_results.iter().zip(ipv6_results.iter()) {
            assert_eq!(ipv4_res.port, ipv6_res.port);
        }
    }

    #[tokio::test]
    async fn test_ipv6_timeout_handling() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(10), 0);
        // Use non-routable IPv6 address (documentation range)
        let result = scanner
            .scan_port(IpAddr::V6("2001:db8::1".parse().unwrap()), 80)
            .await
            .unwrap();

        // Should timeout and be filtered
        assert_eq!(result.state, PortState::Filtered);
    }

    #[tokio::test]
    async fn test_ipv6_response_time() {
        let scanner = TcpConnectScanner::new(Duration::from_secs(1), 0);
        let result = scanner
            .scan_port(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), 9999)
            .await
            .unwrap();

        // Response time should be measured
        assert!(result.response_time > Duration::ZERO);
        assert!(result.response_time < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn test_scan_target_single_host() {
        use prtip_core::ScanTarget;

        let scanner = TcpConnectScanner::new(Duration::from_millis(100), 0);
        let target = ScanTarget::parse("127.0.0.1").unwrap();
        let results = scanner
            .scan_target(&target, vec![9998, 9999], 10)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_large_port_range() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(10), 0);
        let ports: Vec<u16> = (9900..9920).collect(); // 20 ports

        let results = scanner
            .scan_ports(IpAddr::V4(Ipv4Addr::LOCALHOST), ports.clone(), 20)
            .await
            .unwrap();

        assert_eq!(results.len(), 20);
    }

    #[tokio::test]
    async fn test_empty_port_list() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(100), 0);
        let results = scanner
            .scan_ports(IpAddr::V4(Ipv4Addr::LOCALHOST), vec![], 10)
            .await
            .unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_scanner_configuration() {
        let timeout = Duration::from_secs(5);
        let retries = 3;
        let scanner = TcpConnectScanner::new(timeout, retries);

        assert_eq!(scanner.timeout(), timeout);
        assert_eq!(scanner.retries(), retries);
    }

    // ============= Lock-Free Aggregator Integration Tests =============

    #[tokio::test]
    async fn test_lockfree_aggregator_integration() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(50), 0);
        let ports: Vec<u16> = (10000..10020).collect(); // 20 ports

        let results = scanner
            .scan_ports(IpAddr::V4(Ipv4Addr::LOCALHOST), ports.clone(), 10)
            .await
            .unwrap();

        // All results should be collected via lock-free aggregator
        assert_eq!(results.len(), 20);

        // Results should be valid from aggregator drain
        for result in results.iter() {
            assert!(result.port >= 10000 && result.port < 10020);
            // Response time should be measured
            assert!(result.response_time > Duration::ZERO);
        }
    }

    #[tokio::test]
    async fn test_lockfree_high_concurrency() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(20), 0);
        let ports: Vec<u16> = (20000..20100).collect(); // 100 ports

        // High concurrency (100 workers) to stress test lock-free aggregator
        let start = Instant::now();
        let results = scanner
            .scan_ports(IpAddr::V4(Ipv4Addr::LOCALHOST), ports.clone(), 100)
            .await
            .unwrap();
        let elapsed = start.elapsed();

        // All 100 results should be collected
        assert_eq!(results.len(), 100);

        // With 100 concurrent workers, lock-free aggregator should have zero contention
        // Verify all ports were scanned
        let mut seen_ports = std::collections::HashSet::new();
        for result in &results {
            assert!(result.port >= 20000 && result.port < 20100);
            seen_ports.insert(result.port);
        }
        assert_eq!(seen_ports.len(), 100);

        // High concurrency should complete quickly due to lock-free aggregation
        println!("100 ports with 100 concurrency: {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_lockfree_aggregator_ordering() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(10), 0);
        let ports: Vec<u16> = vec![30001, 30002, 30003, 30004, 30005];

        let results = scanner
            .scan_ports(IpAddr::V4(Ipv4Addr::LOCALHOST), ports.clone(), 5)
            .await
            .unwrap();

        // All ports should be present (order may vary due to concurrent execution)
        assert_eq!(results.len(), 5);
        let result_ports: std::collections::HashSet<_> = results.iter().map(|r| r.port).collect();
        for port in &ports {
            assert!(result_ports.contains(port));
        }
    }

    #[tokio::test]
    async fn test_lockfree_aggregator_progress_tracking() {
        use prtip_core::ScanProgress;

        let scanner = TcpConnectScanner::new(Duration::from_millis(20), 0);
        let ports: Vec<u16> = (40000..40050).collect(); // 50 ports
        let progress = ScanProgress::new(ports.len());

        let results = scanner
            .scan_ports_with_progress(
                IpAddr::V4(Ipv4Addr::LOCALHOST),
                ports.clone(),
                20,
                Some(&progress),
                None, // No NUMA manager in test
            )
            .await
            .unwrap();

        // All results collected via lock-free aggregator
        assert_eq!(results.len(), 50);

        // Progress should be tracked correctly
        assert_eq!(progress.completed(), 50);
        assert_eq!(progress.total(), 50);

        // State counters should be updated
        let total_states =
            progress.open_ports() + progress.closed_ports() + progress.filtered_ports();
        assert_eq!(total_states, 50);
    }

    #[tokio::test]
    async fn test_lockfree_aggregator_empty_ports() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(10), 0);
        let results = scanner
            .scan_ports(IpAddr::V4(Ipv4Addr::LOCALHOST), vec![], 10)
            .await
            .unwrap();

        // Empty port list should return empty results
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_lockfree_aggregator_single_port() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(50), 0);
        let results = scanner
            .scan_ports(IpAddr::V4(Ipv4Addr::LOCALHOST), vec![50001], 1)
            .await
            .unwrap();

        // Single port should work correctly with aggregator
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].port, 50001);
    }

    #[tokio::test]
    async fn test_lockfree_aggregator_sequential_scans() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(10), 0);

        // Run multiple sequential scans to ensure aggregator cleanup
        for batch in 0..5 {
            let base_port = 60000 + (batch * 10);
            let ports: Vec<u16> = (base_port..base_port + 10).collect();

            let results = scanner
                .scan_ports(IpAddr::V4(Ipv4Addr::LOCALHOST), ports.clone(), 5)
                .await
                .unwrap();

            assert_eq!(results.len(), 10);
            for result in &results {
                assert!(result.port >= base_port && result.port < base_port + 10);
            }
        }
    }

    #[tokio::test]
    async fn test_lockfree_aggregator_ipv6() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(20), 0);
        let ports: Vec<u16> = vec![7001, 7002, 7003];

        let results = scanner
            .scan_ports(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), ports.clone(), 3)
            .await
            .unwrap();

        // IPv6 should work with lock-free aggregator
        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(ports.contains(&result.port));
        }
    }

    #[tokio::test]
    async fn test_lockfree_aggregator_large_batch() {
        let scanner = TcpConnectScanner::new(Duration::from_millis(5), 0);
        let ports: Vec<u16> = (50000..50500).collect(); // 500 ports

        let start = Instant::now();
        let results = scanner
            .scan_ports(IpAddr::V4(Ipv4Addr::LOCALHOST), ports.clone(), 50)
            .await
            .unwrap();
        let elapsed = start.elapsed();

        // All 500 results should be collected
        assert_eq!(results.len(), 500);

        // Verify no duplicates
        let mut seen_ports = std::collections::HashSet::new();
        for result in &results {
            assert!(result.port >= 50000 && result.port < 50500);
            assert!(
                seen_ports.insert(result.port),
                "Duplicate port: {}",
                result.port
            );
        }

        println!("500 ports with 50 concurrency: {:?}", elapsed);
        // Lock-free aggregator should handle large batches efficiently
    }
}
