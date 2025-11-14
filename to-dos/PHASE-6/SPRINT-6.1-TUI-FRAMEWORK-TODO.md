# Sprint 6.1: TUI Framework & Event Integration - TODO

---
**STATUS:** ✅ COMPLETE (2025-11-14)
**COMMIT:** 9bf9da0 (feat(tui): Sprint 6.1 TUI Framework COMPLETE + Documentation Updates)
**TESTS:** 2,175 passing (71 new TUI tests: 56 unit + 15 integration)
**FILES:** 30 files modified/added (6 modified, 24 new)
**DURATION:** ~40 hours actual (vs 15-20h estimated)
---

**Sprint:** 6.1 of 8 (Phase 6)
**Focus:** TUI Framework Setup & EventBus Integration
**Effort:** 15-20 hours (actual: ~40h)
**Timeline:** Completed 2025-11-14 (originally planned April 1-14, 2026)
**Dependencies:** EventBus from Sprint 5.5.3 ✅
**Status:** ✅ COMPLETE

---

## Sprint Overview

Establish the foundational TUI infrastructure using ratatui + crossterm, integrate with EventBus from Sprint 5.5.3, and implement basic rendering with keyboard navigation. This sprint creates the stable foundation that all subsequent TUI features (Sprints 6.2, 6.5, 6.6) will build upon.

**Key Deliverables:**
- prtip-tui crate (new workspace member)
- EventSubscriber implementation for TUI App
- 60 FPS rendering loop with async event handling
- Basic keyboard navigation (Tab, hjkl, q)
- 25-30 tests + TUI-ARCHITECTURE.md (500 lines)

---

## Task Breakdown

### Task Area 1: Framework Setup (4-5h)

**Tasks 1-5: Dependency Management & Project Structure**
- [ ] **Task 1.1:** Add dependencies to Cargo.toml
  - ratatui = "0.29+" (TUI framework)
  - crossterm = "0.28+" (cross-platform terminal)
  - tui-input = "0.10+" (text input widget)
  - tokio = { version = "1.35+", features = ["full"] } (already have)
- [ ] **Task 1.2:** Create prtip-tui crate in workspace
  - Run: `cargo new --lib prtip-tui`
  - Update workspace Cargo.toml: `members = [..., "prtip-tui"]`
  - Setup module structure: app.rs, ui.rs, events.rs, widgets/
- [ ] **Task 1.3:** Configure terminal initialization
  - Implement terminal setup (raw mode, alternate screen, hide cursor)
  - Add panic hook for terminal restoration (critical for crashes!)
  - Platform-specific setup (Windows ENABLE_VIRTUAL_TERMINAL_PROCESSING)
- [ ] **Task 1.4:** Implement graceful shutdown
  - Ctrl+C handler (restore terminal before exit)
  - Panic handler (ensure terminal restored even on panic)
  - Normal exit path (q key → restore → exit)
- [ ] **Task 1.5:** Cross-platform terminal setup
  - Test on Linux (primary development platform)
  - Document Windows quirks (cmd.exe vs Windows Terminal)
  - Document macOS quirks (iTerm2 vs Terminal.app)

**Success Criteria:**
- ✅ TUI launches without errors on Linux
- ✅ Terminal restores correctly on exit (no broken terminal state)
- ✅ Panic handler tested (force panic, verify restoration)

---

### Task Area 2: App State & Architecture (3-4h)

**Tasks 6-10: State Management & Event Loop**
- [ ] **Task 2.1:** Define App struct with event-carried state pattern
  ```rust
  pub struct App {
      scan_state: Arc<RwLock<ScanState>>,  // Shared with scanner
      ui_state: UIState,                     // Local to TUI
      event_bus: Arc<EventBus>,              // From Sprint 5.5.3
      should_quit: bool,
  }
  ```
- [ ] **Task 2.2:** Implement ScanState (shared state)
  - Arc<RwLock<>> for thread-safe access
  - Fields: targets, discovered_ports, progress_metrics, scan_status
  - Integration with ProgressAggregator from Sprint 5.5.3
- [ ] **Task 2.3:** Create UIState (local ephemeral state)
  - Fields: selected_pane, cursor_position, scroll_offset, input_buffer
  - No locking needed (single-threaded TUI rendering)
- [ ] **Task 2.4:** Design Component trait for modular widgets
  ```rust
  pub trait Component {
      fn render(&self, f: &mut Frame, area: Rect, state: &UIState);
      fn handle_event(&mut self, event: Event) -> bool;
  }
  ```
- [ ] **Task 2.5:** Setup async event loop with tokio::select!
  ```rust
  loop {
      tokio::select! {
          Some(event) = event_stream.next() => {
              // Handle keyboard/mouse input
          }
          Some(scan_event) = event_bus.subscribe() => {
              // Handle EventBus updates
          }
          _ = tick_interval.tick() => {
              // Render at 60 FPS (16ms interval)
          }
      }
  }
  ```

**Success Criteria:**
- ✅ App struct compiles with all fields
- ✅ State pattern allows scanner → TUI communication
- ✅ Event loop handles keyboard, EventBus, timer concurrently

---

### Task Area 3: EventBus Integration (4-5h)

**Tasks 11-15: Subscription & Event Handling**
- [ ] **Task 3.1:** Implement EventSubscriber trait for TUI App
  - Follow Sprint 5.5.3 EventBus API
  - Subscribe on TUI initialization
  - Unsubscribe on TUI shutdown (cleanup)
- [ ] **Task 3.2:** Subscribe to critical event types
  - ScanStarted: Initialize UI, show scan metadata
  - ScanCompleted: Show final statistics, enable review mode
  - PortFound: Add port to table, trigger table re-render
  - ServiceDetected: Update port entry with service info
  - ProgressUpdate: Update progress bar, ETA, throughput
  - ErrorOccurred: Show error toast/notification
- [ ] **Task 3.3:** Create event handler dispatch
  ```rust
  match event {
      Event::ScanStarted { .. } => { /* Update UI */ }
      Event::PortFound { port, .. } => { /* Add to table */ }
      Event::ProgressUpdate { .. } => { /* Update progress */ }
      // ... handle all 18 event variants
  }
  ```
- [ ] **Task 3.4:** Add event rate limiting (prevent UI overload)
  - Aggregate events: 100 PortFound → single "100 ports" update
  - Sample high-frequency events (take 1 every 16ms for 60 FPS)
  - Drop events if queue exceeds threshold (prevent memory growth)
- [ ] **Task 3.5:** Test with mock EventBus
  - Create test harness: publish 10K events/sec
  - Verify no dropped frames (measure FPS, ensure ≥60)
  - Verify event aggregation works (check table update count)

**Success Criteria:**
- ✅ TUI receives events from EventBus (verified with logs)
- ✅ Event handlers update UI state correctly
- ✅ Handles 10K events/sec without lag (60 FPS maintained)

---

### Task Area 4: Basic Rendering (3-4h)

**Tasks 16-20: Rendering Loop & UI Layout**
- [ ] **Task 4.1:** Implement 60 FPS render loop
  - Calculate frame budget: 16ms per frame (1000ms / 60)
  - Measure render time with Instant::now()
  - Skip frame if behind schedule (don't block event handling)
- [ ] **Task 4.2:** Create basic layout
  - Header: Scan target, start time, scan type
  - Status bar: ETA, completion %, throughput
  - Main area: Placeholder for port table (Sprint 6.2)
  - Footer: Help text (keybindings)
- [ ] **Task 4.3:** Implement keyboard navigation
  - Tab/Shift+Tab: Cycle through panes
  - hjkl: Vim-style navigation (up/down/left/right)
  - q: Quit (with confirmation if scan running)
  - ?: Show detailed help screen
- [ ] **Task 4.4:** Add help text footer
  - Always visible: "q: Quit | ?: Help | Tab: Next Pane"
  - Context-sensitive: Show relevant keys for active pane
- [ ] **Task 4.5:** Colorize output
  - Status codes: Green=open, Red=closed, Yellow=filtered, Blue=TLS
  - Use ratatui Color enum: Color::Green, Color::Red, etc.
  - Support 256-color and true-color terminals (detect capability)

**Success Criteria:**
- ✅ Renders at 60 FPS (measure with FPS counter)
- ✅ Layout displays correctly on 80×24 minimum terminal
- ✅ Keyboard navigation works (Tab cycles panes, q quits)
- ✅ Colors display correctly (test on multiple terminals)

---

### Task Area 5: Testing & Documentation (1-2h)

**Tasks 21-25: Quality Assurance**
- [ ] **Task 5.1:** Unit tests - App state transitions (15 tests)
  - Test state initialization
  - Test quit request (should_quit flag)
  - Test pane selection (cycle through panes)
  - Test event handling (mock events, verify state changes)
  - Test edge cases (empty state, invalid transitions)
- [ ] **Task 5.2:** Integration test - TUI launch & EventBus (10 tests)
  - Test: Launch TUI → verify terminal raw mode
  - Test: Send EventBus events → verify UI updates
  - Test: Press keys → verify event handling
  - Test: Graceful shutdown → verify terminal restoration
  - Test: Panic scenario → verify terminal restored
- [ ] **Task 5.3:** Create docs/TUI-ARCHITECTURE.md (500 lines)
  - Section 1: Overview (TUI goals, design principles)
  - Section 2: Component Diagram (App, Widgets, EventBus)
  - Section 3: Data Flow (Scanner → EventBus → TUI → Display)
  - Section 4: State Management (ScanState vs UIState)
  - Section 5: Event Loop (tokio::select! pattern)
  - Section 6: Rendering Pipeline (60 FPS, widget tree)
  - Section 7: Extension Points (Component trait, custom widgets)
- [ ] **Task 5.4:** Document EventBus subscription pattern
  - Code example: Implement EventSubscriber
  - Code example: Handle events in TUI
  - Integration guide for new event types
- [ ] **Task 5.5:** Rustdoc for all public TUI API
  - Document App struct (fields, methods)
  - Document Component trait (render, handle_event)
  - Document UIState (purpose, fields)
  - Cross-reference to TUI-ARCHITECTURE.md

**Success Criteria:**
- ✅ 25-30 tests passing (cargo test --package prtip-tui)
- ✅ 0 clippy warnings (cargo clippy --package prtip-tui)
- ✅ 0 rustfmt violations (cargo fmt --check)
- ✅ TUI-ARCHITECTURE.md complete (500 lines)
- ✅ All public APIs documented (rustdoc builds without warnings)

---

## Definition of Done

Sprint 6.1 is complete when ALL of the following are true:

### Functional Requirements
- [x] TUI launches successfully on Linux (primary development platform)
- [x] Handles 1K events/sec from EventBus without lag or dropped frames
- [x] Keyboard navigation works (Tab, Shift+Tab, hjkl, q)
- [x] Terminal restores correctly on all exit paths (normal, Ctrl+C, panic)
- [x] EventSubscriber implementation receives events from EventBus

### Quality Requirements
- [x] 25-30 tests passing (0 failures)
- [x] 0 clippy warnings (strict mode)
- [x] cargo fmt clean (100% formatted)
- [x] All public APIs documented (rustdoc 0 warnings)

### Documentation Requirements
- [x] docs/TUI-ARCHITECTURE.md complete (500 lines minimum)
- [x] EventBus subscription pattern documented with code examples
- [x] Rustdoc for App, Component, UIState complete

### Integration Requirements
- [x] prtip-tui crate added to workspace (compiles successfully)
- [x] EventBus integration tested (mock EventBus publishes, TUI receives)
- [x] 60 FPS rendering validated (measured with timer)

---

## Testing Plan

### Unit Tests (15 tests)
1. App initialization (default state)
2. Quit request (should_quit flag)
3. Pane selection (cycle through panes with Tab)
4. Event handling (mock ScanStarted event)
5. Event handling (mock PortFound event)
6. Event handling (mock ProgressUpdate event)
7. Event aggregation (100 events → 1 update)
8. UIState transitions (selected_pane changes)
9. ScanState access (Arc<RwLock<>> locking)
10. Error handling (invalid event types)
11. Component trait (mock widget render)
12. Component trait (mock widget handle_event)
13. Event rate limiting (drop events beyond threshold)
14. Frame skipping (render budget exceeded)
15. Terminal restoration (cleanup on drop)

### Integration Tests (10 tests)
1. TUI launch (terminal enters raw mode)
2. TUI shutdown (terminal restored)
3. EventBus subscription (receive ScanStarted)
4. EventBus subscription (receive 1K events)
5. Keyboard input (press 'q', verify quit)
6. Keyboard input (press Tab, verify pane change)
7. Panic scenario (force panic, verify terminal restored)
8. Concurrent events (EventBus + keyboard simultaneously)
9. High-load scenario (10K events/sec)
10. Cross-platform (Linux + macOS + Windows if available)

### Manual Testing Checklist
- [ ] Launch TUI on GNOME Terminal (Linux)
- [ ] Launch TUI on Alacritty (Linux)
- [ ] Launch TUI on iTerm2 (macOS, if available)
- [ ] Launch TUI on Windows Terminal (Windows, if available)
- [ ] Verify colors display correctly (all 4 terminals)
- [ ] Press all navigation keys (Tab, Shift+Tab, hjkl, q, ?)
- [ ] Force crash (panic!), verify terminal restored
- [ ] Resize terminal window, verify layout adapts
- [ ] Small terminal (80×24), verify no rendering errors
- [ ] Large terminal (200×50), verify layout scales

---

## Dependencies

**External Crates:**
- ratatui 0.29+ (TUI framework, MIT license)
- crossterm 0.28+ (terminal backend, MIT license)
- tui-input 0.10+ (text input widget, MIT license)
- tokio 1.35+ (async runtime, already have)

**Internal Dependencies:**
- prtip-core (EventBus from Sprint 5.5.3)
- prtip-scanner (ScanState, ProgressAggregator from Sprint 5.5.3)

**No Blockers:** This sprint has no dependencies on other Phase 6 sprints (clean start).

---

## Risk Mitigation

**Risk 1: EventBus Overload (10K events/sec)**
- **Mitigation:** Event aggregation layer (batch 100 events)
- **Testing:** Stress test with mock EventBus publishing 10K/sec
- **Fallback:** Reduce EventBus publish rate if TUI can't keep up

**Risk 2: Terminal Compatibility Issues**
- **Mitigation:** Use crossterm abstraction (handles platform differences)
- **Testing:** Test on 4+ terminals (GNOME, Alacritty, iTerm2, Windows Terminal)
- **Fallback:** Document known issues, recommend supported terminals

**Risk 3: Rendering Performance**
- **Mitigation:** Profile with `perf`, optimize widget tree
- **Testing:** Measure FPS with timer, ensure ≥60 FPS
- **Fallback:** Reduce render complexity if performance insufficient

---

## Resources

**Documentation:**
- Ratatui Book: https://ratatui.rs/
- Crossterm Docs: https://docs.rs/crossterm/
- Tokio Select: https://tokio.rs/tokio/tutorial/select

**Reference Implementations:**
- RustScan TUI (simple macros, ~100 lines)
- Spotify TUI (complex layout, good example)
- Helix Editor (advanced TUI patterns)

**ProRT-IP Docs:**
- docs/35-EVENT-SYSTEM-GUIDE.md (Sprint 5.5.3 EventBus)
- to-dos/PHASE-6-TUI-INTERFACE.md (this sprint in context)

---

## Sprint Completion Report Template

**When Sprint 6.1 is complete, create: `SPRINT-6.1-COMPLETE.md`**

Include:
1. Executive summary (1 paragraph, key achievements)
2. Effort spent (actual hours vs estimate 15-20h)
3. Tasks completed (25/25 = 100%)
4. Tests added (actual count vs 25-30 target)
5. Documentation created (line counts)
6. Challenges encountered (what was harder than expected?)
7. Lessons learned (what would we do differently?)
8. Next sprint readiness (is foundation stable for Sprint 6.2?)
9. Screenshots/ASCII art (if available, show TUI running)
10. Performance metrics (FPS, event throughput, memory usage)

---

**Status:** PLANNED  
**Ready to Start:** April 1, 2026  
**Estimated Completion:** April 14, 2026  
**Sprint Owner:** TBD  

**This sprint establishes the foundation for all TUI features in Phase 6. Quality and stability are critical - do not rush to meet deadline if foundation is shaky.**
