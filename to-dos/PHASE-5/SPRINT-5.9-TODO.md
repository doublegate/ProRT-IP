# Sprint 5.9: Benchmarking Framework - Todo List

**Status:** 📋 NOT STARTED
**Estimated Duration:** 15-20 hours (Q1 2026)
**Sprint Priority:** HIGH (performance validation, regression detection)
**Phase:** 5 (Advanced Features)
**Version Target:** v0.4.9 or v0.5.0

---

## Executive Summary

**Strategic Value:** Continuous performance validation infrastructure enabling regression detection before shipping, competitive validation (vs Nmap, Masscan, RustScan), and baseline establishment for future optimizations. Demonstrates performance-first engineering culture. Provides reproducible evidence for performance claims. Critical for v0.5.0 release credibility.

**Rationale:** Benchmarking framework scheduled after plugin system (Sprint 5.8) to benchmark complete feature set including plugin overhead. ProRT-IP claims high performance ("10M+ pps", "29x faster than Nmap") but lacks systematic benchmarking infrastructure. Current state: ad-hoc hyperfine tests, no regression detection, no historical tracking. This sprint creates comprehensive suite: 8+ scenarios covering throughput (pps), latency (scan duration), overhead (rate limiting -1.8%, plugins), and accuracy (detection rate).

**Key Decisions:**
- **Tool:** hyperfine (chosen over Criterion.rs for external binary benchmarking)
- **Scenarios:** 8 minimum (SYN, Connect, UDP, Service Detection, IPv6, Idle, Rate Limiting, TLS)
- **Baseline:** v0.4.8 performance metrics
- **CI Integration:** GitHub Actions workflow, regression threshold (>5% slowdown = warning, >10% = failure)
- **Storage:** JSON results in `benchmarks/05-Sprint5.9-Benchmarking-Framework/results/`
- **Regression Detection:** Python script comparing current vs baseline with statistical tests

---

## Progress Tracking

**Total Items:** 37 tasks across 8 phases
**Completed:** 0 / 37 (0%)
**In Progress:** 0
**Remaining:** 37
**Progress:** ░░░░░░░░░░░░ 0%

**Estimated Effort Breakdown:**
- Phase 1: Planning & Design (3h)
- Phase 2: Benchmark Suite Implementation (5h)
- Phase 3: Hyperfine Integration (2h)
- Phase 4: Baseline Establishment (2h)
- Phase 5: CI/CD Integration (3h)
- Phase 6: Regression Detection (3h)
- Phase 7: Documentation (2h)
- Phase 8: Validation & Completion (1h)
- **Contingency:** +2-3h for unexpected issues
- **Total:** 15-20h

---

## Prerequisites & Dependencies

### Requires Completed
- ✅ Sprint 5.1: IPv6 Scanner Completion (benchmark IPv6 performance)
- ✅ Sprint 5.2: Service Detection Enhancement (benchmark detection accuracy)
- ✅ Sprint 5.3: Idle Scan Implementation (benchmark idle scan overhead)
- ✅ Sprint 5.X: AdaptiveRateLimiterV3 (-1.8% overhead validation needed)
- ✅ Sprint 5.5: TLS Certificate Analysis (benchmark TLS parsing 1.33μs claim)
- ✅ Sprint 5.6: Code Coverage (54.92% baseline for benchmark coverage)
- ✅ Sprint 5.7: Fuzz Testing (0 crashes validation)
- ✅ Sprint 5.8: Plugin System (benchmark plugin overhead)

### Blocks
- 🔒 Sprint 5.10: Documentation & Polish (performance metrics needed for docs)
- 🔒 v0.5.0 Release (performance validation required before release)
- 🔒 Phase 6: TUI Interface (performance baseline for comparison)

### External Dependencies
- **hyperfine:** Command-line benchmarking tool
  - Version: `hyperfine >= 1.16.0` (latest stable)
  - Install: `cargo install hyperfine` OR `brew install hyperfine`
  - Features: Statistical rigor (mean, stddev, outliers), JSON export, warmup runs
  - Rationale: External binary benchmarking, cross-platform, industry-standard
- **Python 3.8+:** For regression detection scripts
  - Libraries: `pandas`, `numpy`, `scipy` (statistical tests)
  - Install: `pip install pandas numpy scipy`
- **jq:** JSON processing (optional, for manual analysis)
  - Install: `brew install jq` OR `apt install jq`

### System Requirements
- **Platform:** Linux (primary), macOS (secondary), Windows (optional)
- **Hardware:** 4 CPU cores minimum (parallel benchmarking), 8GB RAM
- **Network:** Loopback interface (127.0.0.1) for localhost benchmarks
- **Storage:** 1GB for benchmark results and baselines

---

## Phase 1: Planning & Design (3 hours)

**Duration:** 3 hours
**Status:** 📋 Not Started
**Progress:** 0 / 5 (0%)

### Research & Strategy (1h)

- [ ] **Task 1.1.1:** Research hyperfine capabilities and best practices (30m)
  - Review: hyperfine documentation, statistical methods (mean, stddev, outlier detection)
  - Research: Warmup runs (--warmup flag), min runs (--min-runs), parameter scanning
  - Research: JSON export format (--export-json), markdown reports (--export-markdown)
  - Compare: hyperfine vs Criterion.rs (external binary vs library benchmarking)
  - Decision: hyperfine for all benchmarks (external binary, no code changes needed)
  - Deliverable: `/tmp/ProRT-IP/HYPERFINE-RESEARCH.md` (~150 lines)

- [ ] **Task 1.1.2:** Define benchmark categories and scenarios (30m)
  - Category 1: **Throughput** - Packets per second, ports scanned per second
  - Category 2: **Latency** - Scan duration (total time), time to first result
  - Category 3: **Overhead** - Rate limiting (-1.8% validation), plugin overhead
  - Category 4: **Accuracy** - Service detection rate (85-90%), false positives
  - Category 5: **Features** - IPv6 overhead (15%), Idle scan (500-800ms), TLS parsing (1.33μs)
  - Scenarios: SYN, Connect, UDP, Service Detection, IPv6, Idle, Rate Limiting, TLS (8 minimum)
  - Deliverable: Category definitions in research doc

### Benchmark Scenarios Design (2h)

- [ ] **Task 1.2.1:** Design 8 core benchmark scenarios (1.5 hours)
  - **Scenario 1: SYN Scan Performance** (validate "10M+ pps" claim)
    - Command: `prtip -sS -p 1-1000 127.0.0.1 --rate-limit 0`
    - Metric: Scan duration (lower = better)
    - Target: <100ms for 1,000 ports on localhost
    - Baseline: Establish v0.4.8 performance

  - **Scenario 2: Connect Scan Performance** (real-world baseline)
    - Command: `prtip -sT -p 80,443,8080 127.0.0.1`
    - Metric: Scan duration for 3 common ports
    - Target: <50ms
    - Comparison: vs Nmap -sT (should be faster)

  - **Scenario 3: UDP Scan Performance** (slow protocol validation)
    - Command: `prtip -sU -p 53,161,123 127.0.0.1`
    - Metric: Scan duration for 3 UDP ports
    - Target: <500ms (UDP is ~10-100x slower than TCP)
    - Note: ICMP rate limiting affects performance

  - **Scenario 4: Service Detection Overhead** (85-90% accuracy validation)
    - Command: `prtip -sV -p 22,80,443 127.0.0.1`
    - Metric: Detection overhead vs plain scan
    - Target: <10% overhead (e.g., 50ms → 55ms)
    - Validation: Accuracy ≥85% (manual check)

  - **Scenario 5: IPv6 Scan Overhead** (15% overhead validation)
    - Command: `prtip -6 -sS -p 1-1000 ::1`
    - Metric: IPv6 scan duration vs IPv4 baseline
    - Target: <15% slower than IPv4 (v0.4.1 claim)
    - Comparison: Scenario 1 (IPv4) vs Scenario 5 (IPv6)

  - **Scenario 6: Idle Scan Performance** (500-800ms per port validation)
    - Command: `prtip -sI <zombie-ip> -p 80,443,8080 <target>`
    - Metric: Time per port scanned
    - Target: 500-800ms per port (v0.4.3 claim)
    - Note: Requires zombie host setup

  - **Scenario 7: Rate Limiting Overhead** (-1.8% overhead validation)
    - Commands:
      - `prtip -sS -p 1-1000 127.0.0.1 --rate-limit 0` (no limiting)
      - `prtip -sS -p 1-1000 127.0.0.1 --rate-limit 10000` (V3 limiter)
    - Metric: Overhead = (limited_time - baseline_time) / baseline_time * 100
    - Target: <5% overhead (claimed -1.8%)
    - Validation: Sprint 5.X claim

  - **Scenario 8: TLS Certificate Parsing** (1.33μs claim validation)
    - Command: `prtip -sV -p 443 badssl.com --tls-cert-analysis`
    - Metric: Average time per certificate parsed
    - Target: ~1.33μs (v0.4.5 claim)
    - Extraction: From verbose logs or instrumentation

  - **Deliverable:** BENCHMARK-SCENARIOS.md (~400 lines with details)

- [ ] **Task 1.2.2:** Document baseline performance targets (0.5 hours)
  - Extract: Claims from CHANGELOG, README, guides (e.g., "10M+ pps", "-1.8% overhead")
  - Format: Table with scenario, metric, target, v0.4.8 baseline (TBD)
  - Purpose: Regression detection thresholds
  - Deliverable: PERFORMANCE-TARGETS.md (~200 lines)

---

## Phase 2: Benchmark Suite Implementation (5 hours)

**Duration:** 5 hours
**Status:** 📋 Not Started
**Progress:** 0 / 8 (0%)

### Directory Structure Setup (0.5h)

- [ ] **Task 2.1.1:** Create benchmark framework directory (0.5 hours)
  - Create: `benchmarks/05-Sprint5.9-Benchmarking-Framework/`
  - Subdirectories:
    - `scripts/` - Hyperfine runner scripts
    - `configs/` - JSON configs for each scenario
    - `baselines/` - Baseline JSON files (v0.4.8)
    - `results/` - Current run results (date-stamped)
    - `reports/` - Markdown analysis reports
  - Create: README.md (framework overview, usage instructions)
  - Deliverable: Directory structure (~5 directories, 1 README)

### Hyperfine Scripts (3.5h)

- [ ] **Task 2.2.1:** Scenario 1 - SYN Scan script (0.5 hours)
  - Script: `scripts/01-syn-scan-1000-ports.sh`
  - Command: `hyperfine --warmup 3 --runs 10 \
               --export-json results/syn-scan-$(date +%Y%m%d).json \
               './target/release/prtip -sS -p 1-1000 127.0.0.1 --rate-limit 0'`
  - Flags:
    - `--warmup 3`: 3 warmup runs (stabilize caches, JIT)
    - `--runs 10`: 10 measurement runs (statistical significance)
    - `--export-json`: Machine-readable results
  - Deliverable: Shell script (~30 lines)

- [ ] **Task 2.2.2:** Scenario 2 - Connect Scan script (0.25 hours)
  - Script: `scripts/02-connect-scan-common-ports.sh`
  - Command: `hyperfine --warmup 3 --runs 10 \
               --export-json results/connect-scan-$(date +%Y%m%d).json \
               './target/release/prtip -sT -p 80,443,8080 127.0.0.1'`
  - Deliverable: Shell script (~30 lines)

- [ ] **Task 2.2.3:** Scenario 3 - UDP Scan script (0.25 hours)
  - Script: `scripts/03-udp-scan-dns-snmp-ntp.sh`
  - Command: `hyperfine --warmup 3 --runs 10 \
               --export-json results/udp-scan-$(date +%Y%m%d).json \
               './target/release/prtip -sU -p 53,161,123 127.0.0.1'`
  - Note: UDP slower (10-100x), expect 500ms+ duration
  - Deliverable: Shell script (~30 lines)

- [ ] **Task 2.2.4:** Scenario 4 - Service Detection script (0.5 hours)
  - Script: `scripts/04-service-detection-overhead.sh`
  - Commands (parallel comparison):
    - Baseline: `prtip -sS -p 22,80,443 127.0.0.1` (no -sV)
    - Detection: `prtip -sV -p 22,80,443 127.0.0.1` (with -sV)
  - Command: `hyperfine --warmup 3 --runs 10 \
               --export-json results/service-detection-$(date +%Y%m%d).json \
               './target/release/prtip -sS -p 22,80,443 127.0.0.1' \
               './target/release/prtip -sV -p 22,80,443 127.0.0.1'`
  - Metric: Overhead = (detection_mean - baseline_mean) / baseline_mean * 100
  - Deliverable: Shell script (~40 lines with overhead calculation)

- [ ] **Task 2.2.5:** Scenario 5 - IPv6 Overhead script (0.5 hours)
  - Script: `scripts/05-ipv6-overhead.sh`
  - Commands (parallel comparison):
    - IPv4: `prtip -4 -sS -p 1-1000 127.0.0.1`
    - IPv6: `prtip -6 -sS -p 1-1000 ::1`
  - Command: `hyperfine --warmup 3 --runs 10 \
               --export-json results/ipv6-overhead-$(date +%Y%m%d).json \
               './target/release/prtip -4 -sS -p 1-1000 127.0.0.1' \
               './target/release/prtip -6 -sS -p 1-1000 ::1'`
  - Target: IPv6 <15% slower than IPv4
  - Deliverable: Shell script (~40 lines)

- [ ] **Task 2.2.6:** Scenario 6 - Idle Scan script (0.5 hours)
  - Script: `scripts/06-idle-scan-timing.sh`
  - Command: `hyperfine --warmup 1 --runs 5 \
               --export-json results/idle-scan-$(date +%Y%m%d).json \
               './target/release/prtip -sI <zombie-ip> -p 80,443,8080 <target>'`
  - Note: Fewer runs (5) due to slow nature (500-800ms/port)
  - Setup: Document zombie host requirements
  - Deliverable: Shell script + ZOMBIE-SETUP.md (~30 lines script, ~150 lines doc)

- [ ] **Task 2.2.7:** Scenario 7 - Rate Limiting Overhead script (0.5 hours)
  - Script: `scripts/07-rate-limiting-overhead.sh`
  - Commands (parallel comparison):
    - No limit: `prtip -sS -p 1-1000 127.0.0.1 --rate-limit 0`
    - V3 limiter: `prtip -sS -p 1-1000 127.0.0.1 --rate-limit 10000`
  - Command: `hyperfine --warmup 3 --runs 10 \
               --export-json results/rate-limiting-$(date +%Y%m%d).json \
               './target/release/prtip -sS -p 1-1000 127.0.0.1 --rate-limit 0' \
               './target/release/prtip -sS -p 1-1000 127.0.0.1 --rate-limit 10000'`
  - Validation: Overhead should be <5% (claimed -1.8%)
  - Deliverable: Shell script (~40 lines)

- [ ] **Task 2.2.8:** Scenario 8 - TLS Parsing script (0.5 hours)
  - Script: `scripts/08-tls-cert-parsing.sh`
  - Command: `hyperfine --warmup 3 --runs 10 \
               --export-json results/tls-parsing-$(date +%Y%m%d).json \
               './target/release/prtip -sV -p 443 badssl.com --tls-cert-analysis'`
  - Metric: Average time per certificate (extract from verbose logs)
  - Target: ~1.33μs (v0.4.5 claim)
  - Note: Requires network access to badssl.com
  - Deliverable: Shell script (~35 lines)

### Orchestrator Script (1h)

- [ ] **Task 2.3.1:** Create run-all-benchmarks.sh orchestrator (1 hour)
  - Script: `scripts/run-all-benchmarks.sh`
  - Functionality:
    - Check: prtip binary exists (`./target/release/prtip`)
    - Create: Results directory with timestamp (`results/YYYY-MM-DD/`)
    - Run: All 8 benchmark scripts sequentially
    - Aggregate: Combine JSON results into single summary
    - Generate: Markdown report with summary statistics
    - Exit: Non-zero if any benchmark fails
  - Arguments:
    - `--baseline`: Save results as baseline (for v0.4.8)
    - `--compare <baseline>`: Compare against baseline file
  - Deliverable: Shell script (~150 lines)

---

## Phase 3: Hyperfine Integration (2 hours)

**Duration:** 2 hours
**Status:** 📋 Not Started
**Progress:** 0 / 3 (0%)

### Hyperfine Configuration (1h)

- [ ] **Task 3.1.1:** Create hyperfine config templates (0.5 hours)
  - Format: JSON config files for each scenario (optional, for reproducibility)
  - Example: `configs/syn-scan.json`
    ```json
    {
      "scenario": "SYN Scan 1000 Ports",
      "command": "./target/release/prtip -sS -p 1-1000 127.0.0.1 --rate-limit 0",
      "warmup": 3,
      "min_runs": 10,
      "export": "results/syn-scan.json"
    }
    ```
  - Purpose: Documentation, reproducibility, CI integration
  - Deliverable: 8 JSON config files (~20 lines each)

- [ ] **Task 3.1.2:** Add parameter scanning for variable workloads (0.5 hours)
  - Use: `--parameter-scan <var> <min> <max> <step>` flag
  - Example: `hyperfine --parameter-scan ports 100 1000 100 \
               './target/release/prtip -sS -p {ports} 127.0.0.1'`
  - Scenarios: Test scaling behavior (100, 200, ..., 1000 ports)
  - Deliverable: `scripts/09-parameter-scan-ports.sh` (~50 lines)

### JSON Export and Parsing (1h)

- [ ] **Task 3.2.1:** Validate JSON export format (0.5 hours)
  - Run: Sample benchmark, examine JSON structure
  - Fields: `command`, `mean`, `stddev`, `median`, `user`, `system`, `min`, `max`, `times`
  - Document: JSON schema for regression detection script
  - Deliverable: JSON-SCHEMA.md (~100 lines)

- [ ] **Task 3.2.2:** Create JSON aggregation script (0.5 hours)
  - Script: `scripts/aggregate-results.sh`
  - Input: Multiple JSON files (8 scenarios)
  - Output: Single aggregated JSON with all results
  - Purpose: Simplified regression analysis
  - Deliverable: Shell script (~80 lines, uses jq)

---

## Phase 4: Baseline Establishment (2 hours)

**Duration:** 2 hours
**Status:** 📋 Not Started
**Progress:** 0 / 3 (0%)

### v0.4.8 Baseline (1.5h)

- [ ] **Task 4.1.1:** Build release binary for v0.4.8 (0.25 hours)
  - Checkout: v0.4.8 tag (or current HEAD if not released yet)
  - Build: `cargo build --release`
  - Verify: `./target/release/prtip --version` shows v0.4.8
  - Deliverable: Release binary in `./target/release/prtip`

- [ ] **Task 4.1.2:** Run all benchmarks for baseline (1 hour)
  - Execute: `./scripts/run-all-benchmarks.sh --baseline`
  - Platform: Linux (Ubuntu 22.04 or similar)
  - Hardware: Document CPU, RAM, network specs
  - Results: Save to `baselines/baseline-v0.4.8.json`
  - Manual: Validate results look reasonable (no outliers, consistent means)
  - Deliverable: baseline-v0.4.8.json (~500-1000 lines JSON)

- [ ] **Task 4.1.3:** Document baseline metadata (0.25 hours)
  - Create: `baselines/baseline-v0.4.8-metadata.md`
  - Content:
    - Date: YYYY-MM-DD
    - Version: v0.4.8
    - Platform: OS, kernel version
    - Hardware: CPU model, cores, RAM, disk
    - hyperfine: Version
    - Results: Path to JSON file
    - Notes: Any caveats, special setup
  - Deliverable: Metadata markdown (~100 lines)

### Cross-Platform Baselines (Optional, 0.5h)

- [ ] **Task 4.2.1:** Establish macOS baseline (optional) (0.25 hours)
  - Platform: macOS (M1 or Intel)
  - Run: Same benchmark suite
  - Results: `baselines/baseline-v0.4.8-macos.json`
  - Note: Expected performance differences (M1 faster on some, slower on UDP)
  - Deliverable: macOS baseline JSON + metadata

- [ ] **Task 4.2.2:** Establish Windows baseline (optional) (0.25 hours)
  - Platform: Windows 11 with Npcap
  - Run: Same benchmark suite (may need WSL for some scripts)
  - Results: `baselines/baseline-v0.4.8-windows.json`
  - Note: Windows loopback limitations (4 SYN tests may fail)
  - Deliverable: Windows baseline JSON + metadata

---

## Phase 5: CI/CD Integration (3 hours)

**Duration:** 3 hours
**Status:** 📋 Not Started
**Progress:** 0 / 4 (0%)

### GitHub Actions Workflow (2h)

- [ ] **Task 5.1.1:** Create benchmark workflow file (1 hour)
  - File: `.github/workflows/benchmark.yml`
  - Trigger:
    - `push` to main (after test workflow passes)
    - `pull_request` (performance validation)
    - `workflow_dispatch` (manual runs)
    - `schedule`: Weekly (Monday 00:00 UTC)
  - Jobs:
    - `benchmark-linux`: Ubuntu 22.04, runs all 8 scenarios
    - `benchmark-macos`: macOS-latest (optional)
  - Steps:
    1. Checkout code
    2. Setup Rust (stable)
    3. Build release binary (`cargo build --release`)
    4. Install hyperfine (`cargo install hyperfine` or `brew install hyperfine`)
    5. Run benchmark suite (`./scripts/run-all-benchmarks.sh`)
    6. Compare against baseline (`./scripts/analyze-results.sh`)
    7. Upload results as artifacts
    8. Comment on PR with summary (if PR context)
    9. Fail if regression >10%
  - Deliverable: `.github/workflows/benchmark.yml` (~120 lines)

- [ ] **Task 5.1.2:** Add benchmark job dependencies (0.5 hours)
  - Dependency: Run after test.yml passes (all tests green)
  - Timeout: 15 minutes (benchmark suite should complete in <10 min)
  - Cache: Rust target directory, hyperfine binary
  - Artifacts: Upload results JSON, markdown reports (7 day retention)
  - Deliverable: Job dependencies + caching in workflow

- [ ] **Task 5.1.3:** Implement PR comment with results (0.5 hours)
  - Use: GitHub Actions `actions/github-script` to comment on PR
  - Content:
    - Summary table (scenario, baseline, current, diff, status)
    - Overall status (PASS / WARN / FAIL)
    - Link to detailed report artifact
  - Example:
    ```markdown
    ## Benchmark Results

    | Scenario | Baseline | Current | Diff | Status |
    |----------|----------|---------|------|--------|
    | SYN Scan | 98ms | 95ms | -3.1% | ✅ IMPROVED |
    | Connect  | 45ms | 46ms | +2.2% | ✅ PASS |
    | UDP      | 520ms | 540ms | +3.8% | ✅ PASS |
    | Service  | 55ms | 62ms | +12.7% | ❌ REGRESSION |

    **Overall:** 1 REGRESSION detected (Service Detection)
    ```
  - Deliverable: PR comment action in workflow (~30 lines)

### Regression Thresholds (1h)

- [ ] **Task 5.2.1:** Define regression detection thresholds (0.5 hours)
  - Thresholds:
    - **PASS:** <5% slower (within noise)
    - **WARN:** 5-10% slower (investigate)
    - **FAIL:** >10% slower (regression, CI fails)
    - **IMPROVED:** Faster than baseline (celebrate!)
  - Implementation: In `scripts/analyze-results.sh`
  - Exit codes:
    - 0: PASS or IMPROVED
    - 1: WARN (exit 0 but log warning)
    - 2: FAIL (exit 1, fails CI)
  - Deliverable: Threshold definitions in script

- [ ] **Task 5.2.2:** Implement statistical significance tests (0.5 hours)
  - Use: Python scipy for t-test (compare means)
  - Test: Two-sample t-test (current vs baseline)
  - Significance: p-value < 0.05 (statistically significant difference)
  - Logic: If p < 0.05 AND diff > 5%, then regression
  - Deliverable: Python script `scripts/statistical-test.py` (~80 lines)

---

## Phase 6: Regression Detection (3 hours)

**Duration:** 3 hours
**Status:** 📋 Not Started
**Progress:** 0 / 4 (0%)

### Analysis Scripts (2h)

- [ ] **Task 6.1.1:** Create analyze-results.sh main script (1 hour)
  - Script: `scripts/analyze-results.sh <baseline> <current>`
  - Functionality:
    1. Load baseline JSON
    2. Load current JSON
    3. For each scenario:
       - Calculate percentage difference: `(current_mean - baseline_mean) / baseline_mean * 100`
       - Run statistical test (t-test) for significance
       - Categorize: PASS / WARN / FAIL / IMPROVED
       - Store result with metadata
    4. Generate summary report (markdown table)
    5. Exit with appropriate code (0/1/2)
  - Output: Markdown report to stdout, exit code for CI
  - Deliverable: Shell script (~200 lines)

- [ ] **Task 6.1.2:** Create comparison-report.sh (0.5 hours)
  - Script: `scripts/comparison-report.sh <baseline> <current>`
  - Output: Markdown report with detailed comparison
  - Sections:
    - Executive Summary (overall status)
    - Per-Scenario Analysis (table with stats)
    - Regressions Detected (if any)
    - Improvements Detected (if any)
    - Recommendations (investigate, celebrate, etc.)
  - Format: Ready for GitHub comment or documentation
  - Deliverable: Shell script (~120 lines)

- [ ] **Task 6.1.3:** Add trend analysis (optional) (0.5 hours)
  - Script: `scripts/trend-analysis.py`
  - Input: Multiple historical results (baseline, v0.4.8, v0.4.9, ...)
  - Output: Trend chart (ASCII or HTML)
  - Analysis: Performance over time (improving / degrading)
  - Purpose: Long-term performance tracking
  - Deliverable: Python script (~150 lines) + chart visualization

### Visualization (1h)

- [ ] **Task 6.2.1:** Generate performance charts (1 hour)
  - Tool: Python matplotlib or gnuplot (or ASCII art for simplicity)
  - Charts:
    - Bar chart: Baseline vs Current (all scenarios)
    - Box plot: Distribution of run times (outliers visible)
    - Line chart: Historical trend (if multiple baselines)
  - Format: PNG images, HTML, or ASCII art (for terminal)
  - Purpose: Visual regression detection, documentation
  - Deliverable: Visualization script `scripts/visualize-results.py` (~200 lines)

---

## Phase 7: Documentation (2 hours)

**Duration:** 2 hours
**Status:** 📋 Not Started
**Progress:** 0 / 4 (0%)

### Benchmarking Guide (1.5h)

- [ ] **Task 7.1.1:** Create comprehensive benchmarking guide (1.5 hours)
  - File: `docs/31-BENCHMARKING-GUIDE.md`
  - Section 1: Overview (~100 lines)
    - Why benchmarking (regression detection, validation)
    - What we measure (throughput, latency, overhead)
    - How we use it (CI/CD, release validation)
  - Section 2: Architecture (~100 lines)
    - Benchmark suite structure
    - Hyperfine integration
    - Regression detection algorithm
  - Section 3: Running Benchmarks Locally (~150 lines)
    - Prerequisites (hyperfine, Python)
    - Quick start: Single scenario
    - Full suite: run-all-benchmarks.sh
    - Baseline establishment: --baseline flag
  - Section 4: Benchmark Scenarios (~200 lines)
    - 8 scenarios detailed (command, metric, target)
    - Rationale for each scenario
    - How to interpret results
  - Section 5: CI Integration (~100 lines)
    - GitHub Actions workflow
    - Automated regression detection
    - PR comments with results
  - Section 6: Interpreting Results (~100 lines)
    - Reading hyperfine output
    - Statistical significance (t-test)
    - Regression thresholds (5%, 10%)
  - Section 7: Adding New Benchmarks (~100 lines)
    - Creating new scenarios
    - Hyperfine configuration
    - CI integration steps
  - Section 8: Troubleshooting (~50 lines)
    - Common issues (hyperfine not found, binary not built)
    - Solutions (install hyperfine, cargo build --release)
  - Deliverable: `docs/31-BENCHMARKING-GUIDE.md` (~900 lines)

### Additional Documentation (0.5h)

- [ ] **Task 7.2.1:** Update README.md with benchmarking section (0.25 hours)
  - Section: Performance Benchmarking
  - Content:
    - Overview (1-2 sentences)
    - Quick start (how to run benchmarks)
    - Link to comprehensive guide (31-BENCHMARKING-GUIDE.md)
  - Deliverable: README.md (~30 lines added)

- [ ] **Task 7.2.2:** Update CHANGELOG.md with Sprint 5.9 entry (0.25 hours)
  - Version: v0.4.9 or v0.5.0 (depending on release timing)
  - Entry: Sprint 5.9 - Benchmarking Framework
  - Content:
    - Added: 8 benchmark scenarios
    - Added: Hyperfine integration
    - Added: CI/CD regression detection
    - Added: Performance baselines for v0.4.8
    - Documentation: 31-BENCHMARKING-GUIDE.md
  - Deliverable: CHANGELOG.md (~50 lines)

---

## Phase 8: Validation & Completion (1 hour)

**Duration:** 1 hour
**Status:** 📋 Not Started
**Progress:** 0 / 3 (0%)

### Final Validation (0.5h)

- [ ] **Task 8.1.1:** Run full benchmark suite locally (0.25 hours)
  - Execute: `./scripts/run-all-benchmarks.sh`
  - Verify: All 8 scenarios complete successfully
  - Verify: JSON results generated
  - Verify: Markdown report readable
  - Deliverable: Successful local run

- [ ] **Task 8.1.2:** Trigger CI benchmark workflow (0.25 hours)
  - Push: Commit to trigger GitHub Actions
  - Monitor: Workflow execution (should complete in <15 min)
  - Verify: All jobs pass
  - Verify: Artifacts uploaded (results JSON, reports)
  - Deliverable: Passing CI benchmark workflow

### Sprint Completion (0.5h)

- [ ] **Task 8.2.1:** Create sprint completion report (0.5 hours)
  - File: `/tmp/ProRT-IP/SPRINT-5.9-COMPLETE.md`
  - Sections:
    - Executive Summary (what was delivered)
    - Deliverables Achieved (code, scripts, docs)
    - Benchmarking Infrastructure Overview (architecture)
    - Example Results (baseline metrics from v0.4.8)
    - Files Changed Summary (new files, lines added)
    - Lessons Learned (what worked, what didn't)
    - Future Work (v0.6.0: performance dashboard, more scenarios)
  - Length: ~600-800 lines
  - Grade: A+ (comprehensive, professional)
  - Deliverable: Sprint completion report

---

## Success Criteria

### Functional Requirements

**Benchmarking Infrastructure:**
- [ ] 8+ benchmark scenarios implemented and functional
- [ ] Hyperfine integration working (JSON export, statistical rigor)
- [ ] Baseline established for v0.4.8
- [ ] CI/CD integration operational (GitHub Actions workflow)
- [ ] Regression detection accurate (tested with mock data)

**Regression Detection:**
- [ ] Thresholds defined (5% warn, 10% fail)
- [ ] Statistical tests implemented (t-test for significance)
- [ ] PR comments working (automated feedback)
- [ ] Exit codes correct (0 pass, 1 warn, 2 fail)

**Documentation:**
- [ ] Comprehensive guide complete (31-BENCHMARKING-GUIDE.md, 900+ lines)
- [ ] README updated with benchmarking section
- [ ] CHANGELOG updated with Sprint 5.9 entry
- [ ] All scripts documented (usage, examples)

### Quality Requirements

**Code Quality:**
- [ ] All scripts executable (`chmod +x`)
- [ ] Error handling in all bash scripts (set -e, validation)
- [ ] JSON parsing validated (jq or Python)
- [ ] No hardcoded paths (use relative paths, `$(dirname "$0")`)
- [ ] Cross-platform where possible (Linux priority, macOS compatible)

**Benchmark Quality:**
- [ ] Statistical significance (p < 0.05 for regressions)
- [ ] Minimum 10 runs per scenario (hyperfine --runs 10)
- [ ] Warmup runs to stabilize caches (hyperfine --warmup 3)
- [ ] Reproducible results (stddev <5% of mean)
- [ ] Realistic workloads (not synthetic micro-benchmarks)

**Documentation Quality:**
- [ ] Clear explanations for all concepts
- [ ] Code examples for all scenarios
- [ ] Troubleshooting section with common issues
- [ ] Links to hyperfine documentation
- [ ] No broken cross-references

### Performance Requirements

**Benchmark Suite:**
- [ ] Total execution time: <10 minutes (all 8 scenarios)
- [ ] Parallel execution where possible (independent scenarios)
- [ ] Minimal overhead (hyperfine itself <1% impact)

**CI/CD:**
- [ ] Workflow timeout: 15 minutes (buffer for variability)
- [ ] Artifact retention: 7 days (balance storage vs history)
- [ ] Regression detection: <30 seconds (fast feedback)

### Documentation Requirements

**Comprehensive Guides:**
- [ ] 31-BENCHMARKING-GUIDE.md complete (900+ lines)
- [ ] All 8 scenarios documented (command, metric, target)
- [ ] API reference complete (hyperfine flags, script arguments)

**Project Documentation:**
- [ ] CHANGELOG entry complete (~50 lines)
- [ ] README updated (~30 lines)
- [ ] Scripts have --help flags

---

## Deliverables Summary

### Code Deliverables

**Benchmark Suite:**
1. `benchmarks/05-Sprint5.9-Benchmarking-Framework/README.md` (overview, ~200 lines)
2. `scripts/01-syn-scan-1000-ports.sh` (~30 lines)
3. `scripts/02-connect-scan-common-ports.sh` (~30 lines)
4. `scripts/03-udp-scan-dns-snmp-ntp.sh` (~30 lines)
5. `scripts/04-service-detection-overhead.sh` (~40 lines)
6. `scripts/05-ipv6-overhead.sh` (~40 lines)
7. `scripts/06-idle-scan-timing.sh` (~30 lines)
8. `scripts/07-rate-limiting-overhead.sh` (~40 lines)
9. `scripts/08-tls-cert-parsing.sh` (~35 lines)
10. `scripts/run-all-benchmarks.sh` (orchestrator, ~150 lines)
11. `scripts/analyze-results.sh` (regression detection, ~200 lines)
12. `scripts/comparison-report.sh` (markdown report, ~120 lines)
13. `scripts/aggregate-results.sh` (JSON aggregation, ~80 lines)
14. `scripts/statistical-test.py` (Python t-test, ~80 lines)
15. `scripts/visualize-results.py` (charts, ~200 lines)

**Total Scripts:** ~1,035 lines across 15 files

**CI/CD:**
16. `.github/workflows/benchmark.yml` (GitHub Actions, ~120 lines)

**Baseline Data:**
17. `baselines/baseline-v0.4.8.json` (hyperfine results, ~500-1000 lines JSON)
18. `baselines/baseline-v0.4.8-metadata.md` (metadata, ~100 lines)

### Documentation Deliverables

**New Documentation:**
1. `docs/31-BENCHMARKING-GUIDE.md` (NEW, ~900 lines)
2. `/tmp/ProRT-IP/HYPERFINE-RESEARCH.md` (internal, ~150 lines)
3. `/tmp/ProRT-IP/BENCHMARK-SCENARIOS.md` (internal, ~400 lines)
4. `/tmp/ProRT-IP/PERFORMANCE-TARGETS.md` (internal, ~200 lines)
5. `/tmp/ProRT-IP/JSON-SCHEMA.md` (internal, ~100 lines)
6. `/tmp/ProRT-IP/ZOMBIE-SETUP.md` (idle scan setup, ~150 lines)
7. `/tmp/ProRT-IP/SPRINT-5.9-COMPLETE.md` (completion report, ~600-800 lines)

**Updated Documentation:**
8. `CHANGELOG.md` (+50 lines)
9. `README.md` (+30 lines)

**Total Documentation:** ~2,580-2,780 lines

### Artifacts

**Performance Data:**
- Baseline JSON for v0.4.8 (8 scenarios × ~100-200 lines each)
- Example run results (for testing)
- Comparison reports (markdown)
- Performance charts (if visualization implemented)

---

## Files to Create/Modify

### New Files (22+)

**Benchmark Scripts (15 files):**
1-9. Individual scenario scripts (8 scenarios + parameter scan)
10. run-all-benchmarks.sh (orchestrator)
11. analyze-results.sh (regression detection)
12. comparison-report.sh (markdown reports)
13. aggregate-results.sh (JSON aggregation)
14. statistical-test.py (Python t-test)
15. visualize-results.py (charts)

**CI/CD (1 file):**
16. `.github/workflows/benchmark.yml`

**Documentation (6+ files):**
17. `docs/31-BENCHMARKING-GUIDE.md`
18. `benchmarks/05-Sprint5.9-Benchmarking-Framework/README.md`
19. `baselines/baseline-v0.4.8-metadata.md`
20-26. Internal docs in `/tmp/ProRT-IP/` (7 files)

**Data Files (1+):**
27. `baselines/baseline-v0.4.8.json`

### Modified Files (2)

1. `CHANGELOG.md` (+50 lines)
2. `README.md` (+30 lines)

---

## Technical Design Notes

### Why hyperfine over Criterion.rs?

**Decision:** Use hyperfine for all benchmarks

**Rationale:**
- **External Binary Benchmarking:** hyperfine benchmarks complete binary (real-world usage)
- **No Code Changes:** Criterion requires adding benches/ directory, dependencies
- **Statistical Rigor:** hyperfine provides mean, stddev, outlier detection (sufficient)
- **JSON Export:** Machine-readable for regression detection
- **Cross-Platform:** Works on Linux, macOS, Windows (Criterion.rs does too)
- **Industry Standard:** Used by ripgrep, fd, bat, and other CLI tools

**Trade-off:** Criterion.rs provides more detailed micro-benchmarks (CPU cycles, cache misses), but hyperfine is better for end-to-end performance validation.

### Benchmark Scenario Selection

**8 Core Scenarios:**
1. **SYN Scan:** Validates "10M+ pps" throughput claim
2. **Connect Scan:** Real-world baseline (most common usage)
3. **UDP Scan:** Slow protocol baseline (10-100x slower than TCP)
4. **Service Detection:** Validates 85-90% accuracy + <10% overhead
5. **IPv6:** Validates 15% overhead claim (Sprint 5.1)
6. **Idle Scan:** Validates 500-800ms per port (Sprint 5.3)
7. **Rate Limiting:** Validates -1.8% overhead claim (Sprint 5.X)
8. **TLS Parsing:** Validates 1.33μs parsing (Sprint 5.5)

**Additional Scenarios (Future):**
- Large network scan (10.0.0.0/24, 254 hosts × 100 ports)
- Plugin overhead (5 plugins loaded vs none)
- Database export (10,000 results to SQLite)

### Regression Detection Algorithm

**Algorithm:**
```python
def detect_regression(baseline, current):
    # 1. Calculate percentage difference
    diff = (current.mean - baseline.mean) / baseline.mean * 100

    # 2. Statistical significance test
    t_stat, p_value = scipy.stats.ttest_ind(baseline.times, current.times)

    # 3. Categorize
    if p_value >= 0.05:
        return "PASS"  # Not statistically significant
    elif diff < -5:
        return "IMPROVED"  # Faster (celebrate!)
    elif diff < 5:
        return "PASS"  # Within noise (<5%)
    elif diff < 10:
        return "WARN"  # Investigate (5-10%)
    else:
        return "FAIL"  # Regression (>10%)
```

**Thresholds:**
- **5%:** Typical noise threshold (CPU frequency scaling, background tasks)
- **10%:** Significant regression (unacceptable performance degradation)
- **Statistical test:** Ensures difference is real, not random variance

### CI/CD Integration Strategy

**Workflow Triggers:**
- **Push to main:** Benchmark after merging (continuous validation)
- **Pull request:** Benchmark before merging (prevent regressions)
- **Scheduled:** Weekly (Monday 00:00 UTC) for long-term trend tracking
- **Manual:** workflow_dispatch (ad-hoc testing)

**Failure Handling:**
- **>10% regression:** Fail CI (blocks merge)
- **5-10% regression:** Pass CI but log warning (investigate)
- **<5% regression:** Pass CI (within noise)

**Artifacts:**
- **Results JSON:** 7-day retention (historical comparison)
- **Markdown reports:** 7-day retention (human-readable summaries)

### Historical Tracking

**Storage Strategy:**
- **Baselines:** `baselines/baseline-vX.Y.Z.json` (one per release)
- **Results:** `results/YYYY-MM-DD/` (date-stamped runs)
- **Trends:** Aggregate baselines for trend analysis (optional Phase 6+)

**Future Enhancements (v0.6.0+):**
- Performance dashboard (GitHub Pages + Chart.js)
- Automated baseline updates (on release tags)
- Multi-platform comparison (Linux vs macOS vs Windows)
- Benchmark history visualization (performance over time)

---

## Risk Assessment

### Risk 1: hyperfine installation issues on CI

**Likelihood:** LOW (10-15%)
- hyperfine is widely used, installation well-documented
- Available via cargo install, apt, brew, GitHub releases

**Impact:** MEDIUM (blocks CI benchmarking)
- Workflow fails if hyperfine not available
- Manual benchmarking still works

**Mitigation:**
- Test installation on CI early (Phase 5 Task 5.1.1)
- Use cargo install (Rust-native, works everywhere)
- Cache hyperfine binary (avoid re-install on every run)
- Document alternative: GitHub release binaries

**Contingency:**
- If cargo install fails: Download from GitHub releases
- If all fail: Skip benchmark job (CI passes, warn user)

### Risk 2: Benchmark results too noisy (high stddev)

**Likelihood:** MEDIUM (30-40%)
- Cloud CI runners have variable performance (CPU throttling, shared resources)
- Network latency varies (internet scans like TLS parsing)

**Impact:** MEDIUM (false regressions, flaky CI)
- High stddev makes regression detection unreliable
- May need to increase threshold (5% → 10%)

**Mitigation:**
- Warmup runs (--warmup 3) to stabilize caches
- Sufficient runs (--runs 10) for statistical significance
- Use localhost scans (minimize network variability)
- Document expected variance per scenario

**Contingency:**
- If stddev >10%: Increase runs (10 → 20)
- If still noisy: Use median instead of mean (more robust)
- If CI too flaky: Run benchmarks on dedicated hardware (Phase 6+)

### Risk 3: Baseline becomes stale (no updates)

**Likelihood:** MEDIUM (40-50%)
- Baselines need manual updates on releases
- Easy to forget to update baseline after performance improvements

**Impact:** LOW (misleading comparisons)
- Benchmarks compare against outdated baseline
- Performance improvements not reflected

**Mitigation:**
- Automate baseline updates on release tags (Phase 6+ enhancement)
- Document process in 31-BENCHMARKING-GUIDE.md
- Remind in release checklist (docs/07-RELEASE-PROCESS.md, future)

**Contingency:**
- If baseline stale: Manually update (run-all-benchmarks.sh --baseline)
- If forgotten: Compare against recent CI runs (artifacts)

### Risk 4: Regression detection too strict (blocks valid merges)

**Likelihood:** MEDIUM (30-40%)
- 10% threshold may be too strict for some scenarios
- Feature additions may legitimately slow scans (e.g., plugin overhead)

**Impact:** MEDIUM (slows development)
- Valid PRs blocked by CI
- Developers frustrated with false failures

**Mitigation:**
- Document threshold rationale in guide
- Allow manual override (workflow_dispatch with --ignore-regression flag)
- Differentiate: Regressions (bad) vs Feature trade-offs (acceptable)

**Contingency:**
- If too strict: Adjust threshold (10% → 15%)
- If specific scenario problematic: Exclude from regression check (e.g., plugin overhead)
- If feature trade-off: Document in PR, override with approval

### Risk 5: Scope creep (too many scenarios)

**Likelihood:** LOW (15-20%)
- Tempting to add many scenarios (10+, 20+)
- Each scenario adds maintenance burden

**Impact:** MEDIUM (delayed sprint)
- More scenarios = more scripts, more maintenance
- CI time increases (>15 min workflow timeout)

**Mitigation:**
- MVP focus: 8 scenarios minimum (covers core claims)
- Defer additional scenarios to Sprint 5.9.1 or v0.6.0
- Prioritize: Scenarios validating specific claims (e.g., "10M+ pps")

**Contingency:**
- If time-constrained: Cut to 6 scenarios (remove Idle, TLS)
- If scope increases: Extend sprint to 20-25h (within contingency)

---

## Research References

**hyperfine Documentation:**
- Official docs: https://github.com/sharkdp/hyperfine
- Examples: https://github.com/sharkdp/hyperfine#usage
- Statistical methods: Mean, stddev, outlier detection (IQR method)
- JSON export: https://github.com/sharkdp/hyperfine#json

**Benchmark Best Practices:**
- Warmup runs: https://pyperf.readthedocs.io/en/latest/api.html#warmup
- Statistical significance: t-test, p-value < 0.05
- Reproducibility: Pin CPU frequency, disable turbo boost (optional)

**Regression Detection:**
- Thresholds: Industry standard ~5-10% for performance regressions
- Statistical tests: scipy.stats.ttest_ind (two-sample t-test)
- Visualization: matplotlib, gnuplot

**CI/CD Integration:**
- GitHub Actions: https://docs.github.com/en/actions
- Artifacts: https://docs.github.com/en/actions/using-workflows/storing-workflow-data-as-artifacts
- PR comments: https://github.com/actions/github-script

---

## Open Questions

### Q1: Use hyperfine vs Criterion.rs?

**Options:**
1. **hyperfine** - External binary benchmarking (recommended)
2. **Criterion.rs** - Library-based micro-benchmarks
3. **Both** - hyperfine for end-to-end, Criterion for micro-benchmarks

**Recommendation:** hyperfine only
- Simpler (no code changes)
- Sufficient statistical rigor
- Matches real-world usage (complete binary)

**Decision:** hyperfine for Sprint 5.9, evaluate Criterion.rs for v0.6.0 if micro-benchmarks needed

### Q2: How many runs per benchmark?

**Options:**
1. **5 runs** - Fast, but may have high variance
2. **10 runs** - Balance speed vs statistical rigor (recommended)
3. **20+ runs** - High confidence, but slow (>15 min total)

**Recommendation:** 10 runs
- hyperfine default is 10 (good balance)
- Sufficient for t-test (n≥10 for normality assumption)
- Total time: <10 min for 8 scenarios

**Decision:** 10 runs (--runs 10), increase to 20 if variance too high

### Q3: Update baseline on every release or manually?

**Options:**
1. **Automatic** - Update baseline on every tagged release
2. **Manual** - Developer runs `--baseline` flag when appropriate
3. **Hybrid** - Auto-update major releases (v0.5.0), manual for patches

**Recommendation:** Manual for Sprint 5.9, automate in Phase 6+
- Simpler implementation (no workflow automation needed)
- Developer controls when baseline updates (e.g., after performance improvements)

**Decision:** Manual baseline updates for v0.4.8, v0.5.0, etc. Automate in v0.6.0

### Q4: Benchmark on all platforms or Linux only?

**Options:**
1. **Linux only** - Fastest, most users develop on Linux
2. **Linux + macOS** - Covers M1 differences
3. **All platforms** - Linux + macOS + Windows

**Recommendation:** Linux primary, macOS optional, Windows defer
- Most CI runners are Linux (faster, cheaper)
- macOS M1 has different performance characteristics (worth testing)
- Windows has loopback issues (4 SYN tests fail, documented)

**Decision:** Linux for Sprint 5.9, macOS optional, Windows in Phase 6+ if demand

---

## Sprint Completion Checklist

### Phase Completion

- [ ] Phase 1: Planning & Design (3h)
- [ ] Phase 2: Benchmark Suite Implementation (5h)
- [ ] Phase 3: Hyperfine Integration (2h)
- [ ] Phase 4: Baseline Establishment (2h)
- [ ] Phase 5: CI/CD Integration (3h)
- [ ] Phase 6: Regression Detection (3h)
- [ ] Phase 7: Documentation (2h)
- [ ] Phase 8: Validation & Completion (1h)

### Deliverables Verification

**Code:**
- [ ] 15 benchmark/analysis scripts created (~1,035 lines)
- [ ] 1 GitHub Actions workflow created (~120 lines)
- [ ] 8+ scenarios functional (all pass)

**Data:**
- [ ] Baseline established for v0.4.8 (JSON + metadata)
- [ ] Example run results generated (for testing)
- [ ] Regression detection tested (mock data)

**Documentation:**
- [ ] 31-BENCHMARKING-GUIDE.md complete (~900 lines)
- [ ] README.md updated (~30 lines)
- [ ] CHANGELOG.md updated (~50 lines)
- [ ] All scripts have --help or usage comments

### Quality Verification

**Functional:**
- [ ] All 8 scenarios run successfully
- [ ] JSON export validated (correct schema)
- [ ] Regression detection accurate (tested with mock regressions)
- [ ] CI workflow passes (dry run or actual PR)
- [ ] PR comments work (if PR context)

**Performance:**
- [ ] Total suite execution <10 minutes
- [ ] Regression detection <30 seconds
- [ ] Results reproducible (stddev <5% of mean)

**Documentation:**
- [ ] Guide comprehensive (all sections complete)
- [ ] No broken links or cross-references
- [ ] Examples work (copy-paste executable)
- [ ] Troubleshooting addresses common issues

### Final Validation

- [ ] cargo fmt passing (if any Rust code added)
- [ ] cargo clippy passing (if any Rust code added)
- [ ] All existing tests passing (1,766 tests, no regressions)
- [ ] Benchmark suite tested locally (manual run)
- [ ] CI workflow tested (push to branch, monitor)
- [ ] Sprint completion report created (SPRINT-5.9-COMPLETE.md)

### Sprint Report

- [ ] Create `/tmp/ProRT-IP/SPRINT-5.9-COMPLETE.md` (~600-800 lines)
  - Executive summary
  - Deliverables achieved (scripts, docs, CI)
  - Benchmarking infrastructure overview
  - Example baseline results (v0.4.8 metrics)
  - Files changed summary
  - Lessons learned
  - Future work (v0.6.0: dashboard, more scenarios)

### Memory Bank Updates

- [ ] Update `CLAUDE.local.md`:
  - Sprint 5.9 completion status
  - Version update (v0.4.9 or v0.5.0)
  - Key decisions (hyperfine, 8 scenarios, regression thresholds)
  - Performance baselines (summary metrics)
  - Next sprint (5.10: Documentation & Polish)

---

## Notes & Observations

**Historical Context:**
- Sprint 5.8 achieved plugin system foundation (Lua, sandboxing, 2 examples)
- Current test count: 1,766 (100% passing)
- Current coverage: 54.92% (maintained since Sprint 5.6)
- Performance claims: "10M+ pps", "-1.8% overhead", "1.33μs TLS", "15% IPv6"

**Benchmarking Strategic Value:**
- **Regression Detection:** Catch performance degradation before shipping
- **Competitive Validation:** Prove claims with reproducible data (vs Nmap, Masscan)
- **Baseline Establishment:** Foundation for future optimizations
- **Performance Culture:** Demonstrates engineering rigor

**Technical Challenges:**
- Benchmark variability: Cloud CI runners have inconsistent performance
- Baseline staleness: Requires manual updates on releases (automate in Phase 6+)
- Threshold selection: 5% vs 10% trade-off (strictness vs flexibility)
- Cross-platform: Linux primary, macOS/Windows differences

**Next Sprints:**
- Sprint 5.10: Documentation & Polish (final Phase 5, comprehensive guides)
- Phase 6: TUI Interface (Q2 2026, interactive terminal UI)
- v0.5.0 Release: Feature completeness milestone (IPv6, Idle, Plugins, Benchmarks)

**Future Enhancements (v0.6.0+):**
- Performance dashboard (GitHub Pages + Chart.js visualization)
- Automated baseline updates (on release tags)
- Multi-platform baselines (Linux + macOS + Windows)
- More scenarios (20+): Large networks, plugin combinations, database export
- Criterion.rs micro-benchmarks (CPU cycles, cache misses)
- Historical trend analysis (performance over time)

**Competitive Positioning:**
- ProRT-IP: Rust safety + systematic benchmarking + regression detection
- Nmap: C/C++ + ad-hoc benchmarking (no CI regression detection)
- Masscan: C + minimal benchmarking (performance-focused but no automation)
- RustScan: Rust + no systematic benchmarking (opportunity for differentiation)

**Result:** ProRT-IP achieves performance validation infrastructure matching industry leaders while enabling continuous regression detection.

---

**Document Version:** 1.0
**Created:** 2025-11-06
**Status:** Ready for Sprint 5.9 execution
**Estimated Start:** Q1 2026 (after Sprint 5.8 complete, v0.4.8 released)
