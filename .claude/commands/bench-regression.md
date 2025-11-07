# Benchmark Regression Detection

Detect performance regressions by comparing current code to baseline.

---

## Phase 1: SETUP AND VALIDATION

Verify benchmark infrastructure and baseline.

### Step 1.1: Check Benchmark Directory

```bash
if [ ! -d "benchmarks" ]; then
    echo "❌ Benchmarks directory not found"
    echo ""
    echo "Create: mkdir benchmarks"
    echo "Add benchmarks to: benches/"
    exit 1
fi

echo "✅ Benchmarks directory found"
echo ""
```

### Step 1.2: List Available Benchmarks

```bash
echo "Available benchmarks:"
echo ""

if [ -d "benches" ]; then
    find benches -name "*.rs" | while read -r bench; do
        BENCH_NAME=$(basename "$bench" .rs)
        echo "  • $BENCH_NAME"
    done
else
    echo "  (No benches/ directory found)"
fi

echo ""
```

### Step 1.3: Check for Baseline

```bash
BASELINE_DIR="benchmarks/baselines"

if [ ! -d "$BASELINE_DIR" ]; then
    echo "⚠️  No baseline directory found"
    echo "   Creating: $BASELINE_DIR"
    mkdir -p "$BASELINE_DIR"
fi

# Check for latest baseline
if [ -z "$(ls -A $BASELINE_DIR 2>/dev/null)" ]; then
    echo "⚠️  No baseline benchmarks found"
    echo ""
    read -p "Create baseline from current code? (Y/n): " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        echo "Creating baseline..."
        NEEDS_BASELINE=true
    else
        echo "Aborted - baseline required for regression detection"
        exit 1
    fi
else
    LATEST_BASELINE=$(ls -t "$BASELINE_DIR" | head -1)
    echo "✅ Found baseline: $LATEST_BASELINE"
    NEEDS_BASELINE=false
fi

echo ""
```

---

## Phase 2: RUN BENCHMARKS

Execute benchmarks for current code and baseline.

### Step 2.1: Create Baseline if Needed

```bash
if [ "$NEEDS_BASELINE" = true ]; then
    BASELINE_NAME="baseline-$(date +%Y%m%d-%H%M%S)"
    echo "Creating baseline: $BASELINE_NAME"
    echo ""

    cargo bench --no-fail-fast 2>&1 | tee "$BASELINE_DIR/$BASELINE_NAME.txt"

    if [ $? -eq 0 ]; then
        echo ""
        echo "✅ Baseline created: $BASELINE_DIR/$BASELINE_NAME.txt"
    else
        echo ""
        echo "❌ Baseline creation failed"
        exit 1
    fi

    echo ""
    echo "Baseline created. Re-run this command after making changes to detect regressions."
    exit 0
fi
```

### Step 2.2: Run Current Benchmarks

```bash
echo "Running current benchmarks..."
echo "This may take 5-15 minutes depending on benchmark count..."
echo ""

CURRENT_BENCH="/tmp/ProRT-IP/bench-current-$(date +%Y%m%d-%H%M%S).txt"

cargo bench --no-fail-fast 2>&1 | tee "$CURRENT_BENCH"

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ Benchmark run failed"
    exit 1
fi

echo ""
echo "✅ Current benchmarks complete"
echo ""
```

---

## Phase 3: COMPARE RESULTS

Compare current results against baseline.

### Step 3.1: Parse Benchmark Results

```bash
echo "Parsing benchmark results..."
echo ""

# Function to extract benchmark times
parse_bench_time() {
    local file=$1
    local bench_name=$2

    grep -A 1 "^test $bench_name" "$file" 2>/dev/null | \
        grep "time:" | \
        grep -oP '\d+\.?\d* [num]s' | \
        awk '{
            val=$1;
            unit=$2;
            if (unit == "ns") print val;
            else if (unit == "us") print val * 1000;
            else if (unit == "ms") print val * 1000000;
            else if (unit == "s") print val * 1000000000;
        }'
}

# Get list of all benchmark names
BENCH_NAMES=$(grep "^test " "$CURRENT_BENCH" | awk '{print $2}' | sort -u)

if [ -z "$BENCH_NAMES" ]; then
    echo "❌ No benchmark results found in output"
    exit 1
fi

echo "Found $(echo "$BENCH_NAMES" | wc -l) benchmarks"
echo ""
```

### Step 3.2: Calculate Regressions

```bash
echo "Comparing against baseline: $LATEST_BASELINE"
echo ""

REGRESSIONS=0
IMPROVEMENTS=0
UNCHANGED=0
THRESHOLD=5  # 5% threshold for significance

mkdir -p /tmp/ProRT-IP/bench-results

cat > /tmp/ProRT-IP/bench-results/comparison.csv << 'EOF'
Benchmark,Baseline (ns),Current (ns),Change (%),Status
EOF

echo "$BENCH_NAMES" | while read -r bench; do
    BASELINE_TIME=$(parse_bench_time "$BASELINE_DIR/$LATEST_BASELINE" "$bench")
    CURRENT_TIME=$(parse_bench_time "$CURRENT_BENCH" "$bench")

    if [ -n "$BASELINE_TIME" ] && [ -n "$CURRENT_TIME" ]; then
        # Calculate percentage change
        CHANGE=$(awk "BEGIN {printf \"%.2f\", (($CURRENT_TIME - $BASELINE_TIME) / $BASELINE_TIME) * 100}")

        # Determine status
        if (( $(echo "$CHANGE > $THRESHOLD" | bc -l) )); then
            STATUS="REGRESSION"
            ((REGRESSIONS++))
        elif (( $(echo "$CHANGE < -$THRESHOLD" | bc -l) )); then
            STATUS="IMPROVEMENT"
            ((IMPROVEMENTS++))
        else
            STATUS="UNCHANGED"
            ((UNCHANGED++))
        fi

        echo "$bench,$BASELINE_TIME,$CURRENT_TIME,$CHANGE,$STATUS" >> /tmp/ProRT-IP/bench-results/comparison.csv
    fi
done

echo "✅ Comparison complete"
echo ""
```

### Step 3.3: Display Results

```bash
echo "═══════════════════════════════════════════════════════════════"
echo "              Benchmark Regression Analysis                    "
echo "═══════════════════════════════════════════════════════════════"
echo ""

echo "Summary:"
echo "  🔴 Regressions: $REGRESSIONS"
echo "  🟢 Improvements: $IMPROVEMENTS"
echo "  ⚪ Unchanged: $UNCHANGED"
echo ""

if [ "$REGRESSIONS" -gt 0 ]; then
    echo "🔴 REGRESSIONS DETECTED (>$THRESHOLD% slower)"
    echo ""
    awk -F, '$5 == "REGRESSION" {printf "  %-40s: %+.2f%%\n", $1, $4}' \
        /tmp/ProRT-IP/bench-results/comparison.csv | \
        sort -t: -k2 -rn
    echo ""
fi

if [ "$IMPROVEMENTS" -gt 0 ]; then
    echo "🟢 IMPROVEMENTS DETECTED (>$THRESHOLD% faster)"
    echo ""
    awk -F, '$5 == "IMPROVEMENT" {printf "  %-40s: %+.2f%%\n", $1, $4}' \
        /tmp/ProRT-IP/bench-results/comparison.csv | \
        sort -t: -k2 -n
    echo ""
fi

echo "Full results: /tmp/ProRT-IP/bench-results/comparison.csv"
echo ""
```

---

## Phase 4: GENERATE REPORT

Create detailed regression report.

### Step 4.1: Create Report

```bash
REPORT_DATE=$(date +"%Y-%m-%d %H:%M:%S")

cat > /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md << EOF
# Benchmark Regression Report

**Date:** $REPORT_DATE
**Baseline:** $LATEST_BASELINE
**Threshold:** ±$THRESHOLD%

---

## Summary

| Category | Count |
|----------|-------|
| 🔴 Regressions | $REGRESSIONS |
| 🟢 Improvements | $IMPROVEMENTS |
| ⚪ Unchanged | $UNCHANGED |
| **Total** | $((REGRESSIONS + IMPROVEMENTS + UNCHANGED)) |

**Status:** $(
    if [ "$REGRESSIONS" -eq 0 ]; then
        echo "✅ NO REGRESSIONS"
    elif [ "$REGRESSIONS" -le 2 ]; then
        echo "⚠️  MINOR REGRESSIONS ($REGRESSIONS)"
    else
        echo "❌ SIGNIFICANT REGRESSIONS ($REGRESSIONS)"
    fi
)

---

## Regressions (>$THRESHOLD% slower)

EOF

if [ "$REGRESSIONS" -gt 0 ]; then
    echo "| Benchmark | Baseline | Current | Change |" >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md
    echo "|-----------|----------|---------|--------|" >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md

    awk -F, '$5 == "REGRESSION" {printf "| %s | %.0f ns | %.0f ns | **+%.2f%%** |\n", $1, $2, $3, $4}' \
        /tmp/ProRT-IP/bench-results/comparison.csv | \
        sort -t'|' -k5 -rn >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md
else
    echo "✅ No regressions detected" >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md
fi

cat >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md << EOF

---

## Improvements (>$THRESHOLD% faster)

EOF

if [ "$IMPROVEMENTS" -gt 0 ]; then
    echo "| Benchmark | Baseline | Current | Change |" >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md
    echo "|-----------|----------|---------|--------|" >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md

    awk -F, '$5 == "IMPROVEMENT" {printf "| %s | %.0f ns | %.0f ns | **%.2f%%** |\n", $1, $2, $3, $4}' \
        /tmp/ProRT-IP/bench-results/comparison.csv | \
        sort -t'|' -k5 -n >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md
else
    echo "No significant improvements" >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md
fi

cat >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md << 'EOF'

---

## Recommendations

EOF

if [ "$REGRESSIONS" -gt 0 ]; then
    cat >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md << 'EOF'

### Investigate Regressions

1. **Profile regressed benchmarks:**
   ```bash
   cargo bench --bench <name> -- --profile-time=5
   ```

2. **Use flamegraph for visualization:**
   ```bash
   cargo flamegraph --bench <name>
   ```

3. **Compare algorithm complexity:**
   - Review recent changes to regressed functions
   - Check for additional allocations
   - Verify data structure usage

4. **Optimize or accept:**
   - If regression is acceptable (e.g., for new feature), document it
   - If unacceptable, optimize until below threshold

EOF
fi

cat >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md << 'EOF'

### Update Baseline

After fixing regressions or accepting changes:

```bash
# Create new baseline
BASELINE_NAME="baseline-$(date +%Y%m%d-%H%M%S)"
cargo bench > "benchmarks/baselines/$BASELINE_NAME.txt"

# Or move current to baseline
mv /tmp/ProRT-IP/bench-current-*.txt "benchmarks/baselines/$BASELINE_NAME.txt"
```

---

## Detailed Results

Full comparison data: \`/tmp/ProRT-IP/bench-results/comparison.csv\`

```csv
EOF

cat /tmp/ProRT-IP/bench-results/comparison.csv >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md

cat >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md << 'EOF'
```

---

## Next Steps

EOF

if [ "$REGRESSIONS" -eq 0 ]; then
    cat >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md << 'EOF'
1. ✅ No regressions - performance maintained
2. ⬜ Consider updating baseline if improvements are significant
3. ⬜ Continue development

EOF
else
    cat >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md << EOF
1. ❌ Investigate $REGRESSIONS regression(s)
2. ⬜ Profile affected benchmarks
3. ⬜ Optimize or document acceptance
4. ⬜ Re-run: /bench-regression
5. ⬜ Update baseline when resolved

EOF
fi

cat >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md << 'EOF'

---

## Benchmark Management

### Create Baseline
```bash
./bench-regression  # Will prompt to create baseline if none exists
```

### Archive Old Baselines
```bash
mkdir -p benchmarks/baselines/archive
mv benchmarks/baselines/baseline-*.txt benchmarks/baselines/archive/
```

### Compare Specific Versions
```bash
cargo bench --baseline <baseline-name>
```

---

EOF

cat >> /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md << EOF
**Generated:** $REPORT_DATE
**Tool:** ProRT-IP Benchmark Regression (/bench-regression)
EOF

echo "✅ Regression report generated"
echo ""
```

### Step 4.2: Display Summary

```bash
cat << EOF

╔════════════════════════════════════════════════════════════════╗
║            Benchmark Regression Detection Complete             ║
╚════════════════════════════════════════════════════════════════╝

📊 RESULTS

• Total Benchmarks: $((REGRESSIONS + IMPROVEMENTS + UNCHANGED))
• Regressions: $REGRESSIONS
• Improvements: $IMPROVEMENTS
• Unchanged: $UNCHANGED

$(if [ "$REGRESSIONS" -eq 0 ]; then
    echo "✅ STATUS: NO REGRESSIONS"
else
    echo "❌ STATUS: $REGRESSIONS REGRESSIONS DETECTED"
fi)

📁 ARTIFACTS

• Report: /tmp/ProRT-IP/BENCH-REGRESSION-REPORT.md
• Comparison: /tmp/ProRT-IP/bench-results/comparison.csv
• Current run: $CURRENT_BENCH
• Baseline: $BASELINE_DIR/$LATEST_BASELINE

🚀 NEXT STEPS

$(if [ "$REGRESSIONS" -eq 0 ]; then
    echo "1. Review improvements (if any)"
    echo "2. Consider updating baseline"
    echo "3. Continue development"
else
    echo "1. Review regression report"
    echo "2. Profile affected benchmarks"
    echo "3. Optimize or document acceptance"
    echo "4. Re-run: /bench-regression"
fi)

📖 DOCUMENTATION

• Benchmarking Guide: docs/12-BENCHMARKING-GUIDE.md
• Performance Guide: docs/07-PERFORMANCE.md

EOF
```

---

## SUCCESS CRITERIA

✅ Benchmark infrastructure validated
✅ Baseline checked or created
✅ Current benchmarks executed
✅ Results compared against baseline
✅ Regressions/improvements identified
✅ Detailed report generated

---

## RELATED COMMANDS

- `/bench-compare` - Compare specific benchmark runs
- `/perf-profile` - Profile specific code sections
- `/pre-release` - Includes performance validation

---

**Execute benchmark regression detection now.**
