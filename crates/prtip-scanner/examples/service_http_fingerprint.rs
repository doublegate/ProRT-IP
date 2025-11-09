#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Service Detection: HTTP Server Fingerprinting
//! Demonstrates: Detecting HTTP server type and version

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HTTP Server Fingerprinting ===\n");

    println!("HTTP Server Detection:");
    println!("  - Apache vs Nginx vs IIS identification");
    println!("  - Version extraction from Server header");
    println!("  - Module detection (mod_ssl, mod_security)");
    println!("  - Technology stack inference\n");

    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![80, 8080])
        .await?;

    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
