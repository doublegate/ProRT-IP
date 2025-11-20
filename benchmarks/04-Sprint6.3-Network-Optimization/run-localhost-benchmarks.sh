#!/bin/bash
# Sprint 6.3 Phase 3: Corrected Batch I/O Performance Benchmarking
# Localhost variant - eliminates network timeout bottleneck
#
# This benchmark scans localhost to measure pure batch I/O performance
# without network latency or timeout interference. This isolates syscall
# overhead as the primary bottleneck, making batch I/O improvements visible.
#
# Usage: sudo ./run-localhost-benchmarks.sh
# Requires: Linux kernel 3.0+, root privileges, hyperfine installed

set -e

# Disable history to avoid JSON corruption
export PRTIP_DISABLE_HISTORY=1

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PRTIP_BIN="../../target/release/prtip"
RESULTS_DIR="results/localhost"
WARMUP_RUNS=2
BENCHMARK_RUNS=10
LOCALHOST="127.0.0.1"

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

    # Detect platform
    PLATFORM=$(uname -s)
    if [ "$PLATFORM" != "Linux" ]; then
        echo -e "${YELLOW}WARNING: Running on $PLATFORM - batch I/O will use fallback mode${NC}"
    fi

    echo -e "${GREEN}✓ All prerequisites satisfied${NC}"
    echo ""
}

# Banner
print_banner() {
    echo -e "${BLUE}"
    echo "======================================================================="
    echo "  Sprint 6.3 Phase 3: Localhost Batch I/O Benchmarking"
    echo "======================================================================="
    echo -e "${NC}"
    echo "Target: 127.0.0.1 (eliminates network latency)"
    echo "Ports: 1-1000 (mix of open/closed, all respond < 1ms)"
    echo "Goal: Isolate syscall overhead as bottleneck"
    echo "Platform: $(uname -s) $(uname -r)"
    echo ""
}

# Scenario 1: Localhost Baseline (Single send/recv per packet)
scenario_1_localhost_baseline() {
    echo -e "${BLUE}=== Scenario 1: Localhost Baseline (Batch Size 1) ===${NC}"
    echo "Description: Single send/recv per packet, scanning localhost"
    echo "Expected: Syscall overhead dominant, 1000 ports × 2ms = ~2 seconds"
    echo ""

    hyperfine --warmup $WARMUP_RUNS --runs $BENCHMARK_RUNS \
        --export-json "$RESULTS_DIR/scenario_1_localhost_baseline.json" \
        --export-markdown "$RESULTS_DIR/scenario_1_localhost_baseline.md" \
        --command-name "Localhost Baseline (Batch 1)" \
        "$PRTIP_BIN -sS -p 1-1000 --output-file /dev/null $LOCALHOST"

    echo -e "${GREEN}✓ Scenario 1 complete${NC}"
    echo ""
}

# Scenario 2: Localhost Batch 32
scenario_2_localhost_batch_32() {
    echo -e "${BLUE}=== Scenario 2: Localhost Batch I/O (Batch Size 32) ===${NC}"
    echo "Description: sendmmsg/recvmmsg with 32 packets per syscall"
    echo "Expected: ~20-30% improvement (96.87% syscall reduction)"
    echo ""

    hyperfine --warmup $WARMUP_RUNS --runs $BENCHMARK_RUNS \
        --export-json "$RESULTS_DIR/scenario_2_localhost_batch_32.json" \
        --export-markdown "$RESULTS_DIR/scenario_2_localhost_batch_32.md" \
        --command-name "Localhost Batch 32" \
        "$PRTIP_BIN -sS -p 1-1000 --mmsg-batch-size 32 --output-file /dev/null $LOCALHOST"

    echo -e "${GREEN}✓ Scenario 2 complete${NC}"
    echo ""
}

# Scenario 3: Localhost Batch 256
scenario_3_localhost_batch_256() {
    echo -e "${BLUE}=== Scenario 3: Localhost Batch I/O (Batch Size 256) ===${NC}"
    echo "Description: sendmmsg/recvmmsg with 256 packets per syscall"
    echo "Expected: ~35-45% improvement (99.61% syscall reduction)"
    echo ""

    hyperfine --warmup $WARMUP_RUNS --runs $BENCHMARK_RUNS \
        --export-json "$RESULTS_DIR/scenario_3_localhost_batch_256.json" \
        --export-markdown "$RESULTS_DIR/scenario_3_localhost_batch_256.md" \
        --command-name "Localhost Batch 256" \
        "$PRTIP_BIN -sS -p 1-1000 --mmsg-batch-size 256 --output-file /dev/null $LOCALHOST"

    echo -e "${GREEN}✓ Scenario 3 complete${NC}"
    echo ""
}

# Scenario 4: Localhost Batch 1024 (Maximum)
scenario_4_localhost_batch_1024() {
    echo -e "${BLUE}=== Scenario 4: Localhost Batch I/O (Batch Size 1024) ===${NC}"
    echo "Description: sendmmsg/recvmmsg with 1024 packets per syscall"
    echo "Expected: ~45-55% improvement (99.90% syscall reduction)"
    echo ""

    hyperfine --warmup $WARMUP_RUNS --runs $BENCHMARK_RUNS \
        --export-json "$RESULTS_DIR/scenario_4_localhost_batch_1024.json" \
        --export-markdown "$RESULTS_DIR/scenario_4_localhost_batch_1024.md" \
        --command-name "Localhost Batch 1024" \
        "$PRTIP_BIN -sS -p 1-1000 --mmsg-batch-size 1024 --output-file /dev/null $LOCALHOST"

    echo -e "${GREEN}✓ Scenario 4 complete${NC}"
    echo ""
}

# Scenario 5: Localhost Large Port Range
scenario_5_localhost_large_range() {
    echo -e "${BLUE}=== Scenario 5: Localhost Large Range (10,000 ports, Batch 256) ===${NC}"
    echo "Description: Larger workload to amplify batch I/O benefits"
    echo "Expected: ~40-50% improvement over baseline"
    echo ""

    # First run baseline
    hyperfine --warmup $WARMUP_RUNS --runs $BENCHMARK_RUNS \
        --export-json "$RESULTS_DIR/scenario_5a_localhost_large_baseline.json" \
        --export-markdown "$RESULTS_DIR/scenario_5a_localhost_large_baseline.md" \
        --command-name "Localhost 10K Ports (Batch 1)" \
        "$PRTIP_BIN -sS -p 1-10000 --output-file /dev/null $LOCALHOST"

    # Then run batch
    hyperfine --warmup $WARMUP_RUNS --runs $BENCHMARK_RUNS \
        --export-json "$RESULTS_DIR/scenario_5b_localhost_large_batch.json" \
        --export-markdown "$RESULTS_DIR/scenario_5b_localhost_large_batch.md" \
        --command-name "Localhost 10K Ports (Batch 256)" \
        "$PRTIP_BIN -sS -p 1-10000 --mmsg-batch-size 256 --output-file /dev/null $LOCALHOST"

    echo -e "${GREEN}✓ Scenario 5 complete${NC}"
    echo ""
}

# Comparative Analysis
generate_comparison() {
    echo -e "${BLUE}=== Generating Comparative Analysis ===${NC}"
    echo ""

    # Create comparison markdown
    cat > "$RESULTS_DIR/00-LOCALHOST-COMPARISON.md" << 'EOF'
# Sprint 6.3 Localhost Batch I/O Performance Comparison

**Date:** $(date +%Y-%m-%d)
**Platform:** $(uname -s) $(uname -r)
**Binary:** prtip v0.5.2 (release build)
**Target:** 127.0.0.1 (localhost - eliminates network latency)

## Purpose

This benchmark eliminates network timeout bottleneck by scanning localhost,
where all responses arrive < 1ms. This isolates syscall overhead as the
primary performance factor, making batch I/O improvements measurable.

## Benchmark Results Summary

| Scenario | Ports | Batch Size | Mean Time | Improvement | Syscall Reduction | Status |
|----------|-------|------------|-----------|-------------|-------------------|--------|
EOF

    # Extract results from JSON files
    if [ -f "$RESULTS_DIR/scenario_1_localhost_baseline.json" ]; then
        baseline_mean=$(jq -r '.results[0].mean' "$RESULTS_DIR/scenario_1_localhost_baseline.json")
        echo "| Baseline | 1-1000 | 1 | ${baseline_mean}s | 0% (reference) | 0% | ✓ |" >> "$RESULTS_DIR/00-LOCALHOST-COMPARISON.md"

        # Calculate improvements for batch scenarios
        for scenario in 2 3 4; do
            result_file="$RESULTS_DIR/scenario_${scenario}_localhost_*.json"
            if ls $result_file 1> /dev/null 2>&1; then
                scenario_mean=$(jq -r '.results[0].mean' $(ls $result_file))
                improvement=$(echo "scale=2; (($baseline_mean - $scenario_mean) / $baseline_mean) * 100" | bc)

                # Determine batch size and syscall reduction
                case $scenario in
                    2) batch_size=32; syscall_reduction="96.87%"; target="20-30%" ;;
                    3) batch_size=256; syscall_reduction="99.61%"; target="35-45%" ;;
                    4) batch_size=1024; syscall_reduction="99.90%"; target="45-55%" ;;
                esac

                # Status check
                min_improvement=$(echo "$target" | cut -d'-' -f1)
                if (( $(echo "$improvement >= $min_improvement" | bc -l) )); then
                    status="✅ PASS"
                else
                    status="❌ FAIL"
                fi

                echo "| Batch $batch_size | 1-1000 | $batch_size | ${scenario_mean}s | +${improvement}% | $syscall_reduction | $status |" >> "$RESULTS_DIR/00-LOCALHOST-COMPARISON.md"
            fi
        done

        # Large range scenarios
        if [ -f "$RESULTS_DIR/scenario_5a_localhost_large_baseline.json" ]; then
            large_baseline=$(jq -r '.results[0].mean' "$RESULTS_DIR/scenario_5a_localhost_large_baseline.json")
            large_batch=$(jq -r '.results[0].mean' "$RESULTS_DIR/scenario_5b_localhost_large_batch.json")
            large_improvement=$(echo "scale=2; (($large_baseline - $large_batch) / $large_baseline) * 100" | bc)

            echo "| Baseline (Large) | 1-10000 | 1 | ${large_baseline}s | 0% (reference) | 0% | ✓ |" >> "$RESULTS_DIR/00-LOCALHOST-COMPARISON.md"
            echo "| Batch 256 (Large) | 1-10000 | 256 | ${large_batch}s | +${large_improvement}% | 99.61% | $(if (( $(echo "$large_improvement >= 40" | bc -l) )); then echo "✅ PASS"; else echo "❌ FAIL"; fi) |" >> "$RESULTS_DIR/00-LOCALHOST-COMPARISON.md"
        fi
    fi

    # Add analysis section
    cat >> "$RESULTS_DIR/00-LOCALHOST-COMPARISON.md" << 'EOF'

## Analysis

### Why Localhost Benchmarking?

The original benchmarks scanned random IPv4 addresses with 1000ms timeouts,
causing 99% of execution time to be network timeout waits. Batch I/O reduces
syscall overhead from ~1% to ~0.01%, which is unmeasurable when network I/O
dominates.

Localhost scanning ensures:
- All responses arrive < 1ms (vs 1000ms timeout)
- Syscall overhead becomes the bottleneck (not network I/O)
- Batch I/O improvements are measurable

### Performance Targets

- Batch 32: 20-30% improvement (96.87% syscall reduction)
- Batch 256: 35-45% improvement (99.61% syscall reduction)
- Batch 1024: 45-55% improvement (99.90% syscall reduction)

### Syscall Reduction Calculations

- 1000 ports × 2 syscalls (send + recv) = 2,000 syscalls
- Batch 32: 2,000 / 32 = 63 syscalls (96.87% reduction)
- Batch 256: 2,000 / 256 = 8 syscalls (99.61% reduction)
- Batch 1024: 2,000 / 1024 = 2 syscalls (99.90% reduction)

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

    echo -e "${BLUE}=== Starting Localhost Benchmark Suite (5 scenarios) ===${NC}"
    echo "This will take approximately 3-5 minutes..."
    echo ""

    scenario_1_localhost_baseline
    scenario_2_localhost_batch_32
    scenario_3_localhost_batch_256
    scenario_4_localhost_batch_1024
    scenario_5_localhost_large_range

    generate_comparison

    echo -e "${GREEN}"
    echo "======================================================================="
    echo "  Localhost Benchmarking Complete!"
    echo "======================================================================="
    echo -e "${NC}"
    echo "Results directory: $RESULTS_DIR/"
    echo "Comparison report: $RESULTS_DIR/00-LOCALHOST-COMPARISON.md"
    echo ""
    echo "Next steps:"
    echo "  1. Review comparison report for batch I/O validation"
    echo "  2. Verify ≥20% improvement for batch sizes 32+"
    echo "  3. Compare with original benchmarks (network timeout bottleneck)"
    echo "  4. Document in Sprint 6.3 completion report"
    echo ""
}

# Execute main
main
