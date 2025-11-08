#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: Lua Plugin System
//!
//! Demonstrates: Custom protocol detection via Lua plugins
//!
//! ## Prerequisites
//! - Network access to target
//!
//! ## Usage
//! ```bash
//! cargo run --example advanced_lua_plugins
//! ```

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Lua Plugin System ===\n");

    println!("Lua plugin system enables custom protocol detection and extension.");
    println!("Sprint 5.8 added full plugin infrastructure.\n");

    // Example Lua script for HTTP detection
    let lua_script = r#"
-- Custom HTTP detection plugin
function detect_http(banner)
    if banner:match("^HTTP/") then
        local version = banner:match("HTTP/(%d%.%d)")
        return { service = "http", version = version }
    end
    return nil
end
"#;

    println!("Example Lua Plugin:");
    println!("{}", lua_script);

    println!("\nPlugin Features:");
    println!("  - Lua 5.4 runtime");
    println!("  - Sandboxed execution");
    println!("  - Capabilities-based security");
    println!("  - Hot reload support");
    println!("  - Three plugin types:");
    println!("    * Protocol detection");
    println!("    * Banner parsing");
    println!("    * Custom output formatting");

    println!("\nPlugin API:");
    println!("  - ctx.send(data) - Send bytes to service");
    println!("  - ctx.recv(timeout) - Receive response");
    println!("  - ctx.log(level, message) - Logging");
    println!("  - Return structured results");

    println!("\nSee docs/30-PLUGIN-SYSTEM.md for full guide");

    // Perform sample scan
    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };
    let scanner = SynScanner::new(config)?;
    let target: IpAddr = "127.0.0.1".parse()?;
    let results = scanner.scan_ports(target, vec![80]).await?;

    println!("\nScan result:");
    for r in results {
        println!("  Port {:5}: {:?}", r.port, r.state);
    }

    Ok(())
}
