# Performance Characteristics

ProRT-IP's performance characteristics across all scan types, features, and deployment scenarios.

## Overview

**Key Performance Indicators (v0.5.0):**

| Metric | Value | Competitive Position |
|--------|-------|---------------------|
| **Stateless Throughput** | 10,200 pps (localhost) | Between Nmap (6,600 pps) and Masscan (300K+ pps) |
| **Stateful Throughput** | 6,600 pps (localhost) | Comparable to Nmap (~6,000 pps) |
| **Rate Limiter Overhead** | -1.8% (faster than unlimited) | Industry-leading (Nmap: +5-10%) |
| **Service Detection** | 85-90% accuracy | Nmap-compatible (87-92%) |
| **Memory Footprint** | <1MB stateless, <100MB/10K hosts | Efficient (Nmap: ~50MB/10K hosts) |
| **TLS Parsing** | 1.33μs per certificate | Fast (production-ready) |
| **IPv6 Overhead** | ~15% vs IPv4 | Acceptable (larger headers) |

**Performance Philosophy:**

ProRT-IP balances three competing goals:

1. **Speed:** Masscan-inspired stateless architecture (10M+ pps capable)
2. **Depth:** Nmap-compatible service/OS detection
3. **Safety:** Built-in rate limiting, minimal system impact

---

## Throughput Metrics

### Stateless Scans (SYN/FIN/NULL/Xmas/ACK)

**Localhost Performance (v0.5.0):**

| Scenario | Ports | Mean Time | Throughput | Target |
|----------|-------|-----------|------------|--------|
| SYN Scan | 1,000 | 98ms | 10,200 pps | <100ms ✅ |
| FIN Scan | 1,000 | 115ms | 8,700 pps | <120ms ✅ |
| NULL Scan | 1,000 | 113ms | 8,850 pps | <120ms ✅ |
| Xmas Scan | 1,000 | 118ms | 8,470 pps | <120ms ✅ |
| ACK Scan | 1,000 | 105ms | 9,520 pps | <110ms ✅ |
| Small Scan | 100 | 6.9ms | 14,490 pps | <20ms ✅ |
| All Ports | 65,535 | 4.8s | 13,650 pps | <5s ✅ |

**Network Performance Factors:**

| Environment | Throughput | Limiting Factor |
|-------------|------------|----------------|
| **Localhost (127.0.0.1)** | 10-15K pps | Kernel processing, socket buffers |
| **LAN (1 Gbps)** | 8-12K pps | Network latency (~1ms RTT), switches |
| **LAN (10 Gbps)** | 20-50K pps | CPU bottleneck (packet crafting) |
| **WAN (Internet)** | 1-5K pps | Bandwidth (100 Mbps), RTT (20-100ms) |
| **VPN** | 500-2K pps | Encryption overhead, MTU fragmentation |

**Timing Template Impact:**

| Template | Rate | Use Case | Overhead vs T3 |
|----------|------|----------|----------------|
| **T0 (Paranoid)** | 1-10 pps | IDS evasion, ultra-stealth | +50,000% |
| **T1 (Sneaky)** | 10-50 pps | Slow scanning | +2,000% |
| **T2 (Polite)** | 50-200 pps | Production, low impact | +500% |
| **T3 (Normal)** | 1-5K pps | Default, balanced | Baseline |
| **T4 (Aggressive)** | 5-10K pps | Fast LANs | -20% |
| **T5 (Insane)** | 10-50K pps | Maximum speed | -40% |

### Stateful Scans (Connect, Idle)

**Connect Scan Performance:**

| Scenario | Ports | Mean Time | Throughput | Notes |
|----------|-------|-----------|------------|-------|
| Connect 3 ports | 3 | 45ms | 66 pps | Common ports (22,80,443) |
| Connect 1K ports | 1,000 | 150ms | 6,600 pps | Full handshake overhead |

**Idle Scan Performance:**

| Scenario | Zombie IP | Accuracy | Duration | Notes |
|----------|-----------|----------|----------|-------|
| Idle 1K ports | Local zombie | 99.5% | 1.8s | 16-probe zombie test + scan |
| Idle 100 ports | Remote zombie | 98.2% | 850ms | Network latency factor |

**Why Connect is Slower:**

- Full TCP 3-way handshake (SYN → SYN-ACK → ACK)
- Application-layer interaction (banner grab, service probe)
- Connection tracking overhead (kernel state)

### UDP Scans

**UDP Performance (ICMP-limited):**

| Scenario | Ports | Mean Time | Throughput | Notes |
|----------|-------|-----------|------------|-------|
| UDP 3 ports | 3 (DNS,SNMP,NTP) | 250ms | 12 pps | Wait for ICMP unreachable |
| UDP 100 ports | 100 | 8-12s | 10-12 pps | ICMP rate limiting (Linux: 200/s) |

**UDP Challenges:**

1. **ICMP Rate Limiting:** Linux kernel limits ICMP unreachable to ~200/s
2. **No Response = Open or Filtered:** Ambiguity requires retries
3. **10-100x Slower:** Compared to TCP SYN scans

**Mitigation Strategies:**

- Focus on known UDP services (DNS:53, SNMP:161, NTP:123)
- Use protocol-specific probes (DNS query, SNMP GET)
- Accept longer scan times (UDP is inherently slow)

---

## Latency Metrics

### End-to-End Scan Latency

**Single Port Scan (p50/p95/p99 percentiles):**

| Operation | p50 | p95 | p99 | Notes |
|-----------|-----|-----|-----|-------|
| **SYN Scan (1 port)** | 3.2ms | 4.5ms | 6.1ms | Minimal overhead |
| **Connect Scan (1 port)** | 8.5ms | 12.3ms | 18.7ms | Handshake latency |
| **Service Detection (1 port)** | 45ms | 78ms | 120ms | Probe matching |
| **OS Fingerprinting (1 host)** | 180ms | 250ms | 350ms | 16-probe sequence |
| **TLS Certificate (1 cert)** | 1.33μs | 2.1μs | 3.8μs | X.509 parsing only |

### Component-Level Latency

**Packet Operations:**

| Operation | Latency | Notes |
|-----------|---------|-------|
| **Packet Crafting** | <100μs | Zero-copy serialization |
| **Checksum Calculation** | <50μs | SIMD-optimized |
| **Socket Send (sendmmsg)** | <500μs | Batch 100-500 packets |
| **Socket Receive (recvmmsg)** | <1ms | Poll-based, batch recv |

**Detection Operations:**

| Operation | Latency | Notes |
|-----------|---------|-------|
| **Regex Matching (banner)** | <5ms | Compiled once, lazy_static |
| **Service Probe Matching** | <20ms | 187 probes, parallel |
| **OS Signature Matching** | <50ms | 2,600+ signatures |
| **TLS Certificate Parsing** | 1.33μs | Fast X.509 decode |

**I/O Operations:**

| Operation | Latency | Notes |
|-----------|---------|-------|
| **File Write (JSON)** | <10ms | Buffered async I/O |
| **Database Insert (SQLite)** | <5ms | Batched transactions (1K/tx) |
| **PCAPNG Write** | <2ms | Streaming, no block |

---

## Memory Usage

### Baseline Memory (No Scan)

| Component | Heap | Stack | Total | Notes |
|-----------|------|-------|-------|-------|
| **Binary Size** | - | - | 12.4 MB | Release build, stripped |
| **Runtime Baseline** | 2.1 MB | 8 KB | 2.1 MB | No scan, idle |

### Scan Memory Footprint

**Stateless Scans (SYN/FIN/NULL/Xmas/ACK):**

| Targets | Ports | Memory | Per-Target Overhead | Notes |
|---------|-------|--------|---------------------|-------|
| 1 host | 1,000 | <1 MB | - | Packet buffer pool |
| 100 hosts | 1,000 | 4.2 MB | 42 KB | Target state tracking |
| 10,000 hosts | 1,000 | 92 MB | 9.2 KB | Efficient batching |

**Stateful Scans (Connect):**

| Targets | Ports | Memory | Per-Connection Overhead | Notes |
|---------|-------|--------|------------------------|-------|
| 1 host | 100 | 3.5 MB | 35 KB | Connection tracking |
| 100 hosts | 100 | 18 MB | 180 KB | Async connection pool |
| 10,000 hosts | 10 | 65 MB | 6.5 KB | Batch processing |

**Service Detection Overhead:**

| Component | Memory | Notes |
|-----------|--------|-------|
| **Probe Database** | 2.8 MB | 187 probes, compiled regexes |
| **OS Signature DB** | 4.5 MB | 2,600+ signatures |
| **Per-Service State** | ~50 KB | Banner buffer, probe history |

**Plugin System Overhead:**

| Component | Memory | Notes |
|-----------|--------|-------|
| **Lua VM (base)** | 1.2 MB | Per-plugin VM |
| **Plugin Code** | <500 KB | Typical plugin size |
| **Plugin State** | Varies | User-defined |

**Event System Overhead:**

| Component | Memory | Notes |
|-----------|--------|-------|
| **Event Bus** | <200 KB | Lock-free queue |
| **Event Subscribers** | <50 KB/subscriber | Handler registration |
| **Event Logging** | File-backed | Streaming to disk |

### Memory Optimization

**Buffer Pooling:**

- Packet buffers: Pre-allocated pool of 1,500-byte buffers
- Connection buffers: Reused across connections
- Reduces allocation overhead: 30-40% faster

**Streaming Results:**

- Write results to disk incrementally
- Don't hold all results in memory
- Enables internet-scale scans (1M+ targets)

**Batch Processing:**

- Process targets in batches (default: 64 hosts)
- Release memory after batch completion
- Trade-off: Slight slowdown for memory efficiency

---

## Scaling Characteristics

### Small-Scale (1-100 hosts)

**Characteristics:**

- **Scaling:** Linear (O(n × p), n=hosts, p=ports)
- **Bottleneck:** Network latency (RTT dominates)
- **Memory:** <10 MB (negligible)
- **CPU:** 10-20% single core (packet I/O bound)

**Optimization Tips:**

- Use timing template T4 or T5
- Disable rate limiting for local scans
- Enable parallel host scanning (`--max-hostgroup 64`)

### Medium-Scale (100-10K hosts)

**Characteristics:**

- **Scaling:** Sub-linear (O(n × p / batch_size))
- **Bottleneck:** File descriptors (ulimit), memory
- **Memory:** 10-100 MB (target state)
- **CPU:** 40-60% multi-core (async I/O overhead)

**Optimization Tips:**

- Increase ulimit: `ulimit -n 65535`
- Enable batch processing: `--max-hostgroup 128`
- Use rate limiting: `--max-rate 10000`
- Stream to database or file

### Large-Scale (10K-1M hosts)

**Characteristics:**

- **Scaling:** Batch-linear (O(n × p / batch_size + batch_overhead))
- **Bottleneck:** Bandwidth, rate limiting, disk I/O
- **Memory:** 100-500 MB (batch state, result buffering)
- **CPU:** 80-100% multi-core (packet crafting, async workers)

**Optimization Tips:**

- Mandatory rate limiting: `--max-rate 50000` (internet)
- Large host groups: `--max-hostgroup 256`
- Streaming output: `--output-file scan.json`
- NUMA optimization: `--numa` (multi-socket systems)
- Reduce port count: Focus on critical ports

**Internet-Scale Considerations:**

| Factor | Impact | Mitigation |
|--------|--------|------------|
| **ISP Rate Limiting** | Scan blocked | Lower `--max-rate` to 10-20K pps |
| **IDS/IPS Detection** | IP blacklisted | Use timing template T2, decoys, fragmentation |
| **ICMP Unreachable** | UDP scans fail | Retry logic, increase timeouts |
| **Geo-Latency** | Slowdown | Parallelize across regions |

---

## Feature Overhead Analysis

### Service Detection (-sV)

**Overhead Breakdown:**

| Component | Time | Overhead vs Baseline |
|-----------|------|---------------------|
| **Baseline SYN Scan** | 98ms (1K ports) | - |
| **+ Connect Handshake** | +35ms | +36% |
| **+ Banner Grab** | +12ms | +12% |
| **+ Probe Matching** | +18ms | +18% |
| **Total (-sV)** | 163ms | +66% |

**Per-Service Cost:**

- HTTP: ~15ms (single probe)
- SSH: ~18ms (banner + version probe)
- MySQL: ~35ms (multi-probe sequence)
- Unknown: ~50ms (all 187 probes tested)

**Optimization:**

- Use `--version-intensity 5` (default: 7) for faster scans
- Focus on known ports (80, 443, 22, 3306, 5432)
- Enable regex caching (done automatically)

### OS Fingerprinting (-O)

**Overhead Breakdown:**

| Component | Time | Overhead vs Baseline |
|-----------|------|---------------------|
| **Baseline SYN Scan** | 98ms (1K ports) | - |
| **+ 16 OS Probes** | +120ms | +122% |
| **+ Signature Matching** | +15ms | +15% |
| **Total (-O)** | 233ms | +138% |

**Accuracy vs Speed:**

- Requires both open and closed ports (ideal: 1 open, 1 closed)
- Accuracy: 75-85% (Nmap-compatible)
- Use `--osscan-limit` to skip hosts without detectable OS

### IPv6 Overhead (--ipv6 or :: notation)

**Overhead Breakdown:**

| Component | Overhead | Reason |
|-----------|----------|--------|
| **Packet Size** | +40 bytes | IPv6 header (40B) vs IPv4 (20B) |
| **Throughput** | +15% | Larger packets, same rate |
| **Memory** | +10% | Larger addresses (128-bit vs 32-bit) |

**ICMPv6 vs ICMP:**

- ICMPv6 more complex (NDP, router advertisements)
- Overhead: +20-30% for UDP scans
- Feature parity: 100% (Sprint 5.1 completion)

### TLS Certificate Analysis (--tls-cert-analysis)

**Overhead Breakdown:**

| Component | Time | Overhead vs HTTPS Scan |
|-----------|------|----------------------|
| **HTTPS Connection** | 45ms | Baseline (TLS handshake) |
| **+ Certificate Download** | +8ms | Download cert chain |
| **+ X.509 Parsing** | +0.00133ms | Negligible (1.33μs) |
| **+ Chain Validation** | +3ms | Verify signatures |
| **Total** | 56ms | +24% |

**Parsing Performance:**

- 1.33μs per certificate (mean)
- Handles chains up to 10 certificates
- SNI support (virtual hosts)

### Evasion Techniques

**Packet Fragmentation (-f):**

| Scenario | Overhead | Reason |
|----------|----------|--------|
| SYN Scan | +18% | Extra packet crafting, 2x packets |

**Decoy Scanning (-D):**

| Decoys | Overhead | Traffic Multiplier |
|--------|----------|-------------------|
| 1 decoy | +100% | 2x traffic (1 decoy + 1 real) |
| 3 decoys | +300% | 4x traffic (3 decoys + 1 real) |
| 10 decoys | +1000% | 11x traffic (10 decoys + 1 real) |

**Source Port Evasion (-g):**

| Technique | Overhead | Effectiveness |
|-----------|----------|---------------|
| Fixed source port | <1% | Bypasses simple firewalls |
| Random source ports | 0% | Default behavior |

### Event System (Sprint 5.5.3)

**Overhead Breakdown:**

| Scenario | Baseline | With Events | Overhead |
|----------|----------|-------------|----------|
| SYN 1K ports | 98ms | 102ms | +4.1% |
| Connect 100 ports | 150ms | 154ms | +2.7% |

**Event Types:**

- Scan start/stop
- Host discovery
- Port state change
- Service detected
- Error events

**Performance Impact:**

- Lock-free event bus: Minimal contention
- Async event dispatch: Non-blocking
- Event logging: Buffered I/O (10-20ms flush interval)

### Rate Limiting (V3 Adaptive)

**Overhead Breakdown (Sprint 5.X optimization):**

| Scenario | No Rate Limit | With Rate Limit | Overhead |
|----------|---------------|-----------------|----------|
| SYN 1K ports | 99.8ms | 98.0ms | **-1.8%** ✅ |
| Connect 100 | 151ms | 149ms | **-1.3%** ✅ |

**Why Faster:**

- Convergence algorithm optimizes system-wide flow
- Reduces kernel queue overflow
- Better CPU cache utilization
- Industry-leading result (Nmap: +5-10%, Masscan: N/A)

**Burst Behavior:**

- Burst size: 100 packets (optimal)
- Convergence: 95% in <500ms
- Adaptive: ICMP error monitoring

---

## Optimization Guide

### System Tuning

**File Descriptor Limits:**

```bash
# Check current limit
ulimit -n

# Increase to 65535 (temporary)
ulimit -n 65535

# Permanent (add to /etc/security/limits.conf)
* soft nofile 65535
* hard nofile 65535
```

**Why:** Each connection requires 1 file descriptor. Default limit (1024) insufficient for large scans.

**Network Tuning (Linux):**

```bash
# Increase socket buffer sizes
sysctl -w net.core.rmem_max=26214400
sysctl -w net.core.wmem_max=26214400

# Increase connection backlog
sysctl -w net.core.netdev_max_backlog=5000

# Reduce TIME_WAIT duration (careful!)
sysctl -w net.ipv4.tcp_fin_timeout=15
```

**Why:** Larger buffers accommodate high packet rates, reduced TIME_WAIT prevents port exhaustion.

**NUMA Optimization (Multi-Socket Systems):**

```bash
# Check NUMA topology
numactl --hardware

# Run with NUMA optimization
prtip --numa -sS -p 1-65535 192.168.1.0/24

# Or manual binding (advanced)
numactl --cpunodebind=0 --membind=0 prtip -sS ...
```

**Why:** Avoids cross-NUMA memory access penalties (30-50% latency penalty).

### ProRT-IP Tuning

**Timing Templates:**

| Use Case | Template | Command |
|----------|----------|---------|
| Localhost | T5 (Insane) | `prtip -T5 -p 1-1000 127.0.0.1` |
| LAN | T4 (Aggressive) | `prtip -T4 -p 1-1000 192.168.1.0/24` |
| Internet | T3 (Normal) | `prtip -T3 -p 80,443 target.com` |
| Stealth | T2 (Polite) | `prtip -T2 -p 1-1000 target.com` |
| IDS Evasion | T0 (Paranoid) | `prtip -T0 -p 80,443 target.com` |

**Host Group Sizing:**

```bash
# Default (64 concurrent hosts)
prtip -sS -p 1-1000 192.168.0.0/16

# Increase for speed (256 concurrent)
prtip --max-hostgroup 256 -sS -p 1-1000 192.168.0.0/16

# Decrease for memory (16 concurrent)
prtip --max-hostgroup 16 -sS -p 1-65535 192.168.0.0/16
```

**Rate Limiting:**

```bash
# Localhost: Disable (safe)
prtip -sS -p 1-1000 127.0.0.1

# LAN: 50K pps
prtip --max-rate 50000 -sS -p 1-1000 192.168.1.0/24

# Internet: 10K pps (safe)
prtip --max-rate 10000 -sS -p 80,443 target.com/24

# Stealth: 1K pps
prtip --max-rate 1000 -T2 -p 80,443 target.com/24
```

### Performance Checklist

**Before Large Scans:**

- [ ] Increase ulimit: `ulimit -n 65535`
- [ ] Set appropriate timing template (T3 for internet, T4 for LAN)
- [ ] Enable rate limiting: `--max-rate 10000` (internet)
- [ ] Stream results: `--output-file scan.json`
- [ ] Test small subset first: `-p 80,443 target.com` (verify connectivity)
- [ ] Monitor system resources: `htop`, `iotop`, `iftop`

**During Scans:**

- [ ] Watch for ICMP errors (rate limiting)
- [ ] Monitor packet loss: `ifconfig` (check RX/TX errors)
- [ ] Check event log for errors: `--event-log events.jsonl`
- [ ] Verify results incrementally (spot-check)

**After Scans:**

- [ ] Analyze results for anomalies
- [ ] Check scan duration vs estimate
- [ ] Review error log for issues
- [ ] Archive results: `benchmarks/history/`

---

## Capacity Planning

### How Many Hosts Can I Scan?

**Memory-Based Capacity:**

| Available RAM | Max Hosts | Ports | Scan Type | Notes |
|---------------|-----------|-------|-----------|-------|
| 1 GB | 10,000 | 100 | SYN | Minimal overhead |
| 4 GB | 50,000 | 1,000 | SYN | Typical desktop |
| 16 GB | 200,000 | 1,000 | SYN | Server-class |
| 64 GB | 1,000,000 | 100 | SYN | Internet-scale |

**Network-Based Capacity:**

| Bandwidth | Packet Size | Max PPS | Hosts/Min (1K ports) |
|-----------|-------------|---------|---------------------|
| 1 Mbps | 60 bytes | 2,083 pps | 2 hosts/min |
| 10 Mbps | 60 bytes | 20,833 pps | 20 hosts/min |
| 100 Mbps | 60 bytes | 208,333 pps | 200 hosts/min |
| 1 Gbps | 60 bytes | 2,083,333 pps | 2,000 hosts/min |

**Formula:**

```
Hosts/Min = (Bandwidth_bps / (Packet_Size_bytes × 8)) / Ports_per_host
```

### How Long Will My Scan Take?

**Estimation Formula:**

```
Duration (sec) = (Hosts × Ports) / Throughput_pps
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

| Feature | Duration Multiplier |
|---------|-------------------|
| Service Detection (-sV) | 1.5-2x |
| OS Fingerprinting (-O) | 1.3-1.5x |
| Decoy Scanning (-D 3 decoys) | 4x |
| Timing T0 (Paranoid) | 500x |
| Timing T2 (Polite) | 5x |
| Timing T4 (Aggressive) | 0.8x |
| Timing T5 (Insane) | 0.6x |

### What Hardware Do I Need?

**CPU Requirements:**

| Scan Type | Min CPU | Recommended CPU | Notes |
|-----------|---------|-----------------|-------|
| Stateless (SYN) | 1 core, 2 GHz | 4 cores, 3 GHz | Packet crafting CPU-bound |
| Stateful (Connect) | 2 cores, 2 GHz | 8 cores, 3 GHz | Async I/O parallelism |
| Service Detection | 2 cores, 2 GHz | 4 cores, 3 GHz | Regex matching CPU-bound |
| Internet-Scale | 8 cores, 3 GHz | 16 cores, 3.5 GHz | Multi-socket NUMA |

**RAM Requirements:**

| Scan Scale | Min RAM | Recommended RAM | Notes |
|------------|---------|-----------------|-------|
| Small (<100 hosts) | 512 MB | 1 GB | Minimal overhead |
| Medium (<10K hosts) | 1 GB | 4 GB | Comfortable buffer |
| Large (<100K hosts) | 4 GB | 16 GB | Batch processing |
| Internet-Scale (1M+) | 16 GB | 64 GB | Streaming required |

**Network Requirements:**

| Scan Type | Min Bandwidth | Recommended Bandwidth |
|-----------|---------------|----------------------|
| Localhost | N/A | N/A |
| LAN (1 Gbps) | 10 Mbps | 100 Mbps |
| LAN (10 Gbps) | 100 Mbps | 1 Gbps |
| Internet | 10 Mbps | 100 Mbps |

**Storage Requirements:**

| Result Format | Storage per Host | Storage for 100K Hosts |
|---------------|------------------|----------------------|
| Text | ~500 bytes | 50 MB |
| JSON | ~1 KB | 100 MB |
| XML (Nmap) | ~1.5 KB | 150 MB |
| PCAPNG | ~50 KB | 5 GB |
| SQLite | ~800 bytes | 80 MB |

---

## Platform Differences

### Linux (Primary Platform)

**Advantages:**

- Native `sendmmsg`/`recvmmsg` support (fast batching)
- AF_PACKET sockets (raw packet access)
- NUMA support (`numactl`)
- Best performance: 10-15K pps localhost

**Limitations:**

- Requires root/CAP_NET_RAW for raw sockets
- ICMP rate limiting (200 unreachable/s)

### macOS

**Advantages:**

- BPF (Berkeley Packet Filter) support
- Good Nmap compatibility

**Limitations:**

- No `sendmmsg`/`recvmmsg` (fallback to send/recv loops)
- Slower: 6-8K pps localhost
- ChmodBPF required for raw socket access

### Windows

**Advantages:**

- Npcap library support (WinPcap successor)

**Limitations:**

- Slower raw socket access: 4-6K pps
- FIN/NULL/Xmas scans unsupported (Windows TCP stack limitation)
- Npcap installation required
- UAC elevation for raw sockets

**Platform Performance Comparison:**

| Platform | SYN Scan (1K) | Connect (100) | Notes |
|----------|---------------|---------------|-------|
| **Linux** | 98ms | 150ms | Best performance |
| **macOS** | 145ms | 180ms | BPF overhead |
| **Windows** | 210ms | 220ms | Npcap overhead |

---

## See Also

- [Performance Tuning](./performance-tuning.md) - Detailed optimization techniques
- [Benchmarking](./benchmarking.md) - Benchmark framework and methodology
- [Rate Limiting](../features/rate-limiting.md) - V3 algorithm details
- [Service Detection](../features/service-detection.md) - Detection overhead analysis
- [Platform Support](../features/platform-support.md) - Platform-specific characteristics
