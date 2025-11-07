# Security Audit - Comprehensive

Run comprehensive security audit (dependencies, code, fuzzing).

---

## Phase 1: DEPENDENCY SECURITY AUDIT

Check dependencies for known vulnerabilities.

### Step 1.1: Run Cargo Audit

```bash
echo "1/4: Auditing dependencies for vulnerabilities..."
echo ""

# Ensure cargo-audit is installed
if ! cargo audit --version &> /dev/null; then
    echo "Installing cargo-audit..."
    cargo install cargo-audit --quiet
fi

# Run audit and save results
cargo audit --json > /tmp/security-audit-deps.json 2>/dev/null || {
    echo "⚠️  cargo audit failed, trying without JSON..."
    cargo audit 2>&1 | tee /tmp/security-audit-deps.txt
}

# Parse results
if [ -f /tmp/security-audit-deps.json ]; then
    VULNS=$(jq -r '.vulnerabilities.count // 0' /tmp/security-audit-deps.json 2>/dev/null || echo "0")
else
    VULNS=$(grep -c "warning:" /tmp/security-audit-deps.txt 2>/dev/null || echo "0")
fi

if [ "$VULNS" -eq 0 ]; then
    echo "✅ No known vulnerabilities in dependencies"
else
    echo "❌ Found $VULNS vulnerabilities"
    echo ""
    cargo audit
fi

echo ""
```

---

## Phase 2: LICENSE COMPLIANCE CHECK

Verify all dependencies use compatible licenses.

### Step 2.1: Run Cargo Deny

```bash
echo "2/4: Checking dependency licenses..."
echo ""

# Ensure cargo-deny is installed
if ! cargo deny --version &> /dev/null; then
    echo "Installing cargo-deny..."
    cargo install cargo-deny --quiet
fi

# Check licenses
if cargo deny check licenses 2>&1 | tee /tmp/security-audit-licenses.txt | grep -q "error"; then
    echo ""
    echo "❌ License issues found"
    LICENSE_STATUS="FAIL"
else
    echo ""
    echo "✅ All licenses compatible with GPL-3.0"
    LICENSE_STATUS="PASS"
fi

echo ""
```

---

## Phase 3: CODE SECURITY SCAN

Run security-focused clippy lints.

### Step 3.1: Security Clippy Lints

```bash
echo "3/4: Running security-focused clippy..."
echo ""

cargo clippy --all-targets --all-features -- \
    -W clippy::suspicious \
    -W clippy::complexity \
    -W clippy::perf \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -W clippy::panic \
    -W clippy::integer_arithmetic \
    2>&1 | tee /tmp/security-audit-clippy.txt

echo ""

# Count security-relevant warnings
SECURITY_WARNINGS=$(grep -c "warning:" /tmp/security-audit-clippy.txt 2>/dev/null || echo "0")

if [ "$SECURITY_WARNINGS" -eq 0 ]; then
    echo "✅ No security-relevant warnings"
else
    echo "⚠️  $SECURITY_WARNINGS security-relevant warnings found"
fi

echo ""
```

---

## Phase 4: FUZZING STATUS CHECK

Verify fuzzing has been run and no crashes exist.

### Step 4.1: Check Fuzz Artifacts

```bash
echo "4/4: Checking fuzzing status..."
echo ""

if [ ! -d "fuzz" ]; then
    echo "⚠️  Fuzz directory not found (fuzzing not set up)"
    echo "   Run Sprint 5.7 to set up fuzzing infrastructure"
    FUZZ_STATUS="NOT_SET_UP"
elif [ ! -d "fuzz/artifacts" ]; then
    echo "⚠️  No fuzz artifacts (fuzzing not yet run)"
    echo "   Run: cargo +nightly fuzz run <target>"
    FUZZ_STATUS="NOT_RUN"
else
    # Check for crashes
    CRASHES=$(find fuzz/artifacts -name "crash-*" -o -name "timeout-*" 2>/dev/null | wc -l)

    if [ "$CRASHES" -eq 0 ]; then
        echo "✅ No fuzz crashes found"
        FUZZ_STATUS="PASS"

        # Show corpus stats if available
        if [ -d "fuzz/corpus" ]; then
            CORPUS_SIZE=$(find fuzz/corpus -type f 2>/dev/null | wc -l)
            echo "   Corpus size: $CORPUS_SIZE seeds"
        fi
    else
        echo "❌ Found $CRASHES crash/timeout artifacts"
        echo ""
        echo "Crash locations:"
        find fuzz/artifacts -name "crash-*" -o -name "timeout-*" 2>/dev/null | head -10
        echo ""
        echo "Investigate with: cargo +nightly fuzz run <target> <artifact-file>"
        FUZZ_STATUS="CRASHES_FOUND"
    fi
fi

echo ""
```

---

## Phase 5: GENERATE SECURITY REPORT

Create comprehensive security audit report.

### Step 5.1: Create Report

```bash
REPORT_DATE=$(date +"%Y-%m-%d %H:%M:%S")

cat > /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << EOF
# Security Audit Report

**Date:** $REPORT_DATE
**Project:** ProRT-IP WarScan
**Version:** $(grep "^version" Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

---

## Executive Summary

| Check | Status | Issues | Severity |
|-------|--------|--------|----------|
| **Dependency Vulnerabilities** | $([ "$VULNS" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL") | $VULNS | $([ "$VULNS" -eq 0 ] && echo "NONE" || echo "HIGH") |
| **License Compliance** | $([ "$LICENSE_STATUS" = "PASS" ] && echo "✅ PASS" || echo "❌ FAIL") | See licenses.txt | $([ "$LICENSE_STATUS" = "PASS" ] && echo "NONE" || echo "MEDIUM") |
| **Code Security** | $([ "$SECURITY_WARNINGS" -eq 0 ] && echo "✅ PASS" || echo "⚠️  WARNING") | $SECURITY_WARNINGS warnings | $([ "$SECURITY_WARNINGS" -eq 0 ] && echo "NONE" || echo "LOW") |
| **Fuzzing** | $([ "$FUZZ_STATUS" = "PASS" ] && echo "✅ PASS" || echo "❌ $FUZZ_STATUS") | ${CRASHES:-0} crashes | $([ "$FUZZ_STATUS" = "PASS" ] && echo "NONE" || echo "CRITICAL") |

**Overall Risk Level:** $(
    if [ "$VULNS" -gt 0 ] || [ "$FUZZ_STATUS" = "CRASHES_FOUND" ]; then
        echo "🔴 HIGH (immediate action required)"
    elif [ "$LICENSE_STATUS" = "FAIL" ] || [ "$SECURITY_WARNINGS" -gt 10 ]; then
        echo "🟡 MEDIUM (address before release)"
    else
        echo "🟢 LOW (acceptable for production)"
    fi
)

---

## 1. Dependency Vulnerabilities

**Status:** $([ "$VULNS" -eq 0 ] && echo "✅ PASS" || echo "❌ $VULNS vulnerabilities")

EOF

if [ "$VULNS" -gt 0 ]; then
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << 'EOF'

### Vulnerabilities Found

EOF
    cargo audit 2>&1 | sed 's/^/    /' >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << 'EOF'

### Remediation

1. Update vulnerable dependencies:
   ```bash
   cargo update
   cargo audit
   ```

2. If updates don't resolve:
   - Check for patched versions
   - Consider alternative dependencies
   - Apply workarounds if available

3. Verify fixes:
   ```bash
   /security-audit
   ```

EOF
else
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << 'EOF'

No vulnerabilities detected in dependencies.

**Last Advisory Check:** $REPORT_DATE
**Recommendation:** Run weekly with `cargo audit`

EOF
fi

cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << EOF

---

## 2. License Compliance

**Status:** $([ "$LICENSE_STATUS" = "PASS" ] && echo "✅ PASS" || echo "❌ FAIL")

**Project License:** GPL-3.0

EOF

if [ "$LICENSE_STATUS" = "FAIL" ]; then
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << 'EOF'

### Issues Found

See /tmp/security-audit-licenses.txt for details.

### Remediation

1. Review incompatible licenses
2. Replace incompatible dependencies
3. Verify GPL-3.0 compatibility
4. Document license decisions

EOF
else
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << 'EOF'

All dependencies use GPL-3.0 compatible licenses.

EOF
fi

cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << EOF

---

## 3. Code Security

**Status:** $([ "$SECURITY_WARNINGS" -eq 0 ] && echo "✅ PASS" || echo "⚠️  $SECURITY_WARNINGS warnings")

EOF

if [ "$SECURITY_WARNINGS" -gt 0 ]; then
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << 'EOF'

### Security Warnings

Top security-relevant warnings:

EOF
    grep "warning:" /tmp/security-audit-clippy.txt | head -10 | sed 's/^/    /' >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << 'EOF'

### Remediation

1. Review each warning context
2. Replace unsafe patterns:
   - `unwrap()` → `unwrap_or()` / `expect()` with context
   - `panic!()` → proper error handling
   - Integer overflow → checked arithmetic
3. Re-run audit after fixes

EOF
else
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << 'EOF'

No security-relevant clippy warnings detected.

EOF
fi

cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << EOF

---

## 4. Fuzzing Status

**Status:** $FUZZ_STATUS

EOF

if [ "$FUZZ_STATUS" = "PASS" ]; then
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << EOF

✅ No crashes found in fuzzing artifacts.

**Corpus Size:** ${CORPUS_SIZE:-Unknown} seeds
**Recommendation:** Continue nightly fuzzing via CI/CD

EOF
elif [ "$FUZZ_STATUS" = "CRASHES_FOUND" ]; then
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << EOF

❌ **CRITICAL:** $CRASHES crash/timeout artifacts found.

### Crash Locations

EOF
    find fuzz/artifacts -name "crash-*" -o -name "timeout-*" 2>/dev/null | sed 's/^/- /' >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << 'EOF'

### Remediation (URGENT)

1. Reproduce each crash:
   ```bash
   cargo +nightly fuzz run <target> <crash-file>
   ```

2. Debug with:
   ```bash
   RUST_BACKTRACE=1 cargo +nightly fuzz run <target> <crash-file>
   ```

3. Fix root causes
4. Verify fixes:
   ```bash
   cargo +nightly fuzz run <target> -- -runs=1000000
   ```

5. Re-run full audit:
   ```bash
   /security-audit
   ```

**DO NOT RELEASE WITH UNRESOLVED CRASHES**

EOF
else
    cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << EOF

⚠️  Fuzzing infrastructure: $FUZZ_STATUS

**Recommendation:** Set up fuzzing infrastructure (Sprint 5.7)

EOF
fi

cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << 'EOF'

---

## Recommendations

### Immediate Actions (Next 24h)

EOF

if [ "$VULNS" -gt 0 ]; then
    echo "- [ ] Fix dependency vulnerabilities (HIGH priority)" >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md
fi

if [ "$FUZZ_STATUS" = "CRASHES_FOUND" ]; then
    echo "- [ ] Fix all fuzz crashes (CRITICAL priority)" >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md
fi

cat >> /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md << EOF

### Short-term Actions (Next Week)

$([ "$LICENSE_STATUS" = "FAIL" ] && echo "- [ ] Resolve license compatibility issues (MEDIUM priority)")
$([ "$SECURITY_WARNINGS" -gt 0 ] && echo "- [ ] Address security-relevant clippy warnings (LOW priority)")
$([ "$FUZZ_STATUS" = "NOT_SET_UP" ] && echo "- [ ] Set up fuzzing infrastructure (MEDIUM priority)")
$([ "$FUZZ_STATUS" = "NOT_RUN" ] && echo "- [ ] Run initial fuzzing campaign (MEDIUM priority)")

### Long-term Actions (Ongoing)

- [ ] Weekly cargo audit runs
- [ ] Nightly fuzzing via CI/CD
- [ ] Quarterly security review
- [ ] Dependency update cadence

---

## Next Steps

1. **Review this report:** /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md
2. **Address HIGH/CRITICAL issues immediately**
3. **Plan fixes for MEDIUM issues**
4. **Document accepted LOW issues**
5. **Re-run audit after fixes:** /security-audit

---

## References

- Security Policy: SECURITY.md
- Security Documentation: docs/08-SECURITY.md
- Fuzzing Guide: docs/29-FUZZING-GUIDE.md (if Sprint 5.7 complete)
- CVE Database: https://rustsec.org

---

**Generated:** $REPORT_DATE
**Tool:** ProRT-IP Security Audit (/security-audit)
EOF

echo "✅ Security audit report generated"
echo ""
```

### Step 5.2: Display Report

```bash
cat /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md

echo ""
echo "Report saved: /tmp/ProRT-IP/SECURITY-AUDIT-REPORT.md"
echo ""
```

---

## SUCCESS CRITERIA

✅ Dependency audit complete (cargo audit)
✅ License compliance checked (cargo deny)
✅ Security clippy lints executed
✅ Fuzzing status verified
✅ Comprehensive report generated
✅ Remediation steps provided

---

## RELATED COMMANDS

- `/deps-update` - Update dependencies after finding vulnerabilities
- `/fuzz-check` - Quick fuzzing validation (Sprint 5.7)
- `/pre-release` - Pre-release checklist (includes security audit)

---

**Execute comprehensive security audit now.**
