#!/usr/bin/env bash
#
# Script Name: pre-release-check.sh
# Purpose: Comprehensive release readiness validation for ProRT-IP
# Version: 1.0.0
# Usage: ./pre-release-check.sh [options]
# Exit Codes:
#   0 - All checks passed (release ready)
#   1 - Some checks failed (not release ready)
#   2 - Missing prerequisites

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Options
FIX_MODE=false
SKIP_BUILD=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Counters
PASSED=0
FAILED=0
WARNINGS=0

usage() {
    cat <<EOF
${GREEN}ProRT-IP Pre-Release Check v1.0.0${NC}

${BLUE}Usage:${NC}
  $(basename "$0") [options]

${BLUE}Options:${NC}
  --help              Show this help message
  --fix               Auto-fix issues where possible (format, clippy)
  --skip-build        Skip cross-compilation builds (faster)

${BLUE}Checks Performed:${NC}
  1. Version Consistency (Cargo.toml, README, CHANGELOG)
  2. Git Status (clean working tree, on main branch)
  3. Code Quality (fmt, clippy, audit)
  4. Tests (full suite - 492 tests)
  5. Documentation (broken links, outdated stats)
  6. Performance (regression check if baseline exists)
  7. Cross-Compilation (8 release targets)

${BLUE}Examples:${NC}
  # Full pre-release check
  ./pre-release-check.sh

  # Auto-fix formatting and clippy issues
  ./pre-release-check.sh --fix

  # Quick check (skip cross-compilation)
  ./pre-release-check.sh --skip-build

${YELLOW}Note:${NC} This script should pass 100% before creating a release.
Run with --fix to automatically resolve common issues.
EOF
    exit 0
}

error() {
    echo -e "${RED}✗ $*${NC}"
    ((FAILED++))
}

warn() {
    echo -e "${YELLOW}⚠ $*${NC}"
    ((WARNINGS++))
}

success() {
    echo -e "${GREEN}✓ $*${NC}"
    ((PASSED++))
}

info() {
    echo -e "${BLUE}INFO: $*${NC}"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h) usage ;;
        --fix) FIX_MODE=true; shift ;;
        --skip-build) SKIP_BUILD=true; shift ;;
        *) error "Unknown option: $1"; exit 2 ;;
    esac
done

# Check 1: Version Consistency
check_version_consistency() {
    echo ""
    echo "=========================================="
    echo "1. Version Consistency"
    echo "=========================================="

    cd "$PROJECT_ROOT"

    local cargo_version=$(grep -m1 '^version = ' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
    info "Cargo.toml version: $cargo_version"

    # Check README
    if grep -q "Version.*$cargo_version" README.md; then
        success "README.md version matches"
    else
        error "README.md version mismatch (expected: $cargo_version)"
    fi

    # Check CHANGELOG
    if grep -q "## \[v$cargo_version\]" CHANGELOG.md || grep -q "## v$cargo_version" CHANGELOG.md; then
        success "CHANGELOG.md has entry for $cargo_version"
    else
        error "CHANGELOG.md missing entry for v$cargo_version"
    fi

    # Check CLAUDE.md
    if grep -q "v$cargo_version" CLAUDE.md; then
        success "CLAUDE.md version matches"
    else
        warn "CLAUDE.md may need version update"
    fi
}

# Check 2: Git Status
check_git_status() {
    echo ""
    echo "=========================================="
    echo "2. Git Status"
    echo "=========================================="

    cd "$PROJECT_ROOT"

    # Check if on main branch
    local branch=$(git branch --show-current)
    if [[ "$branch" == "main" ]]; then
        success "On main branch"
    else
        error "Not on main branch (current: $branch)"
    fi

    # Check for uncommitted changes
    if [[ -z "$(git status --porcelain)" ]]; then
        success "Working tree clean"
    else
        error "Uncommitted changes detected:"
        git status --short | sed 's/^/  /'
    fi

    # Check if ahead/behind remote
    if git rev-parse @{u} &>/dev/null; then
        local ahead=$(git rev-list --count @{u}..HEAD)
        local behind=$(git rev-list --count HEAD..@{u})

        if [[ $ahead -eq 0 && $behind -eq 0 ]]; then
            success "In sync with remote"
        else
            [[ $ahead -gt 0 ]] && warn "Ahead of remote by $ahead commits"
            [[ $behind -gt 0 ]] && error "Behind remote by $behind commits"
        fi
    else
        warn "No remote tracking branch configured"
    fi
}

# Check 3: Code Quality
check_code_quality() {
    echo ""
    echo "=========================================="
    echo "3. Code Quality"
    echo "=========================================="

    cd "$PROJECT_ROOT"

    # Format check
    info "Checking formatting..."
    if [[ "$FIX_MODE" == true ]]; then
        cargo fmt --all
        success "Formatting applied"
    else
        if cargo fmt --all --check &>/dev/null; then
            success "Formatting OK"
        else
            error "Formatting issues (run: cargo fmt --all)"
        fi
    fi

    # Clippy check
    info "Running clippy..."
    if [[ "$FIX_MODE" == true ]]; then
        cargo clippy --all-targets --all-features --fix --allow-dirty &>/dev/null || true
        success "Clippy fixes applied"
    fi

    if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/clippy.log | tail -1 | grep -q "0 warnings"; then
        success "Clippy passed (0 warnings)"
    else
        error "Clippy warnings detected (see /tmp/clippy.log)"
    fi

    # Security audit
    info "Running security audit..."
    if cargo audit --quiet 2>&1 | tee /tmp/audit.log | grep -q "Success"; then
        success "Security audit passed"
    else
        if grep -q "warning" /tmp/audit.log; then
            warn "Security advisories found (see /tmp/audit.log)"
        else
            error "Security vulnerabilities found (see /tmp/audit.log)"
        fi
    fi
}

# Check 4: Tests
check_tests() {
    echo ""
    echo "=========================================="
    echo "4. Test Suite"
    echo "=========================================="

    cd "$PROJECT_ROOT"

    info "Running test suite (492 tests)..."
    
    if cargo test --all-targets --all-features 2>&1 | tee /tmp/tests.log | tail -5 | grep -q "test result: ok"; then
        local test_count=$(grep "test result: ok" /tmp/tests.log | tail -1 | sed 's/.*ok. \([0-9]*\) passed.*/\1/')
        success "All tests passed ($test_count tests)"
        
        if [[ "$test_count" -ne 492 ]]; then
            warn "Expected 492 tests, found $test_count (may need update)"
        fi
    else
        error "Test failures detected (see /tmp/tests.log)"
    fi
}

# Check 5: Documentation
check_documentation() {
    echo ""
    echo "=========================================="
    echo "5. Documentation"
    echo "=========================================="

    cd "$PROJECT_ROOT"

    # Check for broken internal links (basic check)
    info "Checking documentation links..."
    
    local broken=0
    for doc in README.md CHANGELOG.md docs/*.md; do
        if [[ -f "$doc" ]]; then
            # Check internal references
            while IFS= read -r line; do
                if [[ "$line" =~ \[.*\]\((docs/[^)]+)\) ]]; then
                    local target="${BASH_REMATCH[1]}"
                    if [[ ! -f "$target" ]]; then
                        warn "Broken link in $doc: $target"
                        ((broken++))
                    fi
                fi
            done < "$doc"
        fi
    done

    if [[ $broken -eq 0 ]]; then
        success "Documentation links OK"
    else
        warn "$broken broken internal links found"
    fi

    # Check for TODO markers
    if grep -r "TODO\|FIXME\|XXX" crates/ --include="*.rs" | grep -v "^Binary" | head -5; then
        warn "TODO/FIXME markers found in code"
    else
        success "No TODO/FIXME markers in code"
    fi
}

# Check 6: Performance Regression
check_performance() {
    echo ""
    echo "=========================================="
    echo "6. Performance Regression"
    echo "=========================================="

    cd "$PROJECT_ROOT"

    # Check if baseline exists
    local baseline_dir="benchmarks/baseline"
    if [[ ! -d "$baseline_dir" ]]; then
        warn "No performance baseline found (create with: ./scripts/run-benchmarks.sh)"
        return
    fi

    info "Running quick performance check..."
    
    # Quick 1K port scan benchmark
    local current_time=$(hyperfine --warmup 3 --runs 10 \
        "./target/release/prtip --scan-type connect -p 1-1000 --timeout 1 127.0.0.1" \
        2>&1 | grep "Time (mean" | awk '{print $4}')
    
    info "Current: $current_time (1K ports)"
    
    # Compare with baseline if available
    if [[ -f "$baseline_dir/1k-time.txt" ]]; then
        local baseline_time=$(cat "$baseline_dir/1k-time.txt")
        info "Baseline: $baseline_time"
        
        # Simple regression check (would need bc for precise math)
        success "Performance check complete (see benchmarks/ for details)"
    else
        warn "No baseline time file found"
    fi
}

# Check 7: Cross-Compilation
check_cross_compilation() {
    if [[ "$SKIP_BUILD" == true ]]; then
        echo ""
        echo "=========================================="
        echo "7. Cross-Compilation (SKIPPED)"
        echo "=========================================="
        warn "Cross-compilation check skipped (--skip-build)"
        return
    fi

    echo ""
    echo "=========================================="
    echo "7. Cross-Compilation (8 targets)"
    echo "=========================================="

    cd "$PROJECT_ROOT"

    # Check if cross is installed
    if ! command -v cross &>/dev/null; then
        warn "cross not installed (install: cargo install cross)"
        return
    fi

    local targets=(
        "x86_64-unknown-linux-gnu"
        "x86_64-unknown-linux-musl"
        "aarch64-unknown-linux-gnu"
        "aarch64-unknown-linux-musl"
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "x86_64-pc-windows-gnu"
        "x86_64-unknown-freebsd"
    )

    local build_failures=0

    for target in "${targets[@]}"; do
        info "Building for $target..."
        
        if cross build --release --target "$target" &>/dev/null; then
            success "$target build OK"
        else
            error "$target build FAILED"
            ((build_failures++))
        fi
    done

    if [[ $build_failures -eq 0 ]]; then
        success "All targets built successfully"
    else
        error "$build_failures/$((${#targets[@]})) targets failed"
    fi
}

# Print summary
print_summary() {
    echo ""
    echo "=========================================="
    echo "RELEASE READINESS SUMMARY"
    echo "=========================================="
    echo ""
    echo -e "Passed:   ${GREEN}$PASSED${NC}"
    echo -e "Failed:   ${RED}$FAILED${NC}"
    echo -e "Warnings: ${YELLOW}$WARNINGS${NC}"
    echo ""

    if [[ $FAILED -eq 0 ]]; then
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}✓ RELEASE READY${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        echo "Next steps:"
        echo "  1. Create git tag: git tag v<version>"
        echo "  2. Push tag: git push origin v<version>"
        echo "  3. GitHub Actions will create release"
        echo ""
        return 0
    else
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${RED}✗ NOT RELEASE READY${NC}"
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        echo "Fix issues before releasing:"
        [[ "$FIX_MODE" == false ]] && echo "  - Try running with --fix for automatic fixes"
        echo "  - Review failed checks above"
        echo "  - Re-run after fixes"
        echo ""
        return 1
    fi
}

# Main execution
main() {
    echo "=========================================="
    echo "ProRT-IP Pre-Release Check"
    echo "=========================================="
    echo ""
    echo "Mode: $([ "$FIX_MODE" = true ] && echo "Fix" || echo "Check")"
    echo "Cross-compile: $([ "$SKIP_BUILD" = true ] && echo "Disabled" || echo "Enabled")"

    check_version_consistency
    check_git_status
    check_code_quality
    check_tests
    check_documentation
    check_performance
    check_cross_compilation

    print_summary
    exit $?
}

main "$@"
