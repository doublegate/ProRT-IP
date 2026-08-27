//! PostgreSQL startup message parsing for version detection
//!
//! This module parses PostgreSQL protocol startup responses to extract
//! server_version from ParameterStatus messages.
//!
//! # PostgreSQL Message Protocol
//!
//! PostgreSQL startup responses contain a sequence of messages:
//! - 'R' (Authentication): Authentication request
//! - 'S' (ParameterStatus): server_version, client_encoding, etc.
//! - 'K' (BackendKeyData): Process ID and secret key
//! - 'Z' (ReadyForQuery): Server ready
//!
//! # Examples
//!
//! ```
//! use prtip_core::detection::postgresql_detect::PostgresqlDetect;
//! use prtip_core::detection::ProtocolDetector;
//!
//! // PostgreSQL startup response with ParameterStatus
//! // Format: 'S' + length + "server_version\0" + "14.2\0"
//!
//! let detector = PostgresqlDetect::new();
//! // ... response parsing ...
//! ```

use super::{ProtocolDetector, ServiceInfo};
use crate::Error;
use std::str;

/// PostgreSQL startup detector
#[derive(Debug, Clone)]
pub struct PostgresqlDetect {
    priority: u8,
}

impl PostgresqlDetect {
    /// Create new PostgreSQL detector
    pub fn new() -> Self {
        Self { priority: 5 }
    }

    /// Check if response contains PostgreSQL startup messages
    ///
    /// PostgreSQL messages start with a type byte:
    /// - 'R' (0x52): Authentication
    /// - 'S' (0x53): ParameterStatus
    /// - 'K' (0x4B): BackendKeyData
    /// - 'Z' (0x5A): ReadyForQuery
    /// - 'E' (0x45): ErrorResponse
    fn is_postgresql_response(response: &[u8]) -> bool {
        if response.is_empty() {
            return false;
        }

        // Check for typical PostgreSQL message types
        matches!(response[0], b'R' | b'S' | b'K' | b'Z' | b'E')
    }

    /// Parse ParameterStatus message to extract server_version
    ///
    /// ParameterStatus format:
    /// - Byte 0: 'S' (0x53)
    /// - Bytes 1-4: Message length (int32, big-endian)
    /// - Bytes 5+: Name (null-terminated) + Value (null-terminated)
    ///
    /// Returns server version if "server_version" parameter found
    fn extract_server_version(response: &[u8]) -> Option<String> {
        let mut pos = 0;

        // Scan through response looking for ParameterStatus messages
        while pos + 5 < response.len() {
            let msg_type = response[pos];

            // Check if this is a ParameterStatus message ('S')
            if msg_type == b'S' {
                // Read message length (4 bytes, big-endian, includes length field itself)
                let length = u32::from_be_bytes([
                    response[pos + 1],
                    response[pos + 2],
                    response[pos + 3],
                    response[pos + 4],
                ]) as usize;

                // Message content starts at pos + 5
                let content_start = pos + 5;
                let content_end = pos + 1 + length;

                if content_end > response.len() {
                    break; // Incomplete message
                }

                let content = &response[content_start..content_end];

                // Parse parameter name (null-terminated)
                if let Some(null_pos) = content.iter().position(|&b| b == 0) {
                    if let Ok(param_name) = str::from_utf8(&content[..null_pos]) {
                        if param_name == "server_version" {
                            // Extract value (after first null, before second null)
                            let value_start = null_pos + 1;
                            if let Some(value_end) =
                                content[value_start..].iter().position(|&b| b == 0)
                            {
                                if let Ok(version) =
                                    str::from_utf8(&content[value_start..value_start + value_end])
                                {
                                    return Some(version.to_string());
                                }
                            }
                        }
                    }
                }

                // Move to next message
                pos = content_end;
            } else {
                // Skip non-ParameterStatus messages
                // Try to read length and skip
                if pos + 5 < response.len() {
                    let length = u32::from_be_bytes([
                        response[pos + 1],
                        response[pos + 2],
                        response[pos + 3],
                        response[pos + 4],
                    ]) as usize;
                    pos += 1 + length;
                } else {
                    break;
                }
            }
        }

        None
    }

    /// Parse version string to extract version number and OS hints
    ///
    /// Examples:
    /// - "14.2 (Ubuntu 14.2-1ubuntu1)" → ("14.2", "Ubuntu")
    /// - "13.7 (Debian 13.7-1.pgdg110+1)" → ("13.7", "Debian")
    /// - "12.9" → ("12.9", None)
    fn parse_version_string(version: &str) -> (String, Option<String>) {
        // Split by space or opening parenthesis
        let parts: Vec<&str> = version.split(['(', ' ']).collect();
        let version_number = parts[0].trim().to_string();

        // Extract OS hints from parentheses
        let os_hint = if version.contains("Ubuntu") {
            Some("Ubuntu".to_string())
        } else if version.contains("Debian") {
            Some("Debian".to_string())
        } else if version.contains("Red Hat") || version.contains("RHEL") {
            Some("Red Hat Enterprise Linux".to_string())
        } else {
            None
        };

        (version_number, os_hint)
    }

    /// Calculate confidence based on extracted information
    fn calculate_confidence(has_version: bool, has_os: bool) -> f32 {
        let mut score: f32 = 0.7; // Base score for PostgreSQL detection

        if has_version {
            score += 0.15; // Version extracted
        }
        if has_os {
            score += 0.1; // OS hint found
        }

        score.min(1.0)
    }
}

impl Default for PostgresqlDetect {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolDetector for PostgresqlDetect {
    fn detect(&self, response: &[u8]) -> Result<Option<ServiceInfo>, Error> {
        // Check if response looks like PostgreSQL
        if !Self::is_postgresql_response(response) {
            return Ok(None);
        }

        // Try to extract server_version from ParameterStatus messages
        let server_version = match Self::extract_server_version(response) {
            Some(v) => v,
            None => {
                // PostgreSQL response detected but no version info
                let mut service_info = ServiceInfo::new("postgresql");
                service_info.product = Some("PostgreSQL".to_string());
                service_info.confidence = 0.6;
                return Ok(Some(service_info));
            }
        };

        // Parse version string
        let (version_number, os_hint) = Self::parse_version_string(&server_version);

        // Build ServiceInfo
        let mut service_info = ServiceInfo::new("postgresql");
        service_info.product = Some("PostgreSQL".to_string());
        service_info.version = Some(version_number);

        // Build info string
        let mut info_parts = Vec::new();
        if let Some(os) = os_hint.clone() {
            info_parts.push(os.clone());
            service_info.os_type = Some(os);
        }

        if !info_parts.is_empty() {
            service_info.info = Some(info_parts.join(" + "));
        }

        // Calculate confidence
        service_info.confidence =
            Self::calculate_confidence(!server_version.is_empty(), os_hint.is_some());

        Ok(Some(service_info))
    }

    fn confidence(&self) -> f32 {
        0.85 // PostgreSQL responses are well-structured
    }

    fn priority(&self) -> u8 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgresql14_ubuntu_detection() {
        // Minimal PostgreSQL ParameterStatus message with server_version
        // Format: 'S' + length(4 bytes) + "server_version\0" + "14.2 (Ubuntu 14.2-1ubuntu1)\0"
        let param_name = b"server_version\0";
        let param_value = b"14.2 (Ubuntu 14.2-1ubuntu1)\0";
        let content_len = param_name.len() + param_value.len();
        let msg_len = (content_len + 4) as u32; // +4 for length field itself

        let mut response = Vec::new();
        response.push(b'S'); // ParameterStatus
        response.extend_from_slice(&msg_len.to_be_bytes());
        response.extend_from_slice(param_name);
        response.extend_from_slice(param_value);

        let detector = PostgresqlDetect::new();
        let result = detector.detect(&response).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "postgresql");
        assert_eq!(info.product, Some("PostgreSQL".to_string()));
        assert_eq!(info.version, Some("14.2".to_string()));
        assert_eq!(info.os_type, Some("Ubuntu".to_string()));
        assert!(info.confidence >= 0.8);
    }

    #[test]
    fn test_postgresql13_debian_detection() {
        // PostgreSQL 13.7 on Debian
        let param_name = b"server_version\0";
        let param_value = b"13.7 (Debian 13.7-1.pgdg110+1)\0";
        let content_len = param_name.len() + param_value.len();
        let msg_len = (content_len + 4) as u32;

        let mut response = Vec::new();
        response.push(b'S');
        response.extend_from_slice(&msg_len.to_be_bytes());
        response.extend_from_slice(param_name);
        response.extend_from_slice(param_value);

        let detector = PostgresqlDetect::new();
        let result = detector.detect(&response).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "postgresql");
        assert_eq!(info.product, Some("PostgreSQL".to_string()));
        assert_eq!(info.version, Some("13.7".to_string()));
        assert_eq!(info.os_type, Some("Debian".to_string()));
        assert!(info.confidence >= 0.8);
    }

    #[test]
    fn test_postgresql12_plain_detection() {
        // PostgreSQL 12.9 without OS info
        let param_name = b"server_version\0";
        let param_value = b"12.9\0";
        let content_len = param_name.len() + param_value.len();
        let msg_len = (content_len + 4) as u32;

        let mut response = Vec::new();
        response.push(b'S');
        response.extend_from_slice(&msg_len.to_be_bytes());
        response.extend_from_slice(param_name);
        response.extend_from_slice(param_value);

        let detector = PostgresqlDetect::new();
        let result = detector.detect(&response).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "postgresql");
        assert_eq!(info.product, Some("PostgreSQL".to_string()));
        assert_eq!(info.version, Some("12.9".to_string()));
        assert!(info.confidence >= 0.7);
    }

    #[test]
    fn test_non_postgresql_response() {
        let response = b"HTTP/1.1 200 OK\r\n";

        let detector = PostgresqlDetect::new();
        let result = detector.detect(response).unwrap();

        // Should return None for non-PostgreSQL response
        assert!(result.is_none());
    }
}
