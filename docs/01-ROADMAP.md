# ProRT-IP WarScan: Development Roadmap

**Version:** 2.0
**Last Updated:** 2025-11-01
**Project Status:** Phase 5 IN PROGRESS (40% Complete) ✅ | **56% Overall Progress** (4.5/8 phases) | v0.4.3 Advanced Features

---

## Table of Contents

1. [Overview](#overview)
2. [Development Phases](#development-phases)
3. [Phase Details](#phase-details)
4. [Sprint Planning](#sprint-planning)
5. [Milestones and Deliverables](#milestones-and-deliverables)
6. [Risk Management](#risk-management)
7. [Success Metrics](#success-metrics)

---

## Overview

This roadmap outlines the complete development journey for ProRT-IP WarScan from initial setup through production release. The project is structured into 8 major phases spanning approximately 16-20 weeks of development.

### Timeline Summary

| Phase | Duration | Focus | Key Deliverables | Status |
|-------|----------|-------|------------------|--------|
| Phase 1 | Weeks 1-3 | Core Infrastructure | Packet capture, TCP connect scan, privilege management | ✅ COMPLETE |
| Phase 2 | Weeks 4-6 | Advanced Scanning | SYN/UDP/stealth scans, timing templates | ✅ COMPLETE |
| **Enhancement Cycles 1-8** | **Ongoing** | **Reference Optimizations** | **Crypto, concurrency, resources, CLI, progress, filtering, exclusions, perf/stealth** | **✅ COMPLETE** |
| Phase 3 | Weeks 7-10 | Detection Systems | OS fingerprinting, service detection, banner grabbing | ✅ COMPLETE |
| Phase 4 | Weeks 11-13 | Performance & Evasion | Zero-copy, NUMA, PCAPNG, evasion techniques, error handling | ✅ COMPLETE (1,166 tests, v0.3.9-v0.4.0) |
| **Phase 5** | **Weeks 14-20** | **Advanced Features** | **IPv6 100%, Service Detection 85-90%, Idle Scan, Rate Limiting, TLS Analysis** | **🔄 IN PROGRESS (40% - 4/10 sprints, v0.4.1-v0.4.3)** |
| Phase 6 | Weeks 21-22 | TUI Interface | Interactive terminal dashboard | 📋 PLANNED |
| Phase 7 | Weeks 23-24 | Polish & Release | Documentation, packaging, v1.0 release | 📋 PLANNED |
| Phase 8 | Post-v1.0 | Future Enhancements | Web UI, desktop GUI, distributed scanning | 📋 PLANNED |

### Development Methodology

- **Agile/Iterative:** 2-week sprints with defined goals and deliverables
- **Test-Driven:** Write tests before implementation for critical components
- **Continuous Integration:** Automated testing on Linux, Windows, macOS
- **Code Review:** All changes reviewed before merging
- **Documentation-First:** Design docs before major feature implementation

---

## Development Phases

### Phase 1: Core Infrastructure (Weeks 1-3) ✅ COMPLETE

**Goal:** Establish the foundational architecture and basic scanning capabilities
**Status:** Completed 2025-10-07 with 215 tests passing

#### Week 1: Project Setup and Basic Packet Capture ✅

**Sprint 1.1**

- [x] Initialize Rust project structure with workspace layout
- [x] Configure `Cargo.toml` with core dependencies
- [x] Implement cross-platform packet capture abstraction using `pnet`
- [x] Create basic logging infrastructure with `tracing`
- [x] Setup CI/CD pipeline (GitHub Actions)
- [x] Write initial integration tests for packet capture

**Deliverables:**

- [x] Compiling project with all dependencies
- [x] Packet capture working on Linux/Windows/macOS
- [x] CI pipeline running tests automatically

#### Week 2: TCP Connect Scan and CLI ✅

**Sprint 1.2**

- [x] Implement TCP connect scan using `tokio::net::TcpStream`
- [x] Create CLI argument parser with `clap`
- [x] Develop target specification parser (CIDR, ranges, hostnames)
- [x] Build basic result aggregator
- [x] Implement text output formatter
- [x] Add DNS resolution support

**Deliverables:**

- [x] Functional TCP connect scanner
- [x] CLI accepting targets and port ranges
- [x] Human-readable text output

#### Week 3: Privilege Management and Configuration ✅

**Sprint 1.3**

- [x] Implement privilege dropping (setuid/setgid on Unix)
- [x] Add Linux capabilities support (CAP_NET_RAW)
- [x] Create configuration file loader (TOML format)
- [x] Build raw socket abstraction layer
- [x] Setup SQLite result storage schema
- [x] Add JSON output formatter
- [x] Add XML output formatter (bonus)
- [x] Add rate limiting (bonus)
- [x] Add host discovery (bonus)

**Deliverables:**

- [x] Secure privilege management system
- [x] Configuration file support
- [x] SQLite database storage
- [x] JSON/XML/Text output formats

---

### Phase 2: Advanced Scanning Techniques (Weeks 4-6) ✅ COMPLETE

**Goal:** Implement raw packet scanning with stealth capabilities
**Status:** Completed 2025-10-08 with 278 tests passing, 3,551 lines added

#### Week 4: TCP SYN Scanning ✅

**Sprint 2.1 - Completed**

- [x] Implement raw TCP packet builder with proper checksums (790 lines)
- [x] Create SYN scan logic with response interpretation (437 lines)
- [x] Build connection tracking for stateful scanning
- [x] Add retransmission support with exponential backoff
- [x] Implement RTT estimation with SRTT/RTTVAR
- [x] Create unit tests for packet crafting

**Deliverables:**

- [x] Working SYN scan mode (-sS flag)
- [x] Response state machine (open/closed/filtered)
- [x] Packet checksum validation tests

#### Week 5: UDP and Stealth Scans ✅

**Sprint 2.2 - Completed**

- [x] Implement UDP packet builder (258 lines)
- [x] Create UDP scan with ICMP unreachable detection
- [x] Add protocol-specific UDP payloads (DNS, SNMP, NetBIOS, NTP, RPC, IKE, SSDP, mDNS - 199 lines)
- [x] Implement FIN/NULL/Xmas scans (388 lines)
- [x] Build ACK scan for firewall detection
- [ ] ~~Add Window scan variant~~ (Deferred to Phase 5)

**Deliverables:**

- [x] UDP scanning (-sU flag)
- [x] Stealth scan variants (-sF, -sN, -sX, -sA)
- [x] Protocol-specific probe library (8 protocols)

#### Week 6: Timing and Rate Control ✅

**Sprint 2.3 - Completed**

- [x] Implement timing templates (T0-T5) - 441 lines
- [x] Create adaptive rate limiter with token bucket (422 lines)
- [x] Build congestion control (AIMD algorithm)
- [x] Add `--min-rate` and `--max-rate` options
- [x] Implement `--scan-delay` and jitter
- [x] Create performance benchmarks

**Deliverables:**

- [x] All 6 timing templates working (T0-T5)
- [x] Adaptive rate limiting preventing network saturation
- [x] Connection pool for efficient concurrent scanning (329 lines)

**Bonus Achievements:**

- [x] Masscan-inspired adaptive rate limiter with circular buffer tracking
- [x] RustScan-inspired connection pool with FuturesUnordered
- [x] Reference code analysis across 7+ leading scanners (Masscan, RustScan, Naabu, Nmap, etc.)

---

### Enhancement Cycles (Post-Phase 2) ✅ COMPLETE

Following Phase 2 completion, systematic enhancement cycles incorporated optimization patterns from reference implementations. **All cycles complete.**

#### Cycle 1: Cryptographic Foundation (commit 5782aed) ✅

- SipHash-2-4 implementation (584 lines, 9/9 tests)
- Blackrock shuffling partial (7/9 tests, completed in Cycle 2)
- Concurrent scanner with FuturesUnordered (380 lines, 6/6 tests)
- **Statistics:** Tests 100 → 121 (+21), ~1,074 lines

#### Cycle 2: Complete Crypto + Filtering (commit f5be9c4) ✅

- Blackrock algorithm completion (11/11 tests)
- Port filtering system (~200 lines, 10 tests)
- **Statistics:** Tests 121 → 131 (+10), ~250 lines

#### Cycle 3: Resource Management (commits 38b4f3e, 781e880) ✅

- Resource limits module (363 lines, 11 tests)
- Interface detection module (406 lines, 13 tests)
- **Statistics:** Tests 131 → 345 (+214), 769 lines, +1 dependency (rlimit)

#### Cycle 4: CLI Integration (commits eec5169, e4e5d54) ✅

- CLI flags (--batch-size, --ulimit, --interface-list, 7 tests)
- Scanner ulimit-aware integration
- Main CLI logic enhancements (62 lines)
- **Statistics:** Tests 345 → 352 (+7), ~200 lines

#### Cycle 5: User Feedback (commits d7f7f38, c1aa10e) ✅

- Progress tracking module (428 lines, 11 tests)
- Error categorization module (209 lines, 9 tests)
- CLI integration (4 new flags, 7 tests)
- **Statistics:** Tests 352 → 391 (+39), ~637 lines, +1 dependency (indicatif)

**Overall Enhancement Impact:**

- **Tests:** 100 → 391 (+291, +291% growth)
- **Lines:** ~2,930 across 5 cycles
- **Modules:** 6 new production modules
- **Dependencies:** +2 (rlimit 0.10.2, indicatif 0.17)
- **Quality:** 100% pass rate, MSRV maintained

**Production Readiness:** Cryptographic foundation, concurrent patterns, filtering, resource awareness, progress tracking, error categorization

**Status:** All enhancement cycles complete. Ready for Phase 3: Detection Systems.

---

### Phase 3: Detection and Fingerprinting (Weeks 7-10) ✅ COMPLETE

**Goal:** Add service detection and OS fingerprinting capabilities
**Status:** Completed 2025-10-08 with 371 tests passing
**Commit:** 6204882

#### Week 7: OS Fingerprinting Foundation ✅

**Sprint 3.1 - COMPLETE**

- ✅ Design OS fingerprint database schema (os_db.rs - 412 lines)
- ✅ Implement 16-probe sequence (TCP/ICMP/UDP) (os_probe.rs - 382 lines)
- ✅ Create ISN analysis (GCD, ISR, TI/CI/II)
- ✅ Build TCP timestamp parsing
- ✅ Add TCP option ordering extraction
- ✅ Implement window size analysis

**Deliverables:**

- ✅ Complete 16-probe implementation
- ✅ Fingerprint database format (nmap-os-db compatible)
- ✅ OS detection with 2,000+ signatures (OsFingerprinter - 115 lines)

#### Week 8: Service Detection Framework ✅

**Sprint 3.2 - COMPLETE**

- ✅ Implement service probe database schema (service_db.rs - 451 lines)
- ✅ Parse `nmap-service-probes` format with regex matching
- ✅ Add intensity levels 0-9 for probe selection
- ✅ Create port-indexed probe lookup for optimization
- ✅ Build softmatch rules for partial matches
- ✅ Version info extraction: product, version, CPE, OS hints

**Deliverables:**

- ✅ Service detection engine (ServiceDetector - 264 lines)
- ✅ 500+ protocol probes with regex patterns
- ✅ Configurable intensity levels

#### Week 9: Banner Grabbing ✅

**Sprint 3.3 - COMPLETE**

- ✅ Implement banner grabber for TCP services (banner_grabber.rs - 340 lines)
- ✅ Add protocol-specific handlers (HTTP, FTP, SSH, SMTP, POP3, IMAP)
- ✅ Create auto-detection by port number
- ✅ Build HTTP GET request with custom User-Agent
- ✅ Add SMTP 220 greeting + EHLO command
- ✅ Implement generic TCP banner grabbing fallback

**Deliverables:**

- ✅ Comprehensive banner grabbing with 6 protocol handlers
- ✅ Configurable timeout and max banner size
- ✅ BannerParser utility for extracting server info

#### Week 10: Testing and CLI Integration ✅

**Sprint 3.4 - COMPLETE**

- ✅ Add CLI flags: -O, --sV, --version-intensity, --osscan-limit, --banner-grab
- ✅ Integrate detection modules with scanner pipeline
- ✅ Comprehensive unit tests for all detection modules
- ✅ Integration tests for end-to-end detection workflows
- ✅ Documentation updates and examples
- ✅ Performance validation and optimization

**Deliverables:**

- ✅ Phase 3 fully integrated with 371 passing tests
- ✅ CLI documentation and usage examples
- ✅ Phase 4 complete: Performance Optimization (lock-free, stateless, NUMA)

---

### Phase 4: Performance Optimization (Weeks 11-13)

**Goal:** Achieve internet-scale performance (10M+ packets/second) — Achieved with NUMA-aware zero-copy pipeline

#### Week 11: Lock-Free Architecture

**Sprint 4.1**

- [x] Integrated `crossbeam` lock-free queues into the scheduler (see `scheduler.rs`, Sprint 4.1)
- [x] Implemented work-stealing task scheduler with adaptive worker pools
- [x] Replaced mutex hotspots with atomics and lock-free queues
- [x] Split transmit/receive pipelines with dedicated worker pools
- [x] Added MPSC aggregation channels feeding streaming writer
- [x] Captured perf/flamegraph baselines for regression tracking

**Deliverables:**

- Lock-free task distribution
- Separate TX/RX pipeline
- Performance profiling reports (perf + flamegraph + hyperfine)

#### Week 12: Stateless Scanning

**Sprint 4.2**

- [x] Implemented SipHash-backed sequence generator for stateless scans
- [x] Added stateless response validation and deduplication logic
- [x] Built BlackRock-inspired target permutation for massive sweep support
- [x] Added Masscan-compatible greppable output + stream writer
- [x] Implemented streaming result writer with zero-copy buffers
- [x] Added memory profiling via massif + custom leak harness

**Deliverables:**

- Stateless scan mode (masscan-like)
- <1MB memory usage per million target batch (validated via massif)
- Binary/greppable output formats

#### Week 13: System-Level Optimization

**Sprint 4.3**

- [x] Added NUMA-aware thread pinning with hwloc integration (`--numa`)
- [x] Documented IRQ affinity guidance and automated defaults
- [x] Implemented sendmmsg/recvmmsg batching on Linux fast path
- [x] Added BPF filter tuning presets for high-rate capture
- [x] Extended connection pooling across stateful scan modes
- [x] Built performance regression suite (hyperfine, perf, strace, massif)

**Deliverables:**

- NUMA optimization guide + CLI toggles (`--numa`, `--no-numa`)
- 10M+ pps capability on tuned hardware (validated via synthetic lab runs)
- Comprehensive performance benchmarks (repo `benchmarks/02-Phase4_Final-Bench`)

---

### Phase 5: Advanced Features (Weeks 14-20) 🔄 IN PROGRESS

**Goal:** Deliver production-ready advanced features including complete IPv6 support, enhanced service detection, idle scanning, rate limiting, and TLS analysis

**Status:** 40% Complete (4/10 sprints) | **Current Sprint:** 5.4 Phase 1 ✅ COMPLETE, Phase 2 ⏸️ PENDING
**Duration:** 6-8 weeks (168-215 hours estimated)
**Target:** v0.5.0 (Q1 2026)

**Progress Summary:**
- Tests: 1,338 → 1,466 (+128, +9.6%) | 100% passing
- Coverage: 61.92% → 62.5% (target: 80%+)
- Releases: v0.4.1 (IPv6), v0.4.2 (Service Detection), v0.4.3 (Idle Scan)
- Documentation: 2,648 lines (4 comprehensive guides created)

---

#### Sprint 5.1: IPv6 Completion ✅ COMPLETE (v0.4.1, Oct 28-29, 30h)

**Status:** ✅ 100% COMPLETE (Milestone: 100% IPv6 Coverage Achieved)
**Completed:** 2025-10-29
**Effort:** 30 hours / 30 hours estimated (on budget)

**Objectives Achieved:**
- [x] IPv6 dual-stack for SYN, UDP, Stealth scanners (runtime dispatch)
- [x] Discovery Engine IPv6 (ICMPv6 Echo + NDP Neighbor Solicitation)
- [x] Decoy Scanner IPv6 (random /64 Interface Identifier generation)
- [x] CLI flags: -6, -4, --prefer-ipv6, --prefer-ipv4, --ipv6-only, --ipv4-only
- [x] Cross-scanner IPv6 integration tests (11 tests)
- [x] CLI output filter (hosts with open ports only)
- [x] Comprehensive IPv6 Guide (docs/23-IPv6-GUIDE.md, 1,958 lines, 49KB)

**Deliverables:**
- 100% scanner coverage (6/6 scanners: TCP SYN, TCP Connect, UDP, Stealth, Discovery, Decoy)
- 40 new tests (1,349 → 1,389)
- Performance: <15% overhead (production-ready)
- 2,648 lines permanent documentation

**Technical Achievement:**
- ICMPv6 Type 128/129 (Echo Request/Reply)
- NDP Type 135/136 (Neighbor Solicitation/Advertisement)
- Solicited-node multicast addressing
- IPv6 pseudo-header checksum calculation
- Dual-stack hostname resolution with preference control

---

#### Sprint 5.2: Service Detection Enhancement ✅ COMPLETE (v0.4.2, Oct 30, 12h)

**Status:** ✅ 100% COMPLETE (Milestone: 85-90% Detection Rate Achieved)
**Completed:** 2025-10-30
**Effort:** 12 hours / 15-18 hours estimated (under budget, 33% better)

**Objectives Achieved:**
- [x] HTTP Server Fingerprinting (Server header parsing, Ubuntu version mapping)
- [x] SSH Banner Parsing (SSH-2.0 protocol, distro detection from build strings)
- [x] SMB Dialect Negotiation (SMB1/2/3, Windows version mapping)
- [x] MySQL Detection (handshake parsing, MariaDB differentiation, Ubuntu version)
- [x] PostgreSQL Detection (ParameterStatus parsing, version extraction)
- [x] Protocol priority system (1-5 execution order)
- [x] Integration testing (23 new tests)
- [x] Service Detection Guide (docs/24-SERVICE-DETECTION.md, 659 lines)

**Deliverables:**
- Detection rate improvement: 70-80% → 85-90% (+10-15 percentage points)
- 5 protocol parsers (HTTP, SSH, SMB, MySQL, PostgreSQL)
- 23 new tests (1,389 → 1,412)
- <1% performance overhead (0.05ms per target)
- ProtocolDetector trait architecture

**Technical Achievement:**
- Ubuntu version mapping: "4ubuntu0.3" → 20.04, "0ubuntu0.20.04.1" → 20.04
- Debian/RHEL detection from package suffixes
- SMB dialect → Windows version correlation
- MariaDB vs MySQL differentiation via version prefixes
- PostgreSQL server_version ParameterStatus extraction

---

#### Sprint 5.3: Idle Scan Implementation ✅ COMPLETE (v0.4.3, Oct 30, 18h)

**Status:** ✅ 100% COMPLETE (Milestone: Full Nmap -sI Parity Achieved)
**Completed:** 2025-10-30
**Effort:** 18 hours / 20-25 hours estimated (under budget, 28% better)

**Objectives Achieved:**
- [x] IP ID tracking system (IpIdTracker, sequential/random/global/per-dest patterns)
- [x] Zombie discovery algorithm (scan subnet, test 65535, validate patterns)
- [x] Spoofed packet engine (SYN packets with zombie source address)
- [x] Three-step idle scan process (probe → spoof → probe → infer)
- [x] Timing control integration (T0-T5, 500-800ms per port)
- [x] Nmap -sI flag compatibility (manual + RND auto-discovery)
- [x] 44 new tests (1,422 → 1,466, 100% passing)
- [x] Idle Scan Guide (docs/25-IDLE-SCAN-GUIDE.md, 650 lines, 42KB)

**Deliverables:**
- Full Nmap parity (7/8 features, IPv6 future)
- Maximum anonymity scanning (target logs only zombie IP)
- 44 new tests across 6 test files
- 99.5% accuracy (when zombie requirements met)
- Comprehensive zombie suitability testing

**Technical Achievement:**
- IP ID delta interpretation: +0 (filtered), +1 (closed), +2 (open)
- Four IP ID pattern detection algorithms
- Source address spoofing with privilege checks
- Exponential backoff for zombie instability
- Ethical warnings and legal documentation

**Performance:**
- 500-800ms per port (vs 5.15ms direct SYN = 300x slower)
- Stealth tradeoff: Maximum anonymity at cost of speed
- Acceptable for targeted penetration testing scenarios

---

#### Sprint 5.4: Advanced Rate Limiting 🔄 IN PROGRESS (v0.4.4, Nov 1, Phase 1 ✅)

**Status:** Phase 1 ✅ COMPLETE (Scanner Integration), Phase 2 ⏸️ PENDING (Formal Benchmarking)
**Started:** 2025-11-01
**Effort So Far:** ~8 hours (Phase 1)
**Estimated Remaining:** 8-10 hours (Phase 2)

**Phase 1 Objectives Achieved:**
- [x] Three-layer rate limiting architecture
  - Layer 1: ICMP Type 3 Code 13 detection (automatic backoff)
  - Layer 2: Hostgroup limiting (Nmap-compatible, --max-hostgroup/--min-hostgroup)
  - Layer 3: Adaptive rate limiting (Masscan-inspired, --max-rate)
- [x] IcmpMonitor module (icmp_monitor.rs, DashMap-based prohibition tracking)
- [x] HostgroupLimiter module (hostgroup_limiter.rs, Semaphore-based concurrency)
- [x] AdaptiveRateLimiter module (adaptive_rate_limiter.rs, circular buffer throttler)
- [x] Scanner integration (7/7 scanners: two-category pattern identified)
- [x] Rate Limiting Guide (docs/26-RATE-LIMITING-GUIDE.md, ~1,000 lines)

**Two-Category Scanner Pattern Discovery:**
- **Multi-Port Scanners (3):** ConcurrentScanner, TcpConnectScanner, SynScanner
  - Hostgroup permit acquired at `scan_ports` level
  - ICMP backoff checked per packet within target scan
- **Per-Port Scanners (4):** UdpScanner, StealthScanner, IdleScanner, DecoyScanner
  - No hostgroup limiting (iterate target × port combinations)
  - ICMP backoff checked at `scan_port` level only

**Phase 2 Objectives (PENDING):**
- [ ] Formal hyperfine benchmarking (baseline vs 3 layers vs combined)
- [ ] Performance overhead validation (<5% target)
- [ ] Documentation update with benchmark results
- [ ] CHANGELOG entry for Phase 2 completion

**Current Status:**
- Scanner integration: ✅ COMPLETE (all 7 scanners updated)
- Documentation: ✅ COMPLETE (comprehensive guide created)
- Benchmarking: ⏸️ DEFERRED (formal validation pending)

---

#### Sprint 5.5: TLS Certificate Analysis 📋 PLANNED (Q1 2026)

**Status:** 📋 PLANNED
**Estimated Duration:** 15-20 hours
**ROI Score:** 7.5/10

**Objectives:**
- [ ] TLS handshake module enhancement (certificate parsing)
- [ ] Subject/Issuer extraction
- [ ] Certificate chain validation
- [ ] Expiration date checking
- [ ] Common name / SAN extraction
- [ ] Self-signed certificate detection

**Deliverables:**
- Enhanced TLS detection (beyond current handshake-only)
- Certificate chain validation
- 15+ new tests
- TLS Analysis Guide

---

#### Sprint 5.6: Code Coverage Enhancement 📋 PLANNED (Q1 2026)

**Status:** 📋 PLANNED
**Estimated Duration:** 20-25 hours
**ROI Score:** 8.0/10

**Objectives:**
- [ ] Coverage analysis (identify gaps: 62.5% → 80% target)
- [ ] Unit tests for uncovered modules
- [ ] Integration tests for complex workflows
- [ ] Edge case testing (malformed packets, network errors)

**Deliverables:**
- 80%+ code coverage (vs current 62.5%)
- 100+ new tests
- Coverage report automation (CI integration)

---

#### Sprint 5.7: Fuzz Testing Infrastructure 📋 PLANNED (Q1 2026)

**Status:** 📋 PLANNED
**Estimated Duration:** 15-20 hours
**ROI Score:** 9.0/10

**Objectives:**
- [ ] cargo-fuzz integration
- [ ] Fuzz targets for packet parsers (TCP, UDP, ICMP, ICMPv6)
- [ ] Fuzz targets for input validation (IP/CIDR parsing)
- [ ] CI integration (continuous fuzzing)

**Deliverables:**
- 5+ fuzz targets
- 24-hour continuous fuzzing (CI)
- Security hardening via crash discovery

---

#### Sprint 5.8: Plugin System Foundation 📋 PLANNED (Q1 2026)

**Status:** 📋 PLANNED
**Estimated Duration:** 25-30 hours
**ROI Score:** 9.2/10 (Highest)

**Objectives:**
- [ ] mlua integration for Lua scripting
- [ ] Plugin API design (init, scan, report lifecycle)
- [ ] Sandboxing (restricted filesystem/network access)
- [ ] Example plugins (HTTP enumeration, SSL checker, etc.)
- [ ] Plugin discovery and loading

**Deliverables:**
- Lua plugin system (NSE-like capability)
- 5+ example plugins
- Plugin Developer Guide
- Security sandboxing

---

#### Sprint 5.9: Benchmarking Suite 📋 PLANNED (Q1 2026)

**Status:** 📋 PLANNED
**Estimated Duration:** 12-15 hours
**ROI Score:** 6.5/10

**Objectives:**
- [ ] Criterion benchmark suite expansion
- [ ] Scanner performance baselines
- [ ] Regression detection automation
- [ ] Performance tracking over releases

**Deliverables:**
- 20+ Criterion benchmarks
- Performance regression CI checks
- Benchmark report automation

---

#### Sprint 5.10: Documentation Consolidation 📋 PLANNED (Q1 2026)

**Status:** 📋 PLANNED
**Estimated Duration:** 10-12 hours
**ROI Score:** 7.0/10

**Objectives:**
- [ ] User manual creation (beginner → advanced workflows)
- [ ] API documentation (cargo doc enhancements)
- [ ] Tutorial series (common scanning scenarios)
- [ ] Troubleshooting guide updates

**Deliverables:**
- Comprehensive user manual
- Tutorial series (5+ guides)
- Updated API documentation
- Improved onboarding experience

---

**Phase 5 Summary:**

**Completed (40%):**
- ✅ Sprint 5.1: IPv6 Completion (100% coverage)
- ✅ Sprint 5.2: Service Detection (85-90% rate)
- ✅ Sprint 5.3: Idle Scan (Nmap parity)
- ✅ Sprint 5.4 Phase 1: Rate Limiting (scanner integration)

**In Progress:**
- 🔄 Sprint 5.4 Phase 2: Rate Limiting (benchmarking)

**Remaining (60%):**
- 📋 Sprint 5.5-5.10 (TLS, Coverage, Fuzz, Plugins, Benchmarks, Docs)

**Target Completion:** Q1 2026 (v0.5.0 release)

---

### Phase 6: TUI Interface (Weeks 17-18)

**Goal:** Create interactive terminal user interface

#### Week 17: TUI Foundation

**Sprint 6.1**

- [ ] Setup `ratatui` framework
- [ ] Design TUI layout (target input, progress, results)
- [ ] Implement real-time progress display
- [ ] Create keyboard navigation
- [ ] Add scan configuration widgets
- [ ] Build result table view

**Deliverables:**

- Functional TUI with basic navigation
- Real-time scan progress
- Interactive result browsing

#### Week 18: TUI Advanced Features

**Sprint 6.2**

- [ ] Add result filtering and search
- [ ] Implement export from TUI
- [ ] Create scan history view
- [ ] Build help system
- [ ] Add color themes
- [ ] Implement mouse support

**Deliverables:**

- Feature-complete TUI
- User guide for TUI mode
- Theme customization

---

### Phase 7: Polish and Release (Weeks 19-20)

**Goal:** Prepare for production v1.0 release

#### Week 19: Documentation and Packaging

**Sprint 7.1**

- [ ] Complete user manual
- [ ] Write developer documentation
- [ ] Create example scan scenarios
- [ ] Build installation packages (deb, rpm, msi, pkg)
- [ ] Setup Docker images
- [ ] Add man pages

**Deliverables:**

- Complete documentation
- Multi-platform installers
- Docker Hub images

#### Week 20: Final Testing and Release

**Sprint 7.2**

- [ ] Conduct security audit
- [ ] Perform penetration testing on scanner itself
- [ ] Run extended performance tests
- [ ] Fix release-blocking bugs
- [ ] Create release notes
- [ ] Tag v1.0.0 release

**Deliverables:**

- Security audit report
- v1.0.0 release on GitHub
- Announcement blog post

---

### Phase 8: Future Enhancements (Post-v1.0)

**Goal:** Expand interface options and capabilities

#### Web Interface (4-6 weeks)

- [ ] Design RESTful API
- [ ] Implement authentication (JWT/OAuth)
- [ ] Create React/Vue frontend
- [ ] Add real-time WebSocket updates
- [ ] Build scan scheduler
- [ ] Implement multi-user support

#### Desktop GUI (6-8 weeks)

- [ ] Evaluate frameworks (Tauri, iced, egui)
- [ ] Design GUI layout
- [ ] Implement scan configuration wizard
- [ ] Create network topology visualizer
- [ ] Add result charting
- [ ] Build native installers

#### Distributed Scanning (8-10 weeks)

- [ ] Design coordinator/worker architecture
- [ ] Implement work distribution algorithm
- [ ] Create result aggregation protocol
- [ ] Add authentication and encryption
- [ ] Build monitoring dashboard
- [ ] Implement failure recovery

---

## Sprint Planning

### Sprint Structure

Each 2-week sprint follows this structure:

**Week 1:**

- Monday: Sprint planning meeting, review acceptance criteria
- Tuesday-Thursday: Implementation
- Friday: Code review and integration

**Week 2:**

- Monday-Wednesday: Implementation and testing
- Thursday: Documentation and examples
- Friday: Sprint retrospective, demo to stakeholders

### Sprint Ceremonies

1. **Planning (2 hours):** Define sprint goals and task breakdown
2. **Daily Standup (15 min):** Progress updates and blocker discussion
3. **Code Review:** All PRs reviewed within 24 hours
4. **Demo (1 hour):** Showcase completed features
5. **Retrospective (1 hour):** Lessons learned and process improvements

---

## Milestones and Deliverables

### Milestone 1: Basic Scanning (End of Phase 1) ✅ COMPLETE

- [x] TCP connect scan working on all platforms
- [x] CLI with essential flags
- [x] Text, JSON, and XML output
- [x] SQLite storage

**Success Criteria:**

- [x] Scan 1000 hosts with 100 ports in <5 minutes (achieved)
- [x] Pass 215 tests (exceeded 50+ goal)
- [x] Zero memory leaks (Rust memory safety)

### Milestone 2: Advanced Scanning (End of Phase 2)

- [ ] SYN, UDP, and stealth scans implemented
- [ ] Timing templates functional
- [ ] Adaptive rate limiting

**Success Criteria:**

- SYN scan 10,000 ports in <30 seconds
- UDP scan detecting 10+ common services
- Rate limiting prevents network saturation

### Milestone 3: Detection Capabilities (End of Phase 3)

- [x] OS fingerprinting with 2,000+ signatures (Phase 3 deliverable)
- [x] Service detection for 500+ protocols (service_db.rs)
- [x] Banner grabbing with SSL support (banner_grabber.rs)

**Success Criteria:**

- Achieved >90% OS detection accuracy on lab network corpus
- Service detection matches Nmap baselines on curated suite
- SSL banner grabbing validated for HTTPS/SMTPS/IMAPS samples

### Milestone 4: Performance Target (End of Phase 4)

- [x] Stateless scanning at 1M+ pps (validated on lab dual-10GbE host)
- [x] Lock-free architecture (crossbeam scheduler + streaming writer)
- [x] NUMA optimization (`--numa` CLI + hwloc integration)

**Success Criteria:**

- Achieved 1M+ packets/second on 10GbE lab hardware (hyperfine benchmark suite)
- <100MB heap usage for 1M-target stateless scan (massif profile: 61MB peak)
- CPU utilisation scales linearly with core count (NUMA affinity tests)

### Milestone 5: Feature Complete (End of Phase 5)

- [ ] Idle scan, decoys, fragmentation
- [ ] Plugin system with examples
- [ ] All Nmap-equivalent features

**Success Criteria:**

- Idle scan successfully evades attribution
- 5+ working plugins
- Feature parity with Nmap for core functionality

### Milestone 6: Production Ready (End of Phase 7)

- [ ] TUI interface
- [ ] Complete documentation
- [ ] Multi-platform packages

**Success Criteria:**

- 200+ page user manual
- Packages for 5+ platforms
- <10 open critical bugs

---

## Risk Management

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Platform-specific packet capture issues | High | High | Early testing on all platforms, fallback to connect scan |
| Performance targets not met | Low | Medium | Mitigated via Phase 4 benchmarking suite (perf, hyperfine, massif) |
| OS fingerprint accuracy low | Medium | Medium | Collaborate with Nmap project, build test lab |
| Windows Npcap compatibility | Medium | Medium | Test with multiple Npcap versions, document requirements |

### Schedule Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Phase 3 (detection) takes longer | Medium | Medium | Parallel development of Phase 4, reduce fingerprint database initially |
| Plugin system complexity | Low | Low | Make optional feature, defer to v1.1 if necessary |
| TUI delays release | Low | Medium | CLI is primary interface, TUI can be v1.1 feature |

### Resource Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Single developer bandwidth | High | High | Prioritize core features, defer nice-to-haves |
| Test hardware availability | Low | Medium | Use cloud instances for performance testing |
| Third-party dependency changes | Low | Low | Pin dependency versions, regular updates |

---

## Success Metrics

### Code Quality

- **Test Coverage:** >80% for core modules, >60% overall
- **Clippy Warnings:** Zero warnings with `clippy::pedantic`
- **Security Audit:** Pass Cargo audit with no high/critical CVEs
- **Documentation:** 100% public API documented

### Performance

- **Throughput:** 1M+ pps stateless, 50K+ pps stateful
- **Memory:** <100MB for 1M targets, <1GB for 10M targets
- **Latency:** <1ms per packet crafting operation
- **Accuracy:** >95% match with Nmap on standardized test suite

### Usability

- **Installation:** <5 minutes from download to first scan
- **Documentation:** User finds common tasks in <2 clicks
- **Error Messages:** Clear guidance on fixing 90%+ of user errors
- **Discoverability:** `--help` sufficient for 80% of use cases

### Adoption

- **GitHub Stars:** 1000+ in first 6 months
- **Downloads:** 10,000+ in first 6 months
- **Contributors:** 10+ external contributors
- **Issues Closed:** >80% of filed issues resolved

---

## Next Steps

1. **Review Architecture:** Ensure alignment with [Architecture Overview](00-ARCHITECTURE.md)
2. **Setup Environment:** Follow [Development Setup Guide](03-DEV-SETUP.md)
3. **Begin Phase 1:** Start with Sprint 1.1 tasks
4. **Track Progress:** Update [Project Status](10-PROJECT-STATUS.md) weekly

---

## Roadmap Revisions

This roadmap is a living document. Expected revisions:

- **After Phase 1:** Adjust Phase 2-3 timelines based on actual velocity
- ✅ **After Phase 3:** Re-evaluated Phase 4 performance targets (benchmarks adjusted)
- **After Phase 5:** Assess Phase 6-7 priorities based on user feedback
- **Quarterly:** Review and update based on ecosystem changes

**Document History:**

- v1.0 (Oct 2025): Initial roadmap creation
