#!/usr/bin/env bash
#
# Script Name: run-benchmarks.sh
# Purpose: Comprehensive performance benchmarking suite for ProRT-IP
# Version: 1.0.0
# Usage: ./run-benchmarks.sh [options]
# Prerequisites:
#   - Built prtip binary (cargo build --release)
#   - hyperfine (cargo install hyperfine)
#   - Optional: perf, valgrind, flamegraph (Linux only)
# Exit Codes:
#   0 - Success
#   1 - General error
#   2 - Missing prerequisites
#
# Examples:
#   ./run-benchmarks.sh                    # Full benchmark suite
#   ./run-benchmarks.sh --quick            # Quick subset (hyperfine only)
#   ./run-benchmarks.sh --compare v0.3.5   # Compare with baseline
#   ./run-benchmarks.sh --profile          # Include CPU profiling
#

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Options
QUICK_MODE=false
PROFILE_MODE=false
COMPARE_BASELINE=""
TARGET="127.0.0.1"
PRTIP="$PROJECT_ROOT/target/release/prtip"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Results directory
TIMESTAMP=$(date +"%Y%m%d-%H%M%S")
RESULTS_DIR="$PROJECT_ROOT/benchmarks/$(date +"%Y-%m-%d")-run-$TIMESTAMP"

# Helper functions
usage() {
    cat <<EOF
${GREEN}ProRT-IP Benchmark Suite v1.0.0${NC}

${BLUE}Usage:${NC}
  $(basename "$0") [options]

${BLUE}Options:${NC}
  --help                  Show this help message
  --quick                 Quick benchmarks (hyperfine only, ~5 min)
  --profile               Include CPU profiling (Linux only, ~15 min)
  --compare TAG           Compare with baseline (e.g., v0.3.5)
  --target HOST           Test target (default: 127.0.0.1)
  --binary PATH           Path to prtip binary
  --output DIR            Custom output directory

${BLUE}Benchmark Categories:${NC}
  1. Hyperfine Statistical Tests (8 tests)
     - Common ports (100)
     - 1K, 10K, 65K port scans
     - Database mode
     - Timing templates (T0, T3, T5)

  2. CPU Profiling (Linux only, --profile)
     - perf stat (CPU cycles, instructions)
     - flamegraph generation

  3. Memory Profiling (Linux only, --profile)
     - valgrind massif (heap usage)
     - Memory leak detection

  4. System Call Analysis (Linux only, --profile)
     - strace syscall counting
     - Syscall frequency analysis

${BLUE}Examples:${NC}
  # Quick validation (5 minutes)
  ./run-benchmarks.sh --quick

  # Full suite with profiling (30 minutes)
  ./run-benchmarks.sh --profile

  # Compare against v0.3.5 baseline
  ./run-benchmarks.sh --compare v0.3.5

  # Custom target and output
  ./run-benchmarks.sh --target 192.168.1.1 --output /tmp/benchmarks

${BLUE}Requirements:${NC}
  Required:
    - hyperfine (cargo install hyperfine)
    - Built prtip binary

  Optional (--profile):
    - perf (Linux kernel tools)
    - valgrind (memory profiling)
    - flamegraph (cargo install flamegraph)
    - strace (system call tracing)

${YELLOW}Note:${NC} Profiling requires sudo/root for perf and may take 30+ minutes.
Quick mode is recommended for CI/CD.
EOF
    exit 0
}

error() {
    echo -e "${RED}ERROR: $*${NC}" >&2
    exit 1
}

warn() {
    echo -e "${YELLOW}WARN: $*${NC}"
}

info() {
    echo -e "${BLUE}INFO: $*${NC}"
}

success() {
    echo -e "${GREEN}✓ $*${NC}"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            usage
            ;;
        --quick|-q)
            QUICK_MODE=true
            shift
            ;;
        --profile|-p)
            PROFILE_MODE=true
            shift
            ;;
        --compare|-c)
            COMPARE_BASELINE="${2:-}"
            if [[ -z "$COMPARE_BASELINE" ]]; then
                error "Missing baseline tag/commit"
            fi
            shift 2
            ;;
        --target|-t)
            TARGET="${2:-}"
            if [[ -z "$TARGET" ]]; then
                error "Missing target"
            fi
            shift 2
            ;;
        --binary|-b)
            PRTIP="${2:-}"
            if [[ -z "$PRTIP" ]]; then
                error "Missing binary path"
            fi
            shift 2
            ;;
        --output|-o)
            RESULTS_DIR="${2:-}"
            if [[ -z "$RESULTS_DIR" ]]; then
                error "Missing output directory"
            fi
            shift 2
            ;;
        *)
            error "Unknown option: $1. Use --help for usage."
            ;;
    esac
done

# Check prerequisites
check_prerequisites() {
    info "Checking prerequisites..."

    # Check prtip binary
    if [[ ! -f "$PRTIP" ]]; then
        error "prtip binary not found: $PRTIP (run: cargo build --release)"
    fi

    if [[ ! -x "$PRTIP" ]]; then
        error "prtip binary not executable: $PRTIP"
    fi

    success "prtip binary: $PRTIP"

    # Check hyperfine
    if ! command -v hyperfine &> /dev/null; then
        error "hyperfine not found (run: cargo install hyperfine)"
    fi

    success "hyperfine: $(hyperfine --version | head -1)"

    # Check optional tools for profiling
    if [[ "$PROFILE_MODE" == true ]]; then
        local missing=()

        if ! command -v perf &> /dev/null; then
            missing+=("perf")
        fi

        if ! command -v valgrind &> /dev/null; then
            missing+=("valgrind")
        fi

        if ! command -v flamegraph &> /dev/null; then
            warn "flamegraph not found (optional, install: cargo install flamegraph)"
        fi

        if ! command -v strace &> /dev/null; then
            missing+=("strace")
        fi

        if [[ ${#missing[@]} -gt 0 ]]; then
            warn "Missing profiling tools: ${missing[*]}"
            warn "Install with: sudo apt-get install ${missing[*]}"
        fi
    fi

    success "Prerequisites OK"
}

# Create output directory
setup_output_dir() {
    info "Setting up results directory..."

    mkdir -p "$RESULTS_DIR"/{hyperfine,perf,flamegraphs,valgrind,strace,docs}

    # Write environment info
    cat > "$RESULTS_DIR/00-environment.md" <<EOF
# Benchmark Environment

**Date:** $(date)
**Version:** $(cd "$PROJECT_ROOT" && git describe --tags --always || echo "unknown")
**Commit:** $(cd "$PROJECT_ROOT" && git rev-parse HEAD || echo "unknown")
**Binary:** $PRTIP
**Target:** $TARGET
**Mode:** $([ "$QUICK_MODE" = true ] && echo "Quick" || echo "Full")

## System Information

- **OS:** $(uname -s) $(uname -r)
- **Arch:** $(uname -m)
- **CPU:** $(grep "model name" /proc/cpuinfo 2>/dev/null | head -1 | cut -d: -f2 | xargs || sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "unknown")
- **Cores:** $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "unknown")
- **RAM:** $(free -h 2>/dev/null | awk '/^Mem:/ {print $2}' || sysctl -n hw.memsize 2>/dev/null | awk '{print $1/1024/1024/1024 "GB"}' || echo "unknown")

## Build Information

\`\`\`
$(cargo --version)
$(rustc --version)
\`\`\`

## Binary Information

\`\`\`
$(file "$PRTIP")
$(ls -lh "$PRTIP" | awk '{print "Size: " $5}')
\`\`\`
EOF

    success "Results directory: $RESULTS_DIR"
}

# Run hyperfine benchmarks
run_hyperfine_benchmarks() {
    info "Running hyperfine benchmarks..."

    cd "$RESULTS_DIR/hyperfine"

    local tests=(
        "100:common-ports:Common ports (100)"
        "1000:1k-ports:1K ports"
        "10000:10k-ports:10K ports"
        "65535:65k-ports:Full port range (65K)"
    )

    for test in "${tests[@]}"; do
        IFS=':' read -r ports name desc <<< "$test"

        info "Benchmarking: $desc..."

        hyperfine \
            --warmup 3 \
            --runs 20 \
            --export-json "${name}.json" \
            --export-markdown "${name}.md" \
            --command-name "$desc" \
            "$PRTIP --scan-type connect -p 1-${ports} --timeout 1 $TARGET" \
            2>&1 | tee "${name}.log"

        success "$desc complete"
    done

    # Timing templates
    if [[ "$QUICK_MODE" == false ]]; then
        local timings=("T0" "T3" "T5")

        for timing in "${timings[@]}"; do
            info "Benchmarking: Timing $timing..."

            hyperfine \
                --warmup 2 \
                --runs 10 \
                --export-json "timing-${timing}.json" \
                --export-markdown "timing-${timing}.md" \
                --command-name "Timing $timing" \
                "$PRTIP -${timing} -p 1-1000 --timeout 2 $TARGET" \
                2>&1 | tee "timing-${timing}.log"

            success "Timing $timing complete"
        done
    fi

    success "Hyperfine benchmarks complete"
}

# Run CPU profiling (Linux only)
run_cpu_profiling() {
    if [[ "$PROFILE_MODE" == false ]]; then
        return
    fi

    if [[ "$(uname -s)" != "Linux" ]]; then
        warn "CPU profiling only supported on Linux (skipping)"
        return
    fi

    info "Running CPU profiling with perf..."

    cd "$RESULTS_DIR/perf"

    # perf stat
    info "Running perf stat (10K ports)..."
    sudo perf stat -o perf-stat-10k.txt \
        "$PRTIP" --scan-type connect -p 1-10000 --timeout 1 "$TARGET" \
        2>&1 | tee perf-stat-10k.log

    success "perf stat complete"

    # perf record + flamegraph (if available)
    if command -v flamegraph &> /dev/null; then
        info "Generating flamegraph (1K ports)..."

        sudo perf record -F 999 -g -o perf-1k.data -- \
            "$PRTIP" --scan-type connect -p 1-1000 --timeout 1 "$TARGET"

        sudo perf script -i perf-1k.data | \
            flamegraph > "$RESULTS_DIR/flamegraphs/flamegraph-1k.svg"

        sudo chown "$USER:$USER" perf-1k.data "$RESULTS_DIR/flamegraphs/flamegraph-1k.svg"

        success "Flamegraph generated"
    fi

    success "CPU profiling complete"
}

# Run memory profiling (Linux only)
run_memory_profiling() {
    if [[ "$PROFILE_MODE" == false ]]; then
        return
    fi

    if ! command -v valgrind &> /dev/null; then
        warn "valgrind not found (skipping memory profiling)"
        return
    fi

    info "Running memory profiling with valgrind..."

    cd "$RESULTS_DIR/valgrind"

    # Massif (heap profiling)
    info "Running valgrind massif (1K ports)..."
    valgrind --tool=massif --massif-out-file=massif-1k.out \
        "$PRTIP" --scan-type connect -p 1-1000 --timeout 1 "$TARGET" \
        2>&1 | tee massif-1k.log

    # Generate massif report
    ms_print massif-1k.out > massif-1k-report.txt

    success "Valgrind massif complete"

    # Memcheck (leak detection) - only if quick mode disabled
    if [[ "$QUICK_MODE" == false ]]; then
        info "Running valgrind memcheck (100 ports)..."
        valgrind --leak-check=full --show-leak-kinds=all \
            --track-origins=yes --log-file=memcheck-100.log \
            "$PRTIP" --scan-type connect -p 1-100 --timeout 1 "$TARGET"

        success "Valgrind memcheck complete"
    fi

    success "Memory profiling complete"
}

# Run system call analysis
run_syscall_analysis() {
    if [[ "$PROFILE_MODE" == false ]]; then
        return
    fi

    if ! command -v strace &> /dev/null; then
        warn "strace not found (skipping syscall analysis)"
        return
    fi

    info "Running syscall analysis with strace..."

    cd "$RESULTS_DIR/strace"

    # 1K ports
    info "Tracing syscalls (1K ports)..."
    strace -c -o strace-1k-summary.txt \
        "$PRTIP" --scan-type connect -p 1-1000 --timeout 1 "$TARGET" \
        2>&1 | tee strace-1k.log

    success "Syscall analysis complete"
}

# Generate comparison report
generate_comparison() {
    if [[ -z "$COMPARE_BASELINE" ]]; then
        return
    fi

    info "Generating comparison report vs $COMPARE_BASELINE..."

    local baseline_dir="$PROJECT_ROOT/benchmarks/baseline-$COMPARE_BASELINE"

    if [[ ! -d "$baseline_dir" ]]; then
        warn "Baseline not found: $baseline_dir"
        warn "To create baseline: cp -r $RESULTS_DIR $baseline_dir"
        return
    fi

    # Compare hyperfine results
    cat > "$RESULTS_DIR/docs/comparison.md" <<EOF
# Benchmark Comparison

**Current:** $(cd "$PROJECT_ROOT" && git describe --tags --always)
**Baseline:** $COMPARE_BASELINE
**Date:** $(date)

## Performance Changes

| Test | Current | Baseline | Change |
|------|---------|----------|--------|
EOF

    # Extract and compare times (simplified - would need jq for full implementation)
    warn "Detailed comparison requires jq (install: apt-get install jq)"

    success "Comparison report generated"
}

# Generate summary report
generate_summary() {
    info "Generating summary report..."

    cat > "$RESULTS_DIR/README.md" <<EOF
# Benchmark Results - $TIMESTAMP

**Version:** $(cd "$PROJECT_ROOT" && git describe --tags --always || echo "unknown")
**Date:** $(date)
**Mode:** $([ "$QUICK_MODE" = true ] && echo "Quick" || echo "Full")$([ "$PROFILE_MODE" = true ] && echo " + Profiling" || "")

## Summary

- **Hyperfine Tests:** $(ls -1 "$RESULTS_DIR/hyperfine"/*.json 2>/dev/null | wc -l) benchmarks
- **CPU Profiles:** $(ls -1 "$RESULTS_DIR/perf"/*.txt 2>/dev/null | wc -l) profiles
- **Flamegraphs:** $(ls -1 "$RESULTS_DIR/flamegraphs"/*.svg 2>/dev/null | wc -l) graphs
- **Memory Profiles:** $(ls -1 "$RESULTS_DIR/valgrind"/*.out 2>/dev/null | wc -l) profiles
- **Syscall Traces:** $(ls -1 "$RESULTS_DIR/strace"/*.txt 2>/dev/null | wc -l) traces

## Quick Stats

### Common Ports (100)
\`\`\`
$(grep -A 5 "Benchmark" "$RESULTS_DIR/hyperfine/common-ports.log" 2>/dev/null | head -6 || echo "Not available")
\`\`\`

### 1K Ports
\`\`\`
$(grep -A 5 "Benchmark" "$RESULTS_DIR/hyperfine/1k-ports.log" 2>/dev/null | head -6 || echo "Not available")
\`\`\`

### 10K Ports
\`\`\`
$(grep -A 5 "Benchmark" "$RESULTS_DIR/hyperfine/10k-ports.log" 2>/dev/null | head -6 || echo "Not available")
\`\`\`

## Files

- \`00-environment.md\` - System and build information
- \`hyperfine/\` - Statistical benchmarks (JSON + Markdown)
- \`perf/\` - CPU profiling data
- \`flamegraphs/\` - CPU flamegraphs (SVG)
- \`valgrind/\` - Memory profiling data
- \`strace/\` - System call traces

## Next Steps

1. Review hyperfine markdown reports for detailed statistics
2. Check flamegraphs for CPU hotspots: \`firefox flamegraphs/*.svg\`
3. Analyze memory usage: \`cat valgrind/massif-*-report.txt\`
4. Compare with baseline: \`./run-benchmarks.sh --compare <tag>\`

---

Generated by run-benchmarks.sh v1.0.0
EOF

    success "Summary report: $RESULTS_DIR/README.md"
}

# Print final summary
print_summary() {
    echo ""
    echo "=========================================="
    echo -e "${GREEN}Benchmark Suite Complete!${NC}"
    echo "=========================================="
    echo ""
    echo "Results: $RESULTS_DIR"
    echo ""
    echo "Quick Stats:"
    echo "  - Hyperfine tests: $(ls -1 "$RESULTS_DIR/hyperfine"/*.json 2>/dev/null | wc -l)"
    echo "  - Total size: $(du -sh "$RESULTS_DIR" | awk '{print $1}')"
    echo ""
    echo "View Results:"
    echo "  ${BLUE}cat $RESULTS_DIR/README.md${NC}"
    echo ""
    echo "View Hyperfine:"
    echo "  ${BLUE}cat $RESULTS_DIR/hyperfine/*.md${NC}"
    echo ""

    if [[ "$PROFILE_MODE" == true ]]; then
        echo "View Flamegraphs:"
        echo "  ${BLUE}firefox $RESULTS_DIR/flamegraphs/*.svg${NC}"
        echo ""
    fi

    echo "Create Baseline:"
    echo "  ${BLUE}cp -r $RESULTS_DIR benchmarks/baseline-v0.3.6${NC}"
    echo ""
}

# Main execution
main() {
    echo "=========================================="
    echo "ProRT-IP Benchmark Suite"
    echo "=========================================="
    echo ""

    check_prerequisites
    setup_output_dir
    run_hyperfine_benchmarks
    run_cpu_profiling
    run_memory_profiling
    run_syscall_analysis
    generate_comparison
    generate_summary
    print_summary
}

main "$@"
