#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Template: Custom Protocol Handler
//! TODO: Add support for custom protocol detection

use prtip_scanner::TcpConnectScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Protocol Handler Template ===\n");
    println!("TODO: Implement custom protocol detection:");
    println!("  1. Define protocol-specific payload");
    println!("  2. Send probe to service");
    println!("  3. Parse response");
    println!("  4. Extract service info\n");

    let scanner = TcpConnectScanner::new(std::time::Duration::from_secs(2), 1);
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![80], 1)
        .await?;
    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
