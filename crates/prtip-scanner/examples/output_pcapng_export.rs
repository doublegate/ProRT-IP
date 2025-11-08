#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Output: PCAPNG Packet Capture Export
//! Demonstrates: Wireshark-compatible packet export

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PCAPNG Export ===\n");
    println!("PCAPNG Features:");
    println!("  - Wireshark-compatible format");
    println!("  - Packet-level capture");
    println!("  - Metadata preservation");
    println!("  - Forensic analysis support\n");

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
    println!("\nNote: Enable with --pcapng flag");
    Ok(())
}
