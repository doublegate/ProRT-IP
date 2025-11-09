#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Service Detection: Multi-Protocol Banner Grabbing
//! Demonstrates: Extracting service banners from multiple protocols

use prtip_scanner::TcpConnectScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Protocol Banner Grabbing ===\n");

    println!("Banner Grabbing Protocols:");
    println!("  - HTTP: Server header");
    println!("  - SMTP: Greeting message");
    println!("  - FTP: Welcome banner");
    println!("  - SSH: Version string\n");

    let scanner = TcpConnectScanner::new(std::time::Duration::from_secs(2), 1);
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![21, 22, 25, 80], 4)
        .await?;

    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
