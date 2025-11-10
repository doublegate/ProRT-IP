#!/usr/bin/env bash
#
# Scenario 5: IPv6 Overhead
# Purpose: Validate Sprint 5.1 IPv6 claim (15% overhead)
# Target: IPv6 <15% slower than IPv4
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

# Run benchmark (parallel comparison: IPv4 vs IPv6)
echo "Running Scenario 5: IPv6 Overhead..."
echo "Binary: ${BINARY}"
echo "IPv4 Target: 127.0.0.1:1-1000"
echo "IPv6 Target: ::1:1-1000"
echo ""

hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/ipv6-overhead-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/ipv6-overhead-${DATE}.md" \
    --command-name "ipv4" \
    "${BINARY} -4 -sS -p 1-1000 127.0.0.1" \
    --command-name "ipv6" \
    "${BINARY} -6 -sS -p 1-1000 ::1"

# Calculate overhead (manual calculation from JSON)
echo ""
echo "Overhead calculation (manual):"
echo "  Overhead = (ipv6_mean - ipv4_mean) / ipv4_mean * 100"
echo "  Target: <15% (Sprint 5.1 claim)"
echo "  Results saved to:"
echo "    JSON: ${RESULTS_DIR}/ipv6-overhead-${DATE}.json"
echo "    Markdown: ${RESULTS_DIR}/ipv6-overhead-${DATE}.md"
