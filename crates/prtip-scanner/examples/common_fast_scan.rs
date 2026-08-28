#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Example: Fast Scan (100-port priority list)
//!
//! Demonstrates: Quick scanning of ProRT-IP's 100-port priority list (`-F`) with
//! aggressive timing. That list is an editorial ranking of IANA port
//! assignments, not a frequency ranking -- see `prtip_core::top_ports`.
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
//! - Scan of the 100-port priority list in <5 seconds
//! - Open/Closed state for each port
//! - Packets per second metrics

use prtip_core::{Config, ScanConfig, TimingTemplate};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Fast Scan Example ===\n");

    let target: IpAddr = "127.0.0.1".parse()?;

    // The same 100-port list `-F` uses. Taken from the crate constant rather
    // than copied, so it cannot drift out of sync with it.
    let top_100_ports: Vec<u16> = prtip_core::top_ports::PRIORITY_PORTS_100.to_vec();

    println!("Target: {}", target);
    println!("Ports: 100-port priority list (IANA-derived, not frequency-ranked)");
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
