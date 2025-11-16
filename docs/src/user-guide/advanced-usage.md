# Advanced Usage

Advanced features and common use cases for ProRT-IP network scanning.

## Common Use Cases

### Use Case 1: Network Discovery

**Goal:** Find all active hosts on local network

**Command:**
```bash
sudo prtip -sn 192.168.1.0/24
```

**Explanation:**
- `-sn`: Ping scan only (no port scan)
- `192.168.1.0/24`: Scan entire /24 subnet (192.168.1.1-254)

**Expected Output:**
```
Host 192.168.1.1 is up (latency: 2.3ms)
Host 192.168.1.5 is up (latency: 1.8ms)
Host 192.168.1.10 is up (latency: 3.1ms)
...
Scan complete: 3 hosts up (254 scanned)
```

**Follow-Up:**
Once you know active hosts, scan specific ones:
```bash
sudo prtip -sS -p 1-1000 192.168.1.5
```

---

### Use Case 2: Port Scanning

#### 2a: Common Ports (Fast)

**Goal:** Quickly identify common services

**Command:**
```bash
sudo prtip -sS -F 192.168.1.10
```

**Explanation:**
- `-F`: Fast scan (top 100 ports)
- Completes in seconds

**Expected Output:**
```
PORT    STATE  SERVICE
22/tcp  open   ssh
80/tcp  open   http
443/tcp open   https
3306/tcp open  mysql
```

#### 2b: Full Port Scan

**Goal:** Comprehensive scan of all 65,535 ports

**Command:**
```bash
sudo prtip -sS -p- -T4 192.168.1.10 -oN fullscan.txt
```

**Explanation:**
- `-p-`: All ports (1-65535)
- `-T4`: Aggressive timing (faster)
- `-oN fullscan.txt`: Save results

**Note:** Full scan can take 5-30 minutes depending on network and timing template.

#### 2c: Custom Port List

**Goal:** Scan specific ports of interest

**Command:**
```bash
sudo prtip -sS -p 80,443,8080,8443,3000,3306 192.168.1.10
```

**Explanation:**
- Web ports: 80, 443, 8080, 8443, 3000
- Database port: 3306 (MySQL)

---

### Use Case 3: Service Detection

**Goal:** Identify services and versions running on open ports

**Command:**
```bash
sudo prtip -sS -sV -p 1-1000 192.168.1.10
```

**Expected Output:**
```
PORT    STATE  SERVICE  VERSION
22/tcp  open   ssh      OpenSSH 8.9p1 Ubuntu 3ubuntu0.1 (Ubuntu Linux; protocol 2.0)
80/tcp  open   http     Apache httpd 2.4.52 ((Ubuntu))
443/tcp open   https    Apache httpd 2.4.52 ((Ubuntu))
3306/tcp open  mysql    MySQL 8.0.33-0ubuntu0.22.04.2
```

**Interpretation:**
- **OpenSSH 8.9p1:** SSH server version
- **Apache 2.4.52:** Web server version
- **MySQL 8.0.33:** Database version
- **Ubuntu Linux:** Operating system hint

**Use Case:**
- Vulnerability assessment (check for outdated versions)
- Inventory management (document server configurations)

---

### Use Case 4: OS Fingerprinting

**Goal:** Determine operating system of target

**Command:**
```bash
sudo prtip -sS -O -p 1-1000 192.168.1.10
```

**Expected Output:**
```
OS Detection Results:
OS: Linux 5.15 - 6.1 (Ubuntu 22.04)
Confidence: 95%
CPE: cpe:/o:canonical:ubuntu_linux:22.04
```

**Interpretation:**
- **OS:** Linux kernel 5.15-6.1
- **Distribution:** Ubuntu 22.04
- **Confidence:** 95% (high confidence)

**Use Case:**
- Network inventory
- Vulnerability scanning (OS-specific exploits)
- Compliance checks

---

### Use Case 5: Stealth Scanning

**Goal:** Evade intrusion detection systems (IDS)

#### 5a: Slow Timing (T0)

**Command:**
```bash
sudo prtip -sS -T0 -p 80,443,22 192.168.1.10
```

**Explanation:**
- `-T0`: Paranoid timing (5-minute delays between packets)
- Very slow but stealthy

#### 5b: Fragmentation

**Command:**
```bash
sudo prtip -sS -f -p 80,443 192.168.1.10
```

**Explanation:**
- `-f`: Fragment packets to evade simple packet filters

#### 5c: Decoy Scanning

**Command:**
```bash
sudo prtip -sS -D RND:10 -p 80,443 192.168.1.10
```

**Explanation:**
- `-D RND:10`: Use 10 random decoy IPs
- Target sees scan from multiple sources

**Expected Output:**
```
Using decoys: 203.0.113.15, 198.51.100.42, ..., YOUR_IP, ...
Scanning 192.168.1.10...
```

#### 5d: Idle/Zombie Scan

**Command:**
```bash
sudo prtip -sI 192.168.1.5 -p 80,443 192.168.1.10
```

**Explanation:**
- `-sI 192.168.1.5`: Use 192.168.1.5 as zombie host
- Target never sees your IP address

**Note:** Requires suitable zombie host (see [Idle Scan Guide](../features/idle-scan.md))

---

### Use Case 6: Performance Tuning

#### 6a: Large-Scale Scanning

**Goal:** Scan thousands of hosts efficiently

**Command:**
```bash
sudo prtip -sS -F -T4 --max-rate 1000 10.0.0.0/16 -oG results.gnmap
```

**Explanation:**
- `10.0.0.0/16`: 65,536 hosts
- `-T4`: Aggressive timing
- `--max-rate 1000`: Limit to 1000 packets/second (network courtesy)
- `-oG`: Greppable output for parsing

**Performance:**
- Estimated time: 10-30 minutes (depending on network)

#### 6b: NUMA Optimization (Linux)

**Goal:** Maximize performance on multi-core systems

**Command:**
```bash
sudo prtip -sS -p 1-1000 --numa 192.168.1.0/24
```

**Explanation:**
- `--numa`: Enable NUMA-aware thread pinning
- **Benefit:** 10-30% performance improvement on NUMA systems

#### 6c: Rate Limiting

**Goal:** Control scan speed to avoid network congestion, IDS detection, or server overload

##### 6c.1: AdaptiveRateLimiterV3 (Default, Industry-Leading)

ProRT-IP uses **AdaptiveRateLimiterV3**, achieving **-1.8% average overhead** (faster than no rate limiting!).

**Why V3 is Faster:**
- **Two-Tier Convergence:** Hostgroup + per-target scheduling
- **Relaxed Memory Ordering:** Eliminates memory barriers (10-30ns savings)
- **Self-Correction:** Convergence compensates for stale atomic reads
- **CPU Optimization:** Better cache locality with rate limiting enabled

**Performance Characteristics:**

| Rate (pps) | Overhead | Use Case |
|------------|----------|----------|
| 10K | -8.2% | Best case (CPU optimization dominant) |
| 50K | -1.8% | Typical scan rate |
| 75K-200K | -3% to -4% | Sweet spot |
| 500K-1M | +0% to +3.1% | Near-zero at extreme rates |

**Basic Usage:**
```bash
# Rate-limited scan (V3 automatic, 100K pps)
prtip -sS -p 80-443 --max-rate 100000 192.168.1.0/24

# High-speed scan (sweet spot: 75K-200K pps, -3% to -4% overhead)
prtip -sS -p 1-10000 --max-rate 150000 192.168.0.0/16

# Extreme rate (near-zero overhead at 500K-1M pps)
prtip -sS -p- --max-rate 500000 10.0.0.0/8
```

##### 6c.2: Hostgroup Control

Limits concurrent targets being scanned simultaneously (Nmap-compatible).

**Flags:**
- `--max-hostgroup <N>`: Maximum concurrent targets (default: 64)
- `--min-hostgroup <N>`: Minimum concurrent targets (default: 1)
- `--max-parallelism <N>`: Alias for `--max-hostgroup`

**Usage:**
```bash
# Network-friendly (16 hosts max)
prtip -sS -p- --max-hostgroup 16 10.0.0.0/24

# Aggressive (128 hosts)
prtip -sS -p 80,443 --max-hostgroup 128 targets.txt
```

**Tuning Guidelines:**

| Value Range | Network Impact | Scan Speed | IDS Detection | Use Case |
|-------------|----------------|------------|---------------|----------|
| 1-16 | Minimal | Slower | Low risk | Sensitive environments |
| 32-128 | Balanced | Medium | Some alerts | General-purpose |
| 256-1024 | High | Fast | Likely detection | Internal networks, pen tests |

---

### Use Case 7: Firewall Testing

**Goal:** Determine firewall rules

**Command:**
```bash
sudo prtip -sA -p 80,443,22,25 192.168.1.10
```

**Explanation:**
- `-sA`: ACK scan (firewall detection)
- Differentiates filtered vs unfiltered ports

**Expected Output:**
```
PORT   STATE
80/tcp unfiltered   # Firewall allows traffic
443/tcp unfiltered  # Firewall allows traffic
22/tcp filtered     # Firewall blocks SSH
25/tcp filtered     # Firewall blocks SMTP
```

**Interpretation:**
- **Unfiltered:** Port is accessible (firewall allows)
- **Filtered:** Port is blocked by firewall

---

### Use Case 8: SSL/TLS Analysis

**Goal:** Inspect SSL/TLS certificates

**Command:**
```bash
sudo prtip -sS -sV -p 443 --script ssl-cert 192.168.1.10
```

**Expected Output:**
```
PORT    STATE  SERVICE  VERSION
443/tcp open   https    Apache httpd 2.4.52

SSL Certificate:
Subject: CN=example.com
Issuer: CN=Let's Encrypt Authority X3
Valid From: 2025-01-01
Valid Until: 2025-04-01
Self-Signed: No
```

**Use Case:**
- Certificate expiration monitoring
- Security audit (identify self-signed certs)
- Compliance checks

> **See Also:**
> - [TLS Certificate Guide](../features/tls-certificates.md)
> - [Examples: HTTPS Scanning](../getting-started/examples.md#https-tls)

---

### Use Case 9: IPv6 Scanning

**Goal:** Scan IPv6 networks

#### 9a: Single IPv6 Host

**Command:**
```bash
sudo prtip -6 -sS -p 80,443 2001:db8::1
```

**Explanation:**
- `-6`: Force IPv6
- `2001:db8::1`: IPv6 address

#### 9b: IPv6 Subnet

**Command:**
```bash
sudo prtip -6 -sS -p 80,443 2001:db8::/64
```

**Warning:** IPv6 /64 subnets are huge (2^64 addresses). Use targeted scanning or discovery first.

#### 9c: Dual-Stack Scanning

**Command:**
```bash
sudo prtip -sS -p 80,443 example.com
```

**Explanation:**
- Automatically scans both IPv4 and IPv6 if available
- Use `--prefer-ipv6` or `--prefer-ipv4` to control preference

> **See Also:**
> - [IPv6 Guide](../features/ipv6.md)
> - [Examples: IPv6 Scanning](../getting-started/examples.md#ipv6-scanning)

---

### Use Case 10: Plugin System

**Goal:** Extend ProRT-IP with custom Lua plugins for detection, output, and scanning

#### 10.1: Plugin Types

ProRT-IP supports three plugin types:

| Type | Purpose | Use Cases | Example |
|------|---------|-----------|---------|
| **ScanPlugin** | Scan lifecycle hooks | Pre/post-scan processing, custom data collection | Event logger |
| **OutputPlugin** | Custom output formats | CSV, JSON, XML, database export | Database exporter |
| **DetectionPlugin** | Enhanced service detection | Banner analysis, active probing | banner-analyzer |

#### 10.2: Using Plugins

**Basic Usage:**
```bash
# Load single plugin
sudo prtip -sS -sV -p 80,443 --plugin banner-analyzer 192.168.1.10

# Load multiple plugins
sudo prtip -sS -sV -p 80,443 \
    --plugin banner-analyzer \
    --plugin ssl-checker \
    192.168.1.10
```

**List Available Plugins:**
```bash
prtip --list-plugins
```

**Output:**
```
Available Plugins (2 found):

banner-analyzer (v1.0.0) [DetectionPlugin]
  Author: ProRT-IP Team
  Description: Enhanced banner analysis for HTTP, SSH, FTP, SMTP, MySQL, PostgreSQL
  Capabilities: None (passive analysis)
  Location: ~/.prtip/plugins/banner-analyzer/

ssl-checker (v1.0.0) [DetectionPlugin]
  Author: ProRT-IP Team
  Description: SSL/TLS service detection and analysis
  Capabilities: network
  Location: ~/.prtip/plugins/ssl-checker/
```

#### 10.3: Plugin Directory Structure

**Installation Location:**
- User plugins: `~/.prtip/plugins/`
- System plugins: `/opt/prtip/plugins/` (requires root)

**Plugin Structure:**
```
~/.prtip/plugins/my-plugin/
├── plugin.toml    # Metadata (required)
├── main.lua       # Implementation (required)
└── README.md      # Documentation (recommended)
```

**plugin.toml Example:**
```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
author = "Your Name <your.email@example.com>"
description = "One-line plugin description"
plugin_type = "detection"  # scan, output, or detection
capabilities = []  # Required permissions: network, filesystem, system, database
```

**main.lua Example:**
```lua
function on_load(config)
    prtip.log("info", "My plugin loaded")
    return true
end

function on_unload()
    prtip.log("info", "My plugin unloaded")
end

function analyze_banner(banner)
    if string.match(banner, "HTTP") then
        return {
            service = "http",
            confidence = 0.8
        }
    end
    return nil
end
```

#### 10.4: Creating Your First Plugin

**Step 1: Create directory**
```bash
mkdir -p ~/.prtip/plugins/my-detector
cd ~/.prtip/plugins/my-detector
```

**Step 2: Write plugin.toml**
```toml
[plugin]
name = "my-detector"
version = "1.0.0"
author = "Your Name"
description = "Custom service detector"
plugin_type = "detection"
capabilities = []
```

**Step 3: Write main.lua**
```lua
function on_load(config)
    prtip.log("info", "my-detector loaded")
    return true
end

function on_unload()
    prtip.log("info", "my-detector unloaded")
end

function analyze_banner(banner)
    if string.match(banner, "CUSTOM") then
        return {
            service = "custom",
            product = "CustomApp",
            confidence = 0.9
        }
    end
    return nil
end
```

**Step 4: Test plugin**
```bash
# List plugins
prtip --list-plugins

# Test with scan
prtip -sS -sV -p 80 127.0.0.1 --plugin my-detector
```

> **See Also:**
> - [Plugin System Guide](../features/plugin-system.md)
> - [Plugin API Reference](../reference/plugin-api.md)

---

### Use Case 11: Batch Scanning

**Goal:** Scan multiple targets from file

**Command:**
```bash
sudo prtip -sS -p 80,443 -iL targets.txt -oA batch_results
```

**targets.txt:**
```
192.168.1.10
192.168.1.20
10.0.0.0/24
example.com
```

**Output:**
- `batch_results.txt` (normal output)
- `batch_results.json` (JSON)
- `batch_results.xml` (XML)
- `batch_results.gnmap` (greppable)

---

### Use Case 12: CI/CD Integration

**Goal:** Automate security scanning in CI/CD pipeline

**GitLab CI Example:**
```yaml
security_scan:
  stage: test
  image: rust:1.85
  script:
    - cargo install prtip
    - prtip -sS -p 80,443 staging.example.com -oJ scan_results.json
    - python3 check_vulnerabilities.py scan_results.json
  artifacts:
    paths:
      - scan_results.json
    expire_in: 30 days
```

**GitHub Actions Example:**
```yaml
name: Security Scan

on:
  push:
    branches: [main]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install ProRT-IP
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          source $HOME/.cargo/env
          cargo install prtip
      - name: Run Scan
        run: sudo prtip -sS -p 1-1000 staging.example.com -oJ scan.json
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: scan-results
          path: scan.json
```

---

### Use Case 13: Internet-Scale Scanning

**Goal:** Scan large public IP ranges (responsible disclosure)

**Warning:** Only scan networks you own or have permission to scan.

**Command:**
```bash
sudo prtip -sS -F --max-rate 500 203.0.113.0/24 -oG internet_scan.gnmap
```

**Best Practices:**
- Use `--max-rate` to avoid overwhelming networks
- Scan only essential ports (`-F` for top 100)
- Use greppable output for post-processing
- Respect robots.txt and scan policies

---

### Use Case 14: Idle Scan (Anonymous Scanning)

**Goal:** Scan target without revealing your IP

**Command:**
```bash
# 1. Find zombie host
sudo prtip -sI RND 192.168.1.0/24

# 2. Use discovered zombie
sudo prtip -sI 192.168.1.5 -p 80,443 TARGET
```

**Explanation:**
- `-sI RND`: Discover suitable zombie hosts
- `-sI 192.168.1.5`: Use specific zombie
- Target logs show zombie IP, not your IP

**Warning:** Ethical use only.

> **See Also:**
> - [Idle Scan Guide](../features/idle-scan.md)
> - [Examples: Stealth Scanning](../getting-started/examples.md#stealth-scans)

---

### Use Case 15: Performance Benchmarking

**Goal:** Validate ProRT-IP performance claims and track regression

#### 15a: Running Benchmarks Locally

**Prerequisites:**
- hyperfine installed: `cargo install hyperfine`
- ProRT-IP release binary built: `cargo build --release`

**Command:**
```bash
cd benchmarks/05-Sprint5.9-Benchmarking-Framework
./scripts/run-all-benchmarks.sh
```

**Output:**
```
ProRT-IP Benchmarking Framework
Date: 2025-11-07 14:35:00
Binary: /home/user/ProRT-IP/target/release/prtip
Version: 0.5.0

Running: 01-syn-scan-1000-ports.sh
Benchmark 1: prtip -sS -p 1-1000 127.0.0.1
  Time (mean ± σ):      98.2 ms ±   4.5 ms    [User: 12.3 ms, System: 23.4 ms]
  Range (min … max):    90.1 ms … 108.9 ms    10 runs
✅ PASS (within target <100ms)
...
```

**Benchmark Scenarios:**

| # | Scenario | Purpose | Target |
|---|----------|---------|--------|
| 1 | SYN Scan (1K ports) | Throughput validation | <100ms |
| 2 | Connect Scan (3 ports) | Real-world baseline | <50ms |
| 3 | UDP Scan (3 services) | Slow protocol | <500ms |
| 4 | Service Detection | Overhead measurement | <10% |
| 5 | IPv6 Overhead | IPv4 vs IPv6 | <15% |
| 6 | Idle Scan Timing | Stealth cost | 500-800ms/port |
| 7 | Rate Limiting V3 | Performance claim | -1.8% overhead |
| 8 | TLS Cert Parsing | Certificate speed | ~1.33μs |

#### 15b: Interpreting Results

**hyperfine Output Fields:**
- **mean ± σ:** Average time ± standard deviation
- **Range:** Fastest and slowest runs
- **User:** CPU time in user space
- **System:** CPU time in kernel (syscalls)

**Good Results (Reproducible):**
- Stddev <5% of mean (e.g., 98.2ms ± 4.5ms = 4.6%)
- Narrow range (max <20% higher than min)
- User + System ≈ mean (CPU-bound)

**Bad Results (High Variance):**
- Stddev >10% of mean
- Wide range (max >50% higher than min)
- User + System << mean (I/O-bound or waiting)

> **See Also:**
> - [Benchmarking Guide](../advanced/benchmarking.md)
> - [Performance Analysis](../advanced/performance-tuning.md)

---

## CLI User Experience Features

ProRT-IP provides professional-quality CLI experience with enhanced help, intelligent error messages, real-time progress tracking, safety confirmations, and productivity shortcuts.

### Scan Templates

Scan templates provide one-command access to common scanning scenarios, eliminating the need to memorize complex flag combinations.

**Built-in Templates** (10 total):

```bash
# Web Server Scanning
prtip --template web-servers 192.168.1.0/24
# Equivalent to: -p 80,443,8080,8443 -sV --script http-*

# Database Discovery
prtip --template databases 192.168.1.1
# Equivalent to: -p 3306,5432,27017,6379,1521 -sV

# Quick Scan (Top 100 Ports)
prtip --template quick 192.168.1.0/24
# Equivalent to: -F -T4 -sS

# Comprehensive Scan
prtip --template thorough 192.168.1.1
# Equivalent to: -p- -T3 -sV -O

# Stealth Scan (Evasion)
prtip --template stealth 192.168.1.1
# Equivalent to: -sF -T0 -f -D RND:5

# SSL/TLS Analysis
prtip --template ssl-only 192.168.1.1
# Equivalent to: -p 443,8443,993,995,465,636,3389 -sV --tls-analysis
```

**Template Management:**

```bash
# List all available templates
prtip --list-templates

# Show template details
prtip --show-template web-servers

# Override template values
prtip --template quick -p 1-10000  # Override port range
prtip --template stealth -T3       # Override timing
```

**Custom Templates** (`~/.prtip/templates.toml`):

```toml
[templates.staging-web]
description = "Scan staging environment web servers"
ports = "80,443,3000,8080,8443"
scan_type = "syn"
timing = "T4"
service_detection = true
```

---

### Enhanced Help System

Multi-page help with fuzzy search enables quick discovery of any flag or feature.

**Help Categories:**

```bash
# Show all help categories
prtip help

# Topic-specific help
prtip help scan-types    # Connect, SYN, UDP, Stealth, Idle
prtip help timing        # T0-T5 templates, custom timing
prtip help output        # Output formats, filtering, storage
prtip help targeting     # CIDR, ranges, files, exclusions
prtip help detection     # Service, OS, banner grabbing
prtip help evasion       # Fragmentation, decoys, stealth
prtip help advanced      # IPv6, NUMA, plugins, rate limiting
prtip help examples      # Common usage scenarios
```

**Searchable Help (Fuzzy Matching):**

```bash
# Search across all help content
prtip help search "certificate"
# Finds: --tls-analysis, --tls-versions, TLS certificate guide

# Typo tolerance (edit distance ≤ 2)
prtip help search "syn scn"
# Finds: "SYN scan" topics with keyword highlighting
```

---

### Progress Indicators with ETA

Real-time progress tracking with estimated time to completion, throughput metrics, and multi-stage visualization.

**Progress Styles:**

```bash
# Compact (default)
[Stage 3/5] Port Scanning ▓▓▓▓▓▓▓▓▓░ 87% | ETA: 3m 24s

# Detailed
prtip --progress-style detailed -sS -p- 192.168.1.0/24
# Shows: percentage, ETA, packets/sec, hosts/min, bandwidth

# Multi-stage Bars
prtip --progress-style bars -sS -sV -p 1-1000 192.168.1.0/24
# Stage 1: Target Resolution   ▓▓▓▓▓▓▓▓▓▓ 100%
# Stage 2: Host Discovery      ▓▓▓▓▓▓▓▓▓▓ 100%
# Stage 3: Port Scanning        ▓▓▓▓▓▓▓▓░░  87%
# Stage 4: Service Detection    ▓░░░░░░░░░  10%
# Stage 5: Finalization         ░░░░░░░░░░   0%
# Overall ▓▓▓▓▓░░░░░ 52% | ETA: 3m 24s | 1,240 pps | 42 hpm
```

**ETA Algorithms:**

- **Linear ETA:** Simple current-rate projection
- **EWMA ETA:** Exponentially Weighted Moving Average (α=0.2, smooths fluctuations)
- **Multi-stage ETA:** Weighted prediction across 5 scan stages

**Color-Coded Speed:**

- 🟢 Green: On track (<10% over ETA)
- 🟡 Yellow: Slow (10-25% over ETA)
- 🔴 Red: Very slow (>25% over ETA)

**Disable Progress (CI/Automation):**

```bash
prtip --no-progress -sS -p 80,443 192.168.1.0/24
prtip --progress-interval 5 -sS -p- 192.168.1.0/24  # Update every 5s
```

---

### Interactive Confirmations

Smart confirmations protect against accidental execution of dangerous operations.

**Protected Operations:**

```bash
# Internet-scale scans (0.0.0.0/0, ::/0)
prtip -sS -p 80,443 0.0.0.0/0
# ⚠️  Warning: Internet-scale scan detected
# Target: 0.0.0.0/0 (entire IPv4 internet)
# Estimated hosts: 4,294,967,296
# Estimated duration: 7 days 18 hours
# Legal risks: HIGH (may violate ToS, trigger IDS/IPS)
# Are you sure you want to proceed? [y/N]

# Large target sets (>10K hosts)
prtip -sS -p 80,443 10.0.0.0/8
# ⚠️  Warning: Large scan detected
# Scanning 16,777,216 hosts. Estimated duration: 4 days 6 hours
# Continue? [y/N]

# Aggressive timing (T5)
prtip -T5 -sS -p 80,443 192.168.1.1
# ⚠️  Warning: T5 is VERY aggressive
# May trigger IDS/IPS. Continue? [y/N]

# Evasion techniques (fragmentation, decoys)
prtip -f -D RND:10 -sS -p 80,443 192.168.1.1
# ⚠️  Warning: Evasion techniques may be illegal in your jurisdiction
# Continue? [y/N]
```

**Smart Skip Logic (Auto-bypass):**

Confirmations are **automatically skipped** when:
- Running in non-interactive terminal (CI/CD, automation)
- `--yes` flag provided
- Target set is "safe" (RFC1918 private ranges: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)
- Timing is polite (T0-T2)

**Automation Mode:**

```bash
# Bypass all confirmations (CI/CD)
prtip --yes -sS -p 80,443 0.0.0.0/0
```

---

### Command History & Replay

Automatic scan history with replay capability for rapid re-execution and modification.

**View History:**

```bash
# Show last 20 commands
prtip history

# Output:
# [1] 2025-11-08 14:23:45 | prtip -sS -p 80,443 192.168.1.1 (12 results)
# [2] 2025-11-08 14:45:12 | prtip -sT -p 1-1000 10.0.0.1 (245 results)
# [3] 2025-11-08 15:10:33 | prtip --template web-servers 192.168.1.0/24 (89 results)

# Show last N commands
prtip history 50
```

**Replay Commands:**

```bash
# Re-run specific command by index
prtip replay 2

# Re-run most recent command
prtip replay --last

# Replay with modifications
prtip replay 3 -p 1-10000        # Change port range
prtip replay --last --template thorough  # Add template
```

**Storage:**

- **Location:** `~/.prtip/history.json`
- **Format:** JSON with timestamps, args, summaries
- **Auto-rotation:** Keeps latest 1,000 entries
- **Atomic writes:** Prevents corruption on crash

**Clear History:**

```bash
prtip history --clear
```

---

### Better Error Messages

Actionable error messages with platform-specific solutions reduce debugging time.

**Error Categories:**

- 🔴 **Fatal:** Scan cannot proceed (permission, invalid target)
- ⚠️  **Warning:** Scan degraded (rate limited, filtered ports)
- ℹ️  **Info:** Informational (progress milestones)
- 💡 **Tip:** Optimization suggestions

**Example Errors:**

```bash
# Permission denied
prtip -sS -p 80,443 192.168.1.1
# 🔴 Fatal: Permission denied creating raw socket
#
# Solution:
#   Option 1: Run with sudo privileges
#     sudo prtip -sS -p 80,443 192.168.1.1
#
#   Option 2: Set capabilities (Linux only)
#     sudo setcap cap_net_raw+ep $(which prtip)
#
#   Option 3: Use unprivileged scan type
#     prtip -sT -p 80,443 192.168.1.1  # TCP Connect scan

# Invalid IP address
prtip -sS -p 80 999.999.999.999
# 🔴 Fatal: Invalid IP address
#   Expected: x.x.x.x (IPv4) or x:x:x:x:x:x:x:x (IPv6)
#   Got: 999.999.999.999
#   Suggestion: Use 192.168.1.1 or example.com

# Connection timeout
prtip -sS -p 80,443 192.168.1.1 --max-rtt-timeout 100
# ⚠️  Warning: Connection timeouts detected (15/100 hosts)
#   Try: --max-rtt-timeout 5000 OR -T0 (slower but more reliable)

# Too many open files (ulimit)
prtip -sS -p 1-65535 192.168.1.0/24 --batch-size 10000
# ⚠️  Warning: Too many open files
#   Detected ulimit: 1024
#   Recommended: 65535
#   Command: ulimit -n 65535
#   Automatically reducing batch size to 512. Re-run with --batch-size to override.
```

**Coverage:** 95%+ errors include actionable suggestions (19 error patterns)

---

## Next Steps

**Learn More:**
- [Feature Guides](../features/) - Deep dives into specific features
- [Advanced Topics](../advanced/) - Performance tuning, evasion, large-scale scanning
- [Examples](../getting-started/examples.md) - 65 runnable examples

**Get Help:**
- [Troubleshooting](../appendices/troubleshooting.md)
- [FAQ](../appendices/faq.md)
- [Community Discussions](https://github.com/doublegate/ProRT-IP/discussions)

**Contribute:**
- Report bugs and request features
- Improve documentation
- Write plugins
- Submit code contributions
