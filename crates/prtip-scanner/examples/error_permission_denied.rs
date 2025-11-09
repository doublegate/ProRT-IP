#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Error Handling: Permission Errors
use prtip_scanner::TcpConnectScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Permission Error Handling ===\n");
    println!("CAP_NET_RAW missing, fallback to TCP Connect scan");

    let scanner = TcpConnectScanner::new(std::time::Duration::from_secs(2), 1);
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![80], 1)
        .await?;
    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
