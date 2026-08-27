//! ServiceTableWidget - Live service detection table with confidence-based filtering
//!
//! The ServiceTableWidget displays detected services in real-time during a scan with:
//! - 6 columns: Timestamp, IP, Port, Service Name, Version, Confidence
//! - Row selection with highlighting
//! - Keyboard navigation (arrow keys, Page Up/Down, Home/End)
//! - Sorting by any column (ascending/descending)
//! - Confidence-based filtering (All, Low ≥50%, Medium ≥75%, High ≥90%)
//! - Service name search (optional)
//! - Color-coded confidence levels (green ≥90%, yellow 50-89%, red <50%)
//! - Auto-scroll mode for following latest detections
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

use crate::state::{ScanState, ServiceTableColumn, ServiceTableState, SortOrder, UIState};
use crate::widgets::Component;

/// ServiceTableWidget displaying live service detections
///
/// # Layout
///
/// ```text
/// ┌─ Service Detections ──────────────────────────────────────────────────┐
/// │ Time  │ IP           │ Port │ Service │ Version      │ Confidence      │
/// │───────┼──────────────┼──────┼─────────┼──────────────┼─────────────────│
/// │ 12:34 │ 192.168.1.1  │   80 │ http    │ Apache/2.4   │ 95% (green)     │
/// │ 12:34 │ 192.168.1.1  │  443 │ https   │ nginx/1.18   │ 85% (yellow)    │
/// │ 12:35 │ 192.168.1.2  │   22 │ ssh     │ OpenSSH/8.0  │ 45% (red)       │
/// └────────────────────────────────────────────────────────────────────────┘
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
/// - `n` - Sort by service name
/// - `v` - Sort by version
/// - `c` - Sort by confidence
/// - `f` - Cycle confidence filter (All → Low → Medium → High)
/// - `a` - Toggle auto-scroll
///
/// # Examples
///
/// ```rust,ignore
/// use prtip_tui::widgets::ServiceTableWidget;
/// use prtip_tui::state::ScanState;
///
/// let scan_state = ScanState::shared();
/// let service_table = ServiceTableWidget::new(scan_state);
/// ```
pub struct ServiceTableWidget {
    /// Shared scan state (thread-safe)
    scan_state: Arc<RwLock<ScanState>>,
}

impl ServiceTableWidget {
    /// Create a new ServiceTableWidget
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
            12, // Service Name
            20, // Version
            12, // Confidence (95% or 95% (High))
        ]
    }

    /// Render column headers with sort indicators
    fn render_header(ui_state: &UIState) -> Row<'static> {
        let service_table_state = &ui_state.service_table_state;

        // Sort indicator
        let sort_indicator = |column: ServiceTableColumn| -> &'static str {
            if service_table_state.sort_column == column {
                match service_table_state.sort_order {
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
                sort_indicator(ServiceTableColumn::Timestamp)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "IP{}",
                sort_indicator(ServiceTableColumn::Ip)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "Port{}",
                sort_indicator(ServiceTableColumn::Port)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "Service{}",
                sort_indicator(ServiceTableColumn::ServiceName)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "Version{}",
                sort_indicator(ServiceTableColumn::Version)
            )))
            .style(header_style),
            Cell::from(Text::from(format!(
                "Confidence{}",
                sort_indicator(ServiceTableColumn::Confidence)
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

    /// Get confidence color (green ≥90%, yellow 50-89%, red <50%)
    fn get_confidence_color(confidence: f32) -> Color {
        if confidence >= 0.9 {
            Color::Green
        } else if confidence >= 0.5 {
            Color::Yellow
        } else {
            Color::Red
        }
    }

    /// Convert ServiceDetection to table row
    fn service_detection_to_row(
        detection: &crate::state::ServiceDetection,
        is_selected: bool,
    ) -> Row<'static> {
        // Confidence color
        let confidence_color = Self::get_confidence_color(detection.confidence);

        // Row style
        let row_style = if is_selected {
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        Row::new(vec![
            Cell::from(Text::from(Self::format_timestamp(detection.timestamp))),
            Cell::from(Text::from(detection.ip.to_string())),
            Cell::from(Text::from(format!("{}", detection.port))),
            Cell::from(Text::from(detection.service_name.clone())),
            Cell::from(Text::from(
                detection
                    .service_version
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            )),
            Cell::from(Text::from(format!("{:.0}%", detection.confidence * 100.0)))
                .style(Style::default().fg(confidence_color)),
        ])
        .style(row_style)
        .height(1)
    }

    /// Apply filters to service detections
    fn apply_filters(
        detections: &[crate::state::ServiceDetection],
        service_table_state: &ServiceTableState,
    ) -> Vec<crate::state::ServiceDetection> {
        detections
            .iter()
            .filter(|d| {
                // Filter by confidence threshold
                let min_confidence = service_table_state.confidence_filter.min_confidence();
                if d.confidence < min_confidence {
                    return false;
                }

                // Filter by service name (case-insensitive substring match)
                if let Some(ref filter_name) = service_table_state.filter_service_name {
                    let name_lower = filter_name.to_lowercase();
                    let service_lower = d.service_name.to_lowercase();
                    if !service_lower.contains(&name_lower) {
                        return false;
                    }
                }

                // Filter by port
                if let Some(filter_port) = service_table_state.filter_port {
                    if d.port != filter_port {
                        return false;
                    }
                }

                // Filter by IP (case-insensitive substring match)
                if let Some(ref filter_ip) = service_table_state.filter_ip {
                    let ip_str = d.ip.to_string().to_lowercase();
                    let filter_lower = filter_ip.to_lowercase();
                    if !ip_str.contains(&filter_lower) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    /// Apply sorting to service detections
    fn apply_sorting(
        mut detections: Vec<crate::state::ServiceDetection>,
        service_table_state: &ServiceTableState,
    ) -> Vec<crate::state::ServiceDetection> {
        use ServiceTableColumn::*;

        detections.sort_by(|a, b| {
            let ordering = match service_table_state.sort_column {
                Timestamp => a.timestamp.cmp(&b.timestamp),
                Ip => a.ip.cmp(&b.ip),
                Port => a.port.cmp(&b.port),
                ServiceName => a.service_name.cmp(&b.service_name),
                Version => {
                    // Sort by version (None last)
                    match (&a.service_version, &b.service_version) {
                        (Some(a_ver), Some(b_ver)) => a_ver.cmp(b_ver),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                }
                Confidence => {
                    // Sort by confidence (floating point)
                    a.confidence
                        .partial_cmp(&b.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            };

            match service_table_state.sort_order {
                SortOrder::Ascending => ordering,
                SortOrder::Descending => ordering.reverse(),
            }
        });

        detections
    }
}

impl Component for ServiceTableWidget {
    fn render(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        // Read shared scan state (brief lock)
        let scan_state = self.scan_state.read();
        let service_table_state = &state.service_table_state;

        // Convert VecDeque to Vec for processing
        let all_detections: Vec<_> = scan_state.service_detections.iter().cloned().collect();

        // Apply filters
        let filtered_detections = Self::apply_filters(&all_detections, service_table_state);

        // Apply sorting
        let sorted_detections = Self::apply_sorting(filtered_detections, service_table_state);

        // Calculate pagination
        let total_rows = sorted_detections.len();
        let visible_rows = service_table_state.visible_rows;
        let scroll_offset = service_table_state.scroll_offset;
        let selected_row = service_table_state.selected_row;

        // Get visible slice
        let visible_detections: Vec<Row> = sorted_detections
            .iter()
            .skip(scroll_offset)
            .take(visible_rows)
            .enumerate()
            .map(|(idx, detection)| {
                let is_selected = (scroll_offset + idx) == selected_row;
                Self::service_detection_to_row(detection, is_selected)
            })
            .collect();

        // Column widths
        let widths = Self::column_widths();

        // Build title with filter info
        let filter_info = format!(
            " [Filter: {}]",
            service_table_state.confidence_filter.display_name()
        );

        // Build table
        let table = Table::new(
            visible_detections,
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
            "Service Detections ({}/{} shown){}{}",
            total_rows,
            all_detections.len(),
            filter_info,
            if service_table_state.auto_scroll {
                " [AUTO]"
            } else {
                ""
            }
        )));

        frame.render_widget(table, area);
    }

    fn handle_event(&mut self, event: Event) -> bool {
        // ServiceTableWidget state is in UIState, which we can't mutate here
        // Event handling happens in App::run() which has mutable access to UIState

        // For now, return false (event not handled)
        // This will be wired up in App::run() event dispatch
        let _ = event;
        false
    }
}

/// Event handler for ServiceTableWidget keyboard shortcuts
///
/// This function is called from App::run() when ServiceTableWidget is focused.
/// It mutates UIState::service_table_state based on keyboard events.
///
/// # Returns
///
/// `true` if the event was handled, `false` otherwise
pub fn handle_service_table_event(
    event: Event,
    ui_state: &mut UIState,
    scan_state: &Arc<RwLock<ScanState>>,
) -> bool {
    if let Event::Key(KeyEvent { code, .. }) = event {
        let service_table_state = &mut ui_state.service_table_state;

        // Calculate total rows for navigation
        let scan_state_lock = scan_state.read();
        let all_detections: Vec<_> = scan_state_lock.service_detections.iter().cloned().collect();
        drop(scan_state_lock);

        let filtered_detections =
            ServiceTableWidget::apply_filters(&all_detections, service_table_state);
        let total_rows = filtered_detections.len();

        match code {
            // Navigation
            KeyCode::Up => {
                service_table_state.select_previous();
                true
            }
            KeyCode::Down => {
                service_table_state.select_next(total_rows);
                true
            }
            KeyCode::PageUp => {
                service_table_state.page_up();
                true
            }
            KeyCode::PageDown => {
                service_table_state.page_down(total_rows);
                true
            }
            KeyCode::Home => {
                service_table_state.select_first();
                true
            }
            KeyCode::End => {
                service_table_state.select_last(total_rows);
                true
            }

            // Sorting
            KeyCode::Char('t') => {
                service_table_state.toggle_sort(ServiceTableColumn::Timestamp);
                true
            }
            KeyCode::Char('i') => {
                service_table_state.toggle_sort(ServiceTableColumn::Ip);
                true
            }
            KeyCode::Char('p') => {
                service_table_state.toggle_sort(ServiceTableColumn::Port);
                true
            }
            KeyCode::Char('n') => {
                service_table_state.toggle_sort(ServiceTableColumn::ServiceName);
                true
            }
            KeyCode::Char('v') => {
                service_table_state.toggle_sort(ServiceTableColumn::Version);
                true
            }
            KeyCode::Char('c') => {
                service_table_state.toggle_sort(ServiceTableColumn::Confidence);
                true
            }

            // Filtering
            KeyCode::Char('f') => {
                service_table_state.cycle_confidence_filter();
                true
            }

            // Auto-scroll toggle
            KeyCode::Char('a') => {
                service_table_state.toggle_auto_scroll();
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
        ConfidenceFilter, ScanState, ServiceDetection, ServiceTableColumn, ServiceTableState,
        SortOrder,
    };
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::SystemTime;

    fn create_test_detection(port: u16, service_name: &str, confidence: f32) -> ServiceDetection {
        ServiceDetection {
            timestamp: SystemTime::now(),
            ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            port,
            service_name: service_name.to_string(),
            service_version: Some(format!("v{}.0", port / 100)),
            confidence,
        }
    }

    #[test]
    fn test_service_table_widget_creation() {
        let scan_state = ScanState::shared();
        let _widget = ServiceTableWidget::new(scan_state);
        // Should not panic
    }

    #[test]
    fn test_column_widths() {
        let widths = ServiceTableWidget::column_widths();
        assert_eq!(widths.len(), 6);
        assert_eq!(widths[0], 8); // Timestamp
        assert_eq!(widths[1], 15); // IP
        assert_eq!(widths[2], 6); // Port
        assert_eq!(widths[3], 12); // Service
        assert_eq!(widths[4], 20); // Version
        assert_eq!(widths[5], 12); // Confidence
    }

    #[test]
    fn test_format_timestamp() {
        let timestamp = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(3661); // 01:01:01
        let formatted = ServiceTableWidget::format_timestamp(timestamp);
        assert_eq!(formatted, "01:01:01");
    }

    #[test]
    fn test_get_confidence_color() {
        // High confidence (≥90%) = Green
        assert_eq!(ServiceTableWidget::get_confidence_color(0.95), Color::Green);
        assert_eq!(ServiceTableWidget::get_confidence_color(0.9), Color::Green);

        // Medium confidence (50-89%) = Yellow
        assert_eq!(
            ServiceTableWidget::get_confidence_color(0.85),
            Color::Yellow
        );
        assert_eq!(
            ServiceTableWidget::get_confidence_color(0.75),
            Color::Yellow
        );
        assert_eq!(ServiceTableWidget::get_confidence_color(0.5), Color::Yellow);

        // Low confidence (<50%) = Red
        assert_eq!(ServiceTableWidget::get_confidence_color(0.45), Color::Red);
        assert_eq!(ServiceTableWidget::get_confidence_color(0.1), Color::Red);
    }

    #[test]
    fn test_apply_filters_confidence() {
        let detections = vec![
            create_test_detection(80, "http", 0.95),   // High
            create_test_detection(443, "https", 0.75), // Medium
            create_test_detection(22, "ssh", 0.45),    // Low
        ];

        // Test All filter
        let mut service_table_state = ServiceTableState::new();
        service_table_state.confidence_filter = ConfidenceFilter::All;
        let filtered = ServiceTableWidget::apply_filters(&detections, &service_table_state);
        assert_eq!(filtered.len(), 3);

        // Test Low filter (≥50%)
        service_table_state.confidence_filter = ConfidenceFilter::Low;
        let filtered = ServiceTableWidget::apply_filters(&detections, &service_table_state);
        assert_eq!(filtered.len(), 2); // 0.95, 0.75

        // Test Medium filter (≥75%)
        service_table_state.confidence_filter = ConfidenceFilter::Medium;
        let filtered = ServiceTableWidget::apply_filters(&detections, &service_table_state);
        assert_eq!(filtered.len(), 2); // 0.95, 0.75

        // Test High filter (≥90%)
        service_table_state.confidence_filter = ConfidenceFilter::High;
        let filtered = ServiceTableWidget::apply_filters(&detections, &service_table_state);
        assert_eq!(filtered.len(), 1); // 0.95 only
    }

    #[test]
    fn test_apply_filters_service_name() {
        let detections = vec![
            create_test_detection(80, "http", 0.95),
            create_test_detection(443, "https", 0.95),
            create_test_detection(22, "ssh", 0.95),
        ];

        let mut service_table_state = ServiceTableState::new();
        service_table_state.filter_service_name = Some("http".to_string());

        let filtered = ServiceTableWidget::apply_filters(&detections, &service_table_state);
        assert_eq!(filtered.len(), 2); // "http" and "https" (both contain "http")
    }

    #[test]
    fn test_apply_filters_port() {
        let detections = vec![
            create_test_detection(80, "http", 0.95),
            create_test_detection(443, "https", 0.95),
            create_test_detection(22, "ssh", 0.95),
        ];

        let mut service_table_state = ServiceTableState::new();
        service_table_state.filter_port = Some(80);

        let filtered = ServiceTableWidget::apply_filters(&detections, &service_table_state);
        assert_eq!(filtered.len(), 1); // Port 80 only
        assert_eq!(filtered[0].port, 80);
    }

    #[test]
    fn test_apply_sorting_by_port_ascending() {
        let detections = vec![
            create_test_detection(443, "https", 0.95),
            create_test_detection(80, "http", 0.95),
            create_test_detection(22, "ssh", 0.95),
        ];

        let mut service_table_state = ServiceTableState::new();
        service_table_state.sort_column = ServiceTableColumn::Port;
        service_table_state.sort_order = SortOrder::Ascending;

        let sorted = ServiceTableWidget::apply_sorting(detections, &service_table_state);
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].port, 22);
        assert_eq!(sorted[1].port, 80);
        assert_eq!(sorted[2].port, 443);
    }

    #[test]
    fn test_apply_sorting_by_port_descending() {
        let detections = vec![
            create_test_detection(80, "http", 0.95),
            create_test_detection(443, "https", 0.95),
            create_test_detection(22, "ssh", 0.95),
        ];

        let mut service_table_state = ServiceTableState::new();
        service_table_state.sort_column = ServiceTableColumn::Port;
        service_table_state.sort_order = SortOrder::Descending;

        let sorted = ServiceTableWidget::apply_sorting(detections, &service_table_state);
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].port, 443);
        assert_eq!(sorted[1].port, 80);
        assert_eq!(sorted[2].port, 22);
    }

    #[test]
    fn test_apply_sorting_by_confidence() {
        let detections = vec![
            create_test_detection(80, "http", 0.45),
            create_test_detection(443, "https", 0.95),
            create_test_detection(22, "ssh", 0.75),
        ];

        let mut service_table_state = ServiceTableState::new();
        service_table_state.sort_column = ServiceTableColumn::Confidence;
        service_table_state.sort_order = SortOrder::Ascending;

        let sorted = ServiceTableWidget::apply_sorting(detections, &service_table_state);
        assert_eq!(sorted.len(), 3);
        // Order: 0.45 < 0.75 < 0.95
        assert!((sorted[0].confidence - 0.45).abs() < 0.01);
        assert!((sorted[1].confidence - 0.75).abs() < 0.01);
        assert!((sorted[2].confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_apply_sorting_by_service_name() {
        let detections = vec![
            create_test_detection(443, "https", 0.95),
            create_test_detection(80, "http", 0.95),
            create_test_detection(22, "ssh", 0.95),
        ];

        let mut service_table_state = ServiceTableState::new();
        service_table_state.sort_column = ServiceTableColumn::ServiceName;
        service_table_state.sort_order = SortOrder::Ascending;

        let sorted = ServiceTableWidget::apply_sorting(detections, &service_table_state);
        assert_eq!(sorted.len(), 3);
        // Order: "http" < "https" < "ssh" (alphabetically)
        assert_eq!(sorted[0].service_name, "http");
        assert_eq!(sorted[1].service_name, "https");
        assert_eq!(sorted[2].service_name, "ssh");
    }

    #[test]
    fn test_handle_service_table_event_navigation() {
        let mut ui_state = UIState::new();
        let scan_state = ScanState::shared();

        // Add test detections
        {
            let mut scan_state_lock = scan_state.write();
            for i in 0..10 {
                scan_state_lock
                    .service_detections
                    .push_back(create_test_detection(80 + i, "http", 0.95));
            }
        }

        // Test Down
        let event = Event::Key(KeyEvent::from(KeyCode::Down));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(ui_state.service_table_state.selected_row, 1);

        // Test Up
        let event = Event::Key(KeyEvent::from(KeyCode::Up));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(ui_state.service_table_state.selected_row, 0);

        // Test End
        let event = Event::Key(KeyEvent::from(KeyCode::End));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(ui_state.service_table_state.selected_row, 9); // Last row (0-indexed)

        // Test Home
        let event = Event::Key(KeyEvent::from(KeyCode::Home));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(ui_state.service_table_state.selected_row, 0);
    }

    #[test]
    fn test_handle_service_table_event_sorting() {
        let mut ui_state = UIState::new();
        let scan_state = ScanState::shared();

        // Test sort by Port (default is Timestamp, new column defaults to Descending)
        let event = Event::Key(KeyEvent::from(KeyCode::Char('p')));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(
            ui_state.service_table_state.sort_column,
            ServiceTableColumn::Port
        );
        assert_eq!(
            ui_state.service_table_state.sort_order,
            SortOrder::Descending
        );

        // Toggle to ascending (same column)
        let event = Event::Key(KeyEvent::from(KeyCode::Char('p')));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(
            ui_state.service_table_state.sort_order,
            SortOrder::Ascending
        );

        // Test sort by Confidence
        let event = Event::Key(KeyEvent::from(KeyCode::Char('c')));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(
            ui_state.service_table_state.sort_column,
            ServiceTableColumn::Confidence
        );
    }

    #[test]
    fn test_handle_service_table_event_filtering() {
        let mut ui_state = UIState::new();
        let scan_state = ScanState::shared();

        // Verify default filter is All
        assert_eq!(
            ui_state.service_table_state.confidence_filter,
            ConfidenceFilter::All
        );

        // Cycle to Low
        let event = Event::Key(KeyEvent::from(KeyCode::Char('f')));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(
            ui_state.service_table_state.confidence_filter,
            ConfidenceFilter::Low
        );

        // Cycle to Medium
        let event = Event::Key(KeyEvent::from(KeyCode::Char('f')));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(
            ui_state.service_table_state.confidence_filter,
            ConfidenceFilter::Medium
        );

        // Cycle to High
        let event = Event::Key(KeyEvent::from(KeyCode::Char('f')));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(
            ui_state.service_table_state.confidence_filter,
            ConfidenceFilter::High
        );

        // Cycle back to All
        let event = Event::Key(KeyEvent::from(KeyCode::Char('f')));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert_eq!(
            ui_state.service_table_state.confidence_filter,
            ConfidenceFilter::All
        );
    }

    #[test]
    fn test_handle_service_table_event_auto_scroll() {
        let mut ui_state = UIState::new();
        let scan_state = ScanState::shared();

        // Verify default is true
        assert!(ui_state.service_table_state.auto_scroll);

        // Toggle to false
        let event = Event::Key(KeyEvent::from(KeyCode::Char('a')));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert!(!ui_state.service_table_state.auto_scroll);

        // Toggle back to true
        let event = Event::Key(KeyEvent::from(KeyCode::Char('a')));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(handled);
        assert!(ui_state.service_table_state.auto_scroll);
    }

    #[test]
    fn test_handle_service_table_event_unhandled() {
        let mut ui_state = UIState::new();
        let scan_state = ScanState::shared();

        let event = Event::Key(KeyEvent::from(KeyCode::Char('x')));
        let handled = handle_service_table_event(event, &mut ui_state, &scan_state);
        assert!(!handled);
    }
}
