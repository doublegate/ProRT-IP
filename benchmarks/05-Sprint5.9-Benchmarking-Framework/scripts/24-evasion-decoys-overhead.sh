#!/usr/bin/env bash
#
# Scenario 24: Decoy Scanning Overhead
# Purpose: Validate decoy scanning overhead (3 decoys = 4x traffic)
# Target: ~300% overhead (4x packets: 3 decoys + 1 real)
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
echo "Running Scenario 24: Decoy Scanning Overhead..."
echo "Binary: ${BINARY}"
echo "Comparing baseline vs decoy scanning (-D)"
echo ""

echo "=== Baseline (no decoys) ==="
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/decoy-baseline-${DATE}.json" \
    "${BINARY} -sS -p 1-1000 127.0.0.1"

echo ""
echo "=== With Decoys (3 decoys + 1 real = 4x traffic) ==="
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/decoy-scan-${DATE}.json" \
    "${BINARY} -sS -p 1-1000 127.0.0.1 -D 192.168.1.10,192.168.1.20,192.168.1.30"

echo ""
echo "Results saved to:"
echo "  Baseline: ${RESULTS_DIR}/decoy-baseline-${DATE}.json"
echo "  Decoys: ${RESULTS_DIR}/decoy-scan-${DATE}.json"
echo ""
echo "Calculate overhead: (decoy_time - baseline_time) / baseline_time * 100%"
echo "Expected: ~300% (4x packets)"
