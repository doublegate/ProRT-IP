// Sprint 4.20 Phase 5: Source Port Manipulation Tests
// Tests for source port configuration in all scanner types

use prtip_core::config::Config;
use prtip_scanner::{StealthScanner, SynScanner, UdpScanner};

/// Helper to create config with specific source port
fn config_with_source_port(port: Option<u16>) -> Config {
    let mut config = Config::default();
    config.network.source_port = port;
    config
}

// ============================================================================
// Test Group 1: Scanner Creation with Config (5 tests)
// Verify scanners can be created with source_port configuration
// ============================================================================

#[test]
fn test_syn_scanner_creation_with_configured_port() {
    let config = config_with_source_port(Some(5353));
    let scanner = SynScanner::new(config);
    assert!(
        scanner.is_ok(),
        "SYN scanner should create successfully with source port 5353"
    );
}

#[test]
fn test_udp_scanner_creation_with_configured_port() {
    let config = config_with_source_port(Some(5353));
    let scanner = UdpScanner::new(config);
    assert!(
        scanner.is_ok(),
        "UDP scanner should create successfully with source port 5353"
    );
}

#[test]
fn test_stealth_scanner_creation_with_configured_port() {
    let config = config_with_source_port(Some(5353));
    let scanner = StealthScanner::new(config);
    assert!(
        scanner.is_ok(),
        "Stealth scanner should create successfully with source port 5353"
    );
}

#[test]
fn test_syn_scanner_creation_with_dns_port() {
    let config = config_with_source_port(Some(53));
    let scanner = SynScanner::new(config);
    assert!(
        scanner.is_ok(),
        "SYN scanner should create successfully with DNS port 53"
    );
}

#[test]
fn test_udp_scanner_creation_with_dns_port() {
    let config = config_with_source_port(Some(53));
    let scanner = UdpScanner::new(config);
    assert!(
        scanner.is_ok(),
        "UDP scanner should create successfully with DNS port 53"
    );
}

// ============================================================================
// Test Group 2: Random Port Fallback (5 tests)
// Verify scanners work when source_port is None (default behavior)
// ============================================================================

#[test]
fn test_syn_scanner_creation_without_source_port() {
    let config = config_with_source_port(None);
    let scanner = SynScanner::new(config);
    assert!(
        scanner.is_ok(),
        "SYN scanner should create successfully without source port"
    );
}

#[test]
fn test_udp_scanner_creation_without_source_port() {
    let config = config_with_source_port(None);
    let scanner = UdpScanner::new(config);
    assert!(
        scanner.is_ok(),
        "UDP scanner should create successfully without source port"
    );
}

#[test]
fn test_stealth_scanner_creation_without_source_port() {
    let config = config_with_source_port(None);
    let scanner = StealthScanner::new(config);
    assert!(
        scanner.is_ok(),
        "Stealth scanner should create successfully without source port"
    );
}

#[test]
fn test_default_config_source_port_is_none() {
    let config = Config::default();
    assert_eq!(
        config.network.source_port, None,
        "Default config should have source_port = None"
    );
}

#[test]
fn test_scanner_creation_with_default_config() {
    let config = Config::default();
    let scanner = SynScanner::new(config);
    assert!(
        scanner.is_ok(),
        "Scanner should work with default config (no source port)"
    );
}

// ============================================================================
// Test Group 3: Edge Cases (4 tests)
// Verify boundary conditions for port numbers
// ============================================================================

#[test]
fn test_source_port_minimum_1() {
    let config = config_with_source_port(Some(1));
    let scanner = SynScanner::new(config);
    assert!(scanner.is_ok(), "Scanner should accept minimum port 1");
}

#[test]
fn test_source_port_maximum_65535() {
    let config = config_with_source_port(Some(65535));
    let scanner = SynScanner::new(config);
    assert!(scanner.is_ok(), "Scanner should accept maximum port 65535");
}

#[test]
fn test_source_port_1024_boundary() {
    let config = config_with_source_port(Some(1024));
    let scanner = SynScanner::new(config);
    assert!(
        scanner.is_ok(),
        "Scanner should accept port 1024 (ephemeral boundary)"
    );
}

#[test]
fn test_source_port_1023_privileged() {
    let config = config_with_source_port(Some(1023));
    let scanner = SynScanner::new(config);
    assert!(
        scanner.is_ok(),
        "Scanner should accept port 1023 (privileged range)"
    );
}

// ============================================================================
// Test Group 4: Common Evasion Ports (6 tests)
// Verify well-known ports used for firewall evasion
// ============================================================================

#[test]
fn test_source_port_53_dns_evasion() {
    let config = config_with_source_port(Some(53));
    let syn_scanner = SynScanner::new(config.clone());
    let udp_scanner = UdpScanner::new(config.clone());
    let stealth_scanner = StealthScanner::new(config);

    assert!(syn_scanner.is_ok(), "SYN scanner should accept DNS port 53");
    assert!(udp_scanner.is_ok(), "UDP scanner should accept DNS port 53");
    assert!(
        stealth_scanner.is_ok(),
        "Stealth scanner should accept DNS port 53"
    );
}

#[test]
fn test_source_port_20_ftp_data_evasion() {
    let config = config_with_source_port(Some(20));
    let scanner = UdpScanner::new(config);
    assert!(scanner.is_ok(), "Scanner should accept FTP-DATA port 20");
}

#[test]
fn test_source_port_80_http_evasion() {
    let config = config_with_source_port(Some(80));
    let scanner = StealthScanner::new(config);
    assert!(scanner.is_ok(), "Scanner should accept HTTP port 80");
}

#[test]
fn test_source_port_88_kerberos_evasion() {
    let config = config_with_source_port(Some(88));
    let scanner = StealthScanner::new(config);
    assert!(scanner.is_ok(), "Scanner should accept Kerberos port 88");
}

#[test]
fn test_source_port_443_https_evasion() {
    let config = config_with_source_port(Some(443));
    let scanner = SynScanner::new(config);
    assert!(scanner.is_ok(), "Scanner should accept HTTPS port 443");
}

#[test]
fn test_source_port_123_ntp_evasion() {
    let config = config_with_source_port(Some(123));
    let scanner = UdpScanner::new(config);
    assert!(scanner.is_ok(), "Scanner should accept NTP port 123");
}

// ============================================================================
// Test Group 5: Config Threading Verification (4 tests)
// Verify config is properly stored and threaded through scanner creation
// ============================================================================

#[test]
fn test_config_source_port_assignment() {
    let mut config = Config::default();
    assert_eq!(config.network.source_port, None);

    config.network.source_port = Some(53);
    assert_eq!(config.network.source_port, Some(53));
}

#[test]
fn test_multiple_scanners_different_ports() {
    let config1 = config_with_source_port(Some(53));
    let config2 = config_with_source_port(Some(80));

    let scanner1 = SynScanner::new(config1);
    let scanner2 = SynScanner::new(config2);

    assert!(scanner1.is_ok(), "First scanner (port 53) should succeed");
    assert!(scanner2.is_ok(), "Second scanner (port 80) should succeed");
}

#[test]
fn test_scanner_types_same_port() {
    let config = config_with_source_port(Some(53));

    let syn_scanner = SynScanner::new(config.clone());
    let udp_scanner = UdpScanner::new(config.clone());
    let stealth_scanner = StealthScanner::new(config);

    assert!(syn_scanner.is_ok(), "SYN scanner should succeed");
    assert!(udp_scanner.is_ok(), "UDP scanner should succeed");
    assert!(stealth_scanner.is_ok(), "Stealth scanner should succeed");
}

#[test]
fn test_config_clone_preserves_source_port() {
    let config1 = config_with_source_port(Some(53));
    let config2 = config1.clone();

    assert_eq!(config1.network.source_port, Some(53));
    assert_eq!(config2.network.source_port, Some(53));
}
