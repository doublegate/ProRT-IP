Generate performance profile with perf + flamegraph: $*

---

## PERFORMANCE PROFILING WORKFLOW

**Purpose:** Execute comprehensive CPU profiling using Linux perf and generate interactive flamegraph visualization

**Usage:** `/perf-profile <command>`
- **command:** Command to profile (e.g., "./target/release/prtip -p 1-10000 127.0.0.1")

**Example:** `/perf-profile ./target/release/prtip -p 1-10000 127.0.0.1`

---

## Phase 1: PREREQUISITES AND ENVIRONMENT SETUP

**Objective:** Verify perf tooling availability and prepare build configuration

### Step 1.1: Validate perf Installation

```bash
if ! command -v perf &> /dev/null; then
  echo "ERROR: perf not installed"
  echo "Install: sudo pacman -S perf (Arch) or sudo apt install linux-tools-generic (Ubuntu)"
  exit 1
fi

if ! command -v stackcollapse-perf.pl &> /dev/null; then
  echo "ERROR: FlameGraph tools not found"
  echo "Install: git clone https://github.com/brendangregg/FlameGraph"
  echo "Add to PATH: export PATH=\$PATH:/path/to/FlameGraph"
  exit 1
fi
```

### Step 1.2: Check Kernel Permissions

```bash
PERF_PARANOID=$(cat /proc/sys/kernel/perf_event_paranoid)

if [ "$PERF_PARANOID" -gt 1 ]; then
  echo "WARNING: perf_event_paranoid=$PERF_PARANOID (restrictive)"
  echo "Recommend: sudo sysctl -w kernel.perf_event_paranoid=1"
  echo "Or run with sudo for full profiling capabilities"
fi
```

### Step 1.3: Verify CPU Governor (Performance Mode)

```bash
GOVERNOR=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo "unknown")

if [ "$GOVERNOR" != "performance" ]; then
  echo "WARNING: CPU governor is '$GOVERNOR' (not 'performance')"
  echo "Recommend: sudo cpupower frequency-set -g performance"
  echo "This ensures consistent profiling results"
fi
```

---

## Phase 2: BUILD WITH DEBUG SYMBOLS

**Objective:** Compile release build with debug symbols for accurate call graphs

### Step 2.1: Create Temporary Cargo Config

```bash
mkdir -p .cargo
cat > .cargo/config.toml <<EOF
[profile.release]
debug = true
debug-assertions = false
overflow-checks = false
lto = "thin"
strip = false
codegen-units = 16

[build]
rustflags = ["-C", "force-frame-pointers=yes"]
EOF
```

**Why These Settings:**
- `debug = true`: Include debug symbols (function names, line numbers)
- `force-frame-pointers=yes`: Enable accurate call stack unwinding
- `lto = "thin"`: Reduced LTO for faster builds (vs "fat")
- `codegen-units = 16`: Balance compilation speed vs optimization

### Step 2.2: Build Release Binary with Debug Info

```bash
echo "Building release binary with debug symbols..."
cargo build --release

if [ $? -ne 0 ]; then
  echo "ERROR: Build failed"
  rm .cargo/config.toml
  exit 1
fi
```

**Duration:** ~30-60 seconds (varies by system)

### Step 2.3: Verify Binary Has Debug Info

```bash
if file target/release/prtip | grep -q "not stripped"; then
  echo "✅ Binary contains debug symbols"
else
  echo "⚠️ WARNING: Binary appears stripped (profiling may be limited)"
fi
```

---

## Phase 3: EXECUTE PERFORMANCE PROFILING

**Objective:** Run perf record with comprehensive call graph capture

### Step 3.1: Parse Command to Profile

```bash
PROFILE_CMD="$*"

if [ -z "$PROFILE_CMD" ]; then
  echo "ERROR: No command specified to profile"
  echo "Usage: /perf-profile <command>"
  exit 1
fi

echo "Profiling command: $PROFILE_CMD"
```

### Step 3.2: Run perf record

```bash
echo "Starting perf profiling (this will take a moment)..."

perf record \
  --call-graph dwarf \
  --freq 997 \
  --output /tmp/perf.data \
  $PROFILE_CMD

if [ $? -ne 0 ]; then
  echo "ERROR: perf record failed"
  exit 1
fi
```

**perf record Parameters:**
- `--call-graph dwarf`: DWARF-based call graph (most accurate)
- `--freq 997`: Sample at 997 Hz (prime number avoids aliasing)
- `--output /tmp/perf.data`: Output file location

**Alternative (if DWARF fails):** `--call-graph fp` (frame pointer-based)

**Duration:** Depends on profiled command (10K ports ~40-50ms + perf overhead)

### Step 3.3: Verify perf.data File

```bash
PERF_SIZE=$(stat -c%s /tmp/perf.data)
echo "perf.data size: $((PERF_SIZE / 1024)) KB"

if [ "$PERF_SIZE" -lt 1000 ]; then
  echo "WARNING: perf.data is very small (may indicate capture failure)"
fi
```

---

## Phase 4: GENERATE ANALYSIS REPORTS

**Objective:** Create human-readable reports from perf.data

### Step 4.1: Generate perf report (Text)

```bash
echo "Generating perf report..."

perf report \
  --input /tmp/perf.data \
  --stdio \
  --sort comm,dso,symbol \
  > /tmp/perf-report.txt

echo "✅ Text report: /tmp/perf-report.txt"
```

**Report Contents:**
- Top functions by CPU time
- Call chains (parent → child relationships)
- DSO (Dynamic Shared Object) attribution

### Step 4.2: Generate perf script (Detailed)

```bash
echo "Generating perf script..."

perf script \
  --input /tmp/perf.data \
  > /tmp/perf-script.txt

echo "✅ Script output: /tmp/perf-script.txt"
```

**Script Contents:**
- Sample-by-sample data
- Stack traces for each sample
- Used as input for flamegraph generation

### Step 4.3: Display Top Functions (Quick Summary)

```bash
echo ""
echo "=========================================="
echo "Top 10 Functions by CPU Time"
echo "=========================================="
perf report \
  --input /tmp/perf.data \
  --stdio \
  --sort symbol \
  --percent-limit 1 \
  | head -20
echo ""
```

---

## Phase 5: GENERATE FLAMEGRAPH

**Objective:** Create interactive SVG flamegraph for visual analysis

### Step 5.1: Collapse Stack Traces

```bash
echo "Collapsing stack traces..."

stackcollapse-perf.pl /tmp/perf-script.txt \
  > /tmp/perf-collapsed.txt

COLLAPSED_LINES=$(wc -l < /tmp/perf-collapsed.txt)
echo "Collapsed $COLLAPSED_LINES unique stack traces"
```

### Step 5.2: Generate Flamegraph SVG

```bash
echo "Generating flamegraph..."

flamegraph.pl \
  --title "ProRT-IP Performance Profile" \
  --width 1800 \
  --height 900 \
  --colors rust \
  /tmp/perf-collapsed.txt \
  > /tmp/flamegraph.svg

SVG_SIZE=$(stat -c%s /tmp/flamegraph.svg)
echo "✅ Flamegraph: /tmp/flamegraph.svg ($((SVG_SIZE / 1024)) KB)"
```

**Flamegraph Parameters:**
- `--title`: Custom title for SVG
- `--width/--height`: SVG dimensions (pixels)
- `--colors rust`: Rust-specific color palette

### Step 5.3: Verify Flamegraph

```bash
if [ "$SVG_SIZE" -gt 10000 ]; then
  echo "✅ Flamegraph generated successfully (open in browser for interactive visualization)"
else
  echo "⚠️ WARNING: Flamegraph is very small (may indicate insufficient samples)"
fi
```

---

## Phase 6: ANALYSIS AND CLEANUP

**Objective:** Provide analysis guidance and clean up temporary files

### Step 6.1: Display Analysis Guide

```bash
echo ""
echo "=========================================="
echo "Performance Analysis Guide"
echo "=========================================="
echo ""
echo "📊 GENERATED ARTIFACTS:"
echo "  1. /tmp/perf.data           - Raw perf data (~$((PERF_SIZE / 1024)) KB)"
echo "  2. /tmp/perf-report.txt     - Text report with top functions"
echo "  3. /tmp/perf-script.txt     - Detailed sample-by-sample data"
echo "  4. /tmp/perf-collapsed.txt  - Collapsed stack traces"
echo "  5. /tmp/flamegraph.svg      - Interactive flamegraph ($((SVG_SIZE / 1024)) KB)"
echo ""
echo "🔍 FLAMEGRAPH NAVIGATION:"
echo "  - Width: Proportion of CPU time (wider = more time)"
echo "  - Height: Call stack depth (bottom = caller, top = callee)"
echo "  - Click box: Zoom into that function and children"
echo "  - Hover: Show function name and percentage"
echo "  - Search: Ctrl+F to highlight specific functions"
echo ""
echo "🎯 OPTIMIZATION TARGETS:"
echo "  - Wide boxes at top: Hot leaf functions (optimization candidates)"
echo "  - Look for: Unexpected functions, excessive allocations, lock contention"
echo "  - Compare: Run /bench-compare before/after optimizations"
echo ""
echo "🔧 COMMON BOTTLENECKS:"
echo "  - Tokio runtime: async task scheduling overhead"
echo "  - DNS resolution: Network I/O blocking"
echo "  - SQLite writes: Database transaction overhead"
echo "  - Lock contention: Arc<Mutex<>> or RwLock blocking"
echo ""
```

### Step 6.2: Cleanup Temporary Build Config

```bash
echo "Cleaning up temporary build configuration..."
rm .cargo/config.toml

echo "✅ Removed temporary cargo config (restore production settings)"
```

### Step 6.3: Rebuild Production Binary (Optional)

```bash
echo ""
read -p "Rebuild production binary without debug symbols? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
  cargo build --release
  echo "✅ Production binary rebuilt"
fi
```

---

## SUCCESS CRITERIA

✅ perf record completed successfully (perf.data generated)
✅ perf report generated (top functions identified)
✅ perf script generated (detailed samples)
✅ Flamegraph SVG generated (>10 KB interactive visualization)
✅ Temporary build config cleaned up
✅ Analysis guide displayed

---

## ADVANCED USAGE

### Hardware Counter Analysis

```bash
perf stat -e cycles,instructions,cache-misses,branch-misses \
  $PROFILE_CMD
```

**Metrics:**
- IPC (instructions per cycle): >0.4 good, <0.2 poor
- Cache miss rate: <1% excellent, >5% poor
- Branch miss rate: <2% good, >5% poor

### Multi-Threaded Profiling

```bash
perf record \
  --call-graph dwarf \
  --freq 997 \
  --all-cpus \
  $PROFILE_CMD
```

**Use `--all-cpus` for capturing all CPU cores (multi-threaded profiling)**

### Off-CPU Profiling (I/O wait time)

```bash
perf record \
  --call-graph dwarf \
  --event sched:sched_switch \
  $PROFILE_CMD
```

**Captures time spent waiting (I/O, locks, etc.) vs CPU time**

---

## COMMON ISSUES AND SOLUTIONS

### Issue: "perf_event_open failed: Permission denied"

**Solution:** Reduce perf_event_paranoid or run with sudo
```bash
sudo sysctl -w kernel.perf_event_paranoid=1
```

### Issue: Flamegraph shows only "[unknown]" functions

**Solution:** Binary missing debug symbols
- Verify `debug = true` in Cargo.toml profile.release
- Ensure `strip = false` (no symbol stripping)
- Rebuild with debug symbols

### Issue: Very few samples captured

**Solution:** Profiled command ran too quickly
- Increase workload (more ports, more hosts)
- Use `--freq 4999` for higher sample rate
- Run command in loop: `for i in {1..100}; do ...; done`

---

## DELIVERABLES

1. **perf.data:** Raw profiling data (~100-500 KB)
2. **perf-report.txt:** Top functions by CPU time
3. **perf-script.txt:** Detailed sample data
4. **perf-collapsed.txt:** Collapsed stack traces
5. **flamegraph.svg:** Interactive visualization (100-300 KB)
6. **Analysis guide:** Console output with optimization targets

---

## RELATED COMMANDS

**Benchmarking Workflow:**
- `/bench-compare <baseline> <comparison>` - Compare performance before/after optimization
- Statistical benchmarking with hyperfine for accurate metrics
- Complements perf profiling with wall-clock time comparison

**Development Workflow:**
- `/rust-check` - Ensure code compiles and passes tests before profiling
- `/test-quick <pattern>` - Run specific tests to isolate performance issues
- `/module-create` - Create optimized replacement modules based on profiling insights

**Sprint Management:**
- `/sprint-start` - Initialize performance optimization sprint
- `/sprint-complete` - Document performance improvements with benchmarks

**Debugging:**
- `/bug-report` - Report performance bugs with profile data attached

## WORKFLOW INTEGRATION

**Performance Optimization Workflow:**

```
1. Baseline Measurement:
   /bench-compare baseline HEAD  # Measure current performance

2. Identify Bottlenecks:
   /perf-profile ./target/release/prtip <args>
   # Analyze flamegraph - find hot functions

3. Implement Optimizations:
   - Edit code based on profile insights
   - /rust-check  # Validate changes
   - /test-quick <pattern>  # Targeted testing

4. Validate Improvement:
   /bench-compare baseline HEAD  # Confirm speedup
   /perf-profile ./target/release/prtip <args>  # Verify hotspots reduced

5. Document Results:
   /doc-update perf "10K ports: 117ms → 39ms (66% faster)"
```

**Common Optimization Patterns:**

1. **CPU-bound optimization:**
   ```
   perf-profile → identify hot function → optimize algorithm → bench-compare
   ```

2. **I/O-bound optimization:**
   ```
   perf-profile (off-CPU) → find blocking I/O → add async → bench-compare
   ```

3. **Lock contention optimization:**
   ```
   perf-profile → find lock hotspots → replace with lock-free → bench-compare
   ```

## SEE ALSO

- `docs/07-PERFORMANCE.md` - Performance optimization guide
- `docs/04-IMPLEMENTATION-GUIDE.md` - Implementation patterns
- `benchmarks/` - Historical benchmark results
- Brendan Gregg's flamegraph documentation: http://www.brendangregg.com/flamegraphs.html
- Linux perf tutorial: https://perf.wiki.kernel.org/index.php/Tutorial

---

**Profile command: $***
