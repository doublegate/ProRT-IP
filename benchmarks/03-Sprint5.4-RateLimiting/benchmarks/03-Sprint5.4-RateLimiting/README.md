# Sprint 5.4 Phase 2: Rate Limiting Benchmarking

**Sprint:** 5.4 (Phase 2)  
**Objective:** Validate <5% overhead claim for three-layer rate limiting system  
**Date:** 2025-11-01

## Overview

This benchmark suite validates the performance overhead of ProRT-IP's three-layer rate limiting architecture:

1. **Layer 1: ICMP Type 3 Code 13 Detection** - Background ICMP monitoring with per-target backoff
2. **Layer 2: Hostgroup Limiting** - Semaphore-based concurrent target control
3. **Layer 3: Adaptive Rate Limiting** - Masscan-inspired circular buffer with convergence

**Success Criterion:** All rate limiting features must introduce <5% overhead compared to baseline scans.

## Test Suite

### Test 1: Common Ports (Top 100)
- **Scenario:** Single target, 18 common ports (SSH, HTTP, HTTPS, SMB, MySQL, etc.)
- **Purpose:** Lightweight scan, typical security audit use case
- **Variants:** Baseline, Layer 1, Layer 2, Layer 3, Combined

### Test 2: Large Port Range (1-1000)
- **Scenario:** Single target, 1,000 ports
- **Purpose:** Medium-scale scan, broader discovery phase
- **Variants:** Baseline, Layer 1, Layer 2, Layer 3, Combined

### Test 3: Hostgroup Size Impact
- **Scenario:** Single target, 1,000 ports, varying --max-hostgroup
- **Purpose:** Measure hostgroup limiter overhead across sizes (1, 8, 32, 64, 128)
- **Variants:** 6 hostgroup sizes

### Test 4: Multiple Targets with Hostgroup
- **Scenario:** 8 targets (127.0.0.1-127.0.0.8), Fast scan (-F, top 100 ports)
- **Purpose:** Validate hostgroup limiter with concurrent targets
- **Variants:** Serial (1), partial parallel (2, 4), full parallel (8, 64)
- **Note:** Skipped in --quick mode

### Test 5: Adaptive Rate Limiter Impact
- **Scenario:** Single target, 1,000 ports, varying --max-rate
- **Purpose:** Measure adaptive rate limiter overhead across rates (10K, 50K, 100K, 500K, 1M pps)
- **Variants:** 6 rate limits

## Usage

### Prerequisites

1. **Install hyperfine:**
   ```bash
   cargo install hyperfine
   ```

2. **Build release binary:**
   ```bash
   cargo build --release
   ```

3. **Verify jq installed** (for analysis):
   ```bash
   sudo dnf install jq  # Fedora/RHEL
   ```

### Running Benchmarks

**Full benchmark suite (10 runs per test, ~20-30 minutes):**
```bash
cd benchmarks/03-Sprint5.4-RateLimiting
./run_benchmarks.sh
```

**Quick mode (5 runs, skip multi-target test, ~10-15 minutes):**
```bash
./run_benchmarks.sh --quick
```

### Analyzing Results

```bash
./analyze_results.sh ./results
```

The analysis script will:
- Calculate overhead percentage for each variant vs baseline
- Color-code results (green <3%, yellow 3-5%, red >5%)
- Generate pass/fail summary
- Exit code 0 if all tests pass (<5%), 1 if any fail

### Output Files

Results are saved in `./results/` with timestamps:

- `01_common_ports_YYYYMMDD_HHMMSS.json` - Hyperfine JSON output
- `01_common_ports_YYYYMMDD_HHMMSS.md` - Markdown table
- `02_large_range_YYYYMMDD_HHMMSS.json` - Large port range
- `03_hostgroup_single_YYYYMMDD_HHMMSS.json` - Hostgroup sizes
- `04_hostgroup_multi_YYYYMMDD_HHMMSS.json` - Multiple targets (full mode only)
- `05_rate_impact_YYYYMMDD_HHMMSS.json` - Rate limiter impact

## Expected Results

Based on Sprint 5.4 Phase 1 implementation analysis:

### Layer 1 (ICMP Monitor)
- **Overhead:** <0.5%
- **Rationale:** Background task, minimal contention (DashMap lock-free reads)

### Layer 2 (Hostgroup Limiter)
- **Overhead:** <1% (single target), 0-2% (multiple targets)
- **Rationale:** Single semaphore acquire/release per target (RAII pattern)
- **Note:** Single target has minimal benefit (permit always available)

### Layer 3 (Adaptive Rate Limiter)
- **Overhead:** <2% at high rates (>100K pps), 1-3% at low rates (<10K pps)
- **Rationale:** Circular buffer updates, batch size convergence

### Combined (All 3 Layers)
- **Overhead:** <3% (additive effect, but benefits from reduced syscall overhead)
- **Target:** Well under 5% threshold

## Benchmark Design Rationale

### Loopback Target (127.0.0.1)
- Eliminates network variability
- Consistent RTT (~0.1ms)
- Reproducible results
- Safe (no external traffic)

### Warmup Runs (3)
- JIT compilation stabilization
- CPU cache warming
- OS scheduler stabilization

### Benchmark Runs (10 full, 5 quick)
- Statistical significance
- Outlier detection via stddev
- Confidence intervals

### No Service Detection
- Isolates rate limiting overhead
- Removes service probe variability
- Focuses on scan engine performance

### Disabled Host Discovery (-Pn)
- Skips ICMP/ARP discovery phase
- Pure port scanning performance
- Consistent test conditions

## Interpreting Results

### Green Results (<3% overhead)
Excellent performance, rate limiting adds minimal overhead. Expected for:
- Layer 1 (ICMP monitor)
- Layer 2 with single target
- Layer 3 at very high rates (>500K pps)

### Yellow Results (3-5% overhead)
Acceptable performance, within threshold. May occur with:
- Combined layers (additive effect)
- Layer 3 at low rates (<10K pps, higher per-packet overhead)

### Red Results (>5% overhead)
**Investigation required.** Possible causes:
- Lock contention (check DashMap usage)
- Syscall overhead (adaptive rate limiter batch sizing)
- Semaphore contention (hostgroup limiter)
- Test environment issues (CPU throttling, background load)

## Troubleshooting

### hyperfine not found
```bash
cargo install hyperfine
```

### jq not found
```bash
sudo dnf install jq  # Fedora/RHEL
sudo apt install jq  # Debian/Ubuntu
brew install jq      # macOS
```

### prtip binary not found
```bash
cargo build --release
```

### High variability (large stddev)
- Close background applications
- Disable CPU power management
- Run multiple times and average

### Unexpected overhead
1. Check system load: `top` or `htop`
2. Verify no other network scanners running
3. Re-run with `--warmup 5` for more stabilization
4. Check for CPU throttling: `cpupower frequency-info`

## References

- **Sprint 5.4 Phase 1:** Scanner integration (crates/prtip-scanner/src/)
- **Rate Limiting Guide:** docs/26-RATE-LIMITING-GUIDE.md
- **Architecture:** docs/00-ARCHITECTURE.md (Section 7.0)
- **Hyperfine:** https://github.com/sharkdp/hyperfine

---

**Next Steps After Benchmarking:**
1. Run `./analyze_results.sh` to validate <5% overhead
2. Document results in Sprint 5.4 completion report
3. Update 26-RATE-LIMITING-GUIDE.md with performance data
4. Update CHANGELOG.md with Phase 2 completion
