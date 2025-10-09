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

use prtip_core::{Error, Protocol, ServiceMatch, ServiceProbe, ServiceProbeDb};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Service detection engine
pub struct ServiceDetector {
    /// Service probe database
    db: Arc<ServiceProbeDb>,
    /// Detection intensity (0-9)
    intensity: u8,
    /// Timeout for probe responses
    timeout: Duration,
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
}

impl ServiceDetector {
    /// Create new service detector
    pub fn new(db: ServiceProbeDb, intensity: u8) -> Self {
        Self {
            db: Arc::new(db),
            intensity: intensity.min(9),
            timeout: Duration::from_secs(5),
        }
    }

    /// Detect service on target
    pub async fn detect_service(&self, target: SocketAddr) -> Result<ServiceInfo, Error> {
        let protocol = Protocol::Tcp; // Start with TCP
        let port = target.port();

        // Get probes for this port
        let probes = self.db.probes_for_port(port, protocol);

        // Try NULL probe first (many services self-announce)
        if let Some(null_probe) = probes.iter().find(|p| p.name == "NULL") {
            if let Ok(info) = self.try_probe(target, null_probe).await {
                return Ok(info);
            }
        }

        // Try other probes in order of rarity
        for probe in probes {
            if probe.rarity <= self.intensity {
                if let Ok(info) = self.try_probe(target, probe).await {
                    return Ok(info);
                }
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
        })
    }

    /// Try a specific probe
    async fn try_probe(
        &self,
        target: SocketAddr,
        probe: &ServiceProbe,
    ) -> Result<ServiceInfo, Error> {
        // Connect to target
        let mut stream = timeout(self.timeout, TcpStream::connect(target))
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

        match timeout(self.timeout, stream.read(&mut buf)).await {
            Ok(Ok(n)) if n > 0 => {
                response.extend_from_slice(&buf[..n]);
            }
            _ => {
                return Err(Error::Network("No response".to_string()));
            }
        }

        // Match response against patterns
        for service_match in &probe.matches {
            if let Some(info) = self.match_response(&response, service_match) {
                return Ok(info);
            }
        }

        // Try soft matches
        for service_match in &probe.soft_matches {
            if let Some(info) = self.match_response(&response, service_match) {
                return Ok(info);
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
            });
        }

        None
    }

    /// Substitute $1, $2, etc. with capture groups
    fn substitute_captures(template: &str, captures: &regex::Captures) -> String {
        let mut result = template.to_string();

        for i in 1..10 {
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
}
