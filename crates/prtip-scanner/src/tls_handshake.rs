//! TLS handshake implementation for service detection
//!
//! This module provides TLS/SSL handshake capabilities to detect and identify
//! encrypted services (HTTPS, SMTPS, IMAPS, POP3S, FTPS, etc.).
//!
//! # Features
//!
//! - TLS 1.2 and 1.3 support
//! - Certificate parsing (CN, SAN, issuer, expiry)
//! - Configurable timeouts
//! - Accept self-signed/expired certificates (reconnaissance mode)
//! - Server info extraction for service identification
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::tls_handshake::TlsHandshake;
//! use std::net::TcpStream;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let tls_handler = TlsHandshake::new();
//! let server_info = tls_handler.connect("example.com", 443).await?;
//! println!("Server: {} (TLS {})", server_info.common_name, server_info.tls_version);
//! # Ok(())
//! # }
//! ```

use prtip_core::Error;
use rustls::{ClientConfig, RootCertStore, ServerName};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_rustls::{client::TlsStream, TlsConnector};
use tracing::debug;
use x509_parser::prelude::*;

/// Common TLS ports for auto-detection
pub const TLS_PORTS: &[u16] = &[
    443,  // HTTPS
    465,  // SMTPS
    993,  // IMAPS
    995,  // POP3S
    990,  // FTPS control
    989,  // FTPS data
    636,  // LDAPS
    3389, // RDP over TLS
    8443, // Alt HTTPS
];

/// TLS handshake manager
#[derive(Clone)]
pub struct TlsHandshake {
    /// TLS connector with configuration
    connector: Arc<TlsConnector>,
    /// Timeout for handshake operations
    timeout_duration: Duration,
}

/// Server information extracted from TLS certificate and handshake
#[derive(Debug, Clone, PartialEq)]
pub struct ServerInfo {
    /// Common Name from certificate
    pub common_name: String,
    /// Subject Alternative Names (DNS names)
    pub subject_alt_names: Vec<String>,
    /// Certificate issuer
    pub issuer: String,
    /// Certificate expiry time
    pub expiry: SystemTime,
    /// Negotiated TLS version
    pub tls_version: String,
    /// Certificate serial number
    pub serial_number: String,
    /// Whether certificate is self-signed
    pub is_self_signed: bool,
}

impl TlsHandshake {
    /// Create new TLS handshake manager with default configuration
    ///
    /// Uses rustls with WebPKI root certificates. Accepts invalid certificates
    /// for reconnaissance purposes.
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(5))
    }

    /// Create TLS handshake manager with custom timeout
    pub fn with_timeout(timeout_duration: Duration) -> Self {
        // Create root certificate store with WebPKI roots
        let mut root_store = RootCertStore::empty();
        root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        // Create config that accepts invalid certificates (for recon)
        let mut config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Accept invalid certificates for reconnaissance
        config
            .dangerous()
            .set_certificate_verifier(Arc::new(AcceptAllVerifier));

        let connector = TlsConnector::from(Arc::new(config));

        Self {
            connector: Arc::new(connector),
            timeout_duration,
        }
    }

    /// Check if a port is commonly used for TLS
    pub fn is_tls_port(port: u16) -> bool {
        TLS_PORTS.contains(&port)
    }

    /// Attempt TLS handshake to target
    ///
    /// Returns ServerInfo on success, or Error if handshake fails.
    /// Accepts self-signed and expired certificates for reconnaissance.
    pub async fn connect(&self, host: &str, port: u16) -> Result<ServerInfo, Error> {
        debug!("Attempting TLS handshake to {}:{}", host, port);

        // Connect TCP stream
        let tcp_stream = timeout(
            self.timeout_duration,
            TcpStream::connect(format!("{}:{}", host, port)),
        )
        .await
        .map_err(|_| Error::Network("TLS connection timeout".to_string()))?
        .map_err(|e| Error::Network(format!("TCP connection failed: {}", e)))?;

        // Parse server name (required for SNI)
        let server_name = ServerName::try_from(host)
            .map_err(|e| Error::Network(format!("Invalid hostname: {}", e)))?;

        // Perform TLS handshake
        let tls_stream = timeout(
            self.timeout_duration,
            self.connector.connect(server_name, tcp_stream),
        )
        .await
        .map_err(|_| Error::Network("TLS handshake timeout".to_string()))?
        .map_err(|e| Error::Network(format!("TLS handshake failed: {}", e)))?;

        // Extract server information
        self.extract_server_info(&tls_stream).await
    }

    /// Extract server information from TLS stream
    async fn extract_server_info(
        &self,
        stream: &TlsStream<TcpStream>,
    ) -> Result<ServerInfo, Error> {
        // Get connection info
        let (_io, connection) = stream.get_ref();

        // Get TLS version
        let tls_version = match connection.protocol_version() {
            Some(rustls::ProtocolVersion::TLSv1_2) => "TLSv1.2",
            Some(rustls::ProtocolVersion::TLSv1_3) => "TLSv1.3",
            Some(v) => {
                debug!("Unknown TLS version: {:?}", v);
                "TLS"
            }
            None => "unknown",
        }
        .to_string();

        // Get peer certificates
        let certs = connection
            .peer_certificates()
            .ok_or_else(|| Error::Detection("No certificates provided".to_string()))?;

        if certs.is_empty() {
            return Err(Error::Detection("Empty certificate chain".to_string()));
        }

        // Parse first certificate (server certificate)
        let cert_der = &certs[0].0;
        let (_, cert) = X509Certificate::from_der(cert_der)
            .map_err(|e| Error::Detection(format!("Certificate parsing failed: {}", e)))?;

        // Extract common name
        let common_name = cert
            .subject()
            .iter_common_name()
            .next()
            .and_then(|cn| cn.as_str().ok())
            .unwrap_or("unknown")
            .to_string();

        // Extract subject alternative names (SANs)
        let mut subject_alt_names = Vec::new();
        if let Some(san_ext) = cert
            .extensions()
            .iter()
            .find(|ext| ext.oid == x509_parser::oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME)
        {
            if let x509_parser::extensions::ParsedExtension::SubjectAlternativeName(san) =
                san_ext.parsed_extension()
            {
                for name in &san.general_names {
                    if let x509_parser::extensions::GeneralName::DNSName(dns) = name {
                        subject_alt_names.push(dns.to_string());
                    }
                }
            }
        }

        // Extract issuer
        let issuer = cert
            .issuer()
            .iter_common_name()
            .next()
            .and_then(|cn| cn.as_str().ok())
            .unwrap_or("unknown")
            .to_string();

        // Check if self-signed
        let is_self_signed = cert.subject() == cert.issuer();

        // Extract expiry - convert from x509_parser time to SystemTime
        let expiry = {
            let not_after = cert.validity().not_after.to_datetime();
            // Convert to UNIX timestamp (seconds since epoch)
            let timestamp_secs = not_after.unix_timestamp() as u64;
            SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp_secs)
        };

        // Extract serial number
        let serial_number = format!("{:X}", cert.serial);

        debug!(
            "TLS handshake successful: {} (TLS {}, issuer: {})",
            common_name, tls_version, issuer
        );

        Ok(ServerInfo {
            common_name,
            subject_alt_names,
            issuer,
            expiry,
            tls_version,
            serial_number,
            is_self_signed,
        })
    }

    /// Perform TLS handshake and read HTTP response (for HTTPS detection)
    pub async fn https_get(&self, host: &str, port: u16, path: &str) -> Result<String, Error> {
        // Connect TCP stream
        let tcp_stream = timeout(
            self.timeout_duration,
            TcpStream::connect(format!("{}:{}", host, port)),
        )
        .await
        .map_err(|_| Error::Network("Connection timeout".to_string()))?
        .map_err(|e| Error::Network(format!("Connection failed: {}", e)))?;

        // Parse server name
        let server_name = ServerName::try_from(host)
            .map_err(|e| Error::Network(format!("Invalid hostname: {}", e)))?;

        // Perform TLS handshake
        let mut tls_stream = timeout(
            self.timeout_duration,
            self.connector.connect(server_name, tcp_stream),
        )
        .await
        .map_err(|_| Error::Network("TLS handshake timeout".to_string()))?
        .map_err(|e| Error::Network(format!("TLS handshake failed: {}", e)))?;

        // Send HTTP GET request
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host
        );
        tls_stream
            .write_all(request.as_bytes())
            .await
            .map_err(|e| Error::Network(format!("Write failed: {}", e)))?;

        // Read response
        let mut response = String::new();
        tls_stream
            .read_to_string(&mut response)
            .await
            .map_err(|e| Error::Network(format!("Read failed: {}", e)))?;

        Ok(response)
    }

    /// Set timeout for TLS operations
    pub fn set_timeout(&mut self, timeout_duration: Duration) {
        self.timeout_duration = timeout_duration;
    }
}

impl Default for TlsHandshake {
    fn default() -> Self {
        Self::new()
    }
}

/// Certificate verifier that accepts all certificates (for reconnaissance)
struct AcceptAllVerifier;

impl rustls::client::ServerCertVerifier for AcceptAllVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        // Accept all certificates for reconnaissance purposes
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tls_handshake() {
        let tls = TlsHandshake::new();
        assert_eq!(tls.timeout_duration, Duration::from_secs(5));
    }

    #[test]
    fn test_create_with_custom_timeout() {
        let timeout = Duration::from_secs(10);
        let tls = TlsHandshake::with_timeout(timeout);
        assert_eq!(tls.timeout_duration, timeout);
    }

    #[test]
    fn test_is_tls_port() {
        assert!(TlsHandshake::is_tls_port(443));
        assert!(TlsHandshake::is_tls_port(465));
        assert!(TlsHandshake::is_tls_port(993));
        assert!(TlsHandshake::is_tls_port(995));
        assert!(TlsHandshake::is_tls_port(8443));
        assert!(!TlsHandshake::is_tls_port(80));
        assert!(!TlsHandshake::is_tls_port(22));
    }

    #[test]
    fn test_set_timeout() {
        let mut tls = TlsHandshake::new();
        let new_timeout = Duration::from_secs(3);
        tls.set_timeout(new_timeout);
        assert_eq!(tls.timeout_duration, new_timeout);
    }

    #[test]
    fn test_default_impl() {
        let tls = TlsHandshake::default();
        assert_eq!(tls.timeout_duration, Duration::from_secs(5));
    }

    #[test]
    fn test_server_info_equality() {
        let info1 = ServerInfo {
            common_name: "example.com".to_string(),
            subject_alt_names: vec!["*.example.com".to_string()],
            issuer: "Let's Encrypt".to_string(),
            expiry: SystemTime::now(),
            tls_version: "TLSv1.3".to_string(),
            serial_number: "ABC123".to_string(),
            is_self_signed: false,
        };

        let info2 = info1.clone();
        assert_eq!(info1, info2);
    }

    #[test]
    fn test_server_info_debug() {
        let info = ServerInfo {
            common_name: "test.com".to_string(),
            subject_alt_names: vec![],
            issuer: "Test CA".to_string(),
            expiry: SystemTime::now(),
            tls_version: "TLSv1.2".to_string(),
            serial_number: "123".to_string(),
            is_self_signed: true,
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("test.com"));
        assert!(debug_str.contains("TLSv1.2"));
    }

    // Integration tests (require network access)
    #[tokio::test]
    #[ignore] // Run with --ignored for network tests
    async fn test_tls_handshake_success() {
        let tls = TlsHandshake::new();
        let result = tls.connect("example.com", 443).await;
        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(!info.common_name.is_empty());
        assert!(!info.tls_version.is_empty());
    }

    #[tokio::test]
    #[ignore] // Run with --ignored for network tests
    async fn test_tls_certificate_parsing() {
        let tls = TlsHandshake::new();
        let info = tls.connect("www.google.com", 443).await.unwrap();

        // Google's certificate should have specific properties
        assert!(info.common_name.contains("google") || !info.subject_alt_names.is_empty());
        assert!(!info.issuer.is_empty());
        assert!(info.expiry > SystemTime::now()); // Not expired
    }

    #[tokio::test]
    #[ignore] // Run with --ignored for network tests
    async fn test_tls_version_negotiation() {
        let tls = TlsHandshake::new();
        let info = tls.connect("www.cloudflare.com", 443).await.unwrap();

        // Should negotiate TLS 1.2 or 1.3
        assert!(info.tls_version == "TLSv1.2" || info.tls_version == "TLSv1.3");
    }

    #[tokio::test]
    async fn test_tls_handshake_timeout() {
        let tls = TlsHandshake::with_timeout(Duration::from_millis(1));
        let result = tls.connect("example.com", 443).await;

        // Should timeout with very short duration
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }

    #[tokio::test]
    async fn test_tls_handshake_invalid_host() {
        let tls = TlsHandshake::new();
        let result = tls
            .connect("invalid-host-that-does-not-exist.com", 443)
            .await;

        // Should fail with connection error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tls_handshake_refused() {
        let tls = TlsHandshake::new();
        // Port 1 should be refused or filtered
        let result = tls.connect("localhost", 1).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Run with --ignored for network tests
    async fn test_tls_wrapped_http() {
        let tls = TlsHandshake::new();
        let response = tls.https_get("httpbin.org", 443, "/get").await;

        assert!(response.is_ok());
        let body = response.unwrap();
        assert!(body.contains("HTTP/"));
        assert!(body.contains("200") || body.contains("301") || body.contains("302"));
    }

    #[tokio::test]
    #[ignore] // Run with --ignored for network tests
    async fn test_tls_port_detection() {
        let tls = TlsHandshake::new();

        // Test common HTTPS port
        let result443 = tls.connect("www.github.com", 443).await;
        assert!(result443.is_ok());

        // Test alternative HTTPS port (if available)
        // Note: This might fail if the service doesn't use 8443
        let _result8443 = tls.connect("www.github.com", 8443).await;
        // Don't assert this one as not all services use 8443
    }

    #[tokio::test]
    async fn test_tls_fallback_plaintext() {
        let tls = TlsHandshake::new();

        // Try TLS on HTTP port (should fail)
        let result = tls.connect("example.com", 80).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_tls_ports_constant() {
        assert_eq!(TLS_PORTS.len(), 9);
        assert!(TLS_PORTS.contains(&443));
        assert!(TLS_PORTS.contains(&465));
        assert!(TLS_PORTS.contains(&993));
        assert!(TLS_PORTS.contains(&995));
        assert!(TLS_PORTS.contains(&990));
        assert!(TLS_PORTS.contains(&636));
        assert!(TLS_PORTS.contains(&3389));
        assert!(TLS_PORTS.contains(&8443));
    }
}
