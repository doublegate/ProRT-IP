# Contributing to ProRT-IP WarScan

Thank you for your interest in contributing to ProRT-IP WarScan! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Security Guidelines](#security-guidelines)
- [Pull Request Process](#pull-request-process)
- [Commit Message Conventions](#commit-message-conventions)
- [Branch Naming Conventions](#branch-naming-conventions)
- [Issue Guidelines](#issue-guidelines)
- [Code Review Process](#code-review-process)
- [Release Process](#release-process)

## Code of Conduct

This project is dedicated to providing a welcoming and inclusive environment for all contributors. Please be respectful and professional in all interactions.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:

   ```bash
   git clone https://github.com/YOUR_USERNAME/ProRT-IP.git
   cd ProRT-IP
   ```

3. **Add upstream remote**:

   ```bash
   git remote add upstream https://github.com/doublegate/ProRT-IP.git
   ```

4. **Set up your development environment** (see [Development Setup](#development-setup))

## How to Contribute

There are several ways to contribute to ProRT-IP WarScan:

### Reporting Bugs

- **Search existing issues** first to avoid duplicates
- **Use the bug report template** when creating new issues
- **Include detailed information**:
  - Operating system and version
  - Rust version (`rustc --version`)
  - Steps to reproduce
  - Expected vs actual behavior
  - Error messages and stack traces
  - Sample command that triggers the bug

### Suggesting Features

- **Check the roadmap** first: [docs/01-ROADMAP.md](docs/01-ROADMAP.md)
- **Search existing feature requests** to avoid duplicates
- **Provide detailed description**:
  - Use case and problem it solves
  - Proposed solution
  - Alternative approaches considered
  - Impact on existing functionality

### Contributing Code

- **Start with good first issues** labeled `good-first-issue`
- **Discuss major changes** in an issue before starting work
- **Follow the development workflow** outlined below
- **Write tests** for all new functionality
- **Update documentation** as needed

### Improving Documentation

- Fix typos, clarify explanations, add examples
- Update outdated information
- Add missing documentation for features
- Improve README and guides

## Development Setup

Complete development setup instructions are available in **[docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md)**.

### Quick Start

1. **Install Rust** (1.85+ required, MSRV for edition 2024):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install system dependencies**:
   - **Linux**: `libpcap-dev`, `pkg-config`
   - **Windows**: Npcap 1.70+
   - **macOS**: ChmodBPF or run with sudo

3. **Build the project**:

   ```bash
   cargo build
   ```

4. **Run tests**:

   ```bash
   cargo test
   ```

5. **Set up pre-commit hooks** (recommended):

   ProRT-IP uses pre-commit hooks to validate code quality before commits.

   **Option 1: Installation Script (Recommended)**
   ```bash
   ./scripts/install-hooks.sh
   ```

   **Option 2: Manual Installation**
   ```bash
   cp .github/hooks/pre-commit .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

   **Option 3: Use core.hooksPath (Advanced)**
   ```bash
   git config core.hooksPath .github/hooks
   ```

   The pre-commit hook automatically validates:
   - ✅ Cargo.lock synchronized with Cargo.toml changes
   - ✅ Code formatting (`cargo fmt --check`)
   - ✅ Clippy linting (`cargo clippy --workspace -- -D warnings`)

   **Bypassing** (not recommended):
   ```bash
   git commit --no-verify  # Only use in emergencies
   ```

### Hook Files Explained

ProRT-IP has two pre-commit hook files:

1. **`.github/hooks/pre-commit`** (version controlled)
   - Source of truth, shared with all developers
   - Updated via pull requests
   - Tracked in Git repository

2. **`.git/hooks/pre-commit`** (local, not in Git)
   - Active hook that Git executes
   - Copied from template in `.github/hooks/`
   - Not version controlled (lives in `.git/` directory)

**Why two files?** Git only executes hooks from the `.git/hooks/` directory, which is never version controlled. To share hooks with the team, we store the template in `.github/hooks/` (version controlled) and developers copy it to `.git/hooks/` (local installation).

This is a **standard Git workflow pattern** used by projects like Homebrew, Kubernetes, and Linux Kernel.

### Updating Hooks

If the hook template in `.github/hooks/pre-commit` is updated (via pull request):

```bash
# Re-install to get latest version
./scripts/install-hooks.sh

# Or manually
cp .github/hooks/pre-commit .git/hooks/pre-commit
```

**Tip:** Run `git pull` before re-installing to ensure you have the latest template.

## Coding Standards

### Rust Style Guide

- **Use `rustfmt`** for consistent formatting:

  ```bash
  cargo fmt --check  # Check formatting
  cargo fmt          # Apply formatting
  ```

- **Use `clippy`** for linting:

  ```bash
  cargo clippy -- -D warnings
  ```

- **No warnings allowed** in submitted code - all `clippy` warnings must be resolved

### Code Quality Requirements

- **Clear, descriptive names** for functions, variables, types
- **Comprehensive documentation** for all public APIs:

  ```rust
  /// Performs a TCP SYN scan on the specified targets.
  ///
  /// # Arguments
  ///
  /// * `targets` - IP addresses or CIDR ranges to scan
  /// * `ports` - Port numbers to scan
  ///
  /// # Returns
  ///
  /// A `ScanResult` containing open ports and service information
  ///
  /// # Example
  ///
  /// ```
  /// let result = syn_scan(&["192.168.1.0/24"], &[80, 443])?;
  /// ```
  pub fn syn_scan(targets: &[IpAddr], ports: &[u16]) -> Result<ScanResult>
  ```

- **Error handling** using `Result<T, E>` - never `unwrap()` in production code
- **Input validation** at API boundaries
- **Resource cleanup** with RAII patterns and `Drop` implementations

### Performance Considerations

- **Profile before optimizing** - use `cargo bench` and `flamegraph`
- **Document performance-critical sections** with inline comments
- **Use appropriate data structures** (see [docs/07-PERFORMANCE.md](docs/07-PERFORMANCE.md))
- **Avoid premature optimization** - correctness first, speed second

### Security Best Practices

All contributions must follow security guidelines in **[docs/08-SECURITY.md](docs/08-SECURITY.md)**:

- **Input validation** for all user-provided data
- **Privilege dropping** immediately after resource creation
- **Safe packet parsing** - never `panic!` on malformed packets
- **Resource limits** to prevent DoS
- **No shell command construction** from user input

## Testing Requirements

Comprehensive testing guidelines are in **[docs/06-TESTING.md](docs/06-TESTING.md)**.

### Required Tests

All code contributions must include:

1. **Unit tests** for individual functions:

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_parse_port_range() {
           assert_eq!(parse_port_range("80-443").unwrap(), vec![80..=443]);
       }
   }
   ```

2. **Integration tests** for component interactions:

   ```rust
   // tests/scanner_integration.rs
   #[tokio::test]
   async fn test_tcp_connect_scan() {
       let scanner = Scanner::new(config)?;
       let result = scanner.execute().await?;
       assert!(!result.open_ports.is_empty());
   }
   ```

3. **Documentation tests** for public APIs (examples in doc comments)

### Test Coverage Requirements

- **>90% coverage** for core modules (`prtip-core`, `prtip-network`)
- **>60% coverage** for overall codebase (currently 54.92%)
- **Current Status:** 2,111 tests passing, 54.92% line coverage
- Run coverage with:

  ```bash
  cargo tarpaulin --out Html --output-dir coverage/
  # Or use project script:
  ./scripts/setup-dev-env.sh  # Installs cargo-tarpaulin + other tools
  ```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_syn_scan

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test '*'

# Benchmarks
cargo bench
```

## Security Guidelines

### Responsible Disclosure

- **Never publish security vulnerabilities** in public issues
- **Report security issues** privately to the maintainers (see [SECURITY.md](SECURITY.md))
- **Wait for coordinated disclosure** before discussing publicly

### Defensive Security Only

ProRT-IP WarScan is a **defensive security tool** for authorized penetration testing:

- **No offensive capabilities** - reject PRs that add malicious functionality
- **Emphasize authorized use** in documentation
- **Include legal disclaimers** where appropriate
- **Audit logging** for all scanning activities

## Continuous Integration

All pull requests must pass automated CI checks before merging:

### CI Pipeline Requirements

- **Format check**: `cargo fmt --check` - Code must be properly formatted
- **Linting**: `cargo clippy -- -D warnings` - All clippy warnings must be resolved
- **Tests on all platforms**: Linux, Windows, macOS - All tests must pass on each platform
- **Security audit**: `cargo audit` - No known vulnerabilities allowed
- **MSRV check**: Rust 1.70+ - Code must compile with minimum supported version

### CI Workflow Details

CI runs automatically on every push and PR. Check the [Actions tab](https://github.com/doublegate/ProRT-IP/actions) for status.

**Workflow jobs:**

- `format`: ~30 seconds - Checks code formatting
- `clippy`: ~2-3 minutes - Lint checks with caching
- `test`: ~3-5 minutes per platform (parallel) - Build and test (789 tests)
- `security_audit`: ~1-2 minutes - Vulnerability scanning
- `msrv`: ~2-3 minutes - Minimum version verification (Rust 1.85+)

**Total CI time:** ~5-10 minutes (with caching and parallel execution)
**Current Status:** 7/7 jobs passing (100%)

### Local Pre-Push Checks

Run these commands locally before pushing to save CI time:

```bash
# Check formatting
cargo fmt --all -- --check

# Run clippy (strict mode)
cargo clippy --workspace --all-targets -- -D warnings

# Run all tests
cargo test --workspace

# Security audit (optional but recommended)
cargo install cargo-audit
cargo audit
```

### CI Optimization

The CI pipeline uses aggressive caching for faster runs:

- **Cargo registry cache**: Downloaded crate metadata (~100-500 MB)
- **Cargo index cache**: Git index for crates.io (~50-200 MB)
- **Build cache**: Compiled dependencies (~500 MB - 2 GB)

**Cache benefits:**

- Clean build: 5-10 minutes → Cached build: 1-2 minutes (80-90% speedup)
- Cache hit rate: ~80-90% for typical changes

## Pull Request Process

### Before Submitting

1. **Update from upstream**:

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run full test suite**:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

3. **Update documentation** if needed:
   - Update `docs/` if functionality changes
   - Update `CHANGELOG.md` with your changes
   - Add/update examples in doc comments

4. **Ensure clean commit history** - squash WIP commits if needed

### PR Checklist

- [ ] Code follows style guidelines (`cargo fmt`, `cargo clippy`)
- [ ] All tests pass (`cargo test`)
- [ ] New tests added for new functionality
- [ ] Documentation updated (code comments, docs/, README)
- [ ] CHANGELOG.md updated with changes
- [ ] Security guidelines followed (docs/08-SECURITY.md)
- [ ] Commit messages follow conventions (see below)
- [ ] No new warnings introduced
- [ ] Performance impact considered and documented
- [ ] Cross-platform compatibility verified (if applicable)

### PR Description Template

```markdown
## Summary
Brief description of the changes

## Motivation
Why are these changes needed? What problem do they solve?

## Changes
- Bullet list of specific changes
- Include breaking changes if any

## Testing
How were these changes tested?

## Documentation
What documentation was updated?

## Checklist
- [ ] Tests pass
- [ ] Docs updated
- [ ] CHANGELOG.md updated
```

## Commit Message Conventions

We follow **Conventional Commits** format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring (no feature change or bug fix)
- `perf`: Performance improvement
- `test`: Adding or updating tests
- `chore`: Maintenance tasks (dependencies, build config)
- `ci`: CI/CD changes

### Scope

Optional, indicates area of change:

- `network`: Network/packet handling
- `scanner`: Scanning engine
- `detection`: Service/OS detection
- `cli`: Command-line interface
- `core`: Core library
- `docs`: Documentation

### Examples

```bash
feat(scanner): Add UDP scanning with protocol-specific probes

Implement UDP scanning functionality with specialized probes for
common services (DNS, SNMP, NetBIOS). Includes ICMP port unreachable
detection for closed port identification.

Fixes #42
```

```bash
fix(network): Correct TCP checksum calculation for IPv6

The checksum pseudo-header was using IPv4 format for IPv6 packets,
causing scan failures on IPv6-only networks.

Fixes #123
```

```bash
docs(readme): Update installation instructions for Windows

Add detailed Npcap installation steps and troubleshooting for
Windows users encountering packet capture issues.
```

## Branch Naming Conventions

Use descriptive branch names with type prefixes:

- `feature/<description>` - New features
- `fix/<description>` - Bug fixes
- `docs/<description>` - Documentation updates
- `refactor/<description>` - Code refactoring
- `test/<description>` - Test additions/updates

**Examples:**

- `feature/udp-scanning`
- `fix/tcp-checksum-ipv6`
- `docs/windows-setup-guide`
- `refactor/packet-parser`

## Issue Guidelines

### Bug Reports

Include:

- **Environment**: OS, Rust version, dependencies
- **Steps to reproduce**: Minimal reproducible example
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Error messages**: Full stack traces
- **Logs**: Relevant log output with `RUST_LOG=debug`

### Feature Requests

Include:

- **Problem statement**: What problem does this solve?
- **Proposed solution**: How would you implement it?
- **Alternatives**: Other approaches considered
- **Use cases**: Real-world scenarios
- **Breaking changes**: Impact on existing functionality

### Good First Issues

Look for issues tagged `good-first-issue`:

- Well-defined scope
- Clear acceptance criteria
- Mentorship available
- Good learning opportunities

## Code Review Process

### Review Timeline

- **Initial review**: Within 3-5 business days
- **Follow-up reviews**: Within 2 business days
- **Approval required**: At least one maintainer approval

### Review Criteria

Reviewers will check:

1. **Functionality**: Does it work as intended?
2. **Tests**: Adequate test coverage?
3. **Documentation**: Clear and complete?
4. **Code Quality**: Readable, maintainable, follows conventions?
5. **Security**: No vulnerabilities introduced?
6. **Performance**: No significant regressions?
7. **Compatibility**: Works across platforms?

### Addressing Feedback

- **Be responsive** to review comments
- **Ask questions** if feedback is unclear
- **Push new commits** for changes (don't force-push during review)
- **Mark conversations as resolved** when addressed
- **Request re-review** when ready

## Development Workflow

1. **Create feature branch**:

   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make changes** and commit:

   ```bash
   git add .
   git commit -m "feat(scope): Add feature description"
   ```

3. **Keep branch updated**:

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

4. **Push to your fork**:

   ```bash
   git push origin feature/my-feature
   ```

5. **Create pull request** on GitHub

6. **Address review feedback** and push updates

7. **Squash commits** if requested before merge

## Release Process

### Release Checklist

This section provides a comprehensive checklist for maintainers creating new releases. Following this process prevents common issues like Cargo.lock desynchronization that can cause CI failures.

#### Pre-Release Preparation

1. **Version Bump**
   - [ ] Update version in root `Cargo.toml`
   - [ ] Update version in all workspace member `Cargo.toml` files:
     - `prtip-cli/Cargo.toml`
     - `prtip-core/Cargo.toml`
     - `prtip-network/Cargo.toml`
     - `prtip-scanner/Cargo.toml`
     - `prtip-tui/Cargo.toml`
   - [ ] Run `cargo update` to regenerate `Cargo.lock`
   - [ ] Verify with `cargo build --locked` (must pass)
   - [ ] Stage all changes: `git add Cargo.toml */Cargo.toml Cargo.lock`

2. **Documentation Updates**
   - [ ] Update `README.md` (version badge, test count, new features)
   - [ ] Update `CHANGELOG.md` (add new version section)
   - [ ] Update `CLAUDE.md` (status line with new version)
   - [ ] Update `CLAUDE.local.md` (header with new version)
   - [ ] Update `docs/00-ARCHITECTURE.md` (version, test count)
   - [ ] Update `docs/01-ROADMAP.md` (version, progress percentage)
   - [ ] Update `docs/10-PROJECT-STATUS.md` (current version)
   - [ ] Update relevant technical docs (if architecture changed)

3. **Quality Gates**
   - [ ] Run `cargo fmt --check` (0 formatting issues)
   - [ ] Run `cargo clippy --workspace -- -D warnings` (0 warnings)
   - [ ] Run `cargo test --workspace` (all tests passing)
   - [ ] Run `cargo build --release` (clean build)
   - [ ] Run `cargo audit` (no security vulnerabilities)
   - [ ] Check coverage: `cargo tarpaulin --workspace` (>54% baseline)

4. **Release Notes**
   - [ ] Create `/tmp/ProRT-IP/RELEASE-NOTES-vX.Y.Z.md`
   - [ ] Include: Executive summary, features, fixes, performance, quality metrics
   - [ ] Length: 100-200 lines (follow v0.3.7-v0.5.5 standard)
   - [ ] Technical depth: Code examples, metrics, strategic value

5. **Git Workflow**
   - [ ] Stage all changes: `git add .`
   - [ ] Create commit with 200+ line message (conventional format)
   - [ ] Create annotated tag: `git tag -a vX.Y.Z -F /tmp/ProRT-IP/RELEASE-NOTES-vX.Y.Z.md`
   - [ ] Verify tag: `git show vX.Y.Z`
   - [ ] Push commit: `git push origin main`
   - [ ] Push tag: `git push origin vX.Y.Z`

6. **GitHub Release**
   - [ ] Create/update release: `gh release create vX.Y.Z --notes-file /tmp/ProRT-IP/RELEASE-NOTES-vX.Y.Z.md`
   - [ ] Verify release URL and content
   - [ ] Check all CI workflows pass (100% success rate)

#### Post-Release Verification

- [ ] All GitHub Actions workflows passing (green checks)
- [ ] GitHub Release published and visible
- [ ] Release notes comprehensive and accurate
- [ ] No broken links in documentation
- [ ] CI/CD coverage workflow uploading to Codecov
- [ ] All platforms validated (Linux, macOS, Windows)

#### Common Pitfalls

⚠️ **Don't forget to regenerate Cargo.lock** after version bumps
⚠️ **Don't skip `cargo build --locked`** verification
⚠️ **Don't commit without running pre-commit hook** checks
⚠️ **Don't create duplicate GitHub releases** (check existing first)
⚠️ **Don't push before local quality gates pass**

### Using cargo-release (Recommended)

ProRT-IP uses `cargo-release` to automate version bumps and releases, reducing manual errors and ensuring consistency.

#### Installation

```bash
cargo install cargo-release
```

#### Configuration

Release automation is configured in `release.toml` at the repository root. This configuration handles:
- Workspace-wide version bumps
- Cargo.lock regeneration
- Documentation version updates
- Pre-release quality checks

#### Usage

```bash
# Patch release (0.5.5 → 0.5.6)
./scripts/release.sh patch

# Minor release (0.5.6 → 0.6.0)
./scripts/release.sh minor

# Major release (0.6.0 → 1.0.0)
./scripts/release.sh major
```

The `release.sh` script automates:
- Pre-flight checks (fmt, clippy, test, lockfile validation)
- Version bumps in all Cargo.toml files
- Cargo.lock regeneration
- Documentation version updates (README, CLAUDE.md, docs/)
- Git commit and tag creation

#### Manual Steps (Still Required)

1. **Create release notes**: Write `/tmp/ProRT-IP/RELEASE-NOTES-vX.Y.Z.md`
2. **Push to GitHub**: `git push origin main && git push origin --tags`
3. **Create GitHub Release**: `gh release create vX.Y.Z --notes-file /tmp/ProRT-IP/RELEASE-NOTES-vX.Y.Z.md`

#### Dry-Run Testing

Before executing a release, test with dry-run mode:

```bash
cargo release patch --dry-run
```

This shows all changes that would be made without actually executing them.

### Quick Commands

```bash
# Version bump workflow (manual)
cargo update
cargo build --locked

# Quality checks (run before release)
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace

# Release workflow (manual)
git add . && git commit -m "release(vX.Y.Z): ..."
git tag -a vX.Y.Z -F /tmp/ProRT-IP/RELEASE-NOTES-vX.Y.Z.md
git push origin main && git push origin vX.Y.Z
gh release create vX.Y.Z --notes-file /tmp/ProRT-IP/RELEASE-NOTES-vX.Y.Z.md
```

## Additional Resources

- **Architecture**: [docs/00-ARCHITECTURE.md](docs/00-ARCHITECTURE.md)
- **Development Roadmap**: [docs/01-ROADMAP.md](docs/01-ROADMAP.md)
- **Technical Specifications**: [docs/02-TECHNICAL-SPECS.md](docs/02-TECHNICAL-SPECS.md)
- **Development Setup**: [docs/03-DEV-SETUP.md](docs/03-DEV-SETUP.md)
- **Implementation Guide**: [docs/04-IMPLEMENTATION-GUIDE.md](docs/04-IMPLEMENTATION-GUIDE.md)
- **API Reference**: [docs/05-API-REFERENCE.md](docs/05-API-REFERENCE.md)
- **Testing Strategy**: [docs/06-TESTING.md](docs/06-TESTING.md)
- **Performance Optimization**: [docs/07-PERFORMANCE.md](docs/07-PERFORMANCE.md)
- **Security Implementation**: [docs/08-SECURITY.md](docs/08-SECURITY.md)
- **FAQ**: [docs/09-FAQ.md](docs/09-FAQ.md)
- **Project Status**: [docs/10-PROJECT-STATUS.md](docs/10-PROJECT-STATUS.md)

## Questions?

- **Documentation**: See [docs/README.md](docs/README.md) for navigation
- **FAQ**: Check [docs/09-FAQ.md](docs/09-FAQ.md) for common questions
- **Support**: See [SUPPORT.md](SUPPORT.md) for help resources
- **Security**: See [SECURITY.md](SECURITY.md) for vulnerability reporting

## License

By contributing to ProRT-IP WarScan, you agree that your contributions will be licensed under the GNU General Public License v3.0 (GPLv3).

---

**Thank you for contributing to ProRT-IP WarScan!**
