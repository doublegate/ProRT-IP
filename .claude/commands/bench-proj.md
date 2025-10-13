Run comprehensive project-wide benchmarking suite: $*

---

## COMPREHENSIVE PROJECT BENCHMARKING

**Purpose:** Execute complete benchmarking suite following Phase 4 methodology (hyperfine, perf, valgrind, strace, flamegraphs)

**Usage:** `/bench-proj [suite] [target]`
- **suite:** Benchmark suite to run: `quick` | `standard` | `full` (default: `standard`)
- **target:** Target IP/host for scanning benchmarks (default: `127.0.0.1`)

**Examples:**
```bash
/bench-proj                    # Standard suite on localhost (~15-20 min)
/bench-proj quick              # Quick suite (~5-10 min)
/bench-proj full               # Full suite (~40-60 min)
/bench-proj standard 192.168.1.1  # Standard suite on custom target
```

---

## BENCHMARK SUITES

### Quick Suite (~5-10 minutes)
**Purpose:** Fast performance snapshot for rapid iteration
**Tests:**
- Common ports (100): hyperfine statistical benchmark
- 1K ports: hyperfine + basic timing
- Comparison with latest benchmark

**Use When:**
- Quick validation after code changes
- CI/CD integration
- Daily development workflow

### Standard Suite (~15-20 minutes) ⭐ **DEFAULT**
**Purpose:** Balanced coverage for phase boundaries and releases
**Tests:**
- Common ports (100): hyperfine + timing
- 1K ports: hyperfine + perf + valgrind + strace
- 10K ports: hyperfine + perf + flamegraph
- Database mode (10K): hyperfine
- Timing templates (T0/T3/T5): hyperfine
- Comprehensive comparison analysis

**Use When:**
- Phase completion verification
- Release candidate validation
- Performance optimization tracking
- Monthly progress review

### Full Suite (~40-60 minutes)
**Purpose:** Exhaustive analysis for major releases and research
**Tests:**
- All Standard suite tests
- 65K ports (full range): hyperfine + perf + flamegraph
- All timing templates (T0-T5): hyperfine
- Extended valgrind profiling (10K + 65K)
- Extended strace analysis
- Memory leak detection
- Hardware counter analysis

**Use When:**
- Major version releases (v0.X.0, v1.0.0)
- Performance regression investigation
- Competitive analysis documentation
- Research and optimization deep-dives

---

## PHASE 1: TOOL VERIFICATION

### Required Tools (Benchmark will FAIL if missing)
```bash
# Check hyperfine (REQUIRED)
if ! command -v hyperfine &> /dev/null; then
  echo "ERROR: hyperfine not installed (REQUIRED)"
  echo "Install: cargo install hyperfine"
  exit 1
fi

# Check cargo (should always be present)
if ! command -v cargo &> /dev/null; then
  echo "ERROR: cargo not found (Rust toolchain required)"
  exit 1
fi
```

### Optional Tools (Will skip gracefully if missing)
```bash
# Check perf (Linux performance profiling)
if ! command -v perf &> /dev/null; then
  echo "WARNING: perf not installed (CPU profiling will be skipped)"
  echo "Install: sudo pacman -S perf (Arch) or sudo apt install linux-tools-generic (Ubuntu)"
  SKIP_PERF=true
fi

# Check flamegraph (Visual CPU profiling)
if ! command -v flamegraph &> /dev/null; then
  echo "WARNING: flamegraph not installed (flamegraph generation will be skipped)"
  echo "Install: cargo install flamegraph"
  SKIP_FLAMEGRAPH=true
fi

# Check valgrind (Memory profiling)
if ! command -v valgrind &> /dev/null; then
  echo "WARNING: valgrind not installed (memory profiling will be skipped)"
  echo "Install: sudo pacman -S valgrind (Arch) or sudo apt install valgrind (Ubuntu)"
  SKIP_VALGRIND=true
fi

# Check strace (System call tracing)
if ! command -v strace &> /dev/null; then
  echo "WARNING: strace not installed (syscall tracing will be skipped)"
  SKIP_STRACE=true
fi
```

---

## PHASE 2: ENVIRONMENT SETUP

### Build Verification
```bash
echo "Verifying release build..."
cd /home/parobek/Code/ProRT-IP

# Check if build is current
if [ ! -f target/release/prtip ]; then
  echo "Release binary not found. Building..."
  cargo build --release || exit 1
fi

# Optional: Rebuild for fresh benchmarks
read -p "Rebuild release binary before benchmarking? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
  cargo build --release || exit 1
fi
```

### Directory Structure Creation
```bash
# Create timestamped benchmark directory
BENCH_DATE=$(date +%Y-%m-%d)
BENCH_TIME=$(date +%H-%M-%S)
BENCH_DIR="benchmarks/03-Benchmark-${BENCH_DATE}-${BENCH_TIME}"

mkdir -p "$BENCH_DIR"/{hyperfine,perf,flamegraphs,valgrind,strace,timing}

echo "Benchmark output directory: $BENCH_DIR"
```

### Environment Documentation
```bash
cat > "$BENCH_DIR/00-Environment.md" <<EOF
# Benchmark Environment

**Date:** $(date)
**Version:** $(git describe --tags --always)
**Commit:** $(git rev-parse HEAD)
**Branch:** $(git branch --show-current)

## System Information

**Kernel:** $(uname -r)
**CPU:** $(lscpu | grep "Model name" | sed 's/Model name: *//')
**CPU Cores:** $(nproc)
**Memory:** $(free -h | grep Mem | awk '{print $2}')
**Disk:** $(df -h /home | tail -1 | awk '{print $4}') available

## Software Versions

**rustc:** $(rustc --version)
**cargo:** $(cargo --version)
**hyperfine:** $(hyperfine --version 2>&1 || echo "N/A")
**perf:** $(perf --version 2>&1 | head -1 || echo "N/A")
**valgrind:** $(valgrind --version 2>&1 || echo "N/A")
**flamegraph:** $(flamegraph --help 2>&1 | grep -i usage | head -1 || echo "N/A")

## Build Configuration

$(grep -A10 "\[profile.release\]" Cargo.toml || echo "Using default release profile")

**Binary Size:** $(ls -lh target/release/prtip | awk '{print $5}')
**Build Date:** $(stat -c %y target/release/prtip | cut -d' ' -f1)

## Benchmark Suite

**Suite Type:** ${SUITE:-standard}
**Target:** ${TARGET:-127.0.0.1}

## Previous Benchmarks

$(ls -lt benchmarks/ | grep -E "^d" | head -3)
EOF
```

---

## PHASE 3: BENCHMARK EXECUTION

### 3.1 Hyperfine Statistical Benchmarks (ALL SUITES)

**Common Ports (100) - Fast Scan Baseline**
```bash
echo "Running: Common ports (100) benchmark..."
hyperfine --warmup 3 --runs 10 \
  --export-json "$BENCH_DIR/hyperfine/01-Common-Ports.json" \
  --export-markdown "$BENCH_DIR/hyperfine/01-Common-Ports.md" \
  './target/release/prtip -F ${TARGET:-127.0.0.1}'

# Extract mean time for quick summary
COMMON_MEAN=$(jq -r '.results[0].mean' "$BENCH_DIR/hyperfine/01-Common-Ports.json")
echo "✅ Common ports: ${COMMON_MEAN}s"
```

**1K Ports - Standard Benchmark**
```bash
echo "Running: 1K ports benchmark..."
hyperfine --warmup 3 --runs 10 \
  --export-json "$BENCH_DIR/hyperfine/02-1K-Ports.json" \
  --export-markdown "$BENCH_DIR/hyperfine/02-1K-Ports.md" \
  './target/release/prtip -p 1-1000 ${TARGET:-127.0.0.1}'

K1_MEAN=$(jq -r '.results[0].mean' "$BENCH_DIR/hyperfine/02-1K-Ports.json")
echo "✅ 1K ports: ${K1_MEAN}s"
```

**10K Ports - Large Scan Benchmark** (Standard + Full only)
```bash
if [ "$SUITE" != "quick" ]; then
  echo "Running: 10K ports benchmark..."
  hyperfine --warmup 3 --runs 10 \
    --export-json "$BENCH_DIR/hyperfine/03-10K-Ports.json" \
    --export-markdown "$BENCH_DIR/hyperfine/03-10K-Ports.md" \
    './target/release/prtip -p 1-10000 ${TARGET:-127.0.0.1}'

  K10_MEAN=$(jq -r '.results[0].mean' "$BENCH_DIR/hyperfine/03-10K-Ports.json")
  echo "✅ 10K ports: ${K10_MEAN}s"
fi
```

**65K Ports - Full Range** (Full suite only)
```bash
if [ "$SUITE" = "full" ]; then
  echo "Running: 65K ports (full range) benchmark..."
  hyperfine --warmup 2 --runs 5 \
    --export-json "$BENCH_DIR/hyperfine/04-Full-Range.json" \
    --export-markdown "$BENCH_DIR/hyperfine/04-Full-Range.md" \
    './target/release/prtip -p- ${TARGET:-127.0.0.1}'

  K65_MEAN=$(jq -r '.results[0].mean' "$BENCH_DIR/hyperfine/04-Full-Range.json")
  echo "✅ 65K ports: ${K65_MEAN}s"
fi
```

**Database Mode - Storage Overhead** (Standard + Full only)
```bash
if [ "$SUITE" != "quick" ]; then
  echo "Running: 10K ports with database..."
  hyperfine --warmup 3 --runs 10 \
    --export-json "$BENCH_DIR/hyperfine/05-With-Database.json" \
    --export-markdown "$BENCH_DIR/hyperfine/05-With-Database.md" \
    --cleanup 'rm -f /tmp/prtip-bench.db*' \
    './target/release/prtip -p 1-10000 --database /tmp/prtip-bench.db ${TARGET:-127.0.0.1}'

  DB_MEAN=$(jq -r '.results[0].mean' "$BENCH_DIR/hyperfine/05-With-Database.json")
  echo "✅ 10K + DB: ${DB_MEAN}s"
fi
```

**Timing Templates** (Standard: T0/T3/T5, Full: T0-T5)
```bash
if [ "$SUITE" != "quick" ]; then
  if [ "$SUITE" = "full" ]; then
    TIMING_TESTS="0 1 2 3 4 5"
  else
    TIMING_TESTS="0 3 5"
  fi

  for T in $TIMING_TESTS; do
    echo "Running: Timing template T${T}..."
    hyperfine --warmup 2 --runs 5 \
      --export-json "$BENCH_DIR/hyperfine/06-Timing-T${T}.json" \
      --export-markdown "$BENCH_DIR/hyperfine/06-Timing-T${T}.md" \
      "./target/release/prtip -T${T} -p 1-1000 ${TARGET:-127.0.0.1}"

    T_MEAN=$(jq -r '.results[0].mean' "$BENCH_DIR/hyperfine/06-Timing-T${T}.json")
    echo "✅ Timing T${T}: ${T_MEAN}s"
  done
fi
```

### 3.2 Perf CPU Profiling (Standard + Full only, if available)

```bash
if [ "$SUITE" != "quick" ] && [ "$SKIP_PERF" != "true" ]; then
  echo "Running: Perf CPU profiling..."

  # 1K ports profile
  sudo perf record -F 99 -g \
    -o "$BENCH_DIR/perf/01-perf-1K.data" \
    -- ./target/release/prtip -p 1-1000 ${TARGET:-127.0.0.1}

  sudo perf report -i "$BENCH_DIR/perf/01-perf-1K.data" \
    > "$BENCH_DIR/perf/01-CPU-Profile-1K.txt"

  echo "✅ Perf profile (1K): $BENCH_DIR/perf/01-CPU-Profile-1K.txt"

  # 10K ports profile
  sudo perf record -F 99 -g \
    -o "$BENCH_DIR/perf/02-perf-10K.data" \
    -- ./target/release/prtip -p 1-10000 ${TARGET:-127.0.0.1}

  sudo perf report -i "$BENCH_DIR/perf/02-perf-10K.data" \
    > "$BENCH_DIR/perf/02-CPU-Profile-10K.txt"

  echo "✅ Perf profile (10K): $BENCH_DIR/perf/02-CPU-Profile-10K.txt"

  # Fix permissions (perf creates as root)
  sudo chown $USER:$USER "$BENCH_DIR/perf/"*.data
fi
```

### 3.3 FlameGraph Generation (Standard + Full only, if available)

```bash
if [ "$SUITE" != "quick" ] && [ "$SKIP_FLAMEGRAPH" != "true" ]; then
  echo "Running: FlameGraph generation..."

  # 1K ports flamegraph
  sudo flamegraph -o "$BENCH_DIR/flamegraphs/01-CPU-Flamegraph-1K.svg" \
    -- ./target/release/prtip -p 1-1000 ${TARGET:-127.0.0.1}

  # 10K ports flamegraph
  sudo flamegraph -o "$BENCH_DIR/flamegraphs/02-CPU-Flamegraph-10K.svg" \
    -- ./target/release/prtip -p 1-10000 ${TARGET:-127.0.0.1}

  # Fix permissions
  sudo chown $USER:$USER "$BENCH_DIR/flamegraphs/"*.svg

  echo "✅ FlameGraphs generated (open in browser for interactive visualization)"
fi
```

### 3.4 Valgrind Memory Profiling (Standard + Full only, if available)

```bash
if [ "$SUITE" != "quick" ] && [ "$SKIP_VALGRIND" != "true" ]; then
  echo "Running: Valgrind memory profiling (this takes 10-20x longer)..."

  # 1K ports memory profile
  valgrind --tool=massif \
    --massif-out-file="$BENCH_DIR/valgrind/01-Massif-1K.out" \
    ./target/release/prtip -p 1-1000 ${TARGET:-127.0.0.1}

  ms_print "$BENCH_DIR/valgrind/01-Massif-1K.out" \
    > "$BENCH_DIR/valgrind/01-Massif-1K-Report.txt"

  # 10K ports memory profile
  valgrind --tool=massif \
    --massif-out-file="$BENCH_DIR/valgrind/02-Massif-10K.out" \
    ./target/release/prtip -p 1-10000 ${TARGET:-127.0.0.1}

  ms_print "$BENCH_DIR/valgrind/02-Massif-10K.out" \
    > "$BENCH_DIR/valgrind/02-Massif-10K-Report.txt"

  echo "✅ Memory profiling complete"
fi
```

### 3.5 System Call Tracing (Standard + Full only, if available)

```bash
if [ "$SUITE" != "quick" ] && [ "$SKIP_STRACE" != "true" ]; then
  echo "Running: System call tracing..."

  # 1K ports syscall trace
  strace -c -o "$BENCH_DIR/strace/01-Syscalls-1K.txt" \
    ./target/release/prtip -p 1-1000 ${TARGET:-127.0.0.1} 2>&1

  # 10K ports syscall trace
  strace -c -o "$BENCH_DIR/strace/02-Syscalls-10K.txt" \
    ./target/release/prtip -p 1-10000 ${TARGET:-127.0.0.1} 2>&1

  echo "✅ System call tracing complete"
fi
```

---

## PHASE 4: ANALYSIS & COMPARISON

### Find Previous Benchmark
```bash
PREV_BENCH=$(find benchmarks/ -maxdepth 1 -type d -name "*Phase4*" -o -name "*Benchmark*" | grep -v "$BENCH_DIR" | sort -r | head -1)

if [ -n "$PREV_BENCH" ]; then
  echo "Comparing with: $PREV_BENCH"
else
  echo "No previous benchmark found for comparison"
fi
```

### Extract Metrics and Compare
```bash
cat > "$BENCH_DIR/10-Comparison-Analysis.md" <<EOF
# Benchmark Comparison Analysis

**Date:** $(date)
**Current:** $BENCH_DIR
**Previous:** ${PREV_BENCH:-N/A}

## Performance Comparison

| Test | Previous | Current | Delta | % Change | Status |
|------|----------|---------|-------|----------|--------|
EOF

# Compare each test if previous benchmark exists
if [ -n "$PREV_BENCH" ] && [ -f "$PREV_BENCH/hyperfine/02-1K-Ports.json" ]; then
  # Extract and compare 1K ports
  PREV_1K=$(jq -r '.results[0].mean' "$PREV_BENCH/hyperfine/02-1K-Ports.json" 2>/dev/null || echo "N/A")
  CURR_1K=$(jq -r '.results[0].mean' "$BENCH_DIR/hyperfine/02-1K-Ports.json")

  if [ "$PREV_1K" != "N/A" ]; then
    DELTA=$(echo "$CURR_1K - $PREV_1K" | bc -l)
    PERCENT=$(echo "scale=1; ($DELTA / $PREV_1K) * 100" | bc -l)
    STATUS=$([ $(echo "$PERCENT < 0" | bc -l) -eq 1 ] && echo "✅" || echo "⚠️")

    echo "| 1K ports | ${PREV_1K}s | ${CURR_1K}s | ${DELTA}s | ${PERCENT}% | $STATUS |" >> "$BENCH_DIR/10-Comparison-Analysis.md"
  fi
fi

# Add analysis sections
cat >> "$BENCH_DIR/10-Comparison-Analysis.md" <<EOF

## Analysis

### Performance Trends
[Analyze improvement/regression patterns]

### Key Findings
[Highlight significant changes]

### Recommendations
[Suggest optimizations or investigations]
EOF
```

---

## PHASE 5: DOCUMENTATION

### Create README
```bash
cat > "$BENCH_DIR/README.md" <<EOF
# Benchmark Suite: $(date +%Y-%m-%d)

**Suite Type:** ${SUITE:-standard}
**Target:** ${TARGET:-127.0.0.1}
**Version:** $(git describe --tags --always)

## Quick Summary

$([ -n "$COMMON_MEAN" ] && echo "- Common ports (100): ${COMMON_MEAN}s")
$([ -n "$K1_MEAN" ] && echo "- 1K ports: ${K1_MEAN}s")
$([ -n "$K10_MEAN" ] && echo "- 10K ports: ${K10_MEAN}s")
$([ -n "$K65_MEAN" ] && echo "- 65K ports: ${K65_MEAN}s")
$([ -n "$DB_MEAN" ] && echo "- 10K + DB: ${DB_MEAN}s")

## Files Generated

$(find "$BENCH_DIR" -type f | wc -l) files total

- **hyperfine/**: Statistical benchmarks ($(ls "$BENCH_DIR/hyperfine" | wc -l) files)
$([ "$SKIP_PERF" != "true" ] && echo "- **perf/**: CPU profiling ($(ls "$BENCH_DIR/perf" 2>/dev/null | wc -l) files)" || true)
$([ "$SKIP_FLAMEGRAPH" != "true" ] && echo "- **flamegraphs/**: Interactive visualizations ($(ls "$BENCH_DIR/flamegraphs" 2>/dev/null | wc -l) files)" || true)
$([ "$SKIP_VALGRIND" != "true" ] && echo "- **valgrind/**: Memory profiling ($(ls "$BENCH_DIR/valgrind" 2>/dev/null | wc -l) files)" || true)
$([ "$SKIP_STRACE" != "true" ] && echo "- **strace/**: System call traces ($(ls "$BENCH_DIR/strace" 2>/dev/null | wc -l) files)" || true)

## Analysis Documents

- [00-Environment.md](00-Environment.md) - System and build information
- [10-Comparison-Analysis.md](10-Comparison-Analysis.md) - Performance comparison

## Next Steps

1. Review comparison analysis for regressions
2. Check flamegraphs for CPU hotspots (if available)
3. Validate memory usage patterns (if available)
4. Update performance documentation if significant changes

## See Also

- Previous benchmarks: benchmarks/
- Performance docs: docs/07-PERFORMANCE.md
- Benchmarking guide: docs/12-BENCHMARKING-GUIDE.md
EOF
```

---

## PHASE 6: FINAL SUMMARY

```bash
echo ""
echo "=========================================="
echo "Benchmark Suite Complete!"
echo "=========================================="
echo ""
echo "📊 BENCHMARK RESULTS:"
echo ""
echo "Suite: ${SUITE:-standard}"
echo "Target: ${TARGET:-127.0.0.1}"
echo "Output: $BENCH_DIR"
echo ""
echo "📈 PERFORMANCE METRICS:"
$([ -n "$COMMON_MEAN" ] && echo "  Common ports: ${COMMON_MEAN}s")
$([ -n "$K1_MEAN" ] && echo "  1K ports: ${K1_MEAN}s")
$([ -n "$K10_MEAN" ] && echo "  10K ports: ${K10_MEAN}s")
$([ -n "$K65_MEAN" ] && echo "  65K ports: ${K65_MEAN}s")
$([ -n "$DB_MEAN" ] && echo "  10K + DB: ${DB_MEAN}s")
echo ""
echo "📁 FILES GENERATED:"
echo "  Total: $(find "$BENCH_DIR" -type f | wc -l) files"
echo "  Size: $(du -sh "$BENCH_DIR" | cut -f1)"
echo ""
echo "📖 KEY DOCUMENTS:"
echo "  - $BENCH_DIR/README.md"
echo "  - $BENCH_DIR/00-Environment.md"
echo "  - $BENCH_DIR/10-Comparison-Analysis.md"
echo ""
$([ "$SKIP_FLAMEGRAPH" != "true" ] && echo "🔥 FLAMEGRAPHS (open in browser):" && ls "$BENCH_DIR/flamegraphs/"*.svg 2>/dev/null | sed 's/^/  - /' || true)
echo ""
echo "✅ Benchmark suite complete!"
echo ""
```

---

## INTEGRATION WITH EXISTING COMMANDS

### Performance Workflow Integration

**Complete Performance Analysis:**
```bash
# 1. Run comprehensive benchmark
/bench-proj standard

# 2. Deep dive into CPU hotspots
/perf-profile ./target/release/prtip -p 1-10000 127.0.0.1

# 3. Compare with specific baseline
/bench-compare v0.3.0 v0.3.5

# 4. Document findings
/doc-update performance "Observed X% improvement in Y scenario"
```

**Optimization Cycle:**
```bash
# Baseline
/bench-proj quick

# Make changes
# ... edit code ...

# Validate
/rust-check
/test-quick performance

# Re-benchmark
/bench-proj quick

# Compare results in generated comparison analysis
```

### Related Commands

- **`/perf-profile`** - Deep CPU profiling with flamegraphs (single command focus)
- **`/bench-compare`** - Git ref comparison (baseline vs current)
- **`/rust-check`** - Pre-benchmark validation (ensure clean build)
- **`/doc-update`** - Document performance findings
- **`/sprint-complete`** - Include benchmark results in sprint summary

---

## BEST PRACTICES

### When to Benchmark

**Always:**
- Phase completion (Phase 4, Phase 5, etc.)
- Major version releases (v0.X.0, v1.0.0)
- Performance optimization sprints

**Often:**
- Minor version releases (v0.3.X)
- After significant feature additions
- Before/after major refactors

**Sometimes:**
- During feature development (use `quick` suite)
- Investigating performance issues
- Competitive analysis

### Benchmark Hygiene

1. **Clean System:** Close background applications, disable CPU scaling
2. **Consistent Target:** Use localhost (127.0.0.1) for reproducibility
3. **Warm System:** First benchmark run may be slower (hyperfine handles this with warmup)
4. **Document Context:** Note any system changes, code changes, or anomalies
5. **Version Control:** Commit code before benchmarking for accurate baseline

### Interpreting Results

**Hyperfine Results:**
- **Mean:** Primary metric (average across runs)
- **Stddev:** Lower is better (<5% excellent, >20% investigate)
- **Min/Max:** Check range (2x difference? investigate variance)

**Perf Results:**
- **Top Functions:** >5% CPU time? optimization candidate
- **Syscalls:** >50% time in syscalls? I/O bound

**Valgrind Results:**
- **Peak Memory:** Linear scaling? (ports × memory_per_port)
- **Allocations:** Frequent small allocations? consider pooling

**FlameGraphs:**
- **Wide Boxes:** Hot functions (optimization targets)
- **Tall Stacks:** Deep call chains (inlining candidates)
- **Unexpected Functions:** Debug overhead, lock contention

---

## TROUBLESHOOTING

### High Variance (>20% CoV)

**Causes:**
- Background processes competing for CPU
- CPU frequency scaling
- Network latency variance (use localhost)
- Timing-dependent bugs

**Solutions:**
```bash
# Disable CPU scaling
sudo cpupower frequency-set -g performance

# Increase warmup runs
hyperfine --warmup 5 --runs 20 ...

# Run on isolated core
taskset -c 0 ./target/release/prtip ...
```

### Regression Detection

**If benchmarks show slowdown:**

1. Compare flamegraphs (visual diff)
2. Run git bisect to find culprit commit
3. Check for new dependencies or feature flags
4. Verify release build optimizations enabled
5. Test on multiple machines (hardware-specific?)

### Missing Tools

**If optional tools missing:**
- Quick suite: Always works (only requires hyperfine + cargo)
- Standard suite: Gracefully skips unavailable tools
- Full suite: May have reduced coverage but won't fail

---

## OUTPUT STRUCTURE

```
benchmarks/03-Benchmark-YYYY-MM-DD-HH-MM-SS/
├── 00-Environment.md              # System/build/config info
├── 10-Comparison-Analysis.md      # Performance comparison
├── README.md                      # Executive summary
├── hyperfine/                     # Statistical benchmarks
│   ├── 01-Common-Ports.json       # Hyperfine JSON output
│   ├── 01-Common-Ports.md         # Hyperfine Markdown table
│   ├── 02-1K-Ports.json
│   ├── 02-1K-Ports.md
│   ├── 03-10K-Ports.json
│   ├── 03-10K-Ports.md
│   ├── 04-Full-Range.json         # (Full suite only)
│   ├── 05-With-Database.json      # (Standard + Full)
│   └── 06-Timing-T*.json          # (Standard: T0/T3/T5, Full: T0-T5)
├── perf/                          # (if available)
│   ├── 01-perf-1K.data            # Raw perf data
│   ├── 01-CPU-Profile-1K.txt      # Text report
│   ├── 02-perf-10K.data
│   └── 02-CPU-Profile-10K.txt
├── flamegraphs/                   # (if available)
│   ├── 01-CPU-Flamegraph-1K.svg   # Interactive SVG
│   └── 02-CPU-Flamegraph-10K.svg
├── valgrind/                      # (if available)
│   ├── 01-Massif-1K.out           # Raw massif output
│   ├── 01-Massif-1K-Report.txt    # Human-readable report
│   ├── 02-Massif-10K.out
│   └── 02-Massif-10K-Report.txt
└── strace/                        # (if available)
    ├── 01-Syscalls-1K.txt         # Syscall summary
    └── 02-Syscalls-10K.txt
```

---

## SUCCESS CRITERIA

✅ **All suites:**
- hyperfine statistical benchmarks complete
- Comparison analysis generated
- README and environment docs created

✅ **Standard + Full suites (if tools available):**
- Perf CPU profiles generated
- FlameGraphs created (interactive SVG)
- Valgrind memory profiles complete
- System call traces captured

✅ **Quality checks:**
- Mean times have <20% coefficient of variation
- All JSON files parseable with jq
- All analysis documents generated
- Directory structure complete

---

## SEE ALSO

**Commands:**
- `/perf-profile` - Deep CPU profiling for specific scenarios
- `/bench-compare` - Compare performance between git refs
- `/rust-check` - Pre-benchmark validation
- `/sprint-complete` - Include benchmark results in sprint summary

**Documentation:**
- `docs/07-PERFORMANCE.md` - Performance targets and optimization guide
- `docs/12-BENCHMARKING-GUIDE.md` - Detailed benchmarking methodology
- `benchmarks/` - Historical benchmark results
- `benchmarks/01-Phase4_PreFinal-Bench/README.md` - Previous benchmark example

**External Resources:**
- Hyperfine: https://github.com/sharkdp/hyperfine
- Linux perf: https://perf.wiki.kernel.org/
- Valgrind: https://valgrind.org/docs/manual/manual.html
- FlameGraphs: http://www.brendangregg.com/flamegraphs.html

---

**Run comprehensive benchmark suite: $***
