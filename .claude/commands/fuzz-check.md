# Fuzzing Quick Check

Run all fuzz targets for quick validation (5 minutes each).

---

## Phase 1: PREREQUISITES CHECK

Verify fuzzing infrastructure is set up.

### Step 1.1: Check Fuzz Directory

```bash
if [ ! -d "fuzz" ]; then
    echo "❌ Fuzz directory not found"
    echo ""
    echo "Fuzzing infrastructure not set up."
    echo "Run Sprint 5.7 setup to initialize fuzzing."
    echo ""
    echo "Quick setup:"
    echo "  cargo install cargo-fuzz --version 0.13.2 --locked"
    echo "  cargo +nightly fuzz init"
    exit 1
fi

echo "✅ Fuzz directory found"
echo ""
```

### Step 1.2: Check Nightly Toolchain

```bash
if ! rustup toolchain list | grep -q nightly; then
    echo "❌ Rust nightly toolchain not installed"
    echo ""
    echo "Install with:"
    echo "  rustup toolchain install nightly"
    exit 1
fi

echo "✅ Rust nightly toolchain installed"
echo ""
```

### Step 1.3: List Fuzz Targets

```bash
echo "Discovering fuzz targets..."
TARGETS=$(cargo +nightly fuzz list 2>/dev/null)

if [ -z "$TARGETS" ]; then
    echo "❌ No fuzz targets found"
    echo ""
    echo "Available targets should be in fuzz/fuzz_targets/"
    echo "Run Sprint 5.7 to create fuzz targets"
    exit 1
fi

TARGET_COUNT=$(echo "$TARGETS" | wc -w)
echo "✅ Found $TARGET_COUNT fuzz targets:"
echo "$TARGETS" | sed 's/^/  • /'
echo ""
```

---

## Phase 2: RUN FUZZERS

Execute each fuzz target with 5-minute time limit.

### Step 2.1: Create Results Directory

```bash
mkdir -p /tmp/ProRT-IP/fuzz-results
echo "Results will be saved to: /tmp/ProRT-IP/fuzz-results/"
echo ""
```

### Step 2.2: Execute Fuzz Targets

```bash
echo "Running fuzzing quick check (5 minutes per target)..."
echo "Total estimated time: $((TARGET_COUNT * 5)) minutes"
echo ""

START_TIME=$(date +%s)
PIDS=()

for target in $TARGETS; do
    echo "Starting fuzzer: $target"
    LOG_FILE="/tmp/ProRT-IP/fuzz-results/${target}.log"

    # Run fuzzer in background with 5-minute timeout
    (
        timeout 300 cargo +nightly fuzz run "$target" -- \
            -max_total_time=300 \
            -print_final_stats=1 \
            -workers=1 \
            > "$LOG_FILE" 2>&1
        EXIT_CODE=$?
        echo "EXIT_CODE=$EXIT_CODE" >> "$LOG_FILE"
    ) &

    PIDS+=($!)
done

echo ""
echo "All fuzzers started in background..."
echo "Waiting for completion (5 minutes)..."
echo ""

# Wait for all fuzzers to complete
for pid in "${PIDS[@]}"; do
    wait "$pid" 2>/dev/null || true
done

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo "✅ All fuzz runs complete (${DURATION}s)"
echo ""
```

---

## Phase 3: ANALYZE RESULTS

Extract statistics and check for crashes.

### Step 3.1: Parse Execution Statistics

```bash
echo "═════════════════════════════════════════════════════════════"
echo "                    Fuzzing Results                          "
echo "═════════════════════════════════════════════════════════════"
echo ""

TOTAL_EXECS=0
TOTAL_CRASHES=0

for target in $TARGETS; do
    LOG="/tmp/ProRT-IP/fuzz-results/${target}.log"

    if [ -f "$LOG" ]; then
        # Extract execution count
        EXECS=$(grep "stat::number_of_executed_units" "$LOG" 2>/dev/null | tail -1 | awk '{print $2}' || echo "0")
        EXECS=${EXECS:-0}

        # Extract coverage stats
        FEATURES=$(grep "stat::new_features" "$LOG" 2>/dev/null | tail -1 | awk '{print $2}' || echo "0")
        CORPUS=$(grep "stat::corpus_size" "$LOG" 2>/dev/null | tail -1 | awk '{print $2}' || echo "0")

        # Check for crashes
        CRASHES=$(find fuzz/artifacts/"$target" -name "crash-*" -o -name "timeout-*" 2>/dev/null | wc -l)
        CRASHES=${CRASHES:-0}

        # Calculate exec/sec
        EXEC_PER_SEC=$((EXECS / 300))

        echo "Target: $target"
        echo "  Executions: $(printf "%'d" "$EXECS") ($(printf "%'d" "$EXEC_PER_SEC")/sec)"
        echo "  Coverage: $FEATURES features, $CORPUS corpus"
        echo "  Crashes: $CRASHES"
        echo ""

        TOTAL_EXECS=$((TOTAL_EXECS + EXECS))
        TOTAL_CRASHES=$((TOTAL_CRASHES + CRASHES))
    else
        echo "Target: $target"
        echo "  ❌ Log file not found"
        echo ""
    fi
done
```

### Step 3.2: Summary Statistics

```bash
echo "═════════════════════════════════════════════════════════════"
echo "                    Summary Statistics                       "
echo "═════════════════════════════════════════════════════════════"
echo ""
echo "Total Targets: $TARGET_COUNT"
echo "Total Executions: $(printf "%'d" "$TOTAL_EXECS")"
echo "Average Executions/Target: $(printf "%'d" $((TOTAL_EXECS / TARGET_COUNT)))"
echo "Total Duration: ${DURATION}s"
echo "Total Crashes: $TOTAL_CRASHES"
echo ""
```

---

## Phase 4: CRASH ANALYSIS

Report any crashes found and provide investigation steps.

### Step 4.1: List Crashes

```bash
if [ "$TOTAL_CRASHES" -gt 0 ]; then
    echo "❌ CRASHES DETECTED"
    echo ""
    echo "Crash artifacts:"
    echo ""

    for target in $TARGETS; do
        TARGET_CRASHES=$(find fuzz/artifacts/"$target" -name "crash-*" -o -name "timeout-*" 2>/dev/null)

        if [ -n "$TARGET_CRASHES" ]; then
            echo "Target: $target"
            echo "$TARGET_CRASHES" | sed 's/^/  • /'
            echo ""
        fi
    done

    echo "═════════════════════════════════════════════════════════════"
    echo "                  Investigation Steps                        "
    echo "═════════════════════════════════════════════════════════════"
    echo ""
    echo "Reproduce crash:"
    echo "  cargo +nightly fuzz run <target> <crash-file>"
    echo ""
    echo "Debug with backtrace:"
    echo "  RUST_BACKTRACE=1 cargo +nightly fuzz run <target> <crash-file>"
    echo ""
    echo "Minimize crash case:"
    echo "  cargo +nightly fuzz cmin <target>"
    echo ""
    echo "⚠️  DO NOT RELEASE WITH UNRESOLVED CRASHES"
    echo ""
else
    echo "✅ NO CRASHES DETECTED"
    echo ""
    echo "All fuzz targets completed successfully with no crashes."
    echo "This indicates good robustness for the tested inputs."
    echo ""
fi
```

---

## Phase 5: GENERATE REPORT

Create detailed fuzzing report.

### Step 5.1: Create Report

```bash
REPORT_DATE=$(date +"%Y-%m-%d %H:%M:%S")

cat > /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md << EOF
# Fuzzing Quick Check Report

**Date:** $REPORT_DATE
**Duration:** ${DURATION}s (~$((DURATION / 60)) minutes)

---

## Summary

| Metric | Value |
|--------|-------|
| **Targets Tested** | $TARGET_COUNT |
| **Total Executions** | $(printf "%'d" "$TOTAL_EXECS") |
| **Avg Exec/Target** | $(printf "%'d" $((TOTAL_EXECS / TARGET_COUNT))) |
| **Crashes Found** | $TOTAL_CRASHES |
| **Status** | $([ "$TOTAL_CRASHES" -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL") |

---

## Detailed Results

EOF

for target in $TARGETS; do
    LOG="/tmp/ProRT-IP/fuzz-results/${target}.log"

    cat >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md << EOF
### Target: $target

EOF

    if [ -f "$LOG" ]; then
        EXECS=$(grep "stat::number_of_executed_units" "$LOG" 2>/dev/null | tail -1 | awk '{print $2}' || echo "0")
        FEATURES=$(grep "stat::new_features" "$LOG" 2>/dev/null | tail -1 | awk '{print $2}' || echo "0")
        CORPUS=$(grep "stat::corpus_size" "$LOG" 2>/dev/null | tail -1 | awk '{print $2}' || echo "0")
        CRASHES=$(find fuzz/artifacts/"$target" -name "crash-*" -o -name "timeout-*" 2>/dev/null | wc -l)

        cat >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md << EOF
- **Executions:** $(printf "%'d" "${EXECS:-0}")
- **Coverage:** ${FEATURES:-0} features, ${CORPUS:-0} corpus
- **Crashes:** ${CRASHES:-0}
- **Status:** $([ "${CRASHES:-0}" -eq 0 ] && echo "✅ PASS" || echo "❌ $CRASHES crashes")

EOF
    else
        echo "- **Status:** ❌ Log not found" >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md
        echo "" >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md
    fi
done

cat >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md << EOF

---

## Crash Analysis

EOF

if [ "$TOTAL_CRASHES" -gt 0 ]; then
    cat >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md << 'EOF'

⚠️ **CRASHES DETECTED** - Immediate action required

### Crash Artifacts

EOF

    for target in $TARGETS; do
        TARGET_CRASHES=$(find fuzz/artifacts/"$target" -name "crash-*" -o -name "timeout-*" 2>/dev/null)

        if [ -n "$TARGET_CRASHES" ]; then
            echo "" >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md
            echo "**$target:**" >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md
            echo "$TARGET_CRASHES" | sed 's/^/- /' >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md
        fi
    done

    cat >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md << 'EOF'

### Investigation Steps

1. **Reproduce crash:**
   ```bash
   cargo +nightly fuzz run <target> <crash-file>
   ```

2. **Debug with backtrace:**
   ```bash
   RUST_BACKTRACE=1 cargo +nightly fuzz run <target> <crash-file>
   ```

3. **Minimize crash case:**
   ```bash
   cargo +nightly fuzz cmin <target>
   ```

4. **Fix root cause**

5. **Re-run fuzzing:**
   ```bash
   /fuzz-check
   ```

EOF
else
    echo "✅ No crashes detected - All targets passed" >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md
    echo "" >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md
fi

cat >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md << 'EOF'

---

## Recommendations

### For Full Validation

This quick check provides initial confidence. For full validation:

1. **Extended fuzzing:** 10+ minutes per target
2. **Continuous fuzzing:** Run overnight or in CI/CD
3. **Corpus growth:** Monitor new inputs discovered
4. **Coverage tracking:** Ensure all code paths tested

### CI/CD Integration

Enable nightly fuzzing:
```bash
# .github/workflows/fuzz.yml already configured
# Runs 10 minutes per target nightly
```

---

## Next Steps

EOF

if [ "$TOTAL_CRASHES" -eq 0 ]; then
    cat >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md << 'EOF'
1. ✅ Quick check passed
2. ⬜ (Optional) Run extended fuzzing overnight
3. ⬜ Continue development with confidence
4. ⬜ Monitor nightly CI/CD fuzzing results

EOF
else
    cat >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md << 'EOF'
1. ❌ Fix crashes immediately (blocking issue)
2. ⬜ Reproduce and debug each crash
3. ⬜ Add regression tests
4. ⬜ Re-run: /fuzz-check
5. ⬜ Verify fix with extended fuzzing

EOF
fi

cat >> /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md << EOF

---

**Generated:** $REPORT_DATE
**Tool:** ProRT-IP Fuzz Check (/fuzz-check)
**Logs:** /tmp/ProRT-IP/fuzz-results/
EOF

echo "✅ Fuzz report generated"
echo ""
```

### Step 5.2: Display Summary

```bash
cat << EOF

╔════════════════════════════════════════════════════════════════╗
║                 Fuzzing Quick Check Complete                   ║
╚════════════════════════════════════════════════════════════════╝

📊 RESULTS

• Targets: $TARGET_COUNT
• Executions: $(printf "%'d" "$TOTAL_EXECS")
• Duration: ~$((DURATION / 60)) minutes
• Crashes: $TOTAL_CRASHES

$([ "$TOTAL_CRASHES" -eq 0 ] && echo "✅ STATUS: PASSED" || echo "❌ STATUS: FAILED")

📁 ARTIFACTS

• Report: /tmp/ProRT-IP/FUZZ-CHECK-REPORT.md
• Logs: /tmp/ProRT-IP/fuzz-results/
• Crashes: fuzz/artifacts/ (if any)

🚀 NEXT STEPS

$(if [ "$TOTAL_CRASHES" -eq 0 ]; then
    echo "1. Review report for detailed statistics"
    echo "2. (Optional) Run extended fuzzing"
    echo "3. Continue development with confidence"
else
    echo "1. Investigate crashes IMMEDIATELY"
    echo "2. Fix root causes"
    echo "3. Re-run: /fuzz-check"
fi)

📖 DOCUMENTATION

• Fuzzing Guide: docs/29-FUZZING-GUIDE.md
• CI/CD Fuzzing: .github/workflows/fuzz.yml

EOF
```

---

## SUCCESS CRITERIA

✅ Fuzz infrastructure verified (fuzz/ directory, nightly toolchain)
✅ Fuzz targets discovered (1+ targets)
✅ All targets executed (5 minutes each)
✅ Statistics extracted (executions, coverage, crashes)
✅ Crash analysis performed (if any crashes)
✅ Report generated with recommendations

---

## RELATED COMMANDS

- `/security-audit` - Comprehensive security check (includes fuzzing status)
- `/pre-release` - Pre-release validation (includes fuzzing check)

---

**Execute fuzzing quick check now.**
