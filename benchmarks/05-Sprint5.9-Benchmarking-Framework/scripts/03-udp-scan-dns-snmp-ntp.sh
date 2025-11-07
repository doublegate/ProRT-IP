#!/usr/bin/env bash
#
# Scenario 3: UDP Scan Performance (3 UDP services)
# Purpose: Slow protocol validation (10-100x slower than TCP)
# Target: <500ms (UDP is slow due to ICMP rate limiting)
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
echo "Running Scenario 3: UDP Scan (3 UDP services: DNS, SNMP, NTP)..."
echo "Binary: ${BINARY}"
echo "Target: 127.0.0.1:53,161,123"
echo "Note: UDP scanning is 10-100x slower than TCP (ICMP rate limiting)"
echo ""

hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/udp-scan-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/udp-scan-${DATE}.md" \
    "${BINARY} -sU -p 53,161,123 127.0.0.1"

echo ""
echo "Results saved to:"
echo "  JSON: ${RESULTS_DIR}/udp-scan-${DATE}.json"
echo "  Markdown: ${RESULTS_DIR}/udp-scan-${DATE}.md"
