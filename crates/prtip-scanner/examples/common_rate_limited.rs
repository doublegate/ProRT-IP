#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Example: Rate-Limited Scanning (Polite Mode)
//!
//! Demonstrates: Slow, polite scanning with T0-T2 timing for stealth and courtesy
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//! - Network access to target
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_rate_limited
//! ```
//!
//! ## Expected Output
//! - Slow scan with minimal network impact
//! - Comparison of T0, T1, and T2 timing templates
//! - Network courtesy metrics

use prtip_core::{Config, ScanConfig, TimingTemplate};
use prtip_scanner::SynScanner;
use std::net::IpAddr;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Rate-Limited Scanning Example ===\n");

    let target: IpAddr = "127.0.0.1".parse()?;
    let ports = vec![22, 80, 443, 8080];

    println!("Target: {}", target);
    println!("Ports: {:?}", ports);
    println!("\nNote: Polite scanning minimizes network impact and IDS detection");
    println!("      Suitable for production networks and courtesy scanning\n");

    // T0: Paranoid (very slow, IDS evasion)
    println!("=== T0: Paranoid Timing ===");
    println!("Purpose: Maximum stealth, IDS evasion");
    println!("Speed: ~5 minutes per port");

    let mut scan_config = ScanConfig::default();
    scan_config.timing_template = TimingTemplate::Paranoid;

    let config = Config {
        scan: scan_config,
        ..Config::default()
    };

    let scanner = SynScanner::new(config)?;

    let start = Instant::now();
    let t0_results = scanner.scan_ports(target, ports.clone()).await?;
    let t0_duration = start.elapsed();

    println!("Duration: {:.2?}", t0_duration);
    println!("Ports scanned: {}", t0_results.len());
    println!(
        "Rate: {:.2} packets/min\n",
        (ports.len() as f64) / t0_duration.as_secs_f64() * 60.0
    );

    // T1: Sneaky (slow, low detection)
    println!("=== T1: Sneaky Timing ===");
    println!("Purpose: Stealth with reasonable speed");
    println!("Speed: ~15 seconds per port");

    let mut scan_config = ScanConfig::default();
    scan_config.timing_template = TimingTemplate::Sneaky;

    let config = Config {
        scan: scan_config,
        ..Config::default()
    };

    let scanner = SynScanner::new(config)?;

    let start = Instant::now();
    let t1_results = scanner.scan_ports(target, ports.clone()).await?;
    let t1_duration = start.elapsed();

    println!("Duration: {:.2?}", t1_duration);
    println!("Ports scanned: {}", t1_results.len());
    println!(
        "Rate: {:.2} packets/sec\n",
        (ports.len() as f64) / t1_duration.as_secs_f64()
    );

    // T2: Polite (normal, courteous)
    println!("=== T2: Polite Timing ===");
    println!("Purpose: Courteous scanning, production-safe");
    println!("Speed: ~0.4 seconds per port");

    let mut scan_config = ScanConfig::default();
    scan_config.timing_template = TimingTemplate::Polite;

    let config = Config {
        scan: scan_config,
        ..Config::default()
    };

    let scanner = SynScanner::new(config)?;

    let start = Instant::now();
    let t2_results = scanner.scan_ports(target, ports.clone()).await?;
    let t2_duration = start.elapsed();

    println!("Duration: {:.2?}", t2_duration);
    println!("Ports scanned: {}", t2_results.len());
    println!(
        "Rate: {:.2} packets/sec\n",
        (ports.len() as f64) / t2_duration.as_secs_f64()
    );

    // Comparison
    println!("{:=<70}", "");
    println!("Timing Template Comparison:");
    println!("{:-<70}", "");
    println!(
        "{:10} | {:12} | {:15} | {:20}",
        "Template", "Duration", "Speed", "Use Case"
    );
    println!("{:-<70}", "");
    println!(
        "{:10} | {:12.2?} | {:15.2} pkt/min | {:20}",
        "T0 Paranoid",
        t0_duration,
        (ports.len() as f64) / t0_duration.as_secs_f64() * 60.0,
        "IDS evasion"
    );
    println!(
        "{:10} | {:12.2?} | {:15.2} pkt/sec | {:20}",
        "T1 Sneaky",
        t1_duration,
        (ports.len() as f64) / t1_duration.as_secs_f64(),
        "Stealth scanning"
    );
    println!(
        "{:10} | {:12.2?} | {:15.2} pkt/sec | {:20}",
        "T2 Polite",
        t2_duration,
        (ports.len() as f64) / t2_duration.as_secs_f64(),
        "Production networks"
    );
    println!("{:-<70}", "");

    println!("\nRecommendations:");
    println!("  - T0: Use only when stealth is critical (very slow)");
    println!("  - T1: Good balance of stealth and speed");
    println!("  - T2: Default for production environments");
    println!("  - T3: Standard for most scans (default)");
    println!("  - T4-T5: Fast local networks only");

    println!("\nRate Limiting Benefits:");
    println!("  ✓ Avoids triggering IDS/IPS alerts");
    println!("  ✓ Minimizes network congestion");
    println!("  ✓ Reduces target resource consumption");
    println!("  ✓ Professional courtesy to system administrators");

    Ok(())
}
