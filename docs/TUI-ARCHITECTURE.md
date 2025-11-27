# TUI Architecture Documentation

**Version:** 1.3.0
**Last Updated:** 2025-11-23
**Status:** Phase 6.6 COMPLETE (Sprint 6.6 Parts 1-3), Memory-Mapped I/O + Live Event Flow

## Table of Contents

1. [Overview](#overview)
2. [Architecture Diagram](#architecture-diagram)
3. [Core Components](#core-components)
4. [Event Flow](#event-flow)
5. [Performance Characteristics](#performance-characteristics)
6. [State Management](#state-management)
7. [Terminal Lifecycle](#terminal-lifecycle)
8. [Testing Strategy](#testing-strategy)
9. [Future Enhancements](#future-enhancements)
10. [References](#references)

---

## Overview

### Design Goals

The ProRT-IP TUI (Terminal User Interface) is designed to provide real-time visualization of network scanning operations with the following objectives:

1. **Real-time Updates**: Display scan progress, discovered hosts, and open ports as they're found
2. **High Performance**: Handle 10,000+ events/second without UI lag or dropped events
3. **Responsive**: Maintain 60 FPS rendering for smooth user experience
4. **Separation of Concerns**: TUI is consumer-only, scanner has no TUI dependencies
5. **Graceful Degradation**: Clean terminal restoration on all exit paths (normal, Ctrl+C, panic)

### Technology Stack

- **ratatui 0.29+**: Modern TUI framework with immediate mode rendering
- **crossterm**: Cross-platform terminal manipulation
- **tokio**: Async runtime for event loop coordination
- **parking_lot**: High-performance RwLock for shared state
- **prtip-core**: EventBus integration for scan events

### Key Metrics

- **Target FPS**: 60 (16.67ms frame budget)
- **Event Throughput**: 10,000+ events/second
- **Event Aggregation**: 16ms batching interval (60 FPS)
- **Max Buffer Size**: 1,000 events before dropping
- **Test Coverage**: 165 tests (140 unit, 25 integration) [Sprint 6.2]

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                          ProRT-IP Scanner                           │
│                     (prtip-core, no TUI deps)                       │
└────────────────┬────────────────────────────────────────────────────┘
                 │
                 │ publishes
                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                           EventBus                                  │
│              (mpsc::unbounded_channel, broadcast)                   │
└────────────────┬────────────────────────────────────────────────────┘
                 │
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

                    Data Flow Legend:
                    ════════════════
                    │  Event flow (one-way)
                    ▼  Processing direction
                    ┌┐ Component boundary
```

### Architecture Principles

1. **Consumer-Only TUI**: Scanner publishes to EventBus, TUI subscribes (one-way flow)
2. **Immediate Mode Rendering**: Full UI redrawn every frame, ratatui diffs and updates terminal
3. **Event Aggregation**: Batch high-frequency events (PortFound, HostDiscovered) to prevent overload
4. **Shared State**: `Arc<RwLock<ScanState>>` for thread-safe scanner ↔ TUI communication
5. **Graceful Cleanup**: ratatui 0.29+ automatic panic hook ensures terminal restoration

---

## Core Components

### 1. App (`src/app.rs`)

**Purpose**: Main TUI application lifecycle manager

**Responsibilities**:
- Terminal initialization (raw mode, alternate screen)
- EventBus subscription
- Event loop coordination (keyboard, EventBus, timer)
- Terminal restoration on exit

**Key Methods**:
```rust
pub fn new(event_bus: Arc<EventBus>) -> Self
pub async fn run(&mut self) -> Result<()>
pub fn should_quit(&self) -> bool
pub fn scan_state(&self) -> Arc<RwLock<ScanState>>
```

**Event Loop Pattern**:
```rust
loop {
    // Render UI (60 FPS)
    terminal.draw(|frame| ui::render(frame, &scan_state, &ui_state))?;

    // Process events (tokio::select!)
    let control = process_events(...).await;

    if matches!(control, LoopControl::Quit) {
        break;
    }
}
```

### 2. State Management (`src/state/`)

#### ScanState (`src/state/scan_state.rs`)

**Purpose**: Shared state between scanner and TUI

**Type**: `Arc<RwLock<ScanState>>` (thread-safe, shared ownership)

**Fields**:
```rust
pub struct ScanState {
    pub stage: ScanStage,              // Current scan phase
    pub progress_percentage: f32,       // 0.0 - 100.0
    pub completed: u64,                 // Ports scanned
    pub total: u64,                     // Total ports
    pub open_ports: usize,              // Open ports found
    pub closed_ports: usize,            // Closed ports
    pub filtered_ports: usize,          // Filtered ports
    pub detected_services: usize,       // Services detected
    pub errors: usize,                  // Error count
    pub discovered_hosts: Vec<IpAddr>,  // Live hosts
    pub warnings: Vec<String>,          // Warnings
}
```

**Access Pattern**:
```rust
// Read (many readers, non-blocking)
let state = scan_state.read();
let open_ports = state.open_ports;

// Write (exclusive, blocks readers)
let mut state = scan_state.write();
state.open_ports += 10;
```

#### UIState (`src/state/ui_state.rs`)

**Purpose**: Local TUI-only state (ephemeral, not shared)

**Type**: `UIState` (single-threaded, no locking needed)

**Fields**:
```rust
pub struct UIState {
    pub selected_pane: SelectedPane,           // Main | Help
    pub cursor_position: usize,                // Cursor position
    pub scroll_offset: usize,                  // Scroll offset
    pub input_buffer: String,                  // Text input
    pub show_help: bool,                       // Help visibility
    pub fps: f32,                              // Debug FPS counter
    pub aggregator_dropped_events: usize,      // Dropped event count
}
```

**Navigation Methods**:
```rust
pub fn next_pane(&mut self)      // Tab key
pub fn prev_pane(&mut self)      // Shift+Tab
pub fn toggle_help(&mut self)    // F1/? key
```

### 3. Event System (`src/events/`)

#### Event Aggregator (`src/events/aggregator.rs`)

**Purpose**: Rate limiting for high-frequency events (10K+/sec)

**Strategy**:
- **Aggregate**: Count PortFound, HostDiscovered, ServiceDetected (don't buffer individual events)
- **Buffer**: Store lifecycle events (ScanStarted, ScanCompleted, errors, warnings)
- **Flush**: Process batches every 16ms (60 FPS) to prevent UI overload

**Constants**:
```rust
const MAX_BUFFER_SIZE: usize = 1000;               // Drop events if exceeded
const MIN_EVENT_INTERVAL: Duration = 16ms;         // 60 FPS flush rate
```

**Event Statistics**:
```rust
pub struct EventStats {
    pub ports_found: usize,                        // Aggregated count
    pub hosts_discovered: usize,                   // Aggregated count
    pub services_detected: usize,                  // Aggregated count
    pub discovered_ips: HashMap<IpAddr, usize>,    // Deduplication
    pub total_events: usize,                       // Total processed
    pub dropped_events: usize,                     // Rate limit drops
}
```

**Methods**:
```rust
pub fn add_event(&mut self, event: ScanEvent) -> bool
pub fn should_flush(&self) -> bool
pub fn flush(&mut self) -> (Vec<ScanEvent>, EventStats)
pub fn stats(&self) -> &EventStats
```

**Performance**:
- **Throughput**: 10,000+ events/second
- **Latency**: 16ms max (60 FPS)
- **Memory**: ~1,000 events × event size (lifecycle only, aggregated events don't buffer)
- **Overhead**: ~100 bytes per event (estimate)

#### Event Loop (`src/events/loop.rs`)

**Purpose**: Coordinate keyboard, EventBus, and timer events

**Pattern**: `tokio::select!` for concurrent event handling

```rust
pub async fn process_events(
    event_bus: Arc<EventBus>,
    scan_state: Arc<RwLock<ScanState>>,
    ui_state: &mut UIState,
    event_rx: &mut mpsc::UnboundedReceiver<ScanEvent>,
    crossterm_rx: &mut EventStream,
    aggregator: &mut EventAggregator,
) -> LoopControl
```

**Event Handling**:
```rust
tokio::select! {
    // Keyboard events (Ctrl+C, quit, navigation)
    Some(Ok(crossterm_event)) = crossterm_rx.next() => {
        if matches!(crossterm_event, Event::Key(key) if key.code == KeyCode::Char('q')) {
            return LoopControl::Quit;
        }
        // ... other key handlers
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

            // Apply aggregated statistics
            let mut state = scan_state.write();
            state.open_ports += stats.ports_found;
            state.detected_services += stats.services_detected;
            // ... deduplication for discovered_hosts

            ui_state.aggregator_dropped_events = stats.dropped_events;
        }
    }
}
```

### 4. UI Rendering (`src/ui/`)

#### Layout (`src/ui/layout.rs`)

**Purpose**: Define TUI layout structure

**Layout Structure**:
```
┌─────────────────────────────────────────┐
│          Header (scan info)             │  10% height
├─────────────────────────────────────────┤
│                                         │
│         Main Area (results)             │  80% height
│                                         │
├─────────────────────────────────────────┤
│   Footer (help text, FPS, stats)        │  10% height
└─────────────────────────────────────────┘
```

**Key Functions**:
```rust
pub fn create_layout(area: Rect) -> Rc<[Rect]>
pub fn render_header(scan_state: &ScanState) -> Paragraph
pub fn render_main_area(scan_state: &ScanState) -> Paragraph
pub fn render_footer(ui_state: &UIState) -> Paragraph
pub fn render_help_screen() -> Paragraph
```

#### Renderer (`src/ui/renderer.rs`)

**Purpose**: Immediate mode rendering (60 FPS)

**Rendering Pipeline**:
```rust
pub fn render(frame: &mut Frame, scan_state: &ScanState, ui_state: &UIState) {
    // 1. Create layout chunks
    let chunks = layout::create_layout(frame.area());

    // 2. Render header
    frame.render_widget(layout::render_header(scan_state), chunks[0]);

    // 3. Render main area
    frame.render_widget(layout::render_main_area(scan_state), chunks[1]);

    // 4. Render footer
    frame.render_widget(layout::render_footer(ui_state), chunks[2]);

    // 5. Render help screen (overlay) if visible
    if ui_state.show_help {
        frame.render_widget(layout::render_help_screen(), frame.area());
    }
}
```

**Performance Budget**:
- **Frame time**: 16.67ms (60 FPS)
- **Rendering**: <5ms (ratatui diffing)
- **State access**: <1ms (read lock)
- **Event processing**: <10ms (aggregated)
- **Margin**: ~1ms for system overhead

### 5. Widgets (`src/widgets/`)

ProRT-IP TUI includes 7 production-ready widgets as of Sprint 6.2:

- **Phase 6.1 Widgets** (4): StatusBar, MainWidget, LogWidget, HelpWidget
- **Phase 6.2 Dashboard Widgets** (3): PortTableWidget, ServiceTableWidget, MetricsDashboardWidget

#### Component Trait (`src/widgets/component.rs`)

**Purpose**: Common interface for TUI components

**Trait Definition**:
```rust
pub trait Component {
    /// Render the component to a frame
    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// Handle keyboard input
    fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<()>;

    /// Update component state
    fn update(&mut self) -> anyhow::Result<()>;
}
```

---

#### StatusBar Widget **[Phase 6.1]**

**Purpose**: Header widget displaying scan metadata and overall progress

**Location**: `src/widgets/status_bar.rs`

**Features**:
- Scan stage indicator (Initializing, Scanning, Complete, Error)
- Target information (IP/CIDR range)
- Scan type display (SYN, Connect, UDP, etc.)
- Overall progress percentage
- Color-coded status: Green (active), Yellow (warning), Red (error)

**Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ ProRT-IP Scanner | Target: 192.168.1.0/24 | Type: SYN | 45% │
└─────────────────────────────────────────────────────────────┘
```

**Rendering**:
- Fixed height: 3 lines (10% of terminal)
- Horizontal layout with styled text spans
- Immediate mode: Full redraw every frame

**Tests**: Unit tests for status formatting, color selection

---

#### MainWidget **[Phase 6.1]**

**Purpose**: Central content area displaying scan results summary

**Location**: `src/widgets/main.rs`

**Features**:
- Live host count (discovered IPs)
- Port statistics (open/closed/filtered counts)
- Service detection summary
- Error/warning counters
- Scrollable content area

**Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ Discovered Hosts: 24                                        │
│                                                             │
│ Open Ports:     156                                         │
│ Closed Ports:   12,844                                      │
│ Filtered Ports: 0                                           │
│                                                             │
│ Services Detected: 89                                       │
│ Errors: 0                                                   │
└─────────────────────────────────────────────────────────────┘
```

**Rendering**:
- Variable height: 80% of terminal (main area)
- Paragraph widget with line-wrapped text
- Responsive layout (adjusts to terminal width)

**Tests**: Integration tests for state synchronization

---

#### LogWidget **[Phase 6.1]**

**Purpose**: Real-time event log with scrolling and filtering

**Location**: `src/widgets/log.rs`

**Features**:
- Circular buffer (1,000 most recent events)
- Timestamped log entries
- Event type filtering (Info, Warning, Error)
- Auto-scroll toggle (follow mode)
- Keyboard navigation (scroll, search)

**Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ [12:34:56] INFO   Port 80 open on 192.168.1.1              │
│ [12:34:57] INFO   Service detected: nginx 1.18.0           │
│ [12:34:58] WARN   Slow response from 192.168.1.10          │
│ [12:34:59] INFO   Port 443 open on 192.168.1.1             │
│ ...                                                         │
│ [Auto-scroll: ON] | Filters: ALL | Lines: 156/1000         │
└─────────────────────────────────────────────────────────────┘
```

**Keyboard Shortcuts**:
- `↑/↓` or `j/k`: Scroll up/down
- `Page Up/Down`: Scroll by page
- `Home/End`: Jump to start/end
- `a`: Toggle auto-scroll

**Rendering**:
- List widget with stateful scrolling
- Color-coded entries: Info=White, Warn=Yellow, Error=Red
- Performance: <5ms for 1,000 entries

**Tests**: Unit tests for circular buffer, filtering logic

---

#### HelpWidget **[Phase 6.1]**

**Purpose**: Overlay widget showing keyboard shortcuts and commands

**Location**: `src/widgets/help.rs`

**Features**:
- Comprehensive keybinding reference
- Grouped by category (Navigation, Filtering, Views)
- Centered popup overlay
- Semi-transparent background (Clear widget)

**Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│                     ┌───────────────────┐                   │
│                     │  Keyboard Shortcuts │                 │
│                     ├───────────────────┤                   │
│                     │ q, Ctrl+C - Quit   │                 │
│                     │ ?, F1     - Help   │                 │
│                     │ Tab       - Switch │                 │
│                     │ ↑/↓       - Scroll │                 │
│                     │ ...                │                 │
│                     │ Press ? to close   │                 │
│                     └───────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

**Rendering**:
- Block widget with border and title
- Centered with Popup constraint (50% width × 60% height)
- Rendered on top of main content (overlay)

**Tests**: Integration tests for toggle visibility

---

#### PortTableWidget **[Phase 6.2 - Sprint 6.2]**

**Purpose**: Real-time port discovery visualization with sortable table

**Location**: `src/widgets/port_table.rs` (~700 lines, 14 tests)

**Features**:
- **Data Source**: 1,000-entry circular buffer (PortDiscovery events from EventBus)
- **Columns** (6): Timestamp, IP Address, Port, State, Protocol, Scan Type
- **Multi-Column Sorting**: All 6 columns × ascending/descending (12 sort modes)
- **Triple Filtering**:
  - State filter: All, Open, Closed, Filtered
  - Protocol filter: All, TCP, UDP
  - Search: Fuzzy match on IP address or port number
- **Color Coding**: Open=Green, Closed=Red, Filtered=Yellow
- **Auto-Scroll**: Follow mode for live discoveries

**Data Structure**:
```rust
pub struct PortDiscovery {
    pub timestamp: DateTime<Utc>,
    pub ip: IpAddr,
    pub port: u16,
    pub state: PortState,    // Open, Closed, Filtered
    pub protocol: Protocol,  // TCP, UDP
    pub scan_type: ScanType, // SYN, Connect, UDP, etc.
}
```

**Keyboard Shortcuts**:
- `t`: Sort by timestamp
- `i`: Sort by IP address
- `p`: Sort by port number
- `s`: Sort by state
- `r`: Sort by protocol
- `c`: Sort by scan type
- `a`: Toggle auto-scroll
- `f`: Cycle state filter
- `d`: Cycle protocol filter
- `/`: Search mode (IP or port)
- `↑/↓`: Navigate rows
- `Page Up/Down`: Scroll by page

**Rendering Performance**:
- Frame time: <5ms for 1,000 entries
- Table widget with stateful selection
- Row highlighting for selected item
- Header with sort direction indicators (▲/▼)

**Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ Timestamp    │ IP Address    │ Port │ State │ Proto │ Type │
├──────────────┼───────────────┼──────┼───────┼───────┼──────┤
│ 12:34:56.123 │ 192.168.1.1   │   80 │ Open  │ TCP   │ SYN  │
│ 12:34:56.234 │ 192.168.1.1   │  443 │ Open  │ TCP   │ SYN  │
│ 12:34:56.345 │ 192.168.1.2   │   22 │ Filt  │ TCP   │ SYN  │
│ 12:34:56.456 │ 192.168.1.3   │   53 │ Open  │ UDP   │ UDP  │
│ ...                                                         │
│ [Sort: Port ▲] | [Filter: Open+TCP] | 156/1000 | Auto: ON │
└─────────────────────────────────────────────────────────────┘
```

**Integration**:
- Subscribes to EventBus for PortDiscovery events
- Updates ringbuffer in real-time (no blocking)
- Renders in Dashboard Tab 1 (Port Table view)

**Tests** (14):
- Sorting: 6 columns × 2 directions = 12 tests
- Filtering: State filter, protocol filter (2 tests)
- Unit tests only (integration via App event loop)

---

#### ServiceTableWidget **[Phase 6.2 - Sprint 6.2]**

**Purpose**: Real-time service detection visualization with confidence scoring

**Location**: `src/widgets/service_table.rs` (~700 lines, 21 tests)

**Features**:
- **Data Source**: 500-entry circular buffer (ServiceDetection events from EventBus)
- **Columns** (6): Timestamp, IP Address, Port, Service Name, Version, Confidence
- **Confidence-Based Color Coding**:
  - High (≥90%): Green
  - Medium (50-89%): Yellow
  - Low (<50%): Red
- **Multi-Level Filtering**: All, Low (≥50%), Medium (≥75%), High (≥90%)
- **Sorting**: All 6 columns with ascending/descending
- **Search**: Fuzzy match on service name, IP, or port
- **Auto-Scroll**: Follow mode for live detections

**Data Structure**:
```rust
pub struct ServiceDetection {
    pub timestamp: DateTime<Utc>,
    pub ip: IpAddr,
    pub port: u16,
    pub service_name: String,     // "nginx", "ssh", "mysql"
    pub service_version: String,  // "1.18.0", "OpenSSH_8.2p1"
    pub confidence: u8,            // 0-100 (detection confidence %)
}
```

**Keyboard Shortcuts**:
- `1`: Sort by timestamp
- `2`: Sort by IP address
- `3`: Sort by port
- `4`: Sort by service name
- `5`: Sort by service version
- `6`: Sort by confidence
- `c`: Cycle confidence filter (All → Low → Medium → High)
- `a`: Toggle auto-scroll
- `/`: Search mode (service, IP, or port)
- `↑/↓`: Navigate rows
- `Page Up/Down`: Scroll by page

**Rendering Performance**:
- Frame time: <5ms for 500 entries
- Table widget with confidence color styling
- Row-level color application based on confidence
- Header with sort indicators and active filter

**Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ Time     │ IP Address    │ Port │ Service │ Version  │ Conf │
├──────────┼───────────────┼──────┼─────────┼──────────┼──────┤
│ 12:34:56 │ 192.168.1.1   │   80 │ nginx   │ 1.18.0   │ 95%  │ (Green)
│ 12:34:57 │ 192.168.1.1   │  443 │ nginx   │ 1.18.0   │ 95%  │ (Green)
│ 12:34:58 │ 192.168.1.2   │   22 │ ssh     │ OpenSSH…│ 72%  │ (Yellow)
│ 12:34:59 │ 192.168.1.3   │ 3306 │ mysql   │ 5.7.31   │ 42%  │ (Red)
│ ...                                                         │
│ [Sort: Conf ▼] | [Filter: High ≥90%] | 89/500 | Auto: ON  │
└─────────────────────────────────────────────────────────────┘
```

**Integration**:
- Subscribes to EventBus for ServiceDetection events
- Confidence calculation from service detection engine
- Renders in Dashboard Tab 2 (Service Table view)

**Tests** (21):
- Sorting: 6 columns × 2 directions = 12 tests
- Filtering: 4 confidence levels = 4 tests
- Color coding: 3 confidence ranges = 3 tests
- Search: 2 tests (service name, port)

---

#### MetricsDashboardWidget **[Phase 6.2 - Sprint 6.2]**

**Purpose**: Real-time performance metrics and scan statistics

**Location**: `src/widgets/metrics_dashboard.rs` (~740 lines, 24 tests)

**Features**:
- **3-Column Layout**: Progress | Throughput | Statistics
- **Progress Column**:
  - Scan percentage (0-100%)
  - Completed/total ports
  - ETA calculation (based on 5-second rolling average)
  - Scan stage indicator
- **Throughput Column**:
  - Current ports/second (instantaneous)
  - Average ports/second (5-second window)
  - Peak ports/second (session max)
  - Current packets/second
  - Average packets/second
- **Statistics Column**:
  - Open ports count
  - Detected services count
  - Error count
  - Scan duration (HH:MM:SS format)
  - Status indicator (Active, Paused, Error)

**Data Source**: Reads from `Arc<RwLock<ScanState>>` shared state

**Throughput Calculation**:
```rust
// 5-second rolling average
pub struct ThroughputSample {
    pub timestamp: Instant,
    pub ports_per_second: f64,
    pub packets_per_second: f64,
}

// Keep last 5 samples (1-second intervals)
// Average = sum(samples) / sample_count
```

**Human-Readable Formatting**:
- **Durations**: "1h 12m 45s", "23m 15s", "45s"
- **Numbers**: "12,345" (comma separators)
- **Throughput**: "1.23K pps", "456.7 pps", "12.3M pps"

**Color Coding**:
- **Status**: Green (Active), Yellow (Paused), Red (Error)
- **ETA**: White (normal), Yellow (>1h remaining), Red (stalled)
- **Throughput**: Green (≥target), Yellow (50-99%), Red (<50%)

**Keyboard Shortcuts**:
- No interactive controls (read-only metrics display)
- Auto-updates every frame (60 FPS)

**Rendering Performance**:
- Frame time: <5ms (3× under 16.67ms budget)
- 3 Block widgets with bordered panels
- Paragraph widgets for metric text
- Gauge widget for progress percentage

**Layout**:
```
┌──────────────────┬──────────────────┬──────────────────┐
│    PROGRESS      │   THROUGHPUT     │   STATISTICS     │
├──────────────────┼──────────────────┼──────────────────┤
│ Scan:       45%  │ Current:  1.2K/s │ Open Ports:  156 │
│ ████████░░░░░░░░ │ Average:  1.1K/s │ Services:     89 │
│ Completed: 2,925 │ Peak:     2.3K/s │ Errors:        0 │
│ Total:     6,500 │                  │ Duration: 2m 15s │
│ ETA:      5m 12s │ Packets:  4.5K/s │ Status:   Active │
│ Stage: Scanning  │ Avg Pkt:  4.2K/s │                  │
└──────────────────┴──────────────────┴──────────────────┘
```

**Integration**:
- Reads shared `ScanState` every frame
- Calculates derived metrics (ETA, averages, throughput)
- Renders in Dashboard Tab 3 (Metrics view)

**Tests** (24):
- Throughput calculations: 5 tests
- ETA calculations: 5 tests
- Human-readable formatting: 8 tests (durations, numbers, throughput)
- Color selection: 3 tests (status, ETA, throughput)
- Edge cases: 3 tests (zero values, overflow, no data)

---

#### Tabbed Dashboard Interface **[Phase 6.2 - Sprint 6.2]**

**Architecture**: 3-tab dashboard with keyboard navigation

**Tab Switching**:
```rust
pub enum DashboardTab {
    PortTable,      // Tab 1: Real-time port discoveries
    ServiceTable,   // Tab 2: Service detection results
    Metrics,        // Tab 3: Performance metrics
}

impl UIState {
    pub fn switch_tab(&mut self) {
        self.active_tab = match self.active_tab {
            DashboardTab::PortTable => DashboardTab::ServiceTable,
            DashboardTab::ServiceTable => DashboardTab::Metrics,
            DashboardTab::Metrics => DashboardTab::PortTable,  // Cycle
        };
    }
}
```

**Keyboard Shortcuts**:
- `Tab`: Switch to next dashboard (Port → Service → Metrics → Port)
- `Shift+Tab`: Switch to previous dashboard (reverse direction)

**Rendering**:
- Tabs widget at top (shows all 3 tab names, highlights active)
- Conditional rendering: Only active tab widget is rendered
- Each widget receives full dashboard area (80% of terminal)

**Visual Tab Bar**:
```
┌─────────────────────────────────────────────────────────────┐
│ [Port Table] | Service Table | Metrics                      │
├─────────────────────────────────────────────────────────────┤
│ [Active Dashboard Widget Content]                           │
│ ...                                                          │
└─────────────────────────────────────────────────────────────┘
```

**Event Routing**:
- Active tab receives keyboard events (sorting, filtering, navigation)
- Inactive tabs do not process events (performance optimization)

**Integration Tests** (3):
- Tab switching sequence (Port → Service → Metrics → Port)
- Event routing to active tab only
- State persistence across tab switches

---

**Future Components** (Phase 6.3+):
- **ChartWidget**: Sparkline throughput graph (real-time performance visualization)
- **ConfigWidget**: Interactive scan parameter editor (pause, adjust timing, change targets)
- **ExportWidget**: Live results export to JSON/XML/CSV during scan

---

## Event Flow

### 1. Scanner → EventBus → TUI (Enhanced Sprint 6.6)

**Sprint 6.6 Enhancement:** Added lifecycle event publishing (ScanStarted, StageChanged, ScanCompleted) to enable live TUI updates during scan execution.

```
Scanner Thread                EventBus               TUI Thread
──────────────                ────────               ──────────

scan_initialization()
    │
    │ publishes ScanStarted (NEW in 6.6)
    ├──────────────────────▶ broadcast ─────────────▶ event_rx.recv()
    │                                                       │
    │                                                       ▼
    │                                                 aggregator.add_event()
    │                                                       │ (lifecycle event buffered)
    │                                                       ▼
port_scan()
    │
    │ publishes PortFound
    ├──────────────────────▶ broadcast ─────────────▶ event_rx.recv()
    │                                                       │
    │                                                       ▼
    │                                                 aggregator.add_event()
    │                                                       │
    │                                                       │ (aggregates, no processing)
    │                                                       ▼
    │                                                 (buffered)
    │
    │ publishes HostDiscovered
    ├──────────────────────▶ broadcast ─────────────▶ event_rx.recv()
    │                                                       │
    │                                                       ▼
    │                                                 aggregator.add_event()
    │                                                       │
    │                                                       ▼
    │                                                 (buffered)
    │
stage_transition()
    │
    │ publishes StageChanged (NEW in 6.6)
    ├──────────────────────▶ broadcast ─────────────▶ event_rx.recv()
    │                                                       │
    │                                                       ▼
    │                                                 aggregator.add_event()
    │                                                       │ (lifecycle event buffered)
    │                                                       ▼
[16ms passes]
    │                                                 tick_interval.tick()
    │                                                       │
    │                                                       ▼
    │                                                 aggregator.should_flush()
    │                                                       │ (true)
    │                                                       ▼
    │                                                 flush() → (events, stats)
    │                                                       │
    │                                                       ▼
    │                                                 update scan_state
    │                                                       │
    │                                                       ▼
    │                                                 terminal.draw(render)
    │
scan_completion()
    │
    │ publishes ScanCompleted (NEW in 6.6)
    ├──────────────────────▶ broadcast ─────────────▶ event_rx.recv()
    │                                                       │
    │                                                       ▼
    │                                                 aggregator.add_event()
    │                                                       │ (lifecycle event buffered)
    │                                                       ▼
```

**New Lifecycle Events (Sprint 6.6):**

- **ScanStarted**: Published at scan initialization, includes target info and scan type
- **StageChanged**: Published when scanner transitions between phases (Discovery → Enumeration → Deep Inspection)
- **ScanCompleted**: Published when scan finishes, includes final statistics

### 2. Keyboard Input Flow

```
Terminal             crossterm            TUI Event Loop            State
────────             ─────────            ──────────────            ─────

User presses 'q'
    │
    ├──────────▶ EventStream.next()
    │                  │
    │                  ├──────────────▶ process_events()
    │                  │                      │
    │                  │                      │ matches KeyCode::Char('q')
    │                  │                      ▼
    │                  │                return LoopControl::Quit
    │                  │                      │
    │                  │                      ▼
    │                  │                App::run() breaks loop
    │                  │                      │
    │                  │                      ▼
    │                  │                ratatui::restore()
    │                  │                      │
    │                  │                      ▼
    │                  │                Terminal restored
    │
```

### 3. Event Aggregation Example

**Scenario**: Scanner finds 1,000 open ports in 10ms

```
Time    Event            Aggregator State               TUI State
────    ─────            ────────────────               ─────────
0ms     PortFound #1     stats.ports_found = 1          (no update)
1ms     PortFound #2     stats.ports_found = 2          (no update)
2ms     PortFound #3     stats.ports_found = 3          (no update)
...     ...              ...                            ...
10ms    PortFound #1000  stats.ports_found = 1000       (no update)

16ms    (timer tick)     flush()                        scan_state.open_ports += 1000
                         stats.ports_found = 0 (reset)  terminal.draw(render)

        UI displays: "Open Ports: 1000" (single update, no lag)
```

**Without Aggregation**: 1,000 state updates, 1,000 renders, UI freezes

**With Aggregation**: 1 batch update, 1 render, smooth 60 FPS

---

## Performance Characteristics

### Throughput

| Metric | Target | Achieved | Notes |
|--------|--------|----------|-------|
| **FPS** | 60 | 60 | 16.67ms frame budget |
| **Event Rate** | 10,000/sec | 10,000+ | Event aggregation |
| **Latency** | <100ms | 16ms | Max aggregation delay |
| **Memory** | <10MB | ~5MB | Event buffer + state |
| **CPU** | <10% | ~5% | Rendering overhead |

### Event Aggregation Performance

**Test**: 10,000 PortFound events in 1 second

```
Without Aggregation:
- Events: 10,000
- State Updates: 10,000
- Renders: 10,000 (impossible at 60 FPS)
- Result: UI freezes, dropped frames

With Aggregation (16ms interval):
- Events: 10,000
- Batches: 62 (1000ms / 16ms)
- State Updates: 62
- Renders: 60 (capped at 60 FPS)
- Result: Smooth UI, no dropped frames
```

### Memory Usage

**Sprint 6.6 Enhancement:** Memory-mapped I/O (mmap) for scan results reduces RAM footprint by 77-86% during internet-scale scans.

```
Component              Size           Notes
─────────              ────           ─────
ScanState              ~1 KB          Arc<RwLock<T>>
UIState                ~100 bytes     Stack-allocated
EventAggregator        ~100 KB        1,000 × ~100 bytes/event
Event Buffer           ~100 KB        MAX_BUFFER_SIZE
Terminal Buffer        ~10 KB         ratatui screen buffer
MmapResultWriter       Variable       Disk-backed (mmap mode only)

Total: ~211 KB + mmap (negligible overhead)
```

**Memory-Mapped I/O (Sprint 6.6):**

ProRT-IP supports two result storage modes:

1. **Memory Mode** (default for small scans):
   - Results stored in `Vec<ScanResult>`
   - Fast random access (O(1))
   - Limited by available RAM (~10M results max)

2. **Mmap Mode** (automatic for large scans):
   - Results streamed to memory-mapped file
   - Auto-grows in 1MB increments
   - Zero-copy reads via `MmapResultReader`
   - **77-86% RAM reduction** (100K results: 35MB → 5MB)
   - Transparent to TUI (same API)

**Auto-Switching Threshold:**
- Default: >10,000 expected results → mmap mode
- Configurable via `--mmap-threshold N`

**Implementation:**
- **MmapResultWriter** (124 lines): bincode serialization, auto-growth
- **MmapResultReader** (219 lines): zero-copy iteration, index offsets
- **ResultWriter enum**: Memory vs Mmap mode abstraction

### CPU Profiling

```
Component              % CPU          Notes
─────────              ─────          ─────
Event Processing       ~2%            Aggregation logic
State Updates          ~1%            RwLock write overhead
Rendering (ratatui)    ~3%            Diffing + terminal I/O
Keyboard Handling      <1%            Rare events
System Overhead        ~1%            tokio runtime

Total: ~8% CPU (on modern CPU at 10,000 events/sec)
```

---

## State Management

### Shared State Pattern

**Challenge**: Scanner (background thread) needs to update state while TUI (main thread) reads it

**Solution**: `Arc<RwLock<ScanState>>`

```rust
// Scanner thread (writer)
let state = scan_state.write();  // Exclusive lock (blocks readers)
state.open_ports += 1;
drop(state);                      // Release lock ASAP

// TUI thread (reader)
let state = scan_state.read();   // Shared lock (many readers)
let open_ports = state.open_ports;
drop(state);                      // Release lock ASAP
```

**Best Practices**:
1. **Hold locks briefly**: Read/write data, then drop lock immediately
2. **Avoid nested locks**: Prevents deadlocks
3. **Batch updates**: Write multiple fields in single lock acquisition
4. **Read consistency**: Take read lock once per frame, copy data to local vars

### Lock Contention Mitigation

**Problem**: High-frequency writes can block readers (UI stutters)

**Solution 1**: Event aggregation (batch writes every 16ms)

```rust
// Before: 1,000 writes/second (1ms each = UI blocked)
for port in ports {
    let mut state = scan_state.write();  // LOCK
    state.open_ports += 1;               // WRITE
}                                        // UNLOCK

// After: 60 writes/second (16ms batches = smooth UI)
let (events, stats) = aggregator.flush();
let mut state = scan_state.write();      // LOCK ONCE
state.open_ports += stats.ports_found;   // BATCH WRITE
drop(state);                              // UNLOCK
```

**Solution 2**: `parking_lot::RwLock` (faster than std::sync::RwLock)

- **Fast path**: Lock-free reads when no writers
- **Writer priority**: Prevents writer starvation
- **Benchmarks**: 2-3× faster than std::sync::RwLock

---

## Terminal Lifecycle

### Initialization

```rust
// 1. Initialize terminal (ratatui 0.29+ handles panic hook automatically)
let mut terminal = ratatui::init();

// What this does internally:
// - crossterm::terminal::enable_raw_mode()
// - crossterm::execute!(stdout, EnterAlternateScreen)
// - Set panic hook for cleanup
```

### Normal Exit

```rust
// 2. Restore terminal on normal exit
ratatui::restore();

// What this does internally:
// - crossterm::execute!(stdout, LeaveAlternateScreen)
// - crossterm::terminal::disable_raw_mode()
```

### Panic Recovery

```rust
// ratatui 0.29+ automatically handles panic restoration
// No manual cleanup needed!

// Before (manual):
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |panic_info| {
    ratatui::restore();
    original_hook(panic_info);
}));

// After (automatic):
// ratatui::init() handles this for you
```

### Ctrl+C Handling

```rust
// Keyboard event loop detects Ctrl+C
tokio::select! {
    Some(Ok(Event::Key(key))) = crossterm_rx.next() => {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return LoopControl::Quit;  // Graceful exit
                }
                // ...
            }
        }
    }
}

// Main loop breaks, terminal restored in App::run() cleanup
```

---

## Testing Strategy

### Unit Tests (4 tests)

**Location**: `src/events/aggregator.rs`

**Coverage**:
- Event aggregation logic (count PortFound, HostDiscovered)
- Buffer limit enforcement (drop events when full)
- Flush behavior (reset statistics after flush)
- Deduplication (unique IPs in discovered_ips)

**Example**:
```rust
#[test]
fn test_aggregator_buffer_limit() {
    let mut agg = EventAggregator::new();

    // Fill buffer to MAX_BUFFER_SIZE
    for i in 0..MAX_BUFFER_SIZE {
        assert!(agg.add_event(ScanEvent::ProgressUpdate { ... }));
    }

    // Next event should be dropped
    assert!(!agg.add_event(ScanEvent::ProgressUpdate { ... }));
    assert_eq!(agg.stats().dropped_events, 1);
}
```

### Integration Tests (15 tests)

**Location**: `tests/integration_test.rs`

**Coverage**:
- App creation and lifecycle
- ScanState initialization and shared state
- UIState pane navigation, help toggle, cursor movement
- EventAggregator timing (16ms flush interval)
- EventBus subscription (async test)
- Multiple apps sharing state

**Example**:
```rust
#[test]
fn test_scan_state_shared() {
    let state1 = ScanState::shared();
    let state2 = Arc::clone(&state1);

    // Modify via state1
    { let mut s = state1.write(); s.open_ports = 10; }

    // Read via state2 (sees changes)
    { let s = state2.read(); assert_eq!(s.open_ports, 10); }
}
```

### Doctests (2 passing, 1 ignored)

**Location**: Inline in source code

**Coverage**:
- App::new() example
- Crate-level example (lib.rs)
- Component trait (ignored until implementation)

**Example**:
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
```

### Test Metrics

**Phase 6.2 (Sprint 6.2 Complete):**

```
Test Type         Count    Status    Coverage
─────────         ─────    ──────    ────────
Unit Tests        140      ✓ Pass    Aggregator, Widgets (59 dashboard widget tests)
Integration       25       ✓ Pass    App, State, Events, Tab switching
Doctests          2        ✓ Pass    Public API examples
                  1        Ignored   Future Component trait

Total             168      165 Pass  Comprehensive (3 ignored)
```

**Widget Test Breakdown** (59 unit tests):
- PortTableWidget: 14 tests (sorting, filtering)
- ServiceTableWidget: 21 tests (sorting, filtering, color coding)
- MetricsDashboardWidget: 24 tests (calculations, formatting, edge cases)

---

## Future Enhancements

### Phase 6.2: Advanced Widgets (Q1 2026)

**Goal**: Implement production-ready TUI components

**Components**:
1. **MainWidget**: Sortable port table with service info
2. **StatusBar**: Real-time progress bar with ETA
3. **LogWidget**: Scrollable event log with filtering
4. **ChartWidget**: Throughput graph (sparkline)

**Implementation Plan**:
- Use `tui-textarea` for scrollable content
- Implement `Component` trait for all widgets
- Add keyboard navigation (arrow keys, Page Up/Down)
- Support terminal resize events

### Phase 6.3: Advanced Features (Q2 2026)

**Features**:
1. **Interactive Mode**: Pause/resume scans, adjust parameters
2. **Export**: Save results to JSON/XML during scan
3. **Themes**: Customizable color schemes
4. **Plugins**: Lua scripts for custom widgets

### Phase 6.4: Performance Optimization (Q3 2026)

**Optimizations**:
1. **Zero-copy Rendering**: Avoid cloning state in render loop
2. **Incremental Rendering**: Only redraw changed areas
3. **GPU Acceleration**: kitty/wezterm graphics protocol
4. **Compression**: Compress event buffer for low-memory systems

---

## References

### Documentation

- **ratatui**: https://ratatui.rs/
- **crossterm**: https://docs.rs/crossterm/
- **tokio::select!**: https://docs.rs/tokio/latest/tokio/macro.select.html
- **parking_lot**: https://docs.rs/parking_lot/

### ProRT-IP Documentation

- **00-ARCHITECTURE.md**: Overall system design
- **01-ROADMAP.md**: Phase 6 TUI development plan
- **10-PROJECT-STATUS.md**: Current sprint status

### Code Organization

```
crates/prtip-tui/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── app.rs              # App lifecycle
│   ├── state/
│   │   ├── mod.rs          # State re-exports
│   │   ├── scan_state.rs   # Shared scanner state
│   │   └── ui_state.rs     # Local TUI state
│   ├── events/
│   │   ├── mod.rs          # Event re-exports
│   │   ├── aggregator.rs   # Event rate limiting
│   │   ├── loop.rs         # Event loop coordination
│   │   └── handlers.rs     # Keyboard handlers
│   ├── ui/
│   │   ├── mod.rs          # UI re-exports
│   │   ├── renderer.rs     # 60 FPS rendering
│   │   ├── layout.rs       # Layout functions
│   │   └── theme.rs        # Color schemes
│   └── widgets/
│       ├── mod.rs          # Widget re-exports
│       ├── component.rs    # Component trait
│       ├── main.rs         # Main area widget (Phase 6.2)
│       ├── status.rs       # Status bar (Phase 6.2)
│       └── help.rs         # Help screen (Phase 6.2)
├── tests/
│   └── integration_test.rs # Integration tests
├── Cargo.toml              # Dependencies
└── README.md               # Crate overview
```

---

## Glossary

- **Immediate Mode Rendering**: Full UI redrawn every frame, framework diffs and updates
- **Event Aggregation**: Batching high-frequency events to prevent UI overload
- **60 FPS**: 60 frames per second (16.67ms per frame)
- **RwLock**: Read-write lock (many readers, one writer)
- **Arc**: Atomic reference counting (shared ownership across threads)
- **EventBus**: Publish-subscribe event system
- **ratatui**: Rust TUI framework (fork of tui-rs)
- **crossterm**: Cross-platform terminal manipulation
- **tokio::select!**: Concurrent event handling macro

---

**End of TUI Architecture Documentation**

*For questions or feedback, see ProRT-IP repository issues.*
