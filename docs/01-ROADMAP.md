# ProRT-IP WarScan: Development Roadmap

**Version:** 1.5
**Last Updated:** 2025-10-13
**Project Status:** Phase 4 COMPLETE ✅ + Cycles 1-8 COMPLETE ✅ + Testing Infrastructure ✅ | **50% Overall Progress** (4/8 phases) | Phase 5 Ready

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
| Phase 4 | Weeks 11-13 | Performance | Lock-free structures, adaptive parallelism, sendmmsg batching | ✅ COMPLETE (789 tests, 61.92% coverage) |
| Phase 5 | Weeks 14-16 | Advanced Features | Idle scan, decoys, fragmentation, plugins | Planned |
| Phase 6 | Weeks 17-18 | TUI Interface | Interactive terminal dashboard | Planned |
| Phase 7 | Weeks 19-20 | Polish & Release | Documentation, packaging, v1.0 release | Planned |
| Phase 8 | Post-v1.0 | Future Enhancements | Web UI, desktop GUI, distributed scanning | Planned |

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
- ✅ Ready for Phase 4: Performance Optimization

---

### Phase 4: Performance Optimization (Weeks 11-13)

**Goal:** Achieve internet-scale performance (10M+ packets/second)

#### Week 11: Lock-Free Architecture

**Sprint 4.1**

- [ ] Integrate `crossbeam` lock-free queues
- [ ] Implement work-stealing task scheduler
- [ ] Replace mutexes with atomic operations where possible
- [ ] Create separate TX/RX threads
- [ ] Add MPSC channels for result aggregation
- [ ] Profile with `perf` and flamegraphs

**Deliverables:**

- Lock-free task distribution
- Separate TX/RX pipeline
- Performance profiling reports

#### Week 12: Stateless Scanning

**Sprint 4.2**

- [ ] Implement SipHash sequence number generation
- [ ] Create stateless response validation
- [ ] Build target permutation algorithm
- [ ] Add Masscan-compatible output format
- [ ] Implement streaming result writer
- [ ] Create memory profiling tests

**Deliverables:**

- Stateless scan mode (masscan-like)
- <1MB memory usage for arbitrary target count
- Binary output format

#### Week 13: System-Level Optimization

**Sprint 4.3**

- [ ] Add NUMA-aware thread pinning
- [ ] Implement IRQ affinity configuration
- [ ] Create sendmmsg/recvmmsg batching (Linux)
- [ ] Add BPF filter optimization
- [ ] Implement connection pooling for stateful scans
- [ ] Build performance test suite

**Deliverables:**

- NUMA optimization guide
- 10M+ pps capability on appropriate hardware
- Comprehensive performance benchmarks

---

### Phase 5: Advanced Features (Weeks 14-16)

**Goal:** Implement sophisticated stealth and extensibility features

#### Week 14: Idle Scanning and Decoys

**Sprint 5.1**

- [ ] Implement zombie host discovery
- [ ] Create IPID increment detection
- [ ] Build idle scan port prober
- [ ] Add binary search for multiple open ports
- [ ] Implement decoy list parsing and generation
- [ ] Create source port spoofing

**Deliverables:**

- Working idle scan (-sI flag)
- Decoy scanning (-D flag)
- Source port manipulation (-g flag)

#### Week 15: Fragmentation and Packet Manipulation

**Sprint 5.2**

- [ ] Implement IP fragmentation (-f, -ff, --mtu)
- [ ] Add fragment reassembly evasion
- [ ] Create TTL manipulation
- [ ] Build IP options insertion
- [ ] Add MAC address spoofing (--spoof-mac)
- [ ] Implement bad checksum generation

**Deliverables:**

- Fragmentation support
- Advanced packet manipulation
- Evasion technique documentation

#### Week 16: Plugin System

**Sprint 5.3**

- [ ] Design plugin API
- [ ] Integrate `mlua` for Lua scripting
- [ ] Create plugin lifecycle (init, scan, report)
- [ ] Build example plugins (HTTP enum, SSL checker, etc.)
- [ ] Add plugin discovery and loading
- [ ] Implement sandboxing for untrusted scripts

**Deliverables:**

- Lua plugin system
- 5+ example plugins
- Plugin developer guide

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

- [ ] OS fingerprinting with 1000+ signatures
- [ ] Service detection for 100+ protocols
- [ ] Banner grabbing with SSL support

**Success Criteria:**

- OS detection accuracy >85% on test network
- Service detection matches Nmap on test suite
- SSL banner grabbing for HTTPS/SMTPS/etc.

### Milestone 4: Performance Target (End of Phase 4)

- [ ] Stateless scanning at 1M+ pps
- [ ] Lock-free architecture
- [ ] NUMA optimization

**Success Criteria:**

- 1,000,000+ packets/second on test hardware (10GbE)
- <100MB memory for 1M target scan
- CPU usage scales linearly with cores

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
| Performance targets not met | Medium | High | Profile early and often, use proven techniques from Masscan |
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
- **After Phase 3:** Re-evaluate Phase 4 performance targets
- **After Phase 5:** Assess Phase 6-7 priorities based on user feedback
- **Quarterly:** Review and update based on ecosystem changes

**Document History:**

- v1.0 (Oct 2025): Initial roadmap creation
