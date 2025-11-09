#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Example: Host Discovery
//!
//! Demonstrates: Multi-method host discovery using ICMP, ARP, and TCP pings
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//! - Network access to target subnet
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_host_discovery
//! ```
//!
//! ## Expected Output
//! - List of live hosts on the network
//! - Discovery method effectiveness comparison
//! - Response time statistics

use prtip_core::{Config, ScanConfig};
use prtip_scanner::{SynScanner, TcpConnectScanner};
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Host Discovery Example ===\n");

    // Scan local subnet (adjust for your network)
    let targets: Vec<IpAddr> = vec![
        "127.0.0.1".parse()?,
        "127.0.0.2".parse()?,
        "127.0.0.3".parse()?,
        // Add more targets from your local subnet
        // e.g., 192.168.1.1 through 192.168.1.254
    ];

    println!("Targets: {} hosts", targets.len());
    println!("\nNote: Host discovery determines which hosts are up before port scanning");
    println!("      Saves time by avoiding full scans of offline hosts\n");

    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };

    // Method 1: TCP SYN Ping (Ports 80, 443)
    println!("=== Method 1: TCP SYN Ping (Ports 80, 443) ===");
    println!("How it works: Sends SYN to common ports, expects SYN/ACK or RST");

    let syn_scanner = SynScanner::new(config.clone())?;
    let common_ports = vec![80, 443];
    let mut syn_alive = Vec::new();

    let start = std::time::Instant::now();
    for target in &targets {
        let results = syn_scanner
            .scan_ports(*target, common_ports.clone())
            .await?;
        if !results.is_empty() {
            println!("  {} is UP (TCP SYN)", target);
            syn_alive.push(target);
        } else {
            println!("  {} is DOWN (no TCP response)", target);
        }
    }
    let syn_duration = start.elapsed();
    println!(
        "Found {} hosts via TCP SYN in {:.2?}\n",
        syn_alive.len(),
        syn_duration
    );

    // Method 2: TCP Connect to common ports (alternative method)
    println!("=== Method 2: TCP Connect Scan (Ports 22, 80) ===");
    println!("How it works: Full TCP handshake, no root privileges needed");

    let tcp_scanner = TcpConnectScanner::new(std::time::Duration::from_secs(2), 1);
    let connect_ports = vec![22, 80];
    let mut tcp_alive = Vec::new();

    let start = std::time::Instant::now();
    for target in &targets {
        let results = tcp_scanner
            .scan_ports(*target, connect_ports.clone(), 2)
            .await?;

        if !results.is_empty() {
            println!("  {} is UP (TCP Connect)", target);
            tcp_alive.push(target);
        } else {
            println!("  {} is DOWN (no TCP response)", target);
        }
    }
    let tcp_duration = start.elapsed();
    println!(
        "Found {} hosts via TCP Connect in {:.2?}\n",
        tcp_alive.len(),
        tcp_duration
    );

    // Method 3: Combined approach (most reliable)
    println!("=== Method 3: Combined Discovery ===");
    let mut combined_alive = syn_alive.clone();
    for target in &tcp_alive {
        if !combined_alive.contains(target) {
            combined_alive.push(target);
        }
    }

    println!("Total unique hosts discovered: {}", combined_alive.len());
    for target in &combined_alive {
        println!("  {} is UP", target);
    }

    // Summary
    println!("\n{:=<60}", "");
    println!("Discovery Method Comparison:");
    println!("{:-<60}", "");
    println!("{:20} | {:10} | {:10}", "Method", "Hosts", "Duration");
    println!("{:-<60}", "");
    println!(
        "{:20} | {:10} | {:10.2?}",
        "TCP SYN",
        syn_alive.len(),
        syn_duration
    );
    println!(
        "{:20} | {:10} | {:10.2?}",
        "TCP Connect",
        tcp_alive.len(),
        tcp_duration
    );
    println!(
        "{:20} | {:10} | {:10}",
        "Combined",
        combined_alive.len(),
        "-"
    );
    println!("{:-<60}", "");

    println!("\nRecommendations:");
    println!("  - Use TCP SYN for fast host discovery (requires root)");
    println!("  - Use TCP Connect when root privileges not available");
    println!("  - Combine methods for comprehensive discovery");
    println!("  - For internet scanning, use multiple TCP ports (80, 443, 22, 21)");
    println!("  - ICMP Echo can also be used but often blocked by firewalls");

    Ok(())
}
