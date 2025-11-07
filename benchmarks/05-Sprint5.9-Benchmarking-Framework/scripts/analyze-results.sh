#!/usr/bin/env bash
#
# Regression Detection Script
# Compares baseline vs current results and detects performance regressions
#
# Usage:
#   ./analyze-results.sh <baseline.json> <current.json>
#
# Exit Codes:
#   0: PASS or IMPROVED
#   1: WARN (5-10% slower)
#   2: FAIL (>10% slower)
#

set -euo pipefail

# Validate arguments
if [[ $# -ne 2 ]]; then
    echo "Usage: $0 <baseline.json> <current.json>"
    echo ""
    echo "Example:"
    echo "  $0 baselines/baseline-v0.4.8.json results/2025-11-06/current.json"
    exit 1
fi

BASELINE_FILE="$1"
CURRENT_FILE="$2"

# Validate files exist
if [[ ! -f "${BASELINE_FILE}" ]]; then
    echo "Error: Baseline file not found: ${BASELINE_FILE}"
    exit 1
fi

if [[ ! -f "${CURRENT_FILE}" ]]; then
    echo "Error: Current results file not found: ${CURRENT_FILE}"
    exit 1
fi

# Check for jq (JSON processor)
if ! command -v jq &>/dev/null; then
    echo "Error: jq not found (required for JSON processing)"
    echo "Install: apt install jq  OR  brew install jq"
    exit 1
fi

echo "============================================="
echo "Regression Detection Analysis"
echo "============================================="
echo "Baseline: ${BASELINE_FILE}"
echo "Current:  ${CURRENT_FILE}"
echo ""

# Extract results from JSON (simplified - assumes single-scenario files)
# Real implementation would iterate through multiple scenarios

BASELINE_MEAN=$(jq -r '.results[0].mean' "${BASELINE_FILE}")
CURRENT_MEAN=$(jq -r '.results[0].mean' "${CURRENT_FILE}")
BASELINE_STDDEV=$(jq -r '.results[0].stddev' "${BASELINE_FILE}")
CURRENT_STDDEV=$(jq -r '.results[0].stddev' "${CURRENT_FILE}")

# Calculate percentage difference
DIFF_PCT=$(awk "BEGIN {printf \"%.2f\", (($CURRENT_MEAN - $BASELINE_MEAN) / $BASELINE_MEAN) * 100}")

# Determine status
if (( $(awk "BEGIN {print ($DIFF_PCT < -5) ? 1 : 0}") )); then
    STATUS="✅ IMPROVED"
    EXIT_CODE=0
elif (( $(awk "BEGIN {print ($DIFF_PCT < 5) ? 1 : 0}") )); then
    STATUS="✅ PASS"
    EXIT_CODE=0
elif (( $(awk "BEGIN {print ($DIFF_PCT < 10) ? 1 : 0}") )); then
    STATUS="⚠️ WARN"
    EXIT_CODE=1
else
    STATUS="❌ REGRESSION"
    EXIT_CODE=2
fi

# Print summary table
echo "| Metric         | Baseline        | Current         | Diff      | Status |"
echo "|----------------|-----------------|-----------------|-----------|--------|"
printf "| Mean           | %.4fs           | %.4fs           | %+.2f%%    | %s |\n" \
    "$BASELINE_MEAN" "$CURRENT_MEAN" "$DIFF_PCT" "$STATUS"
printf "| Stddev         | %.4fs           | %.4fs           | -         | -      |\n" \
    "$BASELINE_STDDEV" "$CURRENT_STDDEV"

echo ""

# Detailed explanation
if [[ "${STATUS}" == "❌ REGRESSION" ]]; then
    echo "REGRESSION DETECTED:"
    echo "  Performance degraded by ${DIFF_PCT}% (>10% threshold)"
    echo "  Investigate changes since baseline"
    echo ""
elif [[ "${STATUS}" == "⚠️ WARN" ]]; then
    echo "WARNING:"
    echo "  Performance degraded by ${DIFF_PCT}% (5-10% threshold)"
    echo "  Consider investigating before merge"
    echo ""
elif [[ "${STATUS}" == "✅ IMPROVED" ]]; then
    echo "IMPROVED:"
    echo "  Performance improved by ${DIFF_PCT#-}%"
    echo "  Great work! 🎉"
    echo ""
else
    echo "PASS:"
    echo "  Performance within acceptable variance (${DIFF_PCT}%)"
    echo ""
fi

# Recommendations
echo "Thresholds:"
echo "  PASS:       <5% slower (within noise)"
echo "  WARN:       5-10% slower (investigate)"
echo "  FAIL:       >10% slower (regression)"
echo "  IMPROVED:   Faster than baseline"
echo ""

# Statistical significance note
echo "Note: Statistical significance test (t-test) requires Python implementation"
echo "      See scripts/statistical-test.py for full regression analysis"
echo ""

exit ${EXIT_CODE}
