#!/bin/bash
# Memory Profiling Script for ProRT-IP Phase 5
# Uses valgrind massif for heap profiling
#
# Prerequisites:
# - valgrind (with massif tool)
# - ms_print (usually included with valgrind)
#
# Usage: ./profile-memory.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="${BINARY:-./target/release/prtip}"

# Check for valgrind
if ! command -v valgrind &>/dev/null; then
    echo "Error: valgrind not found. Install with: sudo pacman -S valgrind"
    exit 1
fi

echo "=== ProRT-IP Phase 5 Memory Profiling ==="
echo

# Profile 1: Small Scan (baseline)
echo "[1/5] Profiling memory - small scan (100 ports)..."
valgrind --tool=massif \
    --massif-out-file="$SCRIPT_DIR/small-100ports.massif.out" \
    --time-unit=ms \
    "$BINARY" -sS -p 1-100 127.0.0.1

ms_print "$SCRIPT_DIR/small-100ports.massif.out" > "$SCRIPT_DIR/small-100ports.massif.txt"
echo "   Report: small-100ports.massif.txt"

# Profile 2: Medium Scan
echo "[2/5] Profiling memory - medium scan (1000 ports)..."
valgrind --tool=massif \
    --massif-out-file="$SCRIPT_DIR/medium-1000ports.massif.out" \
    --time-unit=ms \
    "$BINARY" -sS -p 1-1000 127.0.0.1

ms_print "$SCRIPT_DIR/medium-1000ports.massif.out" > "$SCRIPT_DIR/medium-1000ports.massif.txt"
echo "   Report: medium-1000ports.massif.txt"

# Profile 3: Large Scan
echo "[3/5] Profiling memory - large scan (10000 ports)..."
valgrind --tool=massif \
    --massif-out-file="$SCRIPT_DIR/large-10000ports.massif.out" \
    --time-unit=ms \
    "$BINARY" -sS -p 1-10000 127.0.0.1

ms_print "$SCRIPT_DIR/large-10000ports.massif.out" > "$SCRIPT_DIR/large-10000ports.massif.txt"
echo "   Report: large-10000ports.massif.txt"

# Profile 4: Service Detection (high memory usage)
echo "[4/5] Profiling memory - service detection..."
valgrind --tool=massif \
    --massif-out-file="$SCRIPT_DIR/service-detect.massif.out" \
    --time-unit=ms \
    "$BINARY" -sS -sV -p 80,443,8080,8443 google.com

ms_print "$SCRIPT_DIR/service-detect.massif.out" > "$SCRIPT_DIR/service-detect.massif.txt"
echo "   Report: service-detect.massif.txt"

# Profile 5: IPv6 Scan
echo "[5/5] Profiling memory - IPv6 scan..."
valgrind --tool=massif \
    --massif-out-file="$SCRIPT_DIR/ipv6-scan.massif.out" \
    --time-unit=ms \
    "$BINARY" -6 -sS -p 1-1000 ::1

ms_print "$SCRIPT_DIR/ipv6-scan.massif.out" > "$SCRIPT_DIR/ipv6-scan.massif.txt"
echo "   Report: ipv6-scan.massif.txt"

echo
echo "=== Memory Profile Summary ==="
echo

# Extract peak memory usage from all profiles
for massif_file in "$SCRIPT_DIR"/*.massif.out; do
    name=$(basename "${massif_file%.massif.out}")
    peak=$(grep "peak" "$massif_file" | head -1 | awk '{print $2}')
    useful=$(grep "useful-heap" "$massif_file" | head -1 | awk '{print $4}')
    echo "  $name: Peak=$peak, Useful=$useful"
done

echo
echo "=== Memory Profiling Complete ==="
echo "Reports: $SCRIPT_DIR/*.massif.txt"
echo "Raw data: $SCRIPT_DIR/*.massif.out"
echo
echo "Open .massif.txt files to analyze memory usage over time"
echo "Look for:"
echo "  - Peak memory usage"
echo "  - Memory growth patterns"
echo "  - Allocation hotspots"
