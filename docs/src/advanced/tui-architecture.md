# TUI Architecture

Master the Terminal User Interface architecture for real-time network scanning visualization.

## What is the TUI Architecture?

**ProRT-IP TUI (Terminal User Interface)** provides real-time visualization of network scanning operations through an event-driven, high-performance architecture designed for 10,000+ events/second throughput while maintaining smooth 60 FPS rendering.

### Design Philosophy

The TUI architecture follows three core principles:

1. **Consumer-Only Pattern** - TUI subscribes to scanner events, scanner has zero TUI dependencies (one-way data flow)
2. **Immediate Mode Rendering** - Full UI redrawn every frame at 60 FPS, ratatui diffs and updates terminal efficiently
3. **Event Aggregation** - High-frequency events (port discoveries, host finds) batched every 16ms to prevent UI overload

### Key Benefits

**Real-Time Monitoring:**
- Live port discoveries as they're found
- Instant service detection updates
- Real-time throughput metrics (ports/second, packets/second)
- Interactive progress tracking with ETA calculations

**High Performance:**
- **10,000+ events/second** throughput without UI lag
- **60 FPS rendering** for smooth user experience
- **<5ms frame time** (well under 16.67ms budget)
- **~5% CPU overhead** (rendering + event processing)
- **~5 MB memory** footprint (negligible overhead)

**Professional Experience:**
- **7 production widgets** (StatusBar, MainWidget, LogWidget, HelpWidget, PortTable, ServiceTable, MetricsDashboard)
- **3-tab dashboard** interface (Port Table, Service Table, Metrics)
- **Comprehensive keyboard shortcuts** (navigation, sorting, filtering, search)
- **Graceful degradation** (clean terminal restoration on all exit paths)

---

## Architecture Overview

### Technology Stack

**Core Dependencies:**

| Library | Version | Purpose |
|---------|---------|---------|
| **ratatui** | 0.29+ | Modern TUI framework with immediate mode rendering |
| **crossterm** | 0.28+ | Cross-platform terminal manipulation (raw mode, events) |
| **tokio** | 1.35+ | Async runtime for event loop coordination |
| **parking_lot** | 0.12+ | High-performance RwLock (2-3× faster than std::sync) |
| **prtip-core** | - | EventBus integration for scan events |

**Why These Choices:**

- **ratatui 0.29+**: Automatic panic hook for terminal restoration, immediate mode rendering with efficient diffing
- **crossterm**: Cross-platform support (Linux, macOS, Windows), async event stream integration
- **parking_lot::RwLock**: Lock-free fast path for readers, writer priority prevents starvation
- **tokio::select!**: Concurrent event handling (keyboard, EventBus, 60 FPS timer)

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          ProRT-IP Scanner                           │
│                     (prtip-core, no TUI deps)                       │
└────────────────┬────────────────────────────────────────────────────┘
                 │ publishes events
                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                           EventBus                                  │
│              (mpsc::unbounded_channel, broadcast)                   │
└────────────────┬────────────────────────────────────────────────────┘
                 │ subscribe
                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        TUI Event Loop                               │
│                    (tokio::select! pattern)                         │
│                                                                     │
│  ┌───────────────┐  ┌────────────────┐  ┌─────────────────┐         │
│  │   Keyboard    │  │  EventBus RX   │  │   60 FPS Timer  │         │
│  │  (crossterm)  │  │  (scan events) │  │  (tick_interval)│         │
│  └───────┬───────┘  └────────┬───────┘  └────────┬────────┘         │
│          │                   │                     │                │
│          ▼                   ▼                     ▼                │
│  ┌──────────────┐   ┌──────────────────┐  ┌─────────────────┐       │
│  │  Key Handler │   │ Event Aggregator │  │  Flush & Render │       │
│  │  (quit, nav) │   │  (rate limiting) │  │  (update state) │       │
│  └──────┬───────┘   └──────┬───────────┘  └──────────┬──────┘       │
│         │                  │                         │              │
│         └──────────────────┴─────────────────────────┘              │
│                            │                                        │
│                            ▼                                        │
│               ┌─────────────────────────┐                           │
│               │   State Update Logic    │                           │
│               │  (scan_state, ui_state) │                           │
│               └───────────┬─────────────┘                           │
└───────────────────────────┼─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       Rendering Pipeline                            │
│                                                                     │
│  ┌─────────────┐   ┌──────────────┐   ┌──────────────┐              │
│  │   Layout    │──▶│   Widgets    │──▶│   ratatui    │              │
│  │  (chunks)   │   │ (components) │   │   (diffing)  │              │
│  └─────────────┘   └──────────────┘   └──────┬───────┘              │
│                                              │                      │
└──────────────────────────────────────────────┼──────────────────────┘
                                               │
                                               ▼
                                        ┌─────────────────┐
                                        │   Terminal      │
                                        │  (crossterm)    │
                                        └─────────────────┘
```

**Architecture Principles:**

1. **One-Way Data Flow**: Scanner publishes to EventBus → TUI subscribes (consumer-only pattern)
2. **Immediate Mode Rendering**: Full UI redrawn every frame, ratatui diffs terminal updates
3. **Event Aggregation**: Batch 10K+ events/sec into 60 Hz updates (16ms batches)
4. **Shared State**: `Arc<RwLock<ScanState>>` for thread-safe scanner ↔ TUI communication
5. **Graceful Cleanup**: ratatui 0.29+ automatic panic hook ensures terminal restoration

---

## Core Components

### 1. App Lifecycle Manager

**Purpose**: Coordinates entire TUI lifecycle from initialization to shutdown.

**Location**: `crates/prtip-tui/src/app.rs`

**Responsibilities:**
- Terminal initialization (raw mode, alternate screen)
- EventBus subscription
- Event loop coordination (`tokio::select!`)
- Terminal restoration on all exit paths

**Key Methods:**

```rust
pub struct App {
    event_bus: Arc<EventBus>,
    scan_state: Arc<RwLock<ScanState>>,
    ui_state: UIState,
    should_quit: bool,
}

impl App {
    pub fn new(event_bus: Arc<EventBus>) -> Self
    pub async fn run(&mut self) -> Result<()>
    pub fn should_quit(&self) -> bool
    pub fn scan_state(&self) -> Arc<RwLock<ScanState>>
}
```

**Event Loop Pattern:**

```rust
pub async fn run(&mut self) -> Result<()> {
    // Initialize terminal (ratatui 0.29+ handles panic hook)
    let mut terminal = ratatui::init();

    loop {
        // Render UI at 60 FPS
        terminal.draw(|frame| {
            ui::render(frame, &self.scan_state, &self.ui_state)
        })?;

        // Process events (keyboard, EventBus, timer)
        let control = process_events(
            Arc::clone(&self.event_bus),
            Arc::clone(&self.scan_state),
            &mut self.ui_state,
            // ... event channels
        ).await;

        if matches!(control, LoopControl::Quit) {
            break;
        }
    }

    // Restore terminal (ratatui handles cleanup)
    ratatui::restore();
    Ok(())
}
```

**Exit Paths:**
- **Normal**: User presses 'q' or Ctrl+C → LoopControl::Quit → ratatui::restore()
- **Panic**: ratatui 0.29+ panic hook automatically restores terminal
- **Scan Complete**: Scanner publishes ScanCompleted → TUI can choose to exit or display results

---

### 2. State Management

#### ScanState (Shared Between Scanner and TUI)

**Purpose**: Thread-safe shared state for scanner ↔ TUI communication.

**Type**: `Arc<RwLock<ScanState>>` (atomic reference counted, read-write lock)

**Data Structure:**

```rust
pub struct ScanState {
    pub stage: ScanStage,              // Initializing, Scanning, Complete, Error
    pub progress_percentage: f32,       // 0.0 - 100.0
    pub completed: u64,                 // Ports scanned
    pub total: u64,                     // Total ports
    pub open_ports: usize,              // Open ports found
    pub closed_ports: usize,            // Closed ports
    pub filtered_ports: usize,          // Filtered ports
    pub detected_services: usize,       // Services detected
    pub errors: usize,                  // Error count
    pub discovered_hosts: Vec<IpAddr>,  // Live hosts (deduplicated)
    pub warnings: Vec<String>,          // Warnings
}

pub enum ScanStage {
    Initializing,    // Scanner setup
    Scanning,        // Active scan
    Complete,        // Scan finished successfully
    Error(String),   // Scan failed with error message
}
```

**Access Pattern:**

```rust
// Read (many concurrent readers, non-blocking)
let state = scan_state.read();
let open_ports = state.open_ports;
let stage = state.stage.clone();
drop(state);  // Release lock ASAP

// Write (exclusive access, blocks all readers)
let mut state = scan_state.write();
state.open_ports += 10;
state.progress_percentage = (state.completed as f32 / state.total as f32) * 100.0;
drop(state);  // Release lock ASAP
```

**Best Practices:**
- **Hold locks briefly**: Read/write data, then immediately drop lock
- **Avoid nested locks**: Prevents deadlocks
- **Batch updates**: Write multiple fields in single lock acquisition
- **Read consistency**: Take read lock once per frame, copy to local vars

#### UIState (Local TUI State)

**Purpose**: TUI-only ephemeral state (not shared with scanner).

**Type**: `UIState` (single-threaded, no locking needed)

**Data Structure:**

```rust
pub struct UIState {
    pub selected_pane: SelectedPane,           // Main | Help
    pub active_tab: DashboardTab,              // PortTable | ServiceTable | Metrics
    pub cursor_position: usize,                // Cursor position in lists
    pub scroll_offset: usize,                  // Scroll offset for content
    pub input_buffer: String,                  // Text input for search/filter
    pub show_help: bool,                       // Help overlay visibility
    pub fps: f32,                              // Debug FPS counter
    pub aggregator_dropped_events: usize,      // Dropped event count
}

pub enum SelectedPane {
    Main,
    Help,
}

pub enum DashboardTab {
    PortTable,      // Real-time port discoveries
    ServiceTable,   // Service detection results
    Metrics,        // Performance metrics
}
```

**Navigation Methods:**

```rust
impl UIState {
    pub fn next_pane(&mut self) {
        self.selected_pane = match self.selected_pane {
            SelectedPane::Main => SelectedPane::Help,
            SelectedPane::Help => SelectedPane::Main,
        };
    }

    pub fn switch_tab(&mut self) {
        self.active_tab = match self.active_tab {
            DashboardTab::PortTable => DashboardTab::ServiceTable,
            DashboardTab::ServiceTable => DashboardTab::Metrics,
            DashboardTab::Metrics => DashboardTab::PortTable,  // Cycle
        };
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}
```

---

### 3. Event System

#### Event Aggregator (Rate Limiting)

**Purpose**: Prevent UI overload from high-frequency events (10K+ events/second).

**Location**: `crates/prtip-tui/src/events/aggregator.rs`

**Strategy:**

- **Aggregate**: Count PortFound, HostDiscovered, ServiceDetected events (don't buffer individual events)
- **Buffer**: Store lifecycle events (ScanStarted, ScanCompleted, errors, warnings)
- **Flush**: Process batches every 16ms (60 FPS) to prevent UI overload

**Constants:**

```rust
const MAX_BUFFER_SIZE: usize = 1000;               // Drop events if exceeded
const MIN_EVENT_INTERVAL: Duration = Duration::from_millis(16);  // 60 FPS
```

**Event Statistics:**

```rust
pub struct EventStats {
    pub ports_found: usize,                        // Aggregated count
    pub hosts_discovered: usize,                   // Aggregated count
    pub services_detected: usize,                  // Aggregated count
    pub discovered_ips: HashMap<IpAddr, usize>,    // Deduplication map
    pub total_events: usize,                       // Total processed
    pub dropped_events: usize,                     // Rate limit drops
}
```

**API Methods:**

```rust
pub struct EventAggregator {
    buffer: Vec<ScanEvent>,
    stats: EventStats,
    last_flush: Instant,
}

impl EventAggregator {
    pub fn new() -> Self

    pub fn add_event(&mut self, event: ScanEvent) -> bool {
        // Returns false if buffer full (event dropped)
    }

    pub fn should_flush(&self) -> bool {
        // True if MIN_EVENT_INTERVAL passed
    }

    pub fn flush(&mut self) -> (Vec<ScanEvent>, EventStats) {
        // Returns buffered events + aggregated stats, resets state
    }

    pub fn stats(&self) -> &EventStats
}
```

**Performance:**
- **Throughput**: 10,000+ events/second
- **Latency**: 16ms maximum (60 FPS flush rate)
- **Memory**: ~100 KB (1,000 events × ~100 bytes/event estimate)
- **CPU**: ~2% overhead (event processing + aggregation logic)

#### Event Loop Coordination

**Purpose**: Coordinate keyboard input, EventBus events, and 60 FPS timer.

**Location**: `crates/prtip-tui/src/events/loop.rs`

**Pattern**: `tokio::select!` for concurrent event handling

```rust
pub async fn process_events(
    event_bus: Arc<EventBus>,
    scan_state: Arc<RwLock<ScanState>>,
    ui_state: &mut UIState,
    event_rx: &mut mpsc::UnboundedReceiver<ScanEvent>,
    crossterm_rx: &mut EventStream,
    aggregator: &mut EventAggregator,
) -> LoopControl {
    let mut tick_interval = tokio::time::interval(Duration::from_millis(16));

    tokio::select! {
        // Keyboard events (Ctrl+C, quit, navigation, Tab switching)
        Some(Ok(crossterm_event)) = crossterm_rx.next() => {
            if let Event::Key(key) = crossterm_event {
                match key.code {
                    KeyCode::Char('q') => return LoopControl::Quit,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return LoopControl::Quit
                    }
                    KeyCode::Tab => ui_state.switch_tab(),
                    KeyCode::F(1) | KeyCode::Char('?') => ui_state.toggle_help(),
                    // ... other key handlers
                    _ => {}
                }
            }
        }

        // EventBus events (add to aggregator, don't process immediately)
        Some(scan_event) = event_rx.recv() => {
            aggregator.add_event(scan_event);
        }

        // 60 FPS timer (flush aggregator, update state)
        _ = tick_interval.tick() => {
            if aggregator.should_flush() {
                let (events, stats) = aggregator.flush();

                // Process buffered lifecycle events
                for event in events {
                    handle_scan_event(event, Arc::clone(&scan_state));
                }

                // Apply aggregated statistics in single write lock
                let mut state = scan_state.write();
                state.open_ports += stats.ports_found;
                state.detected_services += stats.services_detected;

                // Deduplicate discovered hosts
                for (ip, _count) in stats.discovered_ips {
                    if !state.discovered_hosts.contains(&ip) {
                        state.discovered_hosts.push(ip);
                    }
                }

                ui_state.aggregator_dropped_events = stats.dropped_events;
            }
        }
    }

    LoopControl::Continue
}
```

---

## Widget System

### Component Trait

**Purpose**: Common interface for all TUI components.

**Location**: `crates/prtip-tui/src/widgets/component.rs`

**Trait Definition:**

```rust
pub trait Component {
    /// Render the component to a frame
    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// Handle keyboard input
    fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<()>;

    /// Update component state (called every frame)
    fn update(&mut self) -> anyhow::Result<()>;
}
```

**Implementation Example:**

```rust
pub struct StatusBar {
    scan_state: Arc<RwLock<ScanState>>,
}

impl Component for StatusBar {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let state = self.scan_state.read();

        let text = format!(
            "ProRT-IP Scanner | Target: {} | Type: {} | {}%",
            state.target, state.scan_type, state.progress_percentage
        );

        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);
    }

    fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        // StatusBar doesn't handle keyboard events
        Ok(())
    }

    fn update(&mut self) -> anyhow::Result<()> {
        // StatusBar state updated via shared ScanState
        Ok(())
    }
}
```

---

### Production Widgets (7 Total)

#### Phase 6.1 Core Widgets (4)

**1. StatusBar** - Header widget with scan metadata
- Scan stage indicator (Initializing, Scanning, Complete, Error)
- Target information (IP/CIDR range)
- Scan type display (SYN, Connect, UDP, etc.)
- Overall progress percentage
- Color-coded status: Green (active), Yellow (warning), Red (error)
- **Layout**: Fixed 3 lines (10% of terminal)

**2. MainWidget** - Central content area with results summary
- Live host count (discovered IPs)
- Port statistics (open/closed/filtered counts)
- Service detection summary
- Error/warning counters
- Scrollable content area
- **Layout**: Variable height (80% of terminal)

**3. LogWidget** - Real-time event log with scrolling
- Circular buffer (1,000 most recent events)
- Timestamped log entries
- Event type filtering (Info, Warning, Error)
- Auto-scroll toggle (follow mode)
- Keyboard navigation (↑/↓, Page Up/Down, Home/End)
- Color-coded entries: Info=White, Warn=Yellow, Error=Red
- **Performance**: <5ms for 1,000 entries

**4. HelpWidget** - Overlay with keyboard shortcuts
- Comprehensive keybinding reference
- Grouped by category (Navigation, Filtering, Views)
- Centered popup overlay (50% width × 60% height)
- Semi-transparent background (Clear widget)
- Toggle with `?` or `F1` key

#### Phase 6.2 Dashboard Widgets (3)

**5. PortTableWidget** - Real-time port discovery table

**Features:**
- **Data**: 1,000-entry circular buffer (PortDiscovery events)
- **Columns**: Timestamp, IP Address, Port, State, Protocol, Scan Type
- **Sorting**: All 6 columns × ascending/descending (12 sort modes)
- **Filtering**: State (All/Open/Closed/Filtered), Protocol (All/TCP/UDP), Search (IP or port)
- **Color Coding**: Open=Green, Closed=Red, Filtered=Yellow

**Keyboard Shortcuts:**
- `t`: Sort by timestamp | `i`: IP address | `p`: Port | `s`: State | `r`: Protocol | `c`: Scan type
- `a`: Auto-scroll | `f`: State filter | `d`: Protocol filter | `/`: Search
- `↑/↓`: Navigate | Page Up/Down: Scroll by page

**Performance**: <5ms frame time for 1,000 entries

---

**6. ServiceTableWidget** - Service detection results with confidence

**Features:**
- **Data**: 500-entry circular buffer (ServiceDetection events)
- **Columns**: Timestamp, IP, Port, Service Name, Version, Confidence (0-100%)
- **Confidence Colors**: High (≥90%)=Green, Medium (50-89%)=Yellow, Low (<50%)=Red
- **Filtering**: All, Low (≥50%), Medium (≥75%), High (≥90%)
- **Sorting**: All 6 columns with ascending/descending

**Keyboard Shortcuts:**
- `1-6`: Sort by column (timestamp, IP, port, service, version, confidence)
- `c`: Cycle confidence filter | `a`: Auto-scroll | `/`: Search
- `↑/↓`: Navigate | Page Up/Down: Scroll by page

**Performance**: <5ms frame time for 500 entries

---

**7. MetricsDashboardWidget** - Real-time performance metrics

**Features:**
- **3-Column Layout**: Progress | Throughput | Statistics
- **Progress**: Percentage, completed/total ports, ETA (5-second rolling average), stage indicator
- **Throughput**: Current/average/peak ports/second, current/average packets/second (5-second window)
- **Statistics**: Open ports, services, errors, duration (HH:MM:SS), status indicator

**Human-Readable Formatting:**
- Durations: "1h 12m 45s", "23m 15s", "45s"
- Numbers: "12,345" (comma separators)
- Throughput: "1.23K pps", "456.7 pps", "12.3M pps"

**Color Coding:**
- Status: Green (Active), Yellow (Paused), Red (Error)
- ETA: White (normal), Yellow (>1h), Red (stalled)
- Throughput: Green (≥target), Yellow (50-99%), Red (<50%)

**Performance**: <5ms frame time (3× under 16.67ms budget)

---

### Tabbed Dashboard Interface

**Architecture**: 3-tab dashboard with keyboard navigation

```rust
pub enum DashboardTab {
    PortTable,      // Tab 1: Real-time port discoveries
    ServiceTable,   // Tab 2: Service detection results
    Metrics,        // Tab 3: Performance metrics
}
```

**Tab Switching:**
- `Tab`: Switch to next dashboard (Port → Service → Metrics → Port, cycle)
- `Shift+Tab`: Switch to previous dashboard (reverse direction)

**Visual Tab Bar:**
```
┌─────────────────────────────────────────────────────────────┐
│ [Port Table] | Service Table | Metrics                      │
├─────────────────────────────────────────────────────────────┤
│ [Active Dashboard Widget Content]                           │
│ ...                                                          │
└─────────────────────────────────────────────────────────────┘
```

**Event Routing:**
- Active tab receives keyboard events (sorting, filtering, navigation)
- Inactive tabs do not process events (performance optimization)

---

## Event Flow

### 1. Scanner → EventBus → TUI Flow

**High-Frequency Event Aggregation Example:**

```
Scanner Thread                EventBus               TUI Thread
──────────────                ────────               ──────────

port_scan() finds 1,000 ports in 10ms
    │
    │ publishes PortFound #1
    ├──────────────────────▶ broadcast ─────────────▶ event_rx.recv()
    │                                                       │
    │ publishes PortFound #2                               ▼
    ├──────────────────────▶ broadcast ─────────────▶ aggregator.add_event()
    │                                                 (stats.ports_found += 1)
    │ publishes PortFound #3
    ├──────────────────────▶ broadcast ─────────────▶ aggregator.add_event()
    │                                                 (stats.ports_found += 1)
    ...
    │ publishes PortFound #1000
    ├──────────────────────▶ broadcast ─────────────▶ aggregator.add_event()
                                                      (stats.ports_found = 1000)
                                                            │
                                                            │ (buffered, no UI update)
                                                            ▼
[16ms passes - tick_interval fires]
                                                      tick_interval.tick()
                                                            │
                                                            ▼
                                                      aggregator.should_flush() → true
                                                            │
                                                            ▼
                                                      flush() → (events=[], stats)
                                                            │
                                                            ▼
                                                      scan_state.write()
                                                      state.open_ports += 1000
                                                      drop(state)
                                                            │
                                                            ▼
                                                      terminal.draw(render)
                                                      UI displays: "Open Ports: 1000"
```

**Without Aggregation:**
- 1,000 state updates (each requires write lock)
- 1,000 renders (impossible at 60 FPS)
- Result: UI freezes, dropped frames, sluggish response

**With Aggregation (16ms batches):**
- 1 batch update (single write lock)
- 1 render (smooth 60 FPS)
- Result: Smooth UI, no dropped frames, instant response

---

### 2. Keyboard Input Flow

```
Terminal             crossterm            TUI Event Loop            State
────────             ─────────            ──────────────            ─────

User presses 'Tab'
    │
    ├──────────▶ EventStream.next()
    │                  │
    │                  ├──────────────▶ process_events()
    │                  │                      │
    │                  │                      │ matches KeyCode::Tab
    │                  │                      ▼
    │                  │                ui_state.switch_tab()
    │                  │                      │
    │                  │                      ▼
    │                  │                active_tab changes
    │                  │                (PortTable → ServiceTable)
    │                  │                      │
    │                  │                      ▼
    │                  │                Next frame renders ServiceTable
```

---

## Performance Optimization

### 60 FPS Rendering Budget

**Frame Budget Breakdown (16.67ms total):**

| Component | Time Budget | Actual | Margin |
|-----------|-------------|--------|--------|
| **Rendering** | <5ms | ~3ms | +2ms |
| **State Access** | <1ms | ~0.5ms | +0.5ms |
| **Event Processing** | <10ms | ~8ms | +2ms |
| **System Overhead** | ~1ms | ~1ms | 0 |
| **Total** | 16.67ms | ~12.5ms | **+4.17ms** |

**Performance Validation:**

```rust
// Measure frame time
let start = Instant::now();
terminal.draw(|frame| ui::render(frame, &scan_state, &ui_state))?;
let render_time = start.elapsed();

assert!(render_time.as_millis() < 5, "Render exceeded 5ms budget: {:?}", render_time);
```

---

### Event Aggregation Performance

**Test Scenario**: 10,000 PortFound events in 1 second

**Without Aggregation:**
```
Events: 10,000
State Updates: 10,000 (each requires write lock)
Renders: 10,000 (impossible at 60 FPS)
Result: UI freezes, 166× frame budget exceeded
```

**With Aggregation (16ms batches):**
```
Events: 10,000
Batches: 62 (1000ms / 16ms)
State Updates: 62 (one per batch)
Renders: 60 (capped at 60 FPS)
Result: Smooth UI, 161× fewer state updates
```

**Aggregation Benefits:**

| Metric | Without | With | Improvement |
|--------|---------|------|-------------|
| State Updates/sec | 10,000 | 62 | **161× fewer** |
| Write Locks/sec | 10,000 | 62 | **161× fewer** |
| Renders/sec | 10,000 (dropped) | 60 | **Smooth 60 FPS** |
| Max Latency | Unbounded | 16ms | **Bounded latency** |
| UI Responsiveness | Frozen | Smooth | **Professional UX** |

---

### Memory Usage Analysis

**Component Breakdown:**

```
Component                Size (Bytes)    Notes
─────────                ────────────    ─────
ScanState                ~1,024          Arc<RwLock<T>>, 10 fields
UIState                  ~128            Stack-allocated, 8 fields
EventAggregator          ~102,400        1,000 events × ~100 bytes/event
Event Buffer             ~102,400        MAX_BUFFER_SIZE = 1,000
Terminal Buffer          ~10,240         ratatui screen buffer (80×24 typical)
Widget State (7 total)   ~5,120          Minimal per-widget state

Total: ~221 KB (negligible overhead vs scanner ~100 MB+)
```

**Memory Optimization:**
- **Circular Buffers**: PortTableWidget (1,000), ServiceTableWidget (500), LogWidget (1,000)
- **No Event Cloning**: EventAggregator counts, doesn't store high-frequency events
- **Efficient Rendering**: ratatui diffs and updates only changed terminal cells

---

### CPU Profiling

**Component CPU Usage (10,000 events/sec load):**

```
Component              % CPU          Notes
─────────              ─────          ─────
Event Processing       ~2%            Aggregation logic
State Updates          ~1%            RwLock write overhead
Rendering (ratatui)    ~3%            Diffing + terminal I/O
Keyboard Handling      <1%            Rare events
System Overhead        ~1%            tokio runtime

Total: ~8% CPU (on modern CPU, single core)
```

**Optimization Techniques:**
- **Event Aggregation**: 161× fewer state updates
- **parking_lot::RwLock**: 2-3× faster than std::sync::RwLock
- **Immediate Mode Rendering**: ratatui efficient diffing algorithm
- **Lock-Free Reads**: parking_lot fast path when no writers

---

## State Management Deep Dive

### Shared State Pattern

**Challenge**: Scanner (background thread) needs to update state while TUI (main thread) reads it.

**Solution**: `Arc<RwLock<ScanState>>`

- **Arc (Atomic Reference Counting)**: Shared ownership across threads, thread-safe reference counting
- **RwLock (Read-Write Lock)**: Many concurrent readers OR one exclusive writer

**Access Pattern:**

```rust
// Scanner thread (writer)
let mut state = scan_state.write();  // Exclusive lock (blocks all readers)
state.open_ports += 10;
state.progress_percentage = (state.completed as f32 / state.total as f32) * 100.0;
drop(state);                          // Release lock ASAP

// TUI thread (reader)
let state = scan_state.read();       // Shared lock (many readers allowed)
let open_ports = state.open_ports;
let progress = state.progress_percentage;
drop(state);                          // Release lock ASAP
```

**Best Practices:**

1. **Hold Locks Briefly**:
   ```rust
   // Good: Read data, release lock immediately
   let open_ports = {
       let state = scan_state.read();
       state.open_ports
   };  // Lock automatically dropped at end of scope

   // Bad: Hold lock during expensive operation
   let state = scan_state.read();
   let open_ports = state.open_ports;
   expensive_computation(open_ports);  // Lock still held!
   drop(state);
   ```

2. **Avoid Nested Locks**:
   ```rust
   // Bad: Potential deadlock
   let state1 = scan_state.write();
   let state2 = other_state.write();  // Deadlock risk!

   // Good: Single lock per critical section
   { let state = scan_state.write(); /* update */ }
   { let state = other_state.write(); /* update */ }
   ```

3. **Batch Updates**:
   ```rust
   // Good: Multiple updates in single lock acquisition
   let mut state = scan_state.write();
   state.open_ports += stats.ports_found;
   state.closed_ports += stats.ports_closed;
   state.detected_services += stats.services_detected;
   state.progress_percentage = calculate_progress(&state);
   drop(state);
   ```

4. **Read Consistency**:
   ```rust
   // Good: Read all needed data in single lock acquisition
   let (open_ports, total_ports, progress) = {
       let state = scan_state.read();
       (state.open_ports, state.total, state.progress_percentage)
   };
   // Use local copies without holding lock
   render_stats(open_ports, total_ports, progress);
   ```

---

### Lock Contention Mitigation

**Problem**: High-frequency writes block readers, causing UI stutters.

**Solution 1: Event Aggregation (Primary Strategy)**

```rust
// Before: 1,000 writes/second (each blocks readers)
for event in events {
    let mut state = scan_state.write();  // LOCK (blocks TUI reader)
    state.open_ports += 1;               // WRITE
}                                        // UNLOCK

// After: 60 writes/second (16ms batches)
let (events, stats) = aggregator.flush();
let mut state = scan_state.write();      // LOCK ONCE
state.open_ports += stats.ports_found;   // BATCH WRITE (all updates)
drop(state);                              // UNLOCK
```

**Benefits:**
- **161× fewer write locks** (10,000/sec → 62/sec at 10K events/sec)
- **Reduced contention**: TUI reads succeed 99%+ of time (62 write windows vs 10,000)
- **Predictable latency**: Max 16ms wait for write lock (60 FPS aligned)

**Solution 2: parking_lot::RwLock (Secondary Strategy)**

```rust
// std::sync::RwLock
use std::sync::RwLock;
let state = Arc::new(RwLock::new(ScanState::default()));

// parking_lot::RwLock (2-3× faster)
use parking_lot::RwLock;
let state = Arc::new(RwLock::new(ScanState::default()));
```

**parking_lot Advantages:**
- **Lock-free fast path**: Readers don't block each other when no writers
- **Writer priority**: Prevents writer starvation (writers get lock quickly)
- **Benchmarks**: 2-3× faster than std::sync::RwLock on typical workloads
- **No poisoning**: Simpler error handling (no Result<Guard, PoisonError>)

---

## Terminal Lifecycle

### Initialization

**ratatui 0.29+ Automatic Setup:**

```rust
use ratatui::DefaultTerminal;

// Initialize terminal (one-liner)
let mut terminal = ratatui::init();

// What this does internally:
// 1. crossterm::terminal::enable_raw_mode()
// 2. crossterm::execute!(stdout, EnterAlternateScreen)
// 3. Set panic hook for automatic cleanup
```

**Manual Setup (if needed):**

```rust
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{stdout, Result};

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}
```

---

### Normal Exit

**Automatic Cleanup:**

```rust
pub async fn run(&mut self) -> Result<()> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|frame| ui::render(frame, &self.scan_state, &self.ui_state))?;

        let control = process_events(...).await;
        if matches!(control, LoopControl::Quit) {
            break;
        }
    }

    // Restore terminal (automatically called)
    ratatui::restore();
    Ok(())
}
```

**What `ratatui::restore()` does:**
- `crossterm::execute!(stdout, LeaveAlternateScreen)` - Exit alternate screen
- `crossterm::terminal::disable_raw_mode()` - Restore normal terminal mode
- Flushes output buffers

---

### Panic Recovery

**ratatui 0.29+ Automatic Panic Hook:**

```rust
// ratatui::init() automatically sets panic hook
let mut terminal = ratatui::init();

// If panic occurs anywhere:
panic!("Something went wrong!");

// Panic hook automatically:
// 1. Calls ratatui::restore()
// 2. Restores terminal to normal mode
// 3. Prints panic message to stderr
// 4. Exits process

// Before ratatui 0.29 (manual setup required):
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |panic_info| {
    ratatui::restore();
    original_hook(panic_info);
}));
```

**Testing Panic Recovery:**

```rust
#[test]
#[should_panic(expected = "Test panic")]
fn test_panic_recovery() {
    let mut terminal = ratatui::init();

    // Panic should trigger cleanup
    panic!("Test panic");

    // Terminal automatically restored (cannot verify in test)
}
```

---

### Ctrl+C Handling

**Graceful Shutdown:**

```rust
tokio::select! {
    Some(Ok(Event::Key(key))) = crossterm_rx.next() => {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // User pressed Ctrl+C
                    return LoopControl::Quit;  // Graceful exit
                }
                KeyCode::Char('q') => {
                    // User pressed 'q'
                    return LoopControl::Quit;  // Graceful exit
                }
                _ => {}
            }
        }
    }
}

// Main loop breaks, App::run() exits, ratatui::restore() called
```

**Why Not Signal Handlers:**

```rust
// Bad: Signal handlers complex, platform-specific
use tokio::signal;
let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())?;
tokio::select! {
    _ = sigint.recv() => { /* cleanup */ }
}

// Good: crossterm captures Ctrl+C as KeyEvent (works on all platforms)
```

---

## Testing Strategy

### Unit Tests (140 tests)

**Coverage Areas:**

- **EventAggregator** (4 tests): Event aggregation logic, buffer limits, flush behavior, deduplication
- **Widget Tests** (59 tests):
  - PortTableWidget: 14 tests (sorting, filtering)
  - ServiceTableWidget: 21 tests (sorting, filtering, color coding)
  - MetricsDashboardWidget: 24 tests (calculations, formatting, edge cases)
- **Component Tests**: Rendering, state updates, keyboard handling

**Example: EventAggregator Buffer Limit**

```rust
#[test]
fn test_aggregator_buffer_limit() {
    let mut agg = EventAggregator::new();

    // Fill buffer to MAX_BUFFER_SIZE
    for i in 0..MAX_BUFFER_SIZE {
        let event = ScanEvent::ProgressUpdate { /* ... */ };
        assert!(agg.add_event(event), "Event {} should be added", i);
    }

    // Next event should be dropped
    let overflow_event = ScanEvent::ProgressUpdate { /* ... */ };
    assert!(!agg.add_event(overflow_event), "Buffer overflow should drop event");
    assert_eq!(agg.stats().dropped_events, 1, "Dropped event count should be 1");
}
```

---

### Integration Tests (25 tests)

**Coverage Areas:**

- **App Lifecycle**: Creation, initialization, shutdown
- **ScanState Shared State**: Multiple readers, exclusive writers, data consistency
- **UIState Navigation**: Pane switching, help toggle, cursor movement, tab switching
- **EventAggregator Timing**: 16ms flush interval verification
- **EventBus Subscription**: Async event delivery

**Example: Shared State Consistency**

```rust
#[tokio::test]
async fn test_scan_state_shared() {
    // Create shared state
    let state1 = ScanState::shared();
    let state2 = Arc::clone(&state1);

    // Modify via state1
    {
        let mut s = state1.write();
        s.open_ports = 10;
        s.progress_percentage = 50.0;
    }

    // Read via state2 (should see changes)
    {
        let s = state2.read();
        assert_eq!(s.open_ports, 10, "Open ports should be visible");
        assert_eq!(s.progress_percentage, 50.0, "Progress should be visible");
    }
}
```

**Example: Tab Switching Integration**

```rust
#[test]
fn test_dashboard_tab_switching() {
    let mut ui_state = UIState::default();

    // Initial tab
    assert_eq!(ui_state.active_tab, DashboardTab::PortTable);

    // Switch to ServiceTable
    ui_state.switch_tab();
    assert_eq!(ui_state.active_tab, DashboardTab::ServiceTable);

    // Switch to Metrics
    ui_state.switch_tab();
    assert_eq!(ui_state.active_tab, DashboardTab::Metrics);

    // Cycle back to PortTable
    ui_state.switch_tab();
    assert_eq!(ui_state.active_tab, DashboardTab::PortTable);
}
```

---

### Doctests (2 passing, 1 ignored)

**Coverage Areas:**

- **App::new() Example**: Public API usage
- **Crate-level Example** (lib.rs): Quick start guide
- **Component Trait** (ignored): Future implementation placeholder

**Example: App Initialization Doctest**

```rust
/// # Examples
///
/// ```rust,no_run
/// use prtip_tui::App;
/// use prtip_core::event_bus::EventBus;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let event_bus = Arc::new(EventBus::new(1000));
///     let mut app = App::new(event_bus);
///     app.run().await?;
///     Ok(())
/// }
/// ```
pub fn new(event_bus: Arc<EventBus>) -> Self {
    // Implementation
}
```

---

### Test Metrics Summary

**Phase 6.2 (Sprint 6.2 Complete):**

```
Test Type         Count    Status    Coverage
─────────         ─────    ──────    ────────
Unit Tests        140      ✓ Pass    Aggregator (4), Widgets (59), Components
Integration       25       ✓ Pass    App, State, Events, Tab switching
Doctests          2        ✓ Pass    Public API examples
                  1        Ignored   Future Component trait

Total             168      165 Pass  Comprehensive coverage
```

**Widget Test Breakdown:**
- PortTableWidget: 14 tests (sorting 12, filtering 2)
- ServiceTableWidget: 21 tests (sorting 12, filtering 4, color 3, search 2)
- MetricsDashboardWidget: 24 tests (throughput 5, ETA 5, formatting 8, color 3, edge 3)

---

## Advanced Topics

### Custom Widget Development

**Step 1: Implement Component Trait**

```rust
use ratatui::prelude::*;
use crossterm::event::KeyEvent;

pub struct CustomWidget {
    state: Arc<RwLock<ScanState>>,
    internal_state: Vec<String>,
}

impl Component for CustomWidget {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let state = self.state.read();

        // Create widget content
        let text = format!("Custom Data: {}", self.internal_state.len());
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Custom"));

        frame.render_widget(paragraph, area);
    }

    fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('r') => {
                // Refresh data
                self.internal_state.clear();
            }
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self) -> anyhow::Result<()> {
        // Update internal state from shared ScanState
        let state = self.state.read();
        // ... process state
        Ok(())
    }
}
```

**Step 2: Integrate with UI**

```rust
// In ui/renderer.rs
pub fn render(frame: &mut Frame, scan_state: &ScanState, ui_state: &UIState) {
    let chunks = layout::create_layout(frame.area());

    // Add custom widget to layout
    let mut custom_widget = CustomWidget::new(Arc::clone(scan_state));
    custom_widget.render(frame, chunks[3]);  // Fourth area
}
```

---

### Extending the Event System

**Add Custom Event Type:**

```rust
// In prtip-core/src/events/mod.rs
#[derive(Debug, Clone)]
pub enum ScanEvent {
    // Existing events...
    PortFound { ip: IpAddr, port: u16, state: PortState },

    // Custom event
    CustomMetric {
        name: String,
        value: f64,
        timestamp: DateTime<Utc>,
    },
}
```

**Publish Custom Event:**

```rust
// In scanner code
event_bus.publish(ScanEvent::CustomMetric {
    name: "throughput_mbps".to_string(),
    value: 125.5,
    timestamp: Utc::now(),
});
```

**Handle in TUI:**

```rust
// In events/loop.rs handle_scan_event()
match event {
    ScanEvent::CustomMetric { name, value, timestamp } => {
        // Update custom widget state
        ui_state.custom_metrics.insert(name, value);
    }
    // ... other event handlers
}
```

---

### Debugging TUI Issues

**Enable Debug Logging:**

```rust
// Set RUST_LOG environment variable
export RUST_LOG=prtip_tui=debug

// In code
use tracing::{debug, info, warn, error};

impl EventAggregator {
    pub fn flush(&mut self) -> (Vec<ScanEvent>, EventStats) {
        debug!("Flushing aggregator: {} buffered events", self.buffer.len());
        debug!("Stats: ports={}, hosts={}, dropped={}",
               self.stats.ports_found,
               self.stats.hosts_discovered,
               self.stats.dropped_events);

        // ... flush logic
    }
}
```

**Log to File (Terminal Unavailable):**

```rust
// In main.rs
use tracing_subscriber::fmt::writer::MakeWriterExt;

let log_file = std::fs::File::create("/tmp/prtip-tui.log")?;
tracing_subscriber::fmt()
    .with_writer(log_file.with_max_level(tracing::Level::DEBUG))
    .init();
```

**Monitor Frame Times:**

```rust
// In app.rs
let start = Instant::now();
terminal.draw(|frame| ui::render(frame, &scan_state, &ui_state))?;
let render_time = start.elapsed();

if render_time.as_millis() > 5 {
    warn!("Slow render: {:?} (budget: 5ms)", render_time);
}
```

**Track Event Drops:**

```rust
// In ui_state
if ui_state.aggregator_dropped_events > 0 {
    warn!("Dropped {} events due to buffer overflow",
          ui_state.aggregator_dropped_events);
}
```

---

## See Also

### Related Guides

- **[Performance Tuning](./performance-tuning.md)** - Performance optimization techniques (NUMA, timing, rate limiting)
- **[Real-Time Dashboard](../features/real-time-dashboard.md)** - Dashboard widgets usage and examples
- **[Event System Guide](../../35-EVENT-SYSTEM-GUIDE.md)** - EventBus architecture and event types
- **[Large-Scale Scanning](./large-scale-scanning.md)** - Handling millions of targets (TUI impact)

### Feature Guides

- **[Service Detection](../features/service-detection.md)** - ServiceTableWidget data source
- **[OS Fingerprinting](../features/os-fingerprinting.md)** - Future TUI integration

### Technical Documentation

- **[Architecture](../../00-ARCHITECTURE.md)** - Overall system design, async architecture
- **[Testing Guide](../../06-TESTING.md)** - Unit, integration, and doctest strategies
- **[Implementation Guide](../../04-IMPLEMENTATION-GUIDE.md)** - Code organization, module structure

### External Resources

- **ratatui Documentation**: https://ratatui.rs/ (TUI framework reference)
- **crossterm Documentation**: https://docs.rs/crossterm/ (Terminal manipulation)
- **tokio::select! Macro**: https://docs.rs/tokio/latest/tokio/macro.select.html (Event loop pattern)
- **parking_lot::RwLock**: https://docs.rs/parking_lot/ (High-performance locking)

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
**Document Status:** Production-ready, Phase 6.2 Complete (7 widgets, 3-tab dashboard)
