# Sprint 5.5.4: Performance Audit & Optimization - TODO

**Sprint:** 5.5.4 - Performance Audit & Optimization
**Phase:** 5.5 - Pre-TUI Enhancements
**Version Target:** v0.5.1
**Priority:** MEDIUM (establishes baseline for TUI comparison)
**ROI Score:** 7.0/10 (Medium impact, moderate effort)
**Dependencies:** None (can run parallel with other sprints)
**Duration:** 24-32 hours estimated (3-4 days)
**Created:** 2025-11-09
**Status:** PENDING

---

## Executive Summary

### Objective

Establish comprehensive performance baselines for all scan types before Phase 6 TUI development. Identify and fix performance regressions, optimize hot paths, and create regression detection suite for CI/CD.

### Rationale

Phase 5 added significant functionality (IPv6, Service Detection, Idle Scan, TLS Analysis, Plugins, Event System, CLI Enhancements) but performance impacts were not systematically measured. Before TUI (Phase 6) adds UI overhead, we need:

1. **Baseline Metrics:** Know current performance for future comparison
2. **Regression Detection:** Catch performance degradation early
3. **Optimization Opportunities:** Low-hanging fruit before TUI
4. **Documentation:** Performance characteristics for capacity planning

**Current State (Sprint 5.9):**
- ✅ Benchmarking framework exists (hyperfine, 8 scenarios)
- ✅ Documentation complete (`docs/31-BENCHMARKING-GUIDE.md`, 1,044 lines)
- ❌ No continuous benchmarking (no CI integration for performance)
- ❌ No regression detection (manual comparison only)
- ❌ Limited scenarios (8 → need 20+ for comprehensive coverage)
- ❌ No profiling data (flamegraphs, memory analysis)
- ❌ Informal performance claims (not validated recently)

**Why Before Phase 6:**
TUI will add rendering overhead (50-100ms per frame). Without baseline, we won't know if slowdowns are TUI-related or underlying regression. Fixing performance after TUI is harder (more variables).

**Impact of Deferring:**
- No performance baseline (can't prove TUI didn't degrade speed)
- Regressions undetected (gradual performance loss)
- Missed optimization (quick wins left on table)
- Capacity planning unclear (users don't know scaling limits)

### Strategic Value

**Phase 5.5 Context:**
- Sprint 5.5.1: Documentation & Examples (COMPLETE, 21.1h, A+)
- Sprint 5.5.2: CLI Usability & UX (COMPLETE, 15.5h, A+)
- Sprint 5.5.3: Event System & Progress (COMPLETE, 18h+, A+)
- **Sprint 5.5.4: Performance Audit (THIS SPRINT)**
- Sprint 5.5.5: Configuration & State (PENDING)
- Sprint 5.5.6: Integration & Testing (PENDING)

**Why This Sprint Matters:**
- Validates Phase 5 performance claims (10M+ pps, -1.8% rate limiting, etc.)
- Establishes baseline before Phase 6 TUI (critical for comparison)
- Identifies optimization opportunities (quick wins, low effort)
- Creates continuous performance culture (regression detection in CI)
- Professional engineering rigor (not "it feels fast" but "measured fast")

---

## Task Areas Overview

| Task Area | Tasks | Hours | Priority | Dependencies | Status |
|-----------|-------|-------|----------|--------------|--------|
| 1. Comprehensive Benchmarking | 20 | 6-8h | HIGH | None | PENDING |
| 2. Profile Hot Paths | 15 | 5-6h | HIGH | None | PENDING |
| 3. Optimize Bottlenecks | 12 | 6-8h | MEDIUM | Task 2 | PENDING |
| 4. Regression Detection | 10 | 4-5h | HIGH | Task 1 | PENDING |
| 5. Performance Documentation | 8 | 4-5h | MEDIUM | All above | PENDING |
| 6. Results Publishing | 6 | 2-3h | LOW | All above | PENDING |
| **TOTAL** | **71** | **27-35h** | - | - | **0% Complete** |

**Note:** Estimate is 27-35h (worst-case), but with efficiency and focus, likely 24-30h actual.

---

## Task Area 1: Comprehensive Benchmarking (20 tasks, 6-8 hours)

**Goal:** Expand benchmark suite from 8 to 20+ scenarios for comprehensive coverage.

**Current State:**
- Sprint 5.9 created 8 baseline scenarios
- Framework operational (`benchmarks/05-Sprint5.9-Benchmarking-Framework/`)
- hyperfine integrated, JSON export working
- Documentation complete (`docs/31-BENCHMARKING-GUIDE.md`)

**Deliverable:** 12+ new benchmark scenarios (total 20+) covering all major features.

### Subtask 1.1: Expand Scan Type Scenarios (6 tasks, 2h)

**Goal:** Add missing scan type variations to achieve comprehensive coverage.

- [ ] **Task 1.1.1:** Create `09-fin-scan-stealth.sh`
  - **Command:** `prtip -sF -p 1-1000 127.0.0.1 --rate-limit 0`
  - **Purpose:** Validate FIN scan performance (stealth scan type)
  - **Target:** <120ms (slightly slower than SYN due to kernel handling)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/09-fin-scan-stealth.sh`

- [ ] **Task 1.1.2:** Create `10-null-scan-stealth.sh`
  - **Command:** `prtip -sN -p 1-1000 127.0.0.1 --rate-limit 0`
  - **Purpose:** Validate NULL scan performance (stealth scan type)
  - **Target:** <120ms (similar to FIN scan)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/10-null-scan-stealth.sh`

- [ ] **Task 1.1.3:** Create `11-xmas-scan-stealth.sh`
  - **Command:** `prtip -sX -p 1-1000 127.0.0.1 --rate-limit 0`
  - **Purpose:** Validate Xmas scan performance (stealth scan type)
  - **Target:** <120ms (similar to FIN/NULL scans)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/11-xmas-scan-stealth.sh`

- [ ] **Task 1.1.4:** Create `12-ack-scan-firewall-detection.sh`
  - **Command:** `prtip -sA -p 1-1000 127.0.0.1 --rate-limit 0`
  - **Purpose:** Validate ACK scan performance (firewall detection)
  - **Target:** <110ms (similar to SYN, no connection tracking)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/12-ack-scan-firewall-detection.sh`

- [ ] **Task 1.1.5:** Create `13-window-scan.sh` (if implemented)
  - **Command:** `prtip -sW -p 1-1000 127.0.0.1 --rate-limit 0` (check if implemented)
  - **Purpose:** Validate Window scan performance
  - **Target:** <110ms
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/13-window-scan.sh`
  - **Note:** Skip if not implemented, document as future enhancement

- [ ] **Task 1.1.6:** Create `14-maimon-scan.sh` (if implemented)
  - **Command:** `prtip -sM -p 1-1000 127.0.0.1 --rate-limit 0` (check if implemented)
  - **Purpose:** Validate Maimon scan performance
  - **Target:** <110ms
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/14-maimon-scan.sh`
  - **Note:** Skip if not implemented, document as future enhancement

**Acceptance Criteria:**
- All implemented scan types have benchmark scripts
- Scripts follow existing format (hyperfine, JSON export, metadata)
- Executable permissions set (`chmod +x`)
- Documented in `31-BENCHMARKING-GUIDE.md`

### Subtask 1.2: Scale Test Scenarios (6 tasks, 2h)

**Goal:** Add scale tests to validate performance across different workload sizes.

- [ ] **Task 1.2.1:** Create `15-small-scan-1host-100ports.sh`
  - **Command:** `prtip -sS -p 1-100 127.0.0.1`
  - **Purpose:** Small scan baseline (typical home network scan)
  - **Target:** <20ms
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/15-small-scan-1host-100ports.sh`

- [ ] **Task 1.2.2:** Create `16-medium-scan-100hosts-1000ports.sh`
  - **Command:** `prtip -sS -p 1-1000 127.0.0.0/25` (128 hosts, first 1000 ports)
  - **Purpose:** Medium scan (small office network)
  - **Target:** <10s (8000 total port checks)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/16-medium-scan-100hosts-1000ports.sh`
  - **Note:** May need network setup (loopback alias or test network)

- [ ] **Task 1.2.3:** Create `17-large-scan-1000hosts-100ports.sh`
  - **Command:** `prtip -sS -p 80,443,22,21,25,110,143,3389,3306,5432 127.0.0.0/22` (1024 hosts, 10 ports)
  - **Purpose:** Large scan (data center network)
  - **Target:** <30s (10,240 total port checks)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/17-large-scan-1000hosts-100ports.sh`
  - **Note:** May need network setup

- [ ] **Task 1.2.4:** Create `18-all-ports-single-host.sh`
  - **Command:** `prtip -sS -p- 127.0.0.1` (all 65535 ports)
  - **Purpose:** Maximum port coverage single host
  - **Target:** <5s (assuming ~13K pps)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/18-all-ports-single-host.sh`

- [ ] **Task 1.2.5:** Create `19-timing-t0-paranoid.sh`
  - **Command:** `prtip -sS -p 80,443,22 127.0.0.1 -T0`
  - **Purpose:** Slowest timing template (stealth mode)
  - **Target:** >5s (intentionally slow)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/19-timing-t0-paranoid.sh`

- [ ] **Task 1.2.6:** Create `20-timing-t5-insane.sh`
  - **Command:** `prtip -sS -p 1-1000 127.0.0.1 -T5 --rate-limit 0`
  - **Purpose:** Fastest timing template (speed mode)
  - **Target:** <80ms (faster than T3 default ~98ms)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/20-timing-t5-insane.sh`

**Acceptance Criteria:**
- 6 scale scenarios covering small/medium/large workloads
- Timing templates T0 and T5 validated
- Performance targets documented
- Scripts operational

### Subtask 1.3: Feature Overhead Scenarios (6 tasks, 1.5h)

**Goal:** Validate overhead of major Phase 5 features.

- [ ] **Task 1.3.1:** Create `21-os-fingerprinting-overhead.sh`
  - **Commands:**
    - Baseline: `prtip -sS -p 80 127.0.0.1`
    - OS detection: `prtip -sS -p 80 127.0.0.1 -O`
  - **Purpose:** Validate OS fingerprinting overhead (16-probe sequence)
  - **Metric:** Overhead = (os_time - baseline_time) / baseline_time * 100
  - **Target:** <30% overhead (16 probes + analysis)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/21-os-fingerprinting-overhead.sh`

- [ ] **Task 1.3.2:** Create `22-banner-grabbing-overhead.sh`
  - **Commands:**
    - Baseline: `prtip -sS -p 22,80,443 127.0.0.1`
    - Banner grab: `prtip -sT -p 22,80,443 127.0.0.1 --banner-grab`
  - **Purpose:** Validate banner grabbing overhead
  - **Metric:** Overhead percentage
  - **Target:** <15% overhead (3 banners, fast services)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/22-banner-grabbing-overhead.sh`

- [ ] **Task 1.3.3:** Create `23-evasion-fragmentation-overhead.sh`
  - **Commands:**
    - Baseline: `prtip -sS -p 1-1000 127.0.0.1`
    - Fragmentation: `prtip -sS -p 1-1000 127.0.0.1 -f`
  - **Purpose:** Validate packet fragmentation overhead
  - **Metric:** Overhead percentage
  - **Target:** <20% overhead (extra packet crafting)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/23-evasion-fragmentation-overhead.sh`

- [ ] **Task 1.3.4:** Create `24-evasion-decoys-overhead.sh`
  - **Commands:**
    - Baseline: `prtip -sS -p 1-1000 127.0.0.1`
    - Decoys: `prtip -sS -p 1-1000 127.0.0.1 -D 192.168.1.10,192.168.1.20,192.168.1.30`
  - **Purpose:** Validate decoy scanning overhead (3 decoys = 4x traffic)
  - **Metric:** Overhead percentage
  - **Target:** ~300% overhead (4x packets: 3 decoys + 1 real)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/24-evasion-decoys-overhead.sh`

- [ ] **Task 1.3.5:** Create `25-plugin-execution-overhead.sh`
  - **Commands:**
    - Baseline: `prtip -sS -p 80,443 127.0.0.1`
    - Plugin: `prtip -sS -p 80,443 127.0.0.1 --plugin example-banner-parser`
  - **Purpose:** Validate Lua plugin execution overhead (Sprint 5.8)
  - **Metric:** Overhead percentage
  - **Target:** <10% overhead (minimal Lua VM overhead)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/25-plugin-execution-overhead.sh`
  - **Note:** Requires example plugin from Sprint 5.8

- [ ] **Task 1.3.6:** Create `26-event-system-overhead.sh`
  - **Commands:**
    - Baseline: `prtip -sS -p 1-1000 127.0.0.1 --no-events` (if flag exists)
    - Events: `prtip -sS -p 1-1000 127.0.0.1`
  - **Purpose:** Validate event system overhead (Sprint 5.5.3)
  - **Metric:** Overhead percentage
  - **Target:** <5% overhead (minimal pub-sub overhead)
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/26-event-system-overhead.sh`
  - **Note:** May need to check if --no-events flag exists, or measure indirectly

**Acceptance Criteria:**
- All major Phase 5 features have overhead benchmarks
- Overhead percentages documented
- Targets validated or adjusted based on measurements

### Subtask 1.4: Benchmark Automation Enhancement (2 tasks, 0.5h)

**Goal:** Enhance `run-all-benchmarks.sh` to support new scenarios.

- [ ] **Task 1.4.1:** Update `run-all-benchmarks.sh` to include new scenarios
  - **Add to BENCHMARKS array:** Scripts 09-26 (18 new scenarios)
  - **Update scenario count:** 8 → 26 total
  - **Verify orchestration:** All scripts execute in sequence
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/run-all-benchmarks.sh`

- [ ] **Task 1.4.2:** Enhance statistical rigor
  - **Add warmup runs:** Ensure 3 warmup runs (already in hyperfine calls)
  - **Add outlier handling:** Already handled by hyperfine IQR method
  - **Document variance:** Add variance reporting to output
  - **Files:** `run-all-benchmarks.sh` (enhance output summary)

**Acceptance Criteria:**
- `run-all-benchmarks.sh` executes all 26 scenarios
- Statistical rigor validated (warmup, outlier removal)
- Output summary includes variance metrics

---

## Task Area 2: Profile Hot Paths (15 tasks, 5-6 hours)

**Goal:** Generate profiling data (CPU, Memory, I/O) to identify optimization opportunities.

**Current State:**
- No flamegraphs generated for Phase 5 code
- No memory profiling data
- No I/O profiling data
- Previous profiling from Phase 4 exists (`benchmarks/02-Phase4_Final-Bench/`)

**Deliverable:** Flamegraphs, massif reports, strace analysis for all major scan types.

### Subtask 2.1: CPU Profiling (5 tasks, 2-3h)

**Goal:** Generate flamegraphs to identify CPU hotspots.

- [ ] **Task 2.1.1:** Install profiling tools
  - **Install cargo-flamegraph:** `cargo install flamegraph`
  - **Verify perf availability:** `perf --version` (Linux)
  - **Note:** macOS uses Instruments (DTrace), Windows uses Event Tracing for Windows (ETW)
  - **Platform detection:** Document platform-specific setup

- [ ] **Task 2.1.2:** Generate flamegraph for SYN scan
  - **Command:** `cargo flamegraph --bin prtip -- -sS -p 1-10000 127.0.0.1 --rate-limit 0`
  - **Purpose:** Identify packet crafting hotspots
  - **Output:** `benchmarks/flamegraphs/syn-scan-10k-ports.svg`
  - **Analysis:** Look for functions consuming >5% CPU

- [ ] **Task 2.1.3:** Generate flamegraph for service detection
  - **Command:** `cargo flamegraph --bin prtip -- -sV -p 22,80,443,3306,5432 127.0.0.1`
  - **Purpose:** Identify regex matching overhead
  - **Output:** `benchmarks/flamegraphs/service-detection.svg`
  - **Analysis:** Check probe matching efficiency

- [ ] **Task 2.1.4:** Generate flamegraph for TLS parsing
  - **Command:** `cargo flamegraph --bin prtip -- -sV -p 443 badssl.com --tls-cert-analysis`
  - **Purpose:** Identify X.509 parsing overhead
  - **Output:** `benchmarks/flamegraphs/tls-cert-parsing.svg`
  - **Analysis:** Verify 1.33μs claim still holds

- [ ] **Task 2.1.5:** Generate flamegraph for plugin execution
  - **Command:** `cargo flamegraph --bin prtip -- -sS -p 80,443 127.0.0.1 --plugin example-banner-parser`
  - **Purpose:** Identify Lua VM overhead
  - **Output:** `benchmarks/flamegraphs/plugin-execution.svg`
  - **Analysis:** Check mlua overhead

**Acceptance Criteria:**
- 5 flamegraphs generated and saved
- Hotspots identified (functions >5% CPU documented)
- Analysis notes in `benchmarks/flamegraphs/ANALYSIS.md`

### Subtask 2.2: Memory Profiling (5 tasks, 2-3h)

**Goal:** Analyze heap allocations and memory usage patterns.

- [ ] **Task 2.2.1:** Install memory profiling tools
  - **Install valgrind:** `sudo apt install valgrind` (Linux)
  - **Alternative:** heaptrack on Linux, Instruments on macOS
  - **Verify:** `valgrind --version`

- [ ] **Task 2.2.2:** Profile stateless scan memory usage
  - **Command:** `valgrind --tool=massif --massif-out-file=benchmarks/massif/stateless-1k.out target/release/prtip -sS -p 1-1000 127.0.0.1`
  - **Generate report:** `ms_print benchmarks/massif/stateless-1k.out > benchmarks/massif/stateless-1k-report.txt`
  - **Target:** <1MB heap (verify Phase 4 claim)
  - **Analysis:** Check for unexpected allocations

- [ ] **Task 2.2.3:** Profile stateful scan memory usage
  - **Command:** `valgrind --tool=massif --massif-out-file=benchmarks/massif/stateful-10k.out target/release/prtip -sS -p 1-1000 127.0.0.0/22`
  - **Generate report:** `ms_print benchmarks/massif/stateful-10k.out > benchmarks/massif/stateful-10k-report.txt`
  - **Target:** <100MB for 10K hosts (verify claim)
  - **Analysis:** Check per-target memory overhead

- [ ] **Task 2.2.4:** Profile service detection memory usage
  - **Command:** `valgrind --tool=massif --massif-out-file=benchmarks/massif/service-detection.out target/release/prtip -sV -p 22,80,443 127.0.0.1`
  - **Generate report:** `ms_print benchmarks/massif/service-detection.out > benchmarks/massif/service-detection-report.txt`
  - **Target:** <10MB overhead (probe database + regex compilation)
  - **Analysis:** Check probe database memory footprint

- [ ] **Task 2.2.5:** Check for memory leaks
  - **Command:** `valgrind --leak-check=full --show-leak-kinds=all target/release/prtip -sS -p 1-1000 127.0.0.1`
  - **Target:** 0 definitely lost, 0 possibly lost (Rust safety guarantee)
  - **Output:** `benchmarks/massif/leak-check.txt`
  - **Note:** Should be zero leaks (Rust ownership system)

**Acceptance Criteria:**
- 5 massif reports generated
- Memory targets validated or updated
- Zero memory leaks confirmed
- Analysis documented in `benchmarks/massif/ANALYSIS.md`

### Subtask 2.3: I/O Profiling (5 tasks, 1h)

**Goal:** Analyze syscall overhead and I/O batching effectiveness.

- [ ] **Task 2.3.1:** Profile SYN scan syscalls
  - **Command:** `strace -c -o benchmarks/strace/syn-scan.txt target/release/prtip -sS -p 1-1000 127.0.0.1`
  - **Purpose:** Count syscalls (sendmmsg, recvmmsg, etc.)
  - **Analysis:** Check batching effectiveness

- [ ] **Task 2.3.2:** Profile service detection syscalls
  - **Command:** `strace -c -o benchmarks/strace/service-detection.txt target/release/prtip -sV -p 22,80,443 127.0.0.1`
  - **Purpose:** Count connection syscalls (connect, send, recv)
  - **Analysis:** Verify async I/O efficiency

- [ ] **Task 2.3.3:** Analyze sendmmsg batch sizes
  - **Command:** `strace -e sendmmsg -o benchmarks/strace/sendmmsg-detail.txt target/release/prtip -sS -p 1-1000 127.0.0.1`
  - **Purpose:** Verify batch sizes (should be >100 packets/batch)
  - **Analysis:** Check if batching is effective

- [ ] **Task 2.3.4:** Analyze file I/O patterns
  - **Command:** `strace -e write -o benchmarks/strace/file-io.txt target/release/prtip -sS -p 1-1000 127.0.0.1 -oN /tmp/output.txt`
  - **Purpose:** Check result writing efficiency
  - **Analysis:** Should be buffered writes (not per-result)

- [ ] **Task 2.3.5:** Document I/O analysis
  - **Create:** `benchmarks/strace/ANALYSIS.md`
  - **Include:** Syscall counts, batch sizes, optimization opportunities
  - **Recommendations:** Increase batch sizes if <100, async file I/O if blocking

**Acceptance Criteria:**
- 5 strace reports generated
- Syscall counts documented
- Batching effectiveness validated
- Analysis documented

---

## Task Area 3: Optimize Identified Bottlenecks (12 tasks, 6-8 hours)

**Goal:** Implement optimizations based on profiling data to achieve 10%+ speedup on 3+ scenarios.

**Approach:**
1. Prioritize by impact (CPU% × scenario frequency = priority)
2. Implement highest-impact optimizations first
3. Measure before/after for each optimization
4. Stop when 10%+ speedup achieved on 3+ scenarios

**Note:** Actual optimizations depend on profiling results. Tasks below are hypothetical examples based on typical bottlenecks. Adjust based on actual flamegraph/massif/strace data.

### Subtask 3.1: Prioritize Bottlenecks (2 tasks, 1h)

**Goal:** Create prioritized optimization list based on profiling data.

- [ ] **Task 3.1.1:** Analyze profiling data
  - **Read:** All flamegraphs, massif reports, strace analysis
  - **Identify:** Functions consuming >5% CPU
  - **List:** All bottlenecks with CPU percentage
  - **Document:** `benchmarks/OPTIMIZATION-CANDIDATES.md`

- [ ] **Task 3.1.2:** Calculate priority scores
  - **Formula:** Priority = (CPU %) × (Scenario Frequency) × (Optimization Ease)
  - **Scenario Frequency:** 1-10 (how common is this scenario?)
  - **Optimization Ease:** 1-10 (how easy to fix? 10 = trivial, 1 = major refactor)
  - **Sort:** Highest priority first
  - **Select:** Top 5-7 optimizations (within 6-8h budget)

**Acceptance Criteria:**
- Bottlenecks documented with CPU percentages
- Priority scores calculated
- Top 5-7 optimizations selected

### Subtask 3.2: Packet Crafting Optimizations (3 tasks, 2-3h)

**Hypothetical optimizations (adjust based on profiling):**

- [ ] **Task 3.2.1:** Pre-allocate packet buffers
  - **Issue:** Per-packet allocation overhead (if found in flamegraph)
  - **Fix:** Create buffer pool, reuse buffers
  - **Implementation:**
    ```rust
    // Before: Vec::new() per packet
    let mut buffer = Vec::with_capacity(1500);

    // After: Buffer pool
    struct BufferPool {
        pool: Vec<Vec<u8>>,
    }
    impl BufferPool {
        fn get(&mut self) -> Vec<u8> {
            self.pool.pop().unwrap_or_else(|| Vec::with_capacity(1500))
        }
        fn release(&mut self, mut buf: Vec<u8>) {
            buf.clear();
            self.pool.push(buf);
        }
    }
    ```
  - **Files:** `crates/prtip-scanner/src/packet_pool.rs` (new), update packet crafting code
  - **Measure:** Re-run SYN scan benchmark, expect 5-10% speedup

- [ ] **Task 3.2.2:** Batch checksum calculations
  - **Issue:** Per-packet checksum overhead (if found in flamegraph)
  - **Fix:** SIMD checksum calculation (use existing crates like `simd-checksum`)
  - **Implementation:** Replace manual checksum with SIMD version
  - **Files:** Update checksum calculations in packet crafting
  - **Measure:** Re-run SYN scan benchmark, expect 3-5% speedup

- [ ] **Task 3.2.3:** Optimize packet serialization
  - **Issue:** Inefficient serialization (if pnet overhead found)
  - **Fix:** Zero-copy serialization where possible
  - **Implementation:** Use MutablePacket::set_* methods efficiently
  - **Files:** Update packet crafting modules
  - **Measure:** Re-run SYN scan benchmark, expect 2-5% speedup

**Acceptance Criteria:**
- 3 packet crafting optimizations implemented (or skipped if not found in profiling)
- Before/after benchmarks documented
- Speedup percentage calculated

### Subtask 3.3: Service Detection Optimizations (3 tasks, 2-3h)

**Hypothetical optimizations:**

- [ ] **Task 3.3.1:** Compile regexes once (lazy_static)
  - **Issue:** Repeated regex compilation (if found in flamegraph)
  - **Fix:** Use `lazy_static` or `once_cell` to compile regexes once
  - **Implementation:**
    ```rust
    use once_cell::sync::Lazy;
    use regex::Regex;

    static HTTP_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"HTTP/\d\.\d").unwrap()
    });
    ```
  - **Files:** `crates/prtip-core/src/service_detection/probes.rs`
  - **Measure:** Re-run service detection benchmark, expect 10-15% speedup

- [ ] **Task 3.3.2:** Optimize regex patterns
  - **Issue:** Backtracking regexes (if found in profiling)
  - **Fix:** Use possessive quantifiers, avoid catastrophic backtracking
  - **Implementation:** Review and optimize regex patterns in probe database
  - **Files:** Service probe definitions
  - **Measure:** Re-run service detection benchmark, expect 5-10% speedup

- [ ] **Task 3.3.3:** Parallelize probe matching
  - **Issue:** Sequential probe matching (if latency found)
  - **Fix:** Use `rayon` for parallel regex matching
  - **Implementation:** Parallelize probe iteration
  - **Files:** Service detection matching logic
  - **Measure:** Re-run service detection benchmark, expect 15-20% speedup (if many probes)

**Acceptance Criteria:**
- 3 service detection optimizations implemented (or adjusted based on profiling)
- Speedup measured and documented
- No accuracy regression (still 85-90% detection rate)

### Subtask 3.4: I/O Optimizations (2 tasks, 1-2h)

**Hypothetical optimizations:**

- [ ] **Task 3.4.1:** Increase sendmmsg/recvmmsg batch sizes
  - **Issue:** Small batch sizes (if found in strace)
  - **Fix:** Increase batch size from current to 200+ packets
  - **Implementation:** Update batch size constants
  - **Files:** Packet I/O modules
  - **Measure:** Re-run SYN scan benchmark, expect 5-10% speedup

- [ ] **Task 3.4.2:** Async file writes
  - **Issue:** Blocking file writes (if found in strace)
  - **Fix:** Use Tokio async file I/O (`tokio::fs::File`)
  - **Implementation:** Replace std::fs with tokio::fs
  - **Files:** Output formatting modules
  - **Measure:** Re-run scan with file output, expect faster completion

**Acceptance Criteria:**
- I/O optimizations implemented if bottlenecks found
- Benchmarks show improvement
- No functionality regression

### Subtask 3.5: Rate Limiting Optimizations (2 tasks, 1h)

**Hypothetical optimizations (V3 is already -1.8%, but could improve further):**

- [ ] **Task 3.5.1:** Further optimize V3 algorithm
  - **Issue:** Can we improve beyond -1.8%?
  - **Fix:** Profile V3 specifically, look for micro-optimizations
  - **Implementation:** Review atomic operations, convergence math
  - **Files:** `crates/prtip-scanner/src/rate_limiter_v3.rs`
  - **Measure:** Re-run rate limiting overhead benchmark

- [ ] **Task 3.5.2:** Cache convergence calculations
  - **Issue:** Repeated convergence calculations
  - **Fix:** Cache convergence value when rate doesn't change
  - **Implementation:** Add convergence cache
  - **Measure:** Benchmark improvement

**Acceptance Criteria:**
- V3 rate limiter still ≤ -1.8% overhead (maintain or improve)
- Optimizations documented

---

## Task Area 4: Regression Detection Suite (10 tasks, 4-5 hours)

**Goal:** Implement CI/CD integration for automated performance regression detection.

**Deliverable:** GitHub Actions workflow, regression detection script, baseline management.

### Subtask 4.1: CI Workflow Creation (4 tasks, 2-3h)

**Goal:** Create `.github/workflows/benchmarks.yml` for automated benchmarking.

- [ ] **Task 4.1.1:** Create benchmark workflow file
  - **Create:** `.github/workflows/benchmarks.yml`
  - **Triggers:**
    - `workflow_dispatch` (manual runs)
    - `schedule`: Weekly (Sunday 00:00 UTC)
    - Optional: `push` to main (if not too slow)
    - Optional: `pull_request` (performance validation)
  - **Platform:** `ubuntu-latest` (consistent environment)
  - **Timeout:** 30 minutes max

- [ ] **Task 4.1.2:** Implement workflow steps
  - **Steps:**
    1. Checkout code (`actions/checkout@v4`)
    2. Setup Rust toolchain (`actions-rust-lang/setup-rust-toolchain@v1`)
    3. Cache dependencies (`actions/cache@v4` for cargo registry/build)
    4. Build release binary (`cargo build --release`)
    5. Install hyperfine (`cargo install hyperfine` or cache)
    6. Run benchmark suite (`./benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/run-all-benchmarks.sh`)
    7. Compare against baseline (`./scripts/analyze-results.sh`)
    8. Upload results as artifacts (`actions/upload-artifact@v4`, 7-day retention)
    9. Comment on PR (if PR context, using `actions/github-script@v7`)
    10. Fail if regression detected (exit code 2 from analyze-results.sh)

- [ ] **Task 4.1.3:** Implement PR comment template
  - **Template:**
    ```markdown
    ## Benchmark Results

    | Scenario | Baseline | Current | Diff | Status |
    |----------|----------|---------|------|--------|
    | SYN Scan | {{baseline_syn}}ms | {{current_syn}}ms | {{diff_syn}}% | {{status_syn}} |
    ...

    **Overall:** {{regression_count}} regressions, {{improvement_count}} improvements

    **Recommendation:** {{recommendation}}

    [View detailed report]({{artifact_url}})
    ```
  - **Script:** Generate markdown from analyze-results.sh output
  - **Files:** Update `scripts/analyze-results.sh` to output PR comment markdown

- [ ] **Task 4.1.4:** Test workflow locally
  - **Use:** `act` (https://github.com/nektos/act) to test GitHub Actions locally
  - **Install:** `brew install act` or equivalent
  - **Run:** `act workflow_dispatch -W .github/workflows/benchmarks.yml`
  - **Verify:** All steps execute, results uploaded

**Acceptance Criteria:**
- `.github/workflows/benchmarks.yml` created
- Workflow triggers configured (manual + weekly)
- All steps implemented and tested
- PR comment template working

### Subtask 4.2: Regression Detection Script (3 tasks, 1-2h)

**Goal:** Create robust `analyze-results.sh` for regression detection.

- [ ] **Task 4.2.1:** Implement comparison logic
  - **Input:** Two JSON files (baseline, current)
  - **Output:** Markdown table, exit code (0=pass, 1=warn, 2=fail)
  - **Logic:**
    ```bash
    # For each scenario in baseline:
    baseline_mean=$(jq '.results[0].mean' baseline.json)
    current_mean=$(jq '.results[0].mean' current.json)
    diff=$(( (current_mean - baseline_mean) / baseline_mean * 100 ))

    if [ $diff -lt -5 ]; then
        status="✅ IMPROVED"
    elif [ $diff -lt 5 ]; then
        status="✅ PASS"
    elif [ $diff -lt 10 ]; then
        status="⚠️ WARN"
        warn_count=$((warn_count + 1))
    else
        status="❌ FAIL"
        fail_count=$((fail_count + 1))
    fi
    ```
  - **Files:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/scripts/analyze-results.sh`

- [ ] **Task 4.2.2:** Add statistical significance test (optional, Python)
  - **Use:** scipy.stats.ttest_ind for t-test
  - **Implementation:**
    ```python
    from scipy.stats import ttest_ind
    import json

    with open('baseline.json') as f:
        baseline = json.load(f)
    with open('current.json') as f:
        current = json.load(f)

    baseline_times = baseline['results'][0]['times']
    current_times = current['results'][0]['times']

    t_stat, p_value = ttest_ind(baseline_times, current_times)

    if p_value < 0.05:
        print("Statistically significant difference")
    else:
        print("Within noise (accept)")
    ```
  - **Files:** `scripts/statistical-test.py` (optional enhancement)
  - **Note:** Skip if Python not available in CI

- [ ] **Task 4.2.3:** Generate PR comment markdown
  - **Output file:** `results/pr-comment.md`
  - **Format:** Use template from Task 4.1.3
  - **Include:** Summary table, recommendation, artifact link
  - **Files:** Update `analyze-results.sh`

**Acceptance Criteria:**
- `analyze-results.sh` compares baseline vs current
- Exit codes correct (0/1/2)
- PR comment markdown generated
- Statistical test implemented (optional)

### Subtask 4.3: Baseline Management (3 tasks, 1h)

**Goal:** Create baseline storage and update process.

- [ ] **Task 4.3.1:** Create baseline storage structure
  - **Directory:** `benchmarks/baselines/`
  - **Naming:** `baseline-v0.5.1.json` (version-tagged)
  - **Metadata:** `baseline-v0.5.1-metadata.md` (date, system info, commit SHA)
  - **Files:** Create directory structure

- [ ] **Task 4.3.2:** Implement baseline creation script
  - **Script:** `scripts/create-baseline.sh`
  - **Logic:**
    1. Run full benchmark suite
    2. Aggregate results into single JSON
    3. Save as `baselines/baseline-v<version>.json`
    4. Generate metadata file (date, system, commit)
  - **Usage:** `./scripts/create-baseline.sh v0.5.1`
  - **Files:** Create `scripts/create-baseline.sh`

- [ ] **Task 4.3.3:** Document baseline update process
  - **Documentation:** Update `docs/31-BENCHMARKING-GUIDE.md`
  - **Process:**
    1. Tag release: `git tag -a v0.5.1 -m "Release v0.5.1"`
    2. Build release: `cargo build --release`
    3. Create baseline: `./scripts/create-baseline.sh v0.5.1`
    4. Commit baseline: `git add benchmarks/baselines/ && git commit -m "chore: Add v0.5.1 baseline"`
  - **Files:** Update guide

**Acceptance Criteria:**
- Baseline directory structure created
- Baseline creation script operational
- Process documented in guide

---

## Task Area 5: Performance Documentation (8 tasks, 4-5 hours)

**Goal:** Create comprehensive performance documentation for users and developers.

**Deliverable:** `docs/34-PERFORMANCE-CHARACTERISTICS.md` (new guide).

### Subtask 5.1: Performance Guide Creation (8 tasks, 4-5h)

**Goal:** Document all performance characteristics based on benchmark data.

- [ ] **Task 5.1.1:** Create guide file structure
  - **Create:** `docs/34-PERFORMANCE-CHARACTERISTICS.md`
  - **Sections:**
    1. Overview
    2. Throughput Metrics
    3. Latency Metrics
    4. Memory Usage
    5. Scaling Characteristics
    6. Optimization Guide
    7. Capacity Planning
    8. Historical Performance
  - **Estimated length:** 1,500-2,000 lines

- [ ] **Task 5.1.2:** Document throughput section
  - **Include:**
    - Stateless scan: X pps (measured from benchmarks)
    - Stateful scan: Y pps (measured)
    - Service detection: Z hosts/min (measured)
    - OS fingerprinting: A hosts/min (measured)
  - **Format:**
    ```markdown
    ### Throughput Metrics

    **Stateless Scans (SYN/FIN/NULL/Xmas/ACK):**
    - Localhost: 10,200 pps (1,000 ports in 98ms)
    - LAN: 8,500 pps (network latency factor)
    - WAN: 1,000-5,000 pps (bandwidth constrained)

    **Stateful Scans (Connect):**
    - Localhost: 6,600 pps (1,000 ports in 150ms)
    - LAN: 5,000 pps
    - WAN: 500-2,000 pps

    **Service Detection:**
    - HTTP/SSH/HTTPS: 50-100 hosts/min
    - Complex services (MySQL, PostgreSQL): 20-40 hosts/min

    **OS Fingerprinting:**
    - 16-probe sequence: 30-50 hosts/min
    ```

- [ ] **Task 5.1.3:** Document latency section
  - **Include:**
    - Packet crafting: <1ms p99
    - Service regex: <5ms p99
    - TLS parsing: <2ms p99 (1.33μs mean)
    - Plugin execution: <10ms p99
  - **Format:** Table with p50/p95/p99 percentiles

- [ ] **Task 5.1.4:** Document memory section
  - **Include:**
    - Stateless scan: <1MB (verified from massif)
    - Stateful scan: <100MB for 10K hosts (verified)
    - Service detection: <10MB overhead (measured)
    - Plugin system: <5MB per Lua VM (measured)
  - **Format:** Table with scenario vs memory usage

- [ ] **Task 5.1.5:** Document scaling section
  - **Include:**
    - Small (1-100 hosts): Linear scaling
    - Medium (100-10K hosts): Linear with resource limits (file descriptors, memory)
    - Large (10K-1M hosts): Batch processing, streaming results, rate limiting critical
  - **Format:** Scaling charts (conceptual, or actual if data available)

- [ ] **Task 5.1.6:** Document optimization guide
  - **Include:**
    - Tuning ulimit (file descriptors): `ulimit -n 65535`
    - NUMA thread pinning (if applicable)
    - Batch size configuration (sendmmsg/recvmmsg)
    - Rate limiting tuning (--rate-limit flag)
  - **Format:** Step-by-step optimization checklist

- [ ] **Task 5.1.7:** Create capacity planning tables
  - **Include:**
    - "How many hosts can I scan?" (based on memory/network)
    - "How long will X scan take?" (based on throughput)
    - "What hardware do I need for Y throughput?" (CPU cores, RAM, network)
  - **Format:** Decision tables

- [ ] **Task 5.1.8:** Add to documentation index
  - **Update:** `docs/00-DOCUMENTATION-INDEX.md`
  - **Add:** Entry for `34-PERFORMANCE-CHARACTERISTICS.md`
  - **Cross-reference:** Link from README, USER-GUIDE, BENCHMARKING-GUIDE

**Acceptance Criteria:**
- `docs/34-PERFORMANCE-CHARACTERISTICS.md` created (1,500-2,000 lines)
- All sections complete with measured data
- Capacity planning tables practical and accurate
- Cross-references added to other docs

---

## Task Area 6: Results Publishing (6 tasks, 2-3 hours)

**Goal:** Publish benchmark results for transparency and historical tracking.

**Deliverable:** Historical tracking system, public dashboard (README section).

### Subtask 6.1: Historical Tracking (3 tasks, 1-2h)

**Goal:** Store benchmark results over time for trend analysis.

- [ ] **Task 6.1.1:** Create history directory structure
  - **Create:** `benchmarks/history/`
  - **Naming:** `YYYY-MM-DD-v<version>.json` (e.g., `2025-11-09-v0.5.1.json`)
  - **Content:** Aggregated results from all scenarios
  - **Files:** Create directory

- [ ] **Task 6.1.2:** Implement history archival script
  - **Script:** `scripts/archive-benchmark-results.sh`
  - **Logic:**
    1. Copy latest results to `history/YYYY-MM-DD-v<version>.json`
    2. Generate summary metadata
  - **Usage:** `./scripts/archive-benchmark-results.sh v0.5.1`
  - **Files:** Create script

- [ ] **Task 6.1.3:** Generate trend graphs (future enhancement)
  - **Tool:** Python + matplotlib (or gnuplot)
  - **Implementation:**
    ```python
    import json
    import matplotlib.pyplot as plt
    from pathlib import Path

    history_files = sorted(Path('benchmarks/history').glob('*.json'))
    versions = []
    syn_scan_times = []

    for file in history_files:
        with open(file) as f:
            data = json.load(f)
        versions.append(file.stem.split('-')[-1])
        syn_scan_times.append(data['syn_scan']['mean'])

    plt.plot(versions, syn_scan_times, marker='o')
    plt.xlabel('Version')
    plt.ylabel('SYN Scan Time (ms)')
    plt.title('Performance Trend: SYN Scan (1,000 ports)')
    plt.savefig('benchmarks/reports/performance-trend-syn-scan.png')
    ```
  - **Files:** `scripts/generate-trend-graphs.py` (create if time permits)
  - **Note:** Mark as optional, can defer to v0.6.0

**Acceptance Criteria:**
- History directory created
- Archival script working
- Trend graph script created (optional)

### Subtask 6.2: Public Dashboard (3 tasks, 1h)

**Goal:** Update `benchmarks/README.md` with latest results.

- [ ] **Task 6.2.1:** Update benchmark README
  - **File:** `benchmarks/README.md`
  - **Add sections:**
    - Latest benchmark results (table)
    - Historical trends (link to history/)
    - Comparison to competitors (Nmap, Masscan, RustScan)
  - **Format:**
    ```markdown
    ## Latest Benchmark Results (v0.5.1, 2025-11-09)

    | Scenario | Mean | Stddev | Status |
    |----------|------|--------|--------|
    | SYN Scan (1K ports) | 98.2ms | ±4.5ms | ✅ |
    | Connect (3 ports) | 45.3ms | ±2.1ms | ✅ |
    ...

    ## Historical Performance

    ![Performance Trend](reports/performance-trend-syn-scan.png)

    ## Comparison to Competitors

    | Tool | SYN Scan (1K ports) | Notes |
    |------|---------------------|-------|
    | ProRT-IP | 98ms | This project |
    | Nmap | 150ms | Slower due to scripting overhead |
    | Masscan | 50ms | Faster but less features |
    | RustScan | 120ms | Similar feature set |
    ```

- [ ] **Task 6.2.2:** Add comparison to competitors
  - **Benchmark Nmap:** `nmap -sS -p 1-1000 127.0.0.1` (use hyperfine)
  - **Benchmark Masscan:** `masscan 127.0.0.1 -p 1-1000` (if installed)
  - **Benchmark RustScan:** `rustscan -a 127.0.0.1 -p 1-1000` (if installed)
  - **Compare:** Document performance differences
  - **Note:** Skip if tools not installed, use estimates from documentation

- [ ] **Task 6.2.3:** Update root README.md with performance highlights
  - **File:** `README.md`
  - **Update:** Performance section with latest benchmarks
  - **Highlight:** Key metrics (throughput, overhead, memory)
  - **Link:** To `docs/34-PERFORMANCE-CHARACTERISTICS.md`

**Acceptance Criteria:**
- `benchmarks/README.md` updated with latest results
- Competitor comparison added (or documented as future work)
- Root `README.md` updated with highlights

---

## Success Criteria

### Quantitative Targets

- [ ] **Benchmark scenarios:** 20+ total (baseline: 8, target: 150% increase)
  - Current: 8 scenarios (Sprint 5.9)
  - New: 12+ scenarios added
  - Total: 20-26 scenarios

- [ ] **Performance improvement:** 10%+ speedup on 3+ scenarios
  - Identify bottlenecks via profiling
  - Implement optimizations
  - Measure speedup (before/after benchmarks)

- [ ] **Regression detection:** CI runs weekly, <10% tolerance
  - GitHub Actions workflow operational
  - Regression detection script working
  - Weekly scheduled runs configured

- [ ] **Documentation:** All performance metrics documented
  - Throughput (pps, hosts/min)
  - Latency (p50/p95/p99)
  - Memory (MB per scenario)
  - Scaling characteristics

### Qualitative Targets

- [ ] **Baselines established:** Known performance for all scan types
  - SYN/Connect/UDP/FIN/NULL/Xmas/ACK/Idle/Window/Maimon
  - Service detection, OS fingerprinting, TLS parsing
  - Plugin execution, event system overhead

- [ ] **Regressions detectable:** CI catches slowdowns automatically
  - 5% threshold: WARN
  - 10% threshold: FAIL (block PR)
  - PR comments with regression details

- [ ] **Optimizations implemented:** Low-hanging fruit picked
  - Packet crafting optimizations
  - Service detection optimizations
  - I/O batching improvements
  - Rate limiting enhancements (if possible beyond -1.8%)

- [ ] **Performance predictable:** Capacity planning guide accurate
  - Users can estimate scan duration
  - Users can determine hardware requirements
  - Scaling characteristics documented

### Phase 6 Readiness

- [ ] **TUI performance baseline known:** Can attribute overhead
  - Pre-TUI benchmarks complete
  - Post-TUI can compare (Phase 6)

- [ ] **No performance regressions:** Clean slate for Phase 6
  - All benchmarks passing (<10% regression)
  - Optimizations merged

- [ ] **Optimization complete:** TUI won't inherit performance debt
  - All identified bottlenecks addressed
  - Future optimizations documented for Phase 6+

---

## Deliverables Checklist

### Code Deliverables

- [ ] **Benchmark scripts:** 12+ new scenarios (09-26 or subset)
  - All executable (`chmod +x`)
  - All follow existing format (hyperfine, JSON export)
  - All documented in guide

- [ ] **Profiling data:** Flamegraphs, massif, strace for major scenarios
  - 5+ flamegraphs in `benchmarks/flamegraphs/`
  - 5+ massif reports in `benchmarks/massif/`
  - 5+ strace analyses in `benchmarks/strace/`

- [ ] **Optimizations:** 5-7 performance improvements
  - Packet crafting (buffer pool, SIMD checksums, etc.)
  - Service detection (regex compilation, pattern optimization, parallelization)
  - I/O (batch sizes, async file writes)
  - Rate limiting (V3 enhancements)

- [ ] **CI workflow:** `.github/workflows/benchmarks.yml`
  - Automated weekly runs
  - Regression detection
  - PR comments

- [ ] **Scripts:** Baseline management, result archival
  - `scripts/create-baseline.sh`
  - `scripts/archive-benchmark-results.sh`
  - `scripts/analyze-results.sh` (enhanced)

### Documentation Deliverables

- [ ] **Performance guide:** `docs/34-PERFORMANCE-CHARACTERISTICS.md`
  - 1,500-2,000 lines
  - All sections complete
  - Capacity planning tables

- [ ] **Benchmark guide update:** `docs/31-BENCHMARKING-GUIDE.md`
  - New scenarios documented
  - CI integration documented
  - Regression detection documented

- [ ] **Profiling analysis:** `benchmarks/{flamegraphs,massif,strace}/ANALYSIS.md`
  - Hotspots identified
  - Optimization opportunities documented
  - Before/after comparisons

- [ ] **Benchmarks README:** `benchmarks/README.md`
  - Latest results published
  - Historical trends (if graphs created)
  - Competitor comparison (if available)

- [ ] **Root README update:** `README.md`
  - Performance section updated
  - Latest metrics highlighted
  - Link to performance guide

### Sprint Completion Deliverables

- [ ] **CHANGELOG update:** Add Sprint 5.5.4 entry
  - Summary of changes
  - Performance improvements
  - New benchmark scenarios

- [ ] **PROJECT-STATUS update:** `docs/10-PROJECT-STATUS.md`
  - Mark Sprint 5.5.4 complete
  - Update Phase 5.5 progress

- [ ] **CLAUDE.local.md update:** Recent Sessions table
  - Add Sprint 5.5.4 session
  - Document key results
  - Update metrics (tests, coverage if changed)

- [ ] **Phase 5.5 master plan update:** `to-dos/PHASE-5.5-PRE-TUI-ENHANCEMENTS.md`
  - Mark Sprint 5.5.4 complete
  - Update status checkboxes

- [ ] **Sprint completion report:** `SPRINT-5.5.4-COMPLETE.md`
  - Executive summary
  - Task completion (71 tasks)
  - Time spent (actual vs estimated)
  - Performance improvements achieved
  - Deliverables list
  - Grade (A/A+/B/etc.)
  - Next steps

---

## Risk Mitigation

### Risk 1: Benchmarks are noisy (high variance)

**Impact:** MEDIUM (unreliable regression detection)

**Mitigation:**
- Multiple runs (10+ per scenario)
- Statistical rigor (mean ± stddev, IQR outlier removal)
- Isolation (run on dedicated hardware if possible, close background processes)
- Consistency (pin CPU frequency: `sudo cpupower frequency-set --governor performance`)
- Use median instead of mean if variance too high

**Contingency:**
- Increase runs to 20 (more data points)
- Use median (more robust against outliers)
- Run on dedicated CI runner (if available)

### Risk 2: Profiling is time-consuming

**Impact:** LOW (may exceed 6h budget for profiling)

**Mitigation:**
- Automate flamegraph generation (scripts, not manual)
- Prioritize high-impact scenarios first (SYN scan, service detection)
- Use existing tools (cargo flamegraph, massif)
- Skip low-priority scenarios if time runs out

**Contingency:**
- Profile only top 3-5 scenarios (SYN, service, TLS, plugin, event)
- Defer remaining profiling to future sprint

### Risk 3: Optimizations break functionality

**Impact:** HIGH (regression in correctness)

**Mitigation:**
- Run full test suite after each optimization (`cargo test --all`)
- Regression testing (ensure 1,601 tests still pass)
- Incremental changes (one optimization at a time)
- Measure impact (if optimization doesn't help, revert)

**Contingency:**
- Revert optimization if tests fail
- Document why optimization didn't work
- Move to next optimization

### Risk 4: CI benchmarks slow down pipeline

**Impact:** LOW (CI time increased)

**Mitigation:**
- Run weekly (not on every commit)
- On-demand manual trigger (workflow_dispatch)
- Timeout limit (30 minutes max)
- Skip on non-performance PRs (path filtering)

**Contingency:**
- Reduce scenarios in CI (run only 8 core scenarios, not all 20+)
- Increase timeout if needed (up to 60 minutes)

### Risk 5: No significant optimizations found

**Impact:** MEDIUM (can't achieve 10%+ speedup target)

**Mitigation:**
- Profiling will identify bottlenecks (likely some exist)
- Low-hanging fruit usually exists (regex compilation, buffer pooling, etc.)
- Even small optimizations compound (3% + 3% + 4% = 10%)

**Contingency:**
- Document current performance as baseline (even if no optimization)
- Adjust target to "5%+ speedup on 3+ scenarios" (more achievable)
- Document optimization opportunities for future sprints

---

## Dependencies

### External Dependencies

- **hyperfine:** Already installed (Sprint 5.9)
- **cargo-flamegraph:** Need to install (`cargo install flamegraph`)
- **valgrind:** Need to install (`sudo apt install valgrind`, Linux only)
- **perf:** Bundled with Linux kernel (usually available)
- **Python 3.8+:** Optional (statistical tests, trend graphs)
- **scipy:** Optional (t-tests: `pip install scipy`)
- **matplotlib:** Optional (trend graphs: `pip install matplotlib`)

### Internal Dependencies

- **Sprint 5.9:** Benchmarking framework (COMPLETE) ✅
- **Cargo build system:** Functional ✅
- **Release binary:** Can build ✅
- **Test suite:** Passing (1,601 tests) ✅

### Phase 5.5 Dependencies

- **Sprint 5.5.1:** Documentation (COMPLETE) ✅
- **Sprint 5.5.2:** CLI Usability (COMPLETE) ✅
- **Sprint 5.5.3:** Event System (COMPLETE) ✅
- **Sprint 5.5.4:** THIS SPRINT (PENDING)
- **Sprint 5.5.5:** Configuration & State (BLOCKED on 5.5.4 completion)
- **Sprint 5.5.6:** Integration & Testing (BLOCKED on all above)

---

## Execution Strategy

### Phase 1: Benchmarking Expansion (6-8 hours)

**Goal:** Get comprehensive benchmark coverage.

**Tasks:** Task Area 1 (Comprehensive Benchmarking)
- Create 12+ new scenarios
- Update orchestrator script
- Run all 20+ benchmarks
- Validate results

**Output:** 20+ benchmark scenarios, baseline data

### Phase 2: Profiling & Analysis (5-6 hours)

**Goal:** Identify optimization opportunities.

**Tasks:** Task Area 2 (Profile Hot Paths)
- Install profiling tools
- Generate flamegraphs (CPU)
- Generate massif reports (Memory)
- Run strace analysis (I/O)
- Document findings

**Output:** Profiling data, optimization candidate list

### Phase 3: Optimization (6-8 hours)

**Goal:** Implement performance improvements.

**Tasks:** Task Area 3 (Optimize Bottlenecks)
- Prioritize by impact
- Implement top 5-7 optimizations
- Measure before/after
- Achieve 10%+ speedup on 3+ scenarios

**Output:** Performance improvements, speedup metrics

### Phase 4: Automation (4-5 hours)

**Goal:** Set up continuous performance monitoring.

**Tasks:** Task Area 4 (Regression Detection)
- Create CI workflow
- Implement regression detection script
- Set up baseline management
- Test end-to-end

**Output:** Automated regression detection in CI

### Phase 5: Documentation (4-5 hours)

**Goal:** Document everything for users and future developers.

**Tasks:** Task Area 5 (Performance Documentation)
- Create performance guide
- Document all metrics
- Create capacity planning tables
- Update existing docs

**Output:** Comprehensive performance documentation

### Phase 6: Publishing (2-3 hours)

**Goal:** Make results public and trackable.

**Tasks:** Task Area 6 (Results Publishing)
- Set up historical tracking
- Update benchmark README
- Update root README
- Generate trend graphs (optional)

**Output:** Public benchmark dashboard

---

## Testing Strategy

### Benchmark Validation

- **Run all scenarios:** Ensure all 20+ scenarios execute without errors
- **Check variance:** Stddev <10% of mean (reproducible results)
- **Validate targets:** Results match expected performance targets
- **Compare platforms:** Linux vs macOS (if available)

### Optimization Validation

- **Before/after benchmarks:** Measure speedup for each optimization
- **Full test suite:** `cargo test --all` after each change (1,601 tests passing)
- **Clippy clean:** `cargo clippy --all-targets -- -D warnings` (0 warnings)
- **Correctness tests:** Ensure optimizations don't break functionality
  - Service detection still 85-90% accurate
  - Rate limiting still -1.8% overhead
  - No feature regressions

### CI Workflow Validation

- **Local testing:** Use `act` to test workflow locally
- **Manual trigger:** Test workflow_dispatch trigger
- **Regression detection:** Artificially slow a scenario, verify FAIL status
- **PR comments:** Create test PR, verify comment generation

---

## Progress Tracking

### Task Completion Tracking

Use checkboxes in this TODO file to track progress:
- [ ] = Pending
- [x] = Complete

**Update after each task:** Check off completed tasks, add notes if needed.

### Time Tracking

Track actual time spent per task area:

| Task Area | Estimated | Actual | Efficiency |
|-----------|-----------|--------|------------|
| 1. Benchmarking | 6-8h | TBD | TBD |
| 2. Profiling | 5-6h | TBD | TBD |
| 3. Optimization | 6-8h | TBD | TBD |
| 4. Regression | 4-5h | TBD | TBD |
| 5. Documentation | 4-5h | TBD | TBD |
| 6. Publishing | 2-3h | TBD | TBD |
| **TOTAL** | **27-35h** | **TBD** | **TBD** |

**Efficiency calculation:** `(Estimated / Actual) * 100%`
- >100%: Faster than estimated (excellent)
- 80-100%: On target (good)
- <80%: Slower than estimated (acceptable, adjust future estimates)

### Daily Progress Log

Track progress in `/tmp/ProRT-IP/SPRINT-5.5.4-PROGRESS.md`:

**Format:**
```markdown
# Sprint 5.5.4 Progress Log

## 2025-11-09 (Day 1)
- **Hours:** 6h
- **Completed:**
  - Task Area 1: 80% complete (16/20 tasks)
  - 10 new benchmark scenarios created
  - All scripts tested locally
- **Blockers:** None
- **Next:** Finish Task Area 1, start Task Area 2 profiling

## 2025-11-10 (Day 2)
...
```

---

## Completion Checklist

**Before marking sprint complete:**

- [ ] All 71 tasks completed (or documented as skipped with reason)
- [ ] All 6 task areas 100% complete
- [ ] Success criteria met:
  - [ ] 20+ benchmark scenarios operational
  - [ ] 10%+ speedup on 3+ scenarios
  - [ ] CI regression detection working
  - [ ] Performance documentation complete
- [ ] Deliverables created:
  - [ ] 12+ new benchmark scripts
  - [ ] Profiling data (flamegraphs, massif, strace)
  - [ ] 5-7 optimizations implemented
  - [ ] CI workflow operational
  - [ ] Performance guide created
  - [ ] Results published
- [ ] Quality checks passed:
  - [ ] `cargo test --all` (1,601 tests passing)
  - [ ] `cargo clippy --all-targets -- -D warnings` (0 warnings)
  - [ ] `cargo fmt --all -- --check` (formatted)
  - [ ] All benchmarks passing (<10% regression)
- [ ] Documentation updated:
  - [ ] CHANGELOG.md
  - [ ] README.md
  - [ ] docs/10-PROJECT-STATUS.md
  - [ ] docs/31-BENCHMARKING-GUIDE.md
  - [ ] docs/34-PERFORMANCE-CHARACTERISTICS.md (new)
  - [ ] to-dos/PHASE-5.5-PRE-TUI-ENHANCEMENTS.md
- [ ] Sprint completion report created: `SPRINT-5.5.4-COMPLETE.md`
- [ ] Memory banks updated: CLAUDE.local.md

---

## Next Steps (After Sprint 5.5.4)

**Sprint 5.5.5: Configuration & State Management**
- Configuration profiles (save/load scan presets)
- Scan state persistence (resume interrupted scans)
- State management API (TUI prerequisite)

**Phase 6: TUI Interface (Q1-Q2 2026)**
- Real-time terminal UI (ratatui)
- Live scan progress visualization
- Interactive controls
- Uses event system from Sprint 5.5.3
- Uses state management from Sprint 5.5.5
- Performance baseline from Sprint 5.5.4 validates no TUI overhead

---

## References

- **Phase 5.5 Master Plan:** `to-dos/PHASE-5.5-PRE-TUI-ENHANCEMENTS.md`
- **Sprint 5.9 TODO:** `to-dos/PHASE-5/SPRINT-5.9-TODO.md`
- **Benchmarking Guide:** `docs/31-BENCHMARKING-GUIDE.md`
- **Sprint 5.9 Framework:** `benchmarks/05-Sprint5.9-Benchmarking-Framework/`
- **GitHub Actions Docs:** https://docs.github.com/en/actions
- **hyperfine Docs:** https://github.com/sharkdp/hyperfine
- **cargo-flamegraph:** https://github.com/flamegraph-rs/flamegraph
- **valgrind massif:** https://valgrind.org/docs/manual/ms-manual.html

---

**Document Version:** 1.0
**Created:** 2025-11-09
**Status:** PENDING
**Total Tasks:** 71
**Estimated Duration:** 27-35 hours (3-4 days)
**Expected Completion:** 2025-11-12 to 2025-11-13
