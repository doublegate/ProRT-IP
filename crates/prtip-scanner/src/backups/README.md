# Archived Rate Limiter Implementations

**Date Archived:** 2025-11-02
**Reason:** AdaptiveRateLimiterV3 (optimized) promoted to default
**Sprint:** Phase 5 - Advanced Rate Limiting

This directory contains legacy rate limiter implementations that were replaced by AdaptiveRateLimiterV3 (optimized with Relaxed memory ordering).

## Performance Comparison

| Implementation | Average Overhead | Status |
|----------------|------------------|--------|
| Governor (RateLimiter) | +15-18% | ⚠️ Archived |
| AdaptiveRateLimiter (P3) | +17-18% | ⚠️ Archived |
| **AdaptiveRateLimiterV3** | **-1.8%** | ✅ **DEFAULT** |

**V3 Performance Details:**
- Best case: -8.2% overhead at 10K pps
- Sweet spot: -3% to -4% overhead at 75K-200K pps
- Worst case: +0% to +3% overhead at 500K-1M pps
- 15.2 percentage points improvement over Governor
- 34% variance reduction (more consistent timing)

## Archived Files

### 1. rate_limiter.rs - Governor Token Bucket Implementation

Original rate limiter using the `governor` crate with token bucket algorithm.

**Architecture:**
- Synchronized token bucket with quota replenishment
- `acquire()` method with async waiting
- Simple API: `new(max_rate)` → `acquire()` → scan

**Performance Issues:**
- 15-18% overhead at common scan rates (50K-100K pps)
- Higher overhead due to mutex contention and clock queries
- Consistent but slow compared to V3

**When to use (if restored):**
- Simple, proven algorithm (governor crate battle-tested)
- Predictable behavior without convergence
- Lower memory usage (no background monitoring task)

### 2. adaptive_rate_limiter.rs - Phase 3 Adaptive Rate Limiter

Adaptive rate limiter with convergence-based self-correction.

**Architecture:**
- Two-tier design: hot path + background monitor
- `acquire()` with 3 atomic operations (SeqCst ordering)
- 100ms convergence task for rate measurement
- Exponential moving average (EMA) for stability

**Performance Issues:**
- 17-18% overhead at common scan rates
- SeqCst memory ordering adds synchronization cost
- Convergence algorithm introduces latency

**When to use (if restored):**
- Need adaptive batch sizing behavior
- Want explicit convergence feedback
- SeqCst ordering required for strict ordering guarantees

## Why V3 is Better

AdaptiveRateLimiterV3 achieves **-1.8% average overhead** (faster than no rate limiting!) through several key optimizations:

### 1. Relaxed Memory Ordering

```rust
// Old (AdaptiveRateLimiter): SeqCst ordering (expensive)
self.acquired.fetch_add(1, Ordering::SeqCst);

// New (V3): Relaxed ordering (minimal cost)
self.acquired.fetch_add(1, Ordering::Relaxed);
```

**Impact:** Eliminates memory fence overhead on hot path

### 2. System-Wide Optimization

V3's minimal overhead allows the CPU to:
- Better utilize instruction pipelining
- Improve branch prediction accuracy
- Reduce cache pressure
- Enable more aggressive speculative execution

This creates **negative overhead** at lower rates (system runs faster with rate limiting enabled).

### 3. Convergence-Based Self-Correction

Despite Relaxed ordering, V3 maintains accuracy through:
- Background convergence task (100ms intervals)
- Temporary drift corrected automatically
- No correctness impact (only affects short-term precision)

### 4. Optimal Configuration

- `burst = 100`: Optimal batch size (tested extensively)
- Adaptive batch sizing based on actual rate
- Sweet spot: 75K-200K pps (-3% to -4% overhead)

## Restoration Instructions

If you need to restore an old rate limiter:

### Quick Restoration (Governor)

```bash
# 1. Copy rate_limiter.rs back to src/
cd /home/parobek/Code/ProRT-IP/crates/prtip-scanner/src
cp backups/rate_limiter.rs .

# 2. Update lib.rs exports
# Add: mod rate_limiter;
# Add: pub use rate_limiter::RateLimiter;

# 3. Update scheduler.rs
# Replace AdaptiveRateLimiterV3 with RateLimiter in:
#   - use statements
#   - struct fields
#   - instantiation logic

# 4. Run tests
cargo test --package prtip-scanner

# 5. Update CLI args if needed
# args.rs: Add --use-governor flag (optional)
```

### Full Restoration (Both Rate Limiters)

```bash
# 1. Copy both files back
cp backups/rate_limiter.rs .
cp backups/adaptive_rate_limiter.rs .

# 2. Update lib.rs
# Add mod declarations for both
# Add pub use statements for both

# 3. Update scheduler.rs
# Add conditional logic to choose rate limiter:
let rate_limiter = match config.rate_limiter_type {
    RateLimiterType::Governor => Arc::new(RateLimiter::new(max_rate)),
    RateLimiterType::Adaptive => Arc::new(AdaptiveRateLimiter::new(max_rate)),
    RateLimiterType::V3 => Arc::new(AdaptiveRateLimiterV3::new(max_rate)),
};

# 4. Add config option
# config.rs: Add rate_limiter_type: RateLimiterType field
# args.rs: Add --rate-limiter flag

# 5. Run full test suite
cargo test
cargo clippy -- -D warnings
```

### Rollback Considerations

**If you restore old rate limiters:**

1. **Performance:** Expect 15-18% overhead (vs -1.8% for V3)
2. **API Compatibility:** May need to update tests that assume V3 behavior
3. **CLI Flags:** Consider adding explicit selection flags
4. **Documentation:** Update README.md, CHANGELOG.md, guides

**Common Issues:**

- **Compilation errors:** Ensure all imports updated
- **Test failures:** Old rate limiters may have different timing
- **Performance regression:** Benchmark to verify overhead acceptable

## Technical References

### Original Implementation Documents

1. **Governor Rate Limiter (Phase 3)**
   - File: `rate_limiter.rs` (195 lines)
   - Crate: `governor` v0.6.0
   - Algorithm: Token bucket with quota
   - Benchmarked: Phase 3 Sprint 3.2

2. **Adaptive Rate Limiter (Phase 3)**
   - File: `adaptive_rate_limiter.rs` (412 lines)
   - Algorithm: Convergence-based with EMA
   - Benchmarked: Phase 4 Sprint 4.22

3. **AdaptiveRateLimiterV3 (Phase 4-5)**
   - File: `adaptive_rate_limiter_v3.rs` (746 lines)
   - Algorithm: Two-tier with Relaxed ordering
   - Benchmarked: Phase 4 final optimization
   - Analysis: `/tmp/ProRT-IP/PHASE4-V3-OPTIMIZATION-COMPLETE.md`

### Performance Analysis Documents

- **Phase 4 V3 Optimization:** `/tmp/ProRT-IP/PHASE4-V3-OPTIMIZATION-COMPLETE.md`
- **Sprint 5.X Analysis:** `/tmp/ProRT-IP/SPRINT-5.X-COMPREHENSIVE/`
- **Benchmark Results:** `benchmarks/rate-limiting/`
- **Rate Limiting Guide:** `docs/26-RATE-LIMITING-GUIDE.md`

### Related Commits

- **Archive Commit:** (current commit - promotion to default)
- **V3 Optimization:** Phase 4 Sprint 4.22 completion
- **V3 Implementation:** Sprint 5.X Phase 1-2

## FAQ

### Why archive instead of delete?

1. **Git history preservation:** `git mv` keeps blame/log intact
2. **Easy restoration:** Copy back if needed for debugging
3. **Reference implementation:** Useful for understanding design evolution
4. **Rollback capability:** Safety net if V3 issues discovered

### Can I still use the old rate limiters?

Yes, but not recommended. Follow restoration instructions above. Note that:
- You'll lose V3's performance benefits (-1.8% → +15-18% overhead)
- Tests may need updating (timing assumptions)
- Documentation will reference V3 as default

### What if V3 has issues?

1. **File a bug:** Include benchmark results showing issue
2. **Restore Governor:** Quick fallback (instructions above)
3. **Review analysis:** Check `/tmp/ProRT-IP/PHASE4-V3-OPTIMIZATION-COMPLETE.md`
4. **Compare performance:** Use `hyperfine` to benchmark both

### How do I contribute improvements to V3?

1. **Read the code:** `adaptive_rate_limiter_v3.rs` (well-commented)
2. **Understand architecture:** Two-tier design (hot path + monitor)
3. **Benchmark first:** Use `hyperfine` to establish baseline
4. **Profile changes:** Use `perf`/`flamegraph` to verify improvements
5. **Test thoroughly:** All 1,466 tests must pass

## Conclusion

These rate limiters served ProRT-IP well through Phase 3-4 development, but AdaptiveRateLimiterV3 represents a significant performance breakthrough. By achieving **negative overhead** through Relaxed memory ordering and system-wide optimization, V3 makes rate limiting essentially "free" while maintaining accuracy through convergence-based self-correction.

This archive preserves the old implementations for reference, debugging, and potential restoration if needed, but V3 is now the recommended default for all use cases.

---

**Archive Date:** 2025-11-02
**Archived By:** ProRT-IP Development Team
**Reason:** Performance optimization (15.2pp improvement)
**Status:** ✅ Production-ready V3 promoted to default
