# Sprint 6.3 Localhost Batch I/O Performance Comparison

**Date:** $(date +%Y-%m-%d)
**Platform:** $(uname -s) $(uname -r)
**Binary:** prtip v0.5.2 (release build)
**Target:** 127.0.0.1 (localhost - eliminates network latency)

## Purpose

This benchmark eliminates network timeout bottleneck by scanning localhost,
where all responses arrive < 1ms. This isolates syscall overhead as the
primary performance factor, making batch I/O improvements measurable.

## Benchmark Results Summary

| Scenario | Ports | Batch Size | Mean Time | Improvement | Syscall Reduction | Status |
|----------|-------|------------|-----------|-------------|-------------------|--------|
| Baseline | 1-1000 | 1 | 0.00786592782s | 0% (reference) | 0% | ✓ |
| Batch 32 | 1-1000 | 32 | 0.0072324788600000006s | +8.00% | 96.87% | ❌ FAIL |
| Batch 256 | 1-1000 | 256 | 0.00691289796s | +12.00% | 99.61% | ❌ FAIL |
| Batch 1024 | 1-1000 | 1024 | 0.00695417334s | +11.00% | 99.90% | ❌ FAIL |
| Baseline (Large) | 1-10000 | 1 | 0.08448927204000001s | 0% (reference) | 0% | ✓ |
| Batch 256 (Large) | 1-10000 | 256 | 0.07687690992000001s | +9.00% | 99.61% | ❌ FAIL |

## Analysis

### Why Localhost Benchmarking?

The original benchmarks scanned random IPv4 addresses with 1000ms timeouts,
causing 99% of execution time to be network timeout waits. Batch I/O reduces
syscall overhead from ~1% to ~0.01%, which is unmeasurable when network I/O
dominates.

Localhost scanning ensures:
- All responses arrive < 1ms (vs 1000ms timeout)
- Syscall overhead becomes the bottleneck (not network I/O)
- Batch I/O improvements are measurable

### Performance Targets

- Batch 32: 20-30% improvement (96.87% syscall reduction)
- Batch 256: 35-45% improvement (99.61% syscall reduction)
- Batch 1024: 45-55% improvement (99.90% syscall reduction)

### Syscall Reduction Calculations

- 1000 ports × 2 syscalls (send + recv) = 2,000 syscalls
- Batch 32: 2,000 / 32 = 63 syscalls (96.87% reduction)
- Batch 256: 2,000 / 256 = 8 syscalls (99.61% reduction)
- Batch 1024: 2,000 / 1024 = 2 syscalls (99.90% reduction)

### Conclusions

See individual scenario markdown files for detailed statistics.
