#!/usr/bin/env bash
#
# Scenario 25: Event System Overhead
# Purpose: Validate event system overhead (Sprint 5.5.3)
# Target: <5% overhead (minimal pub-sub overhead)
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
echo "Running Scenario 25: Event System Overhead..."
echo "Binary: ${BINARY}"
echo "Comparing baseline vs event logging"
echo ""

echo "=== Baseline (no event logging) ==="
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/event-baseline-${DATE}.json" \
    "${BINARY} -sS -p 1-1000 127.0.0.1"

echo ""
echo "=== With Event Logging ==="
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "${RESULTS_DIR}/event-logging-${DATE}.json" \
    --prepare "rm -f /tmp/events.jsonl" \
    "${BINARY} -sS -p 1-1000 127.0.0.1 --event-log /tmp/events.jsonl"

echo ""
echo "Results saved to:"
echo "  Baseline: ${RESULTS_DIR}/event-baseline-${DATE}.json"
echo "  Event Logging: ${RESULTS_DIR}/event-logging-${DATE}.json"
echo ""
echo "Calculate overhead: (event_time - baseline_time) / baseline_time * 100%"
