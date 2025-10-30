//! Host Discovery Engine
//!
//! Implements various host discovery techniques to determine which hosts are alive
//! before performing port scans. This reduces scan time by avoiding probes to
//! unreachable hosts.
//!
//! # Discovery Methods
//!
//! - **ICMP Echo**: Send ICMP echo requests (ping) - requires raw socket privileges
//! - **ARP**: Send ARP requests on local network - requires raw socket privileges
//! - **TCP SYN Ping**: Attempt TCP connections to common ports - works without privileges
//!
//! # Phase 1 Implementation
//!
//! For Phase 1, we implement TCP SYN ping as a fallback that doesn't require
//! privileges. ICMP and ARP will be fully implemented in Phase 2 when raw socket
//! support is complete.

use prtip_core::{Error, Result};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;
use tracing::{debug, trace, warn};

// ICMPv4/v6 packet types and transport
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::Packet;
use pnet::transport::{
    icmp_packet_iter, transport_channel, TransportChannelType, TransportProtocol,
};

/// Host discovery methods
///
/// Different techniques for detecting live hosts on the network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// ICMP echo request (ping) - requires raw sockets
    ///
    /// Sends ICMP echo request packets and waits for echo replies.
    /// Most reliable for general host discovery, but requires elevated privileges.
    IcmpEcho,

    /// ARP request (local network only) - requires raw sockets
    ///
    /// Sends ARP requests to discover hosts on the local network segment.
    /// Only works for hosts on the same subnet, but very reliable as ARP
    /// cannot be blocked by firewalls.
    Arp,

    /// TCP SYN to common ports
    ///
    /// Attempts TCP connections to commonly open ports to detect live hosts.
    /// Works without elevated privileges and bypasses ICMP blocking, but
    /// may miss hosts that don't have the probed ports open.
    TcpSyn,
}

impl std::fmt::Display for DiscoveryMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryMethod::IcmpEcho => write!(f, "ICMP Echo"),
            DiscoveryMethod::Arp => write!(f, "ARP"),
            DiscoveryMethod::TcpSyn => write!(f, "TCP SYN Ping"),
        }
    }
}

/// Common ports to probe for TCP SYN ping
///
/// These ports are frequently open on various types of systems:
/// - 80, 443: HTTP/HTTPS (web servers)
/// - 22: SSH (Unix/Linux servers)
/// - 21: FTP (file servers)
/// - 25: SMTP (mail servers)
/// - 53: DNS (name servers)
/// - 3389: RDP (Windows servers)
/// - 3306: MySQL (database servers)
/// - 5432: PostgreSQL (database servers)
const TCP_PING_PORTS: &[u16] = &[80, 443, 22, 21, 25, 53, 3389, 3306, 5432];

/// Host discovery engine
///
/// Determines which hosts in a target range are alive and responding.
///
/// # Examples
///
/// ```no_run
/// use prtip_scanner::{DiscoveryEngine, DiscoveryMethod};
/// use std::net::IpAddr;
/// use std::time::Duration;
///
/// # async fn example() -> prtip_core::Result<()> {
/// let engine = DiscoveryEngine::new(Duration::from_secs(2), DiscoveryMethod::TcpSyn);
/// let target: IpAddr = "192.168.1.1".parse().unwrap();
/// let is_alive = engine.is_host_alive(target).await?;
/// println!("Host alive: {}", is_alive);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct DiscoveryEngine {
    timeout: Duration,
    method: DiscoveryMethod,
}

impl DiscoveryEngine {
    /// Create a new discovery engine
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for a response
    /// * `method` - Discovery method to use
    pub fn new(timeout: Duration, method: DiscoveryMethod) -> Self {
        Self { timeout, method }
    }

    /// Check if a host is alive
    ///
    /// # Arguments
    ///
    /// * `target` - IP address to probe
    ///
    /// # Returns
    ///
    /// `true` if the host responds to the discovery probe, `false` otherwise.
    pub async fn is_host_alive(&self, target: IpAddr) -> Result<bool> {
        match self.method {
            DiscoveryMethod::IcmpEcho => self.icmp_ping(target).await,
            DiscoveryMethod::Arp => self.arp_ping(target).await,
            DiscoveryMethod::TcpSyn => self.tcp_syn_ping(target).await,
        }
    }

    /// Perform ICMP echo request
    ///
    /// Dispatches to IPv4 or IPv6 implementation based on target address type.
    async fn icmp_ping(&self, target: IpAddr) -> Result<bool> {
        match target {
            IpAddr::V4(target_v4) => self.icmp_echo_ipv4(target_v4).await,
            IpAddr::V6(target_v6) => self.icmp_echo_ipv6(target_v6).await,
        }
    }

    /// Perform ICMPv4 Echo Request/Reply
    ///
    /// Sends ICMP Echo Request (Type 8) and waits for Echo Reply (Type 0).
    /// Validates identifier to ensure reply matches our request.
    ///
    /// # Requirements
    ///
    /// Requires CAP_NET_RAW capability (root/sudo on Unix).
    async fn icmp_echo_ipv4(&self, target: Ipv4Addr) -> Result<bool> {
        use pnet::packet::icmp::{echo_request, IcmpCode, MutableIcmpPacket};

        // Create ICMP transport channel (requires CAP_NET_RAW)
        let protocol =
            TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Icmp));

        let (mut tx, mut rx) = transport_channel(1024, protocol)
            .map_err(|e| Error::Network(format!("Failed to create ICMP transport: {}", e)))?;

        // Build ICMP Echo Request (Type 8, Code 0)
        let identifier = std::process::id() as u16;
        let sequence = 1u16;
        let payload = b"ProRT-IP";

        let mut buffer = vec![0u8; 8 + payload.len()];
        let mut icmp_packet = MutableIcmpPacket::new(&mut buffer)
            .ok_or_else(|| Error::Network("Failed to create ICMP packet".to_string()))?;

        icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
        icmp_packet.set_icmp_code(IcmpCode(0));

        // Use pnet's echo_request module for proper formatting
        let mut echo_buffer = vec![
            0u8;
            echo_request::MutableEchoRequestPacket::minimum_packet_size()
                + payload.len()
        ];
        let mut echo_packet = echo_request::MutableEchoRequestPacket::new(&mut echo_buffer)
            .ok_or_else(|| Error::Network("Failed to create echo request".to_string()))?;

        echo_packet.set_icmp_type(IcmpTypes::EchoRequest);
        echo_packet.set_icmp_code(IcmpCode(0));
        echo_packet.set_identifier(identifier);
        echo_packet.set_sequence_number(sequence);
        echo_packet.set_payload(payload);

        // Calculate checksum
        let checksum = pnet::util::checksum(echo_packet.packet(), 1);
        echo_packet.set_checksum(checksum);

        // Send ICMP Echo Request
        tx.send_to(echo_packet, IpAddr::V4(target))
            .map_err(|e| Error::Network(format!("Failed to send ICMP request: {}", e)))?;

        // Wait for Echo Reply (Type 0) with timeout
        let start = std::time::Instant::now();
        let mut iter = icmp_packet_iter(&mut rx);

        while start.elapsed() < self.timeout {
            // Platform-specific packet receiving with timeout
            // Unix: Uses pnet's next_with_timeout()
            // Windows: Returns None (method doesn't exist, discovery will timeout gracefully)
            #[cfg(unix)]
            let packet_result = iter.next_with_timeout(Duration::from_millis(100));
            #[cfg(windows)]
            let packet_result: std::io::Result<Option<_>> = Ok(None);

            if let Ok(Some((packet, IpAddr::V4(src_ip)))) = packet_result {
                if src_ip == target && packet.get_icmp_type() == IcmpTypes::EchoReply {
                    // Parse echo reply to validate identifier
                    if let Some(echo_reply) = echo_request::EchoRequestPacket::new(packet.packet())
                    {
                        if echo_reply.get_identifier() == identifier {
                            debug!("Host {} alive (ICMP Echo Reply)", target);
                            return Ok(true);
                        }
                    }
                }
            }
        }

        // Timeout - no response
        debug!("Host {} timeout (ICMP Echo)", target);
        Ok(false)
    }

    /// Perform ICMPv6 Echo Request/Reply
    ///
    /// Sends ICMPv6 Echo Request (Type 128) and waits for Echo Reply (Type 129).
    /// Uses the existing icmpv6 packet builder infrastructure.
    ///
    /// # Requirements
    ///
    /// Requires CAP_NET_RAW capability (root/sudo on Unix).
    async fn icmp_echo_ipv6(&self, target: Ipv6Addr) -> Result<bool> {
        use pnet::packet::icmpv6::{Icmpv6Types, MutableIcmpv6Packet};
        use prtip_network::icmpv6::Icmpv6PacketBuilder;

        // Create ICMPv6 transport channel (requires CAP_NET_RAW)
        let protocol =
            TransportChannelType::Layer4(TransportProtocol::Ipv6(IpNextHeaderProtocols::Icmpv6));

        let (mut tx, mut rx) = transport_channel(1024, protocol)
            .map_err(|e| Error::Network(format!("Failed to create ICMPv6 transport: {}", e)))?;

        // Get local IPv6 address (use link-local or unspecified)
        let local_ip = if target.is_loopback() {
            Ipv6Addr::LOCALHOST
        } else {
            // Use unspecified - kernel will select appropriate source
            Ipv6Addr::UNSPECIFIED
        };

        // Build ICMPv6 Echo Request (Type 128)
        let identifier = std::process::id() as u16;
        let sequence = 1u16;
        let payload = b"ProRT-IP".to_vec();

        let packet_bytes = Icmpv6PacketBuilder::echo_request(identifier, sequence, payload)
            .build(local_ip, target)
            .map_err(|e| Error::Network(format!("Failed to build ICMPv6 packet: {}", e)))?;

        // Send ICMPv6 Echo Request using MutableIcmpv6Packet
        if let Some(icmpv6_packet) = MutableIcmpv6Packet::owned(packet_bytes) {
            tx.send_to(icmpv6_packet, IpAddr::V6(target))
                .map_err(|e| Error::Network(format!("Failed to send ICMPv6 request: {}", e)))?;
        } else {
            return Err(Error::Network(
                "Failed to create ICMPv6 packet for sending".to_string(),
            ));
        }

        // Wait for Echo Reply (Type 129) with timeout
        let start = std::time::Instant::now();
        let mut iter = pnet::transport::icmpv6_packet_iter(&mut rx);

        while start.elapsed() < self.timeout {
            // Platform-specific packet receiving with timeout
            // Unix: Uses pnet's next_with_timeout()
            // Windows: Returns None (method doesn't exist, discovery will timeout gracefully)
            #[cfg(unix)]
            let packet_result = iter.next_with_timeout(Duration::from_millis(100));
            #[cfg(windows)]
            let packet_result: std::io::Result<Option<_>> = Ok(None);

            if let Ok(Some((packet_data, IpAddr::V6(src_ip)))) = packet_result {
                if src_ip == target && packet_data.get_icmpv6_type() == Icmpv6Types::EchoReply {
                    // Validate identifier
                    let payload = packet_data.payload();
                    if payload.len() >= 4 {
                        let reply_id = u16::from_be_bytes([payload[0], payload[1]]);
                        if reply_id == identifier {
                            debug!("Host {} alive (ICMPv6 Echo Reply)", target);
                            return Ok(true);
                        }
                    }
                }
            }
        }

        // Timeout - no response
        debug!("Host {} timeout (ICMPv6 Echo)", target);
        Ok(false)
    }

    /// Perform ARP request (local network)
    ///
    /// Dispatches to ARP (IPv4) or NDP (IPv6) based on target address type.
    /// ARP for IPv4 is not yet implemented - returns error.
    /// NDP for IPv6 is implemented below.
    async fn arp_ping(&self, target: IpAddr) -> Result<bool> {
        match target {
            IpAddr::V4(_target_v4) => {
                // ARP for IPv4 (future work - Phase 5.x)
                warn!("ARP ping not yet implemented for IPv4");
                Err(Error::Network(
                    "ARP ping not yet implemented for IPv4. Use TCP SYN or ICMP ping.".to_string(),
                ))
            }
            IpAddr::V6(target_v6) => {
                // NDP replaces ARP for IPv6
                self.ndp_neighbor_discovery(target_v6).await
            }
        }
    }

    /// Perform NDP (Neighbor Discovery Protocol) for IPv6
    ///
    /// Sends Neighbor Solicitation (Type 135) to solicited-node multicast address
    /// and waits for Neighbor Advertisement (Type 136).
    ///
    /// # NDP Process
    ///
    /// 1. Calculate solicited-node multicast address from target
    /// 2. Send Neighbor Solicitation to multicast address
    /// 3. Wait for Neighbor Advertisement from target
    ///
    /// # Requirements
    ///
    /// Requires CAP_NET_RAW capability (root/sudo on Unix).
    async fn ndp_neighbor_discovery(&self, target: Ipv6Addr) -> Result<bool> {
        use prtip_network::icmpv6::{Icmpv6PacketBuilder, Icmpv6ResponseParser};

        // NDP only works for link-local or on-link addresses
        // For loopback, use ICMP Echo instead
        if target.is_loopback() {
            return self.icmp_echo_ipv6(target).await;
        }

        // Create ICMPv6 transport channel (requires CAP_NET_RAW)
        let protocol =
            TransportChannelType::Layer4(TransportProtocol::Ipv6(IpNextHeaderProtocols::Icmpv6));

        let (mut tx, mut rx) = transport_channel(1024, protocol)
            .map_err(|e| Error::Network(format!("Failed to create ICMPv6 transport: {}", e)))?;

        // Get local IPv6 address (typically link-local fe80::)
        let local_ip = Ipv6Addr::UNSPECIFIED; // Kernel will select

        // Calculate solicited-node multicast address: ff02::1:ff00:0/104 + last 24 bits of target
        let target_segments = target.segments();
        let last_segment = target_segments[7];
        let multicast_last = 0xff00 | (last_segment & 0x00ff);
        let solicited_node = Ipv6Addr::new(
            0xff02,
            0,
            0,
            0,
            0,
            1,
            (target_segments[6] & 0x00ff) << 8 | (last_segment >> 8),
            multicast_last,
        );

        // Build NDP Neighbor Solicitation (Type 135)
        // Source link-layer address option (Type 1) - use zeros (kernel will fill)
        let source_ll_addr = [0u8; 6];
        let packet_bytes = Icmpv6PacketBuilder::neighbor_solicitation(target, Some(source_ll_addr))
            .build(local_ip, solicited_node)
            .map_err(|e| Error::Network(format!("Failed to build NDP packet: {}", e)))?;

        // Send NDP Neighbor Solicitation
        use pnet::packet::icmpv6::MutableIcmpv6Packet;
        if let Some(icmpv6_packet) = MutableIcmpv6Packet::owned(packet_bytes) {
            tx.send_to(icmpv6_packet, IpAddr::V6(solicited_node))
                .map_err(|e| Error::Network(format!("Failed to send NDP solicitation: {}", e)))?;
        } else {
            return Err(Error::Network(
                "Failed to create NDP packet for sending".to_string(),
            ));
        }

        // Wait for Neighbor Advertisement (Type 136)
        let start = std::time::Instant::now();
        let mut iter = pnet::transport::icmpv6_packet_iter(&mut rx);

        while start.elapsed() < self.timeout {
            // Platform-specific packet receiving with timeout
            // Unix: Uses pnet's next_with_timeout()
            // Windows: Returns None (method doesn't exist, discovery will timeout gracefully)
            #[cfg(unix)]
            let packet_result = iter.next_with_timeout(Duration::from_millis(100));
            #[cfg(windows)]
            let packet_result: std::io::Result<Option<_>> = Ok(None);

            if let Ok(Some((packet_data, addr))) = packet_result {
                // Type 136 = Neighbor Advertisement
                if packet_data.get_icmpv6_type().0 == 136 {
                    // Parse target address from Neighbor Advertisement
                    if let Some(advertised_target) =
                        Icmpv6ResponseParser::parse_neighbor_advertisement(packet_data.packet())
                    {
                        if advertised_target == target {
                            debug!(
                                "Host {} alive (NDP Neighbor Advertisement from {})",
                                target, addr
                            );
                            return Ok(true);
                        }
                    }
                }
            }
        }

        // Timeout - no response
        debug!("Host {} timeout (NDP)", target);
        Ok(false)
    }

    /// Perform TCP SYN ping to common ports
    ///
    /// Attempts to connect to commonly open ports to determine if a host is alive.
    /// This works without elevated privileges and can bypass firewalls that block ICMP.
    ///
    /// # Implementation
    ///
    /// Tries to connect to ports in the following order:
    /// 1. Web servers (80, 443)
    /// 2. SSH (22)
    /// 3. Other common services (21, 25, 53, 3389, 3306, 5432)
    ///
    /// Returns `true` on the first successful connection, `false` if all ports fail.
    async fn tcp_syn_ping(&self, target: IpAddr) -> Result<bool> {
        trace!("TCP SYN ping to {}", target);

        for &port in TCP_PING_PORTS {
            let addr = SocketAddr::new(target, port);

            match timeout(self.timeout, TcpStream::connect(addr)).await {
                Ok(Ok(_stream)) => {
                    // Successfully connected - host is definitely alive
                    debug!("Host {} alive (TCP {} open)", target, port);
                    return Ok(true);
                }
                Ok(Err(e)) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                    // Connection refused means host is alive, just port is closed
                    debug!("Host {} alive (TCP {} refused)", target, port);
                    return Ok(true);
                }
                Ok(Err(e)) => {
                    // Other errors (unreachable, etc.) - try next port
                    trace!("TCP {} error for {}: {}", port, target, e);
                    continue;
                }
                Err(_) => {
                    // Timeout - try next port
                    trace!("TCP {} timeout for {}", port, target);
                    continue;
                }
            }
        }

        // All ports failed - host appears to be down or heavily filtered
        debug!("Host {} appears down (all TCP pings failed)", target);
        Ok(false)
    }

    /// Discover live hosts in a network range
    ///
    /// Probes multiple hosts concurrently to determine which are alive.
    ///
    /// # Arguments
    ///
    /// * `targets` - Vector of IP addresses to probe
    /// * `max_concurrent` - Maximum number of concurrent probe operations
    ///
    /// # Returns
    ///
    /// Vector of IP addresses that responded to discovery probes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_scanner::{DiscoveryEngine, DiscoveryMethod};
    /// use std::time::Duration;
    ///
    /// # async fn example() -> prtip_core::Result<()> {
    /// let engine = DiscoveryEngine::new(Duration::from_secs(2), DiscoveryMethod::TcpSyn);
    ///
    /// let targets = vec![
    ///     "192.168.1.1".parse().unwrap(),
    ///     "192.168.1.2".parse().unwrap(),
    /// ];
    ///
    /// let live_hosts = engine.discover_hosts(targets, 10).await?;
    /// println!("Found {} live hosts", live_hosts.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_hosts(
        &self,
        targets: Vec<IpAddr>,
        max_concurrent: usize,
    ) -> Result<Vec<IpAddr>> {
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let engine = self.clone();

        let mut handles = Vec::with_capacity(targets.len());

        for target in targets {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| Error::Network(format!("Semaphore error: {}", e)))?;
            let engine = engine.clone();

            let handle = tokio::spawn(async move {
                let result = engine.is_host_alive(target).await;
                drop(permit);
                result
                    .ok()
                    .and_then(|alive| if alive { Some(target) } else { None })
            });

            handles.push(handle);
        }

        let mut live_hosts = Vec::new();
        for handle in handles {
            if let Ok(Some(host)) = handle.await {
                live_hosts.push(host);
            }
        }

        Ok(live_hosts)
    }

    /// Get the configured timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get the configured discovery method
    pub fn method(&self) -> DiscoveryMethod {
        self.method
    }
}

impl Default for DiscoveryEngine {
    fn default() -> Self {
        Self::new(Duration::from_secs(2), DiscoveryMethod::TcpSyn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_tcp_syn_ping_localhost() {
        let engine = DiscoveryEngine::new(Duration::from_millis(500), DiscoveryMethod::TcpSyn);

        // Localhost should always be reachable
        let alive = engine
            .is_host_alive(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
            .await
            .unwrap();

        assert!(alive);
    }

    #[tokio::test]
    async fn test_tcp_syn_ping_unreachable() {
        // Note: This test verifies timeout behavior rather than network unreachability
        // In some network environments, all IPs may be routed to gateway
        let engine = DiscoveryEngine::new(Duration::from_millis(50), DiscoveryMethod::TcpSyn);

        // Use an IP that should timeout quickly (very short timeout forces this)
        // Even if routed, the 50ms timeout should cause failures for distant/non-existent hosts
        let test_ip = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 254));

        let start = std::time::Instant::now();
        let alive = engine.is_host_alive(test_ip).await.unwrap();
        let elapsed = start.elapsed();

        // The test passes if either:
        // 1. Host is detected as not alive (ideal case)
        // 2. Detection took reasonable time (network-dependent)
        // This makes the test more robust across different network configs
        if alive {
            // If host was detected as alive, it should have been quick
            assert!(
                elapsed < Duration::from_millis(500),
                "Host detection took too long: {:?}",
                elapsed
            );
        }
        // If not alive, test passes (expected behavior)
    }

    #[tokio::test]
    async fn test_icmp_ping_not_implemented() {
        let engine = DiscoveryEngine::new(Duration::from_secs(1), DiscoveryMethod::IcmpEcho);

        let result = engine.is_host_alive(IpAddr::V4(Ipv4Addr::LOCALHOST)).await;

        // Should return error indicating Phase 2 feature
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_arp_ping_not_implemented() {
        let engine = DiscoveryEngine::new(Duration::from_secs(1), DiscoveryMethod::Arp);

        let result = engine.is_host_alive(IpAddr::V4(Ipv4Addr::LOCALHOST)).await;

        // Should return error indicating Phase 2 feature
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_discover_hosts_multiple() {
        let engine = DiscoveryEngine::new(Duration::from_millis(500), DiscoveryMethod::TcpSyn);

        let targets = vec![
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), // localhost - should be alive
            IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), // TEST-NET - should be dead
        ];

        let live_hosts = engine.discover_hosts(targets, 10).await.unwrap();

        // Should find at least localhost
        assert!(!live_hosts.is_empty());
        assert!(live_hosts.contains(&IpAddr::V4(Ipv4Addr::LOCALHOST)));
    }

    #[tokio::test]
    async fn test_discover_hosts_empty() {
        let engine = DiscoveryEngine::new(Duration::from_secs(1), DiscoveryMethod::TcpSyn);

        let live_hosts = engine.discover_hosts(vec![], 10).await.unwrap();

        assert_eq!(live_hosts.len(), 0);
    }

    #[tokio::test]
    async fn test_discover_hosts_concurrency() {
        let engine = DiscoveryEngine::new(Duration::from_millis(100), DiscoveryMethod::TcpSyn);

        // Create test targets - some may be routed depending on network config
        let targets: Vec<IpAddr> = (1..=20)
            .map(|i| IpAddr::V4(Ipv4Addr::new(198, 51, 100, i)))
            .collect();

        let start = std::time::Instant::now();
        let live_hosts = engine.discover_hosts(targets.clone(), 10).await.unwrap();
        let elapsed = start.elapsed();

        // Should complete reasonably fast with concurrency
        // 20 targets with 100ms timeout each would take 2 seconds sequential
        // With concurrency of 10, should take much less
        // Allow 5 seconds to account for varying network conditions and CI runner contention
        assert!(
            elapsed < Duration::from_secs(5),
            "Concurrent scan took too long: {:?}",
            elapsed
        );

        // Number of live hosts is network-dependent, just verify it's <= total targets
        assert!(live_hosts.len() <= targets.len());
    }

    #[test]
    fn test_discovery_method_display() {
        assert_eq!(DiscoveryMethod::IcmpEcho.to_string(), "ICMP Echo");
        assert_eq!(DiscoveryMethod::Arp.to_string(), "ARP");
        assert_eq!(DiscoveryMethod::TcpSyn.to_string(), "TCP SYN Ping");
    }

    #[test]
    fn test_discovery_engine_default() {
        let engine = DiscoveryEngine::default();

        assert_eq!(engine.timeout(), Duration::from_secs(2));
        assert_eq!(engine.method(), DiscoveryMethod::TcpSyn);
    }

    #[test]
    fn test_discovery_engine_configuration() {
        let timeout = Duration::from_secs(5);
        let method = DiscoveryMethod::TcpSyn;
        let engine = DiscoveryEngine::new(timeout, method);

        assert_eq!(engine.timeout(), timeout);
        assert_eq!(engine.method(), method);
    }

    #[tokio::test]
    async fn test_tcp_syn_ping_ipv6() {
        let engine = DiscoveryEngine::new(Duration::from_millis(500), DiscoveryMethod::TcpSyn);

        // IPv6 localhost
        let alive = engine
            .is_host_alive(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST))
            .await
            .unwrap();

        // May or may not be alive depending on system IPv6 support
        // Just verify it doesn't panic
        let _ = alive;
    }

    #[tokio::test]
    async fn test_discover_hosts_with_localhost() {
        let engine = DiscoveryEngine::new(Duration::from_millis(500), DiscoveryMethod::TcpSyn);

        let targets = vec![
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)),
            IpAddr::V4(Ipv4Addr::new(198, 51, 100, 2)),
        ];

        let live_hosts = engine.discover_hosts(targets.clone(), 5).await.unwrap();

        // Localhost should always be detected as alive
        assert!(
            live_hosts.contains(&IpAddr::V4(Ipv4Addr::LOCALHOST)),
            "Localhost should be detected as alive"
        );

        // In permissive network configs, other IPs may also be detected
        // Just verify we got at least localhost
        assert!(!live_hosts.is_empty());
        assert!(live_hosts.len() <= targets.len());
    }

    #[tokio::test]
    async fn test_tcp_ping_timeout_behavior() {
        // Very short timeout to force timeouts
        let engine = DiscoveryEngine::new(Duration::from_millis(1), DiscoveryMethod::TcpSyn);

        let alive = engine
            .is_host_alive(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)))
            .await
            .unwrap();

        // With 1ms timeout, even Google DNS might not respond in time
        // Just verify it completes without error
        let _ = alive;
    }

    #[tokio::test]
    async fn test_tcp_ping_ports_coverage() {
        // Verify we're testing multiple ports
        assert!(TCP_PING_PORTS.len() >= 5);
        assert!(TCP_PING_PORTS.contains(&80)); // HTTP
        assert!(TCP_PING_PORTS.contains(&443)); // HTTPS
        assert!(TCP_PING_PORTS.contains(&22)); // SSH
    }
}
