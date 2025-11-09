//! Event system for real-time scan monitoring
//!
//! This module provides a comprehensive event-driven architecture for ProRT-IP,
//! enabling real-time progress tracking, TUI integration, and monitoring.
//!
//! # Event Types Overview
//!
//! **18 Event Variants** across 5 categories:
//!
//! | Category | Events | Purpose |
//! |----------|--------|---------|
//! | **Lifecycle** | ScanStarted, ScanCompleted, ScanCancelled, ScanPaused, ScanResumed | Scan state transitions |
//! | **Discovery** | HostDiscovered, PortFound, IPv6PortFound | Network discovery results |
//! | **Detection** | ServiceDetected, OSDetected, BannerGrabbed, CertificateFound | Service/OS identification |
//! | **Progress** | ProgressUpdate, StageChanged | Real-time scan progress |
//! | **Diagnostic** | MetricRecorded, WarningIssued, RateLimitTriggered, RetryScheduled | Performance/errors |
//!
//! # Architecture
//!
//! - **Event Types**: 18 event variants covering full scan lifecycle
//! - **Event Bus**: Pub-sub pattern with multi-subscriber support
//! - **Event History**: Ring buffer for querying and replay
//! - **Performance**: <5% overhead, <10ms p99 latency (actual: ~40ns publish)
//!
//! # Common Event Fields
//!
//! All events include:
//! - `scan_id: Uuid` - Unique scan identifier
//! - `timestamp: SystemTime` - Event creation time
//!
//! # Examples
//!
//! ## Creating Events
//!
//! ```no_run
//! use prtip_core::events::{ScanEvent, ScanEventType};
//! use prtip_core::types::ScanType;
//! use uuid::Uuid;
//! use std::time::SystemTime;
//!
//! // Lifecycle event
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
//! ## Serialization (JSON Lines)
//!
//! ```no_run
//! use prtip_core::events::ScanEvent;
//!
//! # fn example(event: ScanEvent) -> Result<(), serde_json::Error> {
//! // Serialize to JSON
//! let json = serde_json::to_string(&event)?;
//!
//! // Deserialize from JSON
//! let event: ScanEvent = serde_json::from_str(&json)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Event Categories
//!
//! ## Lifecycle Events
//!
//! Track scan state transitions:
//! - `ScanStarted` - Scan initialization complete
//! - `ScanCompleted` - Scan finished successfully
//! - `ScanCancelled` - User requested cancellation
//! - `ScanPaused` / `ScanResumed` - Pause/resume support
//!
//! ## Discovery Events
//!
//! Report network discoveries:
//! - `HostDiscovered` - Live host found (ICMP, ARP, etc.)
//! - `PortFound` - Open TCP/UDP port detected
//! - `IPv6PortFound` - IPv6-specific port discovery
//!
//! ## Detection Events
//!
//! Service and OS identification:
//! - `ServiceDetected` - Service identified (HTTP, SSH, etc.)
//! - `OSDetected` - Operating system fingerprinted
//! - `BannerGrabbed` - Application banner retrieved
//! - `CertificateFound` - TLS certificate discovered
//!
//! ## Progress Events
//!
//! Real-time progress tracking:
//! - `ProgressUpdate` - Scan progress (%, ETA, throughput)
//! - `StageChanged` - Scan phase transition
//!
//! ## Diagnostic Events
//!
//! Performance monitoring and errors:
//! - `MetricRecorded` - Performance metric (packets sent, errors)
//! - `WarningIssued` - Non-fatal warning (timeout, rate limit)
//! - `RateLimitTriggered` - Rate limiter activated
//! - `RetryScheduled` - Failed operation retry planned
//!
//! # See Also
//!
//! - **Developer Guide**: [`docs/35-EVENT-SYSTEM-GUIDE.md`](../../../docs/35-EVENT-SYSTEM-GUIDE.md)
//! - **EventBus**: [`crate::event_bus`] - Pub-sub event distribution
//! - **Progress Tracking**: [`crate::progress`] - Real-time metrics
//! - **Event Logging**: [`crate::event_logger`] - JSON Lines logging

mod types;

#[cfg(test)]
mod tests;

pub use types::{
    DiscoveryMethod, MetricType, PauseReason, ScanEvent, ScanEventType, ScanStage, Throughput,
    ValidationError, WarningSeverity,
};
