#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: Basic SYN Scan
//!
//! Demonstrates: Simple TCP SYN scan of a single host on common ports
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//! - Network access to target
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_basic_syn_scan
//! ```
//!
//! ## Expected Output
//! - List of open ports on localhost (22, 80, 443 if running services)
//! - Port states (Open, Closed, Filtered)
//! - Scan duration and statistics

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Basic SYN Scan Example ===\n");

    // Target and ports
    let target: IpAddr = "127.0.0.1".parse()?;
    let ports = vec![22, 80, 443, 8080, 3000, 5432, 3306, 6379];

    println!("Target: {}", target);
    println!("Ports: {:?}\n", ports);

    // Create scanner with default configuration
    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };

    let scanner = SynScanner::new(config)?;
    println!("Scanner initialized (SYN mode)");
    println!("Scanning {} ports...\n", ports.len());

    // Perform scan
    let start = std::time::Instant::now();
    let results = scanner.scan_ports(target, ports.clone()).await?;
    let duration = start.elapsed();

    // Display results
    println!("Results:");
    println!("{:-<50}", "");

    let mut open_count = 0;
    for result in results {
        println!("Port {:5}: {:?}", result.port, result.state);
        if matches!(result.state, prtip_core::PortState::Open) {
            open_count += 1;
        }
    }

    println!("{:-<50}", "");
    println!("\nSummary:");
    println!("  Open ports: {}", open_count);
    println!("  Total scanned: {}", ports.len());
    println!("  Duration: {:.2?}", duration);

    Ok(())
}
