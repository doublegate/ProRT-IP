#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Bonus: Blockchain Smart Contract Scanning
//! TODO: Ethereum/Solana node scanning

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Blockchain Smart Contract Scanning ===\n");
    println!("TODO: Scan blockchain nodes:");
    println!("  - Ethereum RPC (8545)");
    println!("  - Solana RPC (8899)");
    println!("  - WebSocket endpoints");
    println!("  - Detect vulnerable contracts\n");

    let scanner = SynScanner::new(Config {
        scan: ScanConfig::default(),
        ..Config::default()
    })?;
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![8545, 8899])
        .await?;
    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
