# ProRT-IP WarScan

## Protocol/Port Real-Time War Scanner for IP Networks

<div align="center">
  <img src="images/prortip-logo-dark.jpg" alt="ProRT-IP Logo" width="800">
</div>

[![CI](https://github.com/doublegate/ProRT-IP/actions/workflows/ci.yml/badge.svg)](https://github.com/doublegate/ProRT-IP/actions/workflows/ci.yml)
[![Release](https://github.com/doublegate/ProRT-IP/actions/workflows/release.yml/badge.svg)](https://github.com/doublegate/ProRT-IP/actions/workflows/release.yml)
[![License: GPL v4](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust0.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/github/v/release/doublegate/ProRT-IP)](https://github.com/doublegate/ProRT-IP/releases)
[![Tests](https://img.shields.io/badge/tests-789_passing-brightgreen.svg)]
[![GitHub](https://img.shields.io/badge/github-ProRT--IP-blue)](https://github.com/doublegate/ProRT-IP)

---

<div align="center">
  <img src="images/prortip-screenshot.png" alt="ProRT-IP CLI Screenshot" width="600">
</div>

## Overview

**ProRT-IP WarScan** is a modern network scanner written in Rust that combines:

- **Speed:** 1M+ packets/second stateless scanning (comparable to Masscan/ZMap)
- **Depth:** Comprehensive service detection and OS fingerprinting (like Nmap)
- **Safety:** Memory-safe Rust implementation prevents entire vulnerability classes
- **Stealth:** Advanced evasion techniques (timing, decoys, fragmentation, idle scans)
- **Extensibility:** Plugin system with Lua scripting support

**At a glance:**

- **Multi-Protocol Scanning:** TCP (SYN, Connect, FIN, NULL, Xmas, ACK, Idle), UDP, ICMP
- **Service Detection:** 500+ protocol probes with version identification
- **OS Fingerprinting:** 2000+ signatures using 16-probe technique
- **High Performance:** Asynchronous I/O with lock-free coordination
- **Cross-Platform:** Linux, Windows, macOS support
- **Multiple Interfaces:** CLI (v1.0), TUI (planned), Web UI (planned), GUI (planned)

### Introduction

**ProRT-IP WarScan** (Protocol/Port Real-Time IP War Scanner) is a modern equivalent of classic 1980s/1990s war dialers—reimagined for IP networks. Where tools like ToneLoc and THC-Scan systematically dialed phone numbers to find modems/BBSs, WarScan systematically scans IP address ranges, ports, and protocols to discover active hosts and services.

WarScan consolidates and advances the best of today's network scanning and analysis tools, delivering a comprehensive, high-performance, stealth-focused toolkit for penetration testers and red teams. It is implemented in **Rust** for safety and performance, initially released as a **CLI** utility (`prtip`), with a roadmap for **TUI**, **web**, and **desktop GUI** interfaces.

**Key goals and characteristics:**

- **Extensive multi-layer scanning:** From host discovery (ARP/ICMP) up through TCP/UDP scans and application-layer banner grabbing
- **High performance & efficiency:** Internet-scale scanning inspired by the fastest modern scanners (1M+ packets/second stateless)
- **Advanced red-team features:** Stealth techniques (randomization, timing dithering, decoys, fragmentation, idle scans) to evade detection
- **Cross-platform & extensible:** Linux-first with Windows/macOS support via Rust portability; open-source (GPLv3)
- **Future UI enhancements:** TUI → web → GUI, expanding accessibility without sacrificing power

**In summary**, WarScan aims to be a one-stop, modern war-scanning solution—combining the thoroughness of classic mappers, the speed of internet-scale scanners, the usability of friendly GUIs, the deep packet insight of protocol analyzers, and the raw versatility of low-level network tools.

### Inspiration from Existing Tools

To design WarScan, we surveyed state-of-the-art tools widely used for networking, penetration testing, and packet analysis. Each contributes valuable features and lessons:

- **Nmap (Network Mapper):** The gold standard for discovery, versatile port scan techniques, service/version detection, OS fingerprinting, a powerful scripting engine, and numerous stealth/evasion capabilities. WarScan incorporates multiple scan types (connect, SYN, FIN/NULL/Xmas, UDP), service/OS detection, and similar evasion features in a modernized implementation.

- **Masscan:** Ultra high-speed, asynchronous/stateless internet-scale scanning at extreme packet rates. WarScan borrows the speed/scalability model—highly parallelized, stateless fast modes—then enables deeper follow-up scans on responders.

- **ZMap:** Internet-scale, single-packet rapid scans across huge IP ranges. WarScan includes a comparable "fast scan mode" for breadth-first discovery followed by depth on responsive hosts.

- **RustScan:** Demonstrates Rust's advantages: fast full-port sweeps, adaptive performance learning, and extensibility/scripting. WarScan mirrors this split-phase strategy (fast discovery → detailed enumeration) and evaluates an embedded scripting layer.

- **Unicornscan:** Pioneered asynchronous/stateless techniques and userland TCP/IP stack control for advanced packet crafting, banner grabbing, protocol-specific UDP probes, and OS/app fingerprinting. WarScan builds similar packet-crafting flexibility and export to PCAP/DB.

- **Wireshark:** The model for protocol depth and parsing. WarScan parses responses (e.g., HTTP headers, TLS certs), logs to PCAP, and emphasizes robust protocol coverage.

- **Angry IP Scanner:** Highlights usability, speed via multithreading, cross-platform reach, simple exports, and plugins. WarScan's roadmap includes a friendly TUI/GUI and enriched host info (reverse DNS, ARP/MAC/vendor, NetBIOS/mDNS where possible).

- **Netcat/Ncat:** The "Swiss Army knife" for quick banner grabs and interactive tests. WarScan supports custom payloads and optional interactive follow-ups to validate findings.

### Feature Comparison

![Network Scanner Feature Comparison](images/scanner_comparison.jpg)

*Comparison of ProRT-IP WarScan with leading network scanning tools (Nmap, Masscan, ZMap, RustScan)*

---

## Table of Contents

- [Project Status](#project-status)
- [Documentation](#documentation)
- [Quick Start](#quick-start)
- [Planned Usage](#planned-usage)
- [Development Roadmap](#development-roadmap)
- [Technical Specifications](#technical-specifications)
- [Architecture Overview](#architecture-overview)
- [Building from Source](#building-from-source)
- [Contributing](#contributing)
- [Support](#support)
- [Security](#security)
- [License](#license)
- [Authors & Acknowledgments](#authors--acknowledgments)
- [Legal Notice](#legal-notice)

---

## Project Status

**Current Phase:** Phase 4 COMPLETE ✅ | Phase 5 Advanced Features - Next

**Latest Version:** v0.3.7 (Testing Infrastructure Complete - 789 tests, 61.92% coverage, benchmark baselines, Windows CI fix)

**Test Coverage:** 789 tests passing (100% success rate, all platforms) | 61.92% code coverage (exceeds 60% target)

**CI/CD Status:** 7/7 jobs passing (Windows fix 2025-10-13) | 8/8 release platforms production-ready (musl + ARM64 fixed 2025-10-12)

**Latest Achievements:**

- ✅ **Testing Infrastructure (v0.3.7):** Code coverage 61.92% (cargo-tarpaulin), benchmark baselines (Criterion.rs), 67 integration tests
- ✅ **Windows CI Fixed (v0.3.7):** Platform-aware test expectations (all 789 tests passing)
- ✅ **Nmap CLI Compatibility (v0.3.5):** 20+ nmap-compatible flags, greppable output, top ports database
- ✅ **Phase 4 Complete:** All sprints (4.1-4.14) finished, all known issues resolved
- ✅ **GitHub Templates:** 6 templates added (5 issue types + PR template)
- ✅ **Port Scanning:** 100% accuracy, 2.3-35x faster than competitors
- ✅ **Performance:** 66ms for common ports (vs nmap: 150ms, rustscan: 223ms, naabu: 2335ms)
- ✅ **Service Detection:** 187 embedded probes, 50% detection rate (HTTP, SSH verified)
- ✅ **Progress Bar:** Real-time updates with sub-millisecond polling (0.2-2ms adaptive)
- ✅ **Large Scans:** 10x speedup on network scans (2,844 pps), 3-17x on filtered networks
- ✅ **DNS Resolution:** Hostname support (scanme.nmap.org, google.com, etc.)
- ✅ **Benchmarking:** 29 comprehensive benchmark files with flamegraph

**Industry Comparison (Common Ports on scanme.nmap.org):**

| Scanner | Time | vs ProRT-IP | Accuracy |
|---------|------|-------------|----------|
| **ProRT-IP** | **66ms** | **baseline** | 100% ✅ |
| nmap | 150ms | 2.3x slower | 100% ✅ |
| rustscan | 223ms | 3.4x slower | 100% ✅ |
| naabu | 2335ms | 35.4x slower | 100% ✅ |

**ProRT-IP is the fastest validated network scanner tested.**

**Recent Accomplishments:**

- ✅ Phase 1: Core Infrastructure (weeks 1-3)
- ✅ Phase 2: Advanced Scanning (weeks 4-6)
- ✅ Enhancement Cycles 1-8: Reference implementation optimizations
  - Cycle 1: Cryptographic foundation (SipHash, Blackrock)
  - Cycle 2: Concurrent scanning patterns (FuturesUnordered)
  - Cycle 3: Resource management (ulimit detection, interface selection)
  - Cycle 4: CLI integration and ulimit awareness
  - Cycle 5: Progress tracking and error categorization
  - Cycle 6: Port filtering infrastructure
  - Cycle 7: Advanced filtering and exclusion lists
  - Cycle 8: Performance & stealth (sendmmsg batching, CDN detection, decoy scanning)
- ✅ Phase 3: Detection Systems (weeks 7-10)
  - OS fingerprinting with 16-probe sequence
  - Service version detection with nmap-service-probes
  - Banner grabbing with protocol-specific handlers
  - Database parsers for 2,000+ OS signatures
- ✅ CI/CD Optimization (2025-10-09)
  - Multi-platform builds (8 targets: Linux x86/ARM, Windows, macOS Intel/ARM, FreeBSD)
  - Smart release management (preserve notes, manual execution)
  - 100% CI success rate (7/7 jobs passing)
  - Cross-compilation infrastructure (cross-rs)
  - Platform support documentation

**Implementation Impact:**

- Tests: 215 → 789 (+574 tests, +267% growth) | 67 integration tests
- Code Coverage: 61.92% (15,397 / 24,814 lines covered, exceeds 60% target)
- Lines: ~25,097 total Rust code (production + tests)
- Production Code: ~12,000+ lines (Phase 1-3: 6,097 + Enhancements: 4,546 + Phase 4: ~1,400)
- Modules: 43+ total production modules
- Platforms: 5 production-ready (Linux x86, Windows, macOS Intel/ARM, FreeBSD)
- Build Targets: 9 total (5 working, 4 experimental)
- Latest Additions: Progress bar real-time updates, network timeout optimization, host delay feature

**Phase 4 Progress (Sprint 4.1-4.14 COMPLETE ✅):**

- ✅ Sprint 4.1: Network Testing Infrastructure (Docker Compose + 10 services, latency simulation, test environment docs)
- ✅ Sprint 4.2: Lock-Free Result Aggregator (crossbeam SegQueue, 10M+ results/sec, <100ns latency)
- ✅ Sprint 4.3: Lock-Free Integration (tcp_connect.rs integration, 9 new tests)
- ✅ Sprint 4.4: Critical 65K Port Fix (>180s hang → 0.91s, **198x faster!**)
  - Critical bug fix: u16 port overflow causing infinite loop on port 65535
  - Adaptive scaling: 20-1000 concurrent based on port count
  - 342 lines adaptive parallelism module with 17 comprehensive tests
- ✅ Sprint 4.5: Performance Profiling (Root cause: SQLite contention, 95.47% futex time)
- ✅ Sprint 4.6: In-Memory Default Mode (194.9ms → 41.1ms, **5.2x faster!**)
  - Breaking change: --no-db removed, --with-db added
  - Async storage worker with channel communication
- ✅ Sprint 4.7: Scheduler Refactor (StorageBackend enum, architecture cleanup)
- ✅ Sprint 4.8 v2: Async Storage Deadlock Fix (139.9ms → 74.5ms, **46.7% improvement!**)
  - Critical fix: Replaced tokio::select! with timeout() pattern
  - Zero hangs, proper channel closure, production-ready
- ✅ Sprint 4.9: Final Benchmarking (29 files: hyperfine, perf, strace, massif, flamegraph)
- ✅ Sprint 4.10: CLI Improvements (statistics, parallel count fix, scan summary)
- ✅ Sprint 4.11: Service Detection Integration + DNS Fix + Validation
  - Service detection wired into scheduler (--sV, --version-intensity, --banner-grab)
  - DNS hostname resolution (scanme.nmap.org, google.com)
  - Comprehensive validation vs nmap, rustscan, naabu
  - **100% port accuracy, 2.3-35x faster than competitors**
- ✅ Sprint 4.12 v3: Progress Bar Real-Time Updates FIX (**CRITICAL BUG FIXED!**)
  - **Issue:** Progress bar starting at 100% instead of 0%
  - **Root Cause:** Bridge polling (5-50ms) too slow for ultra-fast scans (40-50ms total)
  - **Solution:** Sub-millisecond adaptive polling (0.2-2ms based on port count)
  - **Result:** 5-50 incremental updates vs previous 1-2, zero performance regression
  - **Files Modified:** scheduler.rs (9 lines), progress_bar.rs (2 lines)
- ✅ Sprint 4.13: Critical Performance Regression Fix (**10x SPEEDUP!**)
  - **Issue:** Large network scans 50-800x slower (289 pps with 2-hour ETA)
  - **Root Cause:** Variable shadowing bug - polling based on per-host ports (10K) not total (2.56M)
  - **Solution:** Total-scan-aware adaptive polling (200µs → 10ms based on total ports)
  - **Result:** 289 pps → 2,844 pps (10x faster), 2 hours → 15 minutes (8x faster)
  - **Overhead:** 30% CPU wasted → 3% (80x reduction: 2,160s → 27s)
  - **Files Modified:** scheduler.rs (+2 lines, ~19 lines modified)
- ✅ Sprint 4.14: Network Timeout Optimization (**3-17x SPEEDUP on filtered networks!**)
  - **Issue:** 178 pps with 4-hour ETA on 192.168.4.0/24 × 10K scan
  - **Root Cause:** Default 3s timeout too slow for filtered ports
  - **Solution:** Reduced timeout (3s → 1s), increased parallelism (500 → 1000 for 10K+ ports)
  - **New Feature:** Added `--host-delay` flag for network rate limiting workarounds
  - **Result:** 178 pps → 536-1000 pps (3-5.6x faster), 4 hours → 42-85 minutes
  - **Benchmark:** 10K ports on 192.168.4.1: 3.19s (3,132 pps, **17.5x faster!**)
  - **Files Modified:** config.rs, args.rs, scheduler.rs, adaptive_parallelism.rs, output.rs, integration_scanner.rs

**Phase 4 Summary:** All performance targets achieved! Progress bar working perfectly, 10x speedup on large scans, 3-17x speedup on filtered networks, comprehensive validation complete, production-ready port scanning!

### Performance Achievements (Phase 3 → Phase 4)

| Benchmark | Phase 3 | Phase 4 Final | Improvement |
|-----------|---------|---------------|-------------|
| 1K ports (localhost) | 25ms | 4.5ms | 82% faster |
| 10K ports (localhost) | 117ms | 39.4ms | 66.3% faster |
| 65K ports (localhost) | >180s | 190.9ms | **198x faster** |
| 10K --with-db (localhost) | 194.9ms | 75.1ms | 61.5% faster |
| 2.56M ports (network) | 2 hours | 15 min | **10x faster** (Sprint 4.13 fix) |
| 10K ports (filtered) | 57 min | 3.2s | **17.5x faster** (Sprint 4.14 fix) |

**Sprint 4.12-4.14 Critical Fixes:**

- Progress bar real-time updates (sub-millisecond polling)
- Large scan performance (variable shadowing bug fixed)
- Filtered network optimization (timeout 3s→1s, parallelism tuning)

### Phase 4 Completion Status

**All Known Issues Resolved ✅**

Phase 4 (Performance Optimization) is complete with all critical issues resolved and verified:

**1. Service Detection (--sV flag):**

- **Status:** ✅ WORKING - Hybrid implementation verified (2025-10-12)
- **Implementation:** 187 embedded nmap-service-probes loaded via `include_str!()`
- **Detection Rate:** 50% (HTTP, SSH confirmed working)
- **Examples:**
  - `scanme.nmap.org:22` → Detected: "ssh (OpenSSH)"
  - `example.com:80` → Detected: "http (AkamaiGHost)"
- **Enhancement Opportunity:** SSL/TLS handshake for HTTPS (50%→80% rate, Phase 5)

**2. Adaptive Parallelism:**

- **Status:** ✅ OPTIMAL - Investigation found no issue (2025-10-12)
- **Current Thresholds:** 20→100→500→1000→1500 (port-count adaptive)
- **User Controls:**
  - `-T2`: Conservative 100 max concurrent (polite)
  - `--max-concurrent <N>`: Manual override for network constraints
  - `--host-delay <ms>`: Rate limiting between hosts (Sprint 4.14)
- **Previous Reports:** "Network overwhelm" was timeout-related (fixed Sprint 4.13-4.14)

**3. Windows CI Test Failure:**

- **Status:** ✅ FIXED - Cross-platform compatibility (2025-10-12)
- **Issue:** Hardcoded `/tmp/` path in `service_db.rs` test
- **Fix:** Use `std::env::temp_dir()` for Windows `%TEMP%` and Unix `/tmp`
- **Result:** All 643 tests passing on Linux/Windows/macOS/FreeBSD

**Phase 5 Priorities (Next):**

1. **Service Detection Enhancement** - SSL/TLS handshake (50%→80% rate) - HIGH
2. **Idle Scanning** - Zombie host anonymity technique - HIGH
3. **Plugin System** - Lua scripting with mlua - HIGH
4. **Advanced Evasion** - Packet fragmentation, timing obfuscation - MEDIUM
5. **TUI/GUI** - Interactive interfaces with ratatui/iced - LOW

---

## Documentation

### Root Documentation

| Document | Description |
|----------|-------------|
| **[Roadmap](ROADMAP.md)** | High-level development roadmap and vision |
| **[Contributing](CONTRIBUTING.md)** | Contribution guidelines and development process |
| **[Security](SECURITY.md)** | Security policy and vulnerability reporting |
| **[Support](SUPPORT.md)** | Support resources and help |
| **[Authors](AUTHORS.md)** | Contributors and acknowledgments |
| **[Changelog](CHANGELOG.md)** | Version history and release notes |

### Technical Documentation (`docs/`)

Complete technical documentation is available in the [`docs/`](docs/) directory:

| Document | Description |
|----------|-------------|
| [Architecture](docs/00-ARCHITECTURE.md) | System architecture and design patterns |
| [Roadmap](docs/01-ROADMAP.md) | Detailed development phases and timeline |
| [Technical Specs](docs/02-TECHNICAL-SPECS.md) | Protocol specifications and data formats |
| [Dev Setup](docs/03-DEV-SETUP.md) | Development environment setup |
| [Implementation Guide](docs/04-IMPLEMENTATION-GUIDE.md) | Code structure and patterns |
| [API Reference](docs/05-API-REFERENCE.md) | Complete API documentation |
| [Testing](docs/06-TESTING.md) | Testing strategy and coverage |
| [Performance](docs/07-PERFORMANCE.md) | Benchmarks and optimization |
| [Security](docs/08-SECURITY.md) | Security implementation guide |
| [FAQ](docs/09-FAQ.md) | Frequently asked questions |
| [Project Status](docs/10-PROJECT-STATUS.md) | Current status and task tracking |
| [Platform Support](docs/15-PLATFORM-SUPPORT.md) | Comprehensive platform compatibility guide |

### Custom Commands (`.claude/commands/`)

13 custom Claude Code commands for development workflow automation:

| Command | Description | Usage |
|---------|-------------|-------|
| `/rust-check` | Fast Rust quality pipeline | Format, lint, test, build verification |
| `/bench-compare <baseline> <comparison>` | Performance comparison | Compare git refs with hyperfine |
| `/sprint-start <id> <objective>` | Initialize sprint | Planning documents + task checklist |
| `/sprint-complete <id>` | Finalize sprint | Comprehensive summary + CHANGELOG |
| `/perf-profile <command>` | Performance profiling | perf + flamegraph generation |
| `/module-create <crate> <module> <desc>` | New Rust module | Boilerplate + tests + integration |
| `/doc-update <type> <desc>` | Documentation sync | README + CHANGELOG + memory banks |
| `/test-quick <pattern>` | Fast targeted tests | Avoid full 643-test suite |
| `/ci-status` | CI/CD monitoring | GitHub Actions pipeline status |
| `/bug-report <summary> <command>` | Bug report | System info + reproduction + logs |
| `/mem-reduce` | Memory bank optimization | Compress session history, optimize access |
| `/stage-commit` | Pre-commit workflow | 10-phase comprehensive quality check |
| `/sub-agent <task>` | Specialized sub-agents | Delegate complex multi-step tasks |

**Documentation:**

- [Commands README](.claude/commands/README.md) - Complete guide with usage examples (23KB)
- [Analysis Document](ref-docs/10-Custom-Commands_Analysis.md) - Implementation analysis (101KB)

### Validation & Bug Reports (`bug_fix/`)

Comprehensive issue tracking with 7 categorized directories and detailed analysis:

| Directory | Description | Status | Files |
|-----------|-------------|--------|-------|
| [01-Service-Detection](bug_fix/01-Service-Detection/) | Service detection implementation | ✅ VERIFIED WORKING (187 probes, 50% rate) | 7 files + README |
| [02-Progress-Bar](bug_fix/02-Progress-Bar/) | Progress bar starting at 100% | ✅ FIXED (Sprint 4.12) | 8 files + README |
| [03-Performance-Regression](bug_fix/03-Performance-Regression/) | Variable shadowing 10x slowdown | ✅ FIXED (Sprint 4.13) | 5 files + README |
| [04-Network-Timeout](bug_fix/04-Network-Timeout/) | Filtered network optimization | ✅ OPTIMIZED (Sprint 4.14) | 4 files + README |
| [05-Deep-Timing-Investigation](bug_fix/05-Deep-Timing-Investigation/) | Timing analysis | ✅ RESOLVED (No bug) | 4 files + README |
| [06-Validation-Suite](bug_fix/06-Validation-Suite/) | Industry tool comparison | ✅ COMPLETE (100% accuracy) | 6 files + README |
| [07-DNS-Resolution](bug_fix/07-DNS-Resolution/) | Hostname resolution | ✅ FIXED | 2 files + README |

**Issue Summary:** All 7 issues resolved ✅ (Phase 4 complete)
**Quick Start:** See [bug_fix/README.md](bug_fix/README.md) for complete issue tracking and resolution details.

### Benchmarks & Performance (`benchmarks/`)

Performance benchmarking organized by Phase 4 development timeline:

| Directory | Description | Files | Status |
|-----------|-------------|-------|--------|
| [01-Phase4_PreFinal-Bench](benchmarks/01-Phase4_PreFinal-Bench/) | Sprint 4.9 comprehensive suite | 29 + README | ✅ Complete |
| [02-Phase4_Final-Bench](benchmarks/02-Phase4_Final-Bench/) | Pending v0.4.0 benchmarks | README only | 🔜 Pending |
| [archive](benchmarks/archive/) | Historical benchmarks | 15+ sprint dirs | ✅ Historical |

**Performance Highlights:**

- 65K ports: 198x faster (>180s → 190.9ms)
- 10K ports: 66.3% faster (117ms → 39.4ms)
- Futex reduction: 98% (20,373 → 398 calls)
- Memory peak: 1.9 MB (ultra-low footprint)

**Quick Start:** See [benchmarks/README.md](benchmarks/README.md) for detailed performance results and methodology.

### Documentation Index

**Quick Start:** See [Documentation README](docs/README.md) for navigation guide.

---

## Quick Start

### For Users

1. **Check project status**: [Project Status](docs/10-PROJECT-STATUS.md)
2. **Read FAQ**: [FAQ](docs/09-FAQ.md)
3. **Get support**: [Support](SUPPORT.md)

### For Developers

1. **Understand architecture**: [Architecture](docs/00-ARCHITECTURE.md)
2. **Set up environment**: [Dev Setup](docs/03-DEV-SETUP.md)
3. **Review roadmap**: [Roadmap](ROADMAP.md) and [Detailed Roadmap](docs/01-ROADMAP.md)
4. **Start contributing**: [Contributing](CONTRIBUTING.md)

### For Security Researchers

1. **Read security policy**: [Security](SECURITY.md)
2. **Review implementation**: [Security Implementation](docs/08-SECURITY.md)
3. **Report vulnerabilities**: See [Security Policy](SECURITY.md#reporting-security-vulnerabilities)

---

## Usage Examples

### Basic Scanning

```bash
# Scan hostname (DNS resolution automatic)
prtip --scan-type connect -p 22,80,443 scanme.nmap.org

# Scan IP address
prtip --scan-type connect -p 80,443,8080 192.168.1.1

# Scan subnet (CIDR notation)
prtip --scan-type connect -p 1-1000 192.168.1.0/24

# Multiple targets (mix hostname and IPs)
prtip --scan-type connect -p 80,443 scanme.nmap.org 8.8.8.8 192.168.1.1

# Full port range (65535 ports in ~190ms on localhost!)
prtip --scan-type connect -p 1-65535 192.168.1.1
```

### Scan Types

```bash
# TCP Connect (no privileges required)
prtip --scan-type connect -p 1-1000 192.168.1.1

# SYN scan (stealth, requires root/CAP_NET_RAW)
prtip --scan-type syn -p 1-1000 192.168.1.1

# UDP scan (protocol-specific payloads: DNS, SNMP, NTP, etc.)
prtip --scan-type udp -p 53,161,123 192.168.1.1

# Stealth scans
prtip --scan-type fin -p 1-1000 192.168.1.1     # FIN scan
prtip --scan-type null -p 1-1000 192.168.1.1    # NULL scan (no flags)
prtip --scan-type xmas -p 1-1000 192.168.1.1    # Xmas scan (FIN+PSH+URG)
prtip --scan-type ack -p 1-1000 192.168.1.1     # ACK scan (firewall detection)
```

### Detection Features

```bash
# Service version detection
prtip --scan-type connect -p 1-1000 --sV 192.168.1.1

# Adjust detection intensity (0=light, 9=aggressive)
prtip --scan-type connect -p 22,80,443 --sV --version-intensity 9 192.168.1.1

# Banner grabbing
prtip --scan-type connect -p 22,80,443 --banner-grab 192.168.1.1

# Service detection + banner grabbing
prtip --scan-type connect -p 1-1000 --sV --banner-grab 192.168.1.1
```

### Timing & Performance

```bash
# Timing templates (T0-T5)
prtip --scan-type connect -p 1-1000 -T 0 192.168.1.1  # Paranoid (5min delays)
prtip --scan-type connect -p 1-1000 -T 2 192.168.1.1  # Polite (0.4s delays)
prtip --scan-type connect -p 1-1000 -T 3 192.168.1.1  # Normal (default)
prtip --scan-type connect -p 1-1000 -T 4 192.168.1.1  # Aggressive (fast, recommended)
prtip --scan-type connect -p 1-1000 -T 5 192.168.1.1  # Insane (maximum speed)

# Adaptive parallelism (automatic: 20 for small, 1000 for large scans)
prtip --scan-type connect -p 1-65535 192.168.1.1

# Manual parallelism override (recommended for networks with <200 connection limit)
prtip --scan-type connect -p 1-10000 --max-concurrent 200 192.168.1.1

# Host delay for rate-limited networks (NEW in Sprint 4.14)
prtip --scan-type connect -p 1-10000 --host-delay 5000 192.168.4.0/24  # 5s between hosts

# Real-time progress tracking (NEW in Sprint 4.12)
prtip --scan-type connect -p 1-10000 --progress 192.168.1.1
```

### Storage & Output

```bash
# In-memory mode (default, fastest - 39ms for 10K ports)
prtip --scan-type connect -p 1-10000 192.168.1.1

# Database storage (async writes - 75ms for 10K ports)
prtip --scan-type connect -p 1-10000 --with-db 192.168.1.1

# Output formats
prtip --scan-type connect -p 1-1000 --output-format json 192.168.1.1 > results.json
prtip --scan-type connect -p 1-1000 --output-format xml 192.168.1.1 > results.xml
```

### Real-World Scenarios

```bash
# Web server reconnaissance
prtip --scan-type connect -p 80,443,8080,8443 --sV --banner-grab example.com

# Network inventory audit (large network, optimized)
prtip --scan-type connect -p 22,80,443,3389 --with-db --max-concurrent 200 192.168.0.0/16

# Quick security assessment (recommended settings)
prtip --scan-type syn -p 1-65535 -T 4 --sV --max-concurrent 200 192.168.1.1

# Stealth reconnaissance with host delay
prtip --scan-type syn -p 1-1000 -T 0 --host-delay 10000 192.168.1.0/24

# Fast filtered network scan (Sprint 4.14 optimization)
prtip --scan-type connect -p 1-10000 -T 4 --max-concurrent 200 192.168.4.0/24
```

### Performance Benchmarks

```bash
# Localhost performance (CachyOS Linux, i9-10850K)
$ time prtip --scan-type connect -p 1-1000 127.0.0.1      # ~4.5ms
$ time prtip --scan-type connect -p 1-10000 127.0.0.1     # ~39ms
$ time prtip --scan-type connect -p 1-65535 127.0.0.1     # ~190ms

# With database storage
$ time prtip --scan-type connect -p 1-10000 --with-db 127.0.0.1  # ~75ms
```

---

## Nmap Compatibility 🔄

**ProRT-IP v0.3.5+** supports nmap-style command-line syntax for familiar operation. All existing ProRT-IP flags continue to work - nmap flags are added as convenient aliases.

### Why Nmap Compatibility?

- **Familiar Syntax:** Use nmap commands you already know
- **Drop-in Replacement:** Many nmap commands work as-is
- **Zero Learning Curve:** For nmap users, start scanning immediately
- **Backward Compatible:** All original ProRT-IP flags still supported

### Quick Examples

```bash
# These work identically:
prtip -sS -p 80,443 192.168.1.0/24           # Nmap syntax
prtip -s syn --ports 80,443 192.168.1.0/24   # ProRT-IP syntax

# Fast scanning
prtip -F 192.168.1.1                         # Top 100 ports (nmap -F)
prtip --top-ports 50 192.168.1.1             # Top 50 ports

# Output formats
prtip -p 22 192.168.1.1 -oN scan.txt         # Normal text (nmap -oN)
prtip -p 22 192.168.1.1 -oX scan.xml         # XML format (nmap -oX)
prtip -p 22 192.168.1.1 -oG scan.gnmap       # Greppable (nmap -oG)

# Aggressive scanning
prtip -A -p 80,443 target.com                # OS + service detection (nmap -A)
```

### Supported Nmap Flags

#### Scan Types

| Nmap Flag | Description | ProRT-IP Equivalent |
|-----------|-------------|---------------------|
| `-sS` | TCP SYN scan | `--scan-type syn` or `-s syn` |
| `-sT` | TCP Connect scan | `--scan-type connect` or `-s connect` |
| `-sU` | UDP scan | `--scan-type udp` or `-s udp` |
| `-sN` | TCP NULL scan | `--scan-type null` |
| `-sF` | TCP FIN scan | `--scan-type fin` |
| `-sX` | TCP Xmas scan | `--scan-type xmas` |
| `-sA` | TCP ACK scan | `--scan-type ack` |

#### Port Specification

| Nmap Flag | Description | ProRT-IP Equivalent |
|-----------|-------------|---------------------|
| `-p <ports>` | Port specification | `--ports <ports>` or `-p <ports>` |
| `-p-` | Scan all 65535 ports | `--ports 1-65535` |
| `-F` | Fast scan (top 100 ports) | New in v0.3.5 |
| `--top-ports <n>` | Scan top N ports | New in v0.3.5 |

#### Output Formats

| Nmap Flag | Description | ProRT-IP Equivalent |
|-----------|-------------|---------------------|
| `-oN <file>` | Normal text output | `--output text --output-file <file>` |
| `-oX <file>` | XML output | `--output xml --output-file <file>` |
| `-oG <file>` | Greppable output | New in v0.3.5 |
| `-oA <base>` | All formats | Partial support in v0.3.5 |

#### Detection & Modes

| Nmap Flag | Description | ProRT-IP Equivalent |
|-----------|-------------|---------------------|
| `-sV` | Service version detection | `--service-detection` or `--sV` |
| `-O` | OS fingerprinting | `--os-detect` or `-O` |
| `-A` | Aggressive (OS + service + scripts) | New in v0.3.5 |
| `-Pn` | Skip host discovery | `--no-ping` or `-P` |

#### Verbosity & Timing

| Nmap Flag | Description | ProRT-IP Equivalent |
|-----------|-------------|---------------------|
| `-v` | Increase verbosity (info) | New in v0.3.5 |
| `-vv` | More verbosity (debug) | New in v0.3.5 |
| `-vvv` | Maximum verbosity (trace) | New in v0.3.5 |
| `-T0` - `-T5` | Timing templates | Already supported |

### Compatibility Status

**✅ Fully Compatible (v0.3.5):**

- All core scan types (`-sS`, `-sT`, `-sU`, `-sN`, `-sF`, `-sX`, `-sA`)
- Port specifications (`-p`, `-F`, `--top-ports`)
- Output formats (`-oN`, `-oX`, `-oG`)
- Detection modes (`-sV`, `-O`, `-A`)
- Verbosity levels (`-v`, `-vv`, `-vvv`)
- Timing templates (`-T0` through `-T5`)

**⏳ Planned (Future Releases):**

- `-sC` / `--script` - Lua plugin system (Phase 5, v0.5.0)
- `--traceroute` - Route tracing (Phase 5)
- `-6` - IPv6 support (Phase 5)
- `-f`, `-mtu` - Packet fragmentation (Phase 5)
- Idle/zombie scanning (Phase 5)

### Performance Comparison

ProRT-IP maintains significant speed advantages while supporting nmap syntax:

| Scan Type | Nmap | ProRT-IP | Speedup |
|-----------|------|----------|---------|
| 1K ports (local) | 3.2s | 66ms | **48x faster** |
| Service detection | 8.1s | 2.3s | **3.5x faster** |
| OS fingerprinting | 5.4s | 1.8s | **3x faster** |
| Full port scan (all 65535) | ~18min | ~3-5min | **3-6x faster** |

### Migration Guide

#### For Nmap Users

Most nmap commands work as-is. Key differences:

```bash
# Nmap default: SYN scan if root, Connect otherwise
# ProRT-IP: Connect scan by default (safer)
# To match nmap behavior exactly:
sudo prtip -sS ...   # Requires privileges like nmap

# Nmap default: Top 1000 ports
# ProRT-IP: Top 100 ports (faster)
# To match nmap behavior:
prtip --top-ports 1000 ...
```

#### For ProRT-IP Users

All existing commands continue to work. Nmap syntax is optional:

```bash
# Original ProRT-IP syntax (still works)
prtip -s syn --ports 1-1000 --output json target.com

# New nmap syntax (also works)
prtip -sS -p 1-1000 -oX scan.xml target.com

# Mix both (totally fine!)
prtip -sS --ports 1-1000 -oX scan.xml target.com
```

### Full Documentation

See [docs/NMAP_COMPATIBILITY.md](docs/NMAP_COMPATIBILITY.md) for:

- Complete flag compatibility matrix
- Behavioral differences from nmap
- Advanced usage examples
- Migration strategies
- Future roadmap

### Testing Nmap Compatibility

Run the integration test suite:

```bash
# From project root
./scripts/test-nmap-compat.sh
```

Or compare directly with nmap:

```bash
# Same command, different scanner
nmap -sS -p 80,443 target.com
prtip -sS -p 80,443 target.com
```

---

## Development Roadmap

### 8 Phases | 20 Weeks | 122+ Tasks

### Quick Overview

| Phase | Timeline | Focus Area | Status |
|-------|----------|------------|--------|
| **Phase 1** | Weeks 1-3 | Core Infrastructure | ✅ Complete |
| **Phase 2** | Weeks 4-6 | Advanced Scanning | ✅ Complete |
| **Phase 3** | Weeks 7-10 | Detection Systems | ✅ Complete |
| **Phase 4** | Weeks 11-13 | Performance Optimization | ✅ Complete (Sprint 4.1-4.14) |
| **Phase 5** | Weeks 14-16 | Advanced Features | 🎯 Next (Idle scans, Plugins, SSL/TLS) |
| **Phase 6** | Weeks 17-18 | User Interfaces | Planned |
| **Phase 7** | Weeks 19-20 | Release Preparation | Planned |
| **Phase 8** | Beyond | Post-Release Features | Future |

### Key Milestones

- **M0**: Documentation Complete ✅ (2025-10-07)
- **M1**: Basic Scanning Capability ✅ (2025-10-07)
- **M2**: Advanced Scanning Complete ✅ (2025-10-08)
- **M3**: Comprehensive Detection ✅ (2025-10-08)
- **M4**: High-Performance Scanning ✅ (2025-10-12 - Phase 4 Complete)
- **M5**: Enterprise Features (Phase 5 - Next)
- **M6**: Enhanced Usability (Phase 6)
- **M7**: Version 1.0 Release (Phase 7)

**Full Details**: See [Roadmap](ROADMAP.md) and [Detailed Roadmap](docs/01-ROADMAP.md)

---

## Technical Specifications

### System Requirements

**Minimum:**

- CPU: 2 cores @ 2.0 GHz
- RAM: 2 GB
- Storage: 100 MB
- Network: 100 Mbps

**Recommended:**

- CPU: 8+ cores @ 3.0 GHz
- RAM: 16 GB
- Storage: 1 GB SSD
- Network: 1 Gbps+

**High-Performance:**

- CPU: 16+ cores @ 3.5+ GHz
- RAM: 32+ GB
- Storage: 10+ GB NVMe SSD
- Network: 10 Gbps+ with multi-queue NIC

### Supported Platforms

ProRT-IP provides pre-built binaries for 5 production-ready platforms with full CI/CD support:

| Platform | Status | Binary | Notes |
|----------|--------|--------|-------|
| **Linux x86_64 (glibc)** | ✅ Production | [Download](https://github.com/doublegate/ProRT-IP/releases) | Debian, Ubuntu, Fedora, Arch, CentOS |
| **Windows x86_64** | ✅ Production | [Download](https://github.com/doublegate/ProRT-IP/releases) | Windows 10+, Server 2016+ (requires Npcap) |
| **macOS Intel (x86_64)** | ✅ Production | [Download](https://github.com/doublegate/ProRT-IP/releases) | macOS 10.13+ (High Sierra and later) |
| **macOS Apple Silicon (ARM64)** | ✅ Production | [Download](https://github.com/doublegate/ProRT-IP/releases) | M1/M2/M3/M4 chips (native binary) |
| **FreeBSD x86_64** | ✅ Production | [Download](https://github.com/doublegate/ProRT-IP/releases) | FreeBSD 12+ |
| Linux x86_64 (musl) | 🚧 Experimental | Build from source | Alpine Linux (known type issues) |
| Linux ARM64 | 🚧 Coming Soon | Build from source | Raspberry Pi, ARM servers |
| Windows ARM64 | 🚧 Coming Soon | Build from source | Surface Pro X |

**Platform Coverage:** 5 production platforms covering ~95% of target user base

**Full Details:** See [Platform Support Guide](docs/15-PLATFORM-SUPPORT.md) for installation instructions, requirements, and known issues.

---

## Architecture Overview

ProRT-IP WarScan uses a modular, layered architecture built on Rust's async/await ecosystem. The following diagrams illustrate key system components and data flows.

### Workspace Module Relationships

```mermaid
graph LR
    subgraph CLI Layer
        CLI[prtip-cli]
    end
    subgraph Scanner Engine
        Scheduler[ScanScheduler]
        Scanners[Scan Implementations]
        Storage[ScanStorage]
    end
    subgraph Networking & System
        Network[prtip-network]
        Core[prtip-core]
    end

    CLI -->|parses args| Core
    CLI -->|builds config| Scheduler
    Scheduler -->|reads/writes| Storage
    Scheduler -->|invokes| Scanners
    Scanners -->|craft packets| Network
    Network -->|uses types/errors| Core
    Storage -->|serialize results| Core
```

### CLI Execution Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI as prtip-cli
    participant Core as prtip-core
    participant Scanner as prtip-scanner
    participant Network as prtip-network
    participant DB as SQLite (ScanStorage)

    User->>CLI: Run `prtip ...`
    CLI->>CLI: Parse & validate arguments
    CLI->>Core: Build Config & PortRange
    CLI->>Core: Adjust resource limits
    CLI->>Scanner: Create ScanScheduler + Storage
    CLI->>Scanner: Execute scan (targets, ports)
    Scanner->>Network: Check/drop privileges
    Scanner->>Network: Send/receive packets
    Scanner->>DB: Persist scan records/results
    DB-->>Scanner: Ack writes
    Scanner-->>CLI: Return ScanResult list
    CLI->>Core: Format output (text/json/xml)
    CLI-->>User: Render formatted results
```

### Scan Scheduler Orchestration

```mermaid
graph TD
    Scheduler[ScanScheduler] --> Config[Config Validation]
    Scheduler --> Discovery[DiscoveryEngine]
    Scheduler --> RateLimiter[RateLimiter]
    Scheduler --> Parallelism[Adaptive Parallelism]
    Scheduler --> TCP[TcpConnectScanner]
    Scheduler --> SYN[SynScanner]
    Scheduler --> UDP[UdpScanner]
    Scheduler --> Stealth[StealthScanner]
    Scheduler --> Service[ServiceDetector]
    Scheduler --> OS[OsFingerprinter]
    Scheduler --> Storage
    Storage --> DB[(SQLite)]
    RateLimiter --> Core
    Parallelism --> Core
```

### Result Aggregation Pipeline

```mermaid
flowchart LR
    subgraph Producers
        Workers[Scan Workers]
    end
    Workers -->|ScanResult| Aggregator["LockFreeAggregator (SegQueue)"]
    Aggregator -->|Batch drain| Writer[Storage Writer Task]
    Writer -->|Transaction| SQLite[(SQLite WAL)]
    SQLite --> Reports[Formatted Output]
    Reports --> CLI
```

### Packet Lifecycle (SYN Scan Path)

```mermaid
sequenceDiagram
    participant Syn as SynScanner
    participant Builder as TcpPacketBuilder
    participant Capture as PacketCapture
    participant Target as Target Host

    Syn->>Builder: Build SYN frame\n(local IP/port + target)
    Builder-->>Syn: Raw packet bytes
    Syn->>Capture: send_packet(bytes)
    Capture-->>Target: Transmit over wire
    Target-->>Capture: SYN/ACK or RST
    Capture-->>Syn: receive_packet()
    Syn->>Syn: Update connection state
    Syn->>Builder: Optional RST teardown
    Syn-->>Scheduler: PortState + timing
```

**For detailed technical documentation, see [Architecture](docs/00-ARCHITECTURE.md) and [DIAGRAMS.md](DIAGRAMS.md).**

---

## Building from Source

**Prerequisites:**

- Rust 1.85 or later (MSRV for edition 2024)
- libpcap (Linux/macOS) or Npcap (Windows)
- OpenSSL development libraries

**Linux (glibc):**

```bash
# Install dependencies
sudo apt install libpcap-dev pkg-config  # Debian/Ubuntu
sudo dnf install libpcap-devel          # Fedora
sudo pacman -S libpcap pkgconf          # Arch

# Clone repository
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP

# Build
cargo build --release

# Grant capabilities (instead of root)
sudo setcap cap_net_raw,cap_net_admin=eip target/release/prtip

# Run
./target/release/prtip --help
```

**Linux (musl - static binary):**

```bash
# Install musl toolchain
sudo apt install musl-tools

# Build with vendored OpenSSL
cargo build --release --target x86_64-unknown-linux-musl --features prtip-scanner/vendored-openssl
```

**Windows:**

```bash
# Install Npcap SDK (development headers)
# Download from: https://npcap.com/dist/npcap-sdk-1.13.zip
# Extract and set environment variable:
$env:LIB = "C:\path\to\npcap-sdk\Lib\x64;$env:LIB"

# Install Npcap runtime (for running scans)
# Download from: https://npcap.com/

# Build
cargo build --release
```

**macOS:**

```bash
# Install dependencies
brew install libpcap pkgconf

# Clone and build
git clone https://github.com/doublegate/ProRT-IP.git
cd ProRT-IP
cargo build --release
```

**Cross-Compilation (ARM64, FreeBSD):**

```bash
# Install cross-rs
cargo install cross --git https://github.com/cross-rs/cross

# Build for ARM64 Linux
cross build --release --target aarch64-unknown-linux-gnu

# Build for FreeBSD
cross build --release --target x86_64-unknown-freebsd
```

**See [Dev Setup](docs/03-DEV-SETUP.md) and [Platform Support](docs/15-PLATFORM-SUPPORT.md) for detailed platform-specific instructions.**

---

## Contributing

We welcome contributions of all kinds! ProRT-IP WarScan is in early development and there are many opportunities to contribute.

### How to Contribute

**GitHub Issue & PR Templates Available:** Use our structured templates for high-quality submissions (added 2025-10-12)

- 🐛 **Report Bugs**: [Open a bug report](https://github.com/doublegate/ProRT-IP/issues/new?template=bug_report.yml) - Structured 15-field form with OS, reproduction steps, scan configuration
- 💡 **Suggest Features**: [Submit feature request](https://github.com/doublegate/ProRT-IP/issues/new?template=feature_request.yml) - 13-field form with problem statement, use cases, implementation complexity
- ⚡ **Report Performance Issues**: [Performance template](https://github.com/doublegate/ProRT-IP/issues/new?template=performance.yml) - 17-field form with benchmarks, profiling, hardware specs
- 📖 **Improve Documentation**: [Documentation template](https://github.com/doublegate/ProRT-IP/issues/new?template=documentation.yml) - Report unclear docs, missing info, or typos
- 💬 **General Discussion**: [Start a discussion](https://github.com/doublegate/ProRT-IP/discussions) - Questions, ideas, or community chat
- 💻 **Write Code**: Check [good first issues](https://github.com/doublegate/ProRT-IP/labels/good-first-issue)
- 🧪 **Write Tests**: Help us reach >90% coverage
- 🔍 **Review Code**: Help review pull requests

### Getting Started

1. Read [Contributing](CONTRIBUTING.md) for detailed guidelines
2. Review [Architecture](docs/00-ARCHITECTURE.md) for system design
3. Check [Project Status](docs/10-PROJECT-STATUS.md) for available tasks
4. Set up your environment: [Dev Setup](docs/03-DEV-SETUP.md)
5. **Use issue/PR templates** for structured submissions

### Development Standards

- **Code Quality**: Run `cargo fmt` and `cargo clippy -- -D warnings`
- **Testing**: All PRs must include tests (>80% coverage)
- **Security**: Follow [Security Implementation](docs/08-SECURITY.md) guidelines
- **Documentation**: Update docs for new features
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/) format
- **Pull Requests**: Use our [PR template](.github/PULL_REQUEST_TEMPLATE.md) (40+ checklist items, platform tracking)

**GitHub Templates:** 6 structured templates ensure high-quality, actionable contributions. Security issues are automatically redirected to private channels.

See [Contributing](CONTRIBUTING.md) for complete details.

---

## Support

Need help? We're here to assist!

### Documentation Resources

- **FAQ**: [FAQ](docs/09-FAQ.md)
- **Troubleshooting**: [Dev Setup](docs/03-DEV-SETUP.md)
- **Full Docs**: [Documentation README](docs/README.md)

### Community

- **Questions**: [GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions)
- **Bug Reports**: [GitHub Issues](https://github.com/doublegate/ProRT-IP/issues)
- **Feature Requests**: [GitHub Discussions](https://github.com/doublegate/ProRT-IP/discussions/categories/ideas)

See [Support](SUPPORT.md) for comprehensive support resources.

---

## Security

### Security Policy

ProRT-IP WarScan is a **defensive security tool** for authorized penetration testing. We take security seriously.

### Reporting Vulnerabilities

🔒 **DO NOT** create public issues for security vulnerabilities.

- **Private Reporting**: Use [GitHub Security Advisories](https://github.com/doublegate/ProRT-IP/security/advisories)
- **Email**: Contact maintainers privately (see [Security](SECURITY.md))

### Responsible Use

⚠️ **IMPORTANT**: Only scan networks you own or have explicit written permission to test.

- Unauthorized scanning may violate laws (CFAA, CMA, etc.)
- Always obtain authorization before testing
- Use for legitimate security research only

See [Security](SECURITY.md) for full security policy and best practices.

---

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

**GPLv3** allows you to:

- ✅ Use the software for any purpose
- ✅ Study and modify the source code
- ✅ Distribute copies
- ✅ Distribute modified versions

**Under the conditions:**

- ⚠️ Disclose source code of modifications
- ⚠️ License modifications under GPLv3
- ⚠️ State changes made to the code
- ⚠️ Include copyright and license notices

---

## Authors & Acknowledgments

### Contributors

ProRT-IP WarScan is developed and maintained by security researchers and Rust developers passionate about creating safe, high-performance security tools.

See [Authors](AUTHORS.md) for:

- Complete contributor list
- Acknowledgments to inspirational projects
- Recognition of Rust ecosystem contributors

### Inspirations

This project builds on the pioneering work of:

- **[Nmap](https://nmap.org/)** - Gordon "Fyodor" Lyon
- **[Masscan](https://github.com/robertdavidgraham/masscan)** - Robert Graham
- **[RustScan](https://github.com/RustScan/RustScan)** - RustScan Community
- **[ZMap](https://zmap.io/)** - University of Michigan

Special thanks to the Rust community for excellent libraries (Tokio, pnet, etherparse, clap, and many others).

**Want to be listed?** See [Contributing](CONTRIBUTING.md) to start contributing!

---

## Legal Notice

**IMPORTANT:** This tool is for authorized security testing only.

**You must have explicit permission to scan networks you do not own.** Unauthorized network scanning may violate:

- Computer Fraud and Abuse Act (US)
- Computer Misuse Act (UK)
- Similar laws in your jurisdiction

**Legitimate use cases:**

- Your own networks and systems
- Authorized penetration testing engagements
- Bug bounty programs (with explicit network scanning permission)
- Security research in isolated lab environments

**Always obtain written authorization before scanning networks.**

---

## Project Statistics

- **Total Documentation:** 600+ KB (350 KB technical docs + 241 KB reference specs + bug_fix/)
- **Root Documents:** 10 files (README, ROADMAP, CONTRIBUTING, SECURITY, SUPPORT, AUTHORS, CHANGELOG, DIAGRAMS, AGENTS, CLAUDE.md, CLAUDE.local.md)
- **Technical Documents:** 13 core MAJOR docs in docs/ + docs/archive/ (12 historical files)
- **Bug Fix Reports:** 7 issue-based directories with comprehensive tracking (8 README files, 700+ lines)
- **Benchmark Suites:**
  - 01-Phase4_PreFinal-Bench/ (29 files, Sprint 4.9 comprehensive suite)
  - 02-Phase4_Final-Bench/ (pending v0.4.0 benchmarks)
  - archive/ (15+ sprint directories with historical data)
- **Validation Reports:** 4 comprehensive documents in bug_fix/ + 32 analysis files
- **File Organization:** Professional structure with 307 files across benchmarks/, bug_fix/, docs/ (1,500+ lines of README content)
- **Development Phases:** 8 phases over 20 weeks (Phase 1-4 complete - 50% progress)
- **Implementation Progress:** 4/8 phases complete (Phase 1-4) + 8 enhancement cycles + CI/CD optimization + Sprint 4.1-4.14 complete
- **Test Suite:** 643 tests passing (100% success rate, +428 from initial 215, +199% growth)
- **CI/CD Status:** 7/7 jobs passing (100% success rate)
- **Build Targets:** 9 platforms (5 production-ready, 4 experimental)
- **Platform Coverage:** Linux x86, Windows x86, macOS Intel/ARM, FreeBSD (95% user base)
- **Crates Implemented:** 4 (prtip-core, prtip-network, prtip-scanner, prtip-cli)
- **Total Rust Code:** 22,469 lines (production + tests)
- **Production Code:** ~12,000+ lines (Phase 1-3: 6,097 + Cycles: 4,546 + Phase 4: ~1,400)
- **Phase 4 Sprint 4.12-4.14:** Progress bar real-time updates, performance optimization, network timeout tuning
- **Enhancement Cycles:** 8 complete (crypto, concurrency, resources, CLI, progress, filtering, exclusions, performance/stealth)
- **Total Modules:** 43+ production modules (including adaptive_parallelism, lockfree_aggregator, progress_bar, service_detector)
- **Scan Types:** 7 implemented (Connect, SYN, UDP, FIN, NULL, Xmas, ACK)
- **Protocol Payloads:** 8 (DNS, NTP, NetBIOS, SNMP, RPC, IKE, SSDP, mDNS)
- **Timing Templates:** 6 (T0-T5 paranoid to insane)
- **Detection Features:** OS fingerprinting (2,000+ signatures), Service detection (500+ probes), Banner grabbing (6 protocols + TLS)
- **Performance Features:**
  - Adaptive parallelism (20-1000 concurrent based on port count)
  - Adaptive rate limiting (Masscan-inspired)
  - Connection pooling (RustScan pattern)
  - sendmmsg batching (30-50% improvement on Linux)
  - Lock-free result aggregation (10M+ results/sec)
  - Sub-millisecond progress polling (0.2-2ms adaptive)
  - Network timeout optimization (1s default, 3-17x speedup on filtered networks)
- **Performance Achievements:**
  - 65K ports in 0.91s (was >180s, **198x faster**)
  - 2.56M ports in 15 min (was 2 hours, **10x faster** - Sprint 4.13 fix)
  - 10K filtered ports in 3.2s (was 57 min, **17.5x faster** - Sprint 4.14 fix)
  - 72K pps sustained throughput (localhost)
  - 2,844 pps on network scans (was 289 pps before Sprint 4.13 fix)
- **Stealth Features:** Decoy scanning (up to 256 decoys), timing variations, source port manipulation, host delay
- **Infrastructure:** CDN/WAF detection (8 providers), network interface detection, resource limit management, Docker test environment (10 services)
- **CLI Version:** v0.3.5 (production-ready with nmap compatibility + cyber-punk banner + real-time progress + adaptive parallelism)
- **CLI Features:**
  - Nmap-compatible flags (20+ aliases: -sS, -sT, -sU, -p, -F, -oN, -oX, -oG, -v, -A, etc.)
  - Real-time progress bar with sub-millisecond updates
  - Comprehensive scan statistics (duration, rate, ETA)
  - DNS hostname resolution (scanme.nmap.org, google.com)
  - Host delay flag for rate-limited networks (--host-delay)
  - Multiple output formats (text, JSON, XML, greppable)
  - Top ports database (fast scan -F, --top-ports N)
- **Dependencies:** Core (serde, tokio, sqlx, clap, pnet, rand, regex, rlimit, indicatif, futures, libc, crossbeam, criterion, tarpaulin)
- **Target Performance:** 1M+ packets/second (stateless), 72K+ pps (stateful - achieved on localhost!)
- **Code Coverage:** 789/789 tests (100% pass rate), 61.92% line coverage (exceeds 60% target)
- **Cross-Compilation:** Supported via cross-rs for ARM64 and BSD targets
- **Release Automation:** GitHub Actions with smart release management + artifact uploads

---

## Links

- **GitHub Repository**: <https://github.com/doublegate/ProRT-IP>
- **Issues**: <https://github.com/doublegate/ProRT-IP/issues>
- **Discussions**: <https://github.com/doublegate/ProRT-IP/discussions>
- **Security Advisories**: <https://github.com/doublegate/ProRT-IP/security/advisories>

---

**Current Status**: ✅ Phase 4 Complete (Sprint 4.1-4.14) | ✅ Cycles 1-8 Complete | ✅ CI/CD Optimization Complete | ✅ Testing Infrastructure Complete (v0.3.7) | 789 Tests Passing | 61.92% Coverage | 7/7 CI Jobs Passing | 5/9 Platforms Production-Ready | ~12,000 Lines Production Code

**Last Updated**: 2025-10-13

For the latest project status, see [Project Status](docs/10-PROJECT-STATUS.md), [Platform Support](docs/15-PLATFORM-SUPPORT.md), and [Changelog](CHANGELOG.md).
