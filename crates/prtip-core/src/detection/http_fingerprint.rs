//! HTTP service fingerprinting through header parsing
//!
//! This module extracts detailed version information from HTTP response headers,
//! including Server, X-Powered-By, and X-AspNet-Version headers.
//!
//! # Detection Strategy
//!
//! 1. Parse HTTP response status line (HTTP/1.x, HTTP/2)
//! 2. Extract Server header (Apache, nginx, IIS, etc.)
//! 3. Extract X-Powered-By header (PHP, ASP.NET, Express, etc.)
//! 4. Extract X-AspNet-Version header (IIS version mapping)
//! 5. Combine information with confidence scoring
//!
//! # Examples
//!
//! ```
//! use prtip_core::detection::http_fingerprint::HttpFingerprint;
//! use prtip_core::detection::ProtocolDetector;
//!
//! let response = b"HTTP/1.1 200 OK\r\n\
//!                  Server: Apache/2.4.41 (Ubuntu)\r\n\
//!                  X-Powered-By: PHP/7.4.3\r\n\
//!                  \r\n";
//!
//! let detector = HttpFingerprint::new();
//! if let Ok(Some(info)) = detector.detect(response) {
//!     assert_eq!(info.service, "http");
//!     assert_eq!(info.product, Some("Apache".to_string()));
//!     assert_eq!(info.version, Some("2.4.41".to_string()));
//!     assert!(info.info.as_ref().unwrap().contains("PHP/7.4.3"));
//!     assert!(info.confidence > 0.8); // High confidence
//! }
//! ```

use super::{ProtocolDetector, ServiceInfo};
use crate::Error;
use std::str;

/// HTTP fingerprinting detector
#[derive(Debug, Clone)]
pub struct HttpFingerprint {
    priority: u8,
}

impl HttpFingerprint {
    /// Create new HTTP fingerprint detector
    pub fn new() -> Self {
        Self { priority: 1 } // Highest priority
    }

    /// Parse HTTP response headers
    fn parse_headers(response: &[u8]) -> Option<Vec<(String, String)>> {
        let response_str = str::from_utf8(response).ok()?;

        // Find header section (between first \r\n and \r\n\r\n)
        let header_end = response_str.find("\r\n\r\n")?;
        let headers_section = &response_str[..header_end];

        let mut headers = Vec::new();
        for line in headers_section.lines().skip(1) {
            // Skip status line
            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.push((name, value));
            }
        }

        Some(headers)
    }

    /// Extract Server header information
    fn parse_server_header(server: &str) -> (Option<String>, Option<String>, Option<String>) {
        // Examples:
        // "Apache/2.4.41 (Ubuntu)"
        // "nginx/1.18.0"
        // "Microsoft-IIS/10.0"
        // "cloudflare"

        let mut product = None;
        let mut version = None;
        let mut os_info = None;

        // Split by space to separate product/version from OS/comments
        let parts: Vec<&str> = server.split_whitespace().collect();

        if let Some(first_part) = parts.first() {
            // Parse "Product/Version"
            if let Some(slash_pos) = first_part.find('/') {
                product = Some(first_part[..slash_pos].to_string());
                version = Some(first_part[slash_pos + 1..].to_string());
            } else {
                // No version, just product name
                product = Some(first_part.to_string());
            }
        }

        // Parse OS/comment in parentheses
        if let Some(paren_start) = server.find('(') {
            if let Some(paren_end) = server.find(')') {
                os_info = Some(server[paren_start + 1..paren_end].to_string());
            }
        }

        (product, version, os_info)
    }

    /// Extract X-Powered-By header information
    fn parse_powered_by(powered_by: &str) -> Option<String> {
        // Examples:
        // "PHP/7.4.3"
        // "ASP.NET"
        // "Express"

        Some(powered_by.to_string())
    }

    /// Extract X-AspNet-Version header
    fn parse_aspnet_version(aspnet_version: &str) -> Option<String> {
        // Example: "4.0.30319"
        // Maps to .NET Framework version
        Some(aspnet_version.to_string())
    }

    /// Calculate confidence score based on available information
    fn calculate_confidence(
        has_server: bool,
        has_version: bool,
        has_os: bool,
        has_powered_by: bool,
    ) -> f32 {
        let mut score: f32 = 0.5; // Base score for HTTP detection

        if has_server {
            score += 0.2; // Server header found
        }
        if has_version {
            score += 0.15; // Version extracted
        }
        if has_os {
            score += 0.1; // OS info found
        }
        if has_powered_by {
            score += 0.05; // Additional technology stack info
        }

        score.min(1.0)
    }
}

impl Default for HttpFingerprint {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolDetector for HttpFingerprint {
    fn detect(&self, response: &[u8]) -> Result<Option<ServiceInfo>, Error> {
        // Check if response looks like HTTP
        let response_str = str::from_utf8(response)
            .map_err(|_| Error::Parse("HTTP response contains invalid UTF-8".to_string()))?;

        // Must start with "HTTP/"
        if !response_str.starts_with("HTTP/") {
            return Ok(None);
        }

        // Parse headers
        let headers = match Self::parse_headers(response) {
            Some(h) => h,
            None => return Ok(None),
        };

        let mut service_info = ServiceInfo::new("http");
        let mut has_server = false;
        let mut has_version = false;
        let mut has_os = false;
        let mut has_powered_by = false;
        let mut additional_info = Vec::new();

        // Process headers
        for (name, value) in headers {
            match name.as_str() {
                "server" => {
                    has_server = true;
                    let (product, version, os_info) = Self::parse_server_header(&value);

                    if let Some(p) = product {
                        service_info.product = Some(p);
                    }
                    if let Some(v) = version {
                        has_version = true;
                        service_info.version = Some(v);
                    }
                    if let Some(os) = os_info {
                        has_os = true;
                        service_info.os_type = Some(os.clone());
                        additional_info.push(os);
                    }
                }
                "x-powered-by" => {
                    has_powered_by = true;
                    if let Some(powered) = Self::parse_powered_by(&value) {
                        additional_info.push(powered);
                    }
                }
                "x-aspnet-version" => {
                    if let Some(aspnet) = Self::parse_aspnet_version(&value) {
                        additional_info.push(format!("ASP.NET {}", aspnet));
                    }
                }
                _ => {}
            }
        }

        // Combine additional info
        if !additional_info.is_empty() {
            service_info.info = Some(additional_info.join(" + "));
        }

        // Calculate confidence
        service_info.confidence =
            Self::calculate_confidence(has_server, has_version, has_os, has_powered_by);

        Ok(Some(service_info))
    }

    fn confidence(&self) -> f32 {
        0.9 // HTTP fingerprinting is highly reliable
    }

    fn priority(&self) -> u8 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apache_ubuntu_detection() {
        let response = b"HTTP/1.1 200 OK\r\n\
                         Server: Apache/2.4.41 (Ubuntu)\r\n\
                         Content-Type: text/html\r\n\
                         \r\n\
                         <html></html>";

        let detector = HttpFingerprint::new();
        let result = detector.detect(response).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "http");
        assert_eq!(info.product, Some("Apache".to_string()));
        assert_eq!(info.version, Some("2.4.41".to_string()));
        assert_eq!(info.os_type, Some("Ubuntu".to_string()));
        assert!(info.confidence > 0.8);
    }

    #[test]
    fn test_nginx_detection() {
        let response = b"HTTP/1.1 200 OK\r\n\
                         Server: nginx/1.18.0\r\n\
                         \r\n";

        let detector = HttpFingerprint::new();
        let result = detector.detect(response).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "http");
        assert_eq!(info.product, Some("nginx".to_string()));
        assert_eq!(info.version, Some("1.18.0".to_string()));
        assert!(info.confidence >= 0.7);
    }

    #[test]
    fn test_iis_detection() {
        let response = b"HTTP/1.1 200 OK\r\n\
                         Server: Microsoft-IIS/10.0\r\n\
                         X-Powered-By: ASP.NET\r\n\
                         \r\n";

        let detector = HttpFingerprint::new();
        let result = detector.detect(response).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "http");
        assert_eq!(info.product, Some("Microsoft-IIS".to_string()));
        assert_eq!(info.version, Some("10.0".to_string()));
        assert!(info.info.is_some());
        assert!(info.info.unwrap().contains("ASP.NET"));
        assert!(info.confidence > 0.7);
    }

    #[test]
    fn test_php_powered_by() {
        let response = b"HTTP/1.1 200 OK\r\n\
                         Server: Apache/2.4.41\r\n\
                         X-Powered-By: PHP/7.4.3\r\n\
                         \r\n";

        let detector = HttpFingerprint::new();
        let result = detector.detect(response).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert!(info.info.is_some());
        assert!(info.info.unwrap().contains("PHP/7.4.3"));
        assert!(info.confidence > 0.8);
    }

    #[test]
    fn test_aspnet_version() {
        let response = b"HTTP/1.1 200 OK\r\n\
                         Server: Microsoft-IIS/10.0\r\n\
                         X-AspNet-Version: 4.0.30319\r\n\
                         \r\n";

        let detector = HttpFingerprint::new();
        let result = detector.detect(response).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert!(info.info.is_some());
        assert!(info.info.unwrap().contains("ASP.NET 4.0.30319"));
    }

    #[test]
    fn test_malformed_header() {
        let response = b"HTTP/1.1 200 OK\r\n\
                         Invalid-Header-Without-Colon\r\n\
                         Server: Apache\r\n\
                         \r\n";

        let detector = HttpFingerprint::new();
        let result = detector.detect(response).unwrap();

        // Should still work despite malformed header
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.product, Some("Apache".to_string()));
    }

    #[test]
    fn test_non_http_response() {
        let response = b"SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.3\r\n";

        let detector = HttpFingerprint::new();
        let result = detector.detect(response).unwrap();

        // Should return None for non-HTTP response
        assert!(result.is_none());
    }

    #[test]
    fn test_minimal_http_response() {
        let response = b"HTTP/1.1 200 OK\r\n\r\n";

        let detector = HttpFingerprint::new();
        let result = detector.detect(response).unwrap();

        // Should detect HTTP even without Server header
        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.service, "http");
        assert!(info.confidence >= 0.5); // Base confidence
    }
}
