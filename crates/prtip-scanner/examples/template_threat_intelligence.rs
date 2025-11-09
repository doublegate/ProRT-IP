#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Template: Threat Intelligence Integration
//! TODO: IOC correlation and threat feeds

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Threat Intelligence Template ===\n");
    println!("TODO: Integrate threat feeds:");
    println!("  1. Query IOC databases");
    println!("  2. Correlate IPs with threat actors");
    println!("  3. Enrich results with threat context");
    println!("  4. Alert on known malicious hosts\n");

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
