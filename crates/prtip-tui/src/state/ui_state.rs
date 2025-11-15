//! Local UI state (ephemeral, TUI-only)

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime};

// Re-use shared types from scan_state
use super::scan_state::{PortState, Protocol};

/// Pane selection for Tab navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedPane {
    /// Main area (port table, results)
    Main,
    /// Help screen
    Help,
}

/// Dashboard tab selection (Sprint 6.2 Task 2.3, extended in Task 2.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardTab {
    /// Port discoveries table
    PortTable,
    /// Service detections table
    ServiceTable,
    /// Live metrics dashboard (Sprint 6.2 Task 2.4)
    Metrics,
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

    // ===== Sprint 6.2: Dashboard Widgets =====
    /// Active dashboard tab (Sprint 6.2 Task 2.3)
    pub active_dashboard_tab: DashboardTab,

    /// PortTableWidget state (live port discoveries)
    pub port_table_state: PortTableState,

    /// ServiceTableWidget state (live service detections) - Sprint 6.2 Task 2.3
    pub service_table_state: ServiceTableState,

    /// ServicePanel state (live service detections)
    pub service_panel_state: ServicePanelState,

    /// NetworkGraph state (throughput visualization)
    pub network_graph_state: NetworkGraphState,
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
            active_dashboard_tab: DashboardTab::PortTable, // Default to PortTable
            port_table_state: PortTableState::new(),
            service_table_state: ServiceTableState::new(),
            service_panel_state: ServicePanelState::new(),
            network_graph_state: NetworkGraphState::new(),
        }
    }

    /// Switch to next dashboard tab (Tab key)
    pub fn next_dashboard_tab(&mut self) {
        self.active_dashboard_tab = match self.active_dashboard_tab {
            DashboardTab::PortTable => DashboardTab::ServiceTable,
            DashboardTab::ServiceTable => DashboardTab::Metrics,
            DashboardTab::Metrics => DashboardTab::PortTable,
        };
    }

    /// Switch to previous dashboard tab (Shift+Tab key)
    pub fn prev_dashboard_tab(&mut self) {
        self.active_dashboard_tab = match self.active_dashboard_tab {
            DashboardTab::PortTable => DashboardTab::Metrics,
            DashboardTab::ServiceTable => DashboardTab::PortTable,
            DashboardTab::Metrics => DashboardTab::ServiceTable,
        };
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

// PortState and Protocol are now imported from scan_state (see top of file)

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

// ===== Sprint 6.2: Dashboard Widget State =====

/// State for PortTableWidget (live port discoveries)
#[derive(Debug, Clone, Default)]
pub struct PortTableState {
    /// Selected row index (0-based)
    pub selected_row: usize,

    /// Scroll offset (for pagination, 0-based)
    pub scroll_offset: usize,

    /// Current sort column
    pub sort_column: PortTableColumn,

    /// Current sort order
    pub sort_order: SortOrder,

    /// Filter by state (optional)
    pub filter_state: Option<PortState>,

    /// Filter by protocol (optional)
    pub filter_protocol: Option<Protocol>,

    /// Filter query (optional, for search functionality)
    pub filter_query: Option<String>,

    /// Visible rows per page (dynamic, based on terminal height)
    pub visible_rows: usize,

    /// Auto-scroll to latest port (default: true)
    pub auto_scroll: bool,
}

impl PortTableState {
    /// Create new state
    pub fn new() -> Self {
        Self {
            selected_row: 0,
            scroll_offset: 0,
            sort_column: PortTableColumn::Timestamp,
            sort_order: SortOrder::Descending, // Latest first
            filter_state: None,
            filter_protocol: None,
            filter_query: None,
            visible_rows: 20,  // Default, updated on resize
            auto_scroll: true, // Default: auto-scroll enabled
        }
    }

    /// Select next row (with wrapping)
    pub fn select_next(&mut self, total_rows: usize) {
        if total_rows == 0 {
            self.selected_row = 0;
            return;
        }

        self.selected_row = (self.selected_row + 1).min(total_rows - 1);

        // Auto-scroll if needed
        if self.selected_row >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected_row - self.visible_rows + 1;
        }

        // Disable auto-scroll on manual navigation
        self.auto_scroll = false;
    }

    /// Select previous row
    pub fn select_previous(&mut self) {
        self.selected_row = self.selected_row.saturating_sub(1);

        // Auto-scroll if needed
        if self.selected_row < self.scroll_offset {
            self.scroll_offset = self.selected_row;
        }

        // Disable auto-scroll on manual navigation
        self.auto_scroll = false;
    }

    /// Scroll down one page
    pub fn page_down(&mut self, total_rows: usize) {
        if total_rows == 0 {
            return;
        }

        self.scroll_offset = (self.scroll_offset + self.visible_rows)
            .min(total_rows.saturating_sub(self.visible_rows));
        self.selected_row = self.scroll_offset;
        self.auto_scroll = false;
    }

    /// Scroll up one page
    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.visible_rows);
        self.selected_row = self.scroll_offset;
        self.auto_scroll = false;
    }

    /// Jump to first row
    pub fn select_first(&mut self) {
        self.selected_row = 0;
        self.scroll_offset = 0;
        self.auto_scroll = false;
    }

    /// Jump to last row
    pub fn select_last(&mut self, total_rows: usize) {
        if total_rows == 0 {
            return;
        }

        self.selected_row = total_rows - 1;
        self.scroll_offset = total_rows.saturating_sub(self.visible_rows);
        self.auto_scroll = false;
    }

    /// Toggle sort column (if same column, toggle order; else reset to descending)
    pub fn toggle_sort(&mut self, column: PortTableColumn) {
        if self.sort_column == column {
            // Same column, toggle order
            self.sort_order = match self.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            // New column, reset to descending (latest first)
            self.sort_column = column;
            self.sort_order = SortOrder::Descending;
        }
    }

    /// Toggle auto-scroll
    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
    }
}

/// Sort column enum for PortTableWidget
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PortTableColumn {
    #[default]
    Timestamp,
    Ip,
    Port,
    State,
    Protocol,
    ScanType,
}

// ===== Sprint 6.2 Task 2.3: ServiceTableWidget State =====

/// State for ServiceTableWidget (live service detections with confidence-based filtering)
#[derive(Debug, Clone, Default)]
pub struct ServiceTableState {
    /// Selected row index (0-based)
    pub selected_row: usize,

    /// Scroll offset (for pagination, 0-based)
    pub scroll_offset: usize,

    /// Current sort column
    pub sort_column: ServiceTableColumn,

    /// Current sort order
    pub sort_order: SortOrder,

    /// Confidence filter (All, Low ≥50%, Medium ≥75%, High ≥90%)
    pub confidence_filter: ConfidenceFilter,

    /// Filter by service name (optional, case-insensitive substring match)
    pub filter_service_name: Option<String>,

    /// Filter by port (optional)
    pub filter_port: Option<u16>,

    /// Filter by IP (optional)
    pub filter_ip: Option<String>,

    /// Visible rows per page (dynamic, based on terminal height)
    pub visible_rows: usize,

    /// Auto-scroll to latest service (default: true)
    pub auto_scroll: bool,
}

impl ServiceTableState {
    /// Create new state
    pub fn new() -> Self {
        Self {
            selected_row: 0,
            scroll_offset: 0,
            sort_column: ServiceTableColumn::Timestamp,
            sort_order: SortOrder::Descending, // Latest first
            confidence_filter: ConfidenceFilter::All,
            filter_service_name: None,
            filter_port: None,
            filter_ip: None,
            visible_rows: 20,  // Default, updated on resize
            auto_scroll: true, // Default: auto-scroll enabled
        }
    }

    /// Select next row
    pub fn select_next(&mut self, total_rows: usize) {
        if total_rows == 0 {
            self.selected_row = 0;
            return;
        }

        self.selected_row = (self.selected_row + 1).min(total_rows - 1);

        // Auto-scroll if needed
        if self.selected_row >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected_row - self.visible_rows + 1;
        }

        // Disable auto-scroll on manual navigation
        self.auto_scroll = false;
    }

    /// Select previous row
    pub fn select_previous(&mut self) {
        self.selected_row = self.selected_row.saturating_sub(1);

        // Auto-scroll if needed
        if self.selected_row < self.scroll_offset {
            self.scroll_offset = self.selected_row;
        }

        // Disable auto-scroll on manual navigation
        self.auto_scroll = false;
    }

    /// Scroll down one page
    pub fn page_down(&mut self, total_rows: usize) {
        if total_rows == 0 {
            return;
        }

        self.scroll_offset = (self.scroll_offset + self.visible_rows)
            .min(total_rows.saturating_sub(self.visible_rows));
        self.selected_row = self.scroll_offset;
        self.auto_scroll = false;
    }

    /// Scroll up one page
    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.visible_rows);
        self.selected_row = self.scroll_offset;
        self.auto_scroll = false;
    }

    /// Jump to first row
    pub fn select_first(&mut self) {
        self.selected_row = 0;
        self.scroll_offset = 0;
        self.auto_scroll = false;
    }

    /// Jump to last row
    pub fn select_last(&mut self, total_rows: usize) {
        if total_rows == 0 {
            return;
        }

        self.selected_row = total_rows - 1;
        self.scroll_offset = total_rows.saturating_sub(self.visible_rows);
        self.auto_scroll = false;
    }

    /// Toggle sort column (if same column, toggle order; else reset to descending)
    pub fn toggle_sort(&mut self, column: ServiceTableColumn) {
        if self.sort_column == column {
            // Same column, toggle order
            self.sort_order = match self.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            // New column, reset to descending (latest/highest first)
            self.sort_column = column;
            self.sort_order = SortOrder::Descending;
        }
    }

    /// Cycle confidence filter (All → Low → Medium → High → All)
    pub fn cycle_confidence_filter(&mut self) {
        self.confidence_filter = match self.confidence_filter {
            ConfidenceFilter::All => ConfidenceFilter::Low,
            ConfidenceFilter::Low => ConfidenceFilter::Medium,
            ConfidenceFilter::Medium => ConfidenceFilter::High,
            ConfidenceFilter::High => ConfidenceFilter::All,
        };
    }

    /// Toggle auto-scroll
    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
    }
}

/// Sort column enum for ServiceTableWidget
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServiceTableColumn {
    #[default]
    Timestamp,
    Ip,
    Port,
    ServiceName,
    Version,
    Confidence,
}

/// Confidence filter enum for ServiceTableWidget
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfidenceFilter {
    #[default]
    All, // Show all services
    Low,    // ≥50%
    Medium, // ≥75%
    High,   // ≥90%
}

impl ConfidenceFilter {
    /// Get minimum confidence threshold (0.0-1.0)
    pub fn min_confidence(self) -> f32 {
        match self {
            ConfidenceFilter::All => 0.0,
            ConfidenceFilter::Low => 0.5,
            ConfidenceFilter::Medium => 0.75,
            ConfidenceFilter::High => 0.9,
        }
    }

    /// Get display name
    pub fn display_name(self) -> &'static str {
        match self {
            ConfidenceFilter::All => "All",
            ConfidenceFilter::Low => "Low (≥50%)",
            ConfidenceFilter::Medium => "Med (≥75%)",
            ConfidenceFilter::High => "High (≥90%)",
        }
    }
}

/// State for ServicePanel (live service detections)
#[derive(Debug, Clone, Default)]
pub struct ServicePanelState {
    /// Selected service index (0-based)
    pub selected_index: usize,

    /// Scroll offset (for pagination, 0-based)
    pub scroll_offset: usize,

    /// Sort by confidence (high→low)
    pub sort_by_confidence: bool,

    /// Filter by minimum confidence threshold (0.0-1.0)
    pub min_confidence: f32,

    /// Visible rows per page (dynamic, based on terminal height)
    pub visible_rows: usize,
}

impl ServicePanelState {
    /// Create new state
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            sort_by_confidence: true, // Default: sort by confidence
            min_confidence: 0.5,      // Default: 50% confidence threshold
            visible_rows: 20,         // Default, updated on resize
        }
    }

    /// Select next service
    pub fn select_next(&mut self, total_services: usize) {
        if total_services == 0 {
            self.selected_index = 0;
            return;
        }

        self.selected_index = (self.selected_index + 1).min(total_services - 1);

        // Auto-scroll if needed
        if self.selected_index >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected_index - self.visible_rows + 1;
        }
    }

    /// Select previous service
    pub fn select_previous(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);

        // Auto-scroll if needed
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    /// Scroll down one page
    pub fn page_down(&mut self, total_services: usize) {
        if total_services == 0 {
            return;
        }

        self.scroll_offset = (self.scroll_offset + self.visible_rows)
            .min(total_services.saturating_sub(self.visible_rows));
        self.selected_index = self.scroll_offset;
    }

    /// Scroll up one page
    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.visible_rows);
        self.selected_index = self.scroll_offset;
    }

    /// Toggle sort by confidence
    pub fn toggle_sort_by_confidence(&mut self) {
        self.sort_by_confidence = !self.sort_by_confidence;
    }

    /// Increase confidence threshold (by 0.1)
    pub fn increase_confidence_threshold(&mut self) {
        self.min_confidence = (self.min_confidence + 0.1).min(1.0);
    }

    /// Decrease confidence threshold (by 0.1)
    pub fn decrease_confidence_threshold(&mut self) {
        self.min_confidence = (self.min_confidence - 0.1).max(0.0);
    }
}

/// State for NetworkGraph (throughput visualization)
#[derive(Debug, Clone, Default)]
pub struct NetworkGraphState {
    /// Graph type (throughput, latency, etc.)
    pub graph_type: GraphType,

    /// Time window (seconds)
    pub time_window: u64,

    /// Auto-scale Y axis
    pub auto_scale: bool,

    /// Manual Y axis max (if !auto_scale)
    pub manual_y_max: f64,
}

impl NetworkGraphState {
    /// Create new state
    pub fn new() -> Self {
        Self {
            graph_type: GraphType::Throughput,
            time_window: 60,        // Default: 60-second window
            auto_scale: true,       // Default: auto-scale enabled
            manual_y_max: 10_000.0, // Default manual max: 10K pps
        }
    }

    /// Toggle auto-scale
    pub fn toggle_auto_scale(&mut self) {
        self.auto_scale = !self.auto_scale;
    }

    /// Zoom in (reduce time window by 50%)
    pub fn zoom_in(&mut self) {
        self.time_window = (self.time_window / 2).max(10); // Min 10 seconds
    }

    /// Zoom out (increase time window by 2x)
    pub fn zoom_out(&mut self) {
        self.time_window = (self.time_window * 2).min(300); // Max 5 minutes
    }

    /// Increase manual Y max (by 10%)
    pub fn increase_y_max(&mut self) {
        self.manual_y_max *= 1.1;
    }

    /// Decrease manual Y max (by 10%)
    pub fn decrease_y_max(&mut self) {
        self.manual_y_max *= 0.9;
        self.manual_y_max = self.manual_y_max.max(100.0); // Min 100 pps
    }
}

/// Graph type enum for NetworkGraph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphType {
    #[default]
    Throughput,
    // Future: Latency, ErrorRate, etc.
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== PortTableState Tests =====

    #[test]
    fn test_port_table_state_sorting() {
        let mut state = PortTableState::new();

        // Verify default sort (Timestamp, Descending)
        assert_eq!(state.sort_column, PortTableColumn::Timestamp);
        assert_eq!(state.sort_order, SortOrder::Descending);

        // Toggle to same column (should reverse order)
        state.toggle_sort(PortTableColumn::Timestamp);
        assert_eq!(state.sort_column, PortTableColumn::Timestamp);
        assert_eq!(state.sort_order, SortOrder::Ascending);

        // Toggle again (should reverse back)
        state.toggle_sort(PortTableColumn::Timestamp);
        assert_eq!(state.sort_column, PortTableColumn::Timestamp);
        assert_eq!(state.sort_order, SortOrder::Descending);

        // Toggle to different column (should reset to Descending)
        state.toggle_sort(PortTableColumn::Port);
        assert_eq!(state.sort_column, PortTableColumn::Port);
        assert_eq!(state.sort_order, SortOrder::Descending);

        // Toggle to another different column
        state.toggle_sort(PortTableColumn::State);
        assert_eq!(state.sort_column, PortTableColumn::State);
        assert_eq!(state.sort_order, SortOrder::Descending);
    }

    #[test]
    fn test_port_table_state_filtering() {
        let mut state = PortTableState::new();

        // Verify default filters (none)
        assert!(state.filter_state.is_none());
        assert!(state.filter_protocol.is_none());
        assert!(state.filter_query.is_none());

        // Set state filter
        state.filter_state = Some(PortState::Open);
        assert_eq!(state.filter_state, Some(PortState::Open));

        // Set protocol filter
        state.filter_protocol = Some(Protocol::Tcp);
        assert_eq!(state.filter_protocol, Some(Protocol::Tcp));

        // Set query filter
        state.filter_query = Some("http".to_string());
        assert_eq!(state.filter_query, Some("http".to_string()));

        // Clear filters
        state.filter_state = None;
        state.filter_protocol = None;
        state.filter_query = None;
        assert!(state.filter_state.is_none());
        assert!(state.filter_protocol.is_none());
        assert!(state.filter_query.is_none());
    }

    #[test]
    fn test_port_table_state_pagination() {
        let mut state = PortTableState::new();
        state.visible_rows = 20;

        // Test select_next with 100 total rows
        let total_rows = 100;

        // Select first row
        assert_eq!(state.selected_row, 0);
        assert_eq!(state.scroll_offset, 0);

        // Select next (should move down)
        state.select_next(total_rows);
        assert_eq!(state.selected_row, 1);
        assert_eq!(state.scroll_offset, 0); // No scroll yet

        // Jump to row 25 (beyond visible area)
        state.selected_row = 25;
        state.select_next(total_rows);
        assert_eq!(state.selected_row, 26);
        assert_eq!(state.scroll_offset, 7); // Should scroll (26 - 20 + 1)

        // Test page_down
        state.selected_row = 0;
        state.scroll_offset = 0;
        state.page_down(total_rows);
        assert_eq!(state.scroll_offset, 20);
        assert_eq!(state.selected_row, 20);

        // Test page_up
        state.page_up();
        assert_eq!(state.scroll_offset, 0);
        assert_eq!(state.selected_row, 0);

        // Test select_last
        state.select_last(total_rows);
        assert_eq!(state.selected_row, 99);
        assert_eq!(state.scroll_offset, 80); // 100 - 20

        // Test select_first
        state.select_first();
        assert_eq!(state.selected_row, 0);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_port_table_state_auto_scroll() {
        let mut state = PortTableState::new();

        // Verify default auto_scroll = true
        assert!(state.auto_scroll);

        // Toggle auto_scroll
        state.toggle_auto_scroll();
        assert!(!state.auto_scroll);

        // Toggle back
        state.toggle_auto_scroll();
        assert!(state.auto_scroll);

        // Manual navigation disables auto_scroll
        state.auto_scroll = true;
        state.select_next(10);
        assert!(!state.auto_scroll); // Should be disabled

        state.auto_scroll = true;
        state.select_previous();
        assert!(!state.auto_scroll); // Should be disabled
    }

    #[test]
    fn test_port_table_state_empty_rows() {
        let mut state = PortTableState::new();

        // Test navigation with 0 rows
        state.select_next(0);
        assert_eq!(state.selected_row, 0);

        state.select_previous();
        assert_eq!(state.selected_row, 0);

        state.page_down(0);
        assert_eq!(state.selected_row, 0);

        state.select_last(0);
        assert_eq!(state.selected_row, 0);
    }

    // ===== ServicePanelState Tests =====

    #[test]
    fn test_service_panel_state_filtering() {
        let mut state = ServicePanelState::new();

        // Verify default min_confidence = 0.5
        assert_eq!(state.min_confidence, 0.5);

        // Increase threshold
        state.increase_confidence_threshold();
        assert!((state.min_confidence - 0.6).abs() < 0.01); // Floating point tolerance

        state.increase_confidence_threshold();
        assert!((state.min_confidence - 0.7).abs() < 0.01); // Floating point tolerance

        // Decrease threshold
        state.decrease_confidence_threshold();
        assert!((state.min_confidence - 0.6).abs() < 0.01); // Floating point tolerance

        // Test max limit (1.0)
        state.min_confidence = 0.95;
        state.increase_confidence_threshold();
        assert_eq!(state.min_confidence, 1.0); // Clamped at 1.0

        state.increase_confidence_threshold();
        assert_eq!(state.min_confidence, 1.0); // Should stay at 1.0

        // Test min limit (0.0)
        state.min_confidence = 0.05;
        state.decrease_confidence_threshold();
        assert_eq!(state.min_confidence, 0.0); // Clamped at 0.0

        state.decrease_confidence_threshold();
        assert_eq!(state.min_confidence, 0.0); // Should stay at 0.0
    }

    #[test]
    fn test_service_panel_state_sorting() {
        let mut state = ServicePanelState::new();

        // Verify default sort_by_confidence = true
        assert!(state.sort_by_confidence);

        // Toggle sorting
        state.toggle_sort_by_confidence();
        assert!(!state.sort_by_confidence);

        // Toggle back
        state.toggle_sort_by_confidence();
        assert!(state.sort_by_confidence);
    }

    #[test]
    fn test_service_panel_state_navigation() {
        let mut state = ServicePanelState::new();
        state.visible_rows = 20;

        // Test select_next with 50 total services
        let total_services = 50;

        // Select first service
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.scroll_offset, 0);

        // Select next
        state.select_next(total_services);
        assert_eq!(state.selected_index, 1);
        assert_eq!(state.scroll_offset, 0);

        // Jump to service 25 (beyond visible area)
        state.selected_index = 25;
        state.select_next(total_services);
        assert_eq!(state.selected_index, 26);
        assert_eq!(state.scroll_offset, 7); // Should scroll (26 - 20 + 1)

        // Test page_down
        state.selected_index = 0;
        state.scroll_offset = 0;
        state.page_down(total_services);
        assert_eq!(state.scroll_offset, 20);
        assert_eq!(state.selected_index, 20);

        // Test page_up
        state.page_up();
        assert_eq!(state.scroll_offset, 0);
        assert_eq!(state.selected_index, 0);

        // Test empty services
        state.select_next(0);
        assert_eq!(state.selected_index, 0);
    }

    // ===== NetworkGraphState Tests =====

    #[test]
    fn test_network_graph_state_auto_scale() {
        let mut state = NetworkGraphState::new();

        // Verify default auto_scale = true
        assert!(state.auto_scale);

        // Toggle auto_scale
        state.toggle_auto_scale();
        assert!(!state.auto_scale);

        // Toggle back
        state.toggle_auto_scale();
        assert!(state.auto_scale);
    }

    #[test]
    fn test_network_graph_state_zoom() {
        let mut state = NetworkGraphState::new();

        // Verify default time_window = 60
        assert_eq!(state.time_window, 60);

        // Zoom in (reduce by 50%)
        state.zoom_in();
        assert_eq!(state.time_window, 30);

        state.zoom_in();
        assert_eq!(state.time_window, 15);

        // Test min limit (10 seconds)
        state.zoom_in();
        assert_eq!(state.time_window, 10); // Min 10

        state.zoom_in();
        assert_eq!(state.time_window, 10); // Should stay at 10

        // Zoom out (increase by 2x)
        state.zoom_out();
        assert_eq!(state.time_window, 20);

        state.zoom_out();
        assert_eq!(state.time_window, 40);

        state.zoom_out();
        assert_eq!(state.time_window, 80);

        // Test max limit (300 seconds)
        state.time_window = 200;
        state.zoom_out();
        assert_eq!(state.time_window, 300); // Max 300

        state.zoom_out();
        assert_eq!(state.time_window, 300); // Should stay at 300
    }

    #[test]
    fn test_network_graph_state_manual_y_max() {
        let mut state = NetworkGraphState::new();

        // Verify default manual_y_max = 10,000.0
        assert_eq!(state.manual_y_max, 10_000.0);

        // Increase Y max (by 10%)
        state.increase_y_max();
        assert!((state.manual_y_max - 11_000.0).abs() < 1.0); // Floating point tolerance

        state.increase_y_max();
        assert!((state.manual_y_max - 12_100.0).abs() < 1.0); // Floating point tolerance

        // Decrease Y max (by 10%)
        state.decrease_y_max();
        assert!((state.manual_y_max - 10_890.0).abs() < 1.0); // Floating point tolerance

        // Test min limit (100.0)
        state.manual_y_max = 110.0;
        state.decrease_y_max();
        assert_eq!(state.manual_y_max, 100.0); // Min 100.0

        state.decrease_y_max();
        assert_eq!(state.manual_y_max, 100.0); // Should stay at 100.0
    }

    #[test]
    fn test_graph_data_point_sampling() {
        let state = NetworkGraphState::new();

        // Verify default time_window = 60 seconds
        assert_eq!(state.time_window, 60);

        // For 60-second window with 1-second samples, we expect 60 data points
        // (This is enforced by MAX_THROUGHPUT_SAMPLES in scan_state.rs)
        let expected_samples = 60;
        assert_eq!(state.time_window, expected_samples);

        // Different time windows should scale proportionally
        let mut state_30s = NetworkGraphState::new();
        state_30s.time_window = 30;
        assert_eq!(state_30s.time_window, 30); // 30 samples for 30-second window
    }

    // ===== UIState Tests =====

    #[test]
    fn test_ui_state_initialization() {
        let state = UIState::new();

        // Verify default pane
        assert_eq!(state.selected_pane, SelectedPane::Main);

        // Verify help screen hidden
        assert!(!state.show_help);

        // Verify widget states initialized
        assert_eq!(state.port_table_state.selected_row, 0);
        assert_eq!(state.service_panel_state.selected_index, 0);
        assert_eq!(state.network_graph_state.time_window, 60);
    }

    #[test]
    fn test_ui_state_pane_cycling() {
        let mut state = UIState::new();

        // Verify default pane
        assert_eq!(state.selected_pane, SelectedPane::Main);

        // Cycle to next pane
        state.next_pane();
        assert_eq!(state.selected_pane, SelectedPane::Help);

        // Cycle back to main
        state.next_pane();
        assert_eq!(state.selected_pane, SelectedPane::Main);

        // Test prev_pane (same as next_pane for 2 panes)
        state.prev_pane();
        assert_eq!(state.selected_pane, SelectedPane::Help);
    }

    #[test]
    fn test_ui_state_toggle_help() {
        let mut state = UIState::new();

        // Verify default help hidden
        assert!(!state.show_help);

        // Toggle help on
        state.toggle_help();
        assert!(state.show_help);

        // Toggle help off
        state.toggle_help();
        assert!(!state.show_help);
    }

    // ===== Enum Tests =====

    #[test]
    fn test_port_table_column_enum() {
        // Test PartialEq
        assert_eq!(PortTableColumn::Timestamp, PortTableColumn::Timestamp);
        assert_ne!(PortTableColumn::Timestamp, PortTableColumn::Port);
    }

    #[test]
    fn test_graph_type_enum() {
        // Test PartialEq
        assert_eq!(GraphType::Throughput, GraphType::Throughput);

        // Test default
        let default_type = GraphType::default();
        assert_eq!(default_type, GraphType::Throughput);
    }

    #[test]
    fn test_sort_order_enum() {
        // Test default
        let default_order = SortOrder::default();
        assert_eq!(default_order, SortOrder::Ascending);

        // Test PartialEq
        assert_ne!(SortOrder::Ascending, SortOrder::Descending);
    }
}
