#!/bin/bash
#
# Sprint 6.3 Baseline Management Script
# Manages performance baselines and detects regressions
#
# Usage:
#   ./manage-baselines.sh save <commit-sha>      - Save current results as baseline
#   ./manage-baselines.sh compare <baseline>     - Compare current vs baseline
#   ./manage-baselines.sh list                   - List available baselines
#   ./manage-baselines.sh clean [--keep N]       - Clean old baselines
#
# Regression Detection:
#   Exit Code 0: No regression detected
#   Exit Code 1: Warning - potential regression (5-10% degradation)
#   Exit Code 2: Critical - significant regression (>10% degradation)
#

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_DIR="$(dirname "$SCRIPT_DIR")"
BASELINES_DIR="$BENCHMARK_DIR/baselines"
RESULTS_DIR="$BENCHMARK_DIR/results"

# Thresholds
WARNING_THRESHOLD=0.05    # 5% degradation
CRITICAL_THRESHOLD=0.10   # 10% degradation

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Save current results as baseline
cmd_save() {
    local commit_sha=${1:-$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")}
    local baseline_name="baseline-${commit_sha}-$(date +%Y%m%d-%H%M%S)"
    local baseline_dir="$BASELINES_DIR/$baseline_name"

    log_info "Saving baseline: $baseline_name"

    # Create baseline directory
    mkdir -p "$baseline_dir"

    # Find latest result files
    local result_files=$(find "$RESULTS_DIR" -maxdepth 1 -name "*.json" -type f 2>/dev/null | sort -r | head -20)

    if [ -z "$result_files" ]; then
        log_error "No result files found in $RESULTS_DIR"
        exit 1
    fi

    # Copy results to baseline
    local count=0
    for result_file in $result_files; do
        cp "$result_file" "$baseline_dir/"
        ((count++))
    done

    # Create metadata file
    cat > "$baseline_dir/metadata.json" <<EOF
{
  "baseline_name": "$baseline_name",
  "commit_sha": "$commit_sha",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "git_branch": "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")",
  "file_count": $count,
  "platform": "$OSTYPE"
}
EOF

    log_success "Saved $count result files to baseline"
    log_info "Baseline directory: $baseline_dir"

    # List baselines
    log_info ""
    cmd_list
}

# Compare current results against baseline
cmd_compare() {
    local baseline_name=${1:-}

    if [ -z "$baseline_name" ]; then
        log_error "Usage: $0 compare <baseline-name>"
        log_info "Available baselines:"
        cmd_list
        exit 1
    fi

    local baseline_dir="$BASELINES_DIR/$baseline_name"

    if [ ! -d "$baseline_dir" ]; then
        log_error "Baseline not found: $baseline_name"
        log_info "Available baselines:"
        cmd_list
        exit 1
    fi

    log_info "Comparing against baseline: $baseline_name"
    echo ""

    # Check if jq is available
    if ! command -v jq &> /dev/null; then
        log_error "jq is required for comparison"
        log_info "Install: sudo apt-get install jq"
        exit 1
    fi

    # Read baseline metadata
    if [ -f "$baseline_dir/metadata.json" ]; then
        log_info "Baseline metadata:"
        jq -r '. | "  Commit: \(.commit_sha)\n  Branch: \(.git_branch)\n  Created: \(.created_at)\n  Platform: \(.platform)"' "$baseline_dir/metadata.json"
        echo ""
    fi

    # Compare each benchmark
    local exit_code=0
    local regression_count=0
    local improvement_count=0
    local total_benchmarks=0

    echo "| Benchmark | Baseline | Current | Change | Status |"
    echo "|-----------|----------|---------|--------|--------|"

    # Find matching benchmark pairs
    for baseline_file in "$baseline_dir"/*.json; do
        local baseline_filename=$(basename "$baseline_file")

        # Skip metadata
        if [ "$baseline_filename" == "metadata.json" ]; then
            continue
        fi

        # Extract benchmark name (without timestamp)
        local benchmark_name=$(echo "$baseline_filename" | sed -E 's/_[0-9]{8}-[0-9]{6}\.json$//')

        # Find most recent matching result
        local current_file=$(find "$RESULTS_DIR" -name "${benchmark_name}_*.json" -type f 2>/dev/null | sort -r | head -1)

        if [ -z "$current_file" ] || [ ! -f "$current_file" ]; then
            log_warn "No current result for: $benchmark_name"
            continue
        fi

        # Extract mean times
        local baseline_mean=$(jq -r '.results[0].mean // 0' "$baseline_file" 2>/dev/null)
        local current_mean=$(jq -r '.results[0].mean // 0' "$current_file" 2>/dev/null)

        if [ "$baseline_mean" == "0" ] || [ "$current_mean" == "0" ]; then
            log_warn "Invalid data for: $benchmark_name"
            continue
        fi

        # Calculate percentage change
        local change=$(awk "BEGIN { printf \"%.2f\", (($current_mean - $baseline_mean) / $baseline_mean) * 100 }")
        local abs_change=$(echo "$change" | tr -d '-')

        # Determine status
        local status="✓ OK"
        local status_color="$GREEN"

        if (( $(awk "BEGIN { print ($change > 0) }") )); then
            # Performance degraded (higher time = worse)
            if (( $(awk "BEGIN { print ($change / 100 > $CRITICAL_THRESHOLD) }") )); then
                status="✗ CRITICAL"
                status_color="$RED"
                regression_count=$((regression_count + 1))
                exit_code=2
            elif (( $(awk "BEGIN { print ($change / 100 > $WARNING_THRESHOLD) }") )); then
                status="⚠ WARNING"
                status_color="$YELLOW"
                regression_count=$((regression_count + 1))
                [ $exit_code -eq 0 ] && exit_code=1
            fi
        else
            # Performance improved
            if (( $(awk "BEGIN { print ($abs_change / 100 > 0.05) }") )); then
                status="↑ IMPROVED"
                status_color="$GREEN"
                improvement_count=$((improvement_count + 1))
            fi
        fi

        printf "| %-30s | %8.3fs | %8.3fs | %+6.1f%% | %s%-12s%s |\n" \
            "$benchmark_name" \
            "$baseline_mean" \
            "$current_mean" \
            "$change" \
            "$status_color" \
            "$status" \
            "$NC"

        total_benchmarks=$((total_benchmarks + 1))
    done

    echo ""
    log_info "=== Comparison Summary ==="
    log_info "Total benchmarks: $total_benchmarks"
    log_info "Regressions: $regression_count"
    log_info "Improvements: $improvement_count"

    # Final status
    echo ""
    if [ $exit_code -eq 0 ]; then
        log_success "No regressions detected"
    elif [ $exit_code -eq 1 ]; then
        log_warn "Potential regressions detected (5-10% degradation)"
        log_info "Review recommended, but within acceptable variance"
    elif [ $exit_code -eq 2 ]; then
        log_error "Significant regressions detected (>10% degradation)"
        log_info "Investigation required before merge"
    fi

    return $exit_code
}

# List available baselines
cmd_list() {
    log_info "Available baselines:"

    if [ ! -d "$BASELINES_DIR" ] || [ -z "$(ls -A "$BASELINES_DIR" 2>/dev/null)" ]; then
        log_warn "No baselines found in $BASELINES_DIR"
        log_info "Create a baseline with: $0 save [commit-sha]"
        return
    fi

    echo ""
    echo "| Baseline Name | Commit | Created | Files |"
    echo "|---------------|--------|---------|-------|"

    for baseline_dir in "$BASELINES_DIR"/baseline-*; do
        if [ ! -d "$baseline_dir" ]; then
            continue
        fi

        local baseline_name=$(basename "$baseline_dir")
        local metadata_file="$baseline_dir/metadata.json"

        if [ -f "$metadata_file" ] && command -v jq &> /dev/null; then
            local commit=$(jq -r '.commit_sha // "unknown"' "$metadata_file")
            local created=$(jq -r '.created_at // "unknown"' "$metadata_file")
            local file_count=$(jq -r '.file_count // 0' "$metadata_file")
        else
            local commit="unknown"
            local created="unknown"
            local file_count=$(find "$baseline_dir" -name "*.json" ! -name "metadata.json" | wc -l)
        fi

        printf "| %-40s | %-6s | %-19s | %5d |\n" \
            "$baseline_name" \
            "$commit" \
            "${created:0:19}" \
            "$file_count"
    done

    echo ""
}

# Clean old baselines
cmd_clean() {
    local keep_count=5

    # Parse --keep argument
    while [[ $# -gt 0 ]]; do
        case $1 in
            --keep)
                keep_count=$2
                shift 2
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    log_info "Cleaning old baselines (keeping latest $keep_count)..."

    if [ ! -d "$BASELINES_DIR" ]; then
        log_info "No baselines directory found"
        return
    fi

    # Find baselines sorted by creation time
    local baselines=($(find "$BASELINES_DIR" -maxdepth 1 -type d -name "baseline-*" | sort -r))
    local total_count=${#baselines[@]}

    if [ $total_count -le $keep_count ]; then
        log_info "Only $total_count baseline(s) exist - nothing to clean"
        return
    fi

    local delete_count=$((total_count - keep_count))
    log_info "Deleting $delete_count old baseline(s)..."

    # Delete old baselines
    for ((i=$keep_count; i<$total_count; i++)); do
        local baseline_dir="${baselines[$i]}"
        local baseline_name=$(basename "$baseline_dir")

        rm -rf "$baseline_dir"
        log_success "Deleted: $baseline_name"
    done

    log_success "Cleanup complete"
}

# Main execution
main() {
    local command=${1:-help}

    # Create directories
    mkdir -p "$BASELINES_DIR"
    mkdir -p "$RESULTS_DIR"

    case "$command" in
        save)
            shift
            cmd_save "$@"
            ;;
        compare)
            shift
            cmd_compare "$@"
            ;;
        list)
            cmd_list
            ;;
        clean)
            shift
            cmd_clean "$@"
            ;;
        help|--help|-h)
            echo "Usage: $0 <command> [options]"
            echo ""
            echo "Commands:"
            echo "  save <commit-sha>      Save current results as baseline"
            echo "  compare <baseline>     Compare current results against baseline"
            echo "  list                   List available baselines"
            echo "  clean [--keep N]       Clean old baselines (default: keep 5)"
            echo ""
            echo "Regression Detection:"
            echo "  Exit Code 0: No regression detected"
            echo "  Exit Code 1: Warning - potential regression (5-10% degradation)"
            echo "  Exit Code 2: Critical - significant regression (>10% degradation)"
            echo ""
            echo "Examples:"
            echo "  $0 save v0.5.2"
            echo "  $0 compare baseline-v0.5.2-20241116-120000"
            echo "  $0 clean --keep 10"
            ;;
        *)
            log_error "Unknown command: $command"
            log_info "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
}

main "$@"
