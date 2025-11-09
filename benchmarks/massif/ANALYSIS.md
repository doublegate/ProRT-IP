# Memory Profiling Analysis (Valgrind Massif)

**Sprint:** 5.5.4 - Performance Audit & Optimization
**Created:** 2025-11-09
**Status:** Framework Ready (Profiling Deferred)

---

## Overview

This directory contains heap memory profiles generated using `valgrind --tool=massif`.

**Purpose:**

- Validate memory usage claims (<1MB stateless, <100MB/10K hosts)
- Detect memory leaks (should be zero with Rust ownership)
- Identify allocation hotspots

---

## Planned Memory Profiling Scenarios

### 1. Stateless Scan (1K ports) (`stateless-1k.out`)

**Command:**
```bash
valgrind --tool=massif \
    --massif-out-file=benchmarks/massif/stateless-1k.out \
    target/release/prtip -sS -p 1-1000 127.0.0.1
```

**Generate Report:**
```bash
ms_print benchmarks/massif/stateless-1k.out > benchmarks/massif/stateless-1k-report.txt
```

**Target:** <1MB heap (claim to validate)

**Expected Allocations:**

- Packet buffer pool: ~500KB (pre-allocated)
- Target state: ~200KB (1 host, minimal)
- Runtime overhead: ~300KB

### 2. Stateful Scan (10K hosts) (`stateful-10k.out`)

**Command:**
```bash
valgrind --tool=massif \
    --massif-out-file=benchmarks/massif/stateful-10k.out \
    target/release/prtip -sS -p 1-1000 127.0.0.0/22
```

**Target:** <100MB heap (10,240 hosts × ~9.2KB/host)

**Expected Allocations:**

- Target state: ~95MB (10K hosts)
- Connection tracking: ~3MB
- Runtime overhead: ~2MB

### 3. Service Detection (`service-detection.out`)

**Command:**
```bash
valgrind --tool=massif \
    --massif-out-file=benchmarks/massif/service-detection.out \
    target/release/prtip -sV -p 22,80,443 127.0.0.1
```

**Target:** <10MB overhead (probe database + regex compilation)

**Expected Allocations:**

- Probe database: ~2.8MB (187 probes)
- Compiled regexes: ~4.5MB
- Banner buffers: ~1.5MB

### 4. OS Fingerprinting (`os-fingerprinting.out`)

**Command:**
```bash
valgrind --tool=massif \
    --massif-out-file=benchmarks/massif/os-fingerprinting.out \
    target/release/prtip -O -p 80 127.0.0.1
```

**Target:** <5MB overhead (signature database)

**Expected Allocations:**

- Signature database: ~4.5MB (2,600+ signatures)
- Probe state: ~500KB

### 5. Memory Leak Check (`leak-check.txt`)

**Command:**
```bash
valgrind --leak-check=full \
    --show-leak-kinds=all \
    target/release/prtip -sS -p 1-1000 127.0.0.1 \
    &> benchmarks/massif/leak-check.txt
```

**Target:** 0 definitely lost, 0 possibly lost (Rust safety guarantee)

**Expected Output:**

```
LEAK SUMMARY:
   definitely lost: 0 bytes in 0 blocks
   indirectly lost: 0 bytes in 0 blocks
     possibly lost: 0 bytes in 0 blocks
   still reachable: X bytes in Y blocks
        suppressed: 0 bytes in 0 blocks
```

**Note:** "still reachable" is acceptable (global static allocations)

---

## Analysis Methodology

### Memory Profiling Workflow

1. **Run massif:** Generate `.out` file
2. **Generate report:** `ms_print *.out > report.txt`
3. **Analyze peaks:** Identify maximum heap usage
4. **Identify allocators:** Find allocation call stacks
5. **Validate claims:** Compare against documented targets

### Massif Output Interpretation

**Example Massif Report:**

```
--------------------------------------------------------------------------------
  n        time(B)         total(B)   useful-heap(B) extra-heap(B)    stacks(B)
--------------------------------------------------------------------------------
  0              0                0                0             0            0
  1      1,024,000        1,024,000          975,000        49,000            0
  2      2,048,000        2,048,000        1,950,000        98,000            0
  ...
 50     95,000,000       95,000,000       90,250,000     4,750,000            0
```

**Key Metrics:**

- **total(B):** Total heap allocated
- **useful-heap(B):** User-requested memory
- **extra-heap(B):** Allocator overhead (~5%)
- **Peak:** Maximum total(B) across all snapshots

### Allocation Hotspot Example

**Hypothetical Allocation Trace:**

```
100.00% (95,000,000B) (heap allocation functions) malloc/new/new[], --alloc-fns, etc.
->90.00% (85,500,000B) std::vec::Vec::reserve
  ->70.00% (66,500,000B) prtip_scanner::target_state::TargetState::new
    ->60.00% (57,000,000B) prtip_scanner::batch_processor::process_batch
      ->50.00% (47,500,000B) prtip_scanner::scan_engine::scan_targets
->10.00% (9,500,000B) std::collections::hash::map::HashMap::insert
  ->8.00% (7,600,000B) prtip_core::service_detection::probe_database::load_probes
```

**Interpretation:**

- 90% of memory: Vec allocations (target state)
- 10% of memory: HashMap (probe database)
- Optimization opportunity: Pre-allocate Vec capacity

---

## Memory Optimization Tracking

### Implemented Optimizations (Sprint 5.5.4)

**None Yet (Framework Ready)**

When memory profiling is performed and optimizations implemented, document here:

| Optimization | Before | After | Savings | Status |
|--------------|--------|-------|---------|--------|
| TBD | TBD | TBD | TBD | Pending |

---

## Per-Target Memory Overhead

### Measured Overhead (Hypothetical)

| Scan Type | Per-Target Memory | Notes |
|-----------|------------------|-------|
| Stateless (SYN) | ~9.2 KB | Target state tracking |
| Stateful (Connect) | ~18 KB | Connection pool overhead |
| Service Detection | ~50 KB | Banner buffer, probe history |
| OS Fingerprinting | ~30 KB | 16-probe state |

### Memory Scaling

**Formula:**

```
Total Memory = Baseline + (Hosts × Per_Target_Overhead)
```

**Examples:**

| Hosts | Scan Type | Memory Estimate |
|-------|-----------|-----------------|
| 100 | SYN | 2.1 MB + (100 × 9.2 KB) = ~3 MB |
| 10,000 | SYN | 2.1 MB + (10K × 9.2 KB) = ~94 MB |
| 100,000 | SYN | 2.1 MB + (100K × 9.2 KB) = ~920 MB |

---

## Memory Leak Detection

### Rust Ownership Guarantees

**Expected Behavior:**

- **No memory leaks:** Rust ownership system prevents leaks
- **No use-after-free:** Borrow checker ensures safety
- **No double-free:** Drop trait guarantees single deallocation

**Exceptions (Acceptable):**

1. **Static allocations:** Global variables (still reachable)
2. **External libraries:** OpenSSL, pcap (suppressed)
3. **Intentional leaks:** `Box::leak()` for long-lived data (rare)

### Leak Analysis Workflow

1. Run leak check: `valgrind --leak-check=full`
2. Review "definitely lost" (should be 0)
3. Review "possibly lost" (should be 0)
4. Ignore "still reachable" (global statics)
5. Investigate any leaks (shouldn't happen in Rust)

---

## Future Work

### Additional Memory Scenarios

1. **Plugin System:** Lua VM memory footprint per plugin
2. **Event Logging:** Memory buffer overhead
3. **Large Result Sets:** Memory for 1M+ scan results
4. **Long-Running Scans:** Memory growth over time (leak test)

### Tools to Explore

- **heaptrack:** More detailed allocation tracking (Linux)
- **Instruments:** macOS memory profiling
- **cargo-bloat:** Binary size analysis (code size vs runtime memory)

---

## Notes

- Massif slows execution ~20-50x (use small scans)
- Peak memory may be transient (check snapshots)
- Release builds: Different allocation patterns than debug

---

**Last Updated:** 2025-11-09
**Next Update:** After memory profiling execution (deferred to future sprint)
