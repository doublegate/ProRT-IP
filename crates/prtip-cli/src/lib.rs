//! ProRT-IP CLI library
//!
//! Provides CLI components for the ProRT-IP network scanner.

pub mod args;
pub mod banner;
pub mod confirm;
pub mod db_commands;
pub mod error;
pub mod error_formatter;
pub mod export;
pub mod help;
pub mod history;
pub mod output;
pub mod progress;
pub mod templates;

pub use confirm::{ConfirmConfig, ConfirmationManager};
pub use error::{exit_codes, CliError};
pub use error_formatter::{create_error_formatter, ErrorFormatter};
pub use history::{HistoryEntry, HistoryManager};
pub use progress::{ProgressMetrics, ProgressStyle, ProgressTracker, ScanStage};
pub use templates::{ScanTemplate, TemplateManager};
