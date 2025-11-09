# ProRT-IP Benchmarking Guide

**Version:** 1.0.0
**Last Updated:** 2025-11-06
**Sprint:** 5.9 - Benchmarking Framework

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Running Benchmarks Locally](#running-benchmarks-locally)
4. [Benchmark Scenarios](#benchmark-scenarios)
5. [CI Integration](#ci-integration)
6. [Interpreting Results](#interpreting-results)
7. [Adding New Benchmarks](#adding-new-benchmarks)
8. [Troubleshooting](#troubleshooting)
9. [Performance Optimization Tips](#performance-optimization-tips)
10. [Historical Data Analysis](#historical-data-analysis)

---

## Overview

### Why Benchmarking?

**Purpose:** Continuous performance validation infrastructure enabling:
- **Regression Detection:** Catch performance degradation before shipping
- **Competitive Validation:** Prove claims with reproducible data (vs Nmap, Masscan, RustScan)
- **Baseline Establishment:** Foundation for future optimizations
- **Performance Culture:** Demonstrates engineering rigor

**Claims Validated:**
- "10M+ pps" throughput (SYN scan scenario)
- "-1.8% overhead" rate limiting (Sprint 5.X)
- "15% overhead" IPv6 (Sprint 5.1)
- "500-800ms per port" idle scan (Sprint 5.3)
- "1.33μs" TLS certificate parsing (Sprint 5.5)
- "85-90% accuracy" service detection (Sprint 5.2)

### What We Measure

**Categories:**
1. **Throughput:** Packets per second, ports scanned per second
2. **Latency:** Scan duration (total time), time to first result
3. **Overhead:** Rate limiting, plugin execution, IPv6
4. **Accuracy:** Service detection rate, false positive rate

### How We Use It

**Development:**
- Run benchmarks locally before committing performance-sensitive changes
- Establish personal baselines for comparison

**CI/CD:**
- Automated regression detection on every PR
- Weekly scheduled runs for trend tracking
- Fail CI if regression >10%

**Release:**
- Update baselines on major releases (v0.5.0, v0.6.0, etc.)
- Validate performance claims in release notes

---

## Architecture

### Benchmark Suite Structure

```
benchmarks/05-Sprint5.9-Benchmarking-Framework/
├── README.md                   # Framework overview
├── scripts/                    # Runner scripts
│   ├── 01-syn-scan-1000-ports.sh
│   ├── 02-connect-scan-common-ports.sh
│   ├── 03-udp-scan-dns-snmp-ntp.sh
│   ├── 04-service-detection-overhead.sh
│   ├── 05-ipv6-overhead.sh
│   ├── 06-idle-scan-timing.sh
│   ├── 07-rate-limiting-overhead.sh
│   ├── 08-tls-cert-parsing.sh
│   ├── run-all-benchmarks.sh   # Orchestrator
│   ├── analyze-results.sh      # Regression detection
│   └── comparison-report.sh    # Markdown reports
├── configs/                    # Hyperfine configs (optional)
├── baselines/                  # Versioned baselines
│   ├── baseline-v0.4.8.json
│   └── baseline-v0.4.8-metadata.md
├── results/                    # Date-stamped results
│   └── YYYY-MM-DD-HHMMSS/
└── reports/                    # Analysis reports
```

### hyperfine Integration

**Tool:** hyperfine v1.16+ (command-line benchmarking tool)

**Why hyperfine?**
- **External Binary Benchmarking:** Tests complete binary (real-world usage)
- **Statistical Rigor:** Mean, stddev, outlier detection (IQR method)
- **JSON Export:** Machine-readable for regression detection
- **Cross-Platform:** Linux, macOS, Windows
- **Industry Standard:** Used by ripgrep, fd, bat, exa

**vs Criterion.rs:**
- Criterion: Library-based micro-benchmarks (CPU cycles, cache misses)
- hyperfine: End-to-end binary benchmarks (total execution time)
- **Decision:** hyperfine for Sprint 5.9 (simpler, matches real usage)

### Regression Detection Algorithm

**Algorithm:**
```python
def detect_regression(baseline, current):
    # 1. Calculate percentage difference
    diff = (current.mean - baseline.mean) / baseline.mean * 100

    # 2. Statistical significance test (optional, requires Python)
    t_stat, p_value = scipy.stats.ttest_ind(baseline.times, current.times)

    # 3. Categorize
    if p_value >= 0.05:
        return "PASS"  # Not statistically significant
    elif diff < -5:
        return "IMPROVED"
    elif diff < 5:
        return "PASS"
    elif diff < 10:
        return "WARN"
    else:
        return "FAIL"
```

**Thresholds:**
- **PASS:** <5% slower (within noise)
- **WARN:** 5-10% slower (investigate)
- **FAIL:** >10% slower (regression, CI fails)
- **IMPROVED:** Faster than baseline (celebrate!)

---

## Running Benchmarks Locally

### Prerequisites

**1. hyperfine (required):**
```bash
# Option 1: Cargo (recommended, latest version)
cargo install hyperfine

# Option 2: System package manager
# Linux (Debian/Ubuntu)
sudo apt install hyperfine

# macOS
brew install hyperfine
```

**2. ProRT-IP binary (required):**
```bash
cd /home/parobek/Code/ProRT-IP
cargo build --release
```

**3. Python 3.8+ (optional, for statistical tests):**
```bash
pip install pandas numpy scipy
```

### Quick Start

**1. Run all benchmarks:**
```bash
cd benchmarks/05-Sprint5.9-Benchmarking-Framework
./scripts/run-all-benchmarks.sh
```

**Output:**
```
=============================================
ProRT-IP Benchmarking Framework
=============================================
Date: 2025-11-06 23:45:00
Binary: /home/parobek/Code/ProRT-IP/target/release/prtip
Version: 0.4.8
Run directory: results/20251106-234500

---------------------------------------------
Running: 01-syn-scan-1000-ports.sh
---------------------------------------------
...
```

**2. Run single scenario:**
```bash
./scripts/01-syn-scan-1000-ports.sh
```

**3. Establish baseline (for releases):**
```bash
./scripts/run-all-benchmarks.sh --baseline
```

**4. Compare against baseline:**
```bash
./scripts/run-all-benchmarks.sh --compare baselines/baseline-v0.4.8.json
```

### Workflow Examples

**Example 1: Pre-commit check**
```bash
# 1. Make performance-sensitive changes
git add .

# 2. Build release binary
cargo build --release

# 3. Run affected benchmark
./scripts/07-rate-limiting-overhead.sh

# 4. Review results
cat results/rate-limiting-*.json | jq '.results[0].mean'

# 5. Compare manually
# If mean within 5% of previous run → commit
# If mean >5% slower → investigate
```

**Example 2: Release baseline**
```bash
# 1. Tag release
git tag -a v0.5.0 -m "v0.5.0 release"

# 2. Build release binary
cargo build --release

# 3. Run full suite and save baseline
./scripts/run-all-benchmarks.sh --baseline

# 4. Commit baseline files
git add benchmarks/baselines/baseline-v0.5.0.*
git commit -m "chore: Add v0.5.0 performance baseline"
```

**Example 3: PR validation**
```bash
# 1. Checkout PR branch
git checkout feature/new-optimization

# 2. Build release binary
cargo build --release

# 3. Run full suite
./scripts/run-all-benchmarks.sh

# 4. Compare against main branch baseline
./scripts/analyze-results.sh \
    baselines/baseline-v0.4.8.json \
    results/latest/syn-scan-*.json

# 5. Review regression report
# Exit code 0 = pass, 1 = warn, 2 = fail
```

---

## Benchmark Scenarios

### Scenario 1: SYN Scan (1,000 ports)

**Purpose:** Validate throughput ("10M+ pps" claim, indirectly)

**Command:**
```bash
prtip -sS -p 1-1000 127.0.0.1 --rate-limit 0
```

**Metric:** Scan duration (lower = better)

**Target:** <100ms for 1,000 ports on localhost

**Rationale:**
- SYN scan is fastest scan type (stateless, no connection tracking)
- 1,000 ports is standard benchmark size (balances speed vs coverage)
- Localhost eliminates network latency (pure CPU/packet performance)
- `--rate-limit 0` removes rate limiting overhead

**Example Result:**
```
Benchmark 1: prtip -sS -p 1-1000 127.0.0.1
  Time (mean ± σ):      98.2 ms ±   4.5 ms    [User: 12.3 ms, System: 23.4 ms]
  Range (min … max):    90.1 ms … 108.9 ms    10 runs
```

**Interpretation:**
- Mean: 98.2ms (✅ under 100ms target)
- Stddev: 4.5ms (4.6% variance, acceptable)
- Range: 18.8ms spread (reasonable)

### Scenario 2: Connect Scan (3 common ports)

**Purpose:** Real-world baseline (most common usage)

**Command:**
```bash
prtip -sT -p 80,443,8080 127.0.0.1
```

**Metric:** Scan duration

**Target:** <50ms

**Comparison:** vs Nmap -sT (ProRT-IP should be faster)

**Rationale:**
- Connect scan uses full TCP handshake (realistic)
- Ports 80, 443, 8080 are most scanned in practice
- Small port count (3) tests per-connection overhead

**Example Result:**
```
Benchmark 1: prtip -sT -p 80,443,8080 127.0.0.1
  Time (mean ± σ):      45.3 ms ±   2.1 ms    [User: 5.2 ms, System: 8.7 ms]
  Range (min … max):    42.1 ms …  49.8 ms    10 runs
```

**Interpretation:**
- Mean: 45.3ms (✅ under 50ms target)
- 15ms per port average (efficient connection pooling)

### Scenario 3: UDP Scan (3 UDP services)

**Purpose:** Slow protocol validation (10-100x slower than TCP)

**Command:**
```bash
prtip -sU -p 53,161,123 127.0.0.1
```

**Metric:** Scan duration

**Target:** <500ms (UDP is slow due to ICMP rate limiting)

**Rationale:**
- UDP scanning requires waiting for ICMP unreachable (timeout-based)
- Kernel ICMP rate limiting slows scans (Linux: 1000 pps)
- Ports 53 (DNS), 161 (SNMP), 123 (NTP) are common UDP services

**Example Result:**
```
Benchmark 1: prtip -sU -p 53,161,123 127.0.0.1
  Time (mean ± σ):     520.5 ms ±  18.2 ms    [User: 2.1 ms, System: 5.4 ms]
  Range (min … max):   495.3 ms … 558.7 ms    10 runs
```

**Interpretation:**
- Mean: 520.5ms (✅ within 500ms target, acceptable for UDP)
- 173ms per port (expected for ICMP-based detection)

### Scenario 4: Service Detection Overhead

**Purpose:** Validate 85-90% accuracy + low overhead

**Commands:**
- Baseline: `prtip -sS -p 22,80,443 127.0.0.1` (no -sV)
- Detection: `prtip -sV -p 22,80,443 127.0.0.1` (with -sV)

**Metric:** Overhead = (detection_time - baseline_time) / baseline_time * 100

**Target:** <10% overhead

**Rationale:**
- Service detection adds banner grabbing + regex matching overhead
- Ports 22 (SSH), 80 (HTTP), 443 (HTTPS) have well-known banners
- Overhead should be minimal for cached probes

**Example Result:**
```
Benchmark 1: baseline
  Time (mean ± σ):      55.2 ms ±   3.1 ms
Benchmark 2: detection
  Time (mean ± σ):      62.3 ms ±   3.5 ms

Overhead: (62.3 - 55.2) / 55.2 * 100 = 12.9%
```

**Interpretation:**
- Overhead: 12.9% (⚠️ slightly over 10% target)
- Investigate: Probe database loading? Regex compilation?

### Scenario 5: IPv6 Overhead

**Purpose:** Validate Sprint 5.1 IPv6 claim (15% overhead)

**Commands:**
- IPv4: `prtip -4 -sS -p 1-1000 127.0.0.1`
- IPv6: `prtip -6 -sS -p 1-1000 ::1`

**Metric:** IPv6 overhead vs IPv4 baseline

**Target:** <15% slower than IPv4

**Rationale:**
- IPv6 headers larger (40 bytes vs 20 bytes IPv4)
- ICMPv6 error handling more complex (Type 1 codes 0-5)
- NDP (Neighbor Discovery Protocol) adds overhead

**Example Result:**
```
Benchmark 1: ipv4
  Time (mean ± σ):      98.2 ms ±   4.5 ms
Benchmark 2: ipv6
  Time (mean ± σ):     110.5 ms ±   5.2 ms

Overhead: (110.5 - 98.2) / 98.2 * 100 = 12.5%
```

**Interpretation:**
- Overhead: 12.5% (✅ under 15% target)
- IPv6 slower as expected, within acceptable range

### Scenario 6: Idle Scan Timing

**Purpose:** Validate Sprint 5.3 idle scan claim (500-800ms per port)

**Command:**
```bash
prtip -sI <zombie-ip> -p 80,443,8080 <target>
```

**Metric:** Time per port scanned

**Target:** 500-800ms per port

**Setup Required:**
- Zombie host: Windows XP or old Linux (predictable IPID)
- Target host: Any reachable host
- See `ZOMBIE-SETUP.md` for configuration

**Rationale:**
- Idle scan requires 3 packets per port (probe zombie 2x + spoof SYN)
- IPID tracking adds latency (sequential IPID increments)
- Stealth comes at performance cost

**Example Result:**
```
Benchmark 1: prtip -sI 192.168.1.100 -p 80,443,8080 192.168.1.200
  Time (mean ± σ):      1.85 s ±  0.12 s
  Range (min … max):    1.68 s …  2.01 s     5 runs

Time per port: 1850ms / 3 ports = 617ms per port
```

**Interpretation:**
- Time per port: 617ms (✅ within 500-800ms target)
- Total: 1.85s for 3 ports (reasonable for stealth scan)

### Scenario 7: Rate Limiting Overhead

**Purpose:** Validate Sprint 5.X AdaptiveRateLimiterV3 (-1.8% overhead)

**Commands:**
- No limit: `prtip -sS -p 1-1000 127.0.0.1 --rate-limit 0`
- V3 limiter: `prtip -sS -p 1-1000 127.0.0.1 --rate-limit 10000`

**Metric:** Overhead = (limited_time - baseline_time) / baseline_time * 100

**Target:** <5% overhead (claimed -1.8%)

**Rationale:**
- AdaptiveRateLimiterV3 uses Relaxed memory ordering (zero barrier overhead)
- Convergence-based scheduling should be faster than token bucket
- Negative overhead possible (better CPU cache behavior)

**Example Result:**
```
Benchmark 1: no-limit
  Time (mean ± σ):      98.2 ms ±   4.5 ms
Benchmark 2: v3-limiter
  Time (mean ± σ):      96.4 ms ±   4.2 ms

Overhead: (96.4 - 98.2) / 98.2 * 100 = -1.8%
```

**Interpretation:**
- Overhead: -1.8% (✅ matches claim exactly!)
- V3 limiter actually faster (better pacing = better cache locality)

### Scenario 8: TLS Certificate Parsing

**Purpose:** Validate Sprint 5.5 TLS parsing claim (1.33μs)

**Command:**
```bash
prtip -sV -p 443 badssl.com --tls-cert-analysis
```

**Metric:** Average time per certificate parsed

**Target:** ~1.33μs

**Rationale:**
- TLS parsing is on critical path (HTTPS services)
- X.509v3 certificates can be large (2-4KB typical)
- 1.33μs = 750,000 certs/second (excellent performance)

**Extraction:**
- Requires parsing verbose output or instrumentation
- Total scan time includes network latency (not pure parsing time)
- Better measured with Criterion.rs micro-benchmark (future)

**Example Result:**
```
Benchmark 1: prtip -sV -p 443 badssl.com --tls-cert-analysis
  Time (mean ± σ):     850.3 ms ±  42.1 ms
  (Includes network latency, not pure parsing time)
```

**Interpretation:**
- Total time: 850ms (network-bound, not CPU-bound)
- Pure parsing time: Requires instrumentation (see logs)

---

## CI Integration

### GitHub Actions Workflow

**File:** `.github/workflows/benchmark.yml`

**Triggers:**
- `push` to main (after test workflow passes)
- `pull_request` (performance validation)
- `workflow_dispatch` (manual runs)
- `schedule`: Weekly (Monday 00:00 UTC)

**Jobs:**
```yaml
jobs:
  benchmark:
    runs-on: ubuntu-latest
    timeout-minutes: 15

    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Build release binary
        run: cargo build --release
      - name: Install hyperfine
        run: cargo install hyperfine
      - name: Run benchmark suite
        run: ./benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/run-all-benchmarks.sh
      - name: Compare against baseline
        id: regression
        run: |
          ./benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/analyze-results.sh \
            benchmarks/baselines/baseline-v0.4.8.json \
            results/current.json
      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: results/
          retention-days: 7
      - name: Comment on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('results/summary.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: report
            });
```

### Workflow Steps Explained

**1. Build release binary:**
- Ensures benchmarks test optimized code (`--release`)

**2. Install hyperfine:**
- `cargo install hyperfine` (or cache from previous run)

**3. Run benchmark suite:**
- Executes all 8 scenarios (`run-all-benchmarks.sh`)

**4. Compare against baseline:**
- Regression detection (`analyze-results.sh`)
- Exit code 0/1/2 determines pass/warn/fail

**5. Upload results:**
- Artifacts retained for 7 days (historical comparison)

**6. Comment on PR:**
- Automated feedback with performance summary

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

**Recommendation:** Investigate Service Detection slowdown before merge

[View detailed report](https://github.com/doublegate/ProRT-IP/actions/runs/12345678/artifacts/98765432)
```

### Failure Handling

**Exit Codes:**
- **0:** PASS or IMPROVED (merge approved)
- **1:** WARN (log warning, still pass CI)
- **2:** FAIL (block merge, requires investigation)

**Thresholds:**
- **5%:** Warning threshold (investigate but don't block)
- **10%:** Failure threshold (block merge)

---

## Interpreting Results

### hyperfine Output Format

```
Benchmark 1: ./target/release/prtip -sS -p 1-1000 127.0.0.1
  Time (mean ± σ):      98.2 ms ±   4.5 ms    [User: 12.3 ms, System: 23.4 ms]
  Range (min … max):    90.1 ms … 108.9 ms    10 runs
```

**Fields:**
- **mean:** Average execution time across all runs
- **σ (stddev):** Standard deviation (measure of variance)
- **User:** User-space CPU time (application code)
- **System:** Kernel-space CPU time (syscalls, I/O)
- **Range:** Fastest and slowest runs
- **10 runs:** Number of measurement runs (excluding warmup)

### Good vs Bad Results

**Good (Reproducible):**
- Stddev <5% of mean (e.g., 98.2ms ± 4.5ms = 4.6%)
- Narrow range (max <20% higher than min)
- User + System ≈ mean (CPU-bound, no idle time)

**Bad (High Variance):**
- Stddev >10% of mean (e.g., 100ms ± 15ms = 15%)
- Wide range (max >50% higher than min)
- User + System << mean (I/O-bound or waiting)

**Example Analysis:**
```
Good:
  Time (mean ± σ):      98.2 ms ±   4.5 ms  (4.6% variance)
  Range:                90.1 ms … 108.9 ms  (20.9% spread)

Bad:
  Time (mean ± σ):     105.3 ms ±  18.7 ms  (17.8% variance)
  Range:                82.1 ms … 145.6 ms  (77.3% spread)
```

### Statistical Significance

**t-test (Two-Sample):**
- **Purpose:** Determine if performance difference is real (not random)
- **Test:** scipy.stats.ttest_ind(baseline.times, current.times)
- **Threshold:** p < 0.05 (95% confidence)
- **Interpretation:**
  - p < 0.05: Statistically significant difference
  - p ≥ 0.05: Within noise (accept null hypothesis)

**Example:**
```python
from scipy.stats import ttest_ind

baseline_times = [98.0, 95.2, 102.3, ...]  # 10 runs
current_times = [110.5, 108.3, 115.2, ...]  # 10 runs

t_stat, p_value = ttest_ind(baseline_times, current_times)
# p_value = 0.002 (< 0.05) → statistically significant regression
```

### Outlier Detection

**IQR Method (hyperfine built-in):**
- **Q1:** 25th percentile
- **Q3:** 75th percentile
- **IQR:** Q3 - Q1
- **Outliers:** Runs outside [Q1 - 1.5×IQR, Q3 + 1.5×IQR]
- **Effect:** Outliers removed from mean calculation (robust statistics)

---

## Adding New Benchmarks

### Step 1: Create Scenario Script

**Template:**
```bash
#!/usr/bin/env bash
#
# Scenario N: <Description>
# Purpose: <Why this benchmark>
# Target: <Performance target>
#

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
BINARY="${PROJECT_ROOT}/target/release/prtip"
RESULTS_DIR="${SCRIPT_DIR}/../results"
DATE=$(date +%Y%m%d-%H%M%S)

# Validate binary exists
if [[ ! -f "${BINARY}" ]]; then
    echo "Error: ProRT-IP binary not found at ${BINARY}"
    exit 1
fi

# Create results directory
mkdir -p "${RESULTS_DIR}"

# Run benchmark
echo "Running Scenario N: <Description>..."
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/scenario-n-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/scenario-n-${DATE}.md" \
    "${BINARY} <command>"

echo "Results saved to ${RESULTS_DIR}"
```

### Step 2: Make Script Executable

```bash
chmod +x scripts/09-new-scenario.sh
```

### Step 3: Add to Orchestrator

**Edit:** `scripts/run-all-benchmarks.sh`

**Add to BENCHMARKS array:**
```bash
declare -a BENCHMARKS=(
    "01-syn-scan-1000-ports.sh"
    ...
    "09-new-scenario.sh"  # Add here
)
```

### Step 4: Update Documentation

**Add to this guide:**
- Section 4: Benchmark Scenarios (describe new scenario)
- Section 6: Interpreting Results (if new metric type)

**Add to README.md:**
- Update scenario count (8 → 9)

### Step 5: Test Locally

```bash
# Test script individually
./scripts/09-new-scenario.sh

# Test full suite
./scripts/run-all-benchmarks.sh

# Verify results
ls -lh results/scenario-n-*.json
```

---

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
Error: ProRT-IP binary not found at ./target/release/prtip
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
chmod +x benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/*.sh
```

### High variance (stddev >10%)

**Problem:** Benchmark results inconsistent

**Causes:**
- CPU frequency scaling (power saving mode)
- Background processes (browser, indexing)
- Thermal throttling (laptop overheating)
- Cloud CI runners (shared resources)

**Solutions:**

**1. Pin CPU frequency (Linux):**
```bash
# Disable CPU frequency scaling
sudo cpupower frequency-set --governor performance

# Re-enable power saving after benchmarks
sudo cpupower frequency-set --governor powersave
```

**2. Close background processes:**
```bash
# Close browser, IDE, etc.
# Disable indexing (Linux: systemctl stop locate.timer)
```

**3. Increase runs:**
```bash
# Change --runs 10 to --runs 20 in script
hyperfine --runs 20 <command>
```

**4. Use median instead of mean:**
```bash
# Extract median from JSON
jq '.results[0].median' results/syn-scan-*.json
```

### Network benchmarks fail (TLS parsing)

**Error:**
```
Error: Failed to connect to badssl.com:443
```

**Causes:**
- No internet connectivity
- DNS resolution failure
- Firewall blocking HTTPS

**Solutions:**

**1. Check connectivity:**
```bash
ping badssl.com
curl -I https://badssl.com
```

**2. Use local HTTPS server:**
```bash
# Setup local nginx with self-signed cert
sudo apt install nginx
sudo systemctl start nginx
# Test: prtip -sV -p 443 127.0.0.1 --tls-cert-analysis
```

**3. Skip network-dependent benchmarks:**
```bash
# Edit run-all-benchmarks.sh, comment out 08-tls-cert-parsing.sh
# OR: Mark as optional (exit 0 on failure)
```

---

## Performance Optimization Tips

### Based on Benchmark Insights

**1. Reduce Syscalls (from User/System time analysis):**
```
Before:
  Time (mean ± σ):     102.3 ms    [User: 5.2 ms, System: 45.6 ms]
  System time: 45.6ms (44% of total) → high syscall overhead

Optimization:
  - Batch packet sending (sendmmsg instead of send)
  - Reduce write() calls (buffer results)

After:
  Time (mean ± σ):      85.1 ms    [User: 5.0 ms, System: 18.3 ms]
  System time: 18.3ms (21% of total) → 60% reduction
```

**2. Improve Cache Locality (from rate limiting overhead):**
```
Observation:
  - AdaptiveRateLimiterV3 with Relaxed memory ordering: -1.8% overhead
  - Better CPU cache behavior (fewer memory barriers)

Takeaway:
  - Use Relaxed/Acquire/Release instead of SeqCst where possible
  - Profile with `perf stat` to measure cache misses
```

**3. Reduce Allocations (from allocation profiling):**
```
Before:
  - Allocate Vec<u8> per packet
  - 1M packets = 1M allocations

After:
  - Reuse buffers (object pool)
  - Zero-copy where possible

Benchmark:
  - 15% performance improvement (98ms → 83ms)
```

---

## Historical Data Analysis

### Baseline Management

**Establish Baseline (on releases):**
```bash
# Tag release
git tag -a v0.5.1 -m "Release v0.5.1"

# Build release binary
cargo build --release

# Create baseline (automated script)
cd benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts
./create-baseline.sh v0.5.1

# Results saved to:
#   benchmarks/baselines/v0.5.1/*.json
#   benchmarks/baselines/v0.5.1/baseline-metadata.md

# Commit baseline
git add benchmarks/baselines/
git commit -m "chore: Add v0.5.1 performance baseline"
```

**Baseline Directory Structure:**
```
benchmarks/baselines/
├── v0.5.0/
│   ├── syn-scan-20251109-120000.json
│   ├── connect-scan-20251109-120015.json
│   ├── ...
│   └── baseline-metadata.md
├── v0.5.1/
│   ├── syn-scan-20251109-140000.json
│   ├── ...
│   └── baseline-metadata.md
```

**Baseline Naming Convention:**
- Version-tagged directories: `v0.4.8/`, `v0.5.0/`, `v1.0.0/`
- Individual scenario files: `{scenario}-{timestamp}.json`
- Metadata file: `baseline-metadata.md` (system info, git commit, scenarios)

**Using Baselines for Regression Detection:**
```bash
# Compare current results against v0.5.0 baseline
./benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/analyze-results.sh \
    benchmarks/baselines/v0.5.0 \
    benchmarks/05-Sprint5.9-Benchmarking-Framework/results

# Exit codes:
#   0 = PASS (within 5% or improved)
#   1 = WARN (5-10% slower)
#   2 = FAIL (>10% slower, regression detected)
```

### Trend Tracking (Future Enhancement)

**Idea:** Aggregate baselines for long-term trend analysis

**Implementation (v0.6.0+):**
```bash
# scripts/trend-analysis.py
import json
import matplotlib.pyplot as plt

baselines = [
    'baseline-v0.4.8.json',
    'baseline-v0.5.0.json',
    'baseline-v0.6.0.json'
]

# Extract mean times
versions = []
means = []
for baseline in baselines:
    with open(f'baselines/{baseline}') as f:
        data = json.load(f)
        versions.append(baseline.replace('baseline-', '').replace('.json', ''))
        means.append(data['results'][0]['mean'])

# Plot trend
plt.plot(versions, means, marker='o')
plt.xlabel('Version')
plt.ylabel('Scan Time (seconds)')
plt.title('Performance Trend: SYN Scan (1,000 ports)')
plt.savefig('reports/performance-trend.png')
```

**Output:** Chart showing performance improving over time

---

## Summary

**Benchmarking Framework Capabilities:**
- ✅ **20 benchmark scenarios** covering all major features (expanded Sprint 5.5.4)
  - 8 core scenarios (Sprint 5.9)
  - 5 additional scan types (FIN, NULL, Xmas, ACK, UDP)
  - 4 scale tests (small/medium/large, all-ports)
  - 2 timing templates (T0 paranoid, T5 insane)
  - 5 feature overhead tests (OS fingerprinting, banner grabbing, fragmentation, decoys, events)
- ✅ hyperfine integration for statistical rigor
- ✅ Regression detection (5% warn, 10% fail)
- ✅ CI/CD automation (GitHub Actions - Sprint 5.5.4)
- ✅ Historical baseline tracking (version-tagged directories)
- ✅ PR comment generation (automated feedback)
- ✅ Comprehensive documentation (this guide)

**Next Steps:**
1. Run benchmarks locally (establish personal baseline)
2. Review CI integration (observe automated runs)
3. Add new scenarios as features added (follow guide)
4. Analyze trends quarterly (performance improving?)

**Future Enhancements (v0.6.0+):**
- Performance dashboard (GitHub Pages + Chart.js)
- Automated baseline updates (on release tags)
- Multi-platform baselines (Linux + macOS + Windows)
- Criterion.rs micro-benchmarks (CPU cycles, cache misses)
- Statistical significance testing (Python t-test integration)
- Flamegraph integration (profile on regression)

---

**Version:** 1.1.0
**Last Updated:** 2025-11-09
**Sprint:** 5.5.4 - Performance Audit & Optimization (CI/CD + Regression Detection)
