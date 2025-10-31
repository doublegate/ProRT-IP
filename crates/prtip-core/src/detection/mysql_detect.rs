//! MySQL handshake parsing for version detection
//!
//! This module parses MySQL protocol handshake packets to extract server
//! version and capability information.
//!
//! # MySQL Handshake Protocol
//!
//! The MySQL initial handshake packet contains:
//! - Protocol version (1 byte, always 10 for MySQL 5.x+)
//! - Server version string (null-terminated)
//! - Connection ID (4 bytes)
//! - Auth plugin data + capabilities
//!
//! # Examples
//!
//! ```
//! use prtip_core::detection::mysql_detect::MysqlDetect;
//! use prtip_core::detection::ProtocolDetector;
//!
//! // MySQL 8.0.27 handshake packet
//! let handshake = b"\x4a\x00\x00\x00\x0a8.0.27-0ubuntu0.20.04.1\x00";
//! // ^length ^seq ^proto ^version string...
//!
//! let detector = MysqlDetect::new();
//! if let Ok(Some(info)) = detector.detect(handshake) {
//!     assert_eq!(info.service, "mysql");
//!     assert_eq!(info.product, Some("MySQL".to_string()));
//!     assert!(info.version.as_ref().unwrap().starts_with("8.0.27"));
//! }
//! ```

use super::{ProtocolDetector, ServiceInfo};
use crate::Error;
use std::str;

/// MySQL handshake detector
#[derive(Debug, Clone)]
pub struct MysqlDetect {
    priority: u8,
}

impl MysqlDetect {
    /// Create new MySQL detector
    pub fn new() -> Self {
        Self { priority: 4 }
    }

    /// Parse MySQL handshake packet
    ///
    /// MySQL handshake structure:
    /// - Bytes 0-3: Packet length (little-endian, 3 bytes) + sequence ID (1 byte)
    /// - Byte 4: Protocol version (should be 10)
    /// - Bytes 5+: Server version string (null-terminated)
    ///
    /// Returns (protocol_version, server_version_string)
    fn parse_handshake(response: &[u8]) -> Option<(u8, String)> {
        // Minimum handshake size: 4 (header) + 1 (protocol) + 1 (version) + 1 (null)
        if response.len() < 7 {
            return None;
        }

        // Extract protocol version (byte 4)
        let protocol_version = response[4];

        // Protocol version should be 10 for MySQL 5.x+
        if protocol_version != 10 {
            return None;
        }

        // Extract server version string (starts at byte 5, null-terminated)
        let version_start = 5;
        let mut version_end = version_start;

        // Find null terminator
        while version_end < response.len() && response[version_end] != 0 {
            version_end += 1;
        }

        if version_end >= response.len() {
            return None; // No null terminator found
        }

        // Parse version string
        let version_bytes = &response[version_start..version_end];
        let version_str = str::from_utf8(version_bytes).ok()?;

        Some((protocol_version, version_str.to_string()))
    }

    /// Extract version details from MySQL version string
    ///
    /// Examples:
    /// - "8.0.27-0ubuntu0.20.04.1" → ("8.0.27", "Ubuntu 20.04")
    /// - "5.7.36-log" → ("5.7.36", "")
    /// - "10.5.12-MariaDB" → ("10.5.12", "MariaDB")
    fn parse_version_string(version: &str) -> (String, Option<String>) {
        // Split by hyphen to separate version from suffix
        let parts: Vec<&str> = version.splitn(2, '-').collect();
        let version_number = parts[0].to_string();

        // Extract OS/distribution hints
        let os_hint = if parts.len() > 1 {
            let suffix = parts[1];
            if suffix.contains("ubuntu") {
                // Extract Ubuntu version (e.g., "0ubuntu0.20.04.1" → "Ubuntu 20.04")
                if let Some(pos) = suffix.find("ubuntu") {
                    let after = &suffix[pos..];
                    if let Some(version_match) = Self::extract_ubuntu_version(after) {
                        return (version_number, Some(version_match));
                    }
                }
                Some("Ubuntu".to_string())
            } else if suffix.to_lowercase().contains("mariadb") {
                Some("MariaDB".to_string())
            } else if suffix.contains("debian") {
                Some("Debian".to_string())
            } else if suffix.contains("el7") {
                Some("Red Hat Enterprise Linux 7".to_string())
            } else if suffix.contains("el8") {
                Some("Red Hat Enterprise Linux 8".to_string())
            } else {
                None
            }
        } else {
            None
        };

        (version_number, os_hint)
    }

    /// Extract Ubuntu version from version string
    ///
    /// Example: "ubuntu0.20.04.1" → "Ubuntu 20.04"
    fn extract_ubuntu_version(s: &str) -> Option<String> {
        // Look for pattern like "ubuntu0.20.04" or "ubuntu20.04"
        // Skip "ubuntu" prefix and any leading "0."
        if let Some(pos) = s.find(char::is_numeric) {
            let rest = &s[pos..];
            // Try to extract version number (e.g., "0.20.04.1")
            let version_part: String = rest
                .chars()
                .take_while(|c| c.is_numeric() || *c == '.')
                .collect();

            // Extract major.minor version, skipping leading "0." if present
            let parts: Vec<&str> = version_part.split('.').collect();

            // Handle "0.20.04" pattern (skip leading 0)
            if parts.len() >= 3 && parts[0] == "0" {
                return Some(format!("Ubuntu {}.{}", parts[1], parts[2]));
            }

            // Handle "20.04" pattern (direct version)
            if parts.len() >= 2 {
                return Some(format!("Ubuntu {}.{}", parts[0], parts[1]));
            }
        }
        None
    }

    /// Calculate confidence based on information extracted
    fn calculate_confidence(has_version: bool, has_os: bool) -> f32 {
        let mut score: f32 = 0.7; // Base score for MySQL handshake

        if has_version {
            score += 0.15; // Version extracted
        }
        if has_os {
            score += 0.1; // OS/distribution hint found
        }

        score.min(1.0)
    }
}

impl Default for MysqlDetect {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolDetector for MysqlDetect {
    fn detect(&self, response: &[u8]) -> Result<Option<ServiceInfo>, Error> {
        // Parse handshake
        let (protocol_version, server_version) = match Self::parse_handshake(response) {
            Some(parsed) => parsed,
            None => return Ok(None), // Not a MySQL handshake
        };

        // Extract version details
        let (version_number, os_hint) = Self::parse_version_string(&server_version);

        // Determine if this is MySQL or MariaDB
        let product = if server_version.contains("MariaDB") {
            "MariaDB"
        } else {
            "MySQL"
        };

        // Build ServiceInfo
        let mut service_info = ServiceInfo::new("mysql");
        service_info.product = Some(product.to_string());
        service_info.version = Some(version_number);

        // Build info string
        let mut info_parts = vec![format!("protocol {}", protocol_version)];
        if let Some(os) = os_hint.clone() {
            info_parts.push(os.clone());
            service_info.os_type = Some(os);
        }
        service_info.info = Some(info_parts.join(" + "));

        // Calculate confidence
        service_info.confidence =
            Self::calculate_confidence(!server_version.is_empty(), os_hint.is_some());

        Ok(Some(service_info))
    }

    fn confidence(&self) -> f32 {
        0.85 // MySQL handshakes are well-structured
    }

    fn priority(&self) -> u8 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mysql8_ubuntu_detection() {
        // MySQL 8.0.27 handshake on Ubuntu 20.04
        let mut handshake = vec![0u8; 100];
        handshake[0] = 0x4a; // Packet length (low byte)
        handshake[4] = 10; // Protocol version
        let version = b"8.0.27-0ubuntu0.20.04.1";
        handshake[5..5 + version.len()].copy_from_slice(version);
        handshake[5 + version.len()] = 0; // Null terminator

        let detector = MysqlDetect::new();
        let result = detector.detect(&handshake).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "mysql");
        assert_eq!(info.product, Some("MySQL".to_string()));
        assert_eq!(info.version, Some("8.0.27".to_string()));
        assert_eq!(info.os_type, Some("Ubuntu 20.04".to_string()));
        assert!(info.confidence >= 0.8);
    }

    #[test]
    fn test_mysql57_detection() {
        // MySQL 5.7.36
        let mut handshake = vec![0u8; 50];
        handshake[4] = 10; // Protocol version
        let version = b"5.7.36-log";
        handshake[5..5 + version.len()].copy_from_slice(version);
        handshake[5 + version.len()] = 0;

        let detector = MysqlDetect::new();
        let result = detector.detect(&handshake).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "mysql");
        assert_eq!(info.product, Some("MySQL".to_string()));
        assert_eq!(info.version, Some("5.7.36".to_string()));
        assert!(info.confidence >= 0.7);
    }

    #[test]
    fn test_mariadb_detection() {
        // MariaDB 10.5.12
        let mut handshake = vec![0u8; 60];
        handshake[4] = 10; // Protocol version
        let version = b"10.5.12-MariaDB";
        handshake[5..5 + version.len()].copy_from_slice(version);
        handshake[5 + version.len()] = 0;

        let detector = MysqlDetect::new();
        let result = detector.detect(&handshake).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "mysql");
        assert_eq!(info.product, Some("MariaDB".to_string()));
        assert_eq!(info.version, Some("10.5.12".to_string()));
        assert_eq!(info.os_type, Some("MariaDB".to_string()));
        assert!(info.confidence >= 0.8);
    }

    #[test]
    fn test_non_mysql_response() {
        let response = b"HTTP/1.1 200 OK\r\n";

        let detector = MysqlDetect::new();
        let result = detector.detect(response).unwrap();

        // Should return None for non-MySQL response
        assert!(result.is_none());
    }
}
