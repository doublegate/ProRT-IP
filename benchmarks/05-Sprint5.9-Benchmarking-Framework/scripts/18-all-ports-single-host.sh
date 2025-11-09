#!/usr/bin/env bash
#
# Scenario 18: All Ports Single Host (65535 ports)
# Purpose: Maximum port coverage single host
# Target: <5s (assuming ~13K pps)
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
echo "Running Scenario 18: All Ports Single Host (65535 ports)..."
echo "Binary: ${BINARY}"
echo "Target: 127.0.0.1:1-65535 (all ports)"
echo ""

hyperfine \
    --warmup 1 \
    --runs 5 \
    --export-json "${RESULTS_DIR}/all-ports-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/all-ports-${DATE}.md" \
    "${BINARY} -sS -p- 127.0.0.1"

echo ""
echo "Results saved to:"
echo "  JSON: ${RESULTS_DIR}/all-ports-${DATE}.json"
echo "  Markdown: ${RESULTS_DIR}/all-ports-${DATE}.md"
