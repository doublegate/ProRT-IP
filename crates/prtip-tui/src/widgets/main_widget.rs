//! MainWidget - sortable port table with keyboard navigation
//!
//! The MainWidget displays discovered ports in a sortable table with:
//! - 4 columns: Port, State, Protocol, Service
//! - Row selection with highlighting
//! - Keyboard navigation (arrow keys, Page Up/Down, Home/End)
//! - Sorting by any column (ascending/descending)
//! - Pagination (20-30 visible rows)

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::state::UIState;
use crate::widgets::Component;

/// MainWidget displaying sortable port table
///
/// # Layout
///
/// ```text
/// ┌─ Port Table ──────────────────────────────────────────────┐
/// │ Port │ State    │ Protocol │ Service                      │
/// │──────┼──────────┼──────────┼──────────────────────────────│
/// │   80 │ open     │ tcp      │ http                         │
/// │  443 │ open     │ tcp      │ https                        │
/// │   22 │ filtered │ tcp      │ ssh                          │
/// └───────────────────────────────────────────────────────────┘
/// ```
///
/// # Keyboard Shortcuts
///
/// - `↑`/`↓` - Select row
/// - `Page Up`/`Page Down` - Scroll page
/// - `Home`/`End` - First/last row
/// - `p` - Sort by port
/// - `s` - Sort by state
/// - `r` - Sort by protocol
/// - `v` - Sort by service
///
/// # Examples
///
/// ```rust,ignore
/// use prtip_tui::widgets::MainWidget;
///
/// let main_widget = MainWidget::new();
/// ```
pub struct MainWidget {
    // MainWidget has no internal state
    // All state lives in UIState::main_widget_state
}

impl MainWidget {
    /// Create a new MainWidget
    pub fn new() -> Self {
        Self {}
    }

    /// Get column widths as constraints
    fn column_widths() -> [u16; 4] {
        [
            10, // Port
            12, // State
            10, // Protocol
            0,  // Service (remaining space)
        ]
    }

    /// Render column headers with sort indicators
    fn render_header(ui_state: &UIState) -> Row<'static> {
        use crate::state::{SortColumn, SortOrder};

        let main_state = &ui_state.main_widget_state;

        // Sort indicator
        let sort_indicator = |column: SortColumn| -> &'static str {
            if main_state.sort_column == column {
                match main_state.sort_order {
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
                "Port{}",
                sort_indicator(SortColumn::Port)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "State{}",
                sort_indicator(SortColumn::State)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "Protocol{}",
                sort_indicator(SortColumn::Protocol)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "Service{}",
                sort_indicator(SortColumn::Service)
            )))
            .style(header_style),
        ])
        .height(1)
    }

    /// Convert PortInfo to table row
    fn port_info_to_row(port_info: &crate::state::PortInfo, is_selected: bool) -> Row<'static> {
        use crate::state::{ScanPortState as PortState, ScanProtocol as Protocol};

        // State color
        let state_color = match port_info.state {
            PortState::Open => Color::Green,
            PortState::Filtered => Color::Yellow,
            PortState::Closed => Color::Red,
        };

        // State text
        let state_text = match port_info.state {
            PortState::Open => "open",
            PortState::Filtered => "filtered",
            PortState::Closed => "closed",
        };

        // Protocol text
        let protocol_text = match port_info.protocol {
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
        };

        // Service text (with version if available)
        let service_text = if let Some(ref service) = port_info.service {
            if let Some(ref version) = port_info.version {
                format!("{} ({})", service, version)
            } else {
                service.clone()
            }
        } else {
            "unknown".to_string()
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
            Cell::from(Text::from(format!("{}", port_info.port))),
            Cell::from(Text::from(state_text)).style(Style::default().fg(state_color)),
            Cell::from(Text::from(protocol_text)),
            Cell::from(Text::from(service_text)),
        ])
        .style(row_style)
        .height(1)
    }
}

impl Default for MainWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for MainWidget {
    fn render(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        let main_state = &state.main_widget_state;

        // Get visible ports (pagination)
        let visible_ports = main_state.visible_ports();
        let selected_row = main_state.selected_row;
        let scroll_offset = main_state.scroll_offset;

        // Build table rows
        let rows: Vec<Row> = visible_ports
            .iter()
            .enumerate()
            .map(|(idx, port_info)| {
                let is_selected = (scroll_offset + idx) == selected_row;
                Self::port_info_to_row(port_info, is_selected)
            })
            .collect();

        // Column widths
        let widths = Self::column_widths();

        // Build table
        let table = Table::new(
            rows,
            [
                ratatui::layout::Constraint::Length(widths[0]),
                ratatui::layout::Constraint::Length(widths[1]),
                ratatui::layout::Constraint::Length(widths[2]),
                ratatui::layout::Constraint::Min(widths[3]),
            ],
        )
        .header(Self::render_header(state))
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Port Table ({}/{} ports)",
            main_state.ports.len(),
            main_state.ports.len()
        )));

        frame.render_widget(table, area);
    }

    fn handle_event(&mut self, event: Event) -> bool {
        // MainWidget state is in UIState, which we can't mutate here
        // Event handling happens in App::run() which has mutable access to UIState

        // For now, return false (event not handled)
        // This will be wired up in App::run() event dispatch
        let _ = event;
        false
    }
}

/// Event handler for MainWidget keyboard shortcuts
///
/// This function is called from App::run() when MainWidget is focused.
/// It mutates UIState::main_widget_state based on keyboard events.
///
/// # Returns
///
/// `true` if the event was handled, `false` otherwise
pub fn handle_main_widget_event(event: Event, ui_state: &mut UIState) -> bool {
    use crate::state::SortColumn;

    if let Event::Key(KeyEvent { code, .. }) = event {
        let main_state = &mut ui_state.main_widget_state;

        match code {
            // Navigation
            KeyCode::Up => {
                main_state.select_previous();
                true
            }
            KeyCode::Down => {
                main_state.select_next();
                true
            }
            KeyCode::PageUp => {
                main_state.page_up();
                true
            }
            KeyCode::PageDown => {
                main_state.page_down();
                true
            }
            KeyCode::Home => {
                main_state.select_first();
                true
            }
            KeyCode::End => {
                main_state.select_last();
                true
            }

            // Sorting
            KeyCode::Char('p') => {
                main_state.toggle_sort(SortColumn::Port);
                true
            }
            KeyCode::Char('s') => {
                main_state.toggle_sort(SortColumn::State);
                true
            }
            KeyCode::Char('r') => {
                main_state.toggle_sort(SortColumn::Protocol);
                true
            }
            KeyCode::Char('v') => {
                main_state.toggle_sort(SortColumn::Service);
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
        PortInfo, ScanPortState as PortState, ScanProtocol as Protocol, SortColumn, SortOrder,
        UIState,
    };

    fn create_test_ports() -> Vec<PortInfo> {
        vec![
            PortInfo {
                port: 80,
                state: PortState::Open,
                protocol: Protocol::Tcp,
                service: Some("http".to_string()),
                version: None,
            },
            PortInfo {
                port: 443,
                state: PortState::Open,
                protocol: Protocol::Tcp,
                service: Some("https".to_string()),
                version: Some("TLSv1.3".to_string()),
            },
            PortInfo {
                port: 22,
                state: PortState::Filtered,
                protocol: Protocol::Tcp,
                service: Some("ssh".to_string()),
                version: None,
            },
        ]
    }

    #[test]
    fn test_main_widget_creation() {
        let _widget = MainWidget::new();
        // Should not panic
    }

    #[test]
    fn test_main_widget_default() {
        let _widget = MainWidget::default();
        // Should not panic
    }

    #[test]
    fn test_column_widths() {
        let widths = MainWidget::column_widths();
        assert_eq!(widths[0], 10); // Port
        assert_eq!(widths[1], 12); // State
        assert_eq!(widths[2], 10); // Protocol
        assert_eq!(widths[3], 0); // Service (Min constraint)
    }

    #[test]
    fn test_handle_event_navigation_up() {
        let mut ui_state = UIState::new();
        ui_state.main_widget_state.ports = create_test_ports();
        ui_state.main_widget_state.selected_row = 1;

        let event = Event::Key(KeyEvent::from(KeyCode::Up));
        let handled = handle_main_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.main_widget_state.selected_row, 0);
    }

    #[test]
    fn test_handle_event_navigation_down() {
        let mut ui_state = UIState::new();
        ui_state.main_widget_state.ports = create_test_ports();
        ui_state.main_widget_state.selected_row = 0;

        let event = Event::Key(KeyEvent::from(KeyCode::Down));
        let handled = handle_main_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.main_widget_state.selected_row, 1);
    }

    #[test]
    fn test_handle_event_navigation_home() {
        let mut ui_state = UIState::new();
        ui_state.main_widget_state.ports = create_test_ports();
        ui_state.main_widget_state.selected_row = 2;

        let event = Event::Key(KeyEvent::from(KeyCode::Home));
        let handled = handle_main_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.main_widget_state.selected_row, 0);
    }

    #[test]
    fn test_handle_event_navigation_end() {
        let mut ui_state = UIState::new();
        ui_state.main_widget_state.ports = create_test_ports();
        ui_state.main_widget_state.selected_row = 0;

        let event = Event::Key(KeyEvent::from(KeyCode::End));
        let handled = handle_main_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.main_widget_state.selected_row, 2); // Last row
    }

    #[test]
    fn test_handle_event_sort_by_port() {
        let mut ui_state = UIState::new();
        ui_state.main_widget_state.ports = create_test_ports();
        ui_state.main_widget_state.sort_column = SortColumn::State;

        let event = Event::Key(KeyEvent::from(KeyCode::Char('p')));
        let handled = handle_main_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.main_widget_state.sort_column, SortColumn::Port);
        assert_eq!(ui_state.main_widget_state.sort_order, SortOrder::Ascending);
    }

    #[test]
    fn test_handle_event_sort_toggle_order() {
        let mut ui_state = UIState::new();
        ui_state.main_widget_state.ports = create_test_ports();
        ui_state.main_widget_state.sort_column = SortColumn::Port;
        ui_state.main_widget_state.sort_order = SortOrder::Ascending;

        // Toggle to descending
        let event = Event::Key(KeyEvent::from(KeyCode::Char('p')));
        let handled = handle_main_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.main_widget_state.sort_column, SortColumn::Port);
        assert_eq!(ui_state.main_widget_state.sort_order, SortOrder::Descending);
    }

    #[test]
    fn test_handle_event_sort_by_state() {
        let mut ui_state = UIState::new();
        ui_state.main_widget_state.ports = create_test_ports();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('s')));
        let handled = handle_main_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.main_widget_state.sort_column, SortColumn::State);
    }

    #[test]
    fn test_handle_event_sort_by_protocol() {
        let mut ui_state = UIState::new();
        ui_state.main_widget_state.ports = create_test_ports();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('r')));
        let handled = handle_main_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.main_widget_state.sort_column, SortColumn::Protocol);
    }

    #[test]
    fn test_handle_event_sort_by_service() {
        let mut ui_state = UIState::new();
        ui_state.main_widget_state.ports = create_test_ports();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('v')));
        let handled = handle_main_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.main_widget_state.sort_column, SortColumn::Service);
    }

    #[test]
    fn test_handle_event_unhandled() {
        let mut ui_state = UIState::new();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('x')));
        let handled = handle_main_widget_event(event, &mut ui_state);

        assert!(!handled);
    }
}
