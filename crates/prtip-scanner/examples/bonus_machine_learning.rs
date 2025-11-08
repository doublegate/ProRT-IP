#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Bonus: Machine Learning Service Prediction
//! TODO: ML-based service detection

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Machine Learning Service Detection ===\n");
    println!("TODO: Implement ML-based detection:");
    println!("  - Train on banner patterns");
    println!("  - Predict service from response");
    println!("  - Improve accuracy over time");
    println!("  - Handle unknown services\n");

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
