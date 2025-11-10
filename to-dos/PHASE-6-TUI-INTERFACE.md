# Phase 6: TUI Interface & Performance Enhancements

**Phase:** 6 of 8  
**Timeline:** Q2 2026 (April-June, 12 weeks)  
**Version Target:** v0.6.0  
**Status:** PLANNED  
**Effort Estimate:** 115-154 hours  
**Sprint Count:** 8 sprints  
**Created:** 2025-11-09  
**Last Updated:** 2025-11-09

---

## Executive Summary

Phase 6 represents a major milestone in ProRT-IP's evolution, delivering a production-ready Terminal User Interface (TUI) alongside significant performance optimizations. This phase integrates:

- **Original Phase 6 Goals:** Real-time TUI visualization with interactive controls
- **Tier 1 Quick Wins:** 5 high-ROI performance improvements (32-47h, gains of 15-70%)
- **Tier 2 Medium Impact:** 3 advanced enhancements (18-24h, NUMA + CDN detection)

### Key Achievements Expected

**User Experience:**
- ✨ Modern TUI with ratatui framework (60 FPS, event-driven architecture)
- ✨ Real-time progress tracking with ETA, throughput metrics, live port discovery
- ✨ Interactive target selection with multi-select, preview, conflict detection
- ✨ Scan preset templates (6 built-in + custom templates)
- ✨ Customizable layouts, keyboard-only navigation, colorblind-friendly themes

**Performance Gains:**
- ⚡ 15-30% throughput increase via adaptive batch size tuning
- ⚡ 20-40% throughput gain via sendmmsg/recvmmsg syscall batching (Linux/BSD)
- ⚡ 20-50% memory reduction via memory-mapped result streaming
- ⚡ 30-70% scan time reduction via IP deduplication (overlapping CIDRs)
- ⚡ 10-25% throughput improvement via NUMA awareness (multi-socket systems)

**Technical Excellence:**
- 🎯 160-200 new tests (target: 2,260-2,300 total)
- 🎯 6,500-8,000 lines of new documentation
- 🎯 0 clippy warnings, 100% test pass rate
- 🎯 Maintains >60% code coverage
- 🎯 Cross-platform support (Linux, macOS, Windows)

### Strategic Value

**Competitive Position:** Phase 6 elevates ProRT-IP from #3-4 (tied with RustScan) to #2 overall in the network scanning landscape, trailing only Masscan in raw throughput while excelling in features, accuracy, and user experience.

**Community Impact:** The TUI interface makes ProRT-IP accessible to a broader audience, reducing the learning curve and enabling rapid reconnaissance workflows for security professionals.

**Foundation for Future Phases:** Event-driven architecture from Sprint 5.5.3 (EventBus, 40ns latency) provides a proven foundation for Phase 7 (Web Interface) and Phase 8 (Polish & Community).

---

## 1. Phase 6 Overview

### 1.1 Goals & Objectives

**Primary Goals (Original Phase 6):**
1. Real-time visualization of scan progress and results
2. Live progress tracking with ETA calculation
3. Interactive target selection and configuration
4. Dynamic result streaming as ports are discovered
5. Keyboard-based navigation (no mouse required)
6. Customizable dashboard layouts

**Secondary Goals (Tier 1 Quick Wins):**
1. Adaptive batch size tuning (QW-1) - highest ROI 5.33
2. sendmmsg/recvmmsg syscall batching (QW-2) - ROI 4.00
3. Memory-mapped result streaming (QW-3) - ROI 3.75
4. IP address deduplication (QW-4) - ROI 3.50
5. Scan preset templates (QW-5) - ROI 3.33

**Tertiary Goals (Tier 2 Medium Impact):**
1. Real-time progress indicators (MI-2) - mostly complete via Sprint 5.5.3 EventBus
2. NUMA awareness for multi-socket systems (MI-5) - ROI 2.00
3. CDN/WAF detection (MI-3) - ROI 2.33

**Explicitly Out of Scope:**
- NSE Script Compatibility (MI-4) - deferred to Phase 7
- Custom TCP/IP stack (FE-1) - not recommended (portability loss)
- eBPF/XDP packet processing (FE-4) - not recommended (Linux-only)
- Distributed coordination (FE-3) - premature optimization
- Web-based UI - deferred to Phase 7
- Vulnerability exploitation - outside project mission

### 1.2 Timeline & Phases

**Overall Timeline:** 12 weeks (Q2 2026: April 1 - June 20, 2026)

**Sprint Breakdown:**
- **Sprint 6.1:** TUI Framework & Event Integration (15-20h, Weeks 1-2)
- **Sprint 6.2:** Live Dashboard & Real-time Progress (12-18h, Weeks 2-3)
- **Sprint 6.3:** Network Layer Optimization (16-20h, Weeks 3-4)
- **Sprint 6.4:** Adaptive Performance Tuning (10-14h, Weeks 4-5)
- **Sprint 6.5:** Interactive Target Selection & Templates (14-18h, Weeks 5-7)
- **Sprint 6.6:** Advanced TUI Features (15-20h, Weeks 7-9)
- **Sprint 6.7:** NUMA Awareness & CDN Detection (18-24h, Weeks 9-11)
- **Sprint 6.8:** Documentation, Polish & Release (15-20h, Weeks 11-12)

**Total Effort:** 115-154 hours  
**Average per Sprint:** 14.4-19.3 hours  
**Weekly Pace:** 20h/week (sustainable with testing/debugging buffer)

**Strategic Milestones:**
- **Week 2 (mid-April):** Demo-ready TUI for RSA Conference
- **Week 6 (mid-May):** Performance Quick Wins validated
- **Week 10 (early June):** Feature freeze, enter testing phase
- **Week 12 (June 15-20):** v0.6.0 release

### 1.3 Integration Strategy

**Hybrid Approach (Option C):**

Phase 6 uses a **Foundation → Interleaved → Polish** strategy:

1. **Foundation Phase (Sprints 6.1-6.2):** Establish stable TUI core
   - Ratatui + Crossterm integration
   - EventBus subscription for real-time updates
   - Basic dashboard rendering
   - **Rationale:** Must have solid UI foundation before adding features

2. **Interleaved Phase (Sprints 6.3-6.6):** Parallel TUI features + Performance
   - Network optimizations (6.3-6.4) are independent of TUI work
   - Can run in parallel if team size permits
   - TUI features (6.5-6.6) build on proven foundation
   - **Rationale:** Maximize value delivery, enable parallelization

3. **Polish Phase (Sprints 6.7-6.8):** Advanced features + Documentation
   - NUMA and CDN detection for advanced users
   - Comprehensive documentation for all Phase 6 features
   - User acceptance testing and cross-platform validation
   - **Rationale:** Ensure production-ready quality

**Dependency Management:**
- **Critical Path:** 6.1 → 6.2 → 6.5 → 6.6 → 6.8 (TUI features, sequential)
- **Parallel Path:** 6.3 → 6.4 (Performance, can run alongside TUI work)
- **Late Integration:** 6.7 (Can slot in after 6.4, before 6.8)

**Flexibility:** Plan assumes single developer (sequential), but documents parallel opportunities for team expansion.

### 1.4 Foundation: Sprint 5.5.3 EventBus

**Critical Context:** Sprint 5.5.3 (completed Nov 2025) delivered a production-ready event system that eliminates 20-30 hours of work originally planned for Phase 6:

**What's Already Complete:**
- ✅ **EventBus:** Central pub/sub hub with 40ns publish latency, >10M events/sec throughput
- ✅ **18 Event Variants:** ScanStarted, ScanCompleted, PortFound, ServiceDetected, ProgressUpdate, ErrorOccurred, etc.
- ✅ **ProgressAggregator:** Real-time ETA, throughput (pps, ports/sec), completion percentage
- ✅ **EventLogger:** JSON Lines persistence with auto-rotation
- ✅ **Multi-Scanner Support:** Thread-safe coordination across SYN, UDP, Service Detection
- ✅ **Performance Validated:** -4.1% overhead (concurrent), 40ns latency, 340ns end-to-end

**Impact on Phase 6:**
- **MI-1 (Event-Driven TUI Prep):** ~0 hours remaining (already done!)
- **MI-2 (Real-Time Progress):** ~2-3 hours UI integration (vs 12-18h original estimate)
- **Sprint 6.2 Complexity:** Reduced from 18-24h to 12-18h (EventBus already proven)

**Integration Requirement:** Sprint 6.1 must implement EventSubscriber trait for TUI App, but the hard architectural work is complete.

---

## 2. Sprint Organization

### 2.1 Sprint Overview Matrix

| Sprint | Focus Area | Effort (h) | Dependencies | Parallel? | Tests | Docs (lines) |
|--------|-----------|-----------|--------------|-----------|-------|--------------|
| **6.1** | TUI Framework | 15-20 | None | No (foundation) | 25-30 | 500 |
| **6.2** | Live Dashboard | 12-18 | 6.1 + EventBus | No (builds on 6.1) | 20-25 | 500 |
| **6.3** | Network Optimization | 16-20 | None | **Yes** (independent) | 30-40 | 300 |
| **6.4** | Adaptive Tuning | 10-14 | Soft (6.3) | **Yes** (can use existing network) | 20-25 | 200 |
| **6.5** | Interactive Selection | 14-18 | 6.1, 6.2, 6.3 | No (needs TUI + dedup) | 25-30 | 400 |
| **6.6** | Advanced Features | 15-20 | 6.1, 6.2, 6.5 | No (late-stage UI) | 20-25 | 600 |
| **6.7** | NUMA + CDN | 18-24 | Optional (6.4) | **Yes** (mostly independent) | 25-30 | 1,000 |
| **6.8** | Documentation + Polish | 15-20 | ALL (6.1-6.7) | No (final phase) | 15-20 | 4,000 |
| **TOTAL** | **All Sprints** | **115-154** | - | - | **160-200** | **6,500-8,000** |

### 2.2 Critical Path Analysis

**Primary Path (Sequential, Must Complete in Order):**
```
6.1 (TUI Framework) 
  → 6.2 (Live Dashboard) 
    → 6.5 (Interactive Selection) 
      → 6.6 (Advanced Features) 
        → 6.8 (Documentation)
```
**Total Critical Path Time:** 71-96 hours (49-62% of total effort)

**Secondary Path (Can Run in Parallel with Primary):**
```
6.3 (Network Optimization) 
  → 6.4 (Adaptive Tuning) 
    → 6.7 (NUMA + CDN)
```
**Total Secondary Path Time:** 44-58 hours (30-38% of total effort)

**Parallel Execution Opportunity:**
If team has 2 developers:
- **Dev 1 (TUI Track):** 6.1 → 6.2 → 6.5 → 6.6 (~56-76h)
- **Dev 2 (Performance Track):** 6.3 → 6.4 → 6.7 (~44-58h)
- **Both:** 6.8 together (~15-20h)
- **Timeline Compression:** 12 weeks → 6-8 weeks

**Single Developer (Sequential):**
- **Recommended Order:** 6.1 → 6.2 → 6.3 → 6.4 → 6.5 → 6.6 → 6.7 → 6.8
- **Rationale:** Builds stable TUI foundation (6.1-6.2), then adds independent optimizations (6.3-6.4) before final UI features (6.5-6.6) and polish (6.7-6.8)

---

## 3. Detailed Sprint Breakdowns

### Sprint 6.1: TUI Framework & Event Integration

**Effort:** 15-20 hours  
**Timeline:** Weeks 1-2 (April 1-14, 2026)  
**Dependencies:** None (clean start)  
**Parallel Eligible:** No (foundation must complete first)

#### Objectives
1. Setup ratatui + crossterm + tui-input dependencies
2. Create prtip-tui crate in workspace (UI separation)
3. Implement EventSubscriber for EventBus integration
4. Build basic rendering loop (60 FPS target)
5. Establish keyboard navigation foundation

#### Task Areas

**Task Area 1: Framework Setup (4-5h)**
- Add ratatui 0.29+, crossterm 0.28+, tui-input dependencies to Cargo.toml
- Create prtip-tui crate with proper module structure (app.rs, ui.rs, events.rs, widgets/)
- Configure terminal initialization (raw mode, alternate screen, panic hook restoration)
- Implement graceful shutdown (restore terminal on Ctrl+C, panic)
- Cross-platform terminal setup (Windows/Linux/macOS quirks)

**Task Area 2: App State & Architecture (3-4h)**
- Define App struct using event-carried state transfer pattern
- Implement ScanState (shared with scanner via Arc<RwLock<>>)
- Create UIState (local to TUI: selected_pane, cursor_pos, scroll_offset)
- Design Component trait for modular widgets
- Setup async event loop with tokio::select! (keyboard, EventBus, timer)

**Task Area 3: EventBus Integration (4-5h)**
- Implement EventSubscriber trait for TUI App
- Subscribe to: ScanStarted, ScanCompleted, PortFound, ServiceDetected, ProgressUpdate, ErrorOccurred
- Create event handler dispatch (match on EventType, update state)
- Add event rate limiting (sample to 60 Hz, prevent UI overload at 10K events/sec)
- Test with mock EventBus (publish 10K events/sec, verify no dropped frames)

**Task Area 4: Basic Rendering (3-4h)**
- Implement 60 FPS render loop (16ms budget per frame)
- Create basic layout (header, status bar, main area, footer)
- Implement keyboard navigation (Tab/Shift+Tab for pane switching, vim-style hjkl)
- Add help text footer (show keybindings: q=quit, ?=help, Tab=next pane)
- Colorize output (status codes: green=open, red=closed, yellow=filtered, blue=TLS)

**Task Area 5: Testing & Documentation (1-2h)**
- Unit tests: App state transitions, event handling (15 tests)
- Integration test: Launch TUI, send events via mock EventBus, verify rendering (10 tests)
- Create docs/TUI-ARCHITECTURE.md (500 lines, component diagram, data flow)
- Document EventBus subscription pattern with code examples
- Rustdoc for all public TUI API (App, Component trait, UIState)

#### Success Criteria
- ✅ TUI launches successfully on Linux (primary development platform)
- ✅ Handles 1K events/sec without lag or dropped frames
- ✅ Keyboard navigation works (Tab, Shift+Tab, hjkl, q)
- ✅ 25-30 tests passing, 0 clippy warnings, cargo fmt clean
- ✅ TUI-ARCHITECTURE.md complete with component diagram

#### Deliverables
- prtip-tui crate (new workspace member)
- docs/TUI-ARCHITECTURE.md (500 lines)
- 25-30 tests in prtip-tui/tests/
- Sprint 6.1 completion report

---

### Sprint 6.2: Live Dashboard & Real-time Progress

**Effort:** 12-18 hours  
**Timeline:** Weeks 2-3 (April 14-28, 2026)  
**Dependencies:** Sprint 6.1 (TUI framework), Sprint 5.5.3 (EventBus - already complete)  
**Parallel Eligible:** No (builds on 6.1 foundation)

#### Objectives
1. Create multi-pane dashboard layout (header, progress, port table, log viewer)
2. Integrate ProgressAggregator from Sprint 5.5.3 (real-time ETA, throughput)
3. Implement live port discovery streaming (<100ms latency)
4. Add service detection indicators (color-coded by protocol)
5. Optimize rendering performance (<5% CPU overhead)

#### Task Areas

**Task Area 1: Multi-pane Layout (3-4h)**
- Create HeaderWidget (scan target, start time, scan type, elapsed time)
- Create ProgressWidget (connect to ProgressAggregator for ETA, completion %, throughput)
- Create PortTableWidget (discovered ports with service, status, banner)
- Create LogViewerWidget (recent events from EventBus history ring buffer)
- Implement layout switching (horizontal/vertical split, fullscreen modes, saved preferences)

**Task Area 2: Progress Integration (4-5h)**
- Subscribe to ProgressUpdate events from ProgressAggregator (already implemented in Sprint 5.5.3)
- Display real-time metrics: ETA, completion %, throughput (packets/sec, ports/sec, hosts/sec)
- Add progress bars (overall scan, per-host if multi-target scan)
- Implement sparkline graphs for throughput history (last 60 seconds, ASCII art)
- Test with various scan scenarios (fast SYN 10K pps, slow UDP 100 pps, multi-target 100 hosts)

**Task Area 3: Port Discovery Streaming (3-4h)**
- Subscribe to PortFound and ServiceDetected events
- Update PortTableWidget in real-time (<100ms latency from event to display)
- Implement table sorting (by port number, service name, timestamp, status)
- Add table scrolling (Page Up/Down, vim-style j/k, Home/End)
- Color-code by service type (HTTP=green, SSH=blue, TLS=cyan, unknown=gray)

**Task Area 4: Performance Optimization (1-2h)**
- Profile TUI rendering overhead with `perf` or `cargo flamegraph` (target: <5% CPU)
- Implement event aggregation (batch 100 PortFound events → single table update)
- Add FPS counter (debug mode, press 'f' to toggle, verify consistent 60 FPS)
- Test high-load scenario (10M port scan with 10K events/sec, validate no slowdown)
- Optimize widget tree (skip re-rendering unchanged widgets, dirty flag pattern)

**Task Area 5: Testing & Documentation (1-3h)**
- Unit tests: Widget rendering, state updates, event handling (20 tests)
- Integration tests: End-to-end event flow (EventBus → TUI → display) (10 tests)
- Performance benchmark: CPU overhead measurement (hyperfine TUI vs CLI mode)
- Update docs/TUI-ARCHITECTURE.md (+200 lines, widget interaction diagram)
- Create user-facing docs/36-TUI-INTERFACE-GUIDE.md (+300 lines, basic usage section)

#### Success Criteria
- ✅ Live dashboard displays real-time scan progress with ETA
- ✅ <100ms latency from EventBus event to screen update
- ✅ <5% CPU overhead from TUI rendering (vs CLI mode)
- ✅ 60 FPS maintained with 10K events/sec load
- ✅ 20-25 tests passing, performance benchmarks validated

#### Deliverables
- 4 dashboard widgets (Header, Progress, PortTable, LogViewer)
- docs/TUI-ARCHITECTURE.md update (+200 lines)
- docs/36-TUI-INTERFACE-GUIDE.md (new, 300 lines)
- 20-25 tests
- Sprint 6.2 completion report

---

### Sprint 6.3: Network Layer Optimization

**Effort:** 16-20 hours  
**Timeline:** Weeks 3-4 (April 28 - May 12, 2026)  
**Dependencies:** None (independent network layer work)  
**Parallel Eligible:** **YES** (can run in parallel with Sprints 6.1-6.2 if team allows)

#### Objectives
1. **QW-2:** Implement sendmmsg/recvmmsg batching (20-40% throughput gain)
2. **QW-4:** Add IP deduplication for overlapping CIDR ranges (30-70% scan reduction)
3. Platform-specific optimization (Linux/BSD syscall batching, Windows fallback)
4. Benchmark validation against baseline performance

#### Task Areas

**Task Area 1: sendmmsg/recvmmsg Batching (10-15h)**
- Research platform support (Linux 3.0+, BSD 7.0+, Windows N/A)
- Design BatchPacketSender/BatchPacketReceiver traits (abstracting sendmmsg/recvmmsg)
- Implement Linux sendmmsg batching (16-64 packets per syscall)
- Implement Linux recvmmsg batching (matching batch size for symmetry)
- Add BSD compatibility (similar APIs, test on FreeBSD/OpenBSD if available)
- Windows fallback (use existing send/recv, document performance difference)
- Tune batch size dynamically (start 16, increase to 64 based on success rate)
- Integration with existing network layer (prtip-network crate)

**Task Area 2: IP Deduplication (4-6h)**
- Design IntervalTree-based deduplication structure (CIDR overlap detection)
- Implement hash-based dedup for individual IPs (HashSet<IpAddr>)
- Add pre-scan validation (detect overlapping ranges, report to user)
- Error reporting for conflicts (e.g., "192.168.1.0/24 overlaps with 192.168.0.0/16")
- Integration with target parser (CLI → dedup → scanner)
- Handle DNS expansion with CNAMEs (dedup after resolution)

**Task Area 3: Platform-Specific Testing (2-3h)**
- Unit tests: Batch send/recv logic, edge cases (15 tests)
- Integration tests: End-to-end network I/O with batching (15 tests)
- Platform-specific tests (cfg(target_os = "linux") conditional compilation)
- Deduplication tests: CIDR overlap, DNS expansion, edge cases (10 tests)
- Cross-platform CI matrix (Linux, macOS, Windows, FreeBSD if possible)

**Task Area 4: Benchmark Validation (2-4h)**
- Baseline measurement (current throughput without batching)
- sendmmsg/recvmmsg benchmarks (Linux only, measure 20-40% gain)
- Deduplication benchmarks (large target lists with 50% overlap)
- Hyperfine integration (10 scenarios from Sprint 5.9 framework)
- Regression detection (ensure no slowdowns on non-batching platforms)

#### Success Criteria
- ✅ sendmmsg/recvmmsg delivers 20-40% throughput gain on Linux
- ✅ IP deduplication reduces scans 30-70% on overlapping ranges
- ✅ Graceful fallback on Windows/macOS (no crashes, performance documented)
- ✅ 30-40 tests passing across all platforms
- ✅ Benchmark suite shows gains (no regressions)

#### Deliverables
- BatchPacketSender/BatchPacketReceiver traits (prtip-network)
- IntervalTree deduplication (prtip-core)
- 30-40 tests (unit + integration + platform-specific)
- Performance benchmarks (before/after comparison)
- docs/26-RATE-LIMITING-GUIDE.md update (+300 lines, batching section)
- Sprint 6.3 completion report

---

### Sprint 6.4: Adaptive Performance Tuning

**Effort:** 10-14 hours  
**Timeline:** Weeks 4-5 (May 12-26, 2026)  
**Dependencies:** Soft dependency on Sprint 6.3 (sendmmsg batching enhances adaptive tuning)  
**Parallel Eligible:** **YES** (can use existing network code if 6.3 not ready)

#### Objectives
1. **QW-1:** Adaptive batch size tuning (15-30% throughput increase, highest ROI 5.33)
2. **QW-3 Foundation:** Memory-mapped streaming preparation (evaluate mmap vs io_uring)
3. Runtime profiling with rolling window (last 100 packets)
4. Integration with RateLimiterV3 from Sprint 5.X

#### Task Areas

**Task Area 1: Adaptive Batch Tuning (6-8h)**
- Implement runtime profiling (success rate + RTT measurement per batch)
- Design learning algorithm (RustScan-inspired):
  ```rust
  if success_rate > 0.95 && avg_rtt < threshold {
      batch_size = (batch_size * 1.2).min(max_batch); // Increase 20%
  } else if success_rate < 0.85 {
      batch_size = (batch_size * 0.8).max(min_batch); // Decrease 20%
  }
  ```
- Add rolling window stats (track last 100 packets, update every 500ms)
- Configure bounds (min_batch=16, max_batch=10000, start=1000)
- Integration with sendmmsg batching from Sprint 6.3 (or fallback to existing network)
- Integration with RateLimiterV3 (ensure no conflicts between batch tuning and rate limiting)

**Task Area 2: Memory-Mapped Streaming Prep (4-6h)**
- Evaluate mmap (memmap2 crate) vs io_uring (tokio-uring) for result streaming
- Benchmark current SQLite batch inserts (baseline memory usage, throughput)
- Design StreamingResultWriter interface (trait for pluggable backends):
  ```rust
  pub trait ResultWriter: Send + Sync {
      async fn write_result(&self, result: ScanResult) -> Result<()>;
      async fn flush(&self) -> Result<()>;
  }
  ```
- Prototype mmap-based writer (write results to memory-mapped file, periodic flush)
- Benchmark mmap vs SQLite (memory usage, write throughput, CPU overhead)
- Document trade-offs (mmap = low memory, io_uring = high throughput, SQLite = queryable)

**Task Area 3: Testing & Validation (2-3h)**
- Unit tests: Adaptive tuning algorithm, edge cases (20 tests)
- Integration tests: End-to-end with various network conditions (5 tests)
- Benchmark validation: 15-30% throughput increase vs static batching
- Memory profiling: Validate mmap reduces memory 20-50% vs SQLite
- Regression tests: Ensure no slowdowns when adaptive tuning disabled

#### Success Criteria
- ✅ Adaptive batching delivers 15-30% throughput increase over static batching
- ✅ Algorithm stable across various network conditions (LAN, WAN, internet)
- ✅ Memory-mapped streaming prototype validates 20-50% memory reduction
- ✅ 20-25 tests passing
- ✅ Integration with RateLimiterV3 (no conflicts, coordinated tuning)

#### Deliverables
- Adaptive batch tuning algorithm (prtip-network)
- StreamingResultWriter trait + mmap prototype (prtip-core)
- 20-25 tests
- Performance comparison report (adaptive vs static batching)
- docs/26-RATE-LIMITING-GUIDE.md update (+200 lines, adaptive tuning section)
- Sprint 6.4 completion report

---

### Sprint 6.5: Interactive Target Selection & Templates

**Effort:** 14-18 hours  
**Timeline:** Weeks 5-7 (May 26 - June 9, 2026)  
**Dependencies:** Sprints 6.1 (TUI framework), 6.2 (dashboard), 6.3 (IP dedup)  
**Parallel Eligible:** No (needs stable TUI foundation and deduplication)

#### Objectives
1. Interactive target selection in TUI (arrow keys, multi-select, vim-style)
2. CIDR range expansion preview (show IP count before scan)
3. **QW-5:** Scan preset templates (6 built-in + custom, ROI 3.33)
4. Integration with IP deduplication (show conflicts in UI)
5. Template editor in TUI (customize ports, timing, scan types)

#### Task Areas

**Task Area 1: Interactive Target Selection (5-7h)**
- Create TargetSelectorWidget (list of targets with multi-select)
- Implement arrow key navigation (Up/Down, Page Up/Down, Home/End)
- Add multi-select (Space to toggle, Ctrl+A for all, Ctrl+N for none)
- CIDR expansion preview (192.168.1.0/24 → "256 IPs" before scan)
- Integration with IP deduplication from Sprint 6.3 (highlight conflicts in red)
- Conflict resolution UI (ask user: keep first range, keep second, merge, skip)

**Task Area 2: Scan Preset Templates (QW-5, 4-6h)**
- Design ScanTemplate struct (name, ports, timing, scan_type, evasion_flags)
- Implement 6 built-in templates (Nmap -T0 to -T5 equivalents):
  1. **Paranoid (T0):** 1 port/5min, randomization, fragmentation, decoys
  2. **Sneaky (T1):** 1 port/15sec, stealth scans (FIN/NULL/Xmas)
  3. **Polite (T2):** 1 port/400ms, respectful rate limiting
  4. **Normal (T3):** Default ProRT-IP settings (current behavior)
  5. **Aggressive (T4):** 1000 pps, service detection, OS fingerprinting
  6. **Insane (T5):** Max speed, no rate limiting (with user confirmation!)
- Additional templates:
  7. **Web:** Ports 80,443,8080,8443 + TLS cert extraction
  8. **Database:** Ports 3306,5432,1433,27017 + service detection
- Template storage (JSON files in ~/.config/prtip/templates/)
- CLI integration (--template paranoid or -T0)

**Task Area 3: Template Editor in TUI (3-4h)**
- Create TemplateEditorWidget (form-based UI for template customization)
- Edit template fields (name, ports list, timing profile, scan type flags)
- Save/load custom templates (persist to ~/.config/prtip/templates/)
- Template preview (show what commands will run before starting scan)
- Validation (ensure ports 1-65535, timing valid, scan type compatible)

**Task Area 4: Testing & Documentation (2-3h)**
- Unit tests: Template serialization, validation, conflict detection (15 tests)
- Integration tests: TUI workflow (select targets → choose template → scan) (10 tests)
- Template file format tests (JSON schema validation)
- Update docs/32-USER-GUIDE.md (+400 lines, templates section)
- Create tutorial: "Using Scan Templates" (examples for each built-in template)

#### Success Criteria
- ✅ Interactive target selection supports 10K+ IPs without lag
- ✅ CIDR expansion preview accurate (IPv4 and IPv6)
- ✅ 8 built-in templates work correctly (6 timing + 2 use-case)
- ✅ Custom templates save/load successfully
- ✅ IP deduplication conflicts shown in UI with resolution options
- ✅ 25-30 tests passing

#### Deliverables
- TargetSelectorWidget (prtip-tui)
- ScanTemplate system (prtip-core + prtip-tui)
- 8 built-in templates (JSON files)
- Template editor UI (prtip-tui)
- 25-30 tests
- docs/32-USER-GUIDE.md update (+400 lines)
- Tutorial: "Using Scan Templates" (new)
- Sprint 6.5 completion report

---

### Sprint 6.6: Advanced TUI Features

**Effort:** 15-20 hours  
**Timeline:** Weeks 7-9 (June 9-23, 2026) - **Overlaps into Week 12, adjust timeline**  
**Dependencies:** Sprints 6.1 (TUI), 6.2 (dashboard), 6.5 (interactive features)  
**Parallel Eligible:** No (late-stage UI features requiring stable foundation)

#### Objectives
1. **QW-3 Completion:** Memory-mapped result streaming (20-50% memory reduction)
2. Customizable TUI layouts (save window positions, pane sizes)
3. Export functionality from TUI (JSON, XML, CSV on keypress)
4. Scan history browser (list past scans, resume/review results)
5. Zero-latency result display (streaming writer publishes events)

#### Task Areas

**Task Area 1: Memory-Mapped Streaming (QW-3 Completion, 8-12h)**
- Implement MmapResultWriter (based on prototype from Sprint 6.4)
- Design file format (JSON Lines or binary with index for fast seeking)
- Event publishing (StreamingResultWriter publishes ResultWritten events to EventBus)
- Zero-latency TUI updates (EventBus → PortTableWidget, no waiting for batch flush)
- Benchmark memory usage (target: 20-50% reduction vs in-memory SQLite batching)
- Integration with existing output formats (mmap → convert to JSON/XML on export)
- Error handling (disk full, permission denied, graceful degradation to memory)

**Task Area 2: Customizable Layouts (2-3h)**
- Implement layout serialization (save pane positions, sizes to ~/.config/prtip/layout.json)
- Layout presets (default, compact, wide, focused on ports, focused on logs)
- Keyboard shortcuts (Ctrl+1 to Ctrl+5 for presets, Ctrl+S to save current)
- Resize panes (Ctrl+Arrow keys to adjust split positions)
- Layout restoration (load saved layout on TUI startup)

**Task Area 3: Export from TUI (2-3h)**
- Add export menu (press 'e' to open, show format options: JSON, XML, CSV, Greppable)
- Implement exporters (reuse existing output format code from prtip-cli)
- File picker (enter filename, default: scan-YYYYMMDD-HHMMSS.json)
- Background export (non-blocking, show progress bar in TUI)
- Export notifications (show "Exported 1,234 results to file.json" on completion)

**Task Area 4: Scan History Browser (2-3h)**
- Design history storage (SQLite database in ~/.local/share/prtip/history.db)
- Create HistoryBrowserWidget (list past scans with timestamp, target, result count)
- Implement scan review (select scan → show results in read-only PortTableWidget)
- Add resume functionality (if scan was interrupted, offer to continue)
- History management (delete old scans, export history to archive)

**Task Area 5: Testing & Documentation (1-3h)**
- Unit tests: Mmap writer, layout serialization, export formats (15 tests)
- Integration tests: High-throughput streaming (1M results), export workflow (10 tests)
- Memory profiling: Validate 20-50% reduction vs baseline
- Performance benchmarks: Streaming vs batch writes
- Update docs/36-TUI-INTERFACE-GUIDE.md (+600 lines, advanced features)
- Create tutorial: "Customizing Your Dashboard"

#### Success Criteria
- ✅ Memory-mapped streaming reduces memory 20-50% on 1M+ result scans
- ✅ Zero-latency result display (<10ms from write to UI update)
- ✅ Custom layouts save/restore correctly
- ✅ Export functionality works for all formats (JSON, XML, CSV, Greppable)
- ✅ Scan history browser shows past scans, resume works
- ✅ 20-25 tests passing

#### Deliverables
- MmapResultWriter (prtip-core)
- Layout system (prtip-tui)
- Export menu + exporters (prtip-tui)
- HistoryBrowserWidget (prtip-tui)
- 20-25 tests
- docs/36-TUI-INTERFACE-GUIDE.md update (+600 lines)
- Tutorial: "Customizing Your Dashboard" (new)
- Sprint 6.6 completion report

---

### Sprint 6.7: NUMA Awareness & CDN Detection

**Effort:** 18-24 hours  
**Timeline:** Weeks 9-11 (June 23 - July 7, 2026) - **Extends 1 week beyond Phase 6**  
**Dependencies:** Optional (Sprint 6.4 for complete performance story)  
**Parallel Eligible:** **YES** (can run in parallel with Sprints 6.5-6.6 if team allows)

**Note:** This sprint extends 1 week beyond the original 12-week Phase 6 timeline. Consider:
- **Option A:** Execute as planned (13-week Phase 6)
- **Option B:** Defer to v0.6.1 patch release (release v0.6.0 at week 12)
- **Checkpoint:** Decide after Sprint 6.6 completion based on timeline status

#### Objectives
1. **MI-5:** NUMA-aware packet processing (10-25% gain on multi-socket, ROI 2.00)
2. **MI-3:** CDN/WAF detection (reduce false positives, ROI 2.33)
3. Enterprise-grade performance for 10+ Gbps scanning
4. Security researcher tooling for attack surface mapping

#### Task Areas

**Task Area 1: NUMA-Aware Packet Processing (12-16h)**
- Leverage existing hwlocality integration from Phase 5
- Implement topology discovery (NUMA node count, cores per socket)
- Design per-socket thread pools (one packet handler pool per NUMA node)
- Thread pinning (hwlocality API to bind threads to specific cores)
- NUMA-local memory allocation (use libnuma or manual mmap with node affinity)
- Packet distribution (route packets to threads on same NUMA node as NIC)
- Benchmark on multi-socket systems:
  - AWS c7g.metal (128 cores, 2 sockets, ARM Graviton)
  - Azure Standard_D96a_v4 (96 cores, 2 sockets, AMD EPYC)
  - Google Cloud c3-standard-176 (176 cores, 2 sockets, Intel Sapphire Rapids)
- Graceful degradation (auto-detect single-socket, disable NUMA features)
- Configuration (--numa-mode=auto|manual, --numa-bind-nics flag)

**Task Area 2: CDN/WAF Detection (6-8h)**
- Extend TLS certificate analysis from Sprint 5.5 (already have X.509 parsing)
- HTTP header fingerprinting (Server, X-Cache, CF-Ray, X-Amz-Cf-Id, Akamai-Origin-Hop, Fastly-Debug-Digest)
- ASN lookup integration:
  - Option A: External API (ipapi.co, ip-api.com, free tier limits)
  - Option B: Embedded BGP table (MaxMind GeoLite2 ASN database)
- CDN provider database (top 10 CDNs):
  1. Cloudflare (AS13335, AS209242)
  2. Akamai (AS20940, AS16625, AS16702, AS17204)
  3. Amazon CloudFront (AS16509)
  4. Fastly (AS54113)
  5. Google Cloud CDN (AS15169)
  6. Microsoft Azure CDN (AS8075)
  7. Alibaba Cloud CDN (AS37963, AS45102)
  8. Tencent Cloud CDN (AS45090)
  9. Limelight Networks (AS22822)
  10. StackPath (AS33438)
- TUI indicator (shield icon, provider name, "CDN: Cloudflare" in PortTableWidget)
- CLI flag (--exclude-cdns to auto-skip CDN ranges, report separately)

**Task Area 3: Testing & Benchmarking (4-6h)**
- NUMA tests: Topology detection, thread affinity validation (15 tests)
- CDN tests: Header fingerprinting, ASN lookup, provider detection (15 tests)
- NUMA benchmarks: Single-socket vs multi-socket performance (cloud instances)
- CDN accuracy validation: Test against known CDN endpoints (>90% accuracy target)
- Integration tests: End-to-end NUMA + CDN workflows (10 tests)

**Task Area 4: Documentation (2-4h)**
- Create docs/37-NUMA-OPTIMIZATION-GUIDE.md (~600 lines):
  - NUMA concepts for network scanning
  - ProRT-IP NUMA architecture
  - Configuration options and tuning
  - Benchmark results on multi-socket systems
  - Troubleshooting (topology detection, affinity issues)
- Create docs/38-CDN-DETECTION-GUIDE.md (~400 lines):
  - CDN detection methodology
  - Supported CDN providers
  - HTTP/TLS fingerprinting details
  - Use cases for security research
- Update docs/32-USER-GUIDE.md (+200 lines, NUMA and CDN sections)

#### Success Criteria
- ✅ NUMA optimization delivers 10-25% throughput gain on multi-socket systems
- ✅ Graceful single-socket operation (no errors, auto-detects topology)
- ✅ CDN detection accuracy >90% for top 10 providers
- ✅ TUI shows CDN indicators (shield icon, provider name)
- ✅ 25-30 tests passing
- ✅ Documentation complete (2 new guides, 1,000+ lines)

#### Deliverables
- NUMA-aware thread pooling (prtip-network)
- CDN detection system (prtip-scanner)
- 25-30 tests
- docs/37-NUMA-OPTIMIZATION-GUIDE.md (600 lines)
- docs/38-CDN-DETECTION-GUIDE.md (400 lines)
- docs/32-USER-GUIDE.md update (+200 lines)
- Sprint 6.7 completion report

#### Contingency Planning
**If Timeline Pressures Emerge:**
- **Option A (Recommended):** Extend Phase 6 by 1 week (acceptable, delivers complete feature set)
- **Option B:** Defer Sprint 6.7 to v0.6.1 (release v0.6.0 without NUMA/CDN, add in patch)
- **Decision Point:** After Sprint 6.6 completion (week 9), assess if 2 weeks remain for 6.7+6.8

---

### Sprint 6.8: Documentation, Polish & Release Preparation

**Effort:** 15-20 hours  
**Timeline:** Weeks 11-12 (July 7-14, 2026) - **Final week of Phase 6**  
**Dependencies:** ALL sprints (6.1-6.7 must be complete)  
**Parallel Eligible:** No (final integration and documentation phase)

#### Objectives
1. Comprehensive user documentation (1,500-2,000 lines)
2. API documentation updates (rustdoc, cross-references)
3. Quality assurance (accessibility, cross-platform testing)
4. Performance regression testing (validate all Quick Win gains)
5. Release notes preparation for v0.6.0

#### Task Areas

**Task Area 1: Comprehensive User Documentation (6-8h)**

**docs/36-TUI-INTERFACE-GUIDE.md (1,500-2,000 lines):**
- Section 1: Introduction (what is TUI mode, benefits vs CLI)
- Section 2: Getting Started (launching TUI, basic navigation, first scan)
- Section 3: Dashboard Overview (header, progress, port table, log viewer explained)
- Section 4: Keyboard Shortcuts Reference (comprehensive table, grouped by function)
- Section 5: Interactive Target Selection (multi-select, CIDR preview, conflict resolution)
- Section 6: Scan Templates (built-in templates, creating custom, template editor)
- Section 7: Customizing Layouts (presets, saving layouts, pane resizing)
- Section 8: Advanced Features (export, scan history, performance tuning)
- Section 9: Troubleshooting (common issues, performance tips, terminal compatibility)
- Section 10: Accessibility (colorblind themes, keyboard-only navigation)

**3 TUI Tutorials (~700 lines total):**
1. **"Getting Started with TUI Mode"** (250 lines)
   - Installation and setup
   - Launching TUI for first time
   - Basic scan workflow
   - Understanding dashboard widgets
   - Interpreting results
   
2. **"Customizing Your Dashboard"** (250 lines)
   - Layout presets overview
   - Creating custom layouts
   - Saving and restoring layouts
   - Keyboard shortcuts for productivity
   - Theme customization
   
3. **"Performance Tuning for Large Scans"** (200 lines)
   - Understanding adaptive batching
   - NUMA optimization (if available)
   - Memory-mapped streaming for million+ results
   - Monitoring performance with TUI metrics
   - Troubleshooting slow scans

**Examples Gallery (20+ scenarios, ~800 lines):**
- Basic single-host scan with TUI
- Multi-target scan with interactive selection
- Using built-in templates (T0-T5, Web, Database)
- Creating custom templates
- Exporting results from TUI (JSON, XML, CSV)
- Reviewing scan history
- Large-scale internet scan with memory streaming
- NUMA-optimized enterprise scan (multi-socket systems)
- CDN detection workflow
- Troubleshooting scenarios (terminal issues, performance problems)

**Task Area 2: API Documentation (2-3h)**

**Rustdoc Updates:**
- prtip-tui crate documentation (all public APIs)
- Performance optimization APIs (adaptive batching, NUMA, streaming)
- CDN detection module
- Cross-reference linking:
  - Code → Guides (link rustdoc to 36-TUI-INTERFACE-GUIDE.md sections)
  - Guides → Code (link guide examples to specific functions/structs)
  - Bidirectional network (follow Sprint 5.5.1 pattern)

**Validation:**
- Run `cargo doc --open` and verify all links work
- Check rustdoc tests (ensure code examples compile)
- 0 rustdoc warnings (all public APIs documented)

**Task Area 3: Quality Assurance (3-4h)**

**Accessibility Testing:**
- Colorblind-friendly themes (3 palettes):
  1. **Deuteranopia** (red-green colorblindness, 5% of males)
  2. **Protanopia** (red-green colorblindness, 1% of males)
  3. **Tritanopia** (blue-yellow colorblindness, <1%)
- Keyboard-only navigation validation (no mouse required for any feature)
- Screen reader hints (ARIA-like labels for TUI elements, document in guide)
- High-contrast mode (for low-vision users)

**Cross-Platform Testing:**
- **Linux:** GNOME Terminal, Alacritty, kitty, xterm, konsole
- **macOS:** iTerm2, Terminal.app, Hyper
- **Windows:** Windows Terminal, PowerShell, cmd.exe
- Test matrix (3 OS × 3 terminals = 9 combinations minimum)
- Document known issues (e.g., cmd.exe Unicode limitations)

**Performance Regression Testing:**
- Re-run all benchmarks from Sprints 6.3-6.7
- Verify Quick Win gains:
  - QW-1 (Adaptive Batching): 15-30% ✓
  - QW-2 (sendmmsg/recvmmsg): 20-40% ✓ (Linux only)
  - QW-3 (Memory-mapped streaming): 20-50% memory reduction ✓
  - QW-4 (IP Deduplication): 30-70% scan reduction ✓
  - NUMA: 10-25% gain on multi-socket ✓
- Regression detection (no slowdowns >5% threshold)
- Document performance in release notes

**Task Area 4: Release Preparation (4-5h)**

**CHANGELOG.md Update (comprehensive Phase 6 entry, 150-200 lines):**
- Executive summary (TUI + 5 Quick Wins + 3 Medium Impact)
- Features added (TUI widgets, templates, NUMA, CDN)
- Performance improvements (with benchmark numbers)
- Breaking changes (if any, document migration path)
- Bug fixes (any issues fixed during Phase 6)
- Documentation additions (guide count, line count)
- Contributors (acknowledge anyone who helped)

**Release Notes for v0.6.0 (200-250 lines, follows v0.4.x-v0.5.0 quality standard):**
- Executive summary (1 paragraph, highlight TUI + performance)
- Features (detailed list with screenshots/ASCII art of TUI)
- Performance metrics (before/after comparison table)
- Technical details (architecture diagrams, EventBus integration)
- Files changed (summary statistics)
- Testing (test count, coverage percentage)
- Documentation (new guides, total documentation line count)
- Strategic value (competitive position, user impact)
- Future work (teaser for Phase 7)
- Installation instructions (cargo install, package managers)
- Known issues (document any limitations, workarounds)
- Platform support matrix (OS × architecture combinations)

**Final Integration Testing (15-20 test scenarios):**
1. TUI launches on all platforms (Linux, macOS, Windows)
2. EventBus integration (1K events/sec, verify display)
3. Interactive target selection (10K IPs, multi-select)
4. Scan templates (all 8 built-in templates work)
5. Export functionality (JSON, XML, CSV, Greppable)
6. Scan history (save, review, resume)
7. Adaptive batching (verify throughput increase)
8. sendmmsg/recvmmsg (Linux benchmark)
9. Memory-mapped streaming (1M results, memory profiling)
10. IP deduplication (overlapping CIDRs)
11. NUMA optimization (multi-socket benchmark)
12. CDN detection (test against 10 known CDN hosts)
13. Custom layouts (save, restore)
14. Accessibility (colorblind themes, keyboard-only)
15. Cross-platform (9 OS/terminal combinations)
16. Performance regression (all benchmarks pass)
17. Documentation (all links work, examples compile)
18. Clippy (0 warnings)
19. Rustfmt (clean)
20. Full test suite (2,260-2,300 tests, 100% pass rate)

**User Acceptance Testing (5-10 external testers):**
- Recruit from GitHub community (call for beta testers)
- Provide test scenarios document
- Collect feedback via GitHub Discussions or survey
- Incorporate critical feedback before release
- Document testimonials for release announcement

#### Success Criteria
- ✅ All documentation complete (6,500-8,000 new lines)
- ✅ 0 clippy warnings, 0 rustfmt violations across all code
- ✅ All tests passing (2,102 existing + 160-200 new = 2,260-2,300 total)
- ✅ Performance targets met (all Quick Wins validated)
- ✅ Cross-platform tested (9 combinations, known issues documented)
- ✅ Accessibility validated (colorblind themes, keyboard-only)
- ✅ Release notes ready for v0.6.0 (200-250 lines)
- ✅ User acceptance testing positive (>80% satisfaction)

#### Deliverables
- docs/36-TUI-INTERFACE-GUIDE.md (1,500-2,000 lines)
- 3 TUI tutorials (700 lines)
- Examples gallery (20+ scenarios, 800 lines)
- Rustdoc updates (cross-references, 0 warnings)
- Accessibility themes (3 colorblind palettes)
- CHANGELOG.md Phase 6 entry (150-200 lines)
- Release notes v0.6.0 (200-250 lines)
- Cross-platform test report (9 combinations)
- Performance comparison report (before/after Quick Wins)
- User acceptance testing summary
- Sprint 6.8 completion report
- **v0.6.0 Release** (final deliverable!)

---

## 4. Risk Management

### 4.1 Technical Risks

**Risk 1: TUI Performance Degradation at High Event Rates**
- **Threat:** EventBus can generate >10K events/sec, overwhelming 60 FPS rendering
- **Probability:** Medium (likely on fast scans)
- **Impact:** High (unresponsive TUI, poor user experience)
- **Mitigation:**
  - Implement event sampling/aggregation layer between EventBus and TUI
  - Example: Aggregate 100 PortFound events → single "100 ports discovered" update
  - Rate limit UI updates to 60 Hz (16ms budget)
  - Test with stress scenarios (10M port scan on 1 Gbps link)
- **Owner:** Sprint 6.2 (Live Dashboard)
- **Contingency:** Reduce event publishing rate in scanner if TUI can't keep up

**Risk 2: Cross-Platform TUI Compatibility Issues**
- **Threat:** Terminal emulator differences (Windows cmd.exe vs Linux xterm vs macOS iTerm2)
- **Probability:** Medium (expected on some terminals)
- **Impact:** Medium (degraded UX, potential crashes on unsupported terminals)
- **Mitigation:** 
  - Use crossterm (abstracts platform differences, battle-tested)
  - Test matrix: Windows Terminal, cmd.exe, PowerShell, iTerm2, GNOME Terminal, Alacritty
  - Fallback to basic ASCII if Unicode glyphs unsupported (detect terminal capabilities)
  - Document known issues (e.g., cmd.exe Unicode limited, recommend Windows Terminal)
- **Owner:** Sprint 6.1 (Framework Setup) + 6.8 (Testing)
- **Contingency:** Provide CLI fallback if TUI unsupported on platform

**Risk 3: sendmmsg/recvmmsg Platform Support**
- **Threat:** sendmmsg/recvmmsg are Linux-specific (not on macOS/BSD, Windows N/A)
- **Probability:** High (certain on non-Linux platforms)
- **Impact:** Low (performance difference documented, not a blocker)
- **Mitigation:**
  - Feature flag: `cfg(target_os = "linux")` for batching code
  - Fallback to standard sendto/recvfrom on other platforms (automatic, transparent)
  - Document performance difference (Linux: 2-5x faster, macOS/Windows: baseline)
  - CI testing on all platforms (ensure fallback works)
- **Owner:** Sprint 6.3 (Network Optimization)
- **Contingency:** Accept performance difference, promote Linux for production use

**Risk 4: NUMA Testing Hardware Availability**
- **Threat:** Multi-socket systems expensive/hard to access for testing
- **Probability:** Medium (requires cloud instances or enterprise hardware)
- **Impact:** Medium (can't validate NUMA gains without multi-socket testing)
- **Mitigation:**
  - Use cloud instances (AWS c7g.metal, Azure Standard_D96a_v4) for benchmarking
  - Budget: $50-100 for 8-10 hours of testing (acceptable for professional project)
  - Graceful degradation on single-socket systems (auto-detect via hwlocality, disable NUMA features)
  - Document benchmarks with cloud instance specs (reproducible)
- **Owner:** Sprint 6.7 (NUMA Awareness)
- **Contingency:** Defer NUMA optimization to v0.6.1 if budget/access issues

**Risk 5: Documentation Drift**
- **Threat:** Code evolves faster than documentation updates, stale examples
- **Probability:** Medium (common in fast-paced development)
- **Impact:** High (user frustration, support burden)
- **Mitigation:**
  - Per-sprint documentation requirements (part of Definition of Done)
  - Rustdoc tests (ensure code examples compile, catch API changes)
  - Cross-reference validation script (from Sprint 5.5.1, check all internal links)
  - Documentation review in Sprint 6.8 (dedicated time for catching drift)
- **Owner:** All sprints + Sprint 6.8 (final validation)
- **Contingency:** Hotfix documentation in v0.6.1 if issues discovered post-release

### 4.2 Schedule Risks

**Risk 1: Phase 6 TUI Delays**
- **Threat:** MI-1 preparation takes longer than estimated (20-30h → 40h)
- **Probability:** Low (Sprint 5.5.3 already completed EventBus foundation)
- **Impact:** Medium (delays downstream sprints)
- **Mitigation:**
  - EventBus already proven stable (40ns latency, >10M events/sec, Sprint 5.5.3)
  - Timebox Sprint 6.1 to 20h max (MVP TUI acceptable if pressed)
  - MVP: Basic TUI with EventBus subscription (defer advanced widgets to 6.2)
- **Contingency:** Phase 6 slips 1-2 weeks (still Q2 2026, acceptable)

**Risk 2: Quick Win Cascading Delays**
- **Threat:** QW-1 through QW-5 exceed 52h estimate, delay later sprints
- **Probability:** Medium (performance optimization can reveal unexpected issues)
- **Impact:** Medium (delays Sprint 6.5-6.8)
- **Mitigation:**
  - Prioritize QW-1 and QW-2 (highest ROI, 5.33 and 4.00)
  - Defer QW-5 (Scan Templates, ROI 3.33) to v0.6.1 if needed (lower priority)
  - QW-3 split across 6.4 (prep) and 6.6 (completion) allows adjustment
- **Contingency:** Release v0.6.0 with 3-4 Quick Wins, defer 1-2 to v0.6.1

**Risk 3: Sprint 6.7 Timeline Extension**
- **Threat:** Sprint 6.7 extends 1 week beyond 12-week Phase 6 timeline
- **Probability:** Medium (Sprint 6.7 is 18-24h, may need 2 weeks)
- **Impact:** Low (1-week extension acceptable for completeness)
- **Mitigation:**
  - Checkpoint decision after Sprint 6.6 (assess remaining time)
  - Option A: Extend Phase 6 by 1 week (13 weeks total, still Q2 2026)
  - Option B: Defer Sprint 6.7 to v0.6.1 (release v0.6.0 without NUMA/CDN)
- **Contingency:** Option B (defer 6.7) if hard deadline for v0.6.0

### 4.3 Community Risks

**Risk 1: NSE Compatibility Expectations**
- **Threat:** Users expect 100% NSE script compatibility (MI-4 deferred to Phase 7)
- **Probability:** Medium (Nmap users may assume compatibility)
- **Impact:** Medium (community disappointment, GitHub issues)
- **Mitigation:**
  - Clear documentation of supported subset (ProRT-IP Lua plugins, not NSE)
  - FAQ explaining ProRT-IP plugin advantages (sandboxing, capabilities, security)
  - Roadmap transparency (NSE compatibility planned for Phase 7 or later)
  - Highlight existing plugin system (Sprint 5.8, 2 example plugins)
- **Communication:** Blog post, GitHub README, release notes all mention NSE status
- **Contingency:** Fast-track NSE adapter in v0.7.0 if community demand high

**Risk 2: Performance Claims**
- **Threat:** "15-30% throughput" may not manifest on all networks/hardware
- **Probability:** Medium (network conditions vary widely)
- **Impact:** Medium (credibility risk if claims don't match real-world results)
- **Mitigation:**
  - Document benchmark conditions (hardware specs, network type, scan scenarios)
  - Provide variance ranges (15-30% means some users see 15%, others 30%)
  - Publish raw benchmark data (GitHub repo, invite community validation)
  - Transparency: "Your mileage may vary" disclaimers in release notes
- **Communication:** Performance guide explains factors affecting gains
- **Contingency:** Adjust claims in documentation if community feedback shows lower gains

---

## 5. Success Criteria

### 5.1 Performance Success Criteria

**Quick Win Targets (Tier 1):**
- ✅ **QW-1 (Adaptive Batching):** 15-30% throughput improvement on benchmark suite
- ✅ **QW-2 (sendmmsg/recvmmsg):** 20-40% throughput gain on Linux vs baseline
- ✅ **QW-3 (Memory-mapped streaming):** 20-50% memory reduction for large result sets (1M+ results)
- ✅ **QW-4 (IP Deduplication):** 30-70% scan time reduction for redundant ranges (measured on overlapping CIDRs)
- ✅ **Combined Impact:** 35-70% overall throughput increase (stacked optimizations)

**Medium Impact Targets (Tier 2):**
- ✅ **NUMA Awareness:** 10-25% throughput gain on multi-socket systems (validated on cloud instances)
- ✅ **CDN Detection:** >90% accuracy for top 10 CDN providers (tested against known endpoints)

**TUI Performance:**
- ✅ **Rendering:** <5% CPU overhead vs CLI mode (measured with `perf`)
- ✅ **Frame Rate:** Maintain 60 FPS with 10K events/sec load
- ✅ **Latency:** <100ms from EventBus event to screen update
- ✅ **Event Handling:** Support >10M events/sec without dropped frames (via aggregation)

### 5.2 Functional Success Criteria

**TUI Features:**
- ✅ TUI launches successfully on Linux, macOS, Windows (all 3 platforms tested)
- ✅ Live port discovery updates <100ms latency from EventBus to display
- ✅ Interactive target selection supports 10K+ IPs without lag
- ✅ Scan templates reduce configuration time by 50% vs manual (measured via user testing)
- ✅ Custom layouts save/restore correctly
- ✅ Export functionality works from TUI (JSON, XML, CSV, Greppable formats)
- ✅ Scan history browser shows past scans, resume works

**Performance Features:**
- ✅ Adaptive batching adjusts dynamically based on network conditions (validated with varying RTT)
- ✅ IP deduplication detects overlapping CIDRs accurately (tested with 100+ range combinations)
- ✅ NUMA optimization binds threads to correct sockets (verified with hwlocality)
- ✅ CDN detection identifies top 10 providers (>90% accuracy on test set)

### 5.3 Quality Success Criteria

**Testing:**
- ✅ **Total Tests:** 2,260-2,300 (2,102 existing + 160-200 new)
- ✅ **Test Pass Rate:** 100% (0 failures)
- ✅ **Code Coverage:** Maintain >60% overall (currently 54.92%, target 60%+)
- ✅ **New Code Coverage:** >90% for Phase 6 additions
- ✅ **Benchmark Suite:** All scenarios show performance gains (no regressions >5%)

**Code Quality:**
- ✅ **Clippy Warnings:** 0 (cargo clippy -- -D warnings)
- ✅ **Rustfmt:** 100% compliant (cargo fmt --check)
- ✅ **Rustdoc Warnings:** 0 (all public APIs documented)
- ✅ **Rustdoc Tests:** 100% passing (code examples compile)

**Cross-Platform:**
- ✅ **CI/CD:** All tests pass on Linux, macOS, Windows (GitHub Actions matrix)
- ✅ **Terminal Support:** Works on 9 terminal combinations (3 OS × 3 terminals minimum)
- ✅ **Platform-Specific:** sendmmsg/recvmmsg on Linux, graceful fallback on macOS/Windows

**Documentation:**
- ✅ **Completeness:** 100% public API coverage (rustdoc)
- ✅ **New Documentation:** 6,500-8,000 lines (guides, tutorials, examples)
- ✅ **Cross-References:** 0 broken links (validated with script from Sprint 5.5.1)
- ✅ **Examples:** 20+ TUI scenarios (basic to advanced)

### 5.4 User Experience Success Criteria

**Usability:**
- ✅ **Learning Curve:** <30 second to basic TUI navigation (measured via UAT)
- ✅ **Discoverability:** Help text visible in footer, '?' key shows detailed help
- ✅ **Error Messages:** Actionable error messages with resolution steps
- ✅ **Responsiveness:** UI feels responsive (<100ms input lag)

**Accessibility:**
- ✅ **Colorblind-Friendly:** 3 themes available (deuteranopia, protanopia, tritanopia)
- ✅ **Keyboard-Only:** No mouse required for any TUI feature
- ✅ **Screen Reader:** ARIA-like labels documented for TUI elements
- ✅ **High-Contrast:** Available for low-vision users

**Performance Perception:**
- ✅ **Real-Time Updates:** Port discoveries appear immediately in table (<100ms)
- ✅ **Progress Indicators:** ETA, throughput, completion % always visible
- ✅ **Scan Templates:** Reduce setup time from 5 minutes to 30 seconds (measured)

---

## 6. Testing Strategy

### 6.1 Testing Levels

**Level 1: Unit Testing (per sprint, 15-30 tests each)**
- TUI components: widget rendering, state transitions, keyboard input handling
- Performance optimizations: batch sizing algorithms, deduplication logic, NUMA affinity
- Integration points: EventBus subscriptions, ProgressAggregator connections
- **Coverage Target:** >90% for new code

**Level 2: Integration Testing (per sprint, 10-15 tests each)**
- End-to-end TUI workflows: launch → configure → scan → view results → export
- EventBus → TUI rendering pipeline (verify 60 FPS with 10K events/sec)
- Performance benchmarks: syscall batching throughput, memory-mapped streaming latency
- **Coverage Target:** All critical paths tested

**Level 3: Performance Testing (Sprints 6.3, 6.4, 6.7)**
- Benchmark before/after for each Quick Win (20-40% throughput gain for QW-2, etc.)
- Hyperfine integration (already have from Sprint 5.9)
- Regression detection (5% threshold from benchmarking framework)
- Multi-socket NUMA benchmarks (AWS/Azure cloud instances)
- **Scenarios:** 10 benchmark scenarios from Sprint 5.9 framework

**Level 4: User Acceptance Testing (Sprint 6.8)**
- Manual TUI workflows (10+ scenarios with external testers)
- Accessibility testing (colorblind themes, keyboard-only navigation)
- Cross-platform validation (Linux, macOS, Windows across 9 terminal combinations)
- User satisfaction survey (target >80% satisfaction rate)

### 6.2 Benchmark Scenarios (Sprint 5.9 Framework)

1. **Small Network:** 256 IPs, 100 ports (baseline performance)
2. **Medium Network:** 4096 IPs, 1000 ports (typical corporate scan)
3. **Large Network:** 65536 IPs, 100 ports (internet-scale simulation)
4. **Single Host Full:** 1 IP, 65535 ports (exhaustive port scan)
5. **Internet-Scale Simulation:** 1M IPs, 1 port (stateless SYN sweep)
6. **Service Detection:** 100 IPs, 10 services (deep enumeration)
7. **TLS Certificate Extraction:** 50 HTTPS hosts (Sprint 5.5 validation)
8. **IPv6 Scanning:** 1024 IPv6 addresses (dual-stack testing)
9. **Idle Scan:** Zombie + 100 targets (Sprint 5.3 validation)
10. **Rate-Limited Scan:** 10K IPs at 1000 pps (RateLimiterV3 validation)

**Quick Win Validation:**
- Run all 10 scenarios **before** implementing Quick Wins (baseline)
- Run all 10 scenarios **after** each Quick Win (measure delta)
- Expected gains:
  - QW-1: 15-30% faster (scenarios 1-5)
  - QW-2: 20-40% faster on Linux (scenarios 1-5)
  - QW-3: 20-50% less memory (scenarios 3-5)
  - QW-4: 30-70% faster on overlapping ranges (custom scenario)
  - NUMA: 10-25% faster on multi-socket (cloud instance)

### 6.3 Correctness Validation

**For Each Feature:**
1. **Unit Tests:** 10+ tests per new function/module (edge cases, error paths)
2. **Integration Tests:** End-to-end scenarios (happy path + error cases)
3. **Fuzz Testing:** Structure-aware fuzzing with arbitrary crate (validated in Sprint 5.7)
4. **Cross-Platform:** Linux, Windows, macOS validation (CI matrix)
5. **Coverage:** Maintain >54% overall (current), >90% for new code (target 60%+ overall)

### 6.4 Regression Prevention

**CI/CD Gates (GitHub Actions):**
- ✅ All 2,102+ tests must pass (100% pass rate)
- ✅ 0 clippy warnings (cargo clippy -- -D warnings)
- ✅ cargo fmt check (100% formatted)
- ✅ Benchmark regression checks (5% threshold, fail if >5% slower)
- ✅ Cross-platform matrix (Linux, macOS, Windows × x86_64, aarch64 when feasible)
- ✅ Documentation checks (rustdoc builds, 0 warnings, cross-references valid)

---

## 7. Documentation Plan

### 7.1 Per-Sprint Documentation

**Sprint 6.1 (TUI Framework):**
- docs/TUI-ARCHITECTURE.md (500 lines, component diagram, data flow)
- Rustdoc for prtip-tui crate (all public APIs)

**Sprint 6.2 (Live Dashboard):**
- docs/TUI-ARCHITECTURE.md update (+200 lines, widget interaction diagram)
- docs/36-TUI-INTERFACE-GUIDE.md (new, 300 lines, basic usage section)

**Sprint 6.3 (Network Optimization):**
- docs/26-RATE-LIMITING-GUIDE.md update (+300 lines, batching section)

**Sprint 6.4 (Adaptive Tuning):**
- docs/26-RATE-LIMITING-GUIDE.md update (+200 lines, adaptive tuning section)

**Sprint 6.5 (Interactive Selection + Templates):**
- docs/32-USER-GUIDE.md update (+400 lines, templates section)
- Tutorial: "Using Scan Templates" (new)

**Sprint 6.6 (Advanced Features):**
- docs/36-TUI-INTERFACE-GUIDE.md update (+600 lines, advanced features)
- Tutorial: "Customizing Your Dashboard" (new)

**Sprint 6.7 (NUMA + CDN):**
- docs/37-NUMA-OPTIMIZATION-GUIDE.md (new, 600 lines)
- docs/38-CDN-DETECTION-GUIDE.md (new, 400 lines)
- docs/32-USER-GUIDE.md update (+200 lines, NUMA and CDN sections)

**Sprint 6.8 (Documentation + Polish):**
- docs/36-TUI-INTERFACE-GUIDE.md (complete, 1,500-2,000 lines)
- Tutorial: "Getting Started with TUI Mode" (new)
- Tutorial: "Performance Tuning for Large Scans" (new)
- Examples gallery (20+ TUI scenarios, 800 lines)
- CHANGELOG.md Phase 6 entry (150-200 lines)
- Release notes v0.6.0 (200-250 lines)

### 7.2 Documentation Metrics

**Total New Documentation:** 6,500-8,000 lines

**Breakdown:**
- Technical Guides: 2,400 lines (TUI-ARCHITECTURE, NUMA, CDN, updates)
- User Guides: 2,500 lines (TUI-INTERFACE-GUIDE, USER-GUIDE updates)
- Tutorials: 700 lines (3 tutorials)
- Examples: 800 lines (20+ scenarios)
- Release Materials: 400 lines (CHANGELOG, release notes)
- Rustdoc: ~100 lines (API documentation)

**Quality Standards:**
- 0 broken links (validated with script)
- 100% code examples compile (rustdoc tests)
- Cross-reference network (bidirectional code ↔ guides)
- Accessibility (clear language, examples for all features)

---

## 8. Timeline & Milestones

### 8.1 Weekly Breakdown

**Week 1 (April 1-7): Sprint 6.1 Foundation**
- Setup ratatui + crossterm
- Create prtip-tui crate
- Implement EventSubscriber
- Basic rendering loop
- **Milestone:** TUI launches, handles 1K events/sec

**Week 2 (April 8-14): Sprint 6.1 Complete + Sprint 6.2 Start**
- Complete Sprint 6.1 testing & docs
- Start Sprint 6.2: Multi-pane layout
- **Milestone:** Demo-ready TUI for RSA Conference (mid-April)

**Week 3 (April 15-21): Sprint 6.2 Progress Integration**
- ProgressWidget (ETA, throughput)
- PortTableWidget (live streaming)
- LogViewerWidget
- **Milestone:** Live dashboard operational

**Week 4 (April 22-28): Sprint 6.2 Complete + Sprint 6.3 Start**
- Complete Sprint 6.2 performance optimization
- Start Sprint 6.3: sendmmsg/recvmmsg batching
- **Milestone:** <5% CPU overhead validated

**Week 5 (April 29 - May 5): Sprint 6.3 Network Optimization**
- sendmmsg/recvmmsg implementation
- IP deduplication
- Platform-specific testing
- **Milestone:** 20-40% throughput gain on Linux

**Week 6 (May 6-12): Sprint 6.3 Complete + Sprint 6.4 Start**
- Complete Sprint 6.3 benchmarking
- Start Sprint 6.4: Adaptive batch tuning
- **Milestone:** Performance Quick Wins validated

**Week 7 (May 13-19): Sprint 6.4 Complete + Sprint 6.5 Start**
- Complete Sprint 6.4 (adaptive + mmap prep)
- Start Sprint 6.5: Interactive target selection
- **Milestone:** Adaptive batching 15-30% gain

**Week 8 (May 20-26): Sprint 6.5 Interactive Selection**
- TargetSelectorWidget
- Scan templates (8 built-in)
- Template editor
- **Milestone:** Templates reduce setup time 50%

**Week 9 (May 27 - June 2): Sprint 6.5 Complete + Sprint 6.6 Start**
- Complete Sprint 6.5 testing
- Start Sprint 6.6: Memory-mapped streaming
- **Milestone:** Interactive features complete

**Week 10 (June 3-9): Sprint 6.6 Advanced Features**
- MmapResultWriter (20-50% memory reduction)
- Customizable layouts
- Export functionality
- Scan history browser
- **Milestone:** Feature freeze checkpoint (decide on Sprint 6.7 timeline)

**Week 11 (June 10-16): Sprint 6.6 Complete + Sprint 6.7 Start**
- Complete Sprint 6.6 testing
- Start Sprint 6.7: NUMA awareness
- **Milestone:** All TUI features complete

**Week 12 (June 17-23): Sprint 6.7 NUMA + CDN**
- NUMA thread pinning
- CDN detection
- Multi-socket benchmarking
- **Milestone:** NUMA 10-25% gain validated

**Week 13 (June 24-30): Sprint 6.7 Complete + Sprint 6.8 Documentation**
**Note:** Extends 1 week beyond 12-week Phase 6
- Complete Sprint 6.7 testing
- Sprint 6.8: Comprehensive documentation
- User acceptance testing
- **Milestone:** Documentation complete (6,500-8,000 lines)

**Week 14 (July 1-7): Sprint 6.8 Polish + Release**
- Accessibility testing
- Cross-platform validation
- Performance regression testing
- CHANGELOG & release notes
- **Milestone:** v0.6.0 release ready

**Week 15 (July 8-14): v0.6.0 Release**
- Final integration testing
- Tag v0.6.0
- Create GitHub release
- Publish announcement (blog, Reddit, HN)
- **MILESTONE: v0.6.0 RELEASED** 🎉

### 8.2 Strategic Milestones

**M1: RSA Conference Demo (mid-April, Week 2)**
- TUI launches and displays live scan progress
- Basic dashboard with progress bars
- Demonstrates real-time port discovery
- **Value:** Early visibility for ProRT-IP at major security conference

**M2: Performance Quick Wins Validated (mid-May, Week 6)**
- sendmmsg/recvmmsg delivers 20-40% gain on Linux
- IP deduplication works on overlapping ranges
- Adaptive batching shows 15-30% improvement
- **Value:** Proof of performance optimizations, builds confidence

**M3: Feature Freeze (early June, Week 10)**
- All TUI features complete (6.1-6.6)
- All Quick Wins integrated
- Decision point: Proceed with Sprint 6.7 or defer to v0.6.1
- **Value:** Checkpoint for timeline assessment, allows contingency planning

**M4: Documentation Complete (late June, Week 13)**
- 6,500-8,000 new documentation lines
- All guides, tutorials, examples ready
- Cross-platform testing done
- **Value:** Production-ready documentation quality

**M5: v0.6.0 Release (July 15-20)**
- Complete Phase 6 with TUI + 5 Quick Wins + 3 Medium Impact
- 2,260-2,300 tests passing (100% pass rate)
- Release notes published (200-250 lines)
- **Value:** Major milestone, competitive positioning, community impact

### 8.3 Contingency Timeline

**If Timeline Pressures Emerge (after Sprint 6.6):**

**Option A: 13-Week Phase 6 (Recommended)**
- Extend by 1 week to complete Sprint 6.7 + 6.8
- v0.6.0 release July 15-20 (still Q2-Q3 transition)
- **Pros:** Complete feature set, NUMA + CDN included
- **Cons:** 1 week delay (acceptable)

**Option B: Defer Sprint 6.7 to v0.6.1**
- Release v0.6.0 at Week 12 (mid-June)
- Defer NUMA + CDN to v0.6.1 patch (late June)
- **Pros:** Hits 12-week target, delivers TUI + Quick Wins
- **Cons:** Missing NUMA/CDN advanced features in major release

**Decision Point:** After Sprint 6.6 completion (Week 10), assess:
- Time remaining (2 weeks?)
- Sprint 6.7 progress (if started in parallel)
- Team capacity
- Release deadline pressure

---

## 9. Integration with ProRT-IP Roadmap

### 9.1 Phase Dependencies

**Phase 5 (Complete) → Phase 6:**
- ✅ **Sprint 5.5.3 EventBus:** Provides TUI foundation (40ns latency, 18 events, ProgressAggregator)
- ✅ **Sprint 5.9 Benchmarking:** Framework for validating Quick Win performance gains
- ✅ **Sprint 5.8 Plugin System:** Demonstrates extensibility (not required for Phase 6, but shows architecture)

**Phase 6 → Phase 7 (Web Interface):**
- EventBus from Sprint 5.5.3 (already complete) + TUI patterns from Phase 6 inform web architecture
- Real-time progress metrics (ProgressAggregator) can power web dashboard
- Export functionality from Sprint 6.6 enables web-based report generation

**Phase 6 → Phase 8 (Polish & Community):**
- NSE compatibility (MI-4, deferred from Phase 6) becomes part of Phase 8 ecosystem integration
- Scan templates from Sprint 6.5 enable community-contributed presets
- Documentation quality from Sprint 6.8 lowers barrier to contribution

### 9.2 Competitive Position

**Current State (Phase 5 Complete):**
- ProRT-IP: #3-4 overall (tied with RustScan)
- Strengths: Feature breadth, detection accuracy, modern architecture
- Gaps: Moderate throughput, no TUI, missing some optimizations

**After Phase 6:**
- **ProRT-IP: #2 overall** (behind only Masscan in raw throughput)
- **Strengths:**
  - Best TUI in class (none of the top 5 have ratatui-quality TUI)
  - Competitive throughput (35-70% increase via Quick Wins)
  - Feature completeness (8 scan types, IPv6, service detection, OS fingerprinting, TLS, plugins)
  - Modern UX (real-time progress, interactive selection, templates)
- **Remaining Gaps:**
  - Raw throughput vs Masscan (10M+ pps, ProRT-IP ~50K pps base, ~70-85K after Phase 6)
  - Accepted trade-off: Masscan is stateless-only, ProRT-IP is hybrid (stateful + stateless)

**Positioning Statement (post-Phase 6):**
> "ProRT-IP combines Masscan-class speed with Nmap-depth detection, wrapped in a modern TUI that makes advanced scanning accessible to security professionals at all levels."

### 9.3 Community Impact

**Target Audiences:**
1. **Network Security Professionals:** TUI improves reconnaissance workflows, templates save time
2. **Red Team Operators:** Competitive with commercial tools (Core Impact, Cobalt Strike)
3. **Academic Researchers:** Reproducible benchmarks, NUMA optimization for research-grade experiments
4. **Open Source Contributors:** Professional documentation, clean architecture, extension points

**Expected Outcomes:**
- GitHub stars: 500+ → 1,000+ (TUI visibility boost)
- Active users: 100+ → 500+ (accessibility improvement via TUI)
- Contributors: 5 → 15+ (documentation quality enables contributions)
- Blog/conference mentions: Increase due to TUI demo-ability

---

## 10. Conclusion

Phase 6 represents a transformative milestone for ProRT-IP, elevating it from a capable network scanner to a **production-ready, user-friendly security tool** that competes with industry leaders.

### 10.1 Key Achievements

**Technical Excellence:**
- 8 sprints delivering TUI + 5 Quick Wins + 3 Medium Impact enhancements
- 115-154 hours of focused development over 12-13 weeks
- 160-200 new tests, maintaining >60% coverage
- 6,500-8,000 lines of professional documentation

**Performance Gains:**
- 35-70% combined throughput increase (Quick Wins stacked)
- 20-50% memory reduction for large scans
- 10-25% NUMA optimization on enterprise hardware

**User Experience:**
- Modern TUI with 60 FPS real-time updates
- Interactive target selection and conflict resolution
- 8 scan templates (reduce setup time 50%)
- Keyboard-only navigation, colorblind-friendly themes

### 10.2 Strategic Value

**Competitive Position:** #2 overall in network scanning (behind only Masscan in raw speed)

**Market Differentiation:**
- Only modern scanner with production-ready TUI (Nmap, Masscan, ZMap, RustScan, Naabu all CLI-only)
- Best-in-class detection (service 85-90%, OS fingerprinting, TLS certs, idle scan)
- Enterprise-ready performance (NUMA, adaptive batching, syscall optimization)

**Foundation for Future:**
- EventBus architecture (Sprint 5.5.3) enables Phase 7 Web Interface
- Plugin system (Sprint 5.8) enables community ecosystem
- Documentation quality enables open source growth

### 10.3 Execution Confidence

**Risk Mitigation:**
- Hybrid approach balances ambition with pragmatism
- Parallel opportunities documented for team scaling
- Checkpoint decisions enable course correction
- Contingency plans for timeline pressures

**Success Probability: HIGH**
- EventBus foundation already proven (Sprint 5.5.3, 40ns latency)
- Quick Wins based on proven techniques (Masscan, ZMap, RustScan)
- Historical sprint completion rates strong (Sprints 5.1-5.10 all delivered on schedule)
- Testing strategy comprehensive (4 levels, 10 benchmark scenarios)

### 10.4 Next Steps

**Immediate (Post-Planning):**
1. Review Phase 6 plan with stakeholders (if applicable)
2. Prepare development environment (install ratatui, crossterm, hwlocality)
3. Create Sprint 6.1 TODO file (detailed task breakdown)
4. Schedule Sprint 6.1 kickoff (target: April 1, 2026)

**During Phase 6:**
1. Execute sprints sequentially (or parallel if team expands)
2. Weekly progress check-ins (assess timeline adherence)
3. Checkpoint decision at Sprint 6.6 (proceed with 6.7 or defer?)
4. Community engagement (beta testers for Sprint 6.8 UAT)

**Post-Phase 6:**
1. v0.6.0 release announcement (blog, Reddit, HN, Twitter/X)
2. Conference presentations (Black Hat USA, DEF CON, etc.)
3. Phase 7 planning (Web Interface, Q3-Q4 2026)
4. Community growth (expand contributor base, plugin ecosystem)

---

**Phase 6 Status:** PLANNED  
**Ready for Execution:** YES  
**Confidence Level:** HIGH  
**Expected Outcome:** Production-ready TUI + significant performance gains, positioning ProRT-IP as #2 in network scanning landscape

**Let's build the future of network scanning! 🚀**

---

## Appendices

### Appendix A: Quick Wins Summary

| ID | Quick Win | Effort (h) | Gain | ROI | Sprint |
|----|-----------|-----------|------|-----|--------|
| QW-1 | Adaptive Batch Size Tuning | 6-8 | 15-30% throughput | 5.33 | 6.4 |
| QW-2 | sendmmsg/recvmmsg Batching | 10-15 | 20-40% throughput | 4.00 | 6.3 |
| QW-3 | Memory-Mapped Streaming | 8-12 | 20-50% memory | 3.75 | 6.4, 6.6 |
| QW-4 | IP Deduplication | 4-6 | 30-70% scan reduction | 3.50 | 6.3 |
| QW-5 | Scan Preset Templates | 4-6 | UX improvement | 3.33 | 6.5 |
| **TOTAL** | **All Quick Wins** | **32-47h** | **35-70% combined** | **4.18 avg** | **6.3-6.6** |

### Appendix B: Medium Impact Summary

| ID | Enhancement | Effort (h) | Gain | ROI | Sprint | Notes |
|----|-------------|-----------|------|-----|--------|-------|
| MI-1 | Event-Driven TUI Prep | ~0 | Enables Phase 6 | 2.75 | 5.5.3 | **Already complete!** |
| MI-2 | Real-Time Progress | ~2-3 | Enhanced UX | 2.67 | 6.2 | Mostly done via EventBus |
| MI-3 | CDN/WAF Detection | 6-8 | Reduce false positives | 2.33 | 6.7 | Security research value |
| MI-4 | NSE Script Compatibility | 24-36 | Ecosystem | 2.17 | Phase 7 | **Deferred** |
| MI-5 | NUMA Optimization | 12-16 | 10-25% multi-socket | 2.00 | 6.7 | Enterprise hardware |
| **TOTAL** | **Integrated in Phase 6** | **18-24h** | **Varies** | **2.33 avg** | **6.2, 6.7** | MI-4 deferred |

### Appendix C: Sprint Dependencies Graph

```
Sprint 6.1 (TUI Framework)
    ↓
Sprint 6.2 (Live Dashboard)
    ↓
Sprint 6.5 (Interactive Selection) ←──────────┐
    ↓                                          │
Sprint 6.6 (Advanced Features)                 │
    ↓                                          │
Sprint 6.8 (Documentation + Release)           │
    ↑                                          │
    │                                          │
Sprint 6.3 (Network Optimization) ────────────┘
    ↓
Sprint 6.4 (Adaptive Tuning)
    ↓
Sprint 6.7 (NUMA + CDN)
    ↓
Sprint 6.8 (Documentation + Release)
```

**Legend:**
- `↓` = Sequential dependency (must complete before next)
- `→` = Soft dependency (beneficial but not required)
- Parallel paths can run simultaneously if team size permits

### Appendix D: Glossary

- **EventBus:** Central publish-subscribe hub for scan events (Sprint 5.5.3)
- **ProgressAggregator:** Real-time metrics collector (ETA, throughput, completion %)
- **Quick Win (QW):** High-ROI improvement (Tier 1, ROI > 3.0)
- **Medium Impact (MI):** Strategic enhancement (Tier 2, ROI 2.0-3.0)
- **TUI:** Terminal User Interface (ratatui-based)
- **NUMA:** Non-Uniform Memory Access (multi-socket CPU optimization)
- **CDN:** Content Delivery Network (Cloudflare, Akamai, etc.)
- **sendmmsg/recvmmsg:** Linux syscalls for batch packet I/O
- **ROI:** Return on Investment ((Impact × Strategic Value) / (Effort × Risk))
- **UAT:** User Acceptance Testing (external testers, Sprint 6.8)

---

**Document Version:** 1.0.0  
**Total Lines:** 2,087  
**Word Count:** ~18,500  
**Author:** Claude Code (Anthropic Sonnet 4.5)  
**Planning Session:** 2025-11-09  
**Status:** READY FOR EXECUTION
