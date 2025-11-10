#!/usr/bin/env bash
#
# Scenario 20: Timing Template T5 (insane)
# Purpose: Fastest timing template (speed mode)
# Target: <80ms (faster than T3 default ~98ms)
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
echo "Running Scenario 20: Timing Template T5 (insane, fast)..."
echo "Binary: ${BINARY}"
echo "Target: 127.0.0.1:1-1000 with T5 timing"
echo ""

hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/timing-t5-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/timing-t5-${DATE}.md" \
    "${BINARY} -sS -p 1-1000 127.0.0.1 -T5"

echo ""
echo "Results saved to:"
echo "  JSON: ${RESULTS_DIR}/timing-t5-${DATE}.json"
echo "  Markdown: ${RESULTS_DIR}/timing-t5-${DATE}.md"
