#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Example: Stealth Scanning Techniques
//!
//! Demonstrates: FIN, NULL, and Xmas scans for firewall evasion
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//! - Network access to target
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_stealth_scan
//! ```
//!
//! ## Expected Output
//! - Results from 3 different stealth scan types
//! - Comparison of detection evasion effectiveness
//! - Firewall filtering insights

use prtip_core::{Config, ScanConfig};
use prtip_scanner::{StealthScanType, StealthScanner};
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Stealth Scanning Example ===\n");

    let target: IpAddr = "127.0.0.1".parse()?;
    let ports = vec![22, 80, 443, 8080];

    println!("Target: {}", target);
    println!("Ports: {:?}", ports);
    println!("\nNote: Stealth scans bypass stateful firewalls that only track SYN packets");
    println!("      These scans work on most Linux/Unix systems");
    println!("      May not work on Windows or Cisco devices\n");

    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };

    let mut scanner = StealthScanner::new(config)?;
    scanner.initialize().await?;

    // FIN Scan
    println!("=== FIN Scan (TCP flags: FIN) ===");
    println!("How it works: Sends FIN packet, expects RST from closed ports");

    let start = std::time::Instant::now();
    let mut fin_results = Vec::new();
    for port in &ports {
        let result = scanner
            .scan_port(target, *port, StealthScanType::Fin)
            .await?;
        fin_results.push(result);
    }
    let fin_duration = start.elapsed();

    println!("Results:");
    for result in &fin_results {
        println!("  Port {:5}: {:?}", result.port, result.state);
    }
    println!("Duration: {:.2?}\n", fin_duration);

    // NULL Scan
    println!("=== NULL Scan (TCP flags: None) ===");
    println!("How it works: Sends packet with no flags, expects RST from closed ports");

    let start = std::time::Instant::now();
    let mut null_results = Vec::new();
    for port in &ports {
        let result = scanner
            .scan_port(target, *port, StealthScanType::Null)
            .await?;
        null_results.push(result);
    }
    let null_duration = start.elapsed();

    println!("Results:");
    for result in &null_results {
        println!("  Port {:5}: {:?}", result.port, result.state);
    }
    println!("Duration: {:.2?}\n", null_duration);

    // Xmas Scan
    println!("=== Xmas Scan (TCP flags: FIN, PSH, URG) ===");
    println!("How it works: Sends packet with FIN/PSH/URG, expects RST from closed ports");

    let start = std::time::Instant::now();
    let mut xmas_results = Vec::new();
    for port in &ports {
        let result = scanner
            .scan_port(target, *port, StealthScanType::Xmas)
            .await?;
        xmas_results.push(result);
    }
    let xmas_duration = start.elapsed();

    println!("Results:");
    for result in &xmas_results {
        println!("  Port {:5}: {:?}", result.port, result.state);
    }
    println!("Duration: {:.2?}\n", xmas_duration);

    // Comparison
    println!("{:=<60}", "");
    println!("Comparison Summary:");
    println!("{:-<60}", "");
    println!(
        "{:15} | {:10} | {:10}",
        "Scan Type", "Duration", "Open Ports"
    );
    println!("{:-<60}", "");

    let count_open = |results: &Vec<prtip_core::ScanResult>| -> usize {
        results
            .iter()
            .filter(|r| matches!(r.state, prtip_core::PortState::Open))
            .count()
    };

    println!(
        "{:15} | {:10.2?} | {:10}",
        "FIN",
        fin_duration,
        count_open(&fin_results)
    );
    println!(
        "{:15} | {:10.2?} | {:10}",
        "NULL",
        null_duration,
        count_open(&null_results)
    );
    println!(
        "{:15} | {:10.2?} | {:10}",
        "Xmas",
        xmas_duration,
        count_open(&xmas_results)
    );
    println!("{:-<60}", "");

    println!("\nInterpretation:");
    println!("  - Open|Filtered: Port may be open, or firewall is filtering packets");
    println!("  - Closed: Port is definitely closed (received RST)");
    println!("  - Use multiple scan types to confirm results");

    Ok(())
}
