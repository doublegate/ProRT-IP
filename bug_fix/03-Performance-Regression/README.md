# Performance Regression (Variable Shadowing Bug)

**Status:** ✅ RESOLVED (Sprint 4.13)
**Priority:** CRITICAL
**Fix Commit:** 6a00b73
**Fixed Date:** 2025-10-11

---

## Issue Summary

**Problem:** 50-800x performance degradation on large network scans
**User Report:** 192.168.4.0/24 × 10K ports running at 289 pps with 2-hour ETA (expected: 10-30 min)
**Root Cause:** Variable shadowing in scheduler.rs causing 1ms polling on 2.56M port scan
**Solution:** Capture `total_scan_ports` before loop to enable total-scan-aware adaptive polling
**Result:** 289 pps → 2,844 pps (10x speedup), 2 hours → 15 minutes (8x faster)

---

## Files

- **01-User-Report.md** - Initial problem report
- **02-Investigation.md** - Diagnostic process
- **03-Root-Cause-Variable-Shadowing.md** - Technical deep-dive
- **04-Fix-Summary.md** - Solution details
- **05-Before-After-Performance.md** - Performance comparison (before/after fix)
- **FINAL-REPORT.md** - Complete analysis and summary
- **debug-output-*.txt** - Debug logs
- **test-localhost-10k-fixed.txt** - Validation test

---

## Technical Details

### Bug (scheduler.rs lines 324, 372, 385)
```rust
// Line 324: total_scan_ports calculated
let total_scan_ports = hosts.len() * ports_per_host;

// Line 372: Variable shadowed inside loop!
for target in hosts {
    let total_scan_ports = ports_per_host;  // ← BUG: Shadows outer variable

    // Line 385: Polling based on shadowed value
    let poll_interval = if total_scan_ports < 1000 {
        Duration::from_millis(1)  // ← Wrong! Uses ports_per_host instead of total
    }
}
```

### Fix
```rust
// Capture before loop (line 360)
let total_scan_ports_for_polling = hosts.len() * ports_per_host;

// Use captured value (lines 378-399)
let poll_interval = if total_scan_ports_for_polling < 1000 {
    Duration::from_micros(200)
} else if total_scan_ports_for_polling < 10000 {
    Duration::from_micros(500)
} else if total_scan_ports_for_polling < 100000 {
    Duration::from_millis(1)
} else if total_scan_ports_for_polling < 1000000 {
    Duration::from_millis(5)
} else {
    Duration::from_millis(10)  // ← Correct for 2.56M scan
};
```

---

## Performance Results

### User's Scan (192.168.4.0/24 × 10K ports = 2.56M total)

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **Rate** | 289 pps | 2,844 pps | **10x faster** |
| **Duration** | 2 hours | 15 minutes | **8x faster** |
| **Polling Overhead** | 30% (2,160s) | 3% (27s) | **80x reduction** |
| **Polling Calls** | 7.2M | 90K | **80x fewer** |

### Localhost 10K Ports Validation

| Metric | Before | After | Result |
|--------|--------|-------|--------|
| **Duration** | 211ms | 156ms | 35% faster |
| **Rate** | 210,779 pps | 284,933 pps | 35% improvement |
| **Tests** | 498/498 | 498/498 | No regression |

---

**Last Updated:** 2025-10-11
**Status:** Production-ready, critical regression fixed
