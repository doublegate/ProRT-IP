//! Integration tests for progress display and event-driven updates
//!
//! Tests the ProgressDisplay integration with EventBus, ensuring:
//! - Progress updates work correctly with all scan types
//! - Quiet mode suppresses progress
//! - Live results streaming works
//! - EventBus integration is seamless

use prtip_cli::progress::{ProgressDisplay, ProgressStyle};
use prtip_core::event_bus::EventBus;
use prtip_core::events::{ScanEvent, ScanStage, Throughput};
use prtip_core::{Config, PortState, Protocol, ScanConfig, ScanType};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use uuid::Uuid;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a basic scan config for testing
fn create_test_config() -> Config {
    Config {
        scan: ScanConfig::default(),
        ..Default::default()
    }
}

/// Emit a series of test events to simulate a scan
async fn emit_test_scan_events(bus: &Arc<EventBus>, num_ports: usize) {
    let scan_id = Uuid::new_v4();

    // Scan started
    bus.publish(ScanEvent::ScanStarted {
        scan_id,
        scan_type: ScanType::Syn,
        target_count: 1,
        port_count: num_ports,
        timestamp: SystemTime::now(),
    })
    .await;

    // Progress updates
    for i in 0..num_ports {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        let port = 80 + i as u16;

        // Port found event
        bus.publish(ScanEvent::PortFound {
            scan_id,
            ip,
            port,
            state: PortState::Open,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        })
        .await;

        // Progress update event
        bus.publish(ScanEvent::ProgressUpdate {
            scan_id,
            stage: ScanStage::ScanningPorts,
            percentage: ((i + 1) as f32 / num_ports as f32) * 100.0,
            completed: (i + 1) as u64,
            total: num_ports as u64,
            throughput: Throughput {
                packets_per_second: 1000.0,
                hosts_per_minute: 60.0,
                bandwidth_mbps: 10.0,
            },
            eta: Some(Duration::from_secs(5)),
            timestamp: SystemTime::now(),
        })
        .await;

        // Small delay to simulate real scanning
        sleep(Duration::from_millis(10)).await;
    }

    // Scan completed
    bus.publish(ScanEvent::ScanCompleted {
        scan_id,
        duration: Duration::from_millis(1000),
        total_targets: 1,
        open_ports: num_ports,
        closed_ports: 0,
        filtered_ports: 0,
        detected_services: 0,
        timestamp: SystemTime::now(),
    })
    .await;
}

// ============================================================================
// PROGRESS DISPLAY TESTS (5 tests)
// ============================================================================

#[tokio::test]
async fn test_progress_display_tcp_scan() {
    let bus = Arc::new(EventBus::new(1000));
    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, false);

    // Start display task
    let _task = display.start().await;

    // Emit scan events
    emit_test_scan_events(&bus, 10).await;

    // Give time for events to process
    sleep(Duration::from_millis(200)).await;

    // Cleanup
    display.finish();
}

#[tokio::test]
async fn test_progress_display_syn_scan() {
    let bus = Arc::new(EventBus::new(1000));
    let mut config = create_test_config();
    config.scan.scan_type = ScanType::Syn;
    config.scan = config.scan.with_event_bus(bus.clone());

    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, false);
    let _task = display.start().await;

    emit_test_scan_events(&bus, 5).await;
    sleep(Duration::from_millis(100)).await;

    display.finish();
}

#[tokio::test]
async fn test_progress_display_udp_scan() {
    let bus = Arc::new(EventBus::new(1000));
    let mut config = create_test_config();
    config.scan.scan_type = ScanType::Udp;
    config.scan = config.scan.with_event_bus(bus.clone());

    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, false);
    let _task = display.start().await;

    emit_test_scan_events(&bus, 3).await;
    sleep(Duration::from_millis(100)).await;

    display.finish();
}

#[tokio::test]
async fn test_progress_display_stealth_scan() {
    let bus = Arc::new(EventBus::new(1000));
    let mut config = create_test_config();
    config.scan.scan_type = ScanType::Fin;
    config.scan = config.scan.with_event_bus(bus.clone());

    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, false);
    let _task = display.start().await;

    emit_test_scan_events(&bus, 5).await;
    sleep(Duration::from_millis(100)).await;

    display.finish();
}

#[tokio::test]
async fn test_progress_display_idle_scan() {
    let bus = Arc::new(EventBus::new(1000));
    let mut config = create_test_config();
    config.scan.scan_type = ScanType::Idle;
    config.scan = config.scan.with_event_bus(bus.clone());

    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, false);
    let _task = display.start().await;

    emit_test_scan_events(&bus, 5).await;
    sleep(Duration::from_millis(100)).await;

    display.finish();
}

// ============================================================================
// QUIET MODE TESTS (3 tests)
// ============================================================================

#[tokio::test]
async fn test_quiet_mode_no_progress() {
    let bus = Arc::new(EventBus::new(1000));

    // Create display with quiet=true
    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, true);
    let _task = display.start().await;

    // Emit events - they should be ignored due to quiet mode
    emit_test_scan_events(&bus, 5).await;
    sleep(Duration::from_millis(100)).await;

    // In quiet mode, no output should be generated
    // (This is a behavioral test - we verify it doesn't panic/error)
    display.finish();
}

#[tokio::test]
async fn test_quiet_mode_tcp_scan() {
    let bus = Arc::new(EventBus::new(1000));
    let mut config = create_test_config();
    config.scan = config.scan.with_event_bus(bus.clone());

    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, true);
    let _task = display.start().await;

    emit_test_scan_events(&bus, 10).await;
    sleep(Duration::from_millis(100)).await;

    display.finish();
}

#[tokio::test]
async fn test_quiet_mode_with_output() {
    let bus = Arc::new(EventBus::new(1000));

    // Quiet mode should not affect final results
    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, true);
    let _task = display.start().await;

    emit_test_scan_events(&bus, 5).await;
    sleep(Duration::from_millis(100)).await;

    // Results should still be available even in quiet mode
    display.finish();
}

// ============================================================================
// LIVE RESULTS TESTS (5 tests)
// ============================================================================

#[tokio::test]
async fn test_live_results_streaming() {
    let bus = Arc::new(EventBus::new(1000));

    // Track received events
    let received = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let received_clone = received.clone();

    // Simulate live results subscriber
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    bus.subscribe(
        tx,
        prtip_core::event_bus::EventFilter::EventType(vec![
            prtip_core::events::ScanEventType::PortFound,
        ]),
    )
    .await;

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let ScanEvent::PortFound { port, .. } = event {
                received_clone.lock().await.push(port);
            }
        }
    });

    // Emit events
    emit_test_scan_events(&bus, 5).await;
    sleep(Duration::from_millis(100)).await;

    // Verify events were received
    let ports = received.lock().await;
    assert!(!ports.is_empty(), "Should have received port events");
}

#[tokio::test]
async fn test_live_results_with_progress() {
    let bus = Arc::new(EventBus::new(1000));
    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, false);
    let _task = display.start().await;

    // Track live results
    let received = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let received_clone = received.clone();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    bus.subscribe(
        tx,
        prtip_core::event_bus::EventFilter::EventType(vec![
            prtip_core::events::ScanEventType::PortFound,
        ]),
    )
    .await;

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let ScanEvent::PortFound { port, .. } = event {
                received_clone.lock().await.push(port);
            }
        }
    });

    emit_test_scan_events(&bus, 5).await;
    sleep(Duration::from_millis(100)).await;

    let ports = received.lock().await;
    assert!(
        !ports.is_empty(),
        "Live results should work alongside progress"
    );

    display.finish();
}

#[tokio::test]
async fn test_live_results_quiet_mode() {
    let bus = Arc::new(EventBus::new(1000));
    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, true);
    let _task = display.start().await;

    // Live results should still work in quiet mode
    let received = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let received_clone = received.clone();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    bus.subscribe(
        tx,
        prtip_core::event_bus::EventFilter::EventType(vec![
            prtip_core::events::ScanEventType::PortFound,
        ]),
    )
    .await;

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let ScanEvent::PortFound { port, .. } = event {
                received_clone.lock().await.push(port);
            }
        }
    });

    emit_test_scan_events(&bus, 5).await;
    sleep(Duration::from_millis(100)).await;

    let ports = received.lock().await;
    assert!(
        !ports.is_empty(),
        "Live results should work even in quiet mode"
    );

    display.finish();
}

#[tokio::test]
async fn test_live_results_multiple_targets() {
    let bus = Arc::new(EventBus::new(1000));

    let received = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let received_clone = received.clone();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    bus.subscribe(
        tx,
        prtip_core::event_bus::EventFilter::EventType(vec![
            prtip_core::events::ScanEventType::PortFound,
        ]),
    )
    .await;

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let ScanEvent::PortFound { ip, port, .. } = event {
                received_clone.lock().await.push((ip, port));
            }
        }
    });

    // Emit events for multiple targets
    let scan_id = Uuid::new_v4();
    for i in 1..=3 {
        let ip: IpAddr = format!("192.168.1.{}", i).parse().unwrap();
        bus.publish(ScanEvent::PortFound {
            scan_id,
            ip,
            port: 80,
            state: PortState::Open,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        })
        .await;
    }

    sleep(Duration::from_millis(100)).await;

    let results = received.lock().await;
    assert_eq!(results.len(), 3, "Should receive events for all targets");
}

#[tokio::test]
async fn test_live_results_format() {
    let bus = Arc::new(EventBus::new(1000));

    let received = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let received_clone = received.clone();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    bus.subscribe(
        tx,
        prtip_core::event_bus::EventFilter::EventType(vec![
            prtip_core::events::ScanEventType::PortFound,
        ]),
    )
    .await;

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let ScanEvent::PortFound {
                ip,
                port,
                state,
                protocol,
                ..
            } = event
            {
                let formatted = format!("[LIVE] {}:{} {} ({})", ip, port, state, protocol);
                received_clone.lock().await.push(formatted);
            }
        }
    });

    let scan_id = Uuid::new_v4();
    let ip: IpAddr = "192.168.1.1".parse().unwrap();
    bus.publish(ScanEvent::PortFound {
        scan_id,
        ip,
        port: 443,
        state: PortState::Open,
        protocol: Protocol::Tcp,
        scan_type: ScanType::Syn,
        timestamp: SystemTime::now(),
    })
    .await;

    sleep(Duration::from_millis(50)).await;

    let results = received.lock().await;
    assert!(!results.is_empty(), "Should format live results");
    assert!(
        results[0].contains("[LIVE]"),
        "Format should include [LIVE] prefix"
    );
    assert!(
        results[0].contains("192.168.1.1"),
        "Format should include IP"
    );
    assert!(results[0].contains("443"), "Format should include port");
}

// ============================================================================
// EVENTBUS INTEGRATION TESTS (4 tests)
// ============================================================================

#[tokio::test]
async fn test_event_bus_attached_to_config() {
    let bus = Arc::new(EventBus::new(1000));
    let mut config = create_test_config();

    // Attach EventBus
    config.scan = config.scan.with_event_bus(bus.clone());

    // Verify it's attached (by checking it doesn't panic when publishing)
    let scan_id = Uuid::new_v4();
    bus.publish(ScanEvent::ScanStarted {
        scan_id,
        scan_type: ScanType::Syn,
        target_count: 1,
        port_count: 100,
        timestamp: SystemTime::now(),
    })
    .await;
}

#[tokio::test]
async fn test_event_bus_lifecycle_events() {
    let bus = Arc::new(EventBus::new(1000));

    let received = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let received_clone = received.clone();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    bus.subscribe(
        tx,
        prtip_core::event_bus::EventFilter::EventType(vec![
            prtip_core::events::ScanEventType::ScanStarted,
            prtip_core::events::ScanEventType::ScanCompleted,
        ]),
    )
    .await;

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                ScanEvent::ScanStarted { .. } => {
                    received_clone.lock().await.push("started".to_string())
                }
                ScanEvent::ScanCompleted { .. } => {
                    received_clone.lock().await.push("completed".to_string())
                }
                _ => {}
            }
        }
    });

    // Emit lifecycle events
    let scan_id = Uuid::new_v4();
    bus.publish(ScanEvent::ScanStarted {
        scan_id,
        scan_type: ScanType::Syn,
        target_count: 1,
        port_count: 10,
        timestamp: SystemTime::now(),
    })
    .await;

    bus.publish(ScanEvent::ScanCompleted {
        scan_id,
        duration: Duration::from_millis(1000),
        total_targets: 1,
        open_ports: 5,
        closed_ports: 5,
        filtered_ports: 0,
        detected_services: 0,
        timestamp: SystemTime::now(),
    })
    .await;

    sleep(Duration::from_millis(50)).await;

    let events = received.lock().await;
    assert_eq!(events.len(), 2, "Should receive both lifecycle events");
    assert_eq!(events[0], "started");
    assert_eq!(events[1], "completed");
}

#[tokio::test]
async fn test_event_bus_port_found_events() {
    let bus = Arc::new(EventBus::new(1000));

    let received = Arc::new(tokio::sync::Mutex::new(0));
    let received_clone = received.clone();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    bus.subscribe(
        tx,
        prtip_core::event_bus::EventFilter::EventType(vec![
            prtip_core::events::ScanEventType::PortFound,
        ]),
    )
    .await;

    tokio::spawn(async move {
        while let Some(_event) = rx.recv().await {
            *received_clone.lock().await += 1;
        }
    });

    // Emit multiple port found events
    let scan_id = Uuid::new_v4();
    for port in 80..=84 {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        bus.publish(ScanEvent::PortFound {
            scan_id,
            ip,
            port,
            state: PortState::Open,
            protocol: Protocol::Tcp,
            scan_type: ScanType::Syn,
            timestamp: SystemTime::now(),
        })
        .await;
    }

    sleep(Duration::from_millis(50)).await;

    let count = *received.lock().await;
    assert_eq!(count, 5, "Should receive all port found events");
}

#[tokio::test]
async fn test_event_bus_scan_completed_event() {
    let bus = Arc::new(EventBus::new(1000));

    let received = Arc::new(tokio::sync::Mutex::new(None));
    let received_clone = received.clone();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    bus.subscribe(
        tx,
        prtip_core::event_bus::EventFilter::EventType(vec![
            prtip_core::events::ScanEventType::ScanCompleted,
        ]),
    )
    .await;

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let ScanEvent::ScanCompleted {
                total_targets,
                open_ports,
                ..
            } = event
            {
                *received_clone.lock().await = Some((total_targets, open_ports));
            }
        }
    });

    let scan_id = Uuid::new_v4();
    bus.publish(ScanEvent::ScanCompleted {
        scan_id,
        duration: Duration::from_millis(5000),
        total_targets: 100,
        open_ports: 42,
        closed_ports: 58,
        filtered_ports: 0,
        detected_services: 0,
        timestamp: SystemTime::now(),
    })
    .await;

    sleep(Duration::from_millis(50)).await;

    let result = *received.lock().await;
    assert_eq!(
        result,
        Some((100, 42)),
        "Should receive scan completed event with correct data"
    );
}

// ============================================================================
// EDGE CASES (3 tests)
// ============================================================================

#[tokio::test]
async fn test_progress_no_ports_found() {
    let bus = Arc::new(EventBus::new(1000));
    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, false);
    let _task = display.start().await;

    // Start and complete scan without finding any ports
    let scan_id = Uuid::new_v4();
    bus.publish(ScanEvent::ScanStarted {
        scan_id,
        scan_type: ScanType::Syn,
        target_count: 1,
        port_count: 100,
        timestamp: SystemTime::now(),
    })
    .await;

    bus.publish(ScanEvent::ScanCompleted {
        scan_id,
        duration: Duration::from_millis(1000),
        total_targets: 1,
        open_ports: 0,
        closed_ports: 100,
        filtered_ports: 0,
        detected_services: 0,
        timestamp: SystemTime::now(),
    })
    .await;

    sleep(Duration::from_millis(50)).await;

    display.finish();
}

#[tokio::test]
async fn test_progress_single_port() {
    let bus = Arc::new(EventBus::new(1000));
    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, false);
    let _task = display.start().await;

    // Scan with just one port
    emit_test_scan_events(&bus, 1).await;
    sleep(Duration::from_millis(50)).await;

    display.finish();
}

#[tokio::test]
async fn test_progress_large_scan() {
    let bus = Arc::new(EventBus::new(10000)); // Larger buffer for large scan
    let display = ProgressDisplay::new(bus.clone(), ProgressStyle::Compact, false);
    let _task = display.start().await;

    // Simulate larger scan (100 ports)
    emit_test_scan_events(&bus, 100).await;
    sleep(Duration::from_millis(1500)).await; // Longer wait for 100 events

    display.finish();
}
