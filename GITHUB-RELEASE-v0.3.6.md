# ProRT-IP v0.3.6 - Performance Regression Fix

**Type:** Patch Release (Performance Fix)
**Date:** 2025-10-12
**Status:** Production-Ready ✅

---

## Summary

v0.3.6 resolves a 29% performance regression inadvertently introduced in v0.3.5. Through comprehensive profiling with hyperfine, perf, flamegraph, valgrind, and strace, we identified and eliminated three bottlenecks: debug instrumentation, aggressive polling, and unnecessary preprocessing overhead.

**Result:** 4.6% faster than v0.3.5 baseline + 3x more stable results.

---

## Key Highlights

✅ **4.6% Performance Improvement** - 1K port scans: 6.5ms → 6.2ms
✅ **3x More Stable** - Result variance reduced: 0.9ms → 0.3ms stddev
✅ **Zero Regressions** - All 492 tests passing, zero clippy warnings
✅ **100% Platform Support** - 8/8 architectures building successfully
✅ **Comprehensive Tooling** - 34 files of benchmarking suite (560KB)

---

## Performance Metrics

### Before/After Comparison

| Metric | v0.3.5 Baseline | v0.3.6 | Improvement |
|--------|----------------|--------|-------------|
| **Mean Time** | 6.5ms | 6.2ms | **4.6% faster** |
| **Std Deviation** | 0.9ms | 0.3ms | **3x more stable** |
| **Range** | 5.8-8.4ms | 5.9-6.7ms | **Tighter spread** |

### Benchmark Command

```bash
# Test on your system
hyperfine -w 3 -r 20 'prtip -sT -p 1-1000 127.0.0.1'
```

---

## Root Causes Fixed

### 1. Debug Instrumentation (70% of regression)
- **Issue:** 19 `eprintln!("[TIMING] ...")` statements left in production code
- **Impact:** TTY flushing overhead on every port completion
- **Fix:** Removed all debug statements from scheduler.rs

### 2. Aggressive Polling (20% of regression)
- **Issue:** Progress bar polling every 200µs for small scans
- **Impact:** 5,000 wakeups/second unnecessary for 40-50ms scans
- **Fix:** Optimized intervals (200µs → 1ms for <1K ports, 5x reduction)

### 3. CLI Preprocessing (10% of regression)
- **Issue:** Nmap compatibility layer running even for native syntax
- **Impact:** 40-50µs overhead per invocation
- **Fix:** Added fast path detection to skip unnecessary preprocessing

---

## Investigation Tools

Comprehensive 5-tool profiling suite:

1. **hyperfine** - Statistical benchmarking (20+ runs)
2. **perf stat** - CPU performance counters
3. **perf record + flamegraph** - Call stack visualization
4. **valgrind --tool=massif** - Memory allocation profiling
5. **strace** - Syscall tracing

**Deliverables:** 34 benchmark files (560KB) in `benchmarks/02-Phase4_Final-Bench/`

---

## Platform Support

### All 8 Architectures Building Successfully ✅

| Platform | Target Triple | Use Cases by Platform | Archive | Download |
|----------|--------------|----------------------|---------|----------|
| **Linux x86_64 (glibc)** | x86_64-unknown-linux-gnu | Ubuntu, Debian, Fedora, RHEL, CentOS | tar.gz | [prtip-0.3.6-x86_64-unknown-linux-gnu.tar.gz](https://github.com/doublegate/ProRT-IP/releases/download/v0.3.6/prtip-0.3.6-x86_64-unknown-linux-gnu.tar.gz) |
| **Linux x86_64 (musl)** ✅ FIXED | x86_64-unknown-linux-musl | 🆕 Alpine Linux, Docker/containers, static linking | tar.gz | [prtip-0.3.6-x86_64-unknown-linux-musl.tar.gz](https://github.com/doublegate/ProRT-IP/releases/download/v0.3.6/prtip-0.3.6-x86_64-unknown-linux-musl.tar.gz) |
| **Linux ARM64 (glibc)** ✅ FIXED | aarch64-unknown-linux-gnu | 🆕 Raspberry Pi 4/5, ARM64 servers, AWS Graviton | tar.gz | [prtip-0.3.6-aarch64-unknown-linux-gnu.tar.gz](https://github.com/doublegate/ProRT-IP/releases/download/v0.3.6/prtip-0.3.6-aarch64-unknown-linux-gnu.tar.gz) |
| **Linux ARM64 (musl)** ✅ FIXED | aarch64-unknown-linux-musl | 🆕 ARM64 Alpine, static ARM builds, embedded systems | tar.gz | [prtip-0.3.6-aarch64-unknown-linux-musl.tar.gz](https://github.com/doublegate/ProRT-IP/releases/download/v0.3.6/prtip-0.3.6-aarch64-unknown-linux-musl.tar.gz) |
| **Windows x86_64** | x86_64-pc-windows-msvc | Windows 10/11 (Requires Npcap) | zip | [prtip-0.3.6-x86_64-pc-windows-msvc.zip](https://github.com/doublegate/ProRT-IP/releases/download/v0.3.6/prtip-0.3.6-x86_64-pc-windows-msvc.zip) |
| **macOS Intel** | x86_64-apple-darwin | Intel-based Macs (macOS 11+) | tar.gz | [prtip-0.3.6-x86_64-apple-darwin.tar.gz](https://github.com/doublegate/ProRT-IP/releases/download/v0.3.6/prtip-0.3.6-x86_64-apple-darwin.tar.gz) |
| **macOS Apple Silicon** | aarch64-apple-darwin | M1/M2/M3/M4 Macs (macOS 11+) | tar.gz | [prtip-0.3.6-aarch64-apple-darwin.tar.gz](https://github.com/doublegate/ProRT-IP/releases/download/v0.3.6/prtip-0.3.6-aarch64-apple-darwin.tar.gz) |
| **FreeBSD x86_64** | x86_64-unknown-freebsd | FreeBSD 12+, pfSense, OPNsense | tar.gz | [prtip-0.3.6-x86_64-unknown-freebsd.tar.gz](https://github.com/doublegate/ProRT-IP/releases/download/v0.3.6/prtip-0.3.6-x86_64-unknown-freebsd.tar.gz) |

**✅ FIXED Platforms:** musl libc ioctl type mismatch and ARM64 OpenSSL cross-compilation issues resolved in this release.

---

## Installation

### Quick Install (Linux)

```bash
# Download pre-built binary
wget https://github.com/doublegate/ProRT-IP/releases/download/v0.3.6/prtip-x86_64-unknown-linux-gnu
chmod +x prtip-x86_64-unknown-linux-gnu
sudo mv prtip-x86_64-unknown-linux-gnu /usr/local/bin/prtip

# Verify installation
prtip --version
```

### Build from Source

```bash
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
git checkout v0.3.6
cargo build --release
sudo cp target/release/prtip /usr/local/bin/
```

**Requirements:**
- Rust 1.85+
- libpcap (Linux/macOS) or Npcap (Windows)
- OpenSSL development libraries

See [docs/03-DEV-SETUP.md](https://github.com/doublegate/ProRT-IP/blob/main/docs/03-DEV-SETUP.md) for detailed setup instructions.

---

## Breaking Changes

**None.** This is a pure performance fix with 100% backward compatibility.

All existing scans, scripts, and workflows continue to work without modification.

---

## Testing

### Comprehensive Validation

```bash
# Test Suite
cargo test --workspace
# Result: 492/492 tests passing (100%)

# Code Quality
cargo fmt --all --check  # ✓ Clean
cargo clippy --all-targets --all-features -- -D warnings  # ✓ Zero warnings
cargo build --release  # ✓ Clean (35.5s)
```

### Regression Testing

All scenarios validated with zero regressions:
- 1K/10K/65K port scans
- Network scans (multiple hosts)
- Service detection (`--sV`)
- OS fingerprinting (`-O`)
- All scan types (Connect, SYN, UDP, stealth)
- All output formats (text, JSON, XML, greppable)

---

## Documentation

### New Documents

1. **[RELEASE-NOTES-v0.3.6.md](RELEASE-NOTES-v0.3.6.md)** - Complete technical release notes
2. **[docs/16-REGRESSION-FIX-STRATEGY.md](docs/16-REGRESSION-FIX-STRATEGY.md)** - Root cause analysis (18KB)
3. **[benchmarks/02-Phase4_Final-Bench/](benchmarks/02-Phase4_Final-Bench/)** - Comprehensive benchmarking suite (34 files, 560KB)

### Updated Documents

- [CHANGELOG.md](CHANGELOG.md) - v0.3.6 entry added
- [README.md](README.md) - Performance metrics updated
- [CLAUDE.local.md](CLAUDE.local.md) - Session summary

---

## Known Issues

**None.** v0.3.6 is production-ready with zero known critical bugs.

For bug reports, see: https://github.com/doublegate/ProRT-IP/issues

---

## What's Next?

### Phase 5 Planning (v0.4.0)

**High Priority:**
1. **SSL/TLS Handshake** - Improve service detection 50% → 80%
2. **Idle Scanning** - Anonymous scanning via IP ID exploitation
3. **Lua Plugin System** - Custom service probes with mlua

**Timeline:** v0.4.0 planned for Q1 2026

See [docs/15-PHASE4-COMPLIANCE.md](docs/15-PHASE4-COMPLIANCE.md) for complete Phase 5 roadmap.

---

## Support

- **Documentation:** [docs/](https://github.com/doublegate/ProRT-IP/tree/main/docs)
- **FAQ:** [docs/09-FAQ.md](https://github.com/doublegate/ProRT-IP/blob/main/docs/09-FAQ.md)
- **Issues:** [GitHub Issues](https://github.com/doublegate/ProRT-IP/issues)
- **Discussions:** [GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions)

---

## Contributors

- **DoubleGate** - Performance investigation, fixes, benchmarking suite
- **Claude Code** - Comprehensive analysis, documentation, verification

Special thanks to the Rust community for excellent profiling tools:
hyperfine, perf, flamegraph, valgrind, strace

---

## Full Changelog

See [CHANGELOG.md](CHANGELOG.md#036---2025-10-12) for complete version history.

**Commits:** bf246a0...762a07e (3 commits)
- bf246a0 - fix(performance): Remove debug instrumentation and optimize polling (v0.3.6)
- 762a07e - docs: Comprehensive documentation audit and Phase 4 compliance review
- c45f647 - fix(release): Resolve musl ioctl type mismatch and ARM64 OpenSSL cross-compilation

---

**Recommended Upgrade:** Yes - All users should upgrade for improved performance and stability.

**License:** GPL-3.0 - See [LICENSE](LICENSE) for details.
