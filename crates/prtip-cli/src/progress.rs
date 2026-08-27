//! Progress tracking and display for ProRT-IP scans
//!
//! This module provides real-time progress indicators with ETA calculation,
//! throughput metrics, and multi-stage progress tracking.
//!
//! # Features
//!
//! - **Multi-stage tracking**: Separate progress for resolution, discovery, scanning, detection, finalization
//! - **Multiple ETA algorithms**: Linear, EWMA (Exponential Weighted Moving Average), multi-stage
//! - **Real-time metrics**: Packets/sec, hosts/min, bandwidth utilization
//! - **Multiple display formats**: Compact, detailed, multi-stage bars
//! - **Colorized output**: Green/yellow/red based on speed thresholds
//! - **TTY detection**: Auto-disable progress bars for non-interactive output
//!
//! # Examples
//!
//! ```no_run
//! use prtip_cli::progress::{ProgressTracker, ProgressStyle, ScanStage};
//! use std::time::Duration;
//!
//! let mut tracker = ProgressTracker::new(ProgressStyle::Detailed);
//! tracker.set_display_interval(Duration::from_secs(1));
//!
//! // Start a stage
//! tracker.start_stage(ScanStage::Scanning, 10000);
//!
//! // Update progress
//! for i in 0..10000 {
//!     tracker.update(ScanStage::Scanning, i, 10000);
//!     // ... do work ...
//! }
//!
//! // Complete stage
//! tracker.complete_stage(ScanStage::Scanning);
//! ```
//!
//! # Event-Driven Progress (New)
//!
//! ```no_run
//! use prtip_cli::progress::{ProgressDisplay, ProgressStyle};
//! use prtip_core::event_bus::EventBus;
//! use std::sync::Arc;
//!
//! # async fn example() {
//! let event_bus = Arc::new(EventBus::new(1000));
//! let display = ProgressDisplay::new(event_bus.clone(), ProgressStyle::Detailed, false);
//!
//! // Progress automatically updates from events
//! display.start().await;
//! # }
//! ```

// Allow dead code for now - ProgressTracker will be used in Phase 6 TUI
#![allow(dead_code)]

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle as IndicatifStyle};
use parking_lot::Mutex;
use prtip_core::event_bus::{EventBus, EventFilter};
use prtip_core::events::{ScanEvent, ScanEventType, Throughput};
use prtip_core::progress::ProgressAggregator;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Scan stages for multi-stage progress tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScanStage {
    /// Target resolution (DNS lookups, CIDR expansion)
    Resolution,
    /// Host discovery (ICMP/ARP probes)
    Discovery,
    /// Port scanning (SYN/Connect/UDP)
    Scanning,
    /// Service/OS detection (banner grabbing)
    Detection,
    /// Finalization (writing output, cleanup)
    Finalization,
}

impl ScanStage {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Resolution => "Resolution",
            Self::Discovery => "Discovery",
            Self::Scanning => "Scanning",
            Self::Detection => "Detection",
            Self::Finalization => "Finalization",
        }
    }

    /// Get all stages in order
    pub fn all() -> &'static [ScanStage] {
        &[
            Self::Resolution,
            Self::Discovery,
            Self::Scanning,
            Self::Detection,
            Self::Finalization,
        ]
    }

    /// Get stage index (0-4)
    pub fn index(&self) -> usize {
        match self {
            Self::Resolution => 0,
            Self::Discovery => 1,
            Self::Scanning => 2,
            Self::Detection => 3,
            Self::Finalization => 4,
        }
    }
}

impl fmt::Display for ScanStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Progress display style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStyle {
    /// Compact single-line progress
    /// Example: `[===>    ] 45% | 1,234 pps | ETA: 2m 15s`
    Compact,

    /// Detailed multi-line progress with metrics
    /// Example:
    /// ```text
    /// Stage: Scanning [===>    ] 45% (4,500/10,000 ports)
    /// Speed: 1,234 pps | 245 hosts/min | 512 KB/s
    /// ETA: 2m 15s (linear), 2m 8s (adaptive)
    /// Elapsed: 1m 48s | Remaining: ~2m 10s
    /// ```
    Detailed,

    /// Multi-stage bars showing all stages
    /// Example:
    /// ```text
    /// Resolution  [========] 100% Complete
    /// Discovery   [========] 100% Complete
    /// Scanning    [===>    ] 45% (1,234 pps, ETA: 2m 15s)
    /// Detection   [        ] 0% Pending
    /// Finalization[        ] 0% Pending
    /// ```
    Bars,
}

impl Default for ProgressStyle {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        Self::Compact
    }
}

/// Progress metrics for display
#[derive(Debug, Clone, Default)]
pub struct ProgressMetrics {
    /// Packets per second
    pub packets_per_second: f64,
    /// Hosts per minute
    pub hosts_per_minute: f64,
    /// Bandwidth in bytes per second
    pub bandwidth_bps: f64,
    /// Linear ETA (based on current percentage)
    pub eta_linear: Duration,
    /// EWMA ETA (adaptive, smoothed)
    pub eta_ewma: Duration,
    /// Percentage complete (0-100)
    pub percent_complete: f64,
}

/// Main progress tracker
pub struct ProgressTracker {
    /// Current active stage
    current_stage: ScanStage,
    /// Total work units per stage
    total_work: HashMap<ScanStage, u64>,
    /// Completed work units per stage
    completed_work: HashMap<ScanStage, u64>,
    /// Stage start times
    start_times: HashMap<ScanStage, Instant>,
    /// Stage completion times
    end_times: HashMap<ScanStage, Instant>,
    /// EWMA speed (work units per second)
    ewma_speed: f64,
    /// EWMA alpha parameter (0-1, higher = more responsive)
    ewma_alpha: f64,
    /// Last update time
    last_update: Instant,
    /// Last metrics calculation time
    last_metrics_time: Instant,
    /// Display update interval
    display_interval: Duration,
    /// Display style
    style: ProgressStyle,
    /// Progress bar (if using indicatif)
    progress_bar: Option<ProgressBar>,
    /// Whether output is to a TTY
    is_tty: bool,
    /// Total packets sent (for pps calculation)
    total_packets: u64,
    /// Last packet count
    last_packet_count: u64,
    /// Total bandwidth in bytes
    total_bytes: u64,
    /// Last byte count
    last_byte_count: u64,
    /// Scan start time (global)
    scan_start: Instant,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(style: ProgressStyle) -> Self {
        use std::io::IsTerminal;
        let is_tty = std::io::stdout().is_terminal();
        Self {
            current_stage: ScanStage::Resolution,
            total_work: HashMap::new(),
            completed_work: HashMap::new(),
            start_times: HashMap::new(),
            end_times: HashMap::new(),
            ewma_speed: 0.0,
            ewma_alpha: 0.2, // Good balance of responsiveness vs stability
            last_update: Instant::now(),
            last_metrics_time: Instant::now(),
            display_interval: Duration::from_secs(1),
            style,
            progress_bar: None,
            is_tty,
            total_packets: 0,
            last_packet_count: 0,
            total_bytes: 0,
            last_byte_count: 0,
            scan_start: Instant::now(),
        }
    }

    /// Set display update interval
    pub fn set_display_interval(&mut self, interval: Duration) {
        self.display_interval = interval;
    }

    /// Set EWMA alpha (0-1, higher = more responsive)
    pub fn set_ewma_alpha(&mut self, alpha: f64) {
        self.ewma_alpha = alpha.clamp(0.0, 1.0);
    }

    /// Check if output is to a TTY
    pub fn is_tty(&self) -> bool {
        self.is_tty
    }

    /// Start a new stage
    pub fn start_stage(&mut self, stage: ScanStage, total_work: u64) {
        self.current_stage = stage;
        self.total_work.insert(stage, total_work);
        self.completed_work.insert(stage, 0);
        self.start_times.insert(stage, Instant::now());

        if self.is_tty && self.style == ProgressStyle::Bars {
            self.initialize_progress_bar();
        }
    }

    /// Update progress for current stage
    pub fn update(&mut self, stage: ScanStage, completed: u64, total: u64) {
        // Update work counters
        self.completed_work.insert(stage, completed);
        self.total_work.insert(stage, total);

        // Update EWMA speed
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();

        if elapsed > 0.0 {
            let last_completed = self.completed_work.get(&stage).copied().unwrap_or(0);
            let work_delta = completed.saturating_sub(last_completed) as f64;
            let current_speed = work_delta / elapsed;

            // Update EWMA
            if self.ewma_speed == 0.0 {
                self.ewma_speed = current_speed;
            } else {
                self.ewma_speed =
                    self.ewma_alpha * current_speed + (1.0 - self.ewma_alpha) * self.ewma_speed;
            }
        }

        self.last_update = now;

        // Display update if enough time has passed
        if now.duration_since(self.last_metrics_time) >= self.display_interval {
            self.display();
            self.last_metrics_time = now;
        }
    }

    /// Complete a stage
    pub fn complete_stage(&mut self, stage: ScanStage) {
        self.end_times.insert(stage, Instant::now());
        if let Some(total) = self.total_work.get(&stage) {
            self.completed_work.insert(stage, *total);
        }
        self.display();
    }

    /// Increment packet count
    pub fn add_packets(&mut self, count: u64) {
        self.total_packets += count;
    }

    /// Increment bandwidth
    pub fn add_bytes(&mut self, bytes: u64) {
        self.total_bytes += bytes;
    }

    /// Calculate current metrics
    pub fn calculate_metrics(&self) -> ProgressMetrics {
        let completed = self
            .completed_work
            .get(&self.current_stage)
            .copied()
            .unwrap_or(0);
        let total = self
            .total_work
            .get(&self.current_stage)
            .copied()
            .unwrap_or(1);

        let percent_complete = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // Calculate packets per second
        let elapsed = Instant::now()
            .duration_since(self.last_metrics_time)
            .as_secs_f64();
        let packets_delta = self.total_packets.saturating_sub(self.last_packet_count) as f64;
        let packets_per_second = if elapsed > 0.0 {
            packets_delta / elapsed
        } else {
            0.0
        };

        // Calculate bandwidth
        let bytes_delta = self.total_bytes.saturating_sub(self.last_byte_count) as f64;
        let bandwidth_bps = if elapsed > 0.0 {
            bytes_delta / elapsed
        } else {
            0.0
        };

        // Calculate hosts per minute (assume 1 host per 1000 packets as heuristic)
        let hosts_per_minute = (packets_per_second / 1000.0) * 60.0;

        // Calculate ETAs
        let eta_linear = self.calculate_eta_linear(completed, total);
        let eta_ewma = self.calculate_eta_ewma(completed, total);

        ProgressMetrics {
            packets_per_second,
            hosts_per_minute,
            bandwidth_bps,
            eta_linear,
            eta_ewma,
            percent_complete,
        }
    }

    /// Calculate linear ETA (simple percentage-based)
    fn calculate_eta_linear(&self, completed: u64, total: u64) -> Duration {
        if completed == 0 || total == 0 || completed >= total {
            return Duration::from_secs(0);
        }

        let start = self
            .start_times
            .get(&self.current_stage)
            .unwrap_or(&self.scan_start);
        let elapsed = Instant::now().duration_since(*start);
        let percent = completed as f64 / total as f64;

        if percent > 0.0 {
            let total_time = elapsed.as_secs_f64() / percent;
            let remaining = total_time - elapsed.as_secs_f64();
            Duration::from_secs_f64(remaining.max(0.0))
        } else {
            Duration::from_secs(0)
        }
    }

    /// Calculate EWMA ETA (adaptive, smoothed)
    fn calculate_eta_ewma(&self, completed: u64, total: u64) -> Duration {
        if completed >= total || self.ewma_speed <= 0.0 {
            return Duration::from_secs(0);
        }

        let remaining_work = total.saturating_sub(completed) as f64;
        let eta_seconds = remaining_work / self.ewma_speed;
        Duration::from_secs_f64(eta_seconds.max(0.0))
    }

    /// Initialize indicatif progress bar
    fn initialize_progress_bar(&mut self) {
        if !self.is_tty {
            return;
        }

        let pb = ProgressBar::new(100);
        pb.set_style(
            IndicatifStyle::default_bar()
                .template("{msg}\n{bar:40.cyan/blue} {pos:>3}%")
                .expect("Valid template")
                .progress_chars("=>-"),
        );
        self.progress_bar = Some(pb);
    }

    /// Display progress based on current style
    fn display(&mut self) {
        if !self.is_tty && self.style != ProgressStyle::Compact {
            // For non-TTY, only show compact progress as simple logs
            self.display_compact_log();
            return;
        }

        match self.style {
            ProgressStyle::Compact => self.display_compact(),
            ProgressStyle::Detailed => self.display_detailed(),
            ProgressStyle::Bars => self.display_bars(),
        }
    }

    /// Display compact progress (single line)
    fn display_compact(&self) {
        let metrics = self.calculate_metrics();
        let bar = self.render_progress_bar(metrics.percent_complete as u8);
        let color = self.get_speed_color(metrics.packets_per_second);

        let eta_str = format_duration(metrics.eta_ewma);
        let pps_str = format_metric(metrics.packets_per_second, "pps");

        let line = format!(
            "{} {:.1}% | {} | ETA: {}",
            bar, metrics.percent_complete, pps_str, eta_str
        );

        print!("\r{}", line.color(color));
        let _ = io::stdout().flush();
    }

    /// Display compact progress as log line (for non-TTY)
    fn display_compact_log(&self) {
        let metrics = self.calculate_metrics();
        let pps_str = format_metric(metrics.packets_per_second, "pps");
        let eta_str = format_duration(metrics.eta_ewma);

        println!(
            "[{}] {:.1}% | {} | ETA: {}",
            self.current_stage.name(),
            metrics.percent_complete,
            pps_str,
            eta_str
        );
    }

    /// Display detailed progress (multi-line)
    fn display_detailed(&self) {
        let metrics = self.calculate_metrics();
        let completed = self
            .completed_work
            .get(&self.current_stage)
            .copied()
            .unwrap_or(0);
        let total = self
            .total_work
            .get(&self.current_stage)
            .copied()
            .unwrap_or(1);

        let bar = self.render_progress_bar(metrics.percent_complete as u8);
        let color = self.get_speed_color(metrics.packets_per_second);

        let pps_str = format_metric(metrics.packets_per_second, "pps");
        let hpm_str = format_metric(metrics.hosts_per_minute, "hosts/min");
        let bw_str = format_bandwidth(metrics.bandwidth_bps);

        let eta_linear_str = format_duration(metrics.eta_linear);
        let eta_ewma_str = format_duration(metrics.eta_ewma);

        let elapsed = Instant::now().duration_since(self.scan_start);
        let elapsed_str = format_duration(elapsed);

        // Clear previous lines (4 lines)
        if self.is_tty {
            print!("\x1b[4A\x1b[J");
        }

        println!(
            "{}",
            format!(
                "Stage: {} {} {:.1}% ({}/{})",
                self.current_stage.name(),
                bar,
                metrics.percent_complete,
                format_number(completed),
                format_number(total)
            )
            .color(color)
        );
        println!("Speed: {} | {} | {}", pps_str, hpm_str, bw_str);
        println!(
            "ETA: {} (linear), {} (adaptive)",
            eta_linear_str, eta_ewma_str
        );
        println!("Elapsed: {} | Remaining: ~{}", elapsed_str, eta_ewma_str);

        let _ = io::stdout().flush();
    }

    /// Display multi-stage bars
    fn display_bars(&self) {
        // Clear previous display (5 stages + 1 blank line)
        if self.is_tty {
            print!("\x1b[6A\x1b[J");
        }

        for stage in ScanStage::all() {
            let completed = self.completed_work.get(stage).copied().unwrap_or(0);
            let total = self.total_work.get(stage).copied().unwrap_or(0);

            let (percent, status) = if total == 0 {
                (0.0, "Pending".to_string())
            } else if completed >= total {
                (100.0, "Complete".to_string())
            } else {
                let pct = (completed as f64 / total as f64) * 100.0;
                let metrics = self.calculate_metrics();
                let pps_str = format_metric(metrics.packets_per_second, "pps");
                let eta_str = format_duration(metrics.eta_ewma);
                (pct, format!("{}, ETA: {}", pps_str, eta_str))
            };

            let bar = self.render_progress_bar(percent as u8);
            let color = if completed >= total {
                "green"
            } else if *stage == self.current_stage {
                self.get_speed_color(self.ewma_speed)
            } else {
                "white"
            };

            let line = format!("{:<12} {} {:.0}% {}", stage.name(), bar, percent, status);

            match color {
                "green" => println!("{}", line.green()),
                "yellow" => println!("{}", line.yellow()),
                "red" => println!("{}", line.red()),
                _ => println!("{}", line),
            }
        }

        println!(); // Blank line
        let _ = io::stdout().flush();
    }

    /// Render a progress bar
    fn render_progress_bar(&self, percent: u8) -> String {
        let width = 40;
        let filled = (percent as usize * width) / 100;
        let empty = width - filled;

        let mut bar = String::from("[");
        bar.push_str(&"=".repeat(filled.saturating_sub(1)));
        if filled > 0 {
            bar.push('>');
        }
        bar.push_str(&" ".repeat(empty));
        bar.push(']');
        bar
    }

    /// Get color based on speed
    fn get_speed_color(&self, speed: f64) -> &'static str {
        if speed >= 100_000.0 {
            "green" // Excellent: >100K pps
        } else if speed >= 10_000.0 {
            "yellow" // Good: 10K-100K pps
        } else {
            "red" // Slow: <10K pps
        }
    }

    /// Display final summary
    pub fn finish(&self) {
        if self.is_tty {
            // Clear progress display
            match self.style {
                ProgressStyle::Compact => print!("\r\x1b[K"),
                ProgressStyle::Detailed => print!("\x1b[4A\x1b[J"),
                ProgressStyle::Bars => print!("\x1b[6A\x1b[J"),
            }
        }

        let total_elapsed = Instant::now().duration_since(self.scan_start);
        println!("\nScan completed in {}", format_duration(total_elapsed));

        // Summary statistics
        let total_packets = self.total_packets;
        let total_bytes = self.total_bytes;
        let avg_pps = total_packets as f64 / total_elapsed.as_secs_f64();
        let avg_bw = total_bytes as f64 / total_elapsed.as_secs_f64();

        println!("Total packets: {}", format_number(total_packets));
        println!("Average speed: {}", format_metric(avg_pps, "pps"));
        println!("Average bandwidth: {}", format_bandwidth(avg_bw));
    }
}

impl Drop for ProgressTracker {
    fn drop(&mut self) {
        if let Some(pb) = &self.progress_bar {
            pb.finish_and_clear();
        }
    }
}

/// Format a duration as human-readable string
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Format a number with thousands separators
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();

    for (count, c) in s.chars().rev().enumerate() {
        if count > 0 && count % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }

    result
}

/// Format a metric with SI suffix
fn format_metric(value: f64, unit: &str) -> String {
    if value >= 1_000_000.0 {
        format!("{:.2}M {}", value / 1_000_000.0, unit)
    } else if value >= 1_000.0 {
        format!("{:.2}K {}", value / 1_000.0, unit)
    } else {
        format!("{:.0} {}", value, unit)
    }
}

/// Format bandwidth (bytes/sec) as human-readable
fn format_bandwidth(bps: f64) -> String {
    if bps >= 1_000_000.0 {
        format!("{:.2} MB/s", bps / 1_000_000.0)
    } else if bps >= 1_000.0 {
        format!("{:.2} KB/s", bps / 1_000.0)
    } else {
        format!("{:.0} B/s", bps)
    }
}

/// Event-driven progress display
///
/// Subscribes to EventBus and automatically updates progress based on
/// scan events. Uses ProgressAggregator for real-time ETA and throughput.
pub struct ProgressDisplay {
    /// Event bus for receiving events
    event_bus: Arc<EventBus>,
    /// Progress aggregator for state tracking
    aggregator: Arc<ProgressAggregator>,
    /// Progress bar for display
    progress_bar: Option<ProgressBar>,
    /// Display style
    style: ProgressStyle,
    /// Quiet mode (no display)
    quiet_mode: bool,
    /// Last display update time
    last_update: Arc<Mutex<Instant>>,
    /// Update interval
    update_interval: Duration,
    /// Whether output is to a TTY
    is_tty: bool,
}

impl ProgressDisplay {
    /// Create a new event-driven progress display
    ///
    /// Automatically subscribes to progress events and updates display.
    ///
    /// # Arguments
    ///
    /// * `event_bus` - Event bus to subscribe to
    /// * `style` - Display style (Compact, Detailed, Bars)
    /// * `quiet` - Quiet mode (no output)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_cli::progress::{ProgressDisplay, ProgressStyle};
    /// use prtip_core::event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let event_bus = Arc::new(EventBus::new(1000));
    /// let display = ProgressDisplay::new(
    ///     event_bus,
    ///     ProgressStyle::Detailed,
    ///     false
    /// );
    /// display.start().await;
    /// # }
    /// ```
    pub fn new(event_bus: Arc<EventBus>, style: ProgressStyle, quiet: bool) -> Self {
        use std::io::IsTerminal;
        let is_tty = std::io::stdout().is_terminal();
        let aggregator = Arc::new(ProgressAggregator::new(event_bus.clone()));

        // Initialize progress bar if not quiet
        let progress_bar = if !quiet && is_tty && style == ProgressStyle::Compact {
            let pb = ProgressBar::new(100);
            pb.set_style(
                IndicatifStyle::default_bar()
                    .template("{msg}\n{bar:40.cyan/blue} {pos:>3}%")
                    .expect("Valid template")
                    .progress_chars("=>-"),
            );
            Some(pb)
        } else {
            None
        };

        Self {
            event_bus,
            aggregator,
            progress_bar,
            style,
            quiet_mode: quiet,
            last_update: Arc::new(Mutex::new(Instant::now())),
            update_interval: Duration::from_millis(100), // 100ms debounce
            is_tty,
        }
    }

    /// Start listening to events and updating display
    ///
    /// Spawns a background task that listens to progress events
    /// and updates the display accordingly.
    pub async fn start(&self) -> tokio::task::JoinHandle<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Subscribe to relevant events
        self.event_bus
            .subscribe(
                tx,
                EventFilter::EventType(vec![
                    ScanEventType::ScanStarted,
                    ScanEventType::ProgressUpdate,
                    ScanEventType::PortFound,
                    ScanEventType::ServiceDetected,
                    ScanEventType::ScanCompleted,
                    ScanEventType::ScanError,
                ]),
            )
            .await;

        let quiet = self.quiet_mode;
        let style = self.style;
        let last_update = self.last_update.clone();
        let update_interval = self.update_interval;
        let aggregator = self.aggregator.clone();
        let progress_bar = self.progress_bar.clone();
        let is_tty = self.is_tty;

        // Spawn event listener task
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if quiet {
                    continue;
                }

                // Debounce updates
                {
                    let mut last = last_update.lock();
                    let now = Instant::now();
                    if now.duration_since(*last) < update_interval {
                        continue;
                    }
                    *last = now;
                } // Drop MutexGuard before awaits

                // Update display based on event
                match event {
                    ScanEvent::ScanStarted { .. } => {
                        Self::display_started(&aggregator, &style, is_tty).await;
                    }
                    ScanEvent::ProgressUpdate { .. }
                    | ScanEvent::PortFound { .. }
                    | ScanEvent::ServiceDetected { .. } => {
                        Self::display_progress(&aggregator, &style, &progress_bar, is_tty).await;
                    }
                    ScanEvent::ScanCompleted { .. } => {
                        Self::display_completed(&aggregator, &style, &progress_bar, is_tty).await;
                    }
                    ScanEvent::ScanError { error, .. } => {
                        Self::display_error(&error, is_tty);
                    }
                    _ => {}
                }
            }
        })
    }

    /// Display scan started message
    async fn display_started(
        aggregator: &Arc<ProgressAggregator>,
        style: &ProgressStyle,
        is_tty: bool,
    ) {
        let _state = aggregator.get_state().await;
        let total = 100; // Placeholder, will be updated by events

        if !is_tty {
            println!("[Scan Started] Targets: {}", total);
            return;
        }

        match style {
            ProgressStyle::Compact => {
                println!("Scan started: {} targets", total);
            }
            ProgressStyle::Detailed => {
                println!("╔════════════════════════════════════════╗");
                println!("║          Scan Started                  ║");
                println!("╠════════════════════════════════════════╣");
                println!("║ Targets:  {:>28} ║", format_number(total as u64));
                println!("╚════════════════════════════════════════╝");
            }
            ProgressStyle::Bars => {
                println!("Starting scan: {} targets...", total);
            }
        }
    }

    /// Display progress update
    async fn display_progress(
        aggregator: &Arc<ProgressAggregator>,
        style: &ProgressStyle,
        progress_bar: &Option<ProgressBar>,
        is_tty: bool,
    ) {
        let state = aggregator.get_state().await;

        match style {
            ProgressStyle::Compact => {
                if let Some(pb) = progress_bar {
                    let msg = format!(
                        "{:.1}% | {} | ETA: {}",
                        state.overall_progress,
                        Self::format_throughput(&state.throughput),
                        Self::format_eta(state.eta),
                    );
                    pb.set_position(state.overall_progress as u64);
                    pb.set_message(msg);
                } else if !is_tty {
                    println!(
                        "[Progress] {:.1}% | {} open ports",
                        state.overall_progress, state.open_ports
                    );
                }
            }
            ProgressStyle::Detailed => {
                if is_tty {
                    print!("\x1b[4A\x1b[J"); // Clear 4 lines
                }
                println!("Progress: {:.1}%", state.overall_progress);
                println!(
                    "Ports:    {} open / {} closed / {} filtered",
                    state.open_ports, state.closed_ports, state.filtered_ports
                );
                println!("Speed:    {}", Self::format_throughput(&state.throughput));
                if let Some(eta) = state.eta {
                    println!("ETA:      {}", format_duration(eta));
                } else {
                    println!("ETA:      Calculating...");
                }
                let _ = io::stdout().flush();
            }
            ProgressStyle::Bars => {
                if is_tty {
                    print!("\x1b[2A\x1b[J"); // Clear 2 lines
                }
                let bar = Self::render_bar(state.overall_progress as f64);
                println!(
                    "{} {:.1}% ({} ports found)",
                    bar, state.overall_progress, state.open_ports
                );
                println!(
                    "{} | ETA: {}",
                    Self::format_throughput(&state.throughput),
                    Self::format_eta(state.eta)
                );
                let _ = io::stdout().flush();
            }
        }
    }

    /// Display scan completed message
    async fn display_completed(
        aggregator: &Arc<ProgressAggregator>,
        _style: &ProgressStyle,
        progress_bar: &Option<ProgressBar>,
        is_tty: bool,
    ) {
        let state = aggregator.get_state().await;

        if let Some(pb) = progress_bar {
            pb.finish_and_clear();
        }

        if is_tty {
            print!("\x1b[4A\x1b[J"); // Clear progress display
        }

        println!("\n{}", "Scan Completed".green().bold());
        println!("Discovered hosts: {}", state.discovered_hosts);
        println!("Open ports:       {}", state.open_ports);
        println!("Closed ports:     {}", state.closed_ports);
        println!("Filtered ports:   {}", state.filtered_ports);
        println!("Services found:   {}", state.detected_services);
    }

    /// Display error message
    fn display_error(error: &str, is_tty: bool) {
        if is_tty {
            eprintln!("{} {}", "Error:".red().bold(), error);
        } else {
            eprintln!("[ERROR] {}", error);
        }
    }

    /// Format throughput for display
    fn format_throughput(throughput: &Throughput) -> String {
        format!(
            "{:.0} pps | {:.0} hpm",
            throughput.packets_per_second, throughput.hosts_per_minute
        )
    }

    /// Format ETA for display
    fn format_eta(eta: Option<Duration>) -> String {
        if let Some(duration) = eta {
            format_duration(duration)
        } else {
            "N/A".to_string()
        }
    }

    /// Render a progress bar
    fn render_bar(percent: f64) -> String {
        let width = 40;
        let filled = ((percent / 100.0) * width as f64) as usize;
        let empty = width - filled;

        let mut bar = String::from("[");
        bar.push_str(&"▓".repeat(filled.saturating_sub(1)));
        if filled > 0 {
            bar.push('▓');
        }
        bar.push_str(&"░".repeat(empty));
        bar.push(']');
        bar
    }

    /// Finish and clean up display
    pub fn finish(&self) {
        if let Some(pb) = &self.progress_bar {
            pb.finish_and_clear();
        }
    }
}

impl Drop for ProgressDisplay {
    fn drop(&mut self) {
        self.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_stage_ordering() {
        let stages = ScanStage::all();
        assert_eq!(stages.len(), 5);
        assert_eq!(stages[0], ScanStage::Resolution);
        assert_eq!(stages[4], ScanStage::Finalization);
    }

    #[test]
    fn test_scan_stage_names() {
        assert_eq!(ScanStage::Resolution.name(), "Resolution");
        assert_eq!(ScanStage::Scanning.name(), "Scanning");
    }

    #[test]
    fn test_scan_stage_index() {
        assert_eq!(ScanStage::Resolution.index(), 0);
        assert_eq!(ScanStage::Discovery.index(), 1);
        assert_eq!(ScanStage::Finalization.index(), 4);
    }

    #[test]
    fn test_progress_tracker_new() {
        let tracker = ProgressTracker::new(ProgressStyle::Compact);
        assert_eq!(tracker.style, ProgressStyle::Compact);
        assert_eq!(tracker.ewma_alpha, 0.2);
    }

    #[test]
    fn test_progress_tracker_start_stage() {
        let mut tracker = ProgressTracker::new(ProgressStyle::Compact);
        tracker.start_stage(ScanStage::Scanning, 10000);

        assert_eq!(tracker.current_stage, ScanStage::Scanning);
        assert_eq!(tracker.total_work.get(&ScanStage::Scanning), Some(&10000));
        assert_eq!(tracker.completed_work.get(&ScanStage::Scanning), Some(&0));
        assert!(tracker.start_times.contains_key(&ScanStage::Scanning));
    }

    #[test]
    fn test_progress_tracker_update() {
        let mut tracker = ProgressTracker::new(ProgressStyle::Compact);
        tracker.start_stage(ScanStage::Scanning, 10000);
        tracker.update(ScanStage::Scanning, 5000, 10000);

        assert_eq!(
            tracker.completed_work.get(&ScanStage::Scanning),
            Some(&5000)
        );
    }

    #[test]
    fn test_progress_tracker_complete_stage() {
        let mut tracker = ProgressTracker::new(ProgressStyle::Compact);
        tracker.start_stage(ScanStage::Scanning, 10000);
        tracker.complete_stage(ScanStage::Scanning);

        assert_eq!(
            tracker.completed_work.get(&ScanStage::Scanning),
            Some(&10000)
        );
        assert!(tracker.end_times.contains_key(&ScanStage::Scanning));
    }

    #[test]
    fn test_calculate_metrics_percent() {
        let mut tracker = ProgressTracker::new(ProgressStyle::Compact);
        tracker.start_stage(ScanStage::Scanning, 10000);
        tracker.update(ScanStage::Scanning, 4500, 10000);

        let metrics = tracker.calculate_metrics();
        assert!((metrics.percent_complete - 45.0).abs() < 0.1);
    }

    #[test]
    fn test_eta_linear_calculation() {
        let mut tracker = ProgressTracker::new(ProgressStyle::Compact);
        tracker.start_stage(ScanStage::Scanning, 10000);

        // Simulate 50% completion after 10 seconds
        std::thread::sleep(Duration::from_millis(100));
        let eta = tracker.calculate_eta_linear(5000, 10000);

        // ETA should be approximately equal to elapsed time (50% done)
        assert!(eta.as_secs() < 1); // Should be roughly 100ms more
    }

    #[test]
    fn test_ewma_speed_update() {
        let mut tracker = ProgressTracker::new(ProgressStyle::Compact);
        tracker.start_stage(ScanStage::Scanning, 10000);

        // Initial state
        assert_eq!(tracker.ewma_speed, 0.0);

        // Simulate time passing for first update
        std::thread::sleep(Duration::from_millis(50));
        tracker.update(ScanStage::Scanning, 1000, 10000);
        let first_speed = tracker.ewma_speed;
        assert!(first_speed >= 0.0);

        // Second update should smooth the speed
        std::thread::sleep(Duration::from_millis(50));
        tracker.update(ScanStage::Scanning, 2000, 10000);
        assert!(tracker.ewma_speed >= 0.0);

        // If enough time has passed, EWMA should have been updated
        if first_speed > 0.0 {
            assert!(tracker.ewma_speed > 0.0);
        }
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3665)), "1h 1m");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_metric() {
        assert_eq!(format_metric(500.0, "pps"), "500 pps");
        assert_eq!(format_metric(1500.0, "pps"), "1.50K pps");
        assert_eq!(format_metric(1500000.0, "pps"), "1.50M pps");
    }

    #[test]
    fn test_format_bandwidth() {
        assert_eq!(format_bandwidth(500.0), "500 B/s");
        assert_eq!(format_bandwidth(1500.0), "1.50 KB/s");
        assert_eq!(format_bandwidth(1500000.0), "1.50 MB/s");
    }

    #[test]
    fn test_render_progress_bar() {
        let tracker = ProgressTracker::new(ProgressStyle::Compact);

        let bar_0 = tracker.render_progress_bar(0);
        assert!(bar_0.starts_with('['));
        assert!(bar_0.ends_with(']'));
        assert!(bar_0.contains("        ")); // All empty

        let bar_50 = tracker.render_progress_bar(50);
        assert!(bar_50.contains("="));
        assert!(bar_50.contains('>'));

        let bar_100 = tracker.render_progress_bar(100);
        assert!(bar_100.contains("===="));
    }

    #[test]
    fn test_speed_color_selection() {
        let tracker = ProgressTracker::new(ProgressStyle::Compact);

        assert_eq!(tracker.get_speed_color(150_000.0), "green");
        assert_eq!(tracker.get_speed_color(50_000.0), "yellow");
        assert_eq!(tracker.get_speed_color(5_000.0), "red");
    }

    #[test]
    fn test_add_packets_and_bytes() {
        let mut tracker = ProgressTracker::new(ProgressStyle::Compact);

        tracker.add_packets(1000);
        assert_eq!(tracker.total_packets, 1000);

        tracker.add_bytes(64000);
        assert_eq!(tracker.total_bytes, 64000);

        tracker.add_packets(500);
        assert_eq!(tracker.total_packets, 1500);
    }

    #[test]
    fn test_set_ewma_alpha() {
        let mut tracker = ProgressTracker::new(ProgressStyle::Compact);

        tracker.set_ewma_alpha(0.5);
        assert_eq!(tracker.ewma_alpha, 0.5);

        // Test clamping
        tracker.set_ewma_alpha(1.5);
        assert_eq!(tracker.ewma_alpha, 1.0);

        tracker.set_ewma_alpha(-0.5);
        assert_eq!(tracker.ewma_alpha, 0.0);
    }

    #[test]
    fn test_set_display_interval() {
        let mut tracker = ProgressTracker::new(ProgressStyle::Compact);

        tracker.set_display_interval(Duration::from_secs(5));
        assert_eq!(tracker.display_interval, Duration::from_secs(5));
    }
}
