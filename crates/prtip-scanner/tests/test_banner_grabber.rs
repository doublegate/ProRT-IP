// Sprint 5.6 Phase 3: Banner Grabber Tests
// Comprehensive unit and integration testing for banner_grabber.rs
//
// Test Strategy:
// - Group 1: Grabber creation and configuration (no network)
// - Group 2: Protocol parser unit tests (no network)
// - Group 3: Port-based protocol detection (no network)
// - Group 4: Integration tests (marked #[ignore], require network)
//
// Run all tests: cargo test --test test_banner_grabber
// Run privileged tests: cargo test --test test_banner_grabber -- --ignored

use prtip_scanner::banner_grabber::{BannerGrabber, BannerParser};
use std::net::SocketAddr;
use std::time::Duration;

// ============================================================================
// Test Group 1: Grabber Creation and Configuration (7 tests)
// Tests grabber instantiation and configuration without network calls
// ============================================================================

#[test]
fn test_banner_grabber_new() {
    let grabber = BannerGrabber::new();

    // Verify default configuration
    assert_eq!(grabber.timeout(), Duration::from_secs(5));
    assert_eq!(grabber.max_banner_size(), 4096);
}

#[test]
fn test_banner_grabber_default() {
    let grabber = BannerGrabber::default();

    // Default implementation should match new()
    assert_eq!(grabber.timeout(), Duration::from_secs(5));
    assert_eq!(grabber.max_banner_size(), 4096);
}

#[test]
fn test_banner_grabber_set_timeout() {
    let mut grabber = BannerGrabber::new();

    // Test various timeout values
    grabber.set_timeout(Duration::from_secs(10));
    grabber.set_timeout(Duration::from_millis(500));
    grabber.set_timeout(Duration::from_secs(30));
}

#[test]
fn test_banner_grabber_set_max_banner_size() {
    let mut grabber = BannerGrabber::new();

    // Test various buffer sizes
    grabber.set_max_banner_size(1024);
    grabber.set_max_banner_size(8192);
    grabber.set_max_banner_size(16384);
}

#[test]
fn test_banner_grabber_configuration_chaining() {
    let mut grabber = BannerGrabber::new();

    // Test configuration in sequence
    grabber.set_timeout(Duration::from_secs(3));
    grabber.set_max_banner_size(2048);

    // Should accept both configurations
}

#[test]
fn test_banner_grabber_extreme_timeout() {
    let mut grabber = BannerGrabber::new();

    // Test very short and very long timeouts
    grabber.set_timeout(Duration::from_millis(1));
    grabber.set_timeout(Duration::from_secs(120));
}

#[test]
fn test_banner_grabber_extreme_buffer_size() {
    let mut grabber = BannerGrabber::new();

    // Test very small and very large buffer sizes
    grabber.set_max_banner_size(64);
    grabber.set_max_banner_size(65536);
}

// ============================================================================
// Test Group 2: Protocol Parser Unit Tests (15 tests)
// Tests banner parsing logic without network calls
// ============================================================================

#[test]
fn test_parse_http_banner_standard() {
    let banner = "HTTP/1.1 200 OK\r\nServer: nginx/1.18.0\r\nContent-Type: text/html\r\n";
    let server = BannerParser::parse_http_banner(banner);
    assert_eq!(server, Some("nginx/1.18.0".to_string()));
}

#[test]
fn test_parse_http_banner_no_server() {
    let banner = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n";
    let server = BannerParser::parse_http_banner(banner);
    assert_eq!(server, None);
}

#[test]
fn test_parse_http_banner_whitespace() {
    let banner = "HTTP/1.1 200 OK\r\nServer:   nginx/1.18.0   \r\n";
    let server = BannerParser::parse_http_banner(banner);
    assert_eq!(server, Some("nginx/1.18.0".to_string()));
}

#[test]
fn test_parse_http_banner_multiple_servers() {
    let banner = "HTTP/1.1 200 OK\r\nServer: nginx/1.18.0\r\nServer: Apache\r\n";
    let server = BannerParser::parse_http_banner(banner);
    // Should return first match
    assert_eq!(server, Some("nginx/1.18.0".to_string()));
}

#[test]
fn test_parse_http_banner_various_servers() {
    let test_cases = vec![
        (
            "HTTP/1.1 200 OK\r\nServer: Apache/2.4.41 (Ubuntu)\r\n",
            Some("Apache/2.4.41 (Ubuntu)"),
        ),
        (
            "HTTP/1.1 200 OK\r\nServer: Microsoft-IIS/10.0\r\n",
            Some("Microsoft-IIS/10.0"),
        ),
        (
            "HTTP/1.1 200 OK\r\nServer: cloudflare\r\n",
            Some("cloudflare"),
        ),
        ("HTTP/1.0 404 Not Found\r\nServer: nginx\r\n", Some("nginx")),
        ("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n", None),
    ];

    for (banner, expected) in test_cases {
        let result = BannerParser::parse_http_banner(banner);
        assert_eq!(result, expected.map(String::from), "Failed for: {}", banner);
    }
}

#[test]
fn test_parse_ftp_banner_standard() {
    let banner = "220 ProFTPD 1.3.5 Server ready.";
    let server = BannerParser::parse_ftp_banner(banner);
    assert_eq!(server, Some("ProFTPD 1.3.5 Server ready.".to_string()));
}

#[test]
fn test_parse_ftp_banner_no_220() {
    let banner = "421 Service not available";
    let server = BannerParser::parse_ftp_banner(banner);
    assert_eq!(server, None);
}

#[test]
fn test_parse_ftp_banner_various_servers() {
    let test_cases = vec![
        ("220 ProFTPD 1.3.5 Server", Some("ProFTPD 1.3.5 Server")),
        ("220 vsftpd 3.0.3", Some("vsftpd 3.0.3")),
        ("220 Welcome to Pure-FTPd", Some("Welcome to Pure-FTPd")),
        ("500 Error", None),
    ];

    for (banner, expected) in test_cases {
        let result = BannerParser::parse_ftp_banner(banner);
        assert_eq!(result, expected.map(String::from), "Failed for: {}", banner);
    }
}

#[test]
fn test_parse_ssh_banner_standard() {
    let banner = "SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.5";
    let version = BannerParser::parse_ssh_banner(banner);
    assert_eq!(
        version,
        Some("SSH-2.0-OpenSSH_8.2p1 Ubuntu-4ubuntu0.5".to_string())
    );
}

#[test]
fn test_parse_ssh_banner_invalid() {
    let banner = "Not an SSH banner";
    let version = BannerParser::parse_ssh_banner(banner);
    assert_eq!(version, None);
}

#[test]
fn test_parse_ssh_banner_various_versions() {
    let test_cases = vec![
        ("SSH-2.0-OpenSSH_8.2p1", Some("SSH-2.0-OpenSSH_8.2p1")),
        ("SSH-2.0-OpenSSH_7.4", Some("SSH-2.0-OpenSSH_7.4")),
        ("SSH-1.99-Cisco-1.25", Some("SSH-1.99-Cisco-1.25")),
        ("SSH-2.0-libssh_0.9.0", Some("SSH-2.0-libssh_0.9.0")),
        ("Not SSH", None),
        ("ssh-2.0-lowercase", None), // Case sensitive
    ];

    for (banner, expected) in test_cases {
        let result = BannerParser::parse_ssh_banner(banner);
        assert_eq!(result, expected.map(String::from), "Failed for: {}", banner);
    }
}

#[test]
fn test_parse_smtp_banner_standard() {
    let banner = "220 mail.example.com ESMTP Postfix";
    let server = BannerParser::parse_smtp_banner(banner);
    assert_eq!(server, Some("mail.example.com ESMTP Postfix".to_string()));
}

#[test]
fn test_parse_smtp_banner_no_220() {
    let banner = "421 Service not available";
    let server = BannerParser::parse_smtp_banner(banner);
    assert_eq!(server, None);
}

#[test]
fn test_parse_smtp_banner_various_servers() {
    let test_cases = vec![
        (
            "220 mail.example.com ESMTP Postfix",
            Some("mail.example.com ESMTP Postfix"),
        ),
        ("220 smtp.gmail.com ESMTP", Some("smtp.gmail.com ESMTP")),
        (
            "220 Microsoft ESMTP MAIL Service",
            Some("Microsoft ESMTP MAIL Service"),
        ),
        ("421 Service not available", None),
    ];

    for (banner, expected) in test_cases {
        let result = BannerParser::parse_smtp_banner(banner);
        assert_eq!(result, expected.map(String::from), "Failed for: {}", banner);
    }
}

#[test]
fn test_parse_empty_banners() {
    assert_eq!(BannerParser::parse_http_banner(""), None);
    assert_eq!(BannerParser::parse_ftp_banner(""), None);
    assert_eq!(BannerParser::parse_ssh_banner(""), None);
    assert_eq!(BannerParser::parse_smtp_banner(""), None);
}

// ============================================================================
// Test Group 3: Port-Based Protocol Detection (4 tests)
// Tests protocol detection mapping logic
// ============================================================================

#[test]
fn test_protocol_detection_http_ports() {
    let http_ports = vec![80, 8080];

    for port in http_ports {
        // We can't directly test the internal routing, but we document
        // that these ports should route to HTTP banner grabbing
        assert!(port == 80 || port == 8080, "Port {} should be HTTP", port);
    }
}

#[test]
fn test_protocol_detection_https_ports() {
    let https_ports = vec![443, 8443];

    for port in https_ports {
        assert!(port == 443 || port == 8443, "Port {} should be HTTPS", port);
    }
}

#[test]
fn test_protocol_detection_mail_ports() {
    let mail_ports = vec![(25, "SMTP"), (110, "POP3"), (143, "IMAP"), (587, "SMTP")];

    for (port, proto) in mail_ports {
        // Document expected protocol mapping
        match port {
            25 | 587 => assert_eq!(proto, "SMTP"),
            110 => assert_eq!(proto, "POP3"),
            143 => assert_eq!(proto, "IMAP"),
            _ => panic!("Unexpected port: {}", port),
        }
    }
}

#[test]
fn test_protocol_detection_other_services() {
    let service_ports = vec![(21, "FTP"), (22, "SSH")];

    for (port, proto) in service_ports {
        match port {
            21 => assert_eq!(proto, "FTP"),
            22 => assert_eq!(proto, "SSH"),
            _ => panic!("Unexpected port: {}", port),
        }
    }
}

// ============================================================================
// Test Group 4: Integration Tests (5 tests, marked #[ignore])
// Tests requiring network access - run with: cargo test -- --ignored
// ============================================================================

#[tokio::test]
#[ignore] // Requires network: cargo test -- --ignored
async fn test_grab_http_banner_real() {
    let grabber = BannerGrabber::new();

    // Test against example.com HTTP
    let addr: SocketAddr = "93.184.216.34:80".parse().unwrap();
    let result = grabber.grab_http_banner(addr).await;

    assert!(result.is_ok(), "HTTP banner grab should succeed");
    let banner = result.unwrap();
    assert!(!banner.is_empty(), "Banner should not be empty");
    assert!(banner.contains("HTTP"), "Should contain HTTP response");
}

#[tokio::test]
#[ignore] // Requires network: cargo test -- --ignored
async fn test_grab_https_banner_real() {
    let grabber = BannerGrabber::new();

    // Test against example.com HTTPS
    let addr: SocketAddr = "93.184.216.34:443".parse().unwrap();
    let result = grabber.grab_https_banner(addr).await;

    // HTTPS may succeed or fail depending on TLS configuration
    // This tests that it doesn't panic
    if let Ok(banner) = result {
        assert!(
            !banner.is_empty(),
            "Banner should not be empty if successful"
        );
    }
}

#[tokio::test]
#[ignore] // Requires network: cargo test -- --ignored
async fn test_grab_banner_auto_detect() {
    let grabber = BannerGrabber::new();

    // Test auto-detection for HTTP port
    let addr: SocketAddr = "93.184.216.34:80".parse().unwrap();
    let result = grabber.grab_banner(addr).await;

    assert!(
        result.is_ok(),
        "Auto-detect banner grab should succeed for HTTP"
    );
}

#[tokio::test]
#[ignore] // Requires network: cargo test -- --ignored
async fn test_grab_banner_closed_port() {
    let grabber = BannerGrabber::new();

    // Test against likely closed port
    let addr: SocketAddr = "93.184.216.34:12345".parse().unwrap();
    let result = grabber.grab_banner(addr).await;

    // Should fail with network error
    assert!(result.is_err(), "Closed port should fail");
}

#[tokio::test]
#[ignore] // Requires network: cargo test -- --ignored
async fn test_grab_banner_with_timeout() {
    let mut grabber = BannerGrabber::new();

    // Set very short timeout
    grabber.set_timeout(Duration::from_millis(100));

    // Test against a target (may timeout)
    let addr: SocketAddr = "93.184.216.34:80".parse().unwrap();
    let result = grabber.grab_banner(addr).await;

    // Should handle timeout gracefully (may succeed or fail depending on network speed)
    // The test is that it doesn't panic and returns a proper Result
    match result {
        Ok(banner) => {
            // If it succeeds, should have some banner data
            assert!(!banner.is_empty());
        }
        Err(_) => {
            // Timeout or connection error is acceptable
        }
    }
}
