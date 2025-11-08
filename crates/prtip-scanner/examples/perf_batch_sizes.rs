#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Performance: Batch Size Tuning
//! Demonstrates: Optimal batch sizing for throughput

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Batch Size Tuning ===\n");

    println!("Batch Optimization:");
    println!("  - 1K-10K inserts per transaction");
    println!("  - sendmmsg/recvmmsg batching");
    println!("  - Zero-copy for >10KB transfers");
    println!("  - Throughput vs latency tradeoff\n");

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
