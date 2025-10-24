# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Essential Build & Development Commands

### Build

```bash
# Debug build (all workspace crates)
cargo build --workspace

# Release build (optimized, production-ready)
cargo build --workspace --release

# Build specific binary for direct execution
cargo build -p prtip-cli --release
# Binary output: target/release/prtip
```

### Linting and Formatting

```bash
# Format code (in-place)
cargo fmt --all

# Check formatting without modifying files (CI requirement)
cargo fmt --all -- --check

# Lint with clippy (zero warnings policy - CI requirement)
cargo clippy --workspace --all-targets -- -D warnings
```

### Testing

```bash
# Run full test suite (unit + integration + doc tests)
cargo test --workspace --all-targets

# Run doc tests separately
cargo test --doc

# Run specific test by name pattern
cargo test -p prtip-core <test_name_substring>

# Run specific integration test
cargo test -p prtip-cli --test <test_binary_name> <test_name_substring>

# Run tests with output visible
cargo test -- --nocapture
```

### Code Coverage

```bash
# Install coverage tool (one-time)
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --workspace --all-features --timeout 120 --out Html
# Report location: target/tarpaulin/index.html
```

**Current Coverage**: 61.92% (15,397/24,814 lines) - exceeds 60% target

### Running the CLI

```bash
# Show help
cargo run -p prtip-cli -- --help

# Show version
cargo run -p prtip-cli -- --version

# Run actual scan (example)
sudo cargo run -p prtip-cli --release -- -sS -p 80,443 scanme.nmap.org
```

### Linux Capabilities (Non-Root Raw Sockets)

```bash
# Grant capabilities to release binary (allows non-root raw socket access)
sudo setcap cap_net_raw,cap_net_admin=eip target/release/prtip

# Verify capabilities
getcap -v target/release/prtip

# Remove capabilities
sudo setcap -r target/release/prtip
```

**Note**: Raw packet operations (SYN scan, stealth scans) require elevated privileges on Linux/macOS. Use capabilities on Linux or run with `sudo`.

### Benchmarks

```bash
# Run all benchmarks (Criterion.rs)
cargo bench --workspace

# Run specific benchmark
cargo bench --bench packet_crafting

# View HTML report
firefox target/criterion/report/index.html
```

### CI-Equivalent Local Check

```bash
# Run the same checks as CI pipeline
cargo fmt --all -- --check && \
cargo clippy --workspace --all-targets -- -D warnings && \
cargo test --workspace --all-targets
```

## High-Level Architecture

### Workspace Structure

ProRT-IP is organized as a Rust workspace with four core crates:

- **`prtip-core`**: Shared types, error handling, configuration, and utilities
  - Types: `Target`, `PortRange`, `ScanResult`, `PortState`
  - Configuration: `Config`, `ScanType`, timing templates
  - Utilities: resource limits, progress tracking, top ports database
  
- **`prtip-network`**: Low-level packet construction and I/O
  - Packet building: TCP/UDP/ICMP crafting with `pnet`
  - Socket primitives: raw sockets, packet capture
  - Zero-copy buffers: `PacketBuffer` pools for allocation-free hot paths
  - Privilege management: capability checks and dropping
  
- **`prtip-scanner`**: Scanning orchestration and detection logic
  - Scan types: SYN, Connect, UDP, Stealth (FIN/NULL/Xmas), ACK
  - Detection: service detection (187 probes), OS fingerprinting (2000+ signatures)
  - Orchestration: `ScanScheduler`, adaptive parallelism, rate limiting
  - Storage: async result aggregation, SQLite persistence
  
- **`prtip-cli`**: User-facing command-line interface
  - Argument parsing with `clap` (nmap-compatible flags)
  - Output formatting: text, JSON, XML, greppable, PCAPNG
  - Progress display: real-time progress bar with sub-millisecond updates

### Scanning Philosophy (Hybrid Approach)

ProRT-IP combines two scanning strategies:

1. **Fast Discovery** (Masscan-inspired)
   - Stateless, high-throughput probing (target: 1M+ packets/sec)
   - Minimal per-packet state using SipHash-based tracking
   - Identifies live hosts and open ports quickly

2. **Deep Enumeration** (Nmap-inspired)
   - Stateful connections to discovered endpoints
   - Service version detection with 187 protocol-specific probes
   - OS fingerprinting with 16-probe technique and 2000+ signature database
   - Banner grabbing and TLS certificate extraction

This hybrid design maximizes efficiency: spend minimal resources on discovery, then invest deeper analysis only where warranted.

### Asynchronous Architecture

- **Runtime**: Tokio multi-threaded async runtime
- **Concurrency model**: 
  - Lock-free result aggregation using `crossbeam::SegQueue`
  - Adaptive parallelism (20-1000 concurrent tasks based on scan size)
  - Work-stealing scheduler for CPU-bound operations
- **I/O patterns**:
  - Async packet I/O with `tokio::spawn` for parallel scanning
  - Backpressure handling via `tokio::sync::Semaphore`
  - Stream-to-disk for large result sets

### Performance Patterns

- **Zero-Copy Optimization** (Sprint 4.17):
  - Reusable `PacketBuffer` pools to eliminate allocations in hot paths
  - 15% improvement in packet crafting (68.3ns → 58.8ns per packet)
  - 100% allocation elimination (3-7M allocs/sec → 0)

- **NUMA Optimization** (Sprint 4.19):
  - Topology detection with `hwloc` (Linux only)
  - Thread pinning to NUMA nodes
  - 20-30% improvement on multi-socket systems (dual/quad Xeon/EPYC)
  - Enable with `--numa` flag

- **Adaptive Parallelism**:
  - Scales from 20 (small scans) to 1000 (large scans) concurrent workers
  - Automatically adjusts based on target count and scan type
  - Manual override: `--max-concurrent <N>`

- **Rate Limiting**:
  - Per-target and global rate limits to prevent network overload
  - Adaptive rate control based on drop detection
  - Timing templates T0-T5 (paranoid → insane)

### Security Model

**Privilege Pattern**:
1. Create privileged resources (raw sockets) at startup
2. Drop privileges immediately after resource creation
3. Run all scanning logic unprivileged

**Safe Packet Parsing**:
- Use `pnet` and `etherparse` with bounds checking
- Never `panic!` on malformed packets (return `Option`/`Result`)
- Resource limits: max concurrent connections, per-target rates, timeouts

**Input Validation**:
- `IpAddr::parse()` for IP addresses
- `ipnetwork` crate for CIDR validation
- Never construct shell commands from user input

### Multi-Phase Scanning

Coordinated by `ScanScheduler`:

1. **Discovery** → Identify live hosts (ICMP/ARP/TCP ping)
2. **Enumeration** → Scan ports on discovered hosts
3. **Deep Inspection** → Service/OS detection on open ports

The scheduler manages target queues, assigns work to workers, handles backpressure, and aggregates results across phases.

### Key Data Structures

- **`TargetRandomizer`**: Permutation-based target ordering using Blackrock cipher
- **`RateLimiter`**: Token bucket rate limiting with adaptive backoff
- **`LockFreeAggregator`**: Concurrent result collection without locks
- **`ScanStorage`**: Async SQLite persistence with WAL mode and batch inserts

## Code Conventions

### Style and Tooling

- **Edition**: Rust 2021
- **MSRV**: 1.85+ (required for edition 2024 features)
- **Formatting**: `rustfmt` defaults (4-space indentation, trailing commas)
- **Linting**: Zero clippy warnings enforced (`-D warnings`)

### Commit Message Format

Follow Conventional Commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`

**Examples**:
```
feat(scanner): add UDP scanning with protocol-specific probes
fix(network): correct TCP checksum calculation for IPv6
docs(readme): update installation instructions for Windows
```

### Error Handling

- Use `Result<T, E>` types (prefer `anyhow::Result` for application code)
- **Never use `unwrap()` in production code** (except in tests or with explicit justification)
- Return `Option`/`Result` for recoverable errors
- Use `?` operator for error propagation

### Documentation

- **Doc comments** (`///`) required for all public items
- Include **examples** in doc comments where practical
- Reference related types/functions with `[TypeName]` markdown links
- Document **panics**, **errors**, and **safety** invariants

## Testing Strategy

### Test Organization

- **Unit tests**: Inline with source in `#[cfg(test)]` modules
- **Integration tests**: `tests/` directory at workspace root
  - `tests/common/`: shared test utilities
  - `tests/fixtures/`: test data files
  - `tests/performance/`: performance regression tests

### Test Requirements

- All new features must include tests
- Maintain **60%+ code coverage** (current: 61.92%)
- Use `#[tokio::test]` for async tests
- Tests requiring `CAP_NET_RAW` must be conditionally ignored on unprivileged systems

### Running Tests

```bash
# CI-equivalent full test run
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets

# Quick test subset during development
cargo test -p prtip-core  # Test only core crate
```

### Current Test Metrics

- **Total tests**: 911/921 (98.9% passing, 10 ignored due to `CAP_NET_RAW`)
- **Coverage**: 61.92% (15,397/24,814 lines)
- **Integration tests**: 70 scenarios across scan types, detection, and output formats

## Key Documentation References

- **`AGENTS.md`**: Repository guidelines, current sprint focus, CI requirements, metrics snapshot
- **`README.md`**: Project overview, usage examples, performance benchmarks, status badges
- **`CONTRIBUTING.md`**: Development workflow, PR process, coding standards
- **`docs/00-ARCHITECTURE.md`**: Detailed system design and component interactions
- **`docs/03-DEV-SETUP.md`**: Platform-specific setup instructions (Linux/Windows/macOS)
- **`docs/06-TESTING.md`**: Test strategy, coverage targets, test writing guidelines
- **`docs/07-PERFORMANCE.md`**: Optimization techniques, profiling, NUMA configuration
- **`docs/08-SECURITY.md`**: Security requirements, privilege handling, safe parsing

## Common Development Tasks

### Adding a New Scan Type

1. Implement scanner in `crates/prtip-scanner/src/` (e.g., `my_scanner.rs`)
2. Add variant to `ScanType` enum in `crates/prtip-core/src/types.rs`
3. Wire into `ScanScheduler` in `crates/prtip-scanner/src/scheduler.rs`
4. Add CLI argument in `crates/prtip-cli/src/args.rs`
5. Write unit tests in scanner module
6. Write integration test in `tests/`
7. Update documentation in `docs/`

### Adding a New Protocol Probe

1. Add probe definition to `crates/prtip-core/src/service_db.rs`
2. Implement probe logic in `crates/prtip-scanner/src/service_detector.rs`
3. Add signature matching rules
4. Write unit tests for probe parsing
5. Test against live services

### Performance Profiling

```bash
# Build with debug symbols
RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes" cargo build --release

# Profile with perf (Linux)
sudo perf record --call-graph dwarf -F 997 ./target/release/prtip [args]
perf report

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

### Cross-Platform Testing

The project supports Linux, Windows, macOS, and BSD. When making changes:

- Test on multiple platforms if possible (or rely on CI)
- Be aware of platform-specific differences:
  - **Linux**: Best performance, full raw socket support, NUMA optimization
  - **Windows**: Requires Npcap, some stealth scans don't work
  - **macOS**: Requires root or ChmodBPF, no `sendmmsg` batching
  - **BSD**: Limited testing, raw socket quirks

## Notes

- This file focuses on ProRT-IP-specific commands and architecture
- For general Rust development practices, refer to official Rust documentation
- Keep metrics in sync with `README.md` badges and `AGENTS.md` when values change
- Current version: v0.3.8 (Sprint 4.18.1 complete, Sprint 4.20 Phase 2 partial)
