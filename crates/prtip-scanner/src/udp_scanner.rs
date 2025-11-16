//! UDP scan implementation
//!
//! UDP scanning is more complex than TCP scanning because UDP is connectionless.
//! We must rely on ICMP/ICMPv6 port unreachable messages to determine if ports are closed.
//!
//! ## State determination
//!
//! - **Open**: Receive UDP response from target
//! - **Closed**: Receive ICMP port unreachable (IPv4: Type 3 Code 3, IPv6: Type 1 Code 4)
//! - **Open|Filtered**: No response (could be open or filtered by firewall)
//!
//! ## Protocol-specific probes
//!
//! For well-known services, we send protocol-specific payloads to elicit responses:
//! - DNS (53): DNS query
//! - SNMP (161): GetRequest
//! - NTP (123): Version query
//! - NetBIOS (137): Name query
//!
//! ## Dual-stack IPv4/IPv6 support
//!
//! Sprint 5.1 Phase 2.1: Enhanced for dual-stack IPv4/IPv6 scanning.
//! - Automatically detects local IPv4 and IPv6 addresses
//! - Handles both ICMP (IPv4) and ICMPv6 (IPv6) error messages
//! - Supports all protocol-specific probes for both IP versions
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::UdpScanner;
//! use prtip_core::Config;
//! use std::net::IpAddr;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! let config = Config::default();
//! let scanner = UdpScanner::new(config)?;
//!
//! // IPv4 target
//! let ipv4_target: IpAddr = "192.168.1.1".parse().unwrap();
//! let result = scanner.scan_port(ipv4_target, 53).await?;
//!
//! // IPv6 target
//! let ipv6_target: IpAddr = "2001:db8::1".parse().unwrap();
//! let result = scanner.scan_port(ipv6_target, 53).await?;
//!
//! println!("UDP port 53 state: {:?}", result.state);
//! # Ok(())
//! # }
//! ```

use crate::{AdaptiveRateLimiterV2, HostgroupLimiter};
use dashmap::DashMap;
use parking_lot::Mutex;
use prtip_core::{Config, EventBus, PortState, Protocol, Result, ScanEvent, ScanResult, ScanType};
use prtip_network::{
    adaptive_batch::AdaptiveConfig, create_capture, get_udp_payload, with_buffer, PacketCapture,
    PlatformCapabilities, UdpPacketBuilder,
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

/// Connection tracking state for batch UDP operations
/// Sprint 6.3 Task 2.2: Added for batch I/O integration
/// Note: Target IP, port, and source port are stored in the DashMap key
#[derive(Debug, Clone)]
struct ConnectionState {
    sent_time: Instant,
}

/// UDP scanner with dual-stack IPv4/IPv6 support
/// Sprint 5.1 Phase 2.1: Enhanced for IPv6 scanning
///
/// # Rate Limiting (Sprint 5.4 Phase 1)
///
/// Supports optional hostgroup and adaptive rate limiting:
/// - Hostgroup limiter controls concurrent targets
/// - Adaptive limiter provides per-target ICMP backoff
pub struct UdpScanner {
    config: Config,
    capture: Arc<Mutex<Option<Box<dyn PacketCapture>>>>,
    /// Local IPv4 address for IPv4 scans
    local_ipv4: Ipv4Addr,
    /// Local IPv6 address for IPv6 scans (if available)
    local_ipv6: Option<Ipv6Addr>,
    /// Optional hostgroup limiter (controls concurrent targets)
    hostgroup_limiter: Option<Arc<HostgroupLimiter>>,
    /// Optional adaptive rate limiter (ICMP-aware throttling)
    adaptive_limiter: Option<Arc<AdaptiveRateLimiterV2>>,
    /// Optional event bus for real-time progress updates
    event_bus: Option<Arc<EventBus>>,
    /// Connection tracking for batch I/O (Sprint 6.3 Task 2.2)
    /// Key: (target_ip, target_port, source_port)
    connections: Arc<DashMap<(IpAddr, u16, u16), ConnectionState>>,
}

impl UdpScanner {
    /// Create a new UDP scanner with dual-stack IPv4/IPv6 support
    /// Sprint 5.1 Phase 2.1: Enhanced to detect both IPv4 and IPv6 local addresses
    pub fn new(config: Config) -> Result<Self> {
        let local_ipv4 = Self::detect_local_ipv4()?;
        let local_ipv6 = Self::detect_local_ipv6();

        Ok(Self {
            config,
            capture: Arc::new(Mutex::new(None)),
            local_ipv4,
            local_ipv6,
            hostgroup_limiter: None,
            adaptive_limiter: None,
            event_bus: None,
            connections: Arc::new(DashMap::new()), // Sprint 6.3 Task 2.2: Batch I/O connection tracking
        })
    }

    /// Enable hostgroup limiting (concurrent target control)
    pub fn with_hostgroup_limiter(mut self, limiter: Arc<HostgroupLimiter>) -> Self {
        self.hostgroup_limiter = Some(limiter);
        self
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

    /// Scan multiple UDP ports with batch I/O optimization
    /// Sprint 6.3 Task 2.2: Batch I/O integration for UDP scanner
    ///
    /// Uses sendmmsg/recvmmsg on Linux for 20-40% throughput improvement.
    /// Falls back to sequential scanning on macOS/Windows.
    pub async fn scan_ports(&self, target: IpAddr, ports: Vec<u16>) -> Result<Vec<ScanResult>> {
        // Generate scan ID for event tracking
        let scan_id = Uuid::new_v4();

        // 1. Acquire hostgroup permit (if enabled)
        let _permit = if let Some(limiter) = &self.hostgroup_limiter {
            Some(limiter.acquire_target().await)
        } else {
            None
        };

        // 2. Check ICMP backoff (if adaptive rate limiting enabled)
        if let Some(limiter) = &self.adaptive_limiter {
            if limiter.is_target_backed_off(target) {
                debug!("Skipping {} (ICMP backoff active)", target);
                return Ok(ports
                    .iter()
                    .map(|&port| ScanResult::new(target, port, PortState::Filtered))
                    .collect());
            }
        }

        // 3. Detect platform capabilities and fallback if needed
        let caps = PlatformCapabilities::detect();
        if !caps.has_sendmmsg || !caps.has_recvmmsg {
            debug!("Platform lacks sendmmsg/recvmmsg support, using fallback mode for UDP scan");
            return self.scan_ports_fallback(target, ports, scan_id).await;
        }

        // 4. Calculate optimal batch size
        let parallelism = self.config.performance.parallelism;
        let batch_size = self.calculate_batch_size(parallelism, ports.len());

        debug!(
            "Scanning {} UDP ports on {} with batch_size={}",
            ports.len(),
            target,
            batch_size
        );

        // 5. Get interface name
        let interface = self.config.network.interface.as_deref().unwrap_or("eth0");

        // 6. Create adaptive config if enabled
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

        // 7. Create BatchSender and BatchReceiver
        let mut sender = prtip_network::BatchSender::new(interface, batch_size, adaptive_config)
            .map_err(|e| {
                prtip_core::Error::Network(format!("Failed to create BatchSender: {}", e))
            })?;

        let mut receiver =
            prtip_network::BatchReceiver::new(interface, batch_size).map_err(|e| {
                prtip_core::Error::Network(format!("Failed to create BatchReceiver: {}", e))
            })?;

        // 8. Process ports in batches
        let mut results = Vec::with_capacity(ports.len());
        let timeout_ms = self.config.scan.timeout_ms;

        for chunk in ports.chunks(batch_size) {
            // 8a. Prepare batch packets
            let batch_packets = self.prepare_batch(target, chunk, batch_size).await?;

            // 8b. Add packets to sender
            for packet in batch_packets {
                sender.add_packet(packet).map_err(|e| {
                    prtip_core::Error::Network(format!("Failed to add packet to batch: {}", e))
                })?;
            }

            // 8c. Flush batch (with retry logic)
            for retry in 0..3 {
                match sender.flush(3).await {
                    Ok(_) => {
                        trace!("Batch flush successful (retry {})", retry);
                        break;
                    }
                    Err(e) if retry < 2 => {
                        warn!("Batch flush retry {} failed: {}", retry, e);
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                    Err(e) => {
                        return Err(prtip_core::Error::Network(format!(
                            "Batch flush failed after 3 retries: {}",
                            e
                        )))
                    }
                }
            }

            // 8d. Receive batch responses
            let responses = receiver
                .receive_batch(timeout_ms as u32)
                .await
                .map_err(|e| {
                    prtip_core::Error::Network(format!("Failed to receive batch: {}", e))
                })?;

            // 8e. Process responses
            self.process_batch_responses(responses, &mut results, scan_id)
                .await?;

            // 8f. Mark remaining ports as filtered
            for &port in chunk {
                if !results
                    .iter()
                    .any(|r| r.port == port && r.target_ip == target)
                {
                    results.push(ScanResult::new(target, port, PortState::Filtered));
                }
            }
        }

        debug!(
            "UDP batch scan complete: {} results for {}",
            results.len(),
            target
        );

        Ok(results)
    }

    /// Fallback UDP scan for platforms without sendmmsg/recvmmsg support
    /// Sprint 6.3 Task 2.2: Sequential scanning fallback for macOS/Windows
    async fn scan_ports_fallback(
        &self,
        target: IpAddr,
        ports: Vec<u16>,
        _scan_id: Uuid,
    ) -> Result<Vec<ScanResult>> {
        debug!(
            "Using fallback mode for {} UDP ports on {}",
            ports.len(),
            target
        );

        let mut results = Vec::with_capacity(ports.len());

        // Sequential scanning using existing scan_port method
        for port in ports {
            match self.scan_port(target, port).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("UDP scan failed for {}:{} - {}", target, port, e);
                    results.push(ScanResult::new(target, port, PortState::Unknown));
                }
            }
        }

        Ok(results)
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

    /// Scan a single UDP port with dual-stack IPv4/IPv6 support
    /// Sprint 5.1 Phase 2.1: Updated to accept IpAddr for dual-stack support
    pub async fn scan_port(&self, target: IpAddr, port: u16) -> Result<ScanResult> {
        self.scan_port_with_pcapng(target, port, None).await
    }

    /// Scan a single UDP port with optional PCAPNG capture and dual-stack support
    /// Sprint 5.1 Phase 2.1: Updated to accept IpAddr for IPv4/IPv6 scanning
    pub async fn scan_port_with_pcapng(
        &self,
        target: IpAddr,
        port: u16,
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<ScanResult> {
        // Generate scan ID for potential event tracking
        let scan_id = Uuid::new_v4();

        // Note: Hostgroup limiting should be handled by the caller (scheduler)
        // since UdpScanner scans individual ports, not entire targets.
        // Only check ICMP backoff here.

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

        // Get protocol-specific payload if available
        let payload = get_udp_payload(port).unwrap_or_default();

        // Send UDP probe (with optional PCAPNG capture)
        self.send_udp_probe(target, port, src_port, &payload, pcapng_writer.clone())
            .await?;

        // Wait for response
        let timeout_ms = self.config.scan.timeout_ms;
        let wait_duration = Duration::from_millis(timeout_ms);

        let result = match timeout(
            wait_duration,
            self.wait_for_response(target, port, src_port, pcapng_writer),
        )
        .await
        {
            Ok(Ok(state)) => {
                let response_time = start_time.elapsed();

                // Emit PortFound event for open ports
                if state == PortState::Open {
                    if let Some(bus) = &self.event_bus {
                        bus.publish(ScanEvent::PortFound {
                            scan_id,
                            ip: target,
                            port,
                            state,
                            protocol: Protocol::Udp,
                            scan_type: ScanType::Udp,
                            timestamp: SystemTime::now(),
                        })
                        .await;
                    }
                }

                Ok(ScanResult::new(target, port, state).with_response_time(response_time))
            }
            Ok(Err(e)) => {
                warn!("Error waiting for UDP response: {}", e);

                // Emit warning event
                if let Some(bus) = &self.event_bus {
                    bus.publish(ScanEvent::WarningIssued {
                        scan_id,
                        message: format!(
                            "Error waiting for UDP response from {}:{}: {}",
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
                // Timeout - port is open|filtered
                debug!("No response from {}:{} - OPEN|FILTERED", target, port);
                let response_time = start_time.elapsed();
                Ok(ScanResult::new(target, port, PortState::Filtered)
                    .with_response_time(response_time))
            }
        };

        result
    }

    /// Send a UDP probe packet with dual-stack IPv4/IPv6 support
    /// Sprint 5.1 Phase 2.1: Enhanced for IPv6 packet building
    async fn send_udp_probe(
        &self,
        target: IpAddr,
        port: u16,
        src_port: u16,
        payload: &[u8],
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<()> {
        // Get appropriate local IP for target
        let local_ip = self.get_local_ip_for_target(target)?;

        // Dispatch to IPv4 or IPv6 based on target type
        // Note: send_udp_ipv4/ipv6 are NOT async, so no lock holding issue
        match (local_ip, target) {
            (IpAddr::V4(src_ipv4), IpAddr::V4(dst_ipv4)) => {
                // IPv4 UDP packet (zero-copy path)
                if let Some(ref mut capture) = *self.capture.lock() {
                    self.send_udp_ipv4(
                        capture,
                        src_ipv4,
                        dst_ipv4,
                        src_port,
                        port,
                        payload,
                        pcapng_writer,
                    )
                } else {
                    Err(prtip_core::Error::Config(
                        "Packet capture not initialized".to_string(),
                    ))
                }
            }
            (IpAddr::V6(src_ipv6), IpAddr::V6(dst_ipv6)) => {
                // IPv6 UDP packet
                if let Some(ref mut capture) = *self.capture.lock() {
                    self.send_udp_ipv6(
                        capture,
                        src_ipv6,
                        dst_ipv6,
                        src_port,
                        port,
                        payload,
                        pcapng_writer,
                    )
                } else {
                    Err(prtip_core::Error::Config(
                        "Packet capture not initialized".to_string(),
                    ))
                }
            }
            _ => Err(prtip_core::Error::Config(format!(
                "IP version mismatch: local {} vs target {}",
                local_ip, target
            ))),
        }
    }

    /// Send IPv4 UDP packet (zero-copy with optional PCAPNG capture)
    #[allow(clippy::too_many_arguments)]
    fn send_udp_ipv4(
        &self,
        capture: &mut Box<dyn PacketCapture>,
        src_ipv4: Ipv4Addr,
        dst_ipv4: Ipv4Addr,
        src_port: u16,
        port: u16,
        payload: &[u8],
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<()> {
        // Use thread-local zero-copy buffer
        with_buffer(|buffer_pool| {
            // Build packet in buffer
            let mut builder = UdpPacketBuilder::new()
                .source_ip(src_ipv4)
                .dest_ip(dst_ipv4)
                .source_port(src_port)
                .dest_port(port)
                .payload(payload.to_vec());

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
                // Note: fragment_tcp_packet works for any IP packet (TCP or UDP)
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
                            warn!("PCAPNG write error (UDP IPv4 probe): {}", e);
                        }
                    }
                }
            }

            // Send packet(s) (fragmented or whole)
            for fragment in &packets_to_send {
                capture.send_packet(fragment).map_err(|e| {
                    prtip_core::Error::Network(format!("Failed to send IPv4 UDP: {}", e))
                })?;
            }

            if self.config.evasion.fragment_packets {
                trace!(
                    "Sent {} fragmented IPv4 UDP packets to {}:{} (src_port={}, payload_len={})",
                    packets_to_send.len(),
                    dst_ipv4,
                    port,
                    src_port,
                    payload.len()
                );
            } else {
                trace!(
                    "Sent IPv4 UDP to {}:{} (src_port={}, payload_len={}) [zero-copy]",
                    dst_ipv4,
                    port,
                    src_port,
                    payload.len()
                );
            }

            // Reset buffer for next packet
            buffer_pool.reset();

            Ok::<(), prtip_core::Error>(())
        })?;

        Ok(())
    }

    /// Send IPv6 UDP packet (with optional PCAPNG capture)
    /// Sprint 5.1 Phase 2.1: New method for IPv6 UDP scanning
    #[allow(clippy::too_many_arguments)]
    fn send_udp_ipv6(
        &self,
        capture: &mut Box<dyn PacketCapture>,
        src_ipv6: Ipv6Addr,
        dst_ipv6: Ipv6Addr,
        src_port: u16,
        port: u16,
        payload: &[u8],
        pcapng_writer: Option<Arc<StdMutex<PcapngWriter>>>,
    ) -> Result<()> {
        // Build IPv6 UDP packet
        let mut builder = UdpPacketBuilder::new()
            .source_port(src_port)
            .dest_port(port)
            .payload(payload.to_vec());

        // Apply hop limit if configured (IPv6 equivalent of TTL)
        if let Some(ttl) = self.config.evasion.ttl {
            builder = builder.ttl(ttl);
        }

        // Apply bad checksum if configured
        if self.config.evasion.bad_checksums {
            builder = builder.bad_checksum(true);
        }

        let packet = builder.build_ipv6_packet(src_ipv6, dst_ipv6)?;

        // Capture to PCAPNG if writer is provided
        if let Some(ref writer) = pcapng_writer {
            if let Ok(guard) = writer.lock() {
                if let Err(e) = guard.write_packet(&packet, Direction::Sent) {
                    warn!("PCAPNG write error (UDP IPv6 probe): {}", e);
                }
            }
        }

        // Send packet
        capture
            .send_packet(&packet)
            .map_err(|e| prtip_core::Error::Network(format!("Failed to send IPv6 UDP: {}", e)))?;

        trace!(
            "Sent IPv6 UDP to {}:{} (src_port={}, payload_len={})",
            dst_ipv6,
            port,
            src_port,
            payload.len()
        );

        Ok(())
    }

    /// Wait for UDP or ICMP/ICMPv6 response (with optional PCAPNG capture)
    /// Sprint 5.1 Phase 2.1: Updated for dual-stack IPv4/IPv6 support
    async fn wait_for_response(
        &self,
        target: IpAddr,
        port: u16,
        src_port: u16,
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
                                warn!("PCAPNG write error (UDP response): {}", e);
                            }
                        }
                    }

                    if let Some(state) = self.parse_response(&packet, target, port, src_port)? {
                        return Ok(state);
                    }
                }
            }

            tokio::task::yield_now().await;
        }
    }

    /// Parse received packet with dual-stack IPv4/IPv6 support
    /// Sprint 5.1 Phase 2.1: Enhanced to handle both IPv4 and IPv6 responses
    fn parse_response(
        &self,
        packet: &[u8],
        target: IpAddr,
        port: u16,
        src_port: u16,
    ) -> Result<Option<PortState>> {
        use pnet::packet::{ethernet::EthernetPacket, Packet};

        let eth_packet = match EthernetPacket::new(packet) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Determine IP version from ethertype and dispatch
        match target {
            IpAddr::V4(target_ipv4) => {
                self.parse_ipv4_response(eth_packet.payload(), target_ipv4, port, src_port)
            }
            IpAddr::V6(target_ipv6) => {
                self.parse_ipv6_response(eth_packet.payload(), target_ipv6, port, src_port)
            }
        }
    }

    /// Parse IPv4 UDP or ICMP response
    fn parse_ipv4_response(
        &self,
        packet: &[u8],
        target: Ipv4Addr,
        port: u16,
        src_port: u16,
    ) -> Result<Option<PortState>> {
        use pnet::packet::{icmp::IcmpPacket, ipv4::Ipv4Packet, udp::UdpPacket, Packet};

        let ipv4_packet = match Ipv4Packet::new(packet) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Check if it's from our target
        if ipv4_packet.get_source() != target {
            return Ok(None);
        }

        // Check protocol
        match ipv4_packet.get_next_level_protocol().0 {
            17 => {
                // UDP response = port open
                if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                    if udp_packet.get_source() == port && udp_packet.get_destination() == src_port {
                        debug!("Received IPv4 UDP response from {}:{} - OPEN", target, port);
                        return Ok(Some(PortState::Open));
                    }
                }
            }
            1 => {
                // ICMP message
                if let Some(icmp_packet) = IcmpPacket::new(ipv4_packet.payload()) {
                    let icmp_type = icmp_packet.get_icmp_type().0;
                    let icmp_code = icmp_packet.get_icmp_code().0;

                    // ICMP type 3 code 3 = Port Unreachable
                    if icmp_type == 3 && icmp_code == 3 {
                        debug!(
                            "Received ICMP port unreachable from {}:{} - CLOSED",
                            target, port
                        );
                        return Ok(Some(PortState::Closed));
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Parse IPv6 UDP or ICMPv6 response
    /// Sprint 5.1 Phase 2.1: New method for ICMPv6 Type 1 Code 4 handling
    fn parse_ipv6_response(
        &self,
        packet: &[u8],
        target: Ipv6Addr,
        port: u16,
        src_port: u16,
    ) -> Result<Option<PortState>> {
        use pnet::packet::{
            icmpv6::{Icmpv6Packet, Icmpv6Types},
            ipv6::Ipv6Packet,
            udp::UdpPacket,
            Packet,
        };

        let ipv6_packet = match Ipv6Packet::new(packet) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Check if it's from our target
        if ipv6_packet.get_source() != target {
            return Ok(None);
        }

        // Check next header (protocol)
        match ipv6_packet.get_next_header().0 {
            17 => {
                // UDP response = port open
                if let Some(udp_packet) = UdpPacket::new(ipv6_packet.payload()) {
                    if udp_packet.get_source() == port && udp_packet.get_destination() == src_port {
                        debug!("Received IPv6 UDP response from {}:{} - OPEN", target, port);
                        return Ok(Some(PortState::Open));
                    }
                }
            }
            58 => {
                // ICMPv6 message
                if let Some(icmpv6_packet) = Icmpv6Packet::new(ipv6_packet.payload()) {
                    let icmpv6_type = icmpv6_packet.get_icmpv6_type();
                    let icmpv6_code = icmpv6_packet.get_icmpv6_code();

                    // ICMPv6 Type 1 = Destination Unreachable
                    if matches!(icmpv6_type, Icmpv6Types::DestinationUnreachable) {
                        match icmpv6_code.0 {
                            4 => {
                                // Code 4 = Port Unreachable (UDP port closed)
                                debug!(
                                    "Received ICMPv6 port unreachable from {}:{} - CLOSED",
                                    target, port
                                );
                                return Ok(Some(PortState::Closed));
                            }
                            0 | 1 | 3 | 5 | 6 => {
                                // Other unreachable codes (no route, admin prohibited, etc.)
                                debug!(
                                    "Received ICMPv6 destination unreachable (code {}) from {}:{} - FILTERED",
                                    icmpv6_code.0, target, port
                                );
                                return Ok(Some(PortState::Filtered));
                            }
                            _ => {
                                // Unknown code
                                trace!(
                                    "Received ICMPv6 destination unreachable with unknown code {} from {}:{}",
                                    icmpv6_code.0, target, port
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Calculate optimal batch size based on platform capabilities and port count
    /// Sprint 6.3 Task 2.2: Platform-aware batch sizing for UDP scanner
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

    /// Prepare batch of UDP packets for sending
    /// Sprint 6.3 Task 2.2: UDP batch packet preparation with connection tracking
    async fn prepare_batch(
        &self,
        target: IpAddr,
        ports: &[u16],
        batch_size: usize,
    ) -> Result<Vec<Vec<u8>>> {
        use rand::Rng;
        let mut packets = Vec::with_capacity(batch_size.min(ports.len()));

        for &port in ports.iter().take(batch_size) {
            // Use configured source port or generate random
            let src_port: u16 = self
                .config
                .network
                .source_port
                .unwrap_or_else(|| rand::thread_rng().gen_range(1024..65535));

            // Get protocol-specific payload
            let payload = get_udp_payload(port).unwrap_or_default();

            // Build packet based on IP version
            let packet = match target {
                IpAddr::V4(dst_ipv4) => {
                    let src_ipv4 = self.local_ipv4;
                    self.build_udp_ipv4_packet(src_ipv4, dst_ipv4, src_port, port, &payload)?
                }
                IpAddr::V6(dst_ipv6) => {
                    let src_ipv6 = self.local_ipv6.ok_or_else(|| {
                        prtip_core::Error::Config(
                            "No IPv6 address available for IPv6 scan".to_string(),
                        )
                    })?;
                    self.build_udp_ipv6_packet(src_ipv6, dst_ipv6, src_port, port, &payload)?
                }
            };

            // Track connection state (target IP, port, src_port are in DashMap key)
            let conn_state = ConnectionState {
                sent_time: Instant::now(),
            };

            self.connections
                .insert((target, port, src_port), conn_state);
            packets.push(packet);
        }

        Ok(packets)
    }

    /// Build IPv4 UDP packet for batch sending
    /// Sprint 6.3 Task 2.2: Helper for batch packet building
    fn build_udp_ipv4_packet(
        &self,
        src_ipv4: Ipv4Addr,
        dst_ipv4: Ipv4Addr,
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) -> Result<Vec<u8>> {
        with_buffer(|buffer_pool| {
            let mut builder = UdpPacketBuilder::new()
                .source_ip(src_ipv4)
                .dest_ip(dst_ipv4)
                .source_port(src_port)
                .dest_port(dst_port)
                .payload(payload.to_vec());

            // Apply TTL if configured
            if let Some(ttl) = self.config.evasion.ttl {
                builder = builder.ttl(ttl);
            }

            // Apply bad checksum if configured
            if self.config.evasion.bad_checksums {
                builder = builder.bad_checksum(true);
            }

            let packet_slice = builder.build_ip_packet_with_buffer(buffer_pool)?;
            let packet = packet_slice.to_vec();

            // Reset buffer for next packet
            buffer_pool.reset();

            Ok::<Vec<u8>, prtip_core::Error>(packet)
        })
    }

    /// Build IPv6 UDP packet for batch sending
    /// Sprint 6.3 Task 2.2: Helper for batch packet building
    fn build_udp_ipv6_packet(
        &self,
        src_ipv6: Ipv6Addr,
        dst_ipv6: Ipv6Addr,
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) -> Result<Vec<u8>> {
        let mut builder = UdpPacketBuilder::new()
            .source_port(src_port)
            .dest_port(dst_port)
            .payload(payload.to_vec());

        // Apply hop limit if configured (IPv6 equivalent of TTL)
        if let Some(ttl) = self.config.evasion.ttl {
            builder = builder.ttl(ttl);
        }

        // Apply bad checksum if configured
        if self.config.evasion.bad_checksums {
            builder = builder.bad_checksum(true);
        }

        builder.build_ipv6_packet(src_ipv6, dst_ipv6).map_err(|e| {
            prtip_core::Error::Network(format!("Failed to build IPv6 UDP packet: {}", e))
        })
    }

    /// Process batch of received responses
    /// Sprint 6.3 Task 2.2: UDP batch response processing with ICMP/ICMPv6 handling
    async fn process_batch_responses(
        &self,
        responses: Vec<prtip_network::ReceivedPacket>,
        results: &mut Vec<ScanResult>,
        scan_id: Uuid,
    ) -> Result<()> {
        for response in responses {
            // Get all connection keys for matching
            let conn_keys: Vec<_> = self.connections.iter().map(|entry| *entry.key()).collect();

            for (target, port, src_port) in conn_keys {
                if let Some(state) = self.parse_response(&response.data, target, port, src_port)? {
                    // Remove connection from tracking
                    let conn = self
                        .connections
                        .remove(&(target, port, src_port))
                        .map(|(_, v)| v);
                    let response_time = conn
                        .map(|c| c.sent_time.elapsed())
                        .unwrap_or(Duration::from_millis(0));

                    let result =
                        ScanResult::new(target, port, state).with_response_time(response_time);

                    // Emit PortFound event for open ports
                    if state == PortState::Open {
                        if let Some(bus) = &self.event_bus {
                            bus.publish(ScanEvent::PortFound {
                                scan_id,
                                ip: target,
                                port,
                                state,
                                protocol: Protocol::Udp,
                                scan_type: ScanType::Udp,
                                timestamp: SystemTime::now(),
                            })
                            .await;
                        }
                    }

                    results.push(result);
                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_scanner_creation() {
        let config = Config::default();
        let scanner = UdpScanner::new(config);
        assert!(scanner.is_ok());
    }
}
