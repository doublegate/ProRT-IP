//! Reusable TUI widgets

mod component;
mod help_widget;
mod log_widget;
mod main_widget;
mod metrics_dashboard;
mod port_table;
mod service_table;
mod status;

pub use component::Component;
pub use help_widget::{handle_help_widget_event, HelpWidget};
pub use log_widget::{handle_log_widget_event, LogWidget};
pub use main_widget::{handle_main_widget_event, MainWidget};
pub use metrics_dashboard::MetricsDashboardWidget;
pub use port_table::{handle_port_table_event, PortTableWidget};
pub use service_table::{handle_service_table_event, ServiceTableWidget};
pub use status::StatusBar;
