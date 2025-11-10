#!/usr/bin/env bash
#
# Scenario 4: Service Detection Overhead
# Purpose: Validate 85-90% accuracy + low overhead
# Target: <10% overhead vs plain scan
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

# Run benchmark (parallel comparison: baseline vs detection)
echo "Running Scenario 4: Service Detection Overhead..."
echo "Binary: ${BINARY}"
echo "Target: 127.0.0.1:22,80,443"
echo "Baseline: SYN scan without -sV"
echo "Detection: SYN scan with -sV"
echo ""

hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/service-detection-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/service-detection-${DATE}.md" \
    --command-name "baseline" \
    "${BINARY} -sS -p 22,80,443 127.0.0.1" \
    --command-name "detection" \
    "${BINARY} -sV -p 22,80,443 127.0.0.1"

# Calculate overhead (manual calculation from JSON)
echo ""
echo "Overhead calculation (manual):"
echo "  Overhead = (detection_mean - baseline_mean) / baseline_mean * 100"
echo "  Target: <10%"
echo "  Results saved to:"
echo "    JSON: ${RESULTS_DIR}/service-detection-${DATE}.json"
echo "    Markdown: ${RESULTS_DIR}/service-detection-${DATE}.md"
