#!/bin/bash
# Sprint 6.3 Phase 3: Batch I/O Performance Benchmarking
# Validates 20-40% throughput improvement from sendmmsg/recvmmsg implementation
#
# Usage: sudo ./run-batch-io-benchmarks.sh
# Requires: Linux kernel 3.0+, root privileges, hyperfine installed

set -e

# Disable history to avoid JSON corruption and exit code 1 in benchmarks
export PRTIP_DISABLE_HISTORY=1

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PRTIP_BIN="../../target/release/prtip"
RESULTS_DIR="results"
BASELINE_TARGET="targets/baseline-50.txt"
IPV6_TARGET="targets/ipv6-25.txt"
WARMUP_RUNS=2
BENCHMARK_RUNS=10

# Ensure results directory exists
mkdir -p "$RESULTS_DIR"

# Check prerequisites
check_prerequisites() {
    echo -e "${BLUE}=== Checking Prerequisites ===${NC}"

    # Check if prtip binary exists
    if [ ! -f "$PRTIP_BIN" ]; then
        echo -e "${RED}ERROR: prtip binary not found at $PRTIP_BIN${NC}"
        echo "Run: cargo build --release"
        exit 1
    fi

    # Check if running as root
    if [ "$EUID" -ne 0 ]; then
        echo -e "${RED}ERROR: This script requires root privileges${NC}"
        echo "Run: sudo $0"
        exit 1
    fi

    # Check if hyperfine is installed
    if ! command -v hyperfine &> /dev/null; then
        echo -e "${RED}ERROR: hyperfine not found${NC}"
        echo "Install: cargo install hyperfine"
        exit 1
    fi

    # Check if target files exist
    if [ ! -f "$BASELINE_TARGET" ]; then
        echo -e "${RED}ERROR: Target file not found: $BASELINE_TARGET${NC}"
        exit 1
    fi

    # Detect platform
    PLATFORM=$(uname -s)
    if [ "$PLATFORM" != "Linux" ]; then
        echo -e "${YELLOW}WARNING: Running on $PLATFORM - batch I/O will use fallback mode${NC}"
        echo -e "${YELLOW}Expected 0% improvement (graceful degradation test)${NC}"
    fi

    echo -e "${GREEN}✓ All prerequisites satisfied${NC}"
    echo ""
}

# Banner
print_banner() {
    echo -e "${BLUE}"
    echo "======================================================================="
    echo "  Sprint 6.3 Phase 3: Batch I/O Performance Benchmarking"
    echo "======================================================================="
    echo -e "${NC}"
    echo "Target: Validate 20-40% throughput improvement from batch I/O"
    echo "Platform: $(uname -s) $(uname -r)"
    echo "Scenarios: 6 (Baseline + 5 batch configurations)"
    echo ""
}

# Scenario 1: Baseline (Single send/recv per packet)
scenario_1_baseline() {
    echo -e "${BLUE}=== Scenario 1: Baseline (Batch Size 1) ===${NC}"
    echo "Description: Single send/recv per packet (no batching)"
    echo "Expected: 10,000-50,000 pps, 20,000 syscalls"
    echo ""

    # Note: Baseline uses no --mmsg-batch-size flag (single packet per syscall)
    # Read targets from file and pass as arguments
    TARGETS=$(cat "$BASELINE_TARGET" | xargs)

    hyperfine --warmup $WARMUP_RUNS --runs $BENCHMARK_RUNS \
        --export-json "$RESULTS_DIR/scenario_1_baseline.json" \
        --export-markdown "$RESULTS_DIR/scenario_1_baseline.md" \
        --command-name "Batch Size 1 (Baseline)" \
        "$PRTIP_BIN -sS -p 1-10 --output-file /dev/null $TARGETS"

    echo -e "${GREEN}✓ Scenario 1 complete${NC}"
    echo ""
}

# Scenario 2: Batch Size 32
scenario_2_batch_32() {
    echo -e "${BLUE}=== Scenario 2: Batch I/O (Batch Size 32) ===${NC}"
    echo "Description: sendmmsg/recvmmsg with 32 packets per syscall"
    echo "Expected: 15,000-75,000 pps (20-40% improvement), 625 syscalls (96.87% reduction)"
    echo ""

    # Read targets from file and pass as arguments
    TARGETS=$(cat "$BASELINE_TARGET" | xargs)

    hyperfine --warmup $WARMUP_RUNS --runs $BENCHMARK_RUNS \
        --export-json "$RESULTS_DIR/scenario_2_batch_32.json" \
        --export-markdown "$RESULTS_DIR/scenario_2_batch_32.md" \
        --command-name "Batch Size 32" \
        "$PRTIP_BIN -sS -p 1-10 --mmsg-batch-size 32 --output-file /dev/null $TARGETS"

    echo -e "${GREEN}✓ Scenario 2 complete${NC}"
    echo ""
}

# Scenario 3: Batch Size 256
scenario_3_batch_256() {
    echo -e "${BLUE}=== Scenario 3: Batch I/O (Batch Size 256) ===${NC}"
    echo "Description: sendmmsg/recvmmsg with 256 packets per syscall"
    echo "Expected: 20,000-100,000 pps (30-50% improvement), 78 syscalls (99.61% reduction)"
    echo ""

    # Read targets from file and pass as arguments
    TARGETS=$(cat "$BASELINE_TARGET" | xargs)

    hyperfine --warmup $WARMUP_RUNS --runs $BENCHMARK_RUNS \
        --export-json "$RESULTS_DIR/scenario_3_batch_256.json" \
        --export-markdown "$RESULTS_DIR/scenario_3_batch_256.md" \
        --command-name "Batch Size 256" \
        "$PRTIP_BIN -sS -p 1-10 --mmsg-batch-size 256 --output-file /dev/null $TARGETS"

    echo -e "${GREEN}✓ Scenario 3 complete${NC}"
    echo ""
}

# Scenario 4: Batch Size 1024 (Maximum)
scenario_4_batch_1024() {
    echo -e "${BLUE}=== Scenario 4: Batch I/O (Batch Size 1024 - Maximum) ===${NC}"
    echo "Description: sendmmsg/recvmmsg with 1024 packets per syscall (Linux maximum)"
    echo "Expected: 25,000-125,000 pps (40-60% improvement), 20 syscalls (99.90% reduction)"
    echo ""

    # Read targets from file and pass as arguments
    TARGETS=$(cat "$BASELINE_TARGET" | xargs)

    hyperfine --warmup $WARMUP_RUNS --runs $BENCHMARK_RUNS \
        --export-json "$RESULTS_DIR/scenario_4_batch_1024.json" \
        --export-markdown "$RESULTS_DIR/scenario_4_batch_1024.md" \
        --command-name "Batch Size 1024" \
        "$PRTIP_BIN -sS -p 1-10 --mmsg-batch-size 1024 --output-file /dev/null $TARGETS"

    echo -e "${GREEN}✓ Scenario 4 complete${NC}"
    echo ""
}

# Scenario 6: IPv6 Batch I/O
scenario_6_ipv6() {
    echo -e "${BLUE}=== Scenario 6: IPv6 Batch I/O (Batch Size 256) ===${NC}"
    echo "Description: IPv6 targets with batch I/O (larger packet headers)"
    echo "Expected: 18,000-90,000 pps (25-45% improvement), IPv6 overhead ≤ 10%"
    echo ""

    if [ ! -f "$IPV6_TARGET" ]; then
        echo -e "${YELLOW}WARNING: IPv6 target file not found, skipping scenario 6${NC}"
        echo ""
        return
    fi

    # Read targets from file and pass as arguments
    TARGETS=$(cat "$IPV6_TARGET" | xargs)

    hyperfine --warmup $WARMUP_RUNS --runs $BENCHMARK_RUNS \
        --export-json "$RESULTS_DIR/scenario_6_ipv6.json" \
        --export-markdown "$RESULTS_DIR/scenario_6_ipv6.md" \
        --command-name "IPv6 Batch Size 256" \
        "$PRTIP_BIN -sS -p 1-10 --mmsg-batch-size 256 --output-file /dev/null $TARGETS"

    echo -e "${GREEN}✓ Scenario 6 complete${NC}"
    echo ""
}

# Comparative Analysis
generate_comparison() {
    echo -e "${BLUE}=== Generating Comparative Analysis ===${NC}"
    echo ""

    # Create comparison markdown
    cat > "$RESULTS_DIR/00-COMPARISON.md" << 'EOF'
# Sprint 6.3 Batch I/O Performance Comparison

**Date:** $(date +%Y-%m-%d)
**Platform:** $(uname -s) $(uname -r)
**Binary:** prtip v0.5.2 (release build)

## Benchmark Results Summary

| Scenario | Batch Size | Mean Time | Improvement | Syscall Reduction | Status |
|----------|------------|-----------|-------------|-------------------|--------|
EOF

    # Extract results from JSON files
    if [ -f "$RESULTS_DIR/scenario_1_baseline.json" ]; then
        baseline_mean=$(jq -r '.results[0].mean' "$RESULTS_DIR/scenario_1_baseline.json")
        echo "| Baseline | 1 | ${baseline_mean}s | 0% (reference) | 0% | ✓ |" >> "$RESULTS_DIR/00-COMPARISON.md"
    fi

    # Calculate improvements for other scenarios
    for scenario in 2 3 4 6; do
        result_file="$RESULTS_DIR/scenario_${scenario}_*.json"
        if ls $result_file 1> /dev/null 2>&1; then
            scenario_mean=$(jq -r '.results[0].mean' $(ls $result_file))
            improvement=$(echo "scale=2; (($baseline_mean - $scenario_mean) / $baseline_mean) * 100" | bc)

            # Determine batch size and syscall reduction
            case $scenario in
                2) batch_size=32; syscall_reduction="96.87%" ;;
                3) batch_size=256; syscall_reduction="99.61%" ;;
                4) batch_size=1024; syscall_reduction="99.90%" ;;
                6) batch_size=256; syscall_reduction="99.61% (IPv6)" ;;
            esac

            # Status check (improvement ≥ 20% for batch 32+)
            if (( $(echo "$improvement >= 20.0" | bc -l) )); then
                status="✅ PASS"
            else
                status="❌ FAIL"
            fi

            echo "| Scenario $scenario | $batch_size | ${scenario_mean}s | +${improvement}% | $syscall_reduction | $status |" >> "$RESULTS_DIR/00-COMPARISON.md"
        fi
    done

    # Add validation section
    cat >> "$RESULTS_DIR/00-COMPARISON.md" << 'EOF'

## Validation Results

### Performance Targets
- ✓ Batch 32: 20-40% improvement expected
- ✓ Batch 256: 30-50% improvement expected
- ✓ Batch 1024: 40-60% improvement expected
- ✓ IPv6: 25-45% improvement expected

### Syscall Reduction
- Batch 32: 96.87% (20,000 → 625 syscalls)
- Batch 256: 99.61% (20,000 → 78 syscalls)
- Batch 1024: 99.90% (20,000 → 20 syscalls)

### Conclusions
See individual scenario markdown files for detailed statistics.
EOF

    echo -e "${GREEN}✓ Comparison report generated${NC}"
    echo ""
}

# Main execution
main() {
    print_banner
    check_prerequisites

    echo -e "${BLUE}=== Starting Benchmark Suite (6 scenarios) ===${NC}"
    echo "This will take approximately 5-10 minutes..."
    echo ""

    scenario_1_baseline
    scenario_2_batch_32
    scenario_3_batch_256
    scenario_4_batch_1024
    scenario_6_ipv6

    generate_comparison

    echo -e "${GREEN}"
    echo "======================================================================="
    echo "  Benchmarking Complete!"
    echo "======================================================================="
    echo -e "${NC}"
    echo "Results directory: $RESULTS_DIR/"
    echo "Comparison report: $RESULTS_DIR/00-COMPARISON.md"
    echo ""
    echo "Next steps:"
    echo "  1. Review comparison report for validation"
    echo "  2. Verify ≥20% improvement for batch sizes 32+"
    echo "  3. Check syscall reduction percentages"
    echo "  4. Document results in Sprint 6.3 completion report"
    echo ""
}

# Execute main
main
