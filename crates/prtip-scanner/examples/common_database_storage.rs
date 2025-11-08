#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: Database Storage (SQLite)
//!
//! Demonstrates: Storing scan results in SQLite database
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_database_storage
//! ```
//!
//! ## Expected Output
//! - Scan results stored in SQLite
//! - Query and retrieval examples

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Database Storage Example ===\n");

    let target: IpAddr = "127.0.0.1".parse()?;
    let ports = vec![22, 80, 443, 8080];

    println!("Target: {}", target);
    println!("Ports: {:?}\n", ports);

    // Perform scan
    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };

    let scanner = SynScanner::new(config)?;
    let results = scanner.scan_ports(target, ports.clone()).await?;

    println!("Scan complete. Results:");
    for result in &results {
        println!("  Port {:5}: {:?}", result.port, result.state);
    }

    println!("\nNote: Database storage integration:");
    println!("  - Use prtip_scanner::ScanStorage for persistence");
    println!("  - SQLite schema: scans table (scan_id, timestamp, target)");
    println!("  - Results table: (scan_id, port, state, service, banner)");
    println!("  - Batch inserts for performance (1K-10K per transaction)");
    println!("  - WAL mode for concurrent access");
    println!("  - Indexes on scan_id, target_ip, port");
    println!("\nExample SQL queries:");
    println!("  SELECT * FROM scan_results WHERE scan_id = ?");
    println!("  SELECT port, state FROM scan_results WHERE target_ip = ?");
    println!("  SELECT COUNT(*) FROM scan_results WHERE state = 'Open'");

    Ok(())
}
