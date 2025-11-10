#!/usr/bin/env bash
# Sprint 5.4 Phase 2: Benchmark Results Analysis
# Analyzes hyperfine JSON output and validates <5% overhead claim
#
# Usage: ./analyze_results.sh [results_directory]

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RESULTS_DIR="${1:-./results}"
OVERHEAD_THRESHOLD=5.0  # 5% threshold

if [[ ! -d "$RESULTS_DIR" ]]; then
    echo -e "${RED}Error: Results directory not found: $RESULTS_DIR${NC}"
    exit 1
fi

echo -e "${GREEN}=== Sprint 5.4 Phase 2: Benchmark Analysis ===${NC}"
echo "Results directory: $RESULTS_DIR"
echo "Overhead threshold: ${OVERHEAD_THRESHOLD}%"
echo ""

# Find all JSON result files
JSON_FILES=($(find "$RESULTS_DIR" -name "*.json" -type f | sort))

if [[ ${#JSON_FILES[@]} -eq 0 ]]; then
    echo -e "${RED}Error: No JSON result files found in $RESULTS_DIR${NC}"
    exit 1
fi

echo "Found ${#JSON_FILES[@]} benchmark result files"
echo ""

# Function to calculate overhead percentage
calculate_overhead() {
    local baseline="$1"
    local measured="$2"
    
    # Use bc for floating point arithmetic
    echo "scale=2; (($measured - $baseline) / $baseline) * 100" | bc
}

# Function to format time in ms
format_ms() {
    local seconds="$1"
    echo "scale=2; $seconds * 1000" | bc
}

# Analysis function
analyze_benchmark() {
    local json_file="$1"
    local test_name="$2"
    
    echo -e "${BLUE}=== $test_name ===${NC}"
    echo "File: $(basename "$json_file")"
    echo ""
    
    # Extract baseline time (first command is always baseline)
    local baseline_cmd=$(jq -r '.results[0].command' "$json_file")
    local baseline_time=$(jq -r '.results[0].mean' "$json_file")
    local baseline_stddev=$(jq -r '.results[0].stddev' "$json_file")
    
    echo "Baseline: $baseline_cmd"
    echo "  Mean: $(format_ms "$baseline_time") ms ± $(format_ms "$baseline_stddev") ms"
    echo ""
    
    # Analyze each variant
    local num_results=$(jq '.results | length' "$json_file")
    local max_overhead=0
    local max_overhead_variant=""
    
    for i in $(seq 1 $((num_results - 1))); do
        local variant_cmd=$(jq -r ".results[$i].command" "$json_file")
        local variant_time=$(jq -r ".results[$i].mean" "$json_file")
        local variant_stddev=$(jq -r ".results[$i].stddev" "$json_file")
        
        # Calculate overhead
        local overhead=$(calculate_overhead "$baseline_time" "$variant_time")
        
        # Track maximum overhead
        if (( $(echo "$overhead > $max_overhead" | bc -l) )); then
            max_overhead=$overhead
            max_overhead_variant="$variant_cmd"
        fi
        
        # Color code based on threshold
        local color="$GREEN"
        if (( $(echo "$overhead > $OVERHEAD_THRESHOLD" | bc -l) )); then
            color="$RED"
        elif (( $(echo "$overhead > 3.0" | bc -l) )); then
            color="$YELLOW"
        fi
        
        echo "Variant: $variant_cmd"
        echo "  Mean: $(format_ms "$variant_time") ms ± $(format_ms "$variant_stddev") ms"
        echo -e "  ${color}Overhead: ${overhead}%${NC}"
        echo ""
    done
    
    # Summary
    echo "Maximum overhead: $max_overhead%"
    if (( $(echo "$max_overhead > $OVERHEAD_THRESHOLD" | bc -l) )); then
        echo -e "${RED}❌ FAILED: Exceeds ${OVERHEAD_THRESHOLD}% threshold${NC}"
        echo "Worst variant: $max_overhead_variant"
    else
        echo -e "${GREEN}✅ PASSED: Within ${OVERHEAD_THRESHOLD}% threshold${NC}"
    fi
    echo ""
    echo "---"
    echo ""
}

# Analyze each benchmark
for json_file in "${JSON_FILES[@]}"; do
    case "$(basename "$json_file")" in
        01_common_ports_*)
            analyze_benchmark "$json_file" "Test 1: Common Ports (Top 100)"
            ;;
        02_large_range_*)
            analyze_benchmark "$json_file" "Test 2: Large Port Range (1-1000)"
            ;;
        03_hostgroup_single_*)
            analyze_benchmark "$json_file" "Test 3: Hostgroup Size Impact (Single Target)"
            ;;
        04_hostgroup_multi_*)
            analyze_benchmark "$json_file" "Test 4: Multiple Targets with Hostgroup"
            ;;
        05_rate_impact_*)
            analyze_benchmark "$json_file" "Test 5: Adaptive Rate Limiter Impact"
            ;;
        *)
            echo -e "${YELLOW}Warning: Unknown benchmark file: $(basename "$json_file")${NC}"
            ;;
    esac
done

# Overall summary
echo -e "${GREEN}=== Overall Summary ===${NC}"
echo "Total benchmarks analyzed: ${#JSON_FILES[@]}"
echo ""

# Count pass/fail
total_tests=0
passed_tests=0
failed_tests=0

for json_file in "${JSON_FILES[@]}"; do
    num_results=$(jq '.results | length' "$json_file")
    baseline_time=$(jq -r '.results[0].mean' "$json_file")
    
    for i in $(seq 1 $((num_results - 1))); do
        total_tests=$((total_tests + 1))
        variant_time=$(jq -r ".results[$i].mean" "$json_file")
        overhead=$(calculate_overhead "$baseline_time" "$variant_time")
        
        if (( $(echo "$overhead <= $OVERHEAD_THRESHOLD" | bc -l) )); then
            passed_tests=$((passed_tests + 1))
        else
            failed_tests=$((failed_tests + 1))
        fi
    done
done

echo "Total variants tested: $total_tests"
echo -e "${GREEN}Passed (<${OVERHEAD_THRESHOLD}%): $passed_tests${NC}"
echo -e "${RED}Failed (>${OVERHEAD_THRESHOLD}%): $failed_tests${NC}"
echo ""

if [[ $failed_tests -eq 0 ]]; then
    echo -e "${GREEN}✅ ALL TESTS PASSED: <5% overhead claim validated${NC}"
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED: Review results above${NC}"
    exit 1
fi
