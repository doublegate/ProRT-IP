#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Template: Web UI Integration
//! TODO: REST API for web frontend

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Web UI Integration Template ===\n");
    println!("TODO: Build REST API:");
    println!("  1. Create /api/scan endpoint");
    println!("  2. Stream results via WebSocket");
    println!("  3. Provide progress updates");
    println!("  4. Return JSON responses\n");

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
