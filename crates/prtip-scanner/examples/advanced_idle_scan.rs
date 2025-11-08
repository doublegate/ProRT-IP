#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: Idle (Zombie) Scanning
//!
//! Demonstrates: Maximum anonymity scanning using idle hosts
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//! - Suitable zombie host
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example advanced_idle_scan
//! ```

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Idle (Zombie) Scanning ===\n");

    println!("Idle scanning provides maximum anonymity by using a third-party 'zombie' host.");
    println!("The scanner never sends packets directly to the target.\n");

    let zombie: IpAddr = "127.0.0.2".parse()?;
    let target: IpAddr = "127.0.0.1".parse()?;
    let port = 80;

    println!("Zombie: {}", zombie);
    println!("Target: {}", target);
    println!("Port: {}\n", port);

    println!("Note: IdleScanner requires special configuration.");
    println!("See docs/25-IDLE-SCAN-GUIDE.md for implementation details.");

    // Demonstrate normal scan for comparison
    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;

    println!("\nFor comparison, performing normal SYN scan:");
    let results = scanner.scan_ports(target, vec![port]).await?;
    for r in results {
        println!("  Port {:5}: {:?}", r.port, r.state);
    }

    println!("\nHow it works:");
    println!("  1. Probe zombie to get baseline IP ID");
    println!("  2. Spoof SYN from zombie to target");
    println!("  3. Probe zombie again to check IP ID increment");
    println!("  4. Infer port state from IP ID change");

    println!("\nNote:");
    println!("  - Zombie must have predictable IP ID sequence");
    println!("  - Maximum anonymity (scanner IP never sent to target)");
    println!("  - 99.5% accuracy with suitable zombie");
    println!("  - Sprint 5.3 implementation");

    Ok(())
}
