#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Template: Custom Scanner Implementation
//! TODO: Implement your own scanner type

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Scanner Template ===\n");
    println!("TODO: Extend this template to create your own scanner");
    println!("  1. Define scanner struct with config");
    println!("  2. Implement new() constructor");
    println!("  3. Implement scan_port() method");
    println!("  4. Add custom logic for your protocol\n");

    // Basic example using SynScanner
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
