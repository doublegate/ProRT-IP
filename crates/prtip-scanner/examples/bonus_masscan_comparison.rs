#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Bonus: Masscan Speed Comparison
//! Benchmark: ProRT-IP vs Masscan performance

use prtip_core::{Config, ScanConfig, TimingTemplate};
use prtip_scanner::SynScanner;
use std::net::IpAddr;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Masscan Speed Comparison ===\n");

    let mut cfg = ScanConfig::default();
    cfg.timing_template = TimingTemplate::Aggressive;
    let scanner = SynScanner::new(Config {
        scan: cfg,
        ..Config::default()
    })?;

    let start = Instant::now();
    let results = scanner
        .scan_ports("127.0.0.1".parse::<IpAddr>()?, vec![80, 443])
        .await?;
    let duration = start.elapsed();

    println!("ProRT-IP: {} ports in {:.2?}", results.len(), duration);
    println!("Note: Compare with: masscan 127.0.0.1 -p80,443 --rate 10000");
    Ok(())
}
