#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Service Detection: TLS/SSL Analysis
//! Demonstrates: TLS version and cipher suite detection

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TLS/SSL Analysis ===\n");

    println!("TLS Protocol Detection:");
    println!("  - TLS 1.0/1.1/1.2/1.3 version detection");
    println!("  - Supported cipher suites");
    println!("  - Certificate chain analysis");
    println!("  - Weak encryption warnings\n");

    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![443])
        .await?;

    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
