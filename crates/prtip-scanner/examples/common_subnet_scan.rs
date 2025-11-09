#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Example: Subnet Scanning with CIDR Notation
//!
//! Demonstrates: Scanning multiple hosts in a network range
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//! - Network access to target subnet
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_subnet_scan
//! ```
//!
//! ## Expected Output
//! - Scan results for all IPs in 127.0.0.0/29 range
//! - Port states per host
//! - Summary of active hosts

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Subnet Scanning Example ===\n");

    // Simulate CIDR range: 127.0.0.0/29 (127.0.0.1-127.0.0.7)
    // In production, use ipnetwork crate for proper CIDR parsing
    let subnet_ips: Vec<IpAddr> = (1..=7)
        .map(|i| format!("127.0.0.{}", i).parse().unwrap())
        .collect();

    let ports = vec![22, 80, 443, 8080]; // Common ports to check

    println!("Subnet: 127.0.0.0/29 (simulated)");
    println!("Hosts to scan: {}", subnet_ips.len());
    println!("Ports per host: {:?}\n", ports);

    // Create scanner
    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;

    println!("Starting subnet scan...\n");
    let overall_start = std::time::Instant::now();

    let mut active_hosts = 0;
    let mut total_open_ports = 0;

    // Scan each host
    for ip in &subnet_ips {
        println!("Scanning {}...", ip);

        let start = std::time::Instant::now();
        let results = scanner.scan_ports(*ip, ports.clone()).await?;
        let duration = start.elapsed();

        let open_ports: Vec<u16> = results
            .iter()
            .filter(|r| matches!(r.state, prtip_core::PortState::Open))
            .map(|r| r.port)
            .collect();

        if !open_ports.is_empty() {
            active_hosts += 1;
            total_open_ports += open_ports.len();
            println!(
                "  ✓ {} is UP - {} open ports: {:?} (scanned in {:.2?})",
                ip,
                open_ports.len(),
                open_ports,
                duration
            );
        } else {
            println!("  - {} no open ports (scanned in {:.2?})", ip, duration);
        }
    }

    let total_duration = overall_start.elapsed();

    println!("\n{:=<60}", "");
    println!("Subnet Scan Summary:");
    println!("{:=<60}", "");
    println!("  Total hosts scanned: {}", subnet_ips.len());
    println!("  Active hosts (with open ports): {}", active_hosts);
    println!("  Total open ports found: {}", total_open_ports);
    println!("  Total scan duration: {:.2?}", total_duration);
    println!(
        "  Average time per host: {:.2?}",
        total_duration / subnet_ips.len() as u32
    );

    Ok(())
}
