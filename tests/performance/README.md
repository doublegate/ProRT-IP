# Performance Benchmarks

This directory contains performance benchmarks for ProRT-IP using Criterion.

## Running Benchmarks

From project root:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench benchmarks

# Generate detailed report
cargo bench --bench benchmarks -- --verbose
```

## Benchmark Categories

### Binary Startup
- Help flag latency
- Version flag latency

### Port Parsing
- Single port parsing
- Port range parsing
- Port list parsing

### Localhost Scanning
- Single port scan
- Multiple port scan (3 ports)
- Port range scan (10 ports)

### Output Formats
- Text output generation
- JSON output generation

## Interpreting Results

Criterion provides:
- Mean execution time
- Standard deviation
- Comparison with previous runs
- Regression detection

Look for:
- **Regressions**: >5% slower than previous run
- **High variance**: Standard deviation >10% of mean
- **Baseline comparison**: Track changes over versions

## Performance Targets

Based on `docs/07-PERFORMANCE.md`:

| Benchmark | Target | Current |
|-----------|--------|---------|
| Localhost 100 ports | <10ms | TBD |
| Localhost 1K ports | <100ms | TBD |
| Output generation | <1ms | TBD |

## Adding New Benchmarks

1. Add benchmark function to `benchmarks.rs`
2. Add to `criterion_group!` macro
3. Document target performance
4. Run and establish baseline

Example:

```rust
fn bench_new_feature(c: &mut Criterion) {
    c.bench_function("new_feature", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(result);
        });
    });
}
```

## CI Integration

Benchmarks can be run in CI with:

```yaml
- name: Run benchmarks
  run: cargo bench --no-fail-fast
```

For regression detection:
- Store baseline results
- Compare against baseline
- Fail if regression >10%

## Notes

- Benchmarks use `sample_size(10)` for scans (reduce CI time)
- Network benchmarks may have high variance
- Run on dedicated hardware for accurate results
- Localhost scans avoid network variability
