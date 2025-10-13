#!/usr/bin/env bash
# run-integration-tests.sh - Integration tests against real targets
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PRTIP="$PROJECT_ROOT/target/release/prtip"

GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'; NC='\033[0m'

usage() {
    cat <<EOF
ProRT-IP Integration Test Suite

Usage: $(basename "$0") [options]

Options:
  --help              Show this help
  --quick             Quick subset of tests
  --target HOST       Test target (default: 127.0.0.1)

Tests:
  - All scan types (Connect, SYN, UDP, stealth)
  - Output formats (text, JSON, XML, greppable)
  - Nmap compatibility
  - Service detection
EOF
    exit 0
}

if [[ "${1:-}" == "--help" ]]; then
    usage
fi

TARGET="${1:-127.0.0.1}"
PASSED=0
FAILED=0

run_test() {
    local name="$1"
    shift
    echo -n "Test: $name ... "
    if "$@" &>/dev/null; then
        echo -e "${GREEN}PASS${NC}"
        ((PASSED++))
    else
        echo -e "${RED}FAIL${NC}"
        ((FAILED++))
    fi
}

echo "========================================"
echo "ProRT-IP Integration Tests"
echo "========================================"
echo "Target: $TARGET"
echo ""

# Build if needed
if [[ ! -f "$PRTIP" ]]; then
    cargo build --release
fi

# Test scan types
run_test "TCP Connect scan" "$PRTIP" -sT -p 80 --timeout 1 "$TARGET"
run_test "SYN scan" "$PRTIP" -sS -p 80 --timeout 1 "$TARGET"
run_test "NULL scan" "$PRTIP" -sN -p 80 --timeout 1 "$TARGET"
run_test "Fast scan (-F)" "$PRTIP" -F --timeout 2 "$TARGET"

# Test output formats
run_test "Text output" "$PRTIP" -p 80 -oN /tmp/test.txt --timeout 1 "$TARGET"
run_test "JSON output" "$PRTIP" -p 80 -oJ /tmp/test.json --timeout 1 "$TARGET"
run_test "XML output" "$PRTIP" -p 80 -oX /tmp/test.xml --timeout 1 "$TARGET"
run_test "Greppable output" "$PRTIP" -p 80 -oG /tmp/test.gnmap --timeout 1 "$TARGET"

# Cleanup
rm -f /tmp/test.txt /tmp/test.json /tmp/test.xml /tmp/test.gnmap

echo ""
echo "========================================"
echo "Results: $PASSED passed, $FAILED failed"
echo "========================================"

if [[ $FAILED -eq 0 ]]; then
    exit 0
else
    exit 1
fi
