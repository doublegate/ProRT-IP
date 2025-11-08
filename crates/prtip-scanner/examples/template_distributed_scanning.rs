#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Template: Distributed Scanning
//! TODO: Coordinate multi-node scanning

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Distributed Scanning Template ===\n");
    println!("TODO: Implement distributed coordination:");
    println!("  1. Split target list across nodes");
    println!("  2. Synchronize scan progress");
    println!("  3. Aggregate results");
    println!("  4. Handle node failures\n");

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
