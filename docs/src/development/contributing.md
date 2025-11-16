# Contributing to ProRT-IP

Guide for contributing to ProRT-IP including code of conduct, development workflow, coding standards, and community engagement.

---

## Quick Reference

**Code of Conduct**: Be respectful, inclusive, and professional
**License**: GPL-3.0 (contributions must be compatible)
**Development**: Fork → Branch → Implement → Test → PR
**Communication**: GitHub Issues, Discussions, Pull Requests
**Quality Gates**: Tests passing, zero clippy warnings, formatted code, documentation updated
**Review Time**: 2-7 days for most PRs, critical fixes faster

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of:
- Experience level (beginner to expert)
- Background or identity
- Geographic location
- Age or generation
- Personal opinions or beliefs

### Expected Behavior

**Do**:
- Be respectful and considerate in all interactions
- Provide constructive feedback with specific suggestions
- Accept feedback gracefully and professionally
- Focus on what is best for the project and community
- Help newcomers and answer questions patiently
- Credit others' contributions and ideas

**Don't**:
- Use offensive, discriminatory, or harassing language
- Make personal attacks or insults
- Troll, spam, or deliberately derail discussions
- Share others' private information without permission
- Engage in any behavior that would be unwelcome in a professional setting

### Enforcement

**Violations** will result in:
1. **First offense**: Private warning with explanation
2. **Second offense**: Temporary ban (7-30 days)
3. **Third offense**: Permanent ban from project spaces

**Report violations** to: <security@proRT-IP-project.org> (confidential)

---

## Getting Started

### Prerequisites

**Development environment**:
```bash
# Rust toolchain (1.85+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Development tools
cargo install cargo-clippy cargo-fmt cargo-audit

# Platform-specific packet capture library
# Linux
sudo apt install libpcap-dev

# macOS
brew install libpcap

# Windows
# Download and install Npcap from https://npcap.com/
```

**Clone repository**:
```bash
# Fork on GitHub first, then clone your fork
git clone https://github.com/YOUR_USERNAME/ProRT-IP.git
cd ProRT-IP

# Add upstream remote
git remote add upstream https://github.com/doublegate/ProRT-IP.git
```

**Build and test**:
```bash
# Build project
cargo build

# Run tests
cargo test

# Verify code quality
cargo clippy -- -D warnings
cargo fmt --check

# Build documentation
cargo doc --open
```

### First Contribution

**Good first issues** are labeled [`good first issue`](https://github.com/doublegate/ProRT-IP/labels/good%20first%20issue) on GitHub.

**Recommended path**:
1. Start with documentation improvements (typos, clarifications)
2. Add test coverage for existing features
3. Fix bugs with clear reproduction steps
4. Implement small, well-defined features
5. Tackle larger architectural changes

---

## Development Workflow

### Branching Strategy

**Branch naming**:
```
feature/descriptive-feature-name    # New features
bugfix/issue-number-short-desc      # Bug fixes
docs/documentation-improvement      # Documentation only
refactor/component-name             # Code refactoring
test/coverage-improvement           # Test additions
```

**Examples**:
```bash
✅ Good:
- feature/ipv6-udp-scanning
- bugfix/123-rate-limiter-overflow
- docs/api-reference-examples
- refactor/scanner-state-machine
- test/service-detection-coverage

❌ Bad:
- my-changes
- fix
- update-docs
- ipv6 (ambiguous)
```

### Feature Development Process

**1. Create issue first**:
```markdown
Title: Add IPv6 support for UDP scanning

**Feature Description:**
UDP scanning currently only supports IPv4 addresses. Extend to support IPv6.

**Use Case:**
Enable network scanning of IPv6-only networks and dual-stack environments.

**Proposed Implementation:**
- Extend UdpScanner to handle IPv6 addresses
- Add IPv6-specific ICMP response handling
- Update CLI to accept IPv6 targets

**Acceptance Criteria:**
- [ ] UdpScanner accepts IPv6 addresses
- [ ] ICMPv6 responses parsed correctly
- [ ] Tests cover IPv6 edge cases
- [ ] Documentation updated
```

**2. Get feedback before implementing**:
- Wait for maintainer response (2-3 days typical)
- Discuss approach and scope
- Clarify acceptance criteria
- Get approval to proceed

**3. Create feature branch**:
```bash
git checkout -b feature/ipv6-udp-scanning
```

**4. Implement with tests** (TDD approach):
```bash
# Write failing test first
cargo test udp_scanner::test_ipv6_scanning -- --exact
# (Test fails as expected)

# Implement feature
# Edit crates/prtip-scanner/src/udp.rs

# Verify test passes
cargo test udp_scanner::test_ipv6_scanning -- --exact
# (Test passes)

# Run full test suite
cargo test
```

**5. Commit with clear messages**:
```bash
git add .
git commit -m "feat: add IPv6 support to UDP scanner

- Extend UdpScanner to handle IPv6 addresses
- Implement ICMPv6 Destination Unreachable parsing
- Add integration tests for IPv6 UDP scanning
- Update CLI documentation with IPv6 examples

Resolves #123"
```

**6. Push and create PR**:
```bash
git push origin feature/ipv6-udp-scanning

# Create PR on GitHub with detailed description
```

### Commit Message Guidelines

**Format** (Conventional Commits):
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code restructuring (no behavior change)
- `test`: Test additions or improvements
- `perf`: Performance improvement
- `chore`: Build process, dependencies, tooling

**Examples**:
```
✅ Good:
feat(scanner): add IPv6 support to UDP scanner

Extend UdpScanner to handle IPv6 addresses and parse ICMPv6 responses.
Implementation uses dual-stack sockets when available for efficiency.

- Add IPv6Address type and parsing
- Implement ICMPv6 Destination Unreachable handling
- Add 15 integration tests for IPv6 edge cases
- Update CLI docs with IPv6 examples

Resolves #123

❌ Bad:
Added IPv6 support
(No type, no context, no details)

fix: stuff
(Vague, no explanation)

FEAT: ADD IPV6 SUPPORT
(Wrong capitalization, no details)
```

**Breaking changes**:
```
feat(scanner)!: change ScanConfig API to builder pattern

BREAKING CHANGE: ScanConfig no longer uses struct literals.
Migrate to builder pattern:

Before:
let config = ScanConfig { timeout_ms: 5000, ... };

After:
let config = ScanConfig::builder()
    .timeout_ms(5000)
    .build();

Resolves #456
```

---

## Coding Standards

### Rust Style Guide

**Follow official Rust style** (enforced by `rustfmt`):
```bash
# Auto-format code
cargo fmt

# Check formatting without modifying
cargo fmt --check
```

**Naming conventions**:
```rust
// Types: PascalCase
struct SynScanner { }
enum ScanType { }

// Functions and methods: snake_case
fn scan_target(&self, target: IpAddr) -> Result<ScanResult> { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;

// Modules: snake_case
mod tcp_scanner;
mod rate_limiter;
```

**Imports**:
```rust
// Standard library first
use std::net::IpAddr;
use std::time::Duration;

// External crates second (alphabetical)
use clap::Parser;
use tokio::sync::Mutex;

// Internal crates third
use prtip_network::packet::TcpPacket;
use prtip_scanner::ScanConfig;

// Internal modules last
use crate::error::ScanError;
use crate::scanner::SynScanner;
```

**Error handling**:
```rust
✅ Good:
// Return Result for fallible operations
fn parse_target(input: &str) -> Result<IpAddr, ScanError> {
    input.parse()
        .map_err(|_| ScanError::InvalidTarget(input.to_string()))
}

// Use ? operator for propagation
let target = parse_target(input)?;
let results = scanner.scan(target).await?;

❌ Bad:
// Never use unwrap() or expect() in production code
let target = parse_target(input).unwrap();

// Don't ignore errors
let _ = scanner.scan(target).await;
```

**Documentation**:
```rust
/// Performs TCP SYN scan on specified target and ports.
///
/// SYN scanning sends TCP SYN packets and analyzes responses to determine
/// port states without completing the TCP handshake (stealth).
///
/// # Arguments
///
/// * `target` - IPv4 or IPv6 address to scan
/// * `ports` - Port range to scan (e.g., 1-1000, 80,443,8080)
///
/// # Returns
///
/// Vector of `PortResult` containing open/closed/filtered states.
///
/// # Errors
///
/// Returns `ScanError` if:
/// - Target is unreachable
/// - Packet capture fails
/// - Timeout exceeded
///
/// # Examples
///
/// ```rust
/// use prtip_scanner::SynScanner;
///
/// let scanner = SynScanner::new(config)?;
/// let results = scanner.scan_ports("192.168.1.1", "80,443").await?;
///
/// for result in results {
///     println!("{}: {}", result.port, result.state);
/// }
/// ```
pub async fn scan_ports(&self, target: &str, ports: &str) -> Result<Vec<PortResult>, ScanError> {
    // Implementation
}
```

### Code Quality Standards

**Clippy lints** (zero warnings required):
```bash
# Run clippy with strict settings
cargo clippy --workspace -- -D warnings

# Fix common issues automatically
cargo clippy --fix
```

**Common clippy fixes**:
```rust
// Use if-let instead of match for single pattern
❌ match option {
    Some(value) => process(value),
    None => {}
}

✅ if let Some(value) = option {
    process(value);
}

// Avoid redundant clones
❌ let s = string.clone().to_uppercase();
✅ let s = string.to_uppercase();

// Use .copied() or .cloned() explicitly
❌ let values: Vec<_> = iter.map(|x| *x).collect();
✅ let values: Vec<_> = iter.copied().collect();
```

**Performance considerations**:
```rust
// Preallocate vectors when size is known
let mut results = Vec::with_capacity(1000);

// Use iterators instead of loops when appropriate
❌ let mut squares = Vec::new();
   for i in 1..=10 {
       squares.push(i * i);
   }

✅ let squares: Vec<_> = (1..=10).map(|i| i * i).collect();

// Avoid unnecessary allocations
❌ fn format_ip(ip: IpAddr) -> String {
       format!("{}", ip)
   }

✅ fn format_ip(ip: IpAddr) -> String {
       ip.to_string()
   }
```

---

## Testing Requirements

### Test Coverage

**Minimum coverage requirements**:
- **Core modules**: ≥90% (scanner, network, protocol parsing)
- **Support modules**: ≥70% (CLI, configuration, output)
- **Integration tests**: ≥80% of user-facing features
- **Overall target**: ≥60% (currently 54.92%, improving)

**Check coverage**:
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage/

# View report
open coverage/index.html
```

### Test Organization

**Unit tests** (in same file as code):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syn_packet_construction() {
        let packet = SynPacket::new(source_ip, dest_ip, dest_port);
        assert_eq!(packet.flags(), TcpFlags::SYN);
        assert_eq!(packet.dest_port(), dest_port);
    }

    #[test]
    #[should_panic(expected = "Invalid port")]
    fn test_invalid_port_panics() {
        SynPacket::new(source_ip, dest_ip, 0); // Port 0 invalid
    }
}
```

**Integration tests** (`tests/` directory):
```rust
// tests/syn_scanner_integration.rs
use prtip_scanner::SynScanner;
use std::net::IpAddr;

#[tokio::test]
async fn test_syn_scan_localhost() {
    let scanner = SynScanner::new(default_config()).unwrap();
    let results = scanner.scan_ports("127.0.0.1", "80").await.unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].port, 80);
}

#[tokio::test]
#[ignore = "Requires network access"]
async fn test_syn_scan_internet() {
    // Tests requiring external network access
}
```

**Fuzzing tests** (`fuzz/` directory):
```rust
// fuzz/fuzz_targets/tcp_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use prtip_network::TcpPacket;

fuzz_target!(|data: &[u8]| {
    // Should never panic, even with malformed input
    let _ = TcpPacket::parse(data);
});
```

### Test-Driven Development (TDD)

**Recommended workflow**:

1. **Write failing test first**:
```rust
#[test]
fn test_ipv6_parsing() {
    let result = parse_target("2001:db8::1");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), IpAddr::V6(/* ... */));
}
```

2. **Run test to verify it fails**:
```bash
cargo test test_ipv6_parsing -- --exact
# Should fail: not yet implemented
```

3. **Implement minimal code to pass**:
```rust
fn parse_target(input: &str) -> Result<IpAddr, ParseError> {
    // Minimal implementation
    input.parse().map_err(|_| ParseError::Invalid)
}
```

4. **Run test to verify it passes**:
```bash
cargo test test_ipv6_parsing -- --exact
# Should pass now
```

5. **Refactor if needed**:
```rust
fn parse_target(input: &str) -> Result<IpAddr, ParseError> {
    // Improved implementation with better error messages
    input.parse()
        .map_err(|_| ParseError::Invalid(format!("Invalid IP: {}", input)))
}
```

6. **Run full test suite**:
```bash
cargo test
# Ensure no regressions
```

---

## Pull Request Process

### Before Opening PR

**Checklist**:
- [ ] All tests passing: `cargo test`
- [ ] Zero clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt`
- [ ] Documentation updated (if needed)
- [ ] CHANGELOG.md updated (if user-facing change)
- [ ] Examples compile: `cargo test --doc`
- [ ] Commit messages follow conventional commits format
- [ ] Branch rebased on latest main: `git pull --rebase upstream main`

**Self-review**:
- [ ] Read your own code changes critically
- [ ] Remove debug statements and commented code
- [ ] Verify logic handles edge cases
- [ ] Check for potential security issues
- [ ] Ensure error messages are helpful

### PR Description Template

```markdown
## Description

Brief summary of changes (1-2 sentences).

**Related Issue:** Closes #123

## Changes

- Change 1: Description of modification
- Change 2: Description of modification
- Change 3: Description of modification

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring

## Testing

### Test Coverage
- Added X unit tests
- Added Y integration tests
- Coverage: X% → Y% (use `cargo tarpaulin`)

### Test Execution
```bash
cargo test --workspace
# All tests passing: X passed, 0 failed
```

### Manual Testing
- Tested scenario 1: Result
- Tested scenario 2: Result
- Tested scenario 3: Result

## Documentation

- [ ] Code comments added/updated
- [ ] Rustdoc documentation updated
- [ ] User-facing documentation updated (docs/src/)
- [ ] CHANGELOG.md updated
- [ ] Examples added/updated

## Performance Impact

**Benchmarks**:
```bash
# Before
benchmark_name: 100ms ± 5ms

# After
benchmark_name: 95ms ± 4ms (5% improvement)
```

**Memory impact**: Negligible / +X MB / -Y MB

## Breaking Changes

**None** / **Yes** (describe below)

If breaking:
- What breaks: Description
- Migration path: How users should update
- Deprecation timeline: When old API removed

## Checklist

- [ ] All tests passing
- [ ] Zero clippy warnings
- [ ] Code formatted
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Self-reviewed code
- [ ] Rebased on latest main
```

### Review Process

**Timeline**:
- **Initial review**: 2-7 days (maintainers notified automatically)
- **Follow-up reviews**: 1-3 days (after addressing feedback)
- **Critical fixes**: 0-2 days (security issues, build failures)

**What reviewers check**:
1. **Code quality**: Readability, maintainability, idiomatic Rust
2. **Correctness**: Logic errors, edge cases, error handling
3. **Tests**: Adequate coverage, testing right things
4. **Documentation**: Clear, accurate, complete
5. **Performance**: No unnecessary allocations, efficient algorithms
6. **Security**: No vulnerabilities, proper input validation

**Addressing feedback**:
```bash
# Make requested changes
# Edit files as needed

# Commit changes with reference to review
git add .
git commit -m "refactor: improve error handling per review feedback"

# Push to update PR
git push origin feature/ipv6-udp-scanning
```

**When approved**:
- Maintainer will merge PR
- PR linked to issue will auto-close issue
- Changes included in next release

---

## Documentation Contributions

### Types of Documentation

**Code documentation** (inline Rustdoc):
```rust
/// Brief one-line summary.
///
/// Detailed explanation of functionality, behavior, and design decisions.
///
/// # Arguments
///
/// * `param1` - Description
/// * `param2` - Description
///
/// # Returns
///
/// Description of return value.
///
/// # Errors
///
/// Conditions that cause errors.
///
/// # Examples
///
/// ```rust
/// # use prtip_scanner::SynScanner;
/// let scanner = SynScanner::new(config)?;
/// ```
pub fn function(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // Implementation
}
```

**User documentation** (`docs/src/`):
- Follow [Documentation Standards](doc-standards.md)
- Use kebab-case file names
- Include "See Also" sections
- Test all code examples

**README updates**:
- Keep feature list current
- Update installation instructions if needed
- Maintain version compatibility matrix

**CHANGELOG updates**:
- Add entry under `[Unreleased]` section
- Use categories: Added, Changed, Deprecated, Removed, Fixed, Security
- Follow Keep a Changelog format

### Documentation Review Checklist

**Content**:
- [ ] Technical accuracy verified
- [ ] Code examples tested and working
- [ ] Links valid (internal and external)
- [ ] Spelling and grammar correct
- [ ] Consistent terminology used

**Structure**:
- [ ] Follows documentation standards
- [ ] Proper heading hierarchy (H1 → H2 → H3)
- [ ] Quick Reference section present
- [ ] "See Also" section complete
- [ ] Appropriate document length

**Style**:
- [ ] Active voice used
- [ ] Present tense used
- [ ] Clear, concise sentences
- [ ] Code formatting consistent

---

## Community Engagement

### Communication Channels

**GitHub Issues**:
- Bug reports
- Feature requests
- Questions about implementation
- Roadmap discussions

**GitHub Discussions**:
- General questions
- Showcase your use cases
- Community support
- Feature brainstorming

**Pull Requests**:
- Code contributions
- Documentation improvements
- Technical design discussions

**Security Issues**:
- **DO NOT** open public issues for vulnerabilities
- Email: <security@proRT-IP-project.org>
- See [Security Policy](https://github.com/doublegate/ProRT-IP/blob/main/SECURITY.md)

### Issue Reporting

**Bug report template**:
```markdown
**Describe the bug**
Clear description of what's wrong.

**To Reproduce**
Steps to reproduce:
1. Run command `prtip -sS -p 80 target.com`
2. Observe error: `Error: ...`

**Expected behavior**
What you expected to happen.

**Actual behavior**
What actually happened.

**Environment**
- OS: Ubuntu 22.04
- ProRT-IP version: 0.5.2
- Rust version: 1.85.0
- Installed via: cargo install / binary download / built from source

**Debug output**
```bash
RUST_LOG=debug prtip -sS -p 80 target.com

# Output:
```

**Additional context**
Any other relevant information.
```

**Feature request template**:
```markdown
**Feature description**
Clear description of proposed feature.

**Use case**
Real-world scenario where feature is needed.

**Proposed implementation**
How you think it could work (optional).

**Alternatives considered**
Other ways to achieve the same goal.

**Additional context**
Any other relevant information.
```

### Helping Others

**Ways to contribute without code**:
1. **Answer questions** on GitHub Discussions
2. **Improve documentation** (fix typos, clarify explanations)
3. **Triage issues** (reproduce bugs, add labels)
4. **Write tutorials** (blog posts, videos, examples)
5. **Test pre-releases** (beta testing, feedback)
6. **Translate documentation** (internationalization)

---

## Recognition and Credit

### Contributor Attribution

**All contributors** are recognized in:
- **AUTHORS.md**: Alphabetical list of contributors
- **Git history**: Commit authorship preserved
- **Release notes**: Major contributions highlighted
- **Documentation**: "See Also" credits for significant docs contributions

**How to add yourself to AUTHORS.md**:
```markdown
# Add entry in alphabetical order:

## Contributors

- Jane Doe (@janedoe) - IPv6 scanning implementation
- John Smith (@johnsmith) - Performance optimizations
```

### Contributor Levels

**Recognition levels**:
1. **Contributor**: 1+ merged PR
2. **Regular Contributor**: 5+ merged PRs or significant single contribution
3. **Core Contributor**: 20+ merged PRs, consistent engagement
4. **Maintainer**: Project leadership, code ownership, PR review

**Benefits by level**:
- **Contributor**: Listed in AUTHORS.md, commit in history
- **Regular**: Prioritized PR reviews, input on roadmap
- **Core**: Write access to repository, release responsibilities
- **Maintainer**: Full admin access, strategic decisions

---

## License and Copyright

### Contributor License Agreement

**By contributing**, you agree that:
1. Your contributions are your original work
2. You grant ProRT-IP project perpetual, worldwide, non-exclusive license to your contribution
3. Your contribution is licensed under GPL-3.0 (same as project)
4. You have the right to submit your contribution

**Copyright notice**:
```rust
// Copyright 2025 ProRT-IP Contributors
//
// This file is part of ProRT-IP.
//
// ProRT-IP is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
```

### Third-Party Code

**When adding dependencies**:
1. Verify license compatibility (GPL-compatible required)
2. Update `Cargo.toml` with dependency
3. Run `cargo deny check licenses` to verify
4. Document dependency purpose in PR description

**Compatible licenses**:
- MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause
- MPL-2.0, ISC, CC0-1.0

**Incompatible licenses**:
- Proprietary, closed-source licenses
- Non-commercial restrictions
- Unclear or undocumented licenses

---

## Development Resources

### Learning Resources

**Rust programming**:
- [The Rust Programming Language](https://doc.rust-lang.org/book/) - Official book
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - Hands-on examples
- [Rustlings](https://github.com/rust-lang/rustlings) - Interactive exercises

**Network programming**:
- [RFC 793: TCP](https://www.rfc-editor.org/rfc/rfc793) - TCP specification
- [RFC 791: IP](https://www.rfc-editor.org/rfc/rfc791) - IPv4 specification
- [RFC 8200: IPv6](https://www.rfc-editor.org/rfc/rfc8200) - IPv6 specification
- [TCP/IP Illustrated](https://www.amazon.com/TCP-Illustrated-Vol-Addison-Wesley-Professional/dp/0201633469) - Classic reference

**Security scanning**:
- [Nmap Network Scanning](https://nmap.org/book/) - Comprehensive scanning guide
- [Network Security Assessment](https://www.amazon.com/Network-Security-Assessment-Know-Your/dp/149191095X) - Assessment techniques

### Development Tools

**Recommended IDE setup**:
- [VS Code](https://code.visualstudio.com/) with rust-analyzer extension
- [RustRover](https://www.jetbrains.com/rust/) - JetBrains IDE for Rust
- [Helix](https://helix-editor.com/) - Terminal-based editor

**Debugging tools**:
```bash
# Rust debugger
rust-gdb target/debug/prtip

# Packet inspection
tcpdump -i eth0 -w capture.pcap
wireshark capture.pcap

# Performance profiling
cargo install cargo-flamegraph
cargo flamegraph --bin prtip
```

**Code analysis**:
```bash
# Security audit
cargo audit

# License checking
cargo install cargo-deny
cargo deny check licenses

# Dependency tree
cargo tree
```

---

## Release Contributions

### Feature Freeze

**Before each release**:
1. **Feature freeze** announced (7-14 days before release)
2. **No new features** merged during freeze
3. **Bug fixes only** accepted
4. **Documentation updates** continue
5. **Release candidate** testing period

### Release Testing

**Volunteer testing**:
1. Test pre-release binaries on your platform
2. Report any regressions or issues
3. Verify documentation accuracy
4. Provide feedback on release notes

**Platform testing priorities**:
- Linux x86_64 (primary)
- macOS Intel + ARM64
- Windows x86_64
- FreeBSD x86_64

---

## Long-Term Commitment

### Sustainability

**Project goals**:
- Maintain active development through v1.0 (Q4 2026)
- Continue bug fixes and security updates post-v1.0
- Foster community of contributors
- Document design decisions for future maintainers

### Succession Planning

**Knowledge transfer**:
- Architecture documentation complete
- Implementation details documented
- Design decisions recorded in ADRs (Architecture Decision Records)
- Mentorship program for new contributors

---

## See Also

- [Architecture](architecture.md) - System design and component structure
- [Implementation](implementation.md) - Code organization and design patterns
- [Testing](testing.md) - Testing philosophy and coverage goals
- [CI/CD](ci-cd.md) - Automated testing and deployment
- [Release Process](release-process.md) - Versioning and distribution
- [Documentation Standards](doc-standards.md) - Writing and organizing docs
- [Security Policy](https://github.com/doublegate/ProRT-IP/blob/main/SECURITY.md) - Vulnerability disclosure
