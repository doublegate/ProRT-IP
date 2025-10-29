# ProRT-IP v0.4.0 Phase 4 Final Benchmarking Test Plan

**Date:** 2025-10-28
**Version:** v0.4.0
**Test Environment:** CachyOS Linux, Intel i9-10850K (10C/20T), 62GB RAM
**Previous Baseline:** benchmarks/01-Phase4_PreFinal-Bench/ (v0.3.0, Sprint 4.9)

## Objective

Comprehensive benchmarking of ProRT-IP v0.4.0 to:
1. Validate Phase 4 performance improvements (Sprints 4.15-4.23)
2. Measure evasion technique overhead (Sprint 4.20)
3. Validate zero-copy optimization impact (Sprint 4.17: 15% improvement)
4. Measure NUMA optimization impact (Sprint 4.19: 20-30% expected)
5. Assess error handling overhead (Sprint 4.22: <5% target)
6. Compare to previous v0.3.0 baseline
7. Validate README.md performance claims

## Key Phase 4 Enhancements to Validate

| Sprint | Feature | Expected Impact |
|--------|---------|-----------------|
| 4.15 | TLS handshake | 50% → 70-80% detection rate |
| 4.16 | CLI improvements | No performance impact |
| 4.17 | Zero-copy optimization | 15% improvement (68.3ns → 58.8ns) |
| 4.18 | PCAPNG capture | <5% overhead (opt-in) |
| 4.19 | NUMA optimization | 20-30% improvement (multi-socket) |
| 4.20 | Evasion techniques | 0-7% overhead per technique |
| 4.21 | IPv6 foundation | No performance impact (minimal code) |
| 4.22 | Error handling | <5% overhead (4.2% measured) |

## Test Scenarios

### 1. Scan Performance Benchmarks (scan-performance/)

Test basic scanning performance across different scan types and port ranges.

**Tools:** hyperfine (primary), /usr/bin/time (resource usage)

**Scenarios:**

```bash
# A. TCP SYN - Common ports (6 ports)
hyperfine --warmup 3 --runs 10 \
  --export-json scan-performance/01-syn-common.json \
  './target/release/prtip -sS -p 80,443,8080,22,21,25 127.0.0.1'

# B. TCP SYN - Top 100 ports (fast scan)
hyperfine --warmup 3 --runs 10 \
  --export-json scan-performance/02-syn-top100.json \
  './target/release/prtip -sS -F 127.0.0.1'

# C. TCP SYN - Full range (65K ports - CRITICAL BENCHMARK)
hyperfine --warmup 2 --runs 5 \
  --export-json scan-performance/03-syn-full-range.json \
  './target/release/prtip -sS -p- 127.0.0.1'

# D. TCP Connect - Common ports (no privileges)
hyperfine --warmup 3 --runs 10 \
  --export-json scan-performance/04-connect-common.json \
  './target/release/prtip -sT -p 80,443,8080,22,21,25 127.0.0.1'

# E. Service detection - Common ports
hyperfine --warmup 3 --runs 10 \
  --export-json scan-performance/05-service-detection.json \
  './target/release/prtip -sV -p 80,443,8080,22,21,25 127.0.0.1'

# F. OS fingerprinting
hyperfine --warmup 3 --runs 10 \
  --export-json scan-performance/06-os-fingerprint.json \
  './target/release/prtip -O 127.0.0.1'

# G. Aggressive mode (-A)
hyperfine --warmup 3 --runs 10 \
  --export-json scan-performance/07-aggressive-mode.json \
  './target/release/prtip -A -p 80,443,8080,22,21,25 127.0.0.1'
```

**Expected Results (based on v0.3.0 baseline):**
- Common ports (6): ~4-5ms
- Top 100 ports: ~30-50ms
- Full range (65K): ~190ms (critical - was >180s before Sprint 4.4 fix)
- Service detection: 2-3x slower than basic scan
- OS fingerprinting: 1.5-2x slower than basic scan

### 2. Evasion Technique Overhead (evasion-overhead/)

Measure overhead of Sprint 4.20 evasion techniques.

**Tools:** hyperfine (comparative benchmarking)

**Baseline:**
```bash
hyperfine --warmup 3 --runs 10 \
  --export-json evasion-overhead/00-baseline.json \
  './target/release/prtip -sS -p 80,443,8080 127.0.0.1'
```

**Techniques:**
```bash
# A. IP Fragmentation (-f)
hyperfine --warmup 3 --runs 10 \
  --export-json evasion-overhead/01-fragmentation.json \
  './target/release/prtip -sS -p 80,443,8080 -f 127.0.0.1'

# B. Custom MTU (--mtu 200)
hyperfine --warmup 3 --runs 10 \
  --export-json evasion-overhead/02-custom-mtu.json \
  './target/release/prtip -sS -p 80,443,8080 --mtu 200 127.0.0.1'

# C. TTL Manipulation (--ttl 64)
hyperfine --warmup 3 --runs 10 \
  --export-json evasion-overhead/03-ttl-control.json \
  './target/release/prtip -sS -p 80,443,8080 --ttl 64 127.0.0.1'

# D. Bad Checksums (--badsum)
hyperfine --warmup 3 --runs 10 \
  --export-json evasion-overhead/04-bad-checksum.json \
  './target/release/prtip -sS -p 80,443,8080 --badsum 127.0.0.1'

# E. Decoy Scanning (-D RND:5)
hyperfine --warmup 3 --runs 10 \
  --export-json evasion-overhead/05-decoy-random.json \
  './target/release/prtip -sS -p 80,443,8080 -D RND:5 127.0.0.1'

# F. Source Port (-g 53)
hyperfine --warmup 3 --runs 10 \
  --export-json evasion-overhead/06-source-port.json \
  './target/release/prtip -sS -p 80,443,8080 -g 53 127.0.0.1'

# G. Combined Evasion (all techniques)
hyperfine --warmup 3 --runs 10 \
  --export-json evasion-overhead/07-combined.json \
  './target/release/prtip -sS -p 80,443,8080 -f --ttl 32 --badsum -D RND:3 -g 53 127.0.0.1'
```

**Expected Results:**
- Individual techniques: 0-7% overhead each
- Combined: 10-20% overhead
- Fragmentation likely highest overhead (packet splitting)

### 3. Timing Template Comparison (timing-templates/)

Compare T0-T5 timing templates.

**Tools:** hyperfine (comparative mode)

```bash
hyperfine --warmup 2 --runs 5 \
  --export-json timing-templates/timing-comparison.json \
  -n "T0-Paranoid" './target/release/prtip -sS -T0 -p 80,443,8080 127.0.0.1' \
  -n "T1-Sneaky" './target/release/prtip -sS -T1 -p 80,443,8080 127.0.0.1' \
  -n "T2-Polite" './target/release/prtip -sS -T2 -p 80,443,8080 127.0.0.1' \
  -n "T3-Normal" './target/release/prtip -sS -T3 -p 80,443,8080 127.0.0.1' \
  -n "T4-Aggressive" './target/release/prtip -sS -T4 -p 80,443,8080 127.0.0.1' \
  -n "T5-Insane" './target/release/prtip -sS -T5 -p 80,443,8080 127.0.0.1'
```

**Expected Results:**
- T0-T2: Much slower (intentional delays)
- T3: Baseline (default)
- T4-T5: Minimal difference on localhost (delays don't apply)

### 4. Scalability Analysis (scalability/)

Measure performance scaling with port count.

**Tools:** hyperfine

```bash
hyperfine --warmup 3 --runs 10 \
  --export-json scalability/port-scaling.json \
  -n "6-ports" './target/release/prtip -sS -p 80,443,8080,22,21,25 127.0.0.1' \
  -n "100-ports" './target/release/prtip -sS -F 127.0.0.1' \
  -n "1000-ports" './target/release/prtip -sS -p 1-1000 127.0.0.1' \
  -n "10000-ports" './target/release/prtip -sS -p 1-10000 127.0.0.1'
```

**Expected Results:**
- Linear or sub-linear scaling
- 6 ports: ~4-5ms
- 100 ports: ~30-50ms
- 1K ports: ~4-10ms
- 10K ports: ~39-50ms

### 5. Resource Usage (resource-usage/)

Deep resource profiling.

**Tools:** /usr/bin/time, perf, strace, valgrind/massif

```bash
# A. Memory and CPU usage
/usr/bin/time -v ./target/release/prtip -sS -p- 127.0.0.1 2>&1 | tee resource-usage/time-full-scan.txt

# B. CPU profiling
perf record --call-graph dwarf -F 997 -o resource-usage/perf.data \
  ./target/release/prtip -sS -p 1-10000 127.0.0.1
perf report -i resource-usage/perf.data > resource-usage/perf-report.txt

# C. Syscall tracing
strace -c -o resource-usage/strace-summary.txt \
  ./target/release/prtip -sS -p 1-10000 127.0.0.1

# D. Memory profiling
valgrind --tool=massif --massif-out-file=resource-usage/massif.out \
  ./target/release/prtip -sS -p 1-1000 127.0.0.1
ms_print resource-usage/massif.out > resource-usage/massif-report.txt
```

**Expected Results:**
- Memory peak: <5 MB (was 1.9 MB in v0.3.0)
- Futex calls: <500 (was 398 in v0.3.0)
- CPU utilization: High (60-70% of cores)

### 6. Zero-Copy & NUMA Validation (zero-copy-numa/)

Validate Sprint 4.17 and 4.19 optimizations.

**Note:** NUMA optimization requires multi-socket system. This is single-socket (1 NUMA node).

```bash
# A. Zero-copy validation (packet building microbenchmark)
# Run cargo benchmarks if available
cargo bench --bench packet_building 2>&1 | tee zero-copy-numa/cargo-bench-output.txt

# B. NUMA validation (will show no improvement on single-socket)
hyperfine --warmup 3 --runs 10 \
  --export-json zero-copy-numa/numa-comparison.json \
  -n "Default" './target/release/prtip -sS -p 1-10000 127.0.0.1' \
  -n "NUMA-Enabled" './target/release/prtip -sS -p 1-10000 --numa 127.0.0.1' \
  -n "NUMA-Disabled" './target/release/prtip -sS -p 1-10000 --no-numa 127.0.0.1'
```

**Expected Results:**
- Zero-copy: 58.8ns/packet (15% improvement from 68.3ns)
- NUMA: No difference (single-socket system)

### 7. Error Handling Overhead (included in all benchmarks)

Sprint 4.22 error handling is always active. Overhead should be <5%.

**Validation:** Compare v0.4.0 times to v0.3.0 baseline:
- If <5% slower, overhead is acceptable
- If >5% slower, investigate regression

## Comparison Points

### vs. v0.3.0 Baseline (benchmarks/01-Phase4_PreFinal-Bench/)

| Metric | v0.3.0 | v0.4.0 Target | Test |
|--------|--------|---------------|------|
| 1K ports | 4.5ms | <5ms | scan-performance/01 |
| 10K ports | 39.4ms | <45ms | scalability |
| 65K ports | 190.9ms | <200ms | scan-performance/03 |
| Memory peak | 1.9 MB | <5 MB | resource-usage |
| Futex calls | 398 | <500 | resource-usage |

### vs. README.md Claims

| Claim | Location | Test |
|-------|----------|------|
| 66ms common ports | README line 253 | scan-performance/01 |
| 190ms full 65K scan | README line 304 | scan-performance/03 |
| 58.8ns/packet | README line 308 | zero-copy-numa/cargo-bench |
| 0-7% evasion overhead | README line 217 | evasion-overhead/* |

## Success Criteria

**Performance:**
- ✅ 65K ports in <200ms (critical regression check)
- ✅ 10K ports in <50ms
- ✅ Common ports in <10ms
- ✅ Evasion overhead <10% per technique
- ✅ Combined evasion overhead <25%

**Resources:**
- ✅ Memory peak <10 MB
- ✅ Futex calls <1000
- ✅ No memory leaks

**Validation:**
- ✅ README claims accurate ±10%
- ✅ No >10% regressions vs v0.3.0
- ✅ Error handling overhead <5%

## Tools Configuration

**Hyperfine:**
- Version: 1.19.0
- Warmup: 3 runs (2 for slow tests)
- Measurement: 10 runs (5 for slow tests)
- Shell: None (direct binary)
- Export: JSON for all benchmarks

**Perf:**
- Frequency: 997 Hz
- Call graph: dwarf
- Output: perf.data files

**Strace:**
- Mode: -c (count summary)
- Focus: Futex analysis

**Valgrind/Massif:**
- Tool: massif
- Snapshots: Detailed
- Output: massif.out files

## Build Configuration

```bash
# Clean build with debug symbols for profiling
cargo clean
RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes" cargo build --release

# Verify version
./target/release/prtip --version
```

**Expected:** prtip 0.4.0

## Directory Structure

```
02-Phase4_Final-Bench/
├── TEST-PLAN.md (this file)
├── BENCHMARK-REPORT.md (to be generated)
├── README.md (summary)
├── scan-performance/
│   ├── 01-syn-common.json
│   ├── 02-syn-top100.json
│   ├── 03-syn-full-range.json
│   ├── 04-connect-common.json
│   ├── 05-service-detection.json
│   ├── 06-os-fingerprint.json
│   └── 07-aggressive-mode.json
├── evasion-overhead/
│   ├── 00-baseline.json
│   ├── 01-fragmentation.json
│   ├── 02-custom-mtu.json
│   ├── 03-ttl-control.json
│   ├── 04-bad-checksum.json
│   ├── 05-decoy-random.json
│   ├── 06-source-port.json
│   └── 07-combined.json
├── timing-templates/
│   └── timing-comparison.json
├── scalability/
│   └── port-scaling.json
├── resource-usage/
│   ├── time-full-scan.txt
│   ├── perf.data
│   ├── perf-report.txt
│   ├── strace-summary.txt
│   ├── massif.out
│   └── massif-report.txt
├── zero-copy-numa/
│   ├── cargo-bench-output.txt
│   └── numa-comparison.json
└── raw-data/ (backup of all outputs)
```

## Execution Timeline

**Estimated Total:** 4-6 hours

1. **Setup (30 min):** Build, test environment
2. **Scan Performance (1 hour):** 7 scenarios
3. **Evasion Overhead (1 hour):** 8 benchmarks
4. **Timing Templates (30 min):** 1 comparative benchmark
5. **Scalability (30 min):** 1 benchmark
6. **Resource Usage (1 hour):** 4 profiling runs
7. **Zero-Copy/NUMA (30 min):** 2 benchmarks
8. **Analysis (1 hour):** Compare results, calculate metrics
9. **Report Generation (1 hour):** Write comprehensive report
10. **Documentation Update (30 min):** Update README.md

## Notes

- Use localhost (127.0.0.1) for all tests (safe, no external scanning)
- All tests run with elevated privileges (required for SYN scans)
- Full port scans (65K) are time-consuming but critical for regression testing
- NUMA optimization will show no improvement (single-socket system)
- Results are upper-bound performance (localhost loopback, no network latency)

---

**Status:** Test plan complete ✅ | **Next:** Execute benchmarks
