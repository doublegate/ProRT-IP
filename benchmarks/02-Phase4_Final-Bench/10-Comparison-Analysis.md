# Phase 4 Final Benchmark - Comparison Analysis

**Date:** 2025-10-12
**Version:** v0.3.5 (Post-Nmap Compatibility)
**Baseline:** Phase 4 PreFinal (v0.3.0)

---

## Executive Summary

**Status:** ⚠️ **PERFORMANCE REGRESSION DETECTED**

The v0.3.5 release shows **significant performance regressions** compared to PreFinal benchmarks:
- **1K ports**: +31% slower (5.9ms vs 4.5ms)
- **10K ports**: +96% slower (77.1ms vs 39.4ms) ⚠️ **CRITICAL**
- **65K ports**: +30% slower (248.3ms vs 190.9ms)
- **10K + DB**: -13% faster (65.0ms vs 75.1ms) ✅ Only improvement

**Root Cause Hypothesis**: Changes introduced in v0.3.5 nmap compatibility (CLI parsing, argument handling, or internal restructuring) may have inadvertently impacted core scanning performance.

**Recommendation**: Investigate and revert performance-degrading changes before final release.

---

## Detailed Performance Comparison

### 1. Common Ports (Top 100) - NEW TEST

| Metric | v0.3.5 Final |
|--------|--------------|
| **Mean** | **3.8ms** ± 0.2ms |
| **Range** | 3.6ms - 4.2ms |
| **Status** | ✅ NEW BASELINE |

**Notes:**
- Fast scan test not present in PreFinal benchmarks
- Establishes baseline for future comparisons
- Excellent performance for small port ranges

---

### 2. Standard Scan (1,000 Ports)

| Metric | PreFinal (v0.3.0) | Final (v0.3.5) | Delta | % Change |
|--------|-------------------|----------------|-------|----------|
| **Mean** | 4.46ms ± 0.39ms | 5.85ms ± 0.27ms | +1.39ms | **+31%** ⚠️ |
| **Median** | - | 5.74ms | - | - |
| **Min** | - | 5.59ms | - | - |
| **Max** | - | 6.34ms | - | - |
| **Scan Rate** | 224,215 ports/sec | 170,940 ports/sec | -53,275 pps | **-24%** |

**Analysis:**
- **31% performance regression** for standard scans
- Stddev improved (0.27ms vs 0.39ms) - more consistent but slower
- Dropped below 200K ports/sec threshold

**Impact:** Medium - Affects standard use cases

---

### 3. Large Scan (10,000 Ports)

| Metric | PreFinal (v0.3.0) | Final (v0.3.5) | Delta | % Change |
|--------|-------------------|----------------|-------|----------|
| **Mean** | 39.4ms ± 3.1ms | 77.1ms ± 29.7ms | +37.7ms | **+96%** ⚠️⚠️⚠️ |
| **Median** | - | 66.3ms | - | - |
| **Min** | - | 62.5ms | - | - |
| **Max** | - | 160.4ms | - | - |
| **Scan Rate** | 253,677 ports/sec | 129,702 ports/sec | -123,975 pps | **-49%** |

**Analysis:**
- **CRITICAL: 96% performance regression** (nearly 2x slower!)
- Massive stddev increase (3.1ms → 29.7ms) indicates high variability
- Max time 160.4ms (4x the min!) suggests occasional severe slowdowns
- Scan rate dropped by nearly 50%

**Impact:** CRITICAL - Unacceptable for production use

**Hypothesis:**
1. CLI argument parsing overhead scaling with port count?
2. Memory allocation changes in v0.3.5?
3. Rate limiter misconfiguration?
4. Tokio task spawning changes?

**Next Steps:**
1. Profile 10K port scan with perf/flamegraph
2. Compare memory allocations between versions
3. Check git diff between v0.3.0 and v0.3.5 for performance-impacting changes
4. Consider bisecting commits to find regression introduction point

---

### 4. Full Range (65,535 Ports)

| Metric | PreFinal (v0.3.0) | Final (v0.3.5) | Delta | % Change |
|--------|-------------------|----------------|-------|----------|
| **Mean** | 190.9ms ± 7.1ms | 248.3ms ± 20.0ms | +57.4ms | **+30%** ⚠️ |
| **Median** | - | 250.5ms | - | - |
| **Scan Rate** | 343,195 ports/sec | 263,862 ports/sec | -79,333 pps | **-23%** |

**Analysis:**
- **30% slower** for full port range scans
- Stddev nearly 3x higher (7.1ms → 20.0ms)
- Still maintains >250K pps but below PreFinal baseline

**Impact:** High - Enterprise users scanning full ranges will notice

---

### 5. Storage Overhead (10K Ports + Database)

| Metric | PreFinal (v0.3.0) | Final (v0.3.5) | Delta | % Change |
|--------|-------------------|----------------|-------|----------|
| **Mean** | 75.1ms ± 6.1ms | 65.0ms ± 7.9ms | -10.1ms | **-13%** ✅ |
| **Min** | - | 55.9ms | - | - |
| **Max** | - | 77.5ms | - | - |
| **Scan Rate** | 133,156 ports/sec | 153,846 ports/sec | +20,690 pps | **+16%** |

**Analysis:**
- **ONLY positive result**: 13% faster with database!
- Database write performance improved
- Scan rate increased by 16%

**Impact:** Positive - Database storage mode is now more efficient

**Hypothesis:** SQLite batch insert optimizations or reduced lock contention in v0.3.5

---

### 6. Timing Templates (1,000 Ports)

| Template | PreFinal (v0.3.0) | Final (v0.3.5) | Delta | % Change | Notes |
|----------|-------------------|----------------|-------|----------|-------|
| **T0 (Paranoid)** | 4.96ms ± 0.49ms | 6.3ms ± 0.4ms | +1.34ms | +27% | Slower |
| **T3 (Normal)** | 4.46ms ± 0.39ms | 6.6ms ± 0.3ms | +2.14ms | +48% ⚠️ | Worst |
| **T5 (Insane)** | 4.21ms ± 0.36ms | 6.1ms ± 0.6ms | +1.89ms | +45% ⚠️ | Critical |

**Analysis:**
- **ALL timing templates show regressions** (27-48%)
- T3 and T5 show **45-48% slower** (unacceptable)
- T5 should be fastest but shows high variance (0.6ms stddev)

**Impact:** CRITICAL - Timing templates are a key differentiator, regressions here are unacceptable

---

## Memory Analysis

### Peak Memory Usage (Valgrind Massif)

| Test | Peak Memory | Largest Allocation | Notes |
|------|-------------|-------------------|--------|
| **1K ports** | 2.22 MB | 1.2 MB | Reasonable |
| **10K ports** | 10.42 MB | 1.2 MB | Linear scaling |

**Analysis:**
- Memory usage scales linearly with port count (~1KB per port)
- Largest single allocation: 1.2MB (consistent across tests)
- No obvious memory leaks detected
- **Conclusion:** Memory is NOT the bottleneck

---

## CPU Profiling Analysis

### Perf Sampling (10K Ports, 9 samples)

**Top Hotspots:**
1. **libc syscalls** (16.94%) - System call overhead
2. **tokio scheduler** (10.38%) - Async runtime overhead
3. **tokio yield** (10.38%) - Task scheduling

**Analysis:**
- Very few samples captured (9) due to fast execution
- Hotspots are expected (syscalls, async scheduling)
- No obvious user-space CPU bottlenecks visible

**Limitation:** Scan too fast for meaningful perf analysis. Need longer-running test or higher sampling rate.

---

## System Call Analysis

### Syscall Counts (strace)

| Test | Total Syscalls | Top Syscall | % Time | Notes |
|------|----------------|-------------|--------|-------|
| **1K ports** | 708 | `futex` (178, 79.83%) | 79.83% | Async coordination |
| **10K ports** | 1,078 | `futex` (517, 84.82%) | 84.82% | More async overhead |

**Key Observations:**
- `futex` dominates (Tokio async coordination)
- 10K ports: 52% increase in syscalls (708 → 1,078) but 96% slower
  - **Syscall overhead does NOT explain 96% regression**
- `brk` increases (12 → 65 calls) suggest memory allocation activity

**Analysis:**
- Syscall count increase is modest (52%)
- Performance regression (96%) is disproportionate
- **Hypothesis:** Regression is NOT primarily syscall-related

---

## Flamegraph Analysis

**Files Generated:**
- `01-CPU-Flamegraph-1K.svg` (29 KB, 38 samples)
- `02-CPU-Flamegraph-10K.svg` (39 KB, 276 samples)

**Status:** ✅ Generated successfully

**Next Steps:**
1. Open flamegraphs in browser
2. Identify hot paths in 10K scan
3. Compare with PreFinal flamegraphs (if available)
4. Look for new/expanded stack frames in v0.3.5

---

## Performance Target Validation

### Target: docs/07-PERFORMANCE.md

| Metric | Target | PreFinal | Final (v0.3.5) | Status |
|--------|--------|----------|----------------|--------|
| **TCP Connect (1K ports)** | <10ms | 4.5ms ✅ | 5.9ms ✅ | PASS |
| **TCP Connect (10K ports)** | <100ms | 39.4ms ✅ | 77.1ms ✅ | PASS |
| **TCP Connect (65K ports)** | <500ms | 190.9ms ✅ | 248.3ms ✅ | PASS |
| **Scan Rate (Connect)** | >100K pps | 253K pps ✅ | 130K pps ✅ | PASS |
| **Memory (10K)** | <20MB | <10MB ✅ | 10.4MB ✅ | PASS |

**Analysis:**
- All hard targets still PASS ✅
- However, **significant regression vs PreFinal baseline**
- v0.3.5 is "acceptable" but **notably worse** than v0.3.0

**Recommendation:** Targets need revision or regression needs fixing

---

## Statistical Significance

### Variance Analysis

| Test | PreFinal StdDev | Final StdDev | Variance Change |
|------|-----------------|--------------|-----------------|
| 1K ports | 0.39ms (8.8%) | 0.27ms (4.7%) | ✅ More consistent |
| 10K ports | 3.1ms (7.9%) | 29.7ms (38.5%) | ⚠️ Much less consistent |
| 65K ports | 7.1ms (3.7%) | 20.0ms (8.1%) | ⚠️ More variance |
| 10K+DB | 6.1ms (8.1%) | 7.9ms (12.2%) | ⚠️ Slightly worse |

**Observations:**
- 1K ports: **Improved consistency** despite slower mean
- 10K ports: **4.9x higher variance** - highly unstable!
- 65K ports: **2.8x higher variance** - concerning
- High variance suggests:
  - Contention issues
  - Garbage collection pauses (unlikely in Rust)
  - OS scheduling variability
  - Resource exhaustion

---

## Root Cause Investigation Roadmap

### Immediate Actions (High Priority)

1. **Git Diff Analysis**
   ```bash
   git diff v0.3.0..v0.3.5 -- crates/prtip-scanner/src/
   git diff v0.3.0..v0.3.5 -- crates/prtip-cli/src/args.rs
   git diff v0.3.0..v0.3.5 -- crates/prtip-core/src/
   ```
   - Focus: Scanner engine, CLI parsing, core types

2. **Profile Comparison**
   - Run flamegraph on v0.3.0 for 10K ports
   - Compare side-by-side with v0.3.5 flamegraph
   - Identify new/expanded hot paths

3. **Memory Allocation Profiling**
   ```bash
   valgrind --tool=massif --detailed-freq=1 ./target/release/prtip -p 1-10000 127.0.0.1
   ```
   - Compare allocation patterns between versions

4. **Code Review**
   - Nmap compatibility changes in `args.rs`
   - Greppable output parsing overhead
   - Top ports database queries (if used in hot path)

### Secondary Analysis (Medium Priority)

5. **Timing Template Logic**
   - Check if T0/T3/T5 implementations changed
   - Verify rate limiter configuration

6. **Database Improvements Investigation**
   - What changed to make DB mode 13% faster?
   - Can those optimizations apply to in-memory mode?

7. **Bisect Regression**
   ```bash
   git bisect start v0.3.5 v0.3.0
   git bisect run ./benchmarks/scripts/quick-bench.sh
   ```

### Validation Testing (Low Priority)

8. **Reproduce on Clean System**
   - Rule out environment-specific issues
   - Test on different hardware

9. **Feature Flag Testing**
   - Disable nmap compatibility flags
   - Test raw performance without CLI overhead

---

## Recommendations

### Critical (Must Fix Before Release)

1. **DO NOT RELEASE v0.3.5** until 10K port regression is resolved
   - 96% slowdown is unacceptable for production
   - Users will notice and complain

2. **Investigate 10K Port Performance**
   - Use git bisect to find regression commit
   - Profile and compare flamegraphs
   - Check for O(n²) or O(n log n) algorithms introduced

3. **Review Timing Template Changes**
   - T3/T5 regressions (45-48%) are critical
   - These are advertised features that must perform

### High Priority (Fix Soon)

4. **Reduce Variance**
   - 10K port variance (38.5%) is too high
   - Investigate contention or resource exhaustion

5. **Update Performance Documentation**
   - If regressions cannot be fixed, update targets
   - Be transparent about trade-offs

### Medium Priority (Enhancement)

6. **Port DB Optimizations to In-Memory Mode**
   - DB mode gained 13% - understand why
   - Apply lessons to default in-memory path

7. **Add Performance Regression Testing**
   - CI/CD should catch regressions before merge
   - Establish performance budgets per test

---

## Conclusion

**Summary:**
- v0.3.5 shows **significant performance regressions** vs v0.3.0
- 10K port scan is **96% slower** (CRITICAL)
- Timing templates are **27-48% slower** (CRITICAL)
- Database mode improved 13% (positive outlier)
- All hard targets still pass, but baseline regression is unacceptable

**Root Cause:** Likely related to v0.3.5 changes (nmap compatibility, CLI restructuring, or internal refactoring)

**Action Required:**
1. Investigate and fix regressions before release
2. Use git bisect and profiling to pinpoint root cause
3. Consider reverting performance-degrading commits
4. Add CI performance testing to prevent future regressions

**Status:** ⚠️ **NOT READY FOR RELEASE** - Performance must be restored to PreFinal levels

---

## Appendix A: Raw Data Summary

### Hyperfine Results

```json
{
  "common_ports_100": {
    "mean": 0.0038, "stddev": 0.0002, "min": 0.0036, "max": 0.0042
  },
  "ports_1k": {
    "prefinal": {"mean": 0.0045, "stddev": 0.0004},
    "final": {"mean": 0.0059, "stddev": 0.0003}
  },
  "ports_10k": {
    "prefinal": {"mean": 0.0394, "stddev": 0.0031},
    "final": {"mean": 0.0771, "stddev": 0.0297}
  },
  "ports_65k": {
    "prefinal": {"mean": 0.1909, "stddev": 0.0071},
    "final": {"mean": 0.2483, "stddev": 0.0200}
  },
  "ports_10k_db": {
    "prefinal": {"mean": 0.0751, "stddev": 0.0061},
    "final": {"mean": 0.0650, "stddev": 0.0079}
  }
}
```

### Memory (Valgrind Massif)

```
1K ports:  Peak 2.22 MB, Largest allocation 1.2 MB
10K ports: Peak 10.42 MB, Largest allocation 1.2 MB
```

### System Calls (strace)

```
1K ports:  708 total (178 futex, 79.83% time)
10K ports: 1078 total (517 futex, 84.82% time)
```

---

**Generated:** 2025-10-12 22:40 UTC
**Tool:** Phase 4 Final Benchmark Suite
**Duration:** ~40 minutes
