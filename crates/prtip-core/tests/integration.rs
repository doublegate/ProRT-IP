//! Integration tests for prtip-core

use prtip_core::*;
use std::net::IpAddr;

#[test]
fn test_config_roundtrip() {
    let config = Config::default();
    let toml_str = toml::to_string(&config).unwrap();
    let loaded = Config::load_from_str(&toml_str).unwrap();

    assert_eq!(config.scan.scan_type, loaded.scan.scan_type);
    assert_eq!(config.scan.timeout_ms, loaded.scan.timeout_ms);
}

#[test]
fn test_port_range_comprehensive() {
    let range = PortRange::parse("20-22,80,443,8000-8003").unwrap();
    let ports: Vec<u16> = range.iter().collect();
    assert_eq!(ports, vec![20, 21, 22, 80, 443, 8000, 8001, 8002, 8003]);
}

#[test]
fn test_scan_result_serialization() {
    let ip: IpAddr = "10.0.0.1".parse().unwrap();
    let result = ScanResult::new(ip, 443, PortState::Open)
        .with_service("https".to_string())
        .with_banner("nginx/1.18.0".to_string());

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: ScanResult = serde_json::from_str(&json).unwrap();

    assert_eq!(result.target_ip, deserialized.target_ip);
    assert_eq!(result.port, deserialized.port);
    assert_eq!(result.state, deserialized.state);
    assert_eq!(result.service, deserialized.service);
    assert_eq!(result.banner, deserialized.banner);
}

#[test]
fn test_scan_target_expansion() {
    let target = ScanTarget::parse("192.168.1.0/30").unwrap();
    let hosts = target.expand_hosts();
    assert_eq!(hosts.len(), 4); // /30 = 4 addresses
}

#[test]
fn test_timing_template_consistency() {
    // Ensure timing values are consistent
    assert!(TimingTemplate::Paranoid.timeout_ms() > TimingTemplate::Sneaky.timeout_ms());
    assert!(TimingTemplate::Sneaky.timeout_ms() > TimingTemplate::Polite.timeout_ms());
    assert!(TimingTemplate::Polite.timeout_ms() > TimingTemplate::Normal.timeout_ms());
    assert!(TimingTemplate::Normal.timeout_ms() > TimingTemplate::Aggressive.timeout_ms());
    assert!(TimingTemplate::Aggressive.timeout_ms() > TimingTemplate::Insane.timeout_ms());
}

#[test]
fn test_error_conversion_chain() {
    // Test io::Error conversion
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
    let err: Error = io_err.into();
    assert!(matches!(err, Error::Io(_)));

    // Test AddrParseError conversion
    let parse_result = "invalid".parse::<IpAddr>();
    assert!(parse_result.is_err());
    let err: Error = parse_result.unwrap_err().into();
    assert!(matches!(err, Error::Parse(_)));
}

#[test]
fn test_port_state_ordering_comprehensive() {
    use PortState::*;

    // Enum ordering: Open < Closed < Filtered < Unknown
    assert!(Open < Closed);
    assert!(Closed < Filtered);
    assert!(Filtered < Unknown);

    let mut states = vec![Unknown, Filtered, Open, Closed];
    states.sort();
    assert_eq!(states, vec![Open, Closed, Filtered, Unknown]);
}
