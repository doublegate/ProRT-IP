#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Workflow: Network Topology Mapping
use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Network Mapping Workflow ===\n");
    println!("Subnet discovery, traceroute, topology generation");

    let scanner = SynScanner::new(Config {
        scan: ScanConfig::default(),
        ..Config::default()
    })?;
    let targets = vec!["127.0.0.1", "127.0.0.2", "127.0.0.3"];

    for target in targets {
        let results = scanner
            .scan_ports(target.parse::<IpAddr>()?, vec![80])
            .await?;
        println!(
            "Host {}: {} open ports",
            target,
            results
                .iter()
                .filter(|r| matches!(r.state, prtip_core::PortState::Open))
                .count()
        );
    }
    Ok(())
}
