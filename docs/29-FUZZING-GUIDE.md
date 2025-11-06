# Fuzzing Guide - ProRT-IP

**Version:** 1.0.0
**Last Updated:** 2025-11-05
**Sprint:** 5.7 - Fuzz Testing Infrastructure

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Fuzzing Infrastructure](#fuzzing-infrastructure)
4. [Running Fuzzers Locally](#running-fuzzers-locally)
5. [Adding New Fuzz Targets](#adding-new-fuzz-targets)
6. [Corpus Management](#corpus-management)
7. [CI/CD Fuzzing](#cicd-fuzzing)
8. [Interpreting Results](#interpreting-results)
9. [Troubleshooting](#troubleshooting)
10. [Best Practices](#best-practices)

---

## Overview

ProRT-IP uses **cargo-fuzz** (powered by libFuzzer) to perform coverage-guided fuzzing of packet parsers. Fuzzing automatically discovers crashes, panics, and unexpected behavior by generating millions of test inputs.

### What is Fuzzing?

Fuzzing is an automated software testing technique that provides invalid, unexpected, or random data as input to a program. Coverage-guided fuzzing uses runtime instrumentation to discover new code paths and maximize test coverage.

### Why Fuzz ProRT-IP?

As a network scanner processing untrusted network data, ProRT-IP must handle:
- Malformed packets from misconfigured devices
- Malicious packets from attackers
- Edge cases (truncated, oversized, corrupted data)
- Protocol violations and ambiguities

**Fuzzing ensures:** Parser robustness, no panics on malformed input, security (no exploitable crashes), comprehensive edge case coverage.

### Fuzzing Targets

ProRT-IP has **5 fuzzing targets** covering critical parsers:

| Target | Component | Purpose |
|--------|-----------|---------|
| `fuzz_tcp_parser` | TCP packet parser | SYN/ACK/FIN/RST packets, options, flags |
| `fuzz_udp_parser` | UDP packet parser | DNS, SNMP, NetBIOS payloads |
| `fuzz_ipv6_parser` | IPv6 packet parser | Headers, extension headers, payloads |
| `fuzz_icmpv6_parser` | ICMPv6 packet parser | Echo, ND, Router Discovery |
| `fuzz_tls_parser` | TLS certificate parser | X.509 DER/ASN.1 structures |

---

## Quick Start

### Prerequisites

```bash
# Install Rust nightly (required for cargo-fuzz)
rustup install nightly
rustup default nightly

# Install cargo-fuzz
cargo install cargo-fuzz --version 0.13.1

# Verify installation
cargo +nightly fuzz --version
```

### Run Your First Fuzzer

```bash
cd /path/to/ProRT-IP

# List available fuzz targets
cargo +nightly fuzz list

# Run TCP parser fuzzer for 60 seconds
cargo +nightly fuzz run fuzz_tcp_parser -- -max_total_time=60

# Run with more verbosity
cargo +nightly fuzz run fuzz_tcp_parser -- -max_total_time=60 -verbosity=2
```

### Expected Output

```
INFO: Running with entropic power schedule (0xFF, 100).
INFO: Seed: 1234567890
INFO: 100 files found in fuzz/corpus/fuzz_tcp_parser
#1000000 NEW    cov: 234 corp: 150 exec/s: 50000 rss: 45Mb
#2000000 NEW    cov: 235 corp: 151 exec/s: 51000 rss: 46Mb
...
Done 60000000 runs in 60 seconds
```

**No crashes?** Great! The parser is robust.
**Crash found?** See [Interpreting Results](#interpreting-results).

---

## Fuzzing Infrastructure

### Directory Structure

```
ProRT-IP/
├── fuzz/
│   ├── Cargo.toml                 # Fuzzing dependencies
│   ├── fuzz_targets/              # Fuzzing harnesses
│   │   ├── fuzz_tcp_parser.rs     # TCP fuzzer (149 lines)
│   │   ├── fuzz_udp_parser.rs     # UDP fuzzer (132 lines)
│   │   ├── fuzz_ipv6_parser.rs    # IPv6 fuzzer (217 lines)
│   │   ├── fuzz_icmpv6_parser.rs  # ICMPv6 fuzzer (200 lines)
│   │   └── fuzz_tls_parser.rs     # TLS fuzzer (232 lines)
│   ├── corpus/                    # Seed corpus files (806 files, 3.3MB)
│   │   ├── fuzz_tcp_parser/       # 100+ TCP seeds
│   │   ├── fuzz_udp_parser/       # 80+ UDP seeds
│   │   ├── fuzz_ipv6_parser/      # 100+ IPv6 seeds
│   │   ├── fuzz_icmpv6_parser/    # 80+ ICMPv6 seeds
│   │   ├── fuzz_tls_parser/       # 100+ TLS seeds
│   │   └── README.md              # Corpus documentation
│   ├── scripts/
│   │   └── generate_corpus.sh     # Corpus generation (460 seeds)
│   └── artifacts/                 # Crash artifacts (created on crash)
└── .github/workflows/fuzz.yml     # CI/CD nightly fuzzing
```

### Fuzzing Architecture

#### Structure-Aware Fuzzing

ProRT-IP uses **structure-aware fuzzing** with the `arbitrary` crate to generate valid-ish protocol packets:

```rust
#[derive(Arbitrary, Debug)]
struct FuzzTcpInput {
    source_port: u16,         // Valid port (0-65535)
    dest_port: u16,
    sequence: u32,
    flags: u8,                // TCP flags
    payload: Vec<u8>,         // Variable length
    // ... more fields
}
```

This approach:
- Generates realistic protocol structures
- Explores protocol-specific edge cases
- Finds deeper bugs than pure random fuzzing
- Maintains backward compatibility with unstructured fuzzing

#### Coverage-Guided Fuzzing

libFuzzer uses **coverage feedback** to guide input generation:

1. Run input through target
2. Measure code coverage (basic blocks visited)
3. If new coverage found, save input to corpus
4. Mutate interesting inputs to discover more coverage
5. Repeat millions of times

---

## Running Fuzzers Locally

### Basic Commands

```bash
# Run fuzzer indefinitely (Ctrl+C to stop)
cargo +nightly fuzz run fuzz_tcp_parser

# Run for specific duration (seconds)
cargo +nightly fuzz run fuzz_tcp_parser -- -max_total_time=600

# Run for specific number of iterations
cargo +nightly fuzz run fuzz_tcp_parser -- -runs=1000000

# Run with multiple jobs (parallel fuzzing)
cargo +nightly fuzz run fuzz_tcp_parser -- -jobs=4 -workers=4

# Run with custom corpus directory
cargo +nightly fuzz run fuzz_tcp_parser -- corpus/custom/
```

### Advanced Options

```bash
# Limit memory per job (MB)
cargo +nightly fuzz run fuzz_tcp_parser -- -rss_limit_mb=2048

# Set maximum input length (bytes)
cargo +nightly fuzz run fuzz_tcp_parser -- -max_len=1500

# Print statistics every N seconds
cargo +nightly fuzz run fuzz_tcp_parser -- -print_final_stats=1

# Increase verbosity (0=quiet, 1=default, 2=verbose)
cargo +nightly fuzz run fuzz_tcp_parser -- -verbosity=2

# Use dictionary for more intelligent mutations
cargo +nightly fuzz run fuzz_tcp_parser -- -dict=fuzz/tcp.dict
```

### Fuzzing All Targets

```bash
# Run all targets for 10 minutes each
for target in $(cargo +nightly fuzz list); do
    echo "Fuzzing $target..."
    timeout 600 cargo +nightly fuzz run $target -- -max_total_time=600
done
```

---

## Adding New Fuzz Targets

### Step 1: Create Fuzz Target

```bash
# Create new fuzz target
cargo +nightly fuzz add fuzz_new_parser
```

This creates `fuzz/fuzz_targets/fuzz_new_parser.rs`:

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Your fuzzing code here
    // Example: parse data and ensure no panic
    if let Ok(parsed) = your_parser::parse(data) {
        // Exercise parsed data
        let _ = parsed.some_method();
    }
});
```

### Step 2: Implement Fuzzing Logic

Choose between **unstructured** or **structure-aware** fuzzing:

#### Unstructured Fuzzing (Simple)

```rust
fuzz_target!(|data: &[u8]| {
    // Parse raw bytes
    let _ = TcpPacket::new(data);
});
```

**Pros:** Simple, fast to write
**Cons:** Shallow coverage, mostly tests error handling

#### Structure-Aware Fuzzing (Recommended)

```rust
use arbitrary::{Arbitrary, Unstructured};

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    port: u16,
    flags: u8,
    payload: Vec<u8>,
}

fuzz_target!(|input: FuzzInput| {
    let packet = build_packet(&input);
    if let Some(parsed) = TcpPacket::new(&packet) {
        // Exercise all methods
        let _ = parsed.get_source();
        let _ = parsed.get_flags();
        let _ = parsed.payload();
    }
});
```

**Pros:** Deep coverage, finds protocol bugs
**Cons:** More complex, requires packet building logic

### Step 3: Create Corpus Directory

```bash
mkdir -p fuzz/corpus/fuzz_new_parser

# Add initial seeds
echo -ne "\x00\x50\x00\x50..." > fuzz/corpus/fuzz_new_parser/seed1
```

### Step 4: Test Fuzz Target

```bash
# Build (check compilation)
cargo +nightly fuzz build fuzz_new_parser

# Quick validation (10 seconds)
cargo +nightly fuzz run fuzz_new_parser -- -max_total_time=10

# Full run (10 minutes)
cargo +nightly fuzz run fuzz_new_parser -- -max_total_time=600
```

### Step 5: Add to CI/CD

Edit `.github/workflows/fuzz.yml`:

```yaml
strategy:
  matrix:
    target:
      - fuzz_tcp_parser
      - fuzz_udp_parser
      - fuzz_ipv6_parser
      - fuzz_icmpv6_parser
      - fuzz_tls_parser
      - fuzz_new_parser  # <-- Add here
```

---

## Corpus Management

### What is a Corpus?

A **corpus** is a set of input files that:
- Maximize code coverage
- Provide starting points for mutations
- Serve as regression tests

### Generating Corpus

```bash
# Generate 460 seeds automatically
./fuzz/scripts/generate_corpus.sh

# Verify generation
find fuzz/corpus -type f | wc -l  # Should be 460+
```

### Adding Manual Seeds

```bash
# From hex dump
echo -ne "\x60\x00\x00\x00\x00\x00\x06\x40..." > fuzz/corpus/fuzz_ipv6_parser/my_seed

# From pcap
tcpdump -r capture.pcap -w - | head -c 100 > fuzz/corpus/fuzz_tcp_parser/pcap_seed

# From binary file
xxd -r -p input.hex > fuzz/corpus/fuzz_tcp_parser/hex_seed
```

### Corpus Minimization

Remove redundant seeds that don't add coverage:

```bash
# Minimize single target
cargo +nightly fuzz cmin fuzz_tcp_parser

# Minimize all targets
for target in $(cargo +nightly fuzz list); do
    cargo +nightly fuzz cmin $target
done
```

**Before:**
```
fuzz/corpus/fuzz_tcp_parser/: 500 files, 2.5MB
```

**After:**
```
fuzz/corpus/fuzz_tcp_parser/: 150 files, 800KB
```

### Merging Corpora

Combine multiple corpus directories:

```bash
# Merge corpus from another run
cargo +nightly fuzz merge fuzz_tcp_parser corpus1/ corpus2/

# Merge and minimize
cargo +nightly fuzz merge --minimize fuzz_tcp_parser corpus1/ corpus2/
```

### Corpus Statistics

```bash
# Count seeds per target
find fuzz/corpus -mindepth 1 -maxdepth 1 -type d -exec sh -c \
    'echo "$(basename {}): $(find {} -type f | wc -l) seeds"' \;

# Total corpus size
du -sh fuzz/corpus/

# Size per target
du -sh fuzz/corpus/*/
```

---

## CI/CD Fuzzing

### GitHub Actions Workflow

ProRT-IP runs **nightly fuzzing** via GitHub Actions:

- **Schedule:** Every day at 02:00 UTC
- **Duration:** 10 minutes per target (configurable)
- **Targets:** All 5 fuzz targets in parallel
- **Artifacts:** Crash dumps uploaded for 90 days
- **Alerts:** Workflow fails if crashes found

### Manual Trigger

Trigger fuzzing manually from GitHub UI:

1. Go to **Actions** → **Fuzz Testing**
2. Click **Run workflow**
3. Configure options:
   - Duration: `600` (seconds, default 10 min)
   - Targets: `all` or `fuzz_tcp_parser,fuzz_udp_parser`
4. Click **Run workflow**

### Viewing Results

**Workflow Summary:**
- Executions per second
- New corpus entries discovered
- Coverage statistics
- Crash status (Yes/No)

**Crash Artifacts:**
- Downloaded from **Actions** → **Workflow run** → **Artifacts**
- File name: `fuzz-crashes-<target>-<run_number>.zip`
- Contains: Crash inputs, stack traces, metadata

### Integrating Crashes

```bash
# Download crash artifact
unzip fuzz-crashes-fuzz_tcp_parser-123.zip -d crashes/

# Reproduce crash locally
cargo +nightly fuzz run fuzz_tcp_parser crashes/crash-*

# Add as regression test
cp crashes/crash-12345 tests/regression/tcp_crash_12345.bin

# Create unit test
#[test]
fn test_tcp_crash_12345() {
    let data = include_bytes!("regression/tcp_crash_12345.bin");
    assert!(TcpPacket::new(data).is_none());  // Should not panic
}
```

---

## Interpreting Results

### Normal Output (No Crashes)

```
INFO: Running with entropic power schedule (0xFF, 100).
INFO: Seed: 1699392847
INFO: 100 files found in fuzz/corpus/fuzz_tcp_parser
#1000000 NEW    cov: 234 corp: 150 exec/s: 50000 rss: 45Mb
#2000000 NEW    cov: 235 corp: 151 exec/s: 51000 rss: 46Mb
Done 60000000 runs in 60 seconds
```

**Key Metrics:**
- `cov: 234` - Code coverage (basic blocks)
- `corp: 150` - Corpus size (interesting inputs)
- `exec/s: 50000` - Executions per second
- `rss: 45Mb` - Memory usage

**Interpreting:**
- Coverage increasing → Fuzzer finding new paths ✅
- Coverage stable → Target well-explored ✅
- High exec/s → Efficient fuzzing ✅
- Low exec/s → Target slow, consider optimizing ⚠️

### Crash Detected

```
==1234==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x...
    #0 0x... in TcpPacket::get_options ...
    #1 0x... in fuzz_target::{{closure}} ...

SUMMARY: AddressSanitizer: heap-buffer-overflow ...
MS: 5 CrossOver-EraseBytes-InsertByte-ChangeBit-ShuffleBytes
artifact_prefix='fuzz/artifacts/'; Test unit written to crash-abc123
```

**What Happened:**
- Fuzzer found input causing crash (heap buffer overflow)
- Crash saved to `fuzz/artifacts/fuzz_tcp_parser/crash-abc123`
- Stack trace shows where crash occurred

**Next Steps:**
1. Reproduce: `cargo +nightly fuzz run fuzz_tcp_parser fuzz/artifacts/fuzz_tcp_parser/crash-abc123`
2. Debug: Add `RUST_BACKTRACE=full` for full trace
3. Fix: Patch parser to handle case
4. Verify: Rerun fuzzer with fix
5. Regress: Add crash input as unit test

### Timeout

```
ALARM: working on the last Unit for 60 seconds
  and the timeout value is 60
==1234==ERROR: libFuzzer: timeout after 60 seconds
```

**What Happened:**
- Input caused infinite loop or very slow execution
- Fuzzer terminated after timeout

**Next Steps:**
1. Check for infinite loops in parser
2. Add complexity limits (max recursion depth, max iterations)
3. Increase timeout if legitimate: `-timeout=120`

### Out of Memory

```
==1234==ERROR: AddressSanitizer: requested allocation size 0x... exceeds maximum supported size
```

**What Happened:**
- Input caused excessive memory allocation
- Fuzzer exceeded memory limit

**Next Steps:**
1. Add input size limits: `-max_len=4096`
2. Limit memory: `-rss_limit_mb=2048`
3. Check for unbounded allocations in parser

---

## Troubleshooting

### Fuzzer Not Finding Crashes

**Symptoms:** Fuzzer runs but coverage plateaus, no crashes found
**Causes:**
- Corpus too small
- Parser too simple
- Target not instrumented

**Solutions:**
```bash
# Add more diverse seeds
./fuzz/scripts/generate_corpus.sh

# Increase fuzzing duration
cargo +nightly fuzz run fuzz_tcp_parser -- -max_total_time=3600  # 1 hour

# Check coverage
cargo +nightly fuzz coverage fuzz_tcp_parser

# Verify instrumentation
cargo +nightly fuzz build --verbose fuzz_tcp_parser
```

### Fuzzer Runs Too Slow

**Symptoms:** < 10,000 exec/s
**Causes:**
- Heavy computations in hot path
- Large corpus seeds
- Debug assertions enabled

**Solutions:**
```bash
# Profile fuzzer
cargo +nightly fuzz run fuzz_tcp_parser -- -max_total_time=60 -print_pcs=1

# Optimize corpus
cargo +nightly fuzz cmin fuzz_tcp_parser

# Use release mode (default, but verify)
cargo +nightly fuzz build --release fuzz_tcp_parser
```

### Compilation Errors

**Symptoms:** `cargo +nightly fuzz build` fails
**Causes:**
- Missing dependencies
- API changes
- Nightly version incompatibility

**Solutions:**
```bash
# Update nightly
rustup update nightly

# Clean build
cargo clean
cargo +nightly fuzz build fuzz_tcp_parser

# Check Cargo.toml dependencies
cat fuzz/Cargo.toml
```

### Corpus Not Being Used

**Symptoms:** `INFO: 0 files found in corpus`
**Causes:**
- Corpus directory missing
- Wrong path
- Empty corpus

**Solutions:**
```bash
# Check corpus directory exists
ls -lh fuzz/corpus/fuzz_tcp_parser/

# Generate corpus if missing
./fuzz/scripts/generate_corpus.sh

# Verify seed count
find fuzz/corpus/fuzz_tcp_parser -type f | wc -l
```

### CI/CD Workflow Failing

**Symptoms:** GitHub Actions fuzzing workflow fails
**Causes:**
- Crash found (expected failure)
- Timeout
- Out of memory

**Solutions:**
```bash
# Download crash artifacts from workflow run
# Reproduce locally
cargo +nightly fuzz run fuzz_tcp_parser artifacts/crash-*

# Fix bug
# Commit fix
# Rerun workflow
```

---

## Best Practices

### 1. Fuzz Early, Fuzz Often

- Run fuzzers during development, not just before release
- Integrate fuzzing into CI/CD (already done via GitHub Actions)
- Fuzz for at least 10 minutes per target, daily

### 2. Maintain High-Quality Corpus

- **Diverse:** Include valid, invalid, and edge case inputs
- **Small:** Keep seeds < 5KB for fast fuzzing
- **Minimized:** Remove redundant seeds regularly (`cargo fuzz cmin`)
- **Documented:** Note why interesting seeds were added

### 3. Structure-Aware Fuzzing

- Use `arbitrary` crate for protocol-aware fuzzing
- Generate realistic packets, not pure random bytes
- Finds deeper bugs than unstructured fuzzing

### 4. Monitor Coverage

```bash
# Generate coverage report
cargo +nightly fuzz coverage fuzz_tcp_parser

# View HTML report
open fuzz/coverage/fuzz_tcp_parser/html/index.html
```

### 5. Regression Tests from Crashes

Every crash should become a unit test:

```rust
#[test]
fn test_tcp_crash_12345() {
    let data = include_bytes!("../fuzz/artifacts/crash-12345");
    // Should not panic (fixed)
    assert!(TcpPacket::new(data).is_none());
}
```

### 6. Set Reasonable Limits

- **Max input length:** `-max_len=1500` (typical MTU)
- **Memory limit:** `-rss_limit_mb=2048` (prevent OOM)
- **Timeout:** `-timeout=60` (catch infinite loops)

### 7. Use Sanitizers

cargo-fuzz uses **AddressSanitizer** by default, which detects:
- Buffer overflows
- Use-after-free
- Memory leaks
- Double frees

**Also available:**
- MemorySanitizer (`-s memory`)
- ThreadSanitizer (`-s thread`)
- LeakSanitizer (`-s leak`)

### 8. Parallelize for Speed

```bash
# Use all CPU cores
cargo +nightly fuzz run fuzz_tcp_parser -- -jobs=$(nproc) -workers=$(nproc)
```

### 9. Continuous Fuzzing

- Let fuzzers run overnight
- Use dedicated fuzzing servers
- Integrate with OSS-Fuzz for continuous coverage

### 10. Document Findings

When crash found:
1. Create GitHub issue with crash input
2. Assign severity (critical/high/medium/low)
3. Add crash input to corpus
4. Create regression test
5. Document root cause in commit message

---

## References

### Official Documentation

- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [cargo-fuzz Book](https://rust-fuzz.github.io/book/)
- [Rust Fuzz GitHub](https://github.com/rust-fuzz)

### Tutorials

- [Fuzzing Rust with cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [Structure-Aware Fuzzing with Arbitrary](https://rust-fuzz.github.io/book/cargo-fuzz/structure-aware-fuzzing.html)
- [OSS-Fuzz Integration Guide](https://google.github.io/oss-fuzz/getting-started/new-project-guide/)

### Related ProRT-IP Documentation

- [06-TESTING.md](06-TESTING.md) - Testing strategy
- [08-SECURITY.md](08-SECURITY.md) - Security practices
- [28-CI-CD-COVERAGE.md](28-CI-CD-COVERAGE.md) - CI/CD coverage automation

---

## Appendix: Fuzzing Checklist

Before marking fuzzing complete:

- [ ] All 5 fuzz targets compile (`cargo +nightly fuzz build`)
- [ ] All targets run without immediate crashes (`-max_total_time=300`)
- [ ] Corpus generated (460+ seeds)
- [ ] Corpus minimized (`cargo fuzz cmin`)
- [ ] CI/CD workflow operational (`.github/workflows/fuzz.yml`)
- [ ] Coverage measured (`cargo fuzz coverage`)
- [ ] Crashes triaged and fixed (if any)
- [ ] Regression tests added for crashes
- [ ] Documentation complete (`docs/29-FUZZING-GUIDE.md`)

---

**Last Updated:** 2025-11-05
**Sprint:** 5.7 - Fuzz Testing Infrastructure
**Status:** Complete ✅
