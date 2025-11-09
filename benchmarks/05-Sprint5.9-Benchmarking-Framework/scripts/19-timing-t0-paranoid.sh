#!/usr/bin/env bash
#
# Scenario 19: Timing Template T0 (paranoid)
# Purpose: Slowest timing template (stealth mode)
# Target: >5s (intentionally slow)
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
echo "Running Scenario 19: Timing Template T0 (paranoid, slow)..."
echo "Binary: ${BINARY}"
echo "Target: 127.0.0.1:80,443,22 with T0 timing"
echo ""

hyperfine \
    --warmup 1 \
    --runs 5 \
    --export-json "${RESULTS_DIR}/timing-t0-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/timing-t0-${DATE}.md" \
    "${BINARY} -sS -p 80,443,22 127.0.0.1 -T0"

echo ""
echo "Results saved to:"
echo "  JSON: ${RESULTS_DIR}/timing-t0-${DATE}.json"
echo "  Markdown: ${RESULTS_DIR}/timing-t0-${DATE}.md"
