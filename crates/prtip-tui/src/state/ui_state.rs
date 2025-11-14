//! Local UI state (ephemeral, TUI-only)

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime};

/// Pane selection for Tab navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedPane {
    /// Main area (port table, results)
    Main,
    /// Help screen
    Help,
}

/// Local UI state (single-threaded, no locking needed)
///
/// This state is only accessed by the TUI rendering thread.
#[derive(Debug, Clone)]
pub struct UIState {
    // ===== Existing Fields (Sprint 6.1) =====
    /// Currently selected pane
    pub selected_pane: SelectedPane,

    /// Cursor position within selected pane (DEPRECATED: use widget-specific state)
    pub cursor_position: usize,

    /// Scroll offset for scrollable content (DEPRECATED: use widget-specific state)
    pub scroll_offset: usize,

    /// Text input buffer (for future search/filter features)
    pub input_buffer: String,

    /// Whether to show the help screen
    pub show_help: bool,

    /// FPS counter for debugging
    pub fps: f32,

    /// Number of events dropped by event aggregator (for debug display)
    pub aggregator_dropped_events: usize,

    // ===== New Widget-Specific State (Sprint 6.2) =====
    /// StatusBar widget state
    pub status_bar_state: StatusBarState,

    /// MainWidget (port table) state
    pub main_widget_state: MainWidgetState,

    /// LogWidget (event log) state
    pub log_widget_state: LogWidgetState,

    /// HelpWidget state
    pub help_widget_state: HelpWidgetState,
}

impl UIState {
    /// Create a new UIState with default values
    pub fn new() -> Self {
        Self {
            // Existing fields
            selected_pane: SelectedPane::Main,
            cursor_position: 0,
            scroll_offset: 0,
            input_buffer: String::new(),
            show_help: false,
            fps: 0.0,
            aggregator_dropped_events: 0,

            // New widget state
            status_bar_state: StatusBarState::new(),
            main_widget_state: MainWidgetState::new(),
            log_widget_state: LogWidgetState::new(),
            help_widget_state: HelpWidgetState::new(),
        }
    }

    /// Cycle to the next pane (Tab key)
    pub fn next_pane(&mut self) {
        self.selected_pane = match self.selected_pane {
            SelectedPane::Main => SelectedPane::Help,
            SelectedPane::Help => SelectedPane::Main,
        };
    }

    /// Cycle to the previous pane (Shift+Tab key)
    pub fn prev_pane(&mut self) {
        // For now, same as next_pane since we only have 2 panes
        self.next_pane();
    }

    /// Toggle help screen visibility
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

// ===== Widget-Specific State Structures =====

/// State for StatusBar widget
#[derive(Debug, Clone, Default)]
pub struct StatusBarState {
    /// Last update timestamp (for ETA calculation)
    pub last_update: Option<Instant>,

    /// Rolling average throughput (10 samples, 1-second window)
    pub throughput_history: VecDeque<f64>,

    /// Maximum throughput seen (for "peak" display)
    pub peak_throughput: f64,

    /// Start time (for elapsed time calculation)
    pub start_time: Option<Instant>,
}

impl StatusBarState {
    /// Create new state
    pub fn new() -> Self {
        Self {
            last_update: None,
            throughput_history: VecDeque::with_capacity(10),
            peak_throughput: 0.0,
            start_time: None,
        }
    }

    /// Update throughput history (rolling average)
    pub fn update_throughput(&mut self, pps: f64) {
        if self.throughput_history.len() >= 10 {
            self.throughput_history.pop_front();
        }
        self.throughput_history.push_back(pps);

        if pps > self.peak_throughput {
            self.peak_throughput = pps;
        }

        self.last_update = Some(Instant::now());
    }

    /// Get average throughput (smoothed)
    pub fn average_throughput(&self) -> f64 {
        if self.throughput_history.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.throughput_history.iter().sum();
        sum / self.throughput_history.len() as f64
    }

    /// Calculate ETA (based on average throughput and remaining work)
    pub fn calculate_eta(&self, completed: u64, total: u64) -> Option<Duration> {
        if completed >= total {
            return None;
        }

        let avg_throughput = self.average_throughput();
        if avg_throughput <= 0.0 {
            return None;
        }

        let remaining = total - completed;
        let seconds = (remaining as f64) / avg_throughput;
        Some(Duration::from_secs_f64(seconds))
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }
}

/// State for MainWidget (port table)
#[derive(Debug, Clone, Default)]
pub struct MainWidgetState {
    /// Selected row index (0-based)
    pub selected_row: usize,

    /// Scroll offset (for pagination, 0-based)
    pub scroll_offset: usize,

    /// Current sort column
    pub sort_column: SortColumn,

    /// Current sort order
    pub sort_order: SortOrder,

    /// Cached port data (for sorting/filtering)
    pub ports: Vec<PortInfo>,

    /// Visible rows per page (dynamic, based on terminal height)
    pub visible_rows: usize,

    /// Filter query (optional, for search functionality)
    pub filter_query: Option<String>,
}

impl MainWidgetState {
    /// Create new state
    pub fn new() -> Self {
        Self {
            selected_row: 0,
            scroll_offset: 0,
            sort_column: SortColumn::Port,
            sort_order: SortOrder::Ascending,
            ports: Vec::new(),
            visible_rows: 20, // Default, updated on resize
            filter_query: None,
        }
    }

    /// Select next row (with wrapping)
    pub fn select_next(&mut self) {
        if self.ports.is_empty() {
            self.selected_row = 0;
            return;
        }

        self.selected_row = (self.selected_row + 1).min(self.ports.len() - 1);

        // Auto-scroll if needed
        if self.selected_row >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected_row - self.visible_rows + 1;
        }
    }

    /// Select previous row
    pub fn select_previous(&mut self) {
        self.selected_row = self.selected_row.saturating_sub(1);

        // Auto-scroll if needed
        if self.selected_row < self.scroll_offset {
            self.scroll_offset = self.selected_row;
        }
    }

    /// Scroll down one page
    pub fn page_down(&mut self) {
        if self.ports.is_empty() {
            return;
        }

        self.scroll_offset = (self.scroll_offset + self.visible_rows)
            .min(self.ports.len().saturating_sub(self.visible_rows));
        self.selected_row = self.scroll_offset;
    }

    /// Scroll up one page
    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.visible_rows);
        self.selected_row = self.scroll_offset;
    }

    /// Jump to first row
    pub fn select_first(&mut self) {
        self.selected_row = 0;
        self.scroll_offset = 0;
    }

    /// Jump to last row
    pub fn select_last(&mut self) {
        if self.ports.is_empty() {
            return;
        }

        self.selected_row = self.ports.len() - 1;
        self.scroll_offset = self.ports.len().saturating_sub(self.visible_rows);
    }

    /// Toggle sort column (if same column, toggle order; else reset to ascending)
    pub fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            // Same column, toggle order
            self.sort_order = match self.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            // New column, reset to ascending
            self.sort_column = column;
            self.sort_order = SortOrder::Ascending;
        }

        // Re-sort ports
        self.sort_ports();
    }

    /// Sort ports in-place (based on current sort column and order)
    pub fn sort_ports(&mut self) {
        use std::cmp::Ordering;

        self.ports.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::Port => a.port.cmp(&b.port),
                SortColumn::State => {
                    // Open > Filtered > Closed
                    let a_val = match a.state {
                        PortState::Open => 0,
                        PortState::Filtered => 1,
                        PortState::Closed => 2,
                    };
                    let b_val = match b.state {
                        PortState::Open => 0,
                        PortState::Filtered => 1,
                        PortState::Closed => 2,
                    };
                    a_val.cmp(&b_val)
                }
                SortColumn::Protocol => a.protocol.cmp(&b.protocol),
                SortColumn::Service => {
                    // Compare service names (alphabetical, None last)
                    match (&a.service, &b.service) {
                        (Some(a_svc), Some(b_svc)) => a_svc.cmp(b_svc),
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (None, None) => Ordering::Equal,
                    }
                }
            };

            // Apply sort order
            match self.sort_order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }

    /// Get currently selected port (if any)
    pub fn selected_port(&self) -> Option<&PortInfo> {
        self.ports.get(self.selected_row)
    }

    /// Get visible ports (for rendering)
    pub fn visible_ports(&self) -> &[PortInfo] {
        let start = self.scroll_offset;
        let end = (start + self.visible_rows).min(self.ports.len());
        &self.ports[start..end]
    }
}

/// Sort column enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortColumn {
    #[default]
    Port,
    State,
    Protocol,
    Service,
}

/// Sort order enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    Ascending,
    Descending,
}

/// Port information (cached for display)
#[derive(Debug, Clone)]
pub struct PortInfo {
    pub port: u16,
    pub state: PortState,
    pub protocol: Protocol,
    pub service: Option<String>,
    pub version: Option<String>,
}

/// Port state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    Open,
    Filtered,
    Closed,
}

/// Protocol enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Protocol {
    Tcp,
    Udp,
}

/// State for LogWidget (event log)
#[derive(Debug, Clone, Default)]
pub struct LogWidgetState {
    /// Log entries (ringbuffer, max 1,000)
    pub entries: VecDeque<LogEntry>,

    /// Selected log entry index (for navigation)
    pub selected_index: usize,

    /// Scroll offset (0-based)
    pub scroll_offset: usize,

    /// Auto-scroll enabled (scroll to bottom on new events)
    pub auto_scroll: bool,

    /// Active filter (None = show all)
    pub filter: EventFilter,

    /// Search query (optional, regex)
    pub search_query: Option<String>,

    /// Visible rows per page (dynamic, based on terminal height)
    pub visible_rows: usize,
}

impl LogWidgetState {
    /// Maximum log entries (ringbuffer limit)
    const MAX_ENTRIES: usize = 1000;

    /// Create new state
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(Self::MAX_ENTRIES),
            selected_index: 0,
            scroll_offset: 0,
            auto_scroll: true, // Default: auto-scroll enabled
            filter: EventFilter::All,
            search_query: None,
            visible_rows: 10, // Default, updated on resize
        }
    }

    /// Add a new log entry (ringbuffer, auto-truncate old)
    pub fn add_entry(&mut self, entry: LogEntry) {
        // Auto-truncate if at capacity
        if self.entries.len() >= Self::MAX_ENTRIES {
            self.entries.pop_front();
        }

        self.entries.push_back(entry);

        // Auto-scroll to bottom if enabled
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Clear all log entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Toggle auto-scroll
    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Set filter
    pub fn set_filter(&mut self, filter: EventFilter) {
        self.filter = filter;
        self.scroll_offset = 0;
        self.selected_index = 0;
    }

    /// Scroll to bottom (latest entry)
    pub fn scroll_to_bottom(&mut self) {
        let visible_count = self.visible_entries().len();
        if visible_count > self.visible_rows {
            self.scroll_offset = visible_count - self.visible_rows;
        } else {
            self.scroll_offset = 0;
        }
    }

    /// Scroll up one row
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
        self.auto_scroll = false; // Disable auto-scroll on manual scroll
    }

    /// Scroll down one row
    pub fn scroll_down(&mut self) {
        let visible_count = self.visible_entries().len();
        if self.scroll_offset + self.visible_rows < visible_count {
            self.scroll_offset += 1;
        }
    }

    /// Scroll up one page
    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.visible_rows);
        self.auto_scroll = false;
    }

    /// Scroll down one page
    pub fn page_down(&mut self) {
        let visible_count = self.visible_entries().len();
        let max_offset = visible_count.saturating_sub(self.visible_rows);
        self.scroll_offset = (self.scroll_offset + self.visible_rows).min(max_offset);
    }

    /// Get filtered entries (based on active filter)
    pub fn visible_entries(&self) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| self.filter.matches(entry))
            .collect()
    }

    /// Get paginated entries (for rendering)
    pub fn paginated_entries(&self) -> Vec<&LogEntry> {
        let visible = self.visible_entries();
        let start = self.scroll_offset;
        let end = (start + self.visible_rows).min(visible.len());
        visible[start..end].to_vec()
    }
}

/// Log entry (event record)
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: SystemTime,
    pub event_type: EventType,
    pub message: String,
}

impl LogEntry {
    /// Create new log entry
    pub fn new(event_type: EventType, message: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            event_type,
            message,
        }
    }

    /// Format timestamp (HH:MM:SS.mmm)
    pub fn formatted_time(&self) -> String {
        // Simple formatting (replace with chrono in Phase 6.3)
        format!("{:?}", self.timestamp)
    }
}

/// Event type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Port,
    Host,
    Service,
    Error,
    Warning,
    Info,
}

/// Event filter enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventFilter {
    #[default]
    All,
    Ports,
    Hosts,
    Services,
    Errors,
    Warnings,
}

impl EventFilter {
    /// Check if filter matches log entry
    pub fn matches(&self, entry: &LogEntry) -> bool {
        match self {
            EventFilter::All => true,
            EventFilter::Ports => entry.event_type == EventType::Port,
            EventFilter::Hosts => entry.event_type == EventType::Host,
            EventFilter::Services => entry.event_type == EventType::Service,
            EventFilter::Errors => entry.event_type == EventType::Error,
            EventFilter::Warnings => entry.event_type == EventType::Warning,
        }
    }
}

/// State for HelpWidget
#[derive(Debug, Clone, Default)]
pub struct HelpWidgetState {
    /// Scroll offset (for long help text)
    pub scroll_offset: usize,

    /// Context-sensitive mode (show only relevant shortcuts)
    pub context_mode: bool,

    /// Visible rows per page (dynamic, based on terminal height)
    pub visible_rows: usize,
}

impl HelpWidgetState {
    /// Create new state
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            context_mode: false,
            visible_rows: 20, // Default, updated on resize
        }
    }

    /// Scroll up one row
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down one row
    pub fn scroll_down(&mut self, total_lines: usize) {
        if self.scroll_offset + self.visible_rows < total_lines {
            self.scroll_offset += 1;
        }
    }

    /// Scroll up one page
    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.visible_rows);
    }

    /// Scroll down one page
    pub fn page_down(&mut self, total_lines: usize) {
        let max_offset = total_lines.saturating_sub(self.visible_rows);
        self.scroll_offset = (self.scroll_offset + self.visible_rows).min(max_offset);
    }

    /// Toggle context-sensitive mode
    pub fn toggle_context_mode(&mut self) {
        self.context_mode = !self.context_mode;
    }
}
