//! Core types for network scanning

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;
use std::time::Duration;

/// Scan target specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanTarget {
    /// Target network or single IP
    pub network: IpNetwork,
    /// Optional hostname for display
    pub hostname: Option<String>,
}

impl ScanTarget {
    /// Parse a target specification (IP, CIDR, or hostname)
    pub fn parse(input: &str) -> Result<Self> {
        // Try parsing as CIDR first
        if let Ok(network) = input.parse::<IpNetwork>() {
            return Ok(Self {
                network,
                hostname: None,
            });
        }

        // Try parsing as single IP
        if let Ok(ip) = input.parse::<IpAddr>() {
            let network = match ip {
                IpAddr::V4(addr) => IpNetwork::V4(ipnetwork::Ipv4Network::new(addr, 32)?),
                IpAddr::V6(addr) => IpNetwork::V6(ipnetwork::Ipv6Network::new(addr, 128)?),
            };
            return Ok(Self {
                network,
                hostname: None,
            });
        }

        // Assume it's a hostname
        Ok(Self {
            network: IpNetwork::V4(ipnetwork::Ipv4Network::new(
                std::net::Ipv4Addr::UNSPECIFIED,
                32,
            )?),
            hostname: Some(input.to_string()),
        })
    }

    /// Check if this is a single host (not a network range)
    pub fn is_single_host(&self) -> bool {
        match self.network {
            IpNetwork::V4(net) => net.prefix() == 32,
            IpNetwork::V6(net) => net.prefix() == 128,
        }
    }

    /// Get the number of hosts in this target
    pub fn host_count(&self) -> u64 {
        match self.network {
            IpNetwork::V4(net) => {
                let size = 2u64.pow((32 - net.prefix()) as u32);
                // Subtract network and broadcast addresses for non-/32
                if net.prefix() < 32 {
                    size.saturating_sub(2)
                } else {
                    size
                }
            }
            IpNetwork::V6(net) => {
                let prefix = net.prefix();
                if prefix >= 64 {
                    2u64.pow((128 - prefix) as u32)
                } else {
                    u64::MAX // Too large to represent
                }
            }
        }
    }

    /// Expand into individual host IPs
    pub fn expand_hosts(&self) -> Vec<IpAddr> {
        self.network.iter().collect()
    }
}

/// Port range specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortRange {
    /// Single port
    Single(u16),
    /// Range of ports (inclusive)
    Range(u16, u16),
    /// List of individual ports and ranges
    List(Vec<PortRange>),
}

impl PortRange {
    /// Parse port specification: "80", "1-1000", "80,443,8080", "1-100,443"
    pub fn parse(input: &str) -> Result<Self> {
        if input.is_empty() {
            return Err(Error::InvalidPortRange(
                "empty port specification".to_string(),
            ));
        }

        // Check for comma-separated list
        if input.contains(',') {
            let parts: Result<Vec<PortRange>> = input
                .split(',')
                .map(|s| PortRange::parse(s.trim()))
                .collect();
            return Ok(PortRange::List(parts?));
        }

        // Check for range
        if input.contains('-') {
            let parts: Vec<&str> = input.split('-').collect();
            if parts.len() != 2 {
                return Err(Error::InvalidPortRange(format!(
                    "invalid range format: {}",
                    input
                )));
            }

            let start: u16 = parts[0].trim().parse().map_err(|_| {
                Error::InvalidPortRange(format!("invalid port number: {}", parts[0]))
            })?;
            let end: u16 = parts[1].trim().parse().map_err(|_| {
                Error::InvalidPortRange(format!("invalid port number: {}", parts[1]))
            })?;

            if start == 0 {
                return Err(Error::InvalidPortRange("port 0 is invalid".to_string()));
            }
            if end == 0 {
                return Err(Error::InvalidPortRange("port 0 is invalid".to_string()));
            }
            if end < start {
                return Err(Error::InvalidPortRange(format!(
                    "end port {} < start port {}",
                    end, start
                )));
            }

            return Ok(PortRange::Range(start, end));
        }

        // Single port
        let port: u16 = input
            .trim()
            .parse()
            .map_err(|_| Error::InvalidPortRange(format!("invalid port number: {}", input)))?;

        if port == 0 {
            return Err(Error::InvalidPortRange("port 0 is invalid".to_string()));
        }

        Ok(PortRange::Single(port))
    }

    /// Get iterator over all ports in this range
    pub fn iter(&self) -> PortRangeIterator {
        PortRangeIterator::new(self.clone())
    }

    /// Count total ports in range
    pub fn count(&self) -> usize {
        match self {
            PortRange::Single(_) => 1,
            PortRange::Range(start, end) => *end as usize - *start as usize + 1,
            PortRange::List(ranges) => ranges.iter().map(|r| r.count()).sum(),
        }
    }
}

impl fmt::Display for PortRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortRange::Single(port) => write!(f, "{}", port),
            PortRange::Range(start, end) => write!(f, "{}-{}", start, end),
            PortRange::List(ranges) => {
                let parts: Vec<String> = ranges.iter().map(|r| r.to_string()).collect();
                write!(f, "{}", parts.join(","))
            }
        }
    }
}

/// Iterator for port ranges
pub struct PortRangeIterator {
    ranges: Vec<PortRange>,
    current_range_idx: usize,
    current_port: u16,
}

impl PortRangeIterator {
    fn new(range: PortRange) -> Self {
        let ranges = match range {
            PortRange::List(list) => list,
            single => vec![single],
        };

        let current_port = if let Some(first) = ranges.first() {
            match first {
                PortRange::Single(p) => *p,
                PortRange::Range(start, _) => *start,
                PortRange::List(_) => unreachable!(),
            }
        } else {
            0
        };

        Self {
            ranges,
            current_range_idx: 0,
            current_port,
        }
    }
}

impl Iterator for PortRangeIterator {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_range_idx >= self.ranges.len() {
            return None;
        }

        let current_range = &self.ranges[self.current_range_idx];

        match current_range {
            PortRange::Single(port) => {
                self.current_range_idx += 1;
                if self.current_range_idx < self.ranges.len() {
                    if let Some(next_range) = self.ranges.get(self.current_range_idx) {
                        match next_range {
                            PortRange::Single(p) => self.current_port = *p,
                            PortRange::Range(_start, _) => self.current_port = *_start,
                            _ => {}
                        }
                    }
                }
                Some(*port)
            }
            PortRange::Range(_start, end) => {
                if self.current_port > *end {
                    self.current_range_idx += 1;
                    if self.current_range_idx < self.ranges.len() {
                        if let Some(next_range) = self.ranges.get(self.current_range_idx) {
                            match next_range {
                                PortRange::Single(p) => self.current_port = *p,
                                PortRange::Range(s, _) => self.current_port = *s,
                                _ => {}
                            }
                        }
                        return self.next();
                    }
                    return None;
                }

                let port = self.current_port;
                self.current_port += 1;
                Some(port)
            }
            PortRange::List(_) => unreachable!("Lists should be flattened"),
        }
    }
}

/// State of a scanned port
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PortState {
    /// Port is open and accepting connections
    Open,
    /// Port is closed (RST received)
    Closed,
    /// Port is filtered (no response or ICMP unreachable)
    Filtered,
    /// Port state could not be determined
    Unknown,
}

impl fmt::Display for PortState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortState::Open => write!(f, "open"),
            PortState::Closed => write!(f, "closed"),
            PortState::Filtered => write!(f, "filtered"),
            PortState::Unknown => write!(f, "unknown"),
        }
    }
}

/// Type of scan to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanType {
    /// TCP connect scan (full 3-way handshake)
    Connect,
    /// TCP SYN scan (half-open)
    Syn,
    /// TCP FIN scan (stealth)
    Fin,
    /// TCP NULL scan (no flags)
    Null,
    /// TCP Xmas scan (FIN, PSH, URG flags)
    Xmas,
    /// TCP ACK scan (firewall detection)
    Ack,
    /// UDP scan
    Udp,
    /// Idle scan (zombie)
    Idle,
}

impl fmt::Display for ScanType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanType::Connect => write!(f, "TCP Connect"),
            ScanType::Syn => write!(f, "TCP SYN"),
            ScanType::Fin => write!(f, "TCP FIN"),
            ScanType::Null => write!(f, "TCP NULL"),
            ScanType::Xmas => write!(f, "TCP Xmas"),
            ScanType::Ack => write!(f, "TCP ACK"),
            ScanType::Udp => write!(f, "UDP"),
            ScanType::Idle => write!(f, "Idle"),
        }
    }
}

/// Timing template for scan speed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimingTemplate {
    /// T0 - Paranoid (5-minute delays)
    Paranoid,
    /// T1 - Sneaky (15-second delays)
    Sneaky,
    /// T2 - Polite (0.4-second delays)
    Polite,
    /// T3 - Normal (default)
    Normal,
    /// T4 - Aggressive (fast/reliable networks)
    Aggressive,
    /// T5 - Insane (maximum speed, sacrifices accuracy)
    Insane,
}

impl TimingTemplate {
    /// Get timeout in milliseconds for this timing template
    pub fn timeout_ms(&self) -> u64 {
        match self {
            TimingTemplate::Paranoid => 300_000, // 5 minutes
            TimingTemplate::Sneaky => 15_000,    // 15 seconds
            TimingTemplate::Polite => 10_000,    // 10 seconds
            TimingTemplate::Normal => 3_000,     // 3 seconds
            TimingTemplate::Aggressive => 1_000, // 1 second
            TimingTemplate::Insane => 250,       // 250 ms
        }
    }

    /// Get delay in milliseconds between probes
    pub fn delay_ms(&self) -> u64 {
        match self {
            TimingTemplate::Paranoid => 300_000, // 5 minutes
            TimingTemplate::Sneaky => 15_000,    // 15 seconds
            TimingTemplate::Polite => 400,       // 0.4 seconds
            TimingTemplate::Normal => 0,         // No delay
            TimingTemplate::Aggressive => 0,
            TimingTemplate::Insane => 0,
        }
    }

    /// Get maximum parallelism for this timing
    pub fn max_parallelism(&self) -> usize {
        match self {
            TimingTemplate::Paranoid => 1,
            TimingTemplate::Sneaky => 10,
            TimingTemplate::Polite => 100,
            TimingTemplate::Normal => 1000,
            TimingTemplate::Aggressive => 5000,
            TimingTemplate::Insane => 10000,
        }
    }

    /// Get maximum retries for this timing
    pub fn max_retries(&self) -> Option<u8> {
        match self {
            TimingTemplate::Paranoid => Some(5),
            TimingTemplate::Sneaky => Some(5),
            TimingTemplate::Polite => Some(5),
            TimingTemplate::Normal => Some(2),
            TimingTemplate::Aggressive => Some(6),
            TimingTemplate::Insane => Some(2),
        }
    }
}

impl fmt::Display for TimingTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimingTemplate::Paranoid => write!(f, "T0 (Paranoid)"),
            TimingTemplate::Sneaky => write!(f, "T1 (Sneaky)"),
            TimingTemplate::Polite => write!(f, "T2 (Polite)"),
            TimingTemplate::Normal => write!(f, "T3 (Normal)"),
            TimingTemplate::Aggressive => write!(f, "T4 (Aggressive)"),
            TimingTemplate::Insane => write!(f, "T5 (Insane)"),
        }
    }
}

/// Result of scanning a single port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Target IP address
    pub target_ip: IpAddr,
    /// Port number
    pub port: u16,
    /// Port state
    pub state: PortState,
    /// Response time
    pub response_time: Duration,
    /// Timestamp of the scan
    pub timestamp: DateTime<Utc>,
    /// Optional banner grabbed from service
    pub banner: Option<String>,
    /// Optional service name
    pub service: Option<String>,
}

impl ScanResult {
    /// Create a new scan result
    pub fn new(target_ip: IpAddr, port: u16, state: PortState) -> Self {
        Self {
            target_ip,
            port,
            state,
            response_time: Duration::from_secs(0),
            timestamp: Utc::now(),
            banner: None,
            service: None,
        }
    }

    /// Set response time
    pub fn with_response_time(mut self, duration: Duration) -> Self {
        self.response_time = duration;
        self
    }

    /// Set banner
    pub fn with_banner(mut self, banner: String) -> Self {
        self.banner = Some(banner);
        self
    }

    /// Set service name
    pub fn with_service(mut self, service: String) -> Self {
        self.service = Some(service);
        self
    }

    /// Get target IP
    pub fn target_ip(&self) -> IpAddr {
        self.target_ip
    }

    /// Get port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get state
    pub fn state(&self) -> PortState {
        self.state
    }

    /// Get response time
    pub fn response_time(&self) -> Duration {
        self.response_time
    }

    /// Get service name
    pub fn service(&self) -> Option<&str> {
        self.service.as_deref()
    }

    /// Get banner
    pub fn banner(&self) -> Option<&str> {
        self.banner.as_deref()
    }
}

impl fmt::Display for ScanResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} - {} ({:.2}ms)",
            self.target_ip,
            self.port,
            self.state,
            self.response_time.as_secs_f64() * 1000.0
        )?;

        if let Some(service) = &self.service {
            write!(f, " [{}]", service)?;
        }

        if let Some(banner) = &self.banner {
            write!(f, " \"{}\"", banner.chars().take(50).collect::<String>())?;
        }

        Ok(())
    }
}

/// Port filtering for exclusion/inclusion lists
///
/// Provides efficient port filtering using hash sets for O(1) lookups.
/// Inspired by RustScan and Naabu patterns for port exclusion.
///
/// # Examples
///
/// ```
/// use prtip_core::types::{PortFilter, PortRange};
///
/// // Create filter excluding common web ports
/// let filter = PortFilter::exclude(&["80", "443", "8080"]).unwrap();
/// assert!(!filter.allows(80));
/// assert!(filter.allows(22));
///
/// // Create filter allowing only SSH and HTTP
/// let filter = PortFilter::include(&["22", "80"]).unwrap();
/// assert!(filter.allows(22));
/// assert!(!filter.allows(443));
/// ```
#[derive(Debug, Clone)]
pub struct PortFilter {
    /// Filter mode: true = whitelist (include only), false = blacklist (exclude)
    is_whitelist: bool,
    /// Set of ports to filter
    ports: std::collections::HashSet<u16>,
}

impl PortFilter {
    /// Create a new empty filter (allows all ports)
    pub fn new() -> Self {
        Self {
            is_whitelist: false,
            ports: std::collections::HashSet::new(),
        }
    }

    /// Create a whitelist filter (only allows specified ports)
    ///
    /// # Arguments
    ///
    /// * `specs` - Port specifications: ["80", "443", "8080-8090"]
    ///
    /// # Example
    ///
    /// ```
    /// use prtip_core::types::PortFilter;
    ///
    /// let filter = PortFilter::include(&["22", "80", "443"]).unwrap();
    /// assert!(filter.allows(22));
    /// assert!(!filter.allows(8080));
    /// ```
    pub fn include(specs: &[&str]) -> Result<Self> {
        let mut ports = std::collections::HashSet::new();
        for spec in specs {
            let range = PortRange::parse(spec)?;
            for port in range.iter() {
                ports.insert(port);
            }
        }
        Ok(Self {
            is_whitelist: true,
            ports,
        })
    }

    /// Create a blacklist filter (excludes specified ports)
    ///
    /// # Arguments
    ///
    /// * `specs` - Port specifications: ["80", "443", "8080-8090"]
    ///
    /// # Example
    ///
    /// ```
    /// use prtip_core::types::PortFilter;
    ///
    /// let filter = PortFilter::exclude(&["80", "443"]).unwrap();
    /// assert!(filter.allows(22));
    /// assert!(!filter.allows(80));
    /// ```
    pub fn exclude(specs: &[&str]) -> Result<Self> {
        let mut ports = std::collections::HashSet::new();
        for spec in specs {
            let range = PortRange::parse(spec)?;
            for port in range.iter() {
                ports.insert(port);
            }
        }
        Ok(Self {
            is_whitelist: false,
            ports,
        })
    }

    /// Check if a port is allowed by this filter
    ///
    /// # Arguments
    ///
    /// * `port` - Port number to check
    ///
    /// # Returns
    ///
    /// `true` if port is allowed, `false` if filtered out
    pub fn allows(&self, port: u16) -> bool {
        if self.ports.is_empty() {
            // Empty filter allows all ports
            return true;
        }

        if self.is_whitelist {
            // Whitelist: allow only if in set
            self.ports.contains(&port)
        } else {
            // Blacklist: allow if NOT in set
            !self.ports.contains(&port)
        }
    }

    /// Filter a list of ports, returning only allowed ones
    ///
    /// # Arguments
    ///
    /// * `ports` - Iterator of ports to filter
    ///
    /// # Returns
    ///
    /// Iterator of allowed ports
    ///
    /// # Example
    ///
    /// ```
    /// use prtip_core::types::PortFilter;
    ///
    /// let filter = PortFilter::exclude(&["80", "443"]).unwrap();
    /// let ports = vec![22, 80, 443, 8080];
    /// let filtered = filter.filter_ports(ports);
    /// assert_eq!(filtered, vec![22, 8080]);
    /// ```
    pub fn filter_ports(&self, ports: Vec<u16>) -> Vec<u16> {
        ports.into_iter().filter(|&p| self.allows(p)).collect()
    }

    /// Get the number of filtered ports
    pub fn count(&self) -> usize {
        self.ports.len()
    }

    /// Check if filter is empty (allows all)
    pub fn is_empty(&self) -> bool {
        self.ports.is_empty()
    }

    /// Check if this is a whitelist filter
    pub fn is_whitelist(&self) -> bool {
        self.is_whitelist
    }
}

impl Default for PortFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_range_single() {
        let range = PortRange::parse("80").unwrap();
        assert_eq!(range, PortRange::Single(80));
        assert_eq!(range.count(), 1);
        assert_eq!(range.iter().collect::<Vec<_>>(), vec![80]);
    }

    #[test]
    fn test_port_range_range() {
        let range = PortRange::parse("80-83").unwrap();
        assert_eq!(range, PortRange::Range(80, 83));
        assert_eq!(range.count(), 4);
        assert_eq!(range.iter().collect::<Vec<_>>(), vec![80, 81, 82, 83]);
    }

    #[test]
    fn test_port_range_list() {
        let range = PortRange::parse("80,443,8080").unwrap();
        assert_eq!(range.count(), 3);
        let ports: Vec<u16> = range.iter().collect();
        assert_eq!(ports, vec![80, 443, 8080]);
    }

    #[test]
    fn test_port_range_mixed() {
        let range = PortRange::parse("80-82,443,8080-8082").unwrap();
        assert_eq!(range.count(), 7); // 3 + 1 + 3 = 7
        let ports: Vec<u16> = range.iter().collect();
        assert_eq!(ports, vec![80, 81, 82, 443, 8080, 8081, 8082]);
    }

    #[test]
    fn test_port_range_invalid() {
        assert!(PortRange::parse("0").is_err());
        assert!(PortRange::parse("99999").is_err());
        assert!(PortRange::parse("abc").is_err());
        assert!(PortRange::parse("100-50").is_err());
        assert!(PortRange::parse("").is_err());
    }

    #[test]
    fn test_scan_target_single_ip() {
        let target = ScanTarget::parse("192.168.1.1").unwrap();
        assert!(target.is_single_host());
        assert_eq!(target.host_count(), 1);
    }

    #[test]
    fn test_scan_target_cidr() {
        let target = ScanTarget::parse("192.168.1.0/24").unwrap();
        assert!(!target.is_single_host());
        assert_eq!(target.host_count(), 254); // 256 - network - broadcast
    }

    #[test]
    fn test_scan_target_ipv6() {
        let target = ScanTarget::parse("::1").unwrap();
        assert!(target.is_single_host());
    }

    #[test]
    fn test_scan_target_hostname() {
        let target = ScanTarget::parse("example.com").unwrap();
        assert_eq!(target.hostname, Some("example.com".to_string()));
    }

    #[test]
    fn test_port_state_display() {
        assert_eq!(PortState::Open.to_string(), "open");
        assert_eq!(PortState::Closed.to_string(), "closed");
        assert_eq!(PortState::Filtered.to_string(), "filtered");
        assert_eq!(PortState::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_port_state_ordering() {
        // Enum ordering: Open < Closed < Filtered < Unknown
        assert!(PortState::Open < PortState::Closed);
        assert!(PortState::Closed < PortState::Filtered);
        assert!(PortState::Filtered < PortState::Unknown);
    }

    #[test]
    fn test_scan_result_creation() {
        let ip = "192.168.1.1".parse().unwrap();
        let result = ScanResult::new(ip, 80, PortState::Open)
            .with_response_time(Duration::from_millis(150))
            .with_service("http".to_string())
            .with_banner("Apache/2.4.41".to_string());

        assert_eq!(result.target_ip, ip);
        assert_eq!(result.port, 80);
        assert_eq!(result.state, PortState::Open);
        assert_eq!(result.response_time, Duration::from_millis(150));
        assert_eq!(result.service, Some("http".to_string()));
        assert_eq!(result.banner, Some("Apache/2.4.41".to_string()));
    }

    #[test]
    fn test_scan_result_serialization() {
        let ip = "192.168.1.1".parse().unwrap();
        let result = ScanResult::new(ip, 80, PortState::Open);

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ScanResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.target_ip, deserialized.target_ip);
        assert_eq!(result.port, deserialized.port);
        assert_eq!(result.state, deserialized.state);
    }

    #[test]
    fn test_timing_template_values() {
        assert_eq!(TimingTemplate::Paranoid.timeout_ms(), 300_000);
        assert_eq!(TimingTemplate::Normal.timeout_ms(), 3_000);
        assert_eq!(TimingTemplate::Insane.timeout_ms(), 250);

        assert_eq!(TimingTemplate::Paranoid.delay_ms(), 300_000);
        assert_eq!(TimingTemplate::Normal.delay_ms(), 0);

        assert_eq!(TimingTemplate::Paranoid.max_parallelism(), 1);
        assert_eq!(TimingTemplate::Insane.max_parallelism(), 10000);
    }

    #[test]
    fn test_scan_type_display() {
        assert_eq!(ScanType::Connect.to_string(), "TCP Connect");
        assert_eq!(ScanType::Syn.to_string(), "TCP SYN");
        assert_eq!(ScanType::Udp.to_string(), "UDP");
    }

    #[test]
    fn test_port_filter_empty() {
        let filter = PortFilter::new();
        assert!(filter.is_empty());
        assert!(filter.allows(80));
        assert!(filter.allows(443));
        assert!(filter.allows(8080));
    }

    #[test]
    fn test_port_filter_exclude_single() {
        let filter = PortFilter::exclude(&["80"]).unwrap();
        assert!(!filter.is_whitelist());
        assert!(!filter.allows(80));
        assert!(filter.allows(443));
        assert!(filter.allows(22));
    }

    #[test]
    fn test_port_filter_exclude_multiple() {
        let filter = PortFilter::exclude(&["80", "443", "8080"]).unwrap();
        assert!(!filter.allows(80));
        assert!(!filter.allows(443));
        assert!(!filter.allows(8080));
        assert!(filter.allows(22));
        assert!(filter.allows(3389));
    }

    #[test]
    fn test_port_filter_exclude_range() {
        let filter = PortFilter::exclude(&["8000-8090"]).unwrap();
        assert!(!filter.allows(8000));
        assert!(!filter.allows(8050));
        assert!(!filter.allows(8090));
        assert!(filter.allows(7999));
        assert!(filter.allows(8091));
        assert_eq!(filter.count(), 91); // 8000-8090 inclusive
    }

    #[test]
    fn test_port_filter_include_single() {
        let filter = PortFilter::include(&["80"]).unwrap();
        assert!(filter.is_whitelist());
        assert!(filter.allows(80));
        assert!(!filter.allows(443));
        assert!(!filter.allows(22));
    }

    #[test]
    fn test_port_filter_include_multiple() {
        let filter = PortFilter::include(&["22", "80", "443"]).unwrap();
        assert!(filter.allows(22));
        assert!(filter.allows(80));
        assert!(filter.allows(443));
        assert!(!filter.allows(8080));
        assert!(!filter.allows(3389));
    }

    #[test]
    fn test_port_filter_include_range() {
        let filter = PortFilter::include(&["80-85"]).unwrap();
        assert!(filter.allows(80));
        assert!(filter.allows(82));
        assert!(filter.allows(85));
        assert!(!filter.allows(79));
        assert!(!filter.allows(86));
    }

    #[test]
    fn test_port_filter_mixed_specs() {
        let filter = PortFilter::exclude(&["80", "443", "8000-8090"]).unwrap();
        assert!(!filter.allows(80));
        assert!(!filter.allows(443));
        assert!(!filter.allows(8050));
        assert!(filter.allows(22));
        assert!(filter.allows(9000));
    }

    #[test]
    fn test_port_filter_ports() {
        let filter = PortFilter::exclude(&["80", "443"]).unwrap();
        let ports = vec![22, 80, 443, 3389, 8080];
        let filtered = filter.filter_ports(ports);
        assert_eq!(filtered, vec![22, 3389, 8080]);
    }

    #[test]
    fn test_port_filter_default() {
        let filter = PortFilter::default();
        assert!(filter.is_empty());
        assert!(filter.allows(80));
    }
}
