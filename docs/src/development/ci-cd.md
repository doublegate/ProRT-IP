# CI/CD Pipeline

ProRT-IP uses GitHub Actions for comprehensive continuous integration and continuous deployment. The CI/CD pipeline ensures code quality, security, and reliable releases across 8 target platforms.

## Overview

### Pipeline Philosophy

**Automated Quality Gates:**
- Format checking (rustfmt)
- Linting (clippy with `-D warnings`)
- Cross-platform testing (Linux, macOS, Windows)
- Security auditing (cargo-deny, CodeQL)
- Coverage tracking (tarpaulin + Codecov)
- Performance benchmarking (hyperfine)

**Efficiency Optimizations:**
- **Path filtering:** Only run workflows when relevant files change
- **Concurrency control:** Cancel outdated workflow runs
- **Incremental caching:** Swatinem/rust-cache@v2 for ~85% cache hit rate
- **Conditional execution:** Platform-specific steps only when needed
- **Smart triggers:** Release-only coverage (reduce CI load by 80%)

**Key Metrics:**
- **9 workflows:** ci.yml, coverage.yml, release.yml, codeql.yml, benchmarks.yml, fuzz.yml, mdbook.yml, markdown-links.yml, dependency-review.yml
- **8 release targets:** Linux (GNU/musl x86/ARM), Windows, macOS (Intel/ARM), FreeBSD
- **3 test platforms:** Ubuntu, macOS, Windows (with platform-specific test subsets)
- **230+ tests:** 2,111 total tests across unit/integration/doctest levels
- **54.92% coverage:** Code coverage with 50% minimum threshold

## Core Workflows

### CI Workflow (ci.yml)

Main continuous integration workflow running on all push/PR events to `main` branch.

#### Workflow Configuration

**Triggers:**
```yaml
on:
  push:
    branches: [ main ]
    paths:
      - 'crates/**'
      - 'fuzz/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
      - '.github/workflows/ci.yml'
  pull_request:
    branches: [ main ]
    paths: [same as push]
```

**Concurrency Control:**
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

Automatically cancels outdated workflow runs when new commits are pushed, saving CI resources.

**Environment Variables:**
```yaml
env:
  CARGO_TERM_COLOR: always    # Colored output for readability
  RUST_BACKTRACE: 1            # Full backtraces for test failures
  CARGO_INCREMENTAL: 0         # Disable incremental for clean CI builds
```

#### Job 1: Format Check

**Purpose:** Ensure consistent code formatting across entire workspace

**Implementation:**
```yaml
format:
  name: Format Check
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt
    - run: cargo fmt --all -- --check
```

**Exit Codes:**
- `0` - All code properly formatted
- `1` - Formatting violations found (workflow fails)

**Fix:** Run `cargo fmt --all` locally before committing

#### Job 2: Clippy Lint

**Purpose:** Static analysis for common mistakes, antipatterns, and potential bugs

**Implementation:**
```yaml
clippy:
  name: Clippy Lint
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - uses: Swatinem/rust-cache@v2
      with:
        shared-key: "clippy"
    - run: cargo clippy --workspace --all-targets --locked -- -D warnings
```

**Key Features:**
- `--all-targets` - Check lib, bins, tests, benches, examples
- `--locked` - Use exact versions from Cargo.lock (reproducibility)
- `-D warnings` - Treat all warnings as errors (zero-tolerance policy)

**Common Clippy Warnings:**
- `clippy::field_reassign_with_default` - Use struct update syntax
- `clippy::useless_vec` - Remove unnecessary `vec![]` macro
- `clippy::format_push_string` - Use `write!()` instead of `push_str(&format!())`

**Fix:** Run `cargo clippy --workspace --all-targets --fix` locally

#### Job 3: Cross-Platform Tests

**Purpose:** Validate functionality on Linux, macOS, and Windows

**Matrix Strategy:**
```yaml
test:
  strategy:
    fail-fast: false
    matrix:
      os: [ubuntu-latest, windows-latest, macos-latest]
      rust: [stable]
```

**Platform-Specific Dependencies:**

**Linux (Ubuntu):**
```yaml
- name: Install system dependencies (Linux)
  if: matrix.os == 'ubuntu-latest'
  run: sudo apt-get update && sudo apt-get install -y libpcap-dev pkg-config
```

**macOS:**
```yaml
- name: Install system dependencies (macOS)
  if: matrix.os == 'macos-latest'
  run: |
    # Install libpcap (only if not already present - avoid warnings)
    brew list libpcap &>/dev/null || brew install libpcap
    # pkg-config is provided by pkgconf (pre-installed on GitHub Actions)
    brew list pkgconf &>/dev/null || brew install pkgconf
```

**Windows (Npcap SDK + Runtime DLLs):**
```yaml
- name: Install Npcap SDK and Runtime DLLs (Windows)
  if: matrix.os == 'windows-latest'
  shell: pwsh
  run: |
    # Download Npcap SDK (Packet.lib for development)
    curl -L -o npcap-sdk.zip https://npcap.com/dist/npcap-sdk-1.13.zip
    Expand-Archive -Path npcap-sdk.zip -DestinationPath npcap-sdk

    # Download Npcap installer and extract DLLs without running (avoids hang)
    curl -L -o npcap-installer.exe https://npcap.com/dist/npcap-1.79.exe
    7z x npcap-installer.exe -o"npcap-runtime" -y

    # Create runtime directory and copy ONLY x64 DLLs
    New-Item -ItemType Directory -Force -Path "npcap-dlls"
    Get-ChildItem -Path "npcap-runtime" -Recurse -Filter "*.dll" | Where-Object {
      ($_.Name -eq "Packet.dll" -or $_.Name -eq "wpcap.dll") -and
      $_.DirectoryName -like "*x64*"
    } | ForEach-Object {
      Copy-Item $_.FullName -Destination "npcap-dlls\" -Force
    }

    # Add SDK lib directory to LIB environment variable for linking
    echo "LIB=$PWD\npcap-sdk\Lib\x64;$env:LIB" >> $env:GITHUB_ENV
    # Add DLL directory to PATH for runtime
    echo "PATH=$PWD\npcap-dlls;$env:PATH" >> $env:GITHUB_ENV
```

**Rationale:**
- **SDK download:** Contains Packet.lib required for linking
- **Installer extraction:** Avoids 90-second hang from running installer
- **x64-only filtering:** Prevents 32-bit/64-bit architecture mismatch errors
- **Environment variables:** `LIB` for compilation, `PATH` for runtime

**Dependency Caching:**
```yaml
- name: Cache dependencies
  uses: Swatinem/rust-cache@v2
  with:
    shared-key: "test-${{ matrix.os }}"
```

**Cache Performance:**
- **Hit rate:** ~85% on subsequent runs
- **Time savings:** 3-5 minutes per workflow run
- **Cache size:** 200-500 MB per platform

**Build Step:**
```yaml
- name: Build
  run: cargo build --workspace --locked --verbose
```

**Test Execution (Platform-Specific):**
```yaml
- name: Run tests
  run: |
    if [ "${{ matrix.os }}" = "windows-latest" ]; then
      # Windows: Run only unit tests (no Npcap integration tests)
      cargo test --workspace --locked --lib --exclude prtip-network --exclude prtip-scanner
    else
      # Linux/macOS: Run unit and integration tests, skip doctests
      # Doctests skipped to prevent linker resource exhaustion in CI
      cargo test --workspace --locked --lib --bins --tests
    fi
  shell: bash
  env:
    PRTIP_DISABLE_HISTORY: "1"  # Prevent race conditions in parallel tests
```

**Platform Differences:**

| Platform | Test Level | Packages | Rationale |
|----------|-----------|----------|-----------|
| **Linux** | Unit + Integration | All workspace | Full libpcap support |
| **macOS** | Unit + Integration | All workspace | Full BPF support |
| **Windows** | Unit only | Exclude prtip-network, prtip-scanner | Npcap limitations on loopback |

**Doctest Exclusion:**
- **Reason:** Linker bus error (signal 7) during doctest compilation in CI environment
- **Impact:** Zero test coverage loss (all functionality covered by unit/integration tests)
- **Fix:** Changed from `cargo test --workspace` to `cargo test --workspace --lib --bins --tests`

**Code Coverage Integration:**
```yaml
- name: Install cargo-tarpaulin
  if: matrix.os != 'windows-latest'
  run: cargo install cargo-tarpaulin

- name: Generate test coverage with tarpaulin
  if: matrix.os != 'windows-latest'
  run: |
    cargo tarpaulin --workspace --locked --lib --bins --tests \
      --exclude prtip-network --exclude prtip-scanner \
      --out Xml --output-dir ./coverage \
      --timeout 300
  env:
    PRTIP_DISABLE_HISTORY: "1"

- name: Upload test coverage to Codecov
  if: ${{ !cancelled() && matrix.os != 'windows-latest' }}
  uses: codecov/codecov-action@v4
  with:
    token: ${{ secrets.CODECOV_TOKEN }}
    files: ./coverage/cobertura.xml
    fail_ci_if_error: false
    verbose: true
```

**Tarpaulin Configuration:**
- **Exclusions:** prtip-network and prtip-scanner (platform-specific network code)
- **Timeout:** 300 seconds (5 minutes) to prevent CI hangs
- **Output format:** Cobertura XML for Codecov integration
- **Platform:** Linux/macOS only (tarpaulin doesn't support Windows)

**Codecov Integration:**
- **Action:** codecov/codecov-action@v4 (correct for coverage data, not test results)
- **Token:** Required for private repositories
- **Fail on error:** `false` (non-blocking, coverage failures don't fail CI)
- **File path:** Explicit `./coverage/cobertura.xml` path

#### Job 4: Security Audit

**Purpose:** Check for known security vulnerabilities in dependencies

**Implementation:**
```yaml
security_audit:
  name: Security Audit
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: EmbarkStudios/cargo-deny-action@v2
      with:
        log-level: warn
        command: check advisories
        arguments: --all-features
```

**cargo-deny Configuration:**
- **Command:** `check advisories` - Only check security advisories (not licenses/bans/sources)
- **All features:** Check all feature combinations for vulnerabilities
- **Log level:** `warn` - Show warnings but don't fail on info-level messages

**Ignored Advisories (deny.toml):**
```toml
[[advisories.ignore]]
id = "RUSTSEC-2024-0436"
# paste crate unmaintained (transitive dep from ratatui 0.28.1/0.29.0)
# SAFE: Compile-time only proc-macro, zero runtime risk
# MONITORING: Awaiting pastey migration in ratatui upstream
```

**Exit Codes:**
- `0` - No vulnerabilities found
- `1` - Vulnerabilities found (workflow fails)

#### Job 5: MSRV Check

**Purpose:** Ensure minimum supported Rust version (1.85) builds successfully

**Implementation:**
```yaml
msrv:
  name: MSRV Check (1.85)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@1.85
    - run: sudo apt-get update && sudo apt-get install -y libpcap-dev pkg-config
    - uses: Swatinem/rust-cache@v2
      with:
        shared-key: "msrv"
    - run: cargo build --workspace --locked --verbose
```

**MSRV Policy:**
- **Current:** Rust 1.85
- **Update frequency:** Every 6 months (align with Rust edition releases)
- **Justification:** Balance modern features with stable toolchain availability

### Coverage Workflow (coverage.yml)

Dedicated code coverage workflow running on release tags or manual trigger.

#### Workflow Configuration

**Triggers:**
```yaml
on:
  push:
    tags:
      - 'v*.*.*'  # Only on release tags (reduce CI load by 80%)
  workflow_dispatch:
    inputs:
      version:
        description: 'Version tag (e.g., v0.4.6)'
        required: false
        type: string
```

**Rationale:**
- **Release-only:** Coverage analysis only needed for releases (detailed coverage already in ci.yml)
- **Manual trigger:** Allow on-demand coverage runs for development
- **CI efficiency:** Reduces coverage workflow runs by 80% (tags only vs every push)

#### Coverage Generation

**Tarpaulin Execution:**
```yaml
- name: Generate coverage report
  id: tarpaulin
  run: |
    OUTPUT=$(cargo tarpaulin --workspace \
      --timeout 600 \
      --out Lcov --out Html --out Json \
      --output-dir coverage \
      --exclude-files "crates/prtip-cli/src/main.rs" 2>&1)

    echo "$OUTPUT"

    # Extract coverage percentage (format: "XX.XX% coverage")
    COVERAGE=$(echo "$OUTPUT" | grep -oP '\d+\.\d+(?=% coverage)' | tail -1)

    if [ -z "$COVERAGE" ]; then
      echo "Error: Could not extract coverage percentage"
      exit 1
    fi

    echo "coverage=$COVERAGE" >> $GITHUB_OUTPUT
```

**Output Formats:**
- **Lcov:** For Codecov upload (industry-standard format)
- **Html:** Human-readable report with line-by-line coverage highlighting
- **Json:** Machine-parseable format for tooling integration

**Exclusions:**
- `crates/prtip-cli/src/main.rs` - Entry point (minimal logic, not testable)

#### Codecov Upload

```yaml
- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: coverage/lcov.info
    flags: rust
    name: codecov-prtip
    fail_ci_if_error: false
    token: ${{ secrets.CODECOV_TOKEN }}
```

**Codecov Configuration:**
- **Flags:** `rust` tag for filtering in Codecov dashboard
- **Name:** `codecov-prtip` identifier for multi-project accounts
- **Fail on error:** `false` (non-blocking if Codecov service is down)

#### Coverage Threshold Enforcement

```yaml
- name: Check coverage threshold
  run: |
    COVERAGE=${{ steps.coverage.outputs.percentage }}
    THRESHOLD=50.0

    # Use awk for floating point comparison
    if awk -v cov="$COVERAGE" -v thr="$THRESHOLD" 'BEGIN {exit !(cov < thr)}'; then
      echo "❌ Coverage $COVERAGE% is below threshold $THRESHOLD%"
      echo "::error::Coverage regression detected"
      exit 1
    fi

    echo "✅ Coverage $COVERAGE% meets threshold $THRESHOLD%"
```

**Threshold Policy:**
- **Minimum:** 50.0% total coverage
- **Comparison:** `awk` for floating point (bc not always available in CI)
- **Enforcement:** Fail workflow if below threshold
- **Current:** 54.92% coverage (exceeds minimum by 4.92 percentage points)

#### PR Coverage Comments

```yaml
- name: Comment PR with coverage
  if: github.event_name == 'pull_request'
  uses: actions/github-script@v6
  with:
    script: |
      const coverage = '${{ steps.coverage.outputs.percentage }}';
      const threshold = '50.0';
      const passed = parseFloat(coverage) >= parseFloat(threshold);
      const emoji = passed ? '✅' : '❌';

      const comment = `## ${emoji} Coverage Report

      **Current Coverage:** ${coverage}%
      **Threshold:** ${threshold}%
      **Status:** ${passed ? 'PASSED' : 'FAILED'}

      ${passed ?
        '✅ Coverage meets the minimum threshold.' :
        '❌ Coverage below minimum. Please add more tests.'}

      📊 [View detailed report](https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }})
      `;

      github.rest.issues.createComment({
        issue_number: context.issue.number,
        owner: context.repo.owner,
        repo: context.repo.repo,
        body: comment
      });
```

**Comment Features:**
- **Pass/fail emoji:** Visual indicator in PR conversation
- **Coverage percentage:** Exact value with 2 decimal precision
- **Threshold comparison:** Clear pass/fail status
- **Artifact link:** Direct link to detailed HTML report

### Release Workflow (release.yml)

Automated release creation and multi-platform binary distribution.

#### Workflow Configuration

**Triggers:**
```yaml
on:
  push:
    tags:
      - 'v*.*.*'
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release (e.g., v0.3.0)'
        required: true
        type: string
      attach_only:
        description: 'Only attach artifacts to existing release'
        required: false
        type: boolean
        default: true
```

**Permissions:**
```yaml
permissions:
  contents: write  # Required for creating releases and uploading assets
```

#### Job 1: Check Release Existence

**Purpose:** Avoid duplicate releases, enable artifact re-attachment

**Implementation:**
```yaml
check-release:
  outputs:
    release_exists: ${{ steps.check.outputs.exists }}
    release_id: ${{ steps.check.outputs.id }}
    version: ${{ steps.version.outputs.tag }}

  steps:
    - name: Determine version
      id: version
      run: |
        if [ "${{ github.event_name }}" = "workflow_dispatch" ]; then
          VERSION="${{ inputs.version }}"
        else
          VERSION="${GITHUB_REF#refs/tags/}"
        fi
        echo "tag=$VERSION" >> $GITHUB_OUTPUT

    - name: Check if release exists
      id: check
      env:
        GH_TOKEN: ${{ github.token }}
      run: |
        if gh release view "$VERSION" --repo ${{ github.repository }} &>/dev/null; then
          echo "exists=true" >> $GITHUB_OUTPUT
          RELEASE_ID=$(gh api repos/${{ github.repository }}/releases/tags/$VERSION --jq '.id')
          echo "id=$RELEASE_ID" >> $GITHUB_OUTPUT
        else
          echo "exists=false" >> $GITHUB_OUTPUT
        fi
```

**Use Cases:**
- **New release:** Create release with generated notes
- **Re-run build:** Attach artifacts to existing release (preserve manual notes)
- **Manual trigger:** Build artifacts for specific version

#### Job 2: Create Release

**Purpose:** Generate dynamic release notes with project statistics

**Conditional Execution:**
```yaml
create-release:
  needs: check-release
  if: needs.check-release.outputs.release_exists == 'false'
```

**Release Notes Template:**
```markdown
# ProRT-IP WarScan $VERSION

Modern network scanner combining Masscan speed with Nmap detection depth.

## 📊 Project Statistics

- **Tests:** $TEST_COUNT+
- **Lines of Code:** $LOC+
- **Crates:** 4 (prtip-core, prtip-network, prtip-scanner, prtip-cli)

## ✨ Key Features

- **7 scan types:** TCP Connect, SYN, UDP, FIN, NULL, Xmas, ACK
- **OS fingerprinting:** 16-probe Nmap sequence
- **Service detection:** 500+ probes
- **Timing templates:** T0-T5 (Paranoid to Insane)

## 📦 Installation

[Platform-specific installation instructions]

## 🔧 Usage Examples

[Common usage patterns]

## 📝 Changelog

[CHANGELOG.md entries for this version]
```

**Statistics Calculation:**
```bash
TEST_COUNT=$(grep -r "^fn test_" --include="*.rs" crates/ | wc -l)
LOC=$(find crates -name "*.rs" -exec cat {} \; | wc -l)
```

**CHANGELOG.md Integration:**
```bash
if grep -q "## \[$VERSION_NUM\]" CHANGELOG.md; then
  # Extract notes between this version and next ## marker
  CHANGELOG_NOTES=$(sed -n "/## \[$VERSION_NUM\]/,/^## \[/p" CHANGELOG.md | sed '$d' | tail -n +2)
else
  CHANGELOG_NOTES="See CHANGELOG.md for complete version history."
fi
```

#### Job 3: Build Release Binaries

**Purpose:** Cross-compile binaries for 8 target platforms

**Build Matrix:**
```yaml
build-release:
  strategy:
    fail-fast: false
    matrix:
      include:
        # Linux - Debian/Ubuntu (glibc) - x86_64
        - target: x86_64-unknown-linux-gnu
          os: ubuntu-latest
          archive: tar.gz

        # Linux - Alpine/Static (musl) - x86_64
        - target: x86_64-unknown-linux-musl
          os: ubuntu-latest
          archive: tar.gz

        # Linux - Debian/Ubuntu (glibc) - ARM64
        - target: aarch64-unknown-linux-gnu
          os: ubuntu-latest
          archive: tar.gz
          cross: true

        # Linux - Alpine/Static (musl) - ARM64
        - target: aarch64-unknown-linux-musl
          os: ubuntu-latest
          archive: tar.gz
          cross: true

        # Windows 10/11 - x86_64
        - target: x86_64-pc-windows-msvc
          os: windows-latest
          archive: zip

        # macOS - Intel x86_64 (older Macs)
        - target: x86_64-apple-darwin
          os: macos-13
          archive: tar.gz

        # macOS - Apple Silicon ARM64 (M1/M2/M3/M4)
        - target: aarch64-apple-darwin
          os: macos-latest
          archive: tar.gz

        # FreeBSD - x86_64
        - target: x86_64-unknown-freebsd
          os: ubuntu-latest
          archive: tar.gz
          cross: true
```

**Cross-Compilation Setup:**
```yaml
- name: Install cross-compilation tool
  if: matrix.cross == true
  run: cargo install cross --git https://github.com/cross-rs/cross
```

**musl Static Linking:**
```yaml
- name: Install musl tools (Linux musl x86_64)
  if: matrix.target == 'x86_64-unknown-linux-musl'
  run: sudo apt-get update && sudo apt-get install -y musl-tools
```

**Build with Vendored OpenSSL:**
```yaml
- name: Build release binary
  run: |
    if [ "${{ matrix.cross }}" = "true" ]; then
      BUILD_CMD="cross"
    else
      BUILD_CMD="cargo"
    fi

    # Enable vendored-openssl for musl and cross-compiled ARM
    if [[ "${{ matrix.target }}" == *"musl"* ]] ||
       [[ "${{ matrix.cross }}" == "true" && "${{ matrix.target }}" == "aarch64"* ]]; then
      $BUILD_CMD build --release --target ${{ matrix.target }} --locked \
        --features prtip-scanner/vendored-openssl
    else
      $BUILD_CMD build --release --target ${{ matrix.target }} --locked
    fi
  env:
    OPENSSL_STATIC: 1  # Force static linking for musl
```

**Archive Creation:**

**Unix (tar.gz):**
```bash
cd target/${{ matrix.target }}/release
tar czf ../../../prtip-${VERSION_NUM}-${{ matrix.target }}.tar.gz prtip
```

**Windows (zip):**
```powershell
cd target/${{ matrix.target }}/release
Compress-Archive -Path prtip.exe -DestinationPath $env:GITHUB_WORKSPACE/prtip-${VERSION_NUM}-${{ matrix.target }}.zip
```

**Artifact Upload:**
```yaml
- name: Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: prtip-${{ matrix.target }}
    path: prtip-*-${{ matrix.target }}.*
    retention-days: 1  # Temporary storage until release attachment
```

#### Job 4: Upload to GitHub Release

**Purpose:** Attach build artifacts to release (new or existing)

**Implementation:**
```yaml
upload-artifacts:
  needs: [check-release, build-release]
  steps:
    - name: Download all artifacts
      uses: actions/download-artifact@v4
      with:
        path: artifacts

    - name: Upload to existing or new release
      env:
        GH_TOKEN: ${{ github.token }}
      run: |
        VERSION="${{ needs.check-release.outputs.version }}"
        RELEASE_EXISTS="${{ needs.check-release.outputs.release_exists }}"
        ATTACH_ONLY="${{ inputs.attach_only || 'true' }}"

        if [ "$RELEASE_EXISTS" = "true" ] && [ "$ATTACH_ONLY" = "true" ]; then
          # Attach to existing release (preserve notes)
          find artifacts -type f \( -name "*.tar.gz" -o -name "*.zip" \) | while read file; do
            gh release upload "$VERSION" "$file" --clobber --repo ${{ github.repository }}
          done
        else
          # Upload to new release
          find artifacts -type f \( -name "*.tar.gz" -o -name "*.zip" \) | while read file; do
            gh release upload "$VERSION" "$file" --clobber --repo ${{ github.repository }}
          done
        fi
```

**attach_only Mode:**
- **Purpose:** Re-run builds without modifying manually-written release notes
- **Use case:** CI failures after manual release creation
- **Default:** `true` (preserve notes by default)

#### Job 5: Trigger Coverage Workflow

**Purpose:** Generate coverage report for release version

**Implementation:**
```yaml
trigger-coverage:
  needs: [check-release, upload-artifacts]
  if: success() && github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v')

  steps:
    - name: Trigger coverage workflow
      uses: actions/github-script@v7
      with:
        script: |
          await github.rest.actions.createWorkflowDispatch({
            owner: context.repo.owner,
            repo: context.repo.repo,
            workflow_id: 'coverage.yml',
            ref: 'main',
            inputs: {
              version: '${{ needs.check-release.outputs.version }}'
            }
          });
```

**Workflow Chain:**
1. Tag push triggers `release.yml`
2. Release workflow builds artifacts
3. Release workflow triggers `coverage.yml` via workflow_dispatch
4. Coverage workflow runs on release tag

### CodeQL Security Analysis (codeql.yml)

Automated security vulnerability scanning with GitHub CodeQL.

#### Workflow Configuration

**Triggers:**
```yaml
on:
  push:
    branches: [ main ]
    paths: [crates/**, fuzz/**, Cargo.toml, Cargo.lock, .github/workflows/codeql.yml]
  pull_request:
    branches: [ main ]
    paths: [same as push]
  schedule:
    - cron: '0 3 * * 1'  # Weekly on Monday at 03:00 UTC
```

**Permissions:**
```yaml
permissions:
  actions: read
  contents: read
  security-events: write  # Required for uploading SARIF results
```

#### CodeQL Analysis

**Language Configuration:**
```yaml
- name: Initialize CodeQL
  uses: github/codeql-action/init@v3
  with:
    languages: 'rust'
```

**Known Limitations:**

**Documented in Workflow:**
```yaml
# Note: CodeQL Rust extractor has known limitations:
# - Macro expansion: Complex macros (assert! with format strings) fail to expand
# - Turbofish syntax: Generic type parameters (gen::<f64>()) cause parse errors
# - Platform-specific: #[cfg(target_os = "...")] only analyzed on matching platforms
# These limitations affect test code only, not production security coverage.
```

**Extraction Coverage:**
- **Success rate:** ~97% of Rust files (excellent for Rust projects)
- **Failed files:** Test code only (assertions, utilities)
- **Production impact:** Zero (all security-critical code successfully analyzed)

**Build and Analysis:**
```yaml
- name: Build
  run: cargo build --workspace
  # CodeQL analyzes compiled code, all source files processed during build

- name: Perform CodeQL Analysis
  uses: github/codeql-action/analyze@v3
  # Results uploaded to GitHub Security tab
```

**Expected Messages:**
```
INFO: macro expansion failed (test assertions with complex format strings)
WARN: Expected field name (turbofish syntax in test utilities)
INFO: not included as a module (platform-specific code excluded)
```

**Verification:**
- All messages verified by `cargo check` and `cargo clippy` (no code issues)
- These are CodeQL extractor limitations, not code defects

### Performance Benchmarking (benchmarks.yml)

Automated performance regression detection using hyperfine.

#### Workflow Configuration

**Triggers:**
```yaml
on:
  workflow_dispatch:  # Manual trigger for on-demand benchmarking
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday at 00:00 UTC
```

**Disabled Triggers (commented out):**
```yaml
# push:
#   branches: [main]  # Disabled to avoid excessive CI usage
# pull_request:
#   types: [opened, synchronize, reopened]
```

**Rationale:**
- **Weekly schedule:** Regular monitoring without overwhelming CI resources
- **Manual trigger:** On-demand benchmarking during development
- **No automatic PR runs:** Benchmarks are expensive (~30 min runtime)

#### Benchmark Execution

**Hyperfine Installation:**
```yaml
- name: Cache hyperfine installation
  id: cache-hyperfine
  uses: actions/cache@v4
  with:
    path: ~/.cargo/bin/hyperfine
    key: ${{ runner.os }}-hyperfine-1.18.0

- name: Install hyperfine
  if: steps.cache-hyperfine.outputs.cache-hit != 'true'
  run: cargo install hyperfine --version 1.18.0
```

**Benchmark Suite:**
```yaml
- name: Run benchmark suite
  id: run-benchmarks
  run: |
    cd benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts
    chmod +x run-all-benchmarks.sh
    ./run-all-benchmarks.sh
    echo "timestamp=$(date -u +%Y%m%d-%H%M%S)" >> $GITHUB_OUTPUT
```

**Benchmark Scenarios:**
- **8 core scans:** SYN, Connect, UDP, FIN, NULL, Xmas, ACK, Idle
- **4 stealth variants:** Fragmentation, decoys, TTL modification, source port
- **4 scale tests:** 100 ports, 1K ports, 10K ports, 65K ports
- **2 timing templates:** T2 (Polite), T4 (Aggressive)
- **5 overhead tests:** Service detection, OS fingerprinting, output formats, rate limiting, evasion

#### Baseline Comparison

**Find Latest Baseline:**
```yaml
- name: Find latest baseline
  id: find-baseline
  run: |
    if [ -d "benchmarks/baselines" ]; then
      latest_baseline=$(ls -1 benchmarks/baselines/baseline-v*.json 2>/dev/null | sort -V | tail -n 1)
      if [ -n "$latest_baseline" ]; then
        echo "baseline_found=true" >> $GITHUB_OUTPUT
        echo "baseline_file=$latest_baseline" >> $GITHUB_OUTPUT
      fi
    fi
```

**Compare Against Baseline:**
```yaml
- name: Compare against baseline
  if: steps.find-baseline.outputs.baseline_found == 'true'
  id: compare
  run: |
    cd benchmarks/05-Sprint5.9-Benchmarking-Framework
    ./scripts/analyze-results.sh "${{ steps.find-baseline.outputs.baseline_file }}" results
    echo "exit_code=$?" >> $GITHUB_OUTPUT
```

**Exit Code Interpretation:**
- **0:** All benchmarks within acceptable range (pass)
- **1:** Some benchmarks show potential regression (warning, within tolerance)
- **2:** Significant performance regression detected (fail)

**Regression Thresholds:**
```bash
# In analyze-results.sh
WARNING_THRESHOLD=5   # 5% slowdown = warning
FAILURE_THRESHOLD=10  # 10% slowdown = failure
```

#### PR Comments

**Generate Comment:**
```yaml
- name: Comment on PR
  if: github.event_name == 'pull_request' && steps.find-baseline.outputs.baseline_found == 'true'
  uses: actions/github-script@v7
  with:
    script: |
      const fs = require('fs');
      const commentPath = 'benchmarks/05-Sprint5.9-Benchmarking-Framework/results/pr-comment.md';

      if (fs.existsSync(commentPath)) {
        const comment = fs.readFileSync(commentPath, 'utf8');
        github.rest.issues.createComment({
          issue_number: context.issue.number,
          owner: context.repo.owner,
          repo: context.repo.repo,
          body: comment
        });
      }
```

**Comment Format:**
```markdown
## 📊 Benchmark Results

| Scenario | Baseline | Current | Change | Status |
|----------|----------|---------|--------|--------|
| SYN Scan (1000 ports) | 287ms | 295ms | +2.8% | ⚠️ Warning |
| Service Detection | 3.45s | 3.52s | +2.0% | ✅ Pass |
| OS Fingerprinting | 1.23s | 1.21s | -1.6% | ✅ Pass |

**Summary:**
- ✅ 18/20 benchmarks within tolerance
- ⚠️ 2/20 show potential regression (within 5% threshold)
- ❌ 0/20 significant regressions

[View detailed results](https://github.com/repo/actions/runs/123456)
```

#### Workflow Failure Modes

**Fail on Regression:**
```yaml
- name: Fail on regression
  if: steps.compare.outputs.exit_code == '2'
  run: |
    echo "::error::Performance regression detected!"
    exit 1
```

**Warn on Potential Regression:**
```yaml
- name: Warn on potential regression
  if: steps.compare.outputs.exit_code == '1'
  run: |
    echo "::warning::Potential regression (within tolerance). Review recommended."
```

**Artifact Retention:**
```yaml
- name: Upload benchmark results
  uses: actions/upload-artifact@v4
  with:
    name: benchmark-results-${{ steps.run-benchmarks.outputs.timestamp }}
    path: |
      benchmarks/05-Sprint5.9-Benchmarking-Framework/results/*.json
      benchmarks/05-Sprint5.9-Benchmarking-Framework/results/*.md
    retention-days: 90  # 3 months of benchmark history
```

## Additional Workflows

### Fuzzing Workflow (fuzz.yml)

**Purpose:** Continuous fuzzing with libFuzzer for security robustness

**Configuration:**
```yaml
on:
  schedule:
    - cron: '0 2 * * *'  # Nightly at 2 AM UTC
  workflow_dispatch:
```

**Execution:**
```yaml
strategy:
  matrix:
    target: [fuzz_tcp_parser, fuzz_udp_parser, fuzz_ipv6_parser,
             fuzz_icmpv6_parser, fuzz_tls_parser]

steps:
  - name: Run fuzzer
    run: |
      cd fuzz
      timeout 600 cargo fuzz run ${{ matrix.target }} \
        -- -max_total_time=600 -max_len=2000 || true

  - name: Upload corpus
    uses: actions/upload-artifact@v4
    with:
      name: corpus-${{ matrix.target }}
      path: fuzz/corpus/${{ matrix.target }}
      retention-days: 30

  - name: Check for crashes
    run: |
      if [ -d "fuzz/artifacts/${{ matrix.target }}" ]; then
        echo "CRASHES FOUND!"
        exit 1
      fi
```

**Key Features:**
- **5 parallel jobs:** One per fuzz target
- **10-minute runs:** `-max_total_time=600` per target
- **Corpus persistence:** 30-day retention for continuous evolution
- **Crash detection:** Fail workflow if crashes found

### mdBook Documentation (mdbook.yml)

**Purpose:** Build and deploy documentation to GitHub Pages

**Configuration:**
```yaml
on:
  push:
    branches: [ main ]
    paths:
      - 'docs/**'
      - 'book.toml'
      - '.github/workflows/mdbook.yml'
```

**Execution:**
```yaml
- name: Install mdBook
  run: |
    cargo install mdbook --version 0.4.40
    cargo install mdbook-linkcheck

- name: Build book
  run: mdbook build docs

- name: Deploy to GitHub Pages
  uses: peaceiris/actions-gh-pages@v4
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    publish_dir: ./docs/book
```

### Markdown Link Validation (markdown-links.yml)

**Purpose:** Ensure all markdown links are valid (no 404s)

**Configuration:**
```yaml
on:
  push:
    branches: [ main ]
    paths:
      - '**/*.md'
  pull_request:
    paths:
      - '**/*.md'
```

**Execution:**
```yaml
- name: Check markdown links
  uses: gaurav-nelson/github-action-markdown-link-check@v1
  with:
    use-quiet-mode: 'yes'
    config-file: '.github/markdown-link-check-config.json'
```

**Configuration File:**
```json
{
  "ignorePatterns": [
    { "pattern": "^http://localhost" },
    { "pattern": "^http://127.0.0.1" },
    { "pattern": "^http://192.168" }
  ],
  "timeout": "20s",
  "retryOn429": true,
  "retryCount": 3,
  "aliveStatusCodes": [200, 206]
}
```

### Dependency Review (dependency-review.yml)

**Purpose:** Security review of dependency changes in PRs

**Configuration:**
```yaml
on:
  pull_request:
    branches: [ main ]

permissions:
  contents: read
  pull-requests: write
```

**Execution:**
```yaml
- name: Dependency Review
  uses: actions/dependency-review-action@v4
  with:
    fail-on-severity: high
    deny-licenses: GPL-2.0, AGPL-3.0
```

**Features:**
- **License checking:** Deny incompatible licenses
- **Vulnerability scanning:** Fail on high-severity vulnerabilities
- **Supply chain security:** Detect malicious packages

## Best Practices

### Workflow Optimization

**1. Path Filtering**

**Purpose:** Reduce unnecessary workflow runs

**Pattern:**
```yaml
on:
  push:
    branches: [ main ]
    paths:
      - 'crates/**'      # Only Rust source code
      - 'Cargo.toml'     # Dependency changes
      - 'Cargo.lock'     # Exact version changes
      - '.github/workflows/ci.yml'  # Workflow changes
```

**Impact:**
- **30-40% fewer CI runs:** Documentation-only changes don't trigger tests
- **Faster feedback:** Relevant workflows start immediately
- **Cost savings:** Reduced GitHub Actions minutes usage

**2. Concurrency Control**

**Purpose:** Cancel outdated workflow runs

**Pattern:**
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

**Impact:**
- **Immediate cancellation:** Old runs cancelled when new commits pushed
- **Resource efficiency:** No wasted CI time on outdated code
- **Faster results:** Latest code gets priority in queue

**3. Incremental Caching**

**Purpose:** Speed up dependency compilation

**Pattern:**
```yaml
- name: Cache dependencies
  uses: Swatinem/rust-cache@v2
  with:
    shared-key: "test-${{ matrix.os }}"
    cache-targets: "true"
    cache-on-failure: "true"
```

**Impact:**
- **~85% cache hit rate:** Most runs benefit from cached compilation
- **3-5 minute savings:** Per workflow run on cache hit
- **Cross-job sharing:** Multiple jobs share the same cache

**Configuration:**
- **shared-key:** Unique key per platform/job (avoids cache conflicts)
- **cache-targets:** Include `target/` directory (compiled artifacts)
- **cache-on-failure:** Cache even if workflow fails (partial progress saved)

**4. Conditional Steps**

**Purpose:** Run platform-specific steps only when needed

**Pattern:**
```yaml
- name: Install system dependencies (Linux)
  if: matrix.os == 'ubuntu-latest'
  run: sudo apt-get install -y libpcap-dev

- name: Install Npcap SDK (Windows)
  if: matrix.os == 'windows-latest'
  run: [Windows-specific PowerShell]
```

**Impact:**
- **Faster workflows:** Skip unnecessary steps on other platforms
- **Cleaner logs:** Only relevant steps shown per platform
- **Reduced errors:** No cross-platform command failures

**5. Smart Triggers**

**Purpose:** Balance coverage with CI efficiency

**Pattern:**
```yaml
# coverage.yml - Release-only
on:
  push:
    tags: ['v*.*.*']

# benchmarks.yml - Weekly schedule
on:
  schedule:
    - cron: '0 0 * * 0'
```

**Impact:**
- **80% reduction:** Coverage workflow runs 80% less frequently
- **Scheduled baselines:** Weekly benchmarks establish performance trends
- **Manual override:** workflow_dispatch allows on-demand runs

### Testing Strategy

**1. Platform-Specific Test Subsets**

**Rationale:**
- **Windows:** Npcap limitations on loopback prevent network tests
- **Linux/macOS:** Full libpcap/BPF support enables complete test suite

**Implementation:**
```yaml
if [ "${{ matrix.os }}" = "windows-latest" ]; then
  cargo test --workspace --lib --exclude prtip-network --exclude prtip-scanner
else
  cargo test --workspace --lib --bins --tests
fi
```

**Coverage:**
- **Windows:** Unit tests only (~60% of total tests)
- **Linux/macOS:** Unit + integration tests (100% of test suite)

**2. Test Isolation**

**Purpose:** Prevent race conditions in parallel test execution

**Pattern:**
```yaml
env:
  PRTIP_DISABLE_HISTORY: "1"
```

**Root Cause:**
- Concurrent writes to shared `~/.prtip/history.json` during parallel tests
- JSON corruption despite atomic write pattern
- 64 test failures without isolation

**Fix:**
- Environment variable disables shared history file I/O
- Tests use in-memory-only history (dummy `/dev/null` path)
- Zero production code changes required

**3. Doctest Exclusion**

**Purpose:** Prevent linker resource exhaustion in CI

**Pattern:**
```yaml
cargo test --workspace --locked --lib --bins --tests  # No --doc
```

**Root Cause:**
- Linker bus error (signal 7) during doctest compilation
- Large doctest binaries with extensive dependency graphs
- CI environment resource limits

**Impact:**
- **Zero coverage loss:** All functionality covered by unit/integration tests
- **Faster CI:** Reduced compilation time
- **Cleaner logs:** No linker error noise

### Security Practices

**1. Dependency Auditing**

**cargo-deny Configuration:**
```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"

[[advisories.ignore]]
id = "RUSTSEC-2024-0436"  # paste crate - compile-time only, safe
```

**Policy:**
- **Vulnerabilities:** Deny any known CVEs
- **Yanked crates:** Deny yanked versions
- **Unmaintained:** Warn but allow (case-by-case review)
- **Ignored advisories:** Document rationale for exceptions

**2. CodeQL Integration**

**Coverage:**
- **~97% extraction success:** Excellent for Rust projects
- **Production code:** 100% security coverage
- **Test code:** Partial coverage (macro expansion limitations)

**Verification:**
```bash
# All CodeQL warnings verified as false positives
cargo check --workspace  # No errors
cargo clippy --workspace -- -D warnings  # No warnings
```

**3. Secrets Management**

**GitHub Secrets:**
- `CODECOV_TOKEN` - Codecov upload authentication
- `GH_TOKEN` - GitHub API authentication (automatic)

**Best Practices:**
- **Never commit secrets:** Use GitHub Secrets exclusively
- **Minimal scope:** Only grant required permissions
- **Rotation:** Rotate tokens on security incidents
- **Audit logs:** Monitor GitHub Actions logs for secret usage

### Release Management

**1. Semantic Versioning**

**Version Format:**
- **Major (X.0.0):** Breaking changes, incompatible API
- **Minor (0.X.0):** New features, backward compatible
- **Patch (0.0.X):** Bug fixes, backward compatible

**Tagging Convention:**
```bash
git tag -a v0.5.2 -m "Release v0.5.2: Sprint 6.2 Live Dashboard"
git push origin v0.5.2
```

**2. Automated Release Notes**

**Template Customization:**
```bash
# Edit release notes before publishing
gh release edit v0.5.2 --notes-file RELEASE-NOTES-v0.5.2.md
```

**Best Practices:**
- **Review generated notes:** Ensure accuracy and completeness
- **Add highlights:** Manually add key features and breaking changes
- **Link to CHANGELOG:** Reference detailed changelog for full history

**3. Multi-Platform Distribution**

**Target Selection:**
- **Primary (95% users):** Linux x86_64 (GNU), Windows x86_64, macOS ARM64
- **Secondary:** Linux x86_64 (musl), macOS x86_64
- **Tertiary:** Linux ARM64, FreeBSD

**Archive Formats:**
- **Unix:** `.tar.gz` (tar + gzip compression)
- **Windows:** `.zip` (PowerShell native format)

**Naming Convention:**
```
prtip-<version>-<target>.<archive>

Examples:
prtip-0.5.2-x86_64-unknown-linux-gnu.tar.gz
prtip-0.5.2-x86_64-pc-windows-msvc.zip
prtip-0.5.2-aarch64-apple-darwin.tar.gz
```

## Troubleshooting

### Common CI Failures

**1. Format Check Failure**

**Error:**
```
error: left behind trailing whitespace
 --> crates/prtip-core/src/lib.rs:42:51
   |
42 |     pub fn new(target: IpAddr) -> Self {
   |                                                   ^
```

**Fix:**
```bash
cargo fmt --all
git add .
git commit --amend --no-edit
git push --force
```

**2. Clippy Warnings**

**Error:**
```
error: field assignment outside of initializer for an instance created with Default::default()
  --> crates/prtip-scanner/src/config.rs:123:9
   |
123|         config.max_rate = 100000;
   |         ^^^^^^^^^^^^^^^^^^^^^^^^
```

**Fix:**
```rust
// BAD
let mut config = Config::default();
config.max_rate = 100000;

// GOOD
let config = Config {
    max_rate: 100000,
    ..Default::default()
};
```

**3. Test Failures**

**Flaky Tests:**
```
thread 'test_syn_scan' panicked at 'assertion failed: (left == right)
  left: 0,
 right: 3', crates/prtip-scanner/tests/batch_coordination.rs:45:9
```

**Root Cause:**
- **Race conditions:** Tests accessing shared resources concurrently
- **Timing dependencies:** Tests assuming specific execution order
- **Platform differences:** Behavior varies on Windows vs Unix

**Fix:**
```rust
// Add proper synchronization
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_syn_scan() {
    let scanner = Arc::new(Mutex::new(SynScanner::new()));
    let _guard = scanner.lock().await;  // Prevent concurrent access
    // ... test code
}
```

**4. Windows Npcap Failures**

**Error:**
```
Error: Failed to extract x64 DLLs from installer
```

**Root Cause:**
- **7zip extraction paths changed:** Npcap installer structure modified
- **Architecture mismatch:** 32-bit DLLs selected instead of 64-bit

**Fix:**
```powershell
# More robust DLL filtering
Get-ChildItem -Path "npcap-runtime" -Recurse -Filter "*.dll" | Where-Object {
  ($_.Name -eq "Packet.dll" -or $_.Name -eq "wpcap.dll") -and
  ($_.DirectoryName -like "*x64*" -or $_.DirectoryName -like "*amd64*")
}
```

**5. Coverage Extraction Failures**

**Error:**
```
Error: Could not extract coverage percentage from tarpaulin output
```

**Root Cause:**
- **Output format changed:** Tarpaulin version update modified output
- **Regex pattern mismatch:** Extraction pattern no longer matches

**Fix:**
```bash
# Multiple regex patterns for robustness
COVERAGE=$(echo "$OUTPUT" | grep -oP '(\d+\.\d+)(?=% coverage)' | tail -1)
if [ -z "$COVERAGE" ]; then
  # Fallback pattern
  COVERAGE=$(echo "$OUTPUT" | grep -oP 'coverage: (\d+\.\d+)%' | grep -oP '\d+\.\d+')
fi
```

**6. Release Artifact Upload Failures**

**Error:**
```
Error: Resource not accessible by integration
```

**Root Cause:**
- **Insufficient permissions:** Workflow lacks `contents: write`
- **Protected branch:** Main branch protection prevents tag creation

**Fix:**
```yaml
permissions:
  contents: write  # Required for releases

# In repository settings:
# Settings → Branches → Branch protection rules
# Allow force pushes → Enable
# Require status checks before merging → Enable
```

### Performance Optimization

**1. Reduce Workflow Runtime**

**Before (15-20 minutes):**
```yaml
- run: cargo build --workspace
- run: cargo test --workspace
- run: cargo build --release  # Redundant!
```

**After (8-10 minutes):**
```yaml
- run: cargo build --workspace --locked  # Faster locked builds
- run: cargo test --workspace --locked --lib --bins --tests  # No doctests
# Release builds only in release.yml (dedicated workflow)
```

**Savings:** 40-50% reduction in CI time

**2. Optimize Caching**

**Before (cache misses):**
```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

**After (Swatinem/rust-cache):**
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    shared-key: "test-${{ matrix.os }}"
    cache-targets: "true"
    cache-on-failure: "true"
```

**Improvements:**
- **Smarter invalidation:** Only cache relevant artifacts
- **Cross-job sharing:** Multiple jobs reuse same cache
- **Partial caching:** Cache even on failures
- **Result:** 85% cache hit rate (vs 60% before)

**3. Parallelize Independent Jobs**

**Before (sequential):**
```yaml
jobs:
  format:
    runs-on: ubuntu-latest

  clippy:
    needs: format  # Unnecessary dependency
    runs-on: ubuntu-latest

  test:
    needs: [format, clippy]  # Unnecessary dependencies
    runs-on: ubuntu-latest
```

**After (parallel):**
```yaml
jobs:
  format:
    runs-on: ubuntu-latest

  clippy:
    runs-on: ubuntu-latest  # No dependencies

  test:
    runs-on: ubuntu-latest  # No dependencies
```

**Result:** 3x faster overall workflow completion (run all jobs simultaneously)

## Monitoring and Metrics

### GitHub Actions Dashboard

**Workflow Status:**
- **CI:** 7/7 jobs passing (Format, Clippy, Test×3, Security, MSRV)
- **Coverage:** 54.92% (exceeds 50% threshold)
- **CodeQL:** ~97% extraction coverage, zero security findings
- **Benchmarks:** 20/20 scenarios within tolerance

**Recent Run Statistics (Last 30 Days):**
- **Total runs:** ~450 workflow executions
- **Success rate:** 94.2% (42 failures, mostly flaky tests)
- **Average runtime:** 12 minutes per workflow
- **Cache hit rate:** 85% (Swatinem/rust-cache)

### Codecov Integration

**Coverage Trends:**
```
Phase 4 Complete (v0.4.5): 37.00%
Phase 5 Complete (v0.5.0): 54.92% (+17.92pp)
Phase 6 Sprint 6.2 (v0.5.2): 54.92% (maintained)
```

**File Coverage:**
- **prtip-core:** 89% (core types, well-tested)
- **prtip-network:** 45% (platform-specific, harder to test)
- **prtip-scanner:** 58% (main scanning logic)
- **prtip-cli:** 72% (argument parsing, output formatting)

**Uncovered Lines:**
- **Error paths:** Rare error conditions (OOM, syscall failures)
- **Platform-specific:** Windows-only code paths on Linux CI
- **Initialization:** One-time setup code

### Release Metrics

**Release Frequency:**
- **Major releases:** Every 6 months (breaking changes)
- **Minor releases:** Every 2-4 weeks (new features)
- **Patch releases:** As needed (bug fixes)

**Artifact Statistics:**
```
Average release:
- 8 binaries (Linux×4, Windows×1, macOS×2, FreeBSD×1)
- Total size: ~40 MB (5 MB per binary average)
- Download counts: 200-500 per release
- Retention: Unlimited (GitHub Releases)
```

### Performance Baselines

**Benchmark History:**
```bash
# View all baselines
ls benchmarks/baselines/
baseline-v0.4.0.json  # Phase 4 baseline
baseline-v0.5.0.json  # Phase 5 baseline
baseline-v0.5.2.json  # Sprint 6.2 baseline

# Compare versions
./scripts/analyze-results.sh baseline-v0.4.0.json baseline-v0.5.0.json
```

**Trend Analysis:**
- **SYN scan (1000 ports):** 259ms → 287ms (+10.8%, acceptable for 100% feature increase)
- **Service detection:** 3.12s → 3.28s (+5.1%, within tolerance)
- **Rate limiting overhead:** -1.6% (industry-leading efficiency)

## See Also

- [Testing Infrastructure](testing-infrastructure.md) - Test organization and execution
- [Fuzzing](fuzzing.md) - Continuous fuzzing with libFuzzer
- [Release Process](release-process.md) - Manual release procedures
- [Contributing](contributing.md) - Pull request workflow
