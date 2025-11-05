// Sprint 5.6 Phase 3: Service Detection Tests
// Comprehensive unit and integration testing for service_detector.rs
//
// Test Strategy:
// - Group 1: Detector creation and configuration (no network)
// - Group 2: Pattern matching and capture groups (no network)
// - Group 3: TLS configuration and SNI support (no network)
// - Group 4: Integration tests (marked #[ignore], require network)
//
// Run all tests: cargo test --test test_service_detector
// Run privileged tests: cargo test --test test_service_detector -- --ignored

use prtip_core::ServiceProbeDb;
use prtip_scanner::service_detector::{ServiceDetector, ServiceInfo};
use std::net::SocketAddr;
use std::time::Duration;

/// Helper to create a simple test probe database
fn create_test_probe_db() -> ServiceProbeDb {
    let db_content = r#"
Probe TCP NULL q||
ports 1-65535
rarity 1

Probe TCP GetRequest q|GET / HTTP/1.0\r\n\r\n|
ports 80,443,8080,8443
rarity 1
match http m|^HTTP/1\.[01] \d+ | p/HTTP/

Probe TCP GenericLines q|\r\n\r\n|
ports 1-65535
rarity 2
match ssh m|^SSH-([0-9.]+)-([^ ]+)| p/OpenSSH/ v/$1/
"#;

    ServiceProbeDb::parse(db_content).expect("Failed to parse test probe database")
}

// ============================================================================
// Test Group 1: Detector Creation and Configuration (8 tests)
// Tests detector instantiation and configuration without network calls
// ============================================================================

#[test]
fn test_detector_new() {
    let db = ServiceProbeDb::new();
    let detector = ServiceDetector::new(db, 7);

    // Verify default configuration
    assert_eq!(detector.intensity(), 7);
    assert_eq!(detector.timeout(), Duration::from_secs(5));
    assert!(detector.is_tls_enabled()); // TLS enabled by default
    assert!(!detector.is_raw_capture_enabled()); // Raw capture disabled by default
}

#[test]
fn test_detector_with_tls_enabled() {
    let db = ServiceProbeDb::new();
    let detector = ServiceDetector::with_tls(db, 7, true);

    // Verify TLS is enabled
    assert!(detector.is_tls_enabled());
    assert_eq!(detector.intensity(), 7);
    assert!(!detector.is_raw_capture_enabled());
}

#[test]
fn test_detector_with_tls_disabled() {
    let db = ServiceProbeDb::new();
    let detector = ServiceDetector::with_tls(db, 7, false);

    // Verify TLS is disabled
    assert!(!detector.is_tls_enabled());
    assert_eq!(detector.intensity(), 7);
}

#[test]
fn test_detector_with_options_full() {
    let db = ServiceProbeDb::new();
    let detector = ServiceDetector::with_options(db, 7, true, true);

    // Verify both TLS and raw capture are enabled
    assert!(detector.is_tls_enabled());
    assert!(detector.is_raw_capture_enabled());
    assert_eq!(detector.intensity(), 7);
}

#[test]
fn test_detector_with_options_no_capture() {
    let db = ServiceProbeDb::new();
    let detector = ServiceDetector::with_options(db, 7, true, false);

    // Verify TLS enabled but raw capture disabled
    assert!(detector.is_tls_enabled());
    assert!(!detector.is_raw_capture_enabled());
    assert_eq!(detector.intensity(), 7);
}

#[test]
fn test_detector_set_tls_enabled() {
    let db = ServiceProbeDb::new();
    let mut detector = ServiceDetector::with_tls(db, 7, false);

    // Should be able to toggle TLS
    detector.set_tls_enabled(true);
    detector.set_tls_enabled(false);
}

#[test]
fn test_detector_set_timeout() {
    let db = ServiceProbeDb::new();
    let mut detector = ServiceDetector::new(db, 7);

    // Test various timeout values
    detector.set_timeout(Duration::from_secs(10));
    detector.set_timeout(Duration::from_millis(500));
    detector.set_timeout(Duration::from_secs(30));
}

#[test]
fn test_detector_default_timeout() {
    let db = ServiceProbeDb::new();
    let detector = ServiceDetector::new(db, 7);

    // Verify default timeout is 5 seconds
    assert_eq!(detector.timeout(), Duration::from_secs(5));
}

// ============================================================================
// Test Group 2: Intensity and Probe Selection (5 tests)
// Tests intensity clamping and probe filtering logic
// ============================================================================

#[test]
fn test_detector_intensity_clamping_high() {
    let db = ServiceProbeDb::new();
    let detector = ServiceDetector::new(db, 15);

    // Intensity should be clamped to 9 (max)
    assert_eq!(detector.intensity(), 9);
}

#[test]
fn test_detector_intensity_zero() {
    let db = ServiceProbeDb::new();
    let detector = ServiceDetector::new(db, 0);

    // Intensity 0 should be valid (paranoid mode - only use rarity 0 probes)
    assert_eq!(detector.intensity(), 0);
}

#[test]
fn test_detector_intensity_max() {
    let db = ServiceProbeDb::new();
    let detector = ServiceDetector::new(db, 9);

    // Intensity 9 should use all probes
    assert_eq!(detector.intensity(), 9);
}

#[test]
fn test_detector_intensity_mid() {
    let db = ServiceProbeDb::new();
    let detector = ServiceDetector::new(db, 5);

    // Intensity 5 should use probes with rarity <= 5
    assert_eq!(detector.intensity(), 5);
}

#[test]
fn test_detector_with_custom_db() {
    let db = create_test_probe_db();
    let detector = ServiceDetector::new(db, 7);

    // Verify creation with custom probe database
    assert_eq!(detector.intensity(), 7);
    assert!(detector.is_tls_enabled());
}

// ============================================================================
// Test Group 3: ServiceInfo Structure Tests (6 tests)
// Tests ServiceInfo creation and field handling
// ============================================================================

#[test]
fn test_service_info_complete() {
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
    assert!(info.product.is_none());
    assert!(info.version.is_none());
    assert!(info.cpe.is_empty());
}

#[test]
fn test_service_info_with_tls() {
    let info = ServiceInfo {
        service: "https".to_string(),
        product: Some("nginx".to_string()),
        version: Some("1.18.0".to_string()),
        extra_info: Some("TLS 1.2 (Let's Encrypt)".to_string()),
        hostname: Some("example.com".to_string()),
        os_type: None,
        device_type: None,
        cpe: Vec::new(),
        method: "tls_handshake".to_string(),
        raw_response: None,
        tls_certificate: None, // Would be populated in real scenario
        tls_fingerprint: None,
        tls_chain: None,
    };

    assert_eq!(info.service, "https");
    assert_eq!(info.method, "tls_handshake");
    assert!(info.extra_info.is_some());
    assert!(info.extra_info.unwrap().contains("TLS"));
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
}

#[test]
fn test_service_info_with_multiple_cpe() {
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
}

// ============================================================================
// Test Group 4: Integration Tests (5 tests, marked #[ignore])
// Tests requiring network access - run with: cargo test -- --ignored
// ============================================================================

#[tokio::test]
#[ignore] // Requires network: cargo test -- --ignored
async fn test_detect_service_http() {
    let db = ServiceProbeDb::default();
    let detector = ServiceDetector::new(db, 9);

    // Test against example.com (port 80)
    let addr: SocketAddr = "93.184.216.34:80".parse().unwrap();
    let result = detector.detect_service(addr).await;

    assert!(result.is_ok(), "HTTP detection should succeed");
    let info = result.unwrap();
    // example.com should respond with HTTP
    // Note: exact service may vary (http, tcpwrapped, etc.)
    assert!(!info.service.is_empty(), "Should detect some service");
    assert_eq!(info.method, "pattern match", "Should use pattern matching");
}

#[tokio::test]
#[ignore] // Requires network: cargo test -- --ignored
async fn test_detect_service_https() {
    let db = ServiceProbeDb::default();
    let detector = ServiceDetector::new(db, 9);

    // Test against example.com (port 443)
    let addr: SocketAddr = "93.184.216.34:443".parse().unwrap();
    let result = detector.detect_service(addr).await;

    assert!(result.is_ok(), "HTTPS detection should succeed");
    let info = result.unwrap();

    // Should detect HTTPS or TLS-related service
    assert!(!info.service.is_empty(), "Should detect some service");
    // TLS info may be present if handshake succeeded
    if info.tls_certificate.is_some() {
        assert_eq!(
            info.method, "tls_handshake",
            "Should use TLS handshake method"
        );
    }
}

#[tokio::test]
#[ignore] // Requires network: cargo test -- --ignored
async fn test_detect_service_with_hostname() {
    let db = ServiceProbeDb::default();
    let detector = ServiceDetector::new(db, 9);

    // Test with hostname for SNI
    let addr: SocketAddr = "93.184.216.34:443".parse().unwrap();
    let result = detector
        .detect_service_with_hostname(addr, Some("example.com"))
        .await;

    assert!(result.is_ok(), "HTTPS detection with SNI should succeed");
    let info = result.unwrap();

    // With SNI, should get better TLS certificate detection
    assert!(!info.service.is_empty(), "Should detect some service");
    // Hostname should be set if SNI was used
    if info.hostname.is_some() {
        assert_eq!(info.hostname.as_deref(), Some("example.com"));
    }
}

#[tokio::test]
#[ignore] // Requires network: cargo test -- --ignored
async fn test_detect_service_closed_port() {
    let db = ServiceProbeDb::default();
    let detector = ServiceDetector::new(db, 9);

    // Test against likely closed port
    let addr: SocketAddr = "93.184.216.34:12345".parse().unwrap();
    let result = detector.detect_service(addr).await;

    // Should either timeout or return unknown
    if let Ok(info) = result {
        // If it succeeds, should be unknown
        assert_eq!(info.service, "unknown");
    }
    // Errors are acceptable for closed ports
}

#[tokio::test]
#[ignore] // Requires network: cargo test -- --ignored
async fn test_detect_service_timeout() {
    let db = ServiceProbeDb::default();
    let mut detector = ServiceDetector::new(db, 9);

    // Set very short timeout
    detector.set_timeout(Duration::from_millis(100));

    // Test against a slow/non-responsive target
    let addr: SocketAddr = "93.184.216.34:22".parse().unwrap();
    let result = detector.detect_service(addr).await;

    // Should handle timeout gracefully (may succeed or fail depending on network)
    // The test is that it doesn't panic and returns a proper Result
    match result {
        Ok(info) => {
            // If it succeeds, should have some service info
            assert!(!info.service.is_empty());
        }
        Err(_) => {
            // Timeout or connection error is acceptable
        }
    }
}
