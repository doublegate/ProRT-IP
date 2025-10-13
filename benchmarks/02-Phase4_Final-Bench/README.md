# Phase 4 Final Benchmark Suite - v0.3.5

**Date:** 2025-10-12
**Version:** ProRT-IP v0.3.5 (Post-Nmap Compatibility)
**Baseline:** Phase 4 PreFinal (v0.3.0)
**Duration:** ~40 minutes

---

## Executive Summary

**Status:** ⚠️ **REGRESSIONS DETECTED** - Investigation Required

This comprehensive benchmark suite reveals **significant performance regressions** in v0.3.5 compared to PreFinal v0.3.0:

| Metric | Status | Details |
|--------|--------|---------|
| **Performance** | ⚠️ REGRESSED | 31-96% slower on core scans |
| **Memory** | ✅ EXCELLENT | <11MB for 10K ports |
| **Stability** | ⚠️ CONCERNS | High variance on 10K ports |
| **Targets** | ✅ PASS | All hard targets met |

**Key Findings:**
- ⚠️ **10K port scan: 96% slower** (77ms vs 39ms) - CRITICAL
- ⚠️ 1K port scan: 31% slower (5.9ms vs 4.5ms)
- ⚠️ 65K port scan: 30% slower (248ms vs 191ms)
- ✅ Database mode: 13% faster (65ms vs 75ms) - Only improvement
- ⚠️ High variance: 10K stddev increased 4.9x (29.7ms vs 3.1ms)

**Recommendation:** Investigate and fix regressions before final release. All documented targets still pass, but baseline regression is concerning.

