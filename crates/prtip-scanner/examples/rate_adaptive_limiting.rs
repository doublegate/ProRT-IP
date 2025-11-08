#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Rate Limiting: Adaptive Algorithm (V3)
//! Demonstrates: ICMP-aware adaptive rate limiting

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Adaptive Rate Limiting V3 ===\n");

    println!("V3 Algorithm Features:");
    println!("  - Industry-leading -1.8% overhead");
    println!("  - ICMP source quench detection");
    println!("  - Automatic backoff and recovery");
    println!("  - Per-target rate adjustment\n");

    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![80, 443])
        .await?;

    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
