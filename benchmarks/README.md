# ProRT-IP Benchmarks

This directory contains comprehensive performance benchmarking results for ProRT-IP WarScan across all development phases.

## Organization

Files are organized chronologically with numeric prefixes for easy reference:

### Core Benchmark Reports

| File | Description | Date | Phase |
|------|-------------|------|-------|
| **1-BASELINE-RESULTS.md** | Phase 3 baseline performance (v0.3.0) | 2025-10-09 | Phase 3 |
| **2-PHASE4-NETWORK-BENCHMARKS.md** | Sprint 4.1-4.2 network infrastructure | 2025-10-10 | Sprint 4.1-4.2 |
| **3-SPRINT4-COMPREHENSIVE-REPORT.md** | Sprint 4.3-4.4 comprehensive analysis | 2025-10-10 | Sprint 4.3-4.4 |
| **4-EXECUTIVE-SUMMARY.txt** | Quick reference summary | 2025-10-10 | Sprint 4.3-4.4 |

### Scenario Output Files (Sprint 4.3-4.4)

| File | Scenario | Ports | Description |
|------|----------|-------|-------------|
| **5-SCENARIO-1-SERVICE-DISCOVERY.txt** | Service Discovery | 10 | Metasploitable2 services |
| **6-SCENARIO-2-MEDIUM-RANGE.txt** | Medium Range | 1,025 | IANA well-known + registered |
| **7-SCENARIO-3-LARGE-RANGE.txt** | Large Range | 10,000 | High parallelism test |
| **8-SCENARIO-4-FULL-RANGE-65K.txt** | Full Range | 65,535 | **CRITICAL** - Port overflow fix validation |
| **9-SCENARIO-5A-TIMING-T3.txt** | Timing Template | 1,000 | Normal timing (T3) |
| **10-SCENARIO-5B-TIMING-T4.txt** | Timing Template | 1,000 | Aggressive timing (T4) |
| **11-SCENARIO-6-LOCKFREE-STRESS.txt** | Lock-Free Stress | 10,000 | High concurrency validation |
| **12-SCENARIO-7-SERVICE-DETECTION.txt** | Service Detection | 3 | --sV flag integration check |

### Reference Documents

| File | Description |
|------|-------------|
| **SUMMARY.txt** | Quick summary of Sprint 4.3-4.4 results |
| **NOTES-RUST-VERSION.txt** | Rust version regression investigation notes |

## Key Findings

### Phase 3 Baseline (v0.3.0, 551 tests)
- **1K ports:** 0.061s (16,803 pps)
- **10K ports:** 0.120s (83,333 pps)
- **65K ports:** **>180s HANG** (port overflow bug)

### Sprint 4.3-4.4 (v0.3.0+, 598 tests)
- **1K ports:** 0.133s (+118% slower - regression)
- **10K ports:** 0.277s (+137% slower - regression)
- **65K ports:** **0.994s (198x FASTER!)** - Critical bug fix validated ✅

### Critical Achievement
**Sprint 4.4 successfully fixed the port 65535 overflow bug that caused infinite loops on full port range scans.**

**Before:** >180s hang (unusable)
**After:** 0.994s (production-ready)
**Improvement:** 198x faster

## Performance Regression

Unexpected performance degradation on small/medium scans (2-3x slower):
- **Root Cause:** Under investigation (NOT Rust version - both running 1.90.0)
- **Likely Causes:** System state, timing methodology, statistical variance
- **Priority:** #1 for Sprint 4.5 (performance profiling with perf)

## Sprint 4.3-4.4 Validations

### ✅ Lock-Free Aggregator (Sprint 4.2/4.3)
- Integrated: `crates/prtip-scanner/src/tcp_connect.rs` line 234
- Performance: 10M+ results/sec, <100ns latency
- Correctness: All open ports correctly detected
- Extension: SYN/UDP/stealth scanners (pending Sprint 4.5)

### ✅ Batch Receiver (Sprint 4.3)
- Implemented: `crates/prtip-network/src/batch_sender.rs` lines 657-1061
- Syscall: Linux recvmmsg() for batch packet reception
- Status: NOT integrated (Sprint 4.5 priority #2)
- Expected: 30-50% syscall reduction at 1M+ pps

### ✅ Adaptive Parallelism (Sprint 4.4)
- Module: `crates/prtip-scanner/src/adaptive_parallelism.rs` (342 lines, 17 tests)
- Scaling: 20-1000 concurrent based on port count
- Integration: Fully integrated into scheduler (3 methods)
- Validation: 265% CPU on 65K ports (effective multi-core usage)

### ✅ Critical Bug Fixes (Sprint 4.4)
- Port 65535 overflow: Fixed (no infinite loop)
- Parallelism detection: Fixed (scheduler logic corrected)
- Performance: 198x improvement validated

## Sprint 4.5 Priorities

### HIGH PRIORITY (Blocking)
1. **Performance Profiling** ⭐ CRITICAL - Investigate regression
2. **BatchReceiver Integration** - SYN/UDP scanner packet capture
3. **Service Detection Integration** - Implement --sV functionality
4. **Lock-Free Extension** - Extend to other scanners

### MEDIUM PRIORITY
5. **Network-Based Testing** - External target with realistic latency
6. **CLI Display Bug Fix** - Show actual parallelism value

## Quick Reference

**Latest Benchmark:** Sprint 4.3-4.4 (2025-10-10)
**Test Count:** 598 tests passing (100% success rate)
**Critical Fix:** 65K ports from >180s hang → 0.994s (198x faster)
**Status:** Production-ready for full port range scanning

**Full Analysis:** See `3-SPRINT4-COMPREHENSIVE-REPORT.md` (31KB, 1,402 lines)
**Quick Summary:** See `4-EXECUTIVE-SUMMARY.txt` or `SUMMARY.txt`

---

**Last Updated:** 2025-10-10
**Maintained By:** ProRT-IP Development Team
