//! Event Bus for real-time scan event distribution
//!
//! This module provides a thread-safe pub-sub event bus for ProRT-IP,
//! enabling multiple subscribers to receive scan events in real-time.
//!
//! # Architecture
//!
//! - **Thread Safety**: Arc<Mutex> for shared state
//! - **Multi-Subscriber**: Broadcast to all matching subscribers
//! - **Event History**: Ring buffer (1,000 events) for replay/query
//! - **Filtering**: Subscribe by scan ID, event type, or custom predicate
//!
//! # Performance Targets
//!
//! - **Overhead**: <5% with 10 subscribers
//! - **Latency**: <10ms p99 publish-to-receive
//! - **Throughput**: 10,000+ events/second
//!
//! # Examples
//!
//! ```no_run
//! use prtip_core::event_bus::{EventBus, EventFilter};
//! use prtip_core::events::ScanEvent;
//! use tokio::sync::mpsc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create event bus
//! let bus = EventBus::new(1000); // 1000-event history
//!
//! // Subscribe to all events
//! let (tx, mut rx) = mpsc::unbounded_channel();
//! bus.subscribe(tx, EventFilter::All).await;
//!
//! // Publish events (from scanner)
//! // Events are broadcast to all matching subscribers
//! // and stored in history ring buffer
//!
//! // Receive events
//! while let Some(event) = rx.recv().await {
//!     println!("Received: {}", event.display());
//! }
//! # Ok(())
//! # }
//! ```

use crate::events::{ScanEvent, ScanEventType};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

/// Event filter for selective subscription
#[derive(Clone)]
pub enum EventFilter {
    /// Subscribe to all events
    All,
    /// Subscribe to events from specific scan ID
    ScanId(Uuid),
    /// Subscribe to specific event types
    EventType(Vec<ScanEventType>),
    /// Custom predicate filter
    Custom(Arc<dyn Fn(&ScanEvent) -> bool + Send + Sync>),
}

impl std::fmt::Debug for EventFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventFilter::All => write!(f, "EventFilter::All"),
            EventFilter::ScanId(id) => write!(f, "EventFilter::ScanId({:?})", id),
            EventFilter::EventType(types) => write!(f, "EventFilter::EventType({:?})", types),
            EventFilter::Custom(_) => write!(f, "EventFilter::Custom(<fn>)"),
        }
    }
}

impl EventFilter {
    /// Check if event matches filter
    pub fn matches(&self, event: &ScanEvent) -> bool {
        match self {
            EventFilter::All => true,
            EventFilter::ScanId(id) => event.scan_id() == *id,
            EventFilter::EventType(types) => types.contains(&event.event_type()),
            EventFilter::Custom(predicate) => predicate(event),
        }
    }
}

/// Subscriber information
struct Subscriber {
    /// Channel to send events
    sender: UnboundedSender<ScanEvent>,
    /// Filter for this subscriber
    filter: EventFilter,
}

/// Event bus state
struct EventBusState {
    /// Active subscribers
    subscribers: Vec<Subscriber>,
    /// Event history ring buffer
    history: VecDeque<ScanEvent>,
    /// Maximum history size
    max_history: usize,
    /// Total events published
    total_events: u64,
    /// Events dropped (slow subscriber)
    dropped_events: u64,
}

/// Event Bus for real-time scan event distribution
///
/// Thread-safe pub-sub event bus supporting multiple subscribers
/// with filtering, event history, and replay capabilities.
///
/// # Performance
///
/// - Optimized for low latency (<10ms p99)
/// - Minimal overhead (<5% with 10 subscribers)
/// - High throughput (10,000+ events/sec)
///
/// # Thread Safety
///
/// EventBus uses Arc<Mutex> for interior mutability and can be
/// safely cloned and shared across threads.
#[derive(Clone)]
pub struct EventBus {
    state: Arc<Mutex<EventBusState>>,
}

impl EventBus {
    /// Create a new event bus with specified history size
    ///
    /// # Arguments
    ///
    /// * `max_history` - Maximum events to store in ring buffer (default: 1000)
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::event_bus::EventBus;
    ///
    /// let bus = EventBus::new(1000);
    /// ```
    pub fn new(max_history: usize) -> Self {
        EventBus {
            state: Arc::new(Mutex::new(EventBusState {
                subscribers: Vec::new(),
                history: VecDeque::with_capacity(max_history),
                max_history,
                total_events: 0,
                dropped_events: 0,
            })),
        }
    }

    /// Subscribe to events with optional filter
    ///
    /// Returns immediately. Events matching the filter will be sent to the
    /// provided channel as they are published.
    ///
    /// # Arguments
    ///
    /// * `sender` - Unbounded channel to receive events
    /// * `filter` - Event filter (All, ScanId, EventType, Custom)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::event_bus::{EventBus, EventFilter};
    /// use tokio::sync::mpsc;
    /// use uuid::Uuid;
    ///
    /// # async fn example() {
    /// let bus = EventBus::new(1000);
    /// let (tx, rx) = mpsc::unbounded_channel();
    ///
    /// // Subscribe to specific scan
    /// let scan_id = Uuid::new_v4();
    /// bus.subscribe(tx, EventFilter::ScanId(scan_id)).await;
    /// # }
    /// ```
    pub async fn subscribe(&self, sender: UnboundedSender<ScanEvent>, filter: EventFilter) {
        let mut state = self.state.lock();
        state.subscribers.push(Subscriber { sender, filter });
    }

    /// Publish an event to all matching subscribers
    ///
    /// The event is:
    /// 1. Validated before publishing
    /// 2. Broadcast to all subscribers with matching filters
    /// 3. Added to event history ring buffer
    ///
    /// Slow subscribers (closed channels) are automatically removed.
    ///
    /// # Arguments
    ///
    /// * `event` - Event to publish
    ///
    /// # Returns
    ///
    /// Number of subscribers that received the event
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::event_bus::EventBus;
    /// use prtip_core::events::ScanEvent;
    /// use prtip_core::types::ScanType;
    /// use uuid::Uuid;
    /// use std::time::SystemTime;
    ///
    /// # async fn example() {
    /// let bus = EventBus::new(1000);
    ///
    /// let event = ScanEvent::ScanStarted {
    ///     scan_id: Uuid::new_v4(),
    ///     scan_type: ScanType::Syn,
    ///     target_count: 1000,
    ///     port_count: 1000,
    ///     timestamp: SystemTime::now(),
    /// };
    ///
    /// let delivered = bus.publish(event).await;
    /// println!("Delivered to {} subscribers", delivered);
    /// # }
    /// ```
    pub async fn publish(&self, event: ScanEvent) -> usize {
        // Validate event before publishing
        if let Err(e) = event.validate() {
            tracing::warn!("Invalid event rejected: {}", e);
            return 0;
        }

        let mut state = self.state.lock();

        // Track statistics
        state.total_events += 1;

        // Add to history ring buffer
        if state.history.len() >= state.max_history {
            state.history.pop_front();
        }
        state.history.push_back(event.clone());

        // Broadcast to matching subscribers
        let mut delivered = 0;
        let mut to_remove = Vec::new();

        for (idx, subscriber) in state.subscribers.iter().enumerate() {
            if subscriber.filter.matches(&event) {
                match subscriber.sender.send(event.clone()) {
                    Ok(_) => delivered += 1,
                    Err(_) => {
                        // Subscriber channel closed, mark for removal
                        to_remove.push(idx);
                    }
                }
            }
        }

        // Update dropped events counter
        state.dropped_events += to_remove.len() as u64;

        // Remove closed subscribers (reverse order to maintain indices)
        for idx in to_remove.into_iter().rev() {
            state.subscribers.remove(idx);
        }

        delivered
    }

    /// Get last N events from history
    ///
    /// Returns up to N most recent events, in chronological order.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of events to retrieve
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_core::event_bus::EventBus;
    ///
    /// # async fn example() {
    /// let bus = EventBus::new(1000);
    /// let recent = bus.get_history(10).await;
    /// println!("Last {} events", recent.len());
    /// # }
    /// ```
    pub async fn get_history(&self, count: usize) -> Vec<ScanEvent> {
        let state = self.state.lock();
        let start = state.history.len().saturating_sub(count);
        state.history.iter().skip(start).cloned().collect()
    }

    /// Get events in time range
    ///
    /// Returns events with timestamps between `start` and `end` (inclusive).
    ///
    /// # Arguments
    ///
    /// * `start` - Start time (inclusive)
    /// * `end` - End time (inclusive)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::event_bus::EventBus;
    /// use std::time::{SystemTime, Duration};
    ///
    /// # async fn example() {
    /// let bus = EventBus::new(1000);
    /// let now = SystemTime::now();
    /// let one_min_ago = now - Duration::from_secs(60);
    ///
    /// let recent = bus.get_time_range(one_min_ago, now).await;
    /// # }
    /// ```
    pub async fn get_time_range(&self, start: SystemTime, end: SystemTime) -> Vec<ScanEvent> {
        let state = self.state.lock();
        state
            .history
            .iter()
            .filter(|e| {
                let ts = e.timestamp();
                ts >= start && ts <= end
            })
            .cloned()
            .collect()
    }

    /// Get events matching filter from history
    ///
    /// # Arguments
    ///
    /// * `filter` - Event filter to apply
    /// * `limit` - Maximum events to return
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::event_bus::{EventBus, EventFilter};
    /// use prtip_core::events::ScanEventType;
    ///
    /// # async fn example() {
    /// let bus = EventBus::new(1000);
    ///
    /// // Get all port discovery events
    /// let ports = bus.query_history(
    ///     EventFilter::EventType(vec![ScanEventType::PortFound]),
    ///     100
    /// ).await;
    /// # }
    /// ```
    pub async fn query_history(&self, filter: EventFilter, limit: usize) -> Vec<ScanEvent> {
        let state = self.state.lock();
        state
            .history
            .iter()
            .filter(|e| filter.matches(e))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get event bus statistics
    ///
    /// Returns (total_events, dropped_events, active_subscribers, history_size)
    pub async fn stats(&self) -> (u64, u64, usize, usize) {
        let state = self.state.lock();
        (
            state.total_events,
            state.dropped_events,
            state.subscribers.len(),
            state.history.len(),
        )
    }

    /// Clear event history
    ///
    /// Removes all events from the ring buffer. Does not affect subscribers.
    pub async fn clear_history(&self) {
        let mut state = self.state.lock();
        state.history.clear();
    }

    /// Get number of active subscribers
    pub async fn subscriber_count(&self) -> usize {
        let state = self.state.lock();
        state.subscribers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ScanType;
    use std::time::Duration;

    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = EventBus::new(1000);
        assert_eq!(bus.subscriber_count().await, 0);

        let (total, dropped, subs, history) = bus.stats().await;
        assert_eq!(total, 0);
        assert_eq!(dropped, 0);
        assert_eq!(subs, 0);
        assert_eq!(history, 0);
    }

    #[tokio::test]
    async fn test_subscribe_and_publish() {
        let bus = EventBus::new(1000);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        bus.subscribe(tx, EventFilter::All).await;
        assert_eq!(bus.subscriber_count().await, 1);

        let event = ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            scan_type: ScanType::Syn,
            target_count: 100,
            port_count: 1000,
            timestamp: SystemTime::now(),
        };

        let delivered = bus.publish(event.clone()).await;
        assert_eq!(delivered, 1);

        let received = rx.recv().await.unwrap();
        assert_eq!(received.scan_id(), event.scan_id());
    }

    #[tokio::test]
    async fn test_event_history() {
        let bus = EventBus::new(10);

        for i in 0..15 {
            let event = ScanEvent::ScanStarted {
                scan_id: Uuid::new_v4(),
                scan_type: ScanType::Syn,
                target_count: i,
                port_count: 1000,
                timestamp: SystemTime::now(),
            };
            bus.publish(event).await;
        }

        let history = bus.get_history(100).await;
        // Ring buffer keeps last 10
        assert_eq!(history.len(), 10);

        let (total, _, _, hist_size) = bus.stats().await;
        assert_eq!(total, 15);
        assert_eq!(hist_size, 10);
    }

    #[tokio::test]
    async fn test_filter_by_scan_id() {
        let bus = EventBus::new(1000);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let target_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();

        bus.subscribe(tx, EventFilter::ScanId(target_id)).await;

        // Publish event with target ID
        let event1 = ScanEvent::ScanStarted {
            scan_id: target_id,
            scan_type: ScanType::Syn,
            target_count: 100,
            port_count: 1000,
            timestamp: SystemTime::now(),
        };
        bus.publish(event1).await;

        // Publish event with other ID (should be filtered)
        let event2 = ScanEvent::ScanStarted {
            scan_id: other_id,
            scan_type: ScanType::Syn,
            target_count: 100,
            port_count: 1000,
            timestamp: SystemTime::now(),
        };
        bus.publish(event2).await;

        // Should only receive target ID event
        let received = rx.recv().await.unwrap();
        assert_eq!(received.scan_id(), target_id);

        // No more events
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_filter_by_event_type() {
        let bus = EventBus::new(1000);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        bus.subscribe(
            tx,
            EventFilter::EventType(vec![
                ScanEventType::ScanStarted,
                ScanEventType::ScanCompleted,
            ]),
        )
        .await;

        let scan_id = Uuid::new_v4();

        // Should receive
        let event1 = ScanEvent::ScanStarted {
            scan_id,
            scan_type: ScanType::Syn,
            target_count: 100,
            port_count: 1000,
            timestamp: SystemTime::now(),
        };
        bus.publish(event1).await;

        // Should not receive (ProgressUpdate)
        let event2 = ScanEvent::ProgressUpdate {
            scan_id,
            stage: crate::events::ScanStage::ScanningPorts,
            percentage: 50.0,
            completed: 50,
            total: 100,
            throughput: crate::events::Throughput::default(),
            eta: None,
            timestamp: SystemTime::now(),
        };
        bus.publish(event2).await;

        let received = rx.recv().await.unwrap();
        assert_eq!(received.event_type(), ScanEventType::ScanStarted);

        // No more events
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_time_range_query() {
        let bus = EventBus::new(1000);

        let now = SystemTime::now();
        let past = now - Duration::from_secs(60);
        let future = now + Duration::from_secs(60);

        // Event in past
        let event1 = ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            scan_type: ScanType::Syn,
            target_count: 1,
            port_count: 1000,
            timestamp: past,
        };
        bus.publish(event1).await;

        // Event in present
        let event2 = ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            scan_type: ScanType::Syn,
            target_count: 2,
            port_count: 1000,
            timestamp: now,
        };
        bus.publish(event2).await;

        // Query range excluding past
        let in_range = bus.get_time_range(now, future).await;
        assert_eq!(in_range.len(), 1);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new(1000);

        let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();
        let (tx3, mut rx3) = tokio::sync::mpsc::unbounded_channel();

        bus.subscribe(tx1, EventFilter::All).await;
        bus.subscribe(tx2, EventFilter::All).await;
        bus.subscribe(tx3, EventFilter::All).await;

        let event = ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            scan_type: ScanType::Syn,
            target_count: 100,
            port_count: 1000,
            timestamp: SystemTime::now(),
        };

        let delivered = bus.publish(event.clone()).await;
        assert_eq!(delivered, 3);

        // All subscribers should receive
        assert!(rx1.recv().await.is_some());
        assert!(rx2.recv().await.is_some());
        assert!(rx3.recv().await.is_some());
    }

    #[tokio::test]
    async fn test_closed_subscriber_removal() {
        let bus = EventBus::new(1000);

        let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();

        bus.subscribe(tx1, EventFilter::All).await;
        bus.subscribe(tx2, EventFilter::All).await;

        assert_eq!(bus.subscriber_count().await, 2);

        // Drop rx1 to close channel
        drop(rx1);

        let event = ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            scan_type: ScanType::Syn,
            target_count: 100,
            port_count: 1000,
            timestamp: SystemTime::now(),
        };

        let delivered = bus.publish(event).await;
        assert_eq!(delivered, 1); // Only rx2 received

        // Closed subscriber should be removed
        assert_eq!(bus.subscriber_count().await, 1);

        // rx2 should still work
        assert!(rx2.recv().await.is_some());
    }

    #[tokio::test]
    async fn test_clear_history() {
        let bus = EventBus::new(1000);

        for _ in 0..10 {
            let event = ScanEvent::ScanStarted {
                scan_id: Uuid::new_v4(),
                scan_type: ScanType::Syn,
                target_count: 100,
                port_count: 1000,
                timestamp: SystemTime::now(),
            };
            bus.publish(event).await;
        }

        let (_, _, _, hist_before) = bus.stats().await;
        assert_eq!(hist_before, 10);

        bus.clear_history().await;

        let (_, _, _, hist_after) = bus.stats().await;
        assert_eq!(hist_after, 0);
    }
}
