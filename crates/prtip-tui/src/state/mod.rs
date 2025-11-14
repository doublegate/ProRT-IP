//! TUI state management

mod scan_state;
mod ui_state;

pub use scan_state::ScanState;
pub use ui_state::{
    EventFilter, EventType, HelpWidgetState, LogEntry, LogWidgetState, MainWidgetState, PortInfo,
    PortState, Protocol, SelectedPane, SortColumn, SortOrder, StatusBarState, UIState,
};
