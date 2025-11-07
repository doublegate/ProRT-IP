#!/usr/bin/env bash
#
# Scenario 1: SYN Scan Performance (1,000 ports)
# Purpose: Validate throughput ("10M+ pps" claim, indirectly)
# Target: <100ms for 1,000 ports on localhost
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
echo "Running Scenario 1: SYN Scan (1,000 ports)..."
echo "Binary: ${BINARY}"
echo "Target: 127.0.0.1:1-1000"
echo ""

hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/syn-scan-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/syn-scan-${DATE}.md" \
    "${BINARY} -sS -p 1-1000 127.0.0.1 --rate-limit 0"

echo ""
echo "Results saved to:"
echo "  JSON: ${RESULTS_DIR}/syn-scan-${DATE}.json"
echo "  Markdown: ${RESULTS_DIR}/syn-scan-${DATE}.md"
