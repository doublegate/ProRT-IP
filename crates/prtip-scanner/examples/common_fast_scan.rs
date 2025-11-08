#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: Fast Scan (Top 100 Ports)
//!
//! Demonstrates: Quick scanning of top 100 most common ports with aggressive timing
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//! - Network access to target
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_fast_scan
//! ```
//!
//! ## Expected Output
//! - Scan of top 100 ports in <5 seconds
//! - Open/Closed state for each port
//! - Packets per second metrics

use prtip_core::{Config, ScanConfig, TimingTemplate};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Fast Scan Example ===\n");

    let target: IpAddr = "127.0.0.1".parse()?;

    // Top 100 most common ports (nmap -F equivalent)
    let top_100_ports: Vec<u16> = vec![
        7, 9, 13, 21, 22, 23, 25, 26, 37, 53, 79, 80, 81, 88, 106, 110, 111, 113, 119, 135, 139,
        143, 144, 179, 199, 389, 427, 443, 444, 445, 465, 513, 514, 515, 543, 544, 548, 554, 587,
        631, 646, 873, 990, 993, 995, 1025, 1026, 1027, 1028, 1029, 1110, 1433, 1720, 1723, 1755,
        1900, 2000, 2001, 2049, 2121, 2717, 3000, 3128, 3306, 3389, 3986, 4899, 5000, 5009, 5051,
        5060, 5101, 5190, 5357, 5432, 5631, 5666, 5800, 5900, 6000, 6001, 6646, 7070, 8000, 8008,
        8009, 8080, 8081, 8443, 8888, 9100, 9999, 10000, 32768, 49152, 49153, 49154, 49155, 49156,
        49157,
    ];

    println!("Target: {}", target);
    println!("Ports: Top 100 most common");
    println!("Timing: T4 (Aggressive)\n");

    // Configure aggressive timing (T4 = Aggressive)
    let mut scan_config = ScanConfig::default();
    scan_config.timing_template = TimingTemplate::Aggressive;

    let config = Config {
        scan: scan_config,
        ..Config::default()
    };

    let scanner = SynScanner::new(config)?;
    println!("Scanner initialized with T4 timing");

    // Perform fast scan
    let start = std::time::Instant::now();
    let results = scanner.scan_ports(target, top_100_ports.clone()).await?;
    let duration = start.elapsed();

    // Calculate metrics
    let open_count = results
        .iter()
        .filter(|r| matches!(r.state, prtip_core::PortState::Open))
        .count();

    let pps = (top_100_ports.len() as f64) / duration.as_secs_f64();

    // Display results
    println!("\nResults:");
    println!("{:-<60}", "");

    for result in results
        .iter()
        .filter(|r| matches!(r.state, prtip_core::PortState::Open))
    {
        println!("Port {:5}: OPEN", result.port);
    }

    println!("{:-<60}", "");
    println!("\nSummary:");
    println!("  Open ports: {}/{}", open_count, top_100_ports.len());
    println!("  Duration: {:.2?}", duration);
    println!("  Speed: {:.0} packets/sec", pps);
    println!("\nNote: T4 timing is suitable for fast local networks");
    println!("      Use T3 or lower for internet scanning");

    Ok(())
}
