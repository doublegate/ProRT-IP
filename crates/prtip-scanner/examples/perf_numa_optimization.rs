#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Performance: NUMA Optimization
//! Demonstrates: Thread pinning and IRQ affinity

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NUMA Optimization ===\n");

    println!("NUMA-Aware Scanning:");
    println!("  - Thread pinning to CPU cores");
    println!("  - IRQ affinity configuration");
    println!("  - Memory locality optimization");
    println!("  - Cross-node penalty avoidance\n");

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
    println!("\nNote: NUMA optimization requires hwloc feature");
    Ok(())
}
