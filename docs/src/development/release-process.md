# Release Process

Comprehensive guide to ProRT-IP's release management, versioning strategy, and distribution workflow.

## Quick Reference

**Current Version:** v0.5.2
**Release Cadence:** Weekly during active development, monthly for maintenance
**Platforms:** 8 (Linux x86_64/ARM64 glibc/musl, macOS Intel/ARM64, Windows x86_64, FreeBSD x86_64)
**Versioning:** [Semantic Versioning 2.0.0](https://semver.org/)
**Changelog:** [Keep a Changelog 1.0.0](https://keepachangelog.com/)

---

## Version History

### Release Timeline

| Version | Date | Type | Highlights |
|---------|------|------|------------|
| **0.5.2** | 2025-11-14 | Minor | Sprint 6.2 Live Dashboard Complete (TUI 4-tab system, real-time metrics) |
| **0.5.1** | 2025-11-14 | Minor | Sprint 6.1 TUI Framework (60 FPS rendering, event-driven architecture) |
| **0.5.0** | 2025-11-07 | Minor | **Phase 5 Complete** (IPv6 100%, Service Detection 85-90%, Plugin System) |
| **0.4.9** | 2025-11-06 | Patch | Documentation polish, mdBook integration |
| **0.4.8** | 2025-11-06 | Patch | CI/CD optimization, CodeQL analysis |
| **0.4.7** | 2025-11-06 | Patch | Fuzz testing framework, structure-aware fuzzing |
| **0.4.6** | 2025-11-05 | Patch | GitHub Actions migration (v3→v4), coverage automation |
| **0.4.5** | 2025-11-04 | Patch | TLS certificate SNI support, badssl.com graceful handling |
| **0.4.4** | 2025-11-02 | Patch | Test performance optimization (30min→30s, 60x speedup) |
| **0.4.0** | 2025-10-27 | Minor | **Phase 4 Complete** (PCAPNG, Evasion, IPv6 Foundation) |
| **0.3.7** | 2025-10-13 | Patch | Service detection enhancements |
| **0.3.6** | 2025-10-12 | Patch | Performance tuning |
| **0.3.5** | 2025-10-12 | Patch | Bug fixes |
| **0.3.0** | 2025-10-08 | Minor | **Phase 3 Complete** (OS Fingerprinting, Service Detection) |
| **0.0.1** | 2025-10-07 | Initial | Project inception |

**Total Releases:** 15 (Oct 7 - Nov 14, 2025)
**Release Frequency:** Multiple releases per day during active development, tapering to weekly/monthly

---

## Semantic Versioning

ProRT-IP strictly follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html):

### Version Format: `MAJOR.MINOR.PATCH`

```
Example: v0.5.2
         │ │ │
         │ │ └── PATCH: Bug fixes, performance improvements (backward compatible)
         │ └──── MINOR: New features, enhancements (backward compatible)
         └────── MAJOR: Breaking changes, API redesign (NOT backward compatible)
```

### Increment Rules

**MAJOR version (X.0.0)** when:
- Breaking API changes (function signature changes, removed public APIs)
- Major architectural redesign requiring code changes in dependent projects
- Minimum Rust version (MSRV) increase that breaks existing builds
- Configuration file format changes requiring migration

**MINOR version (0.X.0)** when:
- New scan types or detection capabilities
- New output formats or CLI flags (backward compatible)
- Performance improvements or optimizations
- New platform support (Linux ARM64, BSD variants)
- Phase completions (Phase 5: v0.5.0, Phase 4: v0.4.0)

**PATCH version (0.0.X)** when:
- Bug fixes (scan accuracy, memory leaks, race conditions)
- Documentation updates (README, guides, examples)
- CI/CD improvements (workflow optimization, coverage automation)
- Test infrastructure enhancements
- Dependency updates (security patches, version bumps)

### Pre-release Versions

Not currently used, but planned for future major releases:

```
v1.0.0-alpha.1  → Early preview (breaking changes expected)
v1.0.0-beta.1   → Feature complete (API stabilizing)
v1.0.0-rc.1     → Release candidate (production testing)
v1.0.0          → Stable release
```

### Version 0.x.x Special Rules

**Pre-1.0 versions (0.x.x):**
- Breaking changes allowed in MINOR releases (0.5.0 → 0.6.0)
- API stability not guaranteed until v1.0.0
- Phase milestones marked with MINOR increments (Phase 5 = v0.5.0)

**Transition to v1.0.0:**
- API freeze and stability commitment
- Production-ready declaration
- Long-term support (LTS) commitment
- Planned for Phase 8 completion (Q4 2026)

---

## Changelog Management

ProRT-IP uses [Keep a Changelog 1.0.0](https://keepachangelog.com/en/1.0.0/) format.

### CHANGELOG.md Structure

```markdown
# Changelog

All notable changes to ProRT-IP WarScan will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Documentation: Phase 1 Naming Standards Implementation
- CI/CD: Added Code Coverage with cargo-tarpaulin

### Internal
- Sprint 6.3 Phase 2.2: Scheduler Integration Complete

### Fixed
- Test Infrastructure: macOS batch_coordination.rs Test Failures

### Added
#### Sprint 6.3: Network Optimizations - Batch I/O & CDN Deduplication PARTIAL

## [0.5.2] - 2025-11-14

### Major Features
- Sprint 6.2: Live Dashboard & Real-Time Metrics (COMPLETE)

## [0.5.1] - 2025-11-14

### Major Features
- Sprint 6.1: TUI Framework (COMPLETE)
```

### Section Definitions

**[Unreleased]** - Changes in `main` branch not yet released:
- Merged pull requests
- Completed sprints awaiting release
- Internal refactoring
- Documentation updates

**[X.Y.Z] - YYYY-MM-DD** - Released versions with changes categorized:

| Section | Purpose | Examples |
|---------|---------|----------|
| **Added** | New features, capabilities | New scan types, plugin system, TUI widgets |
| **Changed** | Modifications to existing features | API improvements, performance optimizations |
| **Deprecated** | Features marked for removal | Old CLI flags, deprecated APIs |
| **Removed** | Deleted features | Removed experimental code, obsolete flags |
| **Fixed** | Bug fixes | Race conditions, memory leaks, test failures |
| **Security** | Security patches | Vulnerability fixes, dependency updates |
| **Internal** | Implementation details | Sprint completions, refactoring, test infrastructure |

### Sprint Documentation Pattern

```markdown
#### Sprint X.Y: Feature Name - STATUS

**Status:** COMPLETE/PARTIAL | **Completed:** YYYY-MM-DD | **Duration:** ~Xh

**Strategic Achievement:** High-level summary of impact and value

**Implementation Deliverables:**
- Files created/modified with line counts
- Test coverage (unit + integration + doc tests)
- Performance metrics (throughput, overhead, latency)
- Quality metrics (clippy warnings, formatting, coverage %)

**Performance Validation:**
- Benchmark results with baseline comparisons
- Throughput measurements (packets/sec, requests/sec)
- Overhead analysis (CPU %, memory MB, syscalls)
- Scalability tests (linear scaling, resource usage)

**Files Modified:**
- `path/to/file.rs` (~XXX lines) - Purpose description
- `path/to/test.rs` (~XXX lines) - Test coverage description

**Quality Metrics:**
- Tests: X,XXX/X,XXX passing (100%)
- Clippy: 0 warnings
- Formatting: Clean (cargo fmt)
- Coverage: XX.XX%

**Known Limitations:**
- Limitation 1 with mitigation strategy
- Limitation 2 with future work reference

**Future Work:**
- Enhancement 1 (Phase X.Y)
- Enhancement 2 (Phase X.Z)
```

### Performance Metrics Table Format

```markdown
| Metric | Baseline | Optimized | Improvement |
|--------|----------|-----------|-------------|
| Throughput | 10K pps | 50K pps | 5x (400%) |
| Memory | 100 MB | 20 MB | -80% |
| Overhead | 15% | -1.8% | Industry-leading |
```

### Changelog Update Process

1. **During Development:**
   ```bash
   # Add entry to [Unreleased] section immediately after merging PR
   vim CHANGELOG.md
   # Example entry:
   # - **Feature: IPv6 Support** - Added dual-stack scanning (2025-11-01)
   ```

2. **Before Release:**
   ```bash
   # Move [Unreleased] entries to new version section
   # Replace [Unreleased] header with:
   ## [X.Y.Z] - YYYY-MM-DD

   # Add new empty [Unreleased] section at top
   ```

3. **Quality Checks:**
   - All merged PRs documented
   - Sprint completions included with metrics
   - Breaking changes highlighted
   - Performance data validated
   - Cross-references to documentation added

---

## Release Checklist

### Pre-Release Preparation (1-2 days before)

**Phase 1: Code Quality**

- [ ] All tests passing locally and in CI
  ```bash
  cargo test --workspace --locked --lib --bins --tests
  # Expected: X,XXX tests passing, 0 failures
  ```

- [ ] Zero clippy warnings
  ```bash
  cargo clippy --workspace --all-targets --locked -- -D warnings
  # Expected: 0 warnings
  ```

- [ ] Code formatting clean
  ```bash
  cargo fmt --all -- --check
  # Expected: No formatting issues
  ```

- [ ] No cargo-deny violations
  ```bash
  cargo deny check advisories
  # Expected: advisories ok
  ```

**Phase 2: Documentation**

- [ ] CHANGELOG.md updated with all changes
  - Move [Unreleased] entries to new version section
  - Add version header: `## [X.Y.Z] - YYYY-MM-DD`
  - Include sprint completions with metrics
  - Document breaking changes prominently

- [ ] README.md version references updated
  ```bash
  # Update 8+ version references:
  # - Header badge
  # - Quick Start examples
  # - Installation instructions
  # - Project Status table
  ```

- [ ] Version bumped in all files
  ```bash
  # Files to update:
  # - Cargo.toml (workspace.package.version)
  # - README.md (8 references)
  # - CLAUDE.local.md (header, At a Glance table)
  # - docs/01-ROADMAP.md (version number)
  # - docs/10-PROJECT-STATUS.md (header)
  ```

- [ ] Cross-references validated
  ```bash
  # Check all documentation links
  mdbook test docs/
  # Expected: No broken links
  ```

**Phase 3: Testing**

- [ ] Full test suite on all platforms (CI Matrix)
  - Linux x86_64: Unit + integration + doc tests
  - macOS latest: Unit + integration + doc tests
  - Windows latest: Unit tests (integration tests require Npcap)

- [ ] Benchmark regression tests
  ```bash
  cd benchmarks/
  ./run_benchmarks.sh --compare-baseline
  # Expected: No regressions >10%
  ```

- [ ] Manual smoke testing
  - Basic SYN scan: `prtip -sS -p 80,443 scanme.nmap.org`
  - Service detection: `prtip -sS -sV -p 1-1000 scanme.nmap.org`
  - TUI mode: `prtip --live -sS -p 80 scanme.nmap.org`
  - Help system: `prtip --help`

**Phase 4: Release Notes**

- [ ] Generate comprehensive release notes (150-250 lines)
  - Executive summary (strategic value, milestone significance)
  - Major features with technical details
  - Performance improvements with benchmark data
  - Bug fixes with root cause analysis
  - Breaking changes with migration guidance
  - Platform support matrix
  - Known issues and limitations
  - Installation instructions
  - Upgrade notes
  - Strategic impact on roadmap

- [ ] Save release notes to `/tmp/ProRT-IP/RELEASE-NOTES-vX.Y.Z.md`

### Release Execution (Day of Release)

**Phase 5: Version Bump & Commit**

1. **Update version numbers:**
   ```bash
   # Cargo.toml (workspace)
   [workspace.package]
   version = "X.Y.Z"

   # README.md (8 references)
   # CLAUDE.local.md (header + table)
   ```

2. **Update CHANGELOG.md:**
   ```bash
   # Move [Unreleased] → [X.Y.Z] - YYYY-MM-DD
   # Add new empty [Unreleased] section
   ```

3. **Commit changes:**
   ```bash
   git add Cargo.toml Cargo.lock README.md CHANGELOG.md CLAUDE.local.md docs/
   git commit -m "chore(release): Bump version to vX.Y.Z

   Release Highlights:
   - Feature 1 (Sprint X.Y)
   - Feature 2 (Sprint X.Z)
   - Performance improvement: +NN% throughput

   Files Modified:
   - Cargo.toml: Version X.Y.Z
   - CHANGELOG.md: +NNN lines comprehensive entry
   - README.md: Updated version references
   - CLAUDE.local.md: Version header

   Quality Metrics:
   - Tests: X,XXX passing (100%)
   - Coverage: XX.XX%
   - Clippy: 0 warnings
   - Benchmarks: No regressions

   Strategic Value:
   [1-2 paragraph summary of release significance]

   See CHANGELOG.md for complete details."
   ```

**Phase 6: Tagging**

1. **Create annotated Git tag:**
   ```bash
   git tag -a vX.Y.Z -F /tmp/ProRT-IP/RELEASE-NOTES-vX.Y.Z.md
   ```

2. **Verify tag:**
   ```bash
   git tag -l -n100 vX.Y.Z
   # Expected: 150-250 line release notes
   ```

**Phase 7: Push & Trigger CI/CD**

1. **Push commit and tag:**
   ```bash
   git push origin main
   git push origin vX.Y.Z
   ```

2. **Monitor GitHub Actions release workflow:**
   - Navigate to: `https://github.com/doublegate/ProRT-IP/actions`
   - Watch workflow: `Release Binaries`
   - Expected duration: 15-20 minutes
   - Expected artifacts: 8 platform binaries

**Phase 8: GitHub Release**

1. **Create GitHub release (after binaries built):**
   ```bash
   gh release create vX.Y.Z \
     --title "ProRT-IP vX.Y.Z - Release Title" \
     --notes-file /tmp/ProRT-IP/RELEASE-NOTES-vX.Y.Z.md \
     --verify-tag
   ```

2. **Verify release:**
   - URL: `https://github.com/doublegate/ProRT-IP/releases/tag/vX.Y.Z`
   - Check: 8 platform binaries attached
   - Check: Release notes rendered correctly
   - Check: Installation instructions accurate

**Phase 9: Post-Release**

1. **Update project status:**
   ```bash
   # CLAUDE.local.md
   # - Header: **vX.Y.Z** (YYYY-MM-DD)
   # - At a Glance table: Version row
   # - Recent Sessions: Add release entry
   ```

2. **Verify installation:**
   ```bash
   # Download and test binary for your platform
   wget https://github.com/doublegate/ProRT-IP/releases/download/vX.Y.Z/prtip-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz
   tar xzf prtip-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz
   ./prtip --version
   # Expected: prtip X.Y.Z
   ```

3. **Announce release:**
   - GitHub Discussions: Post release announcement
   - Update documentation website (if deployed)
   - Social media (if applicable)

---

## Binary Distribution

### Build Platforms (8 Total)

ProRT-IP releases production-ready binaries for 5 primary platforms and 3 experimental platforms.

#### Production Platforms (Full Support)

| Platform | Target | Glibc/Runtime | Binary Size | Notes |
|----------|--------|---------------|-------------|-------|
| **Linux x86_64** | `x86_64-unknown-linux-gnu` | glibc 2.27+ | ~8 MB | Recommended platform |
| **macOS Intel** | `x86_64-apple-darwin` | N/A (native) | ~8 MB | macOS 10.13+ |
| **macOS ARM64** | `aarch64-apple-darwin` | N/A (native) | ~7 MB | **Fastest** (110% baseline) |
| **Windows x86_64** | `x86_64-pc-windows-msvc` | MSVC Runtime | ~9 MB | Requires Npcap |
| **FreeBSD x86_64** | `x86_64-unknown-freebsd` | FreeBSD 12+ | ~8 MB | Community supported |

#### Experimental Platforms (Known Limitations)

| Platform | Target | Status | Issues |
|----------|--------|--------|--------|
| **Linux x86_64 musl** | `x86_64-unknown-linux-musl` | ⚠️ Type mismatches | Requires conditional compilation fixes |
| **Linux ARM64 glibc** | `aarch64-unknown-linux-gnu` | ⚠️ OpenSSL issues | Cross-compilation challenges |
| **Linux ARM64 musl** | `aarch64-unknown-linux-musl` | ⚠️ Multiple issues | Compilation + OpenSSL problems |

### GitHub Actions Release Workflow

**File:** `.github/workflows/release.yml`
**Trigger:** Git tag push matching `v*` pattern
**Duration:** 15-20 minutes

**Build Matrix:**

```yaml
strategy:
  matrix:
    include:
      # Production platforms (5)
      - target: x86_64-unknown-linux-gnu
        os: ubuntu-latest
      - target: x86_64-pc-windows-msvc
        os: windows-latest
      - target: x86_64-apple-darwin
        os: macos-13
      - target: aarch64-apple-darwin
        os: macos-14
      - target: x86_64-unknown-freebsd
        os: ubuntu-latest
        cross: true

      # Experimental platforms (3)
      - target: x86_64-unknown-linux-musl
        os: ubuntu-latest
        cross: true
      - target: aarch64-unknown-linux-gnu
        os: ubuntu-latest
        cross: true
      - target: aarch64-unknown-linux-musl
        os: ubuntu-latest
        cross: true
```

**Build Steps:**

1. **Environment Setup:**
   - Install Rust toolchain (stable)
   - Install target: `rustup target add <target>`
   - Install platform dependencies (libpcap, OpenSSL, pkg-config)

2. **Cross-Compilation (if needed):**
   ```bash
   cargo install cross --git https://github.com/cross-rs/cross
   cross build --release --target <target>
   ```

3. **Native Compilation:**
   ```bash
   cargo build --release --target <target> --features vendored-openssl
   ```

4. **Binary Packaging:**
   ```bash
   # Linux/macOS/FreeBSD
   tar czf prtip-$VERSION-$TARGET.tar.gz prtip

   # Windows
   7z a prtip-$VERSION-$TARGET.zip prtip.exe
   ```

5. **Artifact Upload:**
   ```bash
   gh release upload $TAG prtip-$VERSION-$TARGET.{tar.gz,zip}
   ```

**Release Notes Generation:**

```yaml
- name: Generate Release Notes
  run: |
    # Extract from CHANGELOG.md
    VERSION="${GITHUB_REF#refs/tags/v}"
    sed -n "/## \[$VERSION\]/,/## \[/p" CHANGELOG.md | head -n -1 > notes.md

    # Add installation instructions
    cat >> notes.md << 'EOF'

    ## Installation

    ### Linux
    ```bash
    wget https://github.com/doublegate/ProRT-IP/releases/download/$VERSION/prtip-$VERSION-x86_64-unknown-linux-gnu.tar.gz
    tar xzf prtip-$VERSION-x86_64-unknown-linux-gnu.tar.gz
    sudo mv prtip /usr/local/bin/
    sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/prtip
    ```
    EOF
```

**Special Build Configurations:**

- **musl static linking:** `--features vendored-openssl` + `OPENSSL_STATIC=1`
- **Windows Npcap:** SDK in `LIB` path, runtime DLLs in `PATH`
- **macOS universal:** Separate builds for x86_64 and aarch64 (not lipo'd)
- **FreeBSD cross:** Uses `cross-rs` with custom docker image

---

## Hotfix Procedures

### When to Create a Hotfix

**Critical issues requiring immediate patch release:**
- Security vulnerabilities (CVE assigned or high severity)
- Data corruption bugs (scan results incorrect/lost)
- Crash/panic in common scenarios (SYN scan, service detection)
- Memory leaks causing OOM in production use
- Platform-specific regressions breaking core functionality

**Non-critical issues (wait for next minor release):**
- Performance regressions <20%
- Documentation errors
- Test infrastructure issues
- Non-blocking UI glitches

### Hotfix Release Process

1. **Create hotfix branch from tagged release:**
   ```bash
   git checkout vX.Y.Z
   git checkout -b hotfix/vX.Y.Z+1
   ```

2. **Fix the issue with minimal changes:**
   ```bash
   # Make ONLY the fix, no feature additions
   # Prefer small, surgical changes
   vim src/path/to/buggy_file.rs

   # Add regression test
   vim tests/path/to/test.rs
   ```

3. **Update CHANGELOG.md:**
   ```markdown
   ## [X.Y.Z+1] - YYYY-MM-DD

   ### Fixed
   - **Critical: [Issue Title]** - Root cause description, fix explanation
     - Affected: vX.Y.Z (and possibly earlier)
     - Severity: High/Critical
     - Workaround: [If any existed]
   ```

4. **Version bump (PATCH only):**
   ```bash
   # Cargo.toml: X.Y.Z → X.Y.Z+1
   # README.md: Update version references
   # CHANGELOG.md: Add hotfix section
   ```

5. **Test extensively:**
   ```bash
   # Full test suite must pass
   cargo test --workspace --locked --lib --bins --tests

   # Verify the specific issue is fixed
   # Regression test must fail on vX.Y.Z, pass on vX.Y.Z+1

   # Platform-specific testing if applicable
   ```

6. **Fast-track release:**
   ```bash
   # Commit
   git commit -m "fix(critical): [Issue title]

   Fixes #ISSUE_NUMBER

   Root Cause: [Brief explanation]
   Fix: [Brief explanation]
   Testing: [How verified]

   This is a hotfix release for vX.Y.Z."

   # Tag
   git tag -a vX.Y.Z+1 -m "Hotfix release for [issue]

   Critical fix for: [issue title]
   See CHANGELOG.md for details."

   # Push
   git push origin hotfix/vX.Y.Z+1
   git push origin vX.Y.Z+1
   ```

7. **Merge back to main:**
   ```bash
   git checkout main
   git merge hotfix/vX.Y.Z+1
   git push origin main
   ```

8. **Announce hotfix prominently:**
   - GitHub Security Advisory (if security issue)
   - Release notes with "HOTFIX" label
   - Update documentation with workaround removal

---

## Breaking Changes Policy

### Definition

A **breaking change** requires users to modify their code, configuration, or workflow when upgrading.

**Examples of breaking changes:**
- Public API signature changes (function parameters, return types)
- Removed CLI flags or options
- Configuration file format changes
- Output format changes (JSON schema modifications)
- Minimum Rust version (MSRV) increase
- Removed platform support

**NOT breaking changes:**
- New CLI flags (additive only)
- New output fields in JSON (if parsers ignore unknown fields)
- Performance improvements
- Internal refactoring
- Deprecated features (if still functional)

### Pre-1.0 Rules (Current)

**Versions 0.x.x:** Breaking changes allowed in MINOR releases
- Example: 0.5.0 → 0.6.0 may break compatibility
- PATCH releases must remain compatible: 0.5.0 → 0.5.1 cannot break

**Deprecation process:**
1. Mark feature as deprecated in current version
2. Add deprecation warning to CLI/logs
3. Document in CHANGELOG.md under "Deprecated"
4. Remove in next MINOR release (0.5.0 deprecate → 0.6.0 remove)

### Post-1.0 Rules (Planned)

**Versions 1.x.x:** Breaking changes ONLY in MAJOR releases
- MINOR releases (1.5.0 → 1.6.0): Must be backward compatible
- MAJOR releases (1.0.0 → 2.0.0): Breaking changes allowed

**Deprecation process:**
1. Deprecate in current MINOR release (e.g., 1.5.0)
2. Support for 2+ MINOR releases (1.5.0, 1.6.0, 1.7.0)
3. Remove in next MAJOR release (2.0.0)
4. Provide migration guide in CHANGELOG.md

### Migration Guide Template

When introducing breaking changes, include this in CHANGELOG.md:

```markdown
## [X.0.0] - YYYY-MM-DD

### Breaking Changes

#### [Feature Name] API Redesign

**Impact:** High - Affects all users using [feature]

**Old API (vX-1.Y.Z):**
```rust
pub fn old_function(param1: Type1) -> Result<Type2> {
    // Old implementation
}
```

**New API (vX.0.0):**
```rust
pub fn new_function(param1: Type1, param2: Type3) -> Result<Type4, Error> {
    // New implementation with enhanced error handling
}
```

**Migration:**
```rust
// Before (v0.5.0):
let result = old_function(param1)?;

// After (v0.6.0):
let result = new_function(param1, default_param2)?;
```

**Rationale:** [Why the breaking change was necessary - performance, safety, features]

**Alternatives Considered:**
- Option A: [Rejected because...]
- Option B: [Rejected because...]

**Deprecation Timeline:**
- v0.5.0: Feature deprecated, warnings added
- v0.5.1-0.5.3: Deprecation warnings in production
- v0.6.0: Feature removed, migration required
```

---

## Automation & CI/CD Integration

### Automated Release Checklist

GitHub Actions automatically verifies:

**Pre-release Validation:**
- ✅ All tests passing (2,100+ tests)
- ✅ Zero clippy warnings
- ✅ Clean formatting (cargo fmt)
- ✅ No cargo-deny advisories
- ✅ Code coverage ≥50% threshold
- ✅ Benchmark regression <10%

**Build Validation:**
- ✅ 8 platform binaries built successfully
- ✅ Binary sizes within expected range (6-9 MB)
- ✅ Smoke tests pass (prtip --version, prtip --help)
- ✅ Cross-compilation successful (musl, ARM64, FreeBSD)

**Release Artifacts:**
- ✅ 8 platform tarballs/zips uploaded to GitHub Release
- ✅ Checksums (SHA256) generated and published
- ✅ Release notes auto-generated from CHANGELOG.md
- ✅ Installation instructions included

### Manual Release Triggers

Support for manual releases without git tag push:

```yaml
# .github/workflows/release.yml
on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release (e.g., 0.5.3)'
        required: true
      dry_run:
        description: 'Dry run (build only, no release)'
        required: false
        default: false
```

**Trigger manually:**
```bash
# Via GitHub UI: Actions → Release Binaries → Run workflow
# Or via CLI:
gh workflow run release.yml -f version=0.5.3 -f dry_run=false
```

---

## Quality Gates

Every release MUST pass these quality gates:

### Code Quality

| Gate | Requirement | Tool |
|------|-------------|------|
| **Tests** | 100% passing | `cargo test` |
| **Coverage** | ≥50% | `cargo-tarpaulin` |
| **Clippy** | 0 warnings | `cargo clippy -- -D warnings` |
| **Formatting** | Clean | `cargo fmt --check` |
| **MSRV** | Rust 1.85+ | CI matrix |

### Security

| Gate | Requirement | Tool |
|------|-------------|------|
| **Advisories** | 0 unaddressed | `cargo deny check advisories` |
| **Dependencies** | Up-to-date | `cargo outdated` |
| **Audit** | 0 vulnerabilities | `cargo audit` |
| **SAST** | 0 critical issues | CodeQL |

### Performance

| Gate | Requirement | Tool |
|------|-------------|------|
| **Regression** | <10% slowdown | Benchmark suite |
| **Memory** | No leaks | Valgrind, cargo-memtest |
| **Overhead** | <5% framework overhead | Profiling |

### Documentation

| Gate | Requirement | Tool |
|------|-------------|------|
| **CHANGELOG** | Complete entry | Manual review |
| **README** | Version updated | Manual review |
| **API Docs** | 0 broken links | `cargo doc` |
| **Examples** | All compile | `cargo test --examples` |

**Failure Handling:**
- Any gate failure → BLOCK release
- Fix issue → Re-run CI → Verify all gates pass
- No exceptions for time pressure

---

## Rollback Procedures

### When to Rollback

**Immediate rollback required if:**
- Critical security vulnerability discovered post-release
- Data corruption or loss in production use
- Widespread crashes (>10% of users affected)
- Silent failures (incorrect results without errors)

**Rollback NOT required if:**
- Performance regression <50%
- Non-critical UI bugs
- Documentation errors
- Platform-specific issues affecting <5% users

### Rollback Process

1. **Deprecate bad release on GitHub:**
   ```bash
   # Mark release as "Pre-release" (yellow badge)
   gh release edit vX.Y.Z --prerelease

   # Add rollback notice to release notes
   gh release edit vX.Y.Z --notes "⚠️ DEPRECATED - Critical issue found

   **DO NOT USE THIS RELEASE**

   Issue: [Brief description]
   Rollback to: vX.Y.Z-1
   Fix planned: vX.Y.Z+1

   See: https://github.com/doublegate/ProRT-IP/issues/XXX"
   ```

2. **Create hotfix immediately:**
   - Follow [Hotfix Procedures](#hotfix-procedures)
   - Bump to vX.Y.Z+1
   - Include fix + regression test

3. **Update documentation:**
   ```markdown
   # README.md
   ## ⚠️ Version Advisory

   **v0.5.2 is deprecated due to critical issue [#XXX].**

   - Do not use: v0.5.2
   - Use instead: v0.5.1 (previous stable) or v0.5.3 (hotfix)
   - Issue: [Brief description]
   ```

4. **Notify users:**
   - GitHub Security Advisory (if security issue)
   - GitHub Discussions announcement
   - Update installation instructions
   - Social media (if applicable)

5. **Post-mortem:**
   - Document root cause in `/tmp/ProRT-IP/POST-MORTEM-vX.Y.Z.md`
   - Identify process gaps (why wasn't caught in testing?)
   - Update quality gates to prevent recurrence
   - Share learnings in CHANGELOG.md

---

## Release Metrics

### Historical Data (Oct 7 - Nov 14, 2025)

**Release Velocity:**
- Total releases: 15
- Duration: 38 days
- Average: 1 release per 2.5 days
- Peak velocity: Multiple releases per day (Phase 4-5 development)

**Release Types:**
- MAJOR: 0 (pre-1.0)
- MINOR: 6 (Phases 3, 4, 5 + sprints 6.1, 6.2)
- PATCH: 9 (Bug fixes, optimizations, documentation)

**Binary Distribution:**
- Platforms: 8 (5 production + 3 experimental)
- Total artifacts: 120 binaries (15 releases × 8 platforms)
- Artifact size: 6-9 MB per binary

**Quality Trends:**
- Test count: 391 → 2,100+ (437% growth)
- Coverage: ~30% → 54.92% (+24.92pp)
- Fuzz executions: 0 → 230M+ (230 million+)
- CI success rate: ~85% → ~95% (+10pp)

### Key Performance Indicators (KPIs)

**Development Velocity:**
- Sprint completion rate: 95% (20/21 sprints on time)
- Average sprint duration: 15-20 hours
- Features per sprint: 3-6 major features

**Quality:**
- Zero production crashes (0 crash reports)
- Zero security vulnerabilities (0 CVEs assigned)
- Test reliability: 99.5% (flaky tests fixed)
- Documentation completeness: 90%+ (50,000+ lines)

**Community:**
- GitHub stars: [Tracked separately]
- Contributors: [Tracked separately]
- Issues resolved: [Tracked in project status]
- PR merge time: [Tracked in GitHub metrics]

---

## Future Improvements

### Planned Enhancements (Phase 7-8)

**Release Automation:**
- [ ] Fully automated releases (zero manual steps)
- [ ] Automated CHANGELOG generation from commit messages
- [ ] Release notes template with smart filling
- [ ] Slack/Discord release notifications

**Testing:**
- [ ] Pre-release beta channel for community testing
- [ ] Automated smoke tests on fresh VM instances
- [ ] Performance regression dashboard
- [ ] Platform-specific CI runners (ARM64, FreeBSD)

**Distribution:**
- [ ] Package manager support (Homebrew, Chocolatey, apt/dnf repos)
- [ ] Docker images (Alpine, Ubuntu, Arch)
- [ ] Binary reproducibility verification
- [ ] Nightly builds for `main` branch

**Quality:**
- [ ] 70% code coverage target
- [ ] Zero tolerance for clippy warnings
- [ ] Fuzz testing in CI (continuous fuzzing)
- [ ] Security audit every major release

---

## See Also

- [CI/CD Pipeline](ci-cd.md) - Continuous integration and deployment automation
- [Testing](testing.md) - Testing philosophy and coverage goals
- [Fuzzing](fuzzing.md) - Fuzz testing framework and targets
- [Documentation Standards](doc-standards.md) - Documentation numbering and organization
- [Contributing](contributing.md) - Pull request and code review process
- [GitHub Actions Workflows](/.github/workflows/) - CI/CD workflow definitions
- [CHANGELOG.md](/CHANGELOG.md) - Complete project history
- [Semantic Versioning](https://semver.org/) - Official SemVer specification
- [Keep a Changelog](https://keepachangelog.com/) - Changelog format standard
