# Testing

Comprehensive testing guide for ProRT-IP contributors covering unit testing, integration testing, property-based testing, coverage goals, and CI/CD integration.

---

## Overview

Testing is critical for ProRT-IP due to:

- **Security Implications:** Bugs could enable network attacks or scanner exploitation
- **Cross-Platform Complexity:** Must work correctly on Linux, Windows, macOS
- **Performance Requirements:** Must maintain 1M+ pps without degradation
- **Protocol Correctness:** Malformed packets lead to inaccurate results

**Current Test Metrics (v0.5.2):**

| Metric | Value | Status |
|--------|-------|--------|
| **Total Tests** | 2,111 | 100% passing |
| **Line Coverage** | 54.92% | ✅ Above 50% target |
| **Integration Tests** | 150+ tests | End-to-end scenarios |
| **Fuzz Tests** | 5 targets, 230M+ executions | 0 crashes |
| **CI/CD** | 9/9 workflows | All passing |

---

## Testing Philosophy

### 1. Test-Driven Development (TDD) for Core Features

Write tests **before** implementation for critical components (packet crafting, state machines, detection engines):

**TDD Workflow:**

```rust
// Step 1: Write failing test
#[test]
fn test_tcp_syn_packet_crafting() {
    let packet = TcpPacketBuilder::new()
        .source(Ipv4Addr::new(10, 0, 0, 1), 12345)
        .destination(Ipv4Addr::new(10, 0, 0, 2), 80)
        .flags(TcpFlags::SYN)
        .build()
        .expect("packet building failed");

    assert_eq!(packet.get_flags(), TcpFlags::SYN);
    assert!(verify_tcp_checksum(&packet));
}

// Step 2: Implement feature to make test pass
// Step 3: Refactor while keeping test green
```

**When to Use TDD:**

- Packet crafting and parsing
- State machine logic
- Detection algorithms
- Security-critical code paths
- Performance-sensitive operations

### 2. Property-Based Testing for Protocol Handling

Use `proptest` to generate random inputs and verify invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn tcp_checksum_always_valid(
        src_ip: u32,
        dst_ip: u32,
        src_port: u16,
        dst_port: u16,
        seq: u32,
    ) {
        let packet = build_tcp_packet(src_ip, dst_ip, src_port, dst_port, seq);
        prop_assert!(verify_tcp_checksum(&packet));
    }
}
```

**Property Examples:**

- **Checksums:** Always valid for any valid packet
- **Sequence Numbers:** Handle wrapping at u32::MAX correctly
- **Port Ranges:** Accept 1-65535, reject 0 and >65535
- **IP Parsing:** Parse any valid IPv4/IPv6 address without panic
- **CIDR Notation:** Valid CIDR always produces valid IP range

### 3. Regression Testing

Every bug fix must include a test that would have caught the bug:

```rust
// Regression test for issue #42: SYN+ACK responses with window=0 incorrectly marked closed
#[test]
fn test_issue_42_zero_window_syn_ack() {
    let response = create_syn_ack_response(window_size: 0);
    let state = determine_port_state(&response);
    assert_eq!(state, PortState::Open); // Was incorrectly Closed before fix
}
```

**Regression Test Requirements:**

- Reference the issue number in test name and comment
- Include minimal reproduction case
- Verify the fix with assertion
- Add to permanent test suite (never remove)

### 4. Mutation Testing

Periodically run mutation testing to verify test quality:

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation tests
cargo mutants

# Should achieve >90% mutation score on core modules
```

**Mutation Testing Goals:**

- Core modules: >90% mutation score
- Network protocol: >85% mutation score
- Scanning modules: >80% mutation score
- CLI/UI: >60% mutation score

---

## Test Levels

### 1. Unit Tests

**Scope:** Individual functions and structs in isolation

**Location:** Inline with source code in `#[cfg(test)]` modules

**Examples:**

```rust
// crates/prtip-network/src/tcp.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_flags_parsing() {
        let flags = TcpFlags::from_bits(0x02).unwrap();
        assert_eq!(flags, TcpFlags::SYN);
    }

    #[test]
    fn test_sequence_number_wrapping() {
        let seq = SequenceNumber::new(0xFFFF_FFFE);
        let next = seq.wrapping_add(5);
        assert_eq!(next.value(), 3); // Wraps around at u32::MAX
    }

    #[test]
    fn test_tcp_option_serialization() {
        let opt = TcpOption::Mss(1460);
        let bytes = opt.to_bytes();
        assert_eq!(bytes, vec![2, 4, 0x05, 0xB4]);
    }

    #[test]
    #[should_panic(expected = "invalid port")]
    fn test_invalid_port_panics() {
        let _ = TcpPacketBuilder::new().destination_port(0);
    }
}
```

**Run Commands:**

```bash
# All unit tests
cargo test --lib

# Specific crate
cargo test -p prtip-network --lib

# Specific module
cargo test tcp::tests

# With output
cargo test -- --nocapture

# With backtrace
RUST_BACKTRACE=1 cargo test
```

**Unit Test Best Practices:**

- ✅ Test public API functions
- ✅ Test edge cases (0, max values, boundaries)
- ✅ Test error conditions
- ✅ Use descriptive test names (`test_<what>_<condition>`)
- ✅ One assertion per test (preferably)
- ❌ Don't test private implementation details
- ❌ Don't use external dependencies (network, filesystem)
- ❌ Don't write flaky tests with timing dependencies

### 2. Integration Tests

**Scope:** Multiple components working together

**Location:** `tests/` directory (separate from source)

**Examples:**

```rust
// crates/prtip-scanner/tests/integration_syn_scan.rs

use prtip_scanner::{Scanner, ScanConfig, ScanType};
use prtip_core::target::Target;

#[tokio::test]
async fn test_syn_scan_local_host() {
    // Setup: Start local test server on port 8080
    let server = spawn_test_server(8080).await;

    // Execute scan
    let config = ScanConfig {
        scan_type: ScanType::Syn,
        targets: vec![Target::single("127.0.0.1", 8080)],
        timeout: Duration::from_secs(5),
        ..Default::default()
    };

    let mut scanner = Scanner::new(config).unwrap();
    scanner.initialize().await.unwrap();
    let results = scanner.execute().await.unwrap();

    // Verify
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].state, PortState::Open);
    assert_eq!(results[0].port, 8080);

    // Cleanup
    server.shutdown().await;
}

#[tokio::test]
async fn test_syn_scan_filtered_port() {
    // Port 9999 should be filtered (no response, no RST)
    let config = ScanConfig {
        scan_type: ScanType::Syn,
        targets: vec![Target::single("127.0.0.1", 9999)],
        timeout: Duration::from_millis(100),
        max_retries: 1,
        ..Default::default()
    };

    let mut scanner = Scanner::new(config).unwrap();
    scanner.initialize().await.unwrap();
    let results = scanner.execute().await.unwrap();

    assert_eq!(results[0].state, PortState::Filtered);
}
```

**Run Commands:**

```bash
# All integration tests
cargo test --test '*'

# Specific test file
cargo test --test integration_syn_scan

# Single test
cargo test --test integration_syn_scan test_syn_scan_local_host

# Parallel execution (default)
cargo test -- --test-threads=4

# Sequential execution
cargo test -- --test-threads=1
```

**Integration Test Best Practices:**

- ✅ Test realistic end-to-end scenarios
- ✅ Use localhost/loopback for network tests
- ✅ Clean up resources (servers, files, connections)
- ✅ Set appropriate timeouts (5-10 seconds)
- ✅ Use `#[tokio::test]` for async tests
- ❌ Don't rely on external services (flaky)
- ❌ Don't test implementation details (test behavior)
- ❌ Don't write tests that interfere with each other

### 3. Cross-Platform Tests

**Scope:** Platform-specific behavior and compatibility

**Location:** Integration tests with `#[cfg(target_os)]` guards

**Examples:**

```rust
// crates/prtip-scanner/tests/test_platform_compat.rs

#[tokio::test]
#[cfg(target_os = "linux")]
async fn test_sendmmsg_batching() {
    // Linux-specific sendmmsg/recvmmsg batching
    let config = ScanConfig::default();
    let scanner = Scanner::new(config).unwrap();

    // Verify batch mode enabled
    assert!(scanner.supports_batch_mode());
}

#[tokio::test]
#[cfg(target_os = "windows")]
async fn test_npcap_compatibility() {
    // Windows-specific Npcap compatibility
    let capture = PacketCapture::new().unwrap();

    // Verify Npcap initialized
    assert!(capture.is_initialized());
}

#[tokio::test]
#[cfg(target_os = "macos")]
async fn test_bpf_device_access() {
    // macOS-specific BPF device access
    let result = check_bpf_permissions();

    // Should succeed with access_bpf group or root
    assert!(result.is_ok());
}

#[tokio::test]
#[cfg(any(target_os = "windows", target_os = "macos"))]
async fn test_stealth_scan_fallback() {
    // FIN/NULL/Xmas scans not supported on Windows/some macOS
    let config = ScanConfig {
        scan_type: ScanType::Fin,
        ..Default::default()
    };

    let result = Scanner::new(config);

    // Should warn or fall back to SYN scan
    assert!(matches!(result, Ok(_) | Err(ScannerError::UnsupportedScanType(_))));
}
```

**Platform-Specific Considerations:**

| Platform | Considerations | Test Strategy |
|----------|----------------|---------------|
| **Linux** | sendmmsg/recvmmsg batching, raw socket permissions | Test batch mode, CAP_NET_RAW |
| **Windows** | Npcap compatibility, loopback limitations, no stealth scans | Test Npcap init, document loopback failures |
| **macOS** | BPF device access, access_bpf group, kernel differences | Test BPF permissions, verify functionality |

### 4. Property-Based Tests

**Scope:** Invariant testing with random inputs

**Location:** `#[cfg(test)]` modules or `tests/proptest/`

**Examples:**

```rust
// crates/prtip-network/src/ipv4.rs

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn ipv4_checksum_always_valid(
            version: u8,
            ihl: u8,
            total_length: u16,
            ttl: u8,
        ) {
            let header = Ipv4Header::new(version, ihl, total_length, ttl);
            prop_assert!(verify_ipv4_checksum(&header));
        }

        #[test]
        fn port_range_valid(port in 1u16..=65535u16) {
            let result = parse_port(port);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn port_range_invalid(port in 65536u32..=100000u32) {
            let result = parse_port(port as u16);
            prop_assert!(result.is_err());
        }

        #[test]
        fn cidr_always_produces_valid_range(
            ip: u32,
            prefix_len in 0u8..=32u8,
        ) {
            let cidr = Ipv4Cidr::new(ip, prefix_len);
            let range = cidr.to_range();

            prop_assert!(range.start <= range.end);
            prop_assert!(range.len() == 2u32.pow(32 - prefix_len as u32));
        }
    }
}
```

**Property Test Strategies:**

- **Inverse Properties:** `parse(format(x)) == x`
- **Invariants:** `checksum(packet) == valid` for all packets
- **Monotonicity:** `f(x) <= f(y)` when `x <= y`
- **Idempotence:** `f(f(x)) == f(x)`
- **Commutivity:** `f(x, y) == f(y, x)`

### 5. Fuzz Testing

**Scope:** Malformed input handling and crash resistance

**Location:** `fuzz/` directory using cargo-fuzz

**Setup:**

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Initialize fuzzing (if not already done)
cargo fuzz init

# List fuzz targets
cargo fuzz list
```

**Fuzz Targets (5 total):**

```rust
// fuzz/fuzz_targets/tcp_parser.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use prtip_network::parse_tcp_packet;

fuzz_target!(|data: &[u8]| {
    // Should never panic, even with arbitrary input
    let _ = parse_tcp_packet(data);
});
```

**Run Commands:**

```bash
# Fuzz TCP parser (runs indefinitely until crash)
cargo fuzz run tcp_parser

# Run for specific duration
cargo fuzz run tcp_parser -- -max_total_time=300  # 5 minutes

# Run with corpus
cargo fuzz run tcp_parser fuzz/corpus/tcp_parser/

# Run all fuzz targets for 1 hour each
for target in $(cargo fuzz list); do
    cargo fuzz run $target -- -max_total_time=3600
done
```

**Fuzz Targets:**

| Target | Purpose | Corpus Size | Status |
|--------|---------|-------------|--------|
| **tcp_parser** | TCP packet parsing | 1,234 inputs | 0 crashes (230M+ execs) |
| **ipv4_parser** | IPv4 header parsing | 891 inputs | 0 crashes (230M+ execs) |
| **ipv6_parser** | IPv6 header parsing | 673 inputs | 0 crashes (230M+ execs) |
| **service_detector** | Service detection | 2,456 inputs | 0 crashes (230M+ execs) |
| **cidr_parser** | CIDR notation parsing | 512 inputs | 0 crashes (230M+ execs) |

**Fuzzing Best Practices:**

- ✅ Run fuzzing for 24+ hours before releases
- ✅ Add discovered crash cases to regression tests
- ✅ Use structure-aware fuzzing (arbitrary crate)
- ✅ Maintain corpus of interesting inputs
- ❌ Don't fuzz without sanitizers (enable address/leak sanitizers)
- ❌ Don't ignore crashes (fix immediately)

---

## Test Coverage

### Coverage Targets by Module

| Component | Target Coverage | Current Coverage (v0.5.2) | Priority |
|-----------|----------------|---------------------------|----------|
| **Core Engine** | >90% | ~92% | Critical |
| **Network Protocol** | >85% | ~87% | High |
| **Scanning Modules** | >80% | ~82% | High |
| **Detection Systems** | >75% | ~78% | Medium |
| **CLI/UI** | >60% | ~62% | Medium |
| **TUI** | >50% | ~54% | Low |
| **Overall** | **>50%** | **54.92%** ✅ | - |

### Measuring Coverage

```bash
# Install tarpaulin (Linux/macOS only)
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --workspace --locked --lib --bins --tests \
    --exclude prtip-network --exclude prtip-scanner \
    --timeout 300 --out Html --output-dir coverage

# View report
firefox coverage/index.html

# CI mode (exit with error if below threshold)
cargo tarpaulin --fail-under 50

# Generate Cobertura XML for Codecov
cargo tarpaulin --out Xml
```

**CI/CD Coverage:**

- Automated coverage reporting on every CI run
- Codecov integration for trend analysis
- 50% minimum coverage threshold (non-blocking)
- Platform-specific: Linux/macOS only (tarpaulin compatibility)

**Coverage Exclusions:**

- Debug-only code (`#[cfg(debug_assertions)]`)
- Test utilities and fixtures
- Generated code (protocol buffers, bindings)
- Platform-specific code not testable in CI

### Coverage Best Practices

- ✅ Measure coverage regularly (every PR)
- ✅ Investigate coverage drops (>5% decrease)
- ✅ Focus on critical paths (core engine >90%)
- ✅ Use `#[cfg(not(tarpaulin_include))]` for untestable code
- ❌ Don't chase 100% coverage (diminishing returns)
- ❌ Don't write tests just for coverage (test behavior)
- ❌ Don't ignore low coverage in core modules

---

## Test Organization

### Directory Structure

```
ProRT-IP/
├── crates/
│   ├── prtip-core/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── circuit_breaker.rs
│   │   │   └── retry.rs
│   │   └── tests/                    # Integration tests
│   │       ├── test_circuit_breaker.rs
│   │       ├── test_retry.rs
│   │       └── test_resource_monitor.rs
│   │
│   ├── prtip-network/
│   │   ├── src/
│   │   │   ├── tcp.rs                # Unit tests inline: #[cfg(test)] mod tests
│   │   │   ├── ipv4.rs
│   │   │   └── ipv6.rs
│   │   └── tests/
│   │       └── test_security_privilege.rs
│   │
│   ├── prtip-scanner/
│   │   ├── src/
│   │   │   ├── syn_scanner.rs        # Unit tests inline
│   │   │   ├── tcp_scanner.rs
│   │   │   └── udp_scanner.rs
│   │   └── tests/                    # Integration tests
│   │       ├── common/               # Shared test utilities
│   │       │   ├── mod.rs
│   │       │   └── error_injection.rs
│   │       ├── test_syn_scanner_unit.rs
│   │       ├── test_syn_scanner_ipv6.rs
│   │       ├── test_udp_scanner_ipv6.rs
│   │       ├── test_stealth_scanner.rs
│   │       ├── test_cross_scanner_ipv6.rs
│   │       └── test_service_detector.rs
│   │
│   ├── prtip-cli/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   └── args.rs
│   │   └── tests/
│   │       ├── common/               # CLI test utilities
│   │       │   └── mod.rs
│   │       ├── test_cli_args.rs
│   │       ├── test_scan_types.rs
│   │       ├── test_ipv6_cli_flags.rs
│   │       ├── test_error_messages.rs
│   │       └── test_error_integration.rs
│   │
│   └── prtip-tui/
│       ├── src/
│       │   ├── lib.rs                # Unit tests inline
│       │   ├── widgets.rs
│       │   └── events.rs
│       └── tests/
│           └── integration_tui.rs
│
├── tests/                            # System tests (optional)
│   └── system/
│       ├── test_full_network_scan.sh
│       └── verify_results.py
│
├── benches/                          # Criterion benchmarks
│   ├── packet_crafting.rs
│   ├── scan_throughput.rs
│   └── service_detection.rs
│
└── fuzz/                             # Cargo-fuzz targets
    ├── Cargo.toml
    ├── fuzz_targets/
    │   ├── tcp_parser.rs
    │   ├── ipv4_parser.rs
    │   ├── ipv6_parser.rs
    │   ├── service_detector.rs
    │   └── cidr_parser.rs
    └── corpus/
        ├── tcp_parser/
        ├── ipv4_parser/
        └── ...
```

### Test Utilities and Helpers

**Common Test Utilities:**

```rust
// crates/prtip-scanner/tests/common/mod.rs

pub mod error_injection;

use tokio::net::TcpListener;
use std::net::SocketAddr;

/// Spawn a TCP server that responds with custom behavior
pub async fn spawn_mock_tcp_server(
    port: u16,
    response_handler: impl Fn(&[u8]) -> Vec<u8> + Send + 'static,
) -> MockServer {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    let handle = tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = vec![0u8; 1024];
            if let Ok(n) = socket.read(&mut buf).await {
                let response = response_handler(&buf[..n]);
                socket.write_all(&response).await.ok();
            }
        }
    });

    MockServer { handle, port }
}

pub struct MockServer {
    handle: JoinHandle<()>,
    port: u16,
}

impl MockServer {
    pub async fn shutdown(self) {
        self.handle.abort();
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
```

**Error Injection Framework:**

```rust
// crates/prtip-scanner/tests/common/error_injection.rs

use std::net::SocketAddr;
use std::time::Duration;

/// Failure modes for error injection
#[derive(Debug, Clone)]
pub enum FailureMode {
    ConnectionRefused,
    Timeout(Duration),
    NetworkUnreachable,
    HostUnreachable,
    ConnectionReset,
    ConnectionAborted,
    WouldBlock,
    Interrupted,
    TooManyOpenFiles,
    MalformedResponse(Vec<u8>),
    InvalidEncoding,
    SuccessAfter(usize),  // Succeed after N attempts
    Probabilistic(f64),   // Fail with probability (0.0-1.0)
}

/// Error injector for deterministic failure simulation
pub struct ErrorInjector {
    target: SocketAddr,
    failure_mode: FailureMode,
    attempts: AtomicUsize,
}

impl ErrorInjector {
    pub fn new(target: SocketAddr, failure_mode: FailureMode) -> Self {
        Self {
            target,
            failure_mode,
            attempts: AtomicUsize::new(0),
        }
    }

    pub fn inject(&self) -> Result<(), ScannerError> {
        let attempt = self.attempts.fetch_add(1, Ordering::SeqCst);

        match &self.failure_mode {
            FailureMode::ConnectionRefused => {
                Err(ScannerError::ConnectionRefused(self.target))
            }
            FailureMode::Timeout(duration) => {
                std::thread::sleep(*duration);
                Err(ScannerError::Timeout(self.target))
            }
            FailureMode::SuccessAfter(n) => {
                if attempt >= *n {
                    Ok(())
                } else {
                    Err(ScannerError::ConnectionRefused(self.target))
                }
            }
            // ... other failure modes
        }
    }

    pub fn reset_attempts(&self) {
        self.attempts.store(0, Ordering::SeqCst);
    }
}
```

### Test Fixtures

**PCAP Samples:**

```rust
// crates/prtip-scanner/tests/fixtures/mod.rs

pub mod pcap_samples {
    /// Load PCAP file for replay testing
    pub fn load_syn_scan_capture() -> Vec<u8> {
        include_bytes!("pcaps/syn_scan.pcap").to_vec()
    }

    pub fn load_os_fingerprint_capture() -> Vec<u8> {
        include_bytes!("pcaps/os_fingerprint.pcap").to_vec()
    }

    pub fn load_service_detection_capture() -> Vec<u8> {
        include_bytes!("pcaps/service_detection.pcap").to_vec()
    }
}

pub mod fingerprints {
    /// Sample OS fingerprint database for testing
    pub fn test_fingerprints() -> Vec<OsFingerprint> {
        vec![
            OsFingerprint {
                name: "Linux 5.x".to_string(),
                signature: "...".to_string(),
                // ...
            },
            OsFingerprint {
                name: "Windows 10".to_string(),
                signature: "...".to_string(),
                // ...
            },
        ]
    }
}
```

---

## Running Tests

### Basic Commands

```bash
# All tests (unit + integration + doc tests)
cargo test

# All tests with output
cargo test -- --nocapture

# Specific test by name
cargo test test_syn_scan

# Specific package
cargo test -p prtip-scanner

# Specific test file
cargo test --test test_syn_scanner_ipv6

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Doc tests only
cargo test --doc
```

### Advanced Commands

```bash
# Run tests in parallel (default)
cargo test -- --test-threads=4

# Run tests sequentially
cargo test -- --test-threads=1

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test

# Run tests with logging
RUST_LOG=debug cargo test

# Run ignored tests
cargo test -- --ignored

# Run all tests (including ignored)
cargo test -- --include-ignored

# Run specific test with output
cargo test test_syn_scan -- --nocapture --exact
```

### Platform-Specific Tests

```bash
# Linux-specific tests
cargo test --test '*' --target x86_64-unknown-linux-gnu

# Windows-specific tests
cargo test --test '*' --target x86_64-pc-windows-msvc

# macOS-specific tests
cargo test --test '*' --target x86_64-apple-darwin

# Run all tests on all platforms (requires cross)
cross test --target x86_64-unknown-linux-gnu
cross test --target x86_64-pc-windows-msvc
cross test --target x86_64-apple-darwin
```

### Test Filtering

```bash
# Run tests matching pattern
cargo test ipv6

# Run tests NOT matching pattern
cargo test -- --skip ipv6

# Run tests in specific module
cargo test tcp::tests

# Run tests with exact name
cargo test test_syn_scan -- --exact

# Run tests containing "error"
cargo test error
```

---

## CI/CD Integration

### GitHub Actions Workflow

ProRT-IP uses GitHub Actions for continuous integration with 9 workflows:

**Test Workflow (.github/workflows/ci.yml):**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}

      - name: Install dependencies (Linux)
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libpcap-dev libssl-dev

      - name: Check formatting
        run: cargo fmt --check

      - name: Lint
        run: cargo clippy --workspace -- -D warnings

      - name: Build
        run: cargo build --verbose

      - name: Run tests
        run: cargo test --workspace --locked --lib --bins --tests

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: |
          cargo tarpaulin --workspace --locked --lib --bins --tests \
            --exclude prtip-network --exclude prtip-scanner \
            --timeout 300 --out Xml
        env:
          PRTIP_DISABLE_HISTORY: "1"

      - name: Upload to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: ./coverage/cobertura.xml
          fail_ci_if_error: false
          verbose: true
```

**Coverage Workflow Features:**

- Runs on Linux only (tarpaulin compatibility)
- Generates Cobertura XML for Codecov
- 300-second timeout for long-running tests
- Non-blocking (fail_ci_if_error: false)
- Test isolation via PRTIP_DISABLE_HISTORY

### Test Isolation

**Environment Variables:**

```bash
# Disable command history (prevents concurrent write conflicts)
export PRTIP_DISABLE_HISTORY=1

# Set test-specific temp directory
export PRTIP_TEMP_DIR=/tmp/prtip-test-$$

# Enable debug logging
export RUST_LOG=debug
```

**Test Isolation Pattern:**

```rust
// crates/prtip-cli/tests/common/mod.rs

pub fn run_prtip(args: &[&str]) -> Result<Output, io::Error> {
    Command::new("prtip")
        .args(args)
        .env("PRTIP_DISABLE_HISTORY", "1")  // Prevent history conflicts
        .env("PRTIP_TEMP_DIR", "/tmp/prtip-test")
        .output()
}
```

---

## Best Practices

### Test Design

✅ **DO:**

- Write tests first (TDD for core features)
- Test behavior, not implementation
- Use descriptive test names (`test_<what>_<condition>`)
- One assertion per test (preferably)
- Clean up resources (servers, files, connections)
- Use appropriate timeouts (5-10 seconds)
- Test edge cases (0, max values, boundaries)
- Test error conditions
- Use `#[tokio::test]` for async tests

❌ **DON'T:**

- Write flaky tests with timing dependencies
- Rely on external services (network, APIs)
- Test private implementation details
- Write tests without assertions
- Ignore test failures
- Leave commented-out tests
- Write tests that depend on execution order

### Test Anti-Patterns to Avoid

**❌ Flaky Tests (Race Conditions):**

```rust
// BAD: Race condition in test
#[tokio::test]
async fn flaky_test() {
    spawn_server().await;
    // No wait for server to be ready!
    let client = connect().await.unwrap(); // May fail randomly
}

// GOOD: Deterministic test
#[tokio::test]
async fn reliable_test() {
    let server = spawn_server().await;
    server.wait_until_ready().await;
    let client = connect().await.unwrap();
}
```

**❌ External Dependencies:**

```rust
// BAD: Depends on external file
#[test]
fn test_config_loading() {
    let config = load_config("/etc/prtip/config.toml"); // Fails in CI
}

// GOOD: Use fixtures
#[test]
fn test_config_loading() {
    let config = load_config("tests/fixtures/test_config.toml");
}
```

**❌ Tests Without Assertions:**

```rust
// BAD: No verification
#[test]
fn test_scan() {
    let scanner = Scanner::new();
    scanner.scan("192.168.1.1").unwrap();
    // Test passes even if scan did nothing!
}

// GOOD: Verify behavior
#[test]
fn test_scan() {
    let scanner = Scanner::new();
    let results = scanner.scan("192.168.1.1").unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].ip, "192.168.1.1");
}
```

**❌ Order-Dependent Tests:**

```rust
// BAD: Tests depend on execution order
static mut COUNTER: u32 = 0;

#[test]
fn test_increment() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 1); // Fails if test_decrement runs first
}

#[test]
fn test_decrement() {
    unsafe { COUNTER -= 1; }
    assert_eq!(unsafe { COUNTER }, 0); // Fails if test_increment runs first
}

// GOOD: Independent tests
#[test]
fn test_increment() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1);
}

#[test]
fn test_decrement() {
    let mut counter = 1;
    counter -= 1;
    assert_eq!(counter, 0);
}
```

---

## Testing Checklist

### Before Each Commit

- [ ] Code passes `cargo fmt --check`
- [ ] Code passes `cargo clippy --workspace -- -D warnings`
- [ ] All unit tests pass (`cargo test --lib`)
- [ ] New code has accompanying tests
- [ ] Coverage hasn't decreased (check with `cargo tarpaulin`)

### Before Each PR

- [ ] All tests pass on all platforms (CI green)
- [ ] Integration tests pass (`cargo test --test '*'`)
- [ ] No flaky tests (run tests 10+ times)
- [ ] Documentation updated for new features
- [ ] Changelog updated
- [ ] Test names are descriptive

### Before Each Release

- [ ] Full system tests pass
- [ ] Security audit clean (`cargo audit`)
- [ ] Fuzz testing run for 24+ hours without crashes
- [ ] Coverage meets targets (>50% overall, >90% core)
- [ ] Cross-platform testing complete (Linux, Windows, macOS)
- [ ] Memory leak testing clean (valgrind)
- [ ] Performance benchmarks meet targets
- [ ] Release notes written

---

## See Also

- [Testing Infrastructure](testing-infrastructure.md) - Test organization, mocking, utilities
- [Fuzzing](fuzzing.md) - Fuzz targets, running fuzzing, results analysis
- [CI/CD](ci-cd.md) - GitHub Actions, code coverage, release automation
- [Implementation](implementation.md) - Code organization and design patterns
- [Technical Specs](technical-specs.md) - System requirements and protocols
