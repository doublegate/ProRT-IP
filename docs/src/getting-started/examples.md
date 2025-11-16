# Example Scans Gallery

Comprehensive collection of 65 runnable examples demonstrating ProRT-IP capabilities.

## Quick Navigation

- **[Tier 1: Feature-Complete Examples](#tier-1-feature-complete-examples)** (20 examples) - Production-ready use cases
- **[Tier 2: Focused Demonstrations](#tier-2-focused-demonstrations)** (30 examples) - Specific feature showcases
- **[Tier 3: Skeleton Templates](#tier-3-skeleton-templates)** (15 examples) - Development starting points

## How to Run Examples

### From Source

```bash
# Run a specific example
cargo run --example common_basic_syn_scan

# Run with release optimizations
cargo run --release --example performance_large_subnet

# Run with elevated privileges (for raw sockets)
sudo cargo run --example stealth_fin_scan
```

### From Installed Binary

```bash
# Most examples demonstrate CLI usage
prtip -sS -p 80,443 192.168.1.0/24

# See example comments for exact command
cat examples/common_basic_syn_scan.rs
```

## Categories

### Tier 1: Feature-Complete Examples (20)

Production-ready examples demonstrating complete use cases with error handling, logging, and best practices.

#### Common Use Cases (15)

| Example | Description | Difficulty | Privileges |
|---------|-------------|------------|------------|
| `common_basic_syn_scan.rs` | Simple TCP SYN scan of single host | Beginner | Root |
| `common_tcp_connect_scan.rs` | Full TCP handshake (no root needed) | Beginner | User |
| `common_subnet_scan.rs` | CIDR network scanning | Beginner | Root |
| `common_service_detection.rs` | Version detection on common ports | Intermediate | Root |
| `common_fast_scan.rs` | Top 100 ports scan (`-F` equivalent) | Beginner | Root |
| `common_os_fingerprinting.rs` | Operating system detection | Intermediate | Root |
| `common_udp_scan.rs` | UDP service discovery | Intermediate | Root |
| `common_stealth_scan.rs` | FIN/NULL/Xmas scan techniques | Advanced | Root |
| `common_idle_scan.rs` | Zombie host scanning | Advanced | Root |
| `common_web_server_scan.rs` | HTTP/HTTPS service analysis | Intermediate | Root |
| `common_database_scan.rs` | Database service detection | Intermediate | Root |
| `common_ssh_scan.rs` | SSH version enumeration | Beginner | User |
| `common_network_audit.rs` | Complete network security audit | Advanced | Root |
| `common_vulnerability_scan.rs` | Basic vulnerability detection | Advanced | Root |
| `common_compliance_scan.rs` | Compliance checking (PCI, HIPAA) | Advanced | Root |

#### Advanced Techniques (5)

| Example | Description | Difficulty | Privileges |
|---------|-------------|------------|------------|
| `advanced_decoy_scan.rs` | Decoy scanning with random IPs | Advanced | Root |
| `advanced_fragmentation.rs` | IP fragmentation evasion | Advanced | Root |
| `advanced_ttl_manipulation.rs` | TTL/hop limit manipulation | Advanced | Root |
| `advanced_source_port_spoofing.rs` | Source port 53 (DNS) spoofing | Advanced | Root |
| `advanced_combined_evasion.rs` | Multiple evasion techniques | Expert | Root |

---

### Tier 2: Focused Demonstrations (30)

Focused examples demonstrating specific features or techniques.

#### Scan Types (8)

| Example | Description | Scan Type |
|---------|-------------|-----------|
| `scan_types_syn.rs` | TCP SYN scan (half-open) | `-sS` |
| `scan_types_connect.rs` | TCP Connect scan (full handshake) | `-sT` |
| `scan_types_fin.rs` | FIN scan (stealth) | `-sF` |
| `scan_types_null.rs` | NULL scan (no flags) | `-sN` |
| `scan_types_xmas.rs` | Xmas scan (FIN+PSH+URG) | `-sX` |
| `scan_types_ack.rs` | ACK scan (firewall mapping) | `-sA` |
| `scan_types_udp.rs` | UDP scan | `-sU` |
| `scan_types_idle.rs` | Idle/Zombie scan | `-sI` |

#### Service Detection (5)

| Example | Description | Feature |
|---------|-------------|---------|
| `service_http_detection.rs` | HTTP server identification | HTTP probe |
| `service_ssh_banner.rs` | SSH banner grabbing | SSH probe |
| `service_tls_certificate.rs` | TLS certificate extraction | X.509 parsing |
| `service_mysql_version.rs` | MySQL version detection | MySQL probe |
| `service_smb_enumeration.rs` | SMB/CIFS enumeration | SMB probe |

#### Evasion Techniques (5)

| Example | Description | Evasion Type |
|---------|-------------|--------------|
| `evasion_timing_t0.rs` | Paranoid timing (5 min/probe) | T0 |
| `evasion_timing_t1.rs` | Sneaky timing (15 sec/probe) | T1 |
| `evasion_decoy_random.rs` | Random decoy IPs | `-D RND:N` |
| `evasion_fragmentation.rs` | 8-byte packet fragments | `-f` |
| `evasion_badsum.rs` | Invalid checksums | `--badsum` |

#### IPv6 Support (3)

| Example | Description | IPv6 Feature |
|---------|-------------|--------------|
| `ipv6_basic_scan.rs` | IPv6 address scanning | IPv6 support |
| `ipv6_ndp_discovery.rs` | Neighbor Discovery Protocol | NDP |
| `ipv6_icmpv6_scan.rs` | ICMPv6 Echo scanning | ICMPv6 |

#### Performance (4)

| Example | Description | Focus |
|---------|-------------|-------|
| `performance_large_subnet.rs` | /16 network (65K hosts) | Throughput |
| `performance_rate_limiting.rs` | Rate limiting demonstration | Courtesy |
| `performance_parallel_scanning.rs` | Parallel target scanning | Concurrency |
| `performance_adaptive_timing.rs` | Adaptive rate adjustment | Intelligence |

#### Output Formats (3)

| Example | Description | Format |
|---------|-------------|--------|
| `output_json.rs` | JSON output format | `-oJ` |
| `output_xml.rs` | XML output format | `-oX` |
| `output_greppable.rs` | Greppable output | `-oG` |

#### Plugin System (2)

| Example | Description | Plugin Type |
|---------|-------------|-------------|
| `plugin_custom_logger.rs` | Custom logging plugin | Lua integration |
| `plugin_vulnerability_check.rs` | Vulnerability scanner plugin | Security |

---

### Tier 3: Skeleton Templates (15)

Development starting points with TODO markers and architecture guidance.

#### Integration Examples (5)

| Example | Description | Integration |
|---------|-------------|-------------|
| `template_rest_api.rs` | REST API integration | HTTP endpoints |
| `template_database_storage.rs` | Database result storage | PostgreSQL/SQLite |
| `template_siem_integration.rs` | SIEM log forwarding | Syslog/CEF |
| `template_prometheus_metrics.rs` | Prometheus exporter | Metrics |
| `template_grafana_dashboard.rs` | Grafana visualization | Dashboards |

#### Custom Scanners (5)

| Example | Description | Scanner Type |
|---------|-------------|--------------|
| `template_custom_tcp.rs` | Custom TCP scanner | Protocol |
| `template_custom_udp.rs` | Custom UDP scanner | Protocol |
| `template_custom_icmp.rs` | Custom ICMP scanner | ICMP types |
| `template_application_scanner.rs` | Application-layer scanner | Layer 7 |
| `template_protocol_fuzzer.rs` | Protocol fuzzing scanner | Security |

#### Automation (5)

| Example | Description | Automation Type |
|---------|-------------|-----------------|
| `template_continuous_monitoring.rs` | Continuous network monitoring | Cron/systemd |
| `template_change_detection.rs` | Network change detection | Diff analysis |
| `template_alerting.rs` | Alert on specific conditions | Notifications |
| `template_reporting.rs` | Automated report generation | Reports |
| `template_workflow.rs` | Multi-stage scan workflow | Orchestration |

---

## Example Code Snippets

### Example 1: Basic SYN Scan

**File:** `examples/common_basic_syn_scan.rs`

```rust
use prtip_scanner::{Scanner, ScanConfig};
use std::net::IpAddr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure scan
    let config = ScanConfig::builder()
        .scan_type(ScanType::Syn)
        .ports(vec![80, 443])
        .timing(TimingTemplate::Normal)
        .build()?;

    // Create scanner
    let mut scanner = Scanner::new(config)?;

    // Target
    let target: IpAddr = "192.168.1.1".parse()?;

    // Initialize scanner (elevated privileges required)
    scanner.initialize().await?;

    // Execute scan
    let results = scanner.scan_target(target).await?;

    // Print results
    for result in results {
        println!("{:?}", result);
    }

    Ok(())
}
```

**Usage:**
```bash
sudo cargo run --example common_basic_syn_scan
```

### Example 2: Service Detection

**File:** `examples/common_service_detection.rs`

```rust
use prtip_scanner::{Scanner, ScanConfig, ServiceDetector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure scan with service detection
    let config = ScanConfig::builder()
        .scan_type(ScanType::Syn)
        .ports(vec![22, 80, 443, 3306, 5432])
        .enable_service_detection(true)
        .service_intensity(7)  // 0-9, higher = more accurate
        .build()?;

    let mut scanner = Scanner::new(config)?;
    scanner.initialize().await?;

    let target = "192.168.1.100".parse()?;
    let results = scanner.scan_target(target).await?;

    // Service detection results
    for result in results {
        if let Some(service) = result.service {
            println!("Port {}: {} {}",
                result.port,
                service.name,
                service.version.unwrap_or_default()
            );
        }
    }

    Ok(())
}
```

**Expected Output:**
```
Port 22: ssh OpenSSH 7.9p1
Port 80: http Apache httpd 2.4.41
Port 443: https Apache httpd 2.4.41 (SSL)
Port 3306: mysql MySQL 5.7.32
Port 5432: postgresql PostgreSQL 12.4
```

### Example 3: Stealth Scan with Evasion

**File:** `examples/advanced_combined_evasion.rs`

```rust
use prtip_scanner::{Scanner, ScanConfig, EvasionConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Maximum stealth configuration
    let evasion = EvasionConfig::builder()
        .decoys(vec!["192.168.1.10", "192.168.1.20", "ME", "192.168.1.30"])
        .fragmentation(true)
        .mtu(16)  // Fragment size
        .ttl(64)  // Normal TTL
        .source_port(53)  // DNS source port
        .bad_checksum(false)  // Don't use badsum (makes scan invalid)
        .build()?;

    let config = ScanConfig::builder()
        .scan_type(ScanType::Fin)  // Stealth scan
        .ports(vec![80, 443])
        .timing(TimingTemplate::Sneaky)  // T1
        .evasion(evasion)
        .build()?;

    let mut scanner = Scanner::new(config)?;
    scanner.initialize().await?;

    let target = "scanme.nmap.org".parse()?;
    let results = scanner.scan_target(target).await?;

    for result in results {
        println!("{:?}", result);
    }

    Ok(())
}
```

### Example 4: Large Subnet Scan

**File:** `examples/performance_large_subnet.rs`

```rust
use prtip_scanner::{Scanner, ScanConfig};
use ipnetwork::IpNetwork;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure for large-scale scanning
    let config = ScanConfig::builder()
        .scan_type(ScanType::Syn)
        .ports(vec![80, 443])  // Limited ports for speed
        .timing(TimingTemplate::Aggressive)  // T4
        .max_rate(10000)  // 10K packets/second
        .parallelism(1000)  // 1000 concurrent targets
        .build()?;

    let mut scanner = Scanner::new(config)?;
    scanner.initialize().await?;

    // Scan /16 network (65,536 hosts)
    let network: IpNetwork = "10.0.0.0/16".parse()?;

    println!("Scanning {} hosts...", network.size());
    let start = std::time::Instant::now();

    let results = scanner.scan_network(network).await?;

    let duration = start.elapsed();
    println!("Scan complete in {:?}", duration);
    println!("Found {} open ports", results.len());
    println!("Throughput: {:.2} ports/sec",
        results.len() as f64 / duration.as_secs_f64());

    Ok(())
}
```

### Example 5: Custom Plugin

**File:** `examples/plugin_custom_logger.rs`

**Lua Plugin:** `plugins/custom-logger.lua`
```lua
local log_file = nil

return {
    name = "Custom CSV Logger",
    version = "1.0.0",
    description = "Logs scan results to CSV format",

    init = function(config)
        log_file = io.open("scan-results.csv", "w")
        log_file:write("Timestamp,IP,Port,State,Service,Version\n")
    end,

    process = function(result)
        local timestamp = os.date("%Y-%m-%d %H:%M:%S")
        log_file:write(string.format("%s,%s,%d,%s,%s,%s\n",
            timestamp,
            result.ip,
            result.port,
            result.state,
            result.service or "",
            result.version or ""))
        log_file:flush()
    end,

    cleanup = function()
        if log_file then
            log_file:close()
        end
    end
}
```

**Rust Code:**
```rust
use prtip_scanner::{Scanner, ScanConfig, PluginManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load plugin
    let plugin_manager = PluginManager::new()?;
    plugin_manager.load_plugin("plugins/custom-logger.lua")?;

    // Configure scan
    let config = ScanConfig::builder()
        .scan_type(ScanType::Syn)
        .ports(vec![80, 443, 22, 3389])
        .plugin_manager(plugin_manager)
        .build()?;

    let mut scanner = Scanner::new(config)?;
    scanner.initialize().await?;

    // Scan network
    let network = "192.168.1.0/24".parse()?;
    let results = scanner.scan_network(network).await?;

    println!("Results logged to scan-results.csv");
    println!("Total results: {}", results.len());

    Ok(())
}
```

**Output:** `scan-results.csv`
```csv
Timestamp,IP,Port,State,Service,Version
2024-11-15 10:30:15,192.168.1.1,80,open,http,Apache 2.4.41
2024-11-15 10:30:15,192.168.1.1,443,open,https,Apache 2.4.41
2024-11-15 10:30:16,192.168.1.10,22,open,ssh,OpenSSH 7.9
2024-11-15 10:30:17,192.168.1.100,3389,open,rdp,Microsoft RDP
```

---

## Running Examples by Category

### Quick Scans (< 1 minute)

```bash
# Single host, common ports
cargo run --example common_basic_syn_scan

# Fast scan (top 100 ports)
cargo run --example common_fast_scan

# Service detection on few ports
cargo run --example service_http_detection
```

### Medium Scans (1-10 minutes)

```bash
# Subnet scan (/24)
cargo run --example common_subnet_scan

# Service detection on network
cargo run --example common_service_detection

# OS fingerprinting
cargo run --example common_os_fingerprinting
```

### Long Scans (> 10 minutes)

```bash
# Large subnet (/16)
cargo run --release --example performance_large_subnet

# Comprehensive network audit
cargo run --release --example common_network_audit

# Stealth scan with slow timing
sudo cargo run --example evasion_timing_t0
```

### Stealth Scans

```bash
# FIN scan
cargo run --example scan_types_fin

# Decoy scanning
cargo run --example advanced_decoy_scan

# Combined evasion
cargo run --example advanced_combined_evasion
```

### IPv6 Examples

```bash
# Basic IPv6 scan
cargo run --example ipv6_basic_scan

# NDP discovery
cargo run --example ipv6_ndp_discovery

# ICMPv6 scan
cargo run --example ipv6_icmpv6_scan
```

---

## Example Output Formats

### Normal Output

```
Starting ProRT-IP v0.5.2 ( https://github.com/doublegate/ProRT-IP )
Scan report for 192.168.1.1
Host is up (0.0012s latency).

PORT     STATE  SERVICE    VERSION
22/tcp   open   ssh        OpenSSH 7.9p1 Debian 10+deb10u2
80/tcp   open   http       Apache httpd 2.4.41 ((Debian))
443/tcp  open   ssl/http   Apache httpd 2.4.41 ((Debian))
3306/tcp open   mysql      MySQL 5.7.32-0ubuntu0.18.04.1

Scan complete: 4 ports scanned, 4 open, 0 closed, 0 filtered
```

### JSON Output

```json
{
  "scan_info": {
    "version": "0.5.2",
    "scan_type": "syn",
    "start_time": "2024-11-15T10:30:00Z",
    "end_time": "2024-11-15T10:30:15Z",
    "duration_seconds": 15
  },
  "targets": [
    {
      "ip": "192.168.1.1",
      "hostname": "router.local",
      "state": "up",
      "latency_ms": 1.2,
      "ports": [
        {
          "port": 80,
          "protocol": "tcp",
          "state": "open",
          "service": {
            "name": "http",
            "product": "Apache httpd",
            "version": "2.4.41",
            "extra_info": "(Debian)"
          }
        }
      ]
    }
  ]
}
```

### XML Output (Nmap Compatible)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE nmaprun>
<nmaprun scanner="prtip" args="-sS -p 80,443" start="1700053800" version="0.5.2">
  <scaninfo type="syn" protocol="tcp" numservices="2" services="80,443"/>
  <host starttime="1700053800" endtime="1700053815">
    <status state="up" reason="echo-reply"/>
    <address addr="192.168.1.1" addrtype="ipv4"/>
    <ports>
      <port protocol="tcp" portid="80">
        <state state="open" reason="syn-ack"/>
        <service name="http" product="Apache httpd" version="2.4.41"/>
      </port>
      <port protocol="tcp" portid="443">
        <state state="open" reason="syn-ack"/>
        <service name="https" product="Apache httpd" version="2.4.41" tunnel="ssl"/>
      </port>
    </ports>
  </host>
  <runstats>
    <finished time="1700053815" elapsed="15"/>
    <hosts up="1" down="0" total="1"/>
  </runstats>
</nmaprun>
```

---

## Next Steps

After exploring these examples:

1. **[Read the Tutorials](./tutorials.md)** - Step-by-step learning path
2. **[Explore the User Guide](../user-guide/basic-usage.md)** - Comprehensive usage documentation
3. **[Review Feature Guides](../features/)** - Deep dives into specific features
4. **[Study Advanced Topics](../advanced/)** - Performance tuning and optimization

## Contributing Examples

Have a useful example? Contribute it!

1. Fork the repository
2. Add your example to `examples/`
3. Add entry to this gallery
4. Submit pull request

**Example Contribution Guidelines:**

- Include comprehensive comments
- Add error handling
- Follow Rust best practices
- Test on multiple platforms
- Document expected output
- Specify required privileges

See [Contributing Guide](../development/contributing.md) for details.

---

**Last Updated:** 2024-11-15
**Examples Count:** 65 total (20 complete, 30 focused, 15 templates)
