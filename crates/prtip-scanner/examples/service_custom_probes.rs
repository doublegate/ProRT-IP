#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Service Detection: Custom Protocol Probes
//! Demonstrates: Protocol-specific payloads for service detection

use prtip_core::{Config, ScanConfig};
use prtip_scanner::UdpScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Protocol Probes ===\n");

    println!("Protocol-Specific Probes:");
    println!("  - DNS: Query for version.bind");
    println!("  - SNMP: GetRequest for sysDescr");
    println!("  - NTP: Version query");
    println!("  - NetBIOS: Name query\n");

    let scanner = UdpScanner::new(Config {
        scan: ScanConfig::default(),
        ..Config::default()
    })?;
    let result = scanner
        .scan_port("127.0.0.1".parse::<IpAddr>()?, 53)
        .await?;

    println!("DNS Port: {:?}", result.state);
    Ok(())
}
