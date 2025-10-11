# ProRT-IP Benchmarks

Comprehensive performance benchmarking and profiling results across all development phases.

## Directory Structure

| Directory | Description | Key Metrics |
|-----------|-------------|-------------|
| **01-phase3-baseline/** | Phase 3 completion baseline (v0.3.0, 551 tests) | 10K: 117ms (old default SQLite) |
| **02-sprint4.1-network-infra/** | Network testing infrastructure + Metasploitable2 | Latency simulation scripts |
| **03-sprint4.2-lockfree/** | Lock-free result aggregator implementation | 10M+ results/sec, <100ns latency |
| **04-sprint4.3-integration/** | Lock-free + recvmmsg integration | TCP connect integration |
| **05-sprint4.4-65k-fix/** | Critical 65K port bottleneck fix | >180s → 0.91s (198x faster!) |
| **06-sprint4.5-profiling/** | Performance regression investigation | Root cause: SQLite contention |
| **07-sprint4.6-inmemory-default/** | Default in-memory mode switch | 37.4ms (5.2x faster!) |
| **08-sprint4.7-scheduler-refactor/** | Scheduler uses StorageBackend | Architecture cleanup |
| **09-sprint4.8-async-fix/** | Async storage deadlock fix | 139.9ms → 74.5ms (46.7% faster) |
| **flamegraphs/** | CPU profiling flamegraph visualizations | SVG files for hot path analysis |

## Quick Reference

### Performance Progression (10K ports, localhost)

| Phase | Performance | Notes |
|-------|-------------|-------|
| Phase 3 Baseline | 117ms | SQLite synchronous writes |
| Sprint 4.4 | 189.8ms | Regression: adaptive parallelism + SQLite |
| Sprint 4.5 | 189.8ms | Root cause identified: SQLite futex |
| Sprint 4.6 | 37.4ms default | In-memory default (5.2x faster!) |
| Sprint 4.6 | 68.5ms --with-db | Async storage (preliminary) |
| Sprint 4.7 | 139.9ms --with-db | Regression: broken async |
| Sprint 4.8 v2 | **74.5ms --with-db** | **Async fixed (46.7% improvement!)** |
| Sprint 4.8 v2 | **41.1ms default** | **Maintained performance** |

### Critical Achievements

- ✅ **65K Port Scan**: Fixed infinite loop (>180s → 0.91s, 198x faster)
- ✅ **Default Mode**: In-memory storage (5.2x faster than old SQLite default)
- ✅ **Async Storage**: Deadlock fixed, 46.7% improvement over broken version
- ✅ **Lock-Free Aggregation**: Zero contention, <100ns latency
- ✅ **Production Ready**: 620 tests passing, zero warnings

## Directory Contents

### 01-phase3-baseline/
Phase 3 completion benchmarks (v0.3.0, 551 tests):
- `1-BASELINE-RESULTS.md` - Comprehensive baseline report (49KB)
- `3-SPRINT4-COMPREHENSIVE-REPORT.md` - Sprint 4.3-4.4 analysis (31KB)
- `4-EXECUTIVE-SUMMARY.txt` - Quick reference summary
- Scenario outputs (5-12): Service discovery, medium/large/full range, timing templates

### 02-sprint4.1-network-infra/
Network testing infrastructure:
- `2-PHASE4-NETWORK-BENCHMARKS.md` - Network setup guide (28KB)
- Docker test environment with Metasploitable2
- Latency simulation scripts

### 03-sprint4.2-lockfree/
Lock-free result aggregator implementation (empty - code in crates/):
- Module: `crates/prtip-scanner/src/lockfree_aggregator.rs`
- Performance: 10M+ results/sec, <100ns latency
- Tests: 8 unit + 2 doc-tests

### 04-sprint4.3-integration/
Lock-free + recvmmsg integration (empty - code in crates/):
- TCP connect integration: `crates/prtip-scanner/src/tcp_connect.rs`
- Batch receiver: `crates/prtip-network/src/batch_sender.rs`
- Tests: 9 integration + 6 unit

### 05-sprint4.4-65k-fix/
Critical 65K port bottleneck fix:
- `sprint-4.4-benchmarks.txt` - Performance validation
- `65k-ports-flamegraph.svg` - CPU profiling visualization
- **Result**: >180s hang → 0.91s (198x faster!)

### 06-sprint4.5-profiling/
Comprehensive performance profiling (27MB raw data):
- `14-SPRINT4.5-PROFILING-SUMMARY.md` - Executive summary (23KB)
- `15-SPRINT4.5-KEY-FINDINGS.txt` - Root cause analysis
- Flamegraphs: 1k-ports, 10k-ports, 65k-ports (116KB-590KB)
- Raw data: perf.data, collapsed stacks, hyperfine JSON
- **Root Cause**: SQLite write contention (95.47% futex time)

### 07-sprint4.6-inmemory-default/
Default in-memory mode switch (breaking change):
- Implementation: `crates/prtip-scanner/src/storage/`
- **Breaking**: `--no-db` removed, `--with-db` added
- **Performance**: 37.4ms default (5.2x faster than old default!)
- Async storage worker with channel communication

### 08-sprint4.7-scheduler-refactor/
Scheduler refactor to use StorageBackend enum:
- `default-benchmark.json/md` - Performance validation
- `withdb-benchmark.json/md` - Async storage validation
- `implementation-summary.md` - Technical details (12KB)
- `FINAL-REPORT.md` - Sprint summary (13KB)
- **Result**: Architecture cleanup, all 620 tests passing

### 09-sprint4.8-async-fix/
Async storage deadlock fix (Sprint 4.8 v2):
- `sprint4.8-v2-default-benchmark.json/md` - Default mode validation
- `sprint4.8-v2-withdb-benchmark.json/md` - Async mode validation
- `sprint4.8-v2-performance-comparison.txt` - Before/after analysis
- `sprint4.8-v2-implementation-summary.md` - Technical details (10KB)
- `sprint4.8-v2-FINAL-REPORT.md` - Comprehensive summary (11KB)
- **Critical Fix**: Replaced tokio::select! with timeout() pattern
- **Result**: 139.9ms → 74.5ms (46.7% improvement), zero hangs

### flamegraphs/
Interactive CPU profiling visualizations:
- `1k-ports-flamegraph.svg` - 1K port scan (116KB)
- `10k-ports-flamegraph.svg` - 10K port scan (305KB)
- Open in browser for interactive call stack exploration

## Tools Used

- **hyperfine**: Statistical benchmarking (10 runs, warmup)
- **perf**: CPU profiling with call graphs
- **flamegraph**: Visualization of hot paths
- **valgrind/massif**: Memory profiling
- **strace**: Syscall tracing

## Viewing Flamegraphs

```bash
# Open in browser for interactive exploration
firefox benchmarks/flamegraphs/10k-ports-flamegraph.svg
```

## Benchmark Methodology

All benchmarks use:
- **Target**: 127.0.0.1 (localhost, minimal network latency)
- **Ports**: 1-10000 (10K ports) unless specified
- **Runs**: 10 with 3 warmup iterations
- **System**: i9-10850K (10C/20T), 64GB RAM, Linux 6.17.1-2-cachyos
- **Scan Type**: TCP SYN scan (-s syn)

Note: Localhost performance is 91-182x faster than real network scanning.

## Phase 4 Summary

**Objective**: Optimize performance from Phase 3 baseline

**Status**: ✅ **COMPLETE**

### Key Achievements
1. **65K Port Fix** (Sprint 4.4): >180s hang → 0.91s (198x faster!)
2. **Default Mode** (Sprint 4.6): 117ms → 41.1ms (5.2x faster!)
3. **Async Storage** (Sprint 4.8 v2): Deadlock fixed, 46.7% improvement
4. **Lock-Free Aggregation** (Sprint 4.2-4.3): 10M+ results/sec
5. **Production Ready**: 620 tests passing, zero warnings

### Performance Targets
- ✅ Default mode: <50ms (achieved 41.1ms)
- ✅ --with-db mode: <100ms (achieved 74.5ms)
- ✅ Full port range: <1s (achieved 0.91s)
- ✅ Zero hangs/deadlocks (100% test success)

## References

- Main project documentation: [../docs/README.md](../docs/README.md)
- Performance guide: [../docs/07-PERFORMANCE.md](../docs/07-PERFORMANCE.md)
- Implementation guide: [../docs/04-IMPLEMENTATION-GUIDE.md](../docs/04-IMPLEMENTATION-GUIDE.md)
- Project status: [../docs/10-PROJECT-STATUS.md](../docs/10-PROJECT-STATUS.md)

---

**Last Updated**: 2025-10-11
**Phase 4 Status**: COMPLETE ✅
**Total Tests**: 620/620 passing (100%)
