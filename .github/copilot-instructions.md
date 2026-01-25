# GitHub Copilot Custom Instructions for ProRT-IP

## Project Overview

**ProRT-IP WarScan** is a high-performance network scanner implemented in Rust that combines the speed of Masscan/ZMap with the detection depth of Nmap. This is a penetration testing and red team tool designed for security professionals.

- **Current Version**: v0.5.9 (Phase 6 Complete)
- **Test Coverage**: 2,557 tests passing, 51.40% coverage
- **License**: GPL-3.0
- **Repository**: https://github.com/doublegate/ProRT-IP

## Architecture & Key Design Decisions

### Hybrid Scanning Approach
- **Fast Stateless Discovery**: Masscan-style speed (10M+ packets per second)
- **Deep Stateful Enumeration**: Nmap-style detection and fingerprinting
- **Stream to Disk**: Handles internet-scale scanning without memory exhaustion

### Technology Stack
```toml
tokio = "1.35"     # Async runtime (multi-threaded)
pnet = "0.35"      # Cross-platform packet manipulation
clap = "4.5"       # CLI argument parsing
sqlx = "0.8"       # Async SQL operations
ratatui = "0.29"   # Terminal UI framework
```

### Core Architectural Decisions
1. **Async Runtime**: Tokio multi-threaded runtime with CPU-core workers
2. **Lock-Free Communication**: Crossbeam channels for inter-thread communication
3. **Raw Sockets**: Cross-platform support (AF_PACKET/Npcap/BPF via pnet)
4. **Privilege Management**: Create elevated → drop immediately → run unprivileged
5. **Zero-Copy**: For packets >10KB to minimize memory pressure

## Project Structure

This is a Rust workspace with multiple crates:
- `crates/prtip-core`: Core data structures and utilities
- `crates/prtip-network`: Packet manipulation and network I/O
- `crates/prtip-scanner`: Scanning orchestration and algorithms
- `crates/prtip-cli`: Command-line interface
- `crates/prtip-tui`: Terminal user interface (TUI)
- `tests/`: Integration tests
- `fuzz/`: Fuzz testing targets
- `docs/`: Technical documentation
- `ref-docs/`: Reference specifications

## Coding Standards

### Rust Code Style
- **Edition**: Rust 2021
- **Formatting**: Use `cargo fmt` (rustfmt defaults)
- **Indentation**: 4 spaces
- **Naming Conventions**:
  - `snake_case` for functions and variables
  - `SCREAMING_SNAKE_CASE` for constants
  - `CamelCase` for types and traits
- **Documentation**: All public items must have `///` doc comments with examples where practical
- **Error Handling**: Use `Result` or `anyhow::Result`; avoid `unwrap()` outside tests
- **Comments**: Add brief comments explaining protocol assumptions for networking code

### Code Quality Requirements
- **No Warnings**: `cargo clippy --workspace --all-targets -- -D warnings` must pass
- **Format Check**: `cargo fmt --check` must pass before commits
- **Test Coverage**: Maintain >60% overall coverage, >90% for core modules
- **Security**: Follow `docs/08-SECURITY.md` requirements

## Development Workflow

### Building
```bash
cargo build                    # Debug build
cargo build --release         # Optimized release build
```

### Testing
```bash
cargo test --workspace --all-targets    # Run all tests
cargo test --doc                        # Run doc tests
cargo test -p prtip-core                # Test specific crate
```

### Linting
```bash
cargo fmt                                              # Format code
cargo fmt --check                                      # Check formatting
cargo clippy --workspace --all-targets -- -D warnings  # Lint with zero warnings
```

### Running Locally
```bash
cargo run -p prtip-cli -- --help        # Show CLI help
cargo run -p prtip-cli -- -sS -p 1-1000 10.0.0.0/24  # Example SYN scan
```

## Security Requirements

This project handles sensitive network operations and requires strict security practices:

### Input Validation
- Use `IpAddr::parse()` for IP addresses
- Use `ipnetwork` crate for CIDR parsing
- Implement allowlist validation at all trust boundaries
- Never trust user input for packet construction

### Privilege Handling
- Create sockets with elevated privileges
- Drop privileges immediately after socket creation
- Run main application logic unprivileged

### Packet Parsing
- Use pnet/etherparse with bounds checking
- Return `Option`/`Result` for malformed packets
- Never panic on malformed network data

### DoS Prevention
- Use `tokio::sync::Semaphore` to bound concurrent operations
- Stream results to disk for large scans
- Implement adaptive rate limiting
- Respect system resource limits

## Testing Guidelines

### Test Organization
- **Unit Tests**: `#[cfg(test)]` modules alongside code
- **Integration Tests**: `tests/` directory for cross-crate scenarios
- **Doc Tests**: Runnable examples in `///` doc comments
- **Fuzz Tests**: `fuzz/` directory for property-based testing

### Test Naming
- Name test functions with `test_*` prefix
- Use descriptive names: `test_syn_scan_detects_open_port`

### Async Tests
```rust
#[tokio::test]
async fn test_async_operation() {
    // Async test code
}
```

### CI Expectations
Before opening a PR, ensure:
1. `cargo fmt --check` passes
2. `cargo clippy --workspace --all-targets -- -D warnings` passes
3. `cargo test --workspace --all-targets` passes
4. New features have test coverage for both success and failure paths

## CLI Design Principles

### Nmap Compatibility
ProRT-IP maintains compatibility with 50+ Nmap flags:
- Scan types: `-sS`, `-sT`, `-sU`, `-sF`, `-sN`, `-sX`, `-sA`, `-sI`
- Port specs: `-p`, `-F`, `--top-ports`
- Output: `-oN`, `-oX`, `-oG`, `-oJ`
- Timing: `-T0` through `-T5`
- Evasion: `-f`, `--mtu`, `--ttl`, `-D`, `--badsum`, `-g`

### Binary Name
The CLI binary is named `prtip` (not `prtip-cli`)

### Example Usage
```bash
prtip -sS -p 1-1000 10.0.0.0/24          # SYN scan
prtip -F 192.168.1.1                      # Fast scan (top 100 ports)
prtip -A -p 80,443 target.com             # Aggressive scan with detection
```

## Documentation

### Key Documentation Files
- **Root Level**:
  - `README.md`: Project overview and quick start
  - `ROADMAP.md`: Development phases and milestones
  - `CONTRIBUTING.md`: Contribution guidelines
  - `SECURITY.md`: Security policy and reporting
  - `CHANGELOG.md`: Version history
  - `CLAUDE.md`: AI assistant guidance (this is the source of truth)
  - `CLAUDE.local.md`: Session-by-session development log

- **Technical Docs** (`docs/`):
  - `00-ARCHITECTURE.md`: System design
  - `01-ROADMAP.md`: Detailed sprint planning
  - `03-DEV-SETUP.md`: Development environment setup
  - `04-IMPLEMENTATION-GUIDE.md`: Code structure guide
  - `06-TESTING.md`: Testing strategy
  - `07-PERFORMANCE.md`: Performance benchmarks
  - `08-SECURITY.md`: Security audit checklist
  - `10-PROJECT-STATUS.md`: Current task tracking
  - `TUI-ARCHITECTURE.md`: Terminal UI design

- **Reference** (`ref-docs/`): Technical specifications (36KB-190KB guides)

### Documentation Updates
When making changes:
- Update `CHANGELOG.md` for user-visible changes
- Update relevant docs in `docs/` for architectural changes
- Keep `CLAUDE.local.md` updated with session notes
- Maintain accurate module-level docs in code

## Capabilities & Features

### Scan Types (8 Total)
- **TCP**: SYN (default), Connect, FIN/NULL/Xmas (stealth), ACK (firewall mapping), Idle (anonymity)
- **UDP**: Protocol-specific payloads (DNS, SNMP, NetBIOS) with ICMP interpretation

### Detection Capabilities
- **Service Detection**: 187 probes, 85-90% accuracy
- **OS Fingerprinting**: 16-probe system, 2,600+ signature database
- **TLS Detection**: Certificate chain analysis

### Performance Features
- **Packet Rate**: 10M+ packets per second (stateless mode)
- **Scale**: Internet-scale IPv4/IPv6 scanning
- **Timing Profiles**: T0 (paranoid) through T5 (insane)
- **TUI**: 60 FPS, 4-tab dashboard, handles 10K+ events/sec

### Output Formats
- Text (colorized terminal output)
- JSON (structured data)
- XML (Nmap-compatible)
- Greppable (line-based parsing)
- PCAPNG (packet capture)
- SQL (database export)

## Common Patterns

### Error Handling
```rust
use anyhow::{Context, Result};

fn parse_target(input: &str) -> Result<IpAddr> {
    input.parse()
        .context("Failed to parse IP address")
}
```

### Async Network Operations
```rust
use tokio::net::TcpStream;

async fn connect_scan(addr: SocketAddr) -> Result<bool> {
    match tokio::time::timeout(
        Duration::from_secs(3),
        TcpStream::connect(addr)
    ).await {
        Ok(Ok(_)) => Ok(true),
        _ => Ok(false),
    }
}
```

### Privilege Dropping
```rust
// Create socket with elevated privileges
let socket = create_raw_socket()?;

// Drop privileges immediately
drop_privileges()?;

// Continue with unprivileged operations
```

## Important Notes

### Platform-Specific Considerations
- **Linux**: Requires CAP_NET_RAW capability or root for raw sockets
- **Windows**: Requires Npcap; older versions have 90s initialization delay
- **macOS**: Requires ChmodBPF or root; FIN/NULL/Xmas scans don't work on some networks

### Limitations
- **FIN/NULL/Xmas scans**: Don't work reliably on Windows or through Cisco equipment
- **UDP scanning**: ~10-100x slower than TCP due to protocol characteristics
- **Rate limiting**: Required to prevent DoS; default uses adaptive rate limiting

### Release Standards
- **Tag Messages**: 100-150 lines with executive summary, features, metrics, technical details
- **GitHub Releases**: 150-200 lines with all tag content plus links, installation, platform matrix
- **Examples**: See v0.3.7-v0.3.9, v0.4.0-v0.4.5 for reference

## Historical Context

### Development Phases
| Phase | Status | Tests | Key Features |
|-------|--------|-------|--------------|
| 1-3 | ✅ Complete | 391 | Core scanning, protocols, detection |
| 4 | ✅ Complete | 1,166 | Zero-copy, NUMA, PCAPNG, evasion, IPv6 foundation |
| 5 | ✅ Complete | 868 | IPv6 100%, Idle scan, Service detection, Rate limiting, TLS |
| 6 | ✅ Complete | 2,557 | TUI (60 FPS), Dashboard, CDN filtering, Network optimizations, Buffer Pool |

### Current Status (as of latest update)
- **Phase 6**: Production-ready TUI with real-time visualization
- **Tests**: 2,557 passing (96 ignored for platform-specific reasons)
- **Coverage**: 51.40% overall (>90% in core modules)
- **CI**: GitHub Actions with Linux, macOS, Windows matrix
- **Fuzz Testing**: 230M+ executions, 0 crashes

## Quick Reference

### Pre-Commit Checklist
1. `cargo fmt` - Format code
2. `cargo clippy --workspace --all-targets -- -D warnings` - Lint
3. `cargo test --workspace --all-targets` - Run tests
4. `cargo build --release` - Ensure release builds
5. Update documentation if behavior changed
6. Update CHANGELOG.md for user-visible changes

### Common Commands
```bash
# Development
cargo check                                    # Fast compile check
cargo build                                    # Debug build
cargo test                                     # Run tests
cargo doc --open                               # Generate and open docs

# Quality
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cargo audit                                    # Security audit

# Performance
cargo bench                                    # Run benchmarks
cargo build --release --features performance   # Optimized build
```

### Getting Help
- Read `CLAUDE.md` for detailed AI assistant guidance
- Check `docs/` directory for technical documentation
- Review `CONTRIBUTING.md` for contribution process
- See `SECURITY.md` for security concerns
- Check GitHub Issues for known problems

## Maintenance

### Weekly Tasks
- Run `cargo audit` to check for security vulnerabilities
- Update CLAUDE.local.md with session notes
- Sync PROJECT-STATUS.md with current work

### Before Releases
- Review `08-SECURITY.md` checklist
- Update CHANGELOG.md with all changes
- Run full test suite across platforms
- Update version numbers in Cargo.toml files
- Tag release with comprehensive notes (see Release Standards)

## Additional Context

This is a professional-grade security tool used for:
- Network reconnaissance
- Penetration testing
- Security auditing
- Red team operations

Users must have authorization to scan target networks. The tool includes:
- User confirmation for internet-scale scans
- Audit logging capabilities
- Rate limiting to prevent accidental DoS
- Clear warnings about legal and ethical usage

When contributing code:
- Prioritize security and correctness over convenience
- Consider performance implications of changes
- Maintain cross-platform compatibility
- Write comprehensive tests
- Document security-relevant behavior
