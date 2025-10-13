#!/usr/bin/env bash
# cross-compile.sh - Build all release targets
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

GREEN='\033[0;32m'; RED='\033[0;31m'; BLUE='\033[0;34m'; NC='\033[0m'

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-gnu"
    "aarch64-unknown-linux-musl"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-gnu"
    "x86_64-unknown-freebsd"
)

cd "$PROJECT_ROOT"

echo "========================================"
echo "Cross-Compilation for ${#TARGETS[@]} Targets"
echo "========================================"
echo ""

# Check for cross
if ! command -v cross &>/dev/null; then
    echo -e "${RED}ERROR: cross not found${NC}"
    echo "Install: cargo install cross"
    exit 1
fi

SUCCESS=0
FAILED=0

for target in "${TARGETS[@]}"; do
    echo -e "${BLUE}Building: $target${NC}"
    
    if cross build --release --target "$target" 2>&1 | tail -5; then
        echo -e "${GREEN}✓ $target SUCCESS${NC}"
        ((SUCCESS++))
        
        # Show binary info
        binary="target/$target/release/prtip"
        [[ "$target" == *"windows"* ]] && binary="${binary}.exe"
        
        if [[ -f "$binary" ]]; then
            size=$(ls -lh "$binary" | awk '{print $5}')
            echo "  Binary size: $size"
        fi
    else
        echo -e "${RED}✗ $target FAILED${NC}"
        ((FAILED++))
    fi
    echo ""
done

echo "========================================"
echo "Build Summary: $SUCCESS/$((${#TARGETS[@]})) succeeded"
echo "========================================"

[[ $FAILED -eq 0 ]] && exit 0 || exit 1
