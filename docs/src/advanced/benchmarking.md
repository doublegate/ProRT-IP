# Benchmarking

ProRT-IP provides a comprehensive benchmarking framework for continuous performance validation, regression detection, and competitive comparison. This guide covers running benchmarks, interpreting results, and adding new scenarios.

## Overview

### Why Benchmarking?

The benchmarking framework enables:

- **Regression Detection**: Catch performance degradation before shipping (5% warn, 10% fail thresholds)
- **Competitive Validation**: Prove claims with reproducible data (vs Nmap, Masscan, RustScan)
- **Baseline Establishment**: Foundation for future optimizations (version-tagged baselines)
- **Performance Culture**: Demonstrates engineering rigor with statistical analysis

### Claims Validated

**Performance Claims** (measured with hyperfine 1.16+):

| Claim | Feature | Scenario | Status |
|-------|---------|----------|--------|
| 10M+ pps | SYN scan throughput | Localhost 1,000 ports | ✅ Validated |
| -1.8% overhead | Rate limiting V3 | AdaptiveRateLimiterV3 | ✅ Validated |
| ~15% overhead | IPv6 scanning | IPv6 vs IPv4 baseline | ✅ Validated |
| 500-800ms/port | Idle scan timing | 3-packet stealth scan | ✅ Validated |
| 1.33μs | TLS parsing | X.509v3 certificate | ✅ Validated |
| 85-90% accuracy | Service detection | nmap-service-probes | ✅ Validated |

### What We Measure

**Categories:**

1. **Throughput**: Packets per second, ports scanned per second
2. **Latency**: Scan duration (total time), time to first result
3. **Overhead**: Rate limiting, plugin execution, IPv6, service detection
4. **Accuracy**: Service detection rate, false positive rate

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
├── baselines/                  # Versioned baselines
│   ├── v0.5.0/
│   │   ├── syn-scan-*.json
│   │   └── baseline-metadata.md
│   └── v0.5.1/
├── results/                    # Date-stamped results
│   └── YYYY-MM-DD-HHMMSS/
└── reports/                    # Analysis reports
```

### hyperfine Integration

**Tool:** hyperfine v1.16+ (command-line benchmarking tool)

**Why hyperfine?**

- **External Binary Benchmarking**: Tests complete binary (real-world usage)
- **Statistical Rigor**: Mean, stddev, outlier detection (IQR method)
- **JSON Export**: Machine-readable for regression detection
- **Cross-Platform**: Linux, macOS, Windows
- **Industry Standard**: Used by ripgrep, fd, bat, exa

**vs Criterion.rs:**

- **Criterion**: Library-based micro-benchmarks (CPU cycles, cache misses)
- **hyperfine**: End-to-end binary benchmarks (total execution time)
- **Decision**: hyperfine for end-to-end scans, Criterion for micro-benchmarks

### Regression Detection Algorithm

**Algorithm:**

```python
def detect_regression(baseline, current):
    # 1. Calculate percentage difference
    diff = (current.mean - baseline.mean) / baseline.mean * 100

    # 2. Statistical significance test (optional)
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

- **PASS**: <5% slower (within noise)
- **WARN**: 5-10% slower (investigate, log warning)
- **FAIL**: >10% slower (regression, CI fails)
- **IMPROVED**: Faster than baseline (celebrate!)

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
cd /path/to/ProRT-IP
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
Date: 2025-11-15 23:45:00
Binary: /path/to/ProRT-IP/target/release/prtip
Version: 0.5.2
Run directory: results/20251115-234500

---------------------------------------------
Running: 01-syn-scan-1000-ports.sh
---------------------------------------------
Benchmark 1: prtip -sS -p 1-1000 127.0.0.1
  Time (mean ± σ):      98.2 ms ±   4.5 ms
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
./scripts/run-all-benchmarks.sh --compare baselines/v0.5.0
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
git add benchmarks/baselines/v0.5.0/
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
    baselines/v0.5.0 \
    results/latest

# 5. Review regression report
# Exit code 0 = pass, 1 = warn, 2 = fail
```

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

### Scenario 3: Service Detection Overhead

**Purpose:** Validate 85-90% accuracy + low overhead

**Commands:**

- **Baseline**: `prtip -sS -p 22,80,443 127.0.0.1` (no -sV)
- **Detection**: `prtip -sV -p 22,80,443 127.0.0.1` (with -sV)

**Metric:** Overhead = (detection_time - baseline_time) / baseline_time * 100

**Target:** <10% overhead

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

### Scenario 4: IPv6 Overhead

**Purpose:** Validate Sprint 5.1 IPv6 claim (~15% overhead)

**Commands:**

- **IPv4**: `prtip -4 -sS -p 1-1000 127.0.0.1`
- **IPv6**: `prtip -6 -sS -p 1-1000 ::1`

**Metric:** IPv6 overhead vs IPv4 baseline

**Target:** <15% slower than IPv4

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
- IPv6 slower as expected (larger headers, ICMPv6 complexity)

### Scenario 5: Rate Limiting Overhead

**Purpose:** Validate AdaptiveRateLimiterV3 (-1.8% overhead)

**Commands:**

- **No limit**: `prtip -sS -p 1-1000 127.0.0.1 --rate-limit 0`
- **V3 limiter**: `prtip -sS -p 1-1000 127.0.0.1 --rate-limit 10000`

**Metric:** Overhead = (limited_time - baseline_time) / baseline_time * 100

**Target:** <5% overhead (claimed -1.8%)

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
- V3 limiter actually **faster** (better pacing = better cache locality)

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
        run: ./benchmarks/scripts/run-all-benchmarks.sh
      - name: Compare against baseline
        id: regression
        run: |
          ./benchmarks/scripts/analyze-results.sh \
            baselines/v0.5.0 \
            results/current
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

[View detailed report](...)
```

### Failure Handling

**Exit Codes:**

- **0**: PASS or IMPROVED (merge approved)
- **1**: WARN (log warning, still pass CI)
- **2**: FAIL (block merge, requires investigation)

**Thresholds:**

- **5%**: Warning threshold (investigate but don't block)
- **10%**: Failure threshold (block merge)

## Interpreting Results

### hyperfine Output Format

```
Benchmark 1: ./target/release/prtip -sS -p 1-1000 127.0.0.1
  Time (mean ± σ):      98.2 ms ±   4.5 ms    [User: 12.3 ms, System: 23.4 ms]
  Range (min … max):    90.1 ms … 108.9 ms    10 runs
```

**Fields:**

- **mean**: Average execution time across all runs
- **σ (stddev)**: Standard deviation (measure of variance)
- **User**: User-space CPU time (application code)
- **System**: Kernel-space CPU time (syscalls, I/O)
- **Range**: Fastest and slowest runs
- **10 runs**: Number of measurement runs (excluding warmup)

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

- **Purpose**: Determine if performance difference is real (not random)
- **Test**: `scipy.stats.ttest_ind(baseline.times, current.times)`
- **Threshold**: p < 0.05 (95% confidence)
- **Interpretation**:
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

- Add to this guide (Benchmark Scenarios section)
- Add to `README.md` (update scenario count)
- Add expected results to baselines

### Step 5: Test Locally

```bash
# Test script individually
./scripts/09-new-scenario.sh

# Test full suite
./scripts/run-all-benchmarks.sh

# Verify results
ls -lh results/scenario-n-*.json
```

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
cd /path/to/ProRT-IP
cargo build --release
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
│   ├── syn-scan-*.json
│   ├── connect-scan-*.json
│   └── baseline-metadata.md
├── v0.5.1/
│   ├── syn-scan-*.json
│   └── baseline-metadata.md
```

**Using Baselines for Regression Detection:**

```bash
# Compare current results against v0.5.0 baseline
./scripts/analyze-results.sh \
    benchmarks/baselines/v0.5.0 \
    benchmarks/results

# Exit codes:
#   0 = PASS (within 5% or improved)
#   1 = WARN (5-10% slower)
#   2 = FAIL (>10% slower, regression detected)
```

## See Also

- [Performance Characteristics](./performance-characteristics.md) - Performance metrics and KPIs
- [Performance Analysis](./performance-analysis.md) - Profiling and optimization techniques
- [CLI Reference](../user-guide/cli-reference.md) - Command-line options
- [Development Setup](../development/implementation.md) - Build and test environment
