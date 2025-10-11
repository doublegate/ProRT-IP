Fast Rust quality check pipeline - format, lint, test, build verification: $*

---

## COMPREHENSIVE RUST QUALITY PIPELINE

**Purpose:** Execute complete Rust quality checks in optimized phases (fast-fail → comprehensive → build)

**Usage:** `/rust-check [options]` (runs all phases automatically)

**Phases:** 3 sequential phases with early termination on failure

---

## USAGE PATTERNS

**Basic:** `/rust-check` (full pipeline, all packages)
**Filtered:** `/rust-check --package prtip-scanner` (single package)
**Category:** `/rust-check quick` (fast checks only: format + clippy)
**Test Filter:** `/rust-check "tcp_connect"` (test pattern match)

---

## Phase 0: VALIDATE PREREQUISITES AND PARSE PARAMETERS

**Objective:** Ensure Rust toolchain is available and parse optional arguments

### Step 0.1: Check Rust Toolchain

```bash
if ! command -v cargo &> /dev/null; then
  echo "❌ ERROR: cargo not found"
  echo "Install Rust: https://rustup.rs"
  exit 1
fi

if [ ! -f "Cargo.toml" ]; then
  echo "❌ ERROR: Not in a Rust project (no Cargo.toml)"
  exit 1
fi
```

### Step 0.2: Check Required Tools

```bash
MISSING_TOOLS=()
! command -v rustfmt &> /dev/null && MISSING_TOOLS+=("rustfmt")
! command -v cargo-clippy &> /dev/null && MISSING_TOOLS+=("clippy")

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
  echo "⚠️  WARNING: Missing tools (install with rustup component add):"
  printf '  - %s\n' "${MISSING_TOOLS[@]}"
fi

echo "✅ Prerequisites validated"
echo ""
```

### Step 0.3: Parse Parameters and Determine Scope

```bash
ARGS="$*"
SCOPE="full"
PACKAGE=""
FILTER=""

if [ -z "$ARGS" ]; then
  SCOPE="full"
elif [[ "$ARGS" =~ ^prtip-(core|network|scanner|cli)$ ]]; then
  SCOPE="package"
  PACKAGE="$ARGS"
  echo "🎯 Scope: Single package ($PACKAGE)"
elif [ "$ARGS" = "quick" ]; then
  SCOPE="quick"
  echo "🎯 Scope: Quick checks (format + clippy only)"
elif [[ "$ARGS" =~ ^--package\ prtip- ]]; then
  SCOPE="package"
  PACKAGE=$(echo "$ARGS" | grep -oP 'prtip-\w+')
  echo "🎯 Scope: Single package ($PACKAGE)"
else
  SCOPE="filter"
  FILTER="$ARGS"
  echo "🎯 Scope: Test filter ($FILTER)"
fi

echo ""
```

---

## Phase 1: FAST-FAIL QUALITY CHECKS (Parallel)

**Objective:** Catch formatting and lint issues as quickly as possible before running expensive tests

### Step 1.1: Format Check (cargo fmt --check)

```bash
cargo fmt --all -- --check
```

**Expected:** Zero formatting issues
**On Failure:** Abort pipeline, display formatting violations

### Step 1.2: Clippy Lint (cargo clippy --all-targets)

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

**Expected:** Zero clippy warnings
**On Failure:** Abort pipeline, display lint violations

**Parallel Execution:** Run both cargo fmt and cargo clippy simultaneously for maximum speed

---

## Phase 2: COMPREHENSIVE TEST SUITE

**Objective:** Run test suite based on scope (full/package/filter)

**Note:** Skip this phase entirely if SCOPE="quick"

### Step 2.0: Check if Tests Should Run

```bash
if [ "$SCOPE" = "quick" ]; then
  echo "⏭️  Skipping tests (quick mode)"
  exit 0
fi
```

### Step 2.1: Execute Tests Based on Scope

```bash
if [ "$SCOPE" = "package" ]; then
  echo "Running tests for package: $PACKAGE"
  cargo test --package "$PACKAGE"

elif [ "$SCOPE" = "filter" ]; then
  echo "Running tests matching: $FILTER"
  cargo test --workspace "$FILTER"

else
  # Full test suite
  echo "Running full test suite (643 tests)"

  # Unit tests
  cargo test --workspace --lib

  # Integration tests
  cargo test --workspace --test '*'

  # Doc tests
  cargo test --workspace --doc
fi
```

**Expected:** All tests pass for the selected scope
**Coverage:** Core modules >90%, network >85%, scanner >85%, CLI >80%

**Full Suite:** 643 tests expected (100% pass rate)
**Duration:** ~5-6 minutes (full), ~10-60s (package), ~5-20s (filter)

---

## Phase 3: RELEASE BUILD VERIFICATION

**Objective:** Verify optimized release build compiles without warnings

### Step 3.1: Clean Release Build

```bash
cargo clean
cargo build --release --all-targets
```

**Expected:** Zero warnings, successful compilation
**Duration:** ~30-60 seconds (varies by system)

### Step 3.2: Binary Size Check

```bash
ls -lh target/release/prtip
```

**Expected:** ~3-5 MB binary (varies by platform)
**Note:** Significant size increase may indicate dependency bloat

---

## SUCCESS CRITERIA

✅ **All phases must pass:**
- Phase 1: Zero formatting issues, zero clippy warnings
- Phase 2: 643/643 tests passing (100% success rate)
- Phase 3: Release build successful, zero warnings

✅ **Performance Benchmarks:**
- Phase 1: <10 seconds (fast-fail)
- Phase 2: <6 minutes (full test suite)
- Phase 3: <60 seconds (release build)
- **Total:** <8 minutes end-to-end

---

## FAILURE HANDLING

### Format Failures (Phase 1.1)

**Action:** Run `cargo fmt --all` to auto-fix
**Report:** Display file paths with formatting violations

### Clippy Failures (Phase 1.2)

**Action:** Display clippy suggestions with fix hints
**Common Issues:**
- Unused variables (prefix with `_`)
- Unnecessary clones (use references)
- Inefficient algorithms (use clippy suggestions)

### Test Failures (Phase 2)

**Action:** Display failed test names and error messages
**Debug Command:** `cargo test --workspace -- --nocapture` (show stdout)
**Next Steps:**
- Isolate failing test: `cargo test test_name -- --exact`
- Run with logging: `RUST_LOG=debug cargo test test_name`

### Build Failures (Phase 3)

**Action:** Display compiler errors with line numbers
**Common Issues:**
- Type mismatches (check function signatures)
- Lifetime issues (review borrowing)
- Missing dependencies (check Cargo.toml)

---

## OPTIMIZATION NOTES

**Cargo Cache:** Phases use incremental compilation for speed
**Parallel Execution:** Phase 1 runs fmt + clippy in parallel
**Early Termination:** Pipeline aborts on first failure (no wasted time)

**CI/CD Integration:** This workflow matches GitHub Actions CI pipeline
- Format job (20-30s)
- Clippy job (40-60s)
- Test jobs (5-6 min)
- MSRV check (1-2 min)
- Security audit (30-40s)

---

## DELIVERABLES

**On Success:**
1. **Quality Report:** All checks passed, zero issues found
2. **Test Summary:** 643/643 tests passing, coverage maintained
3. **Build Artifacts:** Release binary in `target/release/prtip`
4. **Duration Report:** Total pipeline execution time

**On Failure:**
1. **Failure Report:** Which phase failed, specific errors
2. **Fix Commands:** Suggested commands to resolve issues
3. **Investigation Guide:** How to debug the specific failure type

---

## QUICK REFERENCE

**Full Pipeline:** `/rust-check`
**Format Only:** `cargo fmt --all`
**Lint Only:** `cargo clippy --all-targets -- -D warnings`
**Tests Only:** `cargo test --workspace`
**Build Only:** `cargo build --release`

**Pre-Commit:** Run `/rust-check` before every git commit
**Pre-Push:** Run `/rust-check` before every git push
**Pre-Release:** Run `/rust-check` + `/bench-compare` before version tags

---

## RELATED COMMANDS

- `/test-quick <pattern>` - Run specific test subsets (faster iteration)
- `/bench-compare <baseline> <comparison>` - Validate no performance regressions
- `/sprint-complete <sprint-id>` - Includes full rust-check as final validation
- `/ci-status` - Check if CI pipeline passed (same checks)
- `/bug-report <issue> <command>` - Generate bug report if checks fail

**Workflow Integration:**
- **Pre-commit:** `/rust-check` → Fix issues → Re-run
- **Pre-push:** `/rust-check` + `/bench-compare` → Ensure quality + performance
- **Pre-release:** `/rust-check` + full benchmark suite
- **Debugging:** `/rust-check` fails → `/test-quick <pattern>` → `/bug-report`

---

**Execute this comprehensive quality pipeline now.**
