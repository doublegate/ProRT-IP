#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Template: Custom Output Formatter
//! TODO: Create custom output format

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Output Formatter Template ===\n");
    println!("TODO: Implement custom formatting:");
    println!("  1. Define output structure (CSV, JSON, XML, etc.)");
    println!("  2. Format scan results");
    println!("  3. Add metadata (timestamp, scanner version)");
    println!("  4. Write to file or stdout\n");

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
