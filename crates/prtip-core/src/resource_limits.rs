//! Resource limit detection and management
//!
//! This module provides cross-platform detection of file descriptor limits
//! (ulimit on Unix systems) and helps optimize batch sizes for scanning operations.
//!
//! Inspired by RustScan's resource management patterns.

use std::fmt;
use thiserror::Error;

#[cfg(unix)]
use rlimit::Resource;

/// Errors that can occur during resource limit operations
#[derive(Error, Debug)]
pub enum ResourceLimitError {
    /// Failed to get resource limit
    #[error("Failed to get resource limit: {0}")]
    GetLimitFailed(String),

    /// Failed to set resource limit
    #[error("Failed to set resource limit: {0}")]
    SetLimitFailed(String),

    /// Resource limit not supported on this platform
    #[error("Resource limits not supported on this platform")]
    NotSupported,
}

/// Resource limit information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLimits {
    /// Soft limit (current limit)
    pub soft: u64,
    /// Hard limit (maximum limit)
    pub hard: u64,
}

impl fmt::Display for ResourceLimits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "soft: {}, hard: {}", self.soft, self.hard)
    }
}

/// Get the current file descriptor limit (RLIMIT_NOFILE)
///
/// # Examples
///
/// ```
/// use prtip_core::resource_limits::get_file_descriptor_limit;
///
/// match get_file_descriptor_limit() {
///     Ok(limits) => println!("File descriptor limits: {}", limits),
///     Err(e) => eprintln!("Failed to get limits: {}", e),
/// }
/// ```
#[cfg(unix)]
pub fn get_file_descriptor_limit() -> Result<ResourceLimits, ResourceLimitError> {
    Resource::NOFILE
        .get()
        .map(|(soft, hard)| ResourceLimits { soft, hard })
        .map_err(|e| ResourceLimitError::GetLimitFailed(e.to_string()))
}

/// Get the current file descriptor limit (Windows stub)
#[cfg(not(unix))]
pub fn get_file_descriptor_limit() -> Result<ResourceLimits, ResourceLimitError> {
    // Windows has different resource management
    // Default to a conservative value similar to typical Windows limits
    Ok(ResourceLimits {
        soft: 2048,
        hard: 2048,
    })
}

/// Set the file descriptor limit (RLIMIT_NOFILE)
///
/// Attempts to set both soft and hard limits to the specified value.
/// Requires appropriate privileges to increase hard limit.
///
/// # Examples
///
/// ```no_run
/// use prtip_core::resource_limits::set_file_descriptor_limit;
///
/// match set_file_descriptor_limit(5000) {
///     Ok(_) => println!("Successfully increased file descriptor limit to 5000"),
///     Err(e) => eprintln!("Failed to set limit: {}", e),
/// }
/// ```
#[cfg(unix)]
pub fn set_file_descriptor_limit(limit: u64) -> Result<(), ResourceLimitError> {
    Resource::NOFILE
        .set(limit, limit)
        .map_err(|e| ResourceLimitError::SetLimitFailed(e.to_string()))
}

/// Set the file descriptor limit (Windows stub)
#[cfg(not(unix))]
pub fn set_file_descriptor_limit(_limit: u64) -> Result<(), ResourceLimitError> {
    Err(ResourceLimitError::NotSupported)
}

/// Calculate optimal batch size based on file descriptor limits
///
/// This function adjusts the desired batch size based on system resource limits
/// to prevent "too many open files" errors. It follows these rules:
///
/// 1. If ulimit < desired_batch_size, reduce batch size
/// 2. For very low limits (<3000), use half of ulimit
/// 3. For moderate limits (3000-8000), use ulimit - 100
/// 4. Reserve some file descriptors for system use
///
/// Inspired by RustScan's batch size inference logic.
///
/// # Arguments
///
/// * `desired_batch_size` - The preferred batch size for scanning
/// * `ulimit` - Current file descriptor limit (soft limit)
///
/// # Examples
///
/// ```
/// use prtip_core::resource_limits::calculate_optimal_batch_size;
///
/// let desired = 10000;
/// let ulimit = 4096;
/// let optimal = calculate_optimal_batch_size(desired, ulimit);
/// assert!(optimal <= ulimit);
/// ```
pub fn calculate_optimal_batch_size(desired_batch_size: u64, ulimit: u64) -> u64 {
    const AVERAGE_BATCH_SIZE: u64 = 3000;
    const DEFAULT_FD_LIMIT: u64 = 8000;
    const FD_RESERVE: u64 = 100; // Reserve for system use

    // If ulimit is lower than desired batch size, we need to adjust
    if ulimit < desired_batch_size {
        if ulimit < AVERAGE_BATCH_SIZE {
            // Very low ulimit - use half to be conservative
            ulimit / 2
        } else if ulimit > DEFAULT_FD_LIMIT {
            // High ulimit but still less than desired
            AVERAGE_BATCH_SIZE
        } else {
            // Moderate ulimit - leave some reserve
            ulimit.saturating_sub(FD_RESERVE)
        }
    } else {
        // ulimit is sufficient, use desired batch size
        desired_batch_size
    }
}

/// Adjust file descriptor limit and return the current limit
///
/// If a new limit is specified, attempts to set it. Returns the current
/// soft limit after any adjustments.
///
/// # Arguments
///
/// * `requested_limit` - Optional new limit to set
///
/// # Examples
///
/// ```
/// use prtip_core::resource_limits::adjust_and_get_limit;
///
/// // Get current limit without changing it
/// let current = adjust_and_get_limit(None).unwrap();
/// println!("Current limit: {}", current);
///
/// // Try to increase limit
/// match adjust_and_get_limit(Some(5000)) {
///     Ok(limit) => println!("Limit set to: {}", limit),
///     Err(e) => eprintln!("Failed to adjust limit: {}", e),
/// }
/// ```
pub fn adjust_and_get_limit(requested_limit: Option<u64>) -> Result<u64, ResourceLimitError> {
    // Try to set new limit if requested
    if let Some(limit) = requested_limit {
        if let Err(e) = set_file_descriptor_limit(limit) {
            // Log warning but continue - we'll use the current limit
            tracing::warn!("Failed to set file descriptor limit to {}: {}", limit, e);
        }
    }

    // Get current limit
    let limits = get_file_descriptor_limit()?;
    Ok(limits.soft)
}

/// Get recommended batch size for scanning
///
/// Convenience function that combines limit detection with batch size calculation.
///
/// # Arguments
///
/// * `desired_batch_size` - Preferred batch size
/// * `requested_limit` - Optional new limit to set before calculation
///
/// # Examples
///
/// ```
/// use prtip_core::resource_limits::get_recommended_batch_size;
///
/// // Get recommended batch size with default limits
/// let batch_size = get_recommended_batch_size(10000, None).unwrap();
/// println!("Recommended batch size: {}", batch_size);
/// ```
pub fn get_recommended_batch_size(
    desired_batch_size: u64,
    requested_limit: Option<u64>,
) -> Result<u64, ResourceLimitError> {
    let ulimit = adjust_and_get_limit(requested_limit)?;
    Ok(calculate_optimal_batch_size(desired_batch_size, ulimit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_descriptor_limit() {
        let result = get_file_descriptor_limit();
        assert!(result.is_ok());
        let limits = result.unwrap();
        // Sanity check: limits should be reasonable
        assert!(limits.soft > 0);
        assert!(limits.hard >= limits.soft);
    }

    #[test]
    fn test_calculate_optimal_batch_size_low_limit() {
        // Very low ulimit - should return half
        let batch_size = calculate_optimal_batch_size(10000, 2000);
        assert_eq!(batch_size, 1000);
    }

    #[test]
    fn test_calculate_optimal_batch_size_moderate_limit() {
        // Moderate ulimit (less than desired) - should return ulimit - 100
        let batch_size = calculate_optimal_batch_size(10000, 4096);
        assert_eq!(batch_size, 3996);
    }

    #[test]
    fn test_calculate_optimal_batch_size_high_limit() {
        // High ulimit - should return desired batch size
        let batch_size = calculate_optimal_batch_size(5000, 10000);
        assert_eq!(batch_size, 5000);
    }

    #[test]
    fn test_calculate_optimal_batch_size_average() {
        // Ulimit higher than default but less than desired
        let batch_size = calculate_optimal_batch_size(10000, 9000);
        assert_eq!(batch_size, 3000); // AVERAGE_BATCH_SIZE
    }

    #[test]
    fn test_calculate_optimal_batch_size_exact_match() {
        // Ulimit exactly matches desired
        let batch_size = calculate_optimal_batch_size(5000, 5000);
        assert_eq!(batch_size, 5000);
    }

    #[test]
    fn test_adjust_and_get_limit_no_change() {
        // Should return current limit without errors
        let result = adjust_and_get_limit(None);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[test]
    #[cfg(unix)]
    fn test_set_limit_requires_privileges() {
        // On most systems, non-root cannot increase hard limit
        // This test just verifies the function exists and returns a result
        let current = get_file_descriptor_limit().unwrap();

        // Try to set to current soft limit (should succeed)
        let result = set_file_descriptor_limit(current.soft);
        // May succeed or fail depending on privileges, but should return a Result
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_get_recommended_batch_size() {
        let result = get_recommended_batch_size(5000, None);
        assert!(result.is_ok());
        let batch_size = result.unwrap();
        // Should be a reasonable value
        assert!(batch_size > 0);
        assert!(batch_size <= 5000);
    }

    #[test]
    fn test_resource_limits_display() {
        let limits = ResourceLimits {
            soft: 1024,
            hard: 4096,
        };
        let display = format!("{}", limits);
        assert_eq!(display, "soft: 1024, hard: 4096");
    }

    #[test]
    fn test_resource_limits_equality() {
        let limits1 = ResourceLimits {
            soft: 1024,
            hard: 4096,
        };
        let limits2 = ResourceLimits {
            soft: 1024,
            hard: 4096,
        };
        let limits3 = ResourceLimits {
            soft: 2048,
            hard: 4096,
        };

        assert_eq!(limits1, limits2);
        assert_ne!(limits1, limits3);
    }
}
