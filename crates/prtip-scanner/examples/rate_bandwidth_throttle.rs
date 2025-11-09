#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Rate Limiting: Bandwidth Throttling
//! Demonstrates: Network bandwidth control

use prtip_core::{Config, ScanConfig, TimingTemplate};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Bandwidth Throttling ===\n");

    println!("Bandwidth Control:");
    println!("  - Maximum packets per second");
    println!("  - Burst allowance");
    println!("  - Smooth rate limiting");
    println!("  - Network courtesy mode\n");

    let mut scan_config = ScanConfig::default();
    scan_config.timing_template = TimingTemplate::Polite;

    let scanner = SynScanner::new(Config {
        scan: scan_config,
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
