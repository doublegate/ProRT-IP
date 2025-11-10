#!/usr/bin/env bash
#
# Scenario 21: OS Fingerprinting Overhead
# Purpose: Validate OS fingerprinting overhead (16-probe sequence)
# Target: <30% overhead (16 probes + analysis)
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
echo "Running Scenario 21: OS Fingerprinting Overhead..."
echo "Binary: ${BINARY}"
echo "Comparing baseline vs OS detection (-O)"
echo ""

echo "=== Baseline (no OS detection) ==="
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/os-baseline-${DATE}.json" \
    "${BINARY} -sS -p 80 127.0.0.1"

echo ""
echo "=== With OS Detection (-O) ==="
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/os-detection-${DATE}.json" \
    "${BINARY} -sS -p 80 127.0.0.1 -O"

echo ""
echo "Results saved to:"
echo "  Baseline: ${RESULTS_DIR}/os-baseline-${DATE}.json"
echo "  OS Detection: ${RESULTS_DIR}/os-detection-${DATE}.json"
echo ""
echo "Calculate overhead: (os_time - baseline_time) / baseline_time * 100%"
