# ProRT-IP Performance Optimization Guide

**Version:** 1.0
**Last Updated:** October 2025
**Audience:** End users, DevOps engineers, security researchers

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Performance Tuning](#performance-tuning)
3. [Scan Type Selection](#scan-type-selection)
4. [Timing Templates](14-NMAP-COMPATIBILITY.md)
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

### NUMA Optimization for Multi-Socket Systems (v0.3.8+)

**Available since v0.3.8** - NUMA (Non-Uniform Memory Access) optimization improves performance on multi-socket servers by reducing cross-socket memory access latency and improving cache locality through automatic topology detection and thread pinning.

#### When to Use NUMA

**USE NUMA if:**
- ✅ Dual-socket or quad-socket system (2+ physical CPUs)
- ✅ High-throughput scans (>100K packets/second)
- ✅ Long-running scans (>1 hour)
- ✅ CPU/memory intensive workloads (service detection, OS fingerprinting)

**DON'T USE NUMA if:**
- ❌ Single-socket system (no benefit, slight overhead)
- ❌ Low-throughput scans (<10K packets/second)
- ❌ Short scans (<1 minute)
- ❌ Network-bound workloads (limited by network speed, not CPU)

#### Performance Benefits

| System Type | Expected Improvement | Cache Miss Reduction | Use Case |
|-------------|---------------------|---------------------|----------|
| **Single-Socket** | <5% (negligible) | <2% | Not recommended |
| **Dual-Socket** | 20-30% faster | 15-25% | Recommended ✅ |
| **Quad-Socket** | 30-40% faster | 25-35% | Highly recommended ✅ |

#### Checking NUMA Availability

```bash
# Check NUMA topology
numactl --hardware

# Example output (dual-socket):
# available: 2 nodes (0-1)
# node 0 cpus: 0 1 2 3 4 5 6 7
# node 0 size: 32768 MB
# node 1 cpus: 8 9 10 11 12 13 14 15
# node 1 size: 32768 MB

# Example output (single-socket):
# available: 1 nodes (0)
# node 0 cpus: 0 1 2 3 4 5 6 7
# node 0 size: 16384 MB
```

#### Enabling NUMA

```bash
# Enable NUMA optimization (auto-detects topology)
prtip -sS -p 1-65535 10.0.0.0/16 --numa --rate 1000000

# Explicitly disable NUMA (even if available)
prtip -sS -p 1-65535 10.0.0.0/16 --no-numa

# Default behavior (NUMA disabled for compatibility)
prtip -sS -p 1-65535 10.0.0.0/16  # No NUMA

# Check if NUMA was enabled (look for log messages)
prtip -sS -p 1-65535 10.0.0.0/16 --numa -v | grep -i numa
# Expected output:
#   "NUMA optimization enabled (2 nodes)"
#   "Scheduler thread pinned to core 0"
#   "Worker 0 pinned to core 1 (node 0)"
#   "Worker 1 pinned to core 8 (node 1)"
```

#### How It Works

ProRT-IP uses NUMA-aware thread pinning to optimize performance:

1. **TX Thread Pinning:** Main transmit thread pinned to cores near NIC (NUMA node 0 by default)
   - Reduces PCIe latency for packet transmission
   - Improves DMA performance for NIC access

2. **Worker Distribution:** Worker threads distributed round-robin across NUMA nodes
   - Balances load across all sockets
   - Maximizes aggregate memory bandwidth
   - Reduces cross-socket memory access

**Example Thread Layout (Dual-Socket, 16 cores):**
```
NUMA Node 0 (Cores 0-7):   TX Thread (core 0) + Workers 0, 2, 4, 6, ...
NUMA Node 1 (Cores 8-15):  Workers 1, 3, 5, 7, ...

Benefits:
- TX thread has local access to NIC (node 0)
- Workers evenly distributed (8 per node)
- Memory bandwidth: 2x aggregate (both nodes utilized)
```

#### Performance Validation

**Measure Throughput:**
```bash
hyperfine --warmup 3 --runs 5 \
    'prtip -sS -p 1-65535 192.168.1.0/24 --no-numa' \
    'prtip -sS -p 1-65535 192.168.1.0/24 --numa'

# Example output (dual-socket):
#   Benchmark 1: --no-numa
#     Time (mean ± σ):     42.3 s ±  1.2 s
#   Benchmark 2: --numa
#     Time (mean ± σ):     32.8 s ±  0.9 s
#
#   Summary
#     '--numa' ran 1.29x faster (29% improvement) ✅

# Example output (single-socket):
#   Benchmark 1: --no-numa
#     Time (mean ± σ):     45.2 s ±  1.5 s
#   Benchmark 2: --numa
#     Time (mean ± σ):     45.8 s ±  1.4s
#
#   Summary
#     '--no-numa' ran 1.01x faster (1% faster, within noise)
```

**Measure Cache Misses (Linux only, requires sudo):**
```bash
# With NUMA optimization
sudo perf stat -e cache-misses,cache-references \
    prtip -sS -p 1-65535 192.168.1.0/24 --numa

# Output:
#   5,234,567 cache-misses    # 15.2% of all cache refs
#  34,456,789 cache-references

# Without NUMA optimization
sudo perf stat -e cache-misses,cache-references \
    prtip -sS -p 1-65535 192.168.1.0/24 --no-numa

# Output:
#   6,789,012 cache-misses    # 19.7% of all cache refs
#  34,456,789 cache-references

# Expected: 15-25% fewer cache misses with NUMA on multi-socket systems
# Result: (6,789,012 - 5,234,567) / 6,789,012 = 22.9% reduction ✅
```

#### Troubleshooting NUMA

**Error: "NUMA pinning failed: Permission denied"**

Thread pinning requires `CAP_SYS_NICE` capability on Linux.

*Solution 1: Add capability (recommended):*
```bash
# If installed system-wide
sudo setcap cap_sys_nice+ep /usr/bin/prtip

# If installed via cargo
sudo setcap cap_sys_nice+ep ~/.cargo/bin/prtip

# Verify capability
getcap /usr/bin/prtip
# Expected: /usr/bin/prtip = cap_sys_nice+ep
```

*Solution 2: Run as root (not recommended for security):*
```bash
sudo prtip -sS -p 1-65535 10.0.0.0/16 --numa
```

**Security Note:** `CAP_SYS_NICE` allows setting thread priorities and CPU affinity. This is generally safe for ProRT-IP, but be aware it grants elevated privileges.

**Warning: "NUMA not available on this system"**

NUMA optimization requires a multi-socket system with NUMA support. ProRT-IP will automatically fall back to non-NUMA mode.

*To verify your system has NUMA:*
```bash
numactl --hardware
# If output shows "available: 1 nodes", you have a single-socket system

# Alternative check
ls /sys/devices/system/node/
# Expected (multi-socket): node0 node1 node2 ...
# Expected (single-socket): node0
```

**Warning: "Single-node system detected, NUMA disabled"**

Your system has only one NUMA node (single-socket or NUMA disabled in BIOS). NUMA optimization provides no benefit on single-socket systems.

*Check BIOS settings:*
- Look for "NUMA" or "Node Interleaving" settings
- Disable "Node Interleaving" to enable NUMA
- Reboot and check `numactl --hardware` again

**Warning: "NUMA initialization failed: ..., falling back to non-NUMA mode"**

NUMA detection or manager creation failed. This is usually safe to ignore - ProRT-IP will continue without NUMA optimization.

*Common causes:*
- hwloc library not installed or incompatible version
- Kernel NUMA support disabled
- Unusual system topology

**Performance Not Improving**

*Check:*
1. **System has multiple NUMA nodes:**
   ```bash
   numactl --hardware
   # Should show "available: 2 nodes" or more
   ```

2. **NIC PCIe location:**
   ```bash
   lspci -vv | grep -i numa
   # Ideally NIC should be on NUMA node 0
   ```

3. **Workload is CPU/memory intensive:**
   - NUMA helps most with high-throughput scans (>100K pps)
   - Low-throughput scans may not show improvement

4. **System has sufficient CPU cores:**
   - At least 4 cores per NUMA node recommended
   - With only 2 cores per node, thread pinning may hurt performance

5. **No other resource bottlenecks:**
   - Network bandwidth (use 10GbE or faster)
   - Disk I/O (if writing to database)
   - Target capacity (scanning localhost won't show NUMA benefits)

**Note:** NUMA optimization is most beneficial for:
- ✅ Large-scale scans (>1M packets/second)
- ✅ Dual-socket or quad-socket systems (2-4 CPUs)
- ✅ 10GbE or faster network interfaces
- ✅ CPU-bound workloads (service detection, OS fingerprinting)
- ✅ Long-running scans (>1 hour)

#### Platform Support

| Platform | NUMA Support | Notes |
|----------|--------------|-------|
| **Linux** | ✅ Full | hwloc + sched_setaffinity, requires CAP_SYS_NICE |
| **macOS** | ⚠️ Fallback | Auto-detects single-node, no thread pinning available |
| **Windows** | ⚠️ Fallback | Auto-detects single-node, no thread pinning available |
| **BSD** | ⚠️ Fallback | Auto-detects single-node, cpuset affinity not implemented |

**Linux distributions tested:**
- ✅ Ubuntu 20.04, 22.04, 24.04
- ✅ Debian 11, 12
- ✅ RHEL 8, 9
- ✅ Fedora 38, 39
- ✅ Arch Linux (rolling)

**Compile-Time Feature:**

NUMA support is optional via `numa` feature (enabled by default):

```bash
# Build with NUMA support (default)
cargo build --release

# Build without NUMA support (smaller binary, -300KB)
cargo build --release --no-default-features --features cli
```

#### NUMA Technical Details

**Topology Detection:**
- Uses `hwloc` library (Hardware Locality) for cross-platform topology detection
- Detects: Number of NUMA nodes, CPU cores per node, memory per node, PCI device affinity
- Graceful fallback if hwloc unavailable or detection fails
- Caching: Topology detected once at startup, cached for scan duration

**Thread Affinity:**
- Uses `nix::sched::sched_setaffinity` on Linux (wraps `sched_setaffinity(2)` syscall)
- Pins threads to specific CPU cores using CPU affinity masks (`CpuSet`)
- Requires `CAP_SYS_NICE` capability or root privileges (security consideration)
- Non-root users: Use `setcap cap_sys_nice+ep` as shown in troubleshooting

**Core Allocation Strategy:**
- **TX Thread:** Pinned to core on NUMA node 0 (assumed to be near NIC)
- **Worker Threads:** Round-robin allocation across all NUMA nodes
  - Worker 0 → Node 0
  - Worker 1 → Node 1
  - Worker 2 → Node 0 (wraps around)
- **Avoids over-subscription:** Maximum workers = total CPU cores (configurable with --max-concurrent)

**Memory Allocation:**
- Rust's default allocator (jemalloc or system allocator) handles memory
- With NUMA-aware thread pinning, memory is typically allocated locally (first-touch policy)
- For explicit NUMA memory binding, see future work (libnuma integration)

#### Future Enhancements

**Planned for Sprint 4.20+:**
- Explicit NIC node specification: `--numa-nic-node 1`
- Manual core pinning: `--numa-cores 0-7,16-23`
- Memory binding: `--numa-mem-bind 0` (requires libnuma)
- IRQ affinity: Automatic NIC IRQ pinning to NUMA node 0

**Community contributions welcome!**

#### References

- **hwloc Documentation:** https://www.open-mpi.org/projects/hwloc/
- **Intel NUMA Optimization Guide:** https://software.intel.com/numa
- **Linux sched_setaffinity:** `man 2 sched_setaffinity`
- **NUMA Architecture (Wikipedia):** https://en.wikipedia.org/wiki/Non-uniform_memory_access
- **ProRT-IP Sprint 4.19:** NUMA implementation details and benchmarks

#### NUMA FAQ

**Q: Should I always use --numa?**
A: No. Only use --numa on multi-socket systems. On single-socket systems, NUMA adds slight overhead (~1-2%) with no benefit.

**Q: Does --numa require root?**
A: Not if you grant CAP_SYS_NICE capability: `sudo setcap cap_sys_nice+ep /usr/bin/prtip`

**Q: Can I use --numa with --rate?**
A: Yes! NUMA works with all flags. Combine for best performance: `prtip --numa --rate 1000000 -sS -p- target`

**Q: Does NUMA work with all scan types?**
A: Yes. NUMA optimizes the scanner framework, which benefits all scan types (SYN, UDP, stealth, etc.).

**Q: Why is my NUMA improvement less than 20-30%?**
A: Depends on workload. CPU-bound scans (service detection, OS fingerprinting) benefit more than network-bound scans (simple SYN).

**Q: Can I see which cores my threads are pinned to?**
A: Yes, use verbose logging: `prtip --numa -v | grep -i "pinned to core"`

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

### Zero-Copy Packet Building (v0.3.8+)

ProRT-IP automatically uses zero-copy packet building for maximum performance:

**Benefits:**
- 15% faster packet crafting (68.3ns → 58.8ns per packet)
- 100% allocation elimination (no GC pauses)
- Better scaling at high packet rates (1M+ pps)

**No configuration needed** - zero-copy is enabled by default in v0.3.8+.

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

### NUMA Optimization (v0.3.8+)

**Available since v0.3.8** - NUMA-aware thread pinning for multi-socket systems.

**Quick Reference:**
```bash
# Enable NUMA optimization (Linux only, multi-socket systems)
prtip --numa -sS -p 1-65535 192.168.1.0/24

# Explicitly disable (even if available)
prtip --no-numa -sS -p 80,443 target.com
```

**Performance Impact:**
- **Dual-socket:** 20-30% improvement (15-25% fewer cache misses)
- **Quad-socket:** 30-40% improvement (25-35% fewer cache misses)
- **Single-socket:** <5% difference (within noise, not recommended)

**See Also:** [NUMA Optimization for Multi-Socket Systems](#numa-optimization-for-multi-socket-systems-v038) in Hardware Recommendations for comprehensive documentation, including:
- When to use NUMA (system requirements, use cases)
- Performance validation (hyperfine benchmarks, perf stat)
- Troubleshooting (permissions, verification)

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
