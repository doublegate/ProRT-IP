//! ICMP Error Monitor - Detects Type 3 Code 13 (admin prohibited)
//!
//! Listens for ICMP Destination Unreachable messages and notifies
//! rate limiter to backoff on specific targets.
//!
//! # Algorithm
//!
//! - Background task listens for ICMP Type 3 (Destination Unreachable)
//! - Filters for Code 13 (Communication Administratively Prohibited)
//! - Notifies subscribers via broadcast channel
//! - Per-target exponential backoff (1s → 2s → 4s → 8s → 16s max)
//!
//! # Examples
//!
//! ```no_run
//! use prtip_scanner::IcmpMonitor;
//!
//! # async fn example() -> prtip_core::Result<()> {
//! // Create and start ICMP monitor
//! let monitor = IcmpMonitor::new()?;
//! monitor.start().await?;
//!
//! // Subscribe to error notifications
//! let mut rx = monitor.subscribe();
//!
//! // Process ICMP errors
//! while let Ok(error) = rx.recv().await {
//!     println!("ICMP error from {}: Type {} Code {}",
//!              error.target_ip, error.icmp_type, error.icmp_code);
//! }
//!
//! // Shutdown when done
//! monitor.shutdown().await;
//! # Ok(())
//! # }
//! ```

use pnet::packet::icmp::{IcmpPacket, IcmpTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::{
    icmp_packet_iter, transport_channel, TransportChannelType, TransportProtocol,
};
use prtip_core::{Error, Result};
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tracing::{debug, error, trace, warn};

/// ICMP error notification
#[derive(Debug, Clone)]
pub struct IcmpError {
    /// Target IP address that triggered the ICMP error
    pub target_ip: IpAddr,
    /// ICMP type (should be 3 for Destination Unreachable)
    pub icmp_type: u8,
    /// ICMP code (13 for Communication Administratively Prohibited)
    pub icmp_code: u8,
    /// Timestamp when error was received
    pub timestamp: Instant,
}

/// ICMP monitor - background listener for ICMP errors
///
/// Spawns a background task that listens for ICMP Destination Unreachable
/// messages and broadcasts them to subscribers. Requires raw socket permissions.
///
/// # Thread Safety
///
/// This monitor is fully thread-safe and can be cloned cheaply (uses `Arc` internally).
pub struct IcmpMonitor {
    error_tx: broadcast::Sender<IcmpError>,
    running: Arc<AtomicBool>,
}

impl IcmpMonitor {
    /// Create new ICMP monitor
    ///
    /// # Errors
    ///
    /// Returns error if raw ICMP socket cannot be created (requires elevated privileges).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_scanner::IcmpMonitor;
    ///
    /// let monitor = IcmpMonitor::new()?;
    /// # Ok::<(), prtip_core::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        let (error_tx, _) = broadcast::channel(1000);

        debug!("Created ICMP monitor (channel capacity: 1000)");

        Ok(Self {
            error_tx,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Start ICMP listener (spawns background task)
    ///
    /// Spawns a Tokio task that listens for ICMP packets in the background.
    /// The task runs until `shutdown()` is called.
    ///
    /// # Errors
    ///
    /// - Returns error if monitor is already running
    /// - Raw socket creation errors are logged but don't fail the start
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_scanner::IcmpMonitor;
    /// # async fn example() -> prtip_core::Result<()> {
    /// let monitor = IcmpMonitor::new()?;
    /// monitor.start().await?;
    ///
    /// // ... do work ...
    ///
    /// monitor.shutdown().await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(&self) -> Result<()> {
        if self.running.swap(true, Ordering::Relaxed) {
            return Err(Error::Scanner("ICMP monitor already running".into()));
        }

        debug!("Starting ICMP monitor background task");

        let tx = self.error_tx.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::listener_task(tx, running).await {
                error!("ICMP monitor error: {}", e);
            }
            debug!("ICMP monitor task terminated");
        });

        Ok(())
    }

    /// Subscribe to ICMP error notifications
    ///
    /// Returns a broadcast receiver that will receive all ICMP Type 3 Code 13
    /// errors detected by the monitor.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_scanner::IcmpMonitor;
    /// # async fn example() -> prtip_core::Result<()> {
    /// let monitor = IcmpMonitor::new()?;
    /// monitor.start().await?;
    ///
    /// let mut rx = monitor.subscribe();
    /// while let Ok(error) = rx.recv().await {
    ///     println!("ICMP error: {:?}", error);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe(&self) -> broadcast::Receiver<IcmpError> {
        self.error_tx.subscribe()
    }

    /// Stop ICMP monitor
    ///
    /// Signals the background listener task to stop and waits briefly
    /// for it to terminate gracefully.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_scanner::IcmpMonitor;
    /// # async fn example() -> prtip_core::Result<()> {
    /// let monitor = IcmpMonitor::new()?;
    /// monitor.start().await?;
    ///
    /// // ... work ...
    ///
    /// monitor.shutdown().await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(&self) {
        debug!("Shutting down ICMP monitor");
        self.running.store(false, Ordering::Relaxed);
        tokio::time::sleep(Duration::from_millis(100)).await; // Allow task to stop
    }

    /// Background listener task
    ///
    /// Runs in a spawned Tokio task, listening for ICMP packets and
    /// broadcasting Type 3 Code 13 errors to subscribers.
    ///
    /// Note: This uses blocking I/O on the raw socket. In production, this should
    /// run in a dedicated thread pool or use non-blocking socket APIs.
    async fn listener_task(
        tx: broadcast::Sender<IcmpError>,
        running: Arc<AtomicBool>,
    ) -> Result<()> {
        let protocol =
            TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Icmp));

        let (_sender, mut receiver) = transport_channel(4096, protocol)
            .map_err(|e| Error::Network(format!("Failed to create ICMP socket: {}", e)))?;

        debug!("ICMP monitor listening on raw socket");

        // Run blocking socket operations in dedicated thread
        tokio::task::spawn_blocking(move || {
            #[cfg(unix)]
            let mut iter = icmp_packet_iter(&mut receiver);
            #[cfg(test)]
            let start = Instant::now();

            while running.load(Ordering::Relaxed) {
                // Platform-specific packet receiving with timeout
                // Unix: Uses pnet's next_with_timeout()
                // Windows: Returns None (method doesn't exist, ICMP monitor gracefully degrades)
                #[cfg(unix)]
                let packet_result = iter.next_with_timeout(Duration::from_millis(100));
                #[cfg(windows)]
                let packet_result: std::io::Result<
                    Option<(IcmpPacket<'static>, IpAddr)>,
                > = Ok(None);

                match packet_result {
                    Ok(Some((packet, addr))) => {
                        if Self::is_type_3_code_13(&packet) {
                            let error = IcmpError {
                                target_ip: addr,
                                icmp_type: packet.get_icmp_type().0,
                                icmp_code: packet.get_icmp_code().0,
                                timestamp: Instant::now(),
                            };

                            trace!(
                                "ICMP Type 3 Code 13 from {}: admin prohibited",
                                error.target_ip
                            );

                            let _ = tx.send(error); // Ignore if no subscribers
                        }
                    }
                    Ok(None) => {
                        // Timeout - normal, allows checking running flag
                        #[cfg(windows)]
                        {
                            // On Windows, sleep briefly since we don't have timeouts
                            std::thread::sleep(Duration::from_millis(100));
                        }
                    }
                    Err(e) => {
                        warn!("ICMP receive error: {}", e);
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }

                // Prevent infinite loops in tests - exit after 10s if no running flag changes
                #[cfg(test)]
                if start.elapsed() > Duration::from_secs(10) && !running.load(Ordering::Relaxed) {
                    break;
                }
            }

            debug!("ICMP listener task terminating");
        })
        .await
        .map_err(|e| Error::Network(format!("ICMP listener task failed: {}", e)))?;

        Ok(())
    }

    /// Check if ICMP packet is Type 3 Code 13 (admin prohibited)
    fn is_type_3_code_13(packet: &IcmpPacket) -> bool {
        packet.get_icmp_type() == IcmpTypes::DestinationUnreachable
            && packet.get_icmp_code().0 == 13
    }
}

/// Per-target backoff state
///
/// Tracks exponential backoff for individual target IPs.
/// Escalates through levels 0-4 (1s → 2s → 4s → 8s → 16s max).
#[derive(Debug, Clone)]
pub struct BackoffState {
    /// Expiration time for current backoff
    pub backoff_until: Instant,
    /// Backoff level: 0-4 (2^level seconds: 1s, 2s, 4s, 8s, 16s)
    pub backoff_level: u8,
}

impl BackoffState {
    /// Create new backoff state (level 0, not backed off)
    pub fn new() -> Self {
        Self {
            backoff_until: Instant::now(),
            backoff_level: 0,
        }
    }

    /// Escalate backoff (exponential: 1s → 2s → 4s → 8s → 16s max)
    ///
    /// Each call increases the backoff level and sets a new expiration time.
    /// Maximum level is 4 (16 seconds).
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_scanner::BackoffState;
    ///
    /// let mut state = BackoffState::new();
    /// assert_eq!(state.backoff_level, 0);
    ///
    /// state.escalate(); // Level 1 (2s)
    /// assert_eq!(state.backoff_level, 1);
    ///
    /// state.escalate(); // Level 2 (4s)
    /// assert_eq!(state.backoff_level, 2);
    /// ```
    pub fn escalate(&mut self) {
        self.backoff_level = (self.backoff_level + 1).min(4);
        let backoff_secs = 1 << self.backoff_level; // 2^level
        self.backoff_until = Instant::now() + Duration::from_secs(backoff_secs);

        debug!(
            "Backoff escalated to level {} ({} seconds)",
            self.backoff_level, backoff_secs
        );
    }

    /// Check if backoff expired
    ///
    /// Returns `true` if the current time is past the backoff expiration.
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.backoff_until
    }

    /// Get remaining backoff duration
    ///
    /// Returns `Duration::ZERO` if backoff has expired.
    pub fn remaining(&self) -> Duration {
        self.backoff_until.saturating_duration_since(Instant::now())
    }
}

impl Default for BackoffState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_icmp_monitor_creation() {
        let monitor = IcmpMonitor::new().unwrap();
        assert!(!monitor.running.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_start_stop() {
        let monitor = IcmpMonitor::new().unwrap();
        monitor.start().await.unwrap();
        assert!(monitor.running.load(Ordering::Relaxed));

        monitor.shutdown().await;
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(!monitor.running.load(Ordering::Relaxed));
    }

    #[test]
    fn test_backoff_creation() {
        let state = BackoffState::new();
        assert_eq!(state.backoff_level, 0);
        assert!(state.is_expired());
    }

    #[test]
    fn test_backoff_escalation() {
        let mut state = BackoffState::new();

        state.escalate();
        assert_eq!(state.backoff_level, 1);
        assert!(!state.is_expired());

        // Escalate to max (level 4 = 16s)
        for _ in 0..5 {
            state.escalate();
        }
        assert_eq!(state.backoff_level, 4); // Clamped at max
    }

    #[test]
    fn test_backoff_expiration() {
        let mut state = BackoffState::new();
        state.backoff_until = Instant::now() - Duration::from_secs(1);
        assert!(state.is_expired());

        state.backoff_until = Instant::now() + Duration::from_secs(10);
        assert!(!state.is_expired());
    }

    #[test]
    fn test_backoff_duration_calculation() {
        let mut state = BackoffState::new();

        // Level 0 → 1: 2s
        state.escalate();
        assert_eq!(state.backoff_level, 1);

        // Level 1 → 2: 4s
        state.escalate();
        assert_eq!(state.backoff_level, 2);

        // Level 2 → 3: 8s
        state.escalate();
        assert_eq!(state.backoff_level, 3);

        // Level 3 → 4: 16s (max)
        state.escalate();
        assert_eq!(state.backoff_level, 4);
    }

    #[tokio::test]
    async fn test_subscribe() {
        let monitor = IcmpMonitor::new().unwrap();
        let _rx = monitor.subscribe();
        // Should create receiver successfully
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let monitor = IcmpMonitor::new().unwrap();
        let _rx1 = monitor.subscribe();
        let _rx2 = monitor.subscribe();
        // Both should work independently
    }

    #[test]
    fn test_backoff_remaining_duration() {
        let mut state = BackoffState::new();
        state.backoff_until = Instant::now() + Duration::from_secs(5);

        let remaining = state.remaining();
        assert!(remaining.as_secs() <= 5);
        assert!(remaining.as_secs() >= 4); // Allow some tolerance
    }

    #[tokio::test]
    async fn test_monitor_double_start_error() {
        let monitor = IcmpMonitor::new().unwrap();
        monitor.start().await.unwrap();

        // Second start should error
        let result = monitor.start().await;
        assert!(result.is_err());

        monitor.shutdown().await;
    }

    #[test]
    fn test_backoff_default() {
        let state = BackoffState::default();
        assert_eq!(state.backoff_level, 0);
        assert!(state.is_expired());
    }

    #[test]
    fn test_backoff_levels() {
        let mut state = BackoffState::new();

        // Verify exponential progression
        let expected_levels = [1, 2, 3, 4, 4, 4]; // Caps at 4

        for (i, expected) in expected_levels.iter().enumerate() {
            state.escalate();
            assert_eq!(
                state.backoff_level, *expected,
                "Iteration {}: expected {}, got {}",
                i, expected, state.backoff_level
            );
        }
    }
}
