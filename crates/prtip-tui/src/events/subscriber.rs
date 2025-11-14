//! EventBus subscription logic

use prtip_core::event_bus::{EventBus, EventFilter};
use prtip_core::events::ScanEvent;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Subscribe to EventBus and return event receiver channel
///
/// # Arguments
///
/// * `event_bus` - Arc reference to the EventBus
/// * `filter` - Event filter (All, ScanId, EventType, Custom)
///
/// # Returns
///
/// An unbounded receiver channel for ScanEvents
pub async fn subscribe_to_events(
    event_bus: Arc<EventBus>,
    filter: EventFilter,
) -> mpsc::UnboundedReceiver<ScanEvent> {
    let (tx, rx) = mpsc::unbounded_channel();
    event_bus.subscribe(tx, filter).await;
    rx
}
