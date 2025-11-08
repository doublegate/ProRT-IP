#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: Multiple Output Formats
//!
//! Demonstrates: Generating scan results in JSON, XML, and Greppable formats
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability (for SYN scan)
//! - Or use TCP Connect scan (no privileges required)
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_output_formats
//! ```
//!
//! ## Expected Output
//! - Scan results in three formats:
//!   - JSON (machine-readable, structured)
//!   - XML (Nmap-compatible)
//!   - Greppable (shell scripting friendly)

use prtip_core::{Config, ScanConfig, ScanResult};
use prtip_scanner::SynScanner;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Serialize, Deserialize)]
struct JsonScanResult {
    target: String,
    ports_scanned: usize,
    open_ports: Vec<JsonPortInfo>,
    scan_duration_ms: u128,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct JsonPortInfo {
    port: u16,
    state: String,
    protocol: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Output Formats Example ===\n");

    let target: IpAddr = "127.0.0.1".parse()?;
    let ports = vec![22, 80, 443, 8080, 3000];

    // Perform scan
    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;

    let start = std::time::Instant::now();
    let results = scanner.scan_ports(target, ports.clone()).await?;
    let duration = start.elapsed();

    // 1. JSON Format
    println!("=== JSON Format ===\n");
    output_json(&target, &results, duration)?;

    println!("\n");

    // 2. XML Format (Nmap-compatible)
    println!("=== XML Format (Nmap-Compatible) ===\n");
    output_xml(&target, &results, duration);

    println!("\n");

    // 3. Greppable Format
    println!("=== Greppable Format ===\n");
    output_greppable(&target, &results);

    println!("\n");
    println!("Output formats demonstration complete!");
    println!("These formats can be redirected to files for parsing/analysis.");

    Ok(())
}

fn output_json(
    target: &IpAddr,
    results: &[ScanResult],
    duration: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let open_ports: Vec<JsonPortInfo> = results
        .iter()
        .filter(|r| matches!(r.state, prtip_core::PortState::Open))
        .map(|r| JsonPortInfo {
            port: r.port,
            state: format!("{:?}", r.state),
            protocol: "tcp".to_string(),
        })
        .collect();

    let json_output = JsonScanResult {
        target: target.to_string(),
        ports_scanned: results.len(),
        open_ports,
        scan_duration_ms: duration.as_millis(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let json_str = serde_json::to_string_pretty(&json_output)?;
    println!("{}", json_str);

    Ok(())
}

fn output_xml(target: &IpAddr, results: &[ScanResult], duration: std::time::Duration) {
    println!(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    println!(r#"<nmaprun scanner="ProRT-IP" version="0.5.0">"#);
    println!(r#"  <host>"#);
    println!(r#"    <address addr="{}" addrtype="ipv4"/>"#, target);
    println!(r#"    <ports>"#);

    for result in results {
        let state = format!("{:?}", result.state).to_lowercase();
        println!(r#"      <port protocol="tcp" portid="{}">"#, result.port);
        println!(r#"        <state state="{}"/>"#, state);
        println!(r#"      </port>"#);
    }

    println!(r#"    </ports>"#);
    println!(r#"  </host>"#);
    println!(
        r#"  <runstats><finished time="{}" elapsed="{}"/></runstats>"#,
        chrono::Utc::now().timestamp(),
        duration.as_secs_f64()
    );
    println!(r#"</nmaprun>"#);
}

fn output_greppable(target: &IpAddr, results: &[ScanResult]) {
    for result in results {
        let state = format!("{:?}", result.state);
        println!(
            "Host: {} ()\tPorts: {}/{}//tcp///",
            target,
            result.port,
            state.to_lowercase()
        );
    }
}
