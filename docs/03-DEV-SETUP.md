# ProRT-IP WarScan: Development Setup Guide

**Version:** 1.0
**Last Updated:** October 2025

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Platform-Specific Setup](#platform-specific-setup)
3. [Building the Project](#building-the-project)
4. [Development Tools](#development-tools)
5. [Testing Environment](#testing-environment)
6. [IDE Configuration](#ide-configuration)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Software

#### Rust Toolchain (1.85+)

```bash
# Install rustup (cross-platform Rust installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Configure current shell
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should be 1.85.0 or higher
cargo --version
```

#### Git

```bash
# Linux (Debian/Ubuntu)
sudo apt install git

# Linux (Fedora)
sudo dnf install git

# macOS
brew install git

# Windows
# Download from https://git-scm.com/download/win
```

### System Libraries

Network programming requires platform-specific libraries for raw packet access.

---

## Platform-Specific Setup

### Linux

#### Debian/Ubuntu

```bash
# Update package lists
sudo apt update

# Install development tools
sudo apt install -y \
    build-essential \
    pkg-config \
    libpcap-dev \
    libssl-dev \
    cmake

# Optional: Install setcap for capability management
sudo apt install -y libcap2-bin

# Optional: Performance profiling tools
sudo apt install -y \
    linux-tools-generic \
    linux-tools-$(uname -r) \
    valgrind
```

#### Fedora/RHEL

```bash
# Install development tools
sudo dnf groupinstall "Development Tools"

# Install libraries
sudo dnf install -y \
    libpcap-devel \
    openssl-devel \
    cmake

# Optional: Performance tools
sudo dnf install -y \
    perf \
    valgrind
```

#### Arch Linux

```bash
# Install dependencies
sudo pacman -S \
    base-devel \
    libpcap \
    openssl \
    cmake

# Optional: Performance tools
sudo pacman -S \
    perf \
    valgrind
```

#### Capabilities Setup (Recommended)

Instead of running as root, grant specific capabilities:

```bash
# After building the project
sudo setcap cap_net_raw,cap_net_admin=eip target/release/prtip

# Verify capabilities
getcap target/release/prtip
# Output: target/release/prtip = cap_net_admin,cap_net_raw+eip
```

---

### Windows

#### Prerequisites

1. **Visual Studio Build Tools**
   - Download from: <https://visualstudio.microsoft.com/downloads/>
   - Install "Desktop development with C++"
   - Include: MSVC compiler, Windows SDK

2. **Npcap**

   ```powershell
   # Download Npcap installer from:
   # https://npcap.com/dist/npcap (latest version)

   # Install with options:
   # - [x] Install Npcap in WinPcap API-compatible mode
   # - [x] Support raw 802.11 traffic
   ```

3. **Npcap SDK**

   ```powershell
   # Download SDK from:
   # https://npcap.com/dist/npcap-sdk-1.13.zip

   # Extract to: C:\npcap-sdk

   # Set environment variable
   setx NPCAP_SDK "C:\npcap-sdk"
   ```

#### OpenSSL (for SSL/TLS service detection)

```powershell
# Using vcpkg (recommended)
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat
.\vcpkg integrate install
.\vcpkg install openssl:x64-windows

# Or download pre-built binaries:
# https://slproweb.com/products/Win32OpenSSL.html
```

#### Build Configuration

Create `.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-msvc]
rustflags = [
    "-C", "link-arg=/LIBPATH:C:\\npcap-sdk\\Lib\\x64",
]

[build]
target = "x86_64-pc-windows-msvc"
```

---

### macOS

#### Install Xcode Command Line Tools

```bash
xcode-select --install
```

#### Install Homebrew (if not already installed)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### Install Dependencies

```bash
# libpcap (usually pre-installed, but install latest)
brew install libpcap

# OpenSSL
brew install openssl@3

# Link OpenSSL for pkg-config
echo 'export PKG_CONFIG_PATH="/usr/local/opt/openssl@3/lib/pkgconfig"' >> ~/.zshrc
source ~/.zshrc
```

#### Setup BPF Device Permissions

```bash
# Create access_bpf group (if not exists)
sudo dscl . -create /Groups/access_bpf
sudo dscl . -create /Groups/access_bpf PrimaryGroupID 1001
sudo dscl . -create /Groups/access_bpf GroupMembership $(whoami)

# Install ChmodBPF script
curl -O https://raw.githubusercontent.com/wireshark/wireshark/master/packaging/macosx/ChmodBPF/ChmodBPF
sudo mv ChmodBPF /Library/StartupItems/
sudo chmod +x /Library/StartupItems/ChmodBPF/ChmodBPF

# Or use Wireshark's installer which includes ChmodBPF
brew install --cask wireshark
```

---

## Building the Project

### Clone Repository

```bash
git clone https://github.com/doublegate/ProRT-IP.git
cd prtip-warscan
```

### Project Structure

```
prtip-warscan/
├── Cargo.toml           # Workspace manifest
├── Cargo.lock           # Dependency lock file
├── src/                 # Main source code (future)
├── crates/              # Workspace crates (future)
│   ├── core/            # Core scanning engine
│   ├── net/             # Network protocol implementation
│   ├── detect/          # OS/service detection
│   ├── plugins/         # Plugin system
│   └── ui/              # User interfaces (CLI, TUI)
├── tests/               # Integration tests
├── benches/             # Performance benchmarks
├── docs/                # Documentation
└── scripts/             # Build and development scripts
```

### Build Commands

#### Development Build

```bash
# Build with debug symbols and optimizations disabled
cargo build

# Binary location: target/debug/prtip
```

#### Release Build

```bash
# Build with full optimizations
cargo build --release

# Binary location: target/release/prtip
```

#### Build with Specific Features

```bash
# Build without Lua plugin support
cargo build --release --no-default-features

# Build with all optional features
cargo build --release --all-features

# Build with specific features
cargo build --release --features "lua-plugins,python-plugins"
```

#### Cross-Compilation

```bash
# Install cross-compilation target
rustup target add x86_64-unknown-linux-musl

# Build for musl (static linking)
cargo build --release --target x86_64-unknown-linux-musl
```

---

## Development Tools

### Recommended Cargo Extensions

```bash
# Code formatting
cargo install cargo-fmt
rustup component add rustfmt

# Linting
rustup component add clippy

# Security auditing
cargo install cargo-audit

# Test coverage
cargo install cargo-tarpaulin  # Linux only

# Benchmarking
cargo install cargo-criterion

# Dependency tree visualization
cargo install cargo-tree

# License checking
cargo install cargo-license

# Bloat analysis
cargo install cargo-bloat

# Unused dependency detection
cargo install cargo-udeps
```

### Code Quality Checks

#### Format Code

```bash
# Check formatting (CI mode)
cargo fmt --check

# Auto-format all code
cargo fmt
```

#### Lint Code

```bash
# Run clippy with pedantic warnings
cargo clippy -- -D warnings -W clippy::pedantic

# Fix automatically where possible
cargo clippy --fix
```

#### Security Audit

```bash
# Check for known vulnerabilities in dependencies
cargo audit

# Update advisory database
cargo audit fetch
```

### Performance Profiling

#### Linux: perf + flamegraph

```bash
# Build with debug symbols in release mode
RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes" cargo build --release

# Record performance data
sudo perf record --call-graph dwarf -F 997 ./target/release/prtip [args]

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg

# View in browser
firefox flame.svg
```

#### Cross-Platform: Criterion Benchmarks

```bash
# Run benchmarks
cargo bench

# View HTML report
firefox target/criterion/report/index.html
```

#### Memory Profiling (Linux)

```bash
# Check for memory leaks
valgrind --leak-check=full --show-leak-kinds=all ./target/debug/prtip [args]

# Heap profiling with massif
valgrind --tool=massif ./target/debug/prtip [args]
ms_print massif.out.12345 > massif.txt
```

---

## Testing Environment

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_tcp_checksum

# Run tests with output
cargo test -- --nocapture

# Run tests in single thread (useful for network tests)
cargo test -- --test-threads=1
```

### Integration Tests

```bash
# Run only integration tests
cargo test --test integration_tests

# Run with logging
RUST_LOG=debug cargo test
```

### Test Coverage

```bash
# Linux only (requires cargo-tarpaulin)
cargo tarpaulin --out Html --output-dir coverage

# View report
firefox coverage/index.html
```

### Test Network Setup

#### Create Isolated Test Environment

```bash
# Linux: Create network namespace for testing
sudo ip netns add prtip-test
sudo ip netns exec prtip-test bash

# Inside namespace, setup loopback
ip link set lo up

# Run tests in isolated namespace
cargo test
```

#### Docker Test Environment

```bash
# Build test container
docker build -t prtip-test -f Dockerfile.test .

# Run tests in container
docker run --rm -it prtip-test cargo test
```

---

## IDE Configuration

### Visual Studio Code

#### Recommended Extensions

- `rust-lang.rust-analyzer` - Rust language server
- `vadimcn.vscode-lldb` - Native debugger
- `serayuzgur.crates` - Dependency version management
- `tamasfe.even-better-toml` - TOML syntax support

#### `.vscode/settings.json`

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "editor.rulers": [100],
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

#### `.vscode/launch.json`

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug prtip",
      "cargo": {
        "args": ["build", "--bin=prtip"],
        "filter": {
          "name": "prtip",
          "kind": "bin"
        }
      },
      "args": ["-sS", "-p", "80,443", "192.168.1.0/24"],
      "cwd": "${workspaceFolder}",
      "preLaunchTask": "cargo-build"
    }
  ]
}
```

### IntelliJ IDEA / CLion

#### Install Rust Plugin

- File → Settings → Plugins → Search "Rust" → Install

#### Project Configuration

- Open `Cargo.toml` as project
- Enable "Use rustfmt instead of built-in formatter"
- Set "External linter" to Clippy
- Configure run configurations for different scan types

### Vim/Neovim

#### Using coc.nvim

```vim
" Install coc-rust-analyzer
:CocInstall coc-rust-analyzer

" Add to .vimrc or init.vim
autocmd FileType rust set colorcolumn=100
autocmd FileType rust set expandtab shiftwidth=4 softtabstop=4
```

---

## Troubleshooting

### Common Build Issues

#### "libpcap not found"

**Linux:**

```bash
sudo apt install libpcap-dev  # Debian/Ubuntu
sudo dnf install libpcap-devel  # Fedora
```

**macOS:**

```bash
brew install libpcap
export PKG_CONFIG_PATH="/usr/local/lib/pkgconfig"
```

**Windows:**

```
Ensure Npcap SDK is installed and NPCAP_SDK environment variable is set
```

#### "OpenSSL not found"

**Linux:**

```bash
sudo apt install libssl-dev pkg-config
```

**macOS:**

```bash
brew install openssl@3
export PKG_CONFIG_PATH="/usr/local/opt/openssl@3/lib/pkgconfig"
```

**Windows:**

```powershell
# Install OpenSSL via vcpkg or download binaries
# Set OPENSSL_DIR environment variable
setx OPENSSL_DIR "C:\Program Files\OpenSSL-Win64"
```

#### "Permission denied" on packet capture

**Linux:**

```bash
# Option 1: Use capabilities (recommended)
sudo setcap cap_net_raw,cap_net_admin=eip target/release/prtip

# Option 2: Run as root (not recommended for development)
sudo ./target/release/prtip [args]
```

**macOS:**

```bash
# Ensure you're in access_bpf group
groups | grep access_bpf

# If not, add yourself (requires logout/login)
sudo dseditgroup -o edit -a $(whoami) -t user access_bpf
```

**Windows:**

```
Run terminal as Administrator
```

#### Linker errors on Windows

```powershell
# Ensure Visual Studio Build Tools are installed with C++ support
# Install Windows SDK 10

# Check environment variables
echo %LIB%
echo %INCLUDE%
```

### Runtime Issues

#### "Cannot create raw socket"

This usually indicates insufficient privileges. See solutions in "Permission denied" section above.

#### High CPU usage during compilation

```bash
# Limit parallel compilation jobs
cargo build -j 2

# Or set permanently in ~/.cargo/config.toml
[build]
jobs = 2
```

#### Out of memory during linking

```bash
# Use lld for faster linking with less memory
rustup component add lld

# Add to .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### Testing Issues

#### Tests failing with "Address already in use"

```bash
# Run tests serially instead of parallel
cargo test -- --test-threads=1
```

#### Tests timing out on slow networks

```bash
# Increase test timeout
cargo test -- --test-timeout=300
```

---

## Environment Variables

### Build-Time Variables

```bash
# Set Rust backtrace for debugging
export RUST_BACKTRACE=1          # Short backtrace
export RUST_BACKTRACE=full       # Full backtrace

# Logging during build
export RUST_LOG=debug

# Compilation flags
export RUSTFLAGS="-C target-cpu=native"  # Optimize for current CPU
```

### Runtime Variables

```bash
# Logging verbosity
export RUST_LOG=prtip=debug,tower=info

# Custom config file location
export PRTIP_CONFIG=/etc/prtip/config.toml

# Override default database path
export PRTIP_DB_PATH=/var/lib/prtip/scans.db
```

---

## Continuous Integration

The project uses GitHub Actions for CI/CD with automated testing and release management.

### CI Workflows

**ci.yml** - Continuous Integration:

- Format check: `cargo fmt --check`
- Clippy lint: `cargo clippy -- -D warnings`
- Multi-platform testing: Linux, Windows, macOS
- Security audit: `cargo audit`
- MSRV verification: Rust 1.82+

**release.yml** - Release Automation:

- Triggers on git tags: `v*.*.*`
- Multi-platform binary builds:
  - `x86_64-unknown-linux-gnu` (glibc)
  - `x86_64-unknown-linux-musl` (static)
  - `x86_64-pc-windows-msvc` (Windows)
  - `x86_64-apple-darwin` (macOS)
- Automatic GitHub release creation
- Binary artifacts upload

**dependency-review.yml** - PR Security:

- Scans for vulnerable dependencies
- Detects malicious packages
- Automated on all pull requests

**codeql.yml** - Security Analysis:

- Advanced security scanning with CodeQL
- Weekly scheduled runs
- SARIF upload to GitHub Security tab

### Local Testing

Test formatting/linting before pushing to save CI time:

```bash
# Check formatting
cargo fmt --all -- --check

# Run clippy (strict mode)
cargo clippy --workspace --all-targets -- -D warnings

# Run tests
cargo test --workspace

# Build release
cargo build --release --workspace

# Security audit
cargo install cargo-audit
cargo audit
```

### CI Optimization

The CI pipeline uses aggressive 3-tier caching:

1. **Cargo registry** (~100-500 MB): Downloaded crate metadata
2. **Cargo index** (~50-200 MB): Git index for crates.io
3. **Build cache** (~500 MB - 2 GB): Compiled dependencies

**Performance benefits:**

- Clean build: 5-10 minutes
- Cached build: 1-2 minutes (80-90% speedup)
- Cache hit rate: ~80-90% for typical changes

### Workflow Status

Check workflow runs: [GitHub Actions](https://github.com/doublegate/ProRT-IP/actions)

**Status badges** (add to README):

```markdown
[![CI](https://github.com/doublegate/ProRT-IP/actions/workflows/ci.yml/badge.svg)](https://github.com/doublegate/ProRT-IP/actions/workflows/ci.yml)
[![Release](https://github.com/doublegate/ProRT-IP/actions/workflows/release.yml/badge.svg)](https://github.com/doublegate/ProRT-IP/actions/workflows/release.yml)
```

---

## Next Steps

After completing setup:

1. Build the project: `cargo build --release`
2. Run tests: `cargo test`
3. Review [Architecture Overview](00-ARCHITECTURE.md)
4. Check [Technical Specifications](02-TECHNICAL-SPECS.md) for implementation details
5. Begin development following [Roadmap](01-ROADMAP.md)
6. Review CI/CD Workflows in `.github/workflows/` for automation details

---

## Getting Help

- **Documentation:** See `docs/` directory
- **Issues:** GitHub Issues for bug reports
- **Discussions:** GitHub Discussions for questions
- **CI/CD:** See `.github/workflows/` for workflow documentation
- **Chat:** Join project Discord/Matrix (TBD)
