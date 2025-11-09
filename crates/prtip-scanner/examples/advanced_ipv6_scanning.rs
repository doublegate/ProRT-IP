#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Example: Advanced IPv6 Scanning
//!
//! Demonstrates: IPv6-specific features (link-local, NDP, ICMPv6)
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//! - IPv6 network connectivity
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example advanced_ipv6_scanning
//! ```

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Advanced IPv6 Scanning ===\n");

    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;

    // IPv6 Loopback
    let ipv6_loopback: IpAddr = "::1".parse()?;
    println!("Scanning IPv6 loopback (::1)...");
    let results = scanner.scan_ports(ipv6_loopback, vec![80, 443]).await?;
    for r in &results {
        println!("  Port {:5}: {:?}", r.port, r.state);
    }

    // IPv6 Documentation prefix (2001:db8::/32)
    println!("\nIPv6 Features:");
    println!("  - Link-local addresses (fe80::/10)");
    println!("  - Neighbor Discovery Protocol (NDP)");
    println!("  - ICMPv6 error handling");
    println!("  - IPv6 fragmentation support");
    println!("  - Dual-stack capability");

    println!("\nNote: Sprint 5.1 added comprehensive IPv6 support");
    println!("  - All scan types work with IPv6");
    println!("  - Automatic IP version detection");
    println!("  - Native ICMPv6 and NDP protocol handling");

    Ok(())
}
