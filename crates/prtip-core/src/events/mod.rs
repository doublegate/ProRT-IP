//! Event system for real-time scan monitoring
//!
//! This module provides a comprehensive event-driven architecture for ProRT-IP,
//! enabling real-time progress tracking, TUI integration, and monitoring.
//!
//! # Architecture
//!
//! - **Event Types**: 18+ event variants covering full scan lifecycle
//! - **Event Bus**: Pub-sub pattern with multi-subscriber support
//! - **Event History**: Ring buffer for querying and replay
//! - **Performance**: <5% overhead, <10ms p99 latency
//!
//! # Examples
//!
//! ```no_run
//! use prtip_core::events::{ScanEvent, ScanEventType};
//! use prtip_core::types::ScanType;
//! use uuid::Uuid;
//! use std::time::SystemTime;
//!
//! // Create a scan started event
//! let event = ScanEvent::ScanStarted {
//!     scan_id: Uuid::new_v4(),
//!     scan_type: ScanType::Syn,
//!     target_count: 1000,
//!     port_count: 1000,
//!     timestamp: SystemTime::now(),
//! };
//!
//! // Extract metadata
//! let scan_id = event.scan_id();
//! let event_type = event.event_type();
//!
//! // Validate event data
//! assert!(event.validate().is_ok());
//!
//! // Display for logging
//! println!("{}", event.display());
//! ```
//!
//! # See Also
//!
//! - [Event Bus Documentation](../event_bus/index.html) - Pub-sub event distribution
//! - [Progress Aggregator](../progress_aggregator/index.html) - Real-time state tracking
//! - [Event System Guide](../../docs/35-EVENT-SYSTEM-GUIDE.html) - Complete usage guide

mod types;

#[cfg(test)]
mod tests;

pub use types::{
    DiscoveryMethod, MetricType, PauseReason, ScanEvent, ScanEventType, ScanStage, Throughput,
    ValidationError, WarningSeverity,
};
