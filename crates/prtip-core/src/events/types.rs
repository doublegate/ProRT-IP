//! Event types for scan lifecycle tracking
//!
//! This module defines all events emitted during scanning operations,
//! enabling real-time monitoring, progress tracking, and TUI integration.

use crate::types::{PortState, Protocol, ScanType};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Main scan event enum covering all scan lifecycle events
///
/// All events include `scan_id` for correlation and `timestamp` for ordering.
/// Events are `Clone + Send + Sync` for multi-threaded event dispatch.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScanEvent {
    // ===== Lifecycle Events (6 types) =====
    /// Scan has started
    ///
    /// Emitted once at the beginning of a scan, includes configuration
    /// and target count for consumers to initialize.
    ScanStarted {
        scan_id: Uuid,
        scan_type: ScanType,
        target_count: usize,
        port_count: usize,
        timestamp: SystemTime,
    },

    /// Scan completed successfully
    ///
    /// Emitted once at the end of a successful scan with summary statistics.
    ScanCompleted {
        scan_id: Uuid,
        duration: Duration,
        total_targets: usize,
        open_ports: usize,
        closed_ports: usize,
        filtered_ports: usize,
        detected_services: usize,
        timestamp: SystemTime,
    },

    /// Scan paused (user requested or automatic)
    ///
    /// Emitted when a scan is temporarily halted.
    ScanPaused {
        scan_id: Uuid,
        reason: PauseReason,
        timestamp: SystemTime,
    },

    /// Scan resumed from paused state
    ///
    /// Emitted when a paused scan continues.
    ScanResumed {
        scan_id: Uuid,
        timestamp: SystemTime,
    },

    /// Scan cancelled before completion
    ///
    /// Emitted when user stops scan early (Ctrl+C or API cancellation).
    ScanCancelled {
        scan_id: Uuid,
        reason: String,
        partial_results: bool,
        timestamp: SystemTime,
    },

    /// Fatal scan error occurred
    ///
    /// Emitted when an unrecoverable error stops the scan.
    ScanError {
        scan_id: Uuid,
        error: String,
        recoverable: bool,
        timestamp: SystemTime,
    },

    // ===== Progress Events (2 types) =====
    /// Progress update for current stage
    ///
    /// Emitted periodically (typically every 5% completion) with
    /// current percentage, throughput, and ETA.
    ProgressUpdate {
        scan_id: Uuid,
        stage: ScanStage,
        percentage: f32,
        completed: u64,
        total: u64,
        throughput: Throughput,
        eta: Option<Duration>,
        timestamp: SystemTime,
    },

    /// Scan stage changed
    ///
    /// Emitted when transitioning between scan stages
    /// (e.g., Discovery → Scanning → Detection).
    StageChanged {
        scan_id: Uuid,
        from_stage: ScanStage,
        to_stage: ScanStage,
        timestamp: SystemTime,
    },

    // ===== Discovery Events (6 types) =====
    /// Host discovered as alive
    ///
    /// Emitted during host discovery phase when a host responds
    /// to ICMP/ARP/TCP probes.
    HostDiscovered {
        scan_id: Uuid,
        ip: IpAddr,
        method: DiscoveryMethod,
        latency_ms: Option<u64>,
        timestamp: SystemTime,
    },

    /// Open port discovered
    ///
    /// Emitted immediately when a port responds as open.
    /// Consumers can display this in real-time.
    PortFound {
        scan_id: Uuid,
        ip: IpAddr,
        port: u16,
        state: PortState,
        protocol: Protocol,
        scan_type: ScanType,
        timestamp: SystemTime,
    },

    /// Service detected on open port
    ///
    /// Emitted when service detection identifies the service/version
    /// running on an open port.
    ServiceDetected {
        scan_id: Uuid,
        ip: IpAddr,
        port: u16,
        service_name: String,
        service_version: Option<String>,
        confidence: f32,
        timestamp: SystemTime,
    },

    /// Operating system detected
    ///
    /// Emitted when OS fingerprinting successfully identifies
    /// the target's operating system.
    OsDetected {
        scan_id: Uuid,
        ip: IpAddr,
        os_name: String,
        os_version: Option<String>,
        confidence: f32,
        timestamp: SystemTime,
    },

    /// Banner grabbed from service
    ///
    /// Emitted when raw banner data is retrieved from a service.
    BannerGrabbed {
        scan_id: Uuid,
        ip: IpAddr,
        port: u16,
        banner: String,
        protocol: Protocol,
        timestamp: SystemTime,
    },

    /// TLS certificate found
    ///
    /// Emitted when TLS/SSL certificate is extracted from HTTPS/TLS service.
    CertificateFound {
        scan_id: Uuid,
        ip: IpAddr,
        port: u16,
        subject: String,
        issuer: String,
        valid_from: SystemTime,
        valid_until: SystemTime,
        timestamp: SystemTime,
    },

    // ===== Diagnostic Events (4 types) =====
    /// Rate limiting triggered
    ///
    /// Emitted when rate limiter throttles packet sending to stay
    /// within configured limits.
    RateLimitTriggered {
        scan_id: Uuid,
        current_rate: f64,
        target_rate: f64,
        duration_ms: u64,
        timestamp: SystemTime,
    },

    /// Retry scheduled for failed probe
    ///
    /// Emitted when a probe times out and will be retried.
    RetryScheduled {
        scan_id: Uuid,
        target: IpAddr,
        port: u16,
        attempt: u32,
        delay_ms: u64,
        timestamp: SystemTime,
    },

    /// Warning issued (non-fatal issue)
    ///
    /// Emitted for issues that don't stop the scan but users should know about.
    WarningIssued {
        scan_id: Uuid,
        message: String,
        severity: WarningSeverity,
        timestamp: SystemTime,
    },

    /// Metric recorded for monitoring
    ///
    /// Emitted for performance/operational metrics (packets sent/received, etc.).
    MetricRecorded {
        scan_id: Uuid,
        metric: MetricType,
        value: f64,
        timestamp: SystemTime,
    },
}

impl ScanEvent {
    /// Returns the scan ID for this event
    ///
    /// All events belong to a specific scan identified by UUID.
    pub fn scan_id(&self) -> Uuid {
        match self {
            ScanEvent::ScanStarted { scan_id, .. }
            | ScanEvent::ScanCompleted { scan_id, .. }
            | ScanEvent::ScanPaused { scan_id, .. }
            | ScanEvent::ScanResumed { scan_id, .. }
            | ScanEvent::ScanCancelled { scan_id, .. }
            | ScanEvent::ScanError { scan_id, .. }
            | ScanEvent::ProgressUpdate { scan_id, .. }
            | ScanEvent::StageChanged { scan_id, .. }
            | ScanEvent::HostDiscovered { scan_id, .. }
            | ScanEvent::PortFound { scan_id, .. }
            | ScanEvent::ServiceDetected { scan_id, .. }
            | ScanEvent::OsDetected { scan_id, .. }
            | ScanEvent::BannerGrabbed { scan_id, .. }
            | ScanEvent::CertificateFound { scan_id, .. }
            | ScanEvent::RateLimitTriggered { scan_id, .. }
            | ScanEvent::RetryScheduled { scan_id, .. }
            | ScanEvent::WarningIssued { scan_id, .. }
            | ScanEvent::MetricRecorded { scan_id, .. } => *scan_id,
        }
    }

    /// Returns the timestamp for this event
    ///
    /// Used for event ordering and time-range queries.
    pub fn timestamp(&self) -> SystemTime {
        match self {
            ScanEvent::ScanStarted { timestamp, .. }
            | ScanEvent::ScanCompleted { timestamp, .. }
            | ScanEvent::ScanPaused { timestamp, .. }
            | ScanEvent::ScanResumed { timestamp, .. }
            | ScanEvent::ScanCancelled { timestamp, .. }
            | ScanEvent::ScanError { timestamp, .. }
            | ScanEvent::ProgressUpdate { timestamp, .. }
            | ScanEvent::StageChanged { timestamp, .. }
            | ScanEvent::HostDiscovered { timestamp, .. }
            | ScanEvent::PortFound { timestamp, .. }
            | ScanEvent::ServiceDetected { timestamp, .. }
            | ScanEvent::OsDetected { timestamp, .. }
            | ScanEvent::BannerGrabbed { timestamp, .. }
            | ScanEvent::CertificateFound { timestamp, .. }
            | ScanEvent::RateLimitTriggered { timestamp, .. }
            | ScanEvent::RetryScheduled { timestamp, .. }
            | ScanEvent::WarningIssued { timestamp, .. }
            | ScanEvent::MetricRecorded { timestamp, .. } => *timestamp,
        }
    }

    /// Returns the event type (discriminant without data)
    ///
    /// Used for filtering events by type without matching full event data.
    pub fn event_type(&self) -> ScanEventType {
        match self {
            ScanEvent::ScanStarted { .. } => ScanEventType::ScanStarted,
            ScanEvent::ScanCompleted { .. } => ScanEventType::ScanCompleted,
            ScanEvent::ScanPaused { .. } => ScanEventType::ScanPaused,
            ScanEvent::ScanResumed { .. } => ScanEventType::ScanResumed,
            ScanEvent::ScanCancelled { .. } => ScanEventType::ScanCancelled,
            ScanEvent::ScanError { .. } => ScanEventType::ScanError,
            ScanEvent::ProgressUpdate { .. } => ScanEventType::ProgressUpdate,
            ScanEvent::StageChanged { .. } => ScanEventType::StageChanged,
            ScanEvent::HostDiscovered { .. } => ScanEventType::HostDiscovered,
            ScanEvent::PortFound { .. } => ScanEventType::PortFound,
            ScanEvent::ServiceDetected { .. } => ScanEventType::ServiceDetected,
            ScanEvent::OsDetected { .. } => ScanEventType::OsDetected,
            ScanEvent::BannerGrabbed { .. } => ScanEventType::BannerGrabbed,
            ScanEvent::CertificateFound { .. } => ScanEventType::CertificateFound,
            ScanEvent::RateLimitTriggered { .. } => ScanEventType::RateLimitTriggered,
            ScanEvent::RetryScheduled { .. } => ScanEventType::RetryScheduled,
            ScanEvent::WarningIssued { .. } => ScanEventType::WarningIssued,
            ScanEvent::MetricRecorded { .. } => ScanEventType::MetricRecorded,
        }
    }

    /// Validates event data (sanity checks)
    ///
    /// Returns error if event contains invalid data (e.g., percentage > 100%).
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            ScanEvent::ProgressUpdate {
                percentage,
                completed,
                total,
                ..
            } => {
                if *percentage < 0.0 || *percentage > 100.0 {
                    return Err(ValidationError::InvalidPercentage(*percentage));
                }
                if completed > total {
                    return Err(ValidationError::InvalidProgress {
                        completed: *completed,
                        total: *total,
                    });
                }
            }
            ScanEvent::ServiceDetected { confidence, .. }
            | ScanEvent::OsDetected { confidence, .. } => {
                if *confidence < 0.0 || *confidence > 1.0 {
                    return Err(ValidationError::InvalidConfidence(*confidence));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Format event for human-readable display
    ///
    /// Returns a concise string representation suitable for logging or CLI display.
    pub fn display(&self) -> String {
        match self {
            ScanEvent::ScanStarted {
                scan_type,
                target_count,
                ..
            } => format!(
                "Scan started: {:?} scan, {} targets",
                scan_type, target_count
            ),
            ScanEvent::ScanCompleted {
                duration,
                open_ports,
                ..
            } => format!(
                "Scan completed: {} open ports found in {:.2}s",
                open_ports,
                duration.as_secs_f64()
            ),
            ScanEvent::ProgressUpdate {
                stage, percentage, ..
            } => format!("Progress: {:?} {:.1}%", stage, percentage),
            ScanEvent::PortFound {
                ip,
                port,
                state,
                protocol,
                ..
            } => format!("Port {}/{}  {}  on {}", port, protocol, state, ip),
            ScanEvent::ServiceDetected {
                ip,
                port,
                service_name,
                ..
            } => format!("Service detected: {} on {}:{}", service_name, ip, port),
            ScanEvent::OsDetected { ip, os_name, .. } => {
                format!("OS detected: {} on {}", os_name, ip)
            }
            ScanEvent::WarningIssued { message, .. } => format!("Warning: {}", message),
            ScanEvent::ScanError { error, .. } => format!("Error: {}", error),
            _ => format!("{:?}", self.event_type()),
        }
    }
}

/// Lightweight event type discriminant (without data)
///
/// Used for filtering and categorization without full event matching.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanEventType {
    ScanStarted,
    ScanCompleted,
    ScanPaused,
    ScanResumed,
    ScanCancelled,
    ScanError,
    ProgressUpdate,
    StageChanged,
    HostDiscovered,
    PortFound,
    ServiceDetected,
    OsDetected,
    BannerGrabbed,
    CertificateFound,
    RateLimitTriggered,
    RetryScheduled,
    WarningIssued,
    MetricRecorded,
}

/// Scan stage progression
///
/// Represents the current phase of the scan lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanStage {
    /// Initializing scanner (loading config, setting up sockets)
    Initializing,
    /// Resolving target specifications (DNS, CIDR expansion)
    ResolvingTargets,
    /// Discovering live hosts (ICMP/ARP/NDP probes)
    DiscoveringHosts,
    /// Scanning ports (SYN/Connect/UDP/Stealth)
    ScanningPorts,
    /// Detecting services/OS (banner grabbing, fingerprinting)
    DetectingServices,
    /// Finalizing (writing output, cleanup)
    Finalizing,
    /// Scan completed
    Completed,
}

/// Throughput metrics
///
/// Real-time performance measurements.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Throughput {
    /// Packets sent per second
    pub packets_per_second: f64,
    /// Hosts scanned per minute
    pub hosts_per_minute: f64,
    /// Network bandwidth in Mbps
    pub bandwidth_mbps: f64,
}

/// Reason for scan pause
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PauseReason {
    /// User requested pause (interactive control)
    UserRequested,
    /// Rate limiting active (throttling to stay within limits)
    RateLimited,
    /// Temporary error (recoverable, waiting for retry)
    TemporaryError(String),
    /// Resource exhaustion (waiting for resources)
    ResourceExhaustion,
}

/// Host discovery method
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod {
    /// ICMP Echo Request (ping)
    IcmpEcho,
    /// ICMPv6 Echo Request (ping6)
    Icmpv6Echo,
    /// ARP request (local network)
    Arp,
    /// NDP Neighbor Solicitation (IPv6 local)
    Ndp,
    /// TCP SYN probe (specific port)
    TcpSyn { port: u16 },
    /// UDP probe (specific port)
    Udp { port: u16 },
}

/// Warning severity levels
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningSeverity {
    /// Low severity (informational)
    Low,
    /// Medium severity (may affect results)
    Medium,
    /// High severity (will affect results)
    High,
    /// Critical severity (scan may fail)
    Critical,
}

/// Metric types for monitoring
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    /// Total packets sent
    PacketsSent,
    /// Total packets received
    PacketsReceived,
    /// Total bytes sent
    BytesSent,
    /// Total bytes received
    BytesReceived,
    /// Round-trip time (milliseconds)
    RttMs,
    /// Current rate limit (packets/sec)
    CurrentRateLimit,
    /// Memory usage (bytes)
    MemoryUsageBytes,
    /// CPU usage (percentage)
    CpuUsagePercent,
}

/// Event validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid percentage: {0} (must be 0.0-100.0)")]
    InvalidPercentage(f32),

    #[error("Invalid progress: completed={completed} > total={total}")]
    InvalidProgress { completed: u64, total: u64 },

    #[error("Invalid confidence: {0} (must be 0.0-1.0)")]
    InvalidConfidence(f32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_started_event() {
        let event = ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            scan_type: ScanType::Syn,
            target_count: 1000,
            port_count: 1000,
            timestamp: SystemTime::now(),
        };

        assert_eq!(event.event_type(), ScanEventType::ScanStarted);
        assert!(event.validate().is_ok());
    }

    #[test]
    fn test_progress_update_validation() {
        let scan_id = Uuid::new_v4();

        // Valid progress
        let valid = ScanEvent::ProgressUpdate {
            scan_id,
            stage: ScanStage::ScanningPorts,
            percentage: 50.0,
            completed: 500,
            total: 1000,
            throughput: Throughput::default(),
            eta: None,
            timestamp: SystemTime::now(),
        };
        assert!(valid.validate().is_ok());

        // Invalid percentage (> 100)
        let invalid_pct = ScanEvent::ProgressUpdate {
            scan_id,
            stage: ScanStage::ScanningPorts,
            percentage: 150.0,
            completed: 500,
            total: 1000,
            throughput: Throughput::default(),
            eta: None,
            timestamp: SystemTime::now(),
        };
        assert!(invalid_pct.validate().is_err());

        // Invalid progress (completed > total)
        let invalid_prog = ScanEvent::ProgressUpdate {
            scan_id,
            stage: ScanStage::ScanningPorts,
            percentage: 50.0,
            completed: 1500,
            total: 1000,
            throughput: Throughput::default(),
            eta: None,
            timestamp: SystemTime::now(),
        };
        assert!(invalid_prog.validate().is_err());
    }

    #[test]
    fn test_event_scan_id_extraction() {
        let scan_id = Uuid::new_v4();
        let events = vec![
            ScanEvent::ScanStarted {
                scan_id,
                scan_type: ScanType::Syn,
                target_count: 100,
                port_count: 1000,
                timestamp: SystemTime::now(),
            },
            ScanEvent::ProgressUpdate {
                scan_id,
                stage: ScanStage::ScanningPorts,
                percentage: 50.0,
                completed: 50,
                total: 100,
                throughput: Throughput::default(),
                eta: None,
                timestamp: SystemTime::now(),
            },
            ScanEvent::PortFound {
                scan_id,
                ip: "192.168.1.1".parse().unwrap(),
                port: 80,
                state: PortState::Open,
                protocol: Protocol::Tcp,
                scan_type: ScanType::Syn,
                timestamp: SystemTime::now(),
            },
        ];

        for event in events {
            assert_eq!(event.scan_id(), scan_id);
        }
    }

    #[test]
    fn test_event_display() {
        let event = ScanEvent::PortFound {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.1".parse().unwrap(),
            port: 80,
            state: PortState::Open,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        };

        let display = event.display();
        assert!(display.contains("80"));
        assert!(display.contains("192.168.1.1"));
        assert!(display.contains("open"));
    }

    #[test]
    fn test_confidence_validation() {
        let scan_id = Uuid::new_v4();

        let valid = ScanEvent::ServiceDetected {
            scan_id,
            ip: "192.168.1.1".parse().unwrap(),
            port: 80,
            service_name: "HTTP".to_string(),
            service_version: None,
            confidence: 0.95,
            timestamp: SystemTime::now(),
        };
        assert!(valid.validate().is_ok());

        let invalid = ScanEvent::ServiceDetected {
            scan_id,
            ip: "192.168.1.1".parse().unwrap(),
            port: 80,
            service_name: "HTTP".to_string(),
            service_version: None,
            confidence: 1.5,
            timestamp: SystemTime::now(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_scan_stage_ordering() {
        assert!(ScanStage::Initializing < ScanStage::ResolvingTargets);
        assert!(ScanStage::ResolvingTargets < ScanStage::DiscoveringHosts);
        assert!(ScanStage::DiscoveringHosts < ScanStage::ScanningPorts);
        assert!(ScanStage::ScanningPorts < ScanStage::DetectingServices);
        assert!(ScanStage::DetectingServices < ScanStage::Finalizing);
        assert!(ScanStage::Finalizing < ScanStage::Completed);
    }
}
