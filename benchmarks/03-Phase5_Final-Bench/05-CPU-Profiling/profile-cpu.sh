#!/bin/bash
# CPU Profiling Script for ProRT-IP Phase 5
# Generates flamegraphs using cargo-flamegraph (Rust tool)
#
# Prerequisites:
# - sudo access (perf requires root or CAP_PERFMON capability)
# - cargo-flamegraph: cargo install flamegraph
#
# Usage: sudo ./profile-cpu.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../" && pwd)"

# Check for cargo-flamegraph
if ! command -v flamegraph &> /dev/null; then
    echo "Error: cargo-flamegraph not found"
    echo "Install it with: cargo install flamegraph"
    exit 1
fi

# Create flamegraphs directory
mkdir -p "$SCRIPT_DIR/flamegraphs"

echo "=== ProRT-IP Phase 5 CPU Profiling ==="
echo

# Profile 1: SYN Scan (baseline)
echo "[1/5] Profiling SYN scan (1000 ports)..."
cd "$PROJECT_ROOT"
cargo flamegraph --root --freq 99 -o "$SCRIPT_DIR/flamegraphs/syn-scan-1000ports.svg" -- \
    -sS -p 1-1000 127.0.0.1

echo "   Flamegraph: flamegraphs/syn-scan-1000ports.svg"

# Profile 2: Service Detection
echo "[2/5] Profiling service detection..."
cd "$PROJECT_ROOT"
cargo flamegraph --root --freq 99 -o "$SCRIPT_DIR/flamegraphs/service-detect.svg" -- \
    -sS -sV -p 80,443 google.com

echo "   Flamegraph: flamegraphs/service-detect.svg"

# Profile 3: IPv6 Scan
echo "[3/5] Profiling IPv6 scan..."
cd "$PROJECT_ROOT"
cargo flamegraph --root --freq 99 -o "$SCRIPT_DIR/flamegraphs/ipv6-scan.svg" -- \
    -6 -sS -p 1-1000 ::1

echo "   Flamegraph: flamegraphs/ipv6-scan.svg"

# Profile 4: Large Scale (65K ports)
echo "[4/5] Profiling large-scale scan (65535 ports)..."
cd "$PROJECT_ROOT"
cargo flamegraph --root --freq 99 -o "$SCRIPT_DIR/flamegraphs/full-65535ports.svg" -- \
    -sS -p 1-65535 127.0.0.1

echo "   Flamegraph: flamegraphs/full-65535ports.svg"

# Profile 5: Rate Limiting
echo "[5/5] Profiling rate-limited scan..."
cd "$PROJECT_ROOT"
cargo flamegraph --root --freq 99 -o "$SCRIPT_DIR/flamegraphs/rate-limit-50k.svg" -- \
    -sS -p 1-10000 --max-rate 50000 127.0.0.1

echo "   Flamegraph: flamegraphs/rate-limit-50k.svg"

# Fix permissions (flamegraph already handles perf.data files)
chown -R "$USER:$USER" "$SCRIPT_DIR/flamegraphs" 2>/dev/null || true

echo
echo "=== CPU Profiling Complete ==="
echo "Flamegraphs: $SCRIPT_DIR/flamegraphs/"
echo
echo "Open flamegraphs in browser to analyze CPU hotspots:"
echo "  firefox $SCRIPT_DIR/flamegraphs/*.svg"
