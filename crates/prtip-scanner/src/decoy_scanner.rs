//! Decoy scanning for stealth and IDS evasion
//!
//! This module implements decoy scanning (inspired by Nmap's -D option), where the
//! scanner mixes real probes with spoofed-source probes to make the scan origin
//! harder to detect and trace.
//!
//! # How It Works
//!
//! 1. **Decoy Selection**: User specifies decoy IPs or uses RND:N for random
//! 2. **Probe Mixing**: Real probe is intermixed with decoy probes
//! 3. **Timing**: All probes sent in randomized order within short window
//! 4. **Response Handling**: Only responses to real source IP are processed
//!
//! # Decoy Placement Strategies
//!
//! - **ME**: Position of real source IP in decoy list (default: random)
//! - **RND:N**: Generate N random decoy IPs
//! - **Manual**: Specify exact decoy IPs
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::DecoyScanner;
//! use prtip_core::{Config, ScanTarget};
//! use std::net::{IpAddr, Ipv4Addr};
//!
//! # async fn example() -> prtip_core::Result<()> {
//! let config = Config::default();
//! let mut scanner = DecoyScanner::new(config);
//!
//! // Add 5 random decoys
//! scanner.set_random_decoys(5);
//!
//! // Or specify exact decoys (both IPv4 and IPv6 supported)
//! scanner.add_decoy(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
//! scanner.add_decoy(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 101)));
//!
//! // Set position of real IP (0 = first, None = random)
//! scanner.set_real_position(None);
//!
//! let target = ScanTarget::parse("10.0.0.1")?;
//! let results = scanner.scan_with_decoys(target, 80).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Security Considerations
//!
//! - **Spoofing Limitations**: Requires raw socket capability
//! - **Stateful Firewalls**: May block spoofed responses
//! - **Ethical Use**: Only use on authorized targets
//! - **Network Topology**: Decoys should be topologically plausible

use crate::AdaptiveRateLimiterV2;
use prtip_core::{Config, Error, PortState, Result, ScanResult, ScanTarget};
use prtip_network::{TcpFlags, TcpPacketBuilder};
use rand::Rng;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::debug;

/// Maximum number of decoys (including real source)
pub const MAX_DECOYS: usize = 256;

/// Minimum inter-decoy delay (microseconds)
const MIN_DECOY_DELAY_US: u64 = 100;

/// Maximum inter-decoy delay (microseconds)
const MAX_DECOY_DELAY_US: u64 = 1000;

/// Decoy placement strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoyPlacement {
    /// Real source IP at specific position (0-based index)
    Fixed(usize),
    /// Real source IP at random position
    Random,
}

/// Decoy scanner for stealth scanning with IP spoofing
///
/// # Rate Limiting (Sprint 5.4 Phase 1)
///
/// Supports optional adaptive rate limiting:
/// - Adaptive limiter provides per-target ICMP backoff
/// - Note: Hostgroup limiting handled by scheduler (per-port scanner)
pub struct DecoyScanner {
    /// Scanner configuration
    config: Config,
    /// List of decoy IP addresses (not including real source) - supports both IPv4 and IPv6
    decoys: Vec<IpAddr>,
    /// Real source IP placement strategy
    real_placement: DecoyPlacement,
    /// Number of random decoys to generate
    random_decoy_count: Option<usize>,
    /// Optional adaptive rate limiter (ICMP-aware throttling)
    adaptive_limiter: Option<Arc<AdaptiveRateLimiterV2>>,
}

impl DecoyScanner {
    /// Create new decoy scanner with configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            decoys: Vec::new(),
            real_placement: DecoyPlacement::Random,
            random_decoy_count: None,
            adaptive_limiter: None,
        }
    }

    /// Enable adaptive rate limiting (ICMP-aware throttling)
    pub fn with_adaptive_limiter(mut self, limiter: Arc<AdaptiveRateLimiterV2>) -> Self {
        self.adaptive_limiter = Some(limiter);
        self
    }

    /// Add a specific decoy IP address (supports both IPv4 and IPv6)
    ///
    /// # Arguments
    ///
    /// * `decoy` - IP address to use as decoy (IPv4 or IPv6)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use prtip_scanner::DecoyScanner;
    /// # use prtip_core::Config;
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// let mut scanner = DecoyScanner::new(Config::default());
    /// scanner.add_decoy(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
    /// ```
    pub fn add_decoy(&mut self, decoy: IpAddr) {
        if self.decoys.len() < MAX_DECOYS - 1 {
            // -1 to reserve space for real IP
            self.decoys.push(decoy);
        }
    }

    /// Set number of random decoys to generate (RND:N mode)
    ///
    /// # Arguments
    ///
    /// * `count` - Number of random decoys (max 255)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use prtip_scanner::DecoyScanner;
    /// # use prtip_core::Config;
    /// let mut scanner = DecoyScanner::new(Config::default());
    /// scanner.set_random_decoys(10); // Generate 10 random decoy IPs
    /// ```
    pub fn set_random_decoys(&mut self, count: usize) {
        self.random_decoy_count = Some(count.min(MAX_DECOYS - 1));
    }

    /// Set real source IP placement strategy
    ///
    /// # Arguments
    ///
    /// * `placement` - Position strategy (Fixed index or Random)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use prtip_scanner::{DecoyScanner, DecoyPlacement};
    /// # use prtip_core::Config;
    /// let mut scanner = DecoyScanner::new(Config::default());
    ///
    /// // Put real IP at position 3 (4th in list)
    /// scanner.set_real_position(Some(3));
    ///
    /// // Or randomize position
    /// scanner.set_real_position(None);
    /// ```
    pub fn set_real_position(&mut self, position: Option<usize>) {
        self.real_placement = match position {
            Some(pos) => DecoyPlacement::Fixed(pos),
            None => DecoyPlacement::Random,
        };
    }

    /// Generate random decoy IPs (dispatcher for IPv4/IPv6)
    fn generate_random_decoys(
        &self,
        target: IpAddr,
        count: usize,
        exclude: &[IpAddr],
    ) -> Vec<IpAddr> {
        match target {
            IpAddr::V4(_) => {
                // Extract IPv4 addresses from exclude list
                let exclude_v4: Vec<Ipv4Addr> = exclude
                    .iter()
                    .filter_map(|ip| match ip {
                        IpAddr::V4(v4) => Some(*v4),
                        _ => None,
                    })
                    .collect();

                // Generate IPv4 decoys
                Self::generate_random_decoys_ipv4(count, &exclude_v4)
                    .into_iter()
                    .map(IpAddr::V4)
                    .collect()
            }
            IpAddr::V6(target_v6) => {
                // Extract IPv6 addresses from exclude list
                let exclude_v6: Vec<Ipv6Addr> = exclude
                    .iter()
                    .filter_map(|ip| match ip {
                        IpAddr::V6(v6) => Some(*v6),
                        _ => None,
                    })
                    .collect();

                // Generate IPv6 decoys within same /64
                Self::generate_random_decoys_ipv6(target_v6, count, &exclude_v6)
                    .into_iter()
                    .map(IpAddr::V6)
                    .collect()
            }
        }
    }

    /// Generate random IPv4 decoy IPs
    fn generate_random_decoys_ipv4(count: usize, exclude: &[Ipv4Addr]) -> Vec<Ipv4Addr> {
        let mut rng = rand::thread_rng();
        let mut decoys = Vec::with_capacity(count);
        let exclude_set: HashSet<Ipv4Addr> = exclude.iter().copied().collect();

        // Generate random IPs avoiding reserved ranges
        while decoys.len() < count {
            let ip = Ipv4Addr::new(
                rng.gen_range(1..224), // Avoid 0.x and 224+ (multicast)
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(1..255), // Avoid .0 and .255
            );

            // Skip reserved ranges and duplicates
            if !Self::is_reserved_ipv4(ip) && !exclude_set.contains(&ip) && !decoys.contains(&ip) {
                decoys.push(ip);
            }
        }

        decoys
    }

    /// Generate random IPv6 decoys within same /64 subnet as target
    fn generate_random_decoys_ipv6(
        target: Ipv6Addr,
        count: usize,
        exclude: &[Ipv6Addr],
    ) -> Vec<Ipv6Addr> {
        use rand::Rng;

        // Extract /64 prefix (first 64 bits)
        let target_segments = target.segments();
        let prefix = [
            target_segments[0],
            target_segments[1],
            target_segments[2],
            target_segments[3],
        ]; // First 4 u16 segments = 64 bits

        let mut decoys = Vec::with_capacity(count);
        let mut rng = rand::thread_rng();
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 10000; // Prevent infinite loops

        while decoys.len() < count && attempts < MAX_ATTEMPTS {
            attempts += 1;

            // Generate random interface identifier (last 64 bits)
            let iid = [
                rng.gen::<u16>(),
                rng.gen::<u16>(),
                rng.gen::<u16>(),
                rng.gen::<u16>(),
            ];

            // Combine prefix + interface identifier
            let decoy = Ipv6Addr::new(
                prefix[0], prefix[1], prefix[2], prefix[3], iid[0], iid[1], iid[2], iid[3],
            );

            // Validate: not target, not reserved, not in exclude list, unique
            if decoy != target
                && !Self::is_reserved_ipv6(decoy)
                && !exclude.contains(&decoy)
                && !decoys.contains(&decoy)
            {
                decoys.push(decoy);
            }
        }

        if decoys.len() < count {
            tracing::warn!(
                "Could only generate {} IPv6 decoys (requested {})",
                decoys.len(),
                count
            );
        }

        decoys
    }

    /// Check if IPv4 address is in reserved range
    fn is_reserved_ipv4(ip: Ipv4Addr) -> bool {
        let octets = ip.octets();
        matches!(octets[0], 0 | 10 | 127 | 169 | 172 | 192 | 224..=255)
            || (octets[0] == 172 && (16..=31).contains(&octets[1]))
            || (octets[0] == 192 && octets[1] == 168)
            || (octets[0] == 169 && octets[1] == 254)
    }

    /// Check if IPv6 address is reserved/special
    fn is_reserved_ipv6(ip: Ipv6Addr) -> bool {
        // Loopback (::1)
        if ip.is_loopback() {
            return true;
        }

        // Unspecified (::)
        if ip.is_unspecified() {
            return true;
        }

        // Multicast (ff00::/8)
        if ip.is_multicast() {
            return true;
        }

        let segments = ip.segments();

        // Link-local (fe80::/10)
        if (segments[0] & 0xffc0) == 0xfe80 {
            return true;
        }

        // Unique local addresses (fc00::/7)
        if (segments[0] & 0xfe00) == 0xfc00 {
            return true;
        }

        // Documentation prefix (2001:db8::/32)
        if segments[0] == 0x2001 && segments[1] == 0x0db8 {
            return true;
        }

        // IPv4-mapped IPv6 (::ffff:0:0/96)
        if segments[0..5] == [0, 0, 0, 0, 0] && segments[5] == 0xffff {
            return true;
        }

        false
    }

    /// Build final decoy list with real IP inserted (supports IPv4 and IPv6)
    fn build_decoy_list(&self, real_ip: IpAddr) -> Vec<IpAddr> {
        let mut all_decoys = self.decoys.clone();

        // Add random decoys if requested
        if let Some(count) = self.random_decoy_count {
            let random = self.generate_random_decoys(real_ip, count, &[real_ip]);
            all_decoys.extend(random);
        }

        // Ensure we don't exceed max
        all_decoys.truncate(MAX_DECOYS - 1);

        // Insert real IP at specified position
        let position = match self.real_placement {
            DecoyPlacement::Fixed(pos) => pos.min(all_decoys.len()),
            DecoyPlacement::Random => {
                if all_decoys.is_empty() {
                    0
                } else {
                    rand::thread_rng().gen_range(0..=all_decoys.len())
                }
            }
        };

        all_decoys.insert(position, real_ip);
        all_decoys
    }

    /// Scan target port with decoy IPs
    ///
    /// # Arguments
    ///
    /// * `target` - Target to scan
    /// * `port` - Port number to probe
    ///
    /// # Returns
    ///
    /// Scan result for the target port
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> prtip_core::Result<()> {
    /// # use prtip_scanner::DecoyScanner;
    /// # use prtip_core::{Config, ScanTarget};
    /// let mut scanner = DecoyScanner::new(Config::default());
    /// scanner.set_random_decoys(5);
    ///
    /// let target = ScanTarget::parse("192.168.1.1")?;
    /// let result = scanner.scan_with_decoys(target, 80).await?;
    /// println!("Port 80 state: {:?}", result.state);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn scan_with_decoys(&mut self, target: ScanTarget, port: u16) -> Result<ScanResult> {
        // Get real source IP (from network interface)
        let real_source = self.get_source_ip(&target)?;

        // Get target IP for backoff check
        let hosts = target.expand_hosts();
        let target_ip = if !hosts.is_empty() {
            hosts[0]
        } else {
            return Err(Error::Network("No hosts in target".to_string()));
        };

        // Check ICMP backoff (if adaptive rate limiting enabled)
        if let Some(limiter) = &self.adaptive_limiter {
            if limiter.is_target_backed_off(target_ip) {
                debug!("Skipping {}:{} (ICMP backoff active)", target_ip, port);
                use chrono::Utc;
                return Ok(ScanResult {
                    target_ip,
                    port,
                    state: PortState::Filtered,
                    response_time: Duration::from_millis(0),
                    timestamp: Utc::now(),
                    banner: None,
                    service: None,
                    version: None,
                    raw_response: None,
                });
            }
        }

        // Build final decoy list
        let decoy_list = self.build_decoy_list(real_source);

        tracing::debug!(
            "Scanning {:?} port {} with {} decoys (real IP at position {})",
            target,
            port,
            decoy_list.len() - 1,
            decoy_list
                .iter()
                .position(|&ip| ip == real_source)
                .unwrap_or(0)
        );

        // Send probes from all decoys in randomized order
        let mut send_order = decoy_list.clone();
        self.shuffle_decoys(&mut send_order);

        for (i, &source_ip) in send_order.iter().enumerate() {
            // Build SYN packet with spoofed source
            let packet = self.build_syn_probe(&target, port, source_ip)?;

            // Send packet
            self.send_raw_packet(&packet).await?;

            // Small random delay between decoys to appear more natural
            if i < send_order.len() - 1 {
                let delay_us =
                    rand::thread_rng().gen_range(MIN_DECOY_DELAY_US..=MAX_DECOY_DELAY_US);
                time::sleep(Duration::from_micros(delay_us)).await;
            }
        }

        // Wait for response (only to real source IP)
        let result = self.wait_for_response(&target, port, real_source).await?;

        Ok(result)
    }

    /// Get source IP for target (from routing table or config)
    fn get_source_ip(&self, target: &ScanTarget) -> Result<IpAddr> {
        // For now, use a placeholder - should integrate with interface detection
        // In production, this would query routing table or use configured source IP

        // Determine IP version from target
        let hosts = target.expand_hosts();
        if !hosts.is_empty() {
            match hosts[0] {
                IpAddr::V4(_) => Ok(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))), // IPv4 placeholder
                IpAddr::V6(_) => Ok(IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1))), // IPv6 placeholder (link-local)
            }
        } else {
            Ok(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))) // Default to IPv4
        }
    }

    /// Build SYN probe packet with spoofed source (supports IPv4 and IPv6)
    fn build_syn_probe(
        &self,
        target: &ScanTarget,
        port: u16,
        source_ip: IpAddr,
    ) -> Result<Vec<u8>> {
        // Extract first host from target
        let hosts = target.expand_hosts();
        if hosts.is_empty() {
            return Err(Error::Network("No hosts in target".to_string()));
        }

        let dest_ip = hosts[0];

        // Ensure IP versions match
        if (source_ip.is_ipv4() && dest_ip.is_ipv6()) || (source_ip.is_ipv6() && dest_ip.is_ipv4())
        {
            return Err(Error::Network(format!(
                "IP version mismatch: source {:?}, dest {:?}",
                source_ip, dest_ip
            )));
        }

        let src_port = self
            .config
            .network
            .source_port
            .unwrap_or_else(|| rand::thread_rng().gen_range(10000..60000));

        // Build SYN packet (dual-stack support)
        let mut builder = TcpPacketBuilder::new()
            .source_port(src_port)
            .dest_port(port)
            .flags(TcpFlags::SYN)
            .sequence(rand::thread_rng().gen())
            .window(65535);

        // Apply evasion features from Sprint 4.20

        // Apply TTL if configured (Phase 2)
        if let Some(ttl) = self.config.evasion.ttl {
            builder = builder.ttl(ttl);
        }

        // Apply bad checksum if configured (Phase 6)
        if self.config.evasion.bad_checksums {
            builder = builder.bad_checksum(true);
        }

        // Build packet based on IP version
        let packet = match (source_ip, dest_ip) {
            (IpAddr::V4(src_v4), IpAddr::V4(dst_v4)) => {
                builder = builder.source_ip(src_v4).dest_ip(dst_v4);
                builder.build_ip_packet()?
            }
            (IpAddr::V6(src_v6), IpAddr::V6(dst_v6)) => {
                builder.build_ipv6_packet(src_v6, dst_v6)?
            }
            _ => unreachable!("IP version mismatch already checked"),
        };

        // Apply fragmentation if configured (Phase 2)
        let packets_to_send: Vec<Vec<u8>> = if self.config.evasion.fragment_packets {
            use prtip_network::fragment_tcp_packet;
            let mtu = self.config.evasion.mtu.unwrap_or(1500);
            fragment_tcp_packet(&packet, mtu)
                .map_err(|e| Error::Network(format!("Fragmentation failed: {}", e)))?
        } else {
            vec![packet]
        };

        // For now, return first packet (TODO: handle multiple fragments properly)
        Ok(packets_to_send.into_iter().next().unwrap_or_default())
    }

    /// Send raw packet (placeholder - should use actual packet sender)
    async fn send_raw_packet(&self, _packet: &[u8]) -> Result<()> {
        // TODO: Integrate with actual raw socket sender
        // For now, just simulate sending
        tracing::trace!("Sending decoy probe packet");
        Ok(())
    }

    /// Wait for response to real source IP
    async fn wait_for_response(
        &self,
        target: &ScanTarget,
        port: u16,
        _real_source: IpAddr,
    ) -> Result<ScanResult> {
        // TODO: Integrate with actual response receiver
        // For now, return placeholder result
        let timeout = Duration::from_millis(1000);
        time::sleep(timeout).await;

        use chrono::Utc;
        use std::time::Duration;

        // Get first host IP from target
        let hosts = target.expand_hosts();
        let target_ip = if !hosts.is_empty() {
            hosts[0]
        } else {
            std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
        };

        Ok(ScanResult {
            target_ip,
            port,
            state: PortState::Filtered, // Placeholder
            response_time: Duration::from_millis(100),
            timestamp: Utc::now(),
            banner: None,
            service: None,
            version: None,
            raw_response: None,
        })
    }

    /// Shuffle decoy order using Fisher-Yates (supports IPv4 and IPv6)
    fn shuffle_decoys(&self, decoys: &mut [IpAddr]) {
        let mut rng = rand::thread_rng();
        for i in (1..decoys.len()).rev() {
            let j = rng.gen_range(0..=i);
            decoys.swap(i, j);
        }
    }

    /// Get current decoy count
    pub fn decoy_count(&self) -> usize {
        self.decoys.len() + self.random_decoy_count.unwrap_or(0) + 1 // +1 for real IP
    }

    /// Clear all decoys
    pub fn clear_decoys(&mut self) {
        self.decoys.clear();
        self.random_decoy_count = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoy_scanner_creation() {
        let scanner = DecoyScanner::new(Config::default());
        assert_eq!(scanner.decoy_count(), 1); // Only real IP
    }

    #[test]
    fn test_add_decoy() {
        let mut scanner = DecoyScanner::new(Config::default());
        scanner.add_decoy(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
        scanner.add_decoy(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 101)));

        assert_eq!(scanner.decoy_count(), 3); // 2 decoys + real IP
    }

    #[test]
    fn test_random_decoys() {
        let mut scanner = DecoyScanner::new(Config::default());
        scanner.set_random_decoys(5);

        assert_eq!(scanner.decoy_count(), 6); // 5 random + real IP
    }

    #[test]
    fn test_max_decoys() {
        let mut scanner = DecoyScanner::new(Config::default());
        scanner.set_random_decoys(300); // Request more than max

        assert!(scanner.decoy_count() <= MAX_DECOYS);
    }

    #[test]
    fn test_decoy_placement_fixed() {
        let mut scanner = DecoyScanner::new(Config::default());
        scanner.set_real_position(Some(2));

        assert_eq!(scanner.real_placement, DecoyPlacement::Fixed(2));
    }

    #[test]
    fn test_decoy_placement_random() {
        let mut scanner = DecoyScanner::new(Config::default());
        scanner.set_real_position(None);

        assert_eq!(scanner.real_placement, DecoyPlacement::Random);
    }

    #[test]
    fn test_generate_random_decoys() {
        let scanner = DecoyScanner::new(Config::default());
        let target = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let exclude = vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))];
        let decoys = scanner.generate_random_decoys(target, 10, &exclude);

        assert_eq!(decoys.len(), 10);

        // Check no duplicates
        let unique: HashSet<_> = decoys.iter().collect();
        assert_eq!(unique.len(), 10);

        // Check excluded IP not in list
        assert!(!decoys.contains(&exclude[0]));
    }

    #[test]
    fn test_is_reserved_ipv4() {
        assert!(DecoyScanner::is_reserved_ipv4(Ipv4Addr::new(10, 0, 0, 1))); // Private
        assert!(DecoyScanner::is_reserved_ipv4(Ipv4Addr::new(127, 0, 0, 1))); // Loopback
        assert!(DecoyScanner::is_reserved_ipv4(Ipv4Addr::new(
            192, 168, 1, 1
        ))); // Private
        assert!(DecoyScanner::is_reserved_ipv4(Ipv4Addr::new(224, 0, 0, 1))); // Multicast
        assert!(!DecoyScanner::is_reserved_ipv4(Ipv4Addr::new(8, 8, 8, 8))); // Public
    }

    #[test]
    fn test_build_decoy_list_fixed_position() {
        let mut scanner = DecoyScanner::new(Config::default());
        scanner.add_decoy(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
        scanner.add_decoy(IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2)));
        scanner.set_real_position(Some(1)); // Real IP at position 1

        let real_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let list = scanner.build_decoy_list(real_ip);

        assert_eq!(list.len(), 3);
        assert_eq!(list[1], real_ip); // Real IP at position 1
    }

    #[test]
    fn test_build_decoy_list_with_random() {
        let mut scanner = DecoyScanner::new(Config::default());
        scanner.set_random_decoys(3);

        let real_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let list = scanner.build_decoy_list(real_ip);

        assert_eq!(list.len(), 4); // 3 random + real
        assert!(list.contains(&real_ip));
    }

    #[test]
    fn test_shuffle_decoys() {
        let scanner = DecoyScanner::new(Config::default());
        let original = vec![
            IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
            IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2)),
            IpAddr::V4(Ipv4Addr::new(3, 3, 3, 3)),
            IpAddr::V4(Ipv4Addr::new(4, 4, 4, 4)),
        ];

        let mut shuffled = original.clone();
        scanner.shuffle_decoys(&mut shuffled);

        // Should contain same elements (maybe different order)
        assert_eq!(shuffled.len(), original.len());
        for ip in &original {
            assert!(shuffled.contains(ip));
        }
    }

    #[test]
    fn test_clear_decoys() {
        let mut scanner = DecoyScanner::new(Config::default());
        scanner.add_decoy(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
        scanner.set_random_decoys(5);

        assert_eq!(scanner.decoy_count(), 7);

        scanner.clear_decoys();
        assert_eq!(scanner.decoy_count(), 1); // Only real IP remains
    }
}
