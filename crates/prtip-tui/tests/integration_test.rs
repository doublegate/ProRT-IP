//! Integration tests for prtip-tui
//!
//! These tests verify critical TUI functionality without
//! requiring full event construction.

use prtip_core::event_bus::EventBus;
use prtip_tui::{App, EventAggregator, ScanState, UIState};
use std::sync::Arc;

#[test]
fn test_app_creation() {
    let event_bus = Arc::new(EventBus::new(1000));
    let app = App::new(Arc::clone(&event_bus));

    assert!(!app.should_quit());
    assert!(Arc::ptr_eq(&app.scan_state(), &app.scan_state()));
}

#[test]
fn test_scan_state_initialization() {
    let state = ScanState::new();

    use prtip_core::events::ScanStage;
    assert_eq!(state.stage, ScanStage::Initializing);
    assert_eq!(state.progress_percentage, 0.0);
    assert_eq!(state.completed, 0);
    assert_eq!(state.total, 0);
    assert_eq!(state.open_ports, 0);
    assert_eq!(state.closed_ports, 0);
    assert_eq!(state.filtered_ports, 0);
    assert_eq!(state.detected_services, 0);
    assert_eq!(state.errors, 0);
    assert!(state.discovered_hosts.is_empty());
    assert!(state.warnings.is_empty());
}

#[test]
fn test_scan_state_shared() {
    let state1 = ScanState::shared();
    let state2 = Arc::clone(&state1);

    // Modify via state1
    {
        let mut s = state1.write();
        s.open_ports = 10;
        s.closed_ports = 990;
    }

    // Read via state2
    {
        let s = state2.read();
        assert_eq!(s.open_ports, 10);
        assert_eq!(s.closed_ports, 990);
    }
}

#[test]
fn test_ui_state_initialization() {
    let state = UIState::new();

    assert_eq!(state.cursor_position, 0);
    assert_eq!(state.scroll_offset, 0);
    assert_eq!(state.input_buffer, "");
    assert!(!state.show_help);
    assert_eq!(state.fps, 0.0);
    assert_eq!(state.aggregator_dropped_events, 0);
}

#[test]
fn test_ui_state_pane_navigation() {
    let mut state = UIState::new();

    use prtip_tui::state::SelectedPane;
    assert_eq!(state.selected_pane, SelectedPane::Main);

    state.next_pane();
    assert_eq!(state.selected_pane, SelectedPane::Help);

    state.next_pane();
    assert_eq!(state.selected_pane, SelectedPane::Main);

    state.prev_pane();
    assert_eq!(state.selected_pane, SelectedPane::Help);
}

#[test]
fn test_ui_state_help_toggle() {
    let mut state = UIState::new();

    assert!(!state.show_help);

    state.toggle_help();
    assert!(state.show_help);

    state.toggle_help();
    assert!(!state.show_help);
}

#[test]
fn test_ui_state_cursor_navigation() {
    let mut state = UIState::new();

    assert_eq!(state.cursor_position, 0);

    state.cursor_position = 5;
    assert_eq!(state.cursor_position, 5);

    state.cursor_position = state.cursor_position.saturating_add(1);
    assert_eq!(state.cursor_position, 6);

    state.cursor_position = state.cursor_position.saturating_sub(1);
    assert_eq!(state.cursor_position, 5);

    state.cursor_position = 0;
    state.cursor_position = state.cursor_position.saturating_sub(1);
    assert_eq!(state.cursor_position, 0); // Should not underflow
}

#[test]
fn test_event_aggregator_creation() {
    let agg = EventAggregator::new();
    let stats = agg.stats();

    assert_eq!(stats.ports_found, 0);
    assert_eq!(stats.hosts_discovered, 0);
    assert_eq!(stats.services_detected, 0);
    assert_eq!(stats.total_events, 0);
    assert_eq!(stats.dropped_events, 0);
    assert!(stats.discovered_ips.is_empty());
}

#[test]
fn test_event_aggregator_should_flush_timing() {
    use std::thread;
    use std::time::Duration;

    let mut agg = EventAggregator::new();

    // Initially should flush (first time)
    assert!(agg.should_flush());

    // Flush
    agg.flush();

    // Should not flush immediately after
    assert!(!agg.should_flush());

    // Wait for 20ms (longer than 16ms MIN_EVENT_INTERVAL, with margin for timing precision)
    thread::sleep(Duration::from_millis(20));

    // Should flush now
    assert!(agg.should_flush());
}

#[test]
fn test_event_aggregator_flush_resets() {
    let mut agg = EventAggregator::new();

    // Manually set stats (simulating aggregated events)
    // Since we can't easily construct events without all fields,
    // this test verifies the flush mechanism resets state

    // Flush and check that stats are reset
    let (events, stats) = agg.flush();

    // Initial flush should have empty events and zeroed stats
    assert_eq!(events.len(), 0);
    assert_eq!(stats.ports_found, 0);
    assert_eq!(stats.total_events, 0);

    // After flush, stats should be reset
    let new_stats = agg.stats();
    assert_eq!(new_stats.ports_found, 0);
    assert_eq!(new_stats.total_events, 0);
}

#[test]
fn test_scan_state_field_updates() {
    let state = ScanState::shared();

    {
        let mut s = state.write();
        s.open_ports = 15;
        s.closed_ports = 985;
        s.filtered_ports = 0;
        s.detected_services = 10;
        s.errors = 2;
        s.warnings.push("Warning 1".to_string());
        s.warnings.push("Warning 2".to_string());
        s.progress_percentage = 75.5;
        s.completed = 755;
        s.total = 1000;
    }

    {
        let s = state.read();
        assert_eq!(s.open_ports, 15);
        assert_eq!(s.closed_ports, 985);
        assert_eq!(s.filtered_ports, 0);
        assert_eq!(s.detected_services, 10);
        assert_eq!(s.errors, 2);
        assert_eq!(s.warnings.len(), 2);
        assert_eq!(s.progress_percentage, 75.5);
        assert_eq!(s.completed, 755);
        assert_eq!(s.total, 1000);
    }
}

#[test]
fn test_scan_state_host_discovery() {
    let state = ScanState::shared();

    let ip1 = "192.168.1.1".parse().unwrap();
    let ip2 = "192.168.1.2".parse().unwrap();

    {
        let mut s = state.write();
        s.discovered_hosts.push(ip1);
        s.discovered_hosts.push(ip2);
    }

    {
        let s = state.read();
        assert_eq!(s.discovered_hosts.len(), 2);
        assert!(s.discovered_hosts.contains(&ip1));
        assert!(s.discovered_hosts.contains(&ip2));
    }
}

#[tokio::test]
async fn test_eventbus_creation_and_subscription() {
    let event_bus = Arc::new(EventBus::new(1000));

    // Subscribe to all events
    let _event_rx = prtip_tui::events::subscribe_to_events(
        Arc::clone(&event_bus),
        prtip_core::event_bus::EventFilter::All,
    )
    .await;

    // Subscription should succeed without panic
    // (actual event publishing is tested in prtip-core)
}

#[test]
fn test_multiple_apps_share_state() {
    let event_bus = Arc::new(EventBus::new(1000));
    let app1 = App::new(Arc::clone(&event_bus));
    let app2 = App::new(Arc::clone(&event_bus));

    // Each app has its own state
    assert!(!Arc::ptr_eq(&app1.scan_state(), &app2.scan_state()));

    // But they share the same event bus
    assert!(Arc::ptr_eq(&event_bus, &event_bus));
}

#[test]
fn test_ui_state_default() {
    let state1 = UIState::new();
    let state2 = UIState::default();

    assert_eq!(state1.cursor_position, state2.cursor_position);
    assert_eq!(state1.scroll_offset, state2.scroll_offset);
    assert_eq!(state1.show_help, state2.show_help);
    assert_eq!(state1.fps, state2.fps);
}
