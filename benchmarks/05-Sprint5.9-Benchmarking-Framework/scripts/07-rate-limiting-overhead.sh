#!/usr/bin/env bash
#
# Scenario 7: Rate Limiting Overhead
# Purpose: Validate Sprint 5.X AdaptiveRateLimiterV3 (-1.8% overhead)
# Target: <5% overhead (claimed -1.8%)
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
    echo "Run: cargo build --release"
    exit 1
fi

# Create results directory
mkdir -p "${RESULTS_DIR}"

# Run benchmark (parallel comparison: no limit vs V3 limiter)
echo "Running Scenario 7: Rate Limiting Overhead..."
echo "Binary: ${BINARY}"
echo "Target: 127.0.0.1:1-1000"
echo "Baseline: No rate limiting (--rate-limit 0)"
echo "Limited: V3 rate limiter (--rate-limit 10000)"
echo ""

hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/rate-limiting-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/rate-limiting-${DATE}.md" \
    --command-name "no-limit" \
    "${BINARY} -sS -p 1-1000 127.0.0.1 --rate-limit 0" \
    --command-name "v3-limiter" \
    "${BINARY} -sS -p 1-1000 127.0.0.1 --rate-limit 10000"

# Calculate overhead (manual calculation from JSON)
echo ""
echo "Overhead calculation (manual):"
echo "  Overhead = (limited_mean - baseline_mean) / baseline_mean * 100"
echo "  Target: <5% (Sprint 5.X claim: -1.8%)"
echo "  Results saved to:"
echo "    JSON: ${RESULTS_DIR}/rate-limiting-${DATE}.json"
echo "    Markdown: ${RESULTS_DIR}/rate-limiting-${DATE}.md"
