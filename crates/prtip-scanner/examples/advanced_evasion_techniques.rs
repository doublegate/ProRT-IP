#![allow(clippy::field_reassign_with_default, clippy::useless_vec, clippy::to_string_in_format_args)]
//! Example: Firewall Evasion Techniques
//!
//! Demonstrates: Packet fragmentation, bad checksums, decoy scanning
//!
//! ## Prerequisites
//! - Root privileges or CAP_NET_RAW capability
//!
//! ## Usage
//! ```bash
//! sudo cargo run --example advanced_evasion_techniques
//! ```

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::{IpAddr, Ipv4Addr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProRT-IP: Firewall Evasion Techniques ===\n");

    let target: IpAddr = "127.0.0.1".parse()?;
    let _ports = [80, 443]; // Reserved for future multi-port example

    // Technique 1: Decoy Scanning
    println!("=== Technique 1: Decoy Scanning ===");
    println!("Purpose: Hide scanner IP among decoy addresses");

    let decoys = vec![
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 30)),
    ];

    println!("Decoys: {:?}", decoys);
    println!("Note: DecoyScanner requires special configuration.");
    println!("See source code for implementation details.\n");

    let config = Config {
        scan: ScanConfig::default(),
        ..Config::default()
    };

    // Demonstrate normal scan
    let scanner = SynScanner::new(config)?;
    println!("Performing normal scan for comparison:");
    let results = scanner.scan_ports(target, vec![80]).await?;
    for r in results {
        println!("  Port {:5}: {:?}\n", r.port, r.state);
    }

    // Technique 2: Timing Manipulation
    println!("=== Technique 2: Timing Manipulation ===");
    println!("Purpose: Evade rate-based IDS detection");
    println!("  - Randomize packet intervals");
    println!("  - Use paranoid timing (T0)");
    println!("  - Adaptive delays based on responses\n");

    // Technique 3: Packet Fragmentation
    println!("=== Technique 3: Packet Fragmentation ===");
    println!("Purpose: Bypass simple packet filters");
    println!("  - Fragment TCP header across packets");
    println!("  - Use tiny fragments (8 bytes)");
    println!("  - Overlap fragments for filter evasion\n");

    // Technique 4: Bad Checksums
    println!("=== Technique 4: Invalid Checksums ===");
    println!("Purpose: Detect firewall responses vs real host");
    println!("  - Send packets with incorrect TCP checksum");
    println!("  - Real hosts drop packets");
    println!("  - Firewalls may respond without validating\n");

    // Technique 5: Source Port Manipulation
    println!("=== Technique 5: Source Port Manipulation ===");
    println!("Purpose: Bypass port-based filtering");
    println!("  - Use port 53 (DNS) as source");
    println!("  - Use port 80 (HTTP) as source");
    println!("  - Many firewalls allow traffic from these ports\n");

    println!("Note: ProRT-IP implements 6 evasion techniques:");
    println!("  1. Packet fragmentation (--f, --mtu)");
    println!("  2. Decoy scanning (-D)");
    println!("  3. Bad checksums (--badsum)");
    println!("  4. Source port manipulation (-g)");
    println!("  5. TTL manipulation (--ttl)");
    println!("  6. Timing randomization");

    println!("\nLegal Notice:");
    println!("  Use evasion techniques only with explicit authorization.");
    println!("  Unauthorized network scanning may be illegal.");

    Ok(())
}
