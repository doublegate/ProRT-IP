# Code Coverage

This directory contains code coverage configuration and reports for ProRT-IP.

## Prerequisites

Install cargo-tarpaulin:

```bash
cargo install cargo-tarpaulin
```

**Platform Support:**
- Linux: Full support ✅
- macOS: Limited support (ptrace restrictions)
- Windows: Limited support (requires WSL or Docker)

For macOS/Windows, consider using Docker:
```bash
docker run --security-opt seccomp=unconfined \
  -v "${PWD}:/volume" \
  xd009642/tarpaulin:latest
```

## Generate Coverage Report

From project root:

```bash
# Quick coverage (HTML only)
cargo tarpaulin --out Html --output-dir code_cov

# Full coverage (HTML + Lcov)
cargo tarpaulin --config code_cov/tarpaulin.toml

# Coverage for specific package
cargo tarpaulin -p prtip-core --out Html --output-dir code_cov

# Coverage with verbose output
cargo tarpaulin --config code_cov/tarpaulin.toml --verbose
```

## View Report

Open HTML report in browser:

```bash
# Linux
xdg-open code_cov/tarpaulin-report.html

# macOS
open code_cov/tarpaulin-report.html

# Windows (WSL)
explorer.exe code_cov/tarpaulin-report.html
```

## Coverage Threshold

**Current threshold:** 70% (configured in `tarpaulin.toml`)

**Target coverage by component:**

| Component | Current | Target | Priority |
|-----------|---------|--------|----------|
| prtip-core | TBD | 90% | HIGH |
| prtip-network | TBD | 85% | HIGH |
| prtip-scanner | TBD | 80% | HIGH |
| prtip-cli | TBD | 70% | MEDIUM |
| Overall | TBD | 80% | HIGH |

## Understanding Coverage

**Good coverage indicators:**
- Core business logic: >90%
- Network code: >85%
- Error handling: >80%
- CLI code: >70%

**Acceptable low coverage:**
- Platform-specific code (hard to test all platforms)
- Error recovery paths (hard to trigger)
- Debug/development code

## Improving Coverage

1. **Identify gaps:**
   ```bash
   cargo tarpaulin --out Html --output-dir code_cov
   # Open report, look for red (uncovered) lines
   ```

2. **Add tests:**
   - Unit tests in `src/` files
   - Integration tests in `crates/*/tests/`
   - End-to-end tests in `tests/`

3. **Focus on critical paths:**
   - Packet parsing (security-critical)
   - Port parsing (input validation)
   - Scan logic (core functionality)

4. **Verify improvement:**
   ```bash
   cargo tarpaulin --out Html --output-dir code_cov
   # Compare with previous report
   ```

## CI Integration

### GitHub Actions

Add to `.github/workflows/test.yml`:

```yaml
- name: Install tarpaulin
  run: cargo install cargo-tarpaulin

- name: Generate coverage
  run: cargo tarpaulin --config code_cov/tarpaulin.toml --verbose

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: ./code_cov/lcov.info
    fail_ci_if_error: true
```

### Codecov.io

Upload coverage to Codecov:

```bash
# Install codecov CLI
curl -Os https://uploader.codecov.io/latest/linux/codecov
chmod +x codecov

# Upload coverage
./codecov -f code_cov/lcov.info
```

### Coveralls.io

Upload coverage to Coveralls:

```bash
# Install coveralls
cargo install cargo-coveralls

# Upload coverage
cargo coveralls --lcov --output-path code_cov/lcov.info
```

## Troubleshooting

### High memory usage

Reduce parallelism:
```bash
cargo tarpaulin --jobs 1 --config code_cov/tarpaulin.toml
```

### Timeout errors

Increase timeout in `tarpaulin.toml`:
```toml
timeout = 600  # 10 minutes
```

### Missing coverage for tests

Tarpaulin doesn't count coverage in test files by default (correct behavior).
Coverage measures production code, not test code.

### Platform-specific issues

**macOS ptrace error:**
```bash
# Use Docker instead
docker run --security-opt seccomp=unconfined \
  -v "${PWD}:/volume" \
  xd009642/tarpaulin:latest
```

**Windows:**
```bash
# Use WSL2 or Docker
wsl cargo tarpaulin --config code_cov/tarpaulin.toml
```

## Files Generated

| File | Description | Use |
|------|-------------|-----|
| `tarpaulin-report.html` | Visual coverage report | Local viewing |
| `lcov.info` | Lcov format | CI integration |
| `cobertura.xml` | Cobertura format | Jenkins/CI |

## Exclusions

Coverage excludes:
- Test files (`tests/**`, `*/tests/**`)
- Benchmark files (`*/benches/**`)
- Binary entry points (`*/src/bin/**`)

To exclude specific lines in code:
```rust
#[cfg(not(tarpaulin_include))]
fn debug_function() {
    // This won't be counted in coverage
}
```

## Resources

- [Tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [Codecov Documentation](https://docs.codecov.io/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [ProRT-IP Testing Docs](../docs/06-TESTING.md)
