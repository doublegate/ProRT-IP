#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Bonus: Nmap Flag Translation
//! Demonstrates: Nmap command equivalents

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Nmap Compatibility ===\n");
    println!("Nmap equivalents:");
    println!("  nmap -sS <target> → prtip -sS <target>");
    println!("  nmap -sT <target> → prtip -sT <target>");
    println!("  nmap -sU <target> → prtip -sU <target>");
    println!("  nmap -p 80,443 → prtip -p 80,443");
    println!("  nmap -F <target> → prtip -F <target>");
    println!("  nmap -sV <target> → prtip -sV <target>\n");

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
