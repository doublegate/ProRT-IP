Fast Rust quality check pipeline - format, lint, test, build verification

---

## COMPREHENSIVE RUST QUALITY PIPELINE

**Purpose:** Execute complete Rust quality checks in optimized phases (fast-fail → comprehensive → build)

**Usage:** `/rust-check` (runs all phases automatically)

**Phases:** 3 sequential phases with early termination on failure

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

**Objective:** Run full test suite (643 tests) to validate functionality

### Step 2.1: Unit Tests (all packages)

```bash
cargo test --workspace --lib
```

**Expected:** All unit tests pass
**Coverage:** Core modules >90%, network >85%, scanner >85%, CLI >80%

### Step 2.2: Integration Tests

```bash
cargo test --workspace --test '*'
```

**Expected:** All integration tests pass
**Key Tests:** TCP connect scanner, SYN scanner, UDP scanner, scheduler orchestration

### Step 2.3: Doc Tests

```bash
cargo test --workspace --doc
```

**Expected:** All documentation examples pass
**Purpose:** Validate code examples in documentation

**Total Tests:** 643 expected (100% pass rate)
**Duration:** ~5-6 minutes (varies by system)

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

**Execute this comprehensive quality pipeline now.**
