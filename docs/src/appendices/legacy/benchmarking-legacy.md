# Benchmarking (Legacy)

This document contains the original benchmarking methodology and results from Phase 4.

## Phase 4 Methodology

### Tools

- **hyperfine** - Command-line benchmarking
- **perf** - Linux performance analysis
- **flamegraph** - CPU profiling visualization

### Test Environment

| Component | Specification |
|-----------|---------------|
| CPU | AMD Ryzen 9 5900X |
| Memory | 32GB DDR4-3600 |
| Network | 10Gbps Ethernet |
| OS | Ubuntu 22.04 LTS |

## Phase 4 Baseline Results

### SYN Scan Performance

| Ports | Time | Throughput |
|-------|------|------------|
| 100 | 45ms | 2,222 pps |
| 1,000 | 250ms | 4,000 pps |
| 10,000 | 1.8s | 5,556 pps |
| 65,535 | 8.2s | 7,992 pps |

### Memory Usage

| Operation | Memory |
|-----------|--------|
| Idle | 12MB |
| 1K port scan | 45MB |
| 10K port scan | 78MB |
| 65K port scan | 95MB |

### Comparison with nmap

| Scanner | 1K ports | 10K ports |
|---------|----------|-----------|
| ProRT-IP | 250ms | 1.8s |
| nmap | 3.2s | 28s |
| Speedup | 12.8x | 15.5x |

## Benchmark Commands

```bash
# Basic throughput test
hyperfine --warmup 2 \
    'prtip -sS -p 1-1000 localhost'

# Memory profiling
/usr/bin/time -v prtip -sS -p 1-65535 target

# CPU profiling
perf record prtip -sS -p 1-10000 target
perf report
```

## Current Benchmarking

For current benchmarking methodology, see:

- [Benchmarking Guide](../../advanced/benchmarking.md)
- [Performance Characteristics](../../advanced/performance-characteristics.md)

## Historical Data

Phase 4 baseline data preserved for regression detection:

- Baseline established: October 2025
- Tests: 1,166 passing
- Coverage: 37.26%

## See Also

- [Performance Tuning](../../advanced/performance-tuning.md)
- [Phase 4 Archive](../archives/phase4.md)
