# Performance Benchmarking Guide

**Last Updated:** 2025-10-09
**Project Version:** v0.3.0

## Overview

This document provides comprehensive guidance for benchmarking ProRT-IP WarScan performance before, during, and after Phase 4 optimizations. Establishing baseline metrics is critical for measuring optimization improvements.

**Phase 3 Baseline Results:** See [BASELINE-RESULTS.md](BASELINE-RESULTS.md) for the complete v0.3.0 performance baseline established on 2025-10-09.

---

## Benchmarking Methodology

### Prerequisites

**Hardware Requirements:**
- Isolated test network (no production traffic)
- Test target with known open/closed/filtered ports
- Dedicated network interface (no competing traffic)
- Stable system load (no background processes)

**Software Requirements:**
- ProRT-IP v0.3.0+ installed
- Comparison tools: Nmap 7.90+, Masscan 1.3+, RustScan 2.0+ (optional)
- Monitoring tools: `htop`, `iftop`, `perf`, `flamegraph`
- Test target: Metasploitable VM, Docker container, or dedicated test server

**Environment Setup:**
```bash
# Disable CPU frequency scaling
sudo cpupower frequency-set -g performance

# Increase file descriptor limits
ulimit -n 100000

# Disable firewall on test machine (if safe)
sudo ufw disable  # Ubuntu/Debian
sudo systemctl stop firewalld  # RHEL/CentOS

# Set up test target (Docker example)
docker run -d -p 21:21 -p 22:22 -p 80:80 -p 443:443 --name test-target metasploitablevm/metasploitable2
```

---

## Baseline Metrics (Phase 3 - Pre-Optimization)

These baselines should be established before Phase 4 optimizations begin.

### Test Scenarios

#### 1. TCP Connect Scan - Common Ports (1,000 ports)

**Command:**
```bash
time prtip -sT -p 1-1000 192.168.1.100 --no-progress
```

**Expected Baseline:**
- **Duration:** 5-10 seconds (depends on network latency)
- **Throughput:** 100-200 ports/second
- **Memory:** ~10-20 MB
- **CPU:** 5-15% (single core)

**Metrics to Record:**
- Total scan duration (wall clock time)
- Ports per second rate
- Peak memory usage (`/usr/bin/time -v`)
- Average CPU utilization
- Open/closed/filtered port counts

---

#### 2. SYN Scan - Full Port Range (65,535 ports)

**Command:**
```bash
time sudo prtip -sS -p- 192.168.1.100 -T4 --no-progress
```

**Expected Baseline:**
- **Duration:** 30-60 seconds (T4 timing)
- **Throughput:** 1,000-2,000 ports/second
- **Memory:** ~50-100 MB
- **CPU:** 20-40% (multi-core)

**Metrics to Record:**
- Total scan duration
- Ports per second rate
- Connection pool size and utilization
- Packet loss rate (if measurable)
- Network bandwidth utilization

---

#### 3. UDP Scan - Top 100 UDP Ports

**Command:**
```bash
time sudo prtip -sU -p U:53,123,161,137,500,1900,5353 192.168.1.100 --no-progress
```

**Expected Baseline:**
- **Duration:** 20-40 seconds (UDP is slower)
- **Throughput:** 10-50 ports/second (ICMP rate limiting)
- **Memory:** ~10-20 MB
- **CPU:** 5-10%

**Metrics to Record:**
- Total scan duration
- Protocol payload effectiveness
- ICMP responses vs timeouts
- False positive rate (open|filtered)

---

#### 4. OS Fingerprinting + Service Detection

**Command:**
```bash
time sudo prtip -sS -O --sV -p 1-1000 192.168.1.100 -T4 --no-progress
```

**Expected Baseline:**
- **Duration:** 15-30 seconds
- **Throughput:** 30-70 ports/second (slower due to probing)
- **Memory:** ~100-200 MB (database loaded)
- **CPU:** 30-50%

**Metrics to Record:**
- Total scan duration
- Accuracy of OS detection (compare with Nmap)
- Service detection success rate
- Database loading time
- Probe sequence efficiency

---

#### 5. Timing Template Comparison

**Commands:**
```bash
# T0 (Paranoid)
time sudo prtip -sS -p 1-100 192.168.1.100 -T0 --no-progress

# T3 (Normal - baseline)
time sudo prtip -sS -p 1-100 192.168.1.100 -T3 --no-progress

# T5 (Insane)
time sudo prtip -sS -p 1-100 192.168.1.100 -T5 --no-progress
```

**Expected Baseline:**
- **T0:** 500-600 seconds (5-minute delays)
- **T3:** 10-20 seconds
- **T5:** 2-5 seconds (aggressive)

**Metrics to Record:**
- Duration per timing template
- Accuracy vs speed tradeoff
- Packet loss at T5
- IDS detection rate (if testing stealth)

---

## Performance Profiling

### CPU Profiling with `perf`

**Record Performance Data:**
```bash
# Build with debug symbols
RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes" cargo build --release

# Profile a scan
sudo perf record --call-graph dwarf -F 997 ./target/release/prtip -sS -p 1-1000 192.168.1.100

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg

# View in browser
firefox flame.svg
```

**Key Areas to Analyze:**
- Hot paths (functions consuming >5% CPU)
- Lock contention (spinlock cycles)
- Memory allocations (malloc/free calls)
- System calls (sendto/recvfrom overhead)

---

### Memory Profiling with `valgrind`

**Check for Memory Leaks:**
```bash
valgrind --leak-check=full --show-leak-kinds=all ./target/release/prtip -sT -p 1-100 192.168.1.100
```

**Analyze Memory Usage:**
```bash
/usr/bin/time -v ./target/release/prtip -sS -p- 192.168.1.100 2>&1 | grep -E "Maximum resident set size|User time|System time"
```

**Key Metrics:**
- Peak resident set size (RSS)
- Heap allocations per scan
- Memory leaks (should be zero)

---

### Network Profiling with `iftop`

**Monitor Real-Time Bandwidth:**
```bash
sudo iftop -i eth0 -f "host 192.168.1.100"
```

**Key Metrics:**
- Peak bandwidth utilization (Mbps)
- Packet rate (packets/second)
- Burst patterns vs steady state

---

## Comparative Benchmarking

### vs. Nmap (Detection Accuracy)

**ProRT-IP:**
```bash
time sudo prtip -sS -O --sV -p 1-1000 192.168.1.100 --output json > prortip-results.json
```

**Nmap:**
```bash
time sudo nmap -sS -O -sV -p 1-1000 192.168.1.100 -oX nmap-results.xml
```

**Compare:**
- **Accuracy:** OS detection match percentage
- **Speed:** ProRT-IP should be 1.5-2x faster (baseline)
- **Results:** Compare open port lists, service versions, OS guesses

---

### vs. Masscan (Raw Speed)

**ProRT-IP (T5):**
```bash
time sudo prtip -sS -p- 192.168.1.100 -T5 --no-progress
```

**Masscan:**
```bash
time sudo masscan -p 1-65535 192.168.1.100 --rate 10000
```

**Compare:**
- **Speed:** Masscan will be faster (stateless, ~10x)
- **Accuracy:** ProRT-IP should have fewer false positives
- **Memory:** ProRT-IP will use more memory (stateful tracking)

**Note:** Phase 4 goal is to narrow this gap significantly.

---

### vs. RustScan (Rust Ecosystem)

**ProRT-IP:**
```bash
time sudo prtip -sT -p- 192.168.1.100 --no-progress
```

**RustScan:**
```bash
time rustscan -a 192.168.1.100 --range 1-65535 --batch-size 5000
```

**Compare:**
- **Speed:** Should be comparable (both use FuturesUnordered)
- **Features:** ProRT-IP has more scan types and detection
- **Resource usage:** Compare ulimit handling and concurrency

---

## Phase 4 Optimization Targets

After Phase 4 optimizations, these improvements are expected:

### Performance Improvements

| Metric | Phase 3 Baseline | Phase 4 Target | Improvement |
|--------|------------------|----------------|-------------|
| **TCP SYN Scan (65K ports)** | 30-60s | 10-20s | **3x faster** |
| **Throughput (ports/sec)** | 1,000-2,000 | 10,000+ | **5-10x** |
| **Memory Usage** | 50-100 MB | 30-50 MB | **40% reduction** |
| **CPU Efficiency** | 20-40% | 50-70% | **Better utilization** |
| **Lock Contention** | Moderate | Near-zero | **Lock-free** |

### Specific Optimizations

**1. Lock-Free Data Structures (crossbeam):**
- Replace `Arc<Mutex<HashMap>>` with lock-free alternatives
- Target: Eliminate >90% of lock contention

**2. Batched Syscalls (sendmmsg/recvmmsg):**
- Batch 64-1024 packets per syscall
- Target: 30-50% speedup at 1M+ pps

**3. NUMA-Aware Thread Placement:**
- Pin threads to NUMA nodes
- Target: 10-30% speedup on multi-socket systems

**4. Stateless Scanning Mode:**
- Use Blackrock shuffling and SipHash
- Target: Masscan-like speeds (1M+ pps)

---

## Benchmarking Commands Reference

### Quick Benchmarking Suite

```bash
#!/bin/bash
# benchmark.sh - Quick performance test suite

TARGET="192.168.1.100"
RESULTS="benchmark-results.txt"

echo "ProRT-IP Performance Benchmark Suite" > $RESULTS
echo "Date: $(date)" >> $RESULTS
echo "Version: $(prtip --version)" >> $RESULTS
echo "---" >> $RESULTS

# Test 1: TCP Connect - Common Ports
echo "Test 1: TCP Connect Scan (1000 ports)" >> $RESULTS
/usr/bin/time -v prtip -sT -p 1-1000 $TARGET --no-progress 2>&1 | grep -E "Elapsed|Maximum resident" >> $RESULTS

# Test 2: SYN Scan - Full Range
echo "Test 2: SYN Scan (65535 ports, T4)" >> $RESULTS
/usr/bin/time -v sudo prtip -sS -p- $TARGET -T4 --no-progress 2>&1 | grep -E "Elapsed|Maximum resident" >> $RESULTS

# Test 3: Detection Scan
echo "Test 3: OS + Service Detection (1000 ports)" >> $RESULTS
/usr/bin/time -v sudo prtip -sS -O --sV -p 1-1000 $TARGET -T4 --no-progress 2>&1 | grep -E "Elapsed|Maximum resident" >> $RESULTS

# Test 4: Timing Comparison
echo "Test 4: Timing Template T3 (100 ports)" >> $RESULTS
/usr/bin/time -v sudo prtip -sS -p 1-100 $TARGET -T3 --no-progress 2>&1 | grep "Elapsed" >> $RESULTS

echo "Test 4b: Timing Template T5 (100 ports)" >> $RESULTS
/usr/bin/time -v sudo prtip -sS -p 1-100 $TARGET -T5 --no-progress 2>&1 | grep "Elapsed" >> $RESULTS

echo "---" >> $RESULTS
echo "Benchmark complete. Results saved to $RESULTS"
cat $RESULTS
```

**Run Benchmarks:**
```bash
chmod +x benchmark.sh
./benchmark.sh
```

---

## Platform-Specific Considerations

### Linux

**Optimal Settings:**
```bash
# Increase receive buffer size
sudo sysctl -w net.core.rmem_max=26214400
sudo sysctl -w net.core.rmem_default=26214400

# Increase send buffer size
sudo sysctl -w net.core.wmem_max=26214400
sudo sysctl -w net.core.wmem_default=26214400

# Reduce TIME_WAIT timeout
sudo sysctl -w net.ipv4.tcp_fin_timeout=15

# Enable TCP window scaling
sudo sysctl -w net.ipv4.tcp_window_scaling=1
```

### macOS

**Considerations:**
- BPF buffer size is limited
- Packet capture is slower than Linux
- Expect 20-30% slower than Linux baseline

### Windows

**Considerations:**
- Npcap overhead adds ~10-20% latency
- Administrator privileges required
- Timing tests may be 2-3x slower due to Npcap

---

## Regression Testing

### Automated Performance Tests

Create regression tests to detect performance degradation:

```bash
#!/bin/bash
# regression-test.sh

BASELINE_TIME=30  # seconds for full port scan
TOLERANCE=20      # 20% tolerance

ACTUAL_TIME=$(sudo prtip -sS -p- 192.168.1.100 -T4 --no-progress 2>&1 | grep "Elapsed" | awk '{print $3}')

if [ $ACTUAL_TIME -gt $((BASELINE_TIME + BASELINE_TIME * TOLERANCE / 100)) ]; then
    echo "FAIL: Performance regression detected (${ACTUAL_TIME}s > ${BASELINE_TIME}s + ${TOLERANCE}%)"
    exit 1
else
    echo "PASS: Performance within acceptable range (${ACTUAL_TIME}s)"
    exit 0
fi
```

**Run in CI:**
```yaml
# .github/workflows/performance.yml
- name: Performance Regression Test
  run: ./regression-test.sh
```

---

## Results Documentation

### Recording Benchmark Results

**Template:**
```
ProRT-IP Performance Benchmark
==============================
Date: 2025-10-09
Version: v0.3.0
Phase: 3 (Pre-Optimization)
Platform: Linux x86_64
CPU: Intel i7-9700K @ 3.6 GHz (8 cores)
RAM: 32 GB DDR4-3200
Network: 1 Gbps Ethernet
Target: 192.168.1.100 (Metasploitable VM)

Test 1: TCP Connect Scan (1000 ports)
---------------------------------------
Duration: 8.42 seconds
Throughput: 118.7 ports/second
Memory: 15.3 MB
CPU: 12%

Test 2: SYN Scan (65535 ports, T4)
-----------------------------------
Duration: 42.1 seconds
Throughput: 1,556 ports/second
Memory: 73.2 MB
CPU: 35%

[... additional tests ...]
```

---

## Next Steps

1. **Establish Baselines:** Run all benchmark tests and record results
2. **Identify Bottlenecks:** Profile with perf/flamegraph
3. **Implement Phase 4:** Lock-free data structures, batched syscalls, NUMA
4. **Re-benchmark:** Compare Phase 4 results against baselines
5. **Verify Improvements:** Ensure 3-5x speedup targets are met
6. **Regression Testing:** Add automated performance tests to CI

---

## References

- [07-PERFORMANCE.md](07-PERFORMANCE.md) - Performance optimization techniques
- [02-TECHNICAL-SPECS.md](02-TECHNICAL-SPECS.md) - System requirements
- [01-ROADMAP.md](01-ROADMAP.md) - Phase 4 performance optimization plan

---

**Maintained by:** ProRT-IP WarScan Development Team
**Questions?** Open a GitHub issue with label `performance`
