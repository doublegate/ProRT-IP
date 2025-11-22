#!/bin/bash
# Automated release script using cargo-release
# Usage: ./scripts/release.sh [major|minor|patch]

set -e

LEVEL=${1:-patch}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Starting ProRT-IP release process (level: $LEVEL)${NC}"
echo ""

# Validate input
if [[ ! "$LEVEL" =~ ^(major|minor|patch)$ ]]; then
    echo -e "${RED}❌ Invalid release level: $LEVEL${NC}"
    echo "Usage: $0 [major|minor|patch]"
    exit 1
fi

# Check if cargo-release is installed
if ! command -v cargo-release &> /dev/null; then
    echo -e "${RED}❌ cargo-release is not installed${NC}"
    echo -e "${YELLOW}Install with: cargo install cargo-release${NC}"
    exit 1
fi

# Pre-flight checks
echo -e "${BLUE}📋 Running pre-flight checks...${NC}"

echo -n "  Checking code formatting... "
if cargo fmt --all -- --check &>/dev/null; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌ Format check failed${NC}"
    echo -e "${YELLOW}Run: cargo fmt${NC}"
    exit 1
fi

echo -n "  Running clippy linter... "
if cargo clippy --workspace --all-targets -- -D warnings &>/dev/null; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌ Clippy failed${NC}"
    echo -e "${YELLOW}Fix warnings or run: cargo clippy --fix${NC}"
    exit 1
fi

echo -n "  Running test suite... "
if cargo test --workspace --quiet &>/dev/null; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌ Tests failed${NC}"
    echo -e "${YELLOW}Fix failing tests before releasing${NC}"
    exit 1
fi

echo -n "  Validating Cargo.lock... "
if cargo build --locked --quiet &>/dev/null; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌ Lockfile out of sync${NC}"
    echo -e "${YELLOW}Run: cargo update${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All pre-flight checks passed${NC}"
echo ""

# Show what will be done
echo -e "${BLUE}📦 Preparing to release...${NC}"
CURRENT_VERSION=$(grep -A 1 '^\[workspace.package\]' Cargo.toml | grep '^version' | cut -d'"' -f2)
echo -e "  Current version: ${YELLOW}$CURRENT_VERSION${NC}"

# Calculate new version (approximation - cargo-release will do the actual calculation)
case "$LEVEL" in
    major)
        NEW_VERSION=$(echo $CURRENT_VERSION | awk -F. '{print $1+1".0.0"}')
        ;;
    minor)
        NEW_VERSION=$(echo $CURRENT_VERSION | awk -F. '{print $1"."$2+1".0"}')
        ;;
    patch)
        NEW_VERSION=$(echo $CURRENT_VERSION | awk -F. '{print $1"."$2"."$3+1}')
        ;;
esac
echo -e "  New version (estimated): ${GREEN}$NEW_VERSION${NC}"
echo ""

# Confirm with user
read -p "$(echo -e ${YELLOW}Continue with release? [y/N]: ${NC})" -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${RED}❌ Release cancelled${NC}"
    exit 1
fi
echo ""

# Run cargo-release
echo -e "${BLUE}🔧 Running cargo-release...${NC}"
if cargo release $LEVEL --execute; then
    echo ""
    echo -e "${GREEN}✅ Release prepared successfully${NC}"
    echo ""
    echo -e "${BLUE}📝 Next steps:${NC}"
    echo -e "  1. Review changes: ${YELLOW}git show HEAD${NC}"
    echo -e "  2. Review tag: ${YELLOW}git show v$NEW_VERSION${NC}"
    echo -e "  3. Create release notes: ${YELLOW}/tmp/ProRT-IP/RELEASE-NOTES-v$NEW_VERSION.md${NC}"
    echo -e "  4. Push to GitHub: ${YELLOW}git push origin main && git push origin v$NEW_VERSION${NC}"
    echo -e "  5. Create GitHub Release: ${YELLOW}gh release create v$NEW_VERSION --notes-file /tmp/ProRT-IP/RELEASE-NOTES-v$NEW_VERSION.md${NC}"
    echo ""
else
    echo ""
    echo -e "${RED}❌ cargo-release failed${NC}"
    echo -e "${YELLOW}Check the error messages above and try again${NC}"
    exit 1
fi
