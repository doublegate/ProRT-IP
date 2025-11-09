#!/usr/bin/env bash
#
# Scenario 16: Medium Scan (128 hosts, first 1000 ports)
# Purpose: Medium scan (small office network)
# Target: <10s (128,000 total port checks)
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
echo "Running Scenario 16: Medium Scan (128 hosts, 1000 ports)..."
echo "Binary: ${BINARY}"
echo "Target: 127.0.0.0/25 (128 hosts), ports 1-1000"
echo "Note: This scans localhost only (127.0.0.0-127.0.0.127)"
echo ""

hyperfine \
    --warmup 2 \
    --runs 5 \
    --export-json "${RESULTS_DIR}/medium-scan-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/medium-scan-${DATE}.md" \
    "${BINARY} -sS -p 1-1000 127.0.0.0/25"

echo ""
echo "Results saved to:"
echo "  JSON: ${RESULTS_DIR}/medium-scan-${DATE}.json"
echo "  Markdown: ${RESULTS_DIR}/medium-scan-${DATE}.md"
