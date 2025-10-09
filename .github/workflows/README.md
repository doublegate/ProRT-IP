# GitHub Actions Workflows

This directory contains CI/CD workflows for the ProRT-IP WarScan project.

## Workflows

### ci.yml - Continuous Integration
**Triggers:** Push to main, Pull Requests

**Jobs:**
- **format**: Check code formatting with `cargo fmt --check`
- **clippy**: Lint code with `cargo clippy` (strict: `-D warnings`)
- **test**: Build and test on Linux, Windows, macOS
- **security_audit**: Run `cargo audit` for vulnerabilities
- **msrv**: Verify Minimum Supported Rust Version (1.70)

**Optimizations:**
- Cargo registry/index/build caching for faster runs (50-80% speedup)
- Parallel job execution for quick feedback
- Fail-fast disabled for platform matrix to see all results
- Cache keys based on Cargo.lock hash for reliability

**Estimated Run Time:**
- Format check: ~30 seconds
- Clippy: ~2-3 minutes (with cache)
- Tests per platform: ~3-5 minutes (with cache)
- Security audit: ~1-2 minutes
- MSRV check: ~2-3 minutes (with cache)
- **Total**: ~5-10 minutes (parallel execution)

### release.yml - Release Automation
**Triggers:** Git tags matching `v*.*.*`

**Jobs:**
- **create-release**: Create GitHub release with changelog
- **build-release**: Build binaries for:
  - `x86_64-unknown-linux-gnu` (glibc)
  - `x86_64-unknown-linux-musl` (static, no libc dependency)
  - `x86_64-pc-windows-msvc` (Windows)
  - `x86_64-apple-darwin` (macOS)

**Features:**
- Multi-platform binary builds with cross-compilation
- Automatic asset upload to release (tar.gz, zip)
- Comprehensive release notes with:
  - Major features and capabilities
  - Installation instructions
  - Usage examples
  - Documentation links
  - Security warnings
- Version extraction from git tag

**Estimated Run Time:**
- Release creation: ~30 seconds
- Binary builds (parallel): ~5-10 minutes per platform
- **Total**: ~10-15 minutes (parallel builds)

### dependency-review.yml - Dependency Security
**Triggers:** Pull Requests

**Purpose:**
- Scan for vulnerable dependencies
- Detect malicious packages
- Review license compliance
- Block PRs with security issues

**Automated Actions:**
- Fails PR if high/critical vulnerabilities found
- Comments on PR with security findings
- Links to CVE details and remediation

### codeql.yml - Security Analysis
**Triggers:** Push, PR, Weekly schedule (Monday 00:00 UTC)

**Purpose:**
- Advanced security scanning with CodeQL
- Detect common vulnerabilities:
  - Buffer overflows
  - SQL injection (if applicable)
  - Path traversal
  - Command injection
  - Memory safety issues
- Upload SARIF results to GitHub Security tab

**Languages Analyzed:**
- C/C++ (Rust compiles to native code)

**Scheduled Scans:**
- Weekly on Monday to catch new vulnerabilities
- Runs on every push/PR for immediate feedback

## Local Testing

Test formatting/linting before pushing:

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

## Caching Strategy

### Three-Tier Caching

1. **Cargo Registry Cache** (`~/.cargo/registry`)
   - Stores downloaded crate metadata
   - Key: OS + Cargo.lock hash
   - Typical size: 100-500 MB
   - Hit rate: ~90% (changes only on dependency updates)

2. **Cargo Index Cache** (`~/.cargo/git`)
   - Stores git index for crates.io
   - Key: OS + Cargo.lock hash
   - Typical size: 50-200 MB
   - Hit rate: ~90%

3. **Build Cache** (`target/`)
   - Stores compiled dependencies and build artifacts
   - Key: OS + Rust version + Cargo.lock hash
   - Typical size: 500 MB - 2 GB
   - Hit rate: ~80% (changes on dependency or code updates)

### Cache Performance

**Without Cache:**
- Clean build: 5-10 minutes
- Test run: 3-5 minutes
- Total CI time: 15-25 minutes

**With Cache (typical):**
- Incremental build: 1-2 minutes
- Test run: 1-2 minutes
- Total CI time: 5-10 minutes

**Cache Hit Scenarios:**
- No code changes: 80-90% speedup
- Code changes only: 50-70% speedup
- Dependency changes: 20-40% speedup

## Workflow Status

Check workflow runs: [Actions Tab](https://github.com/doublegate/ProRT-IP/actions)

### Status Badges

Add to README.md:

```markdown
[![CI](https://github.com/doublegate/ProRT-IP/actions/workflows/ci.yml/badge.svg)](https://github.com/doublegate/ProRT-IP/actions/workflows/ci.yml)
[![Release](https://github.com/doublegate/ProRT-IP/actions/workflows/release.yml/badge.svg)](https://github.com/doublegate/ProRT-IP/actions/workflows/release.yml)
```

## Troubleshooting

### Cache Issues

If builds are slow or failing due to cache:

```bash
# Manually clear cache (GitHub Actions UI)
# Settings → Actions → Caches → Delete all caches

# Or force cache miss with updated key
# Increment version in workflow: -v2, -v3, etc.
```

### Platform-Specific Failures

**Linux:**
- Check musl-tools installation for static builds
- Verify libssl-dev for OpenSSL dependencies

**Windows:**
- Ensure MSVC toolchain is available
- Check 7z availability for zip creation

**macOS:**
- Verify Xcode Command Line Tools
- Check for Apple Silicon vs Intel compatibility

### Security Audit Failures

If `cargo audit` fails:

```bash
# Check advisory locally
cargo audit

# Review advisory details
# Update dependency or request CVE exception

# Temporary: Allow specific advisory (not recommended)
# cargo audit --ignore RUSTSEC-XXXX-XXXX
```

## Adding New Workflows

To add a new workflow:

1. Create `.github/workflows/name.yml`
2. Define triggers and jobs
3. Add caching for performance
4. Document in this README
5. Test locally if possible (with `act`)
6. Create PR and verify in CI

## Maintenance

**Monthly:**
- Review cache usage and clean old caches
- Update action versions (@v4 → @v5)
- Check for deprecated actions

**Quarterly:**
- Review and update MSRV
- Benchmark CI performance
- Optimize slow jobs

**Yearly:**
- Audit all workflows for security
- Review caching strategy effectiveness
- Update documentation with lessons learned
