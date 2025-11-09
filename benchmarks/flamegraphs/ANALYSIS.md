# CPU Profiling Analysis (Flamegraphs)

**Sprint:** 5.5.4 - Performance Audit & Optimization
**Created:** 2025-11-09
**Status:** Framework Ready (Profiling Deferred)

---

## Overview

This directory contains CPU flamegraphs generated using `cargo-flamegraph` (Linux perf).

**Purpose:**

- Identify CPU hotspots (functions consuming >5% CPU)
- Validate performance claims (packet crafting, regex matching, etc.)
- Guide optimization priorities

---

## Planned Flamegraph Scenarios

### 1. SYN Scan (`syn-scan-10k-ports.svg`)

**Command:**
```bash
cargo flamegraph --bin prtip -- -sS -p 1-10000 127.0.0.1
```

**Expected Hotspots:**

- Packet crafting (pnet serialization)
- Checksum calculation
- sendmmsg/recvmmsg syscalls
- Rate limiter convergence

**Target:** No single function >20% CPU

### 2. Service Detection (`service-detection.svg`)

**Command:**
```bash
cargo flamegraph --bin prtip -- -sV -p 22,80,443,3306,5432 127.0.0.1
```

**Expected Hotspots:**

- Regex matching (banner patterns)
- Service probe iteration
- TLS handshake (openssl)

**Target:** Regex matching <30% CPU

### 3. TLS Certificate Parsing (`tls-cert-parsing.svg`)

**Command:**
```bash
cargo flamegraph --bin prtip -- -sV -p 443 badssl.com --tls-cert-analysis
```

**Expected Hotspots:**

- X.509 parsing (rcgen/x509-parser)
- Certificate chain validation
- ASN.1 decoding

**Target:** Parsing <10% total CPU (1.33μs claim)

### 4. OS Fingerprinting (`os-fingerprinting.svg`)

**Command:**
```bash
cargo flamegraph --bin prtip -- -O -p 80 127.0.0.1
```

**Expected Hotspots:**

- 16-probe sequence generation
- Signature matching (2,600+ DB)
- TCP option parsing

**Target:** Signature matching <40% CPU

### 5. Event System (`event-system.svg`)

**Command:**
```bash
cargo flamegraph --bin prtip -- -sS -p 1-1000 127.0.0.1 --event-log /tmp/events.jsonl
```

**Expected Hotspots:**

- Event bus publish/subscribe
- JSON serialization (serde)
- File I/O (async buffered writes)

**Target:** Event overhead <5% CPU (claim: 4.1%)

---

## Analysis Methodology

### Hotspot Identification

**Criteria for Optimization:**

1. Function consumes >5% total CPU
2. Function called frequently (>10,000 times)
3. Function is not external dependency (can be optimized)

**Prioritization Formula:**

```
Priority = (CPU% × Call_Count × Optimization_Ease) / 100
```

Where:
- CPU%: Percentage of total CPU time
- Call_Count: Number of invocations (normalized to 1-10 scale)
- Optimization_Ease: 1-10 (10 = trivial, 1 = major refactor)

### Example Analysis

**Hypothetical Flamegraph Output:**

```
Total Samples: 10,000
Top Functions:
  1. pnet::packet::ip::checksum (1,850 samples, 18.5%)
  2. prtip_scanner::packet_crafter::craft_syn (1,200 samples, 12.0%)
  3. regex::exec::match_at (980 samples, 9.8%)
  4. tokio::runtime::task::raw::poll (850 samples, 8.5%)
  5. std::collections::hash::map::HashMap::insert (720 samples, 7.2%)
```

**Optimization Candidates:**

| Function | CPU% | Optimization | Ease | Priority |
|----------|------|--------------|------|----------|
| checksum | 18.5% | SIMD checksums | 9 | 166.5 |
| craft_syn | 12.0% | Buffer pooling | 7 | 84.0 |
| regex::match_at | 9.8% | Compiled regexes | 10 | 98.0 |
| HashMap::insert | 7.2% | Pre-allocation | 8 | 57.6 |

**Top Priority:** checksum SIMD optimization (score: 166.5)

---

## Optimization Tracking

### Implemented Optimizations (Sprint 5.5.4)

**None Yet (Framework Ready)**

When profiling is performed and optimizations implemented, document here:

| Optimization | Before | After | Speedup | Status |
|--------------|--------|-------|---------|--------|
| TBD | TBD | TBD | TBD | Pending |

---

## Future Work

### Additional Profiling Scenarios

1. **Plugin Execution:** Lua VM overhead analysis
2. **Idle Scan:** Zombie probing efficiency
3. **UDP Scan:** ICMP handling overhead
4. **Large-Scale:** Memory allocation patterns at 100K+ hosts

### Tools to Explore

- **perf record/report:** More detailed syscall analysis
- **heaptrack:** Heap allocation flamegraphs
- **cargo-instruments:** macOS profiling (Instruments integration)

---

## Notes

- Flamegraphs require release build for accurate profiling: `cargo flamegraph --release`
- Linux perf requires kernel debug symbols: `debuginfod`
- Interactive SVG files: Click to zoom in/out

---

**Last Updated:** 2025-11-09
**Next Update:** After profiling execution (deferred to future sprint)
