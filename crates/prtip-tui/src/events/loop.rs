//! Async event loop using tokio::select!

use crossterm::event::{Event as CrosstermEvent, EventStream};
use futures::StreamExt;
use parking_lot::RwLock;
use prtip_core::event_bus::EventBus;
use prtip_core::events::ScanEvent;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::events::aggregator::EventAggregator;
use crate::events::handlers::handle_scan_event;
use crate::state::{DashboardTab, ScanState, UIState};
use crate::widgets::{handle_port_table_event, handle_service_table_event};

/// Event loop states
pub enum LoopControl {
    /// Continue running
    Continue,
    /// Quit the TUI
    Quit,
}

/// Run the main TUI event loop
///
/// This function uses tokio::select! to handle three concurrent event streams:
/// 1. Keyboard/mouse events from crossterm
/// 2. ScanEvents from EventBus (with rate limiting via aggregator)
/// 3. 60 FPS tick for rendering and event flushing
///
/// # Arguments
///
/// * `event_bus` - Arc reference to the EventBus
/// * `scan_state` - Arc reference to the shared ScanState
/// * `ui_state` - Mutable reference to local UIState
/// * `event_rx` - Receiver for ScanEvents from EventBus
/// * `crossterm_rx` - Stream of crossterm events
/// * `aggregator` - Event aggregator for rate limiting
///
/// # Returns
///
/// `LoopControl` indicating whether to continue or quit
pub async fn process_events(
    _event_bus: Arc<EventBus>,
    scan_state: Arc<RwLock<ScanState>>,
    ui_state: &mut UIState,
    event_rx: &mut mpsc::UnboundedReceiver<ScanEvent>,
    crossterm_rx: &mut EventStream,
    aggregator: &mut EventAggregator,
) -> LoopControl {
    // 60 FPS = 16.67ms per frame
    let mut tick_interval = interval(Duration::from_millis(16));

    tokio::select! {
        // Handle keyboard/mouse events
        Some(Ok(crossterm_event)) = crossterm_rx.next() => {
            match crossterm_event {
                CrosstermEvent::Key(key) => {
                    use crossterm::event::{KeyCode, KeyModifiers};

                    match (key.code, key.modifiers) {
                        // Quit on 'q' or Ctrl+C
                        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            return LoopControl::Quit;
                        }

                        // Toggle help on '?'
                        (KeyCode::Char('?'), _) => {
                            ui_state.toggle_help();
                        }

                        // Tab navigation - Switch dashboard tabs (Sprint 6.2 Task 2.3)
                        (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::BackTab, _) => {
                            ui_state.next_dashboard_tab();
                        }

                        // Vim navigation (placeholder for Sprint 6.2+)
                        (KeyCode::Char('h'), _) | (KeyCode::Left, _) => {
                            // TODO: Sprint 6.2 - horizontal scroll
                        }
                        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                            ui_state.cursor_position = ui_state.cursor_position.saturating_add(1);
                        }
                        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                            ui_state.cursor_position = ui_state.cursor_position.saturating_sub(1);
                        }
                        (KeyCode::Char('l'), _) | (KeyCode::Right, _) => {
                            // TODO: Sprint 6.2 - horizontal scroll
                        }

                        // Sprint 6.2 Task 2.3-2.5: Route events to active dashboard widget
                        _ => {
                            // Route to active widget based on dashboard tab
                            let handled = match ui_state.active_dashboard_tab {
                                DashboardTab::PortTable => {
                                    handle_port_table_event(
                                        CrosstermEvent::Key(key),
                                        ui_state,
                                        &scan_state,
                                    )
                                }
                                DashboardTab::ServiceTable => {
                                    handle_service_table_event(
                                        CrosstermEvent::Key(key),
                                        ui_state,
                                        &scan_state,
                                    )
                                }
                                DashboardTab::Metrics => {
                                    // Metrics dashboard is read-only, no event handling
                                    false
                                }
                                DashboardTab::NetworkGraph => {
                                    // Network graph is read-only, no event handling
                                    false
                                }
                            };

                            // If not handled, ignore (fall through)
                            let _ = handled;
                        }
                    }
                }

                CrosstermEvent::Resize(_, _) => {
                    // Terminal resized - ratatui handles this automatically
                }

                _ => {}
            }
        }

        // Handle EventBus events (add to aggregator, don't process immediately)
        Some(scan_event) = event_rx.recv() => {
            aggregator.add_event(scan_event);
        }

        // 60 FPS tick for rendering and event flushing
        _ = tick_interval.tick() => {
            // Flush aggregated events every 16ms
            if aggregator.should_flush() {
                let (events, stats) = aggregator.flush();

                // Process buffered lifecycle/important events
                for event in events {
                    handle_scan_event(event, Arc::clone(&scan_state));
                }

                // All high-frequency events (PortFound, HostDiscovered, ServiceDetected)
                // are now buffered and handled by handle_scan_event above.
                // Stats are kept for debugging/monitoring only.

                // Update UI state with aggregator stats (for debug display)
                ui_state.aggregator_dropped_events = stats.dropped_events;
                // Note: stats.ports_found, stats.hosts_discovered, stats.services_detected
                // are available for metrics display if needed in the future.
            }
        }
    }

    LoopControl::Continue
}
