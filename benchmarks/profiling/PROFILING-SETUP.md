# Profiling Setup Guide

**Version:** 1.0.0
**Sprint:** 5.5.5 - Profiling Execution
**Date:** 2025-11-09
**Document Status:** Production-Ready

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Platform-Specific Setup](#platform-specific-setup)
4. [Tool Installation](#tool-installation)
5. [Profiling Workflow](#profiling-workflow)
6. [Troubleshooting](#troubleshooting)
7. [Advanced Configuration](#advanced-configuration)

---

## Overview

### Purpose

This guide documents how to set up and execute performance profiling for ProRT-IP. Profiling is essential for:

- **Identifying Bottlenecks:** CPU hotspots, memory allocation patterns, I/O inefficiencies
- **Data-Driven Optimization:** Measure before optimizing (no speculation)
- **Regression Detection:** Validate performance changes
- **Capacity Planning:** Understand resource requirements

### Profiling Types

| Type | Tool | Purpose | Output |
|------|------|---------|--------|
| **CPU** | cargo-flamegraph / perf | Identify hot functions (>5% CPU) | SVG flamegraphs |
| **Memory** | valgrind massif | Analyze heap allocations, detect leaks | Massif reports |
| **I/O** | strace | Analyze syscall patterns, batching | Syscall summaries |

### Expected Duration

| Profiling Type | Normal Scan | Profiling Overhead | Profiled Duration |
|----------------|-------------|-------------------|------------------|
| **CPU (flamegraph)** | 98ms (SYN 1K) | 2-5x | 200-500ms |
| **Memory (massif)** | 98ms (SYN 1K) | 10-50x | 1-5 seconds |
| **I/O (strace)** | 98ms (SYN 1K) | 2-3x | 200-300ms |

**Note:** Valgrind massif has the highest overhead (10-50x) but provides detailed heap analysis.

---

## Prerequisites

### System Requirements

- **OS:** Linux (primary), macOS (limited), Windows (unsupported for profiling)
- **CPU:** Multi-core recommended (profiling is CPU-intensive)
- **RAM:** 4 GB minimum (8 GB recommended for massif)
- **Storage:** 1 GB free (flamegraphs and massif output)

### Build Requirements

ProRT-IP must be built in **release mode** for accurate profiling:

```bash
cargo build --release
```

**Why:** Debug builds have profiling overhead (bounds checks, debug symbols) that skews results. Always profile release builds.

### Permissions

**Linux:**

Raw socket profiling requires elevated privileges:

```bash
# Option 1: Run profiling wrapper as root (simple)
sudo ./benchmarks/profiling/profile-scenario.sh --scenario test -- -sS -p 80 127.0.0.1

# Option 2: Grant CAP_NET_RAW to profiling tools (recommended)
sudo setcap cap_net_raw+ep target/release/prtip
./benchmarks/profiling/profile-scenario.sh --scenario test -- -sS -p 80 127.0.0.1

# Option 3: Configure perf for non-root (advanced)
sudo sysctl kernel.perf_event_paranoid=-1
```

**macOS:**

ChmodBPF required for BPF access:

```bash
# Install ChmodBPF (one-time)
brew install --cask wireshark  # Includes ChmodBPF

# Or manually configure BPF permissions
sudo chmod 644 /dev/bpf*
```

---

## Platform-Specific Setup

### Linux (Primary Platform - Best Support)

**Supported Distributions:**
- Ubuntu 20.04+ / Debian 11+
- Fedora 35+ / RHEL 8+
- Arch Linux / Manjaro
- CachyOS (tested platform)

**Install All Profiling Tools:**

```bash
# Arch-based (CachyOS, Manjaro)
sudo pacman -S perf valgrind strace
cargo install flamegraph

# Debian-based (Ubuntu)
sudo apt install linux-tools-common linux-tools-generic valgrind strace
cargo install flamegraph

# Fedora/RHEL
sudo dnf install perf valgrind strace
cargo install flamegraph
```

**Verify Installation:**

```bash
cargo-flamegraph --version  # Should show: flamegraph x.x.x
perf --version              # Should show: perf version 5.x+
valgrind --version          # Should show: valgrind-3.x
strace --version            # Should show: strace -- version 5.x+
```

**Configure perf for Non-Root (Optional):**

```bash
# Check current paranoid level
cat /proc/sys/kernel/perf_event_paranoid
# -1 = allow all users
#  0 = disallow kernel profiling (CPU events OK)
#  1 = disallow CPU events
#  2 = disallow raw tracepoints

# Allow non-root profiling (temporary)
sudo sysctl kernel.perf_event_paranoid=-1

# Permanent (add to /etc/sysctl.conf)
echo "kernel.perf_event_paranoid=-1" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

---

### macOS (Limited Support)

**Supported Versions:**
- macOS 11.0+ (Big Sur and later)

**Install Profiling Tools:**

```bash
# Install Xcode Command Line Tools (includes Instruments, DTrace)
xcode-select --install

# Install Homebrew tools
brew install valgrind  # Note: Limited macOS support (Intel only)
cargo install flamegraph

# strace alternative: Use dtrace (built-in)
# No direct strace equivalent, see "macOS Alternatives" below
```

**macOS Alternatives:**

| Linux Tool | macOS Alternative | Notes |
|------------|------------------|-------|
| **perf** | Instruments (Xcode) | Use "Time Profiler" template |
| **valgrind massif** | Instruments "Allocations" | Heap profiling |
| **strace** | dtrace / dtruss | Syscall tracing |
| **cargo-flamegraph** | cargo-flamegraph (works) | Uses DTrace backend |

**Using Instruments (Xcode):**

```bash
# CPU profiling (Time Profiler)
instruments -t "Time Profiler" -D instruments-cpu.trace target/release/prtip -sS -p 1-1000 127.0.0.1

# Memory profiling (Allocations)
instruments -t "Allocations" -D instruments-memory.trace target/release/prtip -sS -p 1-1000 127.0.0.1
```

**Using dtrace for I/O:**

```bash
# Syscall counting (similar to strace -c)
sudo dtrace -n 'syscall:::entry /execname == "prtip"/ { @syscalls[probefunc] = count(); }'

# While running prtip in another terminal:
target/release/prtip -sS -p 1-1000 127.0.0.1
```

---

### Windows (Unsupported for Profiling)

**Status:** Profiling on Windows is not officially supported in Sprint 5.5.5.

**Limitations:**
- No native `perf` or `valgrind` support
- `cargo-flamegraph` requires WSL or custom ETW backend
- `strace` unavailable (use Process Monitor as alternative)

**Recommended Approach:**

1. **Use WSL2 (Windows Subsystem for Linux):**
   - Install Ubuntu 22.04 in WSL2
   - Follow Linux setup instructions above
   - Profile ProRT-IP within WSL2 environment

2. **Use Native Windows Profiling (Advanced):**
   - Visual Studio Profiler (CPU, memory)
   - Windows Performance Toolkit (ETW)
   - Process Monitor (syscall tracing)

**Note:** For production profiling, use Linux or macOS. Windows profiling is experimental.

---

## Tool Installation

### cargo-flamegraph

**Purpose:** Generate CPU flamegraphs from perf/DTrace data.

**Install:**

```bash
cargo install flamegraph
```

**Verify:**

```bash
cargo-flamegraph --version
# Output: flamegraph 0.6.x
```

**Usage:**

```bash
# Profile a single scan
cargo flamegraph --bin prtip -- -sS -p 1-1000 127.0.0.1

# Custom sampling rate (default: 99Hz)
cargo flamegraph --bin prtip --freq 999 -- -sS -p 1-1000 127.0.0.1

# Output to specific file
cargo flamegraph --bin prtip --output flamegraph-syn.svg -- -sS -p 1-1000 127.0.0.1
```

**Troubleshooting:**

```bash
# If "perf not found" on Linux
sudo pacman -S perf  # Arch
sudo apt install linux-tools-generic  # Ubuntu

# If "permission denied" on Linux
sudo sysctl kernel.perf_event_paranoid=-1

# If "dtrace not found" on macOS
xcode-select --install
```

---

### valgrind massif

**Purpose:** Heap memory profiling, allocation tracking.

**Install:**

```bash
# Linux (Arch)
sudo pacman -S valgrind

# Linux (Debian/Ubuntu)
sudo apt install valgrind

# macOS (Intel only, limited support)
brew install valgrind
```

**Verify:**

```bash
valgrind --version
# Output: valgrind-3.20+ (or higher)
```

**Usage:**

```bash
# Run massif on a scan
valgrind --tool=massif \
    --massif-out-file=massif-syn.out \
    target/release/prtip -sS -p 1-1000 127.0.0.1

# Generate human-readable report
ms_print massif-syn.out > massif-syn-report.txt

# View report
less massif-syn-report.txt
```

**Performance Note:**

Massif adds **10-50x overhead**. A 98ms scan may take 1-5 seconds under massif. This is expected.

**Leak Checking:**

```bash
# Check for memory leaks
valgrind --leak-check=full \
    --show-leak-kinds=all \
    --log-file=leak-check.txt \
    target/release/prtip -sS -p 1-1000 127.0.0.1

# View leak report
cat leak-check.txt | grep "definitely lost"
# Expect: 0 bytes definitely lost (Rust safety guarantee)
```

---

### strace

**Purpose:** Syscall tracing, I/O pattern analysis.

**Install:**

```bash
# Linux (usually pre-installed)
strace --version

# If not installed:
sudo pacman -S strace  # Arch
sudo apt install strace  # Ubuntu
```

**Verify:**

```bash
strace --version
# Output: strace -- version 5.x+
```

**Usage:**

```bash
# Syscall summary (count, time)
strace -c -o strace-summary.txt \
    target/release/prtip -sS -p 1-1000 127.0.0.1

# Trace specific syscalls (sendmmsg, recvmmsg)
strace -e sendmmsg,recvmmsg -o strace-detail.txt \
    target/release/prtip -sS -p 1-1000 127.0.0.1

# View summary
cat strace-summary.txt
```

**macOS Alternative (dtrace):**

```bash
# Use dtruss (dtrace wrapper)
sudo dtruss -c target/release/prtip -sS -p 1-1000 127.0.0.1
```

---

## Profiling Workflow

### Step 1: Build Release Binary

```bash
cd /home/parobek/Code/ProRT-IP
cargo build --release
```

**Verify:**

```bash
ls -lh target/release/prtip
# Should show: ~12-15 MB binary (stripped release build)
```

---

### Step 2: Run Profiling Wrapper

**CPU Profiling (Flamegraph):**

```bash
./benchmarks/profiling/profile-scenario.sh \
    --scenario syn-scan-1k \
    --type cpu \
    -- -sS -p 1-1000 127.0.0.1
```

**Output:** `benchmarks/profiling/results/flamegraphs/syn-scan-1k-flamegraph.svg`

**Memory Profiling (Massif):**

```bash
./benchmarks/profiling/profile-scenario.sh \
    --scenario syn-scan-1k \
    --type memory \
    -- -sS -p 1-1000 127.0.0.1
```

**Output:**
- `benchmarks/profiling/results/massif/syn-scan-1k-massif.out`
- `benchmarks/profiling/results/massif/syn-scan-1k-massif-report.txt`

**I/O Profiling (Strace):**

```bash
./benchmarks/profiling/profile-scenario.sh \
    --scenario syn-scan-1k \
    --type io \
    -- -sS -p 1-1000 127.0.0.1
```

**Output:**
- `benchmarks/profiling/results/strace/syn-scan-1k-strace-summary.txt`
- `benchmarks/profiling/results/strace/syn-scan-1k-strace-detail.txt`

---

### Step 3: Analyze Results

**Flamegraph Analysis:**

```bash
# View in browser
firefox benchmarks/profiling/results/flamegraphs/syn-scan-1k-flamegraph.svg

# Look for wide bars (hot functions consuming >5% CPU)
# X-axis: Alphabetical order (NOT time)
# Y-axis: Call stack depth
```

**Massif Analysis:**

```bash
# View human-readable report
less benchmarks/profiling/results/massif/syn-scan-1k-massif-report.txt

# Key sections:
# - Peak heap usage (at top)
# - Allocation tree (functions allocating most memory)
# - Heap growth over time (timeline)
```

**Strace Analysis:**

```bash
# View syscall summary
cat benchmarks/profiling/results/strace/syn-scan-1k-strace-summary.txt

# Look for:
# - sendmmsg/recvmmsg call counts (expect: <20 for 1K packets = >50 packets/batch)
# - Excessive syscalls (too many small I/O operations)
# - Time spent in syscalls (% time column)
```

---

## Troubleshooting

### Issue: "perf: Permission denied"

**Cause:** `perf_event_paranoid` restricts non-root profiling.

**Solution:**

```bash
# Temporary fix
sudo sysctl kernel.perf_event_paranoid=-1

# Permanent fix
echo "kernel.perf_event_paranoid=-1" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

---

### Issue: "valgrind: command not found"

**Cause:** Valgrind not installed.

**Solution:**

```bash
# Linux (Arch)
sudo pacman -S valgrind

# Linux (Ubuntu)
sudo apt install valgrind

# macOS (Intel only)
brew install valgrind
```

**Note:** Valgrind is not available on Apple Silicon (M1/M2). Use Instruments instead.

---

### Issue: Flamegraph shows no data

**Cause:** Profiling duration too short (<1 second).

**Solution:**

```bash
# Increase scan size (more ports, more time)
cargo flamegraph --bin prtip -- -sS -p 1-10000 127.0.0.1

# Or lower sampling rate (more samples)
cargo flamegraph --bin prtip --freq 999 -- -sS -p 1-1000 127.0.0.1
```

---

### Issue: Massif takes too long

**Cause:** Valgrind overhead (10-50x slowdown).

**Solution:**

```bash
# Reduce scan size
valgrind --tool=massif --massif-out-file=massif.out \
    target/release/prtip -sS -p 1-100 127.0.0.1  # Only 100 ports

# Or increase timeout patience (expected behavior)
```

---

## Advanced Configuration

### Differential Flamegraphs

Compare two scenarios to isolate feature overhead:

```bash
# Profile baseline (SYN scan, no rate limit)
cargo flamegraph --bin prtip --output baseline.svg -- -sS -p 1-1000 127.0.0.1

# Profile with feature (SYN scan, with rate limit)
cargo flamegraph --bin prtip --output with-rate-limit.svg -- -sS -p 1-1000 127.0.0.1 --max-rate 10000

# Generate differential (requires flamegraph.pl from Brendan Gregg's repo)
# git clone https://github.com/brendangregg/FlameGraph
# ./FlameGraph/difffolded.pl baseline.folded with-rate-limit.folded | ./FlameGraph/flamegraph.pl > diff.svg
```

**Note:** Differential flamegraphs require manual flamegraph.pl usage (not cargo-flamegraph).

---

### Custom Sampling Rates

**Default:** 99 Hz (99 samples/second)

**Higher sampling (more accurate, more overhead):**

```bash
cargo flamegraph --bin prtip --freq 999 -- -sS -p 1-1000 127.0.0.1
# 999 Hz = 10x more samples, better accuracy for fast functions
```

**Lower sampling (less overhead, less accuracy):**

```bash
cargo flamegraph --bin prtip --freq 49 -- -sS -p 1-1000 127.0.0.1
# 49 Hz = half samples, faster profiling for slow scans
```

---

### NUMA-Aware Profiling

For multi-socket systems:

```bash
# Check NUMA topology
numactl --hardware

# Profile with NUMA binding
numactl --cpunodebind=0 --membind=0 \
    cargo flamegraph --bin prtip -- -sS -p 1-65535 192.168.1.0/24
```

---

## References

### Internal Documentation

- [Performance Characteristics](../docs/34-PERFORMANCE-CHARACTERISTICS.md) - Baseline metrics
- [Benchmarking Guide](../docs/31-BENCHMARKING-GUIDE.md) - Framework usage
- [Sprint 5.5.5 TODO](../../to-dos/SPRINT-5.5.5-TODO.md) - Sprint plan

### External Tools

- **cargo-flamegraph:** https://github.com/flamegraph-rs/flamegraph
- **perf:** https://perf.wiki.kernel.org
- **valgrind:** https://valgrind.org
- **strace:** https://strace.io
- **Brendan Gregg's Flamegraphs:** https://www.brendangregg.com/flamegraphs.html

### Profiling Guides

- **Rust Performance Book:** https://nnethercote.github.io/perf-book/
- **Valgrind User Manual:** https://valgrind.org/docs/manual/manual.html
- **Linux perf Examples:** https://www.brendangregg.com/perf.html

---

**Document Version:** 1.0.0
**Created:** 2025-11-09
**Sprint:** 5.5.5 - Profiling Execution
**Status:** Production-Ready

---

**End of Profiling Setup Guide**
