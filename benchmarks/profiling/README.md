# ProRT-IP Profiling Framework

**Version:** 1.0.0
**Created:** 2025-11-09 (Sprint 5.5.5)
**Updated:** 2025-11-09

## Overview

Comprehensive profiling infrastructure for ProRT-IP performance analysis, optimization target identification, and regression detection.

## Purpose

This framework enables:

1. **Performance Baseline Establishment**: CPU, memory, and I/O profiling across key scenarios
2. **Bottleneck Identification**: Data-driven optimization target discovery
3. **Regression Detection**: Track performance changes across versions
4. **Optimization Validation**: Measure gains from performance improvements

## Directory Structure

```
benchmarks/profiling/
├── README.md                    # This file
├── PROFILING-SETUP.md          # Platform-specific setup guide
├── PROFILING-ANALYSIS.md       # Comprehensive analysis with optimization targets
├── IO-ANALYSIS.md              # I/O-specific syscall analysis
├── profile-scenario.sh         # Standardized profiling wrapper
├── results/                    # Current profiling outputs
│   ├── flamegraphs/           # CPU flamegraphs (SVG)
│   ├── massif/                # Memory profiles (massif.out + reports)
│   └── strace/                # I/O syscall traces
└── v0.5.0/                    # Versioned baseline archive
    ├── METADATA.md            # Profiling run metadata
    ├── cpu/                   # Archived flamegraphs
    ├── memory/                # Archived massif profiles
    └── io/                    # Archived strace analyses
```

## Profiling Types

### 1. CPU Profiling (Flamegraphs)

**Tool:** cargo-flamegraph + perf
**Output:** Interactive SVG flamegraph
**Overhead:** 2-5x slowdown
**Purpose:** Identify hot paths, function-level CPU consumption

**Usage:**
```bash
./profile-scenario.sh \
    --scenario syn-scan-1k \
    --type cpu \
    -- -sS -p 1-1000 127.0.0.1
```

**Output:** `results/flamegraphs/syn-scan-1k-flamegraph.svg`

**Key Scenarios:**
- `syn-scan-1k`: SYN scan 1,000 ports (baseline)
- `connect-scan-100`: Connect scan 100 ports
- `ipv6-scan-500`: IPv6 SYN scan 500 ports
- `service-detect-20`: Service detection 20 common ports
- `tls-cert-10`: TLS certificate extraction 10 HTTPS hosts

### 2. Memory Profiling (Massif)

**Tool:** valgrind --tool=massif
**Output:** massif.out + human-readable report
**Overhead:** 10-50x slowdown
**Purpose:** Heap allocation tracking, memory leak detection

**Usage:**
```bash
./profile-scenario.sh \
    --scenario syn-scan-1k \
    --type memory \
    -- -sS -p 1-1000 127.0.0.1
```

**Output:**
- `results/massif/syn-scan-1k-massif.out` (raw data)
- `results/massif/syn-scan-1k-massif-report.txt` (ms_print output)

**Key Metrics:**
- Peak heap usage
- Allocation sites (stack traces)
- Memory trends over time

### 3. I/O Profiling (strace)

**Tool:** strace
**Output:** Syscall summary + detailed traces
**Overhead:** 5-20x slowdown
**Purpose:** Syscall analysis, batching effectiveness, kernel interaction

**Usage:**
```bash
./profile-scenario.sh \
    --scenario syn-scan-1k \
    --type io \
    -- -sS -p 1-1000 127.0.0.1
```

**Output:**
- `results/strace/syn-scan-1k-strace-summary.txt` (syscall counts)
- `results/strace/syn-scan-1k-strace-detail.txt` (key syscall traces)

**Key Syscalls Monitored:**
- `sendmmsg`: Batch packet transmission
- `recvmmsg`: Batch packet reception
- `write/writev`: Output operations
- `mmap/munmap`: Memory management

## Wrapper Script

**File:** `profile-scenario.sh`

Standardized interface for all profiling types with consistent output organization.

**Key Features:**
- Automatic output directory creation
- Release binary validation (builds if missing)
- Platform-agnostic (Linux/macOS/Windows WSL)
- Configurable sampling rates (CPU profiling)
- Post-processing automation (ms_print for massif)

**Arguments:**
- `--scenario <name>`: Scenario identifier (e.g., "syn-scan-1k")
- `--type <cpu|memory|io>`: Profiling type (default: cpu)
- `--output-dir <path>`: Output directory (default: results/)
- `--sampling-rate <Hz>`: CPU sampling frequency (default: 99Hz)
- `-- <prtip args>`: Arguments passed to prtip binary

**Example:**
```bash
# CPU profiling with 199Hz sampling
./profile-scenario.sh \
    --scenario syn-scan-1k \
    --type cpu \
    --sampling-rate 199 \
    -- -sS -p 1-1000 192.168.1.0/24

# Memory profiling
./profile-scenario.sh \
    --scenario service-detect \
    --type memory \
    -- -sV -p 80,443,22,25,3306 scanme.nmap.org

# I/O profiling
./profile-scenario.sh \
    --scenario connect-scan \
    --type io \
    -- -sT -p 1-100 127.0.0.1
```

## Setup Requirements

See `PROFILING-SETUP.md` for platform-specific installation and configuration.

**Quick Start:**

**Linux:**
```bash
# Install tools
sudo apt install linux-perf valgrind strace
cargo install flamegraph

# Configure perf permissions
echo 0 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

**macOS:**
```bash
# Install tools
brew install valgrind
cargo install flamegraph

# Configure DTrace permissions
sudo chmod o+r /dev/dtracehelper
```

**Windows (WSL):**
```bash
# Use WSL2 with Linux kernel
# Follow Linux setup instructions
```

## Analysis Workflow

### 1. Profile Generation

Run profiling for key scenarios:

```bash
# CPU profiling (5 scenarios)
./profile-scenario.sh --scenario syn-scan-1k --type cpu -- -sS -p 1-1000 127.0.0.1
./profile-scenario.sh --scenario connect-scan-100 --type cpu -- -sT -p 1-100 127.0.0.1
./profile-scenario.sh --scenario ipv6-scan-500 --type cpu -- -6 -sS -p 1-500 ::1
./profile-scenario.sh --scenario service-detect-20 --type cpu -- -sV -p 21,22,25,80,110,143,443,445,3306,5432 127.0.0.1
./profile-scenario.sh --scenario tls-cert-10 --type cpu -- --tls-cert -p 443 scanme.nmap.org

# Memory profiling (3 scenarios - massif overhead is high)
./profile-scenario.sh --scenario syn-scan-1k --type memory -- -sS -p 1-1000 127.0.0.1
./profile-scenario.sh --scenario service-detect-20 --type memory -- -sV -p 80,443 127.0.0.1
./profile-scenario.sh --scenario tls-cert-10 --type memory -- --tls-cert -p 443 scanme.nmap.org

# I/O profiling (3 scenarios)
./profile-scenario.sh --scenario syn-scan-1k --type io -- -sS -p 1-1000 127.0.0.1
./profile-scenario.sh --scenario connect-scan-100 --type io -- -sT -p 1-100 127.0.0.1
./profile-scenario.sh --scenario service-detect-20 --type io -- -sV -p 80,443 127.0.0.1
```

### 2. Data Analysis

Examine profiling outputs:

**CPU (Flamegraphs):**
- Open SVG in browser (Firefox recommended)
- Identify wide stack frames (hot paths)
- Look for unexpected function calls
- Check async task distribution

**Memory (Massif):**
- Review `*-report.txt` for allocation sites
- Check peak heap usage
- Identify allocation-heavy functions
- Look for memory leaks (increasing trends)

**I/O (strace):**
- Check `*-summary.txt` for syscall counts
- Analyze batch sizes (sendmmsg/recvmmsg)
- Review `*-detail.txt` for timing patterns
- Identify syscall bottlenecks

### 3. Optimization Target Identification

See `PROFILING-ANALYSIS.md` for comprehensive analysis methodology.

**Priority Formula:**
```
Priority = (Impact × Frequency × Ease) / 10

Where:
- Impact: Performance gain potential (1-10)
- Frequency: How often code executes (1-10)
- Ease: Implementation complexity (10=easy, 1=hard)
```

**Example:**
- **Target:** Increase sendmmsg batch size 100→300
- **Impact:** 7/10 (5-10% throughput gain)
- **Frequency:** 10/10 (every packet transmission)
- **Ease:** 10/10 (single constant change)
- **Priority:** (7 × 10 × 10) / 10 = **70** (highest)

### 4. Version Archival

Archive baseline after profiling:

```bash
# Create versioned archive
mkdir -p v0.5.0/{cpu,memory,io}

# Copy results
cp results/flamegraphs/*.svg v0.5.0/cpu/
cp results/massif/*.{out,txt} v0.5.0/memory/
cp results/strace/*.txt v0.5.0/io/

# Create metadata
cat > v0.5.0/METADATA.md << 'EOF'
# Profiling Metadata v0.5.0

**Date:** 2025-11-09
**Version:** v0.5.0 (Sprint 5.5.5)
**Commit:** $(git rev-parse HEAD)
**Kernel:** $(uname -r)
**CPU:** $(lscpu | grep "Model name" | cut -d: -f2 | xargs)
**Memory:** $(free -h | grep Mem | awk '{print $2}')
**Tools:**
- cargo-flamegraph $(cargo flamegraph --version)
- valgrind $(valgrind --version | head -1)
- strace $(strace -V 2>&1 | head -1)

## Scenarios Profiled

### CPU (5 scenarios)
- syn-scan-1k: SYN scan 1,000 ports on 127.0.0.1
- connect-scan-100: Connect scan 100 ports on 127.0.0.1
- ipv6-scan-500: IPv6 SYN scan 500 ports on ::1
- service-detect-20: Service detection 20 ports on 127.0.0.1
- tls-cert-10: TLS certificate extraction on scanme.nmap.org

### Memory (3 scenarios)
- syn-scan-1k, service-detect-20, tls-cert-10

### I/O (3 scenarios)
- syn-scan-1k, connect-scan-100, service-detect-20

## Key Findings

See PROFILING-ANALYSIS.md for comprehensive analysis.

**Top Bottlenecks:**
1. Packet crafting (12-15% CPU)
2. Checksum calculation (8-10% CPU)
3. Per-packet allocations (32% heap)
4. Suboptimal batch size (sendmmsg 100 vs 200-300)

**Expected Gains:**
- Combined optimization: 15-25% overall speedup
- Memory reduction: 10-20% heap usage
- Throughput increase: 8-15% for stateless scans
EOF
```

## Regression Detection

Compare profiling data across versions:

```bash
# After implementing optimization, re-profile
./profile-scenario.sh --scenario syn-scan-1k --type cpu -- -sS -p 1-1000 127.0.0.1

# Visual comparison
firefox results/flamegraphs/syn-scan-1k-flamegraph.svg &
firefox v0.5.0/cpu/syn-scan-1k-flamegraph.svg &

# Check for:
# - Reduced stack width in hot paths
# - Lower overall sample counts in optimized functions
# - Improved balance across async tasks
```

## Best Practices

### 1. Profiling Environment

- **Dedicated Hardware:** Minimize background processes
- **Release Builds:** Always profile with `--release` (10-100x faster than debug)
- **Realistic Data:** Use production-like targets (localhost for packet crafting, real hosts for network I/O)
- **Multiple Runs:** Average 3-5 runs to account for variance
- **Controlled Load:** Disable CPU frequency scaling for consistency

### 2. Interpretation

- **CPU:** Look for >1% stack width (significant contributors)
- **Memory:** Focus on allocation sites >10% of peak heap
- **I/O:** Prioritize syscalls with high counts or long durations

### 3. Safety

- **Network Impact:** Use localhost or controlled test networks
- **Resource Limits:** Set timeouts to prevent runaway profiling
- **Disk Space:** Massif profiles can be large (100MB-1GB)

## Troubleshooting

See `PROFILING-SETUP.md` for platform-specific issues.

**Common Problems:**

1. **Permission Denied (perf):**
   ```bash
   echo 0 | sudo tee /proc/sys/kernel/perf_event_paranoid
   ```

2. **Flamegraph Not Generated:**
   - Check cargo-flamegraph installation: `cargo install flamegraph`
   - Verify release binary exists: `cargo build --release`

3. **Massif Runs Forever:**
   - Massif adds 10-50x overhead, be patient
   - Reduce scan scope for testing (e.g., 100 ports instead of 1,000)

4. **Strace Output Empty:**
   - Check strace supports `-c` (summary mode)
   - Verify target is reachable
   - Check for stderr redirection issues

## Integration with Sprint 5.5.6

**Next Sprint:** Performance Optimization Implementation

**Workflow:**
1. Select optimization target from PROFILING-ANALYSIS.md
2. Implement optimization (see Sprint 5.5.6 roadmap)
3. Re-profile same scenario
4. Compare results (flamegraph diff, massif reports)
5. Validate expected gains
6. Document in CHANGELOG.md

**Example:**
```bash
# Before optimization (baseline)
./profile-scenario.sh --scenario syn-scan-1k --type cpu -- -sS -p 1-1000 127.0.0.1
# Result: syn-scan-1k-flamegraph.svg (baseline)

# Implement "Increase Batch Size 100→300" optimization
# (modify src/io/mod.rs, SENDMMSG_BATCH_SIZE = 300)

# After optimization (validation)
cargo build --release
./profile-scenario.sh --scenario syn-scan-1k-optimized --type cpu -- -sS -p 1-1000 127.0.0.1
# Result: syn-scan-1k-optimized-flamegraph.svg (comparison)

# Visual diff
firefox results/flamegraphs/syn-scan-1k-flamegraph.svg &
firefox results/flamegraphs/syn-scan-1k-optimized-flamegraph.svg &
```

## References

- **PROFILING-ANALYSIS.md**: Comprehensive analysis with 7 optimization targets
- **IO-ANALYSIS.md**: I/O-specific syscall analysis
- **docs/34-PERFORMANCE-CHARACTERISTICS.md**: Baseline performance metrics
- **docs/31-BENCHMARKING-GUIDE.md**: Benchmarking framework integration

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-09 | Initial profiling framework (Sprint 5.5.5) |

## License

GPL-3.0 (same as ProRT-IP)

---

**Sprint 5.5.5 Deliverable** - Comprehensive profiling infrastructure for data-driven performance optimization.
