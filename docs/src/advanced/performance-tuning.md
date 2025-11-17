# Performance Tuning

Master advanced performance optimization techniques for ProRT-IP network scanning.

## What is Performance Tuning?

**Performance tuning** optimizes ProRT-IP scans across three competing dimensions:

1. **Speed** - Maximize throughput (packets per second)
2. **Stealth** - Minimize detection by IDS/firewalls
3. **Resource Usage** - Control CPU, memory, and network impact

**When to Tune Performance:**

- **Fast Scans:** Need results quickly (penetration testing, time-critical)
- **Stealth Scans:** Evade intrusion detection systems (red team operations)
- **Large-Scale:** Scanning thousands/millions of hosts (infrastructure audits)
- **Resource-Constrained:** Limited CPU/RAM/bandwidth (cloud instances, embedded systems)
- **Production Networks:** Minimize impact on business-critical systems

**Performance Metrics:**

| Metric | Description | Typical Values |
|--------|-------------|----------------|
| **Throughput (pps)** | Packets per second | 1 pps (stealth) to 100K+ pps (speed) |
| **Latency** | Time to scan N ports | 6.9ms (100 ports) to 4.8s (65K ports) |
| **Memory** | RAM usage | <1 MB (stateless) to 100 MB+ (stateful) |
| **CPU** | Core utilization | 10-100% depending on parallelism |

**ProRT-IP Performance Philosophy:**

ProRT-IP balances Masscan-inspired **speed** (10M+ pps capable) with Nmap-compatible **depth** (service/OS detection) and built-in **safety** (rate limiting, minimal system impact).

**Key Performance Indicators (v0.5.2):**

```
Stateless Throughput: 10,200 pps (localhost)
Stateful Throughput:   6,600 pps (localhost)
Rate Limiter Overhead: -1.8% (faster than unlimited)
Service Detection:     85-90% accuracy
Memory Footprint:      <1 MB stateless, <100 MB/10K hosts
TLS Parsing:           1.33μs per certificate
IPv6 Overhead:         ~15% vs IPv4
```

---

## Understanding Timing Templates

ProRT-IP includes **6 pre-configured timing templates** (T0-T5) inspired by Nmap, balancing speed vs stealth.

### Template Overview

```bash
# T0 - Paranoid (slowest, stealthiest)
prtip -T0 -p 80,443 target.com
# Rate: ~1 pps, IDS evasion, ultra-stealth

# T1 - Sneaky
prtip -T1 -p 1-1000 target.com
# Rate: ~10 pps, cautious scanning, slow

# T2 - Polite
prtip -T2 -p 1-1000 target.com
# Rate: ~100 pps, production networks, low impact

# T3 - Normal (default)
prtip -p 1-1000 target.com
# Rate: ~1K pps, balanced, general use

# T4 - Aggressive (recommended for most users)
prtip -T4 -p 1-65535 target.com
# Rate: ~10K pps, fast LANs, penetration testing

# T5 - Insane (fastest, may lose accuracy)
prtip -T5 -p- target.com
# Rate: ~100K pps, localhost, time-critical
```

### Template Selection Guide

| Use Case | Template | Rate | Overhead vs T3 | When to Use |
|----------|----------|------|----------------|-------------|
| **IDS Evasion** | T0 (Paranoid) | 1-10 pps | +50,000% | Ultra-stealth, advanced IDS bypass |
| **Slow Scanning** | T1 (Sneaky) | 10-50 pps | +2,000% | Cautious reconnaissance |
| **Production** | T2 (Polite) | 50-200 pps | +500% | Business-critical networks |
| **General** | T3 (Normal) | 1-5K pps | Baseline | Default, balanced approach |
| **Fast LANs** | T4 (Aggressive) | 5-10K pps | -20% | Penetration testing, trusted networks |
| **Maximum Speed** | T5 (Insane) | 10-50K pps | -40% | Localhost, time-critical, research |

### Real-World Examples

**Example 1: Corporate Network Audit**

```bash
# Scenario: Scan 1,000 corporate servers for compliance
# Requirements: Minimal network impact, business hours
prtip -T2 -p 80,443,3389,22 192.168.0.0/22 -oJ audit.json

# Why T2: Polite timing (50-200 pps) won't saturate network
# Expected duration: (1,024 hosts × 4 ports) / 100 pps ≈ 41 seconds
```

**Example 2: Penetration Testing (Local Network)**

```bash
# Scenario: Red team engagement, find vulnerable services fast
# Requirements: Speed, comprehensive port coverage
prtip -T4 -p 1-10000 -sV 10.0.0.0/24 -oA pentest

# Why T4: Aggressive timing (5-10K pps), local network can handle
# Expected duration: (256 hosts × 10,000 ports) / 7,500 pps ≈ 5.7 minutes
```

**Example 3: Stealth Scan (IDS Evasion)**

```bash
# Scenario: Evade Snort/Suricata IDS
# Requirements: Ultra-low packet rate, randomization
prtip -T0 -f -D RND:5 -p 80,443,8080 target.com

# Why T0: Paranoid timing (1 pps), fragmentation, decoys
# Expected duration: 3 ports / 1 pps ≈ 3 seconds
```

**Example 4: Localhost Development**

```bash
# Scenario: Test scanning engine performance
# Requirements: Maximum speed, no network limits
prtip -T5 -p 1-65535 127.0.0.1 -oN localhost_scan.txt

# Why T5: Insane timing (100K pps), no network latency
# Expected duration: 65,535 ports / 100,000 pps ≈ 0.65 seconds
```

### Performance Impact Analysis

**Throughput Comparison (1,000 ports, localhost):**

| Scan Type | T3 (Normal) | T4 (Aggressive) | T5 (Insane) | Speed Gain |
|-----------|-------------|-----------------|-------------|------------|
| SYN Scan | 98ms | 78ms | 59ms | 40% faster (T5 vs T3) |
| FIN Scan | 115ms | 92ms | 69ms | 40% faster |
| NULL Scan | 113ms | 90ms | 68ms | 40% faster |
| Xmas Scan | 118ms | 94ms | 71ms | 40% faster |
| ACK Scan | 105ms | 84ms | 63ms | 40% faster |

**Trade-offs:**

- **T4/T5 Benefits:** 20-40% faster scans, better for large port ranges
- **T4/T5 Risks:** Possible packet loss on slow networks, easier IDS detection
- **T0/T1/T2 Benefits:** Stealth, minimal network impact, IDS evasion
- **T0/T1/T2 Risks:** 5-500x slower, impractical for large scans

---

## Manual Rate Control

Override timing templates with **explicit rate limits** for fine-grained control.

### Rate Limiting Flags

```bash
# Maximum packet rate (packets per second)
prtip --max-rate 1000 -p 80,443 192.168.0.0/16

# Minimum delay between packets (milliseconds)
prtip --scan-delay 500 -p 1-1000 target.com

# Combine both for precise control
prtip --max-rate 5000 --scan-delay 10 -p- 10.0.0.0/24
```

### Network-Specific Recommendations

| Network Type | Max Rate | Reasoning | Command |
|-------------|----------|-----------|---------|
| **Localhost** | 100,000+ pps | No network latency, loopback | `prtip --max-rate 100000 127.0.0.1` |
| **LAN (1 Gbps)** | 50,000 pps | Minimal packet loss, trusted | `prtip --max-rate 50000 192.168.1.0/24` |
| **LAN (100 Mbps)** | 5,000 pps | Avoid saturation, legacy switches | `prtip --max-rate 5000 192.168.1.0/24` |
| **Internet (targets)** | 1,000 pps | Avoid IDS/rate limiting, courtesy | `prtip --max-rate 1000 target.com` |
| **Internet (discovery)** | 100,000+ pps | Stateless, distributed load | `prtip --max-rate 100000 -sS 0.0.0.0/0` |

### Rate Limiter V3 Performance

**Industry-Leading Overhead (Sprint 5.X):**

ProRT-IP's adaptive rate limiter actually **improves** performance vs unlimited scans:

| Scenario | No Rate Limit | With Rate Limit | Overhead |
|----------|---------------|-----------------|----------|
| SYN 1K ports | 99.8ms | 98.0ms | **-1.8%** (faster) ✅ |
| Connect 100 ports | 151ms | 149ms | **-1.3%** (faster) ✅ |

**Why Faster:**

1. **Convergence Algorithm:** Optimizes system-wide packet flow
2. **Kernel Queue Management:** Reduces overflow/retransmissions
3. **CPU Cache Utilization:** Better temporal locality
4. **Competitive Advantage:** Nmap has +5-10% overhead, Masscan has no rate limiting

**Configuration:**

```bash
# Default adaptive rate limiting (recommended)
prtip -sS -p 1-1000 target.com
# Automatically adjusts based on ICMP errors

# Disable rate limiting (localhost only)
prtip --max-rate 0 -p 1-1000 127.0.0.1

# Conservative limit (production)
prtip --max-rate 1000 -p 80,443 10.0.0.0/8
```

### Burst Behavior

**Burst Configuration:**

```bash
# Default burst: 100 packets
prtip --max-rate 10000 target.com

# Explanation:
# - Initial burst: 100 packets sent immediately
# - Then steady-state: 10,000 pps average
# - Convergence: 95% stable in <500ms
```

**Adaptive Features:**

- Monitors ICMP "Destination Unreachable" errors
- Automatically backs off if rate limiting detected
- Recovers gradually when errors stop
- No manual tuning required

---

## Parallelism Tuning

Control **concurrent worker threads** for optimal CPU/network utilization.

### Parallelism Flags

```bash
# Auto-detect CPU cores (default, recommended)
prtip -p 80,443 10.0.0.0/16

# Manual parallelism (4 worker threads)
prtip --parallel 4 -p 1-1000 192.168.1.0/24

# Maximum parallelism (all CPU cores)
prtip --parallel $(nproc) -p- target.com

# Single-threaded (debugging, profiling)
prtip --parallel 1 -p 1-1000 target.com
```

### Workload-Specific Strategies

**Rule of Thumb:**

| Workload Type | Bottleneck | Optimal Parallelism | Reasoning |
|---------------|------------|-------------------|-----------|
| **Network-Bound** | Network latency | 4-8 threads | More threads = wasted CPU on waiting |
| **CPU-Bound** | Packet crafting | All cores | Parallel packet building saturates CPU |
| **I/O-Bound** | Disk/database writes | 2-4 threads | Avoid disk contention |
| **Service Detection** | TCP connections | 2-4 threads | Many open connections |

**Examples:**

```bash
# Network-bound: SYN scan over internet
# Bottleneck: RTT latency (10-100ms), not CPU
prtip --parallel 4 -sS -p 1-1000 target.com/24
# Why 4: More threads won't speed up network responses

# CPU-bound: Stateless scan, localhost
# Bottleneck: Packet crafting (CPU cycles)
prtip --parallel $(nproc) -sS -p 1-65535 127.0.0.1
# Why all cores: Pure computation, no I/O wait

# I/O-bound: Service detection with database output
# Bottleneck: TCP handshakes + SQLite writes
prtip --parallel 2 -sV -p 80,443 192.168.1.0/24 --db results.sqlite
# Why 2: Avoid database lock contention

# Service detection: Many simultaneous connections
# Bottleneck: File descriptors, connection tracking
prtip --parallel 4 -sV -p 1-1000 target.com
# Why 4: Balance between concurrency and resource limits
```

### CPU Utilization Analysis

**Single-Threaded (--parallel 1):**

```
CPU Usage: 12% (1 core at 100%, 11 idle on 12-core system)
Throughput: 2,500 pps (limited by single-core packet crafting)
Use Case: Debugging, profiling, low-priority scans
```

**Optimal Parallelism (--parallel 4):**

```
CPU Usage: 45% (4 cores active, good utilization)
Throughput: 10,000 pps (4x single-threaded)
Use Case: Most scans (network-bound, balanced)
```

**Maximum Parallelism (--parallel 12 on 12-core):**

```
CPU Usage: 95% (all cores saturated)
Throughput: 15,000 pps (diminishing returns, network bottleneck)
Use Case: CPU-bound workloads (localhost, packet crafting benchmarks)
```

---

## Hardware Optimization

### Minimum Requirements

**Basic Scanning (Small Networks):**

| Component | Minimum | Recommended | Notes |
|-----------|---------|-------------|-------|
| **CPU** | 2 cores, 2 GHz | 4+ cores, 3 GHz | Parallel scanning efficiency |
| **RAM** | 2 GB | 8 GB | Large scans (1M+ hosts) |
| **Network** | 100 Mbps | 1 Gbps | Throughput limited by NIC |
| **OS** | Linux 4.15+ | Linux 5.10+ | Kernel network optimizations |

### High-Performance Setup

**Internet-Scale Scanning (1M+ hosts, 1M+ pps):**

**Hardware:**

- **CPU:** 8+ cores (AMD Ryzen 9 5900X / Intel i9-12900K)
  - Clock speed: 3.5+ GHz base
  - Multi-socket for NUMA: Dual-socket or quad-socket Xeon/EPYC
- **RAM:** 16 GB+ (32 GB for stateful scanning)
  - Speed: DDR4-3200+ (lower latency = better)
- **NIC:** 10 Gbps (Intel X710, Mellanox ConnectX-5/6)
  - Multiple NICs for bonding (optional)
- **Storage:** NVMe SSD (for result streaming, <5ms latency)

**Software:**

- **OS:** Linux 5.10+ with tuned network stack
- **Kernel:** Custom with XDP support (optional, advanced)
- **ProRT-IP:** Compiled with `cargo build --release` (optimizations enabled)

### System Tuning

**File Descriptor Limits:**

```bash
# Check current limit
ulimit -n
# Typical default: 1024 (insufficient)

# Increase to 65535 (temporary, current session)
ulimit -n 65535

# Permanent (add to /etc/security/limits.conf)
echo "* soft nofile 65535" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65535" | sudo tee -a /etc/security/limits.conf

# Why: Each TCP connection requires 1 file descriptor
# 1024 limit = only 1000 concurrent connections possible
# 65535 = supports full port range scanning
```

**Network Buffer Tuning (Linux):**

```bash
# Increase socket buffer sizes (26 MB)
sudo sysctl -w net.core.rmem_max=26214400
sudo sysctl -w net.core.wmem_max=26214400

# Increase connection backlog (5000 pending connections)
sudo sysctl -w net.core.netdev_max_backlog=5000

# Reduce TIME_WAIT duration (15 seconds instead of 60)
# Caution: May break TCP reliability in high-loss networks
sudo sysctl -w net.ipv4.tcp_fin_timeout=15

# Why: Larger buffers accommodate high packet rates (10K+ pps)
# Reduced TIME_WAIT prevents port exhaustion during scans
```

**CPU Performance Governor:**

```bash
# Enable performance mode (disable frequency scaling)
sudo cpupower frequency-set -g performance

# Verify
cpupower frequency-info

# Why: CPU frequency scaling adds latency jitter
# Performance mode locks cores at max frequency
```

**Make Tuning Permanent:**

```bash
# Add to /etc/sysctl.conf
sudo tee -a /etc/sysctl.conf <<EOF
net.core.rmem_max=26214400
net.core.wmem_max=26214400
net.core.netdev_max_backlog=5000
net.ipv4.tcp_fin_timeout=15
EOF

# Reload
sudo sysctl -p
```

---

## NUMA Optimization

**NUMA (Non-Uniform Memory Access)** optimization for **multi-socket systems** (2+ physical CPUs).

### When to Use NUMA

**Enabled by Default:** No (for compatibility)

**Should Enable When:**

- ✅ Dual-socket or quad-socket server (2-4 physical CPUs)
- ✅ High-throughput scans (>100K pps target)
- ✅ Long-running scans (hours/days)
- ✅ Linux operating system (best support)

**Should NOT Enable When:**

- ❌ Single-socket system (negligible benefit, <5% gain)
- ❌ macOS/Windows (limited/no support, fallback mode)
- ❌ Small scans (<1,000 hosts)

### Performance Benefits

**Expected Improvements:**

| System Type | Performance Gain | Cache Miss Reduction | Use Case |
|-------------|-----------------|---------------------|----------|
| **Single-Socket** | <5% (negligible) | <2% | Not recommended |
| **Dual-Socket** | 20-30% faster | 15-25% | Recommended ✅ |
| **Quad-Socket** | 30-40% faster | 25-35% | Highly recommended ✅ |

**How NUMA Helps:**

1. **Reduced Memory Latency:** Threads access local memory (same socket)
2. **Better Cache Locality:** L3 cache stays on-socket (no cross-socket traffic)
3. **Bandwidth Scaling:** Each socket has dedicated memory controllers

**Performance Penalty Without NUMA:**

- Cross-socket memory access: 30-50% latency penalty
- L3 cache misses: 15-25% more on multi-socket
- Memory bandwidth contention

### Usage Examples

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

### Validation and Troubleshooting

**Check NUMA Topology:**

```bash
# Install numactl (if not present)
sudo apt install numactl  # Debian/Ubuntu
sudo dnf install numactl  # Fedora/RHEL

# Display NUMA topology
numactl --hardware
# Expected output for dual-socket:
#   available: 2 nodes (0-1)
#   node 0 cpus: 0 1 2 3 4 5 6 7
#   node 1 cpus: 8 9 10 11 12 13 14 15
#   node 0 size: 32768 MB
#   node 1 size: 32768 MB
```

**Manual NUMA Binding (Advanced):**

```bash
# Run ProRT-IP on specific NUMA node
numactl --cpunodebind=0 --membind=0 prtip -sS -p 1-65535 target.com
# Forces execution on node 0 (cores 0-7, local memory)

# Interleave memory across nodes (not recommended)
numactl --interleave=all prtip -sS -p 1-65535 target.com
# Distributes memory allocations, reduces locality
```

**Verify Thread Pinning:**

```bash
# Start scan with NUMA + verbose logging
prtip --numa -sS -p 1-1000 127.0.0.1 -v 2>&1 | grep -i "pinned"

# Expected output:
# [INFO] Scheduler thread pinned to core 0 (node 0)
# [INFO] Worker thread 0 pinned to core 1 (node 0)
# [INFO] Worker thread 1 pinned to core 8 (node 1)
```

**Troubleshooting:**

**Problem:** "NUMA optimization requested but not available"

**Cause:** Single-socket system or hwlocality library not found

**Solution:**

```bash
# Check CPU topology
lscpu | grep "Socket(s)"
# If "Socket(s): 1" → single-socket, NUMA won't help

# Install hwlocality support (Rust dependency)
# (ProRT-IP built with hwlocality by default)
```

**Problem:** "Permission denied setting thread affinity"

**Cause:** Missing CAP_SYS_NICE capability

**Solution:**

```bash
# Run with sudo (required for thread pinning)
sudo prtip --numa -sS -p 1-65535 target.com

# Or grant CAP_SYS_NICE capability (persistent)
sudo setcap cap_sys_nice+ep $(which prtip)
```

### Platform Support

| Platform | NUMA Support | Thread Pinning | Notes |
|----------|-------------|----------------|-------|
| **Linux** | Full ✅ | sched_setaffinity | Best performance |
| **macOS** | Fallback | No | Auto-disables, no error |
| **Windows** | Fallback | No | Auto-disables, no error |
| **BSD** | Fallback | Partial | Limited hwloc support |

---

## Advanced Techniques

### Zero-Copy Packet Building (v0.3.8+)

**Automatic optimization** - no configuration needed.

**Benefits:**

- 15% faster packet crafting (68.3ns → 58.8ns per packet)
- 100% allocation elimination (no GC pauses)
- Better scaling at high packet rates (1M+ pps)

**How It Works:**

```rust
// Traditional approach (v0.3.7 and earlier)
let packet = build_syn_packet(target, port); // Heap allocation
socket.send(&packet)?;                        // Copy to kernel

// Zero-copy approach (v0.3.8+)
build_syn_packet_inplace(&mut buffer, target, port); // In-place mutation
socket.send(&buffer)?;                                // Direct send
```

**Enabled by default** - no user action required.

### Batch System Calls (Linux Only)

**sendmmsg/recvmmsg batching** for reduced syscall overhead.

**Benefits:**

- 98.4% syscall reduction (1000 syscalls → 16 with batch size 64)
- 2-5x throughput improvement at high packet rates
- Linux-only (fallback to send/recv on macOS/Windows)

**Configuration:**

```bash
# Default batch size: 64 packets per syscall (recommended)
prtip -sS -p 1-1000 target.com

# Increase batch for throughput (higher latency)
prtip --batch-size 128 -sS -p 1-65535 target.com

# Decrease batch for low latency
prtip --batch-size 16 -sS -p 1-1000 target.com

# Disable batching (compatibility testing)
prtip --batch-size 1 -sS -p 1-1000 target.com
```

**Optimal Batch Sizes:**

| Batch Size | Syscall Reduction | Use Case |
|-----------|------------------|----------|
| 1 | 0% (no batching) | Debugging, compatibility |
| 16 | ~95% | Low latency, real-time |
| 64 | ~98% | Balanced (recommended) |
| 128 | ~99% | Maximum throughput, batch processing |

**Platform Availability:**

- ✅ Linux 3.0+ (sendmmsg/recvmmsg native)
- ❌ macOS (fallback to send/recv loops)
- ❌ Windows (fallback to send/recv loops)

### Profiling and Benchmarking

**CPU Profiling (Find Bottlenecks):**

```bash
# Generate flamegraph (requires cargo-flamegraph)
cargo install flamegraph
sudo cargo flamegraph --bin prtip -- -sS -p 1-1000 127.0.0.1

# Open flamegraph in browser
firefox flamegraph.svg

# Look for functions consuming >5% CPU
```

**Memory Profiling:**

```bash
# Install valgrind
sudo apt install valgrind

# Profile heap allocations
valgrind --tool=massif prtip -sS -p 1-1000 127.0.0.1

# Analyze results
ms_print massif.out.<pid> | less

# Look for peak memory usage, allocation hotspots
```

**I/O Profiling:**

```bash
# Count syscalls with strace
sudo strace -c prtip -sS -p 1-1000 127.0.0.1

# Expected output:
# % time     seconds  usecs/call     calls    errors syscall
# ------ ----------- ----------- --------- --------- ----------------
#  45.23    0.012345          12      1024           sendmmsg
#  32.11    0.008765           8      1024           recvmmsg
#  ...
```

**Benchmarking:**

```bash
# Install hyperfine
cargo install hyperfine

# Compare scan types (SYN vs Connect)
hyperfine --warmup 3 \
  'prtip -sS -p 1-1000 127.0.0.1' \
  'prtip -sT -p 1-1000 127.0.0.1'

# Output:
# Benchmark 1: prtip -sS ...
#   Time (mean ± σ):      98.3 ms ±   2.1 ms
# Benchmark 2: prtip -sT ...
#   Time (mean ± σ):     152.7 ms ±   3.4 ms
# Summary: SYN is 1.55x faster than Connect
```

---

## Troubleshooting Performance Issues

### Symptom 1: Slow Scans

**Problem:** Scan takes much longer than expected (10x+ slower).

**Potential Causes:**

1. **Timing template too conservative (T0/T1/T2)**

   **Diagnosis:**
   ```bash
   # Check if using slow template
   prtip -sS -p 1-1000 target.com -v | grep -i "timing"
   ```

   **Solution:**
   ```bash
   # Use T3 (normal) or T4 (aggressive)
   prtip -T4 -sS -p 1-1000 target.com
   ```

2. **Rate limiting too aggressive**

   **Diagnosis:**
   ```bash
   # Check current rate limit
   prtip -sS -p 1-1000 target.com -v | grep -i "rate"
   ```

   **Solution:**
   ```bash
   # Increase or disable rate limit
   prtip --max-rate 50000 -sS -p 1-1000 target.com
   ```

3. **Network latency (high RTT)**

   **Diagnosis:**
   ```bash
   # Measure round-trip time
   ping -c 10 target.com
   ```

   **Solution:**
   ```bash
   # Increase parallelism to compensate
   prtip --parallel 8 -sS -p 1-1000 target.com
   ```

4. **Service detection overhead**

   **Diagnosis:**
   ```bash
   # Compare scan with/without -sV
   hyperfine 'prtip -sS -p 80,443 target.com' \
             'prtip -sS -sV -p 80,443 target.com'
   ```

   **Solution:**
   ```bash
   # Disable service detection for speed
   prtip -sS -p 1-1000 target.com  # No -sV

   # Or reduce intensity
   prtip -sS -sV --version-intensity 5 -p 80,443 target.com
   ```

### Symptom 2: Packet Loss

**Problem:** Many ports show "filtered" or no response.

**Potential Causes:**

1. **Firewall dropping packets (rate limiting)**

   **Diagnosis:**
   ```bash
   # Check ICMP "Destination Unreachable" errors
   prtip -sS -p 1-1000 target.com -v 2>&1 | grep -i "unreachable"
   ```

   **Solution:**
   ```bash
   # Reduce scan rate
   prtip --max-rate 1000 -sS -p 1-1000 target.com

   # Or use polite timing
   prtip -T2 -sS -p 1-1000 target.com
   ```

2. **Network congestion (saturated link)**

   **Diagnosis:**
   ```bash
   # Check interface errors
   ifconfig eth0 | grep -i error
   # Look for RX/TX errors, dropped packets
   ```

   **Solution:**
   ```bash
   # Reduce packet rate to 10% of link capacity
   # Example: 100 Mbps link → 10 Mbps scanning
   prtip --max-rate 20000 -sS -p 1-1000 target.com
   ```

3. **Kernel buffer overflow**

   **Diagnosis:**
   ```bash
   # Check kernel buffer statistics
   netstat -s | grep -i "buffer"
   ```

   **Solution:**
   ```bash
   # Increase socket buffers
   sudo sysctl -w net.core.rmem_max=26214400
   sudo sysctl -w net.core.wmem_max=26214400
   ```

### Symptom 3: High Memory Usage

**Problem:** ProRT-IP consuming >1 GB RAM.

**Potential Causes:**

1. **Service detection (many open connections)**

   **Diagnosis:**
   ```bash
   # Monitor memory during scan
   top -p $(pgrep prtip)
   ```

   **Solution:**
   ```bash
   # Limit parallelism
   prtip --parallel 2 -sV -p 80,443 192.168.1.0/24

   # Or stream results to disk
   prtip -sV -p 80,443 192.168.1.0/24 --output-file scan.json
   ```

2. **Large host group (too many concurrent hosts)**

   **Diagnosis:**
   ```bash
   # Check default host group size
   prtip -sS -p 1-1000 192.168.1.0/24 -v | grep -i "hostgroup"
   ```

   **Solution:**
   ```bash
   # Reduce host group size
   prtip --max-hostgroup 16 -sS -p 1-1000 192.168.0.0/16
   ```

3. **Memory leak (rare, report bug)**

   **Diagnosis:**
   ```bash
   # Profile with valgrind
   valgrind --leak-check=full prtip -sS -p 1-1000 target.com
   ```

   **Solution:**
   ```bash
   # Report bug with valgrind output
   # GitHub: https://github.com/doublegate/ProRT-IP/issues
   ```

### Symptom 4: No Results (Empty Output)

**Problem:** Scan completes but no ports detected.

**Potential Causes:**

1. **All ports filtered by firewall**

   **Diagnosis:**
   ```bash
   # Try known-open ports
   prtip -sS -p 80,443 google.com
   ```

   **Solution:**
   ```bash
   # Use different scan type (ACK for firewall detection)
   prtip -sA -p 80,443 target.com

   # Or try UDP scan
   prtip -sU -p 53,161 target.com
   ```

2. **Incorrect target (host down)**

   **Diagnosis:**
   ```bash
   # Verify host is reachable
   ping target.com
   ```

   **Solution:**
   ```bash
   # Skip ping check (assume host up)
   prtip -Pn -sS -p 80,443 target.com
   ```

3. **Permissions issue (no raw socket)**

   **Diagnosis:**
   ```bash
   # Check for permission errors
   prtip -sS -p 80,443 target.com 2>&1 | grep -i "permission"
   ```

   **Solution:**
   ```bash
   # Run with sudo (SYN scan requires root)
   sudo prtip -sS -p 80,443 target.com

   # Or use connect scan (no root needed)
   prtip -sT -p 80,443 target.com
   ```

---

## Capacity Planning

### Estimating Scan Duration

**Formula:**

```
Duration (seconds) = (Hosts × Ports) / Throughput_pps
```

**Example Calculations:**

| Scenario | Hosts | Ports | Throughput | Duration |
|----------|-------|-------|------------|----------|
| Home Network | 10 | 1,000 | 10,000 pps | 1 second |
| Small Office | 100 | 1,000 | 10,000 pps | 10 seconds |
| Data Center | 1,000 | 100 | 10,000 pps | 10 seconds |
| Internet /24 | 256 | 10 | 5,000 pps | <1 second |
| Internet /16 | 65,536 | 10 | 5,000 pps | 131 seconds (~2 min) |

**Adjust for Features:**

| Feature | Duration Multiplier | Example |
|---------|-------------------|---------|
| Service Detection (-sV) | 1.5-2x | 10s → 15-20s |
| OS Fingerprinting (-O) | 1.3-1.5x | 10s → 13-15s |
| Decoy Scanning (-D 3) | 4x | 10s → 40s |
| Timing T0 (Paranoid) | 500x | 10s → 5,000s (83 min) |
| Timing T2 (Polite) | 5x | 10s → 50s |
| Timing T4 (Aggressive) | 0.8x | 10s → 8s |
| Timing T5 (Insane) | 0.6x | 10s → 6s |

### Memory Requirements

**Formula:**

```
Memory (MB) = Baseline + (Hosts × Ports × Overhead_per_port)
```

**Baseline:** 2 MB (ProRT-IP core)

**Overhead per Port:**

| Scan Type | Overhead per Port | Example (10K hosts, 100 ports) |
|-----------|------------------|-------------------------------|
| Stateless (SYN/FIN) | ~100 bytes | 2 MB + (10,000 × 100 × 0.0001) = 102 MB |
| Stateful (Connect) | ~1 KB | 2 MB + (10,000 × 100 × 0.001) = 1,002 MB (~1 GB) |
| Service Detection | ~10 KB | 2 MB + (10,000 × 100 × 0.01) = 10,002 MB (~10 GB) |

**Capacity by Available RAM:**

| Available RAM | Max Hosts | Ports | Scan Type | Notes |
|---------------|-----------|-------|-----------|-------|
| 1 GB | 10,000 | 100 | SYN | Minimal overhead |
| 4 GB | 50,000 | 1,000 | SYN | Typical desktop |
| 16 GB | 200,000 | 1,000 | SYN | Server-class |
| 64 GB | 1,000,000 | 100 | SYN | Internet-scale |

### Network Bandwidth Requirements

**Formula:**

```
Bandwidth_required (Mbps) = (Throughput_pps × Packet_size_bytes × 8) / 1,000,000
```

**Example:**

```
10,000 pps × 60 bytes × 8 bits = 4.8 Mbps
```

**Bandwidth-Based Capacity:**

| Bandwidth | Packet Size | Max PPS | Hosts/Min (1K ports) |
|-----------|-------------|---------|---------------------|
| 1 Mbps | 60 bytes | 2,083 pps | 2 hosts/min |
| 10 Mbps | 60 bytes | 20,833 pps | 20 hosts/min |
| 100 Mbps | 60 bytes | 208,333 pps | 200 hosts/min |
| 1 Gbps | 60 bytes | 2,083,333 pps | 2,000 hosts/min |

---

## Benchmarking Your Setup

### Quick Performance Test

**Baseline (Localhost, 1,000 ports):**

```bash
# Install hyperfine
cargo install hyperfine

# Benchmark SYN scan
hyperfine --warmup 3 'prtip -sS -p 1-1000 127.0.0.1'

# Expected output:
# Time (mean ± σ):      98.3 ms ±   2.1 ms    [User: 45.2 ms, System: 53.1 ms]
# Range (min … max):    95.8 ms … 102.7 ms    10 runs

# Target: <100ms (10,000+ pps)
```

**Compare Scan Types:**

```bash
# Benchmark all scan types
hyperfine --warmup 3 \
  'prtip -sS -p 1-1000 127.0.0.1' \
  'prtip -sF -p 1-1000 127.0.0.1' \
  'prtip -sN -p 1-1000 127.0.0.1' \
  'prtip -sX -p 1-1000 127.0.0.1' \
  'prtip -sA -p 1-1000 127.0.0.1'

# Expected ranking (fastest to slowest):
# 1. SYN (98ms)
# 2. ACK (105ms)
# 3. NULL (113ms)
# 4. FIN (115ms)
# 5. Xmas (118ms)
```

**Timing Template Comparison:**

```bash
# Benchmark T3 vs T4 vs T5
hyperfine --warmup 3 \
  'prtip -T3 -p 1-1000 127.0.0.1' \
  'prtip -T4 -p 1-1000 127.0.0.1' \
  'prtip -T5 -p 1-1000 127.0.0.1'

# Expected speedup:
# T3: 98ms (baseline)
# T4: 78ms (20% faster)
# T5: 59ms (40% faster)
```

### Regression Detection

**Baseline Creation:**

```bash
# Create performance baseline (before changes)
hyperfine --warmup 3 --export-json baseline.json \
  'prtip -sS -p 1-1000 127.0.0.1'

# Baseline: 98.3ms ± 2.1ms
```

**Regression Testing:**

```bash
# After code changes, compare to baseline
hyperfine --warmup 3 --export-json current.json \
  'prtip -sS -p 1-1000 127.0.0.1'

# Current: 105.8ms ± 2.5ms

# Calculate regression
# Regression = (105.8 - 98.3) / 98.3 × 100% = +7.6% (regression detected)
```

**Automated CI/CD Integration:**

```bash
# .github/workflows/benchmarks.yml
# Fail CI if regression >5%
if [ $regression_percent -gt 5 ]; then
  echo "Performance regression detected: ${regression_percent}%"
  exit 1
fi
```

---

## See Also

### Related Guides

- **[Timing & Performance (User Guide)](../user-guide/timing-performance.md)** - Basic timing template usage and examples
- **[Large-Scale Scanning](./large-scale-scanning.md)** - Techniques for scanning millions of hosts
- **[Evasion Techniques](./evasion-techniques.md)** - IDS/firewall bypass strategies (timing is critical)
- **[TUI Architecture](./tui-architecture.md)** - Real-time performance monitoring via TUI

### Feature Guides

- **[Rate Limiting](../features/rate-limiting.md)** - V3 adaptive rate limiter deep dive
- **[Service Detection](../features/service-detection.md)** - Service detection performance characteristics
- **[OS Fingerprinting](../features/os-fingerprinting.md)** - OS detection overhead analysis

### Technical Documentation

- **[Architecture](../00-ARCHITECTURE.md)** - System design, async architecture
- **[Performance Characteristics](../34-PERFORMANCE-CHARACTERISTICS.md)** - Comprehensive benchmarking data
- **[Benchmarking Guide](../31-BENCHMARKING-GUIDE.md)** - Framework usage and methodology

### Command Reference

- **[Timing Templates Reference](../reference/timing-templates.md)** - Complete T0-T5 specifications
- **[Command-Line Reference](../reference/command-reference.md)** - All performance-related flags

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
**Document Status:** Production-ready, Phase 6 (Advanced Topics)
