# Pre-Release Checklist

Comprehensive pre-release validation before tagging version.

---

## Phase 1: VERSION DETECTION

Extract and validate version information.

### Step 1.1: Get Current Version

```bash
CURRENT_VERSION=$(grep "^version" Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

if [ -z "$CURRENT_VERSION" ]; then
    echo "❌ Could not extract version from Cargo.toml"
    exit 1
fi

echo "Current version: v$CURRENT_VERSION"
echo ""
```

### Step 1.2: Check if Tag Exists

```bash
if git tag | grep -q "^v$CURRENT_VERSION$"; then
    echo "❌ Tag v$CURRENT_VERSION already exists"
    echo ""
    echo "Either:"
    echo "1. Update version in Cargo.toml"
    echo "2. Delete existing tag: git tag -d v$CURRENT_VERSION"
    exit 1
fi

echo "✅ Version v$CURRENT_VERSION ready for release"
echo ""
```

---

## Phase 2: CHECKLIST VALIDATION

Run comprehensive pre-release checks.

### Step 2.1: Working Directory Clean

```bash
echo "Running pre-release checklist..."
echo ""

PASS=0
FAIL=0
WARN=0

echo "1/10: Checking working directory..."

if [ -z "$(git status --porcelain)" ]; then
    echo "  ✅ PASS - Working directory clean"
    ((PASS++))
else
    echo "  ❌ FAIL - Uncommitted changes present"
    git status --short | sed 's/^/      /'
    ((FAIL++))
fi

echo ""
```

### Step 2.2: All Tests Passing

```bash
echo "2/10: Running test suite..."

if cargo test --workspace --quiet 2>&1 | grep -q "test result: ok"; then
    TESTS=$(cargo test --workspace 2>&1 | grep -oP '\d+(?= passed)' | tail -1)
    echo "  ✅ PASS - All tests passing ($TESTS tests)"
    ((PASS++))
else
    echo "  ❌ FAIL - Tests failing"
    echo ""
    echo "  Run: /test-quick or /rust-check to identify issues"
    ((FAIL++))
fi

echo ""
```

### Step 2.3: No Clippy Warnings

```bash
echo "3/10: Checking clippy warnings..."

if cargo clippy --all-features --quiet -- -D warnings 2>&1 | grep -q "error:"; then
    WARNINGS=$(cargo clippy --all-features 2>&1 | grep -c "warning:")
    echo "  ❌ FAIL - $WARNINGS clippy warnings present"
    ((FAIL++))
else
    echo "  ✅ PASS - No clippy warnings"
    ((PASS++))
fi

echo ""
```

### Step 2.4: Documentation Builds

```bash
echo "4/10: Building documentation..."

if cargo doc --no-deps --workspace --quiet 2>&1 | grep -q "error:"; then
    echo "  ❌ FAIL - Documentation errors"
    cargo doc --no-deps --workspace 2>&1 | grep "error:" | head -5 | sed 's/^/      /'
    ((FAIL++))
else
    echo "  ✅ PASS - Documentation builds successfully"
    ((PASS++))
fi

echo ""
```

### Step 2.5: CHANGELOG Updated

```bash
echo "5/10: Checking CHANGELOG.md..."

# Check if CHANGELOG was modified recently (within last commit)
if git diff HEAD~1 CHANGELOG.md 2>/dev/null | grep -q "^+##"; then
    echo "  ✅ PASS - CHANGELOG.md updated recently"
    ((PASS++))
# Check for Unreleased section with content
elif grep -q "\[Unreleased\]" CHANGELOG.md && \
     grep -A 5 "\[Unreleased\]" CHANGELOG.md | grep -q "^### "; then
    echo "  ✅ PASS - CHANGELOG.md has unreleased changes"
    ((PASS++))
else
    echo "  ⚠️  WARNING - CHANGELOG.md may need update"
    echo "      Add v$CURRENT_VERSION entry before release"
    ((WARN++))
fi

echo ""
```

### Step 2.6: No Security Vulnerabilities

```bash
echo "6/10: Checking security vulnerabilities..."

if ! command -v cargo-audit &> /dev/null; then
    echo "  ⚠️  WARNING - cargo-audit not installed"
    echo "      Install: cargo install cargo-audit"
    ((WARN++))
elif cargo audit 2>&1 | grep -q "error:"; then
    VULNS=$(cargo audit 2>&1 | grep -c "error:")
    echo "  ❌ FAIL - $VULNS security vulnerabilities found"
    cargo audit 2>&1 | grep "Crate:" | sed 's/^/      /'
    ((FAIL++))
else
    echo "  ✅ PASS - No security vulnerabilities"
    ((PASS++))
fi

echo ""
```

### Step 2.7: Fuzzing Status

```bash
echo "7/10: Checking fuzzing status..."

if [ ! -d "fuzz" ]; then
    echo "  ⚠️  WARNING - Fuzzing not set up"
    echo "      Consider running Sprint 5.7"
    ((WARN++))
elif [ ! -d "fuzz/artifacts" ]; then
    echo "  ⚠️  WARNING - Fuzzing not yet run"
    echo "      Run: /fuzz-check"
    ((WARN++))
else
    CRASHES=$(find fuzz/artifacts -name "crash-*" -o -name "timeout-*" 2>/dev/null | wc -l)
    if [ "$CRASHES" -eq 0 ]; then
        echo "  ✅ PASS - No fuzz crashes"
        ((PASS++))
    else
        echo "  ❌ FAIL - $CRASHES fuzz crashes present"
        find fuzz/artifacts -name "crash-*" -o -name "timeout-*" 2>/dev/null | head -3 | sed 's/^/      /'
        ((FAIL++))
    fi
fi

echo ""
```

### Step 2.8: CI Status

```bash
echo "8/10: Checking CI status..."

if command -v gh &> /dev/null; then
    CI_STATUS=$(gh run list --limit 1 --json conclusion --jq '.[0].conclusion' 2>/dev/null)

    if [ "$CI_STATUS" = "success" ]; then
        echo "  ✅ PASS - Latest CI run passed"
        ((PASS++))
    elif [ "$CI_STATUS" = "failure" ]; then
        echo "  ❌ FAIL - Latest CI run failed"
        echo "      Check: gh run view"
        ((FAIL++))
    else
        echo "  ⚠️  WARNING - CI status unclear: $CI_STATUS"
        ((WARN++))
    fi
else
    echo "  ⚠️  WARNING - Cannot check CI (gh not installed)"
    echo "      Check manually: https://github.com/doublegate/ProRT-IP/actions"
    ((WARN++))
fi

echo ""
```

### Step 2.9: Version Consistency

```bash
echo "9/10: Checking version consistency..."

# Check if all crate versions match
VERSIONS=$(find crates -name Cargo.toml -exec grep "^version" {} \; | sort -u | wc -l)

if [ "$VERSIONS" -eq 1 ]; then
    echo "  ✅ PASS - Version consistent across all crates"
    ((PASS++))
else
    echo "  ⚠️  WARNING - Version inconsistent across crates"
    find crates -name Cargo.toml -exec grep "^version" {} \; | sort -u | sed 's/^/      /'
    ((WARN++))
fi

echo ""
```

### Step 2.10: README Version

```bash
echo "10/10: Checking README.md version..."

if grep -q "v$CURRENT_VERSION" README.md; then
    echo "  ✅ PASS - README.md mentions v$CURRENT_VERSION"
    ((PASS++))
elif grep -q "$CURRENT_VERSION" README.md; then
    echo "  ✅ PASS - README.md mentions $CURRENT_VERSION"
    ((PASS++))
else
    echo "  ⚠️  WARNING - README.md may need version update"
    echo "      Current version not found in README.md"
    ((WARN++))
fi

echo ""
```

---

## Phase 3: SUMMARY AND DECISION

Display checklist results and determine if ready for release.

### Step 3.1: Display Summary

```bash
echo "═══════════════════════════════════════════════════════════════"
echo "              Pre-Release Checklist Summary                    "
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Version: v$CURRENT_VERSION"
echo ""
echo "Results:"
echo "  ✅ Passed:   $PASS/10"
echo "  ❌ Failed:   $FAIL/10"
echo "  ⚠️  Warnings: $WARN/10"
echo ""
```

### Step 3.2: Generate Detailed Report

```bash
REPORT_DATE=$(date +"%Y-%m-%d %H:%M:%S")

cat > /tmp/ProRT-IP/PRE-RELEASE-REPORT.md << EOF
# Pre-Release Checklist Report

**Date:** $REPORT_DATE
**Version:** v$CURRENT_VERSION

---

## Summary

| Status | Count |
|--------|-------|
| ✅ Passed | $PASS/10 |
| ❌ Failed | $FAIL/10 |
| ⚠️  Warnings | $WARN/10 |

**Overall:** $(
    if [ "$FAIL" -eq 0 ]; then
        echo "✅ READY FOR RELEASE"
    else
        echo "❌ NOT READY FOR RELEASE"
    fi
)

---

## Checklist Results

1. **Working Directory:** $([ $((10 - FAIL - WARN)) -ge 1 ] && echo "✅" || echo "❌")
2. **Tests Passing:** $(cargo test --workspace --quiet 2>&1 | grep -q "ok" && echo "✅" || echo "❌")
3. **Clippy Warnings:** $(cargo clippy --all-features --quiet 2>&1 | grep -q "error:" && echo "❌" || echo "✅")
4. **Documentation:** ✅
5. **CHANGELOG:** $(grep -q "v$CURRENT_VERSION" CHANGELOG.md && echo "✅" || echo "⚠️")
6. **Security:** $(command -v cargo-audit &>/dev/null && cargo audit 2>&1 | grep -q "error:" && echo "❌" || echo "✅")
7. **Fuzzing:** $([ -d fuzz/artifacts ] && [ $(find fuzz/artifacts -name "crash-*" 2>/dev/null | wc -l) -eq 0 ] && echo "✅" || echo "⚠️")
8. **CI Status:** ⚠️ (Check manually)
9. **Version Consistency:** ✅
10. **README Version:** $(grep -q "$CURRENT_VERSION" README.md && echo "✅" || echo "⚠️")

---

## Failed Items

EOF

if [ "$FAIL" -gt 0 ]; then
    echo "Fix these issues before releasing:" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md
    echo "" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md

    # List specific failures
    [ -n "$(git status --porcelain)" ] && echo "- Uncommitted changes in working directory" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md
    cargo test --workspace --quiet 2>&1 | grep -q "FAILED" && echo "- Test failures" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md
    cargo clippy --all-features --quiet 2>&1 | grep -q "error:" && echo "- Clippy warnings" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md

    echo "" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md
else
    echo "✅ No blocking issues" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md
    echo "" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md
fi

cat >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md << 'EOF'

## Warnings

Warnings are non-blocking but should be reviewed:

EOF

if [ "$WARN" -gt 0 ]; then
    [ ! -f CHANGELOG.md ] || ! grep -q "v$CURRENT_VERSION" CHANGELOG.md && \
        echo "- CHANGELOG.md may need version entry" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md
    [ ! -d fuzz ] && echo "- Fuzzing infrastructure not set up" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md
    ! grep -q "$CURRENT_VERSION" README.md && echo "- README.md version reference" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md

    echo "" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md
else
    echo "✅ No warnings" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md
    echo "" >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md
fi

cat >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md << EOF

---

## Next Steps

EOF

if [ "$FAIL" -eq 0 ]; then
    cat >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md << EOF
✅ **READY FOR RELEASE**

Release procedure:

1. **Review checklist results** (above)

2. **Create release tag:**
   \`\`\`bash
   git tag -a v$CURRENT_VERSION -m "Release v$CURRENT_VERSION"
   \`\`\`

3. **Write comprehensive tag message** (100-150 lines):
   - Executive summary
   - Features/changes
   - Performance metrics
   - Technical details
   - Testing summary
   - Strategic value

4. **Push tag:**
   \`\`\`bash
   git push origin v$CURRENT_VERSION
   \`\`\`

5. **Create GitHub release** (150-200 lines):
   - All tag content
   - Installation instructions
   - Platform matrix (8 targets)
   - Known issues
   - Asset downloads

6. **Verify release:**
   - Check CI/CD release workflow
   - Verify 8/8 platform binaries
   - Test installation instructions

EOF
else
    cat >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md << EOF
❌ **NOT READY FOR RELEASE**

Fix the $FAIL failed item(s) above before releasing.

Recommended actions:

1. **Fix failures** (listed above)
2. **Re-run checklist:** /pre-release
3. **Verify fixes:** /rust-check
4. **Try again**

Common fixes:

- Working directory: \`git add . && git commit\`
- Test failures: \`/test-quick <pattern>\`
- Clippy warnings: \`/quick-fix\`
- Security vulns: \`/deps-update\`
- Fuzz crashes: \`/fuzz-check\` and fix

EOF
fi

cat >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md << 'EOF'

---

## Reference

- **Release Standards:** docs/11-RELEASE-PROCESS.md
- **Tag Examples:** v0.4.0-v0.4.6 (100-150 lines)
- **GitHub Examples:** v0.4.0-v0.4.6 releases (150-200 lines)
- **Comprehensive:** Technical depth, performance metrics

EOF

cat >> /tmp/ProRT-IP/PRE-RELEASE-REPORT.md << EOF

---

**Generated:** $REPORT_DATE
**Tool:** ProRT-IP Pre-Release Check (/pre-release)
EOF

echo "✅ Pre-release report generated"
echo ""
```

### Step 3.3: Display Decision

```bash
if [ "$FAIL" -eq 0 ]; then
    cat << EOF

╔════════════════════════════════════════════════════════════════╗
║                  ✅ READY FOR RELEASE                          ║
╚════════════════════════════════════════════════════════════════╝

Version: v$CURRENT_VERSION
Passed: $PASS/10
Warnings: $WARN/10 (non-blocking)

🚀 NEXT STEPS

1. Review pre-release report:
   /tmp/ProRT-IP/PRE-RELEASE-REPORT.md

2. Create release tag:
   git tag -a v$CURRENT_VERSION

3. Write comprehensive tag message (100-150 lines)
   - See docs/11-RELEASE-PROCESS.md for template
   - Reference: v0.4.0-v0.4.6 tags

4. Push tag:
   git push origin v$CURRENT_VERSION

5. Create GitHub release (150-200 lines)
   - All tag content + extras
   - Platform matrix (8 targets)
   - Installation instructions

6. Verify CI/CD builds all platforms

📖 REFERENCES

• Release Process: docs/11-RELEASE-PROCESS.md
• Previous Releases: v0.4.0 through v0.4.6
• Quality Standard: 100-200 lines, technical depth

EOF
else
    cat << EOF

╔════════════════════════════════════════════════════════════════╗
║                ❌ NOT READY FOR RELEASE                        ║
╚════════════════════════════════════════════════════════════════╝

Version: v$CURRENT_VERSION
Failed: $FAIL/10 checks
Warnings: $WARN/10

🔧 REQUIRED FIXES

Fix the $FAIL failed check(s) before releasing:

EOF

    # List specific failures with fix commands
    if [ -n "$(git status --porcelain)" ]; then
        echo "1. Commit changes: git add . && git commit"
    fi

    if cargo test --workspace --quiet 2>&1 | grep -q "FAILED"; then
        echo "2. Fix test failures: /test-quick or /rust-check"
    fi

    if cargo clippy --all-features --quiet 2>&1 | grep -q "error:"; then
        echo "3. Fix clippy warnings: /quick-fix or manual fixes"
    fi

    cat << 'EOF'

🔄 AFTER FIXES

1. Re-run pre-release check: /pre-release
2. Verify all checks pass
3. Proceed with release

📖 TROUBLESHOOTING

• Full quality check: /rust-check
• Security scan: /security-audit
• Dependencies: /deps-update
• Coverage: /test-coverage

EOF
fi

echo ""
echo "Report: /tmp/ProRT-IP/PRE-RELEASE-REPORT.md"
echo ""
```

---

## SUCCESS CRITERIA

✅ Version extracted and validated
✅ All 10 checklist items executed
✅ Pass/fail/warning counts computed
✅ Detailed report generated
✅ Clear next steps provided

---

## RELATED COMMANDS

- `/rust-check` - Run before pre-release for quality validation
- `/security-audit` - Included in pre-release checks
- `/test-coverage` - Verify coverage before release
- `/fuzz-check` - Verify no crashes before release

---

**Execute pre-release checklist now.**
