//! TUI state management

mod scan_state;
mod ui_state;

pub use scan_state::{
    PortDiscovery, PortState as ScanPortState, Protocol as ScanProtocol, ScanState, ScanType,
    ServiceDetection, ThroughputSample,
};
pub use ui_state::{
    ConfidenceFilter, DashboardTab, EventFilter, EventType, GraphType, HelpWidgetState, LogEntry,
    LogWidgetState, MainWidgetState, NetworkGraphState, PortInfo, PortTableColumn, PortTableState,
    SelectedPane, ServicePanelState, ServiceTableColumn, ServiceTableState, SortColumn, SortOrder,
    StatusBarState, UIState,
};
