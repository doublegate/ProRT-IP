//! Progress tracking for network scanning operations
//!
//! This module provides both legacy (polling-based) and modern (event-driven)
//! progress tracking capabilities for ProRT-IP scanning operations.
//!
//! # Components
//!
//! ## Legacy (Polling-Based)
//!
//! - [`ScanProgress`] - Thread-safe atomic counters for progress tracking
//! - [`ErrorCategory`] - Error classification for statistics
//!
//! ## Event-Driven (Modern)
//!
//! - [`ProgressCalculator`] - Real-time ETA calculation with EWMA smoothing
//! - [`ThroughputMonitor`] - Throughput metrics (pps, hpm, Mbps) with sliding window
//! - [`ProgressAggregator`] - Event-based statistical aggregation
//!
//! # Examples
//!
//! ## Legacy Usage (Polling)
//!
//! ```
//! use prtip_core::progress::ScanProgress;
//!
//! let progress = ScanProgress::new(1000);
//! progress.increment_completed();
//! progress.increment_open();
//!
//! println!("Completed: {}/{}", progress.completed(), progress.total());
//! println!("ETA: {:?}", progress.eta());
//! ```
//!
//! ## Event-Driven Usage
//!
//! ```no_run
//! use prtip_core::progress::ProgressAggregator;
//! use prtip_core::event_bus::EventBus;
//! use std::sync::Arc;
//!
//! # async fn example() {
//! let bus = Arc::new(EventBus::new(1000));
//! let aggregator = ProgressAggregator::new(bus.clone());
//!
//! // Aggregator automatically subscribes to progress events
//! // and maintains real-time statistics with ETA and throughput
//!
//! let state = aggregator.get_state().await;
//! println!("Progress: {:.1}%", state.overall_progress);
//! println!("ETA: {:?}", state.eta);
//! println!("Throughput: {:.0} pps", state.throughput.packets_per_second);
//! # }
//! ```

mod aggregator;
mod calculator;
mod legacy;
mod monitor;

// Re-export legacy types for backward compatibility
pub use legacy::{ErrorCategory, ScanProgress};

// Re-export modern event-driven types
pub use aggregator::{AggregatedState, ProgressAggregator};
pub use calculator::ProgressCalculator;
pub use monitor::ThroughputMonitor;
