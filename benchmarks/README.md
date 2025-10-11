# ProRT-IP Benchmarks

Comprehensive performance benchmarking and profiling results across all development phases.

## Directory Structure

### Root Level (Final Benchmarks)

Comprehensive final benchmarking suite from Phase 4 completion:

| File | Description |
|------|-------------|
| **12-FINAL-BENCHMARK-SUMMARY.md** | Complete Phase 4 performance analysis (12KB document) |
| **01-06-hyperfine-*.{json,md,txt}** | Statistical benchmarks (1K, 10K, 65K ports, timing templates) |
| **06-08-perf-*.txt / 08-flamegraph-*.svg** | CPU profiling (call graphs, flamegraph visualization) |
| **09-10-strace-*.txt** | Syscall tracing (futex analysis, lock contention) |
| **11-massif-*.{out,txt}** | Memory profiling (heap analysis, peak usage) |

**Key Results:**

- 10K ports: **39.4ms ± 3.1ms** (66.3% faster than Phase 3 baseline)
- 65K ports: **190.9ms ± 7.1ms** (198x faster, infinite loop fixed)
- Futex calls: **398** (98% reduction from Sprint 4.5 SQLite contention)
- Memory peak: **1.9 MB** (Valgrind massif, ultra-low footprint)

### Archive (Historical Sprint Benchmarks)

All individual sprint benchmark results are archived for historical reference:

| Directory | Description | Key Metrics |
|-----------|-------------|-------------|
| **archive/01-phase3-baseline/** | Phase 3 completion baseline (v0.3.0, 551 tests) | 10K: 117ms (old default SQLite) |
| **archive/02-sprint4.1-network-infra/** | Network testing infrastructure + Metasploitable2 | Latency simulation scripts |
| **archive/03-sprint4.2-lockfree/** | Lock-free result aggregator implementation | 10M+ results/sec, <100ns latency |
| **archive/04-sprint4.3-integration/** | Lock-free + recvmmsg integration | TCP connect integration |
| **archive/05-sprint4.4-65k-fix/** | Critical 65K port bottleneck fix | >180s → 0.91s (198x faster!) |
| **archive/06-sprint4.5-profiling/** | Performance regression investigation | Root cause: SQLite contention |
| **archive/07-sprint4.6-inmemory-default/** | Default in-memory mode switch | 37.4ms (5.2x faster!) |
| **archive/08-sprint4.7-scheduler-refactor/** | Scheduler uses StorageBackend | Architecture cleanup |
| **archive/09-sprint4.8-async-fix/** | Async storage deadlock fix | 139.9ms → 74.5ms (46.7% faster) |
| **archive/10-sprint4.9-finalization/** | Sprint 4.9 finalization reports | Phase 4 wrap-up |
| **archive/11-sprint4.10-cli-improvements/** | Sprint 4.10 CLI improvements | Enhanced user experience |
| **archive/12-sprint4.14-timeout-optimization/** | Sprint 4.14 network timeout optimization | 3-17x faster filtered port detection |
| **archive/13-sprint4.8-deep-timing/** | Sprint 4.8 deep timing investigation | Polling overhead analysis |

### Flamegraphs

| Directory | Description |
|-----------|-------------|
| **flamegraphs/** | CPU profiling flamegraph visualizations (SVG files for hot path analysis) |

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

## Archive Contents

Historical sprint benchmark results are preserved in the `archive/` directory for reference. See individual sprint directories for detailed results and analysis.

### Key Historical Achievements (from Archive)

- **Sprint 4.4** (archive/05-sprint4.4-65k-fix/): 65K port scan fix (>180s → 0.91s, 198x faster!)
- **Sprint 4.5** (archive/06-sprint4.5-profiling/): Root cause analysis of SQLite contention (95.47% futex time)
- **Sprint 4.6** (archive/07-sprint4.6-inmemory-default/): Default in-memory mode (37.4ms, 5.2x faster!)
- **Sprint 4.8** (archive/09-sprint4.8-async-fix/): Async storage deadlock fix (139.9ms → 74.5ms, 46.7% improvement)

### Flamegraphs

Interactive CPU profiling visualizations:

- Open SVG files in browser for interactive call stack exploration
- Shows hot paths and performance bottlenecks
- Generated from perf call graph data

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
**Phase 4 Status**: COMPLETE (Sprints 4.1-4.14) ✅
**Total Tests**: 643/643 passing (100%)
