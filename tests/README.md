# ProRT-IP Test Infrastructure

Comprehensive testing infrastructure for ProRT-IP v0.3.7.

## Overview

ProRT-IP uses a multi-layered testing strategy:

1. **Unit Tests** - In-crate tests (`crates/*/tests/`)
2. **Integration Tests** - Cross-crate tests (`tests/integration/`)
3. **Performance Tests** - Benchmarks and regression tests (`tests/performance/`)
4. **Fixtures** - Test data and mock targets (`tests/fixtures/`)
5. **Common Utilities** - Shared test code (`tests/common/`)

## Test Statistics (v0.3.7)

- **Total Tests:** 789 (100% passing) ✅
- **Unit Tests:** 492 (in-crate)
- **Integration Tests:** 67 (CLI integration tests)
- **Crate-level Tests:** 230 (cross-crate integration)
- **Coverage:** 61.92% (15,397 / 24,814 lines covered) ✅ **TARGET MET (>60%)**
- **Coverage Tool:** cargo-tarpaulin with HTML reports
- **Benchmarks:** Criterion.rs with v0.3.7 baselines
- **CI/CD:** 7/7 jobs passing on Linux, Windows, macOS, FreeBSD

## Directory Structure

```
tests/
├── README.md                          # This file
├── common/                            # Shared test utilities
│   └── mod.rs                         # Common functions, assertions, helpers
├── fixtures/                          # Test data files
│   ├── sample_targets.json            # Test targets and expected results
│   ├── nmap_compatible_flags.json     # Nmap flag compatibility data
│   ├── expected_outputs.json          # Output format validation patterns
│   └── README.md                      # Fixture documentation
├── performance/                       # Performance benchmarks
│   ├── benchmarks.rs                  # Criterion benchmarks
│   └── README.md                      # Benchmark documentation
└── crates/prtip-cli/tests/           # CLI integration tests
    ├── test_cli_args.rs               # CLI argument parsing (18 tests)
    ├── test_output_formats.rs         # Output format generation (12 tests)
    ├── test_port_parsing.rs           # Port specification parsing (20 tests)
    ├── test_scan_types.rs             # Scan type execution (17 tests)
    ├── integration.rs                 # Existing integration tests (29 tests)
    ├── common/                        # CLI test utilities
    └── fixtures/                      # CLI test fixtures
```

**Note:** Integration tests are located in `crates/prtip-cli/tests/` to have access to the compiled binary.

## Running Tests

### All Tests
```bash
cargo test
```

### Unit Tests Only
```bash
cargo test --lib
```

### Integration Tests
```bash
# Run all CLI integration tests
cargo test --package prtip-cli --tests

# Run specific test suite
cargo test --package prtip-cli --test test_scan_types
cargo test --package prtip-cli --test test_output_formats
cargo test --package prtip-cli --test test_cli_args
cargo test --package prtip-cli --test test_port_parsing
```

### Performance Benchmarks
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench benchmarks
```

### Code Coverage
```bash
# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir code_cov

# Generate HTML + Lcov (for CI)
cargo tarpaulin --config code_cov/tarpaulin.toml

# View report
xdg-open code_cov/tarpaulin-report.html  # Linux
open code_cov/tarpaulin-report.html      # macOS
```

**Current Coverage:** 52.06% (1,919/3,686 lines)
**Target:** 70%+ overall, 90%+ for core modules

See [`code_cov/README.md`](../code_cov/README.md) for detailed coverage information.

### Specific Test Pattern
```bash
cargo test tcp_connect
cargo test nmap_compat
```

## Test Categories

### 1. Unit Tests

**Location:** `crates/*/tests/` and inline `#[cfg(test)]` modules

**Purpose:** Test individual functions and modules in isolation.

**Examples:**
- Packet parsing
- Protocol implementations
- Data structure operations
- Error handling

**Run:** `cargo test --lib`

---

### 2. Integration Tests

**Location:** `crates/prtip-cli/tests/`

**Purpose:** Test CLI functionality, argument parsing, and end-to-end scan execution.

**Total:** 67 integration tests across 4 test suites

#### test_scan_types.rs (17 tests)

Tests all scan types:
- TCP Connect scan (no privileges required)
- TCP SYN scan (with/without privileges)
- UDP scan (protocol payloads)
- Stealth scans (FIN, NULL, Xmas)
- ACK scan (firewall mapping)
- Scan timing and rate limiting
- Fast scan (-F flag)

**Run:** `cargo test --test test_scan_types`

#### test_output_formats.rs (12 tests)

Tests all output formats:
- Text output (human-readable, colorized)
- JSON output (valid JSON, correct schema)
- XML output (valid XML, nmap-compatible)
- Greppable output (-oG format)
- All formats output (-oA flag)
- File permissions and creation

**Run:** `cargo test --test test_output_formats`

#### test_cli_args.rs (18 tests)

Tests CLI argument parsing and nmap compatibility:
- Help and version flags
- Scan type aliases (-sS, -sT, -sU, -sF, -sN, -sX, -sA)
- Port specification (-p, -F)
- Timing templates (-T0 through -T5)
- Output flags (-oN, -oX, -oG)
- Verbose flag (-v)
- Mixed syntax (nmap + ProRT-IP flags)
- Invalid argument handling

**Run:** `cargo test --test test_cli_args`

#### test_port_parsing.rs (20 tests)

Tests port specification parsing:
- Single port (-p 80)
- Port range (-p 1-1000)
- Port list (-p 22,80,443)
- Mixed specification (-p 22,80-85,443)
- All ports (-p-)
- Top ports (-F, --top-ports)
- Boundary values and error handling

**Run:** `cargo test --test test_port_parsing`

---

### 3. Performance Tests

**Location:** `tests/performance/`

**Purpose:** Validate performance targets and detect regressions.

#### benchmarks.rs (Criterion)

Microbenchmarks for critical paths:
- Port scanning (100, 1K, 10K ports)
- nmap-service-probes parsing (187 probes)
- nmap-os-db parsing (2600+ entries)
- Rate limiter throughput
- Packet construction (SYN, UDP, etc.)

**Run:** `cargo bench`

#### regression_tests.rs

Regression detection:
- Store baseline benchmarks in git (JSON)
- Compare current run vs baseline
- Fail CI if >10% regression
- Allow opt-in to update baseline

**Run:** `cargo test --test regression_tests`

---

### 4. Fixtures

**Location:** `tests/fixtures/`

**Purpose:** Test data and mock targets.

**Contents:**
- `sample_nmap_outputs/` - Example nmap outputs for comparison
- `test_service_probes.txt` - Subset of nmap-service-probes
- `mock_targets.json` - Mock target configurations

---

### 5. Common Utilities

**Location:** `tests/common/`

**Purpose:** Shared test code to reduce duplication.

#### mod.rs

Module declarations and re-exports.

#### mock_server.rs

Spawn test servers for integration tests:
- HTTP server (simple echo server)
- SSH server (accept connections, return banner)
- DNS server (respond to queries)
- Auto-cleanup on test completion
- Random port allocation (avoid conflicts)

**Usage:**
```rust
use common::mock_server::spawn_http_server;

#[test]
fn test_http_detection() {
    let server = spawn_http_server(8080);
    // ... test code ...
    // server automatically cleaned up on drop
}
```

#### assertions.rs

Custom assertions for test quality:

**Available Assertions:**
- `assert_scan_result(result, expected_open, expected_closed)`
- `assert_service_detected(result, port, expected_service)`
- `assert_json_valid(output)` - Validates JSON schema
- `assert_xml_valid(output)` - Validates XML schema
- `assert_greppable_format(output)` - Validates greppable format

**Usage:**
```rust
use common::assertions::assert_scan_result;

#[test]
fn test_localhost_scan() {
    let result = scan("127.0.0.1", &[80, 443]);
    assert_scan_result(&result, vec![80], vec![443]);
}
```

---

## Writing New Tests

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = create_test_input();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }

    #[test]
    #[should_panic(expected = "error message")]
    fn test_error_handling() {
        function_that_should_panic();
    }
}
```

### Integration Test Template

```rust
// tests/integration/test_new_feature.rs

use prtip_core::*;
use prtip_scanner::*;

#[test]
fn test_new_feature() {
    // Setup
    let config = ScanConfig::default();

    // Execute
    let result = scan_with_config(&config, "127.0.0.1").unwrap();

    // Verify
    assert!(!result.is_empty());
}

#[test]
#[ignore] // Requires root/CAP_NET_RAW
fn test_privileged_feature() {
    // Test code that requires privileges
}
```

### Benchmark Template

```rust
// tests/performance/benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_feature(c: &mut Criterion) {
    c.bench_function("feature name", |b| {
        b.iter(|| {
            // Code to benchmark
            feature_function(black_box(input))
        });
    });
}

criterion_group!(benches, benchmark_feature);
criterion_main!(benches);
```

---

## Test Configuration

### Cargo.toml

```toml
[dev-dependencies]
criterion = "0.5"
mockall = "0.12"
proptest = "1.4"
```

### test-config.toml

```toml
[integration]
# Skip privileged tests if not root
require_root = false
# Timeout for each test (seconds)
timeout = 30

[performance]
# Baseline file for regression detection
baseline = "tests/performance/baseline.json"
# Regression threshold (%)
regression_threshold = 10.0

[fixtures]
# Path to test data
fixtures_dir = "tests/fixtures"
```

---

## CI/CD Integration

### GitHub Actions

Tests run automatically on:
- Every push to main
- Every pull request
- Scheduled (weekly)

**Platforms:**
- Linux (Ubuntu 22.04)
- Windows (Server 2022)
- macOS (14)
- FreeBSD (via cross)

**Test Matrix:**
```yaml
- name: Run tests
  run: cargo test --all-targets --all-features

- name: Run benchmarks (no-run)
  run: cargo bench --no-run

- name: Run integration tests
  run: |
    ./scripts/test-nmap-compat.sh
    ./scripts/run-integration-tests.sh --quick
```

---

## Test Best Practices

### 1. Arrange-Act-Assert (AAA)

Structure tests clearly:
```rust
#[test]
fn test_example() {
    // Arrange: Setup test data
    let input = create_input();

    // Act: Execute the function
    let result = function(input);

    // Assert: Verify the result
    assert_eq!(result, expected);
}
```

### 2. Descriptive Test Names

Use clear, descriptive names:
```rust
#[test]
fn test_tcp_connect_scan_detects_open_port() { ... }

#[test]
fn test_invalid_ip_returns_error() { ... }
```

### 3. Test One Thing

Each test should verify one behavior:
```rust
// Good
#[test]
fn test_port_parsing_accepts_valid_range() { ... }

#[test]
fn test_port_parsing_rejects_invalid_range() { ... }

// Bad - tests multiple things
#[test]
fn test_port_parsing() { ... }
```

### 4. Use Test Fixtures

Reduce duplication with shared test data:
```rust
fn create_test_config() -> ScanConfig {
    ScanConfig {
        ports: vec![80, 443],
        timeout: Duration::from_secs(1),
        ..Default::default()
    }
}

#[test]
fn test_with_fixture() {
    let config = create_test_config();
    // ... test code ...
}
```

### 5. Clean Up Resources

Always clean up after tests:
```rust
#[test]
fn test_with_temp_file() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    // ... test code ...
    // temp automatically deleted on drop
}
```

### 6. Platform-Specific Tests

Use conditional compilation:
```rust
#[test]
#[cfg(target_os = "linux")]
fn test_linux_specific_feature() { ... }

#[test]
#[ignore] // Requires root
fn test_privileged_feature() { ... }
```

---

## Debugging Tests

### Run Specific Test
```bash
cargo test test_name -- --nocapture
```

### Show Test Output
```bash
cargo test -- --nocapture --test-threads=1
```

### Run Ignored Tests
```bash
cargo test -- --ignored
```

### Run with Logging
```bash
RUST_LOG=debug cargo test
```

### Debug in VS Code

Add to `.vscode/launch.json`:
```json
{
    "type": "lldb",
    "request": "launch",
    "name": "Debug test",
    "cargo": {
        "args": ["test", "--no-run", "--lib"],
        "filter": {
            "name": "test_name",
            "kind": "test"
        }
    }
}
```

---

## Coverage Reports

### Install tarpaulin
```bash
cargo install cargo-tarpaulin
```

### Generate Coverage
```bash
# HTML report
cargo tarpaulin --out Html

# XML for CI
cargo tarpaulin --out Xml

# Specific crate
cargo tarpaulin -p prtip-core
```

### Coverage Targets

- **Overall:** >80%
- **Core modules:** >90%
- **Critical paths:** 100%

---

## Performance Testing

### Quick Performance Check
```bash
./scripts/run-benchmarks.sh --quick
```

### Full Benchmark Suite
```bash
./scripts/run-benchmarks.sh --profile
```

### Compare with Baseline
```bash
./scripts/run-benchmarks.sh --compare v0.3.5
```

---

## Troubleshooting

### Test Timeout
```rust
#[test]
#[timeout(std::time::Duration::from_secs(10))]
fn test_slow_operation() { ... }
```

### Flaky Tests

Avoid flaky tests:
- Don't rely on timing
- Don't use external services
- Use deterministic random seeds
- Clean up resources properly

### Permission Errors

Some tests require privileges:
```bash
# Run as root (Linux)
sudo -E cargo test

# Or use capabilities
sudo setcap cap_net_raw=eip target/debug/prtip
```

---

## Contributing

When adding new tests:

1. Follow AAA pattern (Arrange-Act-Assert)
2. Use descriptive names
3. Test one thing per test
4. Add integration tests for user-facing features
5. Add benchmarks for performance-critical code
6. Update this README with new test categories

---

**Last Updated:** 2025-10-12 | **Version:** v0.3.6 | **Tests:** 492/492 (100%)
