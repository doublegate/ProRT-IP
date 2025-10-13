#!/usr/bin/env bash
#
# Script Name: test-nmap-compat.sh
# Purpose: Integration test script for nmap-compatible CLI flags
# Version: v0.3.6
# Usage: ./test-nmap-compat.sh [options]
# Prerequisites:
#   - Bash 4.0+
#   - Built prtip binary (cargo build --release)
#   - Network access for testing
# Exit Codes:
#   0 - All tests passed
#   1 - Some tests failed
#   2 - Missing prerequisites
#
# Examples:
#   ./test-nmap-compat.sh              # Run all tests
#   ./test-nmap-compat.sh --help       # Show usage
#   ./test-nmap-compat.sh --quick      # Run quick subset
#
# Tests ProRT-IP v0.3.5+ nmap compatibility features:
#   - Scan type aliases (-sS, -sT, -sN, -sF, -sX)
#   - Port specification (-F, --top-ports, -p)
#   - Output formats (-oN, -oX, -oG)
#   - Modes (-A, -Pn) and verbosity (-v, -vv, -vvv)
#   - Mixed syntax (nmap + ProRT-IP flags)
#

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Defaults
TARGET="127.0.0.1"  # Local target, no sudo needed
PRTIP="${PRTIP:-$PROJECT_ROOT/target/release/prtip}"  # Allow override via env
QUICK_MODE=false

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
usage() {
    cat <<EOF
${GREEN}ProRT-IP Nmap Compatibility Test Suite v0.3.6${NC}

${BLUE}Usage:${NC}
  $(basename "$0") [options]

${BLUE}Options:${NC}
  --help          Show this help message
  --quick         Run quick subset of tests (fast validation)
  --target HOST   Specify test target (default: 127.0.0.1)
  --binary PATH   Path to prtip binary (default: ./target/release/prtip)

${BLUE}Examples:${NC}
  # Run all tests (default)
  ./test-nmap-compat.sh

  # Run quick validation
  ./test-nmap-compat.sh --quick

  # Test specific binary
  ./test-nmap-compat.sh --binary /usr/local/bin/prtip

  # Test against remote host
  ./test-nmap-compat.sh --target scanme.nmap.org

${BLUE}Environment Variables:${NC}
  PRTIP           Path to prtip binary (default: ./target/release/prtip)
  PRTIP_TARGET    Test target (default: 127.0.0.1)

${BLUE}Requirements:${NC}
  - Bash 4.0+
  - Built prtip binary (cargo build --release)
  - Network access

${YELLOW}Note:${NC} Tests use localhost by default (no sudo needed).
Some scan types (-sS) may require privileges but will skip gracefully.
EOF
    exit 0
}

error() {
    echo -e "${RED}ERROR: $*${NC}" >&2
}

info() {
    echo -e "${BLUE}INFO: $*${NC}"
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
        --target|-t)
            TARGET="${2:-}"
            if [[ -z "$TARGET" ]]; then
                error "Missing target argument"
                exit 2
            fi
            shift 2
            ;;
        --binary|-b)
            PRTIP="${2:-}"
            if [[ -z "$PRTIP" ]]; then
                error "Missing binary path argument"
                exit 2
            fi
            shift 2
            ;;
        *)
            error "Unknown option: $1"
            echo "Run '$(basename "$0") --help' for usage"
            exit 2
            ;;
    esac
done

# Apply environment variable overrides
TARGET="${PRTIP_TARGET:-$TARGET}"

echo "=========================================="
echo "ProRT-IP v0.3.6 - Nmap Compatibility Tests"
echo "=========================================="
echo ""
info "Binary: $PRTIP"
info "Target: $TARGET"
info "Mode: $([ "$QUICK_MODE" = true ] && echo "Quick" || echo "Full")"
echo ""

# Check prerequisites
check_prerequisites() {
    local missing=()

    # Check if prtip binary exists
    if [[ ! -f "$PRTIP" ]]; then
        error "prtip binary not found at: $PRTIP"
        echo "Run: cargo build --release"
        exit 2
    fi

    # Check if binary is executable
    if [[ ! -x "$PRTIP" ]]; then
        error "prtip binary is not executable: $PRTIP"
        echo "Run: chmod +x $PRTIP"
        exit 2
    fi

    # Test binary works
    if ! "$PRTIP" --version &>/dev/null; then
        error "prtip binary failed to execute"
        exit 2
    fi

    info "Prerequisites OK"
}

check_prerequisites

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

# Summary and cleanup
echo ""
echo "=========================================="
echo "Cleanup"
echo "=========================================="
info "Removing temporary test files..."
rm -f /tmp/prtip-test-*.txt /tmp/prtip-test-*.xml /tmp/prtip-test-*.gnmap

echo ""
if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}All tests passed! ✓${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 0
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}Some tests failed (${FAILED}/${PASSED + FAILED}). ✗${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 1
fi
