//! HelpWidget - scrollable help text with keyboard shortcuts
//!
//! The HelpWidget displays keybindings and help information with:
//! - Scrollable paragraph widget
//! - Context-sensitive mode (show only relevant shortcuts)
//! - Keyboard navigation (arrow keys, Page Up/Down)
//! - Color-coded sections

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::state::UIState;
use crate::widgets::Component;

/// HelpWidget displaying scrollable help text
///
/// # Layout
///
/// ```text
/// ┌─ Help [Context: Global] ──────────────────────────┐
/// │ ProRT-IP TUI - Keyboard Shortcuts                 │
/// │                                                    │
/// │ Global Shortcuts:                                 │
/// │   q, Esc - Quit application                       │
/// │   ? - Toggle help screen                          │
/// │   Tab - Next pane                                 │
/// │   Shift+Tab - Previous pane                       │
/// │                                                    │
/// │ Main Widget (Port Table):                         │
/// │   ↑/↓ - Select row                                │
/// │   Page Up/Down - Scroll page                      │
/// │   p/s/r/v - Sort by Port/State/Protocol/Service   │
/// │                                                    │
/// │ (Press ? to close help)                           │
/// └────────────────────────────────────────────────────┘
/// ```
///
/// # Keyboard Shortcuts
///
/// - `↑`/`↓` - Scroll one row
/// - `Page Up`/`Page Down` - Scroll one page
/// - `c` - Toggle context-sensitive mode
/// - `?` - Close help screen
///
/// # Examples
///
/// ```rust,ignore
/// use prtip_tui::widgets::HelpWidget;
///
/// let help_widget = HelpWidget::new();
/// ```
pub struct HelpWidget {
    // HelpWidget has no internal state
    // All state lives in UIState::help_widget_state
}

impl HelpWidget {
    /// Create a new HelpWidget
    pub fn new() -> Self {
        Self {}
    }

    /// Get help text lines
    fn help_text_lines(context_mode: bool) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "ProRT-IP TUI - Keyboard Shortcuts",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Global Shortcuts:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  q, Esc", Style::default().fg(Color::Green)),
                Span::raw(" - Quit application"),
            ]),
            Line::from(vec![
                Span::styled("  ?", Style::default().fg(Color::Green)),
                Span::raw(" - Toggle help screen"),
            ]),
            Line::from(vec![
                Span::styled("  Tab", Style::default().fg(Color::Green)),
                Span::raw(" - Next pane"),
            ]),
            Line::from(vec![
                Span::styled("  Shift+Tab", Style::default().fg(Color::Green)),
                Span::raw(" - Previous pane"),
            ]),
            Line::from(""),
        ];

        // Add widget-specific shortcuts (unless in context mode)
        if !context_mode {
            lines.extend(vec![
                Line::from(Span::styled(
                    "Main Widget (Port Table):",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(vec![
                    Span::styled("  ↑/↓", Style::default().fg(Color::Green)),
                    Span::raw(" - Select row"),
                ]),
                Line::from(vec![
                    Span::styled("  Page Up/Down", Style::default().fg(Color::Green)),
                    Span::raw(" - Scroll page"),
                ]),
                Line::from(vec![
                    Span::styled("  Home/End", Style::default().fg(Color::Green)),
                    Span::raw(" - First/last row"),
                ]),
                Line::from(vec![
                    Span::styled("  p", Style::default().fg(Color::Green)),
                    Span::raw(" - Sort by Port"),
                ]),
                Line::from(vec![
                    Span::styled("  s", Style::default().fg(Color::Green)),
                    Span::raw(" - Sort by State"),
                ]),
                Line::from(vec![
                    Span::styled("  r", Style::default().fg(Color::Green)),
                    Span::raw(" - Sort by Protocol"),
                ]),
                Line::from(vec![
                    Span::styled("  v", Style::default().fg(Color::Green)),
                    Span::raw(" - Sort by Service"),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Log Widget (Event Log):",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(vec![
                    Span::styled("  ↑/↓", Style::default().fg(Color::Green)),
                    Span::raw(" - Scroll one row"),
                ]),
                Line::from(vec![
                    Span::styled("  Page Up/Down", Style::default().fg(Color::Green)),
                    Span::raw(" - Scroll page"),
                ]),
                Line::from(vec![
                    Span::styled("  s", Style::default().fg(Color::Green)),
                    Span::raw(" - Toggle auto-scroll"),
                ]),
                Line::from(vec![
                    Span::styled("  c", Style::default().fg(Color::Green)),
                    Span::raw(" - Clear log"),
                ]),
                Line::from(vec![
                    Span::styled("  1-6", Style::default().fg(Color::Green)),
                    Span::raw(" - Filter events (All/Ports/Hosts/Services/Errors/Warnings)"),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Help Widget:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(vec![
                    Span::styled("  ↑/↓", Style::default().fg(Color::Green)),
                    Span::raw(" - Scroll one row"),
                ]),
                Line::from(vec![
                    Span::styled("  Page Up/Down", Style::default().fg(Color::Green)),
                    Span::raw(" - Scroll page"),
                ]),
                Line::from(vec![
                    Span::styled("  c", Style::default().fg(Color::Green)),
                    Span::raw(" - Toggle context-sensitive mode"),
                ]),
                Line::from(""),
            ]);
        }

        lines.extend(vec![
            Line::from(""),
            Line::from(Span::styled(
                "(Press ? to close help)",
                Style::default().fg(Color::DarkGray),
            )),
        ]);

        lines
    }

    /// Count total lines in help text
    fn total_lines(context_mode: bool) -> usize {
        Self::help_text_lines(context_mode).len()
    }
}

impl Default for HelpWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for HelpWidget {
    fn render(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        let help_state = &state.help_widget_state;

        // Get help text
        let lines = Self::help_text_lines(help_state.context_mode);

        // Build title
        let title = format!(
            "Help [Context: {}]",
            if help_state.context_mode {
                "Contextual"
            } else {
                "Global"
            }
        );

        // Build paragraph widget
        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: false })
            .scroll((help_state.scroll_offset as u16, 0));

        frame.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, event: Event) -> bool {
        // Event handling happens in App::run() via handle_help_widget_event()
        let _ = event;
        false
    }
}

/// Event handler for HelpWidget keyboard shortcuts
///
/// This function is called from App::run() when HelpWidget is focused.
/// It mutates UIState::help_widget_state based on keyboard events.
///
/// # Returns
///
/// `true` if the event was handled, `false` otherwise
pub fn handle_help_widget_event(event: Event, ui_state: &mut UIState) -> bool {
    if let Event::Key(KeyEvent { code, .. }) = event {
        let help_state = &mut ui_state.help_widget_state;
        let total_lines = HelpWidget::total_lines(help_state.context_mode);

        match code {
            // Navigation
            KeyCode::Up => {
                help_state.scroll_up();
                true
            }
            KeyCode::Down => {
                help_state.scroll_down(total_lines);
                true
            }
            KeyCode::PageUp => {
                help_state.page_up();
                true
            }
            KeyCode::PageDown => {
                help_state.page_down(total_lines);
                true
            }

            // Toggle context mode
            KeyCode::Char('c') => {
                help_state.toggle_context_mode();
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
    use crate::state::UIState;

    #[test]
    fn test_help_widget_creation() {
        let _widget = HelpWidget::new();
        // Should not panic
    }

    #[test]
    fn test_help_widget_default() {
        let _widget = HelpWidget::default();
        // Should not panic
    }

    #[test]
    fn test_help_text_lines_global() {
        let lines = HelpWidget::help_text_lines(false);
        // Global mode should have all sections
        assert!(lines.len() > 20); // At least 20 lines with all sections
    }

    #[test]
    fn test_help_text_lines_context() {
        let lines = HelpWidget::help_text_lines(true);
        // Context mode should have fewer lines (only global shortcuts)
        assert!(lines.len() < 15); // Less than 15 lines (global only)
    }

    #[test]
    fn test_total_lines_global() {
        let total = HelpWidget::total_lines(false);
        assert!(total > 20);
    }

    #[test]
    fn test_total_lines_context() {
        let total = HelpWidget::total_lines(true);
        assert!(total < 15);
    }

    #[test]
    fn test_handle_event_scroll_up() {
        let mut ui_state = UIState::new();
        ui_state.help_widget_state.scroll_offset = 5;

        let event = Event::Key(KeyEvent::from(KeyCode::Up));
        let handled = handle_help_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.help_widget_state.scroll_offset, 4);
    }

    #[test]
    fn test_handle_event_scroll_down() {
        let mut ui_state = UIState::new();
        ui_state.help_widget_state.scroll_offset = 0;
        ui_state.help_widget_state.visible_rows = 10;

        let event = Event::Key(KeyEvent::from(KeyCode::Down));
        let handled = handle_help_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.help_widget_state.scroll_offset, 1);
    }

    #[test]
    fn test_handle_event_page_up() {
        let mut ui_state = UIState::new();
        ui_state.help_widget_state.scroll_offset = 25;
        ui_state.help_widget_state.visible_rows = 20;

        let event = Event::Key(KeyEvent::from(KeyCode::PageUp));
        let handled = handle_help_widget_event(event, &mut ui_state);

        assert!(handled);
        assert_eq!(ui_state.help_widget_state.scroll_offset, 5);
    }

    #[test]
    fn test_handle_event_page_down() {
        let mut ui_state = UIState::new();
        ui_state.help_widget_state.scroll_offset = 0;
        ui_state.help_widget_state.visible_rows = 20;

        let event = Event::Key(KeyEvent::from(KeyCode::PageDown));
        let handled = handle_help_widget_event(event, &mut ui_state);

        assert!(handled);
        assert!(ui_state.help_widget_state.scroll_offset > 0);
    }

    #[test]
    fn test_handle_event_toggle_context() {
        let mut ui_state = UIState::new();
        ui_state.help_widget_state.context_mode = false;

        let event = Event::Key(KeyEvent::from(KeyCode::Char('c')));
        let handled = handle_help_widget_event(event, &mut ui_state);

        assert!(handled);
        assert!(ui_state.help_widget_state.context_mode);
    }

    #[test]
    fn test_handle_event_unhandled() {
        let mut ui_state = UIState::new();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('x')));
        let handled = handle_help_widget_event(event, &mut ui_state);

        assert!(!handled);
    }
}
