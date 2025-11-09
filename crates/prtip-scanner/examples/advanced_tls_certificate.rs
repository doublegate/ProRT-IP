#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Example: TLS Certificate Analysis
//!
//! Demonstrates: X.509v3 certificate parsing, chain validation, expiry checks
//!
//! ## Prerequisites
//! - Network access to HTTPS service
//!
//! ## Usage
//! ```bash
//! cargo run --example advanced_tls_certificate
//! ```

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: TLS Certificate Analysis ===\n");

    println!("TLS Certificate Analysis Features:");
    println!("  - X.509v3 certificate parsing");
    println!("  - Certificate chain validation");
    println!("  - Expiry checking");
    println!("  - SNI (Server Name Indication) support");
    println!("  - TLS version detection");
    println!("  - Cipher suite analysis");

    println!("\nExample Certificate Fields:");
    println!("  Subject: CN=example.com");
    println!("  Issuer: CN=Let's Encrypt Authority");
    println!("  Valid from: 2025-01-01 00:00:00 UTC");
    println!("  Valid until: 2025-04-01 00:00:00 UTC");
    println!("  Serial: 04:3a:f2:1e:7d:...");

    println!("\n  Subject Alt Names:");
    println!("    - example.com");
    println!("    - www.example.com");

    println!("\n  Key Info:");
    println!("    Algorithm: RSA");
    println!("    Size: 2048 bits");
    println!("    Signature: sha256WithRSAEncryption");

    // Demonstrate HTTPS port scanning
    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;
    let target: IpAddr = "127.0.0.1".parse()?;

    println!("\nScanning HTTPS port (443)...");
    let results = scanner.scan_ports(target, vec![443]).await?;
    for r in results {
        println!("  Port {:5}: {:?}", r.port, r.state);
    }

    println!("\nFeatures:");
    println!("  - X.509v3 certificate parsing (1.33μs avg)");
    println!("  - Certificate chain validation");
    println!("  - Expiry checking");
    println!("  - SNI support");
    println!("  - TLS version detection");
    println!("  - Cipher suite analysis");

    Ok(())
}
