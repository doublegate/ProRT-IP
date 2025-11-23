//! ScanEvent handler dispatch

use parking_lot::RwLock;
use prtip_core::events::ScanEvent;
use std::sync::Arc;
use std::time::Instant;

use crate::state::{
    PortDiscovery, ScanState, ServiceDetection, ThroughputSample, MAX_PORT_DISCOVERIES,
    MAX_SERVICE_DETECTIONS, MAX_THROUGHPUT_SAMPLES,
};

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

            // Add throughput sample for network activity graph
            let sample = ThroughputSample {
                timestamp: Instant::now(),
                packets_per_second: throughput.packets_per_second,
            };

            // Add to ringbuffer (pop oldest if at capacity)
            state.throughput_history.push_back(sample);
            if state.throughput_history.len() > MAX_THROUGHPUT_SAMPLES {
                state.throughput_history.pop_front();
            }
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

        ScanEvent::PortFound {
            ip,
            port,
            state: port_state,
            protocol,
            scan_type,
            timestamp,
            ..
        } => {
            let mut state = scan_state.write();
            state.open_ports += 1;

            // Create PortDiscovery entry for widget display
            let discovery = PortDiscovery {
                timestamp,
                ip,
                port,
                state: port_state.into(),
                protocol: protocol.into(),
                scan_type: scan_type.into(),
            };

            // Add to ringbuffer (pop oldest if at capacity)
            state.port_discoveries.push_back(discovery);
            if state.port_discoveries.len() > MAX_PORT_DISCOVERIES {
                state.port_discoveries.pop_front();
            }
        }

        ScanEvent::ServiceDetected {
            ip,
            port,
            service_name,
            service_version,
            confidence,
            timestamp,
            ..
        } => {
            let mut state = scan_state.write();
            state.detected_services += 1;

            // Create ServiceDetection entry for widget display
            let detection = ServiceDetection {
                timestamp,
                ip,
                port,
                service_name,
                service_version,
                confidence,
            };

            // Add to ringbuffer (pop oldest if at capacity)
            state.service_detections.push_back(detection);
            if state.service_detections.len() > MAX_SERVICE_DETECTIONS {
                state.service_detections.pop_front();
            }
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
