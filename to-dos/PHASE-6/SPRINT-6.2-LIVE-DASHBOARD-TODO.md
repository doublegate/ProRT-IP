# Sprint 6.2: Live Dashboard & Real-Time Display

**Status:** 🔄 IN PROGRESS (Tasks 2.1-2.4 COMPLETE - 4/6 tasks, ~60%)
**Effort Estimate:** 12-18 hours (Actual: ~18h so far)
**Timeline:** Weeks 3-4 (2 weeks)
**Dependencies:** Sprint 6.1 (TUI Framework) COMPLETE ✅
**Priority:** HIGH (Critical Path)

## Progress Update (2025-11-14)

**Completed Tasks**:
- ✅ Task 2.1: PortTableWidget (interactive port list with sorting/filtering)
- ✅ Task 2.2: Event Handling Infrastructure (keyboard navigation, Tab switching)
- ✅ Task 2.3: ServiceTableWidget (interactive service list with sorting/filtering)
- ✅ Task 2.4: MetricsDashboardWidget (real-time metrics with 3-column layout)

**Implementation Details**:
- 3-tab dashboard system (Port Table | Service Table | Metrics)
- Tab/Shift+Tab navigation between widgets
- Real-time metrics display (progress, throughput, statistics)
- 5-second rolling average calculations
- Human-readable formatting (durations, numbers, throughput)
- Color-coded status indicators
- All 165 tests passing (140 unit + 25 integration)
- Zero clippy warnings
- Clean formatting

**Files Modified**: 6 files (~800 lines total)
- Created: `widgets/metrics_dashboard.rs` (~740 lines, 24 tests)
- Modified: `widgets/mod.rs`, `state/ui_state.rs`, `ui/renderer.rs`, `events/loop.rs`, `tests/integration_test.rs`

**Next Steps**: Tasks 2.5-2.6 (Network Activity Graph, Final Integration Testing)

## Sprint Overview

### Deliverables
1. **Real-Time Progress Widget** - Live scan statistics with EventBus integration
2. **Port Discovery Table** - Streaming port results with filtering/sorting
3. **Network Activity Graph** - Historical throughput visualization
4. **Service Detection Panel** - Real-time service identification display
5. **Status Bar Enhancements** - Multi-line context with keyboard shortcuts

### Strategic Value
- Enables operators to monitor scan progress without blocking terminal
- Provides actionable insights during long-running scans
- Differentiates ProRT-IP from tools with limited feedback (ZMap, Masscan)
- Foundation for advanced features (export, pause/resume)

### Integration Points
- **EventBus (Sprint 5.5.3):** Subscribe to scan progress, port discoveries, service detections
- **ProgressAggregator:** Real-time metrics (pps, completion %, ETA)
- **ScanState (Arc<RwLock>):** Shared scan results between scanner and TUI

---

## Task Breakdown

### Task Area 1: Progress Widget (3-4 hours)

**Task 1.1: Create ProgressWidget component**
- File: `prtip-tui/src/widgets/progress.rs`
- Implements `ratatui::widgets::Widget` trait
- Display metrics: targets scanned, ports found, packets sent, current pps
- Uses `ratatui::widgets::Gauge` for progress bar (0-100%)
- Color coding: green (on-track), yellow (slow), red (stalled)
- **Estimated Time:** 1h

**Task 1.2: Integrate ProgressAggregator from EventBus**
```rust
// Subscribe to progress events
let mut progress_rx = app.event_bus.subscribe_typed::<ProgressUpdateEvent>();

// In event loop
tokio::select! {
    Some(progress) = progress_rx.recv() => {
        app.ui_state.progress = progress.aggregator.snapshot();
        // Update ETA calculation
        app.ui_state.eta = calculate_eta(&progress);
    }
}
```
- Calculate ETA based on current pps and remaining targets
- Handle edge cases: zero pps, paused scans, rate limiting
- **Estimated Time:** 1.5h

**Task 1.3: Add throughput sparkline graph**
- Use `ratatui::widgets::Sparkline` for compact historical view
- Track last 60 seconds of pps measurements (1 sample/sec)
- Ring buffer implementation: `VecDeque<u64>` capped at 60
- Display max/min/avg throughput in widget header
- **Estimated Time:** 1h

**Task 1.4: Implement auto-refresh logic**
- Refresh rate: 250ms for progress (4 FPS), 1s for ETA
- Use `tokio::time::interval` for scheduled updates
- Debounce rapid EventBus updates to prevent flicker
- **Estimated Time:** 0.5h

**Task 1.5: Write unit tests**
- Test progress percentage calculation (0%, 50%, 100%)
- Test ETA calculation with various pps rates
- Test sparkline ring buffer wrap-around
- Test color coding thresholds
- **Target:** 8-10 tests
- **Estimated Time:** 1h

---

### Task Area 2: Port Discovery Table (4-5 hours)

**Task 2.1: Design PortTableWidget with streaming support**
- File: `prtip-tui/src/widgets/port_table.rs`
- Use `ratatui::widgets::Table` with custom `Row` rendering
- Columns: IP Address, Port, Protocol, State, Service, Banner (truncated)
- Pagination: Display 20 rows at a time, keyboard navigation (PgUp/PgDn)
- **Estimated Time:** 1.5h

**Task 2.2: Implement EventBus streaming**
```rust
let mut port_rx = app.event_bus.subscribe_typed::<PortDiscoveryEvent>();

tokio::select! {
    Some(discovery) = port_rx.recv() => {
        app.ui_state.port_table.add_row(PortRow {
            ip: discovery.target,
            port: discovery.port,
            protocol: discovery.protocol,
            state: discovery.state,
            service: discovery.service,
            banner: discovery.banner.map(|b| truncate(&b, 40)),
        });
        // Auto-scroll to bottom if user hasn't manually scrolled
        if app.ui_state.port_table.is_auto_scroll {
            app.ui_state.port_table.scroll_to_bottom();
        }
    }
}
```
- Handle high-frequency updates (1K-10K ports/sec possible)
- Rate-limit table rendering to 60 FPS (aggregate updates between frames)
- **Estimated Time:** 2h

**Task 2.3: Add filtering and sorting**
- Filter by: protocol (TCP/UDP), state (open/closed), service name regex
- Sort by: IP (lexicographic), port (numeric), service (alphabetic)
- Keyboard shortcuts: `f` (filter), `s` (sort), `/` (search)
- Display active filter in table header
- **Estimated Time:** 1.5h

**Task 2.4: Implement selection and detail view**
- Arrow keys navigate table rows
- `Enter` opens detail panel with full banner, service version, OS hints
- `Esc` closes detail panel
- Highlight selected row with distinct color
- **Estimated Time:** 1h

**Task 2.5: Write integration tests**
- Test streaming 1000 port discoveries
- Test filtering (TCP-only, open-only)
- Test sorting (by port ascending/descending)
- Test pagination and scrolling
- **Target:** 10-12 tests
- **Estimated Time:** 1h

---

### Task Area 3: Network Activity Graph (2-3 hours)

**Task 3.1: Create NetworkGraphWidget**
- File: `prtip-tui/src/widgets/network_graph.rs`
- Use `ratatui::widgets::Chart` for line graph
- X-axis: Time (last 60 seconds)
- Y-axis: Packets per second (log scale for wide range)
- Multiple series: packets sent, packets received, ports discovered
- **Estimated Time:** 1.5h

**Task 3.2: Collect time-series metrics from EventBus**
```rust
struct NetworkMetrics {
    timestamps: VecDeque<Instant>,
    packets_sent: VecDeque<u64>,
    packets_received: VecDeque<u64>,
    ports_discovered: VecDeque<u64>,
}

// Subscribe to throughput events
let mut throughput_rx = app.event_bus.subscribe_typed::<ThroughputEvent>();

// Sample every second
let mut sample_interval = tokio::time::interval(Duration::from_secs(1));
tokio::select! {
    _ = sample_interval.tick() => {
        let snapshot = throughput_rx.recv().await?;
        metrics.add_sample(snapshot);
    }
}
```
- Ring buffer: 60 samples (last minute)
- Calculate derivative for "ports/sec" from cumulative count
- **Estimated Time:** 1.5h

**Task 3.3: Write unit tests**
- Test ring buffer management
- Test metric calculations (pps, derivative)
- Test chart rendering with edge cases (zero data, single data point)
- **Target:** 5-6 tests
- **Estimated Time:** 0.5h

---

### Task Area 4: Service Detection Panel (2-3 hours)

**Task 4.1: Design ServicePanelWidget**
- File: `prtip-tui/src/widgets/service_panel.rs`
- Display recent service detections (last 10)
- Format: `192.168.1.100:80 - HTTP (Apache/2.4.52)`
- Highlight high-value services: SSH, RDP, databases, web servers
- Color coding: green (HTTP/S), yellow (SSH/Telnet), red (databases)
- **Estimated Time:** 1h

**Task 4.2: Subscribe to ServiceDetectionEvent**
```rust
let mut service_rx = app.event_bus.subscribe_typed::<ServiceDetectionEvent>();

tokio::select! {
    Some(detection) = service_rx.recv() => {
        app.ui_state.service_panel.add_detection(ServiceItem {
            target: detection.target,
            port: detection.port,
            service: detection.service_name,
            version: detection.version,
            confidence: detection.confidence,
        });
    }
}
```
- Maintain scrolling list (FIFO, max 50 items)
- Auto-highlight critical services (database ports 3306, 5432, 1433)
- **Estimated Time:** 1h

**Task 4.3: Add detection statistics summary**
- Display counts: total services detected, unique service types, high-confidence detections
- Detection rate: services/minute
- Most common services: top 3 by count
- **Estimated Time:** 0.5h

**Task 4.4: Write unit tests**
- Test service categorization (web, ssh, database)
- Test FIFO eviction (max 50 items)
- Test color coding logic
- **Target:** 5-6 tests
- **Estimated Time:** 0.5h

---

### Task Area 5: Status Bar Enhancements (1-2 hours)

**Task 5.1: Create multi-line status bar**
- File: `prtip-tui/src/widgets/status_bar.rs`
- Line 1: Scan mode, targets, port range, timing template
- Line 2: Active keyboard shortcuts (context-sensitive)
- Line 3: Error/warning messages (if any)
- Use `ratatui::layout::Constraint` for proportional spacing
- **Estimated Time:** 0.5h

**Task 5.2: Add keyboard shortcut help**
- Global shortcuts: `q` (quit), `p` (pause), `r` (resume), `?` (help)
- Context shortcuts: In table (`f` filter, `s` sort), In graph (`z` zoom)
- Display relevant shortcuts based on active widget
- **Estimated Time:** 0.5h

**Task 5.3: Implement status message queue**
- Success messages (green): "Scan started", "Export complete"
- Warning messages (yellow): "Rate limit active", "Low privilege mode"
- Error messages (red): "Network unreachable", "Permission denied"
- Auto-clear messages after 5 seconds
- **Estimated Time:** 0.5h

**Task 5.4: Write unit tests**
- Test message priority (error > warning > info)
- Test auto-clear timeout
- Test keyboard shortcut context switching
- **Target:** 4-5 tests
- **Estimated Time:** 0.5h

---

## Definition of Done

### Functional Requirements
- [ ] Progress widget updates in real-time (<1s latency from EventBus)
- [ ] Port table displays streaming results with filtering/sorting
- [ ] Network graph visualizes last 60 seconds of activity
- [ ] Service panel highlights critical detections
- [ ] Status bar provides contextual help and error messages
- [ ] All widgets responsive to keyboard input
- [ ] No UI freezing during high-throughput scans (10K+ pps)

### Quality Requirements
- [ ] 30-35 tests passing (100% success rate)
- [ ] Zero clippy warnings (--deny warnings)
- [ ] Zero rustdoc warnings
- [ ] Code formatted with `cargo fmt`
- [ ] No panics in widget rendering (graceful error handling)
- [ ] Memory usage stable during 10-minute scan test

### Documentation Requirements
- [ ] Rustdoc comments for all public widget APIs
- [ ] Widget usage examples in doc comments
- [ ] Architecture diagram: EventBus → Widgets data flow
- [ ] Update `docs/36-TUI-ARCHITECTURE.md` with widget details

### Integration Requirements
- [ ] EventBus subscriptions working for all event types
- [ ] Shared state (Arc<RwLock<ScanState>>) accessed safely
- [ ] No race conditions in concurrent rendering + event handling
- [ ] Graceful fallback if EventBus unavailable (e.g., non-TUI mode)

---

## Testing Plan

### Unit Tests (15-18 tests)
```bash
# Run widget tests
cargo test -p prtip-tui widgets::

# Test coverage
cargo tarpaulin -p prtip-tui --out Html
```

**Test Cases:**
1. ProgressWidget: Calculate ETA correctly (various pps rates)
2. ProgressWidget: Color coding thresholds (on-track/slow/stalled)
3. ProgressWidget: Sparkline ring buffer wrap-around
4. PortTableWidget: Streaming 1000 rows
5. PortTableWidget: Filtering by protocol (TCP/UDP)
6. PortTableWidget: Sorting by port (ascending/descending)
7. PortTableWidget: Pagination (PgUp/PgDn navigation)
8. NetworkGraphWidget: Ring buffer management (60 samples)
9. NetworkGraphWidget: Derivative calculation (ports/sec)
10. ServicePanelWidget: Service categorization (web/ssh/database)
11. ServicePanelWidget: FIFO eviction (max 50 items)
12. ServicePanelWidget: Color coding (green/yellow/red)
13. StatusBar: Message priority (error > warning > info)
14. StatusBar: Auto-clear timeout (5 seconds)
15. StatusBar: Keyboard shortcut context switching

### Integration Tests (12-15 tests)
```bash
# Run full TUI integration tests
cargo test -p prtip-tui --test integration_dashboard
```

**Test Cases:**
1. EventBus Integration: Progress events → ProgressWidget
2. EventBus Integration: Port discoveries → PortTableWidget
3. EventBus Integration: Service detections → ServicePanelWidget
4. EventBus Integration: Throughput events → NetworkGraphWidget
5. Concurrent Rendering: 10K port discoveries/sec + 60 FPS rendering
6. State Synchronization: ScanState updates reflected in all widgets
7. Keyboard Navigation: Arrow keys, PgUp/PgDn, Enter, Esc
8. Filtering: Apply TCP-only filter during active scan
9. Sorting: Sort by port while streaming new results
10. Memory Stability: 10-minute scan with 100K ports discovered
11. Error Handling: EventBus disconnection during scan
12. Graceful Degradation: Missing service detection data

### Manual Testing Checklist
- [ ] **Visual Inspection:** All widgets render correctly at 80x24 terminal
- [ ] **Responsiveness:** UI updates within 1 second of scan events
- [ ] **High Throughput:** No flicker or lag at 10K pps
- [ ] **Keyboard Shortcuts:** All shortcuts work as documented
- [ ] **Filtering:** Active filter persists across new results
- [ ] **Sorting:** Sort order maintained during streaming
- [ ] **Color Coding:** Services color-coded correctly (web/ssh/database)
- [ ] **Error Messages:** Clear error messages for common issues
- [ ] **Status Bar:** Context-sensitive help displayed correctly
- [ ] **Long-Running Scans:** Stable for 30+ minute scans

---

## Dependencies

### External Crates
- `ratatui = "0.29"` - TUI framework (widgets, layout)
- `crossterm = "0.28"` - Terminal backend (events, rendering)
- `tokio = "1.35"` - Async runtime
- `chrono = "0.4"` - Timestamp formatting for graphs

### Internal Dependencies
- **Sprint 6.1 (TUI Framework):** App state, event loop, terminal management
- **Sprint 5.5.3 (EventBus):** Progress, port discovery, service detection events
- **Sprint 5.5.3 (ProgressAggregator):** Real-time scan metrics
- **prtip-scanner:** ScanState (shared state), scan results

### Data Structures
- `Arc<RwLock<ScanState>>` - Shared scan state (thread-safe)
- `Arc<EventBus>` - Event distribution
- `UIState` - Local TUI state (widgets, selections, filters)

---

## Risk Mitigation

### Risk 1: High-Frequency Updates Cause UI Flicker
**Impact:** Medium | **Probability:** Medium
**Mitigation:**
- Rate-limit widget rendering to 60 FPS (aggregate EventBus updates)
- Use `tokio::select!` to batch events between render frames
- Test with 10K+ pps scans to validate smoothness

### Risk 2: EventBus Subscription Overhead
**Impact:** Low | **Probability:** Low
**Mitigation:**
- EventBus designed for low overhead (40ns publish, 340ns end-to-end)
- Use typed subscriptions to avoid unnecessary deserialization
- Monitor EventBus backpressure metrics during testing

### Risk 3: Memory Growth from Unbounded Port Table
**Impact:** High | **Probability:** Medium
**Mitigation:**
- Implement LRU eviction policy (max 10K rows in memory)
- Stream older results to disk if table exceeds threshold
- Add memory usage warning in status bar (>1GB table)
- Test with 100K+ port discovery scenario

### Risk 4: Terminal Resize During Rendering
**Impact:** Medium | **Probability:** Low
**Mitigation:**
- Handle `crossterm::event::Event::Resize` gracefully
- Recalculate layout constraints on resize
- Test with various terminal sizes (80x24 to 200x60)

---

## Resources

### Documentation
- **ratatui Widgets:** https://docs.rs/ratatui/0.29/ratatui/widgets/
- **EventBus Guide:** `docs/35-EVENT-SYSTEM-GUIDE.md`
- **TUI Architecture:** `docs/36-TUI-ARCHITECTURE.md` (Sprint 6.1)

### Reference Implementations
- **ratatui Examples:** https://github.com/ratatui/ratatui/tree/main/examples
- **ZMap Terminal Output:** Plain text, no interactivity
- **Nmap Terminal Output:** Periodic updates, no real-time widgets
- **RustScan TUI:** Basic progress bar (reference for what to improve)

### Performance Benchmarks
- **Target:** 60 FPS rendering (16ms frame time)
- **EventBus Latency:** 340ns end-to-end (Sprint 5.5.3 measured)
- **Table Rendering:** <5ms for 20 rows (measured in prototype)

---

## Sprint Completion Report Template

```markdown
# Sprint 6.2 Completion Report

**Date:** [YYYY-MM-DD]
**Actual Duration:** [X] hours
**Status:** ✅ COMPLETE / ⚠️ PARTIAL / ❌ INCOMPLETE

## Deliverables Status
- [ ] Progress Widget - Real-time metrics with ETA
- [ ] Port Discovery Table - Streaming with filtering/sorting
- [ ] Network Activity Graph - 60-second throughput visualization
- [ ] Service Detection Panel - Critical service highlighting
- [ ] Status Bar Enhancements - Multi-line with keyboard help

## Test Results
- Unit Tests: [X/15] passing
- Integration Tests: [X/12] passing
- Manual Testing: [X/10] checklist items verified

## Performance Metrics
- Rendering FPS: [X] FPS (target: 60)
- EventBus Latency: [X] ns (expected: <500ns)
- Memory Usage: [X] MB for 10K ports (target: <100MB)

## Issues Encountered
1. [Issue description] - **Resolution:** [How resolved]
2. [Issue description] - **Resolution:** [How resolved]

## Lessons Learned
- [Key insight from development]
- [Technical decision and rationale]

## Next Sprint Preparation
- Dependencies ready for Sprint 6.3: ✅/❌
- Outstanding technical debt: [List items]
- Recommendations for next sprint: [Suggestions]
```

---

**This sprint transforms ProRT-IP from a CLI-only tool to an interactive monitoring platform. Prioritize UX smoothness and real-time responsiveness - delays >1s will frustrate operators during time-critical scans.**
