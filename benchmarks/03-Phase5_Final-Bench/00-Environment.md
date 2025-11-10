# Phase 5 Final Benchmark - Environment Configuration

**Date:** 2025-11-09
**Version:** v0.5.0-fix
**Benchmark Suite:** Phase 5 Final (Advanced Features Complete)
**Project Status:** Phase 5 + 5.5 COMPLETE (67% overall, 5/8 phases)

## System Information

```
Linux AB-i9 6.17.7-3-cachyos #1 SMP PREEMPT_DYNAMIC Wed, 05 Nov 2025 02:29:51 +0000 x86_64 GNU/Linux
```

## CPU Information

```
Architecture:                            x86_64
CPU(s):                                  20
Model name:                              Intel(R) Core(TM) i9-10850K CPU @ 3.60GHz
Thread(s) per core:                      2
Core(s) per socket:                      10
Socket(s):                               1
CPU(s) scaling MHz:                      88%
CPU max MHz:                             5200.0000
CPU min MHz:                             800.0000
BogoMIPS:                                7200.00
Virtualization:                          VT-x
L1d cache:                               320 KiB (10 instances)
L1i cache:                               320 KiB (10 instances)
L2 cache:                                2.5 MiB (10 instances)
L3 cache:                                20 MiB (1 instance)
NUMA node(s):                            1
NUMA node0 CPU(s):                       0-19
```

## Memory Information

```
               total        used        free      shared  buff/cache   available
Mem:            62Gi        16Gi        32Gi       2.5Gi        16Gi        45Gi
Swap:          126Gi       3.5Gi       123Gi
```

## Software Versions

- **Rust:** rustc 1.91.0 (f8297e351 2025-10-28)
- **Cargo:** cargo 1.91.0 (ea2d97820 2025-10-10)
- **Hyperfine:** hyperfine 1.19.0
- **Perf:** perf version 6.17-3
- **Valgrind:** valgrind-3.25.1

## Build Configuration

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = false              # Keep symbols for profiling
```

## RUSTFLAGS

```bash
RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes"
```

This enables:
- Full debug symbols for profiling (debuginfo=2)
- Frame pointers for accurate stack traces (force-frame-pointers)

## Target Binary

```bash
-rwxr-xr-x target/release/prtip
```

File type: ELF 64-bit LSB pie executable, x86-64, dynamically linked

## Phase 5 Feature Set

This benchmark suite validates:

### Core Capabilities (Phase 1-4)
- **8 Scan Types:** SYN, Connect, UDP, FIN, NULL, Xmas, ACK, Idle
- **Service Detection:** 187 probes, 85-90% accuracy
- **OS Fingerprinting:** 2,600+ signatures, 16-probe sequence
- **Evasion Techniques:** Fragmentation, TTL, decoys, source port, badsum

### Phase 5 Advanced Features (NEW)
- **IPv6 Complete:** 100% scanner coverage (Sprint 5.1)
- **Idle Scan:** Maximum anonymity, 99.5% accuracy (Sprint 5.3)
- **Rate Limiting V3:** -1.8% overhead, industry-leading (Sprint 5.X)
- **TLS Certificate:** X.509v3, SNI support, 1.33μs parsing (Sprint 5.5)
- **Service Detection:** nmap-service-probes compatible (Sprint 5.2)

### Phase 5.5 Pre-TUI Features (NEW)
- **Event System:** -4.1% overhead, pub-sub architecture (Sprint 5.5.3)
- **Plugin System:** Lua 5.4, sandboxed, <10ms overhead (Sprint 5.8)
- **Benchmarking Framework:** 20 scenarios, CI/CD regression detection (Sprint 5.9)
- **Documentation:** 50,510+ lines comprehensive guides (Sprint 5.10)

## Test Parameters

### Network Environment
- **Target:** Localhost loopback (127.0.0.1, ::1)
- **Protocol:** TCP SYN scans (default), Connect scans (comparison)
- **Privileges:** Root/sudo for raw socket access
- **Performance Note:** Localhost is 91-182x faster than real network scans

### Hyperfine Configuration
- **Warmup:** 3 runs (unless specified)
- **Measurement:** 10 runs (standard), 5 runs (slow scans >5s)
- **Export:** JSON (machine-readable) + Markdown (human-readable)
- **Statistical Method:** IQR outlier detection, mean ± stddev

### Profiling Configuration
- **perf:** 997 Hz sampling rate, call-graph dwarf
- **valgrind:** massif heap profiler, detailed allocation tracking
- **strace:** syscall tracing with summary statistics

## Benchmark Categories

### 1. Core Scans (01-Core-Scans/)
Validate all 8 scan types on 1,000 ports:
- SYN scan (stateless, fastest)
- Connect scan (stateful, realistic)
- UDP scan (ICMP-based, slowest)
- FIN/NULL/Xmas scans (stealth)
- ACK scan (firewall detection)
- Idle scan (anonymity)

### 2. Phase 5 Features (02-Phase5-Features/)
Measure new feature performance:
- IPv6 overhead vs IPv4 baseline
- TLS certificate parsing speed
- Event system overhead
- Plugin system (Lua) overhead
- Rate limiting V3 efficiency

### 3. Scale Variations (03-Scale-Variations/)
Test scaling characteristics:
- Small: 100 ports, 1 host
- Medium: 1,000 ports, 1 host
- Large: 10,000 ports, 1 host
- Full: 65,535 ports, 1 host

### 4. Overhead Analysis (04-Overhead-Analysis/)
Feature overhead comparisons:
- Service detection (-sV)
- OS fingerprinting (-O)
- Banner grabbing
- Evasion techniques

### 5. System Profiling (05-07/)
Low-level performance analysis:
- CPU profiling (perf + flamegraphs)
- Memory profiling (valgrind massif)
- I/O analysis (strace syscalls)

### 6. Timing Templates (08-Timing-Templates/)
T0-T5 performance comparison (localhost limitation: no network delays)

### 7. Comparative Analysis (09-Comparative-Analysis/)
Phase 4 vs Phase 5 performance evolution

## Performance Claims to Validate

From v0.5.0-fix release documentation:

| Claim | Source | Expected | Category |
|-------|--------|----------|----------|
| **10M+ pps** | README | Stateless throughput | Core |
| **-1.8% overhead** | Rate Limiting V3 | Faster than unlimited | Phase 5 |
| **15% overhead** | IPv6 | vs IPv4 baseline | Phase 5 |
| **85-90% accuracy** | Service Detection | Detection rate | Phase 5 |
| **99.5% accuracy** | Idle Scan | Zombie detection | Phase 5 |
| **1.33μs parsing** | TLS Certificate | Per certificate | Phase 5 |
| **-4.1% overhead** | Event System | vs no events | Phase 5.5 |
| **<10ms overhead** | Plugin System | Lua execution | Phase 5.5 |
| **100% coverage** | IPv6 | All 8 scan types | Phase 5 |

## Execution Environment

### System State During Benchmarks
- **CPU Governor:** Performance mode (no frequency scaling)
- **Background Processes:** Minimized (no browser, IDE during critical benchmarks)
- **Swap:** Enabled (126 GB available)
- **Network:** Loopback only (no external traffic)
- **Firewall:** Standard iptables (localhost unrestricted)

### Known Limitations
- **Localhost Performance:** 91-182x faster than real networks
- **Single-Socket System:** NUMA optimizations not testable (1 node only)
- **Privileged Operations:** Some scans require root/sudo
- **UDP Scans:** ICMP rate limiting slows scans 10-100x

## Benchmark Execution Order

1. **Environment Documentation** (this file)
2. **Binary Build** (with profiling symbols)
3. **Core Scans** (baseline performance)
4. **Phase 5 Features** (new capability overhead)
5. **Scale Variations** (scalability validation)
6. **Overhead Analysis** (feature comparison)
7. **System Profiling** (low-level analysis)
8. **Comparative Analysis** (Phase 4 vs Phase 5)
9. **Report Synthesis** (comprehensive README.md)

## Data Retention

All raw benchmark data preserved in respective subdirectories:
- JSON files: Machine-readable, regression detection
- Markdown files: Human-readable summaries
- Perf/strace/massif: Raw profiling data
- Flamegraphs: Visual CPU profiling (SVG)

## References

- **Phase 4 Benchmarks:** benchmarks/02-Phase4_Final-Bench/
- **Sprint 5.9 Framework:** benchmarks/archive/16-Sprint5.9-Benchmarking-Framework/
- **Benchmarking Guide:** docs/31-BENCHMARKING-GUIDE.md
- **Performance Guide:** docs/34-PERFORMANCE-CHARACTERISTICS.md
- **Release Notes:** CHANGELOG.md (v0.5.0-fix)

---

**Environment Documentation Complete**
**Status:** Ready for benchmark execution
**Next:** Build release binary with profiling symbols
