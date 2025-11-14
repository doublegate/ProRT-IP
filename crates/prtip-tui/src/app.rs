//! Main TUI application

use crossterm::event::EventStream;
use parking_lot::RwLock;
use prtip_core::event_bus::{EventBus, EventFilter};
use std::sync::Arc;
use std::time::Instant;

use crate::events::{subscribe_to_events, LoopControl};
use crate::state::{ScanState, UIState};
use crate::ui;
use crate::Result;

/// Main TUI application
///
/// The App struct manages the lifecycle of the TUI:
/// - Terminal initialization and restoration
/// - Event loop coordination
/// - State management
/// - Rendering
pub struct App {
    /// Shared scan state (thread-safe, shared with scanner)
    scan_state: Arc<RwLock<ScanState>>,

    /// Local UI state (ephemeral, TUI-only)
    ui_state: UIState,

    /// EventBus for scan events
    event_bus: Arc<EventBus>,

    /// Whether the TUI should quit
    should_quit: bool,
}

impl App {
    /// Create a new TUI application
    ///
    /// # Arguments
    ///
    /// * `event_bus` - Arc reference to the EventBus from the scanner
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use prtip_tui::App;
    /// use prtip_core::event_bus::EventBus;
    /// use std::sync::Arc;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let event_bus = Arc::new(EventBus::new(1000));
    ///     let mut app = App::new(event_bus);
    ///     app.run().await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            scan_state: ScanState::shared(),
            ui_state: UIState::new(),
            event_bus,
            should_quit: false,
        }
    }

    /// Run the TUI application
    ///
    /// This method:
    /// 1. Initializes the terminal (raw mode, alternate screen)
    /// 2. Subscribes to EventBus events
    /// 3. Runs the main event loop
    /// 4. Restores the terminal on exit
    ///
    /// # Errors
    ///
    /// Returns an error if terminal initialization fails or event handling fails.
    pub async fn run(&mut self) -> Result<()> {
        // Initialize terminal (ratatui 0.29+ handles panic hook automatically)
        let mut terminal = ratatui::init();

        // Subscribe to all EventBus events
        let mut event_rx = subscribe_to_events(Arc::clone(&self.event_bus), EventFilter::All).await;

        // Create crossterm event stream
        let mut crossterm_stream = EventStream::new();

        // Create event aggregator for rate limiting
        let mut aggregator = crate::events::EventAggregator::new();

        // FPS tracking
        let mut last_frame = Instant::now();
        let mut frame_count = 0;

        // Main event loop
        loop {
            // Render the UI
            {
                let scan_state = self.scan_state.read();
                terminal.draw(|frame| {
                    ui::render(frame, &scan_state, &self.ui_state);
                })?;
            }

            // Update FPS counter
            frame_count += 1;
            let elapsed = last_frame.elapsed();
            if elapsed.as_secs() >= 1 {
                self.ui_state.fps = frame_count as f32 / elapsed.as_secs_f32();
                frame_count = 0;
                last_frame = Instant::now();
            }

            // Process events (keyboard, EventBus, timer)
            let control = crate::events::process_events(
                Arc::clone(&self.event_bus),
                Arc::clone(&self.scan_state),
                &mut self.ui_state,
                &mut event_rx,
                &mut crossterm_stream,
                &mut aggregator,
            )
            .await;

            // Check for quit signal
            if matches!(control, LoopControl::Quit) {
                self.should_quit = true;
                break;
            }
        }

        // Restore terminal (ratatui 0.29+ handles this automatically in Drop)
        ratatui::restore();

        Ok(())
    }

    /// Check if the TUI should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Get reference to shared scan state
    pub fn scan_state(&self) -> Arc<RwLock<ScanState>> {
        Arc::clone(&self.scan_state)
    }
}
