#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Service Detection: OS Fingerprinting
//! Demonstrates: Operating system detection via TCP/IP stack analysis

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OS Fingerprinting ===\n");

    println!("OS Detection Techniques:");
    println!("  - TCP window size analysis");
    println!("  - IP TTL default values");
    println!("  - TCP options patterns");
    println!("  - ICMP response behavior\n");

    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![80])
        .await?;

    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
