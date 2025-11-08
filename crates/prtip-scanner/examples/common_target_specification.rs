#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: Target Specification
//!
//! Demonstrates: Multiple ways to specify scan targets (single IP, ranges, CIDR, files)
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_target_specification
//! ```
//!
//! ## Expected Output
//! - Various target specification methods
//! - Combined target list
//! - Exclusion filtering

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::{IpAddr, Ipv4Addr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Target Specification Example ===\n");

    // Method 1: Single IP
    let single_target: IpAddr = "127.0.0.1".parse()?;
    println!("Method 1 - Single IP:");
    println!("  {}", single_target);

    // Method 2: IP Range (programmatic generation)
    println!("\nMethod 2 - IP Range (127.0.0.1-127.0.0.5):");
    let mut range_targets = Vec::new();
    for i in 1..=5 {
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, i));
        range_targets.push(ip);
        println!("  {}", ip);
    }

    // Method 3: CIDR notation (using ipnetwork crate in production)
    println!("\nMethod 3 - CIDR (127.0.0.0/30):");
    let cidr_targets = vec![
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 0)),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3)),
    ];
    for ip in &cidr_targets {
        println!("  {}", ip);
    }

    // Method 4: From file (simulated with in-memory list)
    println!("\nMethod 4 - From list:");
    let file_targets = vec![
        "127.0.0.1".parse::<IpAddr>()?,
        "::1".parse::<IpAddr>()?, // IPv6 localhost
    ];
    for ip in &file_targets {
        println!("  {}", ip);
    }

    // Combine all targets
    let mut all_targets = Vec::new();
    all_targets.push(single_target);
    all_targets.extend(&range_targets);
    all_targets.extend(&cidr_targets);
    all_targets.extend(&file_targets);

    // Remove duplicates
    all_targets.sort();
    all_targets.dedup();

    println!("\nCombined unique targets: {}", all_targets.len());

    // Exclusion example
    let exclude = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 0));
    all_targets.retain(|ip| *ip != exclude);
    println!("After excluding {}: {}", exclude, all_targets.len());

    // Perform scan
    println!("\nScanning {} targets on port 80...\n", all_targets.len());

    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;

    for target in &all_targets[..3.min(all_targets.len())] {
        // Scan first 3 for demo
        let results = scanner.scan_ports(*target, vec![80]).await?;
        for result in results {
            println!("{}: Port {}: {:?}", target, result.port, result.state);
        }
    }

    println!("\nNote: In production, use ipnetwork crate for CIDR parsing");
    println!("      Read targets from files with std::fs::read_to_string()");

    Ok(())
}
