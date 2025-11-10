#!/usr/bin/env bash
#
# Scenario 11: Xmas Scan Performance (stealth scan type)
# Purpose: Validate Xmas scan performance
# Target: <120ms (similar to FIN/NULL scans)
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

# Run benchmark
echo "Running Scenario 11: Xmas Scan (1,000 ports, stealth)..."
echo "Binary: ${BINARY}"
echo "Target: 127.0.0.1:1-1000"
echo ""

hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/xmas-scan-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/xmas-scan-${DATE}.md" \
    "${BINARY} -sX -p 1-1000 127.0.0.1"

echo ""
echo "Results saved to:"
echo "  JSON: ${RESULTS_DIR}/xmas-scan-${DATE}.json"
echo "  Markdown: ${RESULTS_DIR}/xmas-scan-${DATE}.md"
