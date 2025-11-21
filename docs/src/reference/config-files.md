# Configuration Files Reference

This document provides a complete reference for ProRT-IP's TOML-based configuration system, including all available sections, options, default values, and validation rules.

## Configuration File Locations

ProRT-IP searches for configuration files in the following order (later files override earlier):

| Priority | Location | Description |
|----------|----------|-------------|
| 1 | `/etc/prtip/config.toml` | System-wide configuration |
| 2 | `~/.config/prtip/config.toml` | User configuration |
| 3 | `~/.prtip/config.toml` | Alternative user location |
| 4 | `./prtip.toml` | Project-specific configuration |
| 5 | CLI flags | Highest priority (always wins) |

## Complete Configuration Example

```toml
# ProRT-IP Configuration File
# All values shown are defaults unless noted otherwise

[scan]
scan_type = "Connect"           # Connect, Syn, Fin, Null, Xmas, Ack, Udp, Idle
timing_template = "Normal"      # Paranoid, Sneaky, Polite, Normal, Aggressive, Insane
timeout_ms = 1000               # Probe timeout (1-3600000 ms)
retries = 0                     # Retry count (0-10)
scan_delay_ms = 0               # Delay between probes
host_delay_ms = 0               # Delay between hosts
progress = false                # Show progress bar

[scan.service_detection]
enabled = false                 # Enable service detection
intensity = 7                   # Detection intensity (0-9)
banner_grab = false             # Grab service banners
probe_db_path = ""              # Custom probe database path
enable_tls = true               # TLS/SSL detection
capture_raw = false             # Capture raw responses

[network]
interface = ""                  # Network interface (empty = auto-detect)
source_port = 0                 # Source port (0 = random)
skip_cdn = false                # Skip CDN IP addresses
cdn_whitelist = []              # Only skip these CDN providers
cdn_blacklist = []              # Never skip these CDN providers

[output]
format = "Text"                 # Text, Json, Xml, Greppable
file = ""                       # Output file (empty = stdout)
verbose = 0                     # Verbosity level (0-3)

[performance]
max_rate = 0                    # Max packets/sec (0 = unlimited)
parallelism = 0                 # Concurrent connections (0 = auto/CPU cores)
batch_size = 0                  # Connection pool batch (0 = auto)
requested_ulimit = 0            # Requested file descriptor limit
numa_enabled = false            # NUMA optimization (Linux only)
adaptive_batch_enabled = false  # Adaptive batch sizing
min_batch_size = 16             # Minimum batch size (1-1024)
max_batch_size = 256            # Maximum batch size (1-1024)

[evasion]
fragment_packets = false        # Enable packet fragmentation
mtu = 0                         # Custom MTU (0 = default, ≥68, multiple of 8)
ttl = 0                         # Custom TTL (0 = OS default ~64)
bad_checksums = false           # Use invalid checksums

[evasion.decoys]
# Random decoys: generates N random IPs
type = "random"
count = 5                       # Number of decoy IPs
me_position = 0                 # Real IP position (0 = append at end)

# OR Manual decoys: specific IP addresses
# type = "manual"
# ips = ["10.0.0.1", "10.0.0.2", "10.0.0.3"]
# me_position = 2               # Real IP at position 2
```

## Configuration Sections

### [scan] - Scan Configuration

Controls the scanning behavior and probe settings.

| Field | Type | Default | Range | Description |
|-------|------|---------|-------|-------------|
| `scan_type` | String | `"Connect"` | See enum | Type of port scan |
| `timing_template` | String | `"Normal"` | See enum | Timing profile (T0-T5) |
| `timeout_ms` | Integer | `1000` | 1-3,600,000 | Probe timeout in milliseconds |
| `retries` | Integer | `0` | 0-10 | Number of retries per probe |
| `scan_delay_ms` | Integer | `0` | ≥0 | Delay between probes (ms) |
| `host_delay_ms` | Integer | `0` | ≥0 | Delay between hosts (ms) |
| `progress` | Boolean | `false` | - | Display progress bar |

#### scan_type Values

| Value | CLI Flag | Description | Privileges |
|-------|----------|-------------|------------|
| `"Connect"` | `-sT` | Full TCP 3-way handshake | None |
| `"Syn"` | `-sS` | Half-open SYN scan | Root/Admin |
| `"Fin"` | `-sF` | TCP FIN scan (stealth) | Root/Admin |
| `"Null"` | `-sN` | TCP NULL scan (no flags) | Root/Admin |
| `"Xmas"` | `-sX` | TCP Xmas (FIN+PSH+URG) | Root/Admin |
| `"Ack"` | `-sA` | TCP ACK (firewall detection) | Root/Admin |
| `"Udp"` | `-sU` | UDP scan | Root/Admin |
| `"Idle"` | `-sI` | Idle/zombie scan | Root/Admin |

#### timing_template Values

| Value | CLI | Timeout | Delay | Parallelism | Use Case |
|-------|-----|---------|-------|-------------|----------|
| `"Paranoid"` | `-T0` | 300,000ms | 300,000ms | 1 | IDS evasion |
| `"Sneaky"` | `-T1` | 15,000ms | 15,000ms | 10 | Low-profile |
| `"Polite"` | `-T2` | 10,000ms | 400ms | 100 | Bandwidth-limited |
| `"Normal"` | `-T3` | 3,000ms | 0ms | 1,000 | Default |
| `"Aggressive"` | `-T4` | 1,000ms | 0ms | 5,000 | Fast networks |
| `"Insane"` | `-T5` | 250ms | 0ms | 10,000 | Maximum speed |

### [scan.service_detection] - Service Detection

Controls service/version detection behavior.

| Field | Type | Default | Range | Description |
|-------|------|---------|-------|-------------|
| `enabled` | Boolean | `false` | - | Enable service detection |
| `intensity` | Integer | `7` | 0-9 | Detection thoroughness |
| `banner_grab` | Boolean | `false` | - | Grab service banners |
| `probe_db_path` | String | `""` | - | Custom probe database |
| `enable_tls` | Boolean | `true` | - | TLS/SSL detection |
| `capture_raw` | Boolean | `false` | - | Capture raw responses |

**Intensity Levels:**

| Level | Description | Probes | Speed |
|-------|-------------|--------|-------|
| 0 | Minimal | ~10 | Fastest |
| 1-3 | Light | ~30 | Fast |
| 4-6 | Standard | ~60 | Normal |
| 7 | Default | ~100 | Balanced |
| 8-9 | Comprehensive | ~187 | Thorough |

### [network] - Network Configuration

Controls network interface and CDN handling.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `interface` | String | `""` | Network interface (empty = auto-detect) |
| `source_port` | Integer | `0` | Source port (0 = random) |
| `skip_cdn` | Boolean | `false` | Skip scanning CDN IPs |
| `cdn_whitelist` | Array | `[]` | Only skip these providers |
| `cdn_blacklist` | Array | `[]` | Never skip these providers |

**CDN Provider Names:**

```toml
# Available CDN providers for whitelist/blacklist
cdn_whitelist = ["cloudflare", "akamai", "fastly", "cloudfront", "azure", "gcp"]
cdn_blacklist = ["akamai"]  # Never skip Akamai even with skip_cdn = true
```

**CDN Configuration Examples:**

```toml
# Skip all known CDN IPs (80-100% scan reduction)
[network]
skip_cdn = true

# Skip only Cloudflare and Fastly
[network]
skip_cdn = true
cdn_whitelist = ["cloudflare", "fastly"]

# Skip all CDNs except Azure
[network]
skip_cdn = true
cdn_blacklist = ["azure"]
```

### [output] - Output Configuration

Controls output format and destination.

| Field | Type | Default | Range | Description |
|-------|------|---------|-------|-------------|
| `format` | String | `"Text"` | See enum | Output format |
| `file` | String | `""` | - | Output file path |
| `verbose` | Integer | `0` | 0-3 | Verbosity level |

#### format Values

| Value | CLI Flag | Description |
|-------|----------|-------------|
| `"Text"` | `-oN` | Human-readable colorized text |
| `"Json"` | `-oJ` | JSON format |
| `"Xml"` | `-oX` | Nmap-compatible XML |
| `"Greppable"` | `-oG` | Greppable single-line format |

#### verbose Levels

| Level | CLI | Description |
|-------|-----|-------------|
| 0 | (default) | Normal output |
| 1 | `-v` | Show filtered/closed ports |
| 2 | `-vv` | Debug information |
| 3 | `-vvv` | Trace-level details |

### [performance] - Performance Configuration

Controls scan speed and resource usage.

| Field | Type | Default | Range | Description |
|-------|------|---------|-------|-------------|
| `max_rate` | Integer | `0` | 0-100,000,000 | Max packets/sec (0 = unlimited) |
| `parallelism` | Integer | Auto | 0-100,000 | Concurrent connections |
| `batch_size` | Integer | `0` | ≥0 | Connection pool batch size |
| `requested_ulimit` | Integer | `0` | ≥0 | Requested file descriptor limit |
| `numa_enabled` | Boolean | `false` | - | NUMA optimization (Linux) |
| `adaptive_batch_enabled` | Boolean | `false` | - | Adaptive batch sizing |
| `min_batch_size` | Integer | `16` | 1-1024 | Minimum batch size |
| `max_batch_size` | Integer | `256` | 1-1024 | Maximum batch size |

**Parallelism:**

- `0` = Auto-detect based on CPU cores
- Values > 0 = Explicit concurrent connection limit

**Batch Configuration:**

```toml
[performance]
# Optimal batch settings (from Sprint 6.3 benchmarks)
adaptive_batch_enabled = true
min_batch_size = 16    # 94% syscall reduction
max_batch_size = 256   # 99.6% syscall reduction, L3 cache friendly
```

**NUMA Optimization (Linux Multi-Socket Systems):**

```toml
[performance]
numa_enabled = true    # Enable NUMA-aware memory allocation
```

### [evasion] - Evasion Configuration

Controls stealth and evasion techniques.

| Field | Type | Default | Range | Description |
|-------|------|---------|-------|-------------|
| `fragment_packets` | Boolean | `false` | - | Enable IP fragmentation |
| `mtu` | Integer | `0` | 0 or ≥68, mod 8 | Custom MTU (0 = default) |
| `ttl` | Integer | `0` | 0-255 | Custom TTL (0 = OS default) |
| `bad_checksums` | Boolean | `false` | - | Send invalid checksums |

**Fragmentation:**

```toml
[evasion]
fragment_packets = true  # Fragment TCP/UDP packets
mtu = 576               # Custom MTU (must be ≥68 and multiple of 8)
```

**TTL Control:**

```toml
[evasion]
ttl = 32   # Short TTL to evade distant firewalls
```

### [evasion.decoys] - Decoy Configuration

Configure decoy scanning (Nmap `-D` equivalent).

**Random Decoys:**

```toml
[evasion.decoys]
type = "random"
count = 5           # Generate 5 random decoy IPs
me_position = 3     # Real IP at position 3 (0 = append at end)
```

**Manual Decoys:**

```toml
[evasion.decoys]
type = "manual"
ips = ["192.168.1.10", "192.168.1.20", "192.168.1.30"]
me_position = 2     # Real IP at position 2
```

## Validation Rules

ProRT-IP validates configuration files when loaded. Invalid configurations produce clear error messages:

| Field | Validation Rule | Error Message |
|-------|-----------------|---------------|
| `timeout_ms` | 1-3,600,000 | "timeout_ms must be greater than 0" / "cannot exceed 1 hour" |
| `retries` | 0-10 | "retries cannot exceed 10" |
| `parallelism` | 0-100,000 | "parallelism cannot exceed 100,000" |
| `max_rate` | 0 or 1-100,000,000 | "max_rate must be greater than 0" / "cannot exceed 100M pps" |
| `mtu` | 0 or ≥68, mod 8 | "MTU must be at least 68 and a multiple of 8" |
| `intensity` | 0-9 | "intensity must be 0-9" |

### Example Validation Error

```bash
$ prtip --config invalid.toml 192.168.1.1
Error: Configuration validation failed
  Caused by: timeout_ms cannot exceed 1 hour (3600000 ms)
```

## Loading Configuration Programmatically

```rust
use prtip_core::config::Config;
use std::path::Path;

// Load from file
let config = Config::load_from_file(Path::new("prtip.toml"))?;

// Load from string
let toml_str = r#"
    [scan]
    scan_type = "Syn"
    timing_template = "Aggressive"

    [performance]
    max_rate = 10000
"#;
let config = Config::load_from_str(toml_str)?;

// Save to file
config.save_to_file(Path::new("output.toml"))?;
```

## Profile Configurations

### Fast Scan Profile

```toml
# fast-scan.toml - Quick network reconnaissance
[scan]
scan_type = "Syn"
timing_template = "Aggressive"
timeout_ms = 500
retries = 0

[performance]
max_rate = 50000
parallelism = 5000

[output]
format = "Greppable"
```

### Stealth Scan Profile

```toml
# stealth-scan.toml - IDS/IPS evasion
[scan]
scan_type = "Fin"
timing_template = "Sneaky"
timeout_ms = 10000
scan_delay_ms = 500

[performance]
max_rate = 100

[evasion]
fragment_packets = true
mtu = 576
ttl = 64

[evasion.decoys]
type = "random"
count = 5
```

### Service Detection Profile

```toml
# service-detection.toml - Full service enumeration
[scan]
scan_type = "Syn"
timing_template = "Normal"
timeout_ms = 5000

[scan.service_detection]
enabled = true
intensity = 8
banner_grab = true
enable_tls = true

[output]
format = "Json"
verbose = 1
```

### Enterprise Network Profile

```toml
# enterprise.toml - Large network scanning
[scan]
scan_type = "Syn"
timing_template = "Polite"
timeout_ms = 3000
retries = 1
host_delay_ms = 100

[network]
skip_cdn = true

[performance]
max_rate = 10000
parallelism = 1000
numa_enabled = true
adaptive_batch_enabled = true

[output]
format = "Xml"
verbose = 0
```

## Environment Variable Mapping

Configuration options can also be set via environment variables:

| Config Path | Environment Variable |
|-------------|---------------------|
| `scan.scan_type` | `PRTIP_SCAN_TYPE` |
| `scan.timing_template` | `PRTIP_TIMING` |
| `performance.max_rate` | `PRTIP_MAX_RATE` |
| `output.format` | `PRTIP_OUTPUT_FORMAT` |
| `output.verbose` | `PRTIP_VERBOSE` |

```bash
# Environment variable example
export PRTIP_SCAN_TYPE=Syn
export PRTIP_MAX_RATE=10000
prtip 192.168.1.0/24
```

## See Also

- [CLI Reference](./command-reference.md) - Command-line flag reference
- [Scan Templates](../user-guide/scan-templates.md) - Pre-configured scan profiles
- [Timing Templates](./timing-templates.md) - T0-T5 timing details
- [Configuration Guide](../user-guide/configuration.md) - User-friendly configuration

---

**Last Updated:** 2025-11-21
**ProRT-IP Version:** v0.5.4
