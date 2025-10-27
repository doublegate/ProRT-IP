//! Circuit breaker pattern for failing targets
//!
//! This module implements the circuit breaker pattern to prevent repeated attempts
//! to scan failing targets. Benefits:
//! - Reduces wasted effort on unreachable/failing targets
//! - Prevents resource exhaustion from retry storms
//! - Allows automatic recovery testing (half-open state)
//! - Tracks per-target statistics for debugging
//!
//! Sprint 4.22 Phase 4: Recovery Mechanisms

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests allowed
    Closed,

    /// Too many failures - requests blocked
    Open,

    /// Testing if service recovered - limited requests allowed
    HalfOpen,
}

/// Circuit breaker statistics for a single target
#[derive(Debug, Clone)]
pub struct CircuitStats {
    pub success_count: u32,
    pub failure_count: u32,
    pub state: CircuitState,
    pub last_failure: Option<Instant>,
    pub opened_at: Option<Instant>,
}

/// Circuit breaker for a single target
#[derive(Debug)]
struct Circuit {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure: Option<Instant>,
    opened_at: Option<Instant>,
}

impl Circuit {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure: None,
            opened_at: None,
        }
    }

    fn to_stats(&self) -> CircuitStats {
        CircuitStats {
            success_count: self.success_count,
            failure_count: self.failure_count,
            state: self.state,
            last_failure: self.last_failure,
            opened_at: self.opened_at,
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,

    /// Number of successes to close circuit from half-open
    pub success_threshold: u32,

    /// How long to wait before trying again after opening
    pub timeout: Duration,

    /// How many requests to allow in half-open state
    pub half_open_limit: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
            half_open_limit: 3,
        }
    }
}

/// Multi-target circuit breaker
///
/// Maintains separate circuit state for each target IP to prevent
/// repeated attempts to failing targets while allowing other targets
/// to continue operating normally.
///
/// # Example
///
/// ```no_run
/// use prtip_core::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
/// use std::net::IpAddr;
///
/// # async fn example() {
/// let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
/// let target: IpAddr = "192.168.1.1".parse().unwrap();
///
/// if breaker.should_attempt(target).await {
///     // Attempt scan
///     match scan_target(target).await {
///         Ok(_) => breaker.record_success(target).await,
///         Err(_) => breaker.record_failure(target).await,
///     }
/// } else {
///     // Circuit is open, skip this target
/// }
/// # }
/// # async fn scan_target(_target: IpAddr) -> Result<(), ()> { Ok(()) }
/// ```
pub struct CircuitBreaker {
    circuits: RwLock<HashMap<IpAddr, Circuit>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            circuits: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Check if a target should be attempted
    ///
    /// Returns `false` if circuit is open (too many failures).
    /// Returns `true` if circuit is closed or transitioning to half-open.
    pub async fn should_attempt(&self, target: IpAddr) -> bool {
        let mut circuits = self.circuits.write().await;
        let circuit = circuits.entry(target).or_insert_with(Circuit::new);

        match circuit.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(opened_at) = circuit.opened_at {
                    if opened_at.elapsed() >= self.config.timeout {
                        // Transition to half-open
                        circuit.state = CircuitState::HalfOpen;
                        circuit.success_count = 0;
                        circuit.failure_count = 0;
                        true
                    } else {
                        false // Still open
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                let total_attempts = circuit.success_count + circuit.failure_count;
                total_attempts < self.config.half_open_limit
            }
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self, target: IpAddr) {
        let mut circuits = self.circuits.write().await;
        let circuit = circuits.entry(target).or_insert_with(Circuit::new);

        circuit.success_count += 1;
        circuit.failure_count = 0; // Reset failure count

        match circuit.state {
            CircuitState::HalfOpen => {
                if circuit.success_count >= self.config.success_threshold {
                    // Close circuit - service recovered
                    circuit.state = CircuitState::Closed;
                    circuit.opened_at = None;
                }
            }
            CircuitState::Closed => {
                // Already closed, nothing to do
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                circuit.state = CircuitState::Closed;
                circuit.opened_at = None;
            }
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self, target: IpAddr) {
        let mut circuits = self.circuits.write().await;
        let circuit = circuits.entry(target).or_insert_with(Circuit::new);

        circuit.failure_count += 1;
        circuit.last_failure = Some(Instant::now());

        match circuit.state {
            CircuitState::Closed => {
                if circuit.failure_count >= self.config.failure_threshold {
                    // Open circuit - too many failures
                    circuit.state = CircuitState::Open;
                    circuit.opened_at = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                // Re-open circuit - service still failing
                circuit.state = CircuitState::Open;
                circuit.opened_at = Some(Instant::now());
                circuit.success_count = 0;
                circuit.failure_count = 1; // Reset to 1
            }
            CircuitState::Open => {
                // Already open, increment failure count
            }
        }
    }

    /// Get statistics for a specific target
    pub async fn get_stats(&self, target: IpAddr) -> Option<CircuitStats> {
        let circuits = self.circuits.read().await;
        circuits.get(&target).map(|c| c.to_stats())
    }

    /// Get statistics for all targets
    pub async fn get_all_stats(&self) -> HashMap<IpAddr, CircuitStats> {
        let circuits = self.circuits.read().await;
        circuits
            .iter()
            .map(|(ip, circuit)| (*ip, circuit.to_stats()))
            .collect()
    }

    /// Reset circuit for a specific target
    pub async fn reset(&self, target: IpAddr) {
        let mut circuits = self.circuits.write().await;
        circuits.remove(&target);
    }

    /// Reset all circuits
    pub async fn reset_all(&self) {
        let mut circuits = self.circuits.write().await;
        circuits.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_starts_closed() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let target: IpAddr = "192.168.1.1".parse().unwrap();

        assert!(breaker.should_attempt(target).await);
    }

    #[tokio::test]
    async fn test_circuit_opens_after_threshold_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        let target: IpAddr = "192.168.1.1".parse().unwrap();

        // Record 3 failures
        breaker.record_failure(target).await;
        breaker.record_failure(target).await;
        breaker.record_failure(target).await;

        // Circuit should be open
        let stats = breaker.get_stats(target).await.unwrap();
        assert_eq!(stats.state, CircuitState::Open);
        assert!(!breaker.should_attempt(target).await);
    }

    #[tokio::test]
    async fn test_circuit_stays_open_during_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_secs(10),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        let target: IpAddr = "192.168.1.1".parse().unwrap();

        // Open circuit
        breaker.record_failure(target).await;
        breaker.record_failure(target).await;

        // Should still be open (timeout not passed)
        assert!(!breaker.should_attempt(target).await);
    }

    #[tokio::test]
    async fn test_circuit_transitions_to_half_open_after_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(50),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        let target: IpAddr = "192.168.1.1".parse().unwrap();

        // Open circuit
        breaker.record_failure(target).await;
        breaker.record_failure(target).await;
        assert!(!breaker.should_attempt(target).await);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should transition to half-open
        assert!(breaker.should_attempt(target).await);
        let stats = breaker.get_stats(target).await.unwrap();
        assert_eq!(stats.state, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_closes_after_success_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(50),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        let target: IpAddr = "192.168.1.1".parse().unwrap();

        // Open circuit
        breaker.record_failure(target).await;
        breaker.record_failure(target).await;

        // Wait for timeout and transition to half-open
        tokio::time::sleep(Duration::from_millis(60)).await;
        breaker.should_attempt(target).await;

        // Record successes
        breaker.record_success(target).await;
        breaker.record_success(target).await;

        // Circuit should be closed
        let stats = breaker.get_stats(target).await.unwrap();
        assert_eq!(stats.state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_reopens_on_failure_in_half_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(50),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        let target: IpAddr = "192.168.1.1".parse().unwrap();

        // Open circuit
        breaker.record_failure(target).await;
        breaker.record_failure(target).await;

        // Wait for timeout and transition to half-open
        tokio::time::sleep(Duration::from_millis(60)).await;
        breaker.should_attempt(target).await;

        // Record failure in half-open state
        breaker.record_failure(target).await;

        // Circuit should be open again
        let stats = breaker.get_stats(target).await.unwrap();
        assert_eq!(stats.state, CircuitState::Open);
        assert!(!breaker.should_attempt(target).await);
    }

    #[tokio::test]
    async fn test_multiple_targets_independently_tracked() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        });
        let target1: IpAddr = "192.168.1.1".parse().unwrap();
        let target2: IpAddr = "192.168.1.2".parse().unwrap();

        // Fail target1
        breaker.record_failure(target1).await;
        breaker.record_failure(target1).await;

        // Target1 should be open, target2 should be closed
        assert!(!breaker.should_attempt(target1).await);
        assert!(breaker.should_attempt(target2).await);
    }

    #[tokio::test]
    async fn test_half_open_limited_requests() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 3, // Need 3 successes to close
            timeout: Duration::from_millis(50),
            half_open_limit: 3,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        let target: IpAddr = "192.168.1.1".parse().unwrap();

        // Open circuit
        breaker.record_failure(target).await;
        breaker.record_failure(target).await;

        // Wait for timeout to transition to half-open
        tokio::time::sleep(Duration::from_millis(60)).await;

        // First should_attempt transitions to half-open
        assert!(breaker.should_attempt(target).await);

        // Record a success (doesn't close yet, need 3)
        breaker.record_success(target).await;

        // Should still allow attempts (limit 3, only used 1)
        assert!(breaker.should_attempt(target).await);
        breaker.record_success(target).await;

        // Should allow one more (2 of 3 used)
        assert!(breaker.should_attempt(target).await);
        breaker.record_success(target).await;

        // Now circuit should be closed (3 successes reached threshold)
        let stats = breaker.get_stats(target).await.unwrap();
        assert_eq!(stats.state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_success_resets_failure_count() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        let target: IpAddr = "192.168.1.1".parse().unwrap();

        // Record 2 failures
        breaker.record_failure(target).await;
        breaker.record_failure(target).await;

        // Record success
        breaker.record_success(target).await;

        // Circuit should still be closed, failure count reset
        let stats = breaker.get_stats(target).await.unwrap();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 0);
    }

    #[tokio::test]
    async fn test_get_all_stats() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let target1: IpAddr = "192.168.1.1".parse().unwrap();
        let target2: IpAddr = "192.168.1.2".parse().unwrap();

        breaker.record_failure(target1).await;
        breaker.record_success(target2).await;

        let all_stats = breaker.get_all_stats().await;
        assert_eq!(all_stats.len(), 2);
        assert!(all_stats.contains_key(&target1));
        assert!(all_stats.contains_key(&target2));
    }

    #[tokio::test]
    async fn test_reset() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        });
        let target: IpAddr = "192.168.1.1".parse().unwrap();

        // Open circuit
        breaker.record_failure(target).await;
        breaker.record_failure(target).await;
        assert!(!breaker.should_attempt(target).await);

        // Reset
        breaker.reset(target).await;

        // Should be closed again
        assert!(breaker.should_attempt(target).await);
    }

    #[tokio::test]
    async fn test_reset_all() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        });
        let target1: IpAddr = "192.168.1.1".parse().unwrap();
        let target2: IpAddr = "192.168.1.2".parse().unwrap();

        breaker.record_failure(target1).await;
        breaker.record_failure(target2).await;

        breaker.reset_all().await;

        let all_stats = breaker.get_all_stats().await;
        assert_eq!(all_stats.len(), 0);
    }
}
