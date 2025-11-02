# Advanced Rate Limiting Guide

**Author:** ProRT-IP Development Team
**Version:** 1.1.0
**Sprint:** 5.4 (Phase 1-2)
**Date:** 2025-11-01 (Updated with Phase 2 benchmarks)

## Overview

ProRT-IP implements three-layer rate limiting for precise control over scan speed, network load, and target-level parallelism:

1. **Simple Rate Limiter** - Token bucket (packets/sec), lightweight
2. **Adaptive Rate Limiter** - Masscan-inspired circular buffer with dynamic batch sizing
3. **Hostgroup Limiter** - Concurrent target control (Nmap-compatible)

This guide focuses on **Layers 2 and 3** (Adaptive + Hostgroup), which provide advanced features for high-performance scanning at scale.

---

## Adaptive Rate Limiter

Inspired by Masscan's throttler, uses a 256-bucket circular buffer to track recent packet transmission rates and dynamically adjust batch sizes.

### Algorithm

**Core Principles:**
- Track packet counts over sliding 256-bucket window
- Adjust batch size via convergence: `batch *= (target/observed)^0.5`
- Range: 1.0 → 10,000.0 packets/batch
- Reset on >1s gap (gracefully handles laptop suspend/resume)

**Performance Characteristics:**
- **Low rates** (<1K pps): batch size ~1, per-packet throttling
- **Medium rates** (1K-100K pps): batch size 2-100, reduced syscall overhead
- **High rates** (>100K pps): batch size 100-10000, minimal overhead

**Advantages over Token Bucket:**
- Better performance at very high rates (>100K pps)
- Adaptive batching reduces syscall overhead
- Handles system suspend/resume without burst
- Uses only recent history (no stale rate data)

### Usage Examples

```bash
# Basic rate-limited scan
prtip -sS -p 80-443 --max-rate 100000 192.168.1.0/24

# High-performance scan (1M pps)
prtip -sS -p 1-1000 --max-rate 1000000 10.0.0.0/16
```

### ICMP Monitoring (Optional)

Detects **ICMP Type 3 Code 13** (Communication Administratively Prohibited) and automatically backs off on affected targets.

**Backoff Strategy:**
- Level 0: Initial state (no backoff)
- Level 1: 2 seconds
- Level 2: 4 seconds
- Level 3: 8 seconds
- Level 4: 16 seconds (maximum)

**Activation:**
```bash
# Enable ICMP monitoring for adaptive backoff
prtip -sS -p 1-1000 --adaptive-rate 192.168.1.0/24
```

**How It Works:**
1. Background task listens for ICMP packets via raw socket
2. Filters for Type 3 Code 13 errors
3. Per-target exponential backoff (tracked in DashMap)
4. Scanner waits for backoff expiration before resuming

**Platform Support:**
- **Linux/macOS**: Full support with `next_with_timeout()` (100ms)
- **Windows**: Graceful degradation (ICMP monitor inactive, no backoff)

---

## Hostgroup Control

Limits concurrent targets being scanned simultaneously (Nmap-compatible via `--max-hostgroup` and `--min-hostgroup`).

### Design

**Implementation:**
- Uses `tokio::sync::Semaphore` for async concurrency control
- RAII pattern: `TargetPermit` auto-releases on drop
- Performance warning if active targets < `min_hostgroup`

**Flags:**
- `--max-hostgroup <N>`: Maximum concurrent targets (default: 64)
- `--min-hostgroup <N>`: Minimum concurrent targets (default: 1)
- `--max-parallelism <N>`: Alias for `--max-hostgroup` (Nmap compat)

### Usage Examples

```bash
# Network-friendly scanning (16 hosts max)
prtip -sS -p- --max-hostgroup 16 10.0.0.0/24

# Aggressive scanning (128 hosts)
prtip -sS -p 80,443 --max-hostgroup 128 targets.txt

# With minimum parallelism enforcement
prtip -sS -p 1-1000 --min-hostgroup 8 --max-hostgroup 64 10.0.0.0/16
```

### Tuning Guidelines

**Low Values (1-16):**
- **Pros**: Minimal network impact, IDS/IPS-friendly
- **Cons**: Slower scan completion
- **Use Case**: Sensitive environments, rate-limited networks

**Medium Values (32-128):**
- **Pros**: Balanced speed and network load
- **Cons**: May trigger some IDS alerts
- **Use Case**: General-purpose scanning, default ProRT-IP

**High Values (256-1024):**
- **Pros**: Maximum scan speed
- **Cons**: High network load, likely IDS detection
- **Use Case**: Internal networks, penetration tests with approval

---

## Combined Usage

Stack all three layers for maximum control:

```bash
# Full rate limiting stack: 50K pps, adaptive backoff, 32 hosts max
prtip -sS -p- \
  --max-rate 50000 \
  --adaptive-rate \
  --max-hostgroup 32 \
  --min-hostgroup 8 \
  10.0.0.0/16
```

---

## Performance Overhead

**Status:** ⚠️ **Optimization Needed** - Benchmarked in Sprint 5.4 Phase 2 (2025-11-01)

### Measured Results (ProRT-IP v0.4.3, hyperfine 1.19.0)

**Small Scans (Common Ports, 18 ports):**
- Baseline: 5.55 ms
- Hostgroup limiter: +1% overhead (5.62 ms) ✅
- Adaptive rate limiter: +1% overhead (5.62 ms) ✅
- ICMP monitor: +6% overhead (5.89 ms) ⚠️
- Combined (all 3 layers): +6% overhead (5.92 ms) ⚠️

**Large Scans (1-1000 ports):**
- Baseline: 6.57 ms
- Hostgroup limiter: +9% overhead (7.21 ms) ⚠️
- Adaptive rate limiter: **+40% overhead (9.23 ms)** ❌
- ICMP monitor: +4% overhead (6.86 ms) ✅
- Combined (all 3 layers): **+42% overhead (9.35 ms)** ❌

**Rate Limiter Scaling (1-1000 ports, varying rates):**
- 10K pps: +28% overhead ❌
- 50K pps: +22% overhead ❌
- 100K pps: +26% overhead ❌
- 500K pps: +36% overhead ❌
- 1M pps: +26% overhead ❌

### Performance Analysis

**✅ Production-Ready:**
- **Hostgroup limiter**: 1-9% overhead (acceptable, sometimes faster than baseline)
- **ICMP monitor (small scans)**: 4-6% overhead (acceptable for adaptive backoff)

**⚠️ Optimization Needed:**
- **Adaptive rate limiter**: 22-40% overhead on large port ranges
  - **Root Cause**: Batch sizing may not be working as designed, convergence calculations too frequent
  - **Impact**: 1.4x slower scans (6.57ms → 9.23ms)
  - **Future Work**: Sprint 5.X optimization planned (15-20 hours)

**❌ Not Recommended for Performance-Critical Scans:**
- Combined layers on large port ranges (42% overhead)
- Adaptive rate limiting without optimization

### Recommendations

**For Best Performance:**
```bash
# Small scans (fast, <10% overhead)
prtip -sS -p 80,443,8080 --max-hostgroup 64 10.0.0.0/24

# Large scans without rate limiting (fastest)
prtip -sS -p- 192.168.1.0/24
```

**For Network-Friendly Scanning:**
```bash
# Use hostgroup limiting only (low overhead)
prtip -sS -p- --max-hostgroup 16 10.0.0.0/24

# ICMP monitoring for adaptive backoff (moderate overhead)
prtip -sS -p 1-1000 --adaptive-rate 192.168.1.0/24
```

**Avoid Until Optimization:**
```bash
# High overhead (40%+) - avoid for large port ranges
prtip -sS -p- --max-rate 100000 10.0.0.0/24
```

### Future Optimization (Sprint 5.X)

Planned improvements to adaptive rate limiter:
1. Verify batch sizing implementation
2. Reduce convergence calculation frequency
3. Profile circular buffer overhead with perf/flamegraph
4. **Target**: <5% overhead across all scenarios
5. **Estimated Effort**: 15-20 hours

**Status**: Deferred to future sprint (optimization separate from integration)

---

## Nmap Compatibility

| Flag | Nmap | ProRT-IP | Status | Notes |
|------|------|----------|--------|-------|
| `--max-hostgroup <N>` | ✅ | ✅ | 100% | Identical semantics |
| `--min-hostgroup <N>` | ✅ | ✅ | 100% | Identical semantics |
| `--max-parallelism <N>` | ✅ | ✅ | 100% | Alias for --max-hostgroup |
| `--max-rate <N>` | ✅ | ✅ | Enhanced | Adaptive algorithm (Masscan-inspired) |
| `--min-rate <N>` | ✅ | ✅ | 100% | Existing functionality |
| ICMP backoff | ❌ | ✅ | ProRT-IP exclusive | Automatic IDS/IPS avoidance |

---

## Troubleshooting

### Issue: "ICMP monitor already running"

**Cause:** Attempted to start ICMP monitor multiple times
**Fix:** Restart application (only one monitor per process)

### Issue: Slow convergence to target rate

**Cause:** Network conditions or insufficient target rate
**Fix:** Increase `--max-rate` value or check network bottlenecks

### Issue: "No targets scanned (all backed off)"

**Cause:** All targets blocked with ICMP errors
**Fix:** Targets likely have strict rate limiting. Disable `--adaptive-rate` or reduce `--max-rate`

### Issue: "Active targets below min_hostgroup" warnings

**Cause:** Not enough targets provided or slow scan progress
**Fix:** Increase number of targets or reduce `--min-hostgroup`

---

## Implementation Details

### Adaptive Rate Limiter

**File:** `crates/prtip-scanner/src/adaptive_rate_limiter.rs` (706 lines)
**Key Components:**
- `AdaptiveRateLimiter` struct with 256-bucket circular buffer
- `next_batch()` method: core throttling with wait logic
- Convergence factors: 1.005× increase, 0.999× decrease
- Max batch size: 10,000 (prevents buffer overwhelming)

**Integration:**
```rust
let mut limiter = AdaptiveRateLimiter::new(100_000.0); // 100K pps
limiter.enable_icmp_monitoring().await?; // Optional

let mut packets_sent = 0;
loop {
    let batch = limiter.next_batch(packets_sent).await?;
    // Send 'batch' packets
    packets_sent += batch;
}
```

### ICMP Monitor

**File:** `crates/prtip-scanner/src/icmp_monitor.rs` (490 lines)
**Key Components:**
- `IcmpMonitor`: Background listener (spawn_blocking + pnet)
- `BackoffState`: Per-target exponential backoff tracking
- `IcmpError`: Notification struct (broadcast channel)

**Integration:**
```rust
let monitor = IcmpMonitor::new()?;
monitor.start().await?;

let mut rx = monitor.subscribe();
while let Ok(error) = rx.recv().await {
    // Handle ICMP Type 3 Code 13 errors
}
```

### Hostgroup Limiter

**File:** `crates/prtip-scanner/src/hostgroup_limiter.rs` (399 lines)
**Key Components:**
- `HostgroupLimiter`: Semaphore-based concurrency control
- `TargetPermit`: RAII wrapper (auto-release on drop)
- `HostgroupConfig`: Max/min settings

**Integration:**
```rust
let limiter = HostgroupLimiter::with_max(64);

for target in targets {
    let _permit = limiter.acquire_target().await;
    // Scan target (permit auto-released when dropped)
}
```

---

## References

- **Masscan Throttler:** Robert Graham's algorithm (circular buffer approach)
- **Nmap Hostgroup:** https://nmap.org/book/performance-max-parallelism.html
- **ICMP RFC:** RFC 792 (ICMPv4), RFC 4443 (ICMPv6)
- **ProRT-IP Docs:** `docs/00-ARCHITECTURE.md`, `docs/01-ROADMAP.md`

---

## Benchmark Data

Detailed benchmark results available in:
- `benchmarks/03-Sprint5.4-RateLimiting/SPRINT-5.4-PHASE-2-ANALYSIS.md` - Comprehensive analysis
- `benchmarks/03-Sprint5.4-RateLimiting/results/` - Raw hyperfine JSON/Markdown output
- `benchmarks/03-Sprint5.4-RateLimiting/README.md` - Benchmark suite documentation

---

**Sprint 5.4 Deliverable** | **Phase 1-2 Complete** | **Optimization Planned** ⚠️
