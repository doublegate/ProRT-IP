//! Metrics Dashboard Widget - displays real-time scan metrics
//!
//! This widget provides a comprehensive overview of scan progress, throughput,
//! and statistics in a 3-column layout.

use crossterm::event::Event;
use parking_lot::RwLock;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;
use std::sync::Arc;
use std::time::Duration;

use crate::state::{ScanState, UIState};

#[cfg(test)]
use crate::state::ThroughputSample;
use crate::widgets::Component;

/// Metrics Dashboard Widget - displays real-time scan metrics
///
/// # Layout
///
/// The widget is divided into three columns:
/// - **Progress**: Scan stage, progress bar, ETA
/// - **Throughput**: Current/average/peak throughput, event rate
/// - **Statistics**: Open ports, services detected, errors, scan duration
///
/// # Example
///
/// ```rust,no_run
/// use prtip_tui::widgets::MetricsDashboardWidget;
/// use prtip_tui::state::ScanState;
/// use std::sync::Arc;
/// use parking_lot::RwLock;
///
/// let scan_state = Arc::new(RwLock::new(ScanState::new()));
/// let widget = MetricsDashboardWidget::new(scan_state);
/// ```
pub struct MetricsDashboardWidget {
    /// Shared scan state (read-only access)
    scan_state: Arc<RwLock<ScanState>>,
}

impl MetricsDashboardWidget {
    /// Create a new MetricsDashboardWidget
    ///
    /// # Arguments
    ///
    /// * `scan_state` - Shared scan state for reading metrics
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use prtip_tui::widgets::MetricsDashboardWidget;
    /// use prtip_tui::state::ScanState;
    /// use std::sync::Arc;
    /// use parking_lot::RwLock;
    ///
    /// let scan_state = Arc::new(RwLock::new(ScanState::new()));
    /// let widget = MetricsDashboardWidget::new(scan_state);
    /// ```
    pub fn new(scan_state: Arc<RwLock<ScanState>>) -> Self {
        Self { scan_state }
    }

    /// Calculate 5-second rolling average throughput
    ///
    /// Returns the average throughput (packets per second) over the last
    /// 5 samples from the throughput history. If fewer than 5 samples exist,
    /// averages all available samples.
    fn calculate_avg_throughput(&self, scan_state: &ScanState) -> f64 {
        if scan_state.throughput_history.is_empty() {
            return 0.0;
        }

        let samples: Vec<_> = scan_state.throughput_history.iter().rev().take(5).collect();

        let sum: f64 = samples.iter().map(|s| s.packets_per_second).sum();
        sum / samples.len() as f64
    }

    /// Calculate peak throughput from history
    ///
    /// Returns the maximum throughput (packets per second) observed
    /// in the throughput history.
    fn calculate_peak_throughput(&self, scan_state: &ScanState) -> f64 {
        scan_state
            .throughput_history
            .iter()
            .map(|s| s.packets_per_second)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Format Duration as human-readable string
    ///
    /// # Format
    ///
    /// - Seconds only: "45s"
    /// - Minutes and seconds: "2m 34s"
    /// - Hours, minutes, and seconds: "1h 12m 45s"
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::time::Duration;
    /// # use prtip_tui::widgets::MetricsDashboardWidget;
    /// # // This is a private method, but we test it via integration
    /// # // Just showing the expected format here
    /// let duration = Duration::from_secs(154);
    /// // Would format as "2m 34s"
    /// ```
    fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, mins, secs)
        } else if mins > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}s", secs)
        }
    }

    /// Format large numbers with commas
    ///
    /// # Example
    ///
    /// ```rust
    /// # use prtip_tui::widgets::MetricsDashboardWidget;
    /// # // This is a private method, but we test it via integration
    /// # // Just showing the expected format here
    /// let num = 12345u64;
    /// // Would format as "12,345"
    /// let num2 = 1234567u64;
    /// // Would format as "1,234,567"
    /// ```
    fn format_number(n: u64) -> String {
        let s = n.to_string();
        let bytes = s.as_bytes();
        let chunks: Vec<&str> = bytes
            .rchunks(3)
            .rev()
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect();
        chunks.join(",")
    }

    /// Format throughput as "12,345 p/s"
    fn format_throughput(pps: f64) -> String {
        format!("{} p/s", Self::format_number(pps as u64))
    }

    /// Get progress bar color based on percentage
    ///
    /// - Green (>=75%)
    /// - Yellow (25-75%)
    /// - Cyan (<25%)
    fn get_progress_color(percentage: f32) -> Color {
        if percentage >= 75.0 {
            Color::Green
        } else if percentage >= 25.0 {
            Color::Yellow
        } else {
            Color::Cyan
        }
    }

    /// Get status color based on errors and warnings
    ///
    /// - Red (errors > 0)
    /// - Yellow (warnings > 0)
    /// - Green (healthy)
    fn get_status_color(errors: usize, warnings: usize) -> Color {
        if errors > 0 {
            Color::Red
        } else if warnings > 0 {
            Color::Yellow
        } else {
            Color::Green
        }
    }

    /// Get status indicator text with color
    fn get_status_indicator(errors: usize, warnings: usize) -> (String, Color) {
        let color = Self::get_status_color(errors, warnings);
        let text = if errors > 0 {
            format!("Error ({}) ●", errors)
        } else if warnings > 0 {
            format!("Warning ({}) ●", warnings)
        } else {
            "Healthy ●".to_string()
        };
        (text, color)
    }

    /// Render progress section (left column)
    fn render_progress_section(&self, frame: &mut Frame, area: Rect, scan_state: &ScanState) {
        let block = Block::default()
            .title("Progress")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Create vertical layout for progress metrics
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Stage
                Constraint::Length(1), // Progress %
                Constraint::Length(1), // Progress bar
                Constraint::Length(1), // Spacing
                Constraint::Length(1), // Completed
                Constraint::Length(1), // Total
                Constraint::Length(1), // Spacing
                Constraint::Length(1), // ETA
                Constraint::Min(0),    // Remaining space
            ])
            .split(inner);

        // Stage
        let stage_text = format!("Stage: {:?}", scan_state.stage);
        let stage = Paragraph::new(stage_text).style(Style::default().fg(Color::White));
        frame.render_widget(stage, chunks[0]);

        // Progress percentage
        let progress_text = format!("Progress: {:.1}%", scan_state.progress_percentage);
        let progress_para = Paragraph::new(progress_text)
            .style(Style::default().fg(Self::get_progress_color(scan_state.progress_percentage)));
        frame.render_widget(progress_para, chunks[1]);

        // Progress bar
        let ratio = if scan_state.total > 0 {
            scan_state.completed as f64 / scan_state.total as f64
        } else {
            0.0
        };
        let gauge = Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(Self::get_progress_color(scan_state.progress_percentage))
                    .add_modifier(Modifier::BOLD),
            )
            .ratio(ratio);
        frame.render_widget(gauge, chunks[2]);

        // Completed
        let completed_text = format!("Completed: {}", Self::format_number(scan_state.completed));
        let completed = Paragraph::new(completed_text).style(Style::default().fg(Color::White));
        frame.render_widget(completed, chunks[4]);

        // Total
        let total_text = format!("Total: {}", Self::format_number(scan_state.total));
        let total = Paragraph::new(total_text).style(Style::default().fg(Color::White));
        frame.render_widget(total, chunks[5]);

        // ETA
        let eta_text = if let Some(eta) = scan_state.eta {
            format!("ETA: {}", Self::format_duration(eta))
        } else {
            "ETA: Calculating...".to_string()
        };
        let eta = Paragraph::new(eta_text).style(Style::default().fg(Color::Cyan));
        frame.render_widget(eta, chunks[7]);
    }

    /// Render throughput section (middle column)
    fn render_throughput_section(&self, frame: &mut Frame, area: Rect, scan_state: &ScanState) {
        let block = Block::default()
            .title("Throughput")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Create vertical layout for throughput metrics
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Current
                Constraint::Length(1), // Average
                Constraint::Length(1), // Peak
                Constraint::Length(1), // Spacing
                Constraint::Length(1), // Packets sent
                Constraint::Length(1), // Responses (estimated)
                Constraint::Length(1), // Spacing
                Constraint::Length(1), // Event rate
                Constraint::Min(0),    // Remaining space
            ])
            .split(inner);

        // Current throughput
        let current_text = format!(
            "Current: {}",
            Self::format_throughput(scan_state.throughput_pps)
        );
        let current = Paragraph::new(current_text).style(Style::default().fg(Color::Green));
        frame.render_widget(current, chunks[0]);

        // Average throughput (5-second rolling)
        let avg = self.calculate_avg_throughput(scan_state);
        let avg_text = format!("Average: {}", Self::format_throughput(avg));
        let avg_para = Paragraph::new(avg_text).style(Style::default().fg(Color::Yellow));
        frame.render_widget(avg_para, chunks[1]);

        // Peak throughput
        let peak = self.calculate_peak_throughput(scan_state);
        let peak_text = format!("Peak: {}", Self::format_throughput(peak));
        let peak_para = Paragraph::new(peak_text).style(Style::default().fg(Color::Cyan));
        frame.render_widget(peak_para, chunks[2]);

        // Packets sent (estimate from completed)
        let packets_text = if scan_state.completed > 1_000_000 {
            format!(
                "Packets Sent: {:.1}M",
                scan_state.completed as f64 / 1_000_000.0
            )
        } else if scan_state.completed > 1_000 {
            format!(
                "Packets Sent: {:.1}K",
                scan_state.completed as f64 / 1_000.0
            )
        } else {
            format!("Packets Sent: {}", scan_state.completed)
        };
        let packets = Paragraph::new(packets_text).style(Style::default().fg(Color::White));
        frame.render_widget(packets, chunks[4]);

        // Responses (estimate from open + closed + filtered)
        let responses = scan_state.open_ports + scan_state.closed_ports + scan_state.filtered_ports;
        let responses_text = if responses > 1_000_000 {
            format!("Responses: {:.1}M", responses as f64 / 1_000_000.0)
        } else if responses > 1_000 {
            format!("Responses: {:.1}K", responses as f64 / 1_000.0)
        } else {
            format!("Responses: {}", responses)
        };
        let resp_para = Paragraph::new(responses_text).style(Style::default().fg(Color::White));
        frame.render_widget(resp_para, chunks[5]);

        // Event rate (placeholder - would need EventBus integration)
        let event_rate_text = "Event Rate: N/A".to_string();
        let event_rate =
            Paragraph::new(event_rate_text).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(event_rate, chunks[7]);
    }

    /// Render statistics section (right column)
    fn render_statistics_section(&self, frame: &mut Frame, area: Rect, scan_state: &ScanState) {
        let block = Block::default()
            .title("Statistics")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Create vertical layout for statistics
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Open ports
                Constraint::Length(1), // Services
                Constraint::Length(1), // Errors
                Constraint::Length(1), // Spacing
                Constraint::Length(1), // Scan duration
                Constraint::Length(1), // Est. remaining
                Constraint::Length(1), // Spacing
                Constraint::Length(1), // Status
                Constraint::Min(0),    // Remaining space
            ])
            .split(inner);

        // Open ports
        let open_text = format!("Open Ports: {}", scan_state.open_ports);
        let open = Paragraph::new(open_text).style(Style::default().fg(Color::Green));
        frame.render_widget(open, chunks[0]);

        // Services detected
        let services_text = format!("Services: {}", scan_state.detected_services);
        let services = Paragraph::new(services_text).style(Style::default().fg(Color::Cyan));
        frame.render_widget(services, chunks[1]);

        // Errors
        let error_color = if scan_state.errors > 0 {
            Color::Red
        } else {
            Color::White
        };
        let errors_text = format!("Errors: {}", scan_state.errors);
        let errors = Paragraph::new(errors_text).style(Style::default().fg(error_color));
        frame.render_widget(errors, chunks[2]);

        // Scan duration (calculate from scan_start_time if available)
        let duration = if let Some(start_time) = scan_state.scan_start_time {
            start_time.elapsed()
        } else {
            Duration::from_secs(0)
        };
        let duration_text = format!("Scan Duration: {}", Self::format_duration(duration));
        let duration_para = Paragraph::new(duration_text).style(Style::default().fg(Color::White));
        frame.render_widget(duration_para, chunks[4]);

        // Est. remaining (same as ETA)
        let remaining_text = if let Some(eta) = scan_state.eta {
            format!("Est. Remaining: {}", Self::format_duration(eta))
        } else {
            "Est. Remaining: N/A".to_string()
        };
        let remaining = Paragraph::new(remaining_text).style(Style::default().fg(Color::Cyan));
        frame.render_widget(remaining, chunks[5]);

        // Status indicator
        let (status_text, status_color) =
            Self::get_status_indicator(scan_state.errors, scan_state.warnings.len());
        let status = Paragraph::new(status_text).style(
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(status, chunks[7]);
    }
}

impl Component for MetricsDashboardWidget {
    fn render(&self, frame: &mut Frame, area: Rect, _state: &UIState) {
        // Read scan state once at the start
        let scan_state = self.scan_state.read();

        // Create 3-column layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33), // Progress
                Constraint::Percentage(33), // Throughput
                Constraint::Percentage(34), // Statistics (slightly wider for alignment)
            ])
            .split(area);

        // Render each section
        self.render_progress_section(frame, chunks[0], &scan_state);
        self.render_throughput_section(frame, chunks[1], &scan_state);
        self.render_statistics_section(frame, chunks[2], &scan_state);
    }

    fn handle_event(&mut self, _event: Event) -> bool {
        // Metrics dashboard is read-only, no event handling needed
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn create_test_scan_state() -> Arc<RwLock<ScanState>> {
        let mut state = ScanState::new();
        // stage stays at default (ScanStage::Initializing)
        state.progress_percentage = 65.3;
        state.completed = 65300;
        state.total = 100000;
        state.throughput_pps = 12345.0;
        state.eta = Some(Duration::from_secs(165)); // 2m 45s
        state.open_ports = 42;
        state.detected_services = 15;
        state.errors = 2;
        Arc::new(RwLock::new(state))
    }

    #[test]
    fn test_widget_creation() {
        let scan_state = create_test_scan_state();
        let _widget = MetricsDashboardWidget::new(scan_state);
        // Widget created successfully (no start_time field anymore)
    }

    #[test]
    fn test_format_duration_seconds_only() {
        let duration = Duration::from_secs(45);
        assert_eq!(MetricsDashboardWidget::format_duration(duration), "45s");
    }

    #[test]
    fn test_format_duration_minutes_seconds() {
        let duration = Duration::from_secs(154); // 2m 34s
        assert_eq!(MetricsDashboardWidget::format_duration(duration), "2m 34s");
    }

    #[test]
    fn test_format_duration_hours_minutes_seconds() {
        let duration = Duration::from_secs(4365); // 1h 12m 45s
        assert_eq!(
            MetricsDashboardWidget::format_duration(duration),
            "1h 12m 45s"
        );
    }

    #[test]
    fn test_format_number_thousands() {
        assert_eq!(MetricsDashboardWidget::format_number(12345), "12,345");
    }

    #[test]
    fn test_format_number_millions() {
        assert_eq!(MetricsDashboardWidget::format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_number_small() {
        assert_eq!(MetricsDashboardWidget::format_number(123), "123");
    }

    #[test]
    fn test_format_throughput() {
        assert_eq!(
            MetricsDashboardWidget::format_throughput(12345.67),
            "12,345 p/s"
        );
    }

    #[test]
    fn test_get_progress_color_low() {
        assert_eq!(
            MetricsDashboardWidget::get_progress_color(10.0),
            Color::Cyan
        );
    }

    #[test]
    fn test_get_progress_color_medium() {
        assert_eq!(
            MetricsDashboardWidget::get_progress_color(50.0),
            Color::Yellow
        );
    }

    #[test]
    fn test_get_progress_color_high() {
        assert_eq!(
            MetricsDashboardWidget::get_progress_color(85.0),
            Color::Green
        );
    }

    #[test]
    fn test_get_status_color_healthy() {
        assert_eq!(MetricsDashboardWidget::get_status_color(0, 0), Color::Green);
    }

    #[test]
    fn test_get_status_color_warning() {
        assert_eq!(
            MetricsDashboardWidget::get_status_color(0, 3),
            Color::Yellow
        );
    }

    #[test]
    fn test_get_status_color_error() {
        assert_eq!(MetricsDashboardWidget::get_status_color(5, 3), Color::Red);
    }

    #[test]
    fn test_get_status_indicator_healthy() {
        let (text, color) = MetricsDashboardWidget::get_status_indicator(0, 0);
        assert_eq!(text, "Healthy ●");
        assert_eq!(color, Color::Green);
    }

    #[test]
    fn test_get_status_indicator_warning() {
        let (text, color) = MetricsDashboardWidget::get_status_indicator(0, 3);
        assert_eq!(text, "Warning (3) ●");
        assert_eq!(color, Color::Yellow);
    }

    #[test]
    fn test_get_status_indicator_error() {
        let (text, color) = MetricsDashboardWidget::get_status_indicator(5, 3);
        assert_eq!(text, "Error (5) ●");
        assert_eq!(color, Color::Red);
    }

    #[test]
    fn test_calculate_avg_throughput_empty() {
        let scan_state = Arc::new(RwLock::new(ScanState::new()));
        let widget = MetricsDashboardWidget::new(Arc::clone(&scan_state));
        let state = scan_state.read();
        assert_eq!(widget.calculate_avg_throughput(&state), 0.0);
    }

    #[test]
    fn test_calculate_avg_throughput_less_than_5() {
        let scan_state = Arc::new(RwLock::new(ScanState::new()));
        {
            let mut state = scan_state.write();
            state.throughput_history.push_back(ThroughputSample {
                timestamp: Instant::now(),
                packets_per_second: 1000.0,
            });
            state.throughput_history.push_back(ThroughputSample {
                timestamp: Instant::now(),
                packets_per_second: 2000.0,
            });
            state.throughput_history.push_back(ThroughputSample {
                timestamp: Instant::now(),
                packets_per_second: 3000.0,
            });
        }

        let widget = MetricsDashboardWidget::new(Arc::clone(&scan_state));
        let state = scan_state.read();
        assert_eq!(widget.calculate_avg_throughput(&state), 2000.0);
    }

    #[test]
    fn test_calculate_avg_throughput_full_window() {
        let scan_state = Arc::new(RwLock::new(ScanState::new()));
        {
            let mut state = scan_state.write();
            // Add 7 samples, should use last 5
            for i in 1..=7 {
                state.throughput_history.push_back(ThroughputSample {
                    timestamp: Instant::now(),
                    packets_per_second: (i * 1000) as f64,
                });
            }
        }

        let widget = MetricsDashboardWidget::new(Arc::clone(&scan_state));
        let state = scan_state.read();
        // Average of last 5: (3000 + 4000 + 5000 + 6000 + 7000) / 5 = 5000
        assert_eq!(widget.calculate_avg_throughput(&state), 5000.0);
    }

    #[test]
    fn test_calculate_peak_throughput_empty() {
        let scan_state = Arc::new(RwLock::new(ScanState::new()));
        let widget = MetricsDashboardWidget::new(Arc::clone(&scan_state));
        let state = scan_state.read();
        assert_eq!(widget.calculate_peak_throughput(&state), 0.0);
    }

    #[test]
    fn test_calculate_peak_throughput() {
        let scan_state = Arc::new(RwLock::new(ScanState::new()));
        {
            let mut state = scan_state.write();
            state.throughput_history.push_back(ThroughputSample {
                timestamp: Instant::now(),
                packets_per_second: 1000.0,
            });
            state.throughput_history.push_back(ThroughputSample {
                timestamp: Instant::now(),
                packets_per_second: 5000.0, // Peak
            });
            state.throughput_history.push_back(ThroughputSample {
                timestamp: Instant::now(),
                packets_per_second: 2000.0,
            });
        }

        let widget = MetricsDashboardWidget::new(Arc::clone(&scan_state));
        let state = scan_state.read();
        assert_eq!(widget.calculate_peak_throughput(&state), 5000.0);
    }

    #[test]
    fn test_component_trait_handle_event() {
        let scan_state = create_test_scan_state();
        let mut widget = MetricsDashboardWidget::new(scan_state);

        // Metrics dashboard doesn't handle events (read-only)
        let event = Event::Key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char('q'),
        ));
        assert!(!widget.handle_event(event));
    }

    #[test]
    fn test_render_with_minimal_state() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let scan_state = Arc::new(RwLock::new(ScanState::new()));
        let widget = MetricsDashboardWidget::new(scan_state);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                let ui_state = UIState::default();
                widget.render(frame, area, &ui_state);
            })
            .unwrap();

        // If render doesn't panic, test passes
    }
}
