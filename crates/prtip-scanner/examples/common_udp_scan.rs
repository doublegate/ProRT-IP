#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: UDP Port Scanning
//!
//! Demonstrates: UDP scanning with protocol-specific probes (DNS, SNMP, NetBIOS)
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//! - Network access to target
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_udp_scan
//! ```
//!
//! ## Expected Output
//! - Open UDP ports with service detection
//! - ICMP unreachable interpretation
//! - Significantly slower than TCP scans

use prtip_core::{Config, ScanConfig};
use prtip_scanner::UdpScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: UDP Scanning Example ===\n");

    let target: IpAddr = "127.0.0.1".parse()?;

    // Common UDP ports
    let ports = vec![
        53,   // DNS
        67,   // DHCP Server
        68,   // DHCP Client
        69,   // TFTP
        123,  // NTP
        137,  // NetBIOS Name Service
        138,  // NetBIOS Datagram Service
        161,  // SNMP
        162,  // SNMP Trap
        500,  // IKE/IPSec
        514,  // Syslog
        1194, // OpenVPN
        1900, // SSDP
        5353, // mDNS
    ];

    println!("Target: {}", target);
    println!("Ports: Common UDP services");
    println!("\nNote: UDP scanning is 10-100x slower than TCP");
    println!("      OS rate limiting of ICMP unreachable messages affects speed");
    println!("      Open ports may appear as Open|Filtered (no response)\n");

    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };

    let scanner = UdpScanner::new(config)?;
    println!("Scanner initialized (UDP mode)");
    println!("Sending protocol-specific probes...\n");

    let start = std::time::Instant::now();
    let mut results = Vec::new();
    for port in &ports {
        let result = scanner.scan_port(target, *port).await?;
        results.push(result);
    }
    let duration = start.elapsed();

    // Display results
    println!("Results:");
    println!("{:-<70}", "");
    println!("{:6} | {:30} | {:15}", "Port", "Service", "State");
    println!("{:-<70}", "");

    for result in &results {
        let service = match result.port {
            53 => "DNS",
            67 => "DHCP Server",
            68 => "DHCP Client",
            69 => "TFTP",
            123 => "NTP",
            137 => "NetBIOS-NS",
            138 => "NetBIOS-DGM",
            161 => "SNMP",
            162 => "SNMP Trap",
            500 => "IKE",
            514 => "Syslog",
            1194 => "OpenVPN",
            1900 => "SSDP",
            5353 => "mDNS",
            _ => "Unknown",
        };

        println!("{:6} | {:30} | {:?}", result.port, service, result.state);
    }

    println!("{:-<70}", "");

    // Summary
    let open_count = results
        .iter()
        .filter(|r| matches!(r.state, prtip_core::PortState::Open))
        .count();

    let filtered_count = results
        .iter()
        .filter(|r| matches!(r.state, prtip_core::PortState::Filtered))
        .count();

    let closed_count = results
        .iter()
        .filter(|r| matches!(r.state, prtip_core::PortState::Closed))
        .count();

    println!("\nSummary:");
    println!("  Open: {} (received UDP response)", open_count);
    println!(
        "  Filtered: {} (no response or filtered by firewall)",
        filtered_count
    );
    println!("  Closed: {} (ICMP port unreachable)", closed_count);
    println!("  Total scanned: {}", ports.len());
    println!("  Duration: {:.2?}", duration);
    println!("  Average: {:.2?} per port", duration / ports.len() as u32);

    println!("\nInterpretation:");
    println!("  - Open: Service responded with UDP packet");
    println!("  - Filtered: No response (could be open or filtered by firewall)");
    println!("  - Closed: Received ICMP Port Unreachable");
    println!("\nTip: Use service detection to confirm filtered ports");

    Ok(())
}
