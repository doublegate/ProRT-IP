Compare performance between git refs (commits/branches/tags): $*

---

## PERFORMANCE COMPARISON WORKFLOW

**Purpose:** Compare benchmark performance between two git references to detect regressions or validate optimizations

**Usage:** `/bench-compare <baseline> <comparison>`
- **baseline:** Git ref for baseline (commit hash, branch, tag)
- **comparison:** Git ref to compare against baseline (commit hash, branch, tag)

**Example:** `/bench-compare v0.3.0 main` (compare v0.3.0 release vs current main branch)

---

## ERROR HANDLING

**All phases use standardized error handling for reliability:**

```bash
set -e  # Exit on error

handle_error() {
  local exit_code=$?
  local line_number=$1

  echo ""
  echo "❌ ERROR: Command failed at line $line_number (exit code $exit_code)"
  echo ""
  echo "📋 Troubleshooting:"
  echo "  1. Check git references are valid: git log --oneline -10"
  echo "  2. Ensure working directory is clean: git status"
  echo "  3. Verify hyperfine is installed: command -v hyperfine"
  echo "  4. Check disk space: df -h /tmp"
  echo ""
  exit $exit_code
}

trap 'handle_error ${LINENO}' ERR
```

---

## Phase 0: VALIDATE PREREQUISITES

**Objective:** Ensure all required tools and clean environment before starting

### Step 0.1: Check hyperfine Installation

```bash
if ! command -v hyperfine &>/dev/null; then
  echo "❌ ERROR: hyperfine not installed"
  echo ""
  echo "Install: cargo install hyperfine"
  echo ""
  echo "hyperfine is required for statistical benchmarking"
  exit 1
fi

echo "✅ hyperfine available"
```

### Step 0.2: Check Git Working Tree

```bash
# Check for uncommitted changes
if ! git diff-index --quiet HEAD -- 2>/dev/null; then
  echo "⚠️  WARNING: Uncommitted changes detected"
  echo ""
  git status --short | head -10
  echo ""
  echo "Benchmarking with uncommitted changes may produce inconsistent results"
  echo ""
  read -p "Continue anyway? (y/N): " -n 1 -r
  echo ""
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted by user"
    exit 0
  fi

  # Stash changes for safety
  echo "Stashing uncommitted changes..."
  git stash push -m "bench-compare auto-stash $(date +%s)"
  STASHED=true
else
  STASHED=false
fi

echo "✅ Git working tree validated"
```

### Step 0.3: Check Disk Space

```bash
# Ensure sufficient disk space for binaries (minimum 1GB)
AVAILABLE=$(df /tmp --output=avail 2>/dev/null | tail -1 || echo "0")
REQUIRED=1048576  # 1GB in KB

if [ "$AVAILABLE" -lt "$REQUIRED" ]; then
  echo "⚠️  WARNING: Low disk space in /tmp"
  echo "Available: $((AVAILABLE / 1024))MB"
  echo "Recommended: $((REQUIRED / 1024))MB"
  echo ""
  read -p "Continue anyway? (y/N): " -n 1 -r
  echo ""
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 0
  fi
fi

echo "✅ Sufficient disk space"
echo ""
```

---

## Phase 1: VALIDATE GIT REFERENCES

**Objective:** Ensure both git refs exist and are valid before proceeding

### Step 1.1: Parse Arguments

```bash
BASELINE="$1"
COMPARISON="$2"

if [ -z "$BASELINE" ] || [ -z "$COMPARISON" ]; then
  echo "ERROR: Usage: /bench-compare <baseline> <comparison>"
  echo "Example: /bench-compare v0.3.0 main"
  exit 1
fi
```

### Step 1.2: Verify Git References

```bash
git rev-parse --verify "$BASELINE" >/dev/null 2>&1
if [ $? -ne 0 ]; then
  echo "ERROR: Baseline ref '$BASELINE' does not exist"
  exit 1
fi

git rev-parse --verify "$COMPARISON" >/dev/null 2>&1
if [ $? -ne 0 ]; then
  echo "ERROR: Comparison ref '$COMPARISON' does not exist"
  exit 1
fi
```

### Step 1.3: Display Comparison Summary

```bash
BASELINE_HASH=$(git rev-parse --short "$BASELINE")
COMPARISON_HASH=$(git rev-parse --short "$COMPARISON")

echo "Benchmark Comparison:"
echo "  Baseline:   $BASELINE ($BASELINE_HASH)"
echo "  Comparison: $COMPARISON ($COMPARISON_HASH)"
```

---

## Phase 2: BUILD BASELINE VERSION

**Objective:** Checkout baseline ref and build optimized release binary

### Step 2.1: Save Current Branch State

```bash
CURRENT_BRANCH=$(git branch --show-current)
echo "Saving current branch: $CURRENT_BRANCH"
```

### Step 2.2: Checkout Baseline

```bash
git checkout "$BASELINE"
```

### Step 2.3: Build Baseline Release Binary

```bash
cargo build --release
```

**Expected:** Successful compilation, zero warnings
**Duration:** ~30-60 seconds (varies by system)

### Step 2.4: Copy Baseline Binary

```bash
cp target/release/prtip /tmp/prtip-baseline
echo "Baseline binary: /tmp/prtip-baseline"
```

---

## Phase 3: BUILD COMPARISON VERSION

**Objective:** Checkout comparison ref and build optimized release binary

### Step 3.1: Checkout Comparison

```bash
git checkout "$COMPARISON"
```

### Step 3.2: Build Comparison Release Binary

```bash
cargo build --release
```

**Expected:** Successful compilation, zero warnings
**Duration:** ~30-60 seconds (varies by system)

### Step 3.3: Copy Comparison Binary

```bash
cp target/release/prtip /tmp/prtip-comparison
echo "Comparison binary: /tmp/prtip-comparison"
```

### Step 3.4: Restore Original Branch

```bash
git checkout "$CURRENT_BRANCH"
echo "Restored branch: $CURRENT_BRANCH"
```

---

## Phase 4: EXECUTE HYPERFINE BENCHMARKS

**Objective:** Run statistical benchmarks using hyperfine for accurate performance comparison

### Step 4.1: Define Benchmark Scenarios

**Common Benchmark Scenarios:**
1. **1K ports (localhost):** Fast baseline
2. **10K ports (localhost):** Medium workload
3. **10K ports --with-db:** Database overhead
4. **Common ports (80,443,22,21,25):** Typical scan

### Step 4.2: Execute Hyperfine Comparison

```bash
hyperfine \
  --warmup 3 \
  --runs 20 \
  --export-json /tmp/bench-compare.json \
  --export-markdown /tmp/bench-compare.md \
  '/tmp/prtip-baseline -p 1-10000 127.0.0.1' \
  '/tmp/prtip-comparison -p 1-10000 127.0.0.1'
```

**Parameters:**
- `--warmup 3`: Discard first 3 runs (cold cache)
- `--runs 20`: Statistical significance (20 samples)
- `--export-json`: Machine-readable results
- `--export-markdown`: Human-readable table

**Duration:** ~60-90 seconds (20 runs × 2 binaries × ~2 seconds per run)

---

## Phase 5: ANALYZE RESULTS AND GENERATE REPORT

**Objective:** Parse hyperfine results, detect regressions, generate comprehensive report

### Step 5.1: Parse JSON Results

```bash
BASELINE_MEAN=$(jq -r '.results[0].mean' /tmp/bench-compare.json)
COMPARISON_MEAN=$(jq -r '.results[1].mean' /tmp/bench-compare.json)
```

### Step 5.2: Calculate Performance Delta

```bash
DELTA=$(echo "scale=2; (($COMPARISON_MEAN - $BASELINE_MEAN) / $BASELINE_MEAN) * 100" | bc)
```

**Interpretation:**
- **DELTA < -5%:** 🚀 **Performance Improvement** (faster)
- **-5% ≤ DELTA ≤ +5%:** ✅ **No Significant Change** (within noise)
- **DELTA > +5%:** ⚠️ **Performance Regression** (slower)

### Step 5.3: Generate Comprehensive Report

**Report Location:** `/tmp/bench-compare-report.md`

**Report Contents:**

```markdown
# Benchmark Comparison Report

**Date:** $(date)
**Baseline:** $BASELINE ($BASELINE_HASH)
**Comparison:** $COMPARISON ($COMPARISON_HASH)

## Results Summary

| Metric | Baseline | Comparison | Delta |
|--------|----------|------------|-------|
| Mean | ${BASELINE_MEAN}s | ${COMPARISON_MEAN}s | ${DELTA}% |
| Std Dev | ... | ... | ... |
| Min | ... | ... | ... |
| Max | ... | ... | ... |

## Performance Analysis

[Auto-generated analysis based on DELTA]

## Detailed Hyperfine Results

[Include full markdown table from /tmp/bench-compare.md]

## Recommendations

[Actionable next steps based on results]
```

### Step 5.4: Display Report Summary

```bash
cat /tmp/bench-compare-report.md
```

---

## SUCCESS CRITERIA

✅ **Both binaries built successfully** (zero compilation errors)
✅ **Hyperfine completed 20 runs** (statistical significance)
✅ **Results exported** (JSON + Markdown + Report)
✅ **Git state restored** (original branch checked out)
✅ **Performance delta calculated** (regression detected if >+5%)

---

## REGRESSION DETECTION

**Automatic Alerts:**

- **REGRESSION (>+5% slower):** ⚠️ **ACTION REQUIRED**
  - Investigate code changes between baseline and comparison
  - Run `/perf-profile` on comparison binary to identify bottlenecks
  - Consider reverting changes if regression is critical

- **IMPROVEMENT (<-5% faster):** 🚀 **OPTIMIZATION SUCCESS**
  - Document optimization in CHANGELOG.md
  - Update performance metrics in README.md
  - Consider benchmarking additional scenarios

- **NO CHANGE (±5%):** ✅ **NEUTRAL**
  - Changes did not impact performance
  - No action required

---

## ADVANCED SCENARIOS

### Multiple Benchmark Scenarios

Run hyperfine with multiple commands:

```bash
hyperfine \
  --warmup 3 --runs 20 \
  --export-json /tmp/bench-multi.json \
  '/tmp/prtip-baseline -p 1-1000 127.0.0.1' \
  '/tmp/prtip-comparison -p 1-1000 127.0.0.1' \
  '/tmp/prtip-baseline -p 1-10000 127.0.0.1' \
  '/tmp/prtip-comparison -p 1-10000 127.0.0.1' \
  '/tmp/prtip-baseline -p 1-10000 --with-db 127.0.0.1' \
  '/tmp/prtip-comparison -p 1-10000 --with-db 127.0.0.1'
```

### Network-Based Benchmarks

Compare against real network targets:

```bash
hyperfine \
  --warmup 1 --runs 5 \
  '/tmp/prtip-baseline -p 80,443 192.168.4.0/24' \
  '/tmp/prtip-comparison -p 80,443 192.168.4.0/24'
```

**Note:** Fewer runs for network scans (higher variance)

---

## CLEANUP

**Temporary Files:**
- `/tmp/prtip-baseline` (binary)
- `/tmp/prtip-comparison` (binary)
- `/tmp/bench-compare.json` (hyperfine JSON)
- `/tmp/bench-compare.md` (hyperfine markdown)
- `/tmp/bench-compare-report.md` (comprehensive report)

**Preserve:** Keep report files for documentation, delete binaries after analysis

### Restore Stashed Changes (if applicable)

```bash
if [ "$STASHED" = true ]; then
  echo ""
  echo "Restoring stashed changes..."
  git stash pop

  if [ $? -eq 0 ]; then
    echo "✅ Stashed changes restored successfully"
  else
    echo "⚠️  WARNING: Stash pop had conflicts"
    echo "Resolve conflicts manually or use: git stash list"
  fi
fi
```

---

## DELIVERABLES

1. **Performance Report:** `/tmp/bench-compare-report.md` (comprehensive analysis)
2. **Raw Results:** `/tmp/bench-compare.json` (machine-readable)
3. **Markdown Table:** `/tmp/bench-compare.md` (human-readable)
4. **Regression Alert:** Console output with performance delta

---

## INTEGRATION WITH SPRINTS

**Sprint Completion:** Run `/bench-compare <previous-sprint-tag> main` to validate performance
**Pre-Release:** Run `/bench-compare <last-release> <release-candidate>` before tagging
**CI/CD:** Automate with GitHub Actions on pull requests (detect regressions before merge)

---

## RELATED COMMANDS

**Performance Analysis:**
- `/perf-profile <command>` - Profile hotspots to understand performance differences
- Complementary: bench-compare measures wall-clock time, perf-profile identifies bottlenecks

**Sprint Workflow:**
- `/sprint-start <sprint-id> "Performance optimization"` - Initialize optimization sprint
- `/sprint-complete <sprint-id>` - Document performance improvements with benchmark results
- Include bench-compare results in implementation-summary.md

**Quality Validation:**
- `/rust-check` - Ensure both baseline and comparison refs compile and pass tests
- `/test-quick <pattern>` - Run targeted tests if benchmarks show unexpected results

**Documentation:**
- `/doc-update perf "10K ports: 117ms → 39ms (66% faster)"` - Document improvements
- Benchmark results feed into CHANGELOG.md performance section

## WORKFLOW INTEGRATION

**Performance Optimization Workflow:**

```
1. Baseline Measurement:
   git tag baseline-before-optimization
   /bench-compare v0.3.0 baseline-before-optimization

2. Identify Bottlenecks:
   /perf-profile ./target/release/prtip <args>
   # Analyze flamegraph for hot functions

3. Implement Optimizations:
   - Modify code based on profiling insights
   - /rust-check  # Validate quality
   - /test-quick <pattern>  # Ensure correctness

4. Measure Impact:
   /bench-compare baseline-before-optimization HEAD
   # Quantify improvement (e.g., "66% faster")

5. Iterate if Needed:
   - If <5% improvement: /perf-profile again
   - If regression: revert and try different approach

6. Document Results:
   /doc-update perf "Optimization results: X → Y (Z% faster)"
   # Update README.md, CHANGELOG.md
```

**Sprint Integration:**

```
Sprint Planning → /sprint-start 4.X "Performance Sprint"
Pre-optimization → /bench-compare v0.3.0 main (baseline)
Development → [code changes]
Validation → /bench-compare v0.3.0 main (measure improvement)
Completion → /sprint-complete 4.X (include benchmark results)
```

**CI/CD Integration:**

- Automate bench-compare on PRs (detect regressions before merge)
- Store benchmark results in benchmarks/ directory
- Fail CI if >10% regression detected (configurable threshold)

## SEE ALSO

- `docs/07-PERFORMANCE.md` - Performance optimization guide
- `benchmarks/` - Historical benchmark results directory
- `CHANGELOG.md` - Performance improvements changelog
- hyperfine documentation: https://github.com/sharkdp/hyperfine
- Brendan Gregg's performance methodology: http://www.brendangregg.com/methodology.html

---

**Execute performance comparison between refs: $***
