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
use std::net::IpAddr;

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
}

impl Default for CdnDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CdnDetector {
    /// Create a new CDN detector with default provider ranges
    pub fn new() -> Self {
        Self {
            ranges: Self::default_ranges(),
            whitelist: None,
            blacklist: Vec::new(),
        }
    }

    /// Create detector with specific whitelisted providers
    ///
    /// Only the specified providers will be considered CDNs.
    pub fn with_whitelist(providers: Vec<CdnProvider>) -> Self {
        Self {
            ranges: Self::default_ranges(),
            whitelist: Some(providers),
            blacklist: Vec::new(),
        }
    }

    /// Create detector with specific blacklisted providers
    ///
    /// The specified providers will never be considered CDNs.
    pub fn with_blacklist(providers: Vec<CdnProvider>) -> Self {
        Self {
            ranges: Self::default_ranges(),
            whitelist: None,
            blacklist: providers,
        }
    }

    /// Set whitelist (replaces existing)
    pub fn set_whitelist(&mut self, providers: Vec<CdnProvider>) {
        self.whitelist = Some(providers);
    }

    /// Clear whitelist
    pub fn clear_whitelist(&mut self) {
        self.whitelist = None;
    }

    /// Add provider to blacklist
    pub fn add_to_blacklist(&mut self, provider: CdnProvider) {
        if !self.blacklist.contains(&provider) {
            self.blacklist.push(provider);
        }
    }

    /// Remove provider from blacklist
    pub fn remove_from_blacklist(&mut self, provider: CdnProvider) {
        self.blacklist.retain(|p| *p != provider);
    }

    /// Detect if an IP belongs to a CDN provider
    ///
    /// Returns `Some(provider)` if the IP belongs to a CDN, respecting
    /// whitelist/blacklist configuration.
    pub fn detect(&self, ip: &IpAddr) -> Option<CdnProvider> {
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
}
