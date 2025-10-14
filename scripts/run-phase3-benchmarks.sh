#!/bin/bash
# Sprint 4.17 Phase 3 - Benchmark Runner
#
# Runs comprehensive performance benchmarks to validate zero-copy improvements:
# 1. Packet crafting benchmarks (old API vs zero-copy)
# 2. Batch I/O benchmarks (from Phase 1)
#
# Results saved to target/criterion/ for HTML reports and comparison.

set -e

echo "=== Sprint 4.17 Phase 3 Benchmarks ==="
echo ""
echo "This will run comprehensive performance benchmarks to validate"
echo "zero-copy packet crafting improvements."
echo ""

# 1. Packet crafting benchmarks (NEW in Phase 3)
echo "1. Running packet crafting benchmarks..."
echo "   - Old API (with allocations)"
echo "   - Zero-copy API (no allocations)"
echo "   - Throughput tests (1K packets)"
echo "   - With TCP options (realistic SYN)"
echo ""

cargo bench --bench packet_crafting -- --save-baseline phase3

echo ""
echo "✅ Packet crafting benchmarks complete"
echo ""

# 2. Batch I/O benchmarks (from Phase 1)
echo "2. Running batch I/O benchmarks..."
echo "   - Sendmmsg batching"
echo "   - Recvmmsg batching"
echo "   - Syscall reduction analysis"
echo ""

cargo bench --bench batch_io -- --save-baseline phase3

echo ""
echo "✅ Batch I/O benchmarks complete"
echo ""

# Summary
echo "============================================"
echo "Benchmarks Complete!"
echo "============================================"
echo ""
echo "Results saved to:"
echo "  - target/criterion/            (raw data)"
echo "  - target/criterion/reports/    (HTML reports)"
echo ""
echo "View HTML reports:"
echo "  firefox target/criterion/report/index.html"
echo ""
echo "Compare baselines:"
echo "  cargo bench --bench packet_crafting -- --baseline phase3"
echo ""
echo "Key metrics to check:"
echo "  - Old API: ~5µs per packet (baseline)"
echo "  - Zero-copy: ~800ns per packet (target)"
echo "  - Speedup: 5-6x improvement expected"
