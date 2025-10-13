# ProRT-IP Test Infrastructure

Comprehensive testing infrastructure for ProRT-IP v0.3.6.

## Overview

ProRT-IP uses a multi-layered testing strategy:

1. **Unit Tests** - In-crate tests (`crates/*/tests/`)
2. **Integration Tests** - Cross-crate tests (`tests/integration/`)
3. **Performance Tests** - Benchmarks and regression tests (`tests/performance/`)
4. **Fixtures** - Test data and mock targets (`tests/fixtures/`)
5. **Common Utilities** - Shared test code (`tests/common/`)

## Test Statistics

- **Total Tests:** 492 (100% passing)
- **Unit Tests:** ~450 (in-crate)
- **Integration Tests:** ~42 (cross-crate)
- **Coverage:** >80% overall, >90% core modules
- **CI/CD:** All tests run on Linux, Windows, macOS, FreeBSD

## Directory Structure

```
tests/
├── README.md                      # This file
├── integration/                   # Integration tests
│   ├── test_scan_types.rs        # All 7 scan types
│   ├── test_output_formats.rs    # Text, JSON, XML, greppable
│   └── test_cli_compatibility.rs # Nmap flag compatibility
├── performance/                   # Performance tests
│   ├── benchmarks.rs              # Criterion benchmarks
│   └── regression_tests.rs        # Performance regression detection
├── fixtures/                      # Test data
│   ├── sample_nmap_outputs/
│   └── test_service_probes.txt
└── common/                        # Shared utilities
    ├── mod.rs
    ├── mock_server.rs             # Test server spawning
    └── assertions.rs              # Custom test assertions
```

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
cargo test --test test_scan_types
cargo test --test test_output_formats
cargo test --test test_cli_compatibility
```

### Performance Benchmarks
```bash
cargo bench
```

### With Coverage
```bash
cargo tarpaulin --out Html
```

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

**Location:** `tests/integration/`

**Purpose:** Test cross-crate functionality and user-facing features.

#### test_scan_types.rs

Tests all 7 scan types:
- TCP Connect (baseline)
- SYN (requires root/CAP_NET_RAW)
- UDP (common protocols)
- FIN, NULL, Xmas (stealth)
- ACK (firewall mapping)

**Run:** `cargo test --test test_scan_types`

#### test_output_formats.rs

Tests all output formats:
- Text (human-readable, colorized)
- JSON (valid JSON, correct schema)
- XML (valid XML, nmap-compatible)
- Greppable (correct format, parseable)

**Run:** `cargo test --test test_output_formats`

#### test_cli_compatibility.rs

Tests nmap CLI compatibility (20+ flags):
- Scan type aliases (-sS, -sT, -sN, -sF, -sX)
- Port specification (-F, --top-ports, -p)
- Output modes (-oN, -oX, -oG)
- Verbosity (-v, -vv, -vvv)
- Mixed syntax (nmap + ProRT-IP)

**Run:** `cargo test --test test_cli_compatibility`

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
