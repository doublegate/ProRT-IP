# ProRT-IP WarScan: Testing Strategy

**Version:** 1.0
**Last Updated:** October 2025

---

## Table of Contents

1. [Overview](#overview)
2. [Testing Philosophy](#testing-philosophy)
3. [Test Levels](#test-levels)
4. [Test Infrastructure](#test-infrastructure)
5. [Test Coverage](#test-coverage)
6. [Continuous Integration](#continuous-integration)
7. [Testing Checklist](#testing-checklist)

---

## Overview

Comprehensive testing is critical for ProRT-IP WarScan due to:

- **Security implications:** Bugs could enable network attacks or scanner exploitation
- **Cross-platform complexity:** Must work correctly on Linux, Windows, macOS
- **Performance requirements:** Must maintain 1M+ pps without degradation
- **Protocol correctness:** Malformed packets lead to inaccurate results

### Testing Goals

1. **Correctness:** All scanning modes produce accurate, repeatable results
2. **Safety:** No memory leaks, data races, or undefined behavior
3. **Performance:** Maintain throughput and latency targets under load
4. **Reliability:** Graceful handling of network errors and edge cases
5. **Security:** Resist malformed packets and DoS attacks

### Test Coverage Targets

| Component | Target Coverage | Current Coverage |
|-----------|----------------|------------------|
| Core Engine | >90% | 0% (pre-development) |
| Network Protocol | >85% | 0% |
| Scanning Modules | >80% | 0% |
| Detection Systems | >75% | 0% |
| CLI/UI | >60% | 0% |
| **Overall** | **>80%** | **0%** |

---

## Testing Philosophy

### 1. Test-Driven Development (TDD) for Core Features

For critical components (packet crafting, state machines, detection engines), write tests **before** implementation:

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

### 2. Property-Based Testing for Protocol Handling

Use `proptest` or `quickcheck` to generate random inputs and verify invariants:

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

### 4. Mutation Testing

Periodically run mutation testing to verify test quality:

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation tests
cargo mutants

# Should achieve >90% mutation score on core modules
```

---

## Test Levels

### 1. Unit Tests

**Scope:** Individual functions and structs in isolation

**Location:** Inline with source code in `#[cfg(test)]` modules

**Examples:**

```rust
// src/net/tcp.rs

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
}
```

**Run Commands:**

```bash
# All unit tests
cargo test --lib

# Specific module
cargo test tcp::tests

# With output
cargo test -- --nocapture
```

### 2. Integration Tests

**Scope:** Multiple components working together

**Location:** `tests/` directory (separate from source)

**Examples:**

```rust
// tests/integration_syn_scan.rs

use prtip::scanner::{Scanner, ScanConfig, ScanType};
use prtip::target::Target;

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

    let scanner = Scanner::new(config).unwrap();
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

    let scanner = Scanner::new(config).unwrap();
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
```

### 3. System Tests

**Scope:** End-to-end scenarios mimicking real-world usage

**Location:** `tests/system/` with helper scripts

**Examples:**

```bash
#!/bin/bash
# tests/system/test_full_network_scan.sh

set -e

# Setup test network (requires Docker)
docker-compose -f tests/fixtures/docker-compose.yml up -d

# Wait for services to start
sleep 5

# Run full scan
cargo run --release -- \
    -sS -sV -O \
    -p- \
    --output json \
    --output-file /tmp/scan_results.json \
    172.20.0.0/24

# Verify expected services found
python3 tests/system/verify_results.py /tmp/scan_results.json

# Cleanup
docker-compose -f tests/fixtures/docker-compose.yml down

echo "✓ System test passed"
```

**Run Commands:**

```bash
bash tests/system/test_full_network_scan.sh
```

### 4. Performance Tests

**Scope:** Throughput, latency, resource usage benchmarks

**Location:** `benches/` directory using Criterion.rs

**Examples:**

```rust
// benches/packet_crafting.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use prtip::net::TcpPacketBuilder;

fn bench_tcp_packet_building(c: &mut Criterion) {
    c.bench_function("tcp_syn_packet", |b| {
        b.iter(|| {
            TcpPacketBuilder::new()
                .source(black_box(Ipv4Addr::new(10, 0, 0, 1)), black_box(12345))
                .destination(black_box(Ipv4Addr::new(10, 0, 0, 2)), black_box(80))
                .flags(TcpFlags::SYN)
                .build()
                .unwrap()
        });
    });
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    for rate in [10_000, 100_000, 1_000_000] {
        group.bench_with_input(BenchmarkId::new("packets_per_sec", rate), &rate, |b, &rate| {
            b.iter(|| {
                // Simulate sending 'rate' packets
                simulate_packet_transmission(black_box(rate))
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_tcp_packet_building, bench_throughput);
criterion_main!(benches);
```

**Run Commands:**

```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench tcp_packet

# With profiling (Linux)
cargo bench --bench packet_crafting -- --profile-time=5
```

### 5. Fuzz Testing

**Scope:** Malformed input handling and crash resistance

**Location:** `fuzz/` directory using cargo-fuzz

**Setup:**

```bash
cargo install cargo-fuzz
cargo fuzz init
```

**Examples:**

```rust
// fuzz/fuzz_targets/tcp_parser.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use prtip::net::parse_tcp_packet;

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
```

---

## Test Infrastructure

### Test Network Setup

#### Docker Compose Test Environment

```yaml
# tests/fixtures/docker-compose.yml

version: '3.8'

services:
  web-server:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    networks:
      testnet:
        ipv4_address: 172.20.0.10

  ssh-server:
    image: linuxserver/openssh-server
    environment:
      - PUID=1000
      - PGID=1000
      - PASSWORD_ACCESS=true
      - USER_PASSWORD=testpass
    ports:
      - "2222:2222"
    networks:
      testnet:
        ipv4_address: 172.20.0.11

  ftp-server:
    image: delfer/alpine-ftp-server
    environment:
      - USERS=testuser|testpass
    ports:
      - "21:21"
    networks:
      testnet:
        ipv4_address: 172.20.0.12

  database:
    image: postgres:15-alpine
    environment:
      - POSTGRES_PASSWORD=testpass
    ports:
      - "5432:5432"
    networks:
      testnet:
        ipv4_address: 172.20.0.13

networks:
  testnet:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/24
```

**Usage:**

```bash
# Start test environment
docker-compose -f tests/fixtures/docker-compose.yml up -d

# Run tests
cargo test --test integration_service_detection

# Cleanup
docker-compose -f tests/fixtures/docker-compose.yml down
```

### Mock Services

```rust
// tests/helpers/mock_server.rs

use tokio::net::TcpListener;

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
}
```

### Test Data Fixtures

```rust
// tests/fixtures/mod.rs

pub mod pcap_samples {
    /// Load PCAP file for replay testing
    pub fn load_syn_scan_capture() -> Vec<u8> {
        include_bytes!("pcaps/syn_scan.pcap").to_vec()
    }

    pub fn load_os_fingerprint_capture() -> Vec<u8> {
        include_bytes!("pcaps/os_fingerprint.pcap").to_vec()
    }
}

pub mod fingerprints {
    /// Sample OS fingerprint database for testing
    pub fn test_fingerprints() -> Vec<OsFingerprint> {
        vec![
            OsFingerprint {
                name: "Linux 5.x",
                signature: "...",
                // ...
            },
            OsFingerprint {
                name: "Windows 10",
                signature: "...",
                // ...
            },
        ]
    }
}
```

---

## Test Coverage

### Measuring Coverage

```bash
# Install tarpaulin (Linux only)
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
firefox coverage/index.html

# CI mode (exit with error if below threshold)
cargo tarpaulin --fail-under 80
```

### Coverage Targets by Module

```rust
// Core modules (>90% coverage)
// - packet crafting
// - checksum calculation
// - state machines

// Medium-priority modules (>80% coverage)
// - scanning algorithms
// - rate limiting
// - result aggregation

// Lower-priority modules (>60% coverage)
// - CLI parsing
// - output formatters
// - TUI components
```

---

## Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml

name: CI

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}

      - name: Install dependencies (Linux)
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libpcap-dev libssl-dev

      - name: Install dependencies (macOS)
        if: runner.os == 'macOS'
        run: brew install libpcap openssl@3

      - name: Install dependencies (Windows)
        if: runner.os == 'Windows'
        run: |
          choco install npcap
          # Download Npcap SDK

      - name: Check formatting
        run: cargo fmt --check

      - name: Lint
        run: cargo clippy -- -D warnings

      - name: Build
        run: cargo build --verbose

      - name: Run tests
        run: cargo test --verbose

      - name: Run integration tests
        run: cargo test --test '*' --verbose

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libpcap-dev libssl-dev
          cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --out Xml

      - name: Upload to codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Security audit
        run: |
          cargo install cargo-audit
          cargo audit
```

---

## Testing Checklist

### Before Each Commit

- [ ] Code passes `cargo fmt --check`
- [ ] Code passes `cargo clippy -- -D warnings`
- [ ] All unit tests pass (`cargo test --lib`)
- [ ] New code has accompanying tests
- [ ] Coverage hasn't decreased

### Before Each PR

- [ ] All tests pass on all platforms (CI green)
- [ ] Integration tests pass (`cargo test --test '*'`)
- [ ] Benchmarks show no regression (`cargo bench`)
- [ ] Documentation updated for new features
- [ ] Changelog updated

### Before Each Release

- [ ] Full system tests pass
- [ ] Security audit clean (`cargo audit`)
- [ ] Fuzz testing run for 24+ hours without crashes
- [ ] Performance benchmarks meet targets
- [ ] Cross-platform testing complete
- [ ] Memory leak testing clean (valgrind)
- [ ] Release notes written

---

## Test Anti-Patterns to Avoid

### ❌ Don't: Flaky tests

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

### ❌ Don't: Tests that depend on external state

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

### ❌ Don't: Tests without assertions

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

---

## Next Steps

- Review [Performance Baselines](07-PERFORMANCE.md) for benchmark targets
- Consult [Security Guide](08-SECURITY.md) for security testing requirements
- See [Architecture](00-ARCHITECTURE.md) for component testing boundaries
