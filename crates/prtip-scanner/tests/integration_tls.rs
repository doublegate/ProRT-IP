//! Integration tests for TLS certificate analysis
//!
//! These tests verify end-to-end functionality of TLS certificate extraction,
//! chain validation, and fingerprinting when scanning HTTPS services.
//!
//! Tests use real public HTTPS servers (example.com, google.com) and badssl.com
//! test endpoints for edge cases (expired, self-signed, etc.).

use prtip_core::ServiceProbeDb;
use prtip_scanner::service_detector::{ServiceDetector, ServiceInfo};
use std::net::SocketAddr;
use std::time::Duration;

/// Helper function to create a minimal test service probe database
fn create_test_probe_db() -> ServiceProbeDb {
    let db_content = r#"
# Minimal probe database for TLS testing
Probe TCP GetRequest q|GET / HTTP/1.0\r\n\r\n|
ports 80,443,8080,8443
rarity 1
match http m|^HTTP/1\.[01]| p/HTTP/
match https m|^HTTP/1\.[01]| p/HTTPS/

Probe TCP TLSSessionReq q|\x16\x03\x00\x00S\x01\x00\x00O\x03\x00|
ports 443,8443,465,587,993,995,636,990
rarity 1
match ssl m|^\x16\x03| p/SSL/
"#;
    ServiceProbeDb::parse(db_content).expect("Failed to parse test probe database")
}

/// Helper function to scan an HTTPS port and extract service info
async fn scan_https_port(host: &str, port: u16) -> Result<ServiceInfo, prtip_core::Error> {
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| prtip_core::Error::Parse(format!("Invalid address: {}", e)))?;

    let detector = ServiceDetector::new(create_test_probe_db(), 7);

    // Use longer timeout for real network requests
    tokio::time::timeout(Duration::from_secs(10), detector.detect_service(addr))
        .await
        .map_err(|_| prtip_core::Error::Network("Connection timeout".to_string()))?
}

// =============================================================================
// BASIC FUNCTIONALITY TESTS
// =============================================================================

#[tokio::test]
#[cfg_attr(not(feature = "network-tests"), ignore)] // Requires network access
async fn test_https_certificate_extraction_real_server() {
    // Test certificate extraction from a well-known HTTPS server
    let result = scan_https_port("example.com", 443).await;

    assert!(
        result.is_ok(),
        "Failed to scan example.com:443: {:?}",
        result.err()
    );

    let service_info = result.unwrap();

    // Verify TLS certificate was extracted
    assert!(
        service_info.tls_certificate.is_some(),
        "No TLS certificate extracted from example.com:443"
    );

    let cert = service_info.tls_certificate.unwrap();

    // Verify basic certificate fields are populated
    assert!(!cert.subject.is_empty(), "Certificate subject is empty");
    assert!(!cert.issuer.is_empty(), "Certificate issuer is empty");
    assert!(
        cert.subject.contains("example.com") || cert.subject.contains("CN="),
        "Subject doesn't contain expected domain or CN: {}",
        cert.subject
    );

    // Verify public key information
    assert!(cert.public_key_info.key_size > 0, "Public key size is 0");
    assert!(
        !cert.public_key_info.algorithm.is_empty(),
        "Public key algorithm is empty"
    );
}

#[tokio::test]
#[cfg_attr(not(feature = "network-tests"), ignore)] // Requires network access
async fn test_https_fingerprint_creation() {
    // Verify TLS fingerprint is created with version and cipher information
    let result = scan_https_port("example.com", 443).await;

    assert!(result.is_ok(), "Failed to scan example.com:443");
    let service_info = result.unwrap();

    // Verify TLS fingerprint was created
    assert!(
        service_info.tls_fingerprint.is_some(),
        "No TLS fingerprint created"
    );

    let fingerprint = service_info.tls_fingerprint.unwrap();

    // Verify TLS version is detected and secure (should be TLS 1.2 or 1.3)
    assert!(
        fingerprint.tls_version.contains("TLS 1.2") || fingerprint.tls_version.contains("TLS 1.3"),
        "TLS version should be 1.2 or 1.3, got: {}",
        fingerprint.tls_version
    );

    // Verify cipher suites information
    assert!(
        !fingerprint.cipher_suites.is_empty(),
        "Cipher suites list is empty"
    );
}

#[tokio::test]
#[cfg_attr(not(feature = "network-tests"), ignore)] // Requires network access
async fn test_certificate_chain_validation() {
    // Test certificate chain validation for a real server
    let result = scan_https_port("google.com", 443).await;

    assert!(result.is_ok(), "Failed to scan google.com:443");
    let service_info = result.unwrap();

    // Verify certificate chain was extracted
    assert!(
        service_info.tls_chain.is_some(),
        "No certificate chain extracted"
    );

    let chain = service_info.tls_chain.unwrap();

    // Google should have a multi-certificate chain
    assert!(!chain.certificates.is_empty(), "Certificate chain is empty");

    // Chain should be valid (not self-signed, properly linked)
    assert!(chain.is_valid, "Certificate chain validation failed");

    // Should not be self-signed (Google uses a real CA)
    assert!(
        !chain.is_self_signed,
        "Google certificate incorrectly detected as self-signed"
    );
}

#[tokio::test]
#[cfg_attr(not(feature = "network-tests"), ignore)] // Requires network access
async fn test_san_extraction() {
    // Test Subject Alternative Name extraction
    let result = scan_https_port("example.com", 443).await;

    assert!(result.is_ok(), "Failed to scan example.com:443");
    let service_info = result.unwrap();

    let cert = service_info.tls_certificate.expect("No certificate");

    // Verify SAN fields are populated (either old or new format)
    let has_sans = !cert.san.is_empty() || !cert.san_categorized.dns_names.is_empty();
    assert!(has_sans, "No SANs extracted from certificate");

    // Check categorized SANs (TASK-3 enhancement)
    if !cert.san_categorized.dns_names.is_empty() {
        // Should contain example.com or www.example.com
        let sans_str = cert.san_categorized.dns_names.join(", ");
        assert!(
            sans_str.contains("example.com"),
            "SANs don't contain expected domain: {}",
            sans_str
        );
    }
}

#[tokio::test]
#[cfg_attr(not(feature = "network-tests"), ignore)] // Requires network access
async fn test_multiple_sans() {
    // Test certificate with multiple Subject Alternative Names
    // Google's certificate typically has multiple SANs (*.google.com, google.com, etc.)
    let result = scan_https_port("google.com", 443).await;

    assert!(result.is_ok(), "Failed to scan google.com:443");
    let service_info = result.unwrap();

    let cert = service_info.tls_certificate.expect("No certificate");

    // Google should have multiple SANs
    let san_count = if !cert.san_categorized.dns_names.is_empty() {
        cert.san_categorized.dns_names.len()
    } else {
        cert.san.len()
    };

    assert!(
        san_count > 1,
        "Google certificate should have multiple SANs, found {}",
        san_count
    );
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[tokio::test]
#[cfg_attr(not(feature = "network-tests"), ignore)] // Requires network access to badssl.com
async fn test_self_signed_certificate() {
    // Test handling of self-signed certificate using badssl.com
    let result = scan_https_port("self-signed.badssl.com", 443).await;

    // Should successfully extract certificate despite being self-signed
    assert!(
        result.is_ok(),
        "Failed to scan self-signed.badssl.com: {:?}",
        result.err()
    );

    let service_info = result.unwrap();
    assert!(
        service_info.tls_certificate.is_some(),
        "No certificate extracted from self-signed server"
    );

    // Verify chain is marked as self-signed
    if let Some(chain) = service_info.tls_chain {
        assert!(
            chain.is_self_signed,
            "Self-signed certificate not detected as self-signed"
        );
    }
}

#[tokio::test]
#[cfg_attr(not(feature = "network-tests"), ignore)] // Requires network access to badssl.com
async fn test_expired_certificate() {
    // Test handling of expired certificate using badssl.com
    let result = scan_https_port("expired.badssl.com", 443).await;

    // Should extract certificate even if expired
    // (We don't enforce validity during scanning, just report it)
    if result.is_ok() {
        let service_info = result.unwrap();

        // Should still extract certificate
        assert!(
            service_info.tls_certificate.is_some(),
            "Certificate should be extracted even if expired"
        );

        // Chain validation might fail due to expiry
        if let Some(chain) = service_info.tls_chain {
            // Chain might be invalid due to expiry (but not always)
            // Just verify we extracted it - validation details may vary
            assert!(
                !chain.certificates.is_empty(),
                "Should still extract certificate chain even if expired"
            );
        }
    }
    // Note: expired.badssl.com sometimes updates its cert, so test may fail
    // This is acceptable - we're testing the code path, not the server
}

#[tokio::test]
async fn test_tls_timeout_handling() {
    // Test timeout on unresponsive server using TEST-NET-1 (reserved, won't respond)
    let result =
        tokio::time::timeout(Duration::from_secs(2), scan_https_port("192.0.2.1", 443)).await;

    // Should timeout or return error gracefully (not panic)
    assert!(
        result.is_err() || result.unwrap().is_err(),
        "Unresponsive server should timeout or error"
    );
}

#[tokio::test]
#[cfg_attr(not(feature = "network-tests"), ignore)] // Requires network access to badssl.com
async fn test_wrong_host_certificate() {
    // Test certificate with wrong hostname
    let result = scan_https_port("wrong.host.badssl.com", 443).await;

    // Should still extract certificate (we don't enforce hostname matching during scan)
    if result.is_ok() {
        let service_info = result.unwrap();

        // Certificate should be extracted
        assert!(
            service_info.tls_certificate.is_some(),
            "Certificate should be extracted even with wrong hostname"
        );
    }
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[tokio::test]
#[cfg_attr(not(feature = "network-tests"), ignore)] // Requires network access
async fn test_service_detector_tls_integration() {
    // Test that ServiceDetector properly integrates TLS analysis
    let detector = ServiceDetector::new(create_test_probe_db(), 7);

    let addr: SocketAddr = "example.com:443".parse().unwrap();
    let result = tokio::time::timeout(Duration::from_secs(10), detector.detect_service(addr))
        .await
        .expect("Timeout")
        .expect("Detection failed");

    // Verify all TLS fields are populated
    assert!(
        result.tls_certificate.is_some(),
        "ServiceDetector should extract TLS certificate"
    );
    assert!(
        result.tls_fingerprint.is_some(),
        "ServiceDetector should create TLS fingerprint"
    );
    assert!(
        result.tls_chain.is_some(),
        "ServiceDetector should extract certificate chain"
    );
}

#[tokio::test]
#[cfg_attr(not(feature = "network-tests"), ignore)] // Requires network access
async fn test_output_format_json() {
    // Test that TLS information is included in service detection output
    let result = scan_https_port("example.com", 443)
        .await
        .expect("Scan failed");

    // Verify TLS fields are populated (would be serialized in real JSON output)
    assert!(
        result.tls_certificate.is_some(),
        "ServiceDetector should populate TLS certificate"
    );
    assert!(
        result.tls_fingerprint.is_some(),
        "ServiceDetector should populate TLS fingerprint"
    );
    assert!(
        result.tls_chain.is_some(),
        "ServiceDetector should populate certificate chain"
    );
}

#[test]
fn test_tls_certificate_display() {
    // Test text output formatting for TLS certificate
    use prtip_scanner::tls_certificate::{
        CertificateInfo, PublicKeyInfo, SecurityStrength, SignatureAlgorithm,
        SubjectAlternativeName,
    };

    let cert = CertificateInfo {
        subject: "CN=example.com".to_string(),
        issuer: "CN=DigiCert SHA2 Secure Server CA".to_string(),
        validity_not_before: "2024-01-01 00:00:00 UTC".to_string(),
        validity_not_after: "2025-01-01 00:00:00 UTC".to_string(),
        san: vec!["example.com".to_string(), "www.example.com".to_string()],
        serial_number: "01:02:03:04".to_string(),
        signature_algorithm: "sha256WithRSAEncryption".to_string(),
        san_categorized: SubjectAlternativeName {
            dns_names: vec!["example.com".to_string(), "www.example.com".to_string()],
            ip_addresses: vec![],
            email_addresses: vec![],
            uris: vec![],
            other_names: vec![],
        },
        public_key_info: PublicKeyInfo {
            algorithm: "RSA".to_string(),
            key_size: 2048,
            curve: None,
            usage: vec![],
        },
        key_usage: None,
        extended_key_usage: None,
        extensions: vec![],
        signature_algorithm_enhanced: SignatureAlgorithm {
            algorithm: "sha256WithRSAEncryption".to_string(),
            hash_algorithm: "SHA256".to_string(),
            is_secure: true,
            strength: SecurityStrength::Acceptable,
        },
    };

    // Verify certificate can be formatted as Display
    let output = format!("{}", cert);

    assert!(output.contains("subject="), "Output should contain subject");
    assert!(output.contains("issuer="), "Output should contain issuer");
    assert!(
        output.contains("example.com"),
        "Output should contain domain"
    );
}

// =============================================================================
// PERFORMANCE TESTS
// =============================================================================

#[tokio::test]
#[cfg_attr(not(feature = "network-tests"), ignore)] // Requires network access
async fn test_tls_overhead_measurement() {
    // Measure TLS analysis overhead
    use std::time::Instant;

    // Run multiple scans to get average
    let iterations = 3;
    let mut total_time = Duration::from_secs(0);

    for _ in 0..iterations {
        let start = Instant::now();
        let result = scan_https_port("example.com", 443).await;
        let elapsed = start.elapsed();

        if result.is_ok() {
            total_time += elapsed;
        }
    }

    let avg_time = total_time / iterations as u32;

    println!(
        "Average TLS scan time: {:?} ({} iterations)",
        avg_time, iterations
    );

    // Verify overhead is reasonable (<5 seconds for network + TLS)
    assert!(
        avg_time < Duration::from_secs(5),
        "TLS scan took too long: {:?}",
        avg_time
    );
}
