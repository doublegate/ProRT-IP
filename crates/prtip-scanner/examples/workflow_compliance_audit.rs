#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Workflow: Compliance Audit (PCI-DSS, HIPAA)
use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Compliance Audit Workflow ===\n");
    println!("Policy validation, baseline enforcement, audit reporting");

    let scanner = SynScanner::new(Config {
        scan: ScanConfig::default(),
        ..Config::default()
    })?;
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![21, 23, 25, 80, 443])
        .await?;

    println!("\nCompliance check:");
    for r in results
        .iter()
        .filter(|r| matches!(r.state, prtip_core::PortState::Open))
    {
        match r.port {
            21 | 23 => println!("⚠ Port {}: Insecure protocol (FTP/Telnet)", r.port),
            _ => println!("✓ Port {}: Allowed", r.port),
        }
    }
    Ok(())
}
