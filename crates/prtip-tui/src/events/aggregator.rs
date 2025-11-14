//! Event aggregation for high-frequency event streams
//!
//! This module provides event rate limiting to prevent UI overload from
//! high-frequency EventBus events (e.g., 10K+ PortFound events/second).
//!
//! Strategy:
//! - Aggregate similar events (100 PortFound → single batch update)
//! - Sample high-frequency events (max 1 per 16ms = 60 FPS)
//! - Drop events if buffer exceeds threshold (prevent memory growth)

use prtip_core::events::ScanEvent;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

/// Maximum events to buffer before dropping
const MAX_BUFFER_SIZE: usize = 1000;

/// Minimum time between event processing (60 FPS = 16.67ms)
const MIN_EVENT_INTERVAL: Duration = Duration::from_millis(16);

/// Aggregated event statistics
#[derive(Debug, Default)]
pub struct EventStats {
    /// Number of PortFound events since last flush
    pub ports_found: usize,

    /// Number of HostDiscovered events since last flush
    pub hosts_discovered: usize,

    /// Number of ServiceDetected events since last flush
    pub services_detected: usize,

    /// Unique IPs discovered (for deduplication)
    pub discovered_ips: HashMap<IpAddr, usize>,

    /// Total events processed
    pub total_events: usize,

    /// Events dropped due to rate limiting
    pub dropped_events: usize,
}

/// Event aggregator for rate limiting
pub struct EventAggregator {
    /// Aggregated statistics
    stats: EventStats,

    /// Last time events were flushed
    last_flush: Instant,

    /// Buffer for non-aggregatable events
    event_buffer: Vec<ScanEvent>,
}

impl EventAggregator {
    /// Create a new event aggregator
    pub fn new() -> Self {
        Self {
            stats: EventStats::default(),
            // Initialize to past time to allow immediate first flush
            last_flush: Instant::now() - MIN_EVENT_INTERVAL,
            event_buffer: Vec::with_capacity(MAX_BUFFER_SIZE),
        }
    }

    /// Add an event to the aggregator
    ///
    /// Returns true if the event was accepted, false if dropped due to rate limiting
    pub fn add_event(&mut self, event: ScanEvent) -> bool {
        // Check buffer size limit
        if self.event_buffer.len() >= MAX_BUFFER_SIZE {
            self.stats.dropped_events += 1;
            return false;
        }

        self.stats.total_events += 1;

        // Aggregate high-frequency events
        match &event {
            ScanEvent::PortFound { .. } => {
                self.stats.ports_found += 1;
                // Don't buffer individual port events - we'll batch them
                true
            }

            ScanEvent::HostDiscovered { ip, .. } => {
                *self.stats.discovered_ips.entry(*ip).or_insert(0) += 1;
                self.stats.hosts_discovered += 1;
                true
            }

            ScanEvent::ServiceDetected { .. } => {
                self.stats.services_detected += 1;
                true
            }

            // For important events (lifecycle, errors, warnings), buffer them
            ScanEvent::ScanStarted { .. }
            | ScanEvent::ScanCompleted { .. }
            | ScanEvent::ScanError { .. }
            | ScanEvent::WarningIssued { .. }
            | ScanEvent::ProgressUpdate { .. }
            | ScanEvent::StageChanged { .. } => {
                self.event_buffer.push(event);
                true
            }

            // For other events, buffer them
            _ => {
                self.event_buffer.push(event);
                true
            }
        }
    }

    /// Check if events should be flushed (based on time interval)
    pub fn should_flush(&self) -> bool {
        self.last_flush.elapsed() >= MIN_EVENT_INTERVAL
    }

    /// Flush aggregated events and return them for processing
    ///
    /// This should be called every 16ms (60 FPS) to maintain smooth UI updates
    pub fn flush(&mut self) -> (Vec<ScanEvent>, EventStats) {
        self.last_flush = Instant::now();

        // Take buffered events
        let events = std::mem::take(&mut self.event_buffer);

        // Take statistics
        let stats = std::mem::take(&mut self.stats);

        (events, stats)
    }

    /// Get current statistics (without flushing)
    pub fn stats(&self) -> &EventStats {
        &self.stats
    }
}

impl Default for EventAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prtip_core::events::{ScanStage, Throughput};
    use prtip_core::{DiscoveryMethod, ScanType};
    use std::net::IpAddr;
    use std::time::SystemTime;
    use uuid::Uuid;

    #[test]
    fn test_aggregator_basic() {
        let mut agg = EventAggregator::new();

        // Add some port found events
        for _ in 0..100 {
            let accepted = agg.add_event(ScanEvent::PortFound {
                scan_id: Uuid::new_v4(),
                ip: "192.168.1.1".parse().unwrap(),
                port: 80,
                protocol: prtip_core::types::Protocol::Tcp,
                state: prtip_core::types::PortState::Open,
                scan_type: ScanType::Syn,
                timestamp: SystemTime::now(),
            });
            assert!(accepted);
        }

        assert_eq!(agg.stats().ports_found, 100);
        assert_eq!(agg.stats().total_events, 100);
    }

    #[test]
    fn test_aggregator_host_discovery() {
        let mut agg = EventAggregator::new();

        // Add host discovered events
        let ips: Vec<IpAddr> = vec![
            "192.168.1.1".parse().unwrap(),
            "192.168.1.2".parse().unwrap(),
            "192.168.1.1".parse().unwrap(), // Duplicate
        ];

        for ip in ips {
            agg.add_event(ScanEvent::HostDiscovered {
                scan_id: Uuid::new_v4(),
                ip,
                method: DiscoveryMethod::IcmpEcho,
                latency_ms: Some(10),
                timestamp: SystemTime::now(),
            });
        }

        assert_eq!(agg.stats().hosts_discovered, 3);
        assert_eq!(agg.stats().discovered_ips.len(), 2); // Only 2 unique IPs
    }

    #[test]
    fn test_aggregator_buffer_limit() {
        let mut agg = EventAggregator::new();

        // Fill buffer with lifecycle events
        for i in 0..MAX_BUFFER_SIZE {
            let accepted = agg.add_event(ScanEvent::ProgressUpdate {
                scan_id: Uuid::new_v4(),
                stage: ScanStage::ScanningPorts,
                percentage: i as f32,
                completed: i as u64,
                total: MAX_BUFFER_SIZE as u64,
                throughput: Throughput {
                    packets_per_second: 1000.0,
                    hosts_per_minute: 100.0,
                    bandwidth_mbps: 10.0,
                },
                eta: None,
                timestamp: SystemTime::now(),
            });
            assert!(accepted);
        }

        // Next event should be dropped
        let accepted = agg.add_event(ScanEvent::ProgressUpdate {
            scan_id: Uuid::new_v4(),
            stage: ScanStage::ScanningPorts,
            percentage: 100.0,
            completed: MAX_BUFFER_SIZE as u64,
            total: MAX_BUFFER_SIZE as u64,
            throughput: Throughput {
                packets_per_second: 1000.0,
                hosts_per_minute: 100.0,
                bandwidth_mbps: 10.0,
            },
            eta: None,
            timestamp: SystemTime::now(),
        });
        assert!(!accepted);
        assert_eq!(agg.stats().dropped_events, 1);
    }

    #[test]
    fn test_aggregator_flush() {
        let mut agg = EventAggregator::new();

        // Add events
        for _ in 0..50 {
            agg.add_event(ScanEvent::PortFound {
                scan_id: Uuid::new_v4(),
                ip: "192.168.1.1".parse().unwrap(),
                port: 80,
                protocol: prtip_core::types::Protocol::Tcp,
                state: prtip_core::types::PortState::Open,
                scan_type: ScanType::Syn,
                timestamp: SystemTime::now(),
            });
        }

        // Add a lifecycle event
        agg.add_event(ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            target_count: 1,
            port_count: 1000,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        });

        // Flush
        let (events, stats) = agg.flush();

        // Check stats
        assert_eq!(stats.ports_found, 50);
        assert_eq!(stats.total_events, 51);

        // Check events (only lifecycle event buffered)
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], ScanEvent::ScanStarted { .. }));

        // Stats should be reset
        assert_eq!(agg.stats().ports_found, 0);
        assert_eq!(agg.stats().total_events, 0);
    }
}
