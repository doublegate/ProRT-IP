# Testing Infrastructure

Comprehensive guide to ProRT-IP's testing infrastructure, coverage strategy, and quality assurance processes.

**Last Updated:** 2025-10-13 | **Version:** v0.3.7 | **Coverage:** 61.92%

---

## Table of Contents

1. [Overview](#overview)
2. [Testing Philosophy](#testing-philosophy)
3. [Test Organization](#test-organization)
4. [Test Categories](#test-categories)
5. [Code Coverage Infrastructure](#code-coverage-infrastructure)
6. [Benchmark Infrastructure](#benchmark-infrastructure)
7. [Running Tests](#running-tests)
8. [CI/CD Integration](#cicd-integration)
9. [Writing New Tests](#writing-new-tests)
10. [Metrics and Achievements](#metrics-and-achievements)
11. [Future Work](#future-work)

---

## Overview

ProRT-IP maintains comprehensive testing infrastructure across multiple levels:

| Test Level | Count | Purpose | Coverage Target |
|------------|-------|---------|-----------------|
| **Unit Tests** | 492 | Test individual functions/modules | >60% lines |
| **Integration Tests** | 67 | Test CLI interface end-to-end | All user-facing features |
| **Benchmarks** | 8 suites | Performance regression detection | No regressions >5% |
| **Crate-Level Tests** | 230 | Cross-crate integration testing | Core workflows |
| **Total** | **789** | Comprehensive quality assurance | **61.92% achieved** |

**Test Execution Time:**
- Quick smoke test (`cargo test --lib`): ~30 seconds
- Full test suite (`cargo test --workspace`): ~2-3 minutes
- Integration tests only: ~45 seconds
- Benchmarks: ~5-8 minutes (statistical validity)

---

## Testing Philosophy

### Core Principles

1. **Test Behavior, Not Implementation**
   - Focus on public APIs and user-visible behavior
   - Avoid testing internal implementation details
   - Refactoring should not break tests

2. **Pragmatic Coverage Targets**
   - **60% coverage**: Industry standard baseline (✅ achieved: 61.92%)
   - **80% coverage**: Aspirational goal for critical paths
   - **100% coverage**: Unrealistic and counterproductive

3. **Test Categories by Value**
   - **High Value**: Parsing, validation, configuration (100% coverage)
   - **Medium Value**: Business logic, state management (70-80% coverage)
   - **Low Value**: Trivial getters, display formatting (50% coverage OK)
   - **Skip**: Async network I/O without mocks (validated via integration tests)

4. **Statistical Validity**
   - Benchmarks use 100 samples (Criterion default)
   - Integration tests run 3x for reliability
   - Flaky tests are fixed immediately (not ignored)

### What We Test

✅ **Do Test:**
- Input parsing and validation (CLI arguments, ports, IPs, CIDR)
- Configuration loading and defaults
- Service detection probe parsing
- Banner grabber protocol implementations
- Output format generation (JSON, XML, greppable, text)
- Error handling and edge cases
- Cross-platform compatibility (via CI matrix)

❌ **Don't Test:**
- Raw socket operations (requires root, platform-specific)
- Network I/O without mocks (unreliable, slow)
- Third-party library internals (trust dependencies)
- Trivial code (single-line functions)

### When Coverage is Not the Goal

**Acceptable Low Coverage Scenarios:**
1. **Async Network Code**: Requires complex mocking infrastructure (future work)
2. **Platform-Specific Code**: Tested via CI/CD on actual platforms
3. **Display/Formatting**: Low-risk code, validated via integration tests
4. **Error Paths**: Some error conditions are hard to trigger artificially

---

## Test Organization

### Directory Structure

```
ProRT-IP/
├── crates/
│   ├── prtip-core/
│   │   └── src/
│   │       ├── lib.rs                 # Unit tests in #[cfg(test)] modules
│   │       ├── port.rs                # Port parsing tests
│   │       └── config.rs              # Configuration tests
│   ├── prtip-network/
│   │   └── src/
│   │       ├── capture.rs             # Packet capture tests
│   │       └── privilege.rs           # Privilege tests (Unix-specific)
│   ├── prtip-scanner/
│   │   └── src/
│   │       ├── banner_grabber.rs      # Protocol tests (26 tests)
│   │       ├── service_detector.rs    # Service detection tests (19 tests)
│   │       └── scheduler.rs           # Scheduler tests
│   └── prtip-cli/
│       ├── src/
│       │   ├── args.rs                # CLI parsing tests
│       │   └── output.rs              # Output formatting tests
│       └── tests/                     # Integration tests (67 tests)
│           ├── integration.rs         # Core CLI tests
│           ├── test_cli_args.rs       # Argument parsing (18 tests)
│           ├── test_output_formats.rs # Output formats (12 tests)
│           ├── test_port_parsing.rs   # Port parsing (20 tests)
│           ├── test_scan_types.rs     # Scan execution (17 tests)
│           ├── common/                # Shared test utilities
│           │   └── mod.rs             # Helper functions (203 lines)
│           └── fixtures/              # Test data
│               ├── sample_scan.json
│               ├── sample_services.json
│               └── README.md
├── tests/
│   ├── common/
│   │   └── mod.rs                     # Workspace-level test utilities
│   ├── performance/
│   │   ├── benchmarks.rs              # Criterion benchmarks (8 suites)
│   │   └── README.md                  # Benchmark documentation
│   └── fixtures/                      # Shared test data
├── code_cov/
│   ├── tarpaulin.toml                 # Coverage configuration
│   ├── tarpaulin-report.html          # HTML coverage report
│   └── README.md                      # Coverage workflow guide
└── benchmarks/
    └── baselines/
        ├── v0.3.7/                    # Criterion baseline data
        └── README.md                  # Baseline usage guide
```

### Test File Conventions

**Unit Tests (in-crate):**
```rust
// src/port.rs
pub fn parse_port(s: &str) -> Result<u16, ParseError> {
    // Implementation...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_port() {
        assert_eq!(parse_port("80").unwrap(), 80);
    }
}
```

**Integration Tests (cross-crate):**
```rust
// crates/prtip-cli/tests/test_cli_args.rs
mod common;  // Import shared utilities

#[test]
fn test_help_flag() {
    let output = common::run_prtip(&["--help"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("USAGE:"));
}
```

**Benchmarks (performance):**
```rust
// tests/performance/benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_port_parsing(c: &mut Criterion) {
    c.bench_function("port_parsing/single_port", |b| {
        b.iter(|| parse_port("80"))
    });
}

criterion_group!(benches, bench_port_parsing);
criterion_main!(benches);
```

---

## Test Categories

### 1. CLI Argument Parsing (18 tests)

**Location:** `crates/prtip-cli/tests/test_cli_args.rs`

**Coverage:**
- Nmap-compatible flags (`-sS`, `-sT`, `-sU`, `-p`, `-F`, etc.)
- ProRT-IP native syntax (`--scan-type`, `--ports`, `--timing`)
- Mixed syntax (nmap + native flags together)
- Invalid combinations (e.g., `-sS` with `-sT`)
- Help and version flags
- Privilege-aware execution (skip SYN scans without root)

**Example Test:**
```rust
#[test]
fn test_nmap_syn_scan_flag() {
    // Test: -sS flag should be recognized as SYN scan
    let output = common::run_prtip(&["-sS", "-p", "80", "127.0.0.1"]);

    if !common::has_elevated_privileges() {
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("SYN scan requires elevated privileges"));
    } else {
        common::assert_scan_success(&output);
    }
}
```

**Why Important:**
- CLI is the primary user interface
- Nmap compatibility is a key feature (20+ flags)
- Parsing errors cause poor user experience

**Coverage Target:** 100% (critical path)

---

### 2. Output Format Validation (12 tests)

**Location:** `crates/prtip-cli/tests/test_output_formats.rs`

**Coverage:**
- Text output (human-readable, colorized)
- JSON output (machine-readable, structured)
- XML output (nmap-compatible)
- Greppable output (nmap .gnmap format)
- File output (`-oN`, `-oX`, `-oG`, `-oJ`)
- Stdout vs file redirection
- Output with zero results (edge case)

**Example Test:**
```rust
#[test]
fn test_json_output_format() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    let output_path = temp.path().to_str().unwrap();

    let output = common::run_prtip(&[
        "-p", "80",
        "127.0.0.1",
        "-oJ", output_path,
    ]);

    common::assert_scan_success(&output);

    // Verify JSON is valid and contains expected fields
    let json_content = std::fs::read_to_string(output_path).unwrap();
    let json: serde_json::Value = common::parse_json_output(json_content.as_bytes());

    assert!(json["scan_results"].is_array());
    assert!(json["summary"]["total_ports_scanned"].as_u64().unwrap() > 0);
}
```

**Why Important:**
- Output is how users consume results
- JSON/XML enable automation and integration
- Greppable format supports scripting workflows

**Coverage Target:** 100% (critical path)

---

### 3. Port Parsing Edge Cases (20 tests)

**Location:** `crates/prtip-cli/tests/test_port_parsing.rs`

**Coverage:**
- Single port (`-p 80`)
- Port ranges (`-p 1-1000`)
- Port lists (`-p 80,443,8080`)
- Mixed syntax (`-p 80,443,1000-2000`)
- Top ports (`-F`, `--top-ports 100`)
- Special values (`-p-` for all 65535 ports)
- CIDR ranges (`192.168.1.0/24`)
- IPv6 addresses
- Invalid ports (0, 65536+, negative)
- Empty/whitespace input

**Example Test:**
```rust
#[test]
fn test_port_range_parsing() {
    let output = common::run_prtip(&["-p", "1-100", "127.0.0.1"]);
    common::assert_scan_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should scan 100 ports
    assert!(stdout.contains("100 ports"));
}

#[test]
fn test_invalid_port_high_value() {
    let output = common::run_prtip(&["-p", "99999", "127.0.0.1"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("port") && stderr.contains("invalid"));
}
```

**Why Important:**
- Port parsing errors are common user mistakes
- Edge cases (0, 65535, ranges) need careful handling
- Security: prevent integer overflow/underflow

**Coverage Target:** 100% (critical path, security-sensitive)

---

### 4. Scan Type Execution (17 tests)

**Location:** `crates/prtip-cli/tests/test_scan_types.rs`

**Coverage:**
- TCP Connect scan (`-sT`, default)
- SYN scan (`-sS`, requires root)
- UDP scan (`-sU`)
- Stealth scans (`-sF`, `-sN`, `-sX`)
- ACK scan (`-sA`)
- Privilege detection (skip root-only tests)
- Error handling (permission denied)
- Scan with service detection (`-sV`)
- Scan with OS detection (`-O`)

**Example Test:**
```rust
#[test]
fn test_syn_scan_without_privileges() {
    if common::has_elevated_privileges() {
        // Skip if running as root (test is for non-root behavior)
        return;
    }

    let output = common::run_prtip(&["-sS", "-p", "80", "127.0.0.1"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("requires elevated privileges") ||
            stderr.contains("permission denied"));
}

#[test]
fn test_tcp_connect_scan() {
    // TCP Connect should work without root
    let output = common::run_prtip(&["-sT", "-p", "80,443", "127.0.0.1"]);
    common::assert_scan_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scan complete") || stdout.contains("ports scanned"));
}
```

**Why Important:**
- Scan types are core functionality
- Privilege handling is security-critical
- Different scan types have different behaviors

**Coverage Target:** 90% (skip raw socket internals)

---

### 5. Banner Grabber Protocols (26 tests)

**Location:** `crates/prtip-scanner/src/banner_grabber.rs`

**Coverage:**
- HTTP/HTTPS banner grabbing
- SSH version detection
- FTP welcome messages
- SMTP greeting
- DNS version queries (BIND)
- SNMP community strings
- POP3/IMAP greetings
- MySQL/PostgreSQL identification
- Error handling (timeout, connection refused)
- Malformed responses

**Example Test:**
```rust
#[test]
fn test_http_banner_parsing() {
    let banner = b"HTTP/1.1 200 OK\r\nServer: nginx/1.18.0\r\n\r\n";
    let result = parse_http_banner(banner);

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.protocol, "HTTP");
    assert_eq!(info.version, "1.1");
    assert_eq!(info.product, Some("nginx".to_string()));
    assert_eq!(info.product_version, Some("1.18.0".to_string()));
}

#[test]
fn test_ssh_version_detection() {
    let banner = b"SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.1\r\n";
    let result = parse_ssh_banner(banner);

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.protocol, "SSH");
    assert_eq!(info.version, "2.0");
    assert_eq!(info.product, Some("OpenSSH".to_string()));
    assert_eq!(info.product_version, Some("8.9p1".to_string()));
}
```

**Why Important:**
- Banner grabbing is how we identify services
- Protocol parsing is error-prone (real-world data is messy)
- Security: prevent buffer overflows, injection attacks

**Coverage Target:** 80% (protocol complexity varies)

---

### 6. Service Detection Configuration (19 tests)

**Location:** `crates/prtip-scanner/src/service_detector.rs`

**Coverage:**
- Service probe loading (nmap-service-probes format)
- Probe matching (regex patterns)
- SSL/TLS detection
- NULL probe (send nothing, read banner)
- Intensity levels (0-9)
- Fallback strategies (protocol defaults)
- Error handling (malformed probes)
- Performance (probe ordering optimization)

**Example Test:**
```rust
#[test]
fn test_probe_loading_from_embedded_data() {
    let detector = ServiceDetector::new(/* intensity */ 7);

    // Should load 187 embedded probes
    assert!(detector.probe_count() >= 187);
}

#[test]
fn test_http_probe_matching() {
    let banner = b"HTTP/1.1 200 OK\r\nServer: Apache/2.4.41\r\n\r\n";
    let result = match_service_probe(banner, "http");

    assert!(result.is_some());
    let service = result.unwrap();
    assert_eq!(service.name, "http");
    assert_eq!(service.product, Some("Apache".to_string()));
    assert_eq!(service.version, Some("2.4.41".to_string()));
}
```

**Why Important:**
- Service detection is a key differentiator (vs simple port scanning)
- Probe parsing is complex (embedded 187-probe database)
- Performance: probe ordering affects scan time (2x speedup possible)

**Coverage Target:** 70% (complex state machine, many edge cases)

---

### 7. Configuration Management (15+ tests)

**Location:** Various (`prtip-core/src/config.rs`, `prtip-cli/src/args.rs`)

**Coverage:**
- Default values (sensible defaults)
- Environment variable overrides
- Config file loading (TOML format)
- CLI argument precedence (CLI > env > config > defaults)
- Validation (rate limits, timeouts, parallelism)
- Platform-specific defaults (Windows vs Linux)

**Example Test:**
```rust
#[test]
fn test_default_config() {
    let config = Config::default();

    assert_eq!(config.timing, Timing::Normal);  // T3
    assert_eq!(config.parallelism, num_cpus::get());
    assert_eq!(config.timeout_ms, 3000);  // 3 seconds
    assert_eq!(config.retries, 1);
}

#[test]
fn test_cli_overrides_defaults() {
    let config = Config::from_args(&["-T4", "--timeout", "5000"]);

    assert_eq!(config.timing, Timing::Aggressive);  // T4
    assert_eq!(config.timeout_ms, 5000);  // Overridden
}
```

**Why Important:**
- Configuration determines scan behavior
- Defaults must be safe (not too aggressive)
- Validation prevents DoS and resource exhaustion

**Coverage Target:** 90% (critical for correctness)

---

### 8. Error Handling (20+ tests)

**Location:** Various (all crates)

**Coverage:**
- Network errors (connection refused, timeout)
- Permission errors (privilege escalation failures)
- Input validation errors (invalid IP, port, CIDR)
- Resource exhaustion (too many file descriptors)
- Platform-specific errors (Windows vs Unix)
- User-friendly error messages

**Example Test:**
```rust
#[test]
fn test_invalid_ip_address() {
    let output = common::run_prtip(&["-p", "80", "999.999.999.999"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid IP address"));
}

#[test]
fn test_permission_denied_raw_socket() {
    if common::has_elevated_privileges() {
        return;  // Skip if running as root
    }

    let output = common::run_prtip(&["-sS", "-p", "80", "127.0.0.1"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("permission") || stderr.contains("privilege"));
}
```

**Why Important:**
- Error messages guide users to solutions
- Security: prevent information leakage in errors
- Robustness: handle all failure modes gracefully

**Coverage Target:** 70% (some error paths are hard to trigger)

---

## Code Coverage Infrastructure

### Overview

ProRT-IP uses **cargo-tarpaulin** for code coverage analysis with HTML report generation.

**Key Metrics (v0.3.7):**
- **Overall Coverage:** 61.92% (1,821/2,941 lines)
- **Target Achieved:** ✅ 61.92% exceeds 60% industry standard baseline
- **Coverage by Crate:** See table below

### Coverage by Crate

| Crate | Coverage | Lines Covered | Total Lines | Priority |
|-------|----------|---------------|-------------|----------|
| **prtip-core** | ~65% | 450/692 | 692 | HIGH (critical path) |
| **prtip-network** | ~55% | 380/691 | 691 | MEDIUM (async I/O) |
| **prtip-scanner** | ~62% | 601/969 | 969 | HIGH (core scanning) |
| **prtip-cli** | ~66% | 390/589 | 589 | HIGH (user interface) |
| **Total** | **61.92%** | **1,821/2,941** | **2,941** | **BASELINE** |

*Note: Exact per-crate breakdowns available in `code_cov/tarpaulin-report.html`*

### Tarpaulin Configuration

**Location:** `code_cov/tarpaulin.toml`

```toml
[global]
# Output formats (HTML for viewing, Lcov for CI integration)
out = ["Html", "Lcov"]

# Output directory (relative to project root)
output-dir = "code_cov"

# Exclude patterns
exclude = [
    "crates/*/tests/**",     # Don't measure test code coverage
    "crates/*/benches/**",   # Don't measure benchmark code
    "tests/**",              # Workspace-level tests excluded
    "code_ref/**",           # Reference code excluded
]

# Exclude lines matching patterns
exclude-lines = [
    "#\\[derive\\(",         # Derived traits
    "unreachable!\\(",       # Unreachable code markers
    "unimplemented!\\(",     # Unimplemented placeholders
]

# Follow symbolic links (for workspace structure)
follow-symlinks = true

# Include all workspace members
workspace = true

# Run all tests (unit + integration)
run-types = ["Tests"]

# Timeout for test execution (2 minutes)
timeout = "120s"

# Use all available CPU cores
parallel = true
```

### Excluded Code and Rationale

**Directories Excluded:**
1. **`code_ref/`**: Reference implementations (RustScan, RustScan, nmap samples) not part of ProRT-IP
2. **`tests/`**: Test code itself (measuring test coverage of tests is circular)
3. **`benches/`**: Benchmark code (performance tests, not functional)

**Code Patterns Excluded:**
1. **Derived Traits**: Auto-generated code (e.g., `#[derive(Debug)]`)
2. **Unreachable Code**: Explicit markers (`unreachable!()`)
3. **Unimplemented**: Placeholders for future work (`unimplemented!()`)

**Why Not Measure Everything?**
- **Diminishing Returns**: 80%+ coverage requires mocking complex async I/O (high effort, low value)
- **False Confidence**: 100% coverage doesn't guarantee bug-free code
- **Maintenance Burden**: Tests for trivial code increase maintenance without improving quality

### Coverage Report Workflow

**Generate HTML Report:**
```bash
cd code_cov
cargo tarpaulin --out Html

# Open report in browser
xdg-open tarpaulin-report.html  # Linux
open tarpaulin-report.html      # macOS
start tarpaulin-report.html     # Windows
```

**Generate Lcov for CI:**
```bash
cd code_cov
cargo tarpaulin --out Lcov

# Upload to coverage service (e.g., Codecov, Coveralls)
bash <(curl -s https://codecov.io/bash) -f code_cov/lcov.info
```

**Interpret Report:**
- **Green Lines**: Executed by tests (covered)
- **Red Lines**: Not executed by tests (uncovered)
- **Yellow Lines**: Partially covered (e.g., branch not taken)
- **Gray Lines**: Excluded from coverage (comments, whitespace)

**Focus Areas:**
1. **Red Critical Paths**: Uncovered lines in parsing, validation, error handling → Add tests
2. **Yellow Branches**: Partially covered conditionals → Add tests for missing branches
3. **Green Non-Critical**: Well-covered code → No action needed

---

## Benchmark Infrastructure

### Overview

ProRT-IP uses **Criterion.rs** for statistical benchmarking with baseline comparison support.

**Benchmark Suites (8 total):**

| Suite | Tests | Purpose | Mean Time (v0.3.7) |
|-------|-------|---------|-------------------|
| **binary_startup** | 2 | Measure CLI overhead | 2.2ms |
| **port_parsing** | 3 | Parsing performance | 1.7ns (sub-nanosecond) |
| **localhost_scan** | 3 | Full scan execution | 5.3ms |
| **output_formats** | 2 | Output generation | 2.1-5.1ms |
| **Total** | **8** | **Performance regression detection** | **Baseline established** |

### Benchmark Details

**1. binary_startup (2 tests)**
- `help`: `prtip --help` execution time
- `version`: `prtip --version` execution time
- **Purpose**: Measure binary startup overhead (should be <5ms)

**2. port_parsing (3 tests)**
- `single_port`: Parse "80"
- `port_range_small`: Parse "1-100"
- `port_list`: Parse "80,443,8080"
- **Purpose**: Ensure parsing is negligible overhead (<1µs)

**3. localhost_scan (3 tests)**
- `single_port`: Scan 127.0.0.1:80
- `three_ports`: Scan 127.0.0.1:80,443,8080
- `port_range_10`: Scan 127.0.0.1:1-10
- **Purpose**: Baseline scan performance (includes TCP handshake + OS overhead)

**4. output_formats (2 tests)**
- `text_output`: Human-readable text format
- `json_output`: Machine-readable JSON format
- **Purpose**: Ensure output formatting doesn't dominate scan time

### Baseline Management

**Baseline Storage:** `benchmarks/baselines/v0.3.7/`

**Workflow:**
```bash
# 1. Run benchmarks and save baseline (already done for v0.3.7)
cargo bench --bench benchmarks -- --save-baseline v0.3.7

# 2. Make code changes...

# 3. Compare against baseline (detect regressions)
cargo bench --bench benchmarks -- --baseline v0.3.7

# 4. Criterion shows performance deltas:
#    "change: +5.0%" = 5% slower (regression)
#    "change: -5.0%" = 5% faster (improvement)
#    "change: +0.1%" = within noise (no significant change)
```

**Interpreting Results:**
- **Change < 1%**: Likely noise, no action needed
- **Change 1-5%**: Minor regression/improvement, investigate if consistent
- **Change 5-10%**: Moderate regression/improvement, review code changes
- **Change > 10%**: Major regression/improvement, requires investigation

**See Also:** `benchmarks/baselines/README.md` for comprehensive baseline usage guide.

---

## Running Tests

### Quick Smoke Test (30 seconds)

Test core functionality without full integration tests:

```bash
cargo test --lib
```

**What it runs:**
- All unit tests (492 tests)
- Skips integration tests
- Skips benchmarks

**When to use:**
- Pre-commit validation
- Quick sanity check after code changes
- CI fast feedback loop

---

### Full Test Suite (2-3 minutes)

Run all tests including integration:

```bash
cargo test --workspace
```

**What it runs:**
- All unit tests (492 tests)
- All integration tests (67 tests)
- All crate-level tests (230 tests)
- Total: 789 tests

**When to use:**
- Pre-push validation
- Before opening PR
- Release candidate validation

---

### Integration Tests Only (45 seconds)

Test CLI interface end-to-end:

```bash
cargo test -p prtip-cli
```

**What it runs:**
- CLI argument parsing (18 tests)
- Output format validation (12 tests)
- Port parsing edge cases (20 tests)
- Scan type execution (17 tests)
- Total: 67 integration tests

**When to use:**
- CLI changes
- Output format changes
- User-facing feature validation

---

### Specific Test Pattern

Run tests matching a pattern:

```bash
# Test all HTTP-related tests
cargo test http

# Test all parsing tests
cargo test parse

# Test specific function
cargo test test_parse_single_port
```

**When to use:**
- Focused development
- Debugging specific functionality
- Iterative test-driven development

---

### Coverage Report (2-3 minutes)

Generate HTML coverage report:

```bash
cd code_cov
cargo tarpaulin --out Html

# Open in browser
xdg-open tarpaulin-report.html  # Linux
open tarpaulin-report.html      # macOS
```

**What it generates:**
- `tarpaulin-report.html`: Interactive coverage report
- `lcov.info`: Machine-readable coverage data (CI integration)

**When to use:**
- Identify untested code paths
- Validate coverage targets
- Pre-release coverage audit

---

### Benchmarks (5-8 minutes)

Run performance benchmarks:

```bash
# Run all benchmarks
cargo bench --bench benchmarks

# Run specific benchmark suite
cargo bench --bench benchmarks port_parsing

# Compare against baseline (detect regressions)
cargo bench --bench benchmarks -- --baseline v0.3.7
```

**What it generates:**
- `target/criterion/`: Detailed benchmark results
- `target/criterion/report/index.html`: Interactive report

**When to use:**
- Performance optimization work
- Release candidate validation
- Detect performance regressions

---

### Verbose Output

See detailed test output:

```bash
# Show all test output (including passed tests)
cargo test -- --nocapture --test-threads=1

# Show timing for each test
cargo test -- --show-output
```

**When to use:**
- Debugging test failures
- Understanding test execution order
- Investigating flaky tests

---

## CI/CD Integration

### GitHub Actions Workflow

**Location:** `.github/workflows/test.yml`

**Current CI Checks:**
1. **Format Check:** `cargo fmt --all --check`
2. **Clippy Lints:** `cargo clippy --all-targets --all-features -- -D warnings`
3. **Unit Tests:** `cargo test --lib` (fast feedback)
4. **Integration Tests:** `cargo test --workspace` (full validation)
5. **MSRV Check:** Verify Rust 1.85 compatibility
6. **Security Audit:** `cargo audit` for known vulnerabilities

**Platform Matrix:**
- Ubuntu 22.04 (Linux)
- Windows 2022 (Windows)
- macOS 12 (macOS)

**Status:** 7/7 checks passing (100% reliability)

### Coverage Reporting (Future)

**Planned Integration:**
```yaml
- name: Generate Coverage
  run: |
    cargo install cargo-tarpaulin
    cd code_cov
    cargo tarpaulin --out Lcov

- name: Upload to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: code_cov/lcov.info
    flags: unittests
    name: codecov-umbrella
```

**Benefits:**
- Coverage trend visualization
- PR coverage diff (shows coverage change per PR)
- Automated coverage gate (fail PR if coverage drops >5%)

### Performance Regression Checks (Future)

**Planned Workflow:**
```yaml
- name: Restore Baseline
  run: |
    git checkout main -- benchmarks/baselines/v0.3.7/
    cp -r benchmarks/baselines/v0.3.7/* target/criterion/

- name: Run Benchmarks
  run: cargo bench --bench benchmarks -- --baseline v0.3.7

- name: Check for Regressions
  run: ./scripts/check-performance-regression.sh
```

**Benefits:**
- Automated performance regression detection
- Prevent slowdowns from merging
- Historical performance tracking

---

## Writing New Tests

### Unit Test Template

```rust
// src/module.rs

pub fn function_to_test(input: &str) -> Result<Output, Error> {
    // Implementation...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_happy_path() {
        // Arrange
        let input = "valid input";

        // Act
        let result = function_to_test(input);

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.field, expected_value);
    }

    #[test]
    fn test_function_error_case() {
        // Arrange
        let input = "invalid input";

        // Act
        let result = function_to_test(input);

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::ValidationError(_)));
    }

    #[test]
    #[should_panic(expected = "panic message")]
    fn test_function_panic_case() {
        function_to_test("panic input");
    }
}
```

### Integration Test Template

```rust
// crates/prtip-cli/tests/test_feature.rs

mod common;

#[test]
fn test_feature_success() {
    // Arrange
    let args = &["--flag", "value", "target"];

    // Act
    let output = common::run_prtip(args);

    // Assert
    common::assert_scan_success(&output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("expected output"));
}

#[test]
fn test_feature_privilege_aware() {
    if !common::has_elevated_privileges() {
        // Skip test if not running as root
        return;
    }

    // Test behavior that requires elevated privileges
    let output = common::run_prtip(&["-sS", "-p", "80", "127.0.0.1"]);
    common::assert_scan_success(&output);
}
```

### Benchmark Template

```rust
// tests/performance/benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_feature(c: &mut Criterion) {
    // Setup (excluded from timing)
    let input = setup_test_data();

    c.bench_function("feature_name", |b| {
        b.iter(|| {
            // Code to benchmark (wrapped in black_box to prevent optimization)
            black_box(function_to_benchmark(black_box(&input)))
        })
    });
}

criterion_group!(benches, bench_feature);
criterion_main!(benches);
```

### Test Naming Conventions

**Good Test Names:**
- `test_parse_single_port` (descriptive, clear intent)
- `test_http_banner_parsing` (module + behavior)
- `test_invalid_port_high_value` (error case explicit)

**Bad Test Names:**
- `test_1` (meaningless)
- `test_port` (ambiguous)
- `test_parse` (too generic)

**Pattern:** `test_<module>_<behavior>_<condition>`

---

## Metrics and Achievements

### v0.3.7 Baseline

**Test Metrics:**
- **Total Tests:** 789 tests
- **Unit Tests:** 492 (62% of total)
- **Integration Tests:** 67 (8% of total)
- **Crate-Level Tests:** 230 (30% of total)
- **Benchmarks:** 8 suites (performance validation)

**Coverage Metrics:**
- **Overall Coverage:** 61.92% (1,821/2,941 lines)
- **Target:** 60% industry standard baseline (✅ ACHIEVED)
- **Improvement:** +9.86 percentage points since v0.3.6 (52.06% → 61.92%)

**Test Additions (v0.3.7):**
- **Total New Tests:** +297 tests (+60% increase)
- **Banner Grabber:** +26 tests (HTTP, SSH, FTP, SMTP, DNS, SNMP)
- **Service Detection:** +19 tests (config, probes, SSL)
- **CLI Argument Parsing:** 18 tests (nmap compatibility)
- **Output Formats:** 12 tests (JSON, XML, greppable, text)
- **Port Parsing:** 20 tests (CIDR, ranges, edge cases)
- **Scan Types:** 17 tests (privilege-aware)

**Quality Metrics:**
- **Pass Rate:** 100% (789/789 tests passing)
- **Flaky Tests:** 0 (all tests reliable)
- **CI Reliability:** 7/7 checks passing (100%)
- **Benchmark Stability:** <5% variance (statistical validity)

**Infrastructure Achievements:**
- ✅ Comprehensive integration test suite (67 tests)
- ✅ Code coverage infrastructure (cargo-tarpaulin + HTML reports)
- ✅ Benchmark baseline system (Criterion + git-tracked baselines)
- ✅ Privilege-aware testing (auto-skip root-only tests)
- ✅ Fixture management (JSON test data)
- ✅ Shared test utilities (203-line common module)

### Historical Comparison

| Version | Total Tests | Coverage | Key Additions |
|---------|-------------|----------|---------------|
| v0.3.0 | ~200 | ~40% | Basic unit tests |
| v0.3.5 | 492 | 52.06% | Core functionality tests |
| v0.3.6 | 492 | 52.06% | Performance fixes (no test changes) |
| **v0.3.7** | **789** | **61.92%** | **+297 tests, coverage infrastructure** |

**Growth:** 294% test increase, 54.8% coverage increase (v0.3.0 → v0.3.7)

---

## Future Work

### Phase 5 Testing Enhancements

**Priority:** HIGH (Enable 65%+ coverage target)

**1. Async Network I/O Mocking (HIGH)**
- **Challenge:** Current network tests skip async I/O (requires complex mocking)
- **Solution:** Implement mock network layer with `tokio_test` and `mockall`
- **Impact:** +10-15% coverage (network crate currently 55% → 70%+)
- **Effort:** 2-3 weeks (significant infrastructure work)

**Example:**
```rust
#[tokio::test]
async fn test_syn_scan_with_mock_socket() {
    let mock_socket = MockRawSocket::new()
        .expect_send()
        .returning(|_| Ok(()))
        .expect_recv()
        .returning(|_| Ok(syn_ack_response()));

    let result = syn_scan(mock_socket, "127.0.0.1", 80).await;
    assert!(result.is_open());
}
```

**2. Real Network Scan Scenarios (MEDIUM)**
- **Challenge:** Integration tests use localhost only (limited real-world validation)
- **Solution:** Add integration tests against controlled test targets
- **Impact:** Increased confidence in production behavior
- **Effort:** 1-2 weeks

**Scenarios:**
- Filtered ports (firewall dropping packets)
- RST responses (closed ports)
- Slow responses (timeout handling)
- Network partitions (connection loss)

**3. Property-Based Testing (MEDIUM)**
- **Challenge:** Edge cases hard to identify manually
- **Solution:** Use `proptest` or `quickcheck` for randomized input generation
- **Impact:** Discover unexpected edge cases, increase robustness
- **Effort:** 1 week

**Example:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_port_parsing_never_panics(input in ".*") {
        // Should never panic, only return Ok or Err
        let _ = parse_port(&input);
    }

    #[test]
    fn test_port_range_always_sorted(start in 1..=65535u16, end in 1..=65535u16) {
        if let Ok(range) = parse_port_range(start, end) {
            assert!(range.is_sorted());
        }
    }
}
```

**4. Mutation Testing (LOW)**
- **Challenge:** High coverage doesn't guarantee tests catch bugs
- **Solution:** Use `cargo-mutants` to verify tests detect intentional bugs
- **Impact:** Improve test quality (not just quantity)
- **Effort:** 1 week setup + ongoing

**Example:**
```bash
cargo install cargo-mutants
cargo mutants --test-tool=nextest

# Output: "Caught 87/100 mutants (87% mutation score)"
# Uncaught mutants = tests that don't detect bugs
```

**5. Fuzz Testing (LOW)**
- **Challenge:** Malformed network packets could cause crashes
- **Solution:** Use `cargo-fuzz` to test packet parsing with random data
- **Impact:** Discover security vulnerabilities (buffer overflows, panics)
- **Effort:** 2-3 days

**Example:**
```rust
#[macro_use] extern crate libfuzzer_sys;
use prtip_network::parse_tcp_packet;

fuzz_target!(|data: &[u8]| {
    // Should never panic or crash
    let _ = parse_tcp_packet(data);
});
```

**6. CI Performance Regression Checks (HIGH)**
- **Challenge:** No automated performance regression detection
- **Solution:** Integrate benchmarks into GitHub Actions
- **Impact:** Prevent performance regressions from merging
- **Effort:** 1-2 days

**Workflow:**
```yaml
- name: Check Performance
  run: |
    cargo bench --bench benchmarks -- --baseline v0.3.7
    ./scripts/check-performance-regression.sh
```

**7. Coverage Reporting Integration (MEDIUM)**
- **Challenge:** No PR coverage diff (can't see coverage impact of changes)
- **Solution:** Integrate Codecov or Coveralls
- **Impact:** Automated coverage trend tracking, PR gates
- **Effort:** 1 day

**8. Expand Service Detection Tests (MEDIUM)**
- **Challenge:** Only 19 tests for 187 embedded probes
- **Solution:** Add tests for more protocols (SSL/TLS, HTTP/2, etc.)
- **Impact:** Increase confidence in service detection accuracy
- **Effort:** 1 week

**9. Windows-Specific Tests (LOW)**
- **Challenge:** Some tests are Linux-specific
- **Solution:** Add Windows-specific integration tests (Npcap, Admin privileges)
- **Impact:** Better Windows support validation
- **Effort:** 1 week

**10. Test Data Generators (LOW)**
- **Challenge:** Fixture management is manual
- **Solution:** Add scripts to generate realistic test data
- **Impact:** Easier test maintenance, more realistic scenarios
- **Effort:** 3-5 days

---

## Summary

ProRT-IP's testing infrastructure provides comprehensive quality assurance:

✅ **789 tests** across unit, integration, and benchmark levels
✅ **61.92% coverage** exceeding 60% industry standard baseline
✅ **100% CI reliability** with 7/7 checks passing
✅ **Statistical benchmarking** with baseline comparison
✅ **Privilege-aware testing** for security-critical functionality
✅ **Multi-platform validation** (Linux, macOS, Windows)

**Key Strengths:**
- Comprehensive CLI integration tests (67 tests)
- Protocol parsing validation (26+ tests)
- Service detection coverage (19+ tests)
- Performance regression detection (Criterion baselines)
- Code coverage infrastructure (cargo-tarpaulin + HTML reports)

**Future Focus:**
- Async network I/O mocking (+10-15% coverage)
- Real network scenario tests (production confidence)
- Property-based testing (edge case discovery)
- CI performance regression checks (automated gates)

**For Contributors:**
- See test templates in "Writing New Tests" section
- Follow naming conventions: `test_<module>_<behavior>_<condition>`
- Run `cargo test --workspace` before submitting PRs
- Maintain >60% coverage (check `code_cov/tarpaulin-report.html`)

---

**Last Updated:** 2025-10-13
**Maintainer:** ProRT-IP Contributors
**Questions:** See `docs/09-FAQ.md` or open a GitHub issue
