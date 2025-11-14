//! 60 FPS rendering loop

use ratatui::Frame;

use crate::state::{ScanState, UIState};
use crate::ui::layout;

/// Render the TUI to the frame
///
/// This function is called at 60 FPS (16.67ms budget per frame).
/// It uses immediate mode rendering - the entire UI is redrawn each frame,
/// but ratatui only renders the diff.
///
/// # Arguments
///
/// * `frame` - The ratatui frame to render to
/// * `scan_state` - Current scan state
/// * `ui_state` - Current UI state
pub fn render(frame: &mut Frame, scan_state: &ScanState, ui_state: &UIState) {
    let chunks = layout::create_layout(frame.area());

    // Render header
    let header = layout::render_header(scan_state);
    frame.render_widget(header, chunks[0]);

    // Render main area (placeholder for Sprint 6.2)
    let main_area = layout::render_main_area(scan_state);
    frame.render_widget(main_area, chunks[1]);

    // Render footer with help text
    let footer = layout::render_footer(ui_state);
    frame.render_widget(footer, chunks[2]);

    // Render help screen if visible
    if ui_state.show_help {
        let help = layout::render_help_screen();
        let area = frame.area();
        frame.render_widget(help, area);
    }
}
