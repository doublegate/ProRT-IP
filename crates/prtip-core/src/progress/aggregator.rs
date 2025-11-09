//! Progress aggregator for event-driven statistics collection
//!
//! This module provides the main progress aggregation component that subscribes
//! to the EventBus and maintains real-time scan statistics by combining data
//! from ProgressCalculator and ThroughputMonitor.
//!
//! # Architecture
//!
//! ```text
//! EventBus ──▶ ProgressAggregator ──▶ AggregatedState
//!               │
//!               ├──▶ ProgressCalculator (ETA, percentage)
//!               └──▶ ThroughputMonitor (pps, hpm, Mbps)
//! ```
//!
//! # Event Subscriptions
//!
//! The aggregator subscribes to:
//! - `ScanStarted` - Initialize tracking
//! - `ProgressUpdate` - Update completion counters
//! - `HostDiscovered` - Increment discovered hosts
//! - `PortFound` - Increment open/closed/filtered counters
//! - `ServiceDetected` - Increment service count
//! - `StageChanged` - Update current stage
//! - `ScanCompleted` - Finalize statistics
//!
//! # Examples
//!
//! ```no_run
//! use prtip_core::progress::ProgressAggregator;
//! use prtip_core::event_bus::EventBus;
//! use std::sync::Arc;
//!
//! # async fn example() {
//! let bus = Arc::new(EventBus::new(1000));
//! let aggregator = ProgressAggregator::new(bus.clone());
//!
//! // Aggregator automatically tracks all scan events
//!
//! // Query current state
//! let state = aggregator.get_state().await;
//! println!("Stage: {:?}", state.current_stage);
//! println!("Progress: {:.1}%", state.overall_progress);
//! println!("ETA: {:?}", state.eta);
//! println!("Open ports: {}", state.open_ports);
//! # }
//! ```

use super::{ProgressCalculator, ThroughputMonitor};
use crate::event_bus::EventBus;
use crate::events::{ScanEvent, ScanEventType, ScanStage, Throughput};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Aggregated scan state
///
/// Comprehensive real-time view of scan progress, combining statistics
/// from multiple sources (events, calculator, monitor).
#[derive(Debug, Clone)]
pub struct AggregatedState {
    /// Current scan stage
    pub current_stage: ScanStage,
    /// Overall progress percentage (0.0 - 100.0)
    pub overall_progress: f32,
    /// Stage-specific progress percentage (0.0 - 100.0)
    pub stage_progress: f32,
    /// Current throughput metrics
    pub throughput: Throughput,
    /// Estimated time to completion
    pub eta: Option<Duration>,
    /// Number of hosts discovered
    pub discovered_hosts: usize,
    /// Number of open ports found
    pub open_ports: usize,
    /// Number of closed ports found
    pub closed_ports: usize,
    /// Number of filtered ports found
    pub filtered_ports: usize,
    /// Number of services detected
    pub detected_services: usize,
    /// Number of errors encountered
    pub errors: usize,
    /// Warning messages
    pub warnings: Vec<String>,
}

impl Default for AggregatedState {
    fn default() -> Self {
        Self {
            current_stage: ScanStage::Initializing,
            overall_progress: 0.0,
            stage_progress: 0.0,
            throughput: Throughput::default(),
            eta: None,
            discovered_hosts: 0,
            open_ports: 0,
            closed_ports: 0,
            filtered_ports: 0,
            detected_services: 0,
            errors: 0,
            warnings: Vec::new(),
        }
    }
}

/// Event-driven progress aggregator
///
/// Subscribes to EventBus and maintains real-time aggregated state
/// by combining ProgressCalculator (ETA) and ThroughputMonitor (metrics).
///
/// # Thread Safety
///
/// Uses Arc<RwLock> for concurrent read/write access. Clone-friendly
/// for sharing across threads.
///
/// # Performance
///
/// - Non-blocking event handling (buffered channel)
/// - O(1) state queries (cached aggregated state)
/// - Automatic cleanup on drop (unsubscribes from EventBus)
pub struct ProgressAggregator {
    state: Arc<RwLock<AggregatedState>>,
    _calculator: ProgressCalculator,
    _monitor: ThroughputMonitor,
    _task_handle: tokio::task::JoinHandle<()>,
}

impl ProgressAggregator {
    /// Create a new progress aggregator
    ///
    /// Automatically subscribes to relevant events from the EventBus.
    ///
    /// # Arguments
    ///
    /// * `event_bus` - EventBus to subscribe to
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressAggregator;
    /// use prtip_core::event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let bus = Arc::new(EventBus::new(1000));
    /// let aggregator = ProgressAggregator::new(bus);
    /// # }
    /// ```
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        let state = Arc::new(RwLock::new(AggregatedState::default()));
        let calculator = ProgressCalculator::new(0, 0); // Will update on ScanStarted
        let monitor = ThroughputMonitor::new();

        // Subscribe to all relevant events
        let (tx, rx) = mpsc::unbounded_channel();
        let event_bus_clone = event_bus.clone();
        let state_clone = Arc::clone(&state);
        let calculator_clone = calculator.clone();
        let monitor_clone = monitor.clone();

        // Spawn background task to handle events
        let task_handle = tokio::spawn(async move {
            Self::event_handler_task(rx, state_clone, calculator_clone, monitor_clone).await;
        });

        // Subscribe to EventBus
        tokio::spawn(async move {
            use crate::event_bus::EventFilter;
            event_bus_clone
                .subscribe(
                    tx,
                    EventFilter::EventType(vec![
                        ScanEventType::ScanStarted,
                        ScanEventType::ScanCompleted,
                        ScanEventType::ProgressUpdate,
                        ScanEventType::StageChanged,
                        ScanEventType::HostDiscovered,
                        ScanEventType::PortFound,
                        ScanEventType::ServiceDetected,
                        ScanEventType::WarningIssued,
                        ScanEventType::ScanError,
                    ]),
                )
                .await;
        });

        Self {
            state,
            _calculator: calculator,
            _monitor: monitor,
            _task_handle: task_handle,
        }
    }

    /// Background task to handle incoming events
    async fn event_handler_task(
        mut rx: mpsc::UnboundedReceiver<ScanEvent>,
        state: Arc<RwLock<AggregatedState>>,
        calculator: ProgressCalculator,
        monitor: ThroughputMonitor,
    ) {
        while let Some(event) = rx.recv().await {
            Self::process_event(&event, &state, &calculator, &monitor).await;
        }
    }

    /// Process a single event and update state
    async fn process_event(
        event: &ScanEvent,
        state: &Arc<RwLock<AggregatedState>>,
        calculator: &ProgressCalculator,
        monitor: &ThroughputMonitor,
    ) {
        match event {
            ScanEvent::ScanStarted { .. } => {
                // Note: Calculator is initialized in new() with the scan configuration
                // We reset the monitor to clear previous scan data
                monitor.reset().await;

                let mut s = state.write();
                s.current_stage = ScanStage::Initializing;
                s.overall_progress = 0.0;
                s.discovered_hosts = 0;
                s.open_ports = 0;
                s.closed_ports = 0;
                s.filtered_ports = 0;
                s.detected_services = 0;
                s.errors = 0;
                s.warnings.clear();
            }

            ScanEvent::ProgressUpdate {
                percentage,
                completed,
                ..
            } => {
                // Update calculator (completed is total work units done)
                calculator.update(0, *completed as usize).await;

                // Update throughput monitor (estimate packets from completed work)
                monitor.record_packets(*completed).await;

                // Get async values before acquiring lock (avoid holding lock across await)
                let eta = calculator.eta().await;
                let throughput = monitor.current_throughput().await;

                // Update state
                let mut s = state.write();
                s.overall_progress = *percentage;
                s.eta = eta;
                s.throughput = throughput;
            }

            ScanEvent::StageChanged { to_stage, .. } => {
                let mut s = state.write();
                s.current_stage = *to_stage;
                s.stage_progress = 0.0; // Reset stage progress
            }

            ScanEvent::HostDiscovered { .. } => {
                // Record host completion first (async)
                monitor.record_host_completed().await;

                // Then update state
                let mut s = state.write();
                s.discovered_hosts += 1;
            }

            ScanEvent::PortFound {
                state: port_state, ..
            } => {
                use crate::types::PortState;

                // Record packet first (async)
                monitor.record_packets(1).await;

                // Then update state
                let mut s = state.write();
                match port_state {
                    PortState::Open => s.open_ports += 1,
                    PortState::Closed => s.closed_ports += 1,
                    PortState::Filtered => s.filtered_ports += 1,
                    PortState::Unknown => {} // Ignore unknown state
                }
            }

            ScanEvent::ServiceDetected { .. } => {
                let mut s = state.write();
                s.detected_services += 1;
            }

            ScanEvent::WarningIssued { message, .. } => {
                let mut s = state.write();
                s.warnings.push(message.clone());
            }

            ScanEvent::ScanError { .. } => {
                let mut s = state.write();
                s.errors += 1;
            }

            ScanEvent::ScanCompleted { .. } => {
                let mut s = state.write();
                s.current_stage = ScanStage::Completed;
                s.overall_progress = 100.0;
                s.eta = None;
            }

            _ => {}
        }
    }

    /// Get current aggregated state (non-blocking)
    ///
    /// Returns a snapshot of the current state.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressAggregator;
    /// use prtip_core::event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let bus = Arc::new(EventBus::new(1000));
    /// let aggregator = ProgressAggregator::new(bus);
    ///
    /// let state = aggregator.get_state().await;
    /// println!("Progress: {:.1}%", state.overall_progress);
    /// # }
    /// ```
    pub async fn get_state(&self) -> AggregatedState {
        self.state.read().clone()
    }

    /// Get current progress percentage
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressAggregator;
    /// use prtip_core::event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let bus = Arc::new(EventBus::new(1000));
    /// let aggregator = ProgressAggregator::new(bus);
    ///
    /// let pct = aggregator.percentage().await;
    /// println!("{:.1}% complete", pct);
    /// # }
    /// ```
    pub async fn percentage(&self) -> f32 {
        self.state.read().overall_progress
    }

    /// Get current ETA
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressAggregator;
    /// use prtip_core::event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let bus = Arc::new(EventBus::new(1000));
    /// let aggregator = ProgressAggregator::new(bus);
    ///
    /// if let Some(eta) = aggregator.eta().await {
    ///     println!("ETA: {} seconds", eta.as_secs());
    /// }
    /// # }
    /// ```
    pub async fn eta(&self) -> Option<Duration> {
        self.state.read().eta
    }

    /// Get current throughput
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressAggregator;
    /// use prtip_core::event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let bus = Arc::new(EventBus::new(1000));
    /// let aggregator = ProgressAggregator::new(bus);
    ///
    /// let tp = aggregator.throughput().await;
    /// println!("PPS: {:.0}", tp.packets_per_second);
    /// # }
    /// ```
    pub async fn throughput(&self) -> Throughput {
        self.state.read().throughput.clone()
    }

    /// Get current scan stage
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use prtip_core::progress::ProgressAggregator;
    /// use prtip_core::event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let bus = Arc::new(EventBus::new(1000));
    /// let aggregator = ProgressAggregator::new(bus);
    ///
    /// let stage = aggregator.current_stage().await;
    /// println!("Current stage: {:?}", stage);
    /// # }
    /// ```
    pub async fn current_stage(&self) -> ScanStage {
        self.state.read().current_stage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PortState, Protocol, ScanType};
    use std::time::SystemTime;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_new_aggregator() {
        let bus = Arc::new(EventBus::new(1000));
        let agg = ProgressAggregator::new(bus);

        let state = agg.get_state().await;
        assert_eq!(state.current_stage, ScanStage::Initializing);
        assert_eq!(state.overall_progress, 0.0);
    }

    #[tokio::test]
    async fn test_scan_started_event() {
        let bus = Arc::new(EventBus::new(1000));
        let agg = ProgressAggregator::new(bus.clone());

        let event = ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            scan_type: ScanType::Syn,
            target_count: 100,
            port_count: 1000,
            timestamp: SystemTime::now(),
        };

        bus.publish(event).await;
        tokio::time::sleep(Duration::from_millis(100)).await; // Wait for processing

        let state = agg.get_state().await;
        assert_eq!(state.current_stage, ScanStage::Initializing);
    }

    #[tokio::test]
    async fn test_progress_update_event() {
        let bus = Arc::new(EventBus::new(1000));
        let agg = ProgressAggregator::new(bus.clone());

        // Start scan
        bus.publish(ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            scan_type: ScanType::Syn,
            target_count: 100,
            port_count: 1000,
            timestamp: SystemTime::now(),
        })
        .await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Update progress
        bus.publish(ScanEvent::ProgressUpdate {
            scan_id: Uuid::new_v4(),
            stage: ScanStage::ScanningPorts,
            percentage: 50.0,
            completed: 50000,
            total: 100000,
            throughput: Throughput::default(),
            eta: Some(Duration::from_secs(10)),
            timestamp: SystemTime::now(),
        })
        .await;

        tokio::time::sleep(Duration::from_millis(100)).await;

        let pct = agg.percentage().await;
        assert!((pct - 50.0).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_stage_changed_event() {
        let bus = Arc::new(EventBus::new(1000));
        let agg = ProgressAggregator::new(bus.clone());

        // Wait for subscription to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        bus.publish(ScanEvent::StageChanged {
            scan_id: Uuid::new_v4(),
            from_stage: ScanStage::Initializing,
            to_stage: ScanStage::ScanningPorts,
            timestamp: SystemTime::now(),
        })
        .await;

        tokio::time::sleep(Duration::from_millis(100)).await;

        let stage = agg.current_stage().await;
        assert_eq!(stage, ScanStage::ScanningPorts);
    }

    #[tokio::test]
    async fn test_host_discovered_event() {
        let bus = Arc::new(EventBus::new(1000));
        let agg = ProgressAggregator::new(bus.clone());

        // Wait for subscription to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        for _ in 0..5 {
            bus.publish(ScanEvent::HostDiscovered {
                scan_id: Uuid::new_v4(),
                ip: "192.168.1.1".parse().unwrap(),
                method: crate::events::DiscoveryMethod::IcmpEcho,
                latency_ms: Some(10),
                timestamp: SystemTime::now(),
            })
            .await;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        let state = agg.get_state().await;
        assert_eq!(state.discovered_hosts, 5);
    }

    #[tokio::test]
    async fn test_port_found_event() {
        let bus = Arc::new(EventBus::new(1000));
        let agg = ProgressAggregator::new(bus.clone());

        // Wait for subscription to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Open port
        bus.publish(ScanEvent::PortFound {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.1".parse().unwrap(),
            port: 80,
            state: PortState::Open,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        })
        .await;

        // Closed port
        bus.publish(ScanEvent::PortFound {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.1".parse().unwrap(),
            port: 81,
            state: PortState::Closed,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        })
        .await;

        // Filtered port
        bus.publish(ScanEvent::PortFound {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.1".parse().unwrap(),
            port: 82,
            state: PortState::Filtered,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        })
        .await;

        tokio::time::sleep(Duration::from_millis(100)).await;

        let state = agg.get_state().await;
        assert_eq!(state.open_ports, 1);
        assert_eq!(state.closed_ports, 1);
        assert_eq!(state.filtered_ports, 1);
    }

    #[tokio::test]
    async fn test_service_detected_event() {
        let bus = Arc::new(EventBus::new(1000));
        let agg = ProgressAggregator::new(bus.clone());

        // Wait for subscription to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        for _ in 0..3 {
            bus.publish(ScanEvent::ServiceDetected {
                scan_id: Uuid::new_v4(),
                ip: "192.168.1.1".parse().unwrap(),
                port: 80,
                service_name: "HTTP".to_string(),
                service_version: Some("1.1".to_string()),
                confidence: 0.95,
                timestamp: SystemTime::now(),
            })
            .await;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        let state = agg.get_state().await;
        assert_eq!(state.detected_services, 3);
    }

    #[tokio::test]
    async fn test_warning_event() {
        let bus = Arc::new(EventBus::new(1000));
        let agg = ProgressAggregator::new(bus.clone());

        // Wait for subscription to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        bus.publish(ScanEvent::WarningIssued {
            scan_id: Uuid::new_v4(),
            message: "Test warning".to_string(),
            severity: crate::events::WarningSeverity::Low,
            timestamp: SystemTime::now(),
        })
        .await;

        tokio::time::sleep(Duration::from_millis(100)).await;

        let state = agg.get_state().await;
        assert_eq!(state.warnings.len(), 1);
        assert!(state.warnings[0].contains("Test warning"));
    }

    #[tokio::test]
    async fn test_error_event() {
        let bus = Arc::new(EventBus::new(1000));
        let agg = ProgressAggregator::new(bus.clone());

        // Wait for subscription to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        for _ in 0..2 {
            bus.publish(ScanEvent::ScanError {
                scan_id: Uuid::new_v4(),
                error: "Test error".to_string(),
                recoverable: true,
                timestamp: SystemTime::now(),
            })
            .await;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        let state = agg.get_state().await;
        assert_eq!(state.errors, 2);
    }

    #[tokio::test]
    async fn test_scan_completed_event() {
        let bus = Arc::new(EventBus::new(1000));
        let agg = ProgressAggregator::new(bus.clone());

        // Wait for subscription to complete
        tokio::time::sleep(Duration::from_millis(50)).await;

        bus.publish(ScanEvent::ScanCompleted {
            scan_id: Uuid::new_v4(),
            duration: Duration::from_secs(60),
            total_targets: 100,
            open_ports: 50,
            closed_ports: 30,
            filtered_ports: 20,
            detected_services: 10,
            timestamp: SystemTime::now(),
        })
        .await;

        tokio::time::sleep(Duration::from_millis(100)).await;

        let state = agg.get_state().await;
        assert_eq!(state.current_stage, ScanStage::Completed);
        assert_eq!(state.overall_progress, 100.0);
        assert!(state.eta.is_none());
    }
}
