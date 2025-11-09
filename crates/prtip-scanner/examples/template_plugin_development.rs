#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Template: Lua Plugin Development
//! TODO: Create custom Lua plugin

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Lua Plugin Development Template ===\n");
    println!("TODO: Write Lua plugin:");
    println!("  1. Create .lua file in plugins/");
    println!("  2. Implement detect() function");
    println!("  3. Use ctx.send() and ctx.recv()");
    println!("  4. Return structured results\n");
    println!("See docs/30-PLUGIN-SYSTEM.md for API reference");

    let scanner = SynScanner::new(Config {
        scan: ScanConfig::default(),
        ..Config::default()
    })?;
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![80])
        .await?;
    for r in results {
        println!("Port {}: {:?}", r.port, r.state);
    }
    Ok(())
}
