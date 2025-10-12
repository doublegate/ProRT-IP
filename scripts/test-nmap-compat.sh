#!/bin/bash
#
# Integration test script for nmap-compatible CLI flags
# Tests ProRT-IP v0.3.5 nmap compatibility features
#

set -e  # Exit on error

TARGET="127.0.0.1"  # Local target, no sudo needed
PRTIP="./target/release/prtip"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=========================================="
echo "ProRT-IP v0.3.5 - Nmap Compatibility Tests"
echo "=========================================="
echo ""

# Check if prtip binary exists
if [ ! -f "$PRTIP" ]; then
    echo -e "${RED}ERROR: prtip binary not found at $PRTIP${NC}"
    echo "Run: cargo build --release"
    exit 1
fi

# Test counter
PASSED=0
FAILED=0

# Helper function to run test
run_test() {
    local test_name="$1"
    local command="$2"

    echo -e "${YELLOW}Test:${NC} $test_name"
    echo "Command: $command"

    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ FAILED${NC}"
        ((FAILED++))
    fi
    echo ""
}

echo "=== Scan Type Aliases ==="
echo ""

run_test "SYN scan (-sS)" "$PRTIP -sS -p 80 $TARGET --timeout 1"
run_test "Connect scan (-sT)" "$PRTIP -sT -p 80 $TARGET --timeout 1"
run_test "NULL scan (-sN)" "$PRTIP -sN -p 80 $TARGET --timeout 1"
run_test "FIN scan (-sF)" "$PRTIP -sF -p 80 $TARGET --timeout 1"
run_test "Xmas scan (-sX)" "$PRTIP -sX -p 80 $TARGET --timeout 1"

echo "=== Backward Compatibility (Original Flags) ==="
echo ""

run_test "Original SYN syntax" "$PRTIP -s syn -p 80 $TARGET --timeout 1"
run_test "Original Connect syntax" "$PRTIP --scan-type connect -p 80 $TARGET --timeout 1"

echo "=== Port Specification ==="
echo ""

run_test "Fast scan (-F)" "$PRTIP -F $TARGET --timeout 2"
run_test "Top ports (--top-ports 50)" "$PRTIP --top-ports 50 $TARGET --timeout 2"
run_test "Port range (-p 1-100)" "$PRTIP -p 1-100 $TARGET --timeout 1"
run_test "Specific ports (-p 22,80,443)" "$PRTIP -p 22,80,443 $TARGET --timeout 1"

echo "=== Output Formats ==="
echo ""

run_test "Normal output (-oN)" "$PRTIP -p 80 $TARGET -oN /tmp/prtip-test-normal.txt --timeout 1"
run_test "XML output (-oX)" "$PRTIP -p 80 $TARGET -oX /tmp/prtip-test-xml.xml --timeout 1"
run_test "Greppable output (-oG)" "$PRTIP -p 80 $TARGET -oG /tmp/prtip-test-grep.gnmap --timeout 1"

# Verify output files created
if [ -f "/tmp/prtip-test-normal.txt" ]; then
    echo -e "${GREEN}✓ Normal output file created${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ Normal output file NOT created${NC}"
    ((FAILED++))
fi

if [ -f "/tmp/prtip-test-xml.xml" ]; then
    echo -e "${GREEN}✓ XML output file created${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ XML output file NOT created${NC}"
    ((FAILED++))
fi

if [ -f "/tmp/prtip-test-grep.gnmap" ]; then
    echo -e "${GREEN}✓ Greppable output file created${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ Greppable output file NOT created${NC}"
    ((FAILED++))
fi

echo ""
echo "=== Modes ==="
echo ""

run_test "Aggressive mode (-A)" "$PRTIP -A -p 80 $TARGET --timeout 2"
run_test "Skip ping (-Pn)" "$PRTIP -Pn -p 80 $TARGET --timeout 1"

echo "=== Verbosity ==="
echo ""

run_test "Verbosity level 1 (-v)" "$PRTIP -v -p 80 $TARGET --timeout 1"
run_test "Verbosity level 2 (-vv)" "$PRTIP -vv -p 80 $TARGET --timeout 1"
run_test "Verbosity level 3 (-vvv)" "$PRTIP -vvv -p 80 $TARGET --timeout 1"

echo "=== Mixed Syntax (Nmap + ProRT-IP) ==="
echo ""

run_test "Mixed: -sS + --ports" "$PRTIP -sS --ports 80 $TARGET --timeout 1"
run_test "Mixed: -F + --output" "$PRTIP -F --output text $TARGET --timeout 2"

echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo "Total: $((PASSED + FAILED))"
echo ""

# Cleanup
rm -f /tmp/prtip-test-*.txt /tmp/prtip-test-*.xml /tmp/prtip-test-*.gnmap

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed! ✓${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. ✗${NC}"
    exit 1
fi
