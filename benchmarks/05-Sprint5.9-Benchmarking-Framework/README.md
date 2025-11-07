# ProRT-IP Benchmarking Framework

**Sprint:** 5.9 - Benchmarking Framework
**Version:** v1.0.0
**Created:** 2025-11-06

## Overview

Comprehensive performance benchmarking infrastructure for ProRT-IP network scanner. Provides regression detection, performance validation, and baseline tracking using hyperfine statistical benchmarking tool.

## Quick Start

### Prerequisites

```bash
# Install hyperfine (if not already installed)
cargo install hyperfine

# Build ProRT-IP release binary
cd /home/parobek/Code/ProRT-IP
cargo build --release
```

### Run All Benchmarks

```bash
# Run complete benchmark suite (8 scenarios, ~5-10 minutes)
./scripts/run-all-benchmarks.sh

# Establish new baseline (for releases)
./scripts/run-all-benchmarks.sh --baseline

# Compare against specific baseline
./scripts/run-all-benchmarks.sh --compare baselines/baseline-v0.4.8.json
```

### Run Individual Scenarios

```bash
# Scenario 1: SYN Scan (1,000 ports)
./scripts/01-syn-scan-1000-ports.sh

# Scenario 2: Connect Scan (3 common ports)
./scripts/02-connect-scan-common-ports.sh

# Scenario 3: UDP Scan (3 UDP services)
./scripts/03-udp-scan-dns-snmp-ntp.sh

# ... (see scripts/ directory for all 8 scenarios)
```

## Directory Structure

```
05-Sprint5.9-Benchmarking-Framework/
├── README.md                   # This file
├── scripts/                    # Benchmark runner scripts
│   ├── 01-syn-scan-1000-ports.sh
│   ├── 02-connect-scan-common-ports.sh
│   ├── 03-udp-scan-dns-snmp-ntp.sh
│   ├── 04-service-detection-overhead.sh
│   ├── 05-ipv6-overhead.sh
│   ├── 06-idle-scan-timing.sh
│   ├── 07-rate-limiting-overhead.sh
│   ├── 08-tls-cert-parsing.sh
│   ├── run-all-benchmarks.sh   # Orchestrator (runs all 8)
│   ├── analyze-results.sh      # Regression detection
│   └── comparison-report.sh    # Markdown report generation
├── configs/                    # Hyperfine JSON configs (optional)
├── baselines/                  # Baseline results (versioned)
│   ├── baseline-v0.4.8.json
│   └── baseline-v0.4.8-metadata.md
├── results/                    # Current run results (date-stamped)
│   └── YYYY-MM-DD/
└── reports/                    # Markdown analysis reports
```

## Benchmark Scenarios

### 1. SYN Scan (1,000 ports)

**Purpose:** Validate throughput ("10M+ pps" claim)

**Command:**
```bash
prtip -sS -p 1-1000 127.0.0.1 --rate-limit 0
```

**Metric:** Scan duration (lower = better)
**Target:** <100ms for 1,000 ports on localhost

### 2. Connect Scan (3 common ports)

**Purpose:** Real-world baseline (most common usage)

**Command:**
```bash
prtip -sT -p 80,443,8080 127.0.0.1
```

**Metric:** Scan duration
**Target:** <50ms
**Comparison:** vs Nmap -sT (should be faster)

### 3. UDP Scan (3 UDP services)

**Purpose:** Slow protocol validation (10-100x slower than TCP)

**Command:**
```bash
prtip -sU -p 53,161,123 127.0.0.1
```

**Metric:** Scan duration
**Target:** <500ms (UDP is slow due to ICMP rate limiting)

### 4. Service Detection Overhead

**Purpose:** Validate 85-90% accuracy + low overhead

**Commands:**
- Baseline: `prtip -sS -p 22,80,443 127.0.0.1`
- Detection: `prtip -sV -p 22,80,443 127.0.0.1`

**Metric:** Overhead = (detection_time - baseline_time) / baseline_time * 100
**Target:** <10% overhead

### 5. IPv6 Overhead

**Purpose:** Validate Sprint 5.1 IPv6 claim (15% overhead)

**Commands:**
- IPv4: `prtip -4 -sS -p 1-1000 127.0.0.1`
- IPv6: `prtip -6 -sS -p 1-1000 ::1`

**Metric:** IPv6 overhead vs IPv4 baseline
**Target:** <15% slower than IPv4

### 6. Idle Scan Timing

**Purpose:** Validate Sprint 5.3 idle scan claim (500-800ms per port)

**Command:**
```bash
prtip -sI <zombie-ip> -p 80,443,8080 <target>
```

**Metric:** Time per port scanned
**Target:** 500-800ms per port
**Note:** Requires zombie host setup (see ZOMBIE-SETUP.md)

### 7. Rate Limiting Overhead

**Purpose:** Validate Sprint 5.X AdaptiveRateLimiterV3 (-1.8% overhead)

**Commands:**
- No limit: `prtip -sS -p 1-1000 127.0.0.1 --rate-limit 0`
- V3 limiter: `prtip -sS -p 1-1000 127.0.0.1 --rate-limit 10000`

**Metric:** Overhead = (limited_time - baseline_time) / baseline_time * 100
**Target:** <5% (claimed -1.8%)

### 8. TLS Certificate Parsing

**Purpose:** Validate Sprint 5.5 TLS parsing claim (1.33μs)

**Command:**
```bash
prtip -sV -p 443 badssl.com --tls-cert-analysis
```

**Metric:** Average time per certificate parsed
**Target:** ~1.33μs
**Note:** Requires network access to badssl.com

## Regression Detection

### Thresholds

- **PASS:** <5% slower (within noise)
- **WARN:** 5-10% slower (investigate)
- **FAIL:** >10% slower (regression, CI fails)
- **IMPROVED:** Faster than baseline (celebrate!)

### Statistical Significance

- **Test:** Two-sample t-test (scipy.stats.ttest_ind)
- **Threshold:** p < 0.05 (95% confidence)
- **Interpretation:** p < 0.05 AND diff >5% → regression

### Example

```bash
# Compare current run against v0.4.8 baseline
./scripts/analyze-results.sh \
  baselines/baseline-v0.4.8.json \
  results/2025-11-06/current.json
```

**Output:**
```
| Scenario | Baseline | Current | Diff | Status |
|----------|----------|---------|------|--------|
| SYN Scan | 98ms | 95ms | -3.1% | ✅ IMPROVED |
| Connect  | 45ms | 46ms | +2.2% | ✅ PASS |
| UDP      | 520ms | 540ms | +3.8% | ✅ PASS |
| Service  | 55ms | 62ms | +12.7% | ❌ REGRESSION |

Overall: 1 REGRESSION detected
Exit code: 2 (CI fails)
```

## CI Integration

### GitHub Actions

Benchmark workflow runs on:
- Push to main (after test workflow passes)
- Pull requests (performance validation)
- Weekly schedule (Monday 00:00 UTC)
- Manual trigger (workflow_dispatch)

### Workflow Steps

1. Build release binary (`cargo build --release`)
2. Install hyperfine (`cargo install hyperfine` or cached)
3. Run benchmark suite (`./scripts/run-all-benchmarks.sh`)
4. Compare against baseline (`./scripts/analyze-results.sh`)
5. Upload results as artifacts (7-day retention)
6. Comment on PR with summary (if PR context)
7. Fail if regression >10%

### PR Comment Example

```markdown
## Benchmark Results

| Scenario | Baseline | Current | Diff | Status |
|----------|----------|---------|------|--------|
| SYN Scan | 98ms | 95ms | -3.1% | ✅ IMPROVED |
| Connect  | 45ms | 46ms | +2.2% | ✅ PASS |
| UDP      | 520ms | 540ms | +3.8% | ✅ PASS |
| Service  | 55ms | 62ms | +12.7% | ❌ REGRESSION |

**Overall:** 1 REGRESSION detected (Service Detection)

[View detailed report](link-to-artifact)
```

## Interpreting Results

### hyperfine Output

```
Benchmark 1: ./target/release/prtip -sS -p 1-1000 127.0.0.1
  Time (mean ± σ):      98.2 ms ±   4.5 ms    [User: 12.3 ms, System: 23.4 ms]
  Range (min … max):    90.1 ms … 108.9 ms    10 runs
```

**Fields:**
- **mean ± σ:** Average time ± standard deviation
- **User/System:** CPU time (user vs kernel)
- **Range:** Fastest and slowest runs
- **10 runs:** Number of measurement runs (excluding warmup)

### Good vs Bad Results

**Good (Reproducible):**
- Stddev <5% of mean (e.g., 98.2ms ± 4.5ms = 4.6%)
- Narrow range (max <20% higher than min)
- Consistent across multiple runs

**Bad (High Variance):**
- Stddev >10% of mean (e.g., 100ms ± 15ms = 15%)
- Wide range (max >50% higher than min)
- Outliers visible in distribution

**Action:** If variance too high, increase runs (10 → 20) or investigate system load

## Troubleshooting

### hyperfine not found

**Error:**
```
./scripts/01-syn-scan-1000-ports.sh: line 10: hyperfine: command not found
```

**Solution:**
```bash
cargo install hyperfine
```

### Binary not built

**Error:**
```
Error: Binary ./target/release/prtip not found
```

**Solution:**
```bash
cd /home/parobek/Code/ProRT-IP
cargo build --release
```

### Permission denied on scripts

**Error:**
```
bash: ./scripts/run-all-benchmarks.sh: Permission denied
```

**Solution:**
```bash
chmod +x scripts/*.sh
```

### High variance (stddev >10%)

**Problem:** Benchmark results inconsistent

**Causes:**
- CPU frequency scaling (power saving mode)
- Background processes (browser, indexing)
- Thermal throttling (laptop overheating)
- Cloud CI runners (shared resources)

**Solutions:**
- Close unnecessary applications
- Pin CPU frequency: `sudo cpupower frequency-set --governor performance` (Linux)
- Increase runs: `--runs 20` (instead of 10)
- Use median instead of mean (more robust against outliers)

### Network benchmarks fail (TLS parsing)

**Error:**
```
Error: Failed to connect to badssl.com:443
```

**Solution:**
- Check network connectivity: `ping badssl.com`
- Retry after network stabilizes
- Skip network-dependent benchmarks in CI (mark as optional)

## Future Enhancements (v0.6.0+)

- **Performance Dashboard:** GitHub Pages + Chart.js visualization
- **Automated Baseline Updates:** On release tags (v0.5.0, v0.6.0, etc.)
- **Multi-Platform Baselines:** Linux + macOS + Windows
- **More Scenarios:** Large networks (10.0.0.0/24), plugin combinations, database export
- **Criterion.rs Micro-Benchmarks:** CPU cycles, cache misses
- **Historical Trend Analysis:** Performance over time (improving/degrading)

## Documentation

- **Comprehensive Guide:** `docs/31-BENCHMARKING-GUIDE.md` (900+ lines)
- **hyperfine Research:** `/tmp/ProRT-IP/HYPERFINE-RESEARCH.md` (technical details)
- **Sprint TODO:** `to-dos/SPRINT-5.9-TODO.md` (1,577 lines)

## References

- **hyperfine:** https://github.com/sharkdp/hyperfine
- **Sprint 5.9 Plan:** Phase 5 Part 2 (Sprints 5.6-5.10)
- **CHANGELOG:** Sprint 5.9 entry (v0.4.9 or v0.5.0)
- **CI Workflow:** `.github/workflows/benchmark.yml`

---

**Status:** ✅ Operational (Sprint 5.9 implementation complete)
**Version:** v1.0.0
**Last Updated:** 2025-11-06
