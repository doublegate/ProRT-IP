# Real-Time Dashboard

**60 FPS live scan visualization with multi-tab interface and interactive widgets.**

## What is the Real-Time Dashboard?

**Real-Time Dashboard** provides live visualization of network scanning operations through a terminal user interface (TUI). Instead of waiting for scan completion, you watch discoveries unfold in real-time through interactive widgets displaying ports, services, and performance metrics.

**ProRT-IP Implementation:**
- **60 FPS Rendering** - Smooth updates with <5ms frame time (16.67ms budget)
- **10,000+ Events/Second** - Handles high-throughput scanning without UI lag
- **4-Tab Interface** - Port discoveries, service detection, performance metrics, network graph
- **7 Production Widgets** - StatusBar, MainWidget, LogWidget, HelpWidget, PortTableWidget, ServiceTableWidget, MetricsDashboardWidget
- **Event-Driven Architecture** - Scanner publishes to EventBus, TUI subscribes (no coupling)
- **Graceful Degradation** - Clean terminal restoration on all exit paths (normal, Ctrl+C, panic)

**Use Cases:**
- **Live Monitoring** - Watch port discoveries and service detection as they happen
- **Performance Analysis** - Track throughput, ETA, and scan progress in real-time
- **Troubleshooting** - Identify slow targets, errors, and filtered ports immediately
- **Demonstrations** - Impressive live visualization for security awareness training

---

## How It Works

### Architecture Overview

ProRT-IP's TUI uses a **consumer-only design** where the scanner publishes events to an EventBus and the TUI subscribes to updates:

```
┌─────────────────────────────────────────────┐
│          ProRT-IP Scanner                   │
│       (prtip-core, no TUI deps)             │
└────────────────┬────────────────────────────┘
                 │ publishes
                 ▼
┌─────────────────────────────────────────────┐
│              EventBus                       │
│   (mpsc::unbounded_channel, broadcast)      │
└────────────────┬────────────────────────────┘
                 │ subscribe
                 ▼
┌─────────────────────────────────────────────┐
│           TUI Event Loop                    │
│        (tokio::select! pattern)             │
│                                             │
│  ┌───────────┐  ┌───────────┐  ┌──────────┐│
│  │ Keyboard  │  │EventBus RX│  │60 FPS    ││
│  │(crossterm)│  │(scan evts)│  │Timer     ││
│  └─────┬─────┘  └─────┬─────┘  └────┬─────┘│
│        │              │              │      │
│        ▼              ▼              ▼      │
│    Key Handler   Event Aggregator  Render  │
└─────────────────────────────────────────────┘
```

**Key Design Principles:**
1. **Separation of Concerns** - Scanner has zero TUI dependencies
2. **Immediate Mode Rendering** - Full UI redrawn every frame (ratatui diffs terminal updates)
3. **Event Aggregation** - Batch 10,000+ high-frequency events/sec into 60 FPS updates
4. **Thread-Safe State** - `Arc<RwLock<ScanState>>` for scanner ↔ TUI communication

### Technology Stack

**Framework:**
- **ratatui 0.29+** - Modern TUI framework with automatic panic recovery
- **crossterm 0.28** - Cross-platform terminal manipulation (Windows/Linux/macOS)
- **tokio** - Async runtime for event loop coordination

**Performance:**
- **parking_lot::RwLock** - High-performance read-write locks (2-3× faster than std)
- **Lock-free queues** - crossbeam for event buffering
- **Event aggregation** - 16ms batching prevents UI overload

### Event Flow

**High-Frequency Events** (aggregated):
```
Scanner Thread          EventBus             TUI Thread
──────────────          ────────             ──────────

port_scan()
    │ publishes 1,000 PortFound events in 10ms
    ├──────────────▶ broadcast ──────────▶ aggregator.add_event()
    │                                           │ (count only, no render)
    │                                           ▼
[16ms passes]                             tick_interval.tick()
    │                                           │
    │                                           ▼
    │                                     aggregator.flush()
    │                                           │ stats.ports_found = 1000
    │                                           ▼
    │                                     scan_state.open_ports += 1000
    │                                           │
    │                                           ▼
    │                                     terminal.draw(render)

UI displays: "Open Ports: 1000" (1 update, smooth 60 FPS)
```

**Without aggregation**: 1,000 state updates, 1,000 renders, UI freezes
**With aggregation**: 1 batch update, 1 render at 60 FPS

---

## Usage

### Launching TUI Mode

**Basic Usage:**
```bash
# Enable TUI mode with --tui flag
prtip --tui -sS -p 1-1000 192.168.1.0/24
```

**Combined with Other Options:**
```bash
# TUI + service detection + OS fingerprinting
prtip --tui -sV -O -p 1-1000 scanme.nmap.org

# TUI + aggressive scan + output to file
prtip --tui -A -p- 192.168.1.0/24 -oJ results.json

# TUI + stealth scan + slow timing
prtip --tui -sS -T2 -p 80,443 target.com
```

**Expected Output:**
```
┌─────────────────────────────────────────────────────────────┐
│ ProRT-IP Scanner | Target: 192.168.1.0/24 | Type: SYN | 45% │
├─────────────────────────────────────────────────────────────┤
│ [Port Table] | Service Table | Metrics                      │
├─────────────────────────────────────────────────────────────┤
│ Timestamp    │ IP Address    │ Port │ State │ Proto │ Type │
├──────────────┼───────────────┼──────┼───────┼───────┼──────┤
│ 12:34:56.123 │ 192.168.1.1   │   80 │ Open  │ TCP   │ SYN  │
│ 12:34:56.234 │ 192.168.1.1   │  443 │ Open  │ TCP   │ SYN  │
│ 12:34:56.345 │ 192.168.1.2   │   22 │ Filt  │ TCP   │ SYN  │
│ ...                                                         │
├─────────────────────────────────────────────────────────────┤
│ Press ? for help | Tab: Switch view | q: Quit | FPS: 60    │
└─────────────────────────────────────────────────────────────┘
```

### Keyboard Shortcuts

**Global Navigation:**
- `q` or `Ctrl+C` - Quit TUI and exit program
- `?` or `F1` - Toggle help screen
- `Tab` - Switch to next dashboard tab (Port Table → Service Table → Metrics → Port Table)
- `Shift+Tab` - Switch to previous dashboard tab (reverse direction)

**Port Table Tab:**
- `t` - Sort by timestamp
- `i` - Sort by IP address
- `p` - Sort by port number
- `s` - Sort by state (Open/Closed/Filtered)
- `r` - Sort by protocol (TCP/UDP)
- `c` - Sort by scan type
- `a` - Toggle auto-scroll (follow live discoveries)
- `f` - Cycle state filter (All → Open → Closed → Filtered)
- `d` - Cycle protocol filter (All → TCP → UDP)
- `/` - Search mode (type IP address or port number)
- `↑/↓` or `j/k` - Navigate rows
- `Page Up/Down` - Scroll by page
- `Home/End` - Jump to start/end

**Service Table Tab:**
- `1` - Sort by timestamp
- `2` - Sort by IP address
- `3` - Sort by port
- `4` - Sort by service name
- `5` - Sort by service version
- `6` - Sort by confidence (detection accuracy)
- `c` - Cycle confidence filter (All → Low ≥50% → Medium ≥75% → High ≥90%)
- `a` - Toggle auto-scroll
- `/` - Search mode (service name, IP, or port)
- `↑/↓` - Navigate rows
- `Page Up/Down` - Scroll by page

**Metrics Tab:**
- No interactive controls (read-only dashboard)
- Auto-updates every frame (60 FPS)

### Interpreting the Dashboard

#### Port Table Tab

**Purpose:** Real-time port discovery visualization

**Columns:**
1. **Timestamp** - Exact discovery time (HH:MM:SS.mmm)
2. **IP Address** - Target host (IPv4 or IPv6)
3. **Port** - Port number (1-65535)
4. **State** - Open (green), Closed (red), Filtered (yellow)
5. **Protocol** - TCP or UDP
6. **Type** - Scan type (SYN, Connect, UDP, etc.)

**Status Bar:**
```
[Sort: Port ▲] | [Filter: Open+TCP] | 156/1000 | Auto: ON
```
- **Sort indicator** - Current sort column and direction (▲ ascending, ▼ descending)
- **Active filters** - State and protocol filters applied
- **Entry count** - Visible entries / total buffer size (max 1,000)
- **Auto-scroll** - Follow mode status (ON = scroll with new discoveries)

**Color Coding:**
- **Green rows** - Open ports (service likely accepting connections)
- **Red rows** - Closed ports (port accessible but not listening)
- **Yellow rows** - Filtered ports (firewall blocking access)

#### Service Table Tab

**Purpose:** Service detection results with confidence scoring

**Columns:**
1. **Timestamp** - Detection time (HH:MM:SS)
2. **IP Address** - Target host
3. **Port** - Service port
4. **Service** - Detected service name ("nginx", "ssh", "mysql")
5. **Version** - Software version ("1.18.0", "OpenSSH 8.2p1")
6. **Confidence** - Detection accuracy (0-100%)

**Confidence Color Coding:**
- **Green (≥90%)** - High confidence (reliable detection)
- **Yellow (50-89%)** - Medium confidence (probable match)
- **Red (<50%)** - Low confidence (uncertain detection)

**Example:**
```
┌─────────────────────────────────────────────────────────────┐
│ Time     │ IP Address    │ Port │ Service │ Version  │ Conf │
├──────────┼───────────────┼──────┼─────────┼──────────┼──────┤
│ 12:34:56 │ 192.168.1.1   │   80 │ nginx   │ 1.18.0   │ 95%  │ (Green)
│ 12:34:57 │ 192.168.1.1   │  443 │ nginx   │ 1.18.0   │ 95%  │ (Green)
│ 12:34:58 │ 192.168.1.2   │   22 │ ssh     │ OpenSSH…│ 72%  │ (Yellow)
│ 12:34:59 │ 192.168.1.3   │ 3306 │ mysql   │ 5.7.31   │ 42%  │ (Red)
└─────────────────────────────────────────────────────────────┘
```

**Interpretation:**
- **nginx 95%** - Very reliable detection (trust this version)
- **ssh 72%** - Probable SSH server (verify if critical)
- **mysql 42%** - Uncertain (may be MySQL or similar database)

#### Metrics Tab

**Purpose:** Real-time performance metrics and scan statistics

**3-Column Layout:**
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

**Progress Column:**
- **Scan Percentage** - Overall completion (0-100%)
- **Progress Bar** - Visual gauge (filled blocks = complete, empty = remaining)
- **Completed/Total** - Ports scanned vs total ports
- **ETA** - Estimated time to completion (based on 5-second rolling average)
- **Stage** - Current scan phase (Initializing, Scanning, Complete, Error)

**Throughput Column:**
- **Current** - Instantaneous ports/second (last 1 second)
- **Average** - Mean ports/second (5-second rolling window)
- **Peak** - Maximum ports/second achieved during scan
- **Packets** - Current packets/second (instantaneous)
- **Avg Pkt** - Average packets/second (5-second window)

**Statistics Column:**
- **Open Ports** - Total open ports discovered
- **Services** - Total services detected
- **Errors** - Error count (timeouts, unreachable, etc.)
- **Duration** - Elapsed time since scan start (HH:MM:SS format)
- **Status** - Scan state (Active=green, Paused=yellow, Error=red)

**Human-Readable Formatting:**
- **Durations**: "1h 12m 45s", "23m 15s", "45s"
- **Numbers**: "12,345" (comma separators every 3 digits)
- **Throughput**: "1.23K pps" (thousands), "456.7 pps" (hundreds), "12.3M pps" (millions)

---

## Dashboard Tabs

### Tab 1: Port Table

**When to Use:**
- Monitor port discoveries in real-time
- Identify filtered ports (firewall detection)
- Track scan coverage across multiple hosts

**Key Features:**
- **1,000-entry circular buffer** - Most recent discoveries retained
- **Multi-column sorting** - Sort by any of 6 columns (timestamp, IP, port, state, protocol, type)
- **Triple filtering** - State (All/Open/Closed/Filtered), Protocol (All/TCP/UDP), Search (fuzzy match)
- **Auto-scroll** - Follow mode for live discoveries (toggle with `a`)

**Best Practices:**
1. **Enable auto-scroll** during active scans to see latest discoveries
2. **Sort by state** (`s` key) to group open/closed/filtered ports
3. **Filter by Open** (`f` key) to focus on interesting targets
4. **Search for specific ports** (`/` then type port number like "443")

### Tab 2: Service Table

**When to Use:**
- Identify services and versions (vulnerability assessment)
- Review detection confidence for critical services
- Track service detection progress

**Key Features:**
- **500-entry circular buffer** - Latest service detections
- **Confidence-based color coding** - Green (≥90%), Yellow (50-89%), Red (<50%)
- **Multi-level filtering** - All, Low (≥50%), Medium (≥75%), High (≥90%)
- **Version tracking** - Exact software versions for CVE matching

**Best Practices:**
1. **Filter by High confidence** (`c` key) to focus on reliable detections
2. **Sort by confidence** (`6` key) to review uncertain detections first
3. **Search for service names** (`/` then type "nginx" or "ssh")
4. **Cross-reference with Port Table** - Verify open ports have services detected

### Tab 3: Metrics Dashboard

**When to Use:**
- Monitor scan performance and throughput
- Estimate time to completion (ETA)
- Identify performance bottlenecks
- Track error rates

**Key Features:**
- **5-second rolling averages** - Smooth throughput metrics (not susceptible to spikes)
- **Automatic ETA calculation** - Based on recent progress rate
- **Color-coded status** - Green (Active), Yellow (Paused), Red (Error)
- **Session statistics** - Peak throughput, total errors, elapsed time

**Best Practices:**
1. **Watch throughput drop** - Indicates slow targets or network congestion
2. **Monitor error count** - Rising errors suggest firewall or rate limiting
3. **Check ETA color** - Yellow (>1h remaining), Red (stalled/no progress)
4. **Compare peak vs average** - Large gap suggests inconsistent network performance

---

## Performance Characteristics

### Rendering Performance

**Targets:**
- **60 FPS** - Smooth visual updates (16.67ms frame budget)
- **<5ms render time** - ratatui diffing + terminal I/O
- **<1ms state access** - read lock on shared state
- **<10ms event processing** - aggregated batch updates
- **~1ms margin** - system overhead

**Achieved:**
```
Component              % CPU          Latency
─────────              ─────          ───────
Event Processing       ~2%            <1ms (aggregation logic)
State Updates          ~1%            <1ms (RwLock write)
Rendering (ratatui)    ~3%            3-5ms (diffing + terminal I/O)
Keyboard Handling      <1%            <1ms (rare events)
System Overhead        ~1%            <1ms (tokio runtime)

Total: ~8% CPU, 16ms max latency (60 FPS)
```

### Event Throughput

**Test:** 10,000 PortFound events in 1 second

**Without Aggregation:**
- Events: 10,000
- State Updates: 10,000
- Renders: 10,000 (impossible at 60 FPS)
- **Result:** UI freezes, dropped frames, lag

**With Aggregation (16ms interval):**
- Events: 10,000
- Batches: 62 (1000ms / 16ms)
- State Updates: 62
- Renders: 60 (capped at 60 FPS)
- **Result:** Smooth UI, no dropped frames, no lag

**Throughput Capability:**
- **10,000+ events/second** - Handled without performance degradation
- **16ms max latency** - Aggregation delay (60 FPS flush rate)
- **1,000 event buffer** - Protects against burst overload (drops events beyond limit)

### Memory Footprint

```
Component              Size           Notes
─────────              ────           ─────
ScanState              ~1 KB          Arc<RwLock<T>>
UIState                ~100 bytes     Stack-allocated
EventAggregator        ~100 KB        1,000 × ~100 bytes/event
Event Buffer           ~100 KB        MAX_BUFFER_SIZE
Terminal Buffer        ~10 KB         ratatui screen buffer
Port Table Buffer      ~100 KB        1,000 × ~100 bytes/entry
Service Table Buffer   ~50 KB         500 × ~100 bytes/entry

Total: ~461 KB (negligible overhead)
```

**Comparison:**
- **ProRT-IP TUI** - ~461 KB memory overhead
- **tmux session** - ~5-10 MB
- **browser tab** - ~100-500 MB

---

## Best Practices

### 1. Use Auto-Scroll During Active Scans

**Problem:** Manual scrolling causes you to miss latest discoveries

**Solution:** Enable auto-scroll (follow mode)

```bash
# Port Table: Press 'a' to toggle auto-scroll
# Service Table: Press 'a' to toggle auto-scroll
```

**When to Disable:**
- Reviewing specific entries (need stable view)
- Searching for patterns (avoid jumping)
- Taking screenshots (maintain position)

### 2. Filter Noise for Large Scans

**Problem:** 10,000+ port table entries overwhelm the view

**Solution:** Apply filters to focus on relevant data

**Port Table Filtering:**
```bash
# Show only open ports
Press 'f' until status shows "Filter: Open"

# Show only TCP ports
Press 'd' until status shows "Proto: TCP"

# Combine filters
Press 'f' (Open) then 'd' (TCP) → "Filter: Open+TCP"
```

**Service Table Filtering:**
```bash
# Show only high-confidence detections
Press 'c' until status shows "Filter: High ≥90%"
```

### 3. Sort Strategically

**Port Table Sorting:**
- **By port** (`p` key) - Identify port ranges (e.g., all 8000-8999 web servers)
- **By state** (`s` key) - Group open/closed/filtered together
- **By IP** (`i` key) - See all ports for single host clustered

**Service Table Sorting:**
- **By confidence** (`6` key) - Review uncertain detections first
- **By service name** (`4` key) - Group same services (all nginx, all ssh)
- **By timestamp** (`1` key) - Chronological discovery order

### 4. Use Search for Targeted Analysis

**Port Table Search:**
```bash
# Find all SSH ports
Press '/', type "22", Enter

# Find specific host
Press '/', type "192.168.1.10", Enter
```

**Service Table Search:**
```bash
# Find all nginx instances
Press '/', type "nginx", Enter

# Find database services
Press '/', type "mysql", Enter
```

### 5. Monitor Metrics for Performance Issues

**Watch for:**
- **Throughput drop** - Average << Peak (network congestion or slow targets)
- **Rising error count** - Firewall blocking or rate limiting
- **ETA turning yellow/red** - Scan stalled or progress very slow
- **Duration >> expected** - Performance bottleneck (check system load)

**Corrective Actions:**
```bash
# If throughput drops:
# - Pause scan (Ctrl+C), adjust timing template (e.g., -T3 → -T2)
# - Check network connectivity (ping targets)

# If errors rise:
# - Review firewall rules (may need whitelist)
# - Reduce scan rate (use -T2 Polite timing)
```

### 6. Take Advantage of Tab Views

**Workflow:**
1. **Start on Port Table** - Watch discoveries accumulate
2. **Switch to Service Table** - Review detected services after 30-60s
3. **Check Metrics** - Verify throughput and ETA are reasonable
4. **Return to Port Table** - Continue monitoring discoveries

**Tab Switching:**
```bash
# Press Tab to cycle: Port → Service → Metrics → Port
# Press Shift+Tab to reverse: Port → Metrics → Service → Port
```

---

## Troubleshooting

### Issue 1: TUI Not Launching

**Symptom:** Error message "Terminal not compatible" or "TUI initialization failed"

**Cause:** Terminal emulator lacks required features (alternate screen, raw mode)

**Solutions:**

1. **Use modern terminal:**
   ```bash
   # Recommended terminals:
   # - Linux: gnome-terminal, konsole, kitty, alacritty
   # - macOS: iTerm2, Terminal.app, kitty, alacritty
   # - Windows: Windows Terminal, ConEmu, Cmder
   ```

2. **Check terminal capabilities:**
   ```bash
   # Verify TERM environment variable
   echo $TERM
   # Should be: xterm-256color, screen-256color, or similar
   ```

3. **Update terminal emulator:**
   ```bash
   # Example: Update gnome-terminal (Ubuntu/Debian)
   sudo apt update && sudo apt upgrade gnome-terminal
   ```

### Issue 2: UI Freezes or Lags

**Symptom:** Dashboard stops updating, keyboard input delayed, FPS drops below 60

**Cause:** Event rate exceeds aggregator capacity (>10,000 events/second sustained)

**Solutions:**

1. **Reduce scan parallelism:**
   ```bash
   # Lower --max-concurrent (default: 1000)
   prtip --tui -sS -p 1-65535 --max-concurrent 500 TARGET
   ```

2. **Use slower timing template:**
   ```bash
   # T4 (Aggressive) → T3 (Normal)
   prtip --tui -sS -T3 -p 1-65535 TARGET
   ```

3. **Scan fewer ports:**
   ```bash
   # Instead of all 65535 ports
   prtip --tui -sS -p 1-10000 TARGET
   ```

4. **Check system load:**
   ```bash
   # Monitor CPU/memory usage
   top  # or htop

   # If CPU >90%, reduce parallelism or timing
   ```

### Issue 3: Garbled Terminal Output

**Symptom:** Terminal displays corrupted characters, colors wrong, layout broken

**Cause:** Terminal state not restored after abnormal exit (crash, kill -9)

**Solutions:**

1. **Manual terminal reset:**
   ```bash
   # Reset terminal to default state
   reset

   # Or clear terminal
   clear
   ```

2. **Restore terminal settings:**
   ```bash
   # Disable raw mode (if stuck)
   stty sane
   ```

3. **Exit and reopen terminal:**
   - Close terminal window
   - Open new terminal session
   - Should be clean state

**Prevention:**
- ProRT-IP TUI automatically restores terminal on:
  - Normal exit (`q` key or scan completion)
  - Ctrl+C interrupt
  - Panic/crash (ratatui 0.29+ panic hook)
- Avoid `kill -9` (SIGKILL) - use `kill` (SIGTERM) instead

### Issue 4: Missing Keyboard Shortcuts

**Symptom:** Pressing keys does nothing, shortcuts not working

**Cause:** Help screen (?) is active, overlaying main UI

**Solutions:**

1. **Close help screen:**
   ```bash
   # Press '?' or 'F1' to toggle help
   # Should return to normal dashboard view
   ```

2. **Check active tab:**
   ```bash
   # Metrics tab has no interactive controls (read-only)
   # Switch to Port Table or Service Table for sorting/filtering
   Press Tab
   ```

3. **Verify terminal focus:**
   - Click terminal window to ensure focus
   - Some multiplexers (tmux, screen) intercept keys
   - May need prefix key first (e.g., tmux: Ctrl+B before commands)

---

## Technical Details

### Widget Architecture

ProRT-IP TUI implements a **Component trait** pattern for all widgets:

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

**7 Production Widgets:**

**Phase 6.1 (Foundation):**
1. **StatusBar** - Header with scan metadata and progress
2. **MainWidget** - Central content area with results summary
3. **LogWidget** - Scrollable event log with filtering
4. **HelpWidget** - Overlay with keyboard shortcuts

**Phase 6.2 (Dashboard):**
5. **PortTableWidget** - Sortable port discovery table (1,000 entries, 14 tests)
6. **ServiceTableWidget** - Service detection table with confidence (500 entries, 21 tests)
7. **MetricsDashboardWidget** - 3-column performance metrics (24 tests)

### State Management

**Shared State:**
```rust
// Thread-safe shared state (scanner writes, TUI reads)
pub type ScanState = Arc<RwLock<ScanStateInner>>;

pub struct ScanStateInner {
    pub stage: ScanStage,              // Initializing, Scanning, Complete, Error
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

**Local State:**
```rust
// TUI-only state (ephemeral, no locking needed)
pub struct UIState {
    pub selected_pane: SelectedPane,           // Main | Help
    pub cursor_position: usize,                // Cursor position
    pub scroll_offset: usize,                  // Scroll offset
    pub input_buffer: String,                  // Text input
    pub show_help: bool,                       // Help visibility
    pub active_tab: DashboardTab,              // PortTable | ServiceTable | Metrics
    pub fps: f32,                              // Debug FPS counter
    pub aggregator_dropped_events: usize,      // Dropped event count
}
```

### Event Aggregation

**Challenge:** Scanner generates 10,000+ events/second, UI can only render 60 FPS

**Solution:** Aggregate high-frequency events into 16ms batches (60 FPS)

```rust
// High-frequency events (aggregated, count only)
PortFound        → stats.ports_found += 1
HostDiscovered   → stats.hosts_discovered += 1
ServiceDetected  → stats.services_detected += 1

// Lifecycle events (buffered, processed on flush)
ScanStarted      → buffer.push(event)
ScanCompleted    → buffer.push(event)
Error            → buffer.push(event)
Warning          → buffer.push(event)
```

**Flush Logic:**
```rust
// Every 16ms (60 FPS timer tick)
if aggregator.should_flush() {
    let (events, stats) = aggregator.flush();

    // Process buffered lifecycle events
    for event in events {
        handle_scan_event(event, scan_state);
    }

    // Apply aggregated statistics (single write lock)
    let mut state = scan_state.write();
    state.open_ports += stats.ports_found;
    state.detected_services += stats.services_detected;
    // ... deduplication for discovered_hosts
    drop(state);  // Release lock ASAP
}
```

### Terminal Lifecycle

**Initialization (automatic with ratatui 0.29+):**
```rust
let mut terminal = ratatui::init();
// - Enable raw mode (disable line buffering, echo)
// - Enter alternate screen (preserve shell history)
// - Set panic hook for cleanup
```

**Normal Exit:**
```rust
ratatui::restore();
// - Leave alternate screen
// - Disable raw mode
// Terminal state restored to pre-TUI
```

**Panic Recovery (automatic):**
```rust
// ratatui 0.29+ handles panic hook automatically
// No manual cleanup needed!
// Terminal always restored even on crash
```

**Ctrl+C Handling:**
```rust
tokio::select! {
    Some(Ok(Event::Key(key))) = crossterm_rx.next() => {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return LoopControl::Quit;  // Graceful exit
        }
    }
}
// Main loop breaks, terminal restored in cleanup
```

---

## See Also

- **[User Guide: Basic Usage](../user-guide/basic-usage.md)** - CLI fundamentals and command-line options
- **[Nmap Compatibility](./nmap-compatibility.md)** - TUI works with all Nmap-compatible commands
- **[Service Detection](./service-detection.md)** - Understanding service detection in Service Table tab
- **[OS Fingerprinting](./os-fingerprinting.md)** - OS detection results visualization (future widget)
- **[TUI Architecture](../../TUI-ARCHITECTURE.md)** - Complete technical specification (1,338 lines)
- **[Performance Guide](../21-PERFORMANCE-GUIDE.md)** - Optimizing TUI performance for large scans

**External Resources:**
- **ratatui Documentation** - https://ratatui.rs/
- **crossterm Documentation** - https://docs.rs/crossterm/
- **ProRT-IP GitHub** - https://github.com/doublegate/ProRT-IP

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
**TUI Version:** Phase 6.2 Complete (4-tab dashboard with 7 widgets)
