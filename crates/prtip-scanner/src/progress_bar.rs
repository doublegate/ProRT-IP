//! Real-time progress bar for scan operations

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::io::Write;
use std::time::{Duration, Instant};

/// Progress bar for scan operations
pub struct ScanProgressBar {
    bar: ProgressBar,
    start_time: Instant,
    total_ports: u64,
    enabled: bool,
}

impl ScanProgressBar {
    /// Create new progress bar
    pub fn new(total_ports: u64, enabled: bool) -> Self {
        let bar = if enabled {
            let pb = ProgressBar::new(total_ports);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ports ({per_sec} pps) ETA {eta}")
                    .unwrap()
                    .progress_chars("█▓░")
            );
            pb.set_draw_target(ProgressDrawTarget::stderr());
            // Note: steady_tick disabled - manual inc() calls provide updates
            // Steady tick can interfere with manual progress updates on fast localhost scans
            // pb.enable_steady_tick(Duration::from_millis(100));
            pb
        } else {
            ProgressBar::hidden()
        };

        Self {
            bar,
            start_time: Instant::now(),
            total_ports,
            enabled,
        }
    }

    /// Increment progress by n ports
    pub fn inc(&self, n: u64) {
        if self.enabled {
            self.bar.inc(n);
            // Force flush to ensure visibility
            let _ = std::io::stderr().flush();
        }
    }

    /// Set current position
    pub fn set_position(&self, pos: u64) {
        if self.enabled {
            self.bar.set_position(pos);
        }
    }

    /// Update message (scan phase)
    pub fn set_message(&self, msg: &str) {
        if self.enabled {
            self.bar.set_message(msg.to_string());
        }
    }

    /// Finish progress bar with message
    pub fn finish(&self, msg: &str) {
        if self.enabled {
            self.bar.finish_with_message(msg.to_string());
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Calculate scan rate (ports/sec)
    pub fn rate(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.bar.position() as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Get total ports being scanned
    pub fn total_ports(&self) -> u64 {
        self.total_ports
    }

    /// Get completion percentage (0.0 to 100.0)
    pub fn completion_percentage(&self) -> f64 {
        if self.total_ports == 0 {
            0.0
        } else {
            (self.bar.position() as f64 / self.total_ports as f64) * 100.0
        }
    }

    /// Get remaining ports to scan
    pub fn remaining_ports(&self) -> u64 {
        self.total_ports.saturating_sub(self.bar.position())
    }

    /// Estimate time remaining (based on current rate)
    pub fn estimated_remaining(&self) -> Duration {
        let remaining = self.remaining_ports();
        let rate = self.rate();

        if rate > 0.0 {
            let seconds = remaining as f64 / rate;
            Duration::from_secs_f64(seconds)
        } else {
            Duration::from_secs(0)
        }
    }

    /// Get a summary string with total ports info
    pub fn summary(&self) -> String {
        format!(
            "Scanned {}/{} ports ({:.1}% complete)",
            self.bar.position(),
            self.total_ports,
            self.completion_percentage()
        )
    }

    /// Print progress to stderr (for debugging when bar not visible)
    pub fn print_debug(&self) {
        if self.enabled {
            eprintln!(
                "Progress: {}/{} ports ({:.1}%) - {:.1} pps - {}s elapsed",
                self.bar.position(),
                self.total_ports,
                self.completion_percentage(),
                self.rate(),
                self.elapsed().as_secs()
            );
        }
    }
}

impl Drop for ScanProgressBar {
    fn drop(&mut self) {
        if self.enabled {
            self.bar.finish_and_clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_creation() {
        let pb = ScanProgressBar::new(1000, false);
        assert_eq!(pb.total_ports, 1000);
        assert!(!pb.enabled);
    }

    #[test]
    fn test_progress_bar_enabled() {
        let pb = ScanProgressBar::new(1000, true);
        assert_eq!(pb.total_ports, 1000);
        assert!(pb.enabled);
    }

    #[test]
    fn test_progress_increment() {
        let pb = ScanProgressBar::new(1000, false);
        pb.inc(10);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_elapsed_time() {
        let pb = ScanProgressBar::new(1000, false);
        std::thread::sleep(Duration::from_millis(10));
        assert!(pb.elapsed().as_millis() >= 10);
    }

    #[test]
    fn test_set_position() {
        let pb = ScanProgressBar::new(1000, false);
        pb.set_position(500);
        // Verify no panic
    }

    #[test]
    fn test_set_message() {
        let pb = ScanProgressBar::new(1000, false);
        pb.set_message("Testing");
        // Verify no panic
    }

    #[test]
    fn test_finish() {
        let pb = ScanProgressBar::new(1000, false);
        pb.finish("Complete");
        // Verify no panic
    }

    #[test]
    fn test_rate_calculation() {
        let pb = ScanProgressBar::new(1000, false);
        pb.inc(100);
        let rate = pb.rate();
        assert!(rate >= 0.0);
    }

    #[test]
    fn test_total_ports_getter() {
        let pb = ScanProgressBar::new(1000, false);
        assert_eq!(pb.total_ports(), 1000);
    }

    #[test]
    fn test_completion_percentage() {
        let pb = ScanProgressBar::new(1000, true);
        pb.set_position(250);
        assert_eq!(pb.completion_percentage(), 25.0);

        pb.set_position(500);
        assert_eq!(pb.completion_percentage(), 50.0);

        pb.set_position(1000);
        assert_eq!(pb.completion_percentage(), 100.0);
    }

    #[test]
    fn test_remaining_ports() {
        let pb = ScanProgressBar::new(1000, true); // enabled = true
        pb.set_position(300);
        assert_eq!(pb.remaining_ports(), 700);

        pb.set_position(1000);
        assert_eq!(pb.remaining_ports(), 0);
    }

    #[test]
    fn test_summary_string() {
        let pb = ScanProgressBar::new(1000, true); // enabled = true
        pb.set_position(250);
        let summary = pb.summary();
        assert!(summary.contains("250/1000"));
        assert!(summary.contains("25.0%"));
    }

    #[test]
    fn test_zero_total_ports() {
        let pb = ScanProgressBar::new(0, false);
        assert_eq!(pb.completion_percentage(), 0.0);
        assert_eq!(pb.remaining_ports(), 0);
    }

    #[test]
    fn test_estimated_remaining() {
        let pb = ScanProgressBar::new(1000, true);
        pb.set_position(500);
        let estimated = pb.estimated_remaining();
        // Should return a duration (may be zero if rate is zero due to fast execution)
        // Just verify it doesn't panic and returns a valid Duration
        let _ = estimated.as_secs();
    }

    #[test]
    fn test_print_debug() {
        let pb = ScanProgressBar::new(1000, true);
        pb.set_position(250);
        // Just verify it doesn't panic
        pb.print_debug();
    }
}
