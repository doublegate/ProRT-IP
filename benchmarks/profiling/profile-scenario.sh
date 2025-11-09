#!/bin/bash
# profile-scenario.sh - Standardized profiling wrapper for ProRT-IP
# Sprint 5.5.5 - Profiling Execution
# Version: 1.0.0

set -euo pipefail

# Default values
SCENARIO=""
TYPE="cpu"
OUTPUT_DIR="benchmarks/profiling/results"
SAMPLING_RATE=99
SCENARIO_ARGS=()

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Usage function
usage() {
    cat <<EOF
Usage: $0 --scenario <name> [OPTIONS]

Standardized profiling wrapper for ProRT-IP performance analysis.

Required Arguments:
    --scenario <name>       Scenario identifier (e.g., "syn-scan-1k")

Optional Arguments:
    --type <cpu|memory|io>  Profiling type (default: cpu)
    --output-dir <path>     Output directory (default: benchmarks/profiling/results)
    --sampling-rate <Hz>    CPU sampling rate (default: 99Hz)
    --help                  Show this help message

Scenario Arguments:
    All additional arguments after options are passed to prtip.
    Example: --scenario syn-scan -- -sS -p 1-1000 127.0.0.1

Examples:
    # CPU profiling (flamegraph)
    $0 --scenario syn-scan-1k --type cpu -- -sS -p 1-1000 127.0.0.1

    # Memory profiling (massif)
    $0 --scenario syn-scan-1k --type memory -- -sS -p 1-1000 127.0.0.1

    # I/O profiling (strace)
    $0 --scenario syn-scan-1k --type io -- -sS -p 1-1000 127.0.0.1

Output Files:
    CPU:    \${OUTPUT_DIR}/flamegraphs/\${SCENARIO}-flamegraph.svg
    Memory: \${OUTPUT_DIR}/massif/\${SCENARIO}-massif.out + report.txt
    I/O:    \${OUTPUT_DIR}/strace/\${SCENARIO}-strace-summary.txt

EOF
    exit 1
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --scenario)
            SCENARIO="$2"
            shift 2
            ;;
        --type)
            TYPE="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --sampling-rate)
            SAMPLING_RATE="$2"
            shift 2
            ;;
        --help)
            usage
            ;;
        --)
            shift
            SCENARIO_ARGS=("$@")
            break
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            usage
            ;;
    esac
done

# Validate required arguments
if [[ -z "$SCENARIO" ]]; then
    echo -e "${RED}Error: --scenario is required${NC}"
    usage
fi

# Validate profiling type
if [[ ! "$TYPE" =~ ^(cpu|memory|io)$ ]]; then
    echo -e "${RED}Error: --type must be cpu, memory, or io${NC}"
    usage
fi

# Validate scenario arguments
if [[ ${#SCENARIO_ARGS[@]} -eq 0 ]]; then
    echo -e "${RED}Error: No scenario arguments provided (use -- before prtip args)${NC}"
    usage
fi

# Ensure output directories exist
mkdir -p "${OUTPUT_DIR}/flamegraphs"
mkdir -p "${OUTPUT_DIR}/massif"
mkdir -p "${OUTPUT_DIR}/strace"

# Check if release binary exists
if [[ ! -f "target/release/prtip" ]]; then
    echo -e "${YELLOW}Warning: Release binary not found, building...${NC}"
    cargo build --release
fi

# Print profiling configuration
echo -e "${GREEN}=== ProRT-IP Profiling ===${NC}"
echo "Scenario:      $SCENARIO"
echo "Type:          $TYPE"
echo "Output Dir:    $OUTPUT_DIR"
echo "Arguments:     ${SCENARIO_ARGS[*]}"
echo ""

# Execute profiling based on type
case "$TYPE" in
    cpu)
        OUTPUT_FILE="${OUTPUT_DIR}/flamegraphs/${SCENARIO}-flamegraph.svg"
        echo -e "${GREEN}Running CPU profiling (flamegraph)...${NC}"
        echo "Output: $OUTPUT_FILE"

        # Use cargo-flamegraph
        cargo flamegraph \
            --bin prtip \
            --freq "$SAMPLING_RATE" \
            --output "$OUTPUT_FILE" \
            -- "${SCENARIO_ARGS[@]}"

        echo -e "${GREEN}✓ Flamegraph generated: $OUTPUT_FILE${NC}"
        echo "View with: firefox $OUTPUT_FILE"
        ;;

    memory)
        OUTPUT_FILE="${OUTPUT_DIR}/massif/${SCENARIO}-massif.out"
        REPORT_FILE="${OUTPUT_DIR}/massif/${SCENARIO}-massif-report.txt"

        echo -e "${GREEN}Running memory profiling (massif)...${NC}"
        echo "Output: $OUTPUT_FILE"
        echo "Report: $REPORT_FILE"

        # Run valgrind massif
        valgrind --tool=massif \
            --massif-out-file="$OUTPUT_FILE" \
            target/release/prtip "${SCENARIO_ARGS[@]}"

        # Generate human-readable report
        ms_print "$OUTPUT_FILE" > "$REPORT_FILE"

        echo -e "${GREEN}✓ Massif profile generated: $OUTPUT_FILE${NC}"
        echo -e "${GREEN}✓ Massif report generated: $REPORT_FILE${NC}"
        echo "View with: less $REPORT_FILE"
        ;;

    io)
        SUMMARY_FILE="${OUTPUT_DIR}/strace/${SCENARIO}-strace-summary.txt"
        DETAIL_FILE="${OUTPUT_DIR}/strace/${SCENARIO}-strace-detail.txt"

        echo -e "${GREEN}Running I/O profiling (strace)...${NC}"
        echo "Summary: $SUMMARY_FILE"
        echo "Detail:  $DETAIL_FILE"

        # Run strace with summary
        strace -c -o "$SUMMARY_FILE" \
            target/release/prtip "${SCENARIO_ARGS[@]}"

        # Run strace with detail on key syscalls
        strace -e sendmmsg,recvmmsg,write,writev -o "$DETAIL_FILE" \
            target/release/prtip "${SCENARIO_ARGS[@]}" 2>&1 || true

        echo -e "${GREEN}✓ Strace summary generated: $SUMMARY_FILE${NC}"
        echo -e "${GREEN}✓ Strace detail generated: $DETAIL_FILE${NC}"
        echo "View with: less $SUMMARY_FILE"
        ;;
esac

echo -e "${GREEN}=== Profiling Complete ===${NC}"
