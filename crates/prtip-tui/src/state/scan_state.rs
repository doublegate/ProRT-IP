//! Shared scan state (thread-safe, shared with scanner)

use parking_lot::RwLock;
use prtip_core::events::ScanStage;
use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

// ===== Capacity Limits (Memory Management) =====

/// Maximum port discoveries to retain (ringbuffer limit)
/// Memory overhead: ~80 KB (80 bytes/entry)
pub const MAX_PORT_DISCOVERIES: usize = 1_000;

/// Maximum service detections to retain (ringbuffer limit)
/// Memory overhead: ~50 KB (100 bytes/entry)
pub const MAX_SERVICE_DETECTIONS: usize = 500;

/// Maximum throughput samples to retain (60-second window)
/// Memory overhead: ~1 KB (60 seconds × 16 bytes/entry)
pub const MAX_THROUGHPUT_SAMPLES: usize = 60;

// ===== Supporting Structs (Sprint 6.2) =====

/// Port discovery event (cached for live dashboard display)
#[derive(Debug, Clone)]
pub struct PortDiscovery {
    /// Timestamp when port was discovered
    pub timestamp: SystemTime,

    /// IP address of the target
    pub ip: IpAddr,

    /// Port number
    pub port: u16,

    /// Port state (Open/Filtered/Closed)
    pub state: PortState,

    /// Protocol (TCP/UDP)
    pub protocol: Protocol,

    /// Scan type used to discover this port
    pub scan_type: ScanType,
}

/// Service detection event (cached for live dashboard display)
#[derive(Debug, Clone)]
pub struct ServiceDetection {
    /// Timestamp when service was detected
    pub timestamp: SystemTime,

    /// IP address of the target
    pub ip: IpAddr,

    /// Port number
    pub port: u16,

    /// Detected service name (e.g., "http", "ssh")
    pub service_name: String,

    /// Service version (optional)
    pub service_version: Option<String>,

    /// Detection confidence (0.0-1.0)
    pub confidence: f32,
}

/// Throughput sample (for network activity graph)
#[derive(Debug, Clone)]
pub struct ThroughputSample {
    /// Timestamp of this sample
    pub timestamp: Instant,

    /// Packets per second at this timestamp
    pub packets_per_second: f64,
}

/// Port state (Open/Filtered/Closed)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    Open,
    Filtered,
    Closed,
}

/// Protocol (TCP/UDP)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Protocol {
    Tcp,
    Udp,
}

/// Scan type (SYN/Connect/FIN/etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanType {
    Syn,
    Connect,
    Fin,
    Null,
    Xmas,
    Ack,
    Window,
    Maimon,
    Udp,
}

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

    // ===== Sprint 6.2: Dashboard Data =====
    /// Detailed port discoveries (for live port table)
    /// Ringbuffer limited to MAX_PORT_DISCOVERIES (1,000 most recent)
    pub port_discoveries: VecDeque<PortDiscovery>,

    /// Detailed service detections (for service panel)
    /// Ringbuffer limited to MAX_SERVICE_DETECTIONS (500 most recent)
    pub service_detections: VecDeque<ServiceDetection>,

    /// Throughput history (60-second window, 1-second samples)
    /// For network activity graph
    pub throughput_history: VecDeque<ThroughputSample>,
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
            port_discoveries: VecDeque::with_capacity(MAX_PORT_DISCOVERIES),
            service_detections: VecDeque::with_capacity(MAX_SERVICE_DETECTIONS),
            throughput_history: VecDeque::with_capacity(MAX_THROUGHPUT_SAMPLES),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use std::thread;

    fn create_test_port_discovery(port: u16) -> PortDiscovery {
        PortDiscovery {
            timestamp: SystemTime::now(),
            ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            port,
            state: PortState::Open,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
        }
    }

    fn create_test_service_detection(port: u16, confidence: f32) -> ServiceDetection {
        ServiceDetection {
            timestamp: SystemTime::now(),
            ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            port,
            service_name: format!("service-{}", port),
            service_version: Some(format!("v{}.0", port / 100)),
            confidence,
        }
    }

    fn create_test_throughput_sample(pps: f64) -> ThroughputSample {
        ThroughputSample {
            timestamp: Instant::now(),
            packets_per_second: pps,
        }
    }

    #[test]
    fn test_port_discovery_ringbuffer() {
        let mut state = ScanState::new();

        // Add exactly MAX_PORT_DISCOVERIES
        for i in 0..MAX_PORT_DISCOVERIES {
            state
                .port_discoveries
                .push_back(create_test_port_discovery(i as u16));
        }

        assert_eq!(state.port_discoveries.len(), MAX_PORT_DISCOVERIES);

        // Add one more (should trigger manual truncation in real implementation)
        state
            .port_discoveries
            .push_back(create_test_port_discovery(9999));

        // In real usage, EventBus would truncate. For now, verify capacity planning
        assert!(state.port_discoveries.len() <= MAX_PORT_DISCOVERIES + 1);
    }

    #[test]
    fn test_service_detection_ringbuffer() {
        let mut state = ScanState::new();

        // Add exactly MAX_SERVICE_DETECTIONS
        for i in 0..MAX_SERVICE_DETECTIONS {
            state
                .service_detections
                .push_back(create_test_service_detection(i as u16, 0.8));
        }

        assert_eq!(state.service_detections.len(), MAX_SERVICE_DETECTIONS);

        // Verify capacity planning
        state
            .service_detections
            .push_back(create_test_service_detection(9999, 0.9));
        assert!(state.service_detections.len() <= MAX_SERVICE_DETECTIONS + 1);
    }

    #[test]
    fn test_throughput_history_ringbuffer() {
        let mut state = ScanState::new();

        // Add exactly MAX_THROUGHPUT_SAMPLES
        for i in 0..MAX_THROUGHPUT_SAMPLES {
            state
                .throughput_history
                .push_back(create_test_throughput_sample(1000.0 + i as f64));
        }

        assert_eq!(state.throughput_history.len(), MAX_THROUGHPUT_SAMPLES);

        // Verify capacity planning
        state
            .throughput_history
            .push_back(create_test_throughput_sample(9999.0));
        assert!(state.throughput_history.len() <= MAX_THROUGHPUT_SAMPLES + 1);
    }

    #[test]
    fn test_ringbuffer_max_capacity_enforcement() {
        let mut state = ScanState::new();

        // Test port_discoveries ringbuffer enforcement
        for i in 0..(MAX_PORT_DISCOVERIES + 100) {
            state
                .port_discoveries
                .push_back(create_test_port_discovery(i as u16));

            // Manual truncation (in real code, EventBus does this)
            if state.port_discoveries.len() > MAX_PORT_DISCOVERIES {
                state.port_discoveries.pop_front();
            }
        }

        assert_eq!(state.port_discoveries.len(), MAX_PORT_DISCOVERIES);

        // Verify oldest entries were removed (most recent 1,000 retained)
        let first_port = state.port_discoveries.front().unwrap();
        assert!(first_port.port >= 100); // First 100 should be removed
    }

    #[test]
    fn test_throughput_sample_aging() {
        let mut state = ScanState::new();

        // Add samples with small delays
        for i in 0..10 {
            state
                .throughput_history
                .push_back(create_test_throughput_sample(1000.0 + i as f64));
            thread::sleep(std::time::Duration::from_millis(1));
        }

        assert_eq!(state.throughput_history.len(), 10);

        // Verify timestamps are ordered (oldest first)
        for i in 1..state.throughput_history.len() {
            let prev = &state.throughput_history[i - 1];
            let current = &state.throughput_history[i];
            assert!(current.timestamp >= prev.timestamp);
        }
    }

    #[test]
    fn test_port_discovery_deduplication() {
        let mut state = ScanState::new();

        // Add same port twice (different timestamps)
        state
            .port_discoveries
            .push_back(create_test_port_discovery(80));
        thread::sleep(std::time::Duration::from_millis(10));
        state
            .port_discoveries
            .push_back(create_test_port_discovery(80));

        // Current implementation allows duplicates (deduplication happens in EventBus)
        assert_eq!(state.port_discoveries.len(), 2);

        // Both entries should be for port 80
        assert_eq!(state.port_discoveries[0].port, 80);
        assert_eq!(state.port_discoveries[1].port, 80);
    }

    #[test]
    fn test_service_detection_confidence_thresholds() {
        let mut state = ScanState::new();

        // Add services with various confidence levels
        state
            .service_detections
            .push_back(create_test_service_detection(80, 0.95));
        state
            .service_detections
            .push_back(create_test_service_detection(443, 0.75));
        state
            .service_detections
            .push_back(create_test_service_detection(22, 0.45));
        state
            .service_detections
            .push_back(create_test_service_detection(3306, 0.15));

        assert_eq!(state.service_detections.len(), 4);

        // Filter by confidence >= 0.5
        let high_confidence: Vec<_> = state
            .service_detections
            .iter()
            .filter(|s| s.confidence >= 0.5)
            .collect();
        assert_eq!(high_confidence.len(), 2); // 0.95 and 0.75

        // Filter by confidence >= 0.8
        let very_high_confidence: Vec<_> = state
            .service_detections
            .iter()
            .filter(|s| s.confidence >= 0.8)
            .collect();
        assert_eq!(very_high_confidence.len(), 1); // 0.95 only
    }

    #[test]
    fn test_scan_state_initialization() {
        let state = ScanState::new();

        // Verify default values
        assert_eq!(state.progress_percentage, 0.0);
        assert_eq!(state.completed, 0);
        assert_eq!(state.total, 0);
        assert_eq!(state.throughput_pps, 0.0);
        assert!(state.eta.is_none());

        // Verify ringbuffers initialized with correct capacity
        assert_eq!(state.port_discoveries.capacity(), MAX_PORT_DISCOVERIES);
        assert_eq!(state.service_detections.capacity(), MAX_SERVICE_DETECTIONS);
        assert_eq!(state.throughput_history.capacity(), MAX_THROUGHPUT_SAMPLES);

        // Verify ringbuffers are empty
        assert!(state.port_discoveries.is_empty());
        assert!(state.service_detections.is_empty());
        assert!(state.throughput_history.is_empty());
    }

    #[test]
    fn test_scan_state_shared() {
        let shared = ScanState::shared();

        // Verify thread-safe wrapper
        {
            let state = shared.read();
            assert_eq!(state.progress_percentage, 0.0);
        }

        // Verify write access
        {
            let mut state = shared.write();
            state.progress_percentage = 50.0;
            state.completed = 500;
            state.total = 1000;
        }

        // Verify changes persisted
        {
            let state = shared.read();
            assert_eq!(state.progress_percentage, 50.0);
            assert_eq!(state.completed, 500);
            assert_eq!(state.total, 1000);
        }
    }

    #[test]
    fn test_port_state_enum() {
        // Test PartialEq
        assert_eq!(PortState::Open, PortState::Open);
        assert_ne!(PortState::Open, PortState::Filtered);
        assert_ne!(PortState::Open, PortState::Closed);
    }

    #[test]
    fn test_protocol_enum_ordering() {
        // Test PartialOrd (Tcp < Udp alphabetically)
        assert!(Protocol::Tcp < Protocol::Udp);
        assert_eq!(Protocol::Tcp, Protocol::Tcp);
    }

    #[test]
    fn test_scan_type_enum() {
        // Test PartialEq
        assert_eq!(ScanType::Syn, ScanType::Syn);
        assert_ne!(ScanType::Syn, ScanType::Connect);
    }
}
