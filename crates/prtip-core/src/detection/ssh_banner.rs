//! SSH banner parsing for version and OS detection
//!
//! This module parses SSH protocol banners to extract server version information
//! and OS hints from the banner comment field.
//!
//! # Banner Format
//!
//! SSH banners follow RFC 4253 format:
//! ```text
//! SSH-protoversion-softwareversion SP comments CR LF
//! ```
//!
//! Examples:
//! - `SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.3`
//! - `SSH-2.0-Dropbear_2020.81`
//! - `SSH-1.99-Cisco-1.25`
//!
//! # Examples
//!
//! ```
//! use prtip_core::detection::ssh_banner::SshBanner;
//! use prtip_core::detection::ProtocolDetector;
//!
//! let banner = b"SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.3\r\n";
//! let detector = SshBanner::new();
//!
//! if let Ok(Some(info)) = detector.detect(banner) {
//!     assert_eq!(info.service, "ssh");
//!     assert_eq!(info.product, Some("OpenSSH".to_string()));
//!     assert_eq!(info.version, Some("8.2p1".to_string()));
//!     assert!(info.info.as_ref().unwrap().contains("Ubuntu"));
//! }
//! ```

use super::{ProtocolDetector, ServiceInfo};
use crate::Error;
use std::str;

/// SSH banner detector
#[derive(Debug, Clone)]
pub struct SshBanner {
    priority: u8,
}

impl SshBanner {
    /// Create new SSH banner detector
    pub fn new() -> Self {
        Self { priority: 2 }
    }

    /// Parse SSH banner into components
    ///
    /// Banner format: SSH-protoversion-softwareversion \[comments\]
    ///
    /// Returns (protocol_version, software_name, software_version, os_hint)
    #[allow(clippy::type_complexity)]
    fn parse_banner(
        banner: &str,
    ) -> Option<(String, Option<String>, Option<String>, Option<String>)> {
        // Must start with "SSH-"
        if !banner.starts_with("SSH-") {
            return None;
        }

        // Split into components
        let banner = banner.trim_end_matches(['\r', '\n']);
        let parts: Vec<&str> = banner.splitn(3, '-').collect();

        if parts.len() < 3 {
            return None;
        }

        let protocol_version = parts[1].to_string(); // "1.99" or "2.0"
        let software_part = parts[2]; // "OpenSSH_8.2p1 Ubuntu-4ubuntu0.3"

        // Split software from comment
        let software_and_comment: Vec<&str> = software_part.splitn(2, ' ').collect();
        let software = software_and_comment[0];
        let comment = software_and_comment.get(1).copied();

        // Parse software name and version
        let (software_name, software_version) = Self::parse_software(software);

        // Extract OS hints from comment
        let os_hint = comment.and_then(Self::extract_os_hint);

        Some((protocol_version, software_name, software_version, os_hint))
    }

    /// Parse software string into name and version
    ///
    /// Examples:
    /// - "OpenSSH_8.2p1" → ("OpenSSH", "8.2p1")
    /// - "Dropbear_2020.81" → ("Dropbear", "2020.81")
    /// - "libssh-0.9.0" → ("libssh", "0.9.0")
    fn parse_software(software: &str) -> (Option<String>, Option<String>) {
        // Try underscore separator first (OpenSSH_8.2p1)
        if let Some(pos) = software.find('_') {
            return (
                Some(software[..pos].to_string()),
                Some(software[pos + 1..].to_string()),
            );
        }

        // Try hyphen separator (libssh-0.9.0)
        if let Some(pos) = software.rfind('-') {
            // Make sure it's not at the start
            if pos > 0 {
                return (
                    Some(software[..pos].to_string()),
                    Some(software[pos + 1..].to_string()),
                );
            }
        }

        // No version separator found
        (Some(software.to_string()), None)
    }

    /// Extract OS hints from comment field
    ///
    /// Patterns:
    /// - "Ubuntu-4ubuntu0.3" → "Ubuntu 20.04 LTS"
    /// - "Debian-10+deb10u2" → "Debian 10"
    /// - "FreeBSD" → "FreeBSD"
    /// - "el6" → "Red Hat Enterprise Linux 6"
    fn extract_os_hint(comment: &str) -> Option<String> {
        let comment_lower = comment.to_lowercase();

        // Ubuntu detection
        if comment_lower.contains("ubuntu") {
            return Some(Self::map_ubuntu_version(comment));
        }

        // Debian detection
        if comment_lower.contains("debian") {
            return Some(Self::map_debian_version(comment));
        }

        // FreeBSD detection
        if comment_lower.contains("freebsd") {
            return Some("FreeBSD".to_string());
        }

        // Red Hat / CentOS detection
        if comment_lower.contains("el6") {
            return Some("Red Hat Enterprise Linux 6".to_string());
        }
        if comment_lower.contains("el7") {
            return Some("Red Hat Enterprise Linux 7".to_string());
        }
        if comment_lower.contains("el8") {
            return Some("Red Hat Enterprise Linux 8".to_string());
        }

        // Generic return comment as-is
        Some(comment.to_string())
    }

    /// Map Ubuntu package version to Ubuntu release
    fn map_ubuntu_version(comment: &str) -> String {
        // Extract Ubuntu package version (e.g., "4ubuntu0.3" from "Ubuntu-4ubuntu0.3")
        // The digit BEFORE "ubuntu" indicates the Ubuntu version
        if let Some(pos) = comment.find("ubuntu") {
            // Look back one character for the version number
            if pos > 0 {
                let before = &comment[..pos];
                // Get the last character (the version digit)
                if let Some(last_char) = before.chars().last() {
                    if last_char.is_ascii_digit() {
                        match last_char {
                            '0' => return "Ubuntu 14.04 LTS (Trusty)".to_string(),
                            '1' | '2' => return "Ubuntu 16.04 LTS (Xenial)".to_string(),
                            '3' => return "Ubuntu 18.04 LTS (Bionic)".to_string(),
                            '4' => return "Ubuntu 20.04 LTS (Focal)".to_string(),
                            '5' => return "Ubuntu 22.04 LTS (Jammy)".to_string(),
                            '6' => return "Ubuntu 24.04 LTS (Noble)".to_string(),
                            _ => {}
                        }
                    }
                }
            }
        }
        "Ubuntu".to_string()
    }

    /// Map Debian package version to Debian release
    fn map_debian_version(comment: &str) -> String {
        if comment.contains("deb10") {
            return "Debian 10 (Buster)".to_string();
        } else if comment.contains("deb11") {
            return "Debian 11 (Bullseye)".to_string();
        } else if comment.contains("deb12") {
            return "Debian 12 (Bookworm)".to_string();
        } else if comment.contains("deb9") {
            return "Debian 9 (Stretch)".to_string();
        }
        "Debian".to_string()
    }

    /// Calculate confidence based on information richness
    fn calculate_confidence(has_version: bool, has_os: bool, has_protocol: bool) -> f32 {
        let mut score: f32 = 0.6; // Base score for SSH banner detection

        if has_protocol {
            score += 0.1; // Protocol version present
        }
        if has_version {
            score += 0.2; // Software version extracted
        }
        if has_os {
            score += 0.1; // OS hint found
        }

        score.min(1.0)
    }
}

impl Default for SshBanner {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolDetector for SshBanner {
    fn detect(&self, response: &[u8]) -> Result<Option<ServiceInfo>, Error> {
        // Convert to UTF-8
        let response_str = str::from_utf8(response)
            .map_err(|_| Error::Parse("SSH banner contains invalid UTF-8".to_string()))?;

        // Parse banner
        let (protocol_version, software_name, software_version, os_hint) =
            match Self::parse_banner(response_str) {
                Some(components) => components,
                None => return Ok(None), // Not an SSH banner
            };

        // Build ServiceInfo
        let mut service_info = ServiceInfo::new("ssh");

        if let Some(name) = software_name {
            service_info.product = Some(name);
        }

        if let Some(version) = software_version.clone() {
            service_info.version = Some(version);
        }

        // Combine protocol version and OS hint into info field
        let mut info_parts = Vec::new();
        info_parts.push(format!("protocol {}", protocol_version));
        if let Some(os) = os_hint.clone() {
            info_parts.push(os.clone());
            service_info.os_type = Some(os);
        }

        service_info.info = Some(info_parts.join(" + "));

        // Calculate confidence
        service_info.confidence = Self::calculate_confidence(
            software_version.is_some(),
            os_hint.is_some(),
            !protocol_version.is_empty(),
        );

        Ok(Some(service_info))
    }

    fn confidence(&self) -> f32 {
        0.85 // SSH banners are reliable
    }

    fn priority(&self) -> u8 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openssh_ubuntu_detection() {
        let banner = b"SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.3\r\n";

        let detector = SshBanner::new();
        let result = detector.detect(banner).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "ssh");
        assert_eq!(info.product, Some("OpenSSH".to_string()));
        assert_eq!(info.version, Some("8.2p1".to_string()));
        assert_eq!(info.os_type, Some("Ubuntu 20.04 LTS (Focal)".to_string()));
        assert!(info.info.as_ref().unwrap().contains("protocol 2.0"));
        assert!(info.confidence > 0.8);
    }

    #[test]
    fn test_openssh_debian_detection() {
        let banner = b"SSH-2.0-OpenSSH_7.4p1 Debian-10+deb9u7\r\n";

        let detector = SshBanner::new();
        let result = detector.detect(banner).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "ssh");
        assert_eq!(info.product, Some("OpenSSH".to_string()));
        assert_eq!(info.version, Some("7.4p1".to_string()));
        assert_eq!(info.os_type, Some("Debian 9 (Stretch)".to_string()));
        assert!(info.confidence >= 0.8);
    }

    #[test]
    fn test_dropbear_detection() {
        let banner = b"SSH-2.0-Dropbear_2020.81\r\n";

        let detector = SshBanner::new();
        let result = detector.detect(banner).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "ssh");
        assert_eq!(info.product, Some("Dropbear".to_string()));
        assert_eq!(info.version, Some("2020.81".to_string()));
        assert!(info.confidence >= 0.7);
    }

    #[test]
    fn test_non_ssh_banner() {
        let banner = b"HTTP/1.1 200 OK\r\n";

        let detector = SshBanner::new();
        let result = detector.detect(banner).unwrap();

        // Should return None for non-SSH response
        assert!(result.is_none());
    }
}
