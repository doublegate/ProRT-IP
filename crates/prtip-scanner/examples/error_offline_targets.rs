#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Error Handling: Offline Targets
//! Demonstrates: Timeout and unreachable host handling

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Offline Target Handling ===\n");
    println!("Timeout Strategies:");
    println!("  - Adaptive timeout adjustment");
    println!("  - Retry with exponential backoff");
    println!("  - Graceful failure handling");
    println!("  - Partial result preservation\n");

    let scanner = SynScanner::new(Config {
        scan: ScanConfig::default(),
        ..Config::default()
    })?;
    let results = scanner
        .scan_ports("192.0.2.1".parse::<IpAddr>()?, vec![80])
        .await?;
    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
