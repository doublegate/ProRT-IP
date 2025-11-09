#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Workflow: Two-Stage Discovery + Enumeration
use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Discovery + Enumeration Workflow ===\n");
    println!("Stage 1: Fast host discovery, Stage 2: Full port scan");

    let scanner = SynScanner::new(Config {
        scan: ScanConfig::default(),
        ..Config::default()
    })?;

    // Stage 1: Fast discovery
    println!("Stage 1: Host discovery (ports 80, 443)...");
    let discovery = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![80, 443])
        .await?;

    // Stage 2: Full enumeration
    if discovery
        .iter()
        .any(|r| matches!(r.state, prtip_core::PortState::Open))
    {
        println!("Stage 2: Full port enumeration...");
        let full = scanner
            .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![22, 80, 443, 8080])
            .await?;
        for r in full {
            println!("Port {}: {:?}", r.port, r.state);
        }
    }
    Ok(())
}
