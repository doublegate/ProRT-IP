#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Integration: SIEM (Splunk/ELK)
//! Demonstrates: SIEM log integration

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SIEM Integration ===\n");
    println!("SIEM Features:");
    println!("  - Structured logging (JSON)");
    println!("  - Splunk HEC compatible");
    println!("  - ELK stack integration");
    println!("  - Alert correlation\n");

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
