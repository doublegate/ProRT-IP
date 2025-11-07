# Quick Fix - Auto Code Quality

Rapidly fix common code quality issues (formatting, clippy warnings, imports).

---

## Phase 1: AUTO-FIX CODE ISSUES

Apply automatic fixes for common issues.

### Step 1.1: Format All Code

```bash
echo "1/4: Formatting code..."
cargo fmt --all

if [ $? -eq 0 ]; then
    echo "✅ Code formatted"
else
    echo "❌ Format failed"
    exit 1
fi

echo ""
```

### Step 1.2: Fix Clippy Warnings

```bash
echo "2/4: Fixing clippy warnings (allow-dirty)..."
cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged 2>&1 | \
    grep -E "Fixing|Fixed|Checking|error" || echo "No auto-fixable warnings"

echo "✅ Clippy fixes applied"
echo ""
```

### Step 1.3: Apply Compiler Suggestions

```bash
echo "3/4: Applying compiler suggestions..."
cargo fix --all-targets --allow-dirty --allow-staged 2>&1 | \
    grep -E "Fixing|Fixed|Checking|error" || echo "No compiler suggestions"

echo "✅ Compiler suggestions applied"
echo ""
```

### Step 1.4: Verify Build

```bash
echo "4/4: Verifying build after fixes..."

if cargo check --all-features --quiet 2>&1 | grep -q "error"; then
    echo "❌ Build errors remain after fixes"
    echo ""
    echo "Remaining errors:"
    cargo check --all-features 2>&1 | grep "error" | head -10
    echo ""
    echo "Manual intervention required"
    exit 1
else
    echo "✅ Build successful"
fi

echo ""
```

---

## Phase 2: SHOW CHANGES

Display what was changed by auto-fix.

### Step 2.1: Git Status

```bash
echo "Changes made by quick-fix:"
echo ""

if [ -z "$(git status --porcelain)" ]; then
    echo "✅ No changes made (code was already clean)"
else
    git status --short
    echo ""
    echo "Diff statistics:"
    git diff --stat
fi

echo ""
```

---

## Phase 3: VERIFY QUALITY

Check remaining issues after auto-fix.

### Step 3.1: Remaining Clippy Warnings

```bash
echo "Checking for remaining clippy warnings..."

CLIPPY_OUTPUT=$(cargo clippy --all-features --quiet 2>&1)
WARNINGS=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:" || echo "0")

if [ "$WARNINGS" -eq 0 ]; then
    echo "✅ Zero clippy warnings"
else
    echo "⚠️  $WARNINGS clippy warnings remain (manual fixes needed)"
    echo ""
    echo "Run: cargo clippy --all-features"
    echo "Or: /rust-check for detailed analysis"
fi

echo ""
```

### Step 3.2: Format Check

```bash
echo "Verifying format consistency..."

if cargo fmt --all -- --check 2>&1 | grep -q "Diff"; then
    echo "⚠️  Some files still need formatting"
    echo "Run: cargo fmt --all"
else
    echo "✅ All files properly formatted"
fi

echo ""
```

### Step 3.3: Build Warnings

```bash
echo "Checking for build warnings..."

BUILD_OUTPUT=$(cargo build --all-features 2>&1)
BUILD_WARNINGS=$(echo "$BUILD_OUTPUT" | grep -c "warning:" || echo "0")

if [ "$BUILD_WARNINGS" -eq 0 ]; then
    echo "✅ Zero build warnings"
else
    echo "⚠️  $BUILD_WARNINGS build warnings remain"
    echo ""
    echo "Common warnings:"
    echo "$BUILD_OUTPUT" | grep "warning:" | head -5
fi

echo ""
```

---

## Phase 4: SUMMARY AND NEXT STEPS

### Step 4.1: Display Summary

```bash
cat << EOF

╔════════════════════════════════════════════════════════════════╗
║                   Quick Fix Summary                            ║
╚════════════════════════════════════════════════════════════════╝

📊 FIXES APPLIED

✅ Code formatted (cargo fmt)
✅ Clippy warnings auto-fixed
✅ Compiler suggestions applied
✅ Build verified

📈 REMAINING ISSUES

• Clippy warnings: $WARNINGS
• Build warnings: $BUILD_WARNINGS
• Format issues: $(cargo fmt --all -- --check 2>&1 | grep -c "Diff" || echo "0")

EOF

if [ "$WARNINGS" -eq 0 ] && [ "$BUILD_WARNINGS" -eq 0 ]; then
    cat << 'EOF'

✅ CODE QUALITY: EXCELLENT

All auto-fixable issues resolved. Code is clean and ready.

🚀 NEXT STEPS

1. Review changes: git diff
2. Test changes: cargo test
3. Commit: git add . && git commit -m "fix: Apply auto code quality fixes"

EOF
else
    cat << 'EOF'

⚠️  MANUAL FIXES NEEDED

Auto-fix resolved simple issues, but some remain.

🔧 NEXT STEPS

1. Review changes: git diff
2. Check detailed issues: cargo clippy --all-features
3. Run comprehensive check: /rust-check
4. Fix remaining issues manually
5. Re-run: /quick-fix

EOF
fi
```

---

## USAGE NOTES

**Best For:**
- Before committing code
- After bulk code changes
- CI/CD failure quick fixes
- Pre-code review cleanup

**Not For:**
- Complex logic errors (needs manual fixing)
- Test failures (use /test-quick)
- API breaking changes (needs design review)

**Typical Runtime:** 30-60 seconds

---

## COMMON FIXES

### Formatting Issues
- Trailing whitespace removed
- Indentation normalized
- Line length adjusted
- Import sorting

### Clippy Auto-Fixes
- Unnecessary clones → references
- Unused variables → prefixed with `_`
- Redundant closures → direct calls
- Inefficient algorithms → optimized versions

### Compiler Suggestions
- Unused imports removed
- Missing `#[must_use]` attributes
- Deprecated API updates
- Type inference improvements

---

## RELATED COMMANDS

- `/rust-check` - Comprehensive quality check (includes quick-fix logic)
- `/test-quick` - Verify fixes didn't break tests
- `/pre-release` - Full pre-release validation

---

**Execute quick code quality fixes now.**
