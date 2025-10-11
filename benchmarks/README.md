# ProRT-IP Benchmarks

Comprehensive performance benchmarking results organized by Phase 4 development timeline and sprint.

## Directory Structure

### Phase 4 Benchmark Suites

- **[01-Phase4_PreFinal-Bench](01-Phase4_PreFinal-Bench/)** - Sprint 4.9 comprehensive suite (29 files + README)
  - **Status:** ✅ Complete
  - **Date:** 2025-10-10
  - **Scope:** Sprints 4.1-4.9 validation
  - **Tools:** hyperfine, perf, flamegraph, strace, massif
  - **Highlights:** 198x speedup on 65K ports, 98% futex reduction, 1.9 MB memory peak

- **[02-Phase4_Final-Bench](02-Phase4_Final-Bench/)** - Pending v0.4.0 benchmarks (README only)
  - **Status:** 🔜 Pending execution
  - **Target:** Before v0.4.0 release
  - **Scope:** Sprints 4.10-4.14 validation
  - **Focus:** Progress bar overhead, large network scans, filtered network optimization

- **[archive](archive/)** - Historical sprint-specific benchmarks (15+ directories)
  - **Status:** ✅ Historical reference
  - **Organization:** Chronological by sprint (01-Phase3-Baseline through 14-Sprint-4.14-Hang-Fix)
  - **Purpose:** Regression analysis, historical comparison

- **[flamegraphs](flamegraphs/)** - CPU profiling visualizations (SVG files)
  - **Status:** ✅ Interactive visualizations
  - **Purpose:** Hot path analysis, performance bottleneck identification

## Key Performance Achievements

### Phase 3 → Phase 4 Final Improvements

| Benchmark | Phase 3 Baseline | Phase 4 Final | Improvement |
|-----------|------------------|---------------|-------------|
| **1K ports (localhost)** | 25ms | 4.5ms | **82% faster** ⚡ |
| **10K ports (localhost)** | 117ms | 39.4ms | **66.3% faster** ⚡ |
| **65K ports (localhost)** | >180s (infinite loop) | 190.9ms | **198x faster** 🚀 |
| **10K --with-db** | 194.9ms | 75.1ms | **61.5% faster** ⚡ |
| **2.56M ports (network)** | 2 hours | 15 min | **10x faster** 🚀 |
| **10K filtered ports** | 57 min | 3.2s | **17.5x faster** 🚀 |

### System Metrics (from 01-Phase4_PreFinal-Bench/)

**Lock-Free Success:**
- Futex calls: 20,373 → 398 (**98% reduction**) ✅
- Validates Sprint 4.2-4.5 lock-free aggregator

**Memory Efficiency:**
- Peak memory: **1.9 MB** (ultra-low footprint) ✅
- No memory leaks detected
- Linear scaling with workload

**Multi-Core Scaling:**
- CPU utilization: **6.092 cores** (60% of 10C/20T system) ✅
- Excellent parallel work distribution

**Cache Efficiency:**
- LLC miss rate: **0.45%** (excellent locality) ✅
- Branch miss rate: **2.42%** (predictable code paths) ✅
- IPC: **0.44** instructions per cycle ✅

## Critical Sprint Benchmarks

### Sprint 4.9 - Final Benchmarking (Most Comprehensive)

**Location:** `01-Phase4_PreFinal-Bench/`
**Files:** 29 benchmark files + comprehensive README
**Status:** ✅ Complete

**Tools Used:**
- hyperfine: Statistical analysis (5 scenarios, 15 files)
- perf: CPU profiling (3 files)
- strace: Syscall tracing (6 files)
- massif: Memory profiling (3 files)
- Summary documents (2 files)

**Key Validations:**
- ✅ 198x speedup on 65K ports (Sprint 4.4 fix)
- ✅ 98% futex reduction (Sprint 4.2-4.5 lock-free)
- ✅ 61.5% improvement with --with-db (Sprint 4.6)
- ✅ Ultra-low memory footprint (1.9 MB)

### Sprint 4.13 - Performance Regression Fix

**Location:** `archive/11-Sprint-4.10-CLI-Improvements/` (related) + bug_fix/03-Performance-Regression/

**Issue:** Variable shadowing bug causing 30% CPU polling overhead
**Before:** 289 pps, 2-hour ETA on 2.56M ports
**After:** 2,844 pps, 15-minute completion
**Improvement:** **10x faster** 🚀

**Validation:**
- ✅ Polling overhead: 30% → 3% (80x reduction)
- ✅ Scan rate: 289 → 2,844 pps
- ✅ Duration: 2 hours → 15 minutes

### Sprint 4.14 - Network Timeout Optimization

**Location:** `archive/12-Sprint-4.14-Timeout-Optimization/` + bug_fix/04-Network-Timeout/

**Changes:** Reduced timeout (3s → 1s), increased parallelism (500 → 1000)
**Before:** 178 pps, 4-hour ETA on filtered network
**After:** 536-1000 pps, 42-85 minute completion
**Improvement:** **3-17x faster** on filtered networks 🚀

**Benchmark Results:**
- ✅ 10K ports on 192.168.4.1: 3.19s (**3,132 pps, 17.5x faster**)
- ✅ Expected network improvement: 3-5.6x
- ✅ New --host-delay flag for rate-limited networks

## Benchmark Tools

### Primary Tools

| Tool | Purpose | Key Metrics | Output Files |
|------|---------|-------------|--------------|
| **hyperfine** | Statistical benchmarking | Mean, std dev, min/max, outliers | JSON, Markdown, TXT |
| **perf** | CPU profiling | Call graphs, hardware counters | TXT (report, script) |
| **flamegraph** | Visualization | Interactive call stack | SVG (190KB) |
| **strace** | Syscall tracing | Futex analysis, efficiency | TXT (full, summary) |
| **massif** | Memory profiling | Heap usage, peak memory | TXT, PostScript |

### Additional Validation Tools

| Tool | Purpose | Use Case |
|------|---------|----------|
| **nmap** | Industry standard | Accuracy validation (100% target) |
| **rustscan** | Speed comparison | Performance benchmarking |
| **naabu** | Alternative scanner | Performance benchmarking |
| **netcat/telnet** | Connectivity | Network validation |

## Reading Benchmark Results

### Localhost vs Network Performance

**Important:** Most benchmarks use localhost loopback (127.0.0.1)

**Performance Difference:**
- Localhost: 91-182x faster than typical network scans
- Reason: Zero network latency, no router/switch overhead, instant TCP handshake

**Interpretation:**
- Use localhost benchmarks as **upper bound** performance targets
- Real network scans will be significantly slower due to:
  - Network RTT (round-trip time)
  - Router/switch processing
  - Firewall rule evaluation
  - TCP handshake timing
  - Packet loss and retransmission

**Example:**
- 10K ports localhost: 39.4ms (254K pps)
- 10K ports network: ~5-10 seconds (1K-2K pps) ← realistic expectation

### System Specifications (Reference Benchmarks)

All benchmarks in 01-Phase4_PreFinal-Bench/ executed on:

**Hardware:**
- **Hostname:** AB-i9
- **CPU:** Intel i9-10850K @ 3.60GHz (10 cores, 20 threads)
- **Base/Turbo:** 3.60 GHz / 5.20 GHz
- **Cache:** 20 MB Intel Smart Cache
- **Memory:** 62GB DDR4 (dual-channel)

**Software:**
- **Kernel:** Linux 6.17.1-2-cachyos
- **OS:** CachyOS (Arch-based, performance-optimized kernel)
- **Rust:** 1.90.0 (2025-09-14)
- **Build:** Release with opt-level=3, LTO=fat

**Note:** Your results may vary based on CPU, memory, kernel, and OS optimizations.

## Historical Benchmarks (archive/)

### Organization

Benchmarks organized chronologically by sprint:

1. **01-Phase3-Baseline** - Pre-Phase 4 baseline (v0.3.0)
2. **02-Sprint-4.1-Network-Infra** - Docker test environment setup
3. **03-Sprint-4.2-Lockfree** - Lock-free aggregator implementation
4. **04-Sprint-4.3-Integration** - Lock-free integration validation
5. **05-Sprint-4.4-65K-Fix** - Critical port overflow fix (198x speedup)
6. **06-Sprint-4.5-Profiling** - SQLite contention identified
7. **07-Sprint-4.6-Inmemory-Default** - In-memory mode (5.2x speedup)
8. **08-Sprint-4.7-Scheduler-Refactor** - Architecture cleanup
9. **09-Sprint-4.8-Async-Fix** - Async storage deadlock fix
10. **10-Sprint-4.9-Finalization** - Finalization benchmarks
11. **11-Sprint-4.10-CLI-Improvements** - CLI enhancements
12. **11.5-Sprint-4.11-Validation** - Industry tool validation
13. **12-Sprint-4.14-Timeout-Optimization** - Network timeout optimization
14. **13-Sprint-4.8-Deep-Timing** - Deep timing investigation (no bug found)
15. **14-Sprint-4.14-Hang-Fix** - Network timeout documentation

**Purpose:** Historical reference, regression analysis, performance tracking over time

## Reproducing Benchmarks

### Prerequisites

```bash
# Ensure release build with debug symbols (for perf/flamegraph)
RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes" cargo build --release

# Set CPU governor to performance mode
sudo cpupower frequency-set -g performance

# Disable swap for consistent timing
sudo swapoff -a
```

### hyperfine Statistical Analysis

```bash
# Basic benchmark (20 runs with 3 warmup)
hyperfine --warmup 3 --runs 20 \
  './target/release/prtip -p 1-10000 127.0.0.1'

# Export results (JSON + Markdown)
hyperfine --warmup 3 --runs 20 \
  --export-json results.json \
  --export-markdown results.md \
  './target/release/prtip -p 1-10000 127.0.0.1'

# Compare scenarios
hyperfine --warmup 3 --runs 20 \
  './target/release/prtip -p 1-10000 127.0.0.1' \
  './target/release/prtip -p 1-10000 --with-db 127.0.0.1'
```

### perf CPU Profiling

```bash
# Record with call graphs
perf record --call-graph dwarf -F 997 \
  ./target/release/prtip -p 1-10000 127.0.0.1

# Generate report
perf report

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

### strace Syscall Tracing

```bash
# Summary statistics
strace -c ./target/release/prtip -p 1-10000 127.0.0.1

# Full trace (large output)
strace -o trace.txt ./target/release/prtip -p 1-10000 127.0.0.1

# Futex analysis
strace -e trace=futex ./target/release/prtip -p 1-10000 127.0.0.1
```

### massif Memory Profiling

```bash
# Run massif profiler
valgrind --tool=massif \
  ./target/release/prtip -p 1-1000 127.0.0.1

# Generate report
ms_print massif.out.* > memory-report.txt
```

### Cleanup

```bash
# Restore CPU governor
sudo cpupower frequency-set -g powersave

# Re-enable swap
sudo swapon -a
```

## Next Benchmarking

**Target:** `02-Phase4_Final-Bench/` (before v0.4.0 release)

**Scope:** Validate Sprints 4.10-4.14 improvements

**Focus Areas:**
- Sprint 4.10: CLI improvements (no performance regression)
- Sprint 4.11: DNS resolution performance, service detection (once bug fixed)
- Sprint 4.12: Progress bar overhead (<0.5% CPU target)
- Sprint 4.13: Large network scan performance (10x speedup validation)
- Sprint 4.14: Filtered network optimization (3-17x speedup validation)

**Execution Plan:** See `02-Phase4_Final-Bench/README.md` for complete plan

## References

- **Performance Guide:** `../docs/07-PERFORMANCE.md`
- **Benchmark Guide:** `../docs/12-BENCHMARKING-GUIDE.md`
- **Bug Fix Reports:** `../bug_fix/` (performance regression fixes)
- **Sprint Documentation:** Each archive/ subdirectory has sprint-specific details

## Contributing

When adding new benchmarks:

1. **Create directory:** Follow naming convention `{NN}-Phase{X}_{Description}/` or `{NN}-Sprint-{X.Y}-{Description}/`
2. **Run comprehensive suite:** hyperfine, perf, strace, massif minimum
3. **Generate README:** Document methodology, results, key findings
4. **Update this file:** Add to directory structure and key results
5. **Preserve history:** Move to archive/ when superseded

---

**Last Updated:** 2025-10-11
**Status:** Pre-final benchmarks complete, final benchmarks pending v0.4.0
**Total Files:** 307 across all directories (benchmarks/, bug_fix/, docs/)
