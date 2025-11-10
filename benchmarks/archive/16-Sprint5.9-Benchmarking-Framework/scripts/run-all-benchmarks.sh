#!/usr/bin/env bash
#
# Run All Benchmarks Orchestrator
# Executes all 20+ benchmark scenarios sequentially
#
# Usage:
#   ./run-all-benchmarks.sh              # Run all benchmarks
#   ./run-all-benchmarks.sh --baseline   # Save results as baseline
#   ./run-all-benchmarks.sh --compare baselines/baseline-v0.4.8.json
#

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FRAMEWORK_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
PROJECT_ROOT="$(cd "${FRAMEWORK_DIR}/../.." && pwd)"
BINARY="${PROJECT_ROOT}/target/release/prtip"
RESULTS_DIR="${FRAMEWORK_DIR}/results"
BASELINES_DIR="${FRAMEWORK_DIR}/baselines"
DATE=$(date +%Y%m%d-%H%M%S)
RUN_DIR="${RESULTS_DIR}/${DATE}"

# Parse arguments
SAVE_BASELINE=false
COMPARE_BASELINE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --baseline)
            SAVE_BASELINE=true
            shift
            ;;
        --compare)
            COMPARE_BASELINE="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--baseline] [--compare <baseline-file>]"
            exit 1
            ;;
    esac
done

# Validate binary exists
if [[ ! -f "${BINARY}" ]]; then
    echo "Error: ProRT-IP binary not found at ${BINARY}"
    echo "Run: cargo build --release"
    exit 1
fi

# Create run directory
mkdir -p "${RUN_DIR}"

# Print header
echo "============================================="
echo "ProRT-IP Benchmarking Framework"
echo "============================================="
echo "Date: $(date)"
echo "Binary: ${BINARY}"
echo "Version: $(${BINARY} --version 2>/dev/null || echo 'unknown')"
echo "Run directory: ${RUN_DIR}"
echo "Save baseline: ${SAVE_BASELINE}"
echo ""

# Benchmark list (20 scenarios - Sprint 5.5.4 expanded from 8)
declare -a BENCHMARKS=(
    # Original Sprint 5.9 scenarios (1-8)
    "01-syn-scan-1000-ports.sh"
    "02-connect-scan-common-ports.sh"
    "03-udp-scan-dns-snmp-ntp.sh"
    "04-service-detection-overhead.sh"
    "05-ipv6-overhead.sh"
    "06-idle-scan-timing.sh"
    "07-rate-limiting-overhead.sh"
    "08-tls-cert-parsing.sh"
    # Sprint 5.5.4 Scan Type Scenarios (9-12)
    "09-fin-scan-stealth.sh"
    "10-null-scan-stealth.sh"
    "11-xmas-scan-stealth.sh"
    "12-ack-scan-firewall-detection.sh"
    # Sprint 5.5.4 Scale Tests (15-20)
    "15-small-scan-1host-100ports.sh"
    "16-medium-scan-100hosts-1000ports.sh"
    "17-large-scan-1000hosts-100ports.sh"
    "18-all-ports-single-host.sh"
    "19-timing-t0-paranoid.sh"
    "20-timing-t5-insane.sh"
    # Sprint 5.5.4 Feature Overhead (21-25)
    "21-os-fingerprinting-overhead.sh"
    "22-banner-grabbing-overhead.sh"
    "23-evasion-fragmentation-overhead.sh"
    "24-evasion-decoys-overhead.sh"
    "25-event-system-overhead.sh"
)

# Run each benchmark
FAILED_BENCHMARKS=()
SKIPPED_BENCHMARKS=()

for bench in "${BENCHMARKS[@]}"; do
    echo "---------------------------------------------"
    echo "Running: ${bench}"
    echo "---------------------------------------------"

    if [[ -f "${SCRIPT_DIR}/${bench}" ]]; then
        # Run benchmark, capture exit code
        if "${SCRIPT_DIR}/${bench}"; then
            echo "✅ ${bench} completed"
        else
            EXIT_CODE=$?
            if [[ ${EXIT_CODE} -eq 0 ]]; then
                echo "⏭️ ${bench} skipped (optional, e.g., network unavailable)"
                SKIPPED_BENCHMARKS+=("${bench}")
            else
                echo "❌ ${bench} failed (exit code: ${EXIT_CODE})"
                FAILED_BENCHMARKS+=("${bench}")
            fi
        fi
    else
        echo "⚠️ Warning: ${bench} not found"
        SKIPPED_BENCHMARKS+=("${bench}")
    fi
    echo ""
done

# Move results to run directory (consolidate)
echo "Consolidating results..."
mv "${RESULTS_DIR}"/*.json "${RUN_DIR}/" 2>/dev/null || true
mv "${RESULTS_DIR}"/*.md "${RUN_DIR}/" 2>/dev/null || true

# Generate summary
echo "---------------------------------------------"
echo "Benchmark Summary"
echo "---------------------------------------------"
echo "Total benchmarks: ${#BENCHMARKS[@]}"
echo "Completed: $((${#BENCHMARKS[@]} - ${#FAILED_BENCHMARKS[@]} - ${#SKIPPED_BENCHMARKS[@]}))"
echo "Failed: ${#FAILED_BENCHMARKS[@]}"
echo "Skipped: ${#SKIPPED_BENCHMARKS[@]}"

if [[ ${#FAILED_BENCHMARKS[@]} -gt 0 ]]; then
    echo ""
    echo "Failed benchmarks:"
    for bench in "${FAILED_BENCHMARKS[@]}"; do
        echo "  - ${bench}"
    done
fi

if [[ ${#SKIPPED_BENCHMARKS[@]} -gt 0 ]]; then
    echo ""
    echo "Skipped benchmarks:"
    for bench in "${SKIPPED_BENCHMARKS[@]}"; do
        echo "  - ${bench}"
    done
fi

echo ""
echo "Results saved to: ${RUN_DIR}"
echo ""

# Save baseline if requested
if [[ "${SAVE_BASELINE}" == "true" ]]; then
    echo "Saving baseline..."
    mkdir -p "${BASELINES_DIR}"

    # Get version for baseline filename
    VERSION=$(${BINARY} --version 2>/dev/null | awk '{print $2}' || echo "unknown")
    BASELINE_FILE="${BASELINES_DIR}/baseline-${VERSION}.json"

    # Aggregate all JSON results into single baseline
    # NOTE: This is a simplified aggregation - real implementation would merge properly
    echo "Baseline would be saved to: ${BASELINE_FILE}"
    echo "TODO: Implement JSON aggregation (see scripts/aggregate-results.sh)"

    # Create metadata file
    METADATA_FILE="${BASELINES_DIR}/baseline-${VERSION}-metadata.md"
    cat > "${METADATA_FILE}" <<EOF
# Baseline Metadata: v${VERSION}

**Date:** $(date)
**Version:** ${VERSION}
**Platform:** $(uname -s) $(uname -r)
**CPU:** $(lscpu 2>/dev/null | grep "Model name" | cut -d: -f2 | xargs || echo "unknown")
**Cores:** $(nproc 2>/dev/null || echo "unknown")
**RAM:** $(free -h 2>/dev/null | awk '/^Mem:/{print $2}' || echo "unknown")
**hyperfine:** $(hyperfine --version 2>/dev/null || echo "unknown")

## Results

JSON file: ${BASELINE_FILE}

## Notes

Baseline established from run: ${DATE}

## Scenarios

$(ls "${RUN_DIR}"/*.json 2>/dev/null | wc -l) scenarios benchmarked

EOF

    echo "Baseline metadata saved to: ${METADATA_FILE}"
fi

# Compare against baseline if requested
if [[ -n "${COMPARE_BASELINE}" ]]; then
    if [[ -f "${COMPARE_BASELINE}" ]]; then
        echo "Comparing against baseline: ${COMPARE_BASELINE}"
        echo "TODO: Run regression detection script"
        echo "  ./scripts/analyze-results.sh ${COMPARE_BASELINE} ${RUN_DIR}/current.json"
    else
        echo "Error: Baseline file not found: ${COMPARE_BASELINE}"
        exit 1
    fi
fi

# Exit with error if any benchmarks failed
if [[ ${#FAILED_BENCHMARKS[@]} -gt 0 ]]; then
    echo "Exiting with error (benchmarks failed)"
    exit 1
fi

echo "✅ All benchmarks completed successfully"
exit 0
