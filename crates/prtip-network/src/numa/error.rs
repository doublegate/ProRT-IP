//! Error types for NUMA operations

use thiserror::Error;

/// Errors that can occur during NUMA operations
#[derive(Debug, Error)]
pub enum NumaError {
    /// NUMA detection failed
    #[error("NUMA detection failed: {0}")]
    Detection(String),

    /// Thread pinning failed
    #[error("Thread pinning failed: {0}")]
    Pinning(String),

    /// NUMA not available on this system
    #[error("NUMA not available on this system (requires Linux with multi-socket hardware)")]
    NotAvailable,

    /// Invalid core ID
    #[error("Invalid core ID: {0}")]
    InvalidCore(String),

    /// Invalid NUMA node
    #[error("Invalid NUMA node: {0}")]
    InvalidNode(String),
}

/// Result type for NUMA operations
pub type Result<T> = std::result::Result<T, NumaError>;
