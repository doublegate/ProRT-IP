# ProRT-IP Benchmark Baselines

This directory contains Criterion benchmark baseline data for performance regression detection across ProRT-IP versions.

## Purpose

Benchmark baselines enable:
- **Performance Regression Detection**: Compare new changes against historical performance
- **Release Validation**: Ensure new versions don't introduce slowdowns
- **CI/CD Integration**: Automated performance checks in GitHub Actions (future)
- **Performance Tracking**: Visualize performance trends over time

## Directory Structure

```
benchmarks/baselines/
├── v0.3.7/              # Baseline for v0.3.7 (current)
│   ├── binary_startup_help/
│   ├── binary_startup_version/
│   ├── port_parsing/
│   ├── localhost_scan/
│   └── output_formats/
└── README.md            # This file
```

Each version directory contains Criterion baseline data copied from `target/criterion/`.

## v0.3.7 Baseline (Current)

**Platform Specifications:**
- **OS**: Linux 6.17.1-2-cachyos (CachyOS)
- **CPU**: Intel(R) Core(TM) i9-10850K @ 3.60GHz (10 cores, 20 threads)
- **RAM**: 62 GiB
- **Rust**: 1.90.0 (1159e78c4 2025-09-14)
- **Build**: `cargo build --release` (opt-level=3, lto="fat")
- **Date**: 2025-10-13

**Benchmark Results:**

| Benchmark Suite | Test | Mean | Std Dev | Notes |
|-----------------|------|------|---------|-------|
| **binary_startup** | help flag | 2.21 ms | ±0.07 ms | Binary startup overhead |
| **binary_startup** | version flag | 2.19 ms | ±0.08 ms | Minimal overhead |
| **port_parsing** | single_port | 1.70 ns | ±0.01 ns | Extremely fast parsing |
| **port_parsing** | port_range_small | 1.67 ns | ±0.00 ns | Range parsing optimized |
| **port_parsing** | port_list | 1.68 ns | ±0.00 ns | List parsing efficient |
| **localhost_scan** | single_port | 5.29 ms | ±0.13 ms | Full scan overhead |
| **localhost_scan** | three_ports | 5.03 ms | ±0.21 ms | Minimal per-port cost |
| **localhost_scan** | port_range_10 | 5.16 ms | ±0.12 ms | Efficient batching |
| **output_formats** | text_output | 5.06 ms | ±0.07 ms | Text formatting |
| **output_formats** | json_output | 2.14 ms | ±0.22 ms | JSON serialization |

**Key Performance Characteristics:**
- **Binary Startup**: ~2.2ms (acceptable overhead for CLI tool)
- **Port Parsing**: Sub-nanosecond performance (zero overhead)
- **Localhost Scans**: 5-6ms baseline (includes TCP handshake + OS overhead)
- **Output Formats**: JSON 2.4x faster than text (serialization vs formatting)

## Usage

### Running Benchmarks with Baseline Comparison

**Initial Baseline Capture (already done for v0.3.7):**
```bash
# Run benchmarks and save baseline
cargo bench --bench benchmarks -- --save-baseline v0.3.7

# Copy baseline data to git (for team sharing)
cp -r target/criterion/* benchmarks/baselines/v0.3.7/
```

**Compare Against Baseline (detect regressions):**
```bash
# Make code changes...

# Run benchmarks and compare against v0.3.7 baseline
cargo bench --bench benchmarks -- --baseline v0.3.7

# Criterion will show performance deltas:
# - "change: +5.0%" means 5% slower (regression)
# - "change: -5.0%" means 5% faster (improvement)
# - "change: +0.1%" within noise threshold (no significant change)
```

**Create New Baseline for Next Version:**
```bash
# After v0.3.8 release
cargo bench --bench benchmarks -- --save-baseline v0.3.8
cp -r target/criterion/* benchmarks/baselines/v0.3.8/

# Update this README.md with v0.3.8 platform specs and results
```

### Using cargo-criterion (Enhanced CLI)

For more powerful baseline management, install `cargo-criterion`:

```bash
cargo install cargo-criterion
```

**List Available Baselines:**
```bash
cargo criterion --list-baselines
```

**Compare Against Specific Baseline:**
```bash
cargo criterion --baseline v0.3.7
```

**Generate Comparison Report:**
```bash
cargo criterion --baseline v0.3.7 --message-format json > comparison.json
```

**Delete Old Baselines:**
```bash
cargo criterion --delete-baseline v0.3.0  # Remove obsolete baseline
```

## Interpreting Results

### Performance Change Thresholds

Criterion uses statistical analysis to determine if changes are significant:

- **Change < 1%**: Likely noise, no action needed
- **Change 1-5%**: Minor regression/improvement, investigate if consistent
- **Change 5-10%**: Moderate regression/improvement, review code changes
- **Change > 10%**: Major regression/improvement, requires investigation

### Regression Detection Example

```
Benchmarking localhost_scan/single_port: Analyzing
localhost_scan/single_port
                        time:   [5.5123 ms 5.6842 ms 5.8123 ms]
                        change: [+7.0% +9.2% +11.5%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

**Interpretation:**
- Mean time increased from 5.29ms → 5.68ms (+9.2%)
- Statistical significance: p < 0.05 (95% confidence)
- Action: Investigate code changes causing 9% slowdown

### Improvement Example

```
Benchmarking output_formats/json_output: Analyzing
output_formats/json_output
                        time:   [1.8123 ms 1.9234 ms 2.0123 ms]
                        change: [-12.5% -10.3% -8.1%] (p = 0.00 < 0.05)
                        Performance has improved.
```

**Interpretation:**
- Mean time decreased from 2.14ms → 1.92ms (-10.3%)
- Statistical significance: p < 0.05 (95% confidence)
- Action: Document optimization in CHANGELOG.md

## CI/CD Integration (Future Work)

**Planned GitHub Actions Workflow:**

```yaml
name: Performance Regression Check

on:
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Fetch all history for baseline comparison

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Restore Baseline
        run: |
          # Checkout main branch baseline
          git checkout main -- benchmarks/baselines/v0.3.7/
          cp -r benchmarks/baselines/v0.3.7/* target/criterion/

      - name: Run Benchmarks
        run: cargo bench --bench benchmarks -- --baseline v0.3.7

      - name: Check for Regressions
        run: |
          # Parse Criterion output for "Performance has regressed"
          # Fail CI if regression > 10%
          ./scripts/check-performance-regression.sh
```

**Benefits:**
- Automated regression detection on every PR
- Prevents performance regressions from merging
- Historical performance tracking over time

## Best Practices

### 1. Consistent Benchmark Environment

For reliable comparisons:
- **Same Hardware**: Run benchmarks on same machine (or document platform changes)
- **Minimal Load**: Close background applications, disable CPU frequency scaling
- **Stable Power**: Plug in laptop (not battery), disable power management
- **Consistent Build**: Always use `cargo build --release` before benchmarking

### 2. Baseline Management

- **Keep Recent Versions**: Keep last 3-5 version baselines for historical comparison
- **Document Platform**: Always document OS, CPU, RAM, Rust version in README
- **Git Track**: Commit baselines to git for team-wide consistency
- **Delete Obsolete**: Remove baselines >6 months old to save space

### 3. Statistical Validity

- **Multiple Runs**: Criterion runs 100 samples by default (sufficient for most tests)
- **Warmup**: Criterion warms up for 3 seconds (avoids cold-start bias)
- **Outliers**: Criterion detects and handles outliers automatically
- **Significance**: Only trust changes with p < 0.05 (95% confidence)

### 4. Debugging Regressions

When a regression is detected:

1. **Verify Reproducibility**: Run benchmarks again (ensure it's not noise)
2. **Git Bisect**: Use `git bisect` to identify commit causing regression
3. **Profile**: Use `perf` or `flamegraph` to identify hotspots
4. **Compare Flamegraphs**: Visual comparison often reveals bottlenecks
5. **Iterate**: Fix issue, re-benchmark, confirm improvement

**Example Debugging Workflow:**
```bash
# Regression detected: localhost_scan +15% slower
git bisect start
git bisect bad HEAD           # Current commit is slow
git bisect good v0.3.7        # v0.3.7 baseline was fast

# Git bisect will checkout commits, run:
cargo bench --bench benchmarks -- --baseline v0.3.7 localhost_scan
git bisect good  # or git bisect bad

# Once found:
git show <commit_hash>        # Review problematic code
cargo flamegraph --bench benchmarks -- localhost_scan  # Profile
```

## Benchmark Suite Details

### binary_startup (2 tests)

**Purpose**: Measure binary startup overhead (CLI parsing, initialization)

**Tests:**
- `help`: `prtip --help` execution time
- `version`: `prtip --version` execution time

**Why Important**: Startup overhead affects user experience for quick scans. Target: <5ms.

### port_parsing (3 tests)

**Purpose**: Measure CLI argument parsing performance

**Tests:**
- `single_port`: Parse "80"
- `port_range_small`: Parse "1-100"
- `port_list`: Parse "80,443,8080"

**Why Important**: Parsing should be negligible overhead. Target: <1µs.

### localhost_scan (3 tests)

**Purpose**: Measure full scan execution overhead (TCP Connect scan)

**Tests:**
- `single_port`: Scan 127.0.0.1:80
- `three_ports`: Scan 127.0.0.1:80,443,8080
- `port_range_10`: Scan 127.0.0.1:1-10

**Why Important**: Baseline for scan performance. Includes TCP handshake + OS overhead. Target: <10ms for localhost.

### output_formats (2 tests)

**Purpose**: Measure output formatting overhead

**Tests:**
- `text_output`: Human-readable text format
- `json_output`: Machine-readable JSON format

**Why Important**: Output formatting should not dominate scan time. Target: <5ms for typical results.

---

## Hyperfine Baselines (Sprint 5.5.4+)

**New in v0.5.1:** In addition to Criterion micro-benchmarks, we now maintain hyperfine baselines for full-scan regression detection.

**Purpose:**
- Measure **end-to-end** scan performance (not just micro-operations)
- Detect regressions in real-world scan scenarios
- Automated CI/CD regression detection

**Location:** Each baseline is stored in a version-tagged directory:
```
benchmarks/baselines/
├── v0.5.0/  # hyperfine baselines
│   ├── syn-scan-{timestamp}.json
│   ├── connect-scan-{timestamp}.json
│   ├── service-detection-{timestamp}.json
│   └── baseline-metadata.md
├── v0.5.1/  # (next version)
└── v0.3.7/  # Criterion baselines (this section)
```

**Creating Hyperfine Baseline:**
```bash
# Use automated script (recommended)
cd benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts
./create-baseline.sh v0.5.1

# Manual workflow (not recommended)
./run-all-benchmarks.sh
mkdir -p benchmarks/baselines/v0.5.1
cp results/*.json benchmarks/baselines/v0.5.1/
```

**Regression Detection:**
```bash
# Compare current results against baseline
./benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/analyze-results.sh \
    benchmarks/baselines/v0.5.0 \
    benchmarks/05-Sprint5.9-Benchmarking-Framework/results

# Exit codes:
#   0 = PASS or IMPROVED
#   1 = WARN (5-10% slower)
#   2 = FAIL (>10% slower, regression)
```

**CI Integration:**
- Workflow: `.github/workflows/benchmarks.yml`
- Schedule: Weekly (Sunday 00:00 UTC)
- PR comments: Automated performance comparison
- Blocking: PRs with >10% regression fail CI

**See:** `docs/31-BENCHMARKING-GUIDE.md` for full documentation

---

## Version History

### v0.5.0 (2025-11-09) - Sprint 5.5.4
- **Hyperfine baseline system implemented**
- 20 benchmark scenarios (expanded from 8)
- Automated regression detection (5% warn, 10% fail)
- CI/CD integration (GitHub Actions)
- Platform: Intel i9-10850K, 32GB RAM, Linux 6.17.7
- Key metrics:
  - SYN scan (1K ports): 98ms
  - Connect scan (3 ports): 45ms
  - Service detection: 235ms
  - Rate limiter overhead: -1.8% (faster than unlimited!)

### v0.3.7 (2025-10-13)
- **Initial Criterion baseline capture**
- Platform: Intel i9-10850K, 62GB RAM, Linux 6.17.1
- Rust: 1.90.0
- Key metrics:
  - Binary startup: 2.2ms
  - Port parsing: 1.7ns (sub-nanosecond)
  - Localhost scan (single): 5.3ms
  - JSON output: 2.1ms

## Future Enhancements

**Planned Improvements:**
1. **CI Integration**: Automated regression checks on PRs
2. **Multi-Platform**: Baselines for Linux, macOS, Windows, FreeBSD
3. **Real Network**: Benchmarks against real targets (not just localhost)
4. **Large Scans**: 1K ports, 10K ports, 65K ports benchmarks
5. **Scan Types**: SYN, UDP, stealth scan benchmarks
6. **Flamegraph Storage**: Store flamegraphs for visual regression analysis
7. **Trend Graphs**: Performance over time visualization

## References

- **Criterion.rs Documentation**: https://bheisler.github.io/criterion.rs/book/
- **cargo-criterion**: https://github.com/bheisler/cargo-criterion
- **ProRT-IP Performance Docs**: `docs/07-PERFORMANCE.md`
- **ProRT-IP Testing Docs**: `docs/17-TESTING-INFRASTRUCTURE.md`

---

**Last Updated**: 2025-10-13
**Maintainer**: ProRT-IP Contributors
**Questions**: See `docs/09-FAQ.md` or open a GitHub issue
