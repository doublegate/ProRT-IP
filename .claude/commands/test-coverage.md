# Test Coverage Analysis

Generate comprehensive test coverage report with detailed analysis.

---

## Phase 1: PREREQUISITES CHECK

Ensure coverage tools are installed.

### Step 1.1: Install cargo-tarpaulin

```bash
if ! cargo tarpaulin --version &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin --quiet
    echo "✅ cargo-tarpaulin installed"
    echo ""
fi
```

---

## Phase 2: GENERATE COVERAGE

Run tests with coverage instrumentation.

### Step 2.1: Run Tarpaulin

```bash
echo "Generating coverage report..."
echo "This will take 2-5 minutes..."
echo ""

mkdir -p coverage

# Run tarpaulin with multiple output formats
OUTPUT=$(cargo tarpaulin \
    --workspace \
    --all-features \
    --timeout 600 \
    --out Html --out Lcov --out Json \
    --output-dir coverage \
    --exclude-files "crates/prtip-cli/src/main.rs" \
    --skip-clean \
    2>&1)

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ Coverage generation failed"
    echo "$OUTPUT" | grep -E "error|Error" | head -10
    exit 1
fi

# Extract coverage percentage from output
COVERAGE=$(echo "$OUTPUT" | grep -oP '\d+\.\d+(?=% coverage)' | tail -1)

if [ -z "$COVERAGE" ]; then
    # Try extracting from JSON
    COVERAGE=$(jq -r '.files | map(.percent_covered) | add / length' coverage/cobertura.json 2>/dev/null || echo "N/A")
fi

echo ""
echo "✅ Coverage report generated: ${COVERAGE}%"
echo ""
```

---

## Phase 3: ANALYZE COVERAGE

Extract detailed coverage statistics.

### Step 3.1: Overall Statistics

```bash
echo "Analyzing coverage data..."
echo ""

if [ -f coverage/tarpaulin-report.json ]; then
    TOTAL_LINES=$(jq -r '[.files[].traces | length] | add' coverage/tarpaulin-report.json 2>/dev/null || echo "N/A")
    COVERED_LINES=$(jq -r '[.files[].traces[] | select(. > 0)] | length' coverage/tarpaulin-report.json 2>/dev/null || echo "N/A")

    echo "📊 OVERALL STATISTICS"
    echo "  Total Coverage: ${COVERAGE}%"
    echo "  Total Lines: $TOTAL_LINES"
    echo "  Covered Lines: $COVERED_LINES"
    echo "  Uncovered Lines: $((TOTAL_LINES - COVERED_LINES))"
    echo ""
fi
```

### Step 3.2: Low Coverage Files

```bash
echo "📉 FILES WITH LOW COVERAGE (<60%)"
echo ""

if [ -f coverage/tarpaulin-report.json ]; then
    jq -r '.files[] | select(.coverage < 60) | "  \(.name): \(.coverage)%"' \
        coverage/tarpaulin-report.json 2>/dev/null | head -15

    LOW_COUNT=$(jq -r '[.files[] | select(.coverage < 60)] | length' \
        coverage/tarpaulin-report.json 2>/dev/null || echo "0")

    if [ "$LOW_COUNT" -eq 0 ]; then
        echo "  ✅ No files below 60% threshold"
    else
        echo ""
        echo "  Found $LOW_COUNT files below 60% threshold"
    fi
else
    echo "  (JSON report not available)"
fi

echo ""
```

### Step 3.3: Coverage by Crate

```bash
echo "📦 COVERAGE BY CRATE"
echo ""

for crate in crates/*; do
    if [ -d "$crate" ]; then
        crate_name=$(basename "$crate")

        if [ -f coverage/tarpaulin-report.json ]; then
            crate_cov=$(jq -r "
                [.files[] | select(.name | contains(\"$crate_name\")) | .coverage] |
                if length > 0 then (add / length) else null end
            " coverage/tarpaulin-report.json 2>/dev/null)

            if [ "$crate_cov" != "null" ] && [ -n "$crate_cov" ]; then
                printf "  %-20s: %5.2f%%\n" "$crate_name" "$crate_cov"
            fi
        fi
    fi
done

echo ""
```

### Step 3.4: Top Uncovered Functions

```bash
echo "🔍 TOP UNCOVERED AREAS"
echo ""

if [ -f coverage/tarpaulin-report.json ]; then
    echo "Files needing most attention (sorted by uncovered lines):"
    echo ""

    jq -r '.files[] |
        select(.coverage < 80) |
        {name: .name, coverage: .coverage, total: (.traces | length), covered: ([.traces[] | select(. > 0)] | length)} |
        "\(.name):\n  Coverage: \(.coverage)%\n  Uncovered: \(.total - .covered) lines\n"' \
        coverage/tarpaulin-report.json 2>/dev/null | head -50
else
    echo "(Detailed analysis requires JSON report)"
fi

echo ""
```

---

## Phase 4: GENERATE REPORT

Create markdown coverage report.

### Step 4.1: Create Coverage Report

```bash
REPORT_DATE=$(date +"%Y-%m-%d %H:%M:%S")

cat > /tmp/ProRT-IP/COVERAGE-REPORT.md << EOF
# Test Coverage Report

**Date:** $REPORT_DATE
**Project:** ProRT-IP WarScan
**Overall Coverage:** ${COVERAGE}%

---

## Executive Summary

| Metric | Value |
|--------|-------|
| **Overall Coverage** | ${COVERAGE}% |
| **Total Lines** | $TOTAL_LINES |
| **Covered Lines** | $COVERED_LINES |
| **Uncovered Lines** | $((TOTAL_LINES - COVERED_LINES)) |
| **Low Coverage Files** | $LOW_COUNT files (<60%) |

**Status:** $(
    if (( $(echo "$COVERAGE >= 60" | bc -l 2>/dev/null || echo "0") )); then
        echo "✅ MEETS MINIMUM THRESHOLD (60%)"
    else
        echo "❌ BELOW MINIMUM THRESHOLD (60%)"
    fi
)

**Target:** 60% minimum, 80% ideal

---

## Coverage by Crate

| Crate | Coverage |
|-------|----------|
EOF

for crate in crates/*; do
    if [ -d "$crate" ]; then
        crate_name=$(basename "$crate")

        if [ -f coverage/tarpaulin-report.json ]; then
            crate_cov=$(jq -r "
                [.files[] | select(.name | contains(\"$crate_name\")) | .coverage] |
                if length > 0 then (add / length) else null end
            " coverage/tarpaulin-report.json 2>/dev/null)

            if [ "$crate_cov" != "null" ] && [ -n "$crate_cov" ]; then
                echo "| $crate_name | $(printf "%.2f" "$crate_cov")% |" >> /tmp/ProRT-IP/COVERAGE-REPORT.md
            fi
        fi
    fi
done

cat >> /tmp/ProRT-IP/COVERAGE-REPORT.md << 'EOF'

---

## Files Needing Attention

### Low Coverage Files (<60%)

EOF

if [ -f coverage/tarpaulin-report.json ]; then
    jq -r '.files[] | select(.coverage < 60) | "- **\(.name)**: \(.coverage)%"' \
        coverage/tarpaulin-report.json 2>/dev/null >> /tmp/ProRT-IP/COVERAGE-REPORT.md || \
        echo "✅ No files below 60% threshold" >> /tmp/ProRT-IP/COVERAGE-REPORT.md
else
    echo "(Analysis requires JSON report)" >> /tmp/ProRT-IP/COVERAGE-REPORT.md
fi

cat >> /tmp/ProRT-IP/COVERAGE-REPORT.md << 'EOF'

---

## Recommendations

### Immediate Actions

1. **Review low coverage files** - Focus on files below 60%
2. **Add tests for critical paths** - Scanner, parser, network logic
3. **Integration tests** - End-to-end scanning scenarios
4. **Edge cases** - Error handling, boundary conditions

### Coverage Improvement Strategy

#### Priority 1: Critical Code (Target 80%+)
- Core scanning algorithms
- Packet parsing logic
- Service detection
- Security-sensitive code

#### Priority 2: Important Code (Target 70%+)
- Network handling
- Rate limiting
- Progress reporting
- Output formatting

#### Priority 3: Supporting Code (Target 60%+)
- CLI parsing
- Configuration handling
- Utility functions

### Testing Techniques

1. **Unit Tests**
   - Test individual functions in isolation
   - Use mocks for dependencies
   - Focus on edge cases

2. **Integration Tests**
   - Test component interactions
   - Use real network (localhost)
   - Verify end-to-end flows

3. **Property-Based Tests**
   - Use proptest for input variation
   - Test invariants hold
   - Catch edge cases automatically

4. **Benchmark Tests**
   - Ensure performance meets targets
   - Catch regressions early
   - Document performance characteristics

---

## Next Steps

1. ✅ Coverage report generated
2. ⬜ Review HTML report: coverage/index.html
3. ⬜ Identify uncovered critical code
4. ⬜ Write tests for uncovered areas
5. ⬜ Re-run coverage: /test-coverage
6. ⬜ Track progress toward 80% target

---

## View Reports

**HTML (Interactive):**
```bash
xdg-open coverage/index.html  # Linux
open coverage/index.html      # macOS
```

**LCOV (IDE Integration):**
```
coverage/lcov.info
```

**JSON (Programmatic):**
```
coverage/tarpaulin-report.json
```

---

## CI/CD Integration

Coverage is automatically tracked in CI/CD:

- **Workflow:** .github/workflows/coverage.yml
- **Codecov:** https://codecov.io/gh/doublegate/ProRT-IP
- **Threshold:** 50% minimum (enforced in CI)

---

**Generated:** $REPORT_DATE
**Tool:** ProRT-IP Coverage Analysis (/test-coverage)
EOF

echo "✅ Coverage report created"
echo ""
```

---

## Phase 5: OPEN REPORTS

Open coverage reports in browser.

### Step 5.1: Open HTML Report

```bash
echo "Opening coverage report in browser..."

if [ -f coverage/index.html ]; then
    if command -v xdg-open &> /dev/null; then
        xdg-open coverage/index.html 2>/dev/null &
    elif command -v open &> /dev/null; then
        open coverage/index.html 2>/dev/null &
    else
        echo "Manual: file://$(pwd)/coverage/index.html"
    fi
    echo "✅ HTML report opened"
else
    echo "❌ HTML report not found"
fi

echo ""
```

---

## Phase 6: SUMMARY

Display summary and next steps.

```bash
cat << EOF

╔════════════════════════════════════════════════════════════════╗
║              Coverage Analysis Complete                        ║
╚════════════════════════════════════════════════════════════════╝

📊 COVERAGE METRICS

• Overall: ${COVERAGE}%
• Target: 60% minimum, 80% ideal
• Status: $(
    if (( $(echo "$COVERAGE >= 60" | bc -l 2>/dev/null || echo "0") )); then
        echo "✅ MEETS THRESHOLD"
    else
        echo "❌ BELOW THRESHOLD"
    fi
)

📁 REPORTS GENERATED

• HTML: coverage/index.html (interactive)
• LCOV: coverage/lcov.info (IDE integration)
• JSON: coverage/tarpaulin-report.json (programmatic)
• Report: /tmp/ProRT-IP/COVERAGE-REPORT.md

🚀 NEXT STEPS

1. Review HTML report for visual coverage
2. Identify red/orange highlighted code
3. Write tests for uncovered critical paths
4. Re-run: /test-coverage
5. Track progress toward 80% target

📖 DOCUMENTATION

• Testing Guide: docs/06-TESTING.md
• CI/CD Coverage: docs/28-CI-CD-COVERAGE.md

EOF
```

---

## SUCCESS CRITERIA

✅ cargo-tarpaulin installed
✅ Coverage generated (HTML, LCOV, JSON)
✅ Statistics extracted and analyzed
✅ Low coverage files identified
✅ Coverage report created
✅ HTML report opened in browser

---

## RELATED COMMANDS

- `/test-quick` - Run specific tests before coverage
- `/rust-check` - Full quality check (includes testing)
- `/pre-release` - Pre-release checks (includes coverage validation)

---

**Execute coverage analysis now.**
