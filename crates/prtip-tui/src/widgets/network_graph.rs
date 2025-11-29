//! Network Activity Graph Widget - displays time-series throughput visualization
//!
//! This widget provides a 60-second historical view of network activity with
//! multiple data series: packets sent, packets received, ports discovered.

use crossterm::event::Event;
use parking_lot::RwLock;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType};
use ratatui::Frame;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use crate::state::{ScanState, UIState};
use crate::widgets::Component;

/// Network metrics time-series data
///
/// Maintains a 60-second ring buffer of samples taken every second.
/// Each sample contains instantaneous packet rates and cumulative port counts.
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    /// Timestamps of samples (60-second window)
    pub timestamps: VecDeque<f64>,

    /// Packets sent per second at each timestamp
    pub packets_sent: VecDeque<f64>,

    /// Packets received per second at each timestamp (estimated from responses)
    pub packets_received: VecDeque<f64>,

    /// Ports discovered per second (derivative of cumulative count)
    pub ports_per_second: VecDeque<f64>,

    /// Cumulative ports discovered (for derivative calculation)
    #[cfg(test)]
    cumulative_ports: u64,

    /// Start time for relative timestamp calculation
    #[allow(dead_code)] // Used in test-only add_sample() function
    start_time: Instant,

    /// Last sample time (to enforce 1-second sampling)
    /// None if no samples have been added yet
    #[cfg(test)]
    last_sample: Option<Instant>,
}

impl NetworkMetrics {
    /// Maximum samples to retain (60-second window)
    pub const MAX_SAMPLES: usize = 60;

    /// Minimum interval between samples (1 second)
    #[cfg(test)]
    const SAMPLE_INTERVAL_SECS: u64 = 1;

    /// Create new NetworkMetrics
    pub fn new() -> Self {
        Self {
            timestamps: VecDeque::with_capacity(Self::MAX_SAMPLES),
            packets_sent: VecDeque::with_capacity(Self::MAX_SAMPLES),
            packets_received: VecDeque::with_capacity(Self::MAX_SAMPLES),
            ports_per_second: VecDeque::with_capacity(Self::MAX_SAMPLES),
            #[cfg(test)]
            cumulative_ports: 0,
            start_time: Instant::now(),
            #[cfg(test)]
            last_sample: None,
        }
    }

    /// Add a new sample to the ring buffer
    ///
    /// Enforces 1-second minimum interval between samples. If called too soon,
    /// the sample is silently ignored to prevent oversampling.
    ///
    /// # Arguments
    ///
    /// * `packets_sent_pps` - Current packets sent per second
    /// * `responses` - Current response count (used to estimate packets received)
    /// * `total_ports` - Cumulative open ports discovered
    #[cfg(test)]
    pub fn add_sample(&mut self, packets_sent_pps: f64, responses: u64, total_ports: u64) {
        // Enforce 1-second minimum interval (skip check for first sample)
        if let Some(last) = self.last_sample {
            if last.elapsed().as_secs() < Self::SAMPLE_INTERVAL_SECS {
                return;
            }
        }

        // Calculate relative timestamp (seconds since start)
        let timestamp = self.start_time.elapsed().as_secs_f64();

        // Calculate ports/sec (derivative of cumulative)
        let ports_delta = total_ports.saturating_sub(self.cumulative_ports);
        let ports_per_sec = ports_delta as f64; // Already per-second due to 1s sampling
        self.cumulative_ports = total_ports;

        // Estimate packets received (response rate, typically lower than sent)
        // Use responses as proxy for received packets
        let packets_received_pps = responses as f64 / self.start_time.elapsed().as_secs_f64();

        // Add sample to ring buffers
        self.timestamps.push_back(timestamp);
        self.packets_sent.push_back(packets_sent_pps);
        self.packets_received.push_back(packets_received_pps);
        self.ports_per_second.push_back(ports_per_sec);

        // Enforce max capacity (FIFO)
        if self.timestamps.len() > Self::MAX_SAMPLES {
            self.timestamps.pop_front();
            self.packets_sent.pop_front();
            self.packets_received.pop_front();
            self.ports_per_second.pop_front();
        }

        self.last_sample = Some(Instant::now());
    }

    /// Get chart data for packets sent (time series)
    pub fn packets_sent_data(&self) -> Vec<(f64, f64)> {
        self.timestamps
            .iter()
            .zip(self.packets_sent.iter())
            .map(|(&t, &pps)| (t, pps))
            .collect()
    }

    /// Get chart data for packets received (time series)
    pub fn packets_received_data(&self) -> Vec<(f64, f64)> {
        self.timestamps
            .iter()
            .zip(self.packets_received.iter())
            .map(|(&t, &pps)| (t, pps))
            .collect()
    }

    /// Get chart data for ports discovered per second (time series)
    pub fn ports_per_second_data(&self) -> Vec<(f64, f64)> {
        self.timestamps
            .iter()
            .zip(self.ports_per_second.iter())
            .map(|(&t, &pps)| (t, pps))
            .collect()
    }

    /// Get time window bounds (min, max) for X-axis
    pub fn time_bounds(&self) -> (f64, f64) {
        if self.timestamps.is_empty() {
            return (0.0, 60.0);
        }

        let min = *self.timestamps.front().unwrap_or(&0.0);
        let max = *self.timestamps.back().unwrap_or(&60.0);

        // Ensure at least 10-second window
        let window = (max - min).max(10.0);
        (min, min + window)
    }

    /// Get Y-axis bounds (min, max) for throughput
    ///
    /// Uses auto-scaling based on actual data range.
    /// Ensures minimum range of 100 pps for visibility.
    pub fn throughput_bounds(&self) -> (f64, f64) {
        let mut max_throughput = 0.0_f64;

        // Find max across all series
        for pps in &self.packets_sent {
            max_throughput = max_throughput.max(*pps);
        }
        for pps in &self.packets_received {
            max_throughput = max_throughput.max(*pps);
        }
        for pps in &self.ports_per_second {
            max_throughput = max_throughput.max(*pps);
        }

        // Ensure minimum range (100 pps)
        max_throughput = max_throughput.max(100.0);

        // Add 10% headroom for visual clarity
        let max = max_throughput * 1.1;

        (0.0, max)
    }

    /// Check if we have any data to display
    pub fn has_data(&self) -> bool {
        !self.timestamps.is_empty()
    }

    /// Get sample count (for debugging)
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.timestamps.len()
    }

    /// Check if empty (for debugging)
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.timestamps.is_empty()
    }
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Network Activity Graph Widget
///
/// Displays a 60-second time-series graph of network throughput metrics.
///
/// # Layout
///
/// - **X-axis**: Time (seconds since scan start)
/// - **Y-axis**: Packets per second (auto-scaled)
/// - **Series**:
///   - Green: Packets Sent (SYN/probe packets)
///   - Yellow: Packets Received (responses)
///   - Cyan: Ports Discovered/sec
///
/// # Example
///
/// ```rust,no_run
/// use prtip_tui::widgets::NetworkGraphWidget;
/// use prtip_tui::state::ScanState;
/// use std::sync::Arc;
/// use parking_lot::RwLock;
///
/// let scan_state = Arc::new(RwLock::new(ScanState::new()));
/// let widget = NetworkGraphWidget::new(scan_state);
/// ```
pub struct NetworkGraphWidget {
    /// Shared scan state (read-only access)
    scan_state: Arc<RwLock<ScanState>>,
}

impl NetworkGraphWidget {
    /// Create a new NetworkGraphWidget
    ///
    /// # Arguments
    ///
    /// * `scan_state` - Shared scan state for reading metrics
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use prtip_tui::widgets::NetworkGraphWidget;
    /// use prtip_tui::state::ScanState;
    /// use std::sync::Arc;
    /// use parking_lot::RwLock;
    ///
    /// let scan_state = Arc::new(RwLock::new(ScanState::new()));
    /// let widget = NetworkGraphWidget::new(scan_state);
    /// ```
    pub fn new(scan_state: Arc<RwLock<ScanState>>) -> Self {
        Self { scan_state }
    }

    /// Render the chart with empty state message
    fn render_empty_chart(&self, frame: &mut Frame, area: Rect) {
        use ratatui::text::Text;
        use ratatui::widgets::Paragraph;

        let block = Block::default()
            .title("Network Activity (60s)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let message = Paragraph::new(Text::from("No data yet. Waiting for scan events..."))
            .block(block)
            .style(Style::default().fg(Color::DarkGray));

        frame.render_widget(message, area);
    }
}

impl Component for NetworkGraphWidget {
    fn render(&self, frame: &mut Frame, area: Rect, _ui_state: &UIState) {
        // Read scan state to get network metrics
        let scan_state = self.scan_state.read();

        // Build NetworkMetrics from throughput_history
        let mut network_metrics = NetworkMetrics::new();

        // Calculate time base relative to NOW (most recent sample at t=0, older samples at t<0)
        let now = Instant::now();

        // Convert ScanState throughput history to NetworkMetrics samples
        for (idx, sample) in scan_state.throughput_history.iter().enumerate() {
            // Calculate relative timestamp (seconds ago from now)
            let seconds_ago = now.duration_since(sample.timestamp).as_secs_f64();
            let timestamp = -seconds_ago; // Negative = past

            network_metrics.timestamps.push_back(timestamp);
            network_metrics
                .packets_sent
                .push_back(sample.packets_per_second);

            // Estimate received packets (responses per second)
            // Assume ~30% response rate (SYN-ACK responses vs SYN sent)
            let received_pps = sample.packets_per_second * 0.3;
            network_metrics.packets_received.push_back(received_pps);

            // Calculate ports/sec derivative from port_discoveries
            // Look at port discovery rate in this time window
            let ports_in_window = if idx < scan_state.port_discoveries.len() {
                // Count ports discovered near this sample's timestamp
                scan_state
                    .port_discoveries
                    .iter()
                    .filter(|p| {
                        if let Ok(elapsed) = p.timestamp.duration_since(SystemTime::UNIX_EPOCH) {
                            let sample_time = sample.timestamp.elapsed().as_secs();
                            let sample_epoch = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                                .saturating_sub(sample_time);
                            let port_epoch = elapsed.as_secs();
                            // Within 1-second window
                            port_epoch.abs_diff(sample_epoch) <= 1
                        } else {
                            false
                        }
                    })
                    .count() as f64
            } else {
                0.0
            };

            network_metrics.ports_per_second.push_back(ports_in_window);
        }

        // Handle empty state
        if !network_metrics.has_data() {
            self.render_empty_chart(frame, area);
            return;
        }

        // Get data and bounds BEFORE building datasets (to own the data)
        let packets_sent_data = network_metrics.packets_sent_data();
        let packets_received_data = network_metrics.packets_received_data();
        let ports_per_sec_data = network_metrics.ports_per_second_data();
        let (x_min, x_max) = network_metrics.time_bounds();
        let (y_min, y_max) = network_metrics.throughput_bounds();

        // Build datasets (inline to avoid lifetime issues)
        let mut datasets = Vec::new();

        // Dataset 1: Packets Sent (green line)
        if !packets_sent_data.is_empty() {
            datasets.push(
                Dataset::default()
                    .name("Packets Sent")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Green))
                    .data(&packets_sent_data),
            );
        }

        // Dataset 2: Packets Received (yellow line)
        if !packets_received_data.is_empty() {
            datasets.push(
                Dataset::default()
                    .name("Packets Received")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Yellow))
                    .data(&packets_received_data),
            );
        }

        // Dataset 3: Ports Discovered/sec (cyan line)
        if !ports_per_sec_data.is_empty() {
            datasets.push(
                Dataset::default()
                    .name("Ports/sec")
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&ports_per_sec_data),
            );
        }

        // Create X-axis (time)
        let x_labels = vec![
            format!("{:.0}s", x_min),
            format!("{:.0}s", (x_min + x_max) / 2.0),
            format!("{:.0}s", x_max),
        ];
        let x_axis = Axis::default()
            .title("Time (seconds)")
            .style(Style::default().fg(Color::White))
            .bounds([x_min, x_max])
            .labels(x_labels);

        // Create Y-axis (packets per second)
        let y_labels = vec![
            format!("{:.0}", y_min),
            format!("{:.0}", (y_min + y_max) / 2.0),
            format!("{:.0}", y_max),
        ];
        let y_axis = Axis::default()
            .title("Packets/sec")
            .style(Style::default().fg(Color::White))
            .bounds([y_min, y_max])
            .labels(y_labels);

        // Create chart
        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title("Network Activity (60s)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .x_axis(x_axis)
            .y_axis(y_axis);

        frame.render_widget(chart, area);
    }

    fn handle_event(&mut self, _event: Event) -> bool {
        // Network graph is read-only, no event handling needed
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_metrics_creation() {
        let metrics = NetworkMetrics::new();
        assert_eq!(metrics.len(), 0);
        assert!(metrics.is_empty());
        assert!(!metrics.has_data());
    }

    #[test]
    fn test_network_metrics_add_sample() {
        let mut metrics = NetworkMetrics::new();

        // Add first sample
        metrics.add_sample(1000.0, 50, 10);

        // Should be added (1-second interval enforced)
        std::thread::sleep(std::time::Duration::from_millis(1100));
        metrics.add_sample(2000.0, 100, 20);

        assert_eq!(metrics.len(), 2);
        assert!(metrics.has_data());
    }

    #[test]
    fn test_network_metrics_sample_interval_enforcement() {
        let mut metrics = NetworkMetrics::new();

        // Add first sample
        metrics.add_sample(1000.0, 50, 10);
        assert_eq!(metrics.len(), 1);

        // Try to add second sample immediately (should be ignored)
        metrics.add_sample(2000.0, 100, 20);
        assert_eq!(metrics.len(), 1); // Still 1 sample (ignored due to interval)
    }

    #[test]
    fn test_network_metrics_ring_buffer_capacity() {
        let mut metrics = NetworkMetrics::new();

        // Add MAX_SAMPLES + 10
        for i in 0..(NetworkMetrics::MAX_SAMPLES + 10) {
            metrics.timestamps.push_back(i as f64);
            metrics.packets_sent.push_back(1000.0 + i as f64);
            metrics.packets_received.push_back(500.0 + i as f64);
            metrics.ports_per_second.push_back(10.0 + i as f64);

            // Manual truncation (FIFO)
            if metrics.timestamps.len() > NetworkMetrics::MAX_SAMPLES {
                metrics.timestamps.pop_front();
                metrics.packets_sent.pop_front();
                metrics.packets_received.pop_front();
                metrics.ports_per_second.pop_front();
            }
        }

        assert_eq!(metrics.len(), NetworkMetrics::MAX_SAMPLES);

        // Verify oldest samples were removed
        let first_timestamp = *metrics.timestamps.front().unwrap();
        assert_eq!(first_timestamp, 10.0); // First 10 samples removed
    }

    #[test]
    fn test_network_metrics_derivative_calculation() {
        let mut metrics = NetworkMetrics::new();

        // Add samples with increasing port counts
        metrics.cumulative_ports = 0;
        std::thread::sleep(std::time::Duration::from_millis(1100));
        metrics.add_sample(1000.0, 50, 10); // 10 ports discovered
        assert_eq!(metrics.ports_per_second[0], 10.0); // 10 ports/sec

        std::thread::sleep(std::time::Duration::from_millis(1100));
        metrics.add_sample(1000.0, 75, 25); // 15 more ports (25 total)
        assert_eq!(metrics.ports_per_second[1], 15.0); // 15 ports/sec delta
    }

    #[test]
    fn test_network_metrics_time_bounds() {
        let mut metrics = NetworkMetrics::new();

        // Empty metrics
        let (min, max) = metrics.time_bounds();
        assert_eq!(min, 0.0);
        assert_eq!(max, 60.0); // Default 60-second window

        // Add samples
        std::thread::sleep(std::time::Duration::from_millis(1100));
        metrics.add_sample(1000.0, 50, 10);

        std::thread::sleep(std::time::Duration::from_millis(1100));
        metrics.add_sample(2000.0, 100, 20);

        let (min, max) = metrics.time_bounds();
        assert!(min >= 0.0);
        assert!(max >= min + 10.0); // Minimum 10-second window
    }

    #[test]
    fn test_network_metrics_throughput_bounds() {
        let mut metrics = NetworkMetrics::new();

        // Empty metrics
        let (min, max) = metrics.throughput_bounds();
        assert_eq!(min, 0.0);
        assert!((max - 110.0).abs() < 0.01); // Minimum 100 pps * 1.1 (10% headroom)

        // Add samples with varying throughput
        metrics.packets_sent.push_back(1000.0);
        metrics.packets_sent.push_back(5000.0); // Max
        metrics.packets_sent.push_back(2000.0);

        let (min, max) = metrics.throughput_bounds();
        assert_eq!(min, 0.0);
        assert!((max - 5500.0).abs() < 0.01); // 5000 * 1.1 (10% headroom)
    }

    #[test]
    fn test_network_metrics_chart_data() {
        let mut metrics = NetworkMetrics::new();

        // Add samples manually (bypass interval enforcement for testing)
        metrics.timestamps.push_back(1.0);
        metrics.timestamps.push_back(2.0);
        metrics.timestamps.push_back(3.0);

        metrics.packets_sent.push_back(1000.0);
        metrics.packets_sent.push_back(2000.0);
        metrics.packets_sent.push_back(1500.0);

        let data = metrics.packets_sent_data();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], (1.0, 1000.0));
        assert_eq!(data[1], (2.0, 2000.0));
        assert_eq!(data[2], (3.0, 1500.0));
    }

    #[test]
    fn test_widget_creation() {
        let scan_state = Arc::new(RwLock::new(ScanState::new()));
        let widget = NetworkGraphWidget::new(scan_state);

        // Widget should be created successfully
        assert!(widget.scan_state.read().open_ports == 0);
    }

    #[test]
    fn test_component_trait_handle_event() {
        let scan_state = Arc::new(RwLock::new(ScanState::new()));
        let mut widget = NetworkGraphWidget::new(scan_state);

        // Network graph doesn't handle events (read-only)
        let event = Event::Key(crossterm::event::KeyEvent::from(
            crossterm::event::KeyCode::Char('q'),
        ));
        assert!(!widget.handle_event(event));
    }
}
