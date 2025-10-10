//! Adaptive Parallelism
//!
//! Automatically calculates optimal parallelism based on port count and system resources.
//! Inspired by RustScan and Masscan's adaptive approaches.
//!
//! # Design Philosophy
//!
//! - **Small scans (≤1K ports)**: Conservative parallelism (20-50) to avoid overhead
//! - **Medium scans (1K-10K ports)**: Moderate parallelism (100-500) for balanced performance
//! - **Large scans (>10K ports)**: Aggressive parallelism (500-2000) for maximum throughput
//! - **User override**: `--max-concurrent` flag takes precedence over adaptive values
//!
//! # System Integration
//!
//! The adaptive calculation respects system file descriptor limits (ulimit -n) and
//! provides actionable warnings when limits constrain performance.
//!
//! # Examples
//!
//! ```
//! use prtip_scanner::adaptive_parallelism::calculate_parallelism;
//!
//! // Small scan: 500 ports → 20 concurrent
//! let parallelism = calculate_parallelism(500, None, None);
//! assert_eq!(parallelism, 20);
//!
//! // Large scan: 65K ports → 1000 concurrent
//! let parallelism = calculate_parallelism(65535, None, None);
//! assert_eq!(parallelism, 1000);
//!
//! // User override takes precedence
//! let parallelism = calculate_parallelism(65535, Some(50), None);
//! assert_eq!(parallelism, 50);
//! ```

use prtip_core::resource_limits::get_file_descriptor_limit;
use tracing::{info, warn};

/// Maximum parallelism allowed (prevents runaway resource usage)
pub const MAX_PARALLELISM: usize = 2000;

/// Minimum parallelism (ensures reasonable performance)
pub const MIN_PARALLELISM: usize = 20;

/// Calculate optimal parallelism based on port count and system resources
///
/// This function implements adaptive parallelism that scales automatically based on
/// the number of ports being scanned. It respects system file descriptor limits and
/// user overrides.
///
/// # Arguments
///
/// * `port_count` - Number of ports being scanned
/// * `user_override` - User-specified parallelism via `--max-concurrent` flag
/// * `ulimit_override` - User-specified ulimit via `--ulimit` flag
///
/// # Returns
///
/// Optimal parallelism value that balances performance and resource usage
///
/// # Algorithm
///
/// 1. If user override provided, use it (bounded by MAX_PARALLELISM)
/// 2. Otherwise, calculate adaptive parallelism:
///    - 0-1000 ports: 20 concurrent (conservative)
///    - 1001-5000 ports: 100 concurrent (moderate)
///    - 5001-20000 ports: 500 concurrent (aggressive)
///    - 20001+ ports: 1000 concurrent (maximum)
/// 3. Check system ulimit and reduce if necessary
/// 4. Apply hard limits (MIN_PARALLELISM, MAX_PARALLELISM)
///
/// # Examples
///
/// ```
/// use prtip_scanner::adaptive_parallelism::calculate_parallelism;
///
/// // Adaptive scaling
/// assert_eq!(calculate_parallelism(500, None, None), 20);
/// assert_eq!(calculate_parallelism(2000, None, None), 100);
/// assert_eq!(calculate_parallelism(10000, None, None), 500);
/// assert_eq!(calculate_parallelism(65535, None, None), 1000);
///
/// // User override
/// assert_eq!(calculate_parallelism(65535, Some(50), None), 50);
/// assert_eq!(calculate_parallelism(1000, Some(500), None), 500);
/// ```
pub fn calculate_parallelism(
    port_count: usize,
    user_override: Option<usize>,
    ulimit_override: Option<u64>,
) -> usize {
    // User override takes absolute precedence
    if let Some(override_value) = user_override {
        let bounded = override_value.min(MAX_PARALLELISM).max(MIN_PARALLELISM);
        if override_value != bounded {
            warn!(
                "User parallelism {} bounded to valid range [{}, {}]",
                override_value, MIN_PARALLELISM, MAX_PARALLELISM
            );
        }
        info!("Using user-specified parallelism: {}", bounded);
        return bounded;
    }

    // Calculate adaptive parallelism based on port count
    let adaptive = match port_count {
        0..=1000 => {
            // Small scans: conservative to minimize overhead
            MIN_PARALLELISM
        }
        1001..=5000 => {
            // Medium scans: moderate parallelism
            100
        }
        5001..=20000 => {
            // Large scans: aggressive parallelism
            500
        }
        _ => {
            // Very large scans (>20K ports): maximum parallelism
            1000
        }
    };

    // Check system ulimit and adjust if necessary
    let ulimit_max = get_ulimit_constraint(ulimit_override);
    let final_parallelism = adaptive.min(ulimit_max).max(MIN_PARALLELISM);

    if adaptive > final_parallelism {
        warn!(
            "Adaptive parallelism {} reduced to {} due to file descriptor limits",
            adaptive, final_parallelism
        );
        warn!(
            "To increase: run 'ulimit -n {}' (Unix) or use --ulimit flag",
            adaptive * 2
        );
    } else {
        info!(
            "Using adaptive parallelism: {} for {} ports",
            final_parallelism, port_count
        );
    }

    final_parallelism
}

/// Get file descriptor constraint for parallelism
///
/// Returns the maximum safe parallelism based on available file descriptors.
/// Uses conservative calculation: available_fds / 2 to leave headroom for other
/// operations (logging, database, etc.).
fn get_ulimit_constraint(ulimit_override: Option<u64>) -> usize {
    match get_file_descriptor_limit() {
        Ok(limits) => {
            // Use override if provided, otherwise use detected soft limit
            let effective_limit = ulimit_override.unwrap_or(limits.soft);

            // Conservative: use 50% of available FDs for scanning
            // Reserve other 50% for: database, logging, OS overhead
            let safe_limit = (effective_limit / 2) as usize;

            safe_limit.max(MIN_PARALLELISM).min(MAX_PARALLELISM)
        }
        Err(e) => {
            warn!("Failed to detect file descriptor limits: {}", e);
            warn!("Using default constraint: {}", MAX_PARALLELISM);
            MAX_PARALLELISM
        }
    }
}

/// Get recommended parallelism for a specific scan type
///
/// Different scan types have different resource requirements. This function
/// provides type-specific recommendations.
///
/// # Arguments
///
/// * `port_count` - Number of ports to scan
/// * `scan_type` - Type of scan (Connect, SYN, UDP, etc.)
/// * `user_override` - User-specified parallelism override
///
/// # Returns
///
/// Recommended parallelism for the scan type
pub fn get_scan_type_parallelism(
    port_count: usize,
    scan_type: prtip_core::ScanType,
    user_override: Option<usize>,
) -> usize {
    use prtip_core::ScanType;

    // If user override provided, respect it exactly (no scan type adjustment)
    if let Some(override_value) = user_override {
        let bounded = override_value.min(MAX_PARALLELISM).max(MIN_PARALLELISM);
        return bounded;
    }

    // Calculate base adaptive parallelism
    let base = calculate_parallelism(port_count, None, None);

    // Adjust based on scan type characteristics
    match scan_type {
        ScanType::Connect => {
            // TCP Connect: full handshake, more resource intensive
            base
        }
        ScanType::Syn => {
            // SYN scan: stateless, can handle higher concurrency
            (base * 2).min(MAX_PARALLELISM)
        }
        ScanType::Udp => {
            // UDP: slower due to lack of response, reduce concurrency
            (base / 2).max(MIN_PARALLELISM)
        }
        ScanType::Fin | ScanType::Null | ScanType::Xmas | ScanType::Ack => {
            // Stealth scans: similar to SYN
            (base * 2).min(MAX_PARALLELISM)
        }
        ScanType::Idle => {
            // Idle (zombie) scan: extremely slow, very low concurrency
            MIN_PARALLELISM
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_parallelism_small_scan() {
        // Small scans: 20 concurrent
        assert_eq!(calculate_parallelism(100, None, None), MIN_PARALLELISM);
        assert_eq!(calculate_parallelism(500, None, None), MIN_PARALLELISM);
        assert_eq!(calculate_parallelism(1000, None, None), MIN_PARALLELISM);
    }

    #[test]
    fn test_adaptive_parallelism_medium_scan() {
        // Medium scans: 100 concurrent
        assert_eq!(calculate_parallelism(1001, None, None), 100);
        assert_eq!(calculate_parallelism(2000, None, None), 100);
        assert_eq!(calculate_parallelism(5000, None, None), 100);
    }

    #[test]
    fn test_adaptive_parallelism_large_scan() {
        // Large scans: 500 concurrent
        assert_eq!(calculate_parallelism(5001, None, None), 500);
        assert_eq!(calculate_parallelism(10000, None, None), 500);
        assert_eq!(calculate_parallelism(20000, None, None), 500);
    }

    #[test]
    fn test_adaptive_parallelism_very_large_scan() {
        // Very large scans: 1000 concurrent
        assert_eq!(calculate_parallelism(20001, None, None), 1000);
        assert_eq!(calculate_parallelism(50000, None, None), 1000);
        assert_eq!(calculate_parallelism(65535, None, None), 1000);
    }

    #[test]
    fn test_user_override() {
        // User override takes precedence
        assert_eq!(calculate_parallelism(65535, Some(50), None), 50);
        assert_eq!(calculate_parallelism(1000, Some(500), None), 500);
        assert_eq!(calculate_parallelism(100, Some(100), None), 100);
    }

    #[test]
    fn test_parallelism_respects_max_limit() {
        // Should not exceed MAX_PARALLELISM
        assert!(calculate_parallelism(1000000, None, None) <= MAX_PARALLELISM);
        assert_eq!(calculate_parallelism(1000, Some(5000), None), MAX_PARALLELISM);
    }

    #[test]
    fn test_parallelism_respects_min_limit() {
        // Should not go below MIN_PARALLELISM
        assert!(calculate_parallelism(10, None, None) >= MIN_PARALLELISM);
        assert_eq!(calculate_parallelism(1000, Some(5), None), MIN_PARALLELISM);
    }

    #[test]
    fn test_ulimit_constraint() {
        // Simulate low ulimit
        let low_ulimit = Some(100);
        let parallelism = calculate_parallelism(65535, None, low_ulimit);

        // Should be constrained by ulimit (100 / 2 = 50, but min is 20)
        assert!(parallelism <= 50);
        assert!(parallelism >= MIN_PARALLELISM);
    }

    #[test]
    fn test_scan_type_parallelism_connect() {
        use prtip_core::ScanType;

        let parallelism = get_scan_type_parallelism(10000, ScanType::Connect, None);
        // Connect scan at 10K ports: base 500
        assert_eq!(parallelism, 500);
    }

    #[test]
    fn test_scan_type_parallelism_syn() {
        use prtip_core::ScanType;

        let parallelism = get_scan_type_parallelism(10000, ScanType::Syn, None);
        // SYN scan: 2x base = 1000
        assert_eq!(parallelism, 1000);
    }

    #[test]
    fn test_scan_type_parallelism_udp() {
        use prtip_core::ScanType;

        let parallelism = get_scan_type_parallelism(10000, ScanType::Udp, None);
        // UDP scan: 0.5x base = 250
        assert_eq!(parallelism, 250);
    }

    #[test]
    fn test_scan_type_parallelism_stealth() {
        use prtip_core::ScanType;

        let parallelism = get_scan_type_parallelism(10000, ScanType::Fin, None);
        // Stealth scan: 2x base = 1000
        assert_eq!(parallelism, 1000);
    }

    #[test]
    fn test_scan_type_respects_max() {
        use prtip_core::ScanType;

        // SYN at 65K ports: base 1000 * 2 = 2000 (exactly MAX)
        let parallelism = get_scan_type_parallelism(65535, ScanType::Syn, None);
        assert_eq!(parallelism, MAX_PARALLELISM);
    }

    #[test]
    fn test_scan_type_user_override() {
        use prtip_core::ScanType;

        let parallelism = get_scan_type_parallelism(65535, ScanType::Syn, Some(100));
        // User override: 100 (not multiplied by scan type)
        assert_eq!(parallelism, 100);
    }
}
