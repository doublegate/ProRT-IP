#!/bin/bash
# Install Git hooks for ProRT-IP development
# Run this once after cloning the repository or when hooks are updated
#
# Usage: ./scripts/install-hooks.sh

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory (works from any location)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}🔧 Installing ProRT-IP Git hooks...${NC}"
echo ""

# Change to project root
cd "$PROJECT_ROOT"

# Verify we're in a git repository
if [ ! -d ".git" ]; then
    echo -e "${RED}❌ ERROR: Not in a Git repository${NC}"
    echo "   This script must be run from the ProRT-IP repository root"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Check if template hook exists
if [ ! -f ".github/hooks/pre-commit" ]; then
    echo -e "${RED}❌ ERROR: .github/hooks/pre-commit not found${NC}"
    echo "   The hook template is missing from the repository"
    exit 1
fi

# Install pre-commit hook
echo -e "${YELLOW}📋 Installing pre-commit hook...${NC}"

# Copy hook from template
cp .github/hooks/pre-commit .git/hooks/pre-commit

# Make executable
chmod +x .git/hooks/pre-commit

# Verify installation
if [ -x .git/hooks/pre-commit ]; then
    echo -e "${GREEN}✅ Pre-commit hook installed${NC}"
else
    echo -e "${RED}❌ ERROR: Hook is not executable${NC}"
    exit 1
fi

# Verify files are identical (checksum match)
TEMPLATE_CHECKSUM=$(md5sum .github/hooks/pre-commit | awk '{print $1}')
INSTALLED_CHECKSUM=$(md5sum .git/hooks/pre-commit | awk '{print $1}')

if [ "$TEMPLATE_CHECKSUM" = "$INSTALLED_CHECKSUM" ]; then
    echo -e "${GREEN}✅ Hook is identical to template (checksum: $TEMPLATE_CHECKSUM)${NC}"
else
    echo -e "${YELLOW}⚠️  WARNING: Checksums do not match${NC}"
    echo "   Template:  $TEMPLATE_CHECKSUM"
    echo "   Installed: $INSTALLED_CHECKSUM"
    exit 1
fi

echo ""
echo -e "${GREEN}✅ All hooks installed successfully!${NC}"
echo ""
echo -e "${BLUE}ℹ️  The pre-commit hook will now run automatically before each commit.${NC}"
echo ""
echo "The hook validates:"
echo "  ${GREEN}✅${NC} Cargo.lock synchronization (prevents CI failures)"
echo "  ${GREEN}✅${NC} Code formatting (cargo fmt --check)"
echo "  ${GREEN}✅${NC} Linter warnings (cargo clippy --workspace)"
echo ""
echo -e "${YELLOW}ℹ️  To bypass the hook (not recommended):${NC}"
echo "   git commit --no-verify"
echo ""
echo -e "${BLUE}ℹ️  To update hooks when .github/hooks/pre-commit changes:${NC}"
echo "   ./scripts/install-hooks.sh"
echo ""
