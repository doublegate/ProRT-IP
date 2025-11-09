#!/usr/bin/env bash
#
# Scenario 23: Packet Fragmentation Overhead
# Purpose: Validate packet fragmentation overhead
# Target: <20% overhead (extra packet crafting)
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
echo "Running Scenario 23: Packet Fragmentation Overhead..."
echo "Binary: ${BINARY}"
echo "Comparing baseline vs fragmentation (-f)"
echo ""

echo "=== Baseline (no fragmentation) ==="
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/frag-baseline-${DATE}.json" \
    "${BINARY} -sS -p 1-1000 127.0.0.1"

echo ""
echo "=== With Fragmentation (-f) ==="
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/fragmentation-${DATE}.json" \
    "${BINARY} -sS -p 1-1000 127.0.0.1 -f"

echo ""
echo "Results saved to:"
echo "  Baseline: ${RESULTS_DIR}/frag-baseline-${DATE}.json"
echo "  Fragmentation: ${RESULTS_DIR}/fragmentation-${DATE}.json"
echo ""
echo "Calculate overhead: (frag_time - baseline_time) / baseline_time * 100%"
