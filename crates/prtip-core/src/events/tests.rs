//! Comprehensive tests for event system
//!
//! Tests cover:
//! - JSON serialization round-trips for all event types
//! - Event validation logic
//! - Helper methods (scan_id, timestamp, event_type, display)
//! - Edge cases and error conditions

use crate::events::*;
use crate::types::{PortState, Protocol, ScanType};
use std::net::IpAddr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

mod serialization_tests {
    use super::*;

    #[test]
    fn test_scan_started_serialization() {
        let event = ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            scan_type: ScanType::Syn,
            target_count: 1000,
            port_count: 65535,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
        assert_eq!(event.event_type(), deserialized.event_type());
    }

    #[test]
    fn test_scan_completed_serialization() {
        let event = ScanEvent::ScanCompleted {
            scan_id: Uuid::new_v4(),
            duration: Duration::from_secs(120),
            total_targets: 1000,
            open_ports: 150,
            closed_ports: 800,
            filtered_ports: 50,
            detected_services: 75,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_progress_update_serialization() {
        let event = ScanEvent::ProgressUpdate {
            scan_id: Uuid::new_v4(),
            stage: ScanStage::ScanningPorts,
            percentage: 45.5,
            completed: 4550,
            total: 10000,
            throughput: Throughput {
                packets_per_second: 12500.0,
                hosts_per_minute: 250.0,
                bandwidth_mbps: 1.5,
            },
            eta: Some(Duration::from_secs(120)),
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
        assert_eq!(event.event_type(), ScanEventType::ProgressUpdate);
    }

    #[test]
    fn test_port_found_serialization() {
        let event = ScanEvent::PortFound {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.100".parse().unwrap(),
            port: 443,
            state: PortState::Open,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_service_detected_serialization() {
        let event = ScanEvent::ServiceDetected {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.100".parse().unwrap(),
            port: 80,
            service_name: "http".to_string(),
            service_version: Some("Apache/2.4.51".to_string()),
            confidence: 0.95,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_os_detected_serialization() {
        let event = ScanEvent::OsDetected {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.100".parse().unwrap(),
            os_name: "Linux".to_string(),
            os_version: Some("Ubuntu 22.04".to_string()),
            confidence: 0.88,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_banner_grabbed_serialization() {
        let event = ScanEvent::BannerGrabbed {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.100".parse().unwrap(),
            port: 22,
            banner: "SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.1".to_string(),
            protocol: Protocol::Tcp,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_certificate_found_serialization() {
        let now = SystemTime::now();
        let event = ScanEvent::CertificateFound {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.100".parse().unwrap(),
            port: 443,
            subject: "CN=example.com".to_string(),
            issuer: "CN=Let's Encrypt Authority X3".to_string(),
            valid_from: now,
            valid_until: now + Duration::from_secs(90 * 24 * 3600),
            timestamp: now,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_host_discovered_serialization() {
        let event = ScanEvent::HostDiscovered {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.100".parse().unwrap(),
            method: DiscoveryMethod::IcmpEcho,
            latency_ms: Some(15),
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_stage_changed_serialization() {
        let event = ScanEvent::StageChanged {
            scan_id: Uuid::new_v4(),
            from_stage: ScanStage::DiscoveringHosts,
            to_stage: ScanStage::ScanningPorts,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_warning_issued_serialization() {
        let event = ScanEvent::WarningIssued {
            scan_id: Uuid::new_v4(),
            message: "Rate limit exceeded, throttling enabled".to_string(),
            severity: WarningSeverity::Medium,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_scan_error_serialization() {
        let event = ScanEvent::ScanError {
            scan_id: Uuid::new_v4(),
            error: "Permission denied: requires CAP_NET_RAW".to_string(),
            recoverable: false,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_rate_limit_triggered_serialization() {
        let event = ScanEvent::RateLimitTriggered {
            scan_id: Uuid::new_v4(),
            current_rate: 15000.0,
            target_rate: 10000.0,
            duration_ms: 500,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_retry_scheduled_serialization() {
        let event = ScanEvent::RetryScheduled {
            scan_id: Uuid::new_v4(),
            target: "192.168.1.100".parse().unwrap(),
            port: 80,
            attempt: 2,
            delay_ms: 1000,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_metric_recorded_serialization() {
        let event = ScanEvent::MetricRecorded {
            scan_id: Uuid::new_v4(),
            metric: MetricType::PacketsSent,
            value: 125000.0,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_scan_paused_serialization() {
        let event = ScanEvent::ScanPaused {
            scan_id: Uuid::new_v4(),
            reason: PauseReason::UserRequested,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_scan_resumed_serialization() {
        let event = ScanEvent::ScanResumed {
            scan_id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_scan_cancelled_serialization() {
        let event = ScanEvent::ScanCancelled {
            scan_id: Uuid::new_v4(),
            reason: "User interrupted with Ctrl+C".to_string(),
            partial_results: true,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());
    }

    #[test]
    fn test_ipv6_port_found_serialization() {
        let event = ScanEvent::PortFound {
            scan_id: Uuid::new_v4(),
            ip: "2001:db8::1".parse().unwrap(),
            port: 443,
            state: PortState::Open,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.scan_id(), deserialized.scan_id());

        // Verify IPv6 address
        match deserialized {
            ScanEvent::PortFound { ip, .. } => {
                assert!(ip.is_ipv6());
            }
            _ => panic!("Expected PortFound event"),
        }
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_valid_progress_percentage() {
        let event = ScanEvent::ProgressUpdate {
            scan_id: Uuid::new_v4(),
            stage: ScanStage::ScanningPorts,
            percentage: 75.0,
            completed: 750,
            total: 1000,
            throughput: Throughput::default(),
            eta: None,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_ok());
    }

    #[test]
    fn test_invalid_percentage_negative() {
        let event = ScanEvent::ProgressUpdate {
            scan_id: Uuid::new_v4(),
            stage: ScanStage::ScanningPorts,
            percentage: -10.0,
            completed: 0,
            total: 1000,
            throughput: Throughput::default(),
            eta: None,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_err());
    }

    #[test]
    fn test_invalid_percentage_over_100() {
        let event = ScanEvent::ProgressUpdate {
            scan_id: Uuid::new_v4(),
            stage: ScanStage::ScanningPorts,
            percentage: 105.0,
            completed: 1050,
            total: 1000,
            throughput: Throughput::default(),
            eta: None,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_err());
    }

    #[test]
    fn test_invalid_completed_greater_than_total() {
        let event = ScanEvent::ProgressUpdate {
            scan_id: Uuid::new_v4(),
            stage: ScanStage::ScanningPorts,
            percentage: 50.0,
            completed: 1500,
            total: 1000,
            throughput: Throughput::default(),
            eta: None,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_err());
    }

    #[test]
    fn test_valid_confidence_service() {
        let event = ScanEvent::ServiceDetected {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.1".parse().unwrap(),
            port: 80,
            service_name: "http".to_string(),
            service_version: None,
            confidence: 0.95,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_ok());
    }

    #[test]
    fn test_invalid_confidence_over_1() {
        let event = ScanEvent::ServiceDetected {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.1".parse().unwrap(),
            port: 80,
            service_name: "http".to_string(),
            service_version: None,
            confidence: 1.5,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_err());
    }

    #[test]
    fn test_invalid_confidence_negative() {
        let event = ScanEvent::OsDetected {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.1".parse().unwrap(),
            os_name: "Linux".to_string(),
            os_version: None,
            confidence: -0.1,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_err());
    }

    #[test]
    fn test_edge_case_zero_progress() {
        let event = ScanEvent::ProgressUpdate {
            scan_id: Uuid::new_v4(),
            stage: ScanStage::Initializing,
            percentage: 0.0,
            completed: 0,
            total: 1000,
            throughput: Throughput::default(),
            eta: None,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_ok());
    }

    #[test]
    fn test_edge_case_complete_progress() {
        let event = ScanEvent::ProgressUpdate {
            scan_id: Uuid::new_v4(),
            stage: ScanStage::Completed,
            percentage: 100.0,
            completed: 1000,
            total: 1000,
            throughput: Throughput::default(),
            eta: Some(Duration::from_secs(0)),
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_ok());
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_scan_id_consistency() {
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
            ScanEvent::ScanCompleted {
                scan_id,
                duration: Duration::from_secs(10),
                total_targets: 100,
                open_ports: 10,
                closed_ports: 80,
                filtered_ports: 10,
                detected_services: 5,
                timestamp: SystemTime::now(),
            },
        ];

        // All events should have same scan_id
        for event in events {
            assert_eq!(event.scan_id(), scan_id);
        }
    }

    #[test]
    fn test_event_type_extraction() {
        let events = vec![
            (
                ScanEvent::ScanStarted {
                    scan_id: Uuid::new_v4(),
                    scan_type: ScanType::Syn,
                    target_count: 100,
                    port_count: 1000,
                    timestamp: SystemTime::now(),
                },
                ScanEventType::ScanStarted,
            ),
            (
                ScanEvent::ProgressUpdate {
                    scan_id: Uuid::new_v4(),
                    stage: ScanStage::ScanningPorts,
                    percentage: 50.0,
                    completed: 50,
                    total: 100,
                    throughput: Throughput::default(),
                    eta: None,
                    timestamp: SystemTime::now(),
                },
                ScanEventType::ProgressUpdate,
            ),
            (
                ScanEvent::PortFound {
                    scan_id: Uuid::new_v4(),
                    ip: "192.168.1.1".parse().unwrap(),
                    port: 80,
                    state: PortState::Open,
                    protocol: Protocol::Tcp,
                    scan_type: ScanType::Syn,
                    timestamp: SystemTime::now(),
                },
                ScanEventType::PortFound,
            ),
        ];

        for (event, expected_type) in events {
            assert_eq!(event.event_type(), expected_type);
        }
    }

    #[test]
    fn test_display_formatting() {
        let event = ScanEvent::PortFound {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.100".parse().unwrap(),
            port: 443,
            state: PortState::Open,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        };

        let display = event.display();
        assert!(display.contains("443"));
        assert!(display.contains("192.168.1.100"));
        assert!(display.contains("open"));
        assert!(display.contains("TCP"));
    }

    #[test]
    fn test_display_scan_completed() {
        let event = ScanEvent::ScanCompleted {
            scan_id: Uuid::new_v4(),
            duration: Duration::from_secs(120),
            total_targets: 1000,
            open_ports: 150,
            closed_ports: 800,
            filtered_ports: 50,
            detected_services: 75,
            timestamp: SystemTime::now(),
        };

        let display = event.display();
        assert!(display.contains("150"));
        assert!(display.contains("completed"));
    }

    #[test]
    fn test_timestamp_ordering() {
        let now = SystemTime::now();
        let earlier = now - Duration::from_secs(60);
        let later = now + Duration::from_secs(60);

        let event1 = ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            scan_type: ScanType::Syn,
            target_count: 100,
            port_count: 1000,
            timestamp: earlier,
        };

        let event2 = ScanEvent::ScanCompleted {
            scan_id: Uuid::new_v4(),
            duration: Duration::from_secs(120),
            total_targets: 100,
            open_ports: 10,
            closed_ports: 80,
            filtered_ports: 10,
            detected_services: 5,
            timestamp: later,
        };

        assert!(event1.timestamp() < event2.timestamp());
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_zero_targets_scan() {
        let event = ScanEvent::ScanStarted {
            scan_id: Uuid::new_v4(),
            scan_type: ScanType::Syn,
            target_count: 0,
            port_count: 0,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_ok());
        assert_eq!(event.event_type(), ScanEventType::ScanStarted);
    }

    #[test]
    fn test_empty_service_version() {
        let event = ScanEvent::ServiceDetected {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.1".parse().unwrap(),
            port: 80,
            service_name: "http".to_string(),
            service_version: None,
            confidence: 0.75,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_ok());
    }

    #[test]
    fn test_empty_banner() {
        let event = ScanEvent::BannerGrabbed {
            scan_id: Uuid::new_v4(),
            ip: "192.168.1.1".parse().unwrap(),
            port: 80,
            banner: String::new(),
            protocol: Protocol::Tcp,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_ok());
    }

    #[test]
    fn test_all_ports_filtered() {
        let event = ScanEvent::ScanCompleted {
            scan_id: Uuid::new_v4(),
            duration: Duration::from_secs(60),
            total_targets: 100,
            open_ports: 0,
            closed_ports: 0,
            filtered_ports: 1000,
            detected_services: 0,
            timestamp: SystemTime::now(),
        };

        assert!(event.validate().is_ok());
    }
}
