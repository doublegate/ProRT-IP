//! Component trait for modular TUI widgets
//!
//! This trait defines the interface that all TUI components must implement
//! for rendering and event handling.

use crossterm::event::Event;
use ratatui::{layout::Rect, Frame};

use crate::state::UIState;

/// Trait for TUI components that can render and handle events
///
/// # Examples
///
/// ```rust,ignore
/// use prtip_tui::{Component, UIState};
/// use ratatui::{Frame, layout::Rect};
/// use crossterm::event::Event;
///
/// struct PortTable {
///     // ... fields
/// }
///
/// impl Component for PortTable {
///     fn render(&self, frame: &mut Frame, area: Rect, state: &UIState) {
///         // Render the port table
///     }
///
///     fn handle_event(&mut self, event: Event) -> bool {
///         // Handle keyboard/mouse events
///         // Return true if event was handled
///         false
///     }
/// }
/// ```
pub trait Component {
    /// Render the component to the given frame area
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render to
    /// * `area` - The rectangular area to render within
    /// * `state` - Current UI state for rendering decisions
    fn render(&self, frame: &mut Frame, area: Rect, state: &UIState);

    /// Handle a crossterm event (keyboard, mouse, resize, etc.)
    ///
    /// # Arguments
    ///
    /// * `event` - The crossterm event to handle
    ///
    /// # Returns
    ///
    /// `true` if the event was handled and should not propagate further,
    /// `false` if the event should continue propagating
    fn handle_event(&mut self, event: Event) -> bool {
        // Default: don't handle events
        let _ = event;
        false
    }
}
