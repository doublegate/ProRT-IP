//! Shared scan state (thread-safe, shared with scanner)

use parking_lot::RwLock;
use prtip_core::events::ScanStage;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

/// Shared scan state accessible from both scanner and TUI
///
/// This state is wrapped in Arc<RwLock<>> for thread-safe access.
/// Updates come from EventBus event handlers.
#[derive(Debug, Clone)]
pub struct ScanState {
    /// Current scan stage
    pub stage: ScanStage,

    /// Scan progress percentage (0.0-100.0)
    pub progress_percentage: f32,

    /// Completed items
    pub completed: u64,

    /// Total items to process
    pub total: u64,

    /// Current throughput (packets per second)
    pub throughput_pps: f64,

    /// Estimated time to completion
    pub eta: Option<Duration>,

    /// Discovered IP addresses
    pub discovered_hosts: Vec<IpAddr>,

    /// Open ports discovered
    pub open_ports: usize,

    /// Closed ports discovered
    pub closed_ports: usize,

    /// Filtered ports discovered
    pub filtered_ports: usize,

    /// Services detected
    pub detected_services: usize,

    /// Errors encountered
    pub errors: usize,

    /// Warning messages
    pub warnings: Vec<String>,
}

impl ScanState {
    /// Create a new ScanState with default values
    pub fn new() -> Self {
        Self {
            stage: ScanStage::Initializing,
            progress_percentage: 0.0,
            completed: 0,
            total: 0,
            throughput_pps: 0.0,
            eta: None,
            discovered_hosts: Vec::new(),
            open_ports: 0,
            closed_ports: 0,
            filtered_ports: 0,
            detected_services: 0,
            errors: 0,
            warnings: Vec::new(),
        }
    }

    /// Create a thread-safe wrapper
    pub fn shared() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new()))
    }
}

impl Default for ScanState {
    fn default() -> Self {
        Self::new()
    }
}
