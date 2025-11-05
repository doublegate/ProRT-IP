//! Service version detection engine
//!
//! This module implements service detection using probe-based matching
//! with configurable intensity levels (0-9).
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::service_detector::ServiceDetector;
//! use prtip_core::ServiceProbeDb;
//! use std::net::SocketAddr;
//!
//! # async fn example() -> Result<(), prtip_core::Error> {
//! // Create service probe database with test data
//! let db_content = r#"
//! Probe TCP GetRequest q|GET / HTTP/1.0\r\n\r\n|
//! ports 80,443
//! rarity 1
//! match http m|^HTTP/1\.[01]| p/HTTP/
//! "#;
//! let db = ServiceProbeDb::parse(&db_content)?;
//! let detector = ServiceDetector::new(db, 7); // intensity 7
//!
//! // Note: This requires network access and a live target
//! let addr = "192.168.1.1:80".parse().unwrap();
//! let result = detector.detect_service(addr).await?;
//!
//! println!("Service: {} {}", result.service, result.version.unwrap_or_default());
//! # Ok(())
//! # }
//! ```

use crate::tls_certificate::{CertificateChain, CertificateInfo, TlsFingerprint};
use crate::tls_handshake::TlsHandshake;
use prtip_core::{Error, Protocol, ServiceMatch, ServiceProbe, ServiceProbeDb};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::debug;

/// Service detection engine
pub struct ServiceDetector {
    /// Service probe database
    db: Arc<ServiceProbeDb>,
    /// Detection intensity (0-9)
    intensity: u8,
    /// Timeout for probe responses
    timeout: Duration,
    /// TLS handshake handler
    tls_handler: TlsHandshake,
    /// Whether to attempt TLS detection
    enable_tls: bool,
    /// Whether to capture raw responses for debugging
    capture_raw: bool,
}

/// Service detection result
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service name (e.g., "http", "ssh")
    pub service: String,
    /// Product name (e.g., "Apache httpd")
    pub product: Option<String>,
    /// Version string (e.g., "2.4.41")
    pub version: Option<String>,
    /// Extra info
    pub extra_info: Option<String>,
    /// Hostname
    pub hostname: Option<String>,
    /// OS type hint
    pub os_type: Option<String>,
    /// Device type
    pub device_type: Option<String>,
    /// CPE identifiers
    pub cpe: Vec<String>,
    /// Detection method
    pub method: String,
    /// Raw response
    pub raw_response: Option<Vec<u8>>,
    /// TLS certificate information (for HTTPS services)
    pub tls_certificate: Option<CertificateInfo>,
    /// TLS fingerprint (version, cipher, extensions)
    pub tls_fingerprint: Option<TlsFingerprint>,
    /// TLS certificate chain
    pub tls_chain: Option<CertificateChain>,
}

impl ServiceDetector {
    /// Create new service detector with TLS enabled
    pub fn new(db: ServiceProbeDb, intensity: u8) -> Self {
        Self::with_tls(db, intensity, true)
    }

    /// Create new service detector with optional TLS
    pub fn with_tls(db: ServiceProbeDb, intensity: u8, enable_tls: bool) -> Self {
        Self::with_options(db, intensity, enable_tls, false)
    }

    /// Create new service detector with all options
    pub fn with_options(
        db: ServiceProbeDb,
        intensity: u8,
        enable_tls: bool,
        capture_raw: bool,
    ) -> Self {
        Self {
            db: Arc::new(db),
            intensity: intensity.min(9),
            timeout: Duration::from_secs(5), // Default for general probes
            tls_handler: TlsHandshake::with_timeout(Duration::from_millis(2000)), // Faster for TLS
            enable_tls,
            capture_raw,
        }
    }

    /// Enable or disable TLS detection
    pub fn set_tls_enabled(&mut self, enabled: bool) {
        self.enable_tls = enabled;
    }

    /// Get adaptive timeout based on port and service characteristics
    ///
    /// Uses shorter timeouts for well-known fast services to reduce scan time.
    /// HTTPS services (443, 8443) get 500ms timeout since TLS handshake is typically fast.
    /// Other common TLS ports get 1s timeout.
    /// Unknown ports use the default 5s timeout for compatibility.
    fn get_adaptive_timeout(&self, port: u16) -> Duration {
        match port {
            // HTTPS ports: TLS handshake is fast, use aggressive timeout
            443 | 8443 => Duration::from_millis(500),
            // Other common TLS ports: slightly more conservative
            465 | 587 | 993 | 995 | 636 | 990 => Duration::from_secs(1),
            // SSH, FTP, HTTP: known fast protocols
            22 | 21 | 80 | 8080 => Duration::from_secs(1),
            // Unknown ports: use default conservative timeout
            _ => self.timeout,
        }
    }

    /// Detect service on target
    pub async fn detect_service(&self, target: SocketAddr) -> Result<ServiceInfo, Error> {
        self.detect_service_with_hostname(target, None).await
    }

    /// Detect service on target with optional hostname for TLS SNI
    ///
    /// When scanning HTTPS services, providing the hostname enables proper Server Name
    /// Indication (SNI), which allows the server to return the correct certificate
    /// for virtual hosts.
    ///
    /// # Arguments
    ///
    /// * `target` - Socket address (IP + port) to scan
    /// * `hostname` - Optional hostname for TLS SNI (e.g., "example.com")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_scanner::service_detector::ServiceDetector;
    /// # use prtip_core::ServiceProbeDb;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = ServiceProbeDb::parse("...")?;
    /// let detector = ServiceDetector::new(db, 7);
    ///
    /// // Scan with hostname for proper SNI
    /// let addr = "93.184.216.34:443".parse()?;
    /// let result = detector.detect_service_with_hostname(addr, Some("example.com")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_service_with_hostname(
        &self,
        target: SocketAddr,
        hostname: Option<&str>,
    ) -> Result<ServiceInfo, Error> {
        let protocol = Protocol::Tcp; // Start with TCP
        let port = target.port();

        // Get probes for this port
        let probes = self.db.probes_for_port(port, protocol);
        debug!(
            "Port {}: Found {} probes to try (intensity={})",
            port,
            probes.len(),
            self.intensity
        );

        // Log all probe names and rarities for debugging
        for (i, p) in probes.iter().enumerate() {
            debug!(
                "Port {}: Probe[{}] = {} (rarity {})",
                port, i, p.name, p.rarity
            );
        }

        // Try NULL probe first (many services self-announce)
        if let Some(null_probe) = probes.iter().find(|p| p.name == "NULL") {
            debug!("Port {}: Trying NULL probe", port);
            match self.try_probe(target, null_probe).await {
                Ok((mut info, raw_response)) => {
                    debug!("Port {}: NULL probe matched: {}", port, info.service);
                    info.raw_response = Some(raw_response);
                    return Ok(info);
                }
                Err(e) => {
                    debug!("Port {}: NULL probe failed: {}", port, e);
                }
            }
        }

        // Try other probes in order of rarity
        let mut probes_tried = 0;
        for probe in probes {
            if probe.rarity <= self.intensity {
                probes_tried += 1;
                debug!(
                    "Port {}: Trying probe {} (rarity {})",
                    port, probe.name, probe.rarity
                );
                match self.try_probe(target, probe).await {
                    Ok((mut info, raw_response)) => {
                        debug!(
                            "Port {}: Probe {} matched: {}",
                            port, probe.name, info.service
                        );
                        info.raw_response = Some(raw_response);
                        return Ok(info);
                    }
                    Err(e) => {
                        debug!("Port {}: Probe {} failed: {}", port, probe.name, e);
                    }
                }
            }
        }

        debug!(
            "Port {}: No match found after trying {} probes",
            port, probes_tried
        );

        // Try TLS detection if enabled and on common TLS port
        if self.enable_tls && TlsHandshake::is_tls_port(port) {
            debug!("Port {}: Attempting TLS handshake", port);
            if let Ok(tls_info) = self.try_tls_detection(target, hostname).await {
                debug!("Port {}: TLS handshake successful", port);
                return Ok(tls_info);
            } else {
                debug!("Port {}: TLS handshake failed", port);
            }
        }

        // No match found - return generic info

        Ok(ServiceInfo {
            service: "unknown".to_string(),

            product: None,

            version: None,

            extra_info: None,

            hostname: None,

            os_type: None,

            device_type: None,

            cpe: Vec::new(),

            method: "no match".to_string(),

            raw_response: None,

            tls_certificate: None,

            tls_fingerprint: None,

            tls_chain: None,
        })
    }

    /// Try TLS detection and extract service info from certificate
    async fn try_tls_detection(
        &self,
        target: SocketAddr,
        hostname: Option<&str>,
    ) -> Result<ServiceInfo, Error> {
        // Use hostname if provided (for SNI), otherwise fall back to IP
        let ip_string = target.ip().to_string();
        let host = hostname.unwrap_or(&ip_string);

        let port = target.port();

        // Attempt TLS handshake

        let server_info = self.tls_handler.connect(host, port).await?;

        // Try to get HTTP response over TLS for better detection

        let mut service = "https".to_string();

        let mut product = None;

        let mut version = None;

        let mut raw_response = None;

        // If port 443 or 8443, try HTTP GET to identify web server

        if port == 443 || port == 8443 {
            if let Ok(response) = self.tls_handler.https_get(host, port, "/").await {
                // Capture raw response if enabled
                if self.capture_raw {
                    raw_response = Some(response.as_bytes().to_vec());
                }

                // Parse Server header

                if let Some(server_line) = response
                    .lines()
                    .find(|l| l.to_lowercase().starts_with("server:"))
                {
                    if let Some(server_value) = server_line.split(':').nth(1) {
                        let server_value = server_value.trim();

                        // Extract product and version from "Server: nginx/1.18.0"

                        if let Some((prod, ver)) = server_value.split_once('/') {
                            product = Some(prod.to_string());

                            version = Some(ver.to_string());
                        } else {
                            product = Some(server_value.to_string());
                        }
                    }
                }
            }
        } else if port == 465 {
            service = "smtps".to_string();
        } else if port == 993 {
            service = "imaps".to_string();
        } else if port == 995 {
            service = "pop3s".to_string();
        } else if port == 636 {
            service = "ldaps".to_string();
        } else if port == 990 {
            service = "ftps".to_string();
        }

        // NEW: Extract comprehensive certificate information
        // Parse certificate using advanced tls_certificate module
        let (tls_certificate, tls_fingerprint, tls_chain) =
            self.extract_certificate_details(&server_info).await;

        Ok(ServiceInfo {
            service,

            product,

            version,

            extra_info: Some(format!(
                "TLS {} ({})",
                server_info.tls_version,
                if server_info.is_self_signed {
                    "self-signed"
                } else {
                    &server_info.issuer
                }
            )),

            hostname: Some(server_info.common_name.clone()),

            os_type: None,

            device_type: None,

            cpe: Vec::new(),

            method: "tls_handshake".to_string(),

            raw_response,

            tls_certificate,

            tls_fingerprint,

            tls_chain,
        })
    }

    /// Extract comprehensive certificate details from TLS handshake
    ///
    /// This method enhances the basic TLS handshake with full certificate parsing,
    /// chain validation, and TLS fingerprinting capabilities from the tls_certificate module.
    async fn extract_certificate_details(
        &self,
        server_info: &crate::tls_handshake::ServerInfo,
    ) -> (
        Option<CertificateInfo>,
        Option<TlsFingerprint>,
        Option<CertificateChain>,
    ) {
        use crate::tls_certificate::{parse_certificate, parse_certificate_chain};

        if server_info.raw_cert_chain.is_empty() {
            debug!("No raw certificate chain available for analysis");
            return (None, None, None);
        }

        // Parse leaf certificate (server's certificate)
        let tls_certificate = match parse_certificate(&server_info.raw_cert_chain[0]) {
            Ok(cert) => {
                debug!(
                    "Parsed certificate: subject={}, issuer={}, SAN count={}",
                    cert.subject,
                    cert.issuer,
                    cert.san.len()
                );
                Some(cert)
            }
            Err(e) => {
                debug!("Failed to parse certificate: {}", e);
                None
            }
        };

        // Parse full certificate chain
        let tls_chain = {
            let cert_refs: Vec<&[u8]> = server_info
                .raw_cert_chain
                .iter()
                .map(|c| c.as_slice())
                .collect();

            match parse_certificate_chain(cert_refs) {
                Ok(chain) => {
                    debug!(
                        "Parsed certificate chain: depth={}, self-signed={}, valid={}",
                        chain.certificates.len(),
                        chain.is_self_signed,
                        chain.is_valid
                    );
                    Some(chain)
                }
                Err(e) => {
                    debug!("Failed to parse certificate chain: {}", e);
                    None
                }
            }
        };

        // Create TLS fingerprint
        // Note: We don't have access to the raw ServerHello message here,
        // so we create a basic fingerprint from available information
        let tls_fingerprint = Some(TlsFingerprint {
            tls_version: server_info.tls_version.clone(),
            cipher_suites: vec![], // Would need ServerHello to populate
            extensions: vec![],    // Would need ServerHello to populate
        });

        debug!(
            "Certificate extraction complete: cert={}, chain={}, fingerprint={}",
            tls_certificate.is_some(),
            tls_chain.is_some(),
            tls_fingerprint.is_some()
        );

        (tls_certificate, tls_fingerprint, tls_chain)
    }

    /// Try a specific probe
    async fn try_probe(
        &self,

        target: SocketAddr,

        probe: &ServiceProbe,
    ) -> Result<(ServiceInfo, Vec<u8>), Error> {
        // Use adaptive timeout based on port
        let probe_timeout = self.get_adaptive_timeout(target.port());

        // Connect to target
        let mut stream = timeout(probe_timeout, TcpStream::connect(target))
            .await
            .map_err(|_| Error::Network("Connection timeout".to_string()))?
            .map_err(|e| Error::Network(format!("Connection failed: {}", e)))?;

        // Send probe (if not NULL)

        if !probe.probe_string.is_empty() {
            stream
                .write_all(&probe.probe_string)
                .await
                .map_err(|e| Error::Network(format!("Write failed: {}", e)))?;
        }

        // Read response

        let mut response = Vec::new();

        let mut buf = [0u8; 4096];

        match timeout(probe_timeout, stream.read(&mut buf)).await {
            Ok(Ok(n)) if n > 0 => {
                response.extend_from_slice(&buf[..n]);
            }

            _ => {
                return Err(Error::Network("No response".to_string()));
            }
        }

        // Log the raw response for debugging
        debug!("Raw response for port {}: {:?}", target.port(), response);

        // Conditionally capture raw response based on flag
        let raw_response = if self.capture_raw {
            response.clone()
        } else {
            Vec::new()
        };

        for service_match in &probe.matches {
            if let Some(info) = self.match_response(&response, service_match) {
                return Ok((info, raw_response));
            }
        }

        // Try soft matches

        for service_match in &probe.soft_matches {
            if let Some(info) = self.match_response(&response, service_match) {
                return Ok((info, raw_response));
            }
        }

        Err(Error::Detection("No pattern match".to_string()))
    }

    /// Match response against pattern
    fn match_response(&self, response: &[u8], service_match: &ServiceMatch) -> Option<ServiceInfo> {
        // Convert response to string (lossy)

        let response_str = String::from_utf8_lossy(response);

        // Check if pattern matches

        if let Some(captures) = service_match.pattern.captures(&response_str) {
            // Extract version info using capture groups

            let product = service_match
                .product
                .as_ref()
                .map(|p| Self::substitute_captures(p, &captures));

            let version = service_match
                .version
                .as_ref()
                .map(|v| Self::substitute_captures(v, &captures));

            let extra_info = service_match
                .info
                .as_ref()
                .map(|i| Self::substitute_captures(i, &captures));

            return Some(ServiceInfo {
                service: service_match.service.clone(),
                product,
                version,
                extra_info,
                hostname: service_match.hostname.clone(),
                os_type: service_match.os_type.clone(),
                device_type: service_match.device_type.clone(),
                cpe: service_match.cpe.clone(),
                method: "pattern match".to_string(),
                raw_response: None,
                tls_certificate: None,
                tls_fingerprint: None,
                tls_chain: None,
            });
        }

        None
    }

    /// Substitute $1, $2, etc. with capture groups
    fn substitute_captures(template: &str, captures: &regex::Captures) -> String {
        let mut result = template.to_string();

        for i in 1..captures.len().min(10) {
            let placeholder = format!("${}", i);
            if let Some(cap) = captures.get(i) {
                result = result.replace(&placeholder, cap.as_str());
            }
        }

        result
    }

    /// Set timeout for probes
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prtip_core::ServiceProbeDb;
    use regex::Regex;

    #[tokio::test]
    async fn test_detect_service_http() {
        let db = ServiceProbeDb::default();
        let detector = ServiceDetector::new(db, 9);
        let addr = "45.33.32.156:80".parse().unwrap();
        let result = detector.detect_service(addr).await.unwrap();
        println!("{:?}", result);
    }

    #[test]
    fn test_create_detector() {
        let db = ServiceProbeDb::new();
        let detector = ServiceDetector::new(db, 7);
        assert_eq!(detector.intensity, 7);
    }

    #[test]
    fn test_substitute_captures() {
        let pattern = Regex::new(r"Server: (\S+)/(\S+)").unwrap();
        let text = "Server: nginx/1.18.0";

        if let Some(captures) = pattern.captures(text) {
            let result = ServiceDetector::substitute_captures("$1 version $2", &captures);
            assert_eq!(result, "nginx version 1.18.0");
        }
    }

    #[test]
    fn test_intensity_clamping() {
        let db = ServiceProbeDb::new();
        let detector = ServiceDetector::new(db, 15);
        assert_eq!(detector.intensity, 9); // Clamped to max
    }

    #[test]
    fn test_intensity_zero() {
        let db = ServiceProbeDb::new();
        let detector = ServiceDetector::new(db, 0);
        assert_eq!(detector.intensity, 0);
    }

    #[test]
    fn test_intensity_boundary() {
        let db = ServiceProbeDb::new();

        // Test boundary values
        let detector_9 = ServiceDetector::new(db.clone(), 9);
        assert_eq!(detector_9.intensity, 9);

        let detector_10 = ServiceDetector::new(db.clone(), 10);
        assert_eq!(detector_10.intensity, 9); // Clamped

        let detector_255 = ServiceDetector::new(db, 255);
        assert_eq!(detector_255.intensity, 9); // Clamped
    }

    #[test]
    fn test_set_timeout() {
        let db = ServiceProbeDb::new();
        let mut detector = ServiceDetector::new(db, 7);

        // Test timeout modification
        detector.set_timeout(Duration::from_secs(10));
        assert_eq!(detector.timeout, Duration::from_secs(10));

        detector.set_timeout(Duration::from_millis(500));
        assert_eq!(detector.timeout, Duration::from_millis(500));
    }

    #[test]
    fn test_default_timeout() {
        let db = ServiceProbeDb::new();
        let detector = ServiceDetector::new(db, 7);
        assert_eq!(detector.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_substitute_captures_no_placeholders() {
        let pattern = Regex::new(r"Server: (\S+)").unwrap();
        let text = "Server: nginx";

        if let Some(captures) = pattern.captures(text) {
            let result = ServiceDetector::substitute_captures("Static text", &captures);
            assert_eq!(result, "Static text");
        }
    }

    #[test]
    fn test_substitute_captures_multiple() {
        let pattern = Regex::new(r"(\w+)/(\d+\.\d+) on (\w+)").unwrap();
        let text = "Apache/2.4 on Linux";

        if let Some(captures) = pattern.captures(text) {
            let result = ServiceDetector::substitute_captures("$1 ver $2 ($3)", &captures);
            assert_eq!(result, "Apache ver 2.4 (Linux)");
        }
    }

    #[test]
    fn test_substitute_captures_missing_group() {
        let pattern = Regex::new(r"(\w+)").unwrap();
        let text = "nginx";

        if let Some(captures) = pattern.captures(text) {
            // $2 doesn't exist, should remain as-is
            let result = ServiceDetector::substitute_captures("$1 and $2", &captures);
            assert_eq!(result, "nginx and $2");
        }
    }

    #[test]
    fn test_substitute_captures_all_groups() {
        let pattern = Regex::new(r"(\w+)/(\w+)/(\w+)/(\w+)/(\w+)").unwrap();
        let text = "a/b/c/d/e";

        if let Some(captures) = pattern.captures(text) {
            let result = ServiceDetector::substitute_captures("$1 $2 $3 $4 $5", &captures);
            assert_eq!(result, "a b c d e");
        }
    }

    #[test]
    fn test_service_info_creation() {
        let info = ServiceInfo {
            service: "http".to_string(),
            product: Some("nginx".to_string()),
            version: Some("1.18.0".to_string()),
            extra_info: Some("Ubuntu".to_string()),
            hostname: Some("example.com".to_string()),
            os_type: Some("Linux".to_string()),
            device_type: Some("general purpose".to_string()),
            cpe: vec!["cpe:/a:nginx:nginx:1.18.0".to_string()],
            method: "pattern match".to_string(),
            raw_response: None,
            tls_certificate: None,
            tls_fingerprint: None,
            tls_chain: None,
        };

        assert_eq!(info.service, "http");
        assert_eq!(info.product, Some("nginx".to_string()));
        assert_eq!(info.version, Some("1.18.0".to_string()));
        assert_eq!(info.method, "pattern match");
    }

    #[test]
    fn test_service_info_minimal() {
        let info = ServiceInfo {
            service: "unknown".to_string(),
            product: None,
            version: None,
            extra_info: None,
            hostname: None,
            os_type: None,
            device_type: None,
            cpe: Vec::new(),
            method: "no match".to_string(),
            raw_response: None,
            tls_certificate: None,
            tls_fingerprint: None,
            tls_chain: None,
        };

        assert_eq!(info.service, "unknown");
        assert_eq!(info.product, None);
        assert_eq!(info.version, None);
        assert_eq!(info.method, "no match");
        assert!(info.cpe.is_empty());
    }

    #[test]
    fn test_service_info_clone() {
        let info = ServiceInfo {
            service: "ssh".to_string(),
            product: Some("OpenSSH".to_string()),
            version: Some("8.2".to_string()),
            extra_info: None,
            hostname: None,
            os_type: None,
            device_type: None,
            cpe: Vec::new(),
            method: "banner".to_string(),
            raw_response: None,
            tls_certificate: None,
            tls_fingerprint: None,
            tls_chain: None,
        };

        let cloned = info.clone();
        assert_eq!(cloned.service, info.service);
        assert_eq!(cloned.product, info.product);
        assert_eq!(cloned.version, info.version);
    }

    #[test]
    fn test_service_info_debug() {
        let info = ServiceInfo {
            service: "http".to_string(),
            product: Some("Apache".to_string()),
            version: Some("2.4".to_string()),
            extra_info: None,
            hostname: None,
            os_type: None,
            device_type: None,
            cpe: Vec::new(),
            method: "test".to_string(),
            raw_response: None,
            tls_certificate: None,
            tls_fingerprint: None,
            tls_chain: None,
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("http"));
        assert!(debug_str.contains("Apache"));
        assert!(debug_str.contains("2.4"));
    }

    #[test]
    fn test_detector_with_empty_db() {
        let db = ServiceProbeDb::new();
        let detector = ServiceDetector::new(db, 7);
        assert_eq!(detector.intensity, 7);
        assert_eq!(detector.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_multiple_cpe_entries() {
        let info = ServiceInfo {
            service: "http".to_string(),
            product: Some("nginx".to_string()),
            version: Some("1.18.0".to_string()),
            extra_info: None,
            hostname: None,
            os_type: None,
            device_type: None,
            cpe: vec![
                "cpe:/a:nginx:nginx:1.18.0".to_string(),
                "cpe:/a:nginx:nginx".to_string(),
            ],
            method: "pattern".to_string(),
            raw_response: None,
            tls_certificate: None,
            tls_fingerprint: None,
            tls_chain: None,
        };

        assert_eq!(info.cpe.len(), 2);
        assert!(info.cpe[0].contains("1.18.0"));
        assert!(!info.cpe[1].contains("1.18.0"));
    }

    #[test]
    fn test_substitute_with_special_characters() {
        let pattern = Regex::new(r"Server: ([^\s]+)").unwrap();
        let text = "Server: nginx/1.18.0(Ubuntu)";

        if let Some(captures) = pattern.captures(text) {
            let result = ServiceDetector::substitute_captures("Found: $1", &captures);
            assert_eq!(result, "Found: nginx/1.18.0(Ubuntu)");
        }
    }
}
