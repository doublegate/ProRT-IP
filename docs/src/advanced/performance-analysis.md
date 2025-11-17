# Performance Analysis

ProRT-IP provides comprehensive performance analysis tools for measuring, profiling, and optimizing network scanning operations. This guide covers methodologies, tools, and techniques for identifying bottlenecks and improving scan performance.

## Overview

**Performance Analysis Goals:**
- Identify bottlenecks (CPU, memory, network, I/O)
- Validate optimization improvements
- Detect performance regressions
- Ensure production readiness

**Key Metrics:**
- **Throughput**: Packets per second (pps), ports per minute
- **Latency**: End-to-end scan time, per-operation timing
- **Resource Usage**: CPU utilization, memory footprint, I/O load
- **Scalability**: Multi-core efficiency, NUMA performance

## Benchmarking Tools

### Criterion.rs Benchmarks

ProRT-IP includes comprehensive Criterion.rs benchmarks for micro-benchmarking critical components.

**Running Benchmarks:**

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench --bench packet_crafting

# Save baseline for comparison
cargo bench --save-baseline before

# Compare against baseline
# ... make changes ...
cargo bench --baseline before

# View HTML report
firefox target/criterion/report/index.html
```

**Example Benchmark Results:**

```
tcp_syn_packet          time:   [850.23 ns 862.41 ns 875.19 ns]
                        change: [-2.3421% -1.1234% +0.4521%] (p = 0.18 > 0.05)
                        No change in performance detected.

udp_packet              time:   [620.15 ns 628.92 ns 638.47 ns]
                        change: [-3.1234% -2.5678% -1.9876%] (p = 0.00 < 0.05)
                        Performance has improved.
```

**Interpreting Results:**
- **Time ranges**: [lower_bound mean upper_bound] with 95% confidence intervals
- **Change percentage**: Positive = slower, negative = faster
- **p-value**: <0.05 indicates statistically significant change
- **Throughput**: Derived from mean time (e.g., 862ns → 1.16M packets/sec)

### Hyperfine Benchmarking

Hyperfine provides statistical end-to-end performance measurement for complete scans.

**Installation:**

```bash
# Linux/macOS
cargo install hyperfine

# Or download from https://github.com/sharkdp/hyperfine
```

**Basic Usage:**

```bash
# Simple benchmark
hyperfine 'prtip -sS -p 1-1000 127.0.0.1'

# Compare different scan types
hyperfine --warmup 3 --runs 10 \
    'prtip -sS -p 1-1000 127.0.0.1' \
    'prtip -sT -p 1-1000 127.0.0.1'

# Export results
hyperfine --export-json results.json \
    'prtip -sS -p- 127.0.0.1'
```

**Example Output:**

```
Benchmark 1: prtip -sS -p 1-1000 127.0.0.1
  Time (mean ± σ):      98.3 ms ±   2.4 ms    [User: 12.5 ms, System: 45.2 ms]
  Range (min … max):    95.1 ms … 104.2 ms    10 runs

Benchmark 2: prtip -sT -p 1-1000 127.0.0.1
  Time (mean ± σ):     150.7 ms ±   3.1 ms    [User: 18.3 ms, System: 52.1 ms]
  Range (min … max):   146.5 ms … 157.2 ms    10 runs

Summary
  'prtip -sS -p 1-1000 127.0.0.1' ran
    1.53 ± 0.04 times faster than 'prtip -sT -p 1-1000 127.0.0.1'
```

**Advanced Hyperfine Features:**

```bash
# Parameter sweeping
hyperfine --warmup 3 --parameter-scan rate 1000 10000 1000 \
    'prtip --max-rate {rate} -sS -p 80,443 192.168.1.0/24'

# Preparation commands
hyperfine --prepare 'sudo sync; sudo sysctl vm.drop_caches=3' \
    'prtip -sS -p- 127.0.0.1'

# Time units
hyperfine --time-unit millisecond 'prtip -sS -p 1-1000 127.0.0.1'
```

## CPU Profiling

### perf (Linux)

The `perf` tool provides low-overhead CPU profiling with flamegraph visualization.

**Build with Debug Symbols:**

```bash
# Enable debug symbols in release mode
RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes" cargo build --release
```

**Record Performance Data:**

```bash
# Basic profiling (requires root or perf_event_paranoid=-1)
sudo perf record --call-graph dwarf -F 997 \
    ./target/release/prtip -sS -p 1-1000 10.0.0.0/24

# Interactive analysis
perf report

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
firefox flame.svg
```

**Key Metrics to Monitor:**

- **CPU cycles** in packet crafting functions (<10% of total)
- **Cache misses** in hot paths (<5% L1d misses)
- **Branch mispredictions** (<2% of branches)
- **Lock contention** (should be minimal with lock-free design)

**Common Bottlenecks:**

```bash
# High lock contention
perf record -e lock:contention_begin ./target/release/prtip [args]
perf report

# Cache misses
perf stat -e cache-misses,cache-references ./target/release/prtip [args]

# Branch mispredictions
perf stat -e branches,branch-misses ./target/release/prtip [args]
```

**Example Analysis:**

```
# perf report output
Overhead  Command  Shared Object       Symbol
   45.2%  prtip    prtip               [.] prtip_network::tcp::send_syn
   12.3%  prtip    prtip               [.] prtip_scanner::syn_scanner::scan_port
    8.7%  prtip    libc-2.31.so        [.] __pthread_mutex_lock
    5.4%  prtip    prtip               [.] crossbeam::queue::pop
```

**Interpretation:**
- 45% time in packet sending (expected for network I/O)
- 8.7% time in mutex locks (optimization target - switch to lock-free)
- 5.4% time in queue operations (efficient crossbeam implementation)

### Instruments (macOS)

macOS users can use Xcode Instruments for profiling.

**Basic Profiling:**

```bash
# Time Profiler
instruments -t "Time Profiler" ./target/release/prtip -sS -p 1-1000 127.0.0.1

# Allocations
instruments -t "Allocations" ./target/release/prtip -sS -p 1-1000 127.0.0.1

# System Trace (comprehensive)
instruments -t "System Trace" ./target/release/prtip -sS -p 1-1000 127.0.0.1
```

## Memory Profiling

### Valgrind Massif

Massif provides heap profiling for memory usage analysis.

**Heap Profiling:**

```bash
# Run massif
valgrind --tool=massif \
    --massif-out-file=massif.out \
    ./target/release/prtip -sS -p 80,443 10.0.0.0/24

# Analyze results
ms_print massif.out > massif.txt
less massif.txt

# Or use massif-visualizer GUI
massif-visualizer massif.out
```

**Expected Memory Usage:**

| Operation | Memory Usage | Notes |
|-----------|--------------|-------|
| Base binary | ~5 MB | Minimal static footprint |
| Stateless scan (1M targets) | <100 MB | O(1) state via SipHash |
| Stateful scan (1K active conns) | ~50 MB | ~50KB per connection |
| Stateful scan (100K active conns) | ~5 GB | Connection state dominates |
| Result storage (1M entries) | ~250 MB | In-memory before DB write |
| OS fingerprint DB | ~10 MB | 2,000+ fingerprints loaded |
| Service probe DB | ~5 MB | 500+ probes loaded |

### Memory Leak Detection

```bash
# Full leak check
valgrind --leak-check=full \
    --show-leak-kinds=all \
    --track-origins=yes \
    ./target/debug/prtip [args]
```

**Expected Results:**
- **Definitely lost:** 0 bytes (no memory leaks)
- **Possibly lost:** <1KB (from static initializers)
- **Peak heap usage:** Matches expected memory targets

**Common Memory Issues:**

1. **Connection state accumulation**: Not cleaning up completed connections
2. **Result buffer overflow**: Not streaming results to disk
3. **Fragmentation**: Fixed-size allocations create holes

## I/O Profiling

### strace (Linux)

System call tracing reveals I/O bottlenecks.

**Basic Tracing:**

```bash
# Trace all syscalls
strace -c ./target/release/prtip -sS -p 80,443 127.0.0.1

# Trace network syscalls only
strace -e trace=network ./target/release/prtip -sS -p 80,443 127.0.0.1

# Detailed timing
strace -tt -T ./target/release/prtip -sS -p 80,443 127.0.0.1
```

**Example Summary:**

```
% time     seconds  usecs/call     calls    errors syscall
------ ----------- ----------- --------- --------- ----------------
 45.23    0.018234          12      1523           sendto
 32.15    0.012956           8      1523           recvfrom
  8.42    0.003391          22       150           poll
  5.18    0.002087          15       138           socket
 ...
------ ----------- ----------- --------- --------- ----------------
100.00    0.040312                  3856       124 total
```

**Optimization Opportunities:**
- **High sendto/recvfrom counts**: Use `sendmmsg`/`recvmmsg` batching
- **Frequent poll calls**: Increase timeout or batch size
- **Many socket creations**: Reuse sockets with connection pooling

## Performance Testing

### Throughput Test Suite

Automated testing for scan performance validation.

**Test Script:**

```bash
#!/bin/bash
# scripts/perf_test.sh

echo "=== ProRT-IP Performance Test Suite ==="

# Test 1: Single port, many hosts
echo "Test 1: Scanning 10.0.0.0/16 port 80..."
time ./target/release/prtip -sS -p 80 --max-rate 100000 10.0.0.0/16

# Test 2: Many ports, single host
echo "Test 2: Scanning 127.0.0.1 all ports..."
time ./target/release/prtip -sS -p- 127.0.0.1

# Test 3: Stateless vs Stateful comparison
echo "Test 3: Stateless scan..."
time ./target/release/prtip --stateless -p 80 10.0.0.0/24

echo "Test 3: Stateful scan (same targets)..."
time ./target/release/prtip -sS -p 80 10.0.0.0/24

# Test 4: Memory usage monitoring
echo "Test 4: Memory usage (large scan)..."
/usr/bin/time -v ./target/release/prtip -sS -p 80,443 10.0.0.0/16 \
    | grep "Maximum resident set size"
```

### Load Testing

Sustained throughput validation for production scenarios.

**Rust Load Test:**

```rust
// tests/load_test.rs

#[test]
fn load_test_sustained_throughput() {
    let target_pps = 100_000;
    let duration = Duration::from_secs(60); // 1 minute sustained

    let scanner = Scanner::new(ScanConfig {
        max_rate: target_pps,
        ..Default::default()
    }).unwrap();

    let start = Instant::now();
    let mut packets_sent = 0;

    while start.elapsed() < duration {
        packets_sent += scanner.send_batch().unwrap();
    }

    let actual_pps = packets_sent / duration.as_secs() as usize;

    // Allow 5% variance
    assert!(actual_pps >= target_pps * 95 / 100);
    assert!(actual_pps <= target_pps * 105 / 100);
}
```

### Regression Detection

Automated CI/CD performance monitoring to catch regressions.

**GitHub Actions Workflow:**

```yaml
# .github/workflows/performance.yml

name: Performance Regression Check

on: [pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Need history for comparison

      - name: Run benchmarks (baseline)
        run: |
          git checkout main
          cargo bench --bench packet_crafting -- --save-baseline main

      - name: Run benchmarks (PR)
        run: |
          git checkout ${{ github.head_ref }}
          cargo bench --bench packet_crafting -- --baseline main

      - name: Check for regression
        run: |
          # Fail if any benchmark regresses >5%
          cargo bench --bench packet_crafting -- --baseline main \
            | grep "change:.*-.*%" && exit 1 || exit 0
```

## Optimization Techniques

### Lock-Free Data Structures

**Problem**: Mutex contention limits scalability beyond 4-8 cores

**Solution**: Use `crossbeam` lock-free queues for task distribution

**Implementation (v0.3.0+):**

```rust
use crossbeam::queue::SegQueue;

// Replace Mutex<VecDeque<Task>>
// With:
let task_queue: Arc<SegQueue<Task>> = Arc::new(SegQueue::new());

// Workers can push/pop without locks
task_queue.push(task);
if let Some(task) = task_queue.pop() {
    // process task
}
```

**Impact**: 3-5x throughput improvement on 16+ core systems

**ProRT-IP Implementation:**
- **SYN Scanner**: `DashMap` for connection table (eliminated lock contention)
- **Rate Limiter**: Atomic operations for state management (lock-free fast path)

**Performance Validation:**

```bash
# Measure lock contention before optimization
perf record -e lock:contention_begin ./target/release/prtip [args]

# Compare before/after
hyperfine --warmup 3 \
    './prtip-v0.2.9 -sS -p 1-10000 127.0.0.1' \
    './prtip-v0.3.0 -sS -p 1-10000 127.0.0.1'
```

### SIMD Optimization

**Problem**: Checksum calculation is CPU-intensive at high packet rates

**Solution**: Use SIMD instructions for parallel addition (leveraged by `pnet` crate)

**ProRT-IP Approach**: The `pnet` library handles SIMD checksum optimizations automatically, providing 2-3x faster checksums on supported platforms.

**Verification:**

```bash
# Check SIMD usage in perf
perf stat -e fp_arith_inst_retired.128b_packed_double:u \
    ./target/release/prtip -sS -p 1-1000 127.0.0.1
```

### Memory Pooling

**Problem**: Allocating buffers per-packet causes allocator contention

**Solution**: Pre-allocate buffer pool, reuse buffers

**Zero-Copy Implementation (v0.3.8+):**

```rust
use prtip_network::packet_buffer::with_buffer;

with_buffer(|pool| {
    let packet = TcpPacketBuilder::new()
        .source_ip(Ipv4Addr::new(10, 0, 0, 1))
        .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
        .source_port(12345)
        .dest_port(80)
        .flags(TcpFlags::SYN)
        .build_ip_packet_with_buffer(pool)?;

    send_packet(packet)?;
    pool.reset();  // Reuse buffer
    Ok(())
})?;
```

**Performance Impact:**

| Metric | Old API | Zero-Copy | Improvement |
|--------|---------|-----------|-------------|
| Per-packet time | 68.3 ns | 58.8 ns | 15% faster |
| Allocations | 3-7 per packet | 0 per packet | 100% reduction |
| Throughput | 14.6M pps | 17.0M pps | +2.4M pps |

### Batched System Calls

**Problem**: System call overhead dominates at high packet rates

**Solution**: Use `sendmmsg`/`recvmmsg` to batch operations (Linux)

**Impact**: 5-10x reduction in syscall overhead

**Configuration:**

```bash
# Adjust batch size (default: 64)
prtip --batch-size 128 [args]

# Optimal values:
# 16:  Low latency, ~95% syscall reduction
# 64:  Balanced (default), ~98% syscall reduction
# 128: Maximum throughput, ~99% syscall reduction
```

### NUMA Optimization

**Problem**: Cross-NUMA memory access penalties (10-30% slowdown)

**Solution**: Pin threads to NUMA nodes matching network interfaces

**ProRT-IP Implementation (v0.3.8+):**
- Automatic NUMA topology detection using `hwloc`
- TX thread pinned to core on NUMA node 0 (near NIC)
- Worker threads distributed round-robin across nodes

**Performance Impact:**
- **Dual-socket**: 20-30% improvement
- **Quad-socket**: 30-40% improvement
- **Single-socket**: <5% (within noise, not recommended)

**See Also**: [Performance Tuning](../user-guide/timing-performance.md#numa-optimization) for usage details.

## Platform-Specific Analysis

### Linux Optimizations

#### AF_PACKET with PACKET_MMAP

Zero-copy packet capture using memory-mapped ring buffers provides 30-50% reduction in CPU usage.

**Benefits:**
- Eliminates packet copy from kernel to userspace
- Reduces context switches
- Improves cache efficiency

#### eBPF/XDP for Ultimate Performance

For 10M+ pps, leverage XDP (eXpress Data Path) with kernel-level filtering.

**Impact**: 24M+ pps per core with hardware offload

### Windows Optimizations

#### Npcap Performance Tuning

Use `SendPacketEx` instead of `SendPacket` for 20-30% improvement.

**Configuration:**
- Increase buffer sizes
- Enable loopback capture if scanning localhost
- Use latest Npcap version (1.79+)

### macOS Optimizations

#### BPF Buffer Sizing

```bash
# Increase BPF buffer size for better batching
sysctl -w kern.ipc.maxsockbuf=8388608
```

**Impact**: Reduces packet loss at high rates

## Troubleshooting Performance Issues

### Low Throughput (<1K pps)

**Symptoms**: Scan much slower than expected

**Diagnostic Steps:**

```bash
# Check privileges
getcap ./target/release/prtip

# Check NIC speed
ethtool eth0 | grep Speed

# Profile to find bottleneck
perf top
```

**Common Causes:**
1. Running without root/capabilities (falling back to connect scan)
2. Network interface limit (check with `ethtool`)
3. CPU bottleneck (check with `htop`)
4. Rate limiting enabled (check `--max-rate`)

### High CPU Usage (>80% on all cores)

**Symptoms**: All cores saturated but low throughput

**Diagnostic Steps:**

```bash
# Profile CPU usage
perf record -g ./target/release/prtip [args]
perf report

# Look for:
# - High time in __pthread_mutex_lock
# - High time in malloc/free
# - Hot loops in packet parsing
```

**Common Causes:**
1. Inefficient packet parsing
2. Lock contention
3. Allocation overhead

### Memory Growth Over Time

**Symptoms**: Memory usage increases continuously during scan

**Diagnostic Steps:**

```bash
# Check for leaks
valgrind --leak-check=full ./target/debug/prtip [args]

# Monitor memory over time
watch -n 1 'ps aux | grep prtip'
```

**Common Causes:**
1. Connection state not being cleaned up
2. Result buffer not flushing
3. Memory leak

### High Packet Loss

**Symptoms**: Many ports reported as filtered/unknown

**Diagnostic Steps:**

```bash
# Check NIC statistics
ethtool -S eth0

# Monitor dropped packets
netstat -s | grep dropped

# Reduce rate
prtip --max-rate 5000 [args]
```

**Common Causes:**
1. Rate too high for network capacity
2. NIC buffer overflow
3. Target rate limiting/firewall

## Best Practices

### Before Optimization

1. **Establish baseline**: Measure current performance with hyperfine
2. **Profile first**: Identify bottlenecks with `perf` or `valgrind`
3. **Focus on hot paths**: Optimize code that runs frequently (80/20 rule)
4. **Validate assumptions**: Use benchmarks to confirm bottleneck location

### During Optimization

1. **One change at a time**: Isolate variables for clear causation
2. **Use version control**: Commit before/after each optimization
3. **Benchmark repeatedly**: Run multiple iterations for statistical validity
4. **Document changes**: Record optimization rationale and expected impact

### After Optimization

1. **Verify improvement**: Compare against baseline with hyperfine
2. **Check regression**: Run full test suite (cargo test)
3. **Monitor production**: Use profiling in production environment
4. **Update documentation**: Record optimization in CHANGELOG and guides

## See Also

- [Performance Characteristics](performance-characteristics.md) - Detailed performance metrics and scaling analysis
- [Benchmarking](benchmarking.md) - Comprehensive benchmarking guide with scenario library
- [Performance Tuning](../user-guide/timing-performance.md) - User-facing optimization guide
- [Architecture](../reference/architecture.md) - System design for performance
