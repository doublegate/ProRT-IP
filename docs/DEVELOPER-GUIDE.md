# ProRT-IP WarScan Developer Guide

**Version:** 1.0.0 | **Last Updated:** 2026-01-25 | **Status:** Production Release

---

## Table of Contents

1. [Introduction](#introduction)
2. [Architecture Overview](#architecture-overview)
3. [Project Structure](#project-structure)
4. [Development Environment](#development-environment)
5. [Build System](#build-system)
6. [Crate Guide](#crate-guide)
7. [Design Patterns](#design-patterns)
8. [API Reference](#api-reference)
9. [Testing Guide](#testing-guide)
10. [Performance Guidelines](#performance-guidelines)
11. [Contributing Code](#contributing-code)
12. [Plugin Development](#plugin-development)
13. [Debugging](#debugging)
14. [Release Process](#release-process)

---

## Introduction

This guide provides comprehensive documentation for developers contributing to ProRT-IP WarScan. Whether you're fixing bugs, adding features, or building plugins, this document covers the essential knowledge needed to work with the codebase effectively.

### Who Should Read This

- **Contributors**: Anyone submitting pull requests
- **Plugin Developers**: Creating custom scanning logic via Lua plugins
- **Integrators**: Embedding ProRT-IP in other applications
- **Security Researchers**: Understanding the scanning implementation

### Prerequisites

- Rust experience (intermediate level)
- Familiarity with async/await patterns
- Basic networking knowledge (TCP/IP, raw sockets)
- Understanding of Unix systems programming

---

## Architecture Overview

ProRT-IP WarScan is a modern network scanner that combines:

- **Masscan Speed**: 10M+ packets/second capability via stateless scanning
- **Nmap-Inspired Depth**: Service detection with 37 probes (226 match rules); accuracy not yet measured

### High-Level Architecture

```
+---------------------------------------------------------+
|                    User Interface Layer                   |
|  (CLI Args Parser, TUI Dashboard, Output Formatters)     |
+----------------------------+-----------------------------+
                             |
+----------------------------v-----------------------------+
|                   Orchestration Layer                     |
|  +---------------+ +---------------+ +----------------+  |
|  | Scanner       | | Rate          | | Result         |  |
|  | Scheduler     | | Controller    | | Aggregator     |  |
|  +---------------+ +---------------+ +----------------+  |
+----------------------------+-----------------------------+
                             |
+----------------------------v-----------------------------+
|                  Scanning Engine Layer                    |
|  +-------------+ +--------------+ +------------------+   |
|  | Host        | | Port Scanner | | Service          |   |
|  | Discovery   | | (TCP/UDP)    | | Detection        |   |
|  +-------------+ +--------------+ +------------------+   |
|  +-------------+ +--------------+ +------------------+   |
|  | OS          | | Stealth      | | Plugin           |   |
|  | Fingerprint | | Module       | | Engine           |   |
|  +-------------+ +--------------+ +------------------+   |
+----------------------------+-----------------------------+
                             |
+----------------------------v-----------------------------+
|                Network Protocol Layer                     |
|  +-------------+ +--------------+ +------------------+   |
|  | Raw Packet  | | TCP Stack    | | Packet           |   |
|  | Crafting    | | (Custom)     | | Capture          |   |
|  | (pnet)      | |              | | (libpcap)        |   |
|  +-------------+ +--------------+ +------------------+   |
+----------------------------+-----------------------------+
                             |
+----------------------------v-----------------------------+
|                Operating System Layer                     |
|  (Linux/Windows/macOS - Raw Sockets, BPF, Npcap)        |
+---------------------------------------------------------+
```

### Core Design Principles

1. **Modularity**: Independent, testable components with clear interfaces
2. **Async-First**: All I/O uses Tokio for maximum concurrency
3. **Zero-Copy**: Minimize allocations in hot paths
4. **Type Safety**: Leverage Rust's type system to prevent invalid states
5. **Fail-Safe Defaults**: Conservative settings prevent network disruption

### Key Technologies

| Component | Technology | Purpose |
|-----------|------------|---------|
| Async Runtime | Tokio 1.35+ | Work-stealing scheduler, I/O events |
| Packet Crafting | pnet 0.35+ | Raw packet construction and parsing |
| Packet Capture | pcap 2.0+ | libpcap bindings |
| Concurrency | crossbeam 0.8+ | Lock-free data structures |
| TUI | ratatui 0.29+ | Terminal user interface |
| CLI | clap 4.5+ | Command-line argument parsing |

---

## Project Structure

### Directory Layout

```
ProRT-IP/
+-- Cargo.toml                    # Workspace manifest
+-- Cargo.lock                    # Dependency lock
+-- README.md                     # Project README
+-- LICENSE                       # GPL-3.0 license
+-- CHANGELOG.md                  # Version history
+-- CONTRIBUTING.md               # Contributor guidelines
|
+-- crates/
|   +-- prtip-core/               # Core types and utilities
|   |   +-- Cargo.toml
|   |   +-- src/
|   |       +-- lib.rs            # Crate root
|   |       +-- config.rs         # Configuration management
|   |       +-- error.rs          # Error types
|   |       +-- rate_limiter.rs   # Rate limiting (V3)
|   |       +-- result.rs         # Scan result types
|   |       +-- types.rs          # Shared type definitions
|   |
|   +-- prtip-network/            # Network protocol layer
|   |   +-- Cargo.toml
|   |   +-- src/
|   |       +-- lib.rs
|   |       +-- packet/           # Packet construction
|   |       |   +-- tcp.rs        # TCP packet builder
|   |       |   +-- udp.rs        # UDP packet builder
|   |       |   +-- icmp.rs       # ICMP/ICMPv6 packets
|   |       |   +-- ipv6.rs       # IPv6 packet builder
|   |       +-- capture.rs        # Packet capture
|   |       +-- checksum.rs       # Checksum calculation
|   |       +-- rawsock.rs        # Raw socket abstraction
|   |
|   +-- prtip-scanner/            # Scanning engine
|   |   +-- Cargo.toml
|   |   +-- src/
|   |       +-- lib.rs
|   |       +-- scanner.rs        # Scanner orchestrator
|   |       +-- scheduler.rs      # Target scheduling
|   |       +-- syn_scanner.rs    # TCP SYN scanner
|   |       +-- tcp_connect.rs    # TCP Connect scanner
|   |       +-- udp_scanner.rs    # UDP scanner
|   |       +-- stealth/          # Stealth scan types
|   |       |   +-- fin.rs        # FIN scan
|   |       |   +-- null.rs       # NULL scan
|   |       |   +-- xmas.rs       # Xmas scan
|   |       |   +-- ack.rs        # ACK scan
|   |       +-- idle_scanner.rs   # Idle (zombie) scan
|   |       +-- decoy_scanner.rs  # Decoy scan
|   |       +-- service_detect.rs # Service detection
|   |       +-- os_fingerprint.rs # OS fingerprinting
|   |       +-- plugin/           # Plugin system
|   |           +-- plugin_manager.rs
|   |           +-- plugin_api.rs
|   |           +-- lua_api.rs
|   |           +-- security.rs
|   |
|   +-- prtip-cli/                # Command-line interface
|   |   +-- Cargo.toml
|   |   +-- src/
|   |       +-- main.rs           # Entry point
|   |       +-- args.rs           # Argument parsing
|   |       +-- output.rs         # Output formatters
|   |
|   +-- prtip-tui/                # Terminal UI
|       +-- Cargo.toml
|       +-- src/
|           +-- lib.rs
|           +-- app.rs            # Application state
|           +-- event.rs          # Event handling
|           +-- widgets/          # UI widgets
|
+-- data/                         # Embedded data files
|   +-- service-probes.txt        # ProRT-IP service detection probe corpus
|   +-- ATTRIBUTION.md            # Data sourcing and licence notes
|
# (No OS fingerprint database is bundled; a caller of the OsFingerprinter API
#  must supply and parse its own signature database.)
|
+-- docs/                         # Documentation
|   +-- 00-ARCHITECTURE.md
|   +-- 01-ROADMAP.md
|   +-- ...
|
+-- tests/                        # Integration tests
+-- benches/                      # Benchmarks
+-- fuzz/                         # Fuzz testing targets
+-- scripts/                      # Utility scripts
+-- docker/                       # Docker configuration
+-- packaging/                    # Distribution packages
+-- man/                          # Man pages
```

### Crate Dependencies

```
prtip-cli
    |
    +-- prtip-scanner
    |       |
    |       +-- prtip-network
    |       |       |
    |       |       +-- prtip-core
    |       |
    |       +-- prtip-core
    |
    +-- prtip-tui
    |       |
    |       +-- prtip-scanner
    |       +-- prtip-core
    |
    +-- prtip-core
```

---

## Development Environment

### System Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| Rust | 1.85+ (MSRV) | Latest stable |
| Memory | 4 GB | 16 GB |
| Disk | 2 GB | 10 GB |
| OS | Linux 4.15+, Windows 10+, macOS 11+ | Linux latest |

### Installation

#### Linux (Debian/Ubuntu)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install system dependencies
sudo apt-get update
sudo apt-get install -y \
    libpcap-dev \
    libssl-dev \
    pkg-config \
    build-essential

# Clone repository
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP

# Build
cargo build
```

#### Linux (Fedora/RHEL)

```bash
# Install dependencies
sudo dnf install -y \
    libpcap-devel \
    openssl-devel \
    pkg-config \
    gcc

# Clone and build
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build
```

#### macOS

```bash
# Install Homebrew dependencies
brew install libpcap openssl@3

# Clone and build
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build
```

#### Windows

```powershell
# Install Rust
winget install Rustlang.Rustup

# Install Npcap (required for packet capture)
# Download from: https://npcap.com/#download
# Install with "WinPcap API-compatible Mode" enabled

# Clone and build
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build
```

### Development Tools

```bash
# Install recommended tools
cargo install cargo-tarpaulin    # Code coverage
cargo install cargo-audit        # Security audit
cargo install cargo-flamegraph   # Performance profiling
cargo install cargo-mutants      # Mutation testing
cargo install cargo-fuzz         # Fuzz testing

# Install pre-commit hooks
./scripts/install-hooks.sh
```

### IDE Setup

#### VS Code

Recommended extensions:
- `rust-analyzer`: Rust language support
- `crates`: Cargo.toml assistance
- `Even Better TOML`: TOML syntax highlighting
- `Error Lens`: Inline error display

Settings (`.vscode/settings.json`):
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "editor.formatOnSave": true
}
```

#### JetBrains RustRover / IntelliJ IDEA

- Install Rust plugin
- Enable "Check on Save" with Clippy
- Configure code style to match `rustfmt.toml`

---

## Build System

### Cargo Commands

```bash
# Debug build (fast compilation, slow execution)
cargo build

# Release build (slow compilation, fast execution)
cargo build --release

# Build specific crate
cargo build -p prtip-scanner

# Build with all features
cargo build --all-features

# Check compilation without building
cargo check

# Build documentation
cargo doc --open
```

### Build Profiles

Configured in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = "fat"         # Link-time optimization
codegen-units = 1   # Better optimization
panic = "abort"     # Smaller binary
strip = true        # Remove symbols

[profile.dev]
opt-level = 0
debug = true

[profile.test]
opt-level = 1       # Faster test builds
```

### Feature Flags

Currently, all features are enabled by default. Future feature flags:

```toml
[features]
default = ["tui", "plugins"]
tui = ["ratatui", "crossterm"]
plugins = ["mlua"]
full = ["tui", "plugins"]
```

### Cross-Compilation

```bash
# Install cross-compilation tool
cargo install cross

# Build for different targets
cross build --release --target x86_64-unknown-linux-musl
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-pc-windows-gnu
```

---

## Crate Guide

### prtip-core

Core types, configuration, and utilities shared across all crates.

**Key Types:**

```rust
// Target specification
pub enum Target {
    Single(IpAddr),
    Range(IpAddr, IpAddr),
    Cidr(IpNetwork),
    Hostname(String),
}

// Port state
pub enum PortState {
    Open,
    Closed,
    Filtered,
    OpenFiltered,
    Unfiltered,
}

// Scan result
pub struct ScanResult {
    pub target: IpAddr,
    pub port: u16,
    pub state: PortState,
    pub service: Option<ServiceInfo>,
    pub timestamp: DateTime<Utc>,
}

// Rate limiter (V3 - default)
pub struct AdaptiveRateLimiterV3 {
    hostgroup_rate: Arc<AtomicU64>,
    batch_size: AtomicU64,
    max_rate: u64,
}
```

### prtip-network

Network protocol implementation including packet crafting and capture.

**Key Types:**

```rust
// TCP packet builder
pub struct TcpPacketBuilder {
    src_ip: IpAddr,
    dst_ip: IpAddr,
    src_port: u16,
    dst_port: u16,
    flags: TcpFlags,
    options: Vec<TcpOption>,
}

impl TcpPacketBuilder {
    pub fn new() -> Self;
    pub fn source(self, ip: IpAddr, port: u16) -> Self;
    pub fn destination(self, ip: IpAddr, port: u16) -> Self;
    pub fn flags(self, flags: TcpFlags) -> Self;
    pub fn build_ipv4(self) -> Result<Vec<u8>>;
    pub fn build_ipv6(self) -> Result<Vec<u8>>;
}

// Packet capture
pub struct PacketCapture {
    handle: pcap::Capture<pcap::Active>,
}

impl PacketCapture {
    pub fn new(interface: &str) -> Result<Self>;
    pub fn set_filter(&mut self, filter: &str) -> Result<()>;
    pub async fn recv_async(&mut self) -> Result<Vec<u8>>;
}
```

### prtip-scanner

Scanning engine implementing all scan types.

**Scanner Types:**

| Scanner | Module | Privileges | Description |
|---------|--------|------------|-------------|
| SynScanner | `syn_scanner.rs` | Root | TCP SYN (half-open) scan |
| TcpConnectScanner | `tcp_connect.rs` | User | TCP Connect (full handshake) |
| UdpScanner | `udp_scanner.rs` | Root | UDP with protocol payloads |
| StealthScanner | `stealth/` | Root | FIN/NULL/Xmas/ACK scans |
| IdleScanner | `idle_scanner.rs` | Root | Zombie scan |
| DecoyScanner | `decoy_scanner.rs` | Root | Source spoofing |

**Example: Using Scanner API:**

```rust
use prtip_scanner::{Scanner, ScanConfig, ScanType};

#[tokio::main]
async fn main() -> Result<()> {
    let config = ScanConfig {
        scan_type: ScanType::Syn,
        targets: vec![Target::from_str("192.168.1.0/24")?],
        ports: PortRange::from_str("1-1000")?,
        rate: 10000,
        timeout: Duration::from_secs(5),
        ..Default::default()
    };

    let scanner = Scanner::new(config)?;
    let results = scanner.execute().await?;

    for result in results {
        println!("{}", result);
    }

    Ok(())
}
```

### prtip-cli

Command-line interface with Nmap-compatible flags.

**Argument Parsing:**

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "prtip")]
pub struct Args {
    /// Target specification
    #[arg(value_name = "TARGETS")]
    pub targets: Vec<String>,

    /// Port specification
    #[arg(short = 'p', long)]
    pub ports: Option<String>,

    /// Scan type
    #[arg(short = 's')]
    pub scan_type: Option<char>,

    /// Enable service detection
    #[arg(short = 'V', long)]
    pub service_detection: bool,

    /// Enable OS detection
    #[arg(short = 'O', long)]
    pub os_detection: bool,

    /// Timing template (0-5)
    #[arg(short = 'T')]
    pub timing: Option<u8>,
}
```

### prtip-tui

Terminal user interface using ratatui.

**Key Components:**

```rust
// Application state
pub struct App {
    state: AppState,
    results: Vec<ScanResult>,
    event_bus: EventBus,
    widgets: WidgetManager,
}

// Event handling
pub enum AppEvent {
    ScanStarted,
    PortFound(ScanResult),
    ScanComplete,
    KeyPress(KeyEvent),
    Tick,
}

// Widget system
pub trait Widget {
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, event: &AppEvent) -> Option<Action>;
}
```

---

## Design Patterns

### Builder Pattern

Used extensively for packet and configuration construction:

```rust
let packet = TcpPacketBuilder::new()
    .source(local_ip, random_port())
    .destination(target_ip, target_port)
    .flags(TcpFlags::SYN)
    .tcp_option(TcpOption::Mss(1460))
    .tcp_option(TcpOption::WindowScale(7))
    .build()?;
```

### Strategy Pattern

Scan type selection:

```rust
pub trait ScanStrategy: Send + Sync {
    async fn scan(&self, target: SocketAddr) -> Result<PortState>;
    fn name(&self) -> &str;
}

struct SynScan;
struct ConnectScan;
struct UdpScan;

impl ScanStrategy for SynScan {
    async fn scan(&self, target: SocketAddr) -> Result<PortState> {
        // SYN scan implementation
    }
}
```

### Observer Pattern

Result streaming via channels:

```rust
pub trait ResultObserver: Send {
    fn on_result(&mut self, result: ScanResult);
    fn on_complete(&mut self);
}

struct FileWriter { /* ... */ }
struct DatabaseWriter { /* ... */ }
struct TuiUpdater { /* ... */ }

impl ResultObserver for FileWriter {
    fn on_result(&mut self, result: ScanResult) {
        writeln!(self.file, "{}", result).ok();
    }
}
```

### Type State Pattern

Compile-time state enforcement:

```rust
struct Scanner<S> {
    config: ScanConfig,
    state: PhantomData<S>,
}

struct Unconfigured;
struct Configured;
struct Running;

impl Scanner<Unconfigured> {
    pub fn new() -> Self { /* ... */ }
    pub fn configure(self, config: ScanConfig) -> Scanner<Configured> { /* ... */ }
}

impl Scanner<Configured> {
    pub async fn start(self) -> Scanner<Running> { /* ... */ }
}

impl Scanner<Running> {
    pub async fn wait(self) -> ScanReport { /* ... */ }
}
```

---

## API Reference

### Public APIs

The following types are exported for external use:

**prtip-core:**
- `Target`, `PortRange`, `PortState`
- `ScanConfig`, `ScanResult`, `ServiceInfo`
- `AdaptiveRateLimiterV3` (as `RateLimiter`)
- Error types

**prtip-scanner:**
- `Scanner`, `ScanReport`
- `ScanType` enum
- Scanner trait implementations

**prtip-network:**
- `TcpPacketBuilder`, `UdpPacketBuilder`, `IcmpPacketBuilder`
- `PacketCapture`
- Checksum functions

### Documentation Standards

All public APIs must have:

```rust
/// Performs a TCP SYN scan on the specified target.
///
/// # Arguments
///
/// * `target` - The target IP address and port
/// * `timeout` - Maximum time to wait for response
///
/// # Returns
///
/// Returns `Ok(PortState)` on success, or `Err` if the scan fails.
///
/// # Example
///
/// ```rust
/// use prtip_scanner::syn_scan;
/// use std::net::SocketAddr;
/// use std::time::Duration;
///
/// let target: SocketAddr = "192.168.1.1:80".parse()?;
/// let state = syn_scan(target, Duration::from_secs(5)).await?;
/// ```
///
/// # Errors
///
/// - `Error::PermissionDenied` - Insufficient privileges for raw sockets
/// - `Error::NetworkError` - Network communication failure
pub async fn syn_scan(target: SocketAddr, timeout: Duration) -> Result<PortState> {
    // Implementation
}
```

---

## Testing Guide

### Test Organization

```
crates/
+-- prtip-core/
|   +-- src/
|   |   +-- lib.rs          # Unit tests inline
|   +-- tests/
|       +-- integration.rs  # Integration tests
|
+-- prtip-scanner/
    +-- src/
    |   +-- scanner.rs      # Unit tests inline
    +-- tests/
        +-- common/
        |   +-- mod.rs
        |   +-- error_injection.rs
        +-- test_syn_scanner.rs
        +-- test_service_detection.rs
```

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p prtip-scanner

# Specific test
cargo test test_syn_scan

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test '*'

# Tests requiring root (Linux)
sudo -E cargo test --features privileged
```

### Writing Tests

**Unit Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_state_display() {
        assert_eq!(format!("{}", PortState::Open), "open");
        assert_eq!(format!("{}", PortState::Closed), "closed");
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }

    #[test]
    #[should_panic(expected = "invalid port")]
    fn test_invalid_port() {
        PortRange::from_str("0-70000").unwrap();
    }
}
```

**Integration Tests:**

```rust
// tests/test_scanner.rs
use prtip_scanner::{Scanner, ScanConfig};

#[tokio::test]
async fn test_tcp_connect_scan_localhost() {
    let config = ScanConfig {
        scan_type: ScanType::Connect,
        targets: vec![Target::from_str("127.0.0.1")?],
        ports: PortRange::single(22),
        ..Default::default()
    };

    let scanner = Scanner::new(config).unwrap();
    let results = scanner.execute().await.unwrap();

    // Port 22 should be either open or closed, not filtered
    assert!(matches!(
        results[0].state,
        PortState::Open | PortState::Closed
    ));
}
```

**Property-Based Tests:**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn tcp_checksum_always_valid(
        src_ip in any::<u32>(),
        dst_ip in any::<u32>(),
        src_port in any::<u16>(),
        dst_port in any::<u16>(),
    ) {
        let packet = build_tcp_packet(src_ip, dst_ip, src_port, dst_port);
        prop_assert!(verify_tcp_checksum(&packet));
    }
}
```

### Code Coverage

```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# View report
xdg-open coverage/index.html

# CI mode with threshold
cargo tarpaulin --fail-under 50
```

### Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench packet_crafting

# Save baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

---

## Performance Guidelines

### Hot Path Optimization

1. **Avoid allocations in loops:**

```rust
// Bad
for target in targets {
    let buffer = vec![0u8; 1500];  // Allocation per iteration
    send_packet(&buffer)?;
}

// Good
let mut buffer = vec![0u8; 1500];  // Single allocation
for target in targets {
    buffer.clear();
    build_packet_into(&mut buffer, target)?;
    send_packet(&buffer)?;
}
```

2. **Use zero-copy where possible:**

```rust
// Parse without copying
let packet = EthernetPacket::new(data)?;  // Borrows data
let tcp = TcpPacket::new(packet.payload())?;  // Borrows slice
```

3. **Prefer stack allocation:**

```rust
// Bad
let flags: Box<TcpFlags> = Box::new(TcpFlags::SYN);

// Good
let flags: TcpFlags = TcpFlags::SYN;  // Stack allocated
```

### Async Best Practices

1. **Spawn CPU-bound work:**

```rust
// Bad - blocks async runtime
async fn process(data: &[u8]) {
    expensive_computation(data);  // Blocks worker thread
}

// Good - offload to thread pool
async fn process(data: Vec<u8>) {
    tokio::task::spawn_blocking(move || {
        expensive_computation(&data);
    }).await?;
}
```

2. **Use appropriate channel types:**

```rust
// High-throughput results
let (tx, rx) = tokio::sync::mpsc::channel(10000);

// Single value signals
let (tx, rx) = tokio::sync::oneshot::channel();

// Broadcast to multiple receivers
let (tx, rx) = tokio::sync::broadcast::channel(100);
```

### Profiling

```bash
# CPU profiling with flamegraph
cargo flamegraph --bin prtip -- -sS -p 1-1000 127.0.0.1

# Memory profiling with heaptrack (Linux)
heaptrack cargo run --release -- -sS -p 1-1000 127.0.0.1

# Perf profiling (Linux)
perf record cargo run --release -- -sS -p 1-1000 127.0.0.1
perf report
```

---

## Contributing Code

### Workflow

1. **Fork and clone** the repository
2. **Create feature branch**: `git checkout -b feature/my-feature`
3. **Make changes** following code style
4. **Run tests**: `cargo test`
5. **Run lints**: `cargo clippy -- -D warnings`
6. **Format code**: `cargo fmt`
7. **Commit** with conventional commit message
8. **Push** and create pull request

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code refactoring
- `test`: Test additions
- `perf`: Performance improvement
- `chore`: Maintenance

**Example:**

```
feat(scanner): add IPv6 support for SYN scanner

Implement IPv6 packet building and checksum calculation
for the SYN scanner. Includes:
- IPv6 pseudo-header checksum
- ICMPv6 response handling
- Dual-stack target support

Closes #123
```

### Code Style

- Run `cargo fmt` before committing
- No Clippy warnings allowed
- Document all public APIs
- Keep functions under 50 lines when possible
- Use descriptive variable names

### Pull Request Checklist

- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] New tests added for new functionality
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No security issues introduced
- [ ] Performance impact considered

---

## Plugin Development

See [Plugin System Guide](30-PLUGIN-SYSTEM-GUIDE.md) for comprehensive documentation.

### Quick Start

1. Create plugin directory:
```bash
mkdir -p ~/.prtip/plugins/my-plugin
```

2. Create `plugin.toml`:
```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
author = "Your Name"
description = "My custom plugin"
plugin_type = "detection"
capabilities = []
```

3. Create `main.lua`:
```lua
function on_load(config)
    prtip.log("info", "Plugin loaded")
    return true
end

function analyze_banner(banner)
    if string.match(banner, "MyService") then
        return { service = "myservice", confidence = 0.9 }
    end
    return nil
end
```

4. Test:
```bash
prtip --list-plugins
prtip -sS -p 80 target.com --plugin my-plugin
```

---

## Debugging

### Logging

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- -sS -p 80 127.0.0.1

# Enable trace logging for specific module
RUST_LOG=prtip_scanner=trace cargo run -- -sS -p 80 127.0.0.1

# Log to file
RUST_LOG=debug cargo run 2>&1 | tee debug.log
```

### GDB/LLDB

```bash
# Debug build with symbols
cargo build

# Run with GDB
gdb target/debug/prtip
(gdb) break main
(gdb) run -sS -p 80 127.0.0.1

# Run with LLDB (macOS)
lldb target/debug/prtip
(lldb) b main
(lldb) r -sS -p 80 127.0.0.1
```

### Common Issues

**Permission Denied:**
```bash
# Set capabilities (Linux)
sudo setcap cap_net_raw,cap_net_admin=eip target/release/prtip

# Or run with sudo
sudo cargo run --release -- -sS -p 80 target.com
```

**Npcap Not Found (Windows):**
- Install Npcap with "WinPcap API-compatible Mode"
- Restart your terminal after installation

**pcap Library Not Found:**
```bash
# Debian/Ubuntu
sudo apt-get install libpcap-dev

# Fedora/RHEL
sudo dnf install libpcap-devel

# macOS
brew install libpcap
```

---

## Release Process

### Version Bumping

1. Update version in all `Cargo.toml` files:
   - Root `Cargo.toml` (`[workspace.package].version`)
   - Each crate's `Cargo.toml` (if overriding)

2. Regenerate `Cargo.lock`:
```bash
cargo update
cargo build --locked
```

3. Update documentation:
   - CHANGELOG.md
   - README.md version badge
   - CLAUDE.md status line

### Release Checklist

```bash
# Quality gates
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo audit

# Build release
cargo build --release

# Create tag
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# GitHub release
gh release create v1.0.0 --notes-file RELEASE-NOTES.md
```

See [CONTRIBUTING.md](../CONTRIBUTING.md) for the complete release checklist.

---

## Additional Resources

- **Architecture**: [00-ARCHITECTURE.md](00-ARCHITECTURE.md)
- **Roadmap**: [01-ROADMAP.md](01-ROADMAP.md)
- **Testing**: [06-TESTING.md](06-TESTING.md)
- **Security**: [08-SECURITY.md](08-SECURITY.md)
- **Plugin System**: [30-PLUGIN-SYSTEM-GUIDE.md](30-PLUGIN-SYSTEM-GUIDE.md)
- **IPv6 Guide**: [23-IPv6-GUIDE.md](23-IPv6-GUIDE.md)
- **Rate Limiting**: [26-RATE-LIMITING-GUIDE.md](26-RATE-LIMITING-GUIDE.md)

---

**Questions?** Open an issue at https://github.com/doublegate/ProRT-IP/issues

**Happy Hacking!**
