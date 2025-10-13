#!/usr/bin/env bash
# update-docs-stats.sh - Auto-update documentation statistics
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

GREEN='\033[0;32m'; RED='\033[0;31m'; NC='\033[0m'

cd "$PROJECT_ROOT"

# Count statistics
TESTS=$(grep -r "^fn test_\|^    fn test_" crates/ --include="*.rs" | wc -l)
LOC=$(find crates -name "*.rs" -exec cat {} \; | wc -l)
MODULES=$(find crates -name "*.rs" -not -name "lib.rs" -not -name "main.rs" | wc -l)
SCAN_TYPES=$(grep -o "ScanType::" crates/prtip-core/src/types.rs | sort -u | wc -l || echo 7)

echo -e "${GREEN}Documentation Statistics:${NC}"
echo "Tests: $TESTS"
echo "Lines of Code: $LOC"
echo "Modules: $MODULES"
echo "Scan Types: $SCAN_TYPES"

# Update README.md (placeholder - would need precise matching)
echo ""
echo "To update README.md:"
echo "  - Tests: $TESTS"
echo "  - LOC: $LOC"

echo -e "\n${GREEN}✓ Statistics collected${NC}"
