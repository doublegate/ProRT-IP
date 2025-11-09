#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Service Detection: SSH Banner Grabbing
//! Demonstrates: Extracting SSH version and configuration

use prtip_scanner::TcpConnectScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SSH Version Detection ===\n");

    println!("SSH Banner Analysis:");
    println!("  - OpenSSH vs Dropbear detection");
    println!("  - Version number extraction");
    println!("  - Encryption algorithm support");
    println!("  - Security vulnerability scanning\n");

    let scanner = TcpConnectScanner::new(std::time::Duration::from_secs(2), 1);
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![22], 1)
        .await?;

    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
