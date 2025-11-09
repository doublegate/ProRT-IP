//! Idle Scanner Core Logic
//!
//! This module implements the main idle scan (zombie scan) logic using the three-step
//! process: baseline IPID measurement → spoofed SYN probe → post-scan IPID measurement.
//!
//! # How It Works
//!
//! For each target port:
//! 1. **Baseline**: Measure zombie's current IPID (send SYN/ACK, get RST response)
//! 2. **Spoof**: Send SYN packet with zombie's source IP to target
//! 3. **Measure**: Measure zombie's new IPID after target response
//! 4. **Interpret**: Calculate IPID delta and infer port state
//!
//! **IPID Delta Interpretation:**
//! - delta = 0: Port filtered (no response reached zombie)
//! - delta = 1: Port closed (target sent RST to zombie)
//! - delta = 2: Port open (target sent SYN-ACK, zombie replied RST)
//! - delta > 2: Zombie traffic noise (unreliable, rescan)
//!
//! # See Also
//!
//! - [Idle Scan Guide](../../../docs/25-IDLE-SCAN-GUIDE.md) - Complete implementation guide
//! - [`ZombieDiscovery`](super::zombie_discovery::ZombieDiscovery) - Finding suitable zombies

use crate::idle::ipid_tracker::IPIDTracker;
use crate::idle::zombie_discovery::ZombieCandidate;
use crate::AdaptiveRateLimiterV2;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::tcp::{MutableTcpPacket, TcpFlags};
use pnet::transport::{transport_channel, TransportChannelType, TransportProtocol};
use prtip_core::{Error, EventBus, PortState, Protocol, Result, ScanEvent, ScanType};
use rand::Rng;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Semaphore;
use tracing::debug;
use uuid::Uuid;

/// Idle scan configuration
#[derive(Debug, Clone)]
pub struct IdleScanConfig {
    /// Zombie host to use
    pub zombie: ZombieCandidate,

    /// Wait time between spoof and measure (milliseconds)
    pub wait_time_ms: u64,

    /// Retry count on inconsistent results
    pub retries: usize,

    /// Minimum confidence threshold
    pub confidence_threshold: f32,
}

impl Default for IdleScanConfig {
    fn default() -> Self {
        Self {
            zombie: ZombieCandidate {
                ip: "127.0.0.1".parse().unwrap(),
                pattern: crate::idle::ipid_tracker::IPIDPattern::Sequential,
                quality_score: 0.0,
                latency_ms: 0,
                last_tested: std::time::Instant::now(),
            },
            wait_time_ms: 300,
            retries: 2,
            confidence_threshold: 0.7,
        }
    }
}

/// Idle scan result
#[derive(Debug, Clone)]
pub struct IdleScanResult {
    /// Target IP address
    pub target: IpAddr,

    /// Target port
    pub port: u16,

    /// Port state
    pub state: PortState,

    /// Result confidence (0.0-1.0)
    pub confidence: f32,

    /// Observed IPID delta
    pub ipid_delta: u16,
}

/// Idle scanner
///
/// # Rate Limiting (Sprint 5.4 Phase 1)
///
/// Supports optional adaptive rate limiting:
/// - Adaptive limiter provides per-target ICMP backoff
/// - Note: Hostgroup limiting handled by scheduler (per-port scanner)
pub struct IdleScanner {
    config: IdleScanConfig,
    /// Optional adaptive rate limiter (ICMP-aware throttling)
    adaptive_limiter: Option<Arc<AdaptiveRateLimiterV2>>,
    /// Optional event bus for real-time progress updates
    event_bus: Option<Arc<EventBus>>,
}

impl IdleScanner {
    /// Create new idle scanner
    pub fn new(config: IdleScanConfig) -> Result<Self> {
        Ok(Self {
            config,
            adaptive_limiter: None,
            event_bus: None,
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

    /// Scan multiple ports on target using idle scan technique
    ///
    /// Parallelizes port scanning with configurable concurrency limit (10 concurrent).
    /// Each port probe follows the 3-step process independently.
    ///
    /// # Arguments
    /// * `target` - Target IP address to scan
    /// * `ports` - List of ports to probe
    ///
    /// # Returns
    /// * `Result<Vec<IdleScanResult>>` - Scan results for all ports
    ///
    /// # Errors
    /// * Network errors during probing
    /// * Zombie becomes unresponsive mid-scan
    /// * Privilege errors (raw packet spoofing requires root/CAP_NET_RAW)
    pub async fn scan_ports(
        &mut self,
        target: IpAddr,
        ports: &[u16],
    ) -> Result<Vec<IdleScanResult>> {
        // Generate scan ID for event tracking
        let scan_id = Uuid::new_v4();
        let scan_start = std::time::Instant::now();

        // Emit ScanStarted event
        if let Some(bus) = &self.event_bus {
            bus.publish(ScanEvent::ScanStarted {
                scan_id,
                scan_type: ScanType::Idle,
                target_count: 1,
                port_count: ports.len(),
                timestamp: SystemTime::now(),
            })
            .await;

            // Emit MetricRecorded for zombie IPID
            bus.publish(ScanEvent::MetricRecorded {
                scan_id,
                metric: prtip_core::MetricType::PacketsSent,
                value: self.config.zombie.quality_score as f64,
                timestamp: SystemTime::now(),
            })
            .await;
        }

        // Limit concurrent port probes to avoid overwhelming zombie
        let semaphore = Arc::new(Semaphore::new(10));
        let mut tasks = Vec::new();

        // Clone event bus for tasks
        let event_bus = self.event_bus.clone();

        for &port in ports {
            let zombie = self.config.zombie.clone();
            let wait_time_ms = self.config.wait_time_ms;
            let retries = self.config.retries;
            let confidence_threshold = self.config.confidence_threshold;
            let adaptive_limiter = self.adaptive_limiter.clone();
            let sem = semaphore.clone();
            let bus = event_bus.clone();

            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let result = Self::scan_single_port_with_retry(
                    zombie,
                    target,
                    port,
                    wait_time_ms,
                    retries,
                    confidence_threshold,
                    adaptive_limiter,
                )
                .await;

                // Emit PortFound event for open ports
                if let Ok(ref scan_result) = result {
                    if scan_result.state == PortState::Open {
                        if let Some(bus) = &bus {
                            bus.publish(ScanEvent::PortFound {
                                scan_id,
                                ip: target,
                                port,
                                state: scan_result.state,
                                protocol: Protocol::Tcp,
                                scan_type: ScanType::Idle,
                                timestamp: SystemTime::now(),
                            })
                            .await;
                        }
                    }
                }

                result
            });

            tasks.push(task);
        }

        // Collect results from all port probes
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    tracing::warn!("Port scan task failed: {}", e);
                }
                Err(e) => {
                    tracing::warn!("Task join error: {}", e);
                }
            }
        }

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
                detected_services: 0, // Idle scan doesn't do service detection
                timestamp: SystemTime::now(),
            })
            .await;
        }

        Ok(results)
    }

    /// Scan single port with retry logic for noisy results
    ///
    /// If IPID delta > 2 (zombie traffic noise), retry up to `retries` times.
    /// Only return result if confidence meets threshold.
    async fn scan_single_port_with_retry(
        zombie: ZombieCandidate,
        target: IpAddr,
        port: u16,
        wait_time_ms: u64,
        retries: usize,
        confidence_threshold: f32,
        adaptive_limiter: Option<Arc<AdaptiveRateLimiterV2>>,
    ) -> Result<IdleScanResult> {
        for attempt in 0..=retries {
            let result = Self::scan_single_port(
                zombie.clone(),
                target,
                port,
                wait_time_ms,
                adaptive_limiter.clone(),
            )
            .await?;

            if result.confidence >= confidence_threshold {
                return Ok(result);
            }

            if attempt < retries {
                tracing::debug!(
                    "Low confidence ({:.2}) on {}:{}, retrying ({}/{})",
                    result.confidence,
                    target,
                    port,
                    attempt + 1,
                    retries
                );
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        // Return last attempt even if low confidence
        Self::scan_single_port(zombie, target, port, wait_time_ms, adaptive_limiter).await
    }

    /// Scan single port via zombie host (3-step idle scan process)
    ///
    /// 1. Measure baseline IPID from zombie
    /// 2. Send spoofed SYN from zombie to target
    /// 3. Wait for target response (300ms default)
    /// 4. Measure post-scan IPID from zombie
    /// 5. Calculate delta and interpret port state
    async fn scan_single_port(
        zombie: ZombieCandidate,
        target: IpAddr,
        port: u16,
        wait_time_ms: u64,
        adaptive_limiter: Option<Arc<AdaptiveRateLimiterV2>>,
    ) -> Result<IdleScanResult> {
        // Check ICMP backoff (if adaptive rate limiting enabled)
        if let Some(limiter) = &adaptive_limiter {
            if limiter.is_target_backed_off(target) {
                debug!("Skipping {}:{} (ICMP backoff active)", target, port);
                return Ok(IdleScanResult {
                    target,
                    port,
                    state: PortState::Filtered,
                    confidence: 1.0,
                    ipid_delta: 0,
                });
            }
        }

        // Step 1: Baseline IPID measurement
        let baseline = baseline_ipid(&zombie).await?;

        // Step 2: Send spoofed SYN from zombie to target
        send_spoofed_syn(zombie.ip, target, port).await?;

        // Step 3: Wait for target to respond to zombie
        tokio::time::sleep(Duration::from_millis(wait_time_ms)).await;

        // Step 4: Post-scan IPID measurement
        let post_scan = post_scan_ipid(&zombie).await?;

        // Step 5: Calculate delta and interpret port state
        let delta = calculate_delta_with_rollover(post_scan, baseline);
        let state = interpret_ipid_delta(delta);
        let confidence = calculate_confidence(delta);

        Ok(IdleScanResult {
            target,
            port,
            state,
            confidence,
            ipid_delta: delta,
        })
    }
}

/// Measure baseline IPID from zombie before port probe
///
/// Sends SYN/ACK probe to zombie, which triggers RST response with IPID field.
async fn baseline_ipid(zombie: &ZombieCandidate) -> Result<u16> {
    let mut tracker = IPIDTracker::new(zombie.ip)?;
    let measurement = tracker.probe().await?;
    Ok(measurement.ipid)
}

/// Measure post-scan IPID from zombie after port probe
///
/// Same as baseline measurement, but performed after target response.
async fn post_scan_ipid(zombie: &ZombieCandidate) -> Result<u16> {
    let mut tracker = IPIDTracker::new(zombie.ip)?;
    let measurement = tracker.probe().await?;
    Ok(measurement.ipid)
}

/// Build and send spoofed SYN packet from zombie to target
///
/// Creates TCP SYN packet with zombie's IP as source address, allowing the target
/// to respond directly to zombie. This requires raw socket privileges (root/CAP_NET_RAW).
///
/// # Packet Structure
/// - TCP: SYN flag, random source port, random ISN
/// - IP: Zombie's IP as source (SPOOFED), target IP as destination
/// - Checksums: Calculated with spoofed source IP
///
/// # Security Note
/// Packet spoofing requires elevated privileges. Ensure privilege dropping is configured
/// in your scanner setup (already implemented in ProRT-IP).
async fn send_spoofed_syn(zombie_ip: IpAddr, target_ip: IpAddr, target_port: u16) -> Result<()> {
    // Only support IPv4 for now (IPv6 idle scan in Phase 5)
    let zombie_ipv4 = match zombie_ip {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(_) => {
            return Err(Error::Scanner(
                "IPv6 idle scan not yet supported (Phase 5)".into(),
            ))
        }
    };

    let target_ipv4 = match target_ip {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(_) => {
            return Err(Error::Scanner(
                "IPv6 idle scan not yet supported (Phase 5)".into(),
            ))
        }
    };

    let mut rng = rand::thread_rng();

    // 1. Build TCP SYN packet
    let mut tcp_buffer = vec![0u8; 20]; // TCP header (no options)
    let mut tcp_packet = MutableTcpPacket::new(&mut tcp_buffer)
        .ok_or_else(|| Error::Scanner("Failed to create TCP packet".into()))?;

    tcp_packet.set_source(rng.gen::<u16>()); // Random source port
    tcp_packet.set_destination(target_port);
    tcp_packet.set_sequence(rng.gen::<u32>()); // Random ISN
    tcp_packet.set_flags(TcpFlags::SYN);
    tcp_packet.set_window(65535);
    tcp_packet.set_data_offset(5); // 20 bytes / 4 = 5

    // 2. Calculate TCP checksum with spoofed source IP
    let checksum =
        pnet::packet::tcp::ipv4_checksum(&tcp_packet.to_immutable(), &zombie_ipv4, &target_ipv4);
    tcp_packet.set_checksum(checksum);

    // 3. Build IPv4 packet with spoofed source
    let total_len = 20 + tcp_buffer.len();
    let mut ip_buffer = vec![0u8; total_len];
    let mut ip_packet = MutableIpv4Packet::new(&mut ip_buffer)
        .ok_or_else(|| Error::Scanner("Failed to create IP packet".into()))?;

    ip_packet.set_version(4);
    ip_packet.set_header_length(5); // 20 bytes / 4 = 5
    ip_packet.set_total_length(total_len as u16);
    ip_packet.set_ttl(64);
    ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
    ip_packet.set_source(zombie_ipv4); // SPOOFED SOURCE
    ip_packet.set_destination(target_ipv4);
    ip_packet.set_payload(&tcp_buffer);

    // 4. Calculate IP checksum
    let ip_checksum = pnet::packet::ipv4::checksum(&ip_packet.to_immutable());
    ip_packet.set_checksum(ip_checksum);

    // 5. Send via raw socket (requires CAP_NET_RAW)
    let protocol =
        TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp));
    let (mut tx, _rx) = transport_channel(4096, protocol)
        .map_err(|e| Error::Network(format!("Failed to create raw socket: {}", e)))?;

    tx.send_to(ip_packet.to_immutable(), IpAddr::V4(target_ipv4))
        .map_err(|e| Error::Network(format!("Failed to send spoofed packet: {}", e)))?;

    Ok(())
}

/// Interpret IPID delta to infer port state
///
/// # Delta Meanings
/// - **0**: Port filtered (no response to zombie)
/// - **1**: Port closed (target sent RST to zombie)
/// - **2**: Port open (target sent SYN-ACK, zombie replied RST)
/// - **>2**: Zombie traffic noise (unreliable result)
fn interpret_ipid_delta(delta: u16) -> PortState {
    match delta {
        0 => PortState::Filtered,
        1 => PortState::Closed,
        2 => PortState::Open,
        _ => PortState::Unknown, // Zombie had traffic (rescan needed)
    }
}

/// Calculate IPID delta with rollover handling
///
/// IPID is 16-bit (0-65535). If scanning many ports, IPID may wrap around:
/// - Pre-scan: 65534
/// - Post-scan: 1
/// - Delta: 3 (not 65534-1=-65533!)
///
/// Uses wrapping subtraction to handle rollover correctly.
fn calculate_delta_with_rollover(post: u16, pre: u16) -> u16 {
    post.wrapping_sub(pre)
}

/// Calculate confidence score based on IPID delta
///
/// # Scoring
/// - **delta 0-2**: 1.0 (expected values, high confidence)
/// - **delta 3-5**: 0.5 (some noise, medium confidence)
/// - **delta >5**: 0.1 (high noise, low confidence, rescan recommended)
fn calculate_confidence(delta: u16) -> f32 {
    match delta {
        0..=2 => 1.0,
        3..=5 => 0.5,
        _ => 0.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpret_ipid_delta_filtered() {
        assert_eq!(interpret_ipid_delta(0), PortState::Filtered);
    }

    #[test]
    fn test_interpret_ipid_delta_closed() {
        assert_eq!(interpret_ipid_delta(1), PortState::Closed);
    }

    #[test]
    fn test_interpret_ipid_delta_open() {
        assert_eq!(interpret_ipid_delta(2), PortState::Open);
    }

    #[test]
    fn test_interpret_ipid_delta_unknown() {
        assert_eq!(interpret_ipid_delta(3), PortState::Unknown);
        assert_eq!(interpret_ipid_delta(10), PortState::Unknown);
    }

    #[test]
    fn test_calculate_delta_normal() {
        assert_eq!(calculate_delta_with_rollover(102, 100), 2);
        assert_eq!(calculate_delta_with_rollover(101, 100), 1);
        assert_eq!(calculate_delta_with_rollover(100, 100), 0);
    }

    #[test]
    fn test_calculate_delta_rollover() {
        // IPID rollover: 65535 → 0
        assert_eq!(calculate_delta_with_rollover(0, 65535), 1);
        assert_eq!(calculate_delta_with_rollover(1, 65535), 2);
        assert_eq!(calculate_delta_with_rollover(5, 65534), 7);
    }

    #[test]
    fn test_calculate_confidence_high() {
        assert_eq!(calculate_confidence(0), 1.0);
        assert_eq!(calculate_confidence(1), 1.0);
        assert_eq!(calculate_confidence(2), 1.0);
    }

    #[test]
    fn test_calculate_confidence_medium() {
        assert_eq!(calculate_confidence(3), 0.5);
        assert_eq!(calculate_confidence(4), 0.5);
        assert_eq!(calculate_confidence(5), 0.5);
    }

    #[test]
    fn test_calculate_confidence_low() {
        assert_eq!(calculate_confidence(6), 0.1);
        assert_eq!(calculate_confidence(10), 0.1);
        assert_eq!(calculate_confidence(100), 0.1);
    }

    #[test]
    fn test_idle_scan_config_default() {
        let config = IdleScanConfig::default();
        assert_eq!(config.wait_time_ms, 300);
        assert_eq!(config.retries, 2);
        assert_eq!(config.confidence_threshold, 0.7);
    }
}
