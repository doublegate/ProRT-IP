#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: Service Version Detection
//!
//! Demonstrates: Detecting service versions on common ports
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//! - nmap-service-probes database file
//! - Running services on target (SSH, HTTP, etc.)
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_service_detection
//! ```
//!
//! ## Expected Output
//! - Detected services with names and versions
//! - Banner information where available
//! - Detection confidence scores

use prtip_core::{Config, ScanConfig, ServiceProbeDb};
use prtip_scanner::{service_detector::ServiceDetector, SynScanner};
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Service Version Detection Example ===\n");

    let target: IpAddr = "127.0.0.1".parse()?;
    let ports = vec![22, 80, 443, 3000, 3306, 5432, 6379, 8080];

    println!("Target: {}", target);
    println!("Ports: {:?}\n", ports);

    // Step 1: Create SYN scanner to find open ports
    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;

    println!("Phase 1: Port scanning...");
    let scan_results = scanner.scan_ports(target, ports.clone()).await?;

    // Filter open ports
    let open_ports: Vec<u16> = scan_results
        .iter()
        .filter(|r| matches!(r.state, prtip_core::PortState::Open))
        .map(|r| r.port)
        .collect();

    if open_ports.is_empty() {
        println!("No open ports found. Ensure target has running services.");
        println!("\nTip: Start some services on localhost:");
        println!("  - SSH: sudo systemctl start sshd");
        println!("  - HTTP: python3 -m http.server 8080");
        return Ok(());
    }

    println!(
        "  Found {} open ports: {:?}\n",
        open_ports.len(),
        open_ports
    );

    // Step 2: Load service detection database
    println!("Phase 2: Loading service detection database...");

    // Try common locations for nmap-service-probes
    let probe_paths = vec![
        "/usr/share/nmap/nmap-service-probes",
        "/opt/homebrew/share/nmap/nmap-service-probes",
        "./nmap-service-probes",
    ];

    let mut probe_data = None;
    for path in &probe_paths {
        if let Ok(data) = std::fs::read_to_string(path) {
            probe_data = Some(data);
            println!("  Loaded probe database from: {}", path);
            break;
        }
    }

    let probe_data = match probe_data {
        Some(data) => data,
        None => {
            println!("  ⚠ Warning: nmap-service-probes not found");
            println!("  Searched: {:?}", probe_paths);
            println!("  Service detection will use default probes only");
            String::new() // Use empty string for basic detection
        }
    };

    let db = if !probe_data.is_empty() {
        ServiceProbeDb::parse(&probe_data)?
    } else {
        ServiceProbeDb::default()
    };

    // Create service detector with intensity 7 (comprehensive)
    let service_detector = ServiceDetector::new(db, 7);

    // Step 3: Detect services
    println!("\nPhase 3: Service detection (intensity 7)...\n");
    println!("{:-<80}", "");
    println!(
        "{:5} | {:20} | {:30} | CPE",
        "Port", "Service", "Version",
    );
    println!("{:-<80}", "");

    for port in open_ports {
        let addr = std::net::SocketAddr::new(target, port);

        match service_detector.detect_service(addr).await {
            Ok(service_info) => {
                let version = service_info.version.as_deref().unwrap_or("Unknown");
                let cpe = service_info
                    .cpe
                    .first()
                    .map(|s| s.as_str())
                    .unwrap_or("N/A");

                println!(
                    "{:5} | {:20} | {:30} | {}",
                    port, service_info.service, version, cpe
                );
            }
            Err(e) => {
                println!("{:5} | {:20} | {:30} | N/A", port, "error", e,);
            }
        }
    }

    println!("{:-<80}", "");
    println!("\nService detection complete!");

    Ok(())
}
