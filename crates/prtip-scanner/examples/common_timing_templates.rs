#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: Timing Templates Comparison
//!
//! Demonstrates: All timing templates (T0-T4) with performance benchmarks
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_timing_templates
//! ```
//!
//! ## Expected Output
//! - Scan duration for each timing template
//! - Speed comparison chart
//! - Use case recommendations

use prtip_core::{Config, ScanConfig, TimingTemplate};
use prtip_scanner::SynScanner;
use std::net::IpAddr;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Timing Templates Comparison ===\n");

    let target: IpAddr = "127.0.0.1".parse()?;
    let ports = vec![22, 80, 443];

    println!("Target: {}", target);
    println!("Ports: {:?}", ports);
    println!("\nBenchmarking all timing templates...\n");

    let templates = vec![
        (TimingTemplate::Paranoid, "T0", "Maximum stealth"),
        (TimingTemplate::Sneaky, "T1", "Stealth with speed"),
        (TimingTemplate::Polite, "T2", "Production safe"),
        (TimingTemplate::Normal, "T3", "Default"),
        (TimingTemplate::Aggressive, "T4", "Fast networks"),
    ];

    let mut results = Vec::new();

    for (template, name, description) in &templates {
        println!("Testing {}: {} - {}...", name, template, description);

        let mut scan_config = ScanConfig::default();
        scan_config.timing_template = *template;

        let config = Config {
            scan: scan_config,
            ..Config::default()
        };

        let scanner = SynScanner::new(config)?;

        let start = Instant::now();
        let scan_results = scanner.scan_ports(target, ports.clone()).await?;
        let duration = start.elapsed();

        println!("  Duration: {:.2?}", duration);
        println!(
            "  Open ports: {}",
            scan_results
                .iter()
                .filter(|r| matches!(r.state, prtip_core::PortState::Open))
                .count()
        );

        results.push((name, description, duration));
    }

    // Comparison table
    println!("\n{:=<80}", "");
    println!("Timing Template Performance Comparison:");
    println!("{:-<80}", "");
    println!(
        "{:10} | {:25} | {:12} | {:15}",
        "Template", "Description", "Duration", "Relative Speed"
    );
    println!("{:-<80}", "");

    let baseline = results[3].2; // T3 Normal is baseline

    for (name, desc, duration) in &results {
        let relative = duration.as_secs_f64() / baseline.as_secs_f64();
        println!(
            "{:10} | {:25} | {:12.2?} | {:15.2}x",
            name, desc, duration, relative
        );
    }
    println!("{:-<80}", "");

    println!("\nRecommendations:");
    println!("  T0 Paranoid: Use only for maximum stealth (very slow)");
    println!("  T1 Sneaky: Good for avoiding IDS detection");
    println!("  T2 Polite: Recommended for production environments");
    println!("  T3 Normal: Default for most scans");
    println!("  T4 Aggressive: Fast local networks only");

    println!("\nNote: Actual performance varies with network conditions");
    println!("      Choose based on stealth requirements vs speed needs");

    Ok(())
}
