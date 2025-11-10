#!/usr/bin/env bash
#
# Regression Detection Script
# Compares baseline vs current results and detects performance regressions
#
# Usage:
#   ./analyze-results.sh <baseline-dir-or-file> <results-dir>
#
# Exit Codes:
#   0: PASS or IMPROVED
#   1: WARN (5-10% slower on any benchmark)
#   2: FAIL (>10% slower on any benchmark)
#

set -euo pipefail

# Validate arguments
if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <baseline-dir-or-file> <results-dir>"
    echo ""
    echo "Examples:"
    echo "  $0 benchmarks/baselines/baseline-v0.5.0.json results/"
    echo "  $0 benchmarks/baselines results/"
    exit 1
fi

BASELINE_ARG="$1"
RESULTS_DIR="$2"

# Check for jq (JSON processor)
if ! command -v jq &>/dev/null; then
    echo "Error: jq not found (required for JSON processing)"
    echo "Install: apt install jq  OR  brew install jq"
    exit 1
fi

# Ensure results directory exists
if [[ ! -d "${RESULTS_DIR}" ]]; then
    echo "Error: Results directory not found: ${RESULTS_DIR}"
    exit 1
fi

# Create output directory for PR comment
mkdir -p "${RESULTS_DIR}"

echo "============================================="
echo "Regression Detection Analysis"
echo "============================================="
echo "Baseline: ${BASELINE_ARG}"
echo "Results:  ${RESULTS_DIR}"
echo ""

# Initialize counters
TOTAL_TESTS=0
IMPROVED_COUNT=0
PASS_COUNT=0
WARN_COUNT=0
FAIL_COUNT=0
WORST_EXIT_CODE=0

# Output arrays for markdown table
declare -a TABLE_ROWS=()

# Function to compare two benchmark JSON files
compare_benchmark() {
    local baseline_file="$1"
    local current_file="$2"
    local scenario_name="$3"

    # Extract mean and stddev
    local baseline_mean=$(jq -r '.results[0].mean' "$baseline_file" 2>/dev/null || echo "0")
    local current_mean=$(jq -r '.results[0].mean' "$current_file" 2>/dev/null || echo "0")
    local baseline_stddev=$(jq -r '.results[0].stddev' "$baseline_file" 2>/dev/null || echo "0")
    local current_stddev=$(jq -r '.results[0].stddev' "$current_file" 2>/dev/null || echo "0")

    # Convert to milliseconds for readability
    local baseline_ms=$(awk "BEGIN {printf \"%.2f\", $baseline_mean * 1000}")
    local current_ms=$(awk "BEGIN {printf \"%.2f\", $current_mean * 1000}")

    # Calculate percentage difference
    local diff_pct=$(awk "BEGIN {printf \"%.2f\", (($current_mean - $baseline_mean) / $baseline_mean) * 100}")

    # Determine status
    local status=""
    local emoji=""
    local exit_code=0

    if (( $(awk "BEGIN {print ($diff_pct < -5) ? 1 : 0}") )); then
        status="IMPROVED"
        emoji="✅"
        exit_code=0
        ((IMPROVED_COUNT++))
    elif (( $(awk "BEGIN {print ($diff_pct < 5) ? 1 : 0}") )); then
        status="PASS"
        emoji="✅"
        exit_code=0
        ((PASS_COUNT++))
    elif (( $(awk "BEGIN {print ($diff_pct < 10) ? 1 : 0}") )); then
        status="WARN"
        emoji="⚠️"
        exit_code=1
        ((WARN_COUNT++))
    else
        status="FAIL"
        emoji="❌"
        exit_code=2
        ((FAIL_COUNT++))
    fi

    # Update worst exit code
    if [[ $exit_code -gt $WORST_EXIT_CODE ]]; then
        WORST_EXIT_CODE=$exit_code
    fi

    ((TOTAL_TESTS++))

    # Add to table rows
    TABLE_ROWS+=("| ${scenario_name} | ${baseline_ms}ms | ${current_ms}ms | ${diff_pct}% | ${emoji} ${status} |")

    # Print to console
    printf "%-40s: %s (%+.2f%%)\n" "$scenario_name" "$status" "$diff_pct"
}

# Find all benchmark result files in results directory
CURRENT_FILES=$(find "$RESULTS_DIR" -name "*.json" -type f | sort)

if [[ -z "$CURRENT_FILES" ]]; then
    echo "Error: No JSON benchmark files found in $RESULTS_DIR"
    exit 1
fi

echo "Comparing benchmarks..."
echo ""

# Process each current result file
while IFS= read -r current_file; do
    # Extract scenario name from filename
    # Example: syn-scan-20251109-123456.json -> syn-scan
    filename=$(basename "$current_file")
    scenario_name=$(echo "$filename" | sed 's/-[0-9]\{8\}-[0-9]\{6\}\.json$//' | sed 's/\.json$//')

    # Find corresponding baseline file
    if [[ -f "$BASELINE_ARG" ]]; then
        # Single baseline file mode (legacy)
        baseline_file="$BASELINE_ARG"
    else
        # Directory mode - find matching baseline
        baseline_file=$(find "$BASELINE_ARG" -name "${scenario_name}*.json" -type f | head -n 1)

        if [[ -z "$baseline_file" ]]; then
            echo "Warning: No baseline found for $scenario_name (skipping)"
            continue
        fi
    fi

    # Compare
    compare_benchmark "$baseline_file" "$current_file" "$scenario_name"
done <<< "$CURRENT_FILES"

echo ""
echo "============================================="
echo "Summary"
echo "============================================="
echo "Total Tests:    $TOTAL_TESTS"
echo "Improved:       $IMPROVED_COUNT"
echo "Passed:         $PASS_COUNT"
echo "Warnings:       $WARN_COUNT"
echo "Failures:       $FAIL_COUNT"
echo ""

# Generate PR comment markdown
PR_COMMENT_FILE="${RESULTS_DIR}/pr-comment.md"

cat > "$PR_COMMENT_FILE" << 'HEADER'
## 📊 Performance Benchmark Results

HEADER

# Add summary table
cat >> "$PR_COMMENT_FILE" << EOF
| Scenario | Baseline | Current | Diff | Status |
|----------|----------|---------|------|--------|
EOF

# Add all table rows
for row in "${TABLE_ROWS[@]}"; do
    echo "$row" >> "$PR_COMMENT_FILE"
done

# Add summary
cat >> "$PR_COMMENT_FILE" << EOF

### Summary

- **Total Tests:** $TOTAL_TESTS
- **Improved:** $IMPROVED_COUNT 🎉
- **Passed:** $PASS_COUNT ✅
- **Warnings:** $WARN_COUNT ⚠️
- **Failures:** $FAIL_COUNT ❌

### Recommendation

EOF

# Add recommendation based on results
if [[ $FAIL_COUNT -gt 0 ]]; then
    cat >> "$PR_COMMENT_FILE" << 'EOF'
**❌ REGRESSION DETECTED**

Performance degraded by >10% on one or more benchmarks. Please investigate before merging.

**Actions Required:**
1. Review code changes for performance impact
2. Run profiling to identify bottlenecks
3. Consider optimizations or defer changes

EOF
elif [[ $WARN_COUNT -gt 0 ]]; then
    cat >> "$PR_COMMENT_FILE" << 'EOF'
**⚠️ POTENTIAL REGRESSION**

Performance degraded by 5-10% on one or more benchmarks. Review recommended.

**Suggested Actions:**
1. Review changes for unexpected overhead
2. Consider if regression is acceptable for new features
3. Run additional profiling if unsure

EOF
elif [[ $IMPROVED_COUNT -gt 0 ]]; then
    cat >> "$PR_COMMENT_FILE" << 'EOF'
**🎉 PERFORMANCE IMPROVEMENT**

Great work! Performance improved on one or more benchmarks.

EOF
else
    cat >> "$PR_COMMENT_FILE" << 'EOF'
**✅ ALL BENCHMARKS PASSING**

Performance is within acceptable variance. No regressions detected.

EOF
fi

# Add thresholds reference
cat >> "$PR_COMMENT_FILE" << 'EOF'

### Thresholds

- **✅ PASS:** <5% slower (within noise)
- **⚠️ WARN:** 5-10% slower (investigate)
- **❌ FAIL:** >10% slower (regression)
- **🎉 IMPROVED:** Faster than baseline

---

<details>
<summary>ℹ️ About Performance Testing</summary>

**Methodology:**
- Benchmarks run with [hyperfine](https://github.com/sharkdp/hyperfine)
- 10 runs per scenario (3 warmup)
- Statistical outlier removal
- Results compared against baseline

**Thresholds:**
- Small variations (<5%) are expected due to system noise
- Medium variations (5-10%) warrant investigation
- Large variations (>10%) indicate regressions

**Baseline:** Latest released version (see [baselines/](../baselines/))

</details>
EOF

echo "PR comment generated: $PR_COMMENT_FILE"
echo ""

# Print recommendation to console
if [[ $FAIL_COUNT -gt 0 ]]; then
    echo "❌ REGRESSION DETECTED"
    echo "Performance degraded by >10% on $FAIL_COUNT benchmark(s)"
    echo "Investigate changes before merge"
elif [[ $WARN_COUNT -gt 0 ]]; then
    echo "⚠️ POTENTIAL REGRESSION"
    echo "Performance degraded by 5-10% on $WARN_COUNT benchmark(s)"
    echo "Review recommended"
elif [[ $IMPROVED_COUNT -gt 0 ]]; then
    echo "🎉 PERFORMANCE IMPROVEMENT"
    echo "$IMPROVED_COUNT benchmark(s) improved"
else
    echo "✅ ALL BENCHMARKS PASSING"
    echo "Performance within acceptable variance"
fi

echo ""
echo "Exit code: $WORST_EXIT_CODE"

exit $WORST_EXIT_CODE
