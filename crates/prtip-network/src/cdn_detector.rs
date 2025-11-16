//! CDN IP Detection and Deduplication
//!
//! Detects whether an IP address belongs to a CDN or cloud provider to enable
//! intelligent scan deduplication. Scanning CDN edge nodes often produces
//! redundant results since they proxy to origin servers.
//!
//! # Supported CDN Providers
//!
//! - Cloudflare
//! - AWS CloudFront
//! - Azure CDN / Azure Front Door
//! - Akamai
//! - Fastly
//! - Google Cloud CDN
//!
//! # Usage
//!
//! ```
//! use prtip_network::cdn_detector::{CdnDetector, CdnProvider};
//! use std::net::IpAddr;
//!
//! let detector = CdnDetector::new();
//! let ip: IpAddr = "104.16.132.229".parse().unwrap();
//!
//! if let Some(provider) = detector.detect(&ip) {
//!     println!("IP belongs to: {:?}", provider);
//! }
//! ```

use pnet::ipnetwork::{Ipv4Network, Ipv6Network};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// CDN/Cloud provider identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CdnProvider {
    /// Cloudflare CDN
    Cloudflare,
    /// AWS CloudFront
    AwsCloudFront,
    /// Microsoft Azure CDN / Front Door
    AzureCdn,
    /// Akamai
    Akamai,
    /// Fastly
    Fastly,
    /// Google Cloud CDN
    GoogleCloud,
}

impl CdnProvider {
    /// Get human-readable provider name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cloudflare => "Cloudflare",
            Self::AwsCloudFront => "AWS CloudFront",
            Self::AzureCdn => "Azure CDN",
            Self::Akamai => "Akamai",
            Self::Fastly => "Fastly",
            Self::GoogleCloud => "Google Cloud CDN",
        }
    }
}

/// CDN IP range detector with whitelist/blacklist support
#[derive(Debug, Clone)]
pub struct CdnDetector {
    /// CDN IP ranges (provider, IPv4 ranges, IPv6 ranges)
    ranges: Vec<(CdnProvider, Vec<Ipv4Network>, Vec<Ipv6Network>)>,
    /// Whitelisted providers (if Some, only these are considered CDNs)
    whitelist: Option<Vec<CdnProvider>>,
    /// Blacklisted providers (these are never considered CDNs)
    blacklist: Vec<CdnProvider>,
    /// Hash map for O(1) IPv4 /24 prefix lookups (prefix → provider)
    ipv4_prefix_map: HashMap<u32, CdnProvider>,
    /// Hash map for O(1) IPv6 /48 prefix lookups (prefix → provider)
    ipv6_prefix_map: HashMap<u128, CdnProvider>,
}

impl Default for CdnDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CdnDetector {
    /// Create a new CDN detector with default provider ranges
    pub fn new() -> Self {
        let mut detector = Self {
            ranges: Self::default_ranges(),
            whitelist: None,
            blacklist: Vec::new(),
            ipv4_prefix_map: HashMap::new(),
            ipv6_prefix_map: HashMap::new(),
        };
        detector.build_prefix_maps();
        detector
    }

    /// Create detector with specific whitelisted providers
    ///
    /// Only the specified providers will be considered CDNs.
    pub fn with_whitelist(providers: Vec<CdnProvider>) -> Self {
        let mut detector = Self {
            ranges: Self::default_ranges(),
            whitelist: Some(providers),
            blacklist: Vec::new(),
            ipv4_prefix_map: HashMap::new(),
            ipv6_prefix_map: HashMap::new(),
        };
        detector.build_prefix_maps();
        detector
    }

    /// Create detector with specific blacklisted providers
    ///
    /// The specified providers will never be considered CDNs.
    pub fn with_blacklist(providers: Vec<CdnProvider>) -> Self {
        let mut detector = Self {
            ranges: Self::default_ranges(),
            whitelist: None,
            blacklist: providers,
            ipv4_prefix_map: HashMap::new(),
            ipv6_prefix_map: HashMap::new(),
        };
        detector.build_prefix_maps();
        detector
    }

    /// Set whitelist (replaces existing)
    pub fn set_whitelist(&mut self, providers: Vec<CdnProvider>) {
        self.whitelist = Some(providers);
        self.build_prefix_maps(); // Rebuild maps when whitelist changes
    }

    /// Clear whitelist
    pub fn clear_whitelist(&mut self) {
        self.whitelist = None;
        self.build_prefix_maps(); // Rebuild maps when whitelist changes
    }

    /// Add provider to blacklist
    pub fn add_to_blacklist(&mut self, provider: CdnProvider) {
        if !self.blacklist.contains(&provider) {
            self.blacklist.push(provider);
            self.build_prefix_maps(); // Rebuild maps when blacklist changes
        }
    }

    /// Remove provider from blacklist
    pub fn remove_from_blacklist(&mut self, provider: CdnProvider) {
        self.blacklist.retain(|p| *p != provider);
        self.build_prefix_maps(); // Rebuild maps when blacklist changes
    }

    /// Build hash maps for O(1) prefix lookups
    ///
    /// Pre-computes /24 prefixes (IPv4) and /48 prefixes (IPv6) for all active
    /// CDN ranges, respecting whitelist/blacklist configuration.
    fn build_prefix_maps(&mut self) {
        self.ipv4_prefix_map.clear();
        self.ipv6_prefix_map.clear();

        for (provider, ipv4_ranges, ipv6_ranges) in &self.ranges {
            // Skip blacklisted providers
            if self.blacklist.contains(provider) {
                continue;
            }

            // Skip non-whitelisted providers (if whitelist is set)
            if let Some(ref whitelist) = self.whitelist {
                if !whitelist.contains(provider) {
                    continue;
                }
            }

            // Build IPv4 prefix map (/24 prefixes)
            for network in ipv4_ranges {
                let prefixes = Self::expand_ipv4_network_to_24_prefixes(*network);
                for prefix in prefixes {
                    // First provider wins (no overwriting)
                    self.ipv4_prefix_map.entry(prefix).or_insert(*provider);
                }
            }

            // Build IPv6 prefix map (/48 prefixes)
            for network in ipv6_ranges {
                let prefixes = Self::expand_ipv6_network_to_48_prefixes(*network);
                for prefix in prefixes {
                    // First provider wins (no overwriting)
                    self.ipv6_prefix_map.entry(prefix).or_insert(*provider);
                }
            }
        }
    }

    /// Convert IPv4 address to /24 prefix (top 24 bits)
    fn ipv4_to_prefix24(ip: Ipv4Addr) -> u32 {
        let octets = ip.octets();
        u32::from_be_bytes([octets[0], octets[1], octets[2], 0])
    }

    /// Convert IPv6 address to /48 prefix (top 48 bits)
    fn ipv6_to_prefix48(ip: Ipv6Addr) -> u128 {
        let segments = ip.segments();
        // Take first 3 segments (48 bits) and zero out the rest
        u128::from(segments[0]) << 112
            | u128::from(segments[1]) << 96
            | u128::from(segments[2]) << 80
    }

    /// Expand IPv4 network to all /24 prefixes it contains
    fn expand_ipv4_network_to_24_prefixes(network: Ipv4Network) -> Vec<u32> {
        let prefix_len = network.prefix();

        if prefix_len >= 24 {
            // Network is /24 or smaller - single prefix
            vec![Self::ipv4_to_prefix24(network.network())]
        } else {
            // Network is larger than /24 - generate all /24 prefixes
            let start_ip = u32::from(network.network());
            let end_ip = u32::from(network.broadcast());

            // Calculate number of /24 networks in this range
            let start_prefix24 = start_ip & 0xFFFFFF00; // Mask to /24
            let end_prefix24 = end_ip & 0xFFFFFF00;

            let mut prefixes = Vec::new();
            let mut current = start_prefix24;

            while current <= end_prefix24 {
                prefixes.push(current);
                current = current.saturating_add(256); // Next /24
                if current == 0 || current < start_prefix24 {
                    // Wrapped around or overflow
                    break;
                }
            }

            prefixes
        }
    }

    /// Expand IPv6 network to all /48 prefixes it contains
    fn expand_ipv6_network_to_48_prefixes(network: Ipv6Network) -> Vec<u128> {
        let prefix_len = network.prefix();

        if prefix_len >= 48 {
            // Network is /48 or smaller - single prefix
            vec![Self::ipv6_to_prefix48(network.network())]
        } else {
            // Network is larger than /48 - generate all /48 prefixes
            // For IPv6, we only generate prefixes for reasonable ranges (< /32)
            // to avoid memory exhaustion
            if prefix_len < 32 {
                // Too large to enumerate - just use network prefix
                return vec![Self::ipv6_to_prefix48(network.network())];
            }

            let start_ip = u128::from(network.network());
            let network_mask = !0u128 << (128 - prefix_len);
            let prefix48_mask = !0u128 << 80; // /48 mask

            let start_prefix48 = start_ip & prefix48_mask;
            let num_prefixes = 1u128 << (48 - prefix_len);

            // Limit to reasonable size (max 65536 prefixes)
            let num_prefixes = num_prefixes.min(65536);

            let mut prefixes = Vec::new();
            for i in 0..num_prefixes {
                let prefix = start_prefix48 | (i << 80);
                // Check if prefix is still within the original network
                if (prefix & network_mask) == (start_ip & network_mask) {
                    prefixes.push(prefix);
                }
            }

            prefixes
        }
    }

    /// Detect if an IP belongs to a CDN provider
    ///
    /// Returns `Some(provider)` if the IP belongs to a CDN, respecting
    /// whitelist/blacklist configuration.
    ///
    /// Uses O(1) hash map lookups for /24 (IPv4) or /48 (IPv6) prefixes,
    /// with fallback to linear search for edge cases not in hash maps.
    pub fn detect(&self, ip: &IpAddr) -> Option<CdnProvider> {
        // Try O(1) hash lookup first
        match ip {
            IpAddr::V4(ipv4) => {
                let prefix = Self::ipv4_to_prefix24(*ipv4);
                if let Some(provider) = self.ipv4_prefix_map.get(&prefix) {
                    return Some(*provider);
                }
            }
            IpAddr::V6(ipv6) => {
                let prefix = Self::ipv6_to_prefix48(*ipv6);
                if let Some(provider) = self.ipv6_prefix_map.get(&prefix) {
                    return Some(*provider);
                }
            }
        }

        // Fallback to linear search for edge cases (rare)
        // This handles CIDR ranges larger than our prefix granularity
        for (provider, ipv4_ranges, ipv6_ranges) in &self.ranges {
            // Check blacklist
            if self.blacklist.contains(provider) {
                continue;
            }

            // Check whitelist (if set)
            if let Some(ref whitelist) = self.whitelist {
                if !whitelist.contains(provider) {
                    continue;
                }
            }

            // Check IP against ranges
            let matches = match ip {
                IpAddr::V4(ipv4) => ipv4_ranges.iter().any(|range| range.contains(*ipv4)),
                IpAddr::V6(ipv6) => ipv6_ranges.iter().any(|range| range.contains(*ipv6)),
            };

            if matches {
                return Some(*provider);
            }
        }

        None
    }

    /// Check if IP is a CDN (any provider)
    pub fn is_cdn(&self, ip: &IpAddr) -> bool {
        self.detect(ip).is_some()
    }

    /// Get all configured CDN providers (after whitelist/blacklist filtering)
    pub fn active_providers(&self) -> Vec<CdnProvider> {
        let mut providers: Vec<CdnProvider> = self.ranges.iter().map(|(p, _, _)| *p).collect();

        // Apply whitelist
        if let Some(ref whitelist) = self.whitelist {
            providers.retain(|p| whitelist.contains(p));
        }

        // Apply blacklist
        providers.retain(|p| !self.blacklist.contains(p));

        providers
    }

    /// Default CDN IP ranges
    ///
    /// Based on public documentation as of 2025-01:
    /// - Cloudflare: https://www.cloudflare.com/ips/
    /// - AWS: https://ip-ranges.amazonaws.com/ip-ranges.json
    /// - Azure: Microsoft Service Tags
    /// - Akamai: ASN-based estimates (104.64.0.0/10)
    /// - Fastly: https://api.fastly.com/public-ip-list
    /// - Google Cloud: https://www.gstatic.com/ipranges/cloud.json
    fn default_ranges() -> Vec<(CdnProvider, Vec<Ipv4Network>, Vec<Ipv6Network>)> {
        vec![
            // Cloudflare
            (
                CdnProvider::Cloudflare,
                vec![
                    "173.245.48.0/20".parse().unwrap(),
                    "103.21.244.0/22".parse().unwrap(),
                    "103.22.200.0/22".parse().unwrap(),
                    "103.31.4.0/22".parse().unwrap(),
                    "141.101.64.0/18".parse().unwrap(),
                    "108.162.192.0/18".parse().unwrap(),
                    "190.93.240.0/20".parse().unwrap(),
                    "188.114.96.0/20".parse().unwrap(),
                    "197.234.240.0/22".parse().unwrap(),
                    "198.41.128.0/17".parse().unwrap(),
                    "162.158.0.0/15".parse().unwrap(),
                    "104.16.0.0/13".parse().unwrap(),
                    "104.24.0.0/14".parse().unwrap(),
                    "172.64.0.0/13".parse().unwrap(),
                    "131.0.72.0/22".parse().unwrap(),
                ],
                vec![
                    "2400:cb00::/32".parse().unwrap(),
                    "2606:4700::/32".parse().unwrap(),
                    "2803:f800::/32".parse().unwrap(),
                    "2405:b500::/32".parse().unwrap(),
                    "2405:8100::/32".parse().unwrap(),
                    "2a06:98c0::/29".parse().unwrap(),
                    "2c0f:f248::/32".parse().unwrap(),
                ],
            ),
            // AWS CloudFront (subset - most common ranges)
            (
                CdnProvider::AwsCloudFront,
                vec![
                    "13.32.0.0/15".parse().unwrap(),
                    "13.35.0.0/16".parse().unwrap(),
                    "13.224.0.0/14".parse().unwrap(),
                    "13.249.0.0/16".parse().unwrap(),
                    "18.64.0.0/14".parse().unwrap(),
                    "34.195.252.0/24".parse().unwrap(),
                    "34.226.14.0/24".parse().unwrap(),
                    "52.46.0.0/18".parse().unwrap(),
                    "52.84.0.0/15".parse().unwrap(),
                    "52.222.128.0/17".parse().unwrap(),
                    "54.182.0.0/16".parse().unwrap(),
                    "54.192.0.0/16".parse().unwrap(),
                    "54.230.0.0/16".parse().unwrap(),
                    "54.239.128.0/18".parse().unwrap(),
                    "54.239.192.0/19".parse().unwrap(),
                    "99.84.0.0/16".parse().unwrap(),
                    "130.176.0.0/16".parse().unwrap(),
                    "204.246.164.0/22".parse().unwrap(),
                    "204.246.168.0/22".parse().unwrap(),
                    "205.251.192.0/19".parse().unwrap(),
                    "205.251.249.0/24".parse().unwrap(),
                    "205.251.250.0/23".parse().unwrap(),
                    "205.251.252.0/23".parse().unwrap(),
                    "205.251.254.0/24".parse().unwrap(),
                ],
                vec![
                    "2600:9000::/28".parse().unwrap(),
                    "2600:9000:f000::/36".parse().unwrap(),
                ],
            ),
            // Azure CDN / Front Door (subset)
            (
                CdnProvider::AzureCdn,
                vec![
                    "13.73.200.0/24".parse().unwrap(),
                    "20.21.0.0/16".parse().unwrap(),
                    "20.36.0.0/16".parse().unwrap(),
                    "20.150.0.0/15".parse().unwrap(),
                    "20.157.0.0/16".parse().unwrap(),
                    "20.190.128.0/18".parse().unwrap(),
                    "40.90.23.0/24".parse().unwrap(),
                    "40.90.24.0/22".parse().unwrap(),
                    "147.243.0.0/16".parse().unwrap(),
                ],
                vec!["2603:1030::/25".parse().unwrap()],
            ),
            // Akamai (primary range)
            (
                CdnProvider::Akamai,
                vec![
                    "23.0.0.0/12".parse().unwrap(),
                    "104.64.0.0/10".parse().unwrap(),
                    "184.24.0.0/13".parse().unwrap(),
                    "2.16.0.0/13".parse().unwrap(),
                ],
                vec!["2600:1400::/24".parse().unwrap()],
            ),
            // Fastly
            (
                CdnProvider::Fastly,
                vec![
                    "23.235.32.0/20".parse().unwrap(),
                    "43.249.72.0/22".parse().unwrap(),
                    "103.244.50.0/24".parse().unwrap(),
                    "103.245.222.0/23".parse().unwrap(),
                    "103.245.224.0/24".parse().unwrap(),
                    "104.156.80.0/20".parse().unwrap(),
                    "146.75.0.0/17".parse().unwrap(),
                    "151.101.0.0/16".parse().unwrap(),
                    "157.52.64.0/18".parse().unwrap(),
                    "167.82.0.0/17".parse().unwrap(),
                    "167.82.128.0/20".parse().unwrap(),
                    "167.82.160.0/20".parse().unwrap(),
                    "167.82.224.0/20".parse().unwrap(),
                    "172.111.64.0/18".parse().unwrap(),
                    "185.31.16.0/22".parse().unwrap(),
                    "199.27.72.0/21".parse().unwrap(),
                    "199.232.0.0/16".parse().unwrap(),
                ],
                vec![
                    "2a04:4e40::/32".parse().unwrap(),
                    "2a04:4e41::/32".parse().unwrap(),
                    "2a04:4e42::/32".parse().unwrap(),
                ],
            ),
            // Google Cloud CDN (subset)
            (
                CdnProvider::GoogleCloud,
                vec![
                    "34.64.0.0/10".parse().unwrap(),
                    "35.184.0.0/13".parse().unwrap(),
                    "35.192.0.0/12".parse().unwrap(),
                    "35.208.0.0/12".parse().unwrap(),
                    "35.224.0.0/12".parse().unwrap(),
                    "35.240.0.0/13".parse().unwrap(),
                ],
                vec!["2600:1900::/28".parse().unwrap()],
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_detection() {
        let detector = CdnDetector::new();

        // Known Cloudflare IPs
        let cf_ips = vec![
            "104.16.132.229".parse().unwrap(), // Cloudflare range 104.16.0.0/13
            "172.64.155.89".parse().unwrap(),  // Cloudflare range 172.64.0.0/13
            "2606:4700:20::681a:8ce5".parse().unwrap(), // Cloudflare IPv6
        ];

        for ip in cf_ips {
            assert_eq!(
                detector.detect(&ip),
                Some(CdnProvider::Cloudflare),
                "IP {} should be detected as Cloudflare",
                ip
            );
            assert!(detector.is_cdn(&ip));
        }
    }

    #[test]
    fn test_aws_cloudfront_detection() {
        let detector = CdnDetector::new();

        // Known AWS CloudFront IPs
        let aws_ips = vec![
            "13.32.1.1".parse().unwrap(),  // CloudFront range
            "54.192.1.1".parse().unwrap(), // CloudFront range
        ];

        for ip in aws_ips {
            assert_eq!(detector.detect(&ip), Some(CdnProvider::AwsCloudFront));
            assert!(detector.is_cdn(&ip));
        }
    }

    #[test]
    fn test_fastly_detection() {
        let detector = CdnDetector::new();

        let fastly_ips = vec![
            "151.101.1.1".parse().unwrap(),  // Fastly range
            "185.31.17.1".parse().unwrap(),  // Fastly range
            "2a04:4e40::1".parse().unwrap(), // Fastly IPv6
        ];

        for ip in fastly_ips {
            assert_eq!(detector.detect(&ip), Some(CdnProvider::Fastly));
            assert!(detector.is_cdn(&ip));
        }
    }

    #[test]
    fn test_non_cdn_ip() {
        let detector = CdnDetector::new();

        // Private/non-CDN IPs
        let non_cdn_ips = vec![
            "192.168.1.1".parse().unwrap(),
            "10.0.0.1".parse().unwrap(),
            "8.8.8.8".parse().unwrap(), // Google DNS, not CDN
            "1.1.1.1".parse().unwrap(), // Cloudflare DNS, not in CDN ranges
        ];

        for ip in non_cdn_ips {
            assert_eq!(detector.detect(&ip), None, "IP {} should not be CDN", ip);
            assert!(!detector.is_cdn(&ip));
        }
    }

    #[test]
    fn test_whitelist() {
        let detector = CdnDetector::with_whitelist(vec![CdnProvider::Cloudflare]);

        // Cloudflare IP should be detected
        let cf_ip: IpAddr = "104.16.132.229".parse().unwrap();
        assert_eq!(detector.detect(&cf_ip), Some(CdnProvider::Cloudflare));

        // AWS IP should NOT be detected (not whitelisted)
        let aws_ip: IpAddr = "13.32.1.1".parse().unwrap();
        assert_eq!(detector.detect(&aws_ip), None);
    }

    #[test]
    fn test_blacklist() {
        let detector = CdnDetector::with_blacklist(vec![CdnProvider::Cloudflare]);

        // Cloudflare IP should NOT be detected (blacklisted)
        let cf_ip: IpAddr = "104.16.132.229".parse().unwrap();
        assert_eq!(detector.detect(&cf_ip), None);

        // AWS IP should be detected (not blacklisted)
        let aws_ip: IpAddr = "13.32.1.1".parse().unwrap();
        assert_eq!(detector.detect(&aws_ip), Some(CdnProvider::AwsCloudFront));
    }

    #[test]
    fn test_dynamic_whitelist_modification() {
        let mut detector = CdnDetector::new();

        let cf_ip: IpAddr = "104.16.132.229".parse().unwrap();
        let aws_ip: IpAddr = "13.32.1.1".parse().unwrap();

        // Initially both detected
        assert!(detector.is_cdn(&cf_ip));
        assert!(detector.is_cdn(&aws_ip));

        // Set whitelist to Cloudflare only
        detector.set_whitelist(vec![CdnProvider::Cloudflare]);
        assert!(detector.is_cdn(&cf_ip));
        assert!(!detector.is_cdn(&aws_ip));

        // Clear whitelist
        detector.clear_whitelist();
        assert!(detector.is_cdn(&cf_ip));
        assert!(detector.is_cdn(&aws_ip));
    }

    #[test]
    fn test_dynamic_blacklist_modification() {
        let mut detector = CdnDetector::new();

        let cf_ip: IpAddr = "104.16.132.229".parse().unwrap();

        // Initially detected
        assert!(detector.is_cdn(&cf_ip));

        // Add to blacklist
        detector.add_to_blacklist(CdnProvider::Cloudflare);
        assert!(!detector.is_cdn(&cf_ip));

        // Remove from blacklist
        detector.remove_from_blacklist(CdnProvider::Cloudflare);
        assert!(detector.is_cdn(&cf_ip));
    }

    #[test]
    fn test_active_providers() {
        let detector = CdnDetector::new();
        let providers = detector.active_providers();

        // Should have all 6 providers
        assert_eq!(providers.len(), 6);
        assert!(providers.contains(&CdnProvider::Cloudflare));
        assert!(providers.contains(&CdnProvider::AwsCloudFront));
        assert!(providers.contains(&CdnProvider::AzureCdn));
        assert!(providers.contains(&CdnProvider::Akamai));
        assert!(providers.contains(&CdnProvider::Fastly));
        assert!(providers.contains(&CdnProvider::GoogleCloud));
    }

    #[test]
    fn test_active_providers_with_whitelist() {
        let detector =
            CdnDetector::with_whitelist(vec![CdnProvider::Cloudflare, CdnProvider::Fastly]);
        let providers = detector.active_providers();

        assert_eq!(providers.len(), 2);
        assert!(providers.contains(&CdnProvider::Cloudflare));
        assert!(providers.contains(&CdnProvider::Fastly));
    }

    #[test]
    fn test_active_providers_with_blacklist() {
        let detector =
            CdnDetector::with_blacklist(vec![CdnProvider::Cloudflare, CdnProvider::Fastly]);
        let providers = detector.active_providers();

        assert_eq!(providers.len(), 4);
        assert!(!providers.contains(&CdnProvider::Cloudflare));
        assert!(!providers.contains(&CdnProvider::Fastly));
        assert!(providers.contains(&CdnProvider::AwsCloudFront));
    }

    #[test]
    fn test_provider_name() {
        assert_eq!(CdnProvider::Cloudflare.name(), "Cloudflare");
        assert_eq!(CdnProvider::AwsCloudFront.name(), "AWS CloudFront");
        assert_eq!(CdnProvider::AzureCdn.name(), "Azure CDN");
        assert_eq!(CdnProvider::Akamai.name(), "Akamai");
        assert_eq!(CdnProvider::Fastly.name(), "Fastly");
        assert_eq!(CdnProvider::GoogleCloud.name(), "Google Cloud CDN");
    }

    #[test]
    fn test_ipv6_detection() {
        let detector = CdnDetector::new();

        // Test various IPv6 CDN ranges
        let ipv6_tests = vec![
            (
                "2606:4700:20::1".parse().unwrap(),
                Some(CdnProvider::Cloudflare),
            ),
            (
                "2600:9000::1".parse().unwrap(),
                Some(CdnProvider::AwsCloudFront),
            ),
            ("2a04:4e40::1".parse().unwrap(), Some(CdnProvider::Fastly)),
            ("2001:4860:4860::8888".parse().unwrap(), None), // Google DNS, not CDN
        ];

        for (ip, expected) in ipv6_tests {
            assert_eq!(
                detector.detect(&ip),
                expected,
                "IPv6 {} detection mismatch",
                ip
            );
        }
    }

    #[test]
    fn test_azure_cdn_detection() {
        let detector = CdnDetector::new();

        // Known Azure CDN IPs
        let azure_ips = vec![
            "20.21.1.1".parse().unwrap(),    // Azure range 20.21.0.0/16
            "147.243.1.1".parse().unwrap(),  // Azure range 147.243.0.0/16
            "2603:1030::1".parse().unwrap(), // Azure IPv6
        ];

        for ip in azure_ips {
            assert_eq!(
                detector.detect(&ip),
                Some(CdnProvider::AzureCdn),
                "IP {} should be detected as Azure CDN",
                ip
            );
            assert!(detector.is_cdn(&ip));
        }
    }

    #[test]
    fn test_akamai_detection() {
        let detector = CdnDetector::new();

        // Known Akamai IPs
        let akamai_ips = vec![
            "23.1.1.1".parse().unwrap(),     // Akamai range 23.0.0.0/12
            "104.64.1.1".parse().unwrap(),   // Akamai range 104.64.0.0/10
            "2600:1400::1".parse().unwrap(), // Akamai IPv6
        ];

        for ip in akamai_ips {
            assert_eq!(
                detector.detect(&ip),
                Some(CdnProvider::Akamai),
                "IP {} should be detected as Akamai",
                ip
            );
            assert!(detector.is_cdn(&ip));
        }
    }

    #[test]
    fn test_google_cloud_cdn_detection() {
        let detector = CdnDetector::new();

        // Known Google Cloud CDN IPs
        let google_ips = vec![
            "34.64.1.1".parse().unwrap(),    // Google Cloud range 34.64.0.0/10
            "35.192.1.1".parse().unwrap(),   // Google Cloud range 35.192.0.0/12
            "2600:1900::1".parse().unwrap(), // Google Cloud IPv6
        ];

        for ip in google_ips {
            assert_eq!(
                detector.detect(&ip),
                Some(CdnProvider::GoogleCloud),
                "IP {} should be detected as Google Cloud CDN",
                ip
            );
            assert!(detector.is_cdn(&ip));
        }
    }

    // Hash-based optimization tests
    #[test]
    fn test_ipv4_prefix_extraction() {
        // Test /24 prefix extraction (top 24 bits)
        let test_cases = vec![
            ("192.168.1.100", 0xC0A80100),  // 192.168.1.0
            ("10.0.0.1", 0x0A000000),       // 10.0.0.0
            ("172.16.255.254", 0xAC10FF00), // 172.16.255.0
            ("104.16.132.229", 0x68108400), // 104.16.132.0 (Cloudflare)
        ];

        for (ip_str, expected_prefix) in test_cases {
            let ip: Ipv4Addr = ip_str.parse().unwrap();
            let prefix = CdnDetector::ipv4_to_prefix24(ip);
            assert_eq!(
                prefix, expected_prefix,
                "IPv4 {} should extract to prefix 0x{:08X}",
                ip_str, expected_prefix
            );
        }
    }

    #[test]
    fn test_ipv6_prefix_extraction() {
        // Test /48 prefix extraction (top 48 bits positioned at bits 127-80)
        let test_cases = vec![
            (
                "2606:4700:20::1",
                0x26064700002000000000000000000000u128, // 2606:4700:20::/48
            ),
            (
                "2600:9000:1234:5678::1",
                0x26009000123400000000000000000000u128, // 2600:9000:1234::/48
            ),
            (
                "2a04:4e40:abcd:ef01::1",
                0x2a044e40abcd00000000000000000000u128, // 2a04:4e40:abcd::/48
            ),
        ];

        for (ip_str, expected_prefix) in test_cases {
            let ip: Ipv6Addr = ip_str.parse().unwrap();
            let prefix = CdnDetector::ipv6_to_prefix48(ip);
            assert_eq!(
                prefix, expected_prefix,
                "IPv6 {} should extract to prefix 0x{:032X}",
                ip_str, expected_prefix
            );
        }
    }

    #[test]
    fn test_ipv4_network_expansion_exact_24() {
        // Test /24 network - should produce single prefix
        let network: Ipv4Network = "192.168.1.0/24".parse().unwrap();
        let prefixes = CdnDetector::expand_ipv4_network_to_24_prefixes(network);

        assert_eq!(prefixes.len(), 1);
        assert_eq!(prefixes[0], 0xC0A80100); // 192.168.1.0
    }

    #[test]
    fn test_ipv4_network_expansion_larger_than_24() {
        // Test /16 network - should produce 256 /24 prefixes
        let network: Ipv4Network = "192.168.0.0/16".parse().unwrap();
        let prefixes = CdnDetector::expand_ipv4_network_to_24_prefixes(network);

        assert_eq!(prefixes.len(), 256);
        assert_eq!(prefixes[0], 0xC0A80000); // 192.168.0.0
        assert_eq!(prefixes[255], 0xC0A8FF00); // 192.168.255.0

        // Test /13 network (Cloudflare 104.16.0.0/13) - should produce 2048 /24 prefixes
        let network: Ipv4Network = "104.16.0.0/13".parse().unwrap();
        let prefixes = CdnDetector::expand_ipv4_network_to_24_prefixes(network);

        assert_eq!(prefixes.len(), 2048);
        assert_eq!(prefixes[0], 0x68100000); // 104.16.0.0
                                             // Last prefix should be 104.23.255.0
        assert!(prefixes[prefixes.len() - 1] >= 0x68170000);
    }

    #[test]
    fn test_ipv6_network_expansion_exact_48() {
        // Test /48 network - should produce single prefix
        let network: Ipv6Network = "2606:4700:20::/48".parse().unwrap();
        let prefixes = CdnDetector::expand_ipv6_network_to_48_prefixes(network);

        assert_eq!(prefixes.len(), 1);
        assert_eq!(prefixes[0], 0x26064700002000000000000000000000u128);
    }

    #[test]
    fn test_ipv6_network_expansion_larger_than_48() {
        // Test /32 network - should produce multiple /48 prefixes
        let network: Ipv6Network = "2606:4700::/32".parse().unwrap();
        let prefixes = CdnDetector::expand_ipv6_network_to_48_prefixes(network);

        // /32 to /48 = 2^16 = 65536 prefixes, but we cap at 65536
        assert!(!prefixes.is_empty());
        assert!(prefixes.len() <= 65536);

        // First prefix should be 2606:4700::/48
        assert_eq!(prefixes[0], 0x26064700000000000000000000000000u128);
    }

    #[test]
    fn test_hash_maps_populated_on_init() {
        let detector = CdnDetector::new();

        // Hash maps should be non-empty after initialization
        assert!(!detector.ipv4_prefix_map.is_empty());
        assert!(!detector.ipv6_prefix_map.is_empty());

        // Should contain entries for Cloudflare (known large provider)
        let cf_ip: IpAddr = "104.16.132.229".parse().unwrap();
        assert_eq!(detector.detect(&cf_ip), Some(CdnProvider::Cloudflare));
    }

    #[test]
    fn test_hash_maps_rebuilt_on_whitelist_change() {
        let mut detector = CdnDetector::new();

        // Initially, hash maps contain all providers
        let original_ipv4_size = detector.ipv4_prefix_map.len();
        let original_ipv6_size = detector.ipv6_prefix_map.len();

        // Set whitelist to only Cloudflare
        detector.set_whitelist(vec![CdnProvider::Cloudflare]);

        // Hash maps should be rebuilt with fewer entries
        let new_ipv4_size = detector.ipv4_prefix_map.len();
        let new_ipv6_size = detector.ipv6_prefix_map.len();

        assert!(new_ipv4_size < original_ipv4_size);
        assert!(new_ipv6_size < original_ipv6_size);

        // Cloudflare should still be detected
        let cf_ip: IpAddr = "104.16.132.229".parse().unwrap();
        assert_eq!(detector.detect(&cf_ip), Some(CdnProvider::Cloudflare));

        // AWS should NOT be detected
        let aws_ip: IpAddr = "13.32.1.1".parse().unwrap();
        assert_eq!(detector.detect(&aws_ip), None);
    }

    #[test]
    fn test_hash_maps_rebuilt_on_blacklist_change() {
        let mut detector = CdnDetector::new();

        // Initially, Cloudflare should be detected
        let cf_ip: IpAddr = "104.16.132.229".parse().unwrap();
        assert_eq!(detector.detect(&cf_ip), Some(CdnProvider::Cloudflare));

        // Add Cloudflare to blacklist
        detector.add_to_blacklist(CdnProvider::Cloudflare);

        // Hash maps should be rebuilt without Cloudflare
        let cf_prefix = CdnDetector::ipv4_to_prefix24("104.16.132.229".parse().unwrap());
        assert!(!detector.ipv4_prefix_map.contains_key(&cf_prefix));

        // Cloudflare should NOT be detected
        assert_eq!(detector.detect(&cf_ip), None);

        // Remove from blacklist
        detector.remove_from_blacklist(CdnProvider::Cloudflare);

        // Hash maps should be rebuilt with Cloudflare again
        assert_eq!(detector.detect(&cf_ip), Some(CdnProvider::Cloudflare));
    }

    #[test]
    fn test_hash_lookup_performance() {
        use std::time::Instant;

        let detector = CdnDetector::new();

        // Test 1000 lookups (should be very fast with hash maps)
        let test_ips: Vec<IpAddr> = vec![
            "104.16.132.229".parse().unwrap(), // Cloudflare
            "13.32.1.1".parse().unwrap(),      // AWS CloudFront
            "151.101.1.1".parse().unwrap(),    // Fastly
            "192.168.1.1".parse().unwrap(),    // Non-CDN
        ];

        let start = Instant::now();
        for _ in 0..1000 {
            for ip in &test_ips {
                let _ = detector.detect(ip);
            }
        }
        let elapsed = start.elapsed();

        // 4000 lookups should complete in < 10ms with O(1) hash lookups
        // (vs ~100-200ms with O(N*M) linear search)
        assert!(
            elapsed.as_millis() < 10,
            "4000 hash lookups took {:?} (expected < 10ms)",
            elapsed
        );
    }
}
