#!/usr/bin/env bash
#
# Scenario 17: Large Scan (1024 hosts, 10 common ports)
# Purpose: Large scan (data center network)
# Target: <30s (10,240 total port checks)
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
echo "Running Scenario 17: Large Scan (1024 hosts, 10 common ports)..."
echo "Binary: ${BINARY}"
echo "Target: 127.0.0.0/22 (1024 hosts), ports 80,443,22,21,25,110,143,3389,3306,5432"
echo "Note: This scans localhost range only (127.0.0.0-127.0.3.255)"
echo ""

hyperfine \
    --warmup 1 \
    --runs 3 \
    --export-json "${RESULTS_DIR}/large-scan-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/large-scan-${DATE}.md" \
    "${BINARY} -sS -p 80,443,22,21,25,110,143,3389,3306,5432 127.0.0.0/22"

echo ""
echo "Results saved to:"
echo "  JSON: ${RESULTS_DIR}/large-scan-${DATE}.json"
echo "  Markdown: ${RESULTS_DIR}/large-scan-${DATE}.md"
