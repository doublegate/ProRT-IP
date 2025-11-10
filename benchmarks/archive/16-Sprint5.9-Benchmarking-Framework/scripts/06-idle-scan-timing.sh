#!/usr/bin/env bash
#
# Scenario 6: Idle Scan Timing
# Purpose: Validate Sprint 5.3 idle scan claim (500-800ms per port)
# Target: 500-800ms per port
#
# NOTE: Requires zombie host setup (see ZOMBIE-SETUP.md)
# This is a placeholder script - requires manual zombie host configuration
#

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
BINARY="${PROJECT_ROOT}/target/release/prtip"
RESULTS_DIR="${SCRIPT_DIR}/../results"
DATE=$(date +%Y%m%d-%H%M%S)

# Zombie host configuration (MUST BE CONFIGURED)
ZOMBIE_IP="${ZOMBIE_IP:-UNCONFIGURED}"
TARGET_IP="${TARGET_IP:-127.0.0.1}"

# Validate binary exists
if [[ ! -f "${BINARY}" ]]; then
    echo "Error: ProRT-IP binary not found at ${BINARY}"
    echo "Run: cargo build --release"
    exit 1
fi

# Validate zombie host configured
if [[ "${ZOMBIE_IP}" == "UNCONFIGURED" ]]; then
    echo "Error: Zombie host not configured"
    echo "Set environment variables:"
    echo "  export ZOMBIE_IP=<zombie-host-ip>"
    echo "  export TARGET_IP=<target-host-ip>"
    echo ""
    echo "See ZOMBIE-SETUP.md for zombie host requirements"
    exit 1
fi

# Create results directory
mkdir -p "${RESULTS_DIR}"

# Run benchmark (fewer runs due to slow nature: 500-800ms/port)
echo "Running Scenario 6: Idle Scan Timing..."
echo "Binary: ${BINARY}"
echo "Zombie: ${ZOMBIE_IP}"
echo "Target: ${TARGET_IP}:80,443,8080"
echo "Expected: 500-800ms per port (total ~1.5-2.4 seconds)"
echo ""

hyperfine \
    --warmup 1 \
    --runs 5 \
    --export-json "${RESULTS_DIR}/idle-scan-${DATE}.json" \
    --export-markdown "${RESULTS_DIR}/idle-scan-${DATE}.md" \
    "${BINARY} -sI ${ZOMBIE_IP} -p 80,443,8080 ${TARGET_IP}"

# Calculate time per port (manual calculation from JSON)
echo ""
echo "Time per port calculation (manual):"
echo "  Time per port = total_time / number_of_ports"
echo "  Expected: 500-800ms per port (Sprint 5.3 claim)"
echo "  Results saved to:"
echo "    JSON: ${RESULTS_DIR}/idle-scan-${DATE}.json"
echo "    Markdown: ${RESULTS_DIR}/idle-scan-${DATE}.md"
