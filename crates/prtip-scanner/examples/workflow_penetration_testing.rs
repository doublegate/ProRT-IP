#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Workflow: Penetration Testing Automation
use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Penetration Testing Workflow ===\n");
    println!("Reconnaissance → Enumeration → Exploitation → Reporting");

    let scanner = SynScanner::new(Config {
        scan: ScanConfig::default(),
        ..Config::default()
    })?;

    println!("Phase 1: Reconnaissance (port scanning)...");
    let results = scanner
        .scan_ports(
            "127.0.0.1".parse::<IpAddr>()?,
            vec![22, 80, 443, 3306, 5432],
        )
        .await?;

    println!("\nDiscovered attack surface:");
    for r in results
        .iter()
        .filter(|r| matches!(r.state, prtip_core::PortState::Open))
    {
        println!("Target port {}: Enumerate services", r.port);
    }
    println!("\nNote: Always obtain written authorization before pen testing");
    Ok(())
}
