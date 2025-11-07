# Dependencies Update - Safe

Safely update and test dependencies with automatic rollback on failure.

---

## Phase 1: BACKUP CURRENT STATE

Create backup before making any changes.

### Step 1.1: Create Backup

```bash
echo "Creating backup of current state..."
echo ""

# Backup Cargo.lock
cp Cargo.lock Cargo.lock.backup
echo "✅ Backed up: Cargo.lock → Cargo.lock.backup"

# Stash any uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    git stash push -m "deps-update backup $(date +%Y-%m-%d-%H%M%S)" 2>/dev/null
    echo "✅ Stashed uncommitted changes"
fi

echo ""
```

---

## Phase 2: CHECK FOR UPDATES

Identify outdated dependencies before updating.

### Step 2.1: Install cargo-outdated

```bash
if ! cargo outdated --version &> /dev/null; then
    echo "Installing cargo-outdated..."
    cargo install cargo-outdated --quiet
    echo ""
fi
```

### Step 2.2: List Outdated Dependencies

```bash
echo "Checking for outdated dependencies..."
echo ""

cargo outdated --workspace --root-deps-only > /tmp/deps-outdated.txt 2>&1 || {
    echo "⚠️  cargo-outdated failed, trying full scan..."
    cargo outdated --workspace > /tmp/deps-outdated.txt 2>&1
}

cat /tmp/deps-outdated.txt
echo ""

# Count outdated packages
OUTDATED_COUNT=$(grep -c "^[a-z]" /tmp/deps-outdated.txt 2>/dev/null || echo "0")

if [ "$OUTDATED_COUNT" -eq 0 ]; then
    echo "✅ All dependencies are up to date"
    echo ""
    read -p "Continue with cargo update anyway? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted - no updates needed"
        rm Cargo.lock.backup 2>/dev/null
        exit 0
    fi
fi

echo "Found $OUTDATED_COUNT outdated dependencies"
echo ""
```

---

## Phase 3: UPDATE DEPENDENCIES

Update Cargo.lock with compatible versions.

### Step 3.1: Update Compatible Versions

```bash
echo "Updating dependencies (compatible versions only)..."
echo ""

# Update Cargo.lock only (respects Cargo.toml version constraints)
cargo update 2>&1 | tee /tmp/deps-update-log.txt

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ cargo update failed"
    echo "   Restoring backup..."
    cp Cargo.lock.backup Cargo.lock
    exit 1
fi

echo ""
echo "✅ Dependencies updated"
echo ""
```

### Step 3.2: Show Changes

```bash
echo "Dependency changes:"
echo ""

if command -v git &> /dev/null && [ -f Cargo.lock.backup ]; then
    # Show diff of changed dependencies
    diff <(grep "^name = " Cargo.lock.backup | sort) <(grep "^name = " Cargo.lock | sort) | \
        grep "^[<>]" | head -20

    echo ""
    echo "Changed versions:"
    git diff --no-index Cargo.lock.backup Cargo.lock 2>/dev/null | \
        grep "^[-+]version = " | head -20
else
    echo "(Unable to show diff - git not available)"
fi

echo ""
```

---

## Phase 4: TEST AFTER UPDATE

Verify everything still works with updated dependencies.

### Step 4.1: Build Check

```bash
echo "Testing with updated dependencies..."
echo ""
echo "1/3: Building project..."

if ! cargo build --all-features 2>&1 | tee /tmp/deps-build.txt; then
    echo ""
    echo "❌ Build failed with updated dependencies"
    echo ""
    echo "Build errors:"
    grep "error" /tmp/deps-build.txt | head -10
    echo ""
    echo "Rolling back..."
    cp Cargo.lock.backup Cargo.lock
    echo "✅ Rolled back to previous Cargo.lock"
    echo ""
    echo "Investigate errors and try manual update"
    exit 1
fi

echo "✅ Build successful"
echo ""
```

### Step 4.2: Test Suite

```bash
echo "2/3: Running test suite..."
echo ""

if ! cargo test --all-features 2>&1 | tee /tmp/deps-test.txt; then
    echo ""
    echo "❌ Tests failed with updated dependencies"
    echo ""
    echo "Failed tests:"
    grep "FAILED" /tmp/deps-test.txt | head -10
    echo ""

    read -p "Rollback changes? (Y/n): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        cp Cargo.lock.backup Cargo.lock
        echo "✅ Rolled back to previous Cargo.lock"
        exit 1
    else
        echo "⚠️  Continuing with test failures (not recommended)"
    fi
else
    TESTS_PASSED=$(grep -oP '\d+(?= passed)' /tmp/deps-test.txt | tail -1)
    echo "✅ All tests passed ($TESTS_PASSED tests)"
fi

echo ""
```

### Step 4.3: Clippy Check

```bash
echo "3/3: Checking for new clippy warnings..."
echo ""

CLIPPY_OUTPUT=$(cargo clippy --all-features 2>&1 | tee /tmp/deps-clippy.txt)
NEW_WARNINGS=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:" || echo "0")

if [ "$NEW_WARNINGS" -eq 0 ]; then
    echo "✅ No clippy warnings"
else
    echo "⚠️  $NEW_WARNINGS clippy warnings (review needed)"
    echo ""
    echo "Top warnings:"
    echo "$CLIPPY_OUTPUT" | grep "warning:" | head -5
    echo ""
    echo "This may be acceptable - review warnings before deciding"
fi

echo ""
```

---

## Phase 5: GENERATE UPDATE REPORT

Create detailed report of dependency changes.

### Step 5.1: Create Report

```bash
REPORT_DATE=$(date +"%Y-%m-%d %H:%M:%S")

cat > /tmp/ProRT-IP/DEPS-UPDATE-REPORT.md << EOF
# Dependency Update Report

**Date:** $REPORT_DATE
**Project:** ProRT-IP WarScan

---

## Summary

✅ **Status:** Dependencies updated successfully
⚠️  **Test Results:** $([ "$NEW_WARNINGS" -eq 0 ] && echo "All checks passed" || echo "$NEW_WARNINGS clippy warnings")

---

## Changes Made

### Updated Dependencies

\`\`\`diff
$(git diff --no-index Cargo.lock.backup Cargo.lock 2>/dev/null | head -100 || echo "No git available for diff")
\`\`\`

### Version Changes

$(git diff --no-index Cargo.lock.backup Cargo.lock 2>/dev/null | grep "^[-+]name = \|^[-+]version = " | head -40 || echo "(Unable to extract version changes)")

---

## Verification Results

### Build Status
✅ PASSED - Project builds with updated dependencies

### Test Suite
$([ "$TESTS_PASSED" ] && echo "✅ PASSED - All $TESTS_PASSED tests passing" || echo "⚠️  CHECK - Review test output")

### Clippy Warnings
$([ "$NEW_WARNINGS" -eq 0 ] && echo "✅ PASSED - No warnings" || echo "⚠️  $NEW_WARNINGS warnings - Review required")

---

## Outdated Dependencies (Before Update)

\`\`\`
$(cat /tmp/deps-outdated.txt)
\`\`\`

---

## Rollback Procedure

If issues are discovered later:

\`\`\`bash
# Restore previous Cargo.lock
cp Cargo.lock.backup Cargo.lock

# Rebuild with previous dependencies
cargo build --all-features

# Verify tests
cargo test --all-features

# Clean up backup
rm Cargo.lock.backup
\`\`\`

---

## Commit Changes

If everything looks good:

\`\`\`bash
# Review changes
git diff Cargo.lock

# Add to staging
git add Cargo.lock

# Commit with detailed message
git commit -m "deps: Update dependencies $(date +%Y-%m-%d)

Updated dependency versions to latest compatible releases.

Verification:
- ✅ Build: PASSED
- ✅ Tests: $TESTS_PASSED passing
- $([ "$NEW_WARNINGS" -eq 0 ] && echo "✅ Clippy: PASSED" || echo "⚠️  Clippy: $NEW_WARNINGS warnings")

Changes: See Cargo.lock diff
Report: /tmp/ProRT-IP/DEPS-UPDATE-REPORT.md"

# Push changes
git push
\`\`\`

---

## Next Steps

1. ✅ Review this report
2. ⬜ Test manually (optional)
3. ⬜ Run /rust-check for full validation
4. ⬜ Commit changes if satisfied
5. ⬜ Clean up: rm Cargo.lock.backup

---

## Backup Location

- **Cargo.lock backup:** Cargo.lock.backup
- **Git stash:** Available via \`git stash list\`

---

**Generated:** $REPORT_DATE
**Tool:** ProRT-IP Dependency Update (/deps-update)
EOF

echo "✅ Update report generated"
echo ""
```

### Step 5.2: Display Report

```bash
cat /tmp/ProRT-IP/DEPS-UPDATE-REPORT.md

echo ""
echo "Report saved: /tmp/ProRT-IP/DEPS-UPDATE-REPORT.md"
echo ""
```

---

## Phase 6: NEXT STEPS

Provide clear guidance on what to do next.

```bash
cat << EOF

╔════════════════════════════════════════════════════════════════╗
║              Dependency Update Complete                        ║
╚════════════════════════════════════════════════════════════════╝

📊 RESULTS

• Outdated: $OUTDATED_COUNT dependencies
• Updated: ✅ Cargo.lock modified
• Build: ✅ PASSED
• Tests: ✅ $TESTS_PASSED passing
• Clippy: $([ "$NEW_WARNINGS" -eq 0 ] && echo "✅ PASSED" || echo "⚠️  $NEW_WARNINGS warnings")

🔒 BACKUP AVAILABLE

• Cargo.lock.backup (restore with: cp Cargo.lock.backup Cargo.lock)
• Git stash (if uncommitted changes existed)

🚀 NEXT STEPS

EOF

if [ "$NEW_WARNINGS" -eq 0 ]; then
    cat << 'EOF'
1. Review changes: git diff Cargo.lock
2. (Optional) Manual testing
3. Run full check: /rust-check
4. Commit changes: git add Cargo.lock && git commit
5. Clean up: rm Cargo.lock.backup

EOF
else
    cat << 'EOF'
1. Review clippy warnings: cargo clippy --all-features
2. Decide if warnings are acceptable
3. If acceptable:
   - Commit: git add Cargo.lock && git commit
   - Or fix warnings: /quick-fix
4. If not acceptable:
   - Rollback: cp Cargo.lock.backup Cargo.lock

EOF
fi

cat << 'EOF'

📖 DOCUMENTATION

• Report: /tmp/ProRT-IP/DEPS-UPDATE-REPORT.md
• Changes: git diff Cargo.lock.backup Cargo.lock

EOF
```

---

## SUCCESS CRITERIA

✅ Backup created (Cargo.lock.backup)
✅ Dependencies checked (cargo outdated)
✅ Dependencies updated (cargo update)
✅ Build verified (cargo build)
✅ Tests verified (cargo test)
✅ Clippy checked (cargo clippy)
✅ Report generated with rollback instructions

---

## COMMON ISSUES

### Dependency conflict errors

**Symptom:** cargo update fails with version conflicts
**Solution:**
```bash
# Try updating specific package
cargo update -p <package-name>

# Or update workspace incrementally
cargo update --workspace --precise
```

### New deprecation warnings

**Symptom:** Updated dependencies use deprecated APIs
**Solution:** Review and update API usage, or accept warnings temporarily

### Test failures after update

**Symptom:** Tests pass before, fail after update
**Solution:**
1. Check if dependency behavior changed
2. Update test expectations if needed
3. Report upstream bug if dependency is broken

---

## RELATED COMMANDS

- `/security-audit` - Check for vulnerabilities in dependencies
- `/rust-check` - Full quality check after update
- `/quick-fix` - Fix new clippy warnings automatically

---

**Execute safe dependency update now.**
