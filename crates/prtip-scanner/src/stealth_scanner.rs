//! Stealth TCP scan techniques
//!
//! This module implements various stealth scanning techniques that exploit
//! different interpretations of the TCP RFC to determine port states.
//!
//! ## Scan Types
//!
//! ### FIN Scan (-sF)
//! Sends packets with only the FIN flag set. Per RFC 793:
//! - **Open ports**: No response
//! - **Closed ports**: RST response
//! - **Filtered**: ICMP unreachable
//!
//! ### NULL Scan (-sN)
//! Sends packets with no flags set:
//! - **Open ports**: No response
//! - **Closed ports**: RST response
//! - **Filtered**: ICMP unreachable
//!
//! ### Xmas Scan (-sX)
//! Sends packets with FIN, PSH, and URG flags ("lit up like a Christmas tree"):
//! - **Open ports**: No response
//! - **Closed ports**: RST response
//! - **Filtered**: ICMP unreachable
//!
//! ### ACK Scan (-sA)
//! Used for firewall rule detection, not port state:
//! - **Unfiltered**: RST response
//! - **Filtered**: No response or ICMP unreachable
//!
//! ## Limitations
//!
//! - **Windows**: Doesn't work (sends RST regardless of port state)
//! - **Cisco**: Many devices send RST for all ports
//! - **Stateful Firewalls**: May detect these unusual flag combinations
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::{StealthScanner, StealthScanType};
//! use prtip_core::Config;
//! use std::net::{IpAddr, Ipv4Addr};
//!
//! # async fn example() -> prtip_core::Result<()> {
//! let config = Config::default();
//! let scanner = StealthScanner::new(config)?;
//!
//! let target = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
//! let result = scanner.scan_port(target, 80, StealthScanType::Fin).await?;
//!
//! println!("FIN scan result: {:?}", result.state);
//! # Ok(())
//! # }
//! ```

use crate::AdaptiveRateLimiterV2;
use dashmap::DashMap;
use parking_lot::Mutex;
use prtip_core::{Config, EventBus, PortState, Protocol, Result, ScanEvent, ScanResult, ScanType};
use prtip_network::{
    adaptive_batch::AdaptiveConfig, create_capture, with_buffer, PacketCapture,
    PlatformCapabilities, TcpFlags, TcpPacketBuilder,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::time::timeout;
use tracing::{debug, trace, warn};
use uuid::Uuid;

// PCAPNG packet capture support
use crate::pcapng::{Direction, PcapngWriter};
use std::sync::Mutex as StdMutex;

/// Connection tracking state for batch stealth scanning
/// Sprint 6.3 Task 2.3: Added for batch I/O integration
/// Note: Target IP, port, source port, and scan type are stored in the DashMap key
#[derive(Debug, Clone)]
struct ConnectionState {
    sent_time: Instant,
}

/// Type of stealth scan to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StealthScanType {
    /// FIN scan - only FIN flag set
    Fin,
    /// NULL scan - no flags set
    Null,
    /// Xmas scan - FIN, PSH, URG flags set
    Xmas,
    /// ACK scan - only ACK flag set (for firewall detection)
    Ack,
}

impl StealthScanType {
    /// Get the TCP flags for this scan type
    pub fn flags(&self) -> TcpFlags {
        match self {
            StealthScanType::Fin => TcpFlags::FIN,
            StealthScanType::Null => TcpFlags::empty(),
            StealthScanType::Xmas => TcpFlags::FIN.combine(TcpFlags::PSH).combine(TcpFlags::URG),
            StealthScanType::Ack => TcpFlags::ACK,
        }
    }

    /// Get the name of this scan type
    pub fn name(&self) -> &'static str {
        match self {
            StealthScanType::Fin => "FIN",
            StealthScanType::Null => "NULL",
            StealthScanType::Xmas => "Xmas",
            StealthScanType::Ack => "ACK",
        }
    }

    /// Convert to ScanType for event publishing
    /// Sprint 6.3 Task 2.3: Added for event system integration
    pub fn to_scan_type(&self) -> ScanType {
        match self {
            StealthScanType::Fin => ScanType::Fin,
            StealthScanType::Null => ScanType::Null,
            StealthScanType::Xmas => ScanType::Xmas,
            StealthScanType::Ack => ScanType::Ack,
        }
    }
}

/// Stealth scanner
/// Sprint 5.1 Phase 2.2: Enhanced with dual-stack IPv4/IPv6 support
///
/// # Rate Limiting (Sprint 5.4 Phase 1)
///
/// Supports optional adaptive rate limiting:
/// - Adaptive limiter provides per-target ICMP backoff
/// - Note: Hostgroup limiting handled by scheduler (per-port scanner)
pub struct StealthScanner {
    config: Config,
    capture: Arc<Mutex<Option<Box<dyn PacketCapture>>>>,
    /// Local IPv4 address for IPv4 scans
    local_ipv4: Ipv4Addr,
    /// Local IPv6 address for IPv6 scans (if available)
    local_ipv6: Option<Ipv6Addr>,
    /// Optional adaptive rate limiter (ICMP-aware throttling)
    adaptive_limiter: Option<Arc<AdaptiveRateLimiterV2>>,
    /// Optional event bus for real-time progress updates
    event_bus: Option<Arc<EventBus>>,
    /// Connection tracking for batch I/O (Sprint 6.3 Task 2.3)
    /// Key: (target_ip, target_port, source_port, scan_type)
    connections: Arc<DashMap<(IpAddr, u16, u16, StealthScanType), ConnectionState>>,
}

impl StealthScanner {
    /// Create a new stealth scanner
    /// Sprint 5.1 Phase 2.2: Enhanced to detect both IPv4 and IPv6 local addresses
    pub fn new(config: Config) -> Result<Self> {
        let local_ipv4 = Self::detect_local_ipv4()?;
        let local_ipv6 = Self::detect_local_ipv6();

        Ok(Self {
            config,
            capture: Arc::new(Mutex::new(None)),
            local_ipv4,
            local_ipv6,
            adaptive_limiter: None,
            event_bus: None,
            connections: Arc::new(DashMap::new()), // Sprint 6.3 Task 2.3: Batch I/O connection tracking
        })
    }

    /// Enable adaptive rate limiting (ICMP-aware throttling)
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

    /// Initialize packet capture
    pub async fn initialize(&mut self) -> Result<()> {
        let mut capture = create_capture()?;
        capture.open(None)?;
        *self.capture.lock() = Some(capture);
        Ok(())
    }

    /// Detect local IPv4 address for the interface
    fn detect_local_ipv4() -> Result<Ipv4Addr> {
        // Simplified detection - in production would use interface detection
        Ok(Ipv4Addr::new(192, 168, 1, 100))
    }

    /// Detect local IPv6 address for the interface
    /// Returns None if no IPv6 address is available
    fn detect_local_ipv6() -> Option<Ipv6Addr> {
        // Simplified detection - in production would use interface detection
        // Use link-local placeholder (fe80::1)
        Some(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1))
    }

    /// Get appropriate local IP address for target
    /// Returns IPv4 address for IPv4 targets, IPv6 address for IPv6 targets
    fn get_local_ip_for_target(&self, target: IpAddr) -> Result<IpAddr> {
        match target {
            IpAddr::V4(_) => Ok(IpAddr::V4(self.local_ipv4)),
            IpAddr::V6(_) => self.local_ipv6.map(IpAddr::V6).ok_or_else(|| {
                prtip_core::Error::Config("No IPv6 address available for IPv6 scan".to_string())
            }),
        }
    }

    /// Scan multiple ports using batch I/O operations
    /// Sprint 6.3 Task 2.3: Added for batch I/O integration
    pub async fn scan_ports(
        &self,
        target: IpAddr,
        ports: Vec<u16>,
        scan_type: StealthScanType,
    ) -> Result<Vec<ScanResult>> {
        // Generate scan ID for event tracking (Sprint 6.3 Task 2.3)
        let scan_id = Uuid::new_v4();
        // Step 1: Check ICMP backoff (if adaptive rate limiting enabled)
        if let Some(limiter) = &self.adaptive_limiter {
            if limiter.is_target_backed_off(target) {
                debug!(
                    "Skipping {} ports on {} (ICMP backoff active)",
                    ports.len(),
                    target
                );
                return Ok(ports
                    .iter()
                    .map(|&port| ScanResult::new(target, port, PortState::Filtered))
                    .collect());
            }
        }

        // Step 2: Detect platform capabilities
        let caps = PlatformCapabilities::detect();

        // Step 3: Use fallback for platforms without sendmmsg/recvmmsg
        if !caps.has_sendmmsg || !caps.has_recvmmsg {
            return self.scan_ports_fallback(target, ports, scan_type).await;
        }

        // Step 4: Calculate optimal batch size
        let batch_size = self.calculate_batch_size(1, ports.len());

        // Step 5: Get network interface
        let interface = self.config.network.interface.as_deref().unwrap_or("eth0");

        // Step 6: Create adaptive config if enabled
        let adaptive_config = if self.config.performance.adaptive_batch_enabled {
            Some(AdaptiveConfig {
                min_batch_size: self.config.performance.min_batch_size,
                max_batch_size: self.config.performance.max_batch_size,
                increase_threshold: 0.95,
                decrease_threshold: 0.85,
                memory_limit: 100_000_000, // 100 MB
                window_size: Duration::from_secs(5),
            })
        } else {
            None
        };

        // Step 7: Create BatchSender and BatchReceiver
        let mut sender = prtip_network::BatchSender::new(interface, batch_size, adaptive_config)
            .map_err(|e| {
                prtip_core::Error::Network(format!("Failed to create BatchSender: {}", e))
            })?;

        let mut receiver =
            prtip_network::BatchReceiver::new(interface, batch_size).map_err(|e| {
                prtip_core::Error::Network(format!("Failed to create BatchReceiver: {}", e))
            })?;

        // Step 8: Process ports in batches
        let mut all_results = Vec::new();
        let timeout_ms = self.config.timing().timeout_ms();

        for chunk in ports.chunks(batch_size) {
            // Step 8a: Prepare batch packets
            let packets = self.prepare_batch(target, chunk, scan_type)?;

            // Step 8b: Add packets to sender
            for packet in &packets {
                sender.add_packet(packet.clone()).map_err(|e| {
                    prtip_core::Error::Network(format!("Failed to add packet: {}", e))
                })?;
            }

            // Step 8c: Flush batch (with 3 retries)
            sender
                .flush(3)
                .await
                .map_err(|e| prtip_core::Error::Network(format!("Failed to flush batch: {}", e)))?;

            // Step 8d: Receive batch responses
            let responses = receiver
                .receive_batch(timeout_ms as u32)
                .await
                .map_err(|e| {
                    prtip_core::Error::Network(format!("Failed to receive batch: {}", e))
                })?;

            // Step 8e: Process responses
            let response_data: Vec<Vec<u8>> = responses.into_iter().map(|r| r.data).collect();
            let batch_results = self
                .process_batch_responses(response_data, scan_type, scan_id)
                .await?;
            all_results.extend(batch_results);

            // Step 8f: Mark remaining ports as open|filtered (no RST = open|filtered for stealth)
            let responded_ports: std::collections::HashSet<u16> = all_results
                .iter()
                .filter(|r| r.target_ip == target)
                .map(|r| r.port)
                .collect();

            for &port in chunk {
                if !responded_ports.contains(&port) {
                    // No RST received = port is open or filtered
                    all_results.push(ScanResult::new(target, port, PortState::Open));
                }
            }
        }

        Ok(all_results)
    }

    /// Fallback scan_ports for macOS/Windows (no batch I/O support)
    /// Sprint 6.3 Task 2.3: Added for platform compatibility
    async fn scan_ports_fallback(
        &self,
        target: IpAddr,
        ports: Vec<u16>,
        scan_type: StealthScanType,
    ) -> Result<Vec<ScanResult>> {
        let mut results = Vec::new();

        for port in ports {
            match self.scan_port(target, port, scan_type).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    debug!("Error scanning {}:{} - {}", target, port, e);
                    results.push(ScanResult::new(target, port, PortState::Filtered));
                }
            }
        }

        Ok(results)
    }

    /// Scan a single port with specified stealth technique
    /// Sprint 5.1 Phase 2.2: Updated to accept IpAddr for dual-stack support
    pub async fn scan_port(
        &self,
        target: IpAddr,
        port: u16,
        scan_type: StealthScanType,
    ) -> Result<ScanResult> {
        self.scan_port_with_pcapng(target, port, scan_type, None)
            .await
    }

    /// Scan a single port with specified stealth technique and optional PCAPNG capture
    /// Sprint 5.1 Phase 2.2: Updated to accept IpAddr for dual-stack support
    pub async fn scan_port_with_pcapng(
        &self,
        target: IpAddr,
        port: u16,
        scan_type: StealthScanType,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<ScanResult> {
        // Generate scan ID for event tracking
        let scan_id = Uuid::new_v4();

        // Check ICMP backoff (if adaptive rate limiting enabled)
        if let Some(limiter) = &self.adaptive_limiter {
            if limiter.is_target_backed_off(target) {
                debug!("Skipping {}:{} (ICMP backoff active)", target, port);

                // Emit rate limit triggered event
                if let Some(bus) = &self.event_bus {
                    bus.publish(ScanEvent::RateLimitTriggered {
                        scan_id,
                        current_rate: 0.0,
                        target_rate: 0.0,
                        duration_ms: 0,
                        timestamp: SystemTime::now(),
                    })
                    .await;
                }

                return Ok(ScanResult::new(target, port, PortState::Filtered));
            }
        }

        let start_time = Instant::now();

        // Use configured source port or generate random
        use rand::Rng;
        let src_port: u16 = self
            .config
            .network
            .source_port
            .unwrap_or_else(|| rand::thread_rng().gen_range(1024..65535));

        // Send probe
        self.send_probe(target, port, src_port, scan_type, pcapng_writer.clone())
            .await?;

        // Wait for response
        let timeout_ms = self.config.scan.timeout_ms;
        let wait_duration = Duration::from_millis(timeout_ms);

        let result = match timeout(
            wait_duration,
            self.wait_for_response(target, port, src_port, scan_type, pcapng_writer),
        )
        .await
        {
            Ok(Ok(state)) => {
                let response_time = start_time.elapsed();

                // Emit PortFound event for open ports
                if state == PortState::Open {
                    if let Some(bus) = &self.event_bus {
                        let stealth_scan_type = match scan_type {
                            StealthScanType::Fin => ScanType::Fin,
                            StealthScanType::Null => ScanType::Null,
                            StealthScanType::Xmas => ScanType::Xmas,
                            StealthScanType::Ack => ScanType::Ack,
                        };

                        bus.publish(ScanEvent::PortFound {
                            scan_id,
                            ip: target,
                            port,
                            state,
                            protocol: Protocol::Tcp,
                            scan_type: stealth_scan_type,
                            timestamp: SystemTime::now(),
                        })
                        .await;
                    }
                }

                Ok(ScanResult::new(target, port, state).with_response_time(response_time))
            }
            Ok(Err(e)) => {
                warn!("Error waiting for response: {}", e);

                // Emit warning event
                if let Some(bus) = &self.event_bus {
                    bus.publish(ScanEvent::WarningIssued {
                        scan_id,
                        message: format!(
                            "Error waiting for response from {}:{}: {}",
                            target, port, e
                        ),
                        severity: prtip_core::WarningSeverity::Medium,
                        timestamp: SystemTime::now(),
                    })
                    .await;
                }

                let response_time = start_time.elapsed();
                Ok(ScanResult::new(target, port, PortState::Unknown)
                    .with_response_time(response_time))
            }
            Err(_) => {
                // Timeout - interpretation depends on scan type
                let state = match scan_type {
                    StealthScanType::Ack => {
                        // For ACK scan, no response = filtered
                        debug!("No response from {}:{} (ACK scan) - FILTERED", target, port);
                        PortState::Filtered
                    }
                    _ => {
                        // For FIN/NULL/Xmas, no response = open|filtered
                        debug!(
                            "No response from {}:{} ({} scan) - OPEN|FILTERED",
                            target,
                            port,
                            scan_type.name()
                        );
                        PortState::Filtered // Could also be open
                    }
                };

                let response_time = start_time.elapsed();
                Ok(ScanResult::new(target, port, state).with_response_time(response_time))
            }
        };

        result
    }

    /// Send a stealth probe packet (zero-copy with optional PCAPNG capture)
    /// Sprint 5.1 Phase 2.2: Updated for dual-stack IPv4/IPv6 support
    async fn send_probe(
        &self,
        target: IpAddr,
        port: u16,
        src_port: u16,
        scan_type: StealthScanType,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<()> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let sequence: u32 = rng.gen();

        // Get appropriate local IP for target
        let local_ip = self.get_local_ip_for_target(target)?;

        // Build and send packet (dispatch based on IP version)
        match (local_ip, target) {
            (IpAddr::V4(src_ipv4), IpAddr::V4(dst_ipv4)) => {
                // IPv4 stealth packet
                self.send_probe_ipv4(
                    dst_ipv4,
                    src_ipv4,
                    port,
                    src_port,
                    sequence,
                    scan_type,
                    pcapng_writer,
                )
                .await
            }
            (IpAddr::V6(src_ipv6), IpAddr::V6(dst_ipv6)) => {
                // IPv6 stealth packet
                self.send_probe_ipv6(
                    dst_ipv6,
                    src_ipv6,
                    port,
                    src_port,
                    sequence,
                    scan_type,
                    pcapng_writer,
                )
                .await
            }
            _ => Err(prtip_core::Error::Config(format!(
                "IP version mismatch: local {} vs target {}",
                local_ip, target
            ))),
        }
    }

    /// Send IPv4 stealth probe packet (zero-copy with optional PCAPNG capture)
    #[allow(clippy::too_many_arguments)]
    async fn send_probe_ipv4(
        &self,
        target: Ipv4Addr,
        local_ip: Ipv4Addr,
        port: u16,
        src_port: u16,
        sequence: u32,
        scan_type: StealthScanType,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<()> {
        if let Some(ref mut capture) = *self.capture.lock() {
            // Use thread-local zero-copy buffer
            with_buffer(|buffer_pool| {
                // Build packet in buffer
                let mut builder = TcpPacketBuilder::new()
                    .source_ip(local_ip)
                    .dest_ip(target)
                    .source_port(src_port)
                    .dest_port(port)
                    .sequence(sequence)
                    .flags(scan_type.flags())
                    .window(1024);

                // Apply TTL if configured (Sprint 4.20: Evasion features)
                if let Some(ttl) = self.config.evasion.ttl {
                    builder = builder.ttl(ttl);
                }

                // Apply bad checksum if configured (Sprint 4.20 Phase 6: Bad checksum)
                if self.config.evasion.bad_checksums {
                    builder = builder.bad_checksum(true);
                }

                let packet_slice = builder.build_ip_packet_with_buffer(buffer_pool)?;

                // Sprint 4.20: Check if packet fragmentation is enabled
                let packets_to_send: Vec<Vec<u8>> = if self.config.evasion.fragment_packets {
                    // Fragment the packet using configured MTU
                    use prtip_network::fragment_tcp_packet;
                    let mtu = self.config.evasion.mtu.unwrap_or(1500);
                    let packet_data = packet_slice.to_vec(); // Copy from pool for fragmentation
                    fragment_tcp_packet(&packet_data, mtu).map_err(|e| {
                        prtip_core::Error::Network(format!("Fragmentation failed: {}", e))
                    })?
                } else {
                    // No fragmentation - send as single packet
                    vec![packet_slice.to_vec()]
                };

                // Capture packets to PCAPNG if writer is provided
                if let Some(ref writer) = pcapng_writer {
                    for packet_data in &packets_to_send {
                        if let Ok(guard) = writer.lock() {
                            if let Err(e) = guard.write_packet(packet_data, Direction::Sent) {
                                // Log error but don't fail scan (PCAPNG is optional)
                                warn!("PCAPNG write error ({} probe): {}", scan_type.name(), e);
                            }
                        }
                    }
                }

                // Send packet(s) (fragmented or whole)
                for fragment in &packets_to_send {
                    capture.send_packet(fragment).map_err(|e| {
                        prtip_core::Error::Network(format!(
                            "Failed to send {} probe: {}",
                            scan_type.name(),
                            e
                        ))
                    })?;
                }

                if self.config.evasion.fragment_packets {
                    trace!(
                        "Sent {} fragmented {} probes to {}:{} (src_port={})",
                        packets_to_send.len(),
                        scan_type.name(),
                        target,
                        port,
                        src_port
                    );
                } else {
                    trace!(
                        "Sent {} probe to {}:{} (src_port={}) [zero-copy]",
                        scan_type.name(),
                        target,
                        port,
                        src_port
                    );
                }

                // Reset buffer
                buffer_pool.reset();

                Ok::<(), prtip_core::Error>(())
            })?;
        } else {
            return Err(prtip_core::Error::Config(
                "Packet capture not initialized".to_string(),
            ));
        }

        Ok(())
    }

    /// Send IPv6 stealth probe packet
    /// Sprint 5.1 Phase 2.2: IPv6 implementation for all 4 stealth scan types
    #[allow(clippy::too_many_arguments)]
    async fn send_probe_ipv6(
        &self,
        target: Ipv6Addr,
        local_ip: Ipv6Addr,
        port: u16,
        src_port: u16,
        sequence: u32,
        scan_type: StealthScanType,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<()> {
        // Build IPv6 TCP packet with scan-specific flags
        let mut builder = TcpPacketBuilder::new()
            .source_port(src_port)
            .dest_port(port)
            .sequence(sequence)
            .flags(scan_type.flags())
            .window(1024);

        // Apply hop limit if configured (IPv6 equivalent of TTL)
        if let Some(ttl) = self.config.evasion.ttl {
            builder = builder.ttl(ttl);
        }

        // Apply bad checksum if configured
        if self.config.evasion.bad_checksums {
            builder = builder.bad_checksum(true);
        }

        // Build IPv6+TCP packet (no fragmentation support for IPv6 yet - Sprint 5.1 Phase 1)
        let packet = builder.build_ipv6_packet(local_ip, target)?;

        // Capture packet to PCAPNG if writer is provided
        if let Some(ref writer) = pcapng_writer {
            if let Ok(guard) = writer.lock() {
                if let Err(e) = guard.write_packet(&packet, Direction::Sent) {
                    warn!(
                        "PCAPNG write error (IPv6 {} probe): {}",
                        scan_type.name(),
                        e
                    );
                }
            }
        }

        // Send packet
        if let Some(ref mut capture) = *self.capture.lock() {
            capture.send_packet(&packet).map_err(|e| {
                prtip_core::Error::Network(format!(
                    "Failed to send IPv6 {} probe: {}",
                    scan_type.name(),
                    e
                ))
            })?;

            trace!(
                "Sent IPv6 {} probe to {}:{} (src_port={})",
                scan_type.name(),
                target,
                port,
                src_port
            );
        } else {
            return Err(prtip_core::Error::Config(
                "Packet capture not initialized".to_string(),
            ));
        }

        Ok(())
    }

    /// Wait for response with optional PCAPNG capture
    /// Sprint 5.1 Phase 2.2: Updated for dual-stack IPv4/IPv6 support
    async fn wait_for_response(
        &self,
        target: IpAddr,
        port: u16,
        src_port: u16,
        scan_type: StealthScanType,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<PortState> {
        loop {
            if let Some(ref mut capture) = *self.capture.lock() {
                if let Some(packet) = capture.receive_packet(100)? {
                    // Capture received packet to PCAPNG if writer is provided
                    if let Some(ref writer) = pcapng_writer {
                        if let Ok(guard) = writer.lock() {
                            if let Err(e) = guard.write_packet(&packet, Direction::Received) {
                                // Log error but don't fail scan (PCAPNG is optional)
                                warn!("PCAPNG write error ({} response): {}", scan_type.name(), e);
                            }
                        }
                    }

                    if let Some(state) =
                        self.parse_response(&packet, target, port, src_port, scan_type)?
                    {
                        return Ok(state);
                    }
                }
            }

            tokio::task::yield_now().await;
        }
    }

    /// Parse received packet
    /// Sprint 5.1 Phase 2.2: Updated for dual-stack IPv4/IPv6 support
    fn parse_response(
        &self,
        packet: &[u8],
        target: IpAddr,
        port: u16,
        src_port: u16,
        scan_type: StealthScanType,
    ) -> Result<Option<PortState>> {
        use pnet::packet::{ethernet::EthernetPacket, Packet};

        let eth_packet = match EthernetPacket::new(packet) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Match on target IP version and parse accordingly
        match target {
            IpAddr::V4(target_ipv4) => self.parse_ipv4_response(
                eth_packet.payload(),
                target_ipv4,
                port,
                src_port,
                scan_type,
            ),
            IpAddr::V6(target_ipv6) => self.parse_ipv6_response(
                eth_packet.payload(),
                target_ipv6,
                port,
                src_port,
                scan_type,
            ),
        }
    }

    /// Parse IPv4 stealth response
    fn parse_ipv4_response(
        &self,
        payload: &[u8],
        target: Ipv4Addr,
        port: u16,
        src_port: u16,
        scan_type: StealthScanType,
    ) -> Result<Option<PortState>> {
        use pnet::packet::{icmp::IcmpPacket, ipv4::Ipv4Packet, tcp::TcpPacket, Packet};

        let ipv4_packet = match Ipv4Packet::new(payload) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Check if it's from our target
        if ipv4_packet.get_source() != target {
            return Ok(None);
        }

        // Check protocol
        match ipv4_packet.get_next_level_protocol().0 {
            6 => {
                // TCP response
                if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                    if tcp_packet.get_source() != port || tcp_packet.get_destination() != src_port {
                        return Ok(None);
                    }

                    let flags = tcp_packet.get_flags();

                    // RST response
                    if (flags & 0x04) == 0x04 {
                        match scan_type {
                            StealthScanType::Ack => {
                                // For ACK scan, RST = unfiltered
                                debug!(
                                    "Received RST from {}:{} (ACK scan) - UNFILTERED",
                                    target, port
                                );
                                return Ok(Some(PortState::Open)); // Using "Open" to mean "Unfiltered"
                            }
                            _ => {
                                // For FIN/NULL/Xmas, RST = closed
                                debug!(
                                    "Received RST from {}:{} ({} scan) - CLOSED",
                                    target,
                                    port,
                                    scan_type.name()
                                );
                                return Ok(Some(PortState::Closed));
                            }
                        }
                    }
                }
            }
            1 => {
                // ICMP response
                if let Some(icmp_packet) = IcmpPacket::new(ipv4_packet.payload()) {
                    let icmp_type = icmp_packet.get_icmp_type().0;
                    let _icmp_code = icmp_packet.get_icmp_code().0;

                    // ICMP type 3 = Destination Unreachable
                    if icmp_type == 3 {
                        debug!(
                            "Received ICMP unreachable from {}:{} ({} scan) - FILTERED",
                            target,
                            port,
                            scan_type.name()
                        );
                        return Ok(Some(PortState::Filtered));
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Parse IPv6 stealth response
    /// Sprint 5.1 Phase 2.2: IPv6 RST response parsing for all 4 stealth scan types
    fn parse_ipv6_response(
        &self,
        payload: &[u8],
        target: Ipv6Addr,
        port: u16,
        src_port: u16,
        scan_type: StealthScanType,
    ) -> Result<Option<PortState>> {
        use pnet::packet::ip::IpNextHeaderProtocols;
        use pnet::packet::{ipv6::Ipv6Packet, tcp::TcpPacket, Packet};

        let ipv6_packet = match Ipv6Packet::new(payload) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Check if it's from our target
        if ipv6_packet.get_source() != target {
            return Ok(None);
        }

        // Sprint 5.1 Phase 2.2: Skip extension headers to find TCP
        // Note: This is a simplified implementation - production should handle all extension header types
        let tcp_payload = ipv6_packet.payload();
        let next_header = ipv6_packet.get_next_header();

        // Check if next header is TCP (protocol 6)
        if next_header != IpNextHeaderProtocols::Tcp {
            // TODO Sprint 5.1 Phase 2.3: Handle extension headers (Fragment, Hop-by-Hop, Routing, Destination Options)
            // For now, only support direct TCP (no extension headers)
            trace!("IPv6 packet with non-TCP next header: {}", next_header.0);
            return Ok(None);
        }

        // Parse TCP packet
        let tcp_packet = match TcpPacket::new(tcp_payload) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Check if it matches our connection
        if tcp_packet.get_source() != port || tcp_packet.get_destination() != src_port {
            return Ok(None);
        }

        // Determine state based on flags
        let flags = tcp_packet.get_flags();

        // RST response
        if (flags & 0x04) == 0x04 {
            match scan_type {
                StealthScanType::Ack => {
                    // For ACK scan, RST = unfiltered
                    debug!(
                        "Received IPv6 RST from {}:{} (ACK scan) - UNFILTERED",
                        target, port
                    );
                    return Ok(Some(PortState::Open)); // Using "Open" to mean "Unfiltered"
                }
                _ => {
                    // For FIN/NULL/Xmas, RST = closed
                    debug!(
                        "Received IPv6 RST from {}:{} ({} scan) - CLOSED",
                        target,
                        port,
                        scan_type.name()
                    );
                    return Ok(Some(PortState::Closed));
                }
            }
        }

        Ok(None)
    }

    /// Prepare a batch of stealth packets for sending
    /// Sprint 6.3 Task 2.3: Added for batch I/O integration
    fn prepare_batch(
        &self,
        target: IpAddr,
        ports: &[u16],
        scan_type: StealthScanType,
    ) -> Result<Vec<Vec<u8>>> {
        let mut packets = Vec::with_capacity(ports.len());

        for &port in ports {
            // Generate random source port
            let src_port = {
                use rand::Rng;
                rand::thread_rng().gen_range(32768..=61000)
            };

            // Build packet based on target IP version
            let packet = match target {
                IpAddr::V4(dst_ipv4) => self.build_stealth_ipv4_packet(
                    self.local_ipv4,
                    dst_ipv4,
                    src_port,
                    port,
                    scan_type,
                )?,
                IpAddr::V6(dst_ipv6) => {
                    let src_ipv6 = self.local_ipv6.ok_or_else(|| {
                        prtip_core::Error::Network(
                            "IPv6 scan requested but no local IPv6 address available".to_string(),
                        )
                    })?;
                    self.build_stealth_ipv6_packet(src_ipv6, dst_ipv6, src_port, port, scan_type)?
                }
            };

            // Track connection state
            let state = ConnectionState {
                sent_time: Instant::now(),
            };
            self.connections
                .insert((target, port, src_port, scan_type), state);

            packets.push(packet);
        }

        Ok(packets)
    }

    /// Build IPv4 stealth packet with specified flags
    /// Sprint 6.3 Task 2.3: Added for batch I/O integration
    fn build_stealth_ipv4_packet(
        &self,
        src_ipv4: Ipv4Addr,
        dst_ipv4: Ipv4Addr,
        src_port: u16,
        dst_port: u16,
        scan_type: StealthScanType,
    ) -> Result<Vec<u8>> {
        with_buffer(|buffer_pool| {
            let mut builder = TcpPacketBuilder::new()
                .source_ip(src_ipv4)
                .dest_ip(dst_ipv4)
                .source_port(src_port)
                .dest_port(dst_port)
                .flags(scan_type.flags())
                .sequence(0)
                .acknowledgment(0)
                .window(1024);

            if let Some(ttl) = self.config.evasion.ttl {
                builder = builder.ttl(ttl);
            }

            if self.config.evasion.bad_checksums {
                builder = builder.bad_checksum(true);
            }

            let packet_slice = builder.build_ip_packet_with_buffer(buffer_pool)?;
            let packet = packet_slice.to_vec();
            buffer_pool.reset();

            Ok::<Vec<u8>, prtip_core::Error>(packet)
        })
    }

    /// Build IPv6 stealth packet with specified flags
    /// Sprint 6.3 Task 2.3: Added for batch I/O integration
    fn build_stealth_ipv6_packet(
        &self,
        src_ipv6: Ipv6Addr,
        dst_ipv6: Ipv6Addr,
        src_port: u16,
        dst_port: u16,
        scan_type: StealthScanType,
    ) -> Result<Vec<u8>> {
        let mut builder = TcpPacketBuilder::new()
            .source_port(src_port)
            .dest_port(dst_port)
            .flags(scan_type.flags())
            .sequence(0)
            .acknowledgment(0)
            .window(1024);

        if let Some(ttl) = self.config.evasion.ttl {
            builder = builder.ttl(ttl);
        }

        if self.config.evasion.bad_checksums {
            builder = builder.bad_checksum(true);
        }

        builder.build_ipv6_packet(src_ipv6, dst_ipv6).map_err(|e| {
            prtip_core::Error::Network(format!("Failed to build IPv6 stealth packet: {}", e))
        })
    }

    /// Calculate optimal batch size based on platform capabilities and port count
    /// Sprint 6.3 Task 2.3: Added for batch I/O integration
    fn calculate_batch_size(&self, _parallelism: usize, port_count: usize) -> usize {
        let caps = PlatformCapabilities::detect();

        let max_batch = if caps.has_sendmmsg && caps.has_recvmmsg {
            self.config.performance.max_batch_size
        } else {
            1 // Fallback mode: no batching
        };

        // Use configured batch size (default 256), capped by max and port count
        let batch_size = self
            .config
            .performance
            .batch_size
            .unwrap_or(256)
            .min(max_batch);
        batch_size.min(port_count).max(1)
    }

    /// Process batch of received stealth responses
    /// Sprint 6.3 Task 2.3: Added for batch I/O integration
    async fn process_batch_responses(
        &self,
        responses: Vec<Vec<u8>>,
        scan_type: StealthScanType,
        scan_id: Uuid,
    ) -> Result<Vec<ScanResult>> {
        let mut results = Vec::new();

        for response_data in responses {
            // Parse response to extract target info
            if let Some((target_ip, target_port, src_port)) =
                self.parse_stealth_response(&response_data, scan_type)
            {
                // Look up connection state
                let key = (target_ip, target_port, src_port, scan_type);
                if let Some((_, state)) = self.connections.remove(&key) {
                    let response_time = state.sent_time.elapsed();

                    // For stealth scans, RST = closed, no response = open|filtered
                    // This response indicates a RST was received
                    let result = ScanResult::new(target_ip, target_port, PortState::Closed)
                        .with_response_time(response_time);

                    // Publish event if event bus available
                    if let Some(ref bus) = self.event_bus {
                        bus.publish(ScanEvent::PortFound {
                            scan_id,
                            ip: target_ip,
                            port: target_port,
                            state: PortState::Closed,
                            protocol: Protocol::Tcp,
                            scan_type: scan_type.to_scan_type(),
                            timestamp: std::time::SystemTime::now(),
                        })
                        .await;
                    }

                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    /// Parse stealth response to extract connection info
    /// Sprint 6.3 Task 2.3: Added for batch I/O integration
    fn parse_stealth_response(
        &self,
        data: &[u8],
        _scan_type: StealthScanType,
    ) -> Option<(IpAddr, u16, u16)> {
        // Minimum IPv4 header (20) + TCP header (20) = 40 bytes
        if data.len() < 40 {
            return None;
        }

        // Check IP version
        let version = (data[0] >> 4) & 0x0F;

        match version {
            4 => {
                // IPv4: Extract source IP, source port, dest port
                let src_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
                let tcp_offset = 20; // IPv4 header size
                let src_port = u16::from_be_bytes([data[tcp_offset], data[tcp_offset + 1]]);
                let dst_port = u16::from_be_bytes([data[tcp_offset + 2], data[tcp_offset + 3]]);

                Some((IpAddr::V4(src_ip), dst_port, src_port))
            }
            6 => {
                // IPv6: Extract source IP, source port, dest port
                let src_ip = Ipv6Addr::new(
                    u16::from_be_bytes([data[8], data[9]]),
                    u16::from_be_bytes([data[10], data[11]]),
                    u16::from_be_bytes([data[12], data[13]]),
                    u16::from_be_bytes([data[14], data[15]]),
                    u16::from_be_bytes([data[16], data[17]]),
                    u16::from_be_bytes([data[18], data[19]]),
                    u16::from_be_bytes([data[20], data[21]]),
                    u16::from_be_bytes([data[22], data[23]]),
                );
                let tcp_offset = 40; // IPv6 header size
                let src_port = u16::from_be_bytes([data[tcp_offset], data[tcp_offset + 1]]);
                let dst_port = u16::from_be_bytes([data[tcp_offset + 2], data[tcp_offset + 3]]);

                Some((IpAddr::V6(src_ip), dst_port, src_port))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_type_flags() {
        assert_eq!(StealthScanType::Fin.flags().0, TcpFlags::FIN.0);
        assert_eq!(StealthScanType::Null.flags().0, 0);
        assert_eq!(StealthScanType::Ack.flags().0, TcpFlags::ACK.0);

        // Xmas should have FIN + PSH + URG
        let xmas_flags = StealthScanType::Xmas.flags();
        assert!(xmas_flags.has(TcpFlags::FIN));
        assert!(xmas_flags.has(TcpFlags::PSH));
        assert!(xmas_flags.has(TcpFlags::URG));
    }

    #[test]
    fn test_scan_type_names() {
        assert_eq!(StealthScanType::Fin.name(), "FIN");
        assert_eq!(StealthScanType::Null.name(), "NULL");
        assert_eq!(StealthScanType::Xmas.name(), "Xmas");
        assert_eq!(StealthScanType::Ack.name(), "ACK");
    }

    #[test]
    fn test_stealth_scanner_creation() {
        let config = Config::default();
        let scanner = StealthScanner::new(config);
        assert!(scanner.is_ok());
    }
}
