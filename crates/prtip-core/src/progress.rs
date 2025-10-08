//! Progress tracking and statistics for network scanning operations.
//!
//! This module provides thread-safe progress tracking during scanning operations,
//! including real-time statistics, rate calculation, and ETA estimation.
//!
//! # Examples
//!
//! ```
//! use prtip_core::progress::ScanProgress;
//!
//! let progress = ScanProgress::new(1000);
//!
//! // During scanning
//! progress.increment_completed();
//! progress.increment_open();
//!
//! // Get statistics
//! let rate = progress.rate_per_second();
//! let summary = progress.summary();
//! println!("{}", summary);
//! ```

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Categories of scan errors for statistics tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Connection refused (port closed)
    ConnectionRefused,
    /// Connection timeout (port filtered)
    Timeout,
    /// Network unreachable
    NetworkUnreachable,
    /// Host unreachable
    HostUnreachable,
    /// Permission denied
    PermissionDenied,
    /// Too many open files
    TooManyOpenFiles,
    /// Other errors
    Other,
}

/// Thread-safe progress tracker for scanning operations.
///
/// Uses atomic operations for lock-free concurrent updates from multiple threads.
/// Tracks completion counts, port states, errors, and timing information.
#[derive(Debug)]
pub struct ScanProgress {
    /// Total number of targets to scan
    total_targets: usize,
    /// Number of completed scans
    completed: AtomicUsize,
    /// Number of open ports found
    open_ports: AtomicUsize,
    /// Number of closed ports found
    closed_ports: AtomicUsize,
    /// Number of filtered ports found
    filtered_ports: AtomicUsize,
    /// Connection refused errors
    connection_refused: AtomicUsize,
    /// Timeout errors
    timeouts: AtomicUsize,
    /// Network unreachable errors
    network_unreachable: AtomicUsize,
    /// Host unreachable errors
    host_unreachable: AtomicUsize,
    /// Permission denied errors
    permission_denied: AtomicUsize,
    /// Too many files errors
    too_many_files: AtomicUsize,
    /// Other errors
    other_errors: AtomicUsize,
    /// Scan start time
    start_time: Instant,
}

impl ScanProgress {
    /// Creates a new progress tracker for the given number of targets.
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::progress::ScanProgress;
    ///
    /// let progress = ScanProgress::new(1000);
    /// assert_eq!(progress.total(), 1000);
    /// assert_eq!(progress.completed(), 0);
    /// ```
    pub fn new(total: usize) -> Self {
        Self {
            total_targets: total,
            completed: AtomicUsize::new(0),
            open_ports: AtomicUsize::new(0),
            closed_ports: AtomicUsize::new(0),
            filtered_ports: AtomicUsize::new(0),
            connection_refused: AtomicUsize::new(0),
            timeouts: AtomicUsize::new(0),
            network_unreachable: AtomicUsize::new(0),
            host_unreachable: AtomicUsize::new(0),
            permission_denied: AtomicUsize::new(0),
            too_many_files: AtomicUsize::new(0),
            other_errors: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }

    /// Returns the total number of targets to scan.
    pub fn total(&self) -> usize {
        self.total_targets
    }

    /// Returns the number of completed scans.
    pub fn completed(&self) -> usize {
        self.completed.load(Ordering::Relaxed)
    }

    /// Returns the number of open ports found.
    pub fn open_ports(&self) -> usize {
        self.open_ports.load(Ordering::Relaxed)
    }

    /// Returns the number of closed ports found.
    pub fn closed_ports(&self) -> usize {
        self.closed_ports.load(Ordering::Relaxed)
    }

    /// Returns the number of filtered ports found.
    pub fn filtered_ports(&self) -> usize {
        self.filtered_ports.load(Ordering::Relaxed)
    }

    /// Increments the completed scan counter.
    pub fn increment_completed(&self) {
        self.completed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the open port counter.
    pub fn increment_open(&self) {
        self.open_ports.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the closed port counter.
    pub fn increment_closed(&self) {
        self.closed_ports.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the filtered port counter.
    pub fn increment_filtered(&self) {
        self.filtered_ports.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the error counter for the specified category.
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::progress::{ScanProgress, ErrorCategory};
    ///
    /// let progress = ScanProgress::new(100);
    /// progress.increment_error(ErrorCategory::Timeout);
    /// assert_eq!(progress.error_count(ErrorCategory::Timeout), 1);
    /// ```
    pub fn increment_error(&self, category: ErrorCategory) {
        match category {
            ErrorCategory::ConnectionRefused => {
                self.connection_refused.fetch_add(1, Ordering::Relaxed);
            }
            ErrorCategory::Timeout => {
                self.timeouts.fetch_add(1, Ordering::Relaxed);
            }
            ErrorCategory::NetworkUnreachable => {
                self.network_unreachable.fetch_add(1, Ordering::Relaxed);
            }
            ErrorCategory::HostUnreachable => {
                self.host_unreachable.fetch_add(1, Ordering::Relaxed);
            }
            ErrorCategory::PermissionDenied => {
                self.permission_denied.fetch_add(1, Ordering::Relaxed);
            }
            ErrorCategory::TooManyOpenFiles => {
                self.too_many_files.fetch_add(1, Ordering::Relaxed);
            }
            ErrorCategory::Other => {
                self.other_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Returns the count for a specific error category.
    pub fn error_count(&self, category: ErrorCategory) -> usize {
        match category {
            ErrorCategory::ConnectionRefused => self.connection_refused.load(Ordering::Relaxed),
            ErrorCategory::Timeout => self.timeouts.load(Ordering::Relaxed),
            ErrorCategory::NetworkUnreachable => self.network_unreachable.load(Ordering::Relaxed),
            ErrorCategory::HostUnreachable => self.host_unreachable.load(Ordering::Relaxed),
            ErrorCategory::PermissionDenied => self.permission_denied.load(Ordering::Relaxed),
            ErrorCategory::TooManyOpenFiles => self.too_many_files.load(Ordering::Relaxed),
            ErrorCategory::Other => self.other_errors.load(Ordering::Relaxed),
        }
    }

    /// Returns the total number of errors across all categories.
    pub fn total_errors(&self) -> usize {
        self.connection_refused.load(Ordering::Relaxed)
            + self.timeouts.load(Ordering::Relaxed)
            + self.network_unreachable.load(Ordering::Relaxed)
            + self.host_unreachable.load(Ordering::Relaxed)
            + self.permission_denied.load(Ordering::Relaxed)
            + self.too_many_files.load(Ordering::Relaxed)
            + self.other_errors.load(Ordering::Relaxed)
    }

    /// Returns the elapsed time since scan start.
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::progress::ScanProgress;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let progress = ScanProgress::new(100);
    /// thread::sleep(Duration::from_millis(10));
    /// assert!(progress.elapsed().as_millis() >= 10);
    /// ```
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Returns the current scan rate in targets per second.
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::progress::ScanProgress;
    ///
    /// let progress = ScanProgress::new(1000);
    /// for _ in 0..100 {
    ///     progress.increment_completed();
    /// }
    /// let rate = progress.rate_per_second();
    /// assert!(rate > 0.0);
    /// ```
    pub fn rate_per_second(&self) -> f64 {
        let completed = self.completed() as f64;
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            completed / elapsed
        } else {
            0.0
        }
    }

    /// Returns the estimated time to completion.
    ///
    /// Returns `None` if no targets have been completed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::progress::ScanProgress;
    ///
    /// let progress = ScanProgress::new(1000);
    /// assert!(progress.eta().is_none());
    ///
    /// for _ in 0..100 {
    ///     progress.increment_completed();
    /// }
    /// assert!(progress.eta().is_some());
    /// ```
    pub fn eta(&self) -> Option<Duration> {
        let completed = self.completed();
        if completed == 0 {
            return None;
        }

        let remaining = self.total_targets.saturating_sub(completed);
        let rate = self.rate_per_second();

        if rate > 0.0 {
            let seconds = (remaining as f64) / rate;
            Some(Duration::from_secs_f64(seconds))
        } else {
            None
        }
    }

    /// Returns the completion percentage (0.0 to 100.0).
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::progress::ScanProgress;
    ///
    /// let progress = ScanProgress::new(100);
    /// for _ in 0..50 {
    ///     progress.increment_completed();
    /// }
    /// assert!((progress.percentage() - 50.0).abs() < 0.01);
    /// ```
    pub fn percentage(&self) -> f64 {
        if self.total_targets == 0 {
            return 100.0;
        }
        (self.completed() as f64 / self.total_targets as f64) * 100.0
    }

    /// Returns a formatted summary of the scan progress.
    ///
    /// Includes elapsed time, rate, completion percentage, port states,
    /// and error breakdown.
    pub fn summary(&self) -> String {
        let elapsed = self.elapsed();
        let rate = self.rate_per_second();
        let percentage = self.percentage();
        let total_errors = self.total_errors();

        let mut summary = format!(
            "Scan Summary:\n\
             Duration: {:.2}s\n\
             Rate: {:.2} ports/sec\n\
             Progress: {}/{} ({:.1}%)\n\
             Open: {}\n\
             Closed: {}\n\
             Filtered: {}\n",
            elapsed.as_secs_f64(),
            rate,
            self.completed(),
            self.total_targets,
            percentage,
            self.open_ports(),
            self.closed_ports(),
            self.filtered_ports()
        );

        if total_errors > 0 {
            summary.push_str(&format!("\nErrors: {}\n", total_errors));

            let connection_refused = self.error_count(ErrorCategory::ConnectionRefused);
            if connection_refused > 0 {
                summary.push_str(&format!("  Connection Refused: {}\n", connection_refused));
            }

            let timeouts = self.error_count(ErrorCategory::Timeout);
            if timeouts > 0 {
                summary.push_str(&format!("  Timeouts: {}\n", timeouts));
            }

            let network_unreachable = self.error_count(ErrorCategory::NetworkUnreachable);
            if network_unreachable > 0 {
                summary.push_str(&format!("  Network Unreachable: {}\n", network_unreachable));
            }

            let host_unreachable = self.error_count(ErrorCategory::HostUnreachable);
            if host_unreachable > 0 {
                summary.push_str(&format!("  Host Unreachable: {}\n", host_unreachable));
            }

            let permission_denied = self.error_count(ErrorCategory::PermissionDenied);
            if permission_denied > 0 {
                summary.push_str(&format!("  Permission Denied: {}\n", permission_denied));
            }

            let too_many_files = self.error_count(ErrorCategory::TooManyOpenFiles);
            if too_many_files > 0 {
                summary.push_str(&format!("  Too Many Files: {}\n", too_many_files));
            }

            let other = self.error_count(ErrorCategory::Other);
            if other > 0 {
                summary.push_str(&format!("  Other: {}\n", other));
            }
        }

        summary
    }

    /// Exports statistics as JSON.
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::progress::ScanProgress;
    ///
    /// let progress = ScanProgress::new(100);
    /// progress.increment_completed();
    /// progress.increment_open();
    ///
    /// let json = progress.to_json().unwrap();
    /// assert!(json.contains("total"));
    /// assert!(json.contains("completed"));
    /// ```
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct ProgressStats {
            total: usize,
            completed: usize,
            open_ports: usize,
            closed_ports: usize,
            filtered_ports: usize,
            elapsed_secs: f64,
            rate_per_second: f64,
            percentage: f64,
            errors: ErrorStats,
        }

        #[derive(Serialize)]
        struct ErrorStats {
            total: usize,
            connection_refused: usize,
            timeouts: usize,
            network_unreachable: usize,
            host_unreachable: usize,
            permission_denied: usize,
            too_many_files: usize,
            other: usize,
        }

        let stats = ProgressStats {
            total: self.total_targets,
            completed: self.completed(),
            open_ports: self.open_ports(),
            closed_ports: self.closed_ports(),
            filtered_ports: self.filtered_ports(),
            elapsed_secs: self.elapsed().as_secs_f64(),
            rate_per_second: self.rate_per_second(),
            percentage: self.percentage(),
            errors: ErrorStats {
                total: self.total_errors(),
                connection_refused: self.error_count(ErrorCategory::ConnectionRefused),
                timeouts: self.error_count(ErrorCategory::Timeout),
                network_unreachable: self.error_count(ErrorCategory::NetworkUnreachable),
                host_unreachable: self.error_count(ErrorCategory::HostUnreachable),
                permission_denied: self.error_count(ErrorCategory::PermissionDenied),
                too_many_files: self.error_count(ErrorCategory::TooManyOpenFiles),
                other: self.error_count(ErrorCategory::Other),
            },
        };

        serde_json::to_string_pretty(&stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_new_progress() {
        let progress = ScanProgress::new(1000);
        assert_eq!(progress.total(), 1000);
        assert_eq!(progress.completed(), 0);
        assert_eq!(progress.open_ports(), 0);
        assert_eq!(progress.closed_ports(), 0);
        assert_eq!(progress.filtered_ports(), 0);
    }

    #[test]
    fn test_increment_counters() {
        let progress = ScanProgress::new(100);

        progress.increment_completed();
        progress.increment_open();
        assert_eq!(progress.completed(), 1);
        assert_eq!(progress.open_ports(), 1);

        progress.increment_completed();
        progress.increment_closed();
        assert_eq!(progress.completed(), 2);
        assert_eq!(progress.closed_ports(), 1);

        progress.increment_completed();
        progress.increment_filtered();
        assert_eq!(progress.completed(), 3);
        assert_eq!(progress.filtered_ports(), 1);
    }

    #[test]
    fn test_error_tracking() {
        let progress = ScanProgress::new(100);

        progress.increment_error(ErrorCategory::Timeout);
        progress.increment_error(ErrorCategory::Timeout);
        progress.increment_error(ErrorCategory::ConnectionRefused);

        assert_eq!(progress.error_count(ErrorCategory::Timeout), 2);
        assert_eq!(progress.error_count(ErrorCategory::ConnectionRefused), 1);
        assert_eq!(progress.total_errors(), 3);
    }

    #[test]
    fn test_elapsed_time() {
        let progress = ScanProgress::new(100);
        thread::sleep(Duration::from_millis(10));
        let elapsed = progress.elapsed();
        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_rate_calculation() {
        let progress = ScanProgress::new(1000);
        thread::sleep(Duration::from_millis(100));

        for _ in 0..100 {
            progress.increment_completed();
        }

        let rate = progress.rate_per_second();
        assert!(rate > 0.0);
        assert!(rate < 10000.0); // Sanity check
    }

    #[test]
    fn test_percentage() {
        let progress = ScanProgress::new(100);
        assert_eq!(progress.percentage(), 0.0);

        for _ in 0..50 {
            progress.increment_completed();
        }
        assert!((progress.percentage() - 50.0).abs() < 0.01);

        for _ in 0..50 {
            progress.increment_completed();
        }
        assert!((progress.percentage() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_eta() {
        let progress = ScanProgress::new(1000);
        assert!(progress.eta().is_none());

        thread::sleep(Duration::from_millis(100));
        for _ in 0..100 {
            progress.increment_completed();
        }

        let eta = progress.eta();
        assert!(eta.is_some());
    }

    #[test]
    fn test_summary() {
        let progress = ScanProgress::new(100);
        progress.increment_completed();
        progress.increment_open();
        progress.increment_error(ErrorCategory::Timeout);

        let summary = progress.summary();
        assert!(summary.contains("Scan Summary"));
        assert!(summary.contains("Open: 1"));
        assert!(summary.contains("Errors: 1"));
        assert!(summary.contains("Timeouts: 1"));
    }

    #[test]
    fn test_json_export() {
        let progress = ScanProgress::new(100);
        progress.increment_completed();
        progress.increment_open();
        progress.increment_error(ErrorCategory::Timeout);

        let json = progress.to_json().unwrap();
        assert!(json.contains("\"total\": 100"));
        assert!(json.contains("\"completed\": 1"));
        assert!(json.contains("\"open_ports\": 1"));
        assert!(json.contains("\"timeouts\": 1"));
    }

    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;

        let progress = Arc::new(ScanProgress::new(1000));
        let mut handles = vec![];

        for _ in 0..10 {
            let progress_clone = Arc::clone(&progress);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    progress_clone.increment_completed();
                    progress_clone.increment_open();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(progress.completed(), 1000);
        assert_eq!(progress.open_ports(), 1000);
    }
}
