#!/usr/bin/env bash
#
# Scenario 22: Banner Grabbing Overhead
# Purpose: Validate banner grabbing overhead
# Target: <15% overhead (3 banners, fast services)
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
echo "Running Scenario 22: Banner Grabbing Overhead..."
echo "Binary: ${BINARY}"
echo "Comparing baseline vs banner grabbing"
echo ""

echo "=== Baseline (SYN scan only) ==="
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/banner-baseline-${DATE}.json" \
    "${BINARY} -sS -p 22,80,443 127.0.0.1"

echo ""
echo "=== With Banner Grabbing (Connect + --banner-grab) ==="
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/banner-grab-${DATE}.json" \
    "${BINARY} -sT -p 22,80,443 127.0.0.1 --banner-grab"

echo ""
echo "Results saved to:"
echo "  Baseline: ${RESULTS_DIR}/banner-baseline-${DATE}.json"
echo "  Banner Grab: ${RESULTS_DIR}/banner-grab-${DATE}.json"
echo ""
echo "Calculate overhead: (banner_time - baseline_time) / baseline_time * 100%"
