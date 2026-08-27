//! LogWidget - scrollable event log with filtering
//!
//! The LogWidget displays scan events in a scrollable list with:
//! - Event filtering (All/Ports/Hosts/Services/Errors/Warnings)
//! - Auto-scroll to bottom for new events
//! - Ringbuffer (max 1,000 entries, auto-truncate)
//! - Keyboard navigation (arrow keys, Page Up/Down)
//! - Color-coded event types

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use std::time::UNIX_EPOCH;

use crate::state::UIState;
use crate::widgets::Component;

/// LogWidget displaying scrollable event log
///
/// # Layout
///
/// ```text
/// ┌─ Event Log [Filter: All] [Auto-scroll: ON] ──────────┐
/// │ 12:34:56 [PORT] Found open port 80/tcp                │
/// │ 12:34:57 [HOST] Discovered host 192.168.1.1           │
/// │ 12:34:58 [ERROR] Connection timeout                   │
/// │ ...                                                    │
/// │                                                        │
/// │ (123/1000 entries, 5 filtered)                        │
/// └────────────────────────────────────────────────────────┘
/// ```
///
/// # Keyboard Shortcuts
///
/// - `↑`/`↓` - Scroll one row
/// - `Page Up`/`Page Down` - Scroll one page
/// - `s` - Toggle auto-scroll
/// - `c` - Clear log
/// - `1` - Filter: All
/// - `2` - Filter: Ports only
/// - `3` - Filter: Hosts only
/// - `4` - Filter: Services only
/// - `5` - Filter: Errors only
/// - `6` - Filter: Warnings only
///
/// # Examples
///
/// ```rust,ignore
/// use prtip_tui::widgets::LogWidget;
///
/// let log_widget = LogWidget::new();
/// ```
pub struct LogWidget {
    // LogWidget has no internal state
    // All state lives in UIState::log_widget_state
}

impl LogWidget {
    /// Create a new LogWidget
    pub fn new() -> Self {
        Self {}
    }

    /// Get event type color
    fn event_type_color(event_type: &crate::state::EventType) -> Color {
        use crate::state::EventType;

        match event_type {
            EventType::Port => Color::Cyan,
            EventType::Host => Color::Blue,
            EventType::Service => Color::Green,
            EventType::Error => Color::Red,
            EventType::Warning => Color::Yellow,
            EventType::Info => Color::White,
        }
    }

    /// Get event type label
    fn event_type_label(event_type: &crate::state::EventType) -> &'static str {
        use crate::state::EventType;

        match event_type {
            EventType::Port => "PORT",
            EventType::Host => "HOST",
            EventType::Service => "SVC",
            EventType::Error => "ERR",
            EventType::Warning => "WARN",
            EventType::Info => "INFO",
        }
    }

    /// Format timestamp (simple HH:MM:SS)
    fn format_timestamp(timestamp: &std::time::SystemTime) -> String {
        // Simple formatting without chrono dependency
        match timestamp.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let total_seconds = duration.as_secs();
                let hours = (total_seconds / 3600) % 24;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;
                format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
            }
            Err(_) => "??:??:??".to_string(),
        }
    }

    /// Convert log entry to list item
    fn entry_to_list_item(entry: &crate::state::LogEntry) -> ListItem<'static> {
        let timestamp = Self::format_timestamp(&entry.timestamp);
        let type_label = Self::event_type_label(&entry.event_type);
        let type_color = Self::event_type_color(&entry.event_type);

        let line = Line::from(vec![
            Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled(
                format!("[{:^5}]", type_label),
                Style::default().fg(type_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::raw(entry.message.clone()),
        ]);

        ListItem::new(line)
    }

    /// Get filter label
    fn filter_label(filter: &crate::state::EventFilter) -> &'static str {
        use crate::state::EventFilter;

        match filter {
            EventFilter::All => "All",
            EventFilter::Ports => "Ports",
            EventFilter::Hosts => "Hosts",
            EventFilter::Services => "Services",
            EventFilter::Errors => "Errors",
            EventFilter::Warnings => "Warnings",
        }
    }
}

impl Default for LogWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for LogWidget {
    fn render(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        let log_state = &state.log_widget_state;

        // Get paginated entries
        let paginated = log_state.paginated_entries();

        // Build list items
        let items: Vec<ListItem> = paginated
            .iter()
            .map(|entry| Self::entry_to_list_item(entry))
            .collect();

        // Calculate stats
        let total_entries = log_state.entries.len();
        let visible_count = log_state.visible_entries().len();
        let filtered_count = total_entries - visible_count;

        // Build title with filter and auto-scroll status
        let title = format!(
            "Event Log [Filter: {}] [Auto-scroll: {}] ({}/{} entries{})",
            Self::filter_label(&log_state.filter),
            if log_state.auto_scroll { "ON" } else { "OFF" },
            visible_count,
            total_entries,
            if filtered_count > 0 {
                format!(", {} filtered", filtered_count)
            } else {
                String::new()
            }
        );

        // Build list widget
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(list, area);
    }

    fn handle_event(&mut self, event: Event) -> bool {
        // Event handling happens in App::run() via handle_log_widget_event()
        let _ = event;
        false
    }
}

/// Event handler for LogWidget keyboard shortcuts
///
/// This function is called from App::run() when LogWidget is focused.
/// It mutates UIState::log_widget_state based on keyboard events.
///
/// # Returns
///
/// `true` if the event was handled, `false` otherwise
pub fn handle_log_widget_event(event: Event, ui_state: &mut UIState) -> bool {
    use crate::state::EventFilter;

    if let Event::Key(KeyEvent { code, .. }) = event {
        let log_state = &mut ui_state.log_widget_state;

        match code {
            // Navigation
            KeyCode::Up => {
                log_state.scroll_up();
                true
            }
            KeyCode::Down => {
                log_state.scroll_down();
                true
            }
            KeyCode::PageUp => {
                log_state.page_up();
                true
            }
            KeyCode::PageDown => {
                log_state.page_down();
                true
            }

            // Auto-scroll toggle
            KeyCode::Char('s') => {
                log_state.toggle_auto_scroll();
                true
            }

            // Clear log
            KeyCode::Char('c') => {
                log_state.clear();
                true
            }

            // Filters
            KeyCode::Char('1') => {
                log_state.set_filter(EventFilter::All);
                true
            }
            KeyCode::Char('2') => {
                log_state.set_filter(EventFilter::Ports);
                true
            }
            KeyCode::Char('3') => {
                log_state.set_filter(EventFilter::Hosts);
                true
            }
            KeyCode::Char('4') => {
                log_state.set_filter(EventFilter::Services);
                true
            }
            KeyCode::Char('5') => {
                log_state.set_filter(EventFilter::Errors);
                true
            }
            KeyCode::Char('6') => {
                log_state.set_filter(EventFilter::Warnings);
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
    use crate::state::{EventFilter, EventType, LogEntry, UIState};

    fn create_test_entries() -> Vec<LogEntry> {
        vec![
            LogEntry::new(EventType::Port, "Found open port 80/tcp".to_string()),
            LogEntry::new(EventType::Host, "Discovered host 192.168.1.1".to_string()),
            LogEntry::new(EventType::Service, "Detected HTTP service".to_string()),
            LogEntry::new(EventType::Error, "Connection timeout".to_string()),
            LogEntry::new(EventType::Warning, "Rate limit active".to_string()),
            LogEntry::new(EventType::Info, "Scan started".to_string()),
        ]
    }

    #[test]
    fn test_log_widget_creation() {
        let _widget = LogWidget::new();
        // Should not panic
    }

    #[test]
    fn test_log_widget_default() {
        let _widget = LogWidget::default();
        // Should not panic
    }

    #[test]
    fn test_event_type_color() {
        assert_eq!(LogWidget::event_type_color(&EventType::Port), Color::Cyan);
        assert_eq!(LogWidget::event_type_color(&EventType::Host), Color::Blue);
        assert_eq!(
            LogWidget::event_type_color(&EventType::Service),
            Color::Green
        );
        assert_eq!(LogWidget::event_type_color(&EventType::Error), Color::Red);
        assert_eq!(
            LogWidget::event_type_color(&EventType::Warning),
            Color::Yellow
        );
        assert_eq!(LogWidget::event_type_color(&EventType::Info), Color::White);
    }

    #[test]
    fn test_event_type_label() {
        assert_eq!(LogWidget::event_type_label(&EventType::Port), "PORT");
        assert_eq!(LogWidget::event_type_label(&EventType::Host), "HOST");
        assert_eq!(LogWidget::event_type_label(&EventType::Service), "SVC");
        assert_eq!(LogWidget::event_type_label(&EventType::Error), "ERR");
        assert_eq!(LogWidget::event_type_label(&EventType::Warning), "WARN");
        assert_eq!(LogWidget::event_type_label(&EventType::Info), "INFO");
    }

    #[test]
    fn test_filter_label() {
        assert_eq!(LogWidget::filter_label(&EventFilter::All), "All");
        assert_eq!(LogWidget::filter_label(&EventFilter::Ports), "Ports");
        assert_eq!(LogWidget::filter_label(&EventFilter::Hosts), "Hosts");
        assert_eq!(LogWidget::filter_label(&EventFilter::Services), "Services");
        assert_eq!(LogWidget::filter_label(&EventFilter::Errors), "Errors");
        assert_eq!(LogWidget::filter_label(&EventFilter::Warnings), "Warnings");
    }

    #[test]
    fn test_handle_event_scroll_up() {
        let mut ui_state = UIState::new();
        let entries = create_test_entries();
        for entry in entries {
            ui_state.log_widget_state.add_entry(entry);
        }
        ui_state.log_widget_state.scroll_offset = 2;

        let event = Event::Key(KeyEvent::from(KeyCode::Up));
        let handled = handle_log_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.log_widget_state.scroll_offset, 1);
    }

    #[test]
    fn test_handle_event_scroll_down() {
        let mut ui_state = UIState::new();
        let entries = create_test_entries();
        for entry in entries {
            ui_state.log_widget_state.add_entry(entry);
        }
        ui_state.log_widget_state.scroll_offset = 0;

        let event = Event::Key(KeyEvent::from(KeyCode::Down));
        let handled = handle_log_widget_event(event, &mut ui_state);

        assert!(handled);
        // Exact value depends on visible_entries and visible_rows
        // Just verify it was handled
    }

    #[test]
    fn test_handle_event_toggle_auto_scroll() {
        let mut ui_state = UIState::new();
        ui_state.log_widget_state.auto_scroll = true;

        let event = Event::Key(KeyEvent::from(KeyCode::Char('s')));
        let handled = handle_log_widget_event(event, &mut ui_state);

        assert!(handled);
        assert!(!ui_state.log_widget_state.auto_scroll);
    }

    #[test]
    fn test_handle_event_clear() {
        let mut ui_state = UIState::new();
        let entries = create_test_entries();
        for entry in entries {
            ui_state.log_widget_state.add_entry(entry);
        }

        let event = Event::Key(KeyEvent::from(KeyCode::Char('c')));
        let handled = handle_log_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.log_widget_state.entries.len(), 0);
    }

    #[test]
    fn test_handle_event_filter_all() {
        let mut ui_state = UIState::new();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('1')));
        let handled = handle_log_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.log_widget_state.filter, EventFilter::All);
    }

    #[test]
    fn test_handle_event_filter_ports() {
        let mut ui_state = UIState::new();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('2')));
        let handled = handle_log_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.log_widget_state.filter, EventFilter::Ports);
    }

    #[test]
    fn test_handle_event_filter_hosts() {
        let mut ui_state = UIState::new();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('3')));
        let handled = handle_log_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.log_widget_state.filter, EventFilter::Hosts);
    }

    #[test]
    fn test_handle_event_filter_services() {
        let mut ui_state = UIState::new();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('4')));
        let handled = handle_log_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.log_widget_state.filter, EventFilter::Services);
    }

    #[test]
    fn test_handle_event_filter_errors() {
        let mut ui_state = UIState::new();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('5')));
        let handled = handle_log_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.log_widget_state.filter, EventFilter::Errors);
    }

    #[test]
    fn test_handle_event_filter_warnings() {
        let mut ui_state = UIState::new();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('6')));
        let handled = handle_log_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.log_widget_state.filter, EventFilter::Warnings);
    }

    #[test]
    fn test_handle_event_unhandled() {
        let mut ui_state = UIState::new();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('x')));
        let handled = handle_log_widget_event(event, &mut ui_state);

        assert!(!handled);
    }
}
