# Timing & Performance

Master ProRT-IP's timing templates and performance optimization features for efficient network scanning.

---

## Timing Templates

Control scan speed and stealth using predefined timing templates (T0-T5).

### Overview

| Template | Description | Speed | Stealth | Use Case |
|----------|-------------|-------|---------|----------|
| `-T0` | Paranoid | Slowest | Highest | IDS evasion |
| `-T1` | Sneaky | Very Slow | High | Stealth scans |
| `-T2` | Polite | Slow | Medium | Production networks |
| `-T3` | Normal (default) | Medium | Low | Balanced |
| `-T4` | Aggressive | Fast | None | Trusted networks |
| `-T5` | Insane | Fastest | None | Local testing |

### Examples

```bash
sudo prtip -sS -T0 -p 80,443 192.168.1.1   # Paranoid (IDS evasion)
sudo prtip -sS -T3 -p 1-1000 192.168.1.1   # Normal (default)
sudo prtip -sS -T4 -p- 192.168.1.1         # Aggressive (fast)
```

### Template Details

#### T0: Paranoid

**Speed:** 5-minute delays between probes

**Use Case:** Maximum stealth for IDS/IPS evasion

**Command:**
```bash
sudo prtip -sS -T0 -p 80,443 192.168.1.1
```

**Characteristics:**
- Extremely slow (hours to days for full scans)
- Near-impossible to detect
- Suitable for red team engagements with strict stealth requirements

#### T1: Sneaky

**Speed:** 15-second delays between probes

**Use Case:** Stealth scans without extreme paranoia

**Command:**
```bash
sudo prtip -sS -T1 -p 80,443 192.168.1.1
```

**Characteristics:**
- Very slow but practical
- Low detection risk
- Good for production networks with IDS monitoring

#### T2: Polite

**Speed:** 0.4-second delays between probes

**Use Case:** Production networks without overwhelming systems

**Command:**
```bash
sudo prtip -sS -T2 -p 1-1000 192.168.1.1
```

**Characteristics:**
- Respectful of network resources
- Minimal performance impact on targets
- Recommended for business hours scanning

#### T3: Normal (Default)

**Speed:** Balanced timing with adaptive delays

**Use Case:** General-purpose scanning

**Command:**
```bash
sudo prtip -sS -p 1-1000 192.168.1.1
```

**Characteristics:**
- Default if no timing specified
- Good balance of speed and reliability
- Suitable for most scenarios

#### T4: Aggressive

**Speed:** Fast with minimal delays

**Use Case:** Trusted networks, time-sensitive scans

**Command:**
```bash
sudo prtip -sS -T4 -p- 192.168.1.1
```

**Characteristics:**
- Fast scanning
- May trigger IDS/IPS alerts
- Recommended for internal network assessments

#### T5: Insane

**Speed:** Maximum speed, no delays

**Use Case:** Local testing, lab environments

**Command:**
```bash
sudo prtip -sS -T5 -p- 127.0.0.1
```

**Characteristics:**
- Fastest possible scanning
- High packet loss risk
- May miss results due to speed
- **Not recommended for production**

### Timing Template Comparison

| Metric | T0 | T1 | T2 | T3 | T4 | T5 |
|--------|----|----|----|----|----|----|
| **Probe Delay** | 5 min | 15s | 0.4s | 0.1s | 0.01s | 0s |
| **Max Parallelism** | 1 | 1 | 4 | 16 | 64 | 256 |
| **Max Rate (pps)** | 100 | 1,000 | 10,000 | 50,000 | 100,000 | 1,000,000 |
| **1K Ports (Est.)** | 5h | 15m | 4m | 1m | 10s | 1s |
| **Detection Risk** | Minimal | Low | Medium | High | Very High | Extreme |

### Combining Templates with Custom Limits

Override template defaults:

```bash
# T4 template + custom rate
sudo prtip -T4 --max-rate 75000 -p- 192.168.0.0/16

# T3 template + custom hostgroup
sudo prtip -T3 --max-hostgroup 32 -p 1-1000 10.0.0.0/24
```

---

## Performance Tuning

### Large-Scale Scanning

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
- **Tip:** Use stateless scanning mode for even faster results (future feature)

### NUMA Optimization (Linux)

**Goal:** Maximize performance on multi-core systems

**Command:**
```bash
sudo prtip -sS -p 1-1000 --numa 192.168.1.0/24
```

**Explanation:**
- `--numa`: Enable NUMA-aware thread pinning
- **Benefit:** 10-30% performance improvement on NUMA systems

**What is NUMA?**

Non-Uniform Memory Access (NUMA) is a memory architecture where memory access time depends on memory location relative to CPU cores.

**How ProRT-IP Uses NUMA:**
1. Detects NUMA topology at startup
2. Pins worker threads to specific CPU cores
3. Allocates memory on local NUMA nodes
4. Reduces cross-node memory access penalties

**Performance Gains:**
- **Small scans (<100 hosts):** 5-10% improvement
- **Medium scans (100-1000 hosts):** 10-20% improvement
- **Large scans (>1000 hosts):** 20-30% improvement

**Platform Support:**
- **Linux:** Full NUMA support
- **Windows/macOS:** Graceful degradation (ignored)

**Verify NUMA Topology:**
```bash
# Check if system supports NUMA
numactl --hardware

# Run ProRT-IP with NUMA
sudo prtip -sS --numa -p 1-1000 192.168.1.0/24
```

---

## Rate Limiting

Control scan speed to avoid network congestion, IDS detection, or server overload.

### AdaptiveRateLimiterV3 (Default, Industry-Leading)

ProRT-IP uses **AdaptiveRateLimiterV3**, achieving **-1.8% average overhead** (faster than no rate limiting!).

**Why V3 is Faster:**
- **Two-Tier Convergence:** Hostgroup + per-target scheduling
- **Relaxed Memory Ordering:** Eliminates memory barriers (10-30ns savings)
- **Self-Correction:** Convergence compensates for stale atomic reads
- **CPU Optimization:** Better cache locality with rate limiting enabled

### Performance Characteristics

| Rate (pps) | Overhead | Use Case |
|------------|----------|----------|
| 10K | -8.2% | Best case (CPU optimization dominant) |
| 50K | -1.8% | Typical scan rate |
| 75K-200K | -3% to -4% | Sweet spot |
| 500K-1M | +0% to +3.1% | Near-zero at extreme rates |

### Basic Usage

```bash
# Rate-limited scan (V3 automatic, 100K pps)
prtip -sS -p 80-443 --max-rate 100000 192.168.1.0/24

# High-speed scan (sweet spot: 75K-200K pps, -3% to -4% overhead)
prtip -sS -p 1-10000 --max-rate 150000 192.168.0.0/16

# Extreme rate (near-zero overhead at 500K-1M pps)
prtip -sS -p- --max-rate 500000 10.0.0.0/8
```

**No Special Flags Needed:**
- V3 is automatic with `--max-rate`
- Old `--adaptive-v3` flag removed (V3 is default)

### Hostgroup Control

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

# With minimum parallelism enforcement
prtip -sS -p 1-1000 --min-hostgroup 8 --max-hostgroup 64 10.0.0.0/16
```

**Tuning Guidelines:**

| Value Range | Network Impact | Scan Speed | IDS Detection | Use Case |
|-------------|----------------|------------|---------------|----------|
| 1-16 | Minimal | Slower | Low risk | Sensitive environments |
| 32-128 | Balanced | Medium | Some alerts | General-purpose |
| 256-1024 | High | Fast | Likely detection | Internal networks, pen tests |

### Combined Rate Limiting

Stack V3 + Hostgroup for maximum control:

```bash
# Full rate limiting stack: V3 (50K pps) + Hostgroup (32 hosts max)
prtip -sS -p- \
  --max-rate 50000 \
  --max-hostgroup 32 \
  --min-hostgroup 8 \
  10.0.0.0/16
```

### ICMP Monitoring (Optional)

**Purpose:** Automatic backoff on ICMP Type 3 Code 13 errors (administratively prohibited)

**Usage:**
```bash
# Enable ICMP monitoring for adaptive backoff
prtip -sS -p 1-1000 --adaptive-rate 192.168.1.0/24
```

**How It Works:**
1. Background task listens for ICMP packets
2. Detects Type 3 Code 13 errors (rate limiting)
3. Per-target exponential backoff (2s → 4s → 8s → 16s max)
4. Scanner waits for backoff expiration before resuming

**Platform Support:**
- **Linux/macOS:** Full support
- **Windows:** Graceful degradation (ICMP monitor inactive)

### Timing Templates with Rate Limits

Timing templates (T0-T5) automatically set rate limits:

| Template | Speed | Max Rate (pps) | Hostgroup | Use Case |
|----------|-------|----------------|-----------|----------|
| T0 (Paranoid) | Very Slow | 100 | 1 | IDS evasion |
| T1 (Sneaky) | Slow | 1,000 | 1 | Slow networks |
| T2 (Polite) | Moderate | 10,000 | 4 | Default |
| T3 (Normal) | Fast | 50,000 | 16 | Typical |
| T4 (Aggressive) | Faster | 100,000 | 64 | Fast scans |
| T5 (Insane) | Fastest | 1,000,000 | 256 | Maximum speed |

**Usage:**
```bash
# Paranoid (very slow, IDS evasion)
prtip -T0 -p- target.com

# Aggressive (fast)
prtip -T4 -p 1-10000 192.168.0.0/16

# Insane (maximum speed, may trigger detection)
prtip -T5 -p- 10.0.0.0/8
```

### Performance Comparison

**AdaptiveRateLimiterV3 vs No Limiting:**

| Scenario | No Limit | With V3 | Overhead | Result |
|----------|----------|---------|----------|--------|
| SYN Scan (1K ports, 10K pps) | 98.2ms | 90.1ms | -8.2% | V3 FASTER |
| SYN Scan (1K ports, 50K pps) | 7.3ms | 7.2ms | -1.8% | V3 FASTER |
| SYN Scan (1K ports, 500K pps) | 7.2ms | 7.2ms | +0.0% | EQUAL |

**Key Insight:** With V3's -1.8% average overhead, **always use rate limiting** for optimal performance!

### Troubleshooting

**Issue:** Slow convergence to target rate
```bash
# Increase --max-rate value
prtip -sS -p- --max-rate 100000 target.com

# Check network bottlenecks
ping target.com
```

**Issue:** "No targets scanned (all backed off)"
```bash
# All targets blocked with ICMP errors
# Solution: Disable --adaptive-rate or reduce --max-rate
prtip -sS -p- --max-rate 10000 target.com
```

**Issue:** "Active targets below min_hostgroup" warnings
```bash
# Not enough targets or slow progress
# Solution: Increase targets or reduce --min-hostgroup
prtip -sS -p- --min-hostgroup 4 small_target_list.txt
```

> **See Also:**
> - [Rate Limiting Guide](../features/rate-limiting.md) - V3 algorithm deep dive
> - [Performance Benchmarking](../getting-started/examples.md#performance) - Benchmark details
> - [Nmap Compatibility](../features/nmap-compatibility.md) - Flag comparison

---

## Timing Control

### Production Network (Polite)

**Goal:** Scan production networks without overwhelming systems

**Command:**
```bash
sudo prtip -sS -T2 -p 1-1000 192.168.1.10
```

**Timing:**
- Scan delay: 400ms between probes
- Won't overwhelm production networks

### Local Testing (Fast)

**Goal:** Maximum speed for local testing

**Command:**
```bash
sudo prtip -sS -T5 -p- 127.0.0.1
```

**Timing:**
- Minimal delays
- Maximum speed for local testing

### Custom Timing

Override individual timing parameters:

```bash
# Custom probe delay
sudo prtip -sS --scan-delay 500ms -p 80,443 192.168.1.1

# Custom initial RTT timeout
sudo prtip -sS --initial-rtt-timeout 2000ms -p 1-1000 192.168.1.1

# Custom max retries
sudo prtip -sS --max-retries 5 -p 80,443 192.168.1.1

# Custom min/max rate
sudo prtip -sS --min-rate 100 --max-rate 1000 -p 1-1000 192.168.1.1
```

---

## Performance Best Practices

### 1. Start with Host Discovery

Before scanning ports, discover which hosts are alive:

```bash
# Host discovery (no port scan)
sudo prtip -sn 192.168.1.0/24 -oN live-hosts.txt

# Review live hosts
cat live-hosts.txt

# Then scan only live hosts
sudo prtip -sS -p 1-1000 -iL live-hosts.txt
```

**Time Savings:**
- If 20 out of 256 hosts are live: **92% faster** (scan 20 instead of 256)
- Reduces network noise

### 2. Limit Scan Scope

Scan only what you need:

```bash
# Scan specific ports
prtip -sS -p 22,80,443,3389 TARGET

# Scan port range
prtip -sS -p 1-1000 TARGET

# Scan all ports (warning: very slow)
prtip -sS -p 1-65535 TARGET  # or -p-
```

**Port Selection Tips:**
- **Web services:** 80, 443, 8080, 8443
- **Remote access:** 22 (SSH), 3389 (RDP), 23 (Telnet)
- **Databases:** 3306 (MySQL), 5432 (PostgreSQL), 1433 (MSSQL)
- **Mail:** 25 (SMTP), 110 (POP3), 143 (IMAP), 587 (SMTP TLS)
- **File sharing:** 445 (SMB), 21 (FTP), 22 (SFTP)

### 3. Choose Appropriate Timing

Balance speed vs detection risk:

```bash
# Production systems: Polite (T2)
sudo prtip -sS -T2 -p 1-1000 production.example.com

# Internal networks: Aggressive (T4)
sudo prtip -sS -T4 -p 1-1000 192.168.1.0/24

# IDS evasion: Paranoid (T0) or Sneaky (T1)
sudo prtip -sS -T1 -p 80,443 monitored.example.com

# Quick testing: Insane (T5) - local only
sudo prtip -sS -T5 -p- 127.0.0.1
```

### 4. Optimize for NUMA

On multi-core Linux systems:

```bash
# Enable NUMA optimization
sudo prtip -sS --numa -p 1-1000 192.168.1.0/24
```

**Expected Gains:** 10-30% performance improvement

### 5. Slow Down When Needed

**Common Mistakes:**
- Using T4/T5 on internet targets (packet loss, detection)
- Scanning production networks with aggressive timing
- Forgetting to use rate limiting on slow networks

**Solutions:**
```bash
# Slow down for internet targets
prtip -sS -T2 -p 80,443 internet-target.com

# Production-friendly scanning
sudo prtip -sS -T2 --max-rate 1000 -p 1-1000 192.168.1.0/24

# Adaptive rate control
sudo prtip -sS --adaptive-rate -p 1-1000 192.168.1.0/24
```

---

## Common Use Cases

### Network Inventory

**Goal:** Comprehensive network inventory without overwhelming infrastructure

**Command:**
```bash
sudo prtip -sS -T2 -p 1-1000 --max-hostgroup 16 10.0.0.0/16 -oA inventory
```

**Characteristics:**
- Polite timing (T2)
- Limited parallelism (16 concurrent hosts)
- Comprehensive port range
- All output formats for analysis

### Penetration Testing

**Goal:** Fast, comprehensive scanning for security assessment

**Command:**
```bash
sudo prtip -sS -T4 -p- --numa 192.168.1.0/24 -oA pentest
```

**Characteristics:**
- Aggressive timing (T4)
- All ports scanned
- NUMA optimization for speed
- Multiple output formats

### Stealth Assessment

**Goal:** Undetected scanning for red team engagement

**Command:**
```bash
sudo prtip -sS -T1 -f --ttl 64 -D RND:5 -p 80,443 target.com -oN stealth.txt
```

**Characteristics:**
- Sneaky timing (T1)
- Fragmentation evasion
- Decoy scanning
- Limited ports to reduce noise

### Internet-Scale Research

**Goal:** Responsible internet-wide scanning for research

**Command:**
```bash
sudo prtip -sS -F --max-rate 500 -T3 0.0.0.0/0 --exclude 10.0.0.0/8,172.16.0.0/12,192.168.0.0/16 -oG internet-scan.gnmap
```

**Characteristics:**
- Fast scan (top 100 ports)
- Rate limited (500 pps, network courtesy)
- Normal timing (T3)
- Excludes private ranges
- Greppable output for analysis

**⚠️ Warning:** Only scan with proper authorization. Internet-scale scanning may violate ToS/laws.

---

## Performance Benchmarking

**Goal:** Validate ProRT-IP performance and track regression

### Running Benchmarks

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

### Benchmark Scenarios

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

### Interpreting Results

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

**Regression Detection:**
```bash
# Compare against baseline
./scripts/analyze-results.sh \
    baselines/baseline-v0.5.0.json \
    results/current.json

# Exit codes:
#   0 = PASS or IMPROVED
#   1 = WARN (5-10% slower)
#   2 = FAIL (>10% slower)
```

> **See Also:**
> - [Benchmarking Guide](../features/index.md) - Complete benchmark reference
> - [Examples: Performance](../getting-started/examples.md#performance) - Performance examples
> - [Advanced Topics: Performance Tuning](../advanced/performance-tuning.md) - Deep dive

---

## Quick Reference

### Timing Template Summary

```bash
# Timing Templates
-T0          # Paranoid (IDS evasion, very slow)
-T1          # Sneaky (stealth, slow)
-T2          # Polite (production-friendly)
-T3          # Normal (balanced, default)
-T4          # Aggressive (fast, trusted networks)
-T5          # Insane (maximum speed, local only)

# Rate Limiting
--max-rate N         # Maximum packets/second
--min-rate N         # Minimum packets/second
--max-hostgroup N    # Maximum concurrent targets
--min-hostgroup N    # Minimum concurrent targets
--adaptive-rate      # Enable ICMP backoff

# Performance
--numa               # NUMA optimization (Linux)
--batch-size N       # Batch size for parallelism
```

### Recommended Settings by Scenario

```bash
# Production Networks
sudo prtip -sS -T2 --max-rate 1000 -p 1-1000 192.168.1.0/24

# Internal Assessments
sudo prtip -sS -T4 --numa -p 1-10000 10.0.0.0/16

# Stealth Scanning
sudo prtip -sS -T1 -f -D RND:5 -p 80,443 target.com

# Local Testing
sudo prtip -sS -T5 -p- 127.0.0.1

# Internet Research (with permission)
sudo prtip -sS -F --max-rate 500 -T3 TARGET -oG results.gnmap
```

---

## Next Steps

**Related Topics:**
- [Scan Types](./scan-types.md) - Detailed scan type guide
- [Output Formats](./output-formats.md) - Save and parse results
- [Advanced Usage](./advanced-usage.md) - IPv6, plugins, evasion

**Deep Dives:**
- [Rate Limiting Guide](../features/rate-limiting.md) - V3 algorithm internals
- [Performance Tuning](../advanced/performance-tuning.md) - Optimization strategies
- [Benchmarking Guide](../features/index.md) - Validation framework

**See Also:**
- [Quick Start Guide](../getting-started/quick-start.md) - Get started quickly
- [Tutorials](../getting-started/tutorials.md) - Step-by-step walkthroughs
- [Examples Gallery](../getting-started/examples.md) - 65 runnable examples
