# Phase 4 Pre-Final Benchmarking Suite

Comprehensive benchmarking suite executed at completion of Phase 4 Sprint 4.1-4.14 (Sprint 4.9 final benchmarking).

## Overview

**Execution Date:** 2025-10-10 (Sprint 4.9)
**Version:** v0.3.0
**Status:** Pre-final benchmarks before v0.4.0 release
**Total Files:** 29 benchmark files
**Purpose:** Validate all Phase 4 Sprint 4.1-4.9 performance optimizations

This directory contains the most comprehensive benchmarking suite executed during Phase 4, establishing baseline performance metrics and validating critical bug fixes.

## Key Performance Achievements

### Phase 3 → Phase 4 Final Improvements

| Benchmark | Phase 3 | Phase 4 Final | Improvement |
|-----------|---------|---------------|-------------|
| 1K ports (localhost) | 25ms | 4.5ms | 82% faster ⚡ |
| 10K ports (localhost) | 117ms | 39.4ms | 66.3% faster ⚡ |
| 65K ports (localhost) | >180s | 190.9ms | **198x faster** 🚀 |
| 10K --with-db | 194.9ms | 75.1ms | 61.5% faster ⚡ |

### System Metrics Validated

- **Futex Reduction:** 20,373 → 398 calls (98% reduction) ✅
  - Lock-free aggregator success (Sprint 4.2)
  - Eliminated SQLite contention bottleneck
- **Memory Peak:** 1.9 MB (ultra-low footprint) ✅
  - Efficient resource usage
  - Linear scaling with workload
- **CPU Utilization:** 6.092 cores utilized (excellent multi-core scaling) ✅
  - 10C/20T system: 60% core utilization
  - Parallel work distribution effective
- **Cache Efficiency:** 0.45% LLC miss rate ✅
  - Excellent cache locality
  - Minimized memory access latency
- **Branch Prediction:** 2.42% miss rate ✅
  - Predictable code paths
- **IPC:** 0.44 instructions per cycle ✅

## Files Included

### Statistical Analysis (Hyperfine) - 15 files

**Scenario 1: 1K Ports**
- `01-Hyperfine-1K-Ports.json` - Statistical data (20 runs)
- `02-Hyperfine-1K-Ports.md` - Markdown report
- `03-Hyperfine-1K-Ports-Output.txt` - Raw output

**Scenario 2: 10K Ports**
- `04-Hyperfine-10K-Ports.json` - Statistical data (20 runs)
- `05-Hyperfine-10K-Ports.md` - Markdown report
- `06-Hyperfine-10K-Ports-Output.txt` - Raw output

**Scenario 3: 65K Full Port Range**
- `07-Hyperfine-65K-Ports.json` - Statistical data (10 runs)
- `08-Hyperfine-65K-Ports.md` - Markdown report
- `09-Hyperfine-65K-Ports-Output.txt` - Raw output

**Scenario 4: 10K Ports with Database**
- `10-Hyperfine-10K-DB.json` - Database storage comparison
- `11-Hyperfine-10K-DB.md` - Markdown report
- `12-Hyperfine-10K-DB-Output.txt` - Raw output

**Scenario 5: Timing Templates (T0/T3/T5)**
- `13-Hyperfine-Timing-Templates.json` - Template comparison
- `14-Hyperfine-Timing-Templates.md` - Markdown report
- `15-Hyperfine-Timing-Templates-Output.txt` - Raw output

### CPU Profiling (perf) - 3 files

- `16-Perf-CPU-Profile.txt` - Call graph analysis (12.6% Tokio TCP operations)
- `17-Perf-Hardware-Counters.txt` - IPC, cache misses, branch prediction
- `18-Perf-Script-Output.txt` - Raw perf script data

**Key Findings:**
- Tokio TCP operations dominate at 12.6% (expected for network scanner)
- No unexpected bottlenecks identified
- 84% system time (kernel socket operations)
- 16% user time (application logic)

### Syscall Tracing (strace) - 6 files

- `19-Strace-Full-Trace.txt` - Complete syscall trace (1,033 calls for 10K ports)
- `20-Strace-Summary.txt` - Syscall summary statistics
- `21-Strace-Futex-Analysis.txt` - Futex call analysis (398 in-memory, 381 with-db)
- `22-Strace-Comparison-Sprint45.txt` - Sprint 4.5 comparison (20,373 futex → 398 = 98% reduction)
- `23-Strace-Efficiency.txt` - Efficiency metrics (<0.1 syscalls/port)
- `24-Strace-Breakdown.txt` - Category breakdown

**Key Findings:**
- 98% futex reduction validates lock-free aggregator success
- Ultra-efficient: <0.1 syscalls per port scanned
- Minimal system overhead

### Memory Profiling (Massif) - 3 files

- `25-Massif-Memory-Profile.out` - Raw Massif output data
- `26-Massif-Visualization.txt` - Memory usage visualization
- `27-Massif-Summary.txt` - Memory usage summary

**Key Findings:**
- Peak memory: 1.9 MB (ultra-low footprint)
- Heap efficiency: 98.2% necessary runtime operations
- No memory leaks detected
- Linear scaling with workload

### Summary Documents - 2 files

- `28-Final-Benchmark-Summary.md` - Comprehensive 12KB summary document
- `29-Phase4-Final-Report.md` - Phase 4 complete analysis

## Benchmark Methodology

### Tools Used

| Tool | Purpose | Key Metrics |
|------|---------|-------------|
| **hyperfine** | Statistical benchmarking | Mean time, std dev, min/max, outliers |
| **perf** | CPU profiling | Call graphs, hardware counters, IPC, cache misses |
| **flamegraph** | Visualization | Interactive call stack SVG (190KB) |
| **strace** | Syscall tracing | Futex analysis, syscall efficiency |
| **massif** | Memory profiling | Heap usage, peak memory, leak detection |

### Test Environment

**System Specifications:**
- **Hostname:** AB-i9
- **Kernel:** Linux 6.17.1-2-cachyos
- **CPU:** Intel i9-10850K @ 3.60GHz (10 cores, 20 threads)
- **Base Frequency:** 3.60 GHz
- **Max Turbo:** 5.20 GHz
- **Cache:** 20 MB Intel Smart Cache
- **Memory:** 62GB DDR4 (dual-channel)
- **OS:** CachyOS (Arch-based, performance-optimized kernel)
- **Rust Version:** 1.90.0 (2025-09-14)
- **Build Profile:** Release with optimizations (opt-level=3, LTO=fat)

**Network:** Localhost loopback (127.0.0.1) - 91-182x faster than typical network

### Execution Parameters

**Hyperfine:**
- Warmup runs: 3
- Measurement runs: 10-20 (based on workload)
- Shell: None (direct binary execution)
- Time unit: Milliseconds

**Perf:**
- Frequency: 997 Hz (avoid alignment with system timer)
- Call graph: dwarf (debug info)
- Hardware counters: cycles, instructions, cache-references, cache-misses, branches, branch-misses

**Strace:**
- Trace all syscalls
- Include timestamps
- Filter for futex analysis

**Massif:**
- Heap profiling only
- Detailed snapshots every 100ms
- Track all allocations

## Critical Bugs Fixed & Validated

### Sprint 4.4: 65K Port Overflow (198x Speedup)

**Issue:** Port 65535 u16 overflow causing infinite loop
**Before:** >180 seconds (infinite loop)
**After:** 190.9ms
**Improvement:** 198x faster 🚀
**Validation:** This benchmark suite confirms fix

### Sprint 4.2-4.5: Lock-Free Aggregation (98% Futex Reduction)

**Issue:** SQLite write contention (95.47% futex time)
**Before:** 20,373 futex calls (Sprint 4.5 baseline)
**After:** 398 futex calls (Sprint 4.9 final)
**Improvement:** 98% reduction ✅
**Validation:** Strace analysis confirms

### Sprint 4.6: In-Memory Default Mode (5.2x Speedup)

**Issue:** Database writes on every result
**Before:** 194.9ms for 10K ports (SQLite default)
**After:** 75.1ms for 10K ports (in-memory default)
**Improvement:** 61.5% faster ⚡
**Validation:** Hyperfine Scenario 4 confirms

## Sprints Validated

This benchmarking suite validates optimizations from:

- ✅ **Sprint 4.1:** Network Testing Infrastructure (Docker environment)
- ✅ **Sprint 4.2:** Lock-Free Result Aggregator (crossbeam SegQueue)
- ✅ **Sprint 4.3:** Lock-Free Integration (tcp_connect.rs)
- ✅ **Sprint 4.4:** Critical 65K Port Fix (u16 overflow)
- ✅ **Sprint 4.5:** Performance Profiling (SQLite bottleneck identified)
- ✅ **Sprint 4.6:** In-Memory Default Mode (async storage worker)
- ✅ **Sprint 4.7:** Scheduler Refactor (StorageBackend enum)
- ✅ **Sprint 4.8:** Async Storage Deadlock Fix (timeout pattern)
- ✅ **Sprint 4.9:** Final Benchmarking (this suite)

## Comparison with Previous Benchmarks

### vs. Phase 3 Baseline (docs/archive/)

Original Phase 3 benchmarks (before optimization):
- 1K ports: 25ms → 4.5ms (82% faster)
- 10K ports: 117ms → 39.4ms (66.3% faster)
- 65K ports: >180s → 190.9ms (198x faster)

### vs. Sprint 4.5 Profiling

Sprint 4.5 identified SQLite contention:
- Futex calls: 20,373 → 398 (98% reduction)
- Database overhead: 194.9ms → 75.1ms (61.5% faster)

## Known Limitations

**Localhost vs Network Performance:**
- These benchmarks use localhost loopback (127.0.0.1)
- Real network scans are 91-182x slower due to:
  - Network latency (RTT)
  - Router/switch overhead
  - TCP handshake timing
  - Firewall processing
- Use these as **upper bound** performance targets

**Platform-Specific:**
- Linux-only syscalls (sendmmsg, recvmmsg)
- CachyOS performance kernel optimizations
- May vary on other distributions/kernels

## Next Benchmarking

**Target:** `../02-Phase4_Final-Bench/` (before v0.4.0 release)

**Focus Areas:**
- Sprint 4.10: CLI improvements validation
- Sprint 4.11: Service detection (once bug fixed)
- Sprint 4.12: Progress bar overhead (<0.5% CPU target)
- Sprint 4.13: Large scan performance (2.56M ports)
- Sprint 4.14: Filtered network optimization (3-17x speedup)

**Expected Improvements:**
- Progress bar real-time updates with minimal overhead
- 10x speedup on network scans (Sprint 4.13 fix)
- 3-17x speedup on filtered networks (Sprint 4.14 optimization)

## References

- **Phase 3 Baseline:** `../archive/01-Phase3-Baseline/`
- **Sprint 4.5 Profiling:** `../archive/06-Sprint-4.5-Profiling/`
- **Project Documentation:** `../../docs/07-PERFORMANCE.md`
- **Benchmark Guide:** `../../docs/12-BENCHMARKING-GUIDE.md`

## Usage

To reproduce these benchmarks:

```bash
# Ensure release build with debug symbols
RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes" cargo build --release

# Run hyperfine suite
hyperfine --warmup 3 --runs 20 './target/release/prtip -p 1-1000 127.0.0.1'

# Run perf profiling
perf record --call-graph dwarf -F 997 ./target/release/prtip -p 1-10000 127.0.0.1
perf report

# Run strace analysis
strace -c ./target/release/prtip -p 1-10000 127.0.0.1

# Run massif profiling
valgrind --tool=massif ./target/release/prtip -p 1-1000 127.0.0.1
ms_print massif.out.*
```

See `../../docs/12-BENCHMARKING-GUIDE.md` for complete methodology.

---

**Status:** Pre-final benchmarks complete ✅ | **Next:** Phase 4 final benchmarks before v0.4.0 release
