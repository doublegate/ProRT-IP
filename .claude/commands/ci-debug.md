# CI/CD Debug - Troubleshoot Failures

Troubleshoot and analyze CI/CD workflow failures.

---

## Phase 1: CHECK CI STATUS

Get current CI/CD workflow status.

### Step 1.1: Verify gh CLI

```bash
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) not installed"
    echo ""
    echo "Install:"
    echo "  Debian/Ubuntu: sudo apt install gh"
    echo "  macOS: brew install gh"
    echo "  Other: https://cli.github.com/"
    echo ""
    echo "After install: gh auth login"
    exit 1
fi

echo "✅ GitHub CLI available"
echo ""
```

### Step 1.2: List Recent Runs

```bash
echo "Recent CI/CD runs:"
echo ""

gh run list --limit 10 --json conclusion,name,headBranch,createdAt,status \
    --jq '.[] | "\(.status | @text) | \(.name) | \(.headBranch) | \(.createdAt)"' | \
    column -t -s '|'

echo ""
```

### Step 1.3: Get Latest Run Status

```bash
LATEST_RUN=$(gh run list --limit 1 --json databaseId,conclusion,name,status --jq '.[0]')

RUN_ID=$(echo "$LATEST_RUN" | jq -r '.databaseId')
RUN_STATUS=$(echo "$LATEST_RUN" | jq -r '.status')
RUN_CONCLUSION=$(echo "$LATEST_RUN" | jq -r '.conclusion')
RUN_NAME=$(echo "$LATEST_RUN" | jq -r '.name')

echo "Latest Run: $RUN_NAME (ID: $RUN_ID)"
echo "Status: $RUN_STATUS"
echo "Conclusion: $RUN_CONCLUSION"
echo ""

if [ "$RUN_CONCLUSION" = "failure" ]; then
    echo "❌ Latest CI run FAILED"
elif [ "$RUN_CONCLUSION" = "success" ]; then
    echo "✅ Latest CI run PASSED"
else
    echo "⏳ Latest CI run: $RUN_STATUS"
fi

echo ""
```

---

## Phase 2: ANALYZE FAILURES

For failed runs, extract failure details.

### Step 2.1: Get Failed Jobs

```bash
if [ "$RUN_CONCLUSION" = "failure" ]; then
    echo "Analyzing failed jobs..."
    echo ""

    FAILED_JOBS=$(gh run view "$RUN_ID" --json jobs \
        --jq '.jobs[] | select(.conclusion == "failure") | .name')

    if [ -z "$FAILED_JOBS" ]; then
        echo "⚠️  No specific job failures found (may still be running)"
    else
        echo "Failed jobs:"
        echo "$FAILED_JOBS" | sed 's/^/  • /'
    fi

    echo ""
fi
```

### Step 2.2: Extract Error Logs

```bash
if [ "$RUN_CONCLUSION" = "failure" ]; then
    echo "Extracting error logs..."
    echo ""

    mkdir -p /tmp/ProRT-IP/ci-debug

    # Get full log
    gh run view "$RUN_ID" --log > /tmp/ProRT-IP/ci-debug/full-log.txt 2>&1

    # Extract errors
    grep -i "error\|Error\|ERROR\|failed\|Failed\|FAILED" /tmp/ProRT-IP/ci-debug/full-log.txt \
        > /tmp/ProRT-IP/ci-debug/errors.txt 2>/dev/null || true

    if [ -s /tmp/ProRT-IP/ci-debug/errors.txt ]; then
        echo "Error summary (first 20 lines):"
        echo ""
        head -20 /tmp/ProRT-IP/ci-debug/errors.txt | sed 's/^/  /'
        echo ""
        echo "Full errors: /tmp/ProRT-IP/ci-debug/errors.txt"
    else
        echo "No specific errors extracted from logs"
    fi

    echo ""
fi
```

---

## Phase 3: COMMON FAILURE PATTERNS

Identify common CI/CD failure patterns.

### Step 3.1: Categorize Failures

```bash
if [ "$RUN_CONCLUSION" = "failure" ] && [ -f /tmp/ProRT-IP/ci-debug/errors.txt ]; then
    echo "Analyzing failure patterns..."
    echo ""

    # Test failures
    if grep -qi "test.*failed\|FAILED.*test" /tmp/ProRT-IP/ci-debug/errors.txt; then
        echo "❌ Pattern: TEST FAILURES"
        grep -i "test.*failed\|FAILED.*test" /tmp/ProRT-IP/ci-debug/errors.txt | head -5 | sed 's/^/  /'
        echo ""
        echo "Fix: /test-quick to reproduce locally"
        echo ""
    fi

    # Build failures
    if grep -qi "error\[E[0-9]\+\]\|could not compile" /tmp/ProRT-IP/ci-debug/errors.txt; then
        echo "❌ Pattern: BUILD FAILURES"
        grep -i "error\[E[0-9]\+\]\|could not compile" /tmp/ProRT-IP/ci-debug/errors.txt | head -5 | sed 's/^/  /'
        echo ""
        echo "Fix: cargo build --all-features locally"
        echo ""
    fi

    # Dependency failures
    if grep -qi "failed to download\|failed to fetch\|dependency.*failed" /tmp/ProRT-IP/ci-debug/errors.txt; then
        echo "❌ Pattern: DEPENDENCY FAILURES"
        grep -i "failed to download\|failed to fetch\|dependency.*failed" /tmp/ProRT-IP/ci-debug/errors.txt | head -5 | sed 's/^/  /'
        echo ""
        echo "Fix: Check Cargo.lock, run /deps-update"
        echo ""
    fi

    # Clippy failures
    if grep -qi "clippy.*error\|clippy.*warning.*error" /tmp/ProRT-IP/ci-debug/errors.txt; then
        echo "❌ Pattern: CLIPPY FAILURES"
        grep -i "clippy.*error\|clippy.*warning.*error" /tmp/ProRT-IP/ci-debug/errors.txt | head -5 | sed 's/^/  /'
        echo ""
        echo "Fix: /quick-fix or cargo clippy --all-features"
        echo ""
    fi

    # Timeout failures
    if grep -qi "timeout\|timed out\|exceeded.*time" /tmp/ProRT-IP/ci-debug/errors.txt; then
        echo "❌ Pattern: TIMEOUT FAILURES"
        grep -i "timeout\|timed out\|exceeded.*time" /tmp/ProRT-IP/ci-debug/errors.txt | head -5 | sed 's/^/  /'
        echo ""
        echo "Fix: Increase timeout in workflow or optimize tests"
        echo ""
    fi

    # Platform-specific failures
    if grep -qi "windows\|macos\|ubuntu" /tmp/ProRT-IP/ci-debug/errors.txt; then
        echo "❌ Pattern: PLATFORM-SPECIFIC FAILURES"
        echo ""
        echo "Platform mentions found - may be platform-specific issue"
        echo "Check: Platform compatibility in docs/13-PLATFORM-SUPPORT.md"
        echo ""
    fi
fi
```

---

## Phase 4: LOCAL REPRODUCTION

Provide commands to reproduce failures locally.

### Step 4.1: Generate Reproduction Steps

```bash
cat > /tmp/ProRT-IP/ci-debug/reproduce.sh << 'EOF'
#!/bin/bash
# CI/CD Failure Reproduction Script

echo "Reproducing CI/CD environment locally..."
echo ""

# 1. Clean environment
echo "1/5: Cleaning build artifacts..."
cargo clean

# 2. Build all features
echo "2/5: Building with all features..."
if ! cargo build --all-features; then
    echo "❌ Build failed (matches CI failure)"
    exit 1
fi

# 3. Run clippy
echo "3/5: Running clippy..."
if ! cargo clippy --all-features -- -D warnings; then
    echo "❌ Clippy failed (matches CI failure)"
    exit 1
fi

# 4. Run tests
echo "4/5: Running tests..."
if ! cargo test --all-features; then
    echo "❌ Tests failed (matches CI failure)"
    exit 1
fi

# 5. Build release
echo "5/5: Building release..."
if ! cargo build --release; then
    echo "❌ Release build failed (matches CI failure)"
    exit 1
fi

echo ""
echo "✅ All checks passed locally"
echo "If CI still fails, may be platform-specific or environment-specific"
EOF

chmod +x /tmp/ProRT-IP/ci-debug/reproduce.sh

echo "Reproduction script created: /tmp/ProRT-IP/ci-debug/reproduce.sh"
echo ""
echo "Run with: bash /tmp/ProRT-IP/ci-debug/reproduce.sh"
echo ""
```

---

## Phase 5: GENERATE DEBUG REPORT

Create comprehensive CI debug report.

### Step 5.1: Create Report

```bash
REPORT_DATE=$(date +"%Y-%m-%d %H:%M:%S")

cat > /tmp/ProRT-IP/CI-DEBUG-REPORT.md << EOF
# CI/CD Debug Report

**Date:** $REPORT_DATE
**Latest Run:** $RUN_NAME (ID: $RUN_ID)
**Status:** $RUN_STATUS
**Conclusion:** $RUN_CONCLUSION

---

## Run Summary

| Field | Value |
|-------|-------|
| **Run ID** | $RUN_ID |
| **Workflow** | $RUN_NAME |
| **Status** | $RUN_STATUS |
| **Conclusion** | $RUN_CONCLUSION |
| **URL** | https://github.com/doublegate/ProRT-IP/actions/runs/$RUN_ID |

---

## Recent Runs

\`\`\`
$(gh run list --limit 10 --json conclusion,name,headBranch,createdAt \
    --jq '.[] | "\(.conclusion) | \(.name) | \(.headBranch) | \(.createdAt)"' 2>/dev/null || echo "N/A")
\`\`\`

---

## Failure Analysis

EOF

if [ "$RUN_CONCLUSION" = "failure" ]; then
    cat >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md << EOF

### Failed Jobs

$(echo "$FAILED_JOBS" | sed 's/^/- /')

### Error Patterns Detected

EOF

    # Add detected patterns
    [ -f /tmp/ProRT-IP/ci-debug/errors.txt ] && {
        grep -qi "test.*failed" /tmp/ProRT-IP/ci-debug/errors.txt && \
            echo "- ❌ Test failures" >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md
        grep -qi "error\[E[0-9]\+\]" /tmp/ProRT-IP/ci-debug/errors.txt && \
            echo "- ❌ Build errors" >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md
        grep -qi "clippy.*error" /tmp/ProRT-IP/ci-debug/errors.txt && \
            echo "- ❌ Clippy errors" >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md
        grep -qi "timeout" /tmp/ProRT-IP/ci-debug/errors.txt && \
            echo "- ⏱️  Timeout issues" >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md
    }

    cat >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md << 'EOF'

### Error Excerpts

```
EOF

    if [ -f /tmp/ProRT-IP/ci-debug/errors.txt ]; then
        head -30 /tmp/ProRT-IP/ci-debug/errors.txt >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md
    else
        echo "(No errors extracted)" >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md
    fi

    cat >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md << 'EOF'
```

### Full Logs

- **Full log:** /tmp/ProRT-IP/ci-debug/full-log.txt
- **Errors only:** /tmp/ProRT-IP/ci-debug/errors.txt

---

## Reproduction Steps

To reproduce locally:

```bash
bash /tmp/ProRT-IP/ci-debug/reproduce.sh
```

Or manually:

```bash
# Clean build
cargo clean

# Build with all features
cargo build --all-features

# Run clippy
cargo clippy --all-features -- -D warnings

# Run tests
cargo test --all-features

# Build release
cargo build --release
```

---

## Recommended Fixes

EOF

    # Add specific fix recommendations based on patterns
    if [ -f /tmp/ProRT-IP/ci-debug/errors.txt ]; then
        if grep -qi "test.*failed" /tmp/ProRT-IP/ci-debug/errors.txt; then
            cat >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md << 'EOF'

### Test Failures

1. Identify failing tests:
   ```bash
   cargo test --all-features | grep FAILED
   ```

2. Run specific test:
   ```bash
   /test-quick <test-name>
   ```

3. Debug with output:
   ```bash
   cargo test <test-name> -- --nocapture
   ```

EOF
        fi

        if grep -qi "clippy.*error" /tmp/ProRT-IP/ci-debug/errors.txt; then
            cat >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md << 'EOF'

### Clippy Errors

1. Auto-fix simple issues:
   ```bash
   /quick-fix
   ```

2. Manual review:
   ```bash
   cargo clippy --all-features
   ```

EOF
        fi
    fi

else
    cat >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md << EOF

✅ Latest CI run passed successfully.

No failures to debug.

EOF
fi

cat >> /tmp/ProRT-IP/CI-DEBUG-REPORT.md << 'EOF'

---

## CI/CD Workflows

Project workflows:

- **CI:** .github/workflows/ci.yml (build, test, clippy)
- **Coverage:** .github/workflows/coverage.yml (code coverage)
- **CodeQL:** .github/workflows/codeql.yml (security scanning)
- **Fuzz:** .github/workflows/fuzz.yml (nightly fuzzing)
- **Release:** .github/workflows/release.yml (multi-platform builds)

---

## Next Steps

1. ✅ Review this report
2. ⬜ Reproduce locally if failed
3. ⬜ Fix identified issues
4. ⬜ Run: /rust-check
5. ⬜ Push fixes and monitor CI

---

## Useful Commands

- **View run:** \`gh run view $RUN_ID\`
- **View logs:** \`gh run view $RUN_ID --log\`
- **Re-run:** \`gh run rerun $RUN_ID\`
- **Watch:** \`gh run watch $RUN_ID\`

---

**Generated:** $REPORT_DATE
**Tool:** ProRT-IP CI Debug (/ci-debug)
EOF

echo "✅ CI debug report generated"
echo ""
```

### Step 5.2: Display Summary

```bash
cat << EOF

╔════════════════════════════════════════════════════════════════╗
║                   CI/CD Debug Summary                          ║
╚════════════════════════════════════════════════════════════════╝

📊 RUN STATUS

• Run ID: $RUN_ID
• Workflow: $RUN_NAME
• Conclusion: $RUN_CONCLUSION

$(if [ "$RUN_CONCLUSION" = "failure" ]; then
    echo "❌ STATUS: FAILED"
    echo ""
    echo "Failed Jobs:"
    echo "$FAILED_JOBS" | sed 's/^/  • /'
else
    echo "✅ STATUS: PASSED"
fi)

📁 ARTIFACTS

• Report: /tmp/ProRT-IP/CI-DEBUG-REPORT.md
$([ -f /tmp/ProRT-IP/ci-debug/full-log.txt ] && echo "• Full log: /tmp/ProRT-IP/ci-debug/full-log.txt")
$([ -f /tmp/ProRT-IP/ci-debug/errors.txt ] && echo "• Errors: /tmp/ProRT-IP/ci-debug/errors.txt")
• Reproduction: /tmp/ProRT-IP/ci-debug/reproduce.sh

🚀 NEXT STEPS

$(if [ "$RUN_CONCLUSION" = "failure" ]; then
    echo "1. Review error patterns above"
    echo "2. Reproduce locally: bash /tmp/ProRT-IP/ci-debug/reproduce.sh"
    echo "3. Fix issues"
    echo "4. Verify: /rust-check"
    echo "5. Push and monitor CI"
else
    echo "✅ CI passing - no action needed"
fi)

🔗 LINKS

• View run: gh run view $RUN_ID
• Re-run: gh run rerun $RUN_ID
• GitHub Actions: https://github.com/doublegate/ProRT-IP/actions

EOF
```

---

## SUCCESS CRITERIA

✅ GitHub CLI verified and authenticated
✅ Recent CI runs listed
✅ Latest run status retrieved
✅ Failure analysis performed (if failed)
✅ Error logs extracted and categorized
✅ Reproduction script generated
✅ Debug report created with recommendations

---

## RELATED COMMANDS

- `/ci-status` - Quick CI status check
- `/rust-check` - Local quality validation before pushing
- `/pre-release` - Includes CI status check

---

**Execute CI/CD debug analysis now.**
