//! ScanEvent handler dispatch

use parking_lot::RwLock;
use prtip_core::events::ScanEvent;
use std::sync::Arc;

use crate::state::ScanState;

/// Handle a ScanEvent and update the shared ScanState
///
/// This function dispatches each event type to appropriate handlers
/// that update the scan state.
///
/// # Arguments
///
/// * `event` - The ScanEvent to handle
/// * `scan_state` - Arc reference to the shared ScanState
pub fn handle_scan_event(event: ScanEvent, scan_state: Arc<RwLock<ScanState>>) {
    match event {
        // Lifecycle events
        ScanEvent::ScanStarted {
            target_count,
            port_count,
            ..
        } => {
            let mut state = scan_state.write();
            state.total = (target_count * port_count) as u64;
            state.completed = 0;
            state.progress_percentage = 0.0;
        }

        ScanEvent::ScanCompleted {
            open_ports,
            closed_ports,
            filtered_ports,
            detected_services,
            ..
        } => {
            let mut state = scan_state.write();
            state.progress_percentage = 100.0;
            state.open_ports = open_ports;
            state.closed_ports = closed_ports;
            state.filtered_ports = filtered_ports;
            state.detected_services = detected_services;
        }

        ScanEvent::ScanError { .. } => {
            let mut state = scan_state.write();
            state.errors += 1;
        }

        // Progress events
        ScanEvent::ProgressUpdate {
            stage,
            percentage,
            completed,
            total,
            throughput,
            eta,
            ..
        } => {
            let mut state = scan_state.write();
            state.stage = stage;
            state.progress_percentage = percentage;
            state.completed = completed;
            state.total = total;
            state.throughput_pps = throughput.packets_per_second;
            state.eta = eta;
        }

        ScanEvent::StageChanged { to_stage, .. } => {
            let mut state = scan_state.write();
            state.stage = to_stage;
        }

        // Discovery events
        ScanEvent::HostDiscovered { ip, .. } => {
            let mut state = scan_state.write();
            if !state.discovered_hosts.contains(&ip) {
                state.discovered_hosts.push(ip);
            }
        }

        ScanEvent::PortFound { .. } => {
            let mut state = scan_state.write();
            state.open_ports += 1;
        }

        ScanEvent::ServiceDetected { .. } => {
            let mut state = scan_state.write();
            state.detected_services += 1;
        }

        // Diagnostic events
        ScanEvent::WarningIssued { message, .. } => {
            let mut state = scan_state.write();
            state.warnings.push(message);
        }

        // Other events - no state updates needed for now
        _ => {}
    }
}
