#!/usr/bin/env bash
#
# Scenario 2: Connect Scan Performance (3 common ports)
# Purpose: Real-world baseline (most common usage)
# Target: <50ms
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
echo "Running Scenario 2: Connect Scan (3 common ports)..."
echo "Binary: ${BINARY}"
echo "Target: 127.0.0.1:80,443,8080"
echo ""

hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/connect-scan-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/connect-scan-${DATE}.md" \
    "${BINARY} -sT -p 80,443,8080 127.0.0.1"

echo ""
echo "Results saved to:"
echo "  JSON: ${RESULTS_DIR}/connect-scan-${DATE}.json"
echo "  Markdown: ${RESULTS_DIR}/connect-scan-${DATE}.md"
