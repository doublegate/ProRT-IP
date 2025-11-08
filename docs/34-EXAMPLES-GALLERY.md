# ProRT-IP Examples Gallery

**65 Runnable Examples** | **Last Updated:** 2025-11-07 | **Sprint 5.5.1 Task 2**

## Overview

This gallery provides copy-paste ready Rust code examples demonstrating all ProRT-IP features. Each example is a complete, compilable program showcasing specific functionality.

## Quick Start

```bash
# List all examples
cargo build --package prtip-scanner --examples

# Run an example (most require root/CAP_NET_RAW)
sudo cargo run --package prtip-scanner --example common_basic_syn_scan

# Run without root (TCP Connect scan)
cargo run --package prtip-scanner --example common_tcp_connect_scan
```

## Categories

### Tier 1: Feature-Complete Examples (20)

Production-ready examples with comprehensive error handling.

#### Common Use Cases (15)

| Example | Description | Difficulty |
|---------|-------------|------------|
| `common_basic_syn_scan.rs` | Simple TCP SYN scan of single host | Beginner |
| `common_tcp_connect_scan.rs` | Full TCP handshake (no root needed) | Beginner |
| `common_subnet_scan.rs` | CIDR network scanning | Beginner |
| `common_service_detection.rs` | Version detection on common ports | Intermediate |
| `common_output_formats.rs` | JSON, XML, Greppable formats | Beginner |
| `common_fast_scan.rs` | Top 100 ports, T4 timing | Beginner |
| `common_stealth_scan.rs` | FIN/NULL/Xmas combinations | Intermediate |
| `common_udp_scan.rs` | DNS, SNMP, NetBIOS discovery | Intermediate |
| `common_host_discovery.rs` | ICMP + TCP ping combinations | Intermediate |
| `common_rate_limited.rs` | Polite mode (T0-T2 timing) | Beginner |
| `common_target_specification.rs` | Ranges, files, exclusions | Beginner |
| `common_timing_templates.rs` | T0→T4 comparisons | Beginner |
| `common_config_files.rs` | TOML configuration loading | Intermediate |
| `common_database_storage.rs` | SQLite result queries | Intermediate |
| `common_ipv4_ipv6_dual.rs` | Dual-stack scanning | Beginner |

#### Advanced Features (5)

| Example | Description | Difficulty |
|---------|-------------|------------|
| `advanced_ipv6_scanning.rs` | Link-local, NDP, ICMPv6 | Advanced |
| `advanced_idle_scan.rs` | Zombie discovery, anonymity | Advanced |
| `advanced_tls_certificate.rs` | X.509v3 parsing, SNI | Intermediate |
| `advanced_lua_plugins.rs` | Custom protocol detection | Advanced |
| `advanced_evasion_techniques.rs` | Fragmentation, decoys | Advanced |

### Tier 2: Focused Demonstrations (30)

Single-purpose API showcases (50-100 lines).

#### Service Detection (6)

| Example | Feature |
|---------|---------|
| `service_http_fingerprint.rs` | HTTP server detection |
| `service_ssh_version.rs` | SSH banner grabbing |
| `service_tls_analysis.rs` | TLS version/cipher detection |
| `service_custom_probes.rs` | Protocol-specific payloads |
| `service_os_fingerprint.rs` | TCP/IP stack analysis |
| `service_banner_grabbing.rs` | Multi-protocol banners |

#### Rate Limiting & Performance (6)

| Example | Feature |
|---------|---------|
| `rate_adaptive_limiting.rs` | V3 algorithm (-1.8% overhead) |
| `rate_bandwidth_throttle.rs` | Network bandwidth control |
| `perf_numa_optimization.rs` | Thread pinning, IRQ affinity |
| `perf_batch_sizes.rs` | Optimal batch tuning |
| `perf_zero_copy.rs` | Zero-copy optimization |
| `perf_ulimit_tuning.rs` | Resource limit handling |

#### Output & Integration (6)

| Example | Feature |
|---------|---------|
| `output_pcapng_export.rs` | Wireshark-compatible capture |
| `output_structured_logging.rs` | JSON audit trails |
| `integration_cicd_pipeline.rs` | GitHub Actions usage |
| `integration_prometheus.rs` | Metrics export |
| `integration_siem.rs` | Splunk/ELK integration |
| `integration_docker.rs` | Container scanning |

#### Error Handling (6)

| Example | Scenario |
|---------|----------|
| `error_offline_targets.rs` | Timeout handling |
| `error_firewalled_hosts.rs` | Filtered port interpretation |
| `error_rate_limited_targets.rs` | Adaptive backoff |
| `error_resource_exhaustion.rs` | Ulimit errors |
| `error_permission_denied.rs` | CAP_NET_RAW missing |
| `error_malformed_responses.rs` | Banner parsing robustness |

#### Multi-Stage Workflows (6)

| Example | Workflow |
|---------|----------|
| `workflow_discovery_enumeration.rs` | Two-stage scan |
| `workflow_vulnerability_scanning.rs` | Scan → detect → report |
| `workflow_asset_inventory.rs` | Periodic scans |
| `workflow_network_mapping.rs` | Topology discovery |
| `workflow_compliance_audit.rs` | Policy validation |
| `workflow_penetration_testing.rs` | Recon automation |

### Tier 3: Skeleton Templates (15)

Compilable templates with TODO comments for user extension.

#### Templates (10)

| Example | Purpose |
|---------|---------|
| `template_custom_scanner.rs` | Build custom scanner |
| `template_protocol_handler.rs` | Custom protocol detection |
| `template_output_formatter.rs` | Custom output format |
| `template_plugin_development.rs` | Lua plugin skeleton |
| `template_distributed_scanning.rs` | Multi-node coordination |
| `template_resume_capability.rs` | State persistence |
| `template_web_ui_integration.rs` | REST API integration |
| `template_cloud_scanning.rs` | AWS VPC / Azure VNET |
| `template_compliance_reporting.rs` | PCI-DSS / HIPAA |
| `template_threat_intelligence.rs` | IOC integration |

#### Bonus Examples (5)

| Example | Feature |
|---------|---------|
| `bonus_masscan_comparison.rs` | Speed benchmark |
| `bonus_nmap_compatibility.rs` | Flag translation |
| `bonus_realtime_dashboard.rs` | Live metrics display |
| `bonus_machine_learning.rs` | Service prediction |
| `bonus_blockchain_audit.rs` | Smart contract scanning |

## How to Run Examples

### Prerequisites

Most examples require:
- Root privileges or `CAP_NET_RAW` capability
- Network access to target
- Rust 1.70+ (for compilation)

### Running with Root

```bash
sudo cargo run --package prtip-scanner --example common_basic_syn_scan
```

### Running without Root

Use TCP Connect scan examples:
```bash
cargo run --package prtip-scanner --example common_tcp_connect_scan
```

### Running Specific Example

```bash
# Pattern: cargo run --package prtip-scanner --example <name>
cargo run --package prtip-scanner --example common_service_detection
```

## Troubleshooting

### Permission Denied

**Error:** `Operation not permitted`

**Solution:** Run with `sudo` or grant `CAP_NET_RAW`:
```bash
sudo setcap cap_net_raw+ep target/debug/examples/common_basic_syn_scan
```

### Compilation Errors

**Error:** Examples fail to compile

**Solution:** Rebuild from project root:
```bash
cargo clean
cargo build --package prtip-scanner --examples
```

### No Results

**Error:** Scan completes but finds no open ports

**Check:**
- Target is reachable: `ping <target>`
- Firewall allows scanning
- Services are actually running on target ports

## Example Structure

All examples follow this pattern:

```rust
//! Example: [Name]
//!
//! Demonstrates: [Feature/Capability]
//!
//! ## Prerequisites
//! - [Requirements]
//!
//! ## Usage
//! ```bash
//! cargo run --example [name]
//! ```
//!
//! ## Expected Output
//! [Description]

use prtip_core::{Config, ScanConfig};
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example implementation
    Ok(())
}
```

## API Patterns

### Scanner Initialization

```rust
// SYN Scanner
let config = Config { scan: ScanConfig::default(), ..Config::default() };
let scanner = SynScanner::new(config)?;

// TCP Connect Scanner
let scanner = TcpConnectScanner::new(Duration::from_secs(2), 1);

// UDP Scanner
let scanner = UdpScanner::new(config)?;
```

### Scanning Ports

```rust
// Scan multiple ports
let results = scanner.scan_ports(target, vec![80, 443]).await?;

// Scan single port (UDP/Stealth)
let result = scanner.scan_port(target, 80).await?;
```

### Timing Templates

```rust
use prtip_core::TimingTemplate;

let mut scan_config = ScanConfig::default();
scan_config.timing_template = TimingTemplate::Aggressive;  // T4
```

## Contributing

Want to add your own example?

1. Create `crates/prtip-scanner/examples/your_example.rs`
2. Follow the example structure pattern above
3. Test compilation: `cargo build --package prtip-scanner --example your_example`
4. Submit PR with description

## Related Documentation

- [Architecture](00-ARCHITECTURE.md) - System design
- [Implementation Guide](04-IMPLEMENTATION-GUIDE.md) - Code structure
- [Testing](06-TESTING.md) - Testing strategy
- [Security](08-SECURITY.md) - Security best practices

## Support

- **GitHub Issues:** https://github.com/doublegate/ProRT-IP/issues
- **Documentation:** All guides in `docs/`
- **Examples Source:** `crates/prtip-scanner/examples/`

---

**Total Examples:** 65 | **Tier 1:** 20 | **Tier 2:** 30 | **Tier 3:** 15 | **Sprint 5.5.1 Complete**
