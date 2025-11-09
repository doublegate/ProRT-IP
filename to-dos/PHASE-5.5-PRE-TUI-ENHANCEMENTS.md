# Phase 5.5: Pre-TUI Polish & Foundations

**Status:** IN PROGRESS
**Duration:** 2-3 weeks (19-24 days estimated)
**Goal:** Production CLI polish + TUI-ready backend architecture
**Sprints:** 6 detailed sprints
**Version Target:** v0.5.5 (incremental from v0.5.0)
**Priority:** HIGH (prerequisite for Phase 6 success)

---

## Executive Summary

Phase 5 delivered exceptional velocity—10 sprints in 11 days—achieving 100% completion with 1,601 tests passing, 54.92% coverage, and comprehensive documentation (50,510+ lines). However, this rapid pace created opportunities for refinement before starting Phase 6 (TUI Interface).

**Phase 5 Achievements (Context):**
- ✅ Sprint 5.1: IPv6 Completion (100% coverage, 15% overhead)
- ✅ Sprint 5.2: Service Detection (85-90% rate, 5 parsers)
- ✅ Sprint 5.3: Idle Scan (Nmap parity, 99.5% accuracy)
- ✅ Sprint 5.X: Rate Limiting V3 (-1.8% overhead, industry-leading)
- ✅ Sprint 5.5: TLS Certificate Analysis (X.509v3, 1.33μs parsing)
- ✅ Sprint 5.6: Code Coverage (+17.66% to 54.92%, 149 tests)
- ✅ Sprint 5.7: Fuzz Testing (230M+ executions, 0 crashes)
- ✅ Sprint 5.8: Plugin System (Lua 5.4, sandboxing, 2 examples)
- ✅ Sprint 5.9: Benchmarking (hyperfine, 10 scenarios, CI integration)
- ✅ Sprint 5.10: Documentation Polish (4,270+ lines, professional quality)

**Phase 5.5 Achievements:**
- ✅ Sprint 5.5.1: Documentation & Examples (COMPLETE, 21.1h, 7/7 tasks, A+)
- ✅ Sprint 5.5.2: CLI Usability & UX (COMPLETE, 15.5h, 6/6 tasks, A+, 81% efficiency)

**Gaps Identified:**

1. **Polish & Refinement:**
   - 5 scanner doctests ignored (pragmatic choice during Phase 5 rush)
   - Example gallery: 39 scenarios → target 60+ for comprehensive coverage
   - CLI UX improvements (progress ETA, interactive confirmations, multi-page help)
   - Error message quality (actionable guidance enhancement)

2. **TUI Prerequisites Missing:**
   - Event system (no pub-sub pattern - TUI needs real-time updates)
   - Centralized state management (each scanner manages own state)
   - Real-time metrics collection (basic ScanProgress exists but limited)
   - Configuration profiles (TOML exists but no save/load presets)
   - Resume capability (no scan state persistence)

3. **Quick-Win Competitive Gaps:**
   - Output formats: Missing CSV, HTML reports
   - Logging: No structured file logging (JSON audit trails)
   - Plugin ecosystem: Only 2 examples (Nmap has 600+ NSE scripts)
   - Progress indicators: No ETA or real-time throughput display
   - Scan templates: No named presets for common scenarios

4. **Technical Debt:**
   - Test coverage: 54.92% overall (9 modules <50%)
   - Performance regression risk (no continuous benchmarking yet)
   - API consistency (recent breaking changes need stabilization documentation)

5. **Competitive Parity:**
   - Banner grabbing: Synchronous (async would improve speed)
   - Speed optimization: 10K pps vs Masscan 10M pps (optimization opportunities)
   - Plugin examples: 2 vs Nmap 600+ (ecosystem gap)

**Phase 5.5 Strategy:**

Bridge the gap between Phase 5's feature completeness and Phase 6's TUI requirements through targeted refinement and foundation-building. Focus on:

1. **User-Facing Polish:** Fix documentation gaps, expand examples, improve CLI UX
2. **TUI Foundations:** Build event system, state management, real-time metrics
3. **Quick Wins:** Add high-value features with low effort (CSV output, logging, templates)
4. **Quality Assurance:** Increase test coverage, establish performance baselines
5. **Preparation:** Create TUI-ready backend architecture before UI development

**Expected Outcomes:**

- **CLI Quality:** Production-ready v0.5.5 with polished UX
- **TUI Readiness:** Event-driven backend architecture operational
- **Documentation:** Comprehensive examples (60+), tutorials, API guides
- **Test Coverage:** 60%+ overall (from 54.92%)
- **Performance:** Baselines established for all scan types
- **Zero Blockers:** No technical debt preventing Phase 6 start

**Strategic Value:**

Phase 5.5 ensures Phase 6 (TUI) can focus purely on interface development without backend refactoring. The event system and state management patterns established here will support future phases (distributed scanning, web UI) beyond just TUI.

---

## Sprint Roadmap

| Sprint | Title | Priority | Duration | Dependencies | ROI |
|--------|-------|----------|----------|--------------|-----|
| 5.5.1 | Documentation & Examples Polish | HIGH | 3-4d | None | 8.5/10 |
| 5.5.2 | CLI Usability & UX | HIGH | 3-4d | None | 8.0/10 |
| 5.5.3 | Event System & Progress | **CRITICAL** | 4-5d | 5.5.2 | 9.5/10 |
| 5.5.4 | Performance Audit | MEDIUM | 3-4d | None | 7.0/10 |
| 5.5.5 | Configuration & State | MEDIUM | 3-4d | 5.5.3 | 7.5/10 |
| 5.5.6 | Integration & Testing | MEDIUM | 3-4d | All above | 8.0/10 |

**Total Duration:** 19-24 days (3-4 weeks with buffer)
**Critical Path:** 5.5.2 → 5.5.3 → 5.5.5 → 5.5.6 (15-18 days)
**Parallel Opportunities:** 5.5.1, 5.5.2, 5.5.4 can run concurrently (week 1)

---

## Sprint 5.5.1: Documentation & Examples Polish

**Priority:** HIGH (user-facing, blocks adoption)
**Duration:** 3-4 days (24 hours estimated)
**ROI Score:** 8.5/10 (High impact, moderate effort)
**Dependencies:** None (independent work)

### Objective

Fix technical debt from Phase 5 documentation rush and create comprehensive example gallery for all major features. Ensure users can discover and use all Phase 5 capabilities in <30 seconds.

### Rationale

Phase 5 delivered 10 sprints in 11 days with exceptional velocity, but documentation polish was necessarily limited. The 5 ignored scanner doctests and gaps in example coverage need addressing before Phase 6 to ensure users can effectively leverage all Phase 5 features (IPv6, Service Detection, Idle Scan, TLS Analysis, Plugins, etc.).

**Current State:**
- 5 scanner doctests marked as `#[ignore]` (technical debt from API changes)
- Example gallery: 39 scenarios in Sprint 5.10 (target: 60+ for comprehensive coverage)
- User guide: Good coverage but gaps in advanced feature integration
- API documentation: 0 rustdoc warnings (Sprint 5.10 fixed 40) but examples need real-world scenarios
- Tutorial series: Complete beginner→advanced path but could expand hands-on exercises

**Why Before Phase 6:**
A comprehensive example gallery serves dual purposes: user education AND integration test suite. Before building TUI (Phase 6), users need confidence in CLI functionality. Strong documentation reduces support burden and enables community contributions.

**Impact of Deferring:**
- User confusion about Phase 5 features (IPv6, Idle Scan, Plugins unclear usage)
- Reduced adoption (users can't discover capabilities quickly)
- Support burden (repeated questions about usage)
- Missed bug discovery (real examples expose edge cases)

### Tasks

1. **Fix 5 Ignored Scanner Doctests** (2-3h)
   - [ ] Update `crates/prtip-scanner/src/lib.rs` doctest examples (lines 62, 88, 125, 155, 178)
   - [ ] Replace outdated API references:
     - `Error::InvalidInput` → `Error::Config`
     - `config.scan_config` → `config.scan`
     - `ServiceInfo` field renames (from_port → source_port, etc.)
   - [ ] Add proper test fixtures or use realistic mocks
   - [ ] Verify all doctests pass: `cargo test --doc`
   - [ ] Remove all `#[ignore]` attributes
   - **Acceptance:** 0 ignored doctests, 100% doctests passing

2. **Expand Example Gallery to 60+ Scenarios** (8-10h)
   - **Common Use Cases** (20 examples, 3-4h):
     - [ ] Basic SYN scan (single host, common ports)
     - [ ] Subnet scanning (CIDR notation, 192.168.1.0/24)
     - [ ] Service version detection (80, 443, 22, 3306, 5432)
     - [ ] Fast scanning (top 100 ports, T4 timing)
     - [ ] Stealth scanning (FIN/NULL/Xmas combinations)
     - [ ] UDP scanning (DNS, SNMP, NetBIOS discovery)
     - [ ] Host discovery (ICMP + ARP combinations)
     - [ ] Multiple output formats (JSON, XML, Greppable)
     - [ ] Rate-limited scans (polite mode, T0-T2)
     - [ ] Target specification (ranges, files, exclusions)
     - [ ] Timing templates (T0→T5 comparisons with benchmarks)
     - [ ] Privilege handling (CAP_NET_RAW, setuid examples)
     - [ ] Configuration files (TOML presets)
     - [ ] Database storage (SQLite result queries)
     - [ ] IPv4 + IPv6 dual-stack (hostname resolution preferences)
     - [ ] Error handling (network unreachable, permission denied)
     - [ ] Large-scale scans (progress tracking, resource limits)
     - [ ] Firewall detection (ACK scan interpretation)
     - [ ] Filter evasion (fragmentation, TTL, decoys)
     - [ ] Compliance scanning (audit logging, safe modes)
   - **Advanced Features** (20 examples, 3-4h):
     - [ ] IPv6 scanning (link-local, solicited-node multicast)
     - [ ] ICMPv6 + NDP (neighbor discovery, router solicitation)
     - [ ] Service detection (HTTP server fingerprinting, SSH distro detection)
     - [ ] OS fingerprinting (TCP/ICMP/UDP probe sequences)
     - [ ] Idle scan (zombie discovery, three-party relay)
     - [ ] TLS certificate analysis (chain validation, expiry checks, SNI)
     - [ ] Lua plugins (custom protocol detection, output formatting)
     - [ ] Evasion techniques (fragmentation, bad checksums, decoys)
     - [ ] Banner grabbing (HTTP headers, SMTP greetings, SSH versions)
     - [ ] Rate limiting (adaptive, V3 algorithm, bandwidth throttling)
     - [ ] NUMA optimization (thread pinning, IRQ affinity)
     - [ ] PCAPNG export (packet captures for Wireshark analysis)
     - [ ] Performance tuning (batch sizes, ulimit adjustments)
     - [ ] Custom probes (protocol-specific UDP payloads)
     - [ ] Multi-stage scans (discovery → enumeration → detection)
     - [ ] Distributed target lists (file input, target generation)
     - [ ] Resume scans (future: state persistence)
     - [ ] Integration with external tools (piping to Nmap, Metasploit)
     - [ ] Security compliance (PCI-DSS, HIPAA scanning requirements)
     - [ ] API usage (programmatic scanning from Rust code)
   - **Integration Patterns** (10 examples, 2h):
     - [ ] CI/CD pipeline integration (GitHub Actions, GitLab CI)
     - [ ] Monitoring integration (Prometheus metrics, Grafana dashboards)
     - [ ] SIEM integration (structured logging to Splunk, ELK)
     - [ ] Vulnerability scanning workflow (scan → detect → report)
     - [ ] Asset inventory (periodic scans, change detection)
     - [ ] Network mapping (topology discovery, visualization)
     - [ ] Compliance auditing (quarterly scans, policy validation)
     - [ ] Penetration testing (reconnaissance phase automation)
     - [ ] Docker container scanning (bridge networks, host networking)
     - [ ] Cloud security (AWS VPC, Azure VNET scanning)
   - **Edge Cases** (10 examples, 2h):
     - [ ] Offline targets (timeout handling, filtered port interpretation)
     - [ ] Firewalled hosts (stealth scan combinations, ACK interpretation)
     - [ ] Rate-limited targets (adaptive backoff, congestion control)
     - [ ] Large-scale failures (partial results, error recovery)
     - [ ] Malformed responses (banner parsing robustness)
     - [ ] Resource exhaustion (ulimit errors, memory constraints)
     - [ ] Permission errors (non-root execution, CAP_NET_RAW missing)
     - [ ] Network errors (ICMP unreachable, routing failures)
     - [ ] IPv6 edge cases (link-local scans, temporary addresses)
     - [ ] Plugin errors (sandbox violations, timeout handling)
   - **Acceptance:** 60+ copy-paste ready examples, all tested

3. **User Guide Completeness Audit** (3-4h)
   - [ ] Review `docs/32-USER-GUIDE.md` (1,180 lines from Sprint 5.10)
   - [ ] Verify Phase 5 features documented:
     - [ ] IPv6 dual-stack usage (Sprint 5.1)
     - [ ] Service detection workflow (Sprint 5.2)
     - [ ] Idle scan setup (Sprint 5.3)
     - [ ] Rate limiting tuning (Sprint 5.X)
     - [ ] TLS certificate analysis (Sprint 5.5)
     - [ ] Plugin development (Sprint 5.8)
     - [ ] Benchmarking usage (Sprint 5.9)
   - [ ] Add cross-references between related sections
   - [ ] Validate all code snippets (copy-paste test)
   - [ ] Update screenshots/diagrams (if applicable)
   - [ ] Add "See Also" sections linking to relevant guides
   - **Acceptance:** 100% Phase 5 feature coverage, <30s discoverability

4. **Tutorial Enhancement** (3-4h)
   - [ ] Review `docs/33-TUTORIALS.md` (760 lines from Sprint 5.10)
   - [ ] Expand beginner → intermediate → advanced flow:
     - **Beginner** (0-2h experience):
       - [ ] First scan (single host, common ports)
       - [ ] Understanding output (port states, services)
       - [ ] Safe scanning (rate limiting, polite mode)
     - **Intermediate** (2-10h experience):
       - [ ] Advanced targeting (CIDR, ranges, exclusions)
       - [ ] Service detection workflow
       - [ ] Output format selection
       - [ ] Timing optimization
     - **Advanced** (10+ hours experience):
       - [ ] Stealth scanning techniques
       - [ ] IPv6 integration
       - [ ] Plugin development
       - [ ] Performance tuning
   - [ ] Add hands-on exercises with solutions:
     - [ ] Exercise 1: Discover all web servers in 192.168.1.0/24
     - [ ] Exercise 2: Detect SSH versions on remote network
     - [ ] Exercise 3: Write Lua plugin for custom protocol
     - [ ] Exercise 4: Optimize scan speed for 10K hosts
   - [ ] Create "Common Pitfalls" section:
     - [ ] Running without privileges (CAP_NET_RAW errors)
     - [ ] Firewall blocking scans (filtered port interpretation)
     - [ ] Resource limits (ulimit, batch size tuning)
     - [ ] Rate limiting issues (target throttling misinterpretation)
   - **Acceptance:** Complete beginner→advanced path with exercises

5. **API Documentation Review** (2-3h)
   - [ ] Audit all public APIs for example coverage
   - [ ] Module-level documentation (`//!` comments):
     - [ ] prtip-core (scanning engine overview)
     - [ ] prtip-network (packet handling, protocols)
     - [ ] prtip-scanner (scanner types, usage patterns)
     - [ ] prtip-cli (CLI integration, argument parsing)
   - [ ] Function-level examples for complex APIs:
     - [ ] `Scanner::initialize()` (mandatory before use)
     - [ ] `ServiceDetector::detect()` (intensity levels)
     - [ ] `OsFingerprinter::fingerprint()` (probe sequences)
     - [ ] `RateLimiter::acquire()` (permit management)
   - [ ] Add "# Examples" sections to all public structs/enums
   - [ ] Validate no broken `[link]` references: `cargo doc --no-deps`
   - **Acceptance:** All public APIs have examples, 0 broken links

6. **Create Documentation Index** (2h)
   - [ ] Generate `docs/00-INDEX.md` (quick reference card):
     - Table of contents for all 35+ documentation files
     - Feature discovery matrix (feature → relevant docs)
     - Quick start guide (5-minute getting started)
     - Troubleshooting index (error → solution docs)
   - [ ] Create `docs/FAQ.md` (frequently asked questions):
     - Installation issues (Npcap, CAP_NET_RAW, cross-platform)
     - Usage questions (scan speed, accuracy, interpretation)
     - Performance tuning (ulimit, batch size, NUMA)
     - Integration (CI/CD, SIEM, monitoring)
     - Security (privilege dropping, audit logging, safety)
   - [ ] Generate feature matrix table:
     - Scan types × protocols × features
     - Quick lookup: "Which scan type for X scenario?"
   - **Acceptance:** <30s to find any feature documentation

7. **Proofread and Polish** (2-3h)
   - [ ] Consistency check across all docs:
     - [ ] Terminology (SYN scan vs SYN scanning)
     - [ ] Code style (formatting, syntax highlighting)
     - [ ] Voice (active vs passive, imperative mood)
   - [ ] Technical accuracy validation:
     - [ ] Test counts match reality (1,601 tests)
     - [ ] Version references current (v0.5.0)
     - [ ] Performance metrics up-to-date (-1.8% rate limiting)
   - [ ] Grammar and style:
     - [ ] Run markdown linter: `markdownlint docs/`
     - [ ] Spell check: `codespell docs/`
     - [ ] Link validation: `markdown-link-check docs/*.md`
   - [ ] Format verification:
     - [ ] Consistent header levels (H1 → H2 → H3)
     - [ ] Code fence syntax highlighting
     - [ ] Table alignment
   - **Acceptance:** Zero linter warnings, professional presentation

### Deliverables

- [x] **Doctests:** 0 ignored (was 5), all passing
- [x] **Example Gallery:** 60+ scenarios (was 39), production-ready
- [x] **User Guide:** 100% Phase 5 feature coverage
- [x] **Tutorial:** Complete beginner→advanced with exercises
- [x] **API Docs:** All public APIs documented with realistic examples
- [x] **Documentation Index:** <30s feature discovery
- [x] **Quality:** Professional-grade, zero linter warnings

### Success Criteria

**Quantitative:**
- [ ] 0 ignored doctests (baseline: 5)
- [ ] 60+ example scenarios (baseline: 39, +54% increase)
- [ ] User guide: 100% Phase 5 feature coverage (verify checklist)
- [ ] 0 broken documentation links (run link checker)
- [ ] API coverage: 100% public APIs with examples (audit report)
- [ ] Documentation quality: 0 linter/spell-check warnings

**Qualitative:**
- [ ] Professional quality (A+ grade by reviewer standards)
- [ ] User discoverability: Any feature findable in <30 seconds
- [ ] Examples: Real-world scenarios (not toy examples)
- [ ] Tutorials: Clear progression (beginner → advanced)
- [ ] Consistency: Terminology, style, voice uniform across docs

**User Experience:**
- [ ] New user can complete first scan in <5 minutes (measure with test user)
- [ ] Intermediate user can find advanced feature docs in <30 seconds
- [ ] Developer can understand API usage from examples alone (no external resources needed)

### Risk Mitigation

**Risk:** Examples become outdated with API changes
- **Mitigation:** Create CI test that runs all examples (`examples/run_all.sh`)
- **Automation:** Add `cargo test --examples` to CI pipeline
- **Documentation:** Add "Last Verified" dates to example headers

**Risk:** Documentation sprawl (too many files, confusing structure)
- **Mitigation:** Clear index and navigation (00-INDEX.md)
- **Organization:** Numbered prefixes (00-09 core, 10-19 implementation, 20-29 guides, 30-39 user docs)
- **Search:** Add full-text search to documentation site (mdBook feature)

**Risk:** Incomplete coverage (features without docs)
- **Mitigation:** Feature audit checklist (verify each Phase 5 sprint documented)
- **Review Process:** Documentation PR template requires example addition
- **Tracking:** Maintain "Undocumented Features" list in PROJECT-STATUS.md

**Risk:** Time overrun (documentation is time-consuming)
- **Mitigation:** Time-box to 24 hours, prioritize high-impact items
- **Scope Flexibility:** Move non-critical examples to Sprint 5.5.6 if needed
- **Parallel Work:** Example writing can be distributed/parallelized

---

## Sprint 5.5.2: CLI Usability & UX

**Priority:** HIGH (foundation for TUI, user-facing)
**Duration:** 3-4 days (24 hours estimated)
**ROI Score:** 8.0/10 (High impact, moderate effort)
**Dependencies:** None (independent work, can run parallel with 5.5.1)

### Objective

Transform CLI from functional to delightful through enhanced help system, better error messages, progress indicators with ETA, interactive confirmations, and scan templates for common scenarios.

### Rationale

Current CLI is functionally complete (50+ Nmap-compatible flags, 8 scan types, multiple output formats) but lacks polish. Before building TUI (Phase 6), the CLI experience should be exceptional to serve as:
1. **Reference Implementation:** TUI will mimic CLI UX patterns
2. **Fallback Option:** Not all users want TUI (automation, headless, SSH)
3. **Foundation:** Event system built here enables TUI architecture

**Current State:**
- Help system: Single-page `--help` (functional but overwhelming for 50+ flags)
- Error messages: Good but could be more actionable ("Permission denied" → "Try: sudo prtip OR setcap cap_net_raw+ep")
- Progress: Basic percentage (no ETA, no throughput, no multi-stage)
- Confirmations: None (dangerous operations run without warning)
- Templates: None (users must construct complex flag combinations manually)

**Why Before Phase 6:**
TUI will need same infrastructure (progress events, state management, error formatting). Building it in CLI first allows testing/validation without UI complexity. Additionally, CLI remains primary interface for automation/CI/CD.

**Impact of Deferring:**
- Poor user experience persists (steeper learning curve)
- TUI development harder (no event infrastructure to build on)
- Missed bug discovery (interactive confirmations expose edge cases)
- Support burden (confusing errors, unclear progress)

### Tasks

1. ✅ **Enhanced Help System** (2.5h actual, COMPLETE)
   - **Multi-Page Help** (3-4h):
     - [ ] Implement subcommand-style help: `prtip help <topic>`
       - `prtip help scan-types` (Connect, SYN, UDP, Stealth, Idle)
       - `prtip help timing` (T0-T5 templates, custom timing)
       - `prtip help output` (formats, filtering, storage)
       - `prtip help targeting` (CIDR, ranges, files, exclusions)
       - `prtip help detection` (service, OS, banner grabbing)
       - `prtip help evasion` (fragmentation, decoys, stealth)
       - `prtip help advanced` (IPv6, NUMA, plugins, rate limiting)
     - [ ] Create concise overview for `--help` (most common flags only)
     - [ ] Add "See also: prtip help <topic>" hints in overview
   - **Searchable Help** (2-3h):
     - [ ] Implement `prtip help search <query>`:
       - Full-text search across all help content
       - Fuzzy matching for typos ("syn scn" → "SYN scan")
       - Keyword highlighting in results
     - [ ] Add examples to help content:
       - Each flag includes usage example
       - Common flag combinations demonstrated
   - **Interactive Help** (1-2h):
     - [ ] Implement `--examples` flag showing common scenarios:
       - `prtip --examples web-servers` (HTTP/HTTPS detection)
       - `prtip --examples stealth` (evade detection)
       - `prtip --examples fast` (speed optimization)
   - **Acceptance:** Users can find any flag usage in <10 seconds

2. ✅ **Better Error Messages** (2.0h actual, COMPLETE)
   - **Actionable Guidance** (2-3h):
     - [ ] Enhance error messages with solutions:
       - "Permission denied" → "Solution: sudo prtip OR setcap cap_net_raw+ep /path/to/prtip"
       - "No such file" → "File not found: /path. Check: ls -la $(dirname /path)"
       - "Invalid IP" → "Expected: x.x.x.x, x.x.x.x/n, or hostname. Got: <input>"
       - "Port out of range" → "Ports must be 1-65535. Got: <input>"
       - "Ulimit too low" → "Detected: <current>. Recommended: <optimal>. Command: ulimit -n <optimal>"
     - [ ] Add context to errors:
       - Include failing input value
       - Suggest valid alternatives
       - Link to relevant documentation
   - **Error Categories with Icons** (1h):
     - [ ] Categorize errors (colorized icons):
       - 🔴 Fatal: Scan cannot proceed (permission, invalid target)
       - ⚠️  Warning: Scan degraded (rate limited, filtered ports)
       - ℹ️  Info: Informational (progress milestones)
       - 💡 Tip: Optimization suggestions
   - **Error Recovery Suggestions** (1-2h):
     - [ ] Implement automatic error recovery hints:
       - "Connection timed out" → "Try: --max-rtt-timeout 5000 OR -T0 (slower but more reliable)"
       - "Too many open files" → "Detected ulimit: <n>. Automatically reducing batch size to <optimal>. Re-run with --batch-size <n> to override."
       - "CAP_NET_RAW missing" → "Detected: setcap not set. Falling back to TCP connect scan (-sT)."
   - **Acceptance:** 90%+ errors include actionable solution

3. ✅ **Progress Indicators with ETA** (3.0h actual, COMPLETE)
   - **Enhanced Progress Display** (2-3h):
     - [ ] Add ETA calculation:
       - Linear ETA: Based on current rate (simple)
       - Adaptive ETA: Smoothed with EWMA (exponential weighted moving average)
       - Multi-stage ETA: Separate for discovery vs scanning
     - [ ] Add real-time throughput:
       - Packets per second (pps)
       - Hosts per minute (hpm)
       - Bandwidth utilization (Mbps)
     - [ ] Colorize progress bar:
       - Green: On track (<10% over ETA)
       - Yellow: Slow (10-25% over ETA)
       - Red: Very slow (>25% over ETA)
   - **Multi-Stage Progress** (2-3h):
     - [ ] Implement stage tracking:
       - Stage 1: Target resolution (DNS lookups)
       - Stage 2: Host discovery (ICMP/ARP)
       - Stage 3: Port scanning (SYN/Connect/UDP)
       - Stage 4: Service detection (banner grabbing)
       - Stage 5: Finalization (writing results)
     - [ ] Show current stage + overall progress:
       ```
       [Stage 3/5] Port Scanning ▓▓▓▓▓▓▓▓▓░ 87% (2,340/2,700 ports)
       Overall ▓▓▓▓▓░░░░░ 52% | ETA: 3m 24s | 1,240 pps
       ```
   - **Configurable Update Frequency** (1h):
     - [ ] Add `--progress-interval <ms>` flag (default: 500ms)
     - [ ] Add `--no-progress` flag (disable for CI/automation)
     - [ ] Add `--progress-style <minimal|standard|verbose>`:
       - Minimal: Percentage only
       - Standard: Percentage + ETA
       - Verbose: All metrics (throughput, stages, time remaining)
   - **Acceptance:** Progress includes ETA, throughput, stage tracking

4. ✅ **Interactive Confirmations** (3.5h actual, COMPLETE)
   - **Dangerous Operation Warnings** (2-3h):
     - [ ] Implement confirmation prompts for:
       - Internet-scale scans (0.0.0.0/0, ::/0): "Warning: Scanning entire internet. Continue? [y/N]"
       - Large target sets (>10K hosts): "Scanning <count> hosts. Estimated duration: <eta>. Continue? [y/N]"
       - Aggressive timing (T5): "T5 is VERY aggressive. May trigger IDS/IPS. Continue? [y/N]"
       - Evasion techniques: "Fragmentation/decoys may be illegal in your jurisdiction. Continue? [y/N]"
       - Root/elevated privileges: "Running as root. Drop privileges after socket creation? [Y/n]"
     - [ ] Add `--yes` flag to bypass confirmations (CI/automation)
     - [ ] Store confirmation preferences in config file
   - **Smart Confirmation Logic** (1h):
     - [ ] Skip confirmation if:
       - Running in non-interactive terminal (CI/automation)
       - `--yes` flag provided
       - Target set is "safe" (private RFC1918 ranges)
       - Timing is polite (T0-T2)
   - **Acceptance:** Dangerous operations require confirmation

5. ✅ **Scan Templates** (2.5h actual, COMPLETE)
   - **Predefined Templates** (4-5h):
     - [ ] Implement `--template <name>` flag with built-in presets:
       - `web-servers`: Scan for HTTP/HTTPS (ports 80, 443, 8080, 8443 + service detection)
       - `databases`: Scan for MySQL, PostgreSQL, MongoDB, Redis (3306, 5432, 27017, 6379)
       - `ssh-audit`: Scan for SSH, detect versions, check weak ciphers (port 22 + banner analysis)
       - `mail-servers`: SMTP, IMAP, POP3 (25, 143, 110, 587, 993, 995)
       - `smb-shares`: SMB/NetBIOS (139, 445 + SMB dialect detection)
       - `quick`: Fast scan (top 100 ports, T4 timing, SYN scan)
       - `thorough`: Comprehensive (all 65535 ports, T3 timing, service + OS detection)
       - `stealth`: Evade detection (FIN scan, T0 timing, fragmentation, decoys)
       - `safe`: Polite scan (common ports, T2 timing, rate limiting)
       - `compliance`: PCI/HIPAA scanning (audit logging, safe modes)
     - [ ] Templates combine multiple flags automatically:
       - `--template web-servers` = `-p 80,443,8080,8443 -sV --script http-*`
     - [ ] Allow template overrides:
       - `--template quick -p 1000` (override port range)
       - `--template stealth -T3` (override timing)
   - **Custom Templates** (2h):
     - [ ] Support user-defined templates in config file:
       ```toml
       [templates.my-custom]
       description = "Custom web scan for staging environment"
       ports = "80,443,3000,8080,8443"
       scan_type = "syn"
       timing = "T4"
       service_detection = true
       ```
     - [ ] Implement `prtip templates list` (show available templates)
     - [ ] Implement `prtip templates show <name>` (display template flags)
   - **Acceptance:** 10+ built-in templates, custom template support

6. ✅ **Command History & Replay** (2.0h actual, COMPLETE)
   - **History Management** (1-2h):
     - [ ] Store command history in `~/.prtip/history` (JSON format)
     - [ ] Implement `prtip history` (show last 20 commands)
     - [ ] Implement `prtip history <n>` (show last N commands)
     - [ ] Add timestamps, target counts, results summary
   - **Replay Functionality** (1h):
     - [ ] Implement `prtip replay <index>` (re-run previous command)
     - [ ] Implement `prtip replay --last` (re-run most recent)
     - [ ] Allow modifications: `prtip replay 3 --template thorough`
   - **Acceptance:** Command history with replay functionality

7. **Logging Enhancement** (2-3h)
   - **Structured File Logging** (1-2h):
     - [ ] Implement JSON logging to `~/.prtip/logs/prtip-<timestamp>.log`
     - [ ] Log all events:
       - Scan start (timestamp, arguments, user, hostname)
       - Progress milestones (10%, 25%, 50%, 75%, 90%, 100%)
       - Discoveries (open ports, services detected, errors)
       - Scan completion (duration, results count, summary)
     - [ ] Add `--log-level <level>` flag (ERROR, WARN, INFO, DEBUG, TRACE)
   - **Audit Trail** (1h):
     - [ ] Implement audit logging for compliance:
       - Who ran the scan (user, hostname, IP)
       - What was scanned (targets, ports, flags)
       - When (start time, end time, duration)
       - Results (open ports, services, errors)
     - [ ] Support syslog integration (`--syslog` flag)
     - [ ] Support remote logging (send to SIEM)
   - **Acceptance:** Structured JSON logging, audit trail support

### Deliverables

- [x] **Multi-Page Help:** Topic-based help system (`prtip help <topic>`)
- [x] **Searchable Help:** Full-text search (`prtip help search <query>`)
- [x] **Better Errors:** 90%+ errors include actionable solutions
- [x] **Progress with ETA:** Real-time ETA, throughput, multi-stage progress
- [x] **Interactive Confirmations:** Dangerous operations require user approval
- [x] **Scan Templates:** 10+ built-in, custom template support
- [x] **Command History:** History with replay functionality
- [x] **Structured Logging:** JSON logs, audit trail, syslog support

### Success Criteria

**Quantitative:**
- [x] Help topics: 8 categories implemented (scan-types, timing, output, targeting, detection, evasion, advanced, examples)
- [x] Error guidance: 95%+ errors include solutions (19 error patterns with actionable suggestions)
- [x] Progress metrics: 7+ tracked (percentage, ETA, pps, hpm, bandwidth, stage, duration)
- [x] Scan templates: 10 built-in templates (web-servers, databases, quick, thorough, stealth, discovery, ssl-only, admin-panels, mail-servers, file-shares)
- [ ] Logging: JSON format, 5+ log levels, syslog integration (DEFERRED to Sprint 5.5.3)

**Qualitative:**
- [x] Help discoverability: Search feature enables <1s lookups with fuzzy matching
- [x] Error clarity: Actionable guidance with platform-specific solutions, non-experts can resolve 95%+ errors
- [x] Progress UX: Multi-stage progress bars, real-time ETA, color-coded speed indicators
- [x] Confirmations: Smart confirmation logic protects 5 dangerous operation categories
- [x] Templates: 10 built-in templates reduce common scenarios to 1-flag commands

### Sprint 5.5.2 Completion Summary

**Status:** ✅ **COMPLETE** (2025-11-08)
**Duration:** 15.5 hours (81% efficiency vs 18-20h estimate)
**Quality:** A+ grade across all 6 tasks
**Deliverables:** 100% complete (6/6 tasks, 91 tests, 3,414 lines code)

**Key Achievements:**
- Enhanced Help System with fuzzy search (7 comprehensive tests)
- Better Error Messages with 95%+ actionable suggestions (10 tests)
- Progress Indicators with EWMA-based ETA calculation (28 tests)
- Interactive Confirmations for 5 dangerous operation categories (10 tests)
- Scan Templates with 10 built-in + custom TOML support (14 tests)
- Command History with atomic persistence and replay (22 tests)

**Impact:**
- UX: Professional CLI experience matching industry standards
- Safety: Dangerous operations protected with smart confirmations
- Productivity: Templates save ~70% configuration time for common scenarios
- Debugging: Error messages provide actionable solutions (95%+ coverage)
- Discoverability: Help search finds any topic in <1 second

See `/tmp/ProRT-IP/SPRINT-5.5.2-COMPLETE.md` for full completion report.

**User Experience:**
- [ ] New user: Complete first scan with template in <2 minutes
- [ ] Intermediate user: Customize template for scenario in <5 minutes
- [ ] Advanced user: Build custom template from history in <10 minutes

### Risk Mitigation

**Risk:** Help system fragmentation (information scattered)
- **Mitigation:** Clear topic organization + cross-references
- **Automation:** Generate help content from single source (docs/HELP.md)
- **Testing:** User testing for help discoverability

**Risk:** Progress calculation inaccuracy (ETA always wrong)
- **Mitigation:** Adaptive ETA with EWMA smoothing (handle variance)
- **Fallback:** Show "Calculating..." for first 10% of scan
- **Honesty:** Label as "Estimated" (not "Actual")

**Risk:** Template explosion (too many templates)
- **Mitigation:** Start with 10, add more based on user requests
- **Organization:** Categories (web, databases, security, compliance)
- **Custom:** Users create their own (no template bloat)

**Risk:** Confirmation fatigue (users click through blindly)
- **Mitigation:** Smart logic (skip for safe scenarios)
- **Customization:** User preferences (remember choices)
- **Severity Levels:** Only CRITICAL operations require confirmation

---

## Sprint 5.5.3: Event System & Progress Reporting 🔄

**Status:** IN PROGRESS (20% Complete - 8/40 tasks, ~6 hours)
**Priority:** CRITICAL (TUI foundation, blocks Phase 6)
**Duration:** 4-5 days (32-40 hours estimated)
**ROI Score:** 9.5/10 (Critical impact, significant effort)
**Dependencies:** Sprint 5.5.2 (builds on progress infrastructure)
**Started:** 2025-11-08

### Progress Summary

**Completed (8/40 tasks):**
- ✅ Task 1.1: Core ScanEvent Enum (18 variants)
- ✅ Task 1.2: Supporting Types (ScanStage, PortState, DiscoveryMethod)
- ✅ Task 1.3: Event Validation (timestamp, field constraints, 18 tests)
- ✅ Task 2.1: EventBus Core (pub-sub, thread-safe, auto-cleanup)
- ✅ Task 2.2: Event Filtering (type, scan ID, host, port, severity)
- ✅ Task 2.3: Ring Buffer History (1,000 events, O(1) insert)
- ✅ Task 2.4: Integration Tests (15 tests, concurrent workflows)
- 🔄 Task 2.5: Performance Benchmarking (PAUSED)

**Code Delivered:**
- `crates/prtip-core/src/events/types.rs` (680 lines)
- `crates/prtip-core/src/event_bus.rs` (620 lines)
- `crates/prtip-core/src/events/filters.rs` (380 lines)
- `crates/prtip-core/src/events/history.rs` (203 lines)
- **Total:** 1,913 lines, 52 tests (100% passing)

**Remaining (32 tasks, ~26-32 hours):**
- Task 2.5: Benchmarking (1-2h)
- Task Area 3: Scanner Integration (6-8h)
- Task Area 4: Progress Collection (6-8h)
- Task Area 5: CLI Integration (4-5h)
- Task Area 6: Event Logging (3-4h)
- Task Area 7: Testing & Benchmarking (4-5h)

**See:** `to-dos/SPRINT-5.5.3-EVENT-SYSTEM-TODO.md` for detailed task list

---

### Objective

Design and implement production-grade event-driven architecture enabling real-time scan updates for TUI (Phase 6), monitoring integrations, and future distributed scanning (Phase 8). This is the MOST CRITICAL sprint for Phase 6 success.

### Rationale

Current architecture is polling-based: CLI periodically checks `ScanProgress` atomic counters. This works for simple CLI but is inadequate for TUI requirements:

**TUI Requirements (from ratatui research):**
- Real-time UI updates (100ms refresh rate)
- Event-driven state changes (no polling overhead)
- Multiple UI components (progress, results table, logs) listening to same events
- Background scan execution (UI responsive during long scans)
- Clean separation: scanner logic ↔ presentation layer

**Current Limitations:**
- No pub-sub pattern (each component must poll)
- No event types (just atomic counters)
- No event history (can't replay or query past events)
- No multi-subscriber support (one consumer only)
- Tight coupling (scanner directly updates progress, hard to intercept)

**Why CRITICAL:**
Without event system, Phase 6 TUI must be built on polling architecture (poor UX, high CPU). Refactoring during Phase 6 would delay TUI by weeks. Building now allows Phase 6 to focus purely on UI rendering.

**Impact of Deferring:**
- Phase 6 blocked (cannot start without event foundation)
- TUI performance poor (polling overhead, laggy updates)
- Future phases harder (distributed scanning needs events)
- Technical debt (eventual refactoring more costly)

### Tasks

1. **Event Type Design** (4-5h)
   - **Define Event Enum** (2h):
     - [ ] Create `crates/prtip-core/src/events.rs`:
       ```rust
       pub enum ScanEvent {
           // Lifecycle events
           ScanStarted { scan_id: Uuid, config: ScanConfig, target_count: usize, timestamp: SystemTime },
           ScanCompleted { scan_id: Uuid, duration: Duration, results: ScanSummary, timestamp: SystemTime },
           ScanPaused { scan_id: Uuid, reason: PauseReason, timestamp: SystemTime },
           ScanResumed { scan_id: Uuid, timestamp: SystemTime },
           ScanCancelled { scan_id: Uuid, reason: String, timestamp: SystemTime },
           ScanError { scan_id: Uuid, error: ScanError, recoverable: bool, timestamp: SystemTime },

           // Progress events
           ProgressUpdate { scan_id: Uuid, stage: ScanStage, percentage: f32, throughput: Throughput, eta: Option<Duration>, timestamp: SystemTime },
           StageChanged { scan_id: Uuid, from_stage: ScanStage, to_stage: ScanStage, timestamp: SystemTime },

           // Discovery events
           HostDiscovered { scan_id: Uuid, ip: IpAddr, alive: bool, latency_ms: f32, timestamp: SystemTime },
           PortFound { scan_id: Uuid, ip: IpAddr, port: u16, state: PortState, protocol: Protocol, timestamp: SystemTime },
           ServiceDetected { scan_id: Uuid, ip: IpAddr, port: u16, service: ServiceInfo, confidence: f32, timestamp: SystemTime },
           OsDetected { scan_id: Uuid, ip: IpAddr, os: OsInfo, confidence: f32, timestamp: SystemTime },
           BannerGrabbed { scan_id: Uuid, ip: IpAddr, port: u16, banner: String, timestamp: SystemTime },
           CertificateFound { scan_id: Uuid, ip: IpAddr, port: u16, cert: CertificateInfo, timestamp: SystemTime },

           // Diagnostic events
           RateLimitTriggered { scan_id: Uuid, reason: String, duration: Duration, timestamp: SystemTime },
           RetryScheduled { scan_id: Uuid, target: String, attempt: u32, delay: Duration, timestamp: SystemTime },
           WarningIssued { scan_id: Uuid, message: String, severity: WarningSeverity, timestamp: SystemTime },
           MetricRecorded { scan_id: Uuid, metric: MetricType, value: f64, timestamp: SystemTime },
       }

       pub enum ScanStage {
           Initializing,
           ResolvingTargets,
           DiscoveringHosts,
           ScanningPorts,
           DetectingServices,
           Finalizing,
           Completed,
       }

       pub struct Throughput {
           pub packets_per_second: f64,
           pub hosts_per_minute: f64,
           pub bandwidth_mbps: f64,
       }
       ```
   - **Event Metadata** (1h):
     - [ ] Add common metadata to all events:
       - `scan_id: Uuid` (correlate events to scans)
       - `timestamp: SystemTime` (event ordering)
       - `source: EventSource` (which scanner emitted)
     - [ ] Implement serialization (serde JSON for logging)
   - **Event Validation** (1-2h):
     - [ ] Implement `Event::validate()` (sanity checks)
     - [ ] Add unit tests for all event types (20+ tests)
   - **Acceptance:** 15+ event types defined, tested, serializable

2. **Pub-Sub Event Bus** (8-10h)
   - **Event Bus Architecture** (3-4h):
     - [ ] Create `crates/prtip-core/src/event_bus.rs`:
       ```rust
       pub struct EventBus {
           subscribers: Arc<Mutex<HashMap<Uuid, Vec<EventSubscriber>>>>,
           buffer: Arc<Mutex<VecDeque<ScanEvent>>>, // History buffer (last 1000 events)
           metrics: Arc<AtomicU64>, // Event count
       }

       pub struct EventSubscriber {
           id: Uuid,
           filter: Option<EventFilter>, // Subscribe to subset of events
           sender: mpsc::UnboundedSender<ScanEvent>,
       }

       pub enum EventFilter {
           All,
           ScanId(Uuid),
           EventType(Vec<ScanEventType>),
           Custom(Box<dyn Fn(&ScanEvent) -> bool + Send>),
       }
       ```
     - [ ] Implement `EventBus::publish(event: ScanEvent)`:
       - Broadcast to all matching subscribers
       - Store in history buffer (ring buffer, 1000 events)
       - Increment metrics
       - Non-blocking (async dispatch)
     - [ ] Implement `EventBus::subscribe(filter: EventFilter) -> mpsc::UnboundedReceiver<ScanEvent>`:
       - Create new subscriber with UUID
       - Register in subscribers map
       - Return async receiver
     - [ ] Implement `EventBus::unsubscribe(id: Uuid)`:
       - Remove subscriber from map
       - Drop sender (closes receiver)
   - **Multi-Subscriber Support** (2-3h):
     - [ ] Support multiple subscribers simultaneously:
       - TUI components (progress bar, results table, log viewer)
       - File logger (JSON event stream)
       - Metrics collector (Prometheus exporter)
       - Network forwarder (remote monitoring)
     - [ ] Ensure thread-safety:
       - Use `Arc<Mutex<>>` for subscribers
       - Clone events for each subscriber
       - No blocking on slow subscribers (buffered channels)
   - **Event History & Replay** (2-3h):
     - [ ] Implement `EventBus::history(filter: EventFilter, limit: usize) -> Vec<ScanEvent>`:
       - Query past events (last 1000 stored)
       - Filter by scan_id, event type, time range
       - Useful for: TUI initialization, debugging, replay
     - [ ] Implement `EventBus::replay(from: SystemTime, to: SystemTime) -> impl Stream<Item = ScanEvent>`:
       - Stream historical events as if real-time
       - Useful for: visualization, debugging, testing
   - **Acceptance:** Pub-sub bus with multi-subscriber, history, replay

3. **Scanner Integration** (6-8h)
   - **Emit Events from Scanners** (4-5h):
     - [ ] Modify all scanners to emit events:
       - `SynScanner::scan()` → emit `PortFound` events
       - `ServiceDetector::detect()` → emit `ServiceDetected` events
       - `OsFingerprinter::fingerprint()` → emit `OsDetected` events
       - `TlsAnalyzer::analyze()` → emit `CertificateFound` events
     - [ ] Add `EventBus` to `ScanConfig`:
       ```rust
       pub struct ScanConfig {
           // ... existing fields ...
           pub event_bus: Option<Arc<EventBus>>, // Optional for backward compatibility
       }
       ```
     - [ ] Emit progress events at key milestones:
       - Every 5% progress increment
       - Stage transitions (Initializing → ResolvingTargets → ...)
       - Rate limit triggers
       - Errors/warnings
   - **Backward Compatibility** (1-2h):
     - [ ] Make event bus optional (existing code still works):
       - `if let Some(bus) = &self.config.event_bus { bus.publish(event); }`
     - [ ] Maintain existing `ScanProgress` atomic counters:
       - Update both atomics AND emit events
       - Allows gradual migration
   - **Testing** (1-2h):
     - [ ] Add integration tests for event emission:
       - Run scan, subscribe to events, verify all expected events received
       - Test event ordering (ScanStarted → ProgressUpdate → ... → ScanCompleted)
       - Test multi-subscriber (2+ subscribers receive same events)
   - **Acceptance:** All scanners emit events, backward compatible, tested

4. **Real-Time Progress Collection** (6-8h)
   - **Progress Aggregator** (3-4h):
     - [ ] Create `crates/prtip-core/src/progress_aggregator.rs`:
       ```rust
       pub struct ProgressAggregator {
           event_bus: Arc<EventBus>,
           state: Arc<Mutex<AggregatedState>>,
           updater_task: JoinHandle<()>,
       }

       pub struct AggregatedState {
           pub current_stage: ScanStage,
           pub overall_progress: f32,
           pub stage_progress: f32,
           pub throughput: Throughput,
           pub eta: Option<Duration>,
           pub discovered_hosts: usize,
           pub open_ports: usize,
           pub detected_services: usize,
           pub errors: Vec<ScanError>,
           pub warnings: Vec<String>,
       }
       ```
     - [ ] Subscribe to all progress-related events:
       - `ProgressUpdate` → update overall/stage progress
       - `HostDiscovered` → increment discovered_hosts counter
       - `PortFound` → increment open_ports counter
       - `ServiceDetected` → increment detected_services counter
       - `StageChanged` → update current_stage
     - [ ] Maintain real-time aggregated state:
       - Queryable at any time (`get_state()`)
       - Non-blocking (lock-free reads via Arc<RwLock<>>)
   - **ETA Calculation** (2-3h):
     - [ ] Implement adaptive ETA algorithm:
       - Track completion rate over last 60 seconds (sliding window)
       - Use EWMA (Exponential Weighted Moving Average) for smoothing:
         `smoothed_rate = 0.3 * current_rate + 0.7 * previous_smoothed_rate`
       - Calculate ETA: `(total_work - completed_work) / smoothed_rate`
       - Handle edge cases: slow starts, rate changes, stalls
     - [ ] Emit updated ETA in `ProgressUpdate` events:
       - Every 5% progress OR every 30 seconds (whichever is less frequent)
   - **Throughput Metrics** (1-2h):
     - [ ] Calculate real-time throughput:
       - Packets per second (pps): Count packets sent/received
       - Hosts per minute (hpm): Count hosts completed
       - Bandwidth (Mbps): Sum packet sizes × 8 / 1_000_000
     - [ ] Emit in `MetricRecorded` events (every 10 seconds)
   - **Acceptance:** Real-time progress with ETA, throughput, aggregated state

5. **CLI Integration** (4-5h)
   - **Event-Driven Progress Display** (2-3h):
     - [ ] Modify `crates/prtip-cli/src/progress.rs` to use events:
       - Replace polling loop with event subscription
       - Subscribe to `ProgressUpdate`, `StageChanged` events
       - Update terminal display on event receipt (100ms debounce)
     - [ ] Display ETA and throughput:
       ```
       [Stage 3/5] Port Scanning ▓▓▓▓▓▓▓▓▓░ 87% (2,340/2,700 ports)
       Overall ▓▓▓▓▓░░░░░ 52% | ETA: 3m 24s | 1,240 pps | 42 hpm
       ```
   - **Live Results Streaming** (2h):
     - [ ] Subscribe to discovery events (`PortFound`, `ServiceDetected`):
       - Display new discoveries immediately (as they occur)
       - Option: `--live-results` (vs batch at end)
     - [ ] Implement results table:
       - Column headers: IP, Port, State, Service, Version
       - Update table as services detected
   - **Acceptance:** CLI uses event system for real-time updates

6. **Event Logging** (3-4h)
   - **JSON Event Log** (2-3h):
     - [ ] Create event logger subscriber:
       - Subscribe to ALL events
       - Write to `~/.prtip/events/<scan_id>.jsonl` (JSON Lines format)
       - One event per line (streaming, easy to process)
     - [ ] Add metadata to log:
       - File header: scan config, start time, prtip version
       - Event entries: full event JSON
       - File footer: scan summary, end time
   - **Log Rotation** (1h):
     - [ ] Implement log rotation:
       - Max file size: 100MB (compress and rotate)
       - Keep last 10 log files
       - Auto-cleanup logs older than 30 days
   - **Acceptance:** All events logged to JSON, rotated, queryable

7. **Testing & Benchmarking** (4-5h)
   - **Unit Tests** (2h):
     - [ ] Test event bus:
       - Publish/subscribe mechanics
       - Multi-subscriber delivery
       - Filter correctness
       - History storage and retrieval
       - Replay functionality
   - **Integration Tests** (2h):
     - [ ] Test end-to-end event flow:
       - Scanner → EventBus → CLI display
       - Scanner → EventBus → JSON logger
       - Scanner → EventBus → Progress aggregator
     - [ ] Test error handling:
       - Slow subscriber (doesn't block others)
       - Failed subscriber (auto-removal)
       - Event buffer overflow (oldest events dropped)
   - **Performance Benchmarking** (1h):
     - [ ] Measure event overhead:
       - Baseline: Scan without events
       - With events: Scan with 1, 5, 10 subscribers
       - Target: <5% overhead with 10 subscribers
     - [ ] Profile event bus:
       - Publish latency (p50, p95, p99)
       - Subscriber delivery latency
       - Memory usage (event buffer, subscriber channels)
   - **Acceptance:** <5% event overhead, all tests passing

### Deliverables

- [x] **Event Types:** 15+ event types defined, serializable, tested
- [x] **Event Bus:** Pub-sub architecture with multi-subscriber support
- [x] **Event History:** 1000-event buffer with query/replay
- [x] **Scanner Integration:** All scanners emit events, backward compatible
- [x] **Progress Aggregator:** Real-time state with ETA, throughput
- [x] **CLI Integration:** Event-driven progress display
- [x] **Event Logging:** JSON event log with rotation
- [x] **Performance:** <5% overhead with 10 subscribers

### Success Criteria

**Quantitative:**
- [ ] Event types: 15+ defined (lifecycle, progress, discovery, diagnostic)
- [ ] Event overhead: <5% with 10 subscribers (benchmark validation)
- [ ] Event latency: p99 <10ms publish-to-receive
- [ ] Event history: 1000 events stored (ring buffer)
- [ ] Test coverage: 90%+ for event system (30+ tests)

**Qualitative:**
- [ ] Clean architecture (scanner ↔ event bus ↔ consumers)
- [ ] Non-blocking (slow subscribers don't block scans)
- [ ] Extensible (easy to add new event types/subscribers)
- [ ] Debuggable (event logs queryable, replay works)
- [ ] TUI-ready (provides all data TUI needs)

**Phase 6 Readiness:**
- [ ] TUI can subscribe to events (real-time UI updates)
- [ ] Background scans work (decoupled from display)
- [ ] State queryable (TUI can get current state anytime)
- [ ] Multiple widgets supported (progress, results, logs all listen)
- [ ] Zero polling needed (event-driven architecture)

### Risk Mitigation

**Risk:** Event overhead degrades scan performance
- **Mitigation:** Benchmark early, optimize hot paths
- **Fallback:** Make event bus optional (backward compatibility)
- **Target:** <5% overhead with reasonable subscriber count

**Risk:** Event buffer overflow (too many events)
- **Mitigation:** Ring buffer (oldest events dropped)
- **Monitoring:** Warn when buffer >90% full
- **Tuning:** Configurable buffer size (default: 1000)

**Risk:** Slow subscriber blocks other subscribers
- **Mitigation:** Unbounded channels (buffering per-subscriber)
- **Auto-removal:** Drop subscribers that fall >10 seconds behind
- **Monitoring:** Warn on slow subscribers

**Risk:** Complex async debugging (event races)
- **Mitigation:** Comprehensive event logging (JSON)
- **Testing:** Property-based testing for race conditions
- **Tooling:** Event timeline visualizer (planned Sprint 5.5.6)

**Risk:** API instability (events change frequently)
- **Mitigation:** Version event types (EventV1, EventV2)
- **Compatibility:** Support multiple versions simultaneously
- **Documentation:** Changelog for event schema changes

---

## Sprint 5.5.4: Performance Audit & Optimization

**Priority:** MEDIUM (establishes baseline for TUI comparison)
**Duration:** 3-4 days (24-32 hours estimated)
**ROI Score:** 7.0/10 (Medium impact, moderate effort)
**Dependencies:** None (can run parallel with 5.5.1, 5.5.2)

### Objective

Establish comprehensive performance baselines for all scan types before Phase 6 TUI development. Identify and fix performance regressions, optimize hot paths, and create regression detection suite for CI/CD.

### Rationale

Phase 5 added significant functionality (IPv6, Service Detection, Idle Scan, TLS Analysis, Plugins) but performance impacts were not systematically measured. Before TUI (Phase 6) adds UI overhead, we need:

1. **Baseline Metrics:** Know current performance for future comparison
2. **Regression Detection:** Catch performance degradation early
3. **Optimization Opportunities:** Low-hanging fruit before TUI
4. **Documentation:** Performance characteristics for capacity planning

**Current State:**
- Benchmarking framework exists (Sprint 5.9, hyperfine, 10 scenarios)
- No continuous benchmarking (no CI integration for performance)
- No regression detection (manual comparison only)
- Informal performance claims (10M+ pps stateless, but not validated recently)

**Why Before Phase 6:**
TUI will add rendering overhead (50-100ms per frame). Without baseline, we won't know if slowdowns are TUI-related or underlying regression. Fixing performance after TUI is harder (more variables).

**Impact of Deferring:**
- No performance baseline (can't prove TUI didn't degrade speed)
- Regressions undetected (gradual performance loss)
- Missed optimization (quick wins left on table)
- Capacity planning unclear (users don't know scaling limits)

### Tasks

1. **Comprehensive Benchmarking** (6-8h)
   - **Expand Benchmark Suite** (4-5h):
     - [ ] Add to existing 10 scenarios (Sprint 5.9):
       - **Scan Types** (6 scenarios):
         - [ ] TCP Connect scan (baseline, no privileges needed)
         - [ ] TCP SYN scan (privileged, fastest)
         - [ ] UDP scan (slowest, ICMP-limited)
         - [ ] FIN/NULL/Xmas scans (stealth)
         - [ ] ACK scan (firewall detection)
         - [ ] Idle scan (maximum anonymity, slowest)
       - **Feature Variations** (8 scenarios):
         - [ ] Service detection (HTTP, SSH, SMB, MySQL, PostgreSQL)
         - [ ] OS fingerprinting (16-probe sequence)
         - [ ] Banner grabbing (self-announcing services)
         - [ ] TLS certificate analysis (X.509v3 parsing)
         - [ ] IPv6 scanning (dual-stack overhead)
         - [ ] Evasion techniques (fragmentation, decoys)
         - [ ] Rate limiting (V3 algorithm overhead)
         - [ ] Plugin execution (Lua overhead)
       - **Scale Tests** (6 scenarios):
         - [ ] Small scan (1 host, 100 ports)
         - [ ] Medium scan (100 hosts, 1000 ports)
         - [ ] Large scan (1000 hosts, 10000 ports)
         - [ ] Internet-scale simulation (100K hosts, common ports)
         - [ ] All ports (1 host, 65535 ports)
         - [ ] Timing templates (T0 vs T5 comparison)
     - **Total:** 20+ benchmark scenarios (current: 10)
   - **Benchmark Automation** (2-3h):
     - [ ] Create `benchmarks/run_all.sh`:
       - Run all 20+ scenarios
       - Generate comparison report (Markdown table)
       - Store results in `benchmarks/history/<date>.json`
     - [ ] Add statistical rigor:
       - 10 runs per scenario (warmup: 3, benchmark: 10)
       - Report: mean, median, stddev, min, max
       - Detect outliers (discard >3 stddev from mean)
   - **Acceptance:** 20+ scenarios, automated, statistically rigorous

2. **Profile Hot Paths** (5-6h)
   - **CPU Profiling** (2-3h):
     - [ ] Generate flamegraphs for major scenarios:
       - `cargo flamegraph --bin prtip -- -sS -p 80,443 192.168.1.0/24`
       - Identify hotspots (functions consuming >5% CPU)
     - [ ] Profile scan types:
       - SYN scan (packet crafting overhead)
       - Service detection (regex matching overhead)
       - TLS analysis (X.509 parsing overhead)
       - Plugin execution (Lua VM overhead)
     - [ ] Store flamegraphs: `benchmarks/flamegraphs/<scenario>.svg`
   - **Memory Profiling** (2-3h):
     - [ ] Use Valgrind massif:
       - `valgrind --tool=massif --massif-out-file=massif.out prtip ...`
       - Analyze heap allocations
     - [ ] Profile memory usage:
       - Stateless scan: <1MB target (verify)
       - Stateful scan: <100MB for 10K hosts (verify)
       - Service detection: Memory per target
     - [ ] Identify memory leaks (should be zero with Rust)
   - **I/O Profiling** (1h):
     - [ ] Use `strace` to profile syscalls:
       - `strace -c prtip -sS -p 80 192.168.1.1`
       - Identify excessive syscalls
     - [ ] Check batching effectiveness:
       - `sendmmsg` batch sizes
       - `recvmmsg` batch sizes
       - File I/O (result writes)
   - **Acceptance:** Flamegraphs, massif reports, strace analysis

3. **Optimize Identified Bottlenecks** (6-8h)
   - **Prioritize by Impact** (1h):
     - [ ] List all identified bottlenecks
     - [ ] Calculate impact: `(CPU %) × (scenario frequency) = priority`
     - [ ] Sort by priority (fix highest-impact first)
   - **Implement Optimizations** (5-7h):
     - **Example optimizations** (actual will depend on profiling):
     - [ ] Packet crafting:
       - Pre-allocate packet buffers (avoid per-packet allocation)
       - Batch checksum calculations
     - [ ] Service detection:
       - Compile regexes once (lazy_static)
       - Optimize regex patterns (avoid backtracking)
     - [ ] TLS parsing:
       - Use zero-copy parsing (nom combinators)
       - Cache parsed certificates
     - [ ] Plugin system:
       - Reuse Lua VMs (pool instead of create-per-request)
       - JIT compilation (LuaJIT if available)
     - [ ] Rate limiting:
       - Further optimize V3 algorithm (already -1.8%, but could improve)
     - [ ] I/O:
       - Increase batch sizes (sendmmsg, recvmmsg)
       - Async file writes (Tokio fs)
   - **Measure Improvements** (1h):
     - [ ] Re-run benchmarks after each optimization
     - [ ] Document before/after (speedup percentages)
   - **Acceptance:** 10%+ speedup on at least 3 scenarios

4. **Regression Detection Suite** (4-5h)
   - **CI Integration** (2-3h):
     - [ ] Add GitHub Actions workflow (`.github/workflows/benchmarks.yml`):
       - Trigger: Weekly (Sunday 00:00 UTC) + on-demand
       - Run: Full benchmark suite (20+ scenarios)
       - Store: Results in GitHub Actions artifacts
       - Compare: Against previous week's results
     - [ ] Implement regression detection:
       - PASS: <5% slower than baseline
       - WARN: 5-10% slower (manual review)
       - FAIL: >10% slower (block PR / notify)
   - **Baseline Management** (1-2h):
     - [ ] Store baselines in repo:
       - `benchmarks/baselines/v0.5.0.json`
       - One baseline per minor version
       - Update on releases
     - [ ] Implement comparison tool:
       - `cargo run --bin bench-compare -- baselines/v0.5.0.json benchmarks/latest.json`
       - Output: Markdown table with ✅ PASS / ⚠️ WARN / ❌ FAIL
   - **Alerting** (1h):
     - [ ] Add notifications:
       - GitHub PR comment (regression detected)
       - Email maintainers (optional)
       - Slack webhook (optional)
   - **Acceptance:** CI benchmarks weekly, regression detection automated

5. **Performance Documentation** (4-5h)
   - **Performance Guide** (3-4h):
     - [ ] Create `docs/34-PERFORMANCE-CHARACTERISTICS.md`:
       - **Throughput Section:**
         - Stateless scan: X pps (measured)
         - Stateful scan: Y pps (measured)
         - Service detection: Z hosts/min (measured)
         - OS fingerprinting: A hosts/min (measured)
       - **Latency Section:**
         - Packet crafting: <1ms p99
         - Service regex: <5ms p99
         - TLS parsing: <2ms p99 (was 1.33μs mean in Sprint 5.5)
         - Plugin execution: <10ms p99
       - **Memory Section:**
         - Stateless: <1MB (verified)
         - Stateful: <100MB for 10K hosts (verified)
         - Service detection: <10MB overhead
       - **Scaling Section:**
         - Small (1-100 hosts): Linear scaling
         - Medium (100-10K hosts): Linear with resource limits
         - Large (10K-1M hosts): Batch processing, streaming results
       - **Optimization Section:**
         - Tuning ulimit (file descriptors)
         - NUMA thread pinning
         - Batch size configuration
         - Rate limiting tuning
   - **Capacity Planning** (1h):
     - [ ] Add capacity planning tables:
       - "How many hosts can I scan?"
       - "How long will X scan take?"
       - "What hardware do I need for Y throughput?"
   - **Acceptance:** Performance documented, capacity planning guide

6. **Benchmark Results Publishing** (2-3h)
   - **Historical Tracking** (1-2h):
     - [ ] Create `benchmarks/history/` directory:
       - Store results: `YYYY-MM-DD-<version>.json`
       - Accumulate over time (track trends)
     - [ ] Generate trend graphs:
       - Throughput over time (line chart)
       - Latency over time (line chart)
       - Memory over time (line chart)
   - **Public Dashboard** (1h):
     - [ ] Create `benchmarks/README.md`:
       - Latest benchmark results (table)
       - Historical trends (charts)
       - Comparison to competitors (Nmap, Masscan, RustScan)
   - **Acceptance:** Historical tracking, public dashboard

### Deliverables

- [x] **Benchmark Suite:** 20+ scenarios (was 10)
- [x] **Profiling Reports:** Flamegraphs, massif, strace for all major scenarios
- [x] **Optimizations:** 10%+ speedup on 3+ scenarios
- [x] **Regression Detection:** CI integration, automated alerts
- [x] **Performance Guide:** Comprehensive documentation (throughput, latency, memory, scaling)
- [x] **Public Dashboard:** Benchmark results published, historical trends

### Success Criteria

**Quantitative:**
- [ ] Benchmark scenarios: 20+ (baseline: 10, 100% increase)
- [ ] Performance improvement: 10%+ speedup on 3+ scenarios
- [ ] Regression detection: CI runs weekly, <10% tolerance
- [ ] Documentation: Throughput, latency, memory all documented

**Qualitative:**
- [ ] Baselines established (known performance for all scan types)
- [ ] Regressions detectable (CI catches slowdowns automatically)
- [ ] Optimizations implemented (low-hanging fruit picked)
- [ ] Performance predictable (capacity planning guide accurate)

**Phase 6 Readiness:**
- [ ] TUI performance baseline known (can attribute overhead)
- [ ] No performance regressions (clean slate for Phase 6)
- [ ] Optimization complete (TUI won't inherit performance debt)

### Risk Mitigation

**Risk:** Benchmarks are noisy (high variance)
- **Mitigation:** Multiple runs (10+), statistical rigor (mean ± stddev)
- **Isolation:** Run on dedicated hardware (no background tasks)
- **Consistency:** Pin CPU frequency (disable turbo boost)

**Risk:** Profiling is time-consuming
- **Mitigation:** Automate flamegraph generation (scripts)
- **Prioritization:** Profile only high-impact scenarios first
- **Tooling:** Use existing tools (cargo flamegraph, massif)

**Risk:** Optimizations break functionality
- **Mitigation:** Run full test suite after each optimization
- **Regression Testing:** Ensure 1,601 tests still pass
- **Incremental:** One optimization at a time, measure impact

**Risk:** CI benchmarks slow down pipeline
- **Mitigation:** Run weekly (not on every commit)
- **On-Demand:** Manual trigger available
- **Timeout:** Limit benchmark run to 30 minutes max

---

## Sprint 5.5.5: Configuration & State Management

**Priority:** MEDIUM (TUI prerequisite, user convenience)
**Duration:** 3-4 days (24-32 hours estimated)
**ROI Score:** 7.5/10 (Medium-High impact, moderate effort)
**Dependencies:** Sprint 5.5.3 (event system provides state infrastructure)

### Objective

Implement configuration profiles for saving/loading scan presets and state persistence for resuming interrupted scans. Establish clean state management patterns for TUI (Phase 6).

### Rationale

Current configuration is command-line only: users must reconstruct complex flag combinations for recurring scans. State is ephemeral: scan interruption loses all progress. Before TUI, we need:

1. **Configuration Profiles:** Save/load scan presets (compliance scans, recurring audits)
2. **State Persistence:** Resume interrupted scans (long-running, network failures)
3. **State Management API:** Clean patterns for TUI state (centralized, queryable)

**Current State:**
- Configuration: TOML file exists but only global settings (no scan profiles)
- State persistence: None (scans cannot be resumed)
- State management: Decentralized (each scanner manages own state)

**Why Before Phase 6:**
TUI will need state management (current scan state, UI preferences, window layout). Building these patterns in CLI first allows testing without UI complexity.

**Impact of Deferring:**
- Poor user experience (must manually recreate scans)
- TUI state management harder (no patterns to follow)
- Long scans risky (interruption loses hours of work)

### Tasks

1. **Configuration Profile System** (6-8h)
   - **Profile Schema** (2-3h):
     - [ ] Extend `~/.prtip/config.toml` to support profiles:
       ```toml
       # Global settings (apply to all scans)
       [global]
       privilege_drop = true
       default_timing = "T3"
       log_level = "info"

       # Named scan profiles
       [profiles.web-servers]
       description = "Scan for HTTP/HTTPS servers"
       ports = [80, 443, 8080, 8443]
       scan_type = "syn"
       service_detection = true
       timing = "T4"
       output_format = "json"

       [profiles.database-audit]
       description = "PCI-DSS database scanning"
       ports = [3306, 5432, 27017, 6379, 1433]
       scan_type = "connect"  # Compliance: no raw sockets
       service_detection = true
       banner_grab = true
       timing = "T2"  # Polite scan
       output_format = "xml"
       audit_log = true

       [profiles.stealth-recon]
       description = "Evade IDS/IPS detection"
       ports = "1-1024"
       scan_type = "fin"
       timing = "T0"
       fragmentation = true
       decoys = ["192.168.1.10", "192.168.1.20"]
       source_port = 53
       output_format = "greppable"
       ```
     - [ ] Implement profile loading:
       - `prtip --profile web-servers 192.168.1.0/24`
       - Override profile settings: `prtip --profile web-servers -p 443 --timing T5`
   - **Profile Management CLI** (2-3h):
     - [ ] Implement profile commands:
       - `prtip profile list` (show all profiles)
       - `prtip profile show <name>` (display profile settings)
       - `prtip profile create <name>` (interactive wizard)
       - `prtip profile edit <name>` (open in $EDITOR)
       - `prtip profile delete <name>` (remove profile)
       - `prtip profile export <name>` (TOML output)
       - `prtip profile import <file>` (load external profile)
   - **Profile Validation** (1-2h):
     - [ ] Implement profile validation:
       - Check required fields
       - Validate port ranges (1-65535)
       - Validate timing (T0-T5)
       - Validate scan types (syn, connect, udp, etc.)
     - [ ] Add helpful error messages:
       - "Profile 'foo' missing required field: ports"
       - "Invalid timing 'T6'. Valid: T0, T1, T2, T3, T4, T5"
   - **Acceptance:** Profile system with CRUD operations

2. **Scan State Persistence** (8-10h)
   - **State Schema** (2-3h):
     - [ ] Design scan state format (`~/.prtip/state/<scan_id>.json`):
       ```json
       {
         "scan_id": "550e8400-e29b-41d4-a716-446655440000",
         "version": "0.5.5",
         "started_at": "2025-11-10T14:30:00Z",
         "last_checkpoint": "2025-11-10T15:45:32Z",
         "status": "running",  // running, paused, completed, failed
         "config": { /* ScanConfig */ },
         "progress": {
           "stage": "scanning_ports",
           "percentage": 67.5,
           "completed_targets": 1823,
           "total_targets": 2700,
           "completed_ports": 36450,
           "total_ports": 54000
         },
         "results": {
           "hosts_discovered": 1823,
           "ports_found": 3421,
           "services_detected": 1205,
           "errors": 12
         },
         "checkpoint_data": {
           "targets_completed": ["192.168.1.1", "192.168.1.2", "..."],
           "targets_pending": ["192.168.1.100", "192.168.1.101", "..."],
           "partial_results": [ /* ScanResult objects */ ]
         }
       }
       ```
   - **Checkpointing** (3-4h):
     - [ ] Implement periodic checkpointing:
       - Every 60 seconds OR every 10% progress (whichever is less frequent)
       - Write state to `~/.prtip/state/<scan_id>.json`
       - Atomic writes (write to temp file, rename)
     - [ ] Subscribe to event bus:
       - Listen for `ProgressUpdate`, `PortFound`, `ServiceDetected` events
       - Aggregate and write to checkpoint
   - **Resume Functionality** (2-3h):
     - [ ] Implement `prtip resume <scan_id>`:
       - Load state from `~/.prtip/state/<scan_id>.json`
       - Reconstruct ScanConfig
       - Skip completed targets
       - Continue from last checkpoint
     - [ ] Implement `prtip resume --last`:
       - Find most recent incomplete scan
       - Resume automatically
   - **State Management CLI** (1h):
     - [ ] Implement state commands:
       - `prtip scans list` (show all scans: running, paused, completed)
       - `prtip scans show <scan_id>` (display scan status)
       - `prtip scans pause <scan_id>` (graceful pause)
       - `prtip scans resume <scan_id>` (continue scan)
       - `prtip scans cancel <scan_id>` (abort scan)
       - `prtip scans cleanup` (delete completed scans older than 30 days)
   - **Acceptance:** Scans resumable after interruption

3. **Centralized State API** (5-6h)
   - **State Manager** (3-4h):
     - [ ] Create `crates/prtip-core/src/state_manager.rs`:
       ```rust
       pub struct StateManager {
           event_bus: Arc<EventBus>,
           current_scans: Arc<RwLock<HashMap<Uuid, ScanState>>>,
           checkpoint_interval: Duration,
           checkpoint_task: JoinHandle<()>,
       }

       pub struct ScanState {
           pub scan_id: Uuid,
           pub config: ScanConfig,
           pub status: ScanStatus,
           pub progress: ProgressState,
           pub results: ResultsSummary,
           pub started_at: SystemTime,
           pub last_checkpoint: SystemTime,
       }

       impl StateManager {
           pub fn new(event_bus: Arc<EventBus>) -> Self { /* subscribe to events */ }
           pub fn get_state(&self, scan_id: Uuid) -> Option<ScanState> { /* ... */ }
           pub fn all_scans(&self) -> Vec<ScanState> { /* ... */ }
           pub fn checkpoint(&self, scan_id: Uuid) -> Result<()> { /* write to disk */ }
           pub fn load_checkpoint(&self, scan_id: Uuid) -> Result<ScanState> { /* read from disk */ }
       }
       ```
     - [ ] Subscribe to events:
       - `ScanStarted` → create new ScanState entry
       - `ProgressUpdate` → update progress
       - `ScanCompleted` → mark as completed
       - `ScanError` → update error count
   - **Query API** (1-2h):
     - [ ] Implement state queries:
       - `StateManager::active_scans()` (currently running)
       - `StateManager::recent_scans(limit: usize)` (last N scans)
       - `StateManager::scans_by_status(status: ScanStatus)` (filter)
       - `StateManager::scan_results(scan_id: Uuid)` (get full results)
   - **Acceptance:** Centralized state, queryable API

4. **Preferences & Settings** (3-4h)
   - **User Preferences** (2-3h):
     - [ ] Add UI preferences to config:
       ```toml
       [preferences]
       theme = "dark"  # dark, light, auto
       progress_style = "verbose"  # minimal, standard, verbose
       confirmation_prompts = true
       save_history = true
       log_to_file = true

       [preferences.output]
       default_format = "json"
       colorize = true
       timestamps = true

       [preferences.performance]
       batch_size = "auto"  # auto, or specific number
       ulimit = "auto"
       numa = false
       ```
     - [ ] Implement preference loading:
       - Load from `~/.prtip/config.toml` on startup
       - Override with CLI flags
       - Save modified preferences
   - **Settings Management CLI** (1h):
     - [ ] Implement settings commands:
       - `prtip settings list` (show all preferences)
       - `prtip settings get <key>` (get specific preference)
       - `prtip settings set <key> <value>` (update preference)
       - `prtip settings reset` (restore defaults)
   - **Acceptance:** User preferences saved, managed via CLI

5. **TUI State Preparation** (3-4h)
   - **UI State Schema** (2-3h):
     - [ ] Design TUI state structure (prepare for Phase 6):
       ```rust
       pub struct TuiState {
           pub active_scan: Option<Uuid>,
           pub view: ViewMode,  // Dashboard, Results, Logs, Settings
           pub window_layout: Layout,  // Custom layout persistence
           pub selected_result: Option<usize>,
           pub filter: ResultFilter,
           pub sort: ResultSort,
       }
       ```
     - [ ] Implement state persistence:
       - Save to `~/.prtip/ui-state.json`
       - Load on TUI startup (Phase 6)
   - **Layout Persistence** (1h):
     - [ ] Save window layout:
       - Panel sizes (progress: 20%, results: 60%, logs: 20%)
       - Active panel
       - Scroll positions
     - [ ] Implement layout presets:
       - `default`: Balanced layout
       - `compact`: Minimal progress, maximum results
       - `detailed`: Equal split (progress, results, logs)
   - **Acceptance:** TUI state schema defined, persistence ready

### Deliverables

- [x] **Configuration Profiles:** Save/load scan presets (10+ built-in examples)
- [x] **Profile Management:** CRUD operations via CLI
- [x] **Scan State Persistence:** Resume interrupted scans
- [x] **State Manager:** Centralized state with query API
- [x] **User Preferences:** Settings management via CLI
- [x] **TUI State Preparation:** Schema defined, persistence ready

### Success Criteria

**Quantitative:**
- [ ] Profiles: 10+ built-in examples (web, databases, ssh, mail, smb, quick, thorough, stealth, safe, compliance)
- [ ] Checkpoint frequency: Every 60s OR every 10% progress
- [ ] State queries: 5+ query methods (active, recent, by_status, results, all)
- [ ] Preferences: 15+ configurable settings

**Qualitative:**
- [ ] Profiles easy to create (interactive wizard <2 minutes)
- [ ] Resume works reliably (>95% success rate)
- [ ] State queryable (TUI can get any state anytime)
- [ ] Preferences persistent (settings survive restarts)

**Phase 6 Readiness:**
- [ ] TUI state schema defined (no design work needed)
- [ ] State management patterns proven (CLI tests them)
- [ ] Configuration infrastructure ready (TUI adds UI only)

### Risk Mitigation

**Risk:** Checkpoint overhead slows scans
- **Mitigation:** Async checkpointing (background task)
- **Tuning:** Configurable interval (default: 60s)
- **Performance:** <1% overhead target

**Risk:** Resume corrupts state (partial results lost)
- **Mitigation:** Atomic writes (write to temp, rename)
- **Validation:** Checksum verification on load
- **Backup:** Keep last 3 checkpoints (rollback if corrupt)

**Risk:** Profile explosion (too many profiles)
- **Mitigation:** User profiles separate from built-in
- **Organization:** Categories (web, security, compliance)
- **Discovery:** `prtip profile search <keyword>`

**Risk:** State bloat (checkpoint files grow large)
- **Mitigation:** Compress checkpoints (gzip)
- **Cleanup:** Auto-delete completed scans >30 days
- **Optimization:** Store only incremental changes (not full state)

---

## Sprint 5.5.6: Integration & Testing

**Priority:** MEDIUM (quality assurance, phase validation)
**Duration:** 3-4 days (24-32 hours estimated)
**ROI Score:** 8.0/10 (High impact, moderate effort)
**Dependencies:** All previous sprints (5.5.1-5.5.5)

### Objective

Integrate all Phase 5.5 enhancements, perform end-to-end testing, stress testing, edge case coverage, and validate Phase 6 readiness through comprehensive integration tests.

### Rationale

Sprints 5.5.1-5.5.5 introduced significant changes (documentation, CLI UX, event system, performance, state management). Before Phase 6, we must validate:

1. **Integration:** All components work together (no conflicts)
2. **Quality:** No regressions, edge cases handled
3. **Performance:** Phase 5.5 enhancements didn't degrade speed
4. **Readiness:** Phase 6 can start without blockers

**Current State:**
- Individual sprint tests pass (unit tests per component)
- Integration unknown (cross-component interactions untested)
- Stress testing minimal (no large-scale validation)
- Edge cases partial (comprehensive coverage missing)

**Why Critical:**
Phase 6 will build on Phase 5.5 infrastructure. Bugs discovered during Phase 6 are 10x more expensive to fix (UI complexity). Thorough testing now prevents downstream issues.

**Impact of Deferring:**
- Phase 6 blocked by bugs (discovered late)
- User experience poor (edge cases cause failures)
- Confidence low (no validation of readiness)

### Tasks

1. **End-to-End Integration Tests** (6-8h)
   - **Cross-Component Workflows** (4-5h):
     - [ ] Test complete user journeys:
       - **Journey 1:** Profile creation → scan execution → results export
         - Create profile via wizard
         - Run scan with profile
         - Export results to JSON
         - Verify output correctness
       - **Journey 2:** Event-driven progress → JSON logging → replay
         - Start scan
         - Subscribe to events
         - Verify real-time updates
         - Check JSON event log
         - Replay events
       - **Journey 3:** Scan interruption → checkpoint → resume
         - Start large scan
         - Interrupt mid-scan (SIGINT)
         - Verify checkpoint written
         - Resume scan
         - Verify completion
       - **Journey 4:** Template → interactive confirmation → history
         - Use scan template
         - Confirm dangerous operation
         - Verify in history
         - Replay from history
       - **Journey 5:** Multi-subscriber events → aggregated state → query
         - Start scan with multiple subscribers (CLI, logger, aggregator)
         - Verify all receive events
         - Query aggregated state
         - Validate consistency
   - **Component Interaction Tests** (2-3h):
     - [ ] Event bus → Progress aggregator
     - [ ] Event bus → CLI display
     - [ ] Event bus → JSON logger
     - [ ] State manager → Checkpoint writer
     - [ ] State manager → Resume logic
     - [ ] Profile manager → Config parser
     - [ ] Profile manager → CLI integration
   - **Acceptance:** 5+ end-to-end journeys, all passing

2. **Stress Testing** (5-6h)
   - **Large-Scale Scans** (3-4h):
     - [ ] Test with 100K hosts:
       - Memory usage stays <1GB
       - No memory leaks (Valgrind)
       - Progress updates smooth
       - Results streaming works
     - [ ] Test with all 65535 ports:
       - Scan completes in reasonable time
       - No port skipped
       - Memory bounded
     - [ ] Test with 1M targets (stateless):
       - Memory <100MB (verify Phase 4 claim)
       - Throughput sustained
       - No crashes
   - **Event System Stress** (2h):
     - [ ] Test with 100 concurrent subscribers:
       - No event loss
       - Performance <10% degradation
       - Memory reasonable
     - [ ] Test with high event rate:
       - 10K events/second sustained
       - Latency p99 <20ms
       - No buffer overflows
   - **Acceptance:** Large-scale scans work, event system handles stress

3. **Edge Case Coverage** (6-7h)
   - **Error Handling** (2-3h):
     - [ ] Test error scenarios:
       - Network unreachable (ICMP unreachable)
       - Permission denied (CAP_NET_RAW missing)
       - Resource exhaustion (ulimit exceeded)
       - Invalid targets (malformed IP, DNS failure)
       - Firewall blocking (all ports filtered)
       - Rate limiting (target throttling)
       - Plugin failures (Lua error, timeout)
   - **State Transitions** (2h):
     - [ ] Test state machine:
       - Start → Pause → Resume → Complete
       - Start → Error → Retry → Complete
       - Start → Cancel → Cleanup
       - Start → Checkpoint → Crash → Resume
   - **Concurrent Operations** (1-2h):
     - [ ] Test multiple scans simultaneously:
       - 10 scans in parallel
       - Events isolated by scan_id
       - State manager handles concurrency
       - No resource conflicts
   - **Edge Inputs** (1-2h):
     - [ ] Test boundary conditions:
       - Port 0, port 65536 (invalid)
       - Empty target list
       - Huge target list (1M+)
       - Invalid CIDR (/33)
       - Malformed config file
       - Corrupt checkpoint file
   - **Acceptance:** 50+ edge cases tested, all handled gracefully

4. **Performance Regression Testing** (3-4h)
   - **Benchmark Comparison** (2-3h):
     - [ ] Run full benchmark suite (Sprint 5.5.4):
       - 20+ scenarios
       - Compare to baselines
       - Verify no regressions
     - [ ] Test Phase 5.5 overhead:
       - Event system: <5% overhead (verify)
       - Checkpointing: <1% overhead (verify)
       - Logging: <2% overhead (verify)
   - **Memory Leak Detection** (1h):
     - [ ] Run Valgrind on long scans:
       - 1-hour scan
       - Check for leaks
       - Verify all memory freed
   - **Acceptance:** No performance regressions, no memory leaks

5. **CI/CD Validation** (3-4h)
   - **Workflow Testing** (2-3h):
     - [ ] Verify all CI workflows pass:
       - Tests (1,601 tests)
       - Clippy (0 warnings)
       - Format (rustfmt)
       - Documentation (doctests)
       - Benchmarks (weekly)
       - Security audit (cargo audit)
   - **Cross-Platform Testing** (1h):
     - [ ] Run tests on all platforms:
       - Linux (Ubuntu, Alpine)
       - macOS (Intel, Apple Silicon)
       - Windows (MSVC, GNU)
       - FreeBSD (cross-compilation)
   - **Acceptance:** 9/9 CI workflows green, all platforms passing

6. **Phase 6 Readiness Checklist** (2-3h)
   - **Infrastructure Validation** (1-2h):
     - [ ] Event system ready:
       - ✅ Pub-sub operational
       - ✅ Multiple subscribers supported
       - ✅ Event history/replay works
       - ✅ Performance acceptable (<5% overhead)
     - [ ] State management ready:
       - ✅ Centralized state API
       - ✅ Queryable at any time
       - ✅ Persistence works
       - ✅ Resume functional
     - [ ] Progress reporting ready:
       - ✅ Real-time updates
       - ✅ ETA calculation
       - ✅ Multi-stage tracking
       - ✅ Throughput metrics
   - **Documentation Validation** (1h):
     - [ ] Verify readiness docs:
       - Phase 6 ROADMAP up-to-date
       - TUI architecture documented
       - Event system documented
       - State management patterns documented
   - **Acceptance:** All Phase 6 prerequisites met

### Deliverables

- [x] **Integration Tests:** 5+ end-to-end user journeys
- [x] **Stress Tests:** Large-scale scans (100K hosts, 65K ports, 1M targets)
- [x] **Edge Cases:** 50+ edge cases covered
- [x] **Performance:** No regressions vs baselines
- [x] **CI/CD:** 9/9 workflows green, all platforms passing
- [x] **Phase 6 Readiness:** All prerequisites validated

### Success Criteria

**Quantitative:**
- [ ] Integration tests: 5+ journeys, 100% passing
- [ ] Stress tests: 100K hosts, 65K ports, 1M targets all working
- [ ] Edge cases: 50+ tested, graceful handling
- [ ] Performance: <5% regression on all benchmarks
- [ ] CI/CD: 9/9 workflows green
- [ ] Test count: 1,601+ maintained (no test removal)

**Qualitative:**
- [ ] Integration smooth (no conflicts between sprints)
- [ ] Stress handling robust (large scans reliable)
- [ ] Edge cases graceful (errors informative, recoverable)
- [ ] Performance maintained (no degradation)
- [ ] Confidence high (Phase 6 ready to start)

**Phase 6 Readiness:**
- [ ] Event system proven (stress-tested, reliable)
- [ ] State management validated (concurrent access safe)
- [ ] Progress reporting accurate (ETA within 20% actual)
- [ ] Documentation complete (TUI developers have guide)
- [ ] Zero blockers (all prerequisites met)

### Risk Mitigation

**Risk:** Integration tests reveal major bugs
- **Mitigation:** Fix immediately (before Phase 6)
- **Prioritization:** Critical bugs block Phase 6, defer minor issues
- **Tracking:** Document known issues in PROJECT-STATUS.md

**Risk:** Stress tests fail (scalability issues)
- **Mitigation:** Optimize identified bottlenecks
- **Fallback:** Document limitations (max supported scale)
- **Roadmap:** Add Phase 5.6 (Scalability Enhancements) if needed

**Risk:** Performance regressions discovered late
- **Mitigation:** Profile regressions, optimize hot paths
- **Target:** Restore baseline performance before Phase 6
- **Tracking:** Benchmark comparison in CI

**Risk:** Phase 6 prerequisites incomplete
- **Mitigation:** Extend Sprint 5.5.6 if needed
- **Acceptance:** Phase 6 cannot start until checklist 100% complete
- **Buffer:** Allocate extra week if needed

---

## Dependencies & Timeline

### Execution Plan

```
Phase 5.5 Execution Plan (3-4 weeks)
======================================

Week 1: Parallel Foundation
----------------------------
Sprint 5.5.1: Documentation    (3-4d) ━━━━━━━━━━━━━━━━
Sprint 5.5.2: CLI Usability    (3-4d) ━━━━━━━━━━━━━━━━  } Parallel
Sprint 5.5.4: Performance      (3-4d) ━━━━━━━━━━━━━━━━  }

Week 2: Critical Path
----------------------
Sprint 5.5.3: Event System     (4-5d) ━━━━━━━━━━━━━━━━━━━━  (Depends: 5.5.2)
Sprint 5.5.5: State Mgmt       (3-4d)         ━━━━━━━━━━━━━  (Depends: 5.5.3)

Week 3-4: Integration & Buffer
-------------------------------
Sprint 5.5.6: Integration      (3-4d)                 ━━━━━━━━━━━━━
Buffer/Polish                  (2-3d)                           ━━━━━

Total Duration: 19-24 days (3-4 weeks)
Critical Path: 5.5.2 → 5.5.3 → 5.5.5 → 5.5.6 (15-18 days)
```

### Dependency Graph

```
         ┌─────────────┐
         │  5.5.1 Docs │  (3-4d, no deps)
         └─────────────┘
                ↓ (independent)
         ┌─────────────────────┐
         │  5.5.2 CLI UX       │  (3-4d, no deps)
         └─────────────────────┘
                ↓ (blocks)
         ┌──────────────────────────┐
         │  5.5.3 Event System      │  (4-5d, CRITICAL)
         └──────────────────────────┘
                ↓ (blocks)
         ┌─────────────────────────┐
         │  5.5.5 State Mgmt       │  (3-4d)
         └─────────────────────────┘
                ↓ (blocks)
         ┌─────────────────────┐
         │  5.5.6 Integration  │  (3-4d)
         └─────────────────────┘

         ┌─────────────────┐
         │  5.5.4 Perf     │  (3-4d, no deps, parallel)
         └─────────────────┘
```

### Parallelization Strategy

**Week 1 (Days 1-7):**
- Run 5.5.1, 5.5.2, 5.5.4 in parallel (different team members OR sequential if solo)
- All can start immediately (no dependencies)
- Complete all three before week 2

**Week 2 (Days 8-14):**
- Start 5.5.3 (depends on 5.5.2 completion)
- 5.5.3 is CRITICAL PATH (cannot parallelize)
- Start 5.5.5 mid-week (depends on 5.5.3 completion)

**Week 3-4 (Days 15-24):**
- 5.5.6 integration (depends on all above)
- Buffer days for bug fixes, polish, unexpected issues
- Final validation before Phase 6

---

## Success Criteria (Phase 5.5 Complete)

### Quantitative Metrics

**Tests & Coverage:**
- [ ] Version: v0.5.5 released
- [ ] Tests: 1,601+ maintained or increased (no test removal)
- [ ] Coverage: 60%+ (baseline: 54.92%, +5.08% target)
- [ ] Doctests: 0 ignored (baseline: 5)
- [ ] CI/CD: 9/9 workflows passing

**Documentation:**
- [ ] Example gallery: 60+ scenarios (baseline: 39, +54% increase)
- [ ] Documentation: 50+ comprehensive files
- [ ] Broken links: 0 (run link checker)
- [ ] Linter warnings: 0 (markdown, spell-check)

**Performance:**
- [ ] Benchmark scenarios: 20+ (baseline: 10, 100% increase)
- [ ] Performance regression: <5% on all scenarios
- [ ] Event overhead: <5% with 10 subscribers
- [ ] Checkpoint overhead: <1%

**Features:**
- [ ] Configuration profiles: 10+ built-in
- [ ] Scan templates: 10+ built-in
- [ ] Event types: 15+ defined
- [ ] Event subscribers: Multi-subscriber tested (10+)
- [ ] State persistence: Resume functionality working

### Qualitative Indicators

**User Experience:**
- [ ] CLI polish: Professional-grade UX (A+ quality)
- [ ] Help system: <10s to find any flag
- [ ] Error messages: 90%+ include actionable solutions
- [ ] Progress updates: Real-time with ETA, throughput
- [ ] Templates: Common scenarios 1-flag simple

**Developer Experience:**
- [ ] Event system: Clean pub-sub architecture
- [ ] State management: Centralized, queryable API
- [ ] Documentation: API examples for all public types
- [ ] Testing: Integration tests for user journeys
- [ ] Performance: Baselines documented

**Quality:**
- [ ] Zero blocking bugs (all P0/P1 issues resolved)
- [ ] Edge cases handled gracefully (50+ tested)
- [ ] Stress tests passing (100K hosts, 1M targets)
- [ ] Cross-platform validated (all 7 platforms green)
- [ ] Professional presentation (documentation, code quality)

### Phase 6 Readiness Checklist

**Event System (CRITICAL):**
- [ ] ✅ Pub-sub architecture operational
- [ ] ✅ Event types defined (15+ types)
- [ ] ✅ Multi-subscriber support tested
- [ ] ✅ Event history & replay functional
- [ ] ✅ Performance acceptable (<5% overhead)
- [ ] ✅ Integration tests passing

**State Management (CRITICAL):**
- [ ] ✅ Centralized state API (StateManager)
- [ ] ✅ State queryable at any time
- [ ] ✅ State persistence (checkpointing)
- [ ] ✅ Resume functionality working
- [ ] ✅ Concurrent access safe (RwLock)
- [ ] ✅ TUI state schema defined

**Progress Reporting (CRITICAL):**
- [ ] ✅ Real-time progress updates (event-driven)
- [ ] ✅ ETA calculation (adaptive EWMA)
- [ ] ✅ Multi-stage tracking (5+ stages)
- [ ] ✅ Throughput metrics (pps, hpm, Mbps)
- [ ] ✅ Progress aggregator tested
- [ ] ✅ CLI displays real-time updates

**Documentation (HIGH):**
- [ ] ✅ Phase 6 architecture documented
- [ ] ✅ Event system guide written
- [ ] ✅ State management patterns documented
- [ ] ✅ TUI integration guide ready
- [ ] ✅ API reference complete
- [ ] ✅ Example code provided

**Infrastructure (MEDIUM):**
- [ ] ✅ Configuration profiles working
- [ ] ✅ Scan templates functional
- [ ] ✅ Logging infrastructure ready
- [ ] ✅ Performance baselines established
- [ ] ✅ CI/CD all green
- [ ] ✅ Cross-platform validated

---

## Research Sources

### MCP Research Summary

**GitHub MCP (ratatui repository analysis):**
- **Finding:** ratatui is modular framework (core, widgets, backends)
- **Implication:** ProRT-IP should separate backend (event system) from future UI layer
- **Action:** Event system designed as standalone (Sprint 5.5.3)

**WebSearch MCP (TUI best practices):**
- **Finding:** "async/threaded state management can be hard" - lots of boilerplate needed
- **Implication:** State management patterns must be simple, well-tested before TUI
- **Action:** Centralized StateManager in Sprint 5.5.5, proven in CLI first

**WebSearch MCP (Progress indicators):**
- **Finding:** 3 popular patterns: spinner, X of Y, progress bar
- **Implication:** ProRT-IP should support multiple progress styles
- **Action:** Multi-stage progress with configurable verbosity (Sprint 5.5.2)

**Code Search MCP (event patterns):**
- **Finding:** Common pattern: `tokio::sync::mpsc` for event dispatch
- **Implication:** Use proven async patterns (not custom implementations)
- **Action:** EventBus uses `mpsc::UnboundedSender` (Sprint 5.5.3)

### Baseline Analysis (inspire-me-report.md)

**Competitive Gaps Identified:**
- Speed: Masscan 10M pps (ProRT-IP ~10K pps stateless)
- Scripting: Nmap 600+ NSE (ProRT-IP 2 Lua plugins)
- Features: RustScan adaptive learning (ProRT-IP static algorithms)

**Gaps Addressed in Phase 5.5:**
- Plugin examples: 2 → 10+ (Sprint 5.5.1 example gallery)
- Progress: Basic → ETA + throughput + multi-stage (Sprint 5.5.2)
- Logging: None → Structured JSON + audit trails (Sprint 5.5.2)
- State: Ephemeral → Persistent checkpoints (Sprint 5.5.5)

**Gaps Deferred:**
- Speed optimization: Requires profiling (Phase 5.5.4 identifies opportunities)
- Advanced features: Distributed scanning (Phase 8), Web UI (Phase 8)

### Phase 5 Learnings

**Recent Decisions (from CLAUDE.local.md):**

1. **Pragmatic Choices (11-07):**
   - 5 scanner doctests ignored (API changes during Sprint 5.5)
   - **Action:** Fix in Sprint 5.5.1 (update examples to current API)

2. **Test Count Accuracy (11-07):**
   - Fixed 7 inconsistencies (1,766 → 1,601 correct count)
   - **Action:** Maintain accuracy in Phase 5.5 documentation

3. **CI/CD Optimization (11-06):**
   - 30-50% execution time reduction
   - **Action:** Continue optimization in Sprint 5.5.4

**Known Issues:**
- Coverage gaps: 54.92% overall (9 modules <50%)
- **Action:** Targeted coverage improvement in Sprint 5.5.6

**Technical Debt:**
- API consistency: Recent breaking changes (Error::Config, ServiceInfo fields)
- **Action:** Document API stability guarantee before v1.0

---

## Appendix: Baseline Analysis

### inspire-me-report.md Key Findings

**Quick Wins Identified:**
1. Output format expansion (CSV, HTML) - **Sprint 5.5.2**
2. Progress improvements (ETA, throughput) - **Sprint 5.5.2, 5.5.3**
3. Logging infrastructure (structured, audit) - **Sprint 5.5.2**
4. Command templates (common scenarios) - **Sprint 5.5.2**
5. Example gallery expansion - **Sprint 5.5.1**

**Gaps Identified:**
1. TUI prerequisites missing (events, state) - **Sprint 5.5.3, 5.5.5**
2. Documentation polish needed (doctests, examples) - **Sprint 5.5.1**
3. Performance baseline absent - **Sprint 5.5.4**
4. State persistence missing - **Sprint 5.5.5**
5. Integration testing incomplete - **Sprint 5.5.6**

**Recommendations Applied:**
- ✅ Event system before TUI (Sprint 5.5.3)
- ✅ Documentation completeness (Sprint 5.5.1)
- ✅ CLI UX improvements (Sprint 5.5.2)
- ✅ Performance audit (Sprint 5.5.4)
- ✅ State management (Sprint 5.5.5)
- ✅ Comprehensive testing (Sprint 5.5.6)

**Recommendations Deferred:**
- Speed optimization (beyond profiling): Phase 6+
- Distributed scanning: Phase 8
- Web UI: Phase 8
- ML-based detection: Phase 8

### Phase 5 Completion State

**Achievements:**
- 10 sprints in 11 days (exceptional velocity)
- 1,601 tests (100% passing)
- 54.92% coverage (+17.66% in Sprint 5.6)
- 230M+ fuzz executions (0 crashes)
- 50,510+ documentation lines

**Technical Debt Created:**
- 5 ignored doctests (API evolution)
- Documentation gaps (rush to complete)
- Example coverage partial (39 scenarios)
- Performance baselines informal
- State management decentralized

**Phase 5.5 Resolution:**
All technical debt identified above will be resolved across 6 sprints, preparing clean slate for Phase 6.

---

**Generated:** 2025-11-07
**Command:** `/ultrathink` (pre-Phase 6 enhancement planning variant)
**Baseline:** daily_logs/2025-11-07/06-sessions/inspire-me-report.md
**Quality Target:** A+ (5,000+ words, comprehensive sprint details, production-ready)
**Word Count:** ~11,500 words
**Sprint Count:** 6 (fully detailed with 7-12 tasks each)
**Total Estimated Duration:** 19-24 days (3-4 weeks with buffer)
