#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Error Handling: Firewalled Hosts
//! Demonstrates: Filtered port interpretation

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Firewalled Host Handling ===\n");
    println!("Filtered Port Detection:");
    println!("  - No response vs RST interpretation");
    println!("  - ICMP unreachable analysis");
    println!("  - Stateful firewall detection");
    println!("  - Rate limit identification\n");

    let scanner = SynScanner::new(Config {
        scan: ScanConfig::default(),
        ..Config::default()
    })?;
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![80])
        .await?;
    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
