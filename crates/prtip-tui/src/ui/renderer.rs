//! 60 FPS rendering loop

use parking_lot::RwLock;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Tabs;
use ratatui::Frame;
use std::sync::Arc;

use crate::state::{DashboardTab, ScanState, UIState};
use crate::ui::layout;
use crate::widgets::{Component, MetricsDashboardWidget, PortTableWidget, ServiceTableWidget};

/// Render the TUI to the frame
///
/// This function is called at 60 FPS (16.67ms budget per frame).
/// It uses immediate mode rendering - the entire UI is redrawn each frame,
/// but ratatui only renders the diff.
///
/// # Arguments
///
/// * `frame` - The ratatui frame to render to
/// * `scan_state` - Arc reference to shared scan state
/// * `ui_state` - Current UI state
pub fn render(frame: &mut Frame, scan_state: Arc<RwLock<ScanState>>, ui_state: &UIState) {
    let chunks = layout::create_layout(frame.area());

    // Read scan state for header (brief lock)
    let scan_state_snapshot = scan_state.read();

    // Render header
    let header = layout::render_header(&scan_state_snapshot);
    frame.render_widget(header, chunks[0]);

    // Drop lock before rendering main area
    drop(scan_state_snapshot);

    // Sprint 6.2 Task 2.3: Split main area into tab bar + content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tab bar
            Constraint::Min(0),    // Content area
        ])
        .split(chunks[1]);

    // Render tab bar
    let tab_titles = vec![
        Line::from("Port Table"),
        Line::from("Service Table"),
        Line::from("Metrics"),
    ];
    let active_tab_index = match ui_state.active_dashboard_tab {
        DashboardTab::PortTable => 0,
        DashboardTab::ServiceTable => 1,
        DashboardTab::Metrics => 2,
    };
    let tabs = Tabs::new(tab_titles)
        .select(active_tab_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");
    frame.render_widget(tabs, main_chunks[0]);

    // Render active widget in content area
    match ui_state.active_dashboard_tab {
        DashboardTab::PortTable => {
            let port_table = PortTableWidget::new(Arc::clone(&scan_state));
            port_table.render(frame, main_chunks[1], ui_state);
        }
        DashboardTab::ServiceTable => {
            let service_table = ServiceTableWidget::new(Arc::clone(&scan_state));
            service_table.render(frame, main_chunks[1], ui_state);
        }
        DashboardTab::Metrics => {
            let metrics_dashboard = MetricsDashboardWidget::new(Arc::clone(&scan_state));
            metrics_dashboard.render(frame, main_chunks[1], ui_state);
        }
    }

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
