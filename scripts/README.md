# ProRT-IP Scripts

Automation scripts for development, testing, and release workflows.

## Quick Reference

| Script | Purpose | Use When |
|--------|---------|----------|
| **setup-dev-env.sh** | Setup development environment | First-time setup, new contributors |
| **test-nmap-compat.sh** | Test nmap CLI compatibility | After CLI changes, before release |
| **run-benchmarks.sh** | Performance benchmarking suite | Performance validation, regression testing |
| **pre-release-check.sh** | Release readiness validation | Before creating releases |
| **run-integration-tests.sh** | Integration testing | Testing against real targets |
| **cross-compile.sh** | Build all release targets | Local release validation |
| **generate-release-notes.sh** | Auto-generate release notes | Creating releases |
| **update-docs-stats.sh** | Update documentation statistics | After major code changes |
| **network-latency.sh** | Add network latency for testing | Performance testing under latency |

## Development Scripts

### setup-dev-env.sh

**Purpose:** One-command development environment setup.

**What it does:**
- Detects platform (Linux/macOS/FreeBSD/Windows WSL)
- Installs Rust toolchain (1.85+)
- Installs system dependencies (libpcap, OpenSSL, pkg-config)
- Installs development tools (clippy, rustfmt, cargo-audit)
- Installs optional tools (hyperfine, perf, valgrind)
- Sets up git hooks (pre-commit formatting/linting)
- Runs smoke test to verify setup

**Usage:**
```bash
# Full setup with all tools
./scripts/setup-dev-env.sh

# Quick setup (skip benchmarking/profiling tools)
./scripts/setup-dev-env.sh --quick

# Setup without git hooks
./scripts/setup-dev-env.sh --no-hooks
```

**Prerequisites:** None (script installs everything)

**Exit Codes:**
- 0: Success
- 1: General error
- 2: Unsupported platform

---

### test-nmap-compat.sh

**Purpose:** Integration testing for nmap CLI compatibility (20+ flags).

**What it tests:**
- Scan type aliases (-sS, -sT, -sN, -sF, -sX)
- Backward compatibility (original ProRT-IP flags)
- Port specification (-F, --top-ports, -p)
- Output formats (-oN, -oX, -oG)
- Modes (-A, -Pn)
- Verbosity (-v, -vv, -vvv)
- Mixed syntax (nmap + ProRT-IP flags)

**Usage:**
```bash
# Run all tests (default)
./scripts/test-nmap-compat.sh

# Run quick validation
./scripts/test-nmap-compat.sh --quick

# Test specific binary
./scripts/test-nmap-compat.sh --binary /usr/local/bin/prtip

# Test against remote host
./scripts/test-nmap-compat.sh --target scanme.nmap.org
```

**Prerequisites:**
- Built prtip binary (`cargo build --release`)
- Network access

**Exit Codes:**
- 0: All tests passed
- 1: Some tests failed
- 2: Missing prerequisites

---

### run-benchmarks.sh

**Purpose:** Comprehensive performance benchmarking suite.

**Benchmark Categories:**
1. **Hyperfine Statistical Tests** (8 tests)
   - Common ports (100)
   - 1K, 10K, 65K port scans
   - Database mode
   - Timing templates (T0, T3, T5)
2. **CPU Profiling** (Linux only, --profile)
   - perf stat, flamegraph generation
3. **Memory Profiling** (Linux only, --profile)
   - valgrind massif, leak detection
4. **System Call Analysis** (Linux only, --profile)
   - strace syscall counting

**Usage:**
```bash
# Quick validation (5 minutes)
./scripts/run-benchmarks.sh --quick

# Full suite with profiling (30 minutes)
./scripts/run-benchmarks.sh --profile

# Compare against v0.3.5 baseline
./scripts/run-benchmarks.sh --compare v0.3.5

# Custom target and output
./scripts/run-benchmarks.sh --target 192.168.1.1 --output /tmp/benchmarks
```

**Prerequisites:**
- Built prtip binary
- hyperfine (`cargo install hyperfine`)
- Optional: perf, valgrind, flamegraph, strace

**Results:** Timestamped directory in `benchmarks/` with comprehensive reports.

---

### pre-release-check.sh

**Purpose:** Comprehensive release readiness validation.

**Checks Performed:**
1. Version consistency (Cargo.toml, README, CHANGELOG)
2. Git status (clean working tree, on main branch)
3. Code quality (fmt, clippy, audit)
4. Tests (full suite - 492 tests)
5. Documentation (broken links, outdated stats)
6. Performance (regression check if baseline exists)
7. Cross-compilation (8 release targets)

**Usage:**
```bash
# Full pre-release check
./scripts/pre-release-check.sh

# Auto-fix formatting and clippy issues
./scripts/pre-release-check.sh --fix

# Quick check (skip cross-compilation)
./scripts/pre-release-check.sh --skip-build
```

**Prerequisites:**
- Git repository
- Rust toolchain
- cross (for cross-compilation)

**Exit Codes:**
- 0: All checks passed (release ready)
- 1: Some checks failed (not release ready)
- 2: Missing prerequisites

---

### run-integration-tests.sh

**Purpose:** Integration tests against real targets.

**What it tests:**
- All scan types (Connect, SYN, UDP, stealth)
- Output formats (text, JSON, XML, greppable)
- Nmap compatibility
- Service detection (planned)

**Usage:**
```bash
# Test against localhost (default)
./scripts/run-integration-tests.sh

# Test against specific target
./scripts/run-integration-tests.sh --target 192.168.1.1

# Quick subset
./scripts/run-integration-tests.sh --quick
```

**Prerequisites:**
- Built prtip binary
- Network access to test target

---

### cross-compile.sh

**Purpose:** Build all 8 release targets locally.

**Targets:**
- x86_64-unknown-linux-gnu
- x86_64-unknown-linux-musl
- aarch64-unknown-linux-gnu
- aarch64-unknown-linux-musl
- x86_64-apple-darwin
- aarch64-apple-darwin
- x86_64-pc-windows-gnu
- x86_64-unknown-freebsd

**Usage:**
```bash
# Build all targets
./scripts/cross-compile.sh
```

**Prerequisites:**
- cross (`cargo install cross`)
- Docker (required by cross)

**Output:** Binaries in `target/<TARGET>/release/prtip`

---

### generate-release-notes.sh

**Purpose:** Auto-generate release notes from git commits.

**What it generates:**
- Features (commits with `feat:`)
- Bug fixes (commits with `fix:`)
- Performance improvements (commits with `perf:`)
- Documentation changes (commits with `docs:`)
- Statistics (commits, files changed, contributors)

**Usage:**
```bash
# Generate notes since last tag
./scripts/generate-release-notes.sh

# Generate notes from specific version
./scripts/generate-release-notes.sh v0.3.5
```

**Prerequisites:**
- Git repository with tags
- Conventional commit messages

**Output:** Markdown formatted release notes (stdout)

---

### update-docs-stats.sh

**Purpose:** Automatically update documentation statistics.

**What it counts:**
- Tests
- Lines of code
- Modules
- Scan types
- Protocols

**Usage:**
```bash
./scripts/update-docs-stats.sh
```

**Output:** Statistics printed to stdout (manual update to docs)

---

## Testing Scripts

### network-latency.sh

**Purpose:** Add artificial network latency using Linux tc (traffic control).

**Common Scenarios:**
- LAN testing: 10ms (20ms RTT)
- WAN testing: 50ms (100ms RTT)
- Internet testing: 100ms (200ms RTT)
- Satellite link: 300ms (600ms RTT)

**Usage:**
```bash
# Add 50ms latency to docker0 (100ms RTT)
sudo ./scripts/network-latency.sh add docker0 50ms

# Quick docker latency setup
sudo ./scripts/network-latency.sh docker 25ms

# Remove latency from docker0
sudo ./scripts/network-latency.sh remove docker0

# Show current configuration
./scripts/network-latency.sh show docker0
```

**Prerequisites:**
- Linux kernel with tc (traffic control) support
- Root privileges (sudo)
- iproute2 package

---

## Deprecated Scripts (Archived)

The following scripts have been moved to `scripts/archive/sprint-4.12-validation/`:

- **final-test.sh** - Sprint 4.12 progress bar validation (resolved)
- **progress-test-matrix.sh** - Sprint 4.12 progress visibility matrix (resolved)
- **test-progress-visibility.sh** - Sprint 4.12 progress visibility tests (resolved)

These scripts were created to validate the Sprint 4.12 progress bar fix and are no longer needed in v0.3.6.

---

## Script Standards

All scripts follow these standards:

- ✅ `set -euo pipefail` (strict error handling)
- ✅ Header with purpose, usage, examples, exit codes
- ✅ Help/usage function (`--help` flag)
- ✅ Prerequisite checks (tools, permissions)
- ✅ Colored output (success/warning/error)
- ✅ Proper exit codes (0=success, 1=error, 2=missing deps)
- ✅ Cleanup on exit (temp files, servers)
- ✅ Platform detection (Linux/macOS/FreeBSD)
- ✅ Relative paths (configurable via env vars)
- ✅ Version flag (`--version` where applicable)

---

## CI/CD Integration

Scripts designed for CI/CD:

- **test-nmap-compat.sh** - GitHub Actions integration tests
- **run-benchmarks.sh --quick** - Performance regression checks
- **pre-release-check.sh --skip-build** - Fast validation

Example GitHub Actions:

```yaml
- name: Run nmap compatibility tests
  run: ./scripts/test-nmap-compat.sh

- name: Quick performance check
  run: ./scripts/run-benchmarks.sh --quick

- name: Pre-release validation
  run: ./scripts/pre-release-check.sh --skip-build
```

---

## Development Workflow

### First-Time Setup
```bash
# 1. Setup environment
./scripts/setup-dev-env.sh

# 2. Build release binary
cargo build --release

# 3. Run tests
cargo test
```

### Before Committing
```bash
# Format and lint (or let git hooks do it)
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

### Before Pull Request
```bash
# Run all tests
cargo test

# Test nmap compatibility
./scripts/test-nmap-compat.sh

# Run integration tests
./scripts/run-integration-tests.sh
```

### Before Release
```bash
# Run full pre-release check
./scripts/pre-release-check.sh

# Generate release notes
./scripts/generate-release-notes.sh > release-notes.md

# Build all targets (optional, CI does this)
./scripts/cross-compile.sh
```

---

## Troubleshooting

### Script Permission Denied
```bash
chmod +x scripts/*.sh
```

### Binary Not Found
```bash
# Build release binary
cargo build --release
```

### cross Not Found (cross-compile.sh)
```bash
cargo install cross
```

### hyperfine Not Found (run-benchmarks.sh)
```bash
cargo install hyperfine
```

### Platform-Specific Issues

**Linux:**
- Install libpcap: `sudo apt-get install libpcap-dev`
- Install perf: `sudo apt-get install linux-tools-generic`
- Install valgrind: `sudo apt-get install valgrind`

**macOS:**
- Install Homebrew: https://brew.sh
- Install libpcap: `brew install libpcap`
- valgrind not officially supported on macOS

**FreeBSD:**
- Install dependencies: `sudo pkg install libpcap openssl pkgconf`

---

## Contributing

When adding new scripts:

1. Follow script standards (see above)
2. Add comprehensive documentation to this README
3. Test on Linux, macOS, and FreeBSD (if possible)
4. Add error handling and prerequisite checks
5. Include usage examples

---

**Last Updated:** 2025-10-12 | **Version:** v0.3.6
