#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: Dual-Stack IPv4/IPv6 Scanning
//!
//! Demonstrates: Scanning both IPv4 and IPv6 targets
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//! - IPv6 network connectivity
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_ipv4_ipv6_dual
//! ```
//!
//! ## Expected Output
//! - IPv4 and IPv6 scan results
//! - Dual-stack capability demonstration

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Dual-Stack IPv4/IPv6 Example ===\n");

    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;

    let ports = vec![80, 443];

    // IPv4 Scan
    println!("=== IPv4 Scan ===");
    let ipv4_target: IpAddr = "127.0.0.1".parse()?;
    println!("Target: {}", ipv4_target);

    let ipv4_results = scanner.scan_ports(ipv4_target, ports.clone()).await?;
    for result in &ipv4_results {
        println!("  Port {:5}: {:?}", result.port, result.state);
    }

    // IPv6 Scan
    println!("\n=== IPv6 Scan ===");
    let ipv6_target: IpAddr = "::1".parse()?; // IPv6 localhost
    println!("Target: {}", ipv6_target);

    let ipv6_results = scanner.scan_ports(ipv6_target, ports.clone()).await?;
    for result in &ipv6_results {
        println!("  Port {:5}: {:?}", result.port, result.state);
    }

    // Summary
    println!("\n{:=<60}", "");
    println!("Summary:");
    println!(
        "  IPv4 open ports: {}",
        ipv4_results
            .iter()
            .filter(|r| matches!(r.state, prtip_core::PortState::Open))
            .count()
    );
    println!(
        "  IPv6 open ports: {}",
        ipv6_results
            .iter()
            .filter(|r| matches!(r.state, prtip_core::PortState::Open))
            .count()
    );

    println!("\nNote: ProRT-IP supports dual-stack scanning:");
    println!("  - Automatic IP version detection");
    println!("  - Native IPv6 support (Sprint 5.1)");
    println!("  - ICMPv6 and NDP protocol handling");
    println!("  - Link-local address scanning");

    Ok(())
}
