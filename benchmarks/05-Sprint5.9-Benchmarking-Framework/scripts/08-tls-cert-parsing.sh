#!/usr/bin/env bash
#
# Scenario 8: TLS Certificate Parsing
# Purpose: Validate Sprint 5.5 TLS parsing claim (1.33μs)
# Target: ~1.33μs per certificate parsed
#
# NOTE: Requires network access to badssl.com
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

# Check network connectivity
if ! ping -c 1 -W 2 badssl.com &>/dev/null; then
    echo "Warning: Cannot reach badssl.com (network issue)"
    echo "This benchmark requires internet connectivity"
    echo "Skipping..."
    exit 0
fi

# Run benchmark
echo "Running Scenario 8: TLS Certificate Parsing..."
echo "Binary: ${BINARY}"
echo "Target: badssl.com:443"
echo "Expected: ~1.33μs per certificate (Sprint 5.5 claim)"
echo "Note: Requires network access"
echo ""

hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/tls-parsing-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/tls-parsing-${DATE}.md" \
    "${BINARY} -sV -p 443 badssl.com --tls-cert-analysis"

echo ""
echo "TLS parsing time extraction:"
echo "  Requires parsing verbose output or instrumentation"
echo "  Target: ~1.33μs per certificate"
echo "  Results saved to:"
echo "    JSON: ${RESULTS_DIR}/tls-parsing-${DATE}.json"
echo "    Markdown: ${RESULTS_DIR}/tls-parsing-${DATE}.md"
