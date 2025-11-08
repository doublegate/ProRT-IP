#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: TCP Connect Scan (No Root Required)
//!
//! Demonstrates: TCP Connect scan using OS sockets (portable, no privileges)
//!
//! ## Prerequisites
//! - None (uses OS TCP stack, no raw sockets)
//! - Network access to target
//!
//! ## Usage
//! ```bash
//! cargo run --example common_tcp_connect_scan
//! ```
//!
//! ## Expected Output
//! - List of open ports on localhost
//! - Connection success/failure for each port
//! - Scan performance metrics

use prtip_scanner::TcpConnectScanner;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: TCP Connect Scan Example ===\n");
    println!("(No root privileges required)\n");

    // Configuration
    let target = "127.0.0.1".parse()?;
    let ports = vec![22, 80, 443, 8080, 3000, 3306, 5432, 6379, 8000, 9000];
    let timeout = Duration::from_secs(2);
    let retries = 1;

    println!("Target: {}", target);
    println!("Timeout: {:?}", timeout);
    println!("Retries: {}", retries);
    println!("Ports: {:?}\n", ports);

    // Create scanner
    let scanner = TcpConnectScanner::new(timeout, retries);

    // Perform scan
    println!("Starting TCP Connect scan...\n");
    let start = std::time::Instant::now();
    let results = scanner.scan_ports(target, ports.clone(), 10).await?;
    let duration = start.elapsed();

    // Analyze results
    println!("Results:");
    println!("{:-<60}", "");

    let mut open_ports = Vec::new();
    for result in results {
        let status = if matches!(result.state, prtip_core::PortState::Open) {
            open_ports.push(result.port);
            "OPEN"
        } else {
            "CLOSED/FILTERED"
        };

        println!(
            "Port {:5}: {:15} (state: {:?})",
            result.port, status, result.state
        );
    }

    println!("{:-<60}", "");
    println!("\nSummary:");
    println!("  Open ports: {} -> {:?}", open_ports.len(), open_ports);
    println!("  Total scanned: {}", ports.len());
    println!("  Duration: {:.2?}", duration);
    println!(
        "  Rate: {:.0} ports/sec",
        ports.len() as f64 / duration.as_secs_f64()
    );

    if open_ports.is_empty() {
        println!("\nNote: No open ports found on localhost.");
        println!("Consider running services (e.g., SSH, HTTP) to see open ports.");
    }

    Ok(())
}
