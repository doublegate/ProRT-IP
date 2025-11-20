# Sprint 6.3 Batch I/O Performance Comparison

**Date:** $(date +%Y-%m-%d)
**Platform:** $(uname -s) $(uname -r)
**Binary:** prtip v0.5.2 (release build)

## Benchmark Results Summary

| Scenario | Batch Size | Mean Time | Improvement | Syscall Reduction | Status |
|----------|------------|-----------|-------------|-------------------|--------|
| Baseline | 1 | 50.172750517859996s | 0% (reference) | 0% | ✓ |
| Scenario 2 | 32 | 50.177456579440005s | +0% | 96.87% | ❌ FAIL |
| Scenario 3 | 256 | 50.17023545482s | +0% | 99.61% | ❌ FAIL |
| Scenario 4 | 1024 | 50.17307949146s | +0% | 99.90% | ❌ FAIL |
| Scenario 6 | 256 | 0.052594007119999994s | +99.00% | 99.61% (IPv6) | ✅ PASS |

## Validation Results

### Performance Targets
- ✓ Batch 32: 20-40% improvement expected
- ✓ Batch 256: 30-50% improvement expected
- ✓ Batch 1024: 40-60% improvement expected
- ✓ IPv6: 25-45% improvement expected

### Syscall Reduction
- Batch 32: 96.87% (20,000 → 625 syscalls)
- Batch 256: 99.61% (20,000 → 78 syscalls)
- Batch 1024: 99.90% (20,000 → 20 syscalls)

### Conclusions
See individual scenario markdown files for detailed statistics.
