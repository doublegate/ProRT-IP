#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Template: Cloud Scanning (AWS/Azure)
//! TODO: Cloud-specific network scanning

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cloud Scanning Template ===\n");
    println!("TODO: Implement cloud scanning:");
    println!("  1. Enumerate VPC/VNET instances");
    println!("  2. Handle security groups/NSGs");
    println!("  3. Respect cloud provider rate limits");
    println!("  4. Tag resources in results\n");

    let scanner = SynScanner::new(Config {
        scan: ScanConfig::default(),
        ..Config::default()
    })?;
    let results = scanner
        .scan_ports("10.0.1.0".parse::<IpAddr>()?, vec![80])
        .await?;
    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
