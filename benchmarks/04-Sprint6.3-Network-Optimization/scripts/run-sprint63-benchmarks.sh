#!/bin/bash
#
# Sprint 6.3 Network Optimization Benchmark Runner
# Executes comprehensive benchmark suite for CI/CD and manual testing
#
# Usage: ./run-sprint63-benchmarks.sh [--quick|--full|--ci]
# Modes:
#   --quick: Fast localhost tests (<2 minutes, suitable for PR checks)
#   --full:  Complete benchmark suite (<10 minutes, weekly regression)
#   --ci:    CI/CD optimized (2-5 minutes, automated testing)
#
# Requirements:
#   - Compiled release binary (cargo build --release)
#   - hyperfine installed (cargo install hyperfine)
#   - Root privileges for raw sockets (sudo)
#   - Linux kernel 3.0+ for batch I/O (fallback on macOS/Windows)
#

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(cd "$BENCHMARK_DIR/../.." && pwd)"
BINARY="$PROJECT_ROOT/target/release/prtip"
RESULTS_DIR="$BENCHMARK_DIR/results"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
MODE="${1:-quick}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check binary
    if [ ! -f "$BINARY" ]; then
        log_error "Release binary not found: $BINARY"
        log_info "Run: cargo build --release"
        exit 1
    fi
    log_success "Binary found: $BINARY"

    # Check hyperfine
    if ! command -v hyperfine &> /dev/null; then
        log_error "hyperfine not installed"
        log_info "Install: cargo install hyperfine --version 1.18.0"
        exit 1
    fi
    log_success "hyperfine found: $(hyperfine --version)"

    # Check root (only warn, since some tests work without)
    if [ "$EUID" -ne 0 ]; then
        log_warn "Not running as root - some benchmarks may fail"
        log_info "Run with: sudo $0 $*"
    fi

    # Check platform
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        log_success "Platform: Linux (full batch I/O support)"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        log_warn "Platform: macOS (batch I/O fallback mode)"
    else
        log_warn "Platform: $OSTYPE (batch I/O fallback mode)"
    fi

    # Create results directory
    mkdir -p "$RESULTS_DIR"
    log_success "Results directory: $RESULTS_DIR"
}

run_benchmark() {
    local name=$1
    local command=$2
    local warmup=${3:-2}
    local runs=${4:-5}

    log_info "Running benchmark: $name"

    # Generate unique result filename
    local result_file="$RESULTS_DIR/${name}_${TIMESTAMP}.json"

    # Run hyperfine with JSON export
    hyperfine \
        --warmup "$warmup" \
        --runs "$runs" \
        --export-json "$result_file" \
        --show-output \
        "$command" || {
            log_error "Benchmark failed: $name"
            return 1
        }

    log_success "Benchmark completed: $result_file"
    echo "$result_file"
}

# Benchmark Suite: Quick Mode (PR checks)
run_quick_benchmarks() {
    log_info "=== Quick Benchmark Mode ==="
    log_info "Estimated time: <2 minutes"
    log_info "Purpose: Fast PR validation"
    echo ""

    local results=()

    # 1. Baseline localhost scan (100 ports)
    results+=($(run_benchmark \
        "quick_baseline_100ports" \
        "sudo $BINARY -sS -p 1-100 127.0.0.1" \
        1 3))

    # 2. Batch I/O localhost (256 batch size)
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        results+=($(run_benchmark \
            "quick_batch_256_100ports" \
            "sudo $BINARY -sS -p 1-100 --batch-size 256 127.0.0.1" \
            1 3))
    else
        log_warn "Skipping batch I/O (Linux-only)"
    fi

    # 3. CDN filtering (if targets exist)
    if [ -f "$BENCHMARK_DIR/targets/baseline-50.txt" ]; then
        results+=($(run_benchmark \
            "quick_cdn_filtering_50ips" \
            "sudo $BINARY -sS -p 80,443 --cdn-filter --target-file $BENCHMARK_DIR/targets/baseline-50.txt" \
            1 3))
    else
        log_warn "Skipping CDN test (targets/baseline-50.txt not found)"
    fi

    log_success "Quick benchmarks complete: ${#results[@]} scenarios"
    generate_summary "${results[@]}"
}

# Benchmark Suite: Full Mode (comprehensive validation)
run_full_benchmarks() {
    log_info "=== Full Benchmark Mode ==="
    log_info "Estimated time: 8-10 minutes"
    log_info "Purpose: Comprehensive performance validation"
    echo ""

    local results=()

    # CDN Deduplication Benchmarks
    log_info "--- CDN Deduplication Suite ---"

    if [ -f "$BENCHMARK_DIR/targets/baseline-1000.txt" ]; then
        # Baseline (no filtering)
        results+=($(run_benchmark \
            "full_cdn_baseline_1000ips" \
            "sudo $BINARY -sS -p 80,443 --target-file $BENCHMARK_DIR/targets/baseline-1000.txt" \
            2 5))

        # Default mode (skip all CDNs)
        results+=($(run_benchmark \
            "full_cdn_default_1000ips" \
            "sudo $BINARY -sS -p 80,443 --cdn-filter --target-file $BENCHMARK_DIR/targets/baseline-1000.txt" \
            2 5))

        # Whitelist mode (Cloudflare + AWS only)
        results+=($(run_benchmark \
            "full_cdn_whitelist_1000ips" \
            "sudo $BINARY -sS -p 80,443 --cdn-filter --cdn-whitelist cloudflare,aws --target-file $BENCHMARK_DIR/targets/baseline-1000.txt" \
            2 5))
    else
        log_warn "Skipping CDN benchmarks (targets/baseline-1000.txt not found)"
    fi

    # Batch I/O Performance Benchmarks (Linux only)
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        log_info "--- Batch I/O Performance Suite ---"

        # Baseline (batch=1)
        results+=($(run_benchmark \
            "full_batch_baseline_1000ports" \
            "sudo $BINARY -sS -p 1-1000 --batch-size 1 127.0.0.1" \
            2 5))

        # Batch 32
        results+=($(run_benchmark \
            "full_batch_32_1000ports" \
            "sudo $BINARY -sS -p 1-1000 --batch-size 32 127.0.0.1" \
            2 5))

        # Batch 256
        results+=($(run_benchmark \
            "full_batch_256_1000ports" \
            "sudo $BINARY -sS -p 1-1000 --batch-size 256 127.0.0.1" \
            2 5))

        # Batch 1024 (optimal)
        results+=($(run_benchmark \
            "full_batch_1024_1000ports" \
            "sudo $BINARY -sS -p 1-1000 --batch-size 1024 127.0.0.1" \
            2 5))

        # Adaptive batch sizing
        if [ -f "$BENCHMARK_DIR/targets/baseline-50.txt" ]; then
            results+=($(run_benchmark \
                "full_adaptive_batch_50ips" \
                "sudo $BINARY -sS -p 80,443 --adaptive-batch --target-file $BENCHMARK_DIR/targets/baseline-50.txt" \
                2 5))
        fi
    else
        log_warn "Skipping batch I/O benchmarks (Linux-only)"
    fi

    # IPv6 Performance (if targets exist)
    if [ -f "$BENCHMARK_DIR/targets/ipv6-25.txt" ]; then
        log_info "--- IPv6 Performance Suite ---"

        results+=($(run_benchmark \
            "full_ipv6_baseline_25ips" \
            "sudo $BINARY -sS -p 80,443 --target-file $BENCHMARK_DIR/targets/ipv6-25.txt" \
            2 5))

        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            results+=($(run_benchmark \
                "full_ipv6_batch_256_25ips" \
                "sudo $BINARY -sS -p 80,443 --batch-size 256 --target-file $BENCHMARK_DIR/targets/ipv6-25.txt" \
                2 5))
        fi
    else
        log_warn "Skipping IPv6 benchmarks (targets/ipv6-25.txt not found)"
    fi

    log_success "Full benchmarks complete: ${#results[@]} scenarios"
    generate_summary "${results[@]}"
}

# Benchmark Suite: CI Mode (optimized for GitHub Actions)
run_ci_benchmarks() {
    log_info "=== CI/CD Benchmark Mode ==="
    log_info "Estimated time: 2-5 minutes"
    log_info "Purpose: Automated regression detection"
    echo ""

    local results=()

    # Core performance tests only (localhost for speed)

    # 1. Baseline scan
    results+=($(run_benchmark \
        "ci_baseline_500ports" \
        "sudo $BINARY -sS -p 1-500 127.0.0.1" \
        1 3))

    # 2. Batch I/O (Linux only)
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        results+=($(run_benchmark \
            "ci_batch_1024_500ports" \
            "sudo $BINARY -sS -p 1-500 --batch-size 1024 127.0.0.1" \
            1 3))
    fi

    # 3. CDN filtering (minimal targets)
    if [ -f "$BENCHMARK_DIR/targets/baseline-50.txt" ]; then
        results+=($(run_benchmark \
            "ci_cdn_filter_50ips" \
            "sudo $BINARY -sS -p 80,443 --cdn-filter --target-file $BENCHMARK_DIR/targets/baseline-50.txt" \
            1 3))
    fi

    log_success "CI benchmarks complete: ${#results[@]} scenarios"
    generate_summary "${results[@]}"
}

generate_summary() {
    local result_files=("$@")

    if [ ${#result_files[@]} -eq 0 ]; then
        log_warn "No benchmark results to summarize"
        return
    fi

    log_info "=== Benchmark Summary ==="
    echo ""

    echo "| Benchmark | Mean Time | Std Dev | Runs |"
    echo "|-----------|-----------|---------|------|"

    for result_file in "${result_files[@]}"; do
        if [ ! -f "$result_file" ]; then
            continue
        fi

        # Extract benchmark name from filename
        local benchmark_name=$(basename "$result_file" | sed "s/_${TIMESTAMP}.json//")

        # Parse JSON with jq (if available)
        if command -v jq &> /dev/null; then
            local mean=$(jq -r '.results[0].mean' "$result_file" 2>/dev/null || echo "N/A")
            local stddev=$(jq -r '.results[0].stddev' "$result_file" 2>/dev/null || echo "N/A")
            local runs=$(jq -r '.results[0].times | length' "$result_file" 2>/dev/null || echo "N/A")

            printf "| %-30s | %10.3fs | %8.3fs | %4s |\n" \
                "$benchmark_name" \
                "${mean:-N/A}" \
                "${stddev:-N/A}" \
                "${runs:-N/A}"
        else
            echo "| $benchmark_name | (jq not available) | - | - |"
        fi
    done

    echo ""
    log_info "Results saved to: $RESULTS_DIR"
    log_info "Timestamp: $TIMESTAMP"
}

# Main execution
main() {
    echo "==================================="
    echo "Sprint 6.3 Benchmark Runner"
    echo "==================================="
    echo ""

    check_prerequisites
    echo ""

    case "$MODE" in
        --quick)
            run_quick_benchmarks
            ;;
        --full)
            run_full_benchmarks
            ;;
        --ci)
            run_ci_benchmarks
            ;;
        *)
            log_error "Invalid mode: $MODE"
            log_info "Usage: $0 [--quick|--full|--ci]"
            log_info "  --quick: Fast localhost tests (<2 minutes)"
            log_info "  --full:  Complete benchmark suite (8-10 minutes)"
            log_info "  --ci:    CI/CD optimized (2-5 minutes)"
            exit 1
            ;;
    esac

    echo ""
    log_success "=== Benchmark Suite Complete ==="
    log_info "Mode: $MODE"
    log_info "Results: $RESULTS_DIR"
    log_info "For baseline management: ./scripts/manage-baselines.sh"
}

main "$@"
