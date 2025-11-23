//! PortTableWidget - Live port discovery table with sorting and filtering
//!
//! The PortTableWidget displays discovered ports in real-time during a scan with:
//! - 6 columns: Timestamp, IP, Port, State, Protocol, Scan Type
//! - Row selection with highlighting
//! - Keyboard navigation (arrow keys, Page Up/Down, Home/End)
//! - Sorting by any column (ascending/descending)
//! - Filtering by state, protocol, or search query
//! - Auto-scroll mode for following latest discoveries
//! - Pagination (configurable visible rows)

use crossterm::event::{Event, KeyCode, KeyEvent};
use parking_lot::RwLock;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use std::sync::Arc;

use crate::state::{
    PortDiscovery, PortTableColumn, PortTableState, ScanPortState as PortState,
    ScanProtocol as Protocol, ScanState, ScanType, SortOrder, UIState,
};
use crate::widgets::Component;

/// PortTableWidget displaying live port discoveries
///
/// # Layout
///
/// ```text
/// ┌─ Port Discoveries ────────────────────────────────────────┐
/// │ Time  │ IP           │ Port │ State  │ Proto │ Scan Type  │
/// │───────┼──────────────┼──────┼────────┼───────┼────────────│
/// │ 12:34 │ 192.168.1.1  │   80 │ open   │ tcp   │ SYN        │
/// │ 12:34 │ 192.168.1.1  │  443 │ open   │ tcp   │ SYN        │
/// │ 12:35 │ 192.168.1.2  │   22 │ filter │ tcp   │ SYN        │
/// └───────────────────────────────────────────────────────────┘
/// ```
///
/// # Keyboard Shortcuts
///
/// - `↑`/`↓` - Select row
/// - `Page Up`/`Page Down` - Scroll page
/// - `Home`/`End` - First/last row
/// - `t` - Sort by timestamp
/// - `i` - Sort by IP
/// - `p` - Sort by port
/// - `s` - Sort by state
/// - `r` - Sort by protocol
/// - `c` - Sort by scan type
/// - `a` - Toggle auto-scroll
/// - `/` - Filter by search query (future)
///
/// # Examples
///
/// ```rust,ignore
/// use prtip_tui::widgets::PortTableWidget;
/// use prtip_tui::state::ScanState;
///
/// let scan_state = ScanState::shared();
/// let port_table = PortTableWidget::new(scan_state);
/// ```
pub struct PortTableWidget {
    /// Shared scan state (thread-safe)
    scan_state: Arc<RwLock<ScanState>>,
}

impl PortTableWidget {
    /// Create a new PortTableWidget
    ///
    /// # Arguments
    ///
    /// * `scan_state` - Shared scan state from scanner
    pub fn new(scan_state: Arc<RwLock<ScanState>>) -> Self {
        Self { scan_state }
    }

    /// Get column widths as fixed lengths
    fn column_widths() -> [u16; 6] {
        [
            8,  // Timestamp (HH:MM:SS)
            15, // IP Address
            6,  // Port
            8,  // State
            6,  // Protocol
            10, // Scan Type
        ]
    }

    /// Render column headers with sort indicators
    fn render_header(ui_state: &UIState) -> Row<'static> {
        let port_table_state = &ui_state.port_table_state;

        // Sort indicator
        let sort_indicator = |column: PortTableColumn| -> &'static str {
            if port_table_state.sort_column == column {
                match port_table_state.sort_order {
                    SortOrder::Ascending => " ▲",
                    SortOrder::Descending => " ▼",
                }
            } else {
                ""
            }
        };

        let header_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        Row::new(vec![
            Cell::from(Text::from(format!(
                "Time{}",
                sort_indicator(PortTableColumn::Timestamp)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "IP{}",
                sort_indicator(PortTableColumn::Ip)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "Port{}",
                sort_indicator(PortTableColumn::Port)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "State{}",
                sort_indicator(PortTableColumn::State)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "Proto{}",
                sort_indicator(PortTableColumn::Protocol)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "Scan{}",
                sort_indicator(PortTableColumn::ScanType)
            )))
            .style(header_style),
        ])
        .height(1)
    }

    /// Format timestamp as HH:MM:SS
    fn format_timestamp(timestamp: std::time::SystemTime) -> String {
        use std::time::UNIX_EPOCH;

        let duration = timestamp.duration_since(UNIX_EPOCH).unwrap_or_default();
        let secs = duration.as_secs();

        let hours = (secs / 3600) % 24;
        let minutes = (secs / 60) % 60;
        let seconds = secs % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// Convert PortDiscovery to table row
    fn port_discovery_to_row(discovery: &PortDiscovery, is_selected: bool) -> Row<'static> {
        // State color
        let state_color = match discovery.state {
            PortState::Open => Color::Green,
            PortState::Filtered => Color::Yellow,
            PortState::Closed => Color::Red,
        };

        // State text
        let state_text = match discovery.state {
            PortState::Open => "open",
            PortState::Filtered => "filter",
            PortState::Closed => "closed",
        };

        // Protocol text
        let protocol_text = match discovery.protocol {
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
        };

        // Scan type text
        let scan_type_text = match discovery.scan_type {
            ScanType::Syn => "SYN",
            ScanType::Connect => "Connect",
            ScanType::Fin => "FIN",
            ScanType::Null => "NULL",
            ScanType::Xmas => "Xmas",
            ScanType::Ack => "ACK",
            ScanType::Udp => "UDP",
            ScanType::Idle => "Idle",
        };

        // Row style
        let row_style = if is_selected {
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        Row::new(vec![
            Cell::from(Text::from(Self::format_timestamp(discovery.timestamp))),
            Cell::from(Text::from(discovery.ip.to_string())),
            Cell::from(Text::from(format!("{}", discovery.port))),
            Cell::from(Text::from(state_text)).style(Style::default().fg(state_color)),
            Cell::from(Text::from(protocol_text)),
            Cell::from(Text::from(scan_type_text)),
        ])
        .style(row_style)
        .height(1)
    }

    /// Apply filters to port discoveries
    fn apply_filters(
        discoveries: &[PortDiscovery],
        port_table_state: &PortTableState,
    ) -> Vec<PortDiscovery> {
        discoveries
            .iter()
            .filter(|d| {
                // Filter by state
                if let Some(filter_state) = port_table_state.filter_state {
                    if d.state != filter_state {
                        return false;
                    }
                }

                // Filter by protocol
                if let Some(filter_protocol) = port_table_state.filter_protocol {
                    if d.protocol != filter_protocol {
                        return false;
                    }
                }

                // Filter by search query
                if let Some(ref query_str) = port_table_state.filter_query {
                    let query = query_str.to_lowercase();
                    let ip_str = d.ip.to_string().to_lowercase();
                    let port_str = d.port.to_string();

                    if !ip_str.contains(&query) && !port_str.contains(&query) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    /// Apply sorting to port discoveries
    fn apply_sorting(
        mut discoveries: Vec<PortDiscovery>,
        port_table_state: &PortTableState,
    ) -> Vec<PortDiscovery> {
        use PortTableColumn::*;

        discoveries.sort_by(|a, b| {
            let ordering = match port_table_state.sort_column {
                Timestamp => a.timestamp.cmp(&b.timestamp),
                Ip => a.ip.cmp(&b.ip),
                Port => a.port.cmp(&b.port),
                State => {
                    // Sort order: Open < Filtered < Closed
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
                Protocol => a.protocol.cmp(&b.protocol),
                ScanType => {
                    // Sort alphabetically by scan type name
                    let a_str = format!("{:?}", a.scan_type);
                    let b_str = format!("{:?}", b.scan_type);
                    a_str.cmp(&b_str)
                }
            };

            match port_table_state.sort_order {
                SortOrder::Ascending => ordering,
                SortOrder::Descending => ordering.reverse(),
            }
        });

        discoveries
    }
}

impl Component for PortTableWidget {
    fn render(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        // Read shared scan state (brief lock)
        let scan_state = self.scan_state.read();
        let port_table_state = &state.port_table_state;

        // Convert VecDeque to Vec for processing
        let all_discoveries: Vec<_> = scan_state.port_discoveries.iter().cloned().collect();

        // Apply filters
        let filtered_discoveries = Self::apply_filters(&all_discoveries, port_table_state);

        // Apply sorting
        let sorted_discoveries = Self::apply_sorting(filtered_discoveries, port_table_state);

        // Calculate pagination
        let total_rows = sorted_discoveries.len();
        let visible_rows = port_table_state.visible_rows;
        let scroll_offset = port_table_state.scroll_offset;
        let selected_row = port_table_state.selected_row;

        // Get visible slice
        let visible_discoveries: Vec<_> = sorted_discoveries
            .iter()
            .skip(scroll_offset)
            .take(visible_rows)
            .enumerate()
            .map(|(idx, discovery)| {
                let is_selected = (scroll_offset + idx) == selected_row;
                Self::port_discovery_to_row(discovery, is_selected)
            })
            .collect();

        // Column widths
        let widths = Self::column_widths();

        // Build table
        let table = Table::new(
            visible_discoveries,
            [
                ratatui::layout::Constraint::Length(widths[0]),
                ratatui::layout::Constraint::Length(widths[1]),
                ratatui::layout::Constraint::Length(widths[2]),
                ratatui::layout::Constraint::Length(widths[3]),
                ratatui::layout::Constraint::Length(widths[4]),
                ratatui::layout::Constraint::Length(widths[5]),
            ],
        )
        .header(Self::render_header(state))
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Port Discoveries ({}/{} shown){}",
            total_rows,
            all_discoveries.len(),
            if port_table_state.auto_scroll {
                " [AUTO]"
            } else {
                ""
            }
        )));

        frame.render_widget(table, area);
    }

    fn handle_event(&mut self, event: Event) -> bool {
        // PortTableWidget state is in UIState, which we can't mutate here
        // Event handling happens in App::run() which has mutable access to UIState

        // For now, return false (event not handled)
        // This will be wired up in App::run() event dispatch
        let _ = event;
        false
    }
}

/// Event handler for PortTableWidget keyboard shortcuts
///
/// This function is called from App::run() when PortTableWidget is focused.
/// It mutates UIState::port_table_state based on keyboard events.
///
/// # Returns
///
/// `true` if the event was handled, `false` otherwise
pub fn handle_port_table_event(
    event: Event,
    ui_state: &mut UIState,
    scan_state: &Arc<RwLock<ScanState>>,
) -> bool {
    if let Event::Key(KeyEvent { code, .. }) = event {
        let port_table_state = &mut ui_state.port_table_state;

        // Calculate total rows for navigation
        let scan_state_lock = scan_state.read();
        let all_discoveries: Vec<_> = scan_state_lock.port_discoveries.iter().cloned().collect();
        drop(scan_state_lock);

        let filtered_discoveries =
            PortTableWidget::apply_filters(&all_discoveries, port_table_state);
        let total_rows = filtered_discoveries.len();

        match code {
            // Navigation
            KeyCode::Up => {
                port_table_state.select_previous();
                true
            }
            KeyCode::Down => {
                port_table_state.select_next(total_rows);
                true
            }
            KeyCode::PageUp => {
                port_table_state.page_up();
                true
            }
            KeyCode::PageDown => {
                port_table_state.page_down(total_rows);
                true
            }
            KeyCode::Home => {
                port_table_state.select_first();
                true
            }
            KeyCode::End => {
                port_table_state.select_last(total_rows);
                true
            }

            // Sorting
            KeyCode::Char('t') => {
                port_table_state.toggle_sort(PortTableColumn::Timestamp);
                true
            }
            KeyCode::Char('i') => {
                port_table_state.toggle_sort(PortTableColumn::Ip);
                true
            }
            KeyCode::Char('p') => {
                port_table_state.toggle_sort(PortTableColumn::Port);
                true
            }
            KeyCode::Char('s') => {
                port_table_state.toggle_sort(PortTableColumn::State);
                true
            }
            KeyCode::Char('r') => {
                port_table_state.toggle_sort(PortTableColumn::Protocol);
                true
            }
            KeyCode::Char('c') => {
                port_table_state.toggle_sort(PortTableColumn::ScanType);
                true
            }

            // Auto-scroll toggle
            KeyCode::Char('a') => {
                port_table_state.toggle_auto_scroll();
                true
            }

            _ => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{
        PortDiscovery, PortTableColumn, PortTableState, ScanPortState as PortState,
        ScanProtocol as Protocol, ScanType, SortOrder,
    };
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::SystemTime;

    fn create_test_discovery(port: u16, state: PortState, protocol: Protocol) -> PortDiscovery {
        PortDiscovery {
            timestamp: SystemTime::now(),
            ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            port,
            state,
            protocol,
            scan_type: ScanType::Syn,
        }
    }

    #[test]
    fn test_port_table_widget_creation() {
        let scan_state = ScanState::shared();
        let _widget = PortTableWidget::new(scan_state);
        // Should not panic
    }

    #[test]
    fn test_column_widths() {
        let widths = PortTableWidget::column_widths();
        assert_eq!(widths.len(), 6);
        assert_eq!(widths[0], 8); // Timestamp
        assert_eq!(widths[1], 15); // IP
        assert_eq!(widths[2], 6); // Port
        assert_eq!(widths[3], 8); // State
        assert_eq!(widths[4], 6); // Protocol
        assert_eq!(widths[5], 10); // Scan Type
    }

    #[test]
    fn test_format_timestamp() {
        let timestamp = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(3661); // 01:01:01
        let formatted = PortTableWidget::format_timestamp(timestamp);
        assert_eq!(formatted, "01:01:01");
    }

    #[test]
    fn test_apply_filters_state() {
        let discoveries = vec![
            create_test_discovery(80, PortState::Open, Protocol::Tcp),
            create_test_discovery(443, PortState::Open, Protocol::Tcp),
            create_test_discovery(22, PortState::Filtered, Protocol::Tcp),
            create_test_discovery(23, PortState::Closed, Protocol::Tcp),
        ];

        let mut port_table_state = PortTableState::new();
        port_table_state.filter_state = Some(PortState::Open);

        let filtered = PortTableWidget::apply_filters(&discoveries, &port_table_state);
        assert_eq!(filtered.len(), 2); // Only 2 open ports
        assert_eq!(filtered[0].port, 80);
        assert_eq!(filtered[1].port, 443);
    }

    #[test]
    fn test_apply_filters_protocol() {
        let discoveries = vec![
            create_test_discovery(80, PortState::Open, Protocol::Tcp),
            create_test_discovery(53, PortState::Open, Protocol::Udp),
            create_test_discovery(443, PortState::Open, Protocol::Tcp),
        ];

        let mut port_table_state = PortTableState::new();
        port_table_state.filter_protocol = Some(Protocol::Tcp);

        let filtered = PortTableWidget::apply_filters(&discoveries, &port_table_state);
        assert_eq!(filtered.len(), 2); // Only 2 TCP ports
        assert_eq!(filtered[0].port, 80);
        assert_eq!(filtered[1].port, 443);
    }

    #[test]
    fn test_apply_filters_query() {
        let discoveries = vec![
            create_test_discovery(80, PortState::Open, Protocol::Tcp),
            create_test_discovery(443, PortState::Open, Protocol::Tcp),
            create_test_discovery(8080, PortState::Open, Protocol::Tcp),
        ];

        let mut port_table_state = PortTableState::new();
        port_table_state.filter_query = Some("80".to_string());

        let filtered = PortTableWidget::apply_filters(&discoveries, &port_table_state);
        assert_eq!(filtered.len(), 2); // Port 80 and 8080 (both contain "80")
    }

    #[test]
    fn test_apply_sorting_by_port_ascending() {
        let discoveries = vec![
            create_test_discovery(443, PortState::Open, Protocol::Tcp),
            create_test_discovery(80, PortState::Open, Protocol::Tcp),
            create_test_discovery(22, PortState::Open, Protocol::Tcp),
        ];

        let mut port_table_state = PortTableState::new();
        port_table_state.sort_column = PortTableColumn::Port;
        port_table_state.sort_order = SortOrder::Ascending;

        let sorted = PortTableWidget::apply_sorting(discoveries, &port_table_state);
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].port, 22);
        assert_eq!(sorted[1].port, 80);
        assert_eq!(sorted[2].port, 443);
    }

    #[test]
    fn test_apply_sorting_by_port_descending() {
        let discoveries = vec![
            create_test_discovery(80, PortState::Open, Protocol::Tcp),
            create_test_discovery(443, PortState::Open, Protocol::Tcp),
            create_test_discovery(22, PortState::Open, Protocol::Tcp),
        ];

        let mut port_table_state = PortTableState::new();
        port_table_state.sort_column = PortTableColumn::Port;
        port_table_state.sort_order = SortOrder::Descending;

        let sorted = PortTableWidget::apply_sorting(discoveries, &port_table_state);
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].port, 443);
        assert_eq!(sorted[1].port, 80);
        assert_eq!(sorted[2].port, 22);
    }

    #[test]
    fn test_apply_sorting_by_state() {
        let discoveries = vec![
            create_test_discovery(80, PortState::Closed, Protocol::Tcp),
            create_test_discovery(443, PortState::Open, Protocol::Tcp),
            create_test_discovery(22, PortState::Filtered, Protocol::Tcp),
        ];

        let mut port_table_state = PortTableState::new();
        port_table_state.sort_column = PortTableColumn::State;
        port_table_state.sort_order = SortOrder::Ascending;

        let sorted = PortTableWidget::apply_sorting(discoveries, &port_table_state);
        assert_eq!(sorted.len(), 3);
        // Order: Open (0) < Filtered (1) < Closed (2)
        assert_eq!(sorted[0].state, PortState::Open);
        assert_eq!(sorted[1].state, PortState::Filtered);
        assert_eq!(sorted[2].state, PortState::Closed);
    }

    #[test]
    fn test_apply_sorting_by_protocol() {
        let discoveries = vec![
            create_test_discovery(53, PortState::Open, Protocol::Udp),
            create_test_discovery(80, PortState::Open, Protocol::Tcp),
            create_test_discovery(443, PortState::Open, Protocol::Tcp),
        ];

        let mut port_table_state = PortTableState::new();
        port_table_state.sort_column = PortTableColumn::Protocol;
        port_table_state.sort_order = SortOrder::Ascending;

        let sorted = PortTableWidget::apply_sorting(discoveries, &port_table_state);
        assert_eq!(sorted.len(), 3);
        // Order: Tcp < Udp (alphabetically)
        assert_eq!(sorted[0].protocol, Protocol::Tcp);
        assert_eq!(sorted[1].protocol, Protocol::Tcp);
        assert_eq!(sorted[2].protocol, Protocol::Udp);
    }

    #[test]
    fn test_handle_port_table_event_navigation() {
        let mut ui_state = UIState::new();
        let scan_state = ScanState::shared();

        // Add test discoveries
        {
            let mut scan_state_lock = scan_state.write();
            for i in 0..10 {
                scan_state_lock
                    .port_discoveries
                    .push_back(create_test_discovery(
                        80 + i,
                        PortState::Open,
                        Protocol::Tcp,
                    ));
            }
        }

        // Test Down
        let event = Event::Key(KeyEvent::from(KeyCode::Down));
        let handled = handle_port_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(ui_state.port_table_state.selected_row, 1);

        // Test Up
        let event = Event::Key(KeyEvent::from(KeyCode::Up));
        let handled = handle_port_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(ui_state.port_table_state.selected_row, 0);

        // Test End
        let event = Event::Key(KeyEvent::from(KeyCode::End));
        let handled = handle_port_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(ui_state.port_table_state.selected_row, 9); // Last row (0-indexed)

        // Test Home
        let event = Event::Key(KeyEvent::from(KeyCode::Home));
        let handled = handle_port_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(ui_state.port_table_state.selected_row, 0);
    }

    #[test]
    fn test_handle_port_table_event_sorting() {
        let mut ui_state = UIState::new();
        let scan_state = ScanState::shared();

        // Test sort by Port (default is Timestamp, new column defaults to Descending)
        let event = Event::Key(KeyEvent::from(KeyCode::Char('p')));
        let handled = handle_port_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(ui_state.port_table_state.sort_column, PortTableColumn::Port);
        assert_eq!(ui_state.port_table_state.sort_order, SortOrder::Descending);

        // Toggle to ascending (same column)
        let event = Event::Key(KeyEvent::from(KeyCode::Char('p')));
        let handled = handle_port_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(ui_state.port_table_state.sort_order, SortOrder::Ascending);

        // Test sort by State
        let event = Event::Key(KeyEvent::from(KeyCode::Char('s')));
        let handled = handle_port_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(
            ui_state.port_table_state.sort_column,
            PortTableColumn::State
        );
    }

    #[test]
    fn test_handle_port_table_event_auto_scroll() {
        let mut ui_state = UIState::new();
        let scan_state = ScanState::shared();

        // Verify default is true
        assert!(ui_state.port_table_state.auto_scroll);

        // Toggle to false
        let event = Event::Key(KeyEvent::from(KeyCode::Char('a')));
        let handled = handle_port_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert!(!ui_state.port_table_state.auto_scroll);

        // Toggle back to true
        let event = Event::Key(KeyEvent::from(KeyCode::Char('a')));
        let handled = handle_port_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert!(ui_state.port_table_state.auto_scroll);
    }

    #[test]
    fn test_handle_port_table_event_unhandled() {
        let mut ui_state = UIState::new();
        let scan_state = ScanState::shared();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('x')));
        let handled = handle_port_table_event(event, &mut ui_state, &scan_state);
        assert!(!handled);
    }
}
