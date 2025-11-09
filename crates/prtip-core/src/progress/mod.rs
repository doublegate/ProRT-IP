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
//! ## Event-Driven (Modern) **← Recommended**
//!
//! - [`ProgressCalculator`] - Real-time ETA calculation with EWMA smoothing
//! - [`ThroughputMonitor`] - Throughput metrics (pps, hpm, Mbps) with sliding window
//! - [`ProgressAggregator`] - Event-based statistical aggregation
//!
//! # ProgressAggregator Overview
//!
//! **Primary component for real-time scan monitoring**
//!
//! **Features:**
//! - **Auto-Subscribes**: Automatically subscribes to EventBus progress events
//! - **Real-Time Metrics**: Percentage, completed/total, throughput, ETA
//! - **ETA Calculation**: EWMA-smoothed estimates based on recent throughput
//! - **Multi-Scanner**: Supports concurrent scan aggregation
//! - **Thread-Safe**: `Arc<RwLock<>>` for concurrent access
//!
//! **Metrics Provided:**
//! - `percentage: f64` - Scan completion (0-100%)
//! - `completed: usize` - Items processed
//! - `total: usize` - Total items
//! - `throughput: Throughput` - Current packets/sec, ports/sec
//! - `eta: Option<Duration>` - Estimated time to completion
//! - `stage: ScanStage` - Current scan phase
//!
//! **Update Frequency:**
//! - Metrics updated on every ProgressUpdate event
//! - Throughput calculated from last 5 seconds (moving average)
//! - ETA recalculated every update
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
//! ## Event-Driven Usage (Recommended)
//!
//! ```ignore
//! use prtip_core::progress::ProgressAggregator;
//! use prtip_core::event_bus::EventBus;
//! use std::sync::Arc;
//! use uuid::Uuid;
//!
//! # async fn example() {
//! let bus = Arc::new(EventBus::new(1000));
//! let scan_id = Uuid::new_v4();
//!
//! // Create aggregator (auto-subscribes to progress events)
//! let aggregator = Arc::new(
//!     ProgressAggregator::new(bus.clone(), scan_id)
//! );
//!
//! // Initialize with totals
//! aggregator.start(1000, 100).await; // 1000 targets, 100 ports
//!
//! // Get current metrics (async, non-blocking)
//! let metrics = aggregator.get_current_metrics().await;
//! println!("Progress: {:.1}%", metrics.percentage);
//! println!("Throughput: {} pps", metrics.throughput.packets_per_sec);
//!
//! if let Some(eta) = metrics.eta {
//!     println!("ETA: {} seconds", eta.as_secs());
//! }
//! # }
//! ```
//!
//! ## Real-Time Monitoring
//!
//! ```ignore
//! use prtip_core::progress::ProgressAggregator;
//! use tokio::time::{interval, Duration};
//! # use std::sync::Arc;
//! # use prtip_core::event_bus::EventBus;
//! # use uuid::Uuid;
//!
//! # async fn example() {
//! # let bus = Arc::new(EventBus::new(1000));
//! # let scan_id = Uuid::new_v4();
//! let aggregator = Arc::new(ProgressAggregator::new(bus, scan_id));
//! aggregator.start(1000, 100).await;
//!
//! // Update every second
//! let mut ticker = interval(Duration::from_secs(1));
//!
//! loop {
//!     ticker.tick().await;
//!
//!     let metrics = aggregator.get_current_metrics().await;
//!
//!     print!("\rProgress: {:.1}% ({}/{}) ",
//!         metrics.percentage,
//!         metrics.completed,
//!         metrics.total
//!     );
//!
//!     if metrics.percentage >= 100.0 {
//!         println!("\nComplete!");
//!         break;
//!     }
//! }
//! # }
//! ```
//!
//! # See Also
//!
//! - **Developer Guide**: [`docs/35-EVENT-SYSTEM-GUIDE.md`](../../../docs/35-EVENT-SYSTEM-GUIDE.md)
//! - **EventBus**: [`crate::event_bus`] - Event distribution
//! - **Event Types**: [`crate::events`] - ProgressUpdate events
//! - **Benchmarks**: [`benchmarks/event-system-baseline.md`](../../../benchmarks/event-system-baseline.md)

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
