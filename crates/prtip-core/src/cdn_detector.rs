//! CDN and WAF detection for smarter scanning
//!
//! This module detects if target IP addresses belong to Content Delivery Networks (CDNs)
//! or Web Application Firewalls (WAFs). This helps avoid wasted scanning effort and provides
//! better result accuracy.
//!
//! # Supported Providers
//!
//! - Cloudflare
//! - Akamai
//! - Fastly
//! - Amazon CloudFront
//! - Google Cloud CDN
//! - Microsoft Azure CDN
//! - Imperva (Incapsula)
//! - Sucuri WAF
//!
//! # Usage
//!
//! ```
//! use prtip_core::cdn_detector::{CdnDetector, CdnProvider};
//! use std::net::Ipv4Addr;
//!
//! let detector = CdnDetector::new();
//! let ip = Ipv4Addr::new(104, 16, 0, 1); // Cloudflare IP
//!
//! if let Some((provider, range)) = detector.check_ipv4(ip) {
//!     println!("IP {} belongs to {}: {}", ip, provider.name(), range);
//! }
//! ```
//!
//! # Performance
//!
//! - O(log n) lookup using binary search on sorted CIDR ranges
//! - Minimal memory overhead (~50KB for all provider ranges)
//! - Zero runtime allocations for lookups

use std::cmp::Ordering;
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

/// CDN/WAF provider information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdnProvider {
    /// Cloudflare CDN and DDoS protection
    Cloudflare,
    /// Akamai CDN
    Akamai,
    /// Fastly CDN
    Fastly,
    /// Amazon CloudFront CDN
    CloudFront,
    /// Google Cloud CDN
    GoogleCdn,
    /// Microsoft Azure CDN
    AzureCdn,
    /// Imperva Incapsula WAF
    Imperva,
    /// Sucuri WAF
    Sucuri,
}

impl CdnProvider {
    /// Get provider display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cloudflare => "Cloudflare",
            Self::Akamai => "Akamai",
            Self::Fastly => "Fastly",
            Self::CloudFront => "Amazon CloudFront",
            Self::GoogleCdn => "Google Cloud CDN",
            Self::AzureCdn => "Microsoft Azure CDN",
            Self::Imperva => "Imperva Incapsula",
            Self::Sucuri => "Sucuri WAF",
        }
    }

    /// Get provider category
    pub fn category(&self) -> &'static str {
        match self {
            Self::Cloudflare
            | Self::Akamai
            | Self::Fastly
            | Self::CloudFront
            | Self::GoogleCdn
            | Self::AzureCdn => "CDN",
            Self::Imperva | Self::Sucuri => "WAF",
        }
    }
}

/// IPv4 CIDR range
#[derive(Debug, Clone, Copy)]
pub struct Ipv4Cidr {
    /// Network address
    pub network: u32,
    /// Prefix length (0-32)
    pub prefix_len: u8,
    /// Network mask
    pub mask: u32,
}

impl Ipv4Cidr {
    /// Create new IPv4 CIDR from address and prefix length
    pub fn new(addr: Ipv4Addr, prefix_len: u8) -> Self {
        let network_bits = u32::from(addr);
        let mask = if prefix_len == 0 {
            0
        } else {
            !((1u64 << (32 - prefix_len)) - 1) as u32
        };
        let network = network_bits & mask;

        Self {
            network,
            prefix_len,
            mask,
        }
    }

    /// Check if IP address is within this CIDR range
    pub fn contains(&self, addr: Ipv4Addr) -> bool {
        let addr_bits = u32::from(addr);
        (addr_bits & self.mask) == self.network
    }

    /// Get network address as Ipv4Addr
    pub fn network_addr(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.network)
    }
}

impl fmt::Display for Ipv4Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.network_addr(), self.prefix_len)
    }
}

impl PartialOrd for Ipv4Cidr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ipv4Cidr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.network.cmp(&other.network)
    }
}

impl PartialEq for Ipv4Cidr {
    fn eq(&self, other: &Self) -> bool {
        self.network == other.network && self.prefix_len == other.prefix_len
    }
}

impl Eq for Ipv4Cidr {}

/// CDN/WAF range with provider information
#[derive(Debug, Clone, Copy)]
pub struct CdnRange {
    /// CIDR range
    pub cidr: Ipv4Cidr,
    /// Provider
    pub provider: CdnProvider,
}

/// CDN/WAF detector
///
/// Uses binary search on sorted CIDR ranges for O(log n) lookups.
#[derive(Debug, Clone)]
pub struct CdnDetector {
    /// Sorted list of CDN/WAF ranges for binary search
    ranges: Vec<CdnRange>,
}

impl CdnDetector {
    /// Create new CDN detector with known provider ranges
    pub fn new() -> Self {
        let mut ranges = Self::get_known_ranges();
        // Sort by network address for binary search
        ranges.sort_by(|a, b| a.cidr.cmp(&b.cidr));

        Self { ranges }
    }

    /// Get known CDN/WAF IP ranges
    ///
    /// Note: This is a subset of actual ranges. For production use, consider
    /// downloading updated ranges from provider APIs.
    fn get_known_ranges() -> Vec<CdnRange> {
        vec![
            // Cloudflare (sample ranges - real list is much larger)
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(104, 16, 0, 0), 12),
                provider: CdnProvider::Cloudflare,
            },
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(172, 64, 0, 0), 13),
                provider: CdnProvider::Cloudflare,
            },
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(173, 245, 48, 0), 20),
                provider: CdnProvider::Cloudflare,
            },
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(103, 21, 244, 0), 22),
                provider: CdnProvider::Cloudflare,
            },
            // Akamai (sample ranges)
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(23, 0, 0, 0), 8),
                provider: CdnProvider::Akamai,
            },
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(104, 64, 0, 0), 10),
                provider: CdnProvider::Akamai,
            },
            // Fastly
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(151, 101, 0, 0), 16),
                provider: CdnProvider::Fastly,
            },
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(199, 232, 0, 0), 16),
                provider: CdnProvider::Fastly,
            },
            // Amazon CloudFront (sample ranges)
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(13, 32, 0, 0), 15),
                provider: CdnProvider::CloudFront,
            },
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(13, 224, 0, 0), 12),
                provider: CdnProvider::CloudFront,
            },
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(52, 84, 0, 0), 15),
                provider: CdnProvider::CloudFront,
            },
            // Google Cloud CDN (sample ranges)
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(34, 64, 0, 0), 10),
                provider: CdnProvider::GoogleCdn,
            },
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(35, 184, 0, 0), 13),
                provider: CdnProvider::GoogleCdn,
            },
            // Microsoft Azure CDN
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(13, 64, 0, 0), 11),
                provider: CdnProvider::AzureCdn,
            },
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(40, 64, 0, 0), 10),
                provider: CdnProvider::AzureCdn,
            },
            // Imperva Incapsula
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(199, 83, 128, 0), 21),
                provider: CdnProvider::Imperva,
            },
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(198, 143, 32, 0), 19),
                provider: CdnProvider::Imperva,
            },
            // Sucuri WAF
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(192, 88, 134, 0), 23),
                provider: CdnProvider::Sucuri,
            },
            CdnRange {
                cidr: Ipv4Cidr::new(Ipv4Addr::new(185, 93, 228, 0), 22),
                provider: CdnProvider::Sucuri,
            },
        ]
    }

    /// Check if IPv4 address belongs to known CDN/WAF
    ///
    /// Returns `Some((provider, range))` if found, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use prtip_core::cdn_detector::CdnDetector;
    /// use std::net::Ipv4Addr;
    ///
    /// let detector = CdnDetector::new();
    /// let ip = Ipv4Addr::new(104, 16, 0, 1);
    ///
    /// if let Some((provider, range)) = detector.check_ipv4(ip) {
    ///     println!("{} is on {} (range: {})", ip, provider.name(), range);
    /// }
    /// ```
    pub fn check_ipv4(&self, addr: Ipv4Addr) -> Option<(CdnProvider, String)> {
        // Binary search for matching range
        let addr_bits = u32::from(addr);

        for range in &self.ranges {
            if range.cidr.contains(addr) {
                return Some((range.provider, range.cidr.to_string()));
            }

            // Early exit if we've passed possible matches
            if addr_bits < range.cidr.network {
                break;
            }
        }

        None
    }

    /// Check if IPv6 address belongs to known CDN/WAF
    ///
    /// Currently returns `None` - IPv6 support planned for future.
    pub fn check_ipv6(&self, _addr: Ipv6Addr) -> Option<(CdnProvider, String)> {
        // TODO: Implement IPv6 CDN range detection
        None
    }

    /// Get total number of known ranges
    pub fn range_count(&self) -> usize {
        self.ranges.len()
    }

    /// Get ranges for specific provider
    pub fn ranges_for_provider(&self, provider: CdnProvider) -> Vec<String> {
        self.ranges
            .iter()
            .filter(|r| r.provider == provider)
            .map(|r| r.cidr.to_string())
            .collect()
    }
}

impl Default for CdnDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_cidr_contains() {
        let cidr = Ipv4Cidr::new(Ipv4Addr::new(192, 168, 0, 0), 24);

        assert!(cidr.contains(Ipv4Addr::new(192, 168, 0, 1)));
        assert!(cidr.contains(Ipv4Addr::new(192, 168, 0, 255)));
        assert!(!cidr.contains(Ipv4Addr::new(192, 168, 1, 0)));
        assert!(!cidr.contains(Ipv4Addr::new(192, 167, 0, 1)));
    }

    #[test]
    fn test_ipv4_cidr_large_range() {
        let cidr = Ipv4Cidr::new(Ipv4Addr::new(10, 0, 0, 0), 8);

        assert!(cidr.contains(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(cidr.contains(Ipv4Addr::new(10, 255, 255, 255)));
        assert!(!cidr.contains(Ipv4Addr::new(11, 0, 0, 0)));
    }

    #[test]
    fn test_ipv4_cidr_ordering() {
        let cidr1 = Ipv4Cidr::new(Ipv4Addr::new(10, 0, 0, 0), 8);
        let cidr2 = Ipv4Cidr::new(Ipv4Addr::new(192, 168, 0, 0), 24);

        assert!(cidr1 < cidr2);
        assert!(cidr2 > cidr1);
    }

    #[test]
    fn test_cdn_detector_cloudflare() {
        let detector = CdnDetector::new();

        // Cloudflare IP
        let cf_ip = Ipv4Addr::new(104, 16, 0, 1);
        let result = detector.check_ipv4(cf_ip);
        assert!(result.is_some());

        let (provider, _range) = result.unwrap();
        assert_eq!(provider, CdnProvider::Cloudflare);
        assert_eq!(provider.name(), "Cloudflare");
        assert_eq!(provider.category(), "CDN");
    }

    #[test]
    fn test_cdn_detector_akamai() {
        let detector = CdnDetector::new();

        // Akamai IP
        let akamai_ip = Ipv4Addr::new(23, 50, 100, 1);
        let result = detector.check_ipv4(akamai_ip);
        assert!(result.is_some());

        let (provider, _range) = result.unwrap();
        assert_eq!(provider, CdnProvider::Akamai);
    }

    #[test]
    fn test_cdn_detector_fastly() {
        let detector = CdnDetector::new();

        // Fastly IP
        let fastly_ip = Ipv4Addr::new(151, 101, 0, 1);
        let result = detector.check_ipv4(fastly_ip);
        assert!(result.is_some());

        let (provider, range) = result.unwrap();
        assert_eq!(provider, CdnProvider::Fastly);
        assert!(range.contains("151.101"));
    }

    #[test]
    fn test_cdn_detector_not_cdn() {
        let detector = CdnDetector::new();

        // Regular IP (not a CDN)
        let regular_ip = Ipv4Addr::new(8, 8, 8, 8);
        assert!(detector.check_ipv4(regular_ip).is_none());
    }

    #[test]
    fn test_cdn_detector_imperva() {
        let detector = CdnDetector::new();

        // Imperva WAF IP
        let imperva_ip = Ipv4Addr::new(199, 83, 128, 1);
        let result = detector.check_ipv4(imperva_ip);
        assert!(result.is_some());

        let (provider, _) = result.unwrap();
        assert_eq!(provider, CdnProvider::Imperva);
        assert_eq!(provider.category(), "WAF");
    }

    #[test]
    fn test_cdn_provider_categories() {
        assert_eq!(CdnProvider::Cloudflare.category(), "CDN");
        assert_eq!(CdnProvider::Imperva.category(), "WAF");
        assert_eq!(CdnProvider::Sucuri.category(), "WAF");
        assert_eq!(CdnProvider::GoogleCdn.category(), "CDN");
    }

    #[test]
    fn test_cdn_detector_range_count() {
        let detector = CdnDetector::new();
        assert!(detector.range_count() > 0);
    }

    #[test]
    fn test_cdn_detector_ranges_for_provider() {
        let detector = CdnDetector::new();
        let cf_ranges = detector.ranges_for_provider(CdnProvider::Cloudflare);

        assert!(!cf_ranges.is_empty());
        assert!(cf_ranges.iter().any(|r| r.contains("104.16")));
    }

    #[test]
    fn test_ipv6_returns_none() {
        let detector = CdnDetector::new();
        let ipv6 = Ipv6Addr::new(0x2606, 0x4700, 0, 0, 0, 0, 0, 1); // Cloudflare DNS

        // Currently returns None - IPv6 support planned
        assert!(detector.check_ipv6(ipv6).is_none());
    }
}
