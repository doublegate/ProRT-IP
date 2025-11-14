//! Event handling and EventBus integration

pub mod aggregator;
pub mod handlers;
pub mod subscriber;

// Internal module for event loop
pub(crate) mod r#loop;

pub use aggregator::{EventAggregator, EventStats};
pub use r#loop::{process_events, LoopControl};
pub use subscriber::subscribe_to_events;
