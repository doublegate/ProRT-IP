//! Terminal layout components

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::state::{ScanState, UIState};
use crate::ui::theme;

/// Create the main layout with header, main area, and footer
///
/// Layout:
/// ```text
/// ┌─────────────────────────────┐
/// │ Header (3 lines)            │
/// ├─────────────────────────────┤
/// │                             │
/// │ Main Area (expandable)      │
/// │                             │
/// ├─────────────────────────────┤
/// │ Footer (1 line)             │
/// └─────────────────────────────┘
/// ```
pub fn create_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main area
            Constraint::Length(1), // Footer
        ])
        .split(area)
        .to_vec()
}

/// Render the header with scan metadata
pub fn render_header(scan_state: &ScanState) -> Paragraph<'static> {
    let stage_text = format!("Stage: {:?}", scan_state.stage);
    let progress_text = format!("Progress: {:.1}%", scan_state.progress_percentage);
    let throughput_text = format!("Throughput: {:.0} pps", scan_state.throughput_pps);

    let lines = vec![
        Line::from(vec![
            Span::styled("ProRT-IP Scanner", theme::header_style()),
            Span::raw(" | "),
            Span::styled(stage_text, theme::stage_style()),
        ]),
        Line::from(vec![
            Span::styled(progress_text, theme::progress_style()),
            Span::raw(" | "),
            Span::styled(throughput_text, theme::throughput_style()),
        ]),
    ];

    Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Scan Status"))
}

/// Render the main area (placeholder for Sprint 6.2 port table)
pub fn render_main_area(scan_state: &ScanState) -> Paragraph<'static> {
    let placeholder_text = format!(
        "Scan in progress...\n\
         Open ports: {}\n\
         Closed ports: {}\n\
         Filtered ports: {}\n\
         Services detected: {}\n\
         \n\
         (Port table will be implemented in Sprint 6.2)",
        scan_state.open_ports,
        scan_state.closed_ports,
        scan_state.filtered_ports,
        scan_state.detected_services,
    );

    Paragraph::new(placeholder_text).block(Block::default().borders(Borders::ALL).title("Results"))
}

/// Render the footer with help text
pub fn render_footer(ui_state: &UIState) -> Paragraph<'static> {
    let help_text = if ui_state.show_help {
        "Press ? to close help | q: Quit"
    } else {
        "q: Quit | ?: Help | Tab: Next Pane | hjkl: Navigate"
    };

    Paragraph::new(help_text).style(theme::footer_style())
}

/// Render the help screen overlay
pub fn render_help_screen() -> Paragraph<'static> {
    let help_text = "\
╔═══════════════════════════════════════════════════════════════╗
║                      ProRT-IP TUI Help                        ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║  Navigation:                                                  ║
║    q              Quit TUI                                    ║
║    ?              Toggle this help screen                     ║
║    Tab            Next pane                                   ║
║    Shift+Tab      Previous pane                               ║
║    h/←            Move left (Sprint 6.2+)                     ║
║    j/↓            Move down                                   ║
║    k/↑            Move up                                     ║
║    l/→            Move right (Sprint 6.2+)                    ║
║                                                               ║
║  Features:                                                    ║
║    • Real-time scan progress tracking                         ║
║    • Live port discovery updates                              ║
║    • 60 FPS rendering                                         ║
║    • EventBus integration                                     ║
║                                                               ║
║  Sprint 6.1: TUI Framework & EventBus Integration             ║
║  Foundation for future interactive features                   ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝

Press ? to close";

    Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL))
        .style(theme::help_style())
}
