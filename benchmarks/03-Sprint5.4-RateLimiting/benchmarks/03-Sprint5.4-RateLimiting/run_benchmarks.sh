#!/usr/bin/env bash
# Sprint 5.4 Phase 2: Rate Limiting Overhead Benchmarking
# Validates <5% overhead claim for three-layer rate limiting system
#
# Usage: ./run_benchmarks.sh [--quick]
#
# Requirements:
# - hyperfine 1.19.0+
# - Release build of prtip
# - Loopback target (127.0.0.1)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PRTIP="../../target/release/prtip"
OUTPUT_DIR="./results"
WARMUP_RUNS=3
BENCHMARK_RUNS=10
QUICK_MODE=false

# Parse arguments
if [[ "${1:-}" == "--quick" ]]; then
    QUICK_MODE=true
    BENCHMARK_RUNS=5
    echo -e "${YELLOW}Quick mode enabled: $BENCHMARK_RUNS runs instead of 10${NC}"
fi

# Check dependencies
echo -e "${BLUE}Checking dependencies...${NC}"
if ! command -v hyperfine &> /dev/null; then
    echo -e "${RED}Error: hyperfine not found. Install with: cargo install hyperfine${NC}"
    exit 1
fi

if [[ ! -f "$PRTIP" ]]; then
    echo -e "${YELLOW}Building release binary...${NC}"
    cargo build --release
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo -e "${GREEN}=== Sprint 5.4 Phase 2: Rate Limiting Benchmarking ===${NC}"
echo "Date: $(date)"
echo "prtip version: $($PRTIP --version)"
echo "hyperfine version: $(hyperfine --version)"
echo "Output directory: $OUTPUT_DIR"
echo "Runs per benchmark: $BENCHMARK_RUNS"
echo ""

# Test 1: Common Ports Scan (Top 100)
echo -e "${BLUE}Test 1: Common Ports (Top 100) - Baseline vs Layers${NC}"
hyperfine \
    --warmup $WARMUP_RUNS \
    --runs $BENCHMARK_RUNS \
    --export-json "$OUTPUT_DIR/01_common_ports_${TIMESTAMP}.json" \
    --export-markdown "$OUTPUT_DIR/01_common_ports_${TIMESTAMP}.md" \
    --command-name "Baseline" "$PRTIP -sS -p 21-23,25,53,80,110-111,135,139,143,443,445,993,995,1723,3306,3389,5900,8080 127.0.0.1 -Pn --no-service-detection" \
    --command-name "Layer1_ICMP" "$PRTIP -sS -p 21-23,25,53,80,110-111,135,139,143,443,445,993,995,1723,3306,3389,5900,8080 127.0.0.1 -Pn --no-service-detection --adaptive-rate" \
    --command-name "Layer2_Hostgroup" "$PRTIP -sS -p 21-23,25,53,80,110-111,135,139,143,443,445,993,995,1723,3306,3389,5900,8080 127.0.0.1 -Pn --no-service-detection --max-hostgroup 64" \
    --command-name "Layer3_Adaptive" "$PRTIP -sS -p 21-23,25,53,80,110-111,135,139,143,443,445,993,995,1723,3306,3389,5900,8080 127.0.0.1 -Pn --no-service-detection --max-rate 100000" \
    --command-name "Combined_All3" "$PRTIP -sS -p 21-23,25,53,80,110-111,135,139,143,443,445,993,995,1723,3306,3389,5900,8080 127.0.0.1 -Pn --no-service-detection --adaptive-rate --max-hostgroup 64 --max-rate 100000"

echo ""

# Test 2: Large Port Range (1-1000)
echo -e "${BLUE}Test 2: Large Port Range (1-1000) - Baseline vs Layers${NC}"
hyperfine \
    --warmup $WARMUP_RUNS \
    --runs $BENCHMARK_RUNS \
    --export-json "$OUTPUT_DIR/02_large_range_${TIMESTAMP}.json" \
    --export-markdown "$OUTPUT_DIR/02_large_range_${TIMESTAMP}.md" \
    --command-name "Baseline" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection" \
    --command-name "Layer1_ICMP" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --adaptive-rate" \
    --command-name "Layer2_Hostgroup" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-hostgroup 64" \
    --command-name "Layer3_Adaptive" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-rate 100000" \
    --command-name "Combined_All3" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --adaptive-rate --max-hostgroup 64 --max-rate 100000"

echo ""

# Test 3: Hostgroup Size Impact (Single Target, 1-1000 ports)
echo -e "${BLUE}Test 3: Hostgroup Size Impact (Single Target)${NC}"
hyperfine \
    --warmup $WARMUP_RUNS \
    --runs $BENCHMARK_RUNS \
    --export-json "$OUTPUT_DIR/03_hostgroup_single_${TIMESTAMP}.json" \
    --export-markdown "$OUTPUT_DIR/03_hostgroup_single_${TIMESTAMP}.md" \
    --command-name "Baseline" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection" \
    --command-name "Hostgroup_1" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-hostgroup 1" \
    --command-name "Hostgroup_8" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-hostgroup 8" \
    --command-name "Hostgroup_32" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-hostgroup 32" \
    --command-name "Hostgroup_64" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-hostgroup 64" \
    --command-name "Hostgroup_128" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-hostgroup 128"

echo ""

# Test 4: Multiple Targets with Hostgroup (8 targets, Fast scan)
if [[ "$QUICK_MODE" == "false" ]]; then
    echo -e "${BLUE}Test 4: Multiple Targets with Hostgroup (8 targets)${NC}"
    hyperfine \
        --warmup $WARMUP_RUNS \
        --runs $BENCHMARK_RUNS \
        --export-json "$OUTPUT_DIR/04_hostgroup_multi_${TIMESTAMP}.json" \
        --export-markdown "$OUTPUT_DIR/04_hostgroup_multi_${TIMESTAMP}.md" \
        --command-name "Baseline" "$PRTIP -sS -F 127.0.0.1-127.0.0.8 -Pn --no-service-detection" \
        --command-name "Hostgroup_1" "$PRTIP -sS -F 127.0.0.1-127.0.0.8 -Pn --no-service-detection --max-hostgroup 1" \
        --command-name "Hostgroup_2" "$PRTIP -sS -F 127.0.0.1-127.0.0.8 -Pn --no-service-detection --max-hostgroup 2" \
        --command-name "Hostgroup_4" "$PRTIP -sS -F 127.0.0.1-127.0.0.8 -Pn --no-service-detection --max-hostgroup 4" \
        --command-name "Hostgroup_8" "$PRTIP -sS -F 127.0.0.1-127.0.0.8 -Pn --no-service-detection --max-hostgroup 8" \
        --command-name "Hostgroup_64" "$PRTIP -sS -F 127.0.0.1-127.0.0.8 -Pn --no-service-detection --max-hostgroup 64"

    echo ""
fi

# Test 5: Rate Limiter Impact (different max-rate values)
echo -e "${BLUE}Test 5: Adaptive Rate Limiter Impact${NC}"
hyperfine \
    --warmup $WARMUP_RUNS \
    --runs $BENCHMARK_RUNS \
    --export-json "$OUTPUT_DIR/05_rate_impact_${TIMESTAMP}.json" \
    --export-markdown "$OUTPUT_DIR/05_rate_impact_${TIMESTAMP}.md" \
    --command-name "Baseline" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection" \
    --command-name "Rate_10K" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-rate 10000" \
    --command-name "Rate_50K" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-rate 50000" \
    --command-name "Rate_100K" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-rate 100000" \
    --command-name "Rate_500K" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-rate 500000" \
    --command-name "Rate_1M" "$PRTIP -sS -p 1-1000 127.0.0.1 -Pn --no-service-detection --max-rate 1000000"

echo ""
echo -e "${GREEN}=== Benchmarking Complete ===${NC}"
echo "Results saved to: $OUTPUT_DIR"
echo ""
echo "To analyze results:"
echo "  ./analyze_results.sh $OUTPUT_DIR"
