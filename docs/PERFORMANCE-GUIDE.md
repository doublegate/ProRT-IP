# ProRT-IP Performance Optimization Guide

**Version:** 1.0
**Last Updated:** October 2025
**Audience:** End users, DevOps engineers, security researchers

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Performance Tuning](#performance-tuning)
3. [Scan Type Selection](#scan-type-selection)
4. [Timing Templates](#timing-templates)
5. [Hardware Recommendations](#hardware-recommendations)
6. [Troubleshooting](#troubleshooting)
7. [Advanced Optimizations](#advanced-optimizations)

---

## Quick Start

### TL;DR: Fastest Scans

```bash
# Fast scan (top 100 ports, ~3-5 seconds)
prtip -F 192.168.1.0/24

# All ports, aggressive timing (~30-60 seconds for Class C)
prtip -p- -T4 192.168.1.0/24

# Internet-scale scan (1M+ hosts, stateless)
prtip --max-rate 100000 -p 80,443 0.0.0.0/8
```

### Performance Hierarchy (Fastest → Slowest)

| Scan Type | Speed | Accuracy | Use Case |
|-----------|-------|----------|----------|
| **Stateless** | ⚡⚡⚡⚡⚡ | ⭐⭐⭐ | Internet-wide discovery |
| **SYN (-sS)** | ⚡⚡⚡⚡ | ⭐⭐⭐⭐ | Default (requires root) |
| **Connect (-sT)** | ⚡⚡⚡ | ⭐⭐⭐⭐ | No privileges required |
| **Service Detection (-sV)** | ⚡⚡ | ⭐⭐⭐⭐⭐ | Deep enumeration |
| **OS Fingerprinting (-O)** | ⚡ | ⭐⭐⭐⭐ | Host identification |

---

## Performance Tuning

### Timing Templates (-T0 to -T5)

ProRT-IP includes 6 timing templates inspired by Nmap:

```bash
# T0 - Paranoid (slowest, stealthiest)
prtip -T0 -p 80,443 target.com
# Rate: ~1 pps, ideal for: IDS evasion

# T1 - Sneaky
prtip -T1 -p 1-1000 target.com
# Rate: ~10 pps, ideal for: Cautious scanning

# T2 - Polite
prtip -T2 -p 1-1000 target.com
# Rate: ~100 pps, ideal for: Production networks

# T3 - Normal (default)
prtip -p 1-1000 target.com
# Rate: ~1K pps, ideal for: General use

# T4 - Aggressive (recommended for most users)
prtip -T4 -p 1-65535 target.com
# Rate: ~10K pps, ideal for: Penetration testing

# T5 - Insane (fastest, may lose accuracy)
prtip -T5 -p- target.com
# Rate: ~100K pps, ideal for: Local networks, time-critical
```

### Manual Rate Control

Override timing templates with explicit rate limits:

```bash
# Limit to 1000 packets/second
prtip --max-rate 1000 -p 80,443 192.168.0.0/16

# Minimum delay between packets (500ms)
prtip --scan-delay 500 -p 1-1000 target.com

# Combine both for fine-grained control
prtip --max-rate 5000 --scan-delay 10 -p- 10.0.0.0/24
```

**Recommended Rates by Network Type:**

| Network | Max Rate | Reasoning |
|---------|----------|-----------|
| **Localhost** | 100,000+ pps | No network latency |
| **Local LAN (1 Gbps)** | 50,000 pps | Minimal packet loss |
| **Local LAN (100 Mbps)** | 5,000 pps | Avoid saturation |
| **Internet (targets)** | 1,000 pps | Avoid IDS/rate limiting |
| **Internet (discovery)** | 100,000+ pps | Stateless, distributed load |

### Parallelism Control

Adjust concurrency for different workloads:

```bash
# Auto-detect CPU cores (default)
prtip -p 80,443 10.0.0.0/16

# Manual parallelism (4 worker threads)
prtip --parallel 4 -p 1-1000 192.168.1.0/24

# Max parallelism (all CPU cores)
prtip --parallel $(nproc) -p- target.com
```

**Rule of Thumb:**
- **Network-bound scans:** Use 4-8 threads (network is bottleneck)
- **CPU-bound scans:** Use all cores (packet crafting intensive)
- **Service detection:** Use 2-4 threads (I/O bound, many connections)

---

## Scan Type Selection

### SYN Scan (-sS) [Recommended]

**Speed:** ⚡⚡⚡⚡ (10K-100K pps)
**Requires:** Root/Administrator privileges
**Stealth:** Moderate (half-open connections)

```bash
# Fast SYN scan (default)
prtip -sS -p 1-1000 target.com

# With service detection
prtip -sS -sV -p 1-1000 target.com
```

**Pros:**
- Fast (no full TCP handshake)
- Accurate (direct to kernel)
- Low resource usage

**Cons:**
- Requires elevated privileges
- Detectable by IDS (SYN without ACK)

### Connect Scan (-sT)

**Speed:** ⚡⚡⚡ (1K-10K pps)
**Requires:** No privileges
**Stealth:** Low (full TCP connections logged)

```bash
# Connect scan (no root required)
prtip -sT -p 80,443,8080 target.com
```

**Pros:**
- No privileges required
- Works everywhere (cross-platform)
- Respects OS networking stack

**Cons:**
- Slower (full TCP handshake)
- More detectable (complete connections)
- Higher resource usage

### UDP Scan (-sU)

**Speed:** ⚡ (100-1K pps, rate-limited by ICMP)
**Requires:** Root/Administrator privileges
**Accuracy:** Medium (many false negatives)

```bash
# UDP scan (slow, requires root)
prtip -sU -p 53,161,500 target.com

# Top 100 UDP ports
prtip -sU -F target.com
```

**Pros:**
- Discovers UDP services (DNS, SNMP, etc.)
- Essential for comprehensive enumeration

**Cons:**
- Very slow (ICMP rate limiting)
- Many false negatives (no response != closed)
- Requires root privileges

**Performance Tip:** Combine with TCP scan for efficiency:
```bash
# Scan TCP and UDP top 100 ports
prtip -sS -sU -F target.com
```

### Stealth Scans (-sF, -sN, -sX)

**Speed:** ⚡⚡⚡ (similar to SYN)
**Requires:** Root/Administrator privileges
**Stealth:** High (bypasses some firewalls)

```bash
# FIN scan (stealthy)
prtip -sF -p 1-1000 target.com

# NULL scan (no flags)
prtip -sN -p 1-1000 target.com

# Xmas scan (FIN, PSH, URG)
prtip -sX -p 1-1000 target.com
```

**Pros:**
- Bypass stateless firewalls (SYN filter evasion)
- Lower detectability (unusual packet types)

**Cons:**
- Requires root privileges
- Doesn't work on Windows targets (RST always sent)
- Doesn't work through stateful firewalls

---

## Hardware Recommendations

### Minimum Requirements

| Component | Minimum | Recommended | Notes |
|-----------|---------|-------------|-------|
| **CPU** | 2 cores | 4+ cores | Parallel scanning |
| **RAM** | 2 GB | 8 GB | Large scans (1M+ hosts) |
| **Network** | 100 Mbps | 1 Gbps | Throughput limited by NIC |
| **OS** | Linux 4.15+ | Linux 5.10+ | Kernel optimizations |

### High-Performance Setup

For 1M+ pps throughput:

**Hardware:**
- **CPU:** 8+ cores (AMD Ryzen 9 / Intel i9)
- **RAM:** 16 GB+ (for stateful scanning)
- **NIC:** 10 Gbps (Intel X710, Mellanox ConnectX)
- **Storage:** SSD (for result streaming)

**Software:**
- **OS:** Linux 5.10+ with tuned network stack
- **Kernel:** Custom with XDP support (optional)
- **ProRT-IP:** Compiled with `--release` (optimizations enabled)

**System Tuning:**
```bash
# Increase socket buffers
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728

# Increase file descriptors
ulimit -n 65535

# Enable performance governor (CPU scaling)
sudo cpupower frequency-set -g performance
```

---

## Troubleshooting

### Symptom: Slow Scan Performance (<1K pps)

**Possible Causes:**

1. **Insufficient Privileges**
   ```bash
   # Check if running as root
   id

   # Grant capabilities (Linux)
   sudo setcap cap_net_raw,cap_net_admin=eip ./target/release/prtip

   # Or run with sudo
   sudo ./target/release/prtip [args]
   ```

2. **Rate Limiting Enabled**
   ```bash
   # Check current rate limit
   prtip --help | grep max-rate

   # Increase rate limit
   prtip --max-rate 10000 [args]
   ```

3. **Conservative Timing Template**
   ```bash
   # Use aggressive timing
   prtip -T4 [args]

   # Or insane timing (careful!)
   prtip -T5 [args]
   ```

### Symptom: High Packet Loss

**Possible Causes:**

1. **Rate Too High**
   ```bash
   # Reduce rate to 50% of current
   prtip --max-rate 5000 [args]

   # Or use adaptive rate limiting (automatic)
   prtip --adaptive-rate [args]
   ```

2. **Network Saturation**
   ```bash
   # Check NIC utilization
   ifstat -i eth0 1

   # Reduce parallelism
   prtip --parallel 4 [args]
   ```

3. **Firewall Rate Limiting**
   ```bash
   # Use slower timing template
   prtip -T2 [args]

   # Add random delays
   prtip --randomize-hosts [args]
   ```

### Symptom: High Memory Usage

**Possible Causes:**

1. **Too Many Active Connections**
   ```bash
   # Limit concurrent connections
   prtip --max-parallelism 1000 [args]

   # Stream results to disk
   prtip -oN results.txt [args]
   ```

2. **Large Target Range**
   ```bash
   # Split into smaller scans
   prtip -p 80,443 10.0.0.0/24 &
   prtip -p 80,443 10.0.1.0/24 &
   ```

### Symptom: No Results / All Ports Filtered

**Possible Causes:**

1. **Firewall Blocking**
   ```bash
   # Try stealth scan
   prtip -sF -p 80,443 target.com

   # Use decoys (hide source)
   prtip -D RND:10 -p 80,443 target.com
   ```

2. **Wrong Network Interface**
   ```bash
   # List interfaces
   ip link show

   # Specify interface explicitly
   prtip -e eth0 [args]
   ```

3. **Routing Issue**
   ```bash
   # Check routing table
   ip route show

   # Ping target first
   ping -c 3 target.com
   ```

---

## Advanced Optimizations

### Zero-Copy Packet Building (v0.3.9+)

ProRT-IP automatically uses zero-copy packet building for maximum performance:

**Benefits:**
- 15% faster packet crafting (68.3ns → 58.8ns per packet)
- 100% allocation elimination (no GC pauses)
- Better scaling at high packet rates (1M+ pps)

**No configuration needed** - zero-copy is enabled by default in v0.3.9+.

**Benchmarks:**
```bash
# Run packet crafting benchmarks
cargo bench --bench packet_crafting

# View results
firefox target/criterion/report/index.html
```

### Batch System Calls (Linux Only)

ProRT-IP uses `sendmmsg`/`recvmmsg` on Linux for batched I/O:

**Benefits:**
- 98.4% syscall reduction (1000 syscalls → 16 with batch size 64)
- 2-5x throughput improvement at high packet rates

**Configuration:**
```bash
# Adjust batch size (default: 64)
prtip --batch-size 128 [args]

# Disable batching (compatibility)
prtip --batch-size 1 [args]
```

**Optimal Batch Sizes:**
- **16:** Low latency, ~95% syscall reduction
- **64:** Balanced (recommended), ~98% syscall reduction
- **128:** Maximum throughput, ~99% syscall reduction, higher latency

### NUMA Optimization (Future)

For multi-socket systems, NUMA-aware thread pinning will be available in a future release:

**Expected Benefits:**
- 10-30% improvement on multi-socket systems (AMD EPYC, Intel Xeon)
- Reduced cross-NUMA memory access penalties

**Configuration (when available):**
```bash
# Enable NUMA optimization
prtip --numa [args]

# Pin threads to specific NUMA nodes
prtip --numa-node 0 [args]
```

### Profiling Your Scans

Identify bottlenecks in your scanning workflow:

```bash
# Enable verbose mode for timing info
prtip -v -p 80,443 target.com

# Very verbose (debug info)
prtip -vv -p 1-1000 target.com

# Extremely verbose (packet-level tracing)
prtip -vvv -p 80 target.com
```

**Output Includes:**
- Scan initialization time
- Packet send rate (pps)
- Response rate (pps)
- Timeouts and retransmits
- Total scan duration

---

## Performance FAQ

### Q: How fast is ProRT-IP compared to Nmap/Masscan?

**A:** ProRT-IP performance positioning:

| Tool | Speed | Accuracy | Safety |
|------|-------|----------|--------|
| **Masscan** | ⚡⚡⚡⚡⚡ (10M+ pps) | ⭐⭐⭐ | ⚠️ C, unsafe |
| **ProRT-IP** | ⚡⚡⚡⚡ (1M+ pps) | ⭐⭐⭐⭐ | ✅ Rust, safe |
| **Nmap** | ⚡⚡ (300K pps) | ⭐⭐⭐⭐⭐ | ⚠️ C, unsafe |
| **RustScan** | ⚡⚡⚡ (65K ports in 3s) | ⭐⭐⭐ | ✅ Rust, safe |

**Key Differentiators:**
- Faster than Nmap (3-10x for stateless scans)
- Safer than Masscan (Rust memory safety)
- More accurate than RustScan (comprehensive detection)

### Q: What's the fastest possible scan?

**A:** For discovery scans (port 80/443 on large networks):

```bash
# Maximum speed (1M+ pps, requires root + 10GbE)
prtip -T5 --max-rate 1000000 -p 80,443 0.0.0.0/8
```

**Realistic Speeds:**
- **Localhost:** 100K-500K pps (no network latency)
- **LAN (1 Gbps):** 50K-100K pps (minimal latency)
- **Internet:** 10K-50K pps (network and target rate limiting)

### Q: Does ProRT-IP support GPU acceleration?

**A:** Not currently. Packet crafting is CPU-bound and benefits from multi-core parallelism rather than GPU parallelism. Future releases may explore GPU acceleration for specific tasks (e.g., cryptographic operations).

### Q: How do I maximize throughput for internet-wide scans?

**A:** Best practices for large-scale scanning:

1. **Use stateless mode** (no connection tracking overhead)
2. **Limit ports** (-p 80,443 instead of -p-)
3. **Aggressive timing** (-T5)
4. **High rate limit** (--max-rate 100000)
5. **Multiple instances** (split /8 into /16 subnets, run in parallel)
6. **Stream to disk** (-oN/-oX to avoid memory buildup)

```bash
# Example: Scan entire internet for HTTP/HTTPS
prtip -T5 --max-rate 100000 -p 80,443 -oN results.txt 0.0.0.0/0
```

**Warning:** Internet-scale scanning may violate ISP terms of service or local laws. Always scan responsibly and ethically.

---

## See Also

- [Architecture Documentation](00-ARCHITECTURE.md) - System design
- [Testing Guide](06-TESTING.md) - Performance test implementation
- [Security Guide](08-SECURITY.md) - Secure scanning practices
- [Performance Baselines](07-PERFORMANCE.md) - Technical optimization details

---

**Last Updated:** October 2025 | **Version:** 1.0 | **Sprint:** 4.17 (Zero-Copy Optimization)
