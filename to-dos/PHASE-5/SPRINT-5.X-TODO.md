# Sprint 5.X: Adaptive Rate Limiter Optimization - Todo List

**Status:** Planned (Not Started)
**Estimated Duration:** 15-20 hours (12-15h base + 3-5h buffer)
**Target Completion:** TBD (Deferred from Sprint 5.4 Phase 2)
**Sprint Priority:** MEDIUM (Performance optimization, not blocking other sprints)
**Prerequisites:** Sprint 5.4 Phase 1-2 Complete ✅

---

## Background

Sprint 5.4 Phase 2 comprehensive benchmarking revealed that the adaptive rate limiter introduces **22-40% overhead** on large port ranges, significantly exceeding the <5% target. This sprint will optimize the implementation to achieve production-ready performance.

**Root Causes Identified:**
1. Batch sizing may not be working as designed (overhead independent of rate)
2. Convergence calculations potentially too frequent (every packet vs every N packets)
3. Circular buffer update overhead not profiled
4. Sleep/timing overhead not amortized at high rates

**Current Benchmark Results (v0.4.3):**
- Small scans (18 ports): +1% overhead ✅ (acceptable)
- Large scans (1000 ports): +40% overhead ❌ (unacceptable)
- Rate scaling: 22-36% overhead across 10K-1M pps ❌ (should decrease with rate)

**Target:** <5% overhead across all scenarios (consistent with hostgroup limiter performance)

---

## Progress Tracking

**Total Items:** 39 tasks across 6 phases
**Completed:** 0 / 39 (0%)
**In Progress:** 0
**Remaining:** 39
**Progress:** ▱▱▱▱▱▱▱▱▱▱ 0%

---

## Phase 1: Profiling & Root Cause Analysis (3-4h)

**Duration:** 3-4 hours
**Status:** Not Started
**Progress:** 0 / 8 (0%)

### Task 1.1: Environment Setup (30m)

- [ ] **Task 1.1.1:** Build release binary with debug symbols (cargo build --release with debug=1) (10m)
- [ ] **Task 1.1.2:** Verify perf is installed (perf --version, install if needed) (5m)
- [ ] **Task 1.1.3:** Create /tmp/ProRT-IP/SPRINT-5.X/ benchmark directory (5m)
- [ ] **Task 1.1.4:** Copy Sprint 5.4 Phase 2 benchmark scripts to new directory (10m)

### Task 1.2: CPU Profiling with perf (1.5h)

- [ ] **Task 1.2.1:** Profile baseline scan (no rate limiting) for reference (20m)
- [ ] **Task 1.2.2:** Profile adaptive rate limiter scan (--max-rate 100000) (20m)
- [ ] **Task 1.2.3:** Generate flamegraph for both profiles (30m)
- [ ] **Task 1.2.4:** Compare flamegraphs, identify hotspots (20m)

### Task 1.3: Code Analysis (1h)

- [ ] **Task 1.3.1:** Instrument adaptive_rate_limiter.rs with debug logging (20m)
- [ ] **Task 1.3.2:** Run instrumented scan, analyze batch sizing behavior (20m)
- [ ] **Task 1.3.3:** Verify convergence calculation frequency (10m)
- [ ] **Task 1.3.4:** Document findings in analysis report (10m)

**Deliverables:**
- [ ] `/tmp/ProRT-IP/SPRINT-5.X/PROFILING-REPORT.md` (500 lines, flamegraphs + analysis)
- [ ] `/tmp/ProRT-IP/SPRINT-5.X/baseline.svg` (flamegraph)
- [ ] `/tmp/ProRT-IP/SPRINT-5.X/adaptive.svg` (flamegraph)
- [ ] `/tmp/ProRT-IP/SPRINT-5.X/BATCH-SIZE-ANALYSIS.md` (200 lines, behavior log)

---

## Phase 2: Fix Batch Sizing (If Broken) (2-3h)

**Duration:** 2-3 hours (conditional on Phase 1 findings)
**Status:** Not Started
**Progress:** 0 / 6 (0%)

**Note:** This phase assumes batch sizing is broken. Skip if Phase 1 shows it's working correctly.

### Task 2.1: Verify Batch Calculation (30m)

- [ ] **Task 2.1.1:** Add unit test for batch size convergence (15m)
- [ ] **Task 2.1.2:** Verify batch size reaches expected values at high rates (10m)
- [ ] **Task 2.1.3:** Document expected vs actual batch sizes (5m)

### Task 2.2: Fix Implementation (1.5h)

- [ ] **Task 2.2.1:** Review convergence algorithm in next_batch() method (20m)
- [ ] **Task 2.2.2:** Fix batch size calculation if broken (40m)
- [ ] **Task 2.2.3:** Ensure batch size is actually used in throttling logic (20m)
- [ ] **Task 2.2.4:** Add assertions to verify batch size is applied (10m)

### Task 2.3: Verification (30m)

- [ ] **Task 2.3.1:** Run unit tests (cargo test adaptive_rate_limiter) (10m)
- [ ] **Task 2.3.2:** Run quick benchmark to verify overhead improvement (15m)
- [ ] **Task 2.3.3:** Document changes in change log (5m)

**Deliverables:**
- [ ] `crates/prtip-scanner/src/adaptive_rate_limiter.rs` (fixes applied)
- [ ] `crates/prtip-scanner/tests/test_adaptive_rate_limiter.rs` (+2-3 tests)
- [ ] `/tmp/ProRT-IP/SPRINT-5.X/BATCH-FIX-VERIFICATION.md` (200 lines)

---

## Phase 3: Reduce Convergence Frequency (2-3h)

**Duration:** 2-3 hours
**Status:** Not Started
**Progress:** 0 / 7 (0%)

**Goal:** Reduce convergence calculation overhead by updating less frequently

### Task 3.1: Design Optimization (30m)

- [ ] **Task 3.1.1:** Analyze current convergence frequency (every packet) (10m)
- [ ] **Task 3.1.2:** Design lazy convergence strategy (every 100 packets or 10ms) (15m)
- [ ] **Task 3.1.3:** Document design decision in ADR (Architecture Decision Record) (5m)

### Task 3.2: Implementation (1.5h)

- [ ] **Task 3.2.1:** Add convergence_interval field to AdaptiveRateLimiter struct (10m)
- [ ] **Task 3.2.2:** Implement lazy convergence logic in next_batch() (40m)
- [ ] **Task 3.2.3:** Add configuration option for convergence interval (20m)
- [ ] **Task 3.2.4:** Update documentation and rustdocs (20m)

### Task 3.3: Testing (30m)

- [ ] **Task 3.3.1:** Add unit tests for lazy convergence (15m)
- [ ] **Task 3.3.2:** Verify convergence still works correctly (10m)
- [ ] **Task 3.3.3:** Run performance benchmark to verify improvement (5m)

**Deliverables:**
- [ ] `crates/prtip-scanner/src/adaptive_rate_limiter.rs` (convergence optimization)
- [ ] `crates/prtip-scanner/tests/test_adaptive_rate_limiter.rs` (+2 tests)
- [ ] `/tmp/ProRT-IP/SPRINT-5.X/ADR-LAZY-CONVERGENCE.md` (300 lines, design rationale)

---

## Phase 4: Circular Buffer Optimization (2-3h)

**Duration:** 2-3 hours
**Status:** Not Started
**Progress:** 0 / 6 (0%)

**Goal:** Optimize circular buffer update overhead identified in profiling

### Task 4.1: Profile-Guided Optimization (1h)

- [ ] **Task 4.1.1:** Identify hotspots in circular buffer code from flamegraph (20m)
- [ ] **Task 4.1.2:** Analyze lock contention or atomic operation overhead (20m)
- [ ] **Task 4.1.3:** Design optimization strategy (batched updates, lock-free, etc.) (20m)

### Task 4.2: Implementation (1.5h)

- [ ] **Task 4.2.1:** Apply optimization to circular buffer update logic (50m)
- [ ] **Task 4.2.2:** Ensure correctness with unit tests (30m)
- [ ] **Task 4.2.3:** Benchmark before/after performance (10m)

### Task 4.3: Verification (30m)

- [ ] **Task 4.3.1:** Run full test suite to verify no regressions (15m)
- [ ] **Task 4.3.2:** Re-profile with perf/flamegraph to verify hotspot eliminated (10m)
- [ ] **Task 4.3.3:** Document optimization in change log (5m)

**Deliverables:**
- [ ] `crates/prtip-scanner/src/adaptive_rate_limiter.rs` (buffer optimization)
- [ ] `/tmp/ProRT-IP/SPRINT-5.X/BUFFER-OPTIMIZATION.md` (400 lines, before/after)

---

## Phase 5: Comprehensive Re-Benchmarking (3-4h)

**Duration:** 3-4 hours
**Status:** Not Started
**Progress:** 0 / 7 (0%)

**Goal:** Validate <5% overhead claim after all optimizations

### Task 5.1: Re-run Sprint 5.4 Benchmarks (1.5h)

- [ ] **Task 5.1.1:** Run all 5 benchmark scenarios from Sprint 5.4 Phase 2 (40m)
- [ ] **Task 5.1.2:** Generate comparison tables (before/after optimization) (30m)
- [ ] **Task 5.1.3:** Calculate overhead improvements (20m)

### Task 5.2: Additional Benchmark Scenarios (1h)

- [ ] **Task 5.2.1:** Add very large port range test (10000 ports) (20m)
- [ ] **Task 5.2.2:** Add multi-target scan benchmark (256 targets) (20m)
- [ ] **Task 5.2.3:** Add edge case tests (very low/high rates) (20m)

### Task 5.3: Statistical Validation (1h)

- [ ] **Task 5.3.1:** Run benchmarks with 30 runs (increased from 10) for significance (30m)
- [ ] **Task 5.3.2:** Calculate confidence intervals (95% CI) (20m)
- [ ] **Task 5.3.3:** Create comprehensive benchmark report (10m)

**Deliverables:**
- [ ] `/tmp/ProRT-IP/SPRINT-5.X/BENCHMARK-RESULTS.md` (1000 lines, comprehensive)
- [ ] `benchmarks/04-Sprint5.X-RateLimiting-Optimized/` (new directory with all results)
- [ ] `/tmp/ProRT-IP/SPRINT-5.X/COMPARISON-TABLES.md` (500 lines, before/after)

---

## Phase 6: Documentation & Completion (2-3h)

**Duration:** 2-3 hours
**Status:** Not Started
**Progress:** 0 / 5 (0%)

### Task 6.1: Update Rate Limiting Guide (1h)

- [ ] **Task 6.1.1:** Update Performance Overhead section in 26-RATE-LIMITING-GUIDE.md (30m)
- [ ] **Task 6.1.2:** Add optimization notes and updated recommendations (20m)
- [ ] **Task 6.1.3:** Add benchmark data links to new results (10m)

### Task 6.2: Update Project Documentation (1h)

- [ ] **Task 6.2.1:** Update CHANGELOG.md with Sprint 5.X entry (20m)
- [ ] **Task 6.2.2:** Update README.md with performance metrics (15m)
- [ ] **Task 6.2.3:** Update CLAUDE.local.md with session summary (15m)
- [ ] **Task 6.2.4:** Create SPRINT-5.X-COMPLETE.md completion report (10m)

### Task 6.3: Final Verification (30m)

- [ ] **Task 6.3.1:** Run full test suite (cargo test) (10m)
- [ ] **Task 6.3.2:** Run clippy (cargo clippy) (5m)
- [ ] **Task 6.3.3:** Verify CI/CD passing (15m)

**Deliverables:**
- [ ] `docs/26-RATE-LIMITING-GUIDE.md` (Performance section updated to v1.2.0)
- [ ] `CHANGELOG.md` (+100 lines)
- [ ] `README.md` (+20 lines)
- [ ] `CLAUDE.local.md` (session entry)
- [ ] `/tmp/ProRT-IP/SPRINT-5.X-COMPLETE.md` (2000+ lines, comprehensive report)

---

## Success Criteria

### Performance Requirements (PRIMARY GOAL)

- [ ] **Adaptive rate limiter overhead <5%** on all scenarios (currently 22-40%)
  - [ ] Small scans (18 ports): <3% overhead (currently 1% ✅)
  - [ ] Large scans (1000 ports): <5% overhead (currently 40% ❌)
  - [ ] Very large scans (10000 ports): <5% overhead (new test)
  - [ ] Rate scaling: overhead decreases with higher rates (currently independent ❌)

### Functional Requirements

- [ ] All existing functionality preserved (no regressions)
- [ ] Convergence algorithm still works correctly
- [ ] Batch sizing reaches expected values at high rates
- [ ] System suspend/resume handling still works (1s gap reset)

### Quality Requirements

- [ ] All tests passing: 1,466 tests maintained or increased
- [ ] Zero clippy warnings
- [ ] Coverage ≥62.5% maintained
- [ ] Zero production panics
- [ ] CI/CD 7/7 passing

### Documentation Requirements

- [ ] 26-RATE-LIMITING-GUIDE.md updated with accurate overhead numbers
- [ ] CHANGELOG entry complete with technical details
- [ ] README performance metrics updated
- [ ] Benchmark comparison report comprehensive (before/after)

---

## Files to Create/Modify

### New Files (Temporary Analysis)

1. `/tmp/ProRT-IP/SPRINT-5.X/PROFILING-REPORT.md` (500 lines)
2. `/tmp/ProRT-IP/SPRINT-5.X/baseline.svg` (flamegraph)
3. `/tmp/ProRT-IP/SPRINT-5.X/adaptive.svg` (flamegraph)
4. `/tmp/ProRT-IP/SPRINT-5.X/BATCH-SIZE-ANALYSIS.md` (200 lines)
5. `/tmp/ProRT-IP/SPRINT-5.X/BATCH-FIX-VERIFICATION.md` (200 lines)
6. `/tmp/ProRT-IP/SPRINT-5.X/ADR-LAZY-CONVERGENCE.md` (300 lines)
7. `/tmp/ProRT-IP/SPRINT-5.X/BUFFER-OPTIMIZATION.md` (400 lines)
8. `/tmp/ProRT-IP/SPRINT-5.X/BENCHMARK-RESULTS.md` (1000 lines)
9. `/tmp/ProRT-IP/SPRINT-5.X/COMPARISON-TABLES.md` (500 lines)
10. `/tmp/ProRT-IP/SPRINT-5.X-COMPLETE.md` (2000+ lines)

### New Directories (Permanent Benchmarks)

1. `benchmarks/04-Sprint5.X-RateLimiting-Optimized/` (new directory)
   - `run_benchmarks.sh` (copied from Sprint 5.4, may need updates)
   - `analyze_results.sh` (copied from Sprint 5.4)
   - `README.md` (updated with optimization context)
   - `results/*.json` (hyperfine output)
   - `results/*.md` (markdown tables)

### Modified Files

1. `crates/prtip-scanner/src/adaptive_rate_limiter.rs` (~50-100 lines changed)
   - Batch sizing fixes (if needed)
   - Lazy convergence implementation
   - Circular buffer optimization
2. `crates/prtip-scanner/tests/test_adaptive_rate_limiter.rs` (+6-10 new tests)
3. `docs/26-RATE-LIMITING-GUIDE.md` (~80 lines changed, v1.1.0 → v1.2.0)
4. `CHANGELOG.md` (+100 lines)
5. `README.md` (+20 lines)
6. `CLAUDE.local.md` (session entry)

**Total Temporary Files:** ~3,700 lines (analysis and reports)
**Total Permanent Code Changes:** ~150-200 lines
**Total Documentation Updates:** ~200 lines

---

## Test Count Tracking

| Phase | Phase Tests | Cumulative | Notes |
|-------|-------------|------------|-------|
| Start | - | 1,466 | Sprint 5.4 Phase 2 complete |
| Phase 2 | +2-3 | 1,468-1,469 | Batch sizing tests |
| Phase 3 | +2 | 1,470-1,471 | Lazy convergence tests |
| Phase 6 | +2-3 | 1,472-1,474 | Additional edge cases |
| **Final** | **+6-10** | **1,472-1,476** | **+0.4-0.7% growth** |

---

## Time Budget

| Phase | Estimated | Actual | Variance |
|-------|-----------|--------|----------|
| Phase 1: Profiling | 3-4h | ___ | ___ |
| Phase 2: Batch Sizing | 2-3h | ___ | ___ |
| Phase 3: Convergence | 2-3h | ___ | ___ |
| Phase 4: Buffer | 2-3h | ___ | ___ |
| Phase 5: Benchmarks | 3-4h | ___ | ___ |
| Phase 6: Documentation | 2-3h | ___ | ___ |
| **Base Total** | **14-20h** | **___** | **___** |
| Buffer (0%) | 0h | ___ | ___ |
| **Grand Total** | **15-20h** | **___** | **___** |

**Note:** Lower buffer than usual because:
1. Profiling will reveal exact issues (less uncertainty)
2. Optimizations are targeted (not exploratory)
3. Benchmark infrastructure already exists (Sprint 5.4 Phase 2)

---

## Risk Mitigation

| Risk | Mitigation | Contingency |
|------|------------|-------------|
| Profiling shows no obvious hotspots | Use multiple profiling tools (perf, flamegraph, Instruments on macOS) | Accept current overhead, document trade-offs |
| Batch sizing is already working | Profiling phase will confirm quickly | Skip Phase 2, focus on Phase 3-4 |
| Optimizations don't reach <5% | Incremental improvements still valuable | Accept 5-10% if other benefits exist |
| Optimizations break correctness | Comprehensive test suite will catch | Revert changes, mark as "needs deeper redesign" |
| CI/CD failures | Extensive local testing before commit | Debug specific platform issues |

---

## Dependencies

### Internal Dependencies

- [x] Sprint 5.4 Phase 1: Scanner Integration (COMPLETE ✅)
- [x] Sprint 5.4 Phase 2: Benchmarking (COMPLETE ✅)
  - Status: Completed 2025-11-01
  - Benchmark infrastructure ready for re-use
  - Root causes identified

### External Dependencies

- [ ] `perf` tool (Linux profiling, apt install linux-tools-generic)
- [ ] `flamegraph` crate (cargo install flamegraph)
- [ ] `hyperfine` 1.19.0+ (already available from Sprint 5.4)
- [ ] Debug symbols in release build (Cargo.toml: debug = 1)

### Optional Dependencies

- [ ] Instruments.app (macOS profiling, alternative to perf)
- [ ] `cargo-asm` (view generated assembly, cargo install cargo-asm)
- [ ] `valgrind` (memory profiling, apt install valgrind)

---

## Expected Outcomes

### Best Case Scenario (12-15h)

- Batch sizing was broken, easy fix brings overhead to <3%
- Lazy convergence adds another 1-2% improvement
- Circular buffer optimization negligible
- **Result:** <5% overhead achieved, Sprint 5.X completes under budget

### Realistic Scenario (15-18h)

- Multiple small optimizations compound to 5-7% overhead
- Some scenarios still at 8-10% (but acceptable)
- Most users see <5% overhead on common use cases
- **Result:** "Good enough" performance, document trade-offs

### Worst Case Scenario (18-20h + potential redesign)

- Fundamental design issue requires significant refactor
- Current optimizations insufficient
- Need to consider alternative algorithms (token bucket+batching hybrid?)
- **Result:** Defer to Sprint 5.Y for deeper redesign, accept current overhead

---

## Next Steps After Completion

### If <5% Overhead Achieved ✅

1. Update all documentation to claim "<5% overhead" confidently
2. Mark adaptive rate limiter as "Production-Ready"
3. Consider making it the default rate limiting strategy
4. Write blog post or documentation about optimization journey

### If 5-10% Overhead Achieved ⚠️

1. Update documentation with accurate numbers
2. Recommend hostgroup limiting for performance-critical scans
3. Mark adaptive rate limiter as "Acceptable overhead for network-friendly scanning"
4. Consider adding performance mode that disables convergence

### If >10% Overhead Persists ❌

1. Mark adaptive rate limiter as "Experimental"
2. Recommend disabling for large scans
3. Create Sprint 5.Y for fundamental redesign
4. Consider alternative algorithms (research phase)

---

## Sprint Completion Report Template

**To be filled upon completion:**

```
Sprint 5.X: Adaptive Rate Limiter Optimization - COMPLETE ✅

**Duration:** ___ hours (estimate: 15-20h)
**Completion Date:** 2025-MM-DD
**Tests:** 1,466 → ___ (+___ = +__%)
**Overhead Improvement:** 40% → __% (target: <5%)
**Outcome:** [Best Case / Realistic / Worst Case]

**Key Achievements:**
- Batch sizing: [Fixed / Already Working / N/A]
- Lazy convergence: [Implemented / Skipped / Partial]
- Buffer optimization: [Implemented / Negligible / Skipped]
- Overhead reduction: __% → __% (___% improvement)

**Benchmark Results:**
- Small scans (18 ports): __% overhead (target: <3%)
- Large scans (1000 ports): __% overhead (target: <5%)
- Very large scans (10000 ports): __% overhead (target: <5%)
- Rate scaling: [Improved / Same / Degraded]

**Quality Metrics:**
- Zero clippy warnings: ___
- All tests passing: ___
- CI/CD 7/7 passing: ___
- Zero regressions: ___

**Documentation:**
- 26-RATE-LIMITING-GUIDE.md updated: ___
- CHANGELOG.md entry: ___
- Benchmark comparison report: ___

**Next Sprint:** Sprint 5.5 or Sprint 5.Y (if redesign needed)
```

---

## Quick Reference

**Planning Document:** `to-dos/SPRINT-5.X-TODO.md` (this file)
**Benchmark Infrastructure:** `benchmarks/03-Sprint5.4-RateLimiting/` (Phase 2 baseline)
**Analysis Document:** `benchmarks/03-Sprint5.4-RateLimiting/SPRINT-5.4-PHASE-2-ANALYSIS.md`
**Target File:** `crates/prtip-scanner/src/adaptive_rate_limiter.rs` (706 lines)

**Current Status:** Planned (Not Started)
**Target Completion:** TBD (Deferred from Sprint 5.4)
**Estimated Effort:** 15-20 hours
**Priority:** MEDIUM (Performance optimization, not blocking)

---

**Last Updated:** 2025-11-01
**Status:** Ready to Execute (After Sprint 5.4 documentation update) ✅
**Blocker:** None (Sprint 5.4 Phase 2 complete, benchmark infrastructure ready)
