#!/bin/bash
# I/O Profiling Script for ProRT-IP Phase 5
# Uses strace to analyze system calls and I/O patterns
#
# Prerequisites:
# - strace
#
# Usage: ./profile-io.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="${BINARY:-./target/release/prtip}"

# Check for strace
if ! command -v strace &>/dev/null; then
    echo "Error: strace not found. Install with: sudo pacman -S strace"
    exit 1
fi

echo "=== ProRT-IP Phase 5 I/O Profiling ==="
echo

# Profile 1: SYN Scan Baseline
echo "[1/5] I/O profiling - SYN scan (1000 ports)..."
strace -c -o "$SCRIPT_DIR/syn-scan-1000ports.strace-summary.txt" \
    "$BINARY" -sS -p 1-1000 127.0.0.1 &>/dev/null

strace -tt -T -o "$SCRIPT_DIR/syn-scan-1000ports.strace-detailed.txt" \
    "$BINARY" -sS -p 1-1000 127.0.0.1 &>/dev/null

echo "   Summary: syn-scan-1000ports.strace-summary.txt"
echo "   Detailed: syn-scan-1000ports.strace-detailed.txt"

# Profile 2: Service Detection
echo "[2/5] I/O profiling - service detection..."
strace -c -o "$SCRIPT_DIR/service-detect.strace-summary.txt" \
    "$BINARY" -sS -sV -p 80,443 google.com &>/dev/null

strace -tt -T -o "$SCRIPT_DIR/service-detect.strace-detailed.txt" \
    "$BINARY" -sS -sV -p 80,443 google.com &>/dev/null

echo "   Summary: service-detect.strace-summary.txt"
echo "   Detailed: service-detect.strace-detailed.txt"

# Profile 3: IPv6 Scan
echo "[3/5] I/O profiling - IPv6 scan..."
strace -c -o "$SCRIPT_DIR/ipv6-scan.strace-summary.txt" \
    "$BINARY" -6 -sS -p 1-1000 ::1 &>/dev/null

strace -tt -T -o "$SCRIPT_DIR/ipv6-scan.strace-detailed.txt" \
    "$BINARY" -6 -sS -p 1-1000 ::1 &>/dev/null

echo "   Summary: ipv6-scan.strace-summary.txt"
echo "   Detailed: ipv6-scan.strace-detailed.txt"

# Profile 4: Large Scale (Network-intensive)
echo "[4/5] I/O profiling - large-scale scan (10000 ports)..."
strace -c -o "$SCRIPT_DIR/large-10000ports.strace-summary.txt" \
    "$BINARY" -sS -p 1-10000 127.0.0.1 &>/dev/null

strace -tt -T -o "$SCRIPT_DIR/large-10000ports.strace-detailed.txt" \
    "$BINARY" -sS -p 1-10000 127.0.0.1 &>/dev/null

echo "   Summary: large-10000ports.strace-summary.txt"
echo "   Detailed: large-10000ports.strace-detailed.txt"

# Profile 5: Rate Limiting (syscall patterns)
echo "[5/5] I/O profiling - rate-limited scan..."
strace -c -o "$SCRIPT_DIR/rate-limit-50k.strace-summary.txt" \
    "$BINARY" -sS -p 1-1000 --max-rate 50000 127.0.0.1 &>/dev/null

strace -tt -T -o "$SCRIPT_DIR/rate-limit-50k.strace-detailed.txt" \
    "$BINARY" -sS -p 1-1000 --max-rate 50000 127.0.0.1 &>/dev/null

echo "   Summary: rate-limit-50k.strace-summary.txt"
echo "   Detailed: rate-limit-50k.strace-detailed.txt"

echo
echo "=== I/O Profile Summary ==="
echo

# Analyze syscall patterns
for summary in "$SCRIPT_DIR"/*.strace-summary.txt; do
    name=$(basename "${summary%.strace-summary.txt}")
    echo "[$name]"

    # Top 5 syscalls by count
    echo "  Top 5 syscalls by count:"
    grep -E "^\s*[0-9]" "$summary" | sort -k1 -rn | head -5 | \
        awk '{printf "    %s: %s calls (%.1f%%)\n", $5, $1, $3}'

    # Top 5 syscalls by time
    echo "  Top 5 syscalls by time:"
    grep -E "^\s*[0-9]" "$summary" | sort -k2 -rn | head -5 | \
        awk '{printf "    %s: %.6f seconds (%.1f%%)\n", $5, $2, $4}'

    echo
done

# Network I/O analysis
echo "=== Network I/O Analysis ==="
echo

for detailed in "$SCRIPT_DIR"/*.strace-detailed.txt; do
    name=$(basename "${detailed%.strace-detailed.txt}")
    echo "[$name]"

    # Count socket operations
    socket_create=$(grep -c "socket(" "$detailed" || true)
    bind_calls=$(grep -c "bind(" "$detailed" || true)
    sendto_calls=$(grep -c "sendto(" "$detailed" || true)
    recvfrom_calls=$(grep -c "recvfrom(" "$detailed" || true)

    echo "  socket(): $socket_create"
    echo "  bind(): $bind_calls"
    echo "  sendto(): $sendto_calls"
    echo "  recvfrom(): $recvfrom_calls"
    echo
done

echo "=== I/O Profiling Complete ==="
echo "Summary reports: $SCRIPT_DIR/*.strace-summary.txt"
echo "Detailed traces: $SCRIPT_DIR/*.strace-detailed.txt"
echo
echo "Key metrics to analyze:"
echo "  - sendto/recvfrom ratios (network efficiency)"
echo "  - futex calls (lock contention)"
echo "  - mmap/munmap (memory allocation)"
echo "  - Total syscall count (lower is better)"
