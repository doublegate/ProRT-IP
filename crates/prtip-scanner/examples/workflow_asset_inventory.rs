#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Workflow: Periodic Asset Inventory
use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Asset Inventory Workflow ===\n");
    println!("Periodic scanning, baseline comparison, change detection");

    let scanner = SynScanner::new(Config {
        scan: ScanConfig::default(),
        ..Config::default()
    })?;
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![80, 443])
        .await?;

    println!("\nInventory snapshot:");
    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    println!("\nNote: Compare against baseline database");
    Ok(())
}
