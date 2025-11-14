//! Reusable TUI widgets

mod component;
mod help_widget;
mod log_widget;
mod main_widget;
mod status;

pub use component::Component;
pub use help_widget::{handle_help_widget_event, HelpWidget};
pub use log_widget::{handle_log_widget_event, LogWidget};
pub use main_widget::{handle_main_widget_event, MainWidget};
pub use status::StatusBar;
