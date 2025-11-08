#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Performance: Resource Limit Tuning
//! Demonstrates: ulimit and resource optimization

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Resource Limit Tuning ===\n");
    println!("Resource Optimization:");
    println!("  - ulimit -n (open files)");
    println!("  - ulimit -u (max processes)");
    println!("  - sysctl net.ipv4.ip_local_port_range");
    println!("  - TCP TIME_WAIT tuning\n");

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
