# Regression Strategy

This document describes the regression testing strategy used during Phase 4-5 transition.

## Overview

The regression strategy ensured that Phase 5 additions did not break Phase 4 functionality.

## Test Categories

### Category 1: Core Functionality

Tests that must never fail:

- TCP SYN scanning
- TCP Connect scanning
- Port state detection
- Basic output formats

### Category 2: Performance Critical

Tests with performance requirements:

- Packet throughput benchmarks
- Memory usage limits
- Response time constraints

### Category 3: Feature Tests

Tests for specific features:

- Evasion techniques
- PCAPNG output
- Service detection

## Regression Detection

### Automated Checks

```bash
# Run regression suite
cargo test --features regression

# Check performance baseline
hyperfine --warmup 2 './target/release/prtip -sS -F localhost'
```

### Performance Baseline

| Operation | Phase 4 Baseline | Tolerance |
|-----------|------------------|-----------|
| SYN scan 1K ports | 250ms | +10% |
| Connect scan 100 ports | 500ms | +10% |
| Memory baseline | 80MB | +20% |

## Regression Response

### If Regression Detected

1. Identify failing tests
2. Bisect to find cause
3. Fix or document intentional change
4. Update baseline if appropriate

### Documentation

All intentional regressions documented with:

- Reason for change
- Performance impact
- Migration guidance

## Results

Phase 4 to Phase 5 transition:

- 0 core functionality regressions
- +10.8% performance regression (documented, justified by new features)
- All 1,166 Phase 4 tests continue passing

## See Also

- [Testing Strategy](../../development/testing.md)
- [Performance Tuning](../../advanced/performance-tuning.md)
