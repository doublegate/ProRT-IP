//! StatusBar widget - displays scan progress, throughput, and statistics
//!
//! The StatusBar shows:
//! - Progress bar with percentage
//! - Throughput stats (current, average, peak)
//! - ETA and elapsed time
//! - Resource stats (hosts/ports discovered)

use parking_lot::RwLock;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use std::sync::Arc;
use std::time::Duration;

use crate::state::{ScanState, UIState};
use crate::widgets::Component;

/// StatusBar widget displaying scan progress and statistics
///
/// # Layout
///
/// The StatusBar is divided into three sections:
/// - Left: Progress bar (60% width)
/// - Middle: Throughput stats (20% width)
/// - Right: Resource stats (20% width)
///
/// # Examples
///
/// ```rust,ignore
/// use prtip_tui::widgets::StatusBar;
/// use prtip_tui::state::ScanState;
///
/// let scan_state = ScanState::shared();
/// let status_bar = StatusBar::new(scan_state);
/// ```
pub struct StatusBar {
    /// Shared scan state (thread-safe)
    scan_state: Arc<RwLock<ScanState>>,
}

impl StatusBar {
    /// Create a new StatusBar widget
    ///
    /// # Arguments
    ///
    /// * `scan_state` - Shared scan state from scanner
    pub fn new(scan_state: Arc<RwLock<ScanState>>) -> Self {
        Self { scan_state }
    }

    /// Format duration as "HH:MM:SS" or "MM:SS" if < 1 hour
    fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }

    /// Format ETA duration with units (e.g., "5m 23s", "2h 15m", "< 1s")
    fn format_eta(duration: Duration) -> String {
        let total_seconds = duration.as_secs();

        if total_seconds < 1 {
            return "< 1s".to_string();
        }

        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    /// Format throughput as "X.XX K/s" or "X.XX M/s"
    fn format_throughput(pps: f64) -> String {
        if pps >= 1_000_000.0 {
            format!("{:.2} M/s", pps / 1_000_000.0)
        } else if pps >= 1_000.0 {
            format!("{:.2} K/s", pps / 1_000.0)
        } else {
            format!("{:.0} /s", pps)
        }
    }

    /// Render the progress bar section
    fn render_progress(
        &self,
        frame: &mut Frame,
        area: Rect,
        scan_state: &ScanState,
        ui_state: &UIState,
    ) {
        let status_state = &ui_state.status_bar_state;

        // Progress percentage
        let progress_pct = scan_state.progress_percentage.clamp(0.0, 100.0);

        // ETA string
        let eta_str = if let Some(eta_duration) =
            status_state.calculate_eta(scan_state.completed, scan_state.total)
        {
            format!(" ETA: {}", Self::format_eta(eta_duration))
        } else if scan_state.completed >= scan_state.total && scan_state.total > 0 {
            " Complete".to_string()
        } else {
            String::new()
        };

        // Elapsed time string
        let elapsed_str = if let Some(elapsed) = status_state.elapsed() {
            format!(" Elapsed: {}", Self::format_duration(elapsed))
        } else {
            String::new()
        };

        // Progress label
        let label = format!(
            "{:.1}%{} {} | {}/{}",
            progress_pct, eta_str, elapsed_str, scan_state.completed, scan_state.total
        );

        // Color based on progress
        let gauge_style = if progress_pct >= 100.0 {
            Style::default().fg(Color::Green)
        } else if progress_pct >= 50.0 {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Progress"))
            .gauge_style(gauge_style.add_modifier(Modifier::BOLD))
            .label(label)
            .percent(progress_pct as u16);

        frame.render_widget(gauge, area);
    }

    /// Render the throughput stats section
    fn render_throughput(
        &self,
        frame: &mut Frame,
        area: Rect,
        scan_state: &ScanState,
        ui_state: &UIState,
    ) {
        let status_state = &ui_state.status_bar_state;

        // Current throughput
        let current_pps = scan_state.throughput_pps;

        // Average throughput
        let avg_pps = status_state.average_throughput();

        // Peak throughput
        let peak_pps = status_state.peak_throughput;

        let lines = vec![
            Line::from(vec![
                Span::raw("Cur: "),
                Span::styled(
                    Self::format_throughput(current_pps),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Avg: "),
                Span::styled(
                    Self::format_throughput(avg_pps),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::raw("Peak: "),
                Span::styled(
                    Self::format_throughput(peak_pps),
                    Style::default().fg(Color::Green),
                ),
            ]),
        ];

        let paragraph =
            Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Throughput"));

        frame.render_widget(paragraph, area);
    }

    /// Render the resource stats section
    fn render_resources(&self, frame: &mut Frame, area: Rect, scan_state: &ScanState) {
        let total_ports =
            scan_state.open_ports + scan_state.closed_ports + scan_state.filtered_ports;

        let lines = vec![
            Line::from(vec![
                Span::raw("Hosts: "),
                Span::styled(
                    format!("{}", scan_state.discovered_hosts.len()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("Ports: "),
                Span::styled(
                    format!("{}", total_ports),
                    Style::default().fg(Color::White),
                ),
                Span::raw(" ("),
                Span::styled(
                    format!("{}", scan_state.open_ports),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(" open)"),
            ]),
            Line::from(vec![
                Span::raw("Svc: "),
                Span::styled(
                    format!("{}", scan_state.detected_services),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
        ];

        let paragraph =
            Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Resources"));

        frame.render_widget(paragraph, area);
    }
}

impl Component for StatusBar {
    fn render(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        // Read shared scan state (brief lock)
        let scan_state = self.scan_state.read();

        // Layout: [Progress 60%][Throughput 20%][Resources 20%]
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .split(area);

        // Render each section
        self.render_progress(frame, chunks[0], &scan_state, state);
        self.render_throughput(frame, chunks[1], &scan_state, state);
        self.render_resources(frame, chunks[2], &scan_state);
    }

    // StatusBar doesn't handle events (read-only display)
    // Default implementation returns false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_under_hour() {
        let duration = Duration::from_secs(125); // 2m 5s
        assert_eq!(StatusBar::format_duration(duration), "02:05");
    }

    #[test]
    fn test_format_duration_over_hour() {
        let duration = Duration::from_secs(3665); // 1h 1m 5s
        assert_eq!(StatusBar::format_duration(duration), "01:01:05");
    }

    #[test]
    fn test_format_duration_zero() {
        let duration = Duration::from_secs(0);
        assert_eq!(StatusBar::format_duration(duration), "00:00");
    }

    #[test]
    fn test_format_eta_seconds() {
        let duration = Duration::from_secs(45);
        assert_eq!(StatusBar::format_eta(duration), "45s");
    }

    #[test]
    fn test_format_eta_minutes() {
        let duration = Duration::from_secs(185); // 3m 5s
        assert_eq!(StatusBar::format_eta(duration), "3m 5s");
    }

    #[test]
    fn test_format_eta_hours() {
        let duration = Duration::from_secs(7325); // 2h 2m 5s
        assert_eq!(StatusBar::format_eta(duration), "2h 2m");
    }

    #[test]
    fn test_format_eta_under_second() {
        let duration = Duration::from_millis(500);
        assert_eq!(StatusBar::format_eta(duration), "< 1s");
    }

    #[test]
    fn test_format_throughput_per_second() {
        assert_eq!(StatusBar::format_throughput(523.7), "524 /s");
    }

    #[test]
    fn test_format_throughput_thousands() {
        assert_eq!(StatusBar::format_throughput(12_345.0), "12.35 K/s");
    }

    #[test]
    fn test_format_throughput_millions() {
        assert_eq!(StatusBar::format_throughput(8_500_000.0), "8.50 M/s");
    }

    #[test]
    fn test_statusbar_creation() {
        let scan_state = ScanState::shared();
        let _status_bar = StatusBar::new(scan_state);
        // Should not panic - just testing construction
    }
}
