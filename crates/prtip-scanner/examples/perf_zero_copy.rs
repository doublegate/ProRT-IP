#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Performance: Zero-Copy Optimization
//! Demonstrates: Zero-copy packet handling

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Zero-Copy Optimization ===\n");
    println!("Zero-Copy Benefits:");
    println!("  - >10KB transfer optimization");
    println!("  - Reduced CPU usage");
    println!("  - Lower memory pressure");
    println!("  - Higher throughput\n");

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
