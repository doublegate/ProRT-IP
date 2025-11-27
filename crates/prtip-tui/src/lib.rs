//! # prtip-tui
//!
//! Terminal User Interface for ProRT-IP network scanner.
//!
//! This crate provides a real-time TUI built with ratatui and crossterm,
//! integrating with ProRT-IP's EventBus for live scan monitoring.
//!
//! # Architecture
//!
//! The TUI follows an event-driven architecture:
//! - **Scanner** → EventBus → **TUI** (consumer-only, one-way)
//! - 60 FPS immediate mode rendering
//! - Async event loop with tokio::select!
//!
//! # Examples
//!
//! ```rust,no_run
//! use prtip_tui::App;
//! use prtip_core::event_bus::EventBus;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let event_bus = Arc::new(EventBus::new(1000));
//!     let mut app = App::new(event_bus);
//!     app.run().await?;
//!     Ok(())
//! }
//! ```

// Public API
pub mod app;
pub mod events;
pub mod shortcuts;
pub mod state;
pub mod ui;
pub mod widgets;

// Re-exports
pub use app::App;
pub use events::{EventAggregator, EventStats};
pub use shortcuts::ShortcutManager;
pub use state::{ScanState, UIState};
pub use widgets::Component;

/// TUI-specific error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Terminal initialization failed: {0}")]
    TerminalInit(String),

    #[error("Event handling error: {0}")]
    EventHandling(String),

    #[error("Rendering error: {0}")]
    Rendering(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
