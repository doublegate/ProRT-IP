#![allow(
    clippy::field_reassign_with_default,
    clippy::useless_vec,
    clippy::to_string_in_format_args
)]
//! Example: Configuration Files (TOML)
//!
//! Demonstrates: Loading scan configuration from TOML files
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example common_config_files
//! ```
//!
//! ## Expected Output
//! - Configuration loaded from TOML
//! - Scan performed with config settings
//! - Serialization/deserialization examples

use prtip_core::{Config, ScanConfig, TimingTemplate};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Configuration Files Example ===\n");

    // Example 1: Create a configuration
    let scan_config = ScanConfig {
        timing_template: TimingTemplate::Aggressive,
        timeout_ms: 2000,
        retries: 2,
        ..Default::default()
    };

    let config = Config {
        scan: scan_config,
        ..Config::default()
    };

    // Example 2: Serialize to TOML
    let toml_string = toml::to_string(&config)?;
    println!("Configuration as TOML:");
    println!("{:-<60}", "");
    println!("{}", toml_string);
    println!("{:-<60}", "");

    // Example 3: Deserialize from TOML
    let toml_config = r#"
[scan]
scan_type = "Syn"
timing_template = "Aggressive"
timeout_ms = 2000
retries = 2
scan_delay_ms = 0
max_retransmits = 3
min_rate_pps = 100
max_rate_pps = 10000

[database]
enabled = false
path = ":memory:"

[output]
format = "Text"
verbosity = 1
color = false
"#;

    let loaded_config: Config = toml::from_str(toml_config)?;
    println!("\nLoaded configuration from TOML:");
    println!("  Timing: {:?}", loaded_config.scan.timing_template);
    println!("  Timeout: {}ms", loaded_config.scan.timeout_ms);
    println!("  Retries: {}", loaded_config.scan.retries);

    // Example 4: Use the loaded configuration
    println!("\nPerforming scan with loaded configuration...\n");

    let target: IpAddr = "127.0.0.1".parse()?;
    let ports = vec![80, 443];

    let scanner = SynScanner::new(loaded_config)?;
    let results = scanner.scan_ports(target, ports.clone()).await?;

    for result in results {
        println!("Port {:5}: {:?}", result.port, result.state);
    }

    println!("\nNote: In production:");
    println!("  - Store configs in ~/.config/prtip/config.toml");
    println!("  - Support multiple profiles (fast, stealth, polite)");
    println!("  - Validate configs before use");
    println!("  - Use std::fs::read_to_string() to load from file");

    Ok(())
}
