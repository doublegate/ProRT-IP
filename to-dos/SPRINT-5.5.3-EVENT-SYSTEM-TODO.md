# Sprint 5.5.3: Event System & Progress Integration - TODO

**Status:** PLANNED
**Priority:** CRITICAL (TUI prerequisite, blocks Phase 6)
**Duration Estimate:** 32-40 hours (4-5 days)
**Start Date:** [To be filled on sprint start]
**Target Completion:** [To be filled on sprint start]
**Sprint Phase:** 5.5 Pre-TUI Polish & Foundations
**ROI Score:** 9.5/10 (Critical impact, significant effort)

---

## Executive Summary

Design and implement production-grade event-driven architecture to enable real-time scan updates for TUI (Phase 6), monitoring integrations, and future distributed scanning (Phase 8). This is the **MOST CRITICAL** sprint for Phase 6 success - without this event infrastructure, TUI development cannot proceed.

**Core Objectives:**
1. Build pub-sub event bus with multi-subscriber support
2. Define comprehensive event types (lifecycle, progress, discovery, diagnostic)
3. Integrate event emission into all scanners (8 types)
4. Create real-time progress aggregator with ETA calculation
5. Convert CLI to event-driven architecture (eliminate polling)
6. Implement event logging with JSON persistence
7. Achieve <5% performance overhead with 10 subscribers

**Why This Sprint is CRITICAL:**
- **Blocks Phase 6:** TUI cannot be built without event foundation
- **Architecture Shift:** Moves from polling to event-driven (fundamental change)
- **Foundation for Future:** Enables distributed scanning (Phase 8), monitoring integrations
- **Performance Sensitive:** Must maintain scan speed while adding event overhead
- **Backward Compatibility:** Existing code must continue working during migration

---

## Context & Prerequisites

### Sprint 5.5.2 Deliverables (Prerequisites)

Sprint 5.5.2 (CLI Usability & UX) completed **15.5 hours** delivering:
- ✅ Multi-page help system with search
- ✅ Actionable error messages with solutions
- ✅ Progress indicators with ETA calculation (polling-based)
- ✅ Interactive confirmations for dangerous operations
- ✅ Scan templates for common scenarios
- ✅ Command history with replay capability

**Key Infrastructure Built:**
- `prtip-cli/src/progress.rs` (876 lines): ETA calculation, multi-stage tracking, throughput metrics
- `ScanProgress` (prtip-core): Atomic counters for thread-safe updates
- Progress display styles: minimal, standard, verbose

**Integration Points for This Sprint:**
- Sprint 5.5.2 built polling-based progress → Sprint 5.5.3 converts to event-driven
- ETA calculation logic (EWMA) → will be integrated into ProgressAggregator
- Multi-stage tracking (5 stages) → will emit StageChanged events
- Throughput metrics (pps, hpm) → will be calculated from events

### Current Architecture Limitations

**Polling-Based Progress:**
```rust
// Current approach (Sprint 5.5.2)
loop {
    let completed = progress.completed();  // Poll atomic counter
    let total = progress.total();
    let percentage = (completed as f32 / total as f32) * 100.0;
    display_progress_bar(percentage);
    sleep(Duration::from_millis(500));  // 500ms polling interval
}
```

**Problems:**
- CPU overhead: Constant polling every 500ms
- Latency: Up to 500ms delay before UI updates
- Scalability: Each UI component must poll independently
- Coupling: UI directly queries scanner state
- No history: Can't replay or query past events

**Event-Driven Architecture (This Sprint):**
```rust
// New approach (Sprint 5.5.3)
let mut event_rx = event_bus.subscribe(EventFilter::All).await;
while let Some(event) = event_rx.recv().await {
    match event {
        ScanEvent::ProgressUpdate { percentage, eta, .. } => {
            display_progress_bar(percentage, eta);  // Instant update
        }
        ScanEvent::PortFound { ip, port, state, .. } => {
            add_result_to_table(ip, port, state);
        }
        _ => {}
    }
}
```

**Benefits:**
- Zero polling overhead (event-driven)
- Instant updates (<10ms latency)
- Decoupled: UI only knows about events, not scanner internals
- Multi-subscriber: Multiple UI components listen to same events
- History: Event buffer allows replay and querying

### Why Before Phase 6 TUI

**TUI Requirements (from ratatui research):**
1. Real-time UI updates (100ms refresh rate maximum)
2. Event-driven state changes (no polling)
3. Multiple widgets listening (progress bar, results table, log viewer)
4. Background scanning (scan runs while UI renders)
5. State queryable (get current state on demand)

**Without Event System:**
- TUI forced to poll → High CPU usage, laggy UI
- No multi-widget support → Can't update multiple UI components
- Tight coupling → TUI code mixed with scanner logic
- Refactoring cost → Weeks of delay during Phase 6

**With Event System:**
- TUI subscribes to events → Zero polling, instant updates
- Each widget subscribes → Progress, results, logs all independent
- Clean separation → TUI is just an event consumer
- Phase 6 focus → Pure UI rendering, no backend work needed

### Dependencies

**Required from Sprint 5.5.2:**
- ✅ `prtip-cli/src/progress.rs` (ETA calculation logic)
- ✅ `ScanProgress` struct (atomic counters)
- ✅ Multi-stage tracking (5 stages: Initializing → Completed)

**Required from Phase 5:**
- ✅ All 8 scanner types implemented and tested
- ✅ Service detection infrastructure
- ✅ OS fingerprinting system
- ✅ TLS certificate analysis
- ✅ Plugin system (event subscribers can be plugins)

**No External Dependencies:**
- Tokio 1.35+ (already in use)
- Serde 1.0+ (already in use for serialization)
- No new crates required (pure Rust std + Tokio)

---

## Technical Architecture

### Event System Design Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Event System Architecture                    │
└─────────────────────────────────────────────────────────────────────┘

┌──────────────────┐
│   Scanners       │
│  (8 types)       │
│                  │
│  - SynScanner    │
│  - ConnectScanner│
│  - UdpScanner    │
│  - StealthScanner│
│  - IdleScanner   │
│  - DecoyScanner  │
│  - Discovery     │
│  - ServiceDetect │
└────────┬─────────┘
         │ emit events
         ▼
┌──────────────────────────────────────────────────────────────────────┐
│                           EventBus                                    │
│  ┌────────────────┐  ┌─────────────────┐  ┌──────────────────┐     │
│  │   Publishers   │  │  Event Buffer   │  │   Subscribers    │     │
│  │  (scanners)    │→│  (ring, 1000)   │→│  (consumers)     │     │
│  └────────────────┘  └─────────────────┘  └──────────────────┘     │
│                                                                       │
│  Features:                                                           │
│  - Multi-subscriber (broadcast to all)                              │
│  - Event filtering (by scan_id, type, custom predicate)            │
│  - History buffer (last 1000 events, queryable)                    │
│  - Replay capability (simulate past events)                        │
│  - Non-blocking (async dispatch)                                   │
└──────────────────────────────────────────────────────────────────────┘
         │
         ├─────────────┬────────────────┬────────────────┬─────────────┐
         ▼             ▼                ▼                ▼             ▼
┌─────────────┐ ┌──────────────┐ ┌─────────────┐ ┌──────────────┐ ┌─────────┐
│ TUI Widgets │ │  CLI Display │ │ JSON Logger │ │  Prometheus  │ │ Plugins │
│             │ │              │ │             │ │  Exporter    │ │         │
│ - Progress  │ │ - Progress   │ │ - Audit     │ │ - Metrics    │ │ - Custom│
│ - Results   │ │ - Live       │ │   trail     │ │ - Alerts     │ │   logic │
│ - Logs      │ │   results    │ │ - Replay    │ │              │ │         │
└─────────────┘ └──────────────┘ └─────────────┘ └──────────────┘ └─────────┘
```

### Event Flow Diagram

```
Scanner Thread                EventBus                  Subscriber Threads
     │                           │                              │
     │  1. Emit PortFound        │                              │
     ├──────────────────────────>│                              │
     │                           │  2. Broadcast to subscribers │
     │                           ├─────────────────────────────>│
     │                           │                              │ 3. Process event
     │                           │                              ├──> Update UI
     │                           │  4. Store in buffer          │
     │                           ├──> [Ring Buffer]             │
     │                           │                              │
     │  5. Emit ProgressUpdate   │                              │
     ├──────────────────────────>│                              │
     │                           │  6. Filter & broadcast       │
     │                           ├─────────────────────────────>│
     │                           │                              │ 7. Update progress
     │                           │                              ├──> Recalc ETA
     │                           │                              │
     │  8. Query history         │                              │
     │<──────────────────────────┤                              │
     │   [Last 1000 events]      │                              │
```

### Event Type Hierarchy

```rust
pub enum ScanEvent {
    // Lifecycle Events (6 types)
    ScanStarted { scan_id, config, target_count, timestamp },
    ScanCompleted { scan_id, duration, results, timestamp },
    ScanPaused { scan_id, reason, timestamp },
    ScanResumed { scan_id, timestamp },
    ScanCancelled { scan_id, reason, timestamp },
    ScanError { scan_id, error, recoverable, timestamp },

    // Progress Events (2 types)
    ProgressUpdate { scan_id, stage, percentage, throughput, eta, timestamp },
    StageChanged { scan_id, from_stage, to_stage, timestamp },

    // Discovery Events (6 types)
    HostDiscovered { scan_id, ip, alive, latency_ms, timestamp },
    PortFound { scan_id, ip, port, state, protocol, timestamp },
    ServiceDetected { scan_id, ip, port, service, confidence, timestamp },
    OsDetected { scan_id, ip, os, confidence, timestamp },
    BannerGrabbed { scan_id, ip, port, banner, timestamp },
    CertificateFound { scan_id, ip, port, cert, timestamp },

    // Diagnostic Events (4 types)
    RateLimitTriggered { scan_id, reason, duration, timestamp },
    RetryScheduled { scan_id, target, attempt, delay, timestamp },
    WarningIssued { scan_id, message, severity, timestamp },
    MetricRecorded { scan_id, metric, value, timestamp },
}
```

**Design Decisions:**
- All events include `scan_id: Uuid` for correlation
- All events include `timestamp: SystemTime` for ordering
- Events are `Clone + Send + Sync` for multi-threading
- Events implement `Serialize + Deserialize` for logging
- Enums over traits for type safety and exhaustiveness checking

### Threading Model

```
Main Thread                  Worker Threads (N)           Event Thread
    │                              │                          │
    │  1. Create EventBus          │                          │
    │  ├──> Arc<EventBus>          │                          │
    │                              │                          │
    │  2. Spawn scanners           │                          │
    ├─────────────────────────────>│                          │
    │                              │  3. Emit events          │
    │                              ├─────────────────────────>│
    │                              │                          │ 4. Broadcast
    │                              │                          ├──> Subscriber 1
    │                              │                          ├──> Subscriber 2
    │                              │                          ├──> Subscriber N
    │                              │                          │
    │  5. Subscribe to events      │                          │
    ├─────────────────────────────────────────────────────────>
    │  <── Event stream            │                          │
    │                              │                          │
    │  6. Wait for completion      │                          │
    │  <──────────────────────────┤                          │
```

**Thread Safety:**
- `EventBus: Arc<Mutex<EventBusInner>>` for thread-safe publishing
- `EventSubscriber: mpsc::UnboundedReceiver<ScanEvent>` for async receiving
- Events are cloned per subscriber (minimal overhead, events are small)
- No locks held during event emission (non-blocking)

### Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| **Event Latency (p99)** | <10ms | Acceptable for 100ms UI refresh |
| **Event Overhead** | <5% with 10 subscribers | Scan speed priority |
| **Memory Usage** | <10MB for buffer | 1000 events × ~10KB each |
| **Throughput** | 10,000 events/sec | 1M pps scan = 100 events/sec |
| **Subscriber Lag** | <1 second | Auto-drop slow subscribers |

**Optimization Strategies:**
- Use `Relaxed` memory ordering for metrics (non-critical)
- Unbounded channels for subscribers (no backpressure)
- Ring buffer for history (O(1) insert, bounded memory)
- Clone-on-write for events (small events, cheap to clone)
- Background thread for event dispatch (non-blocking scanners)

---

## Task Area 1: Event Type Design (4-5 hours)

### Task 1.1: Define Core Event Enum (2h)

**Description:**
Design comprehensive `ScanEvent` enum covering all scan lifecycle events. The enum must be exhaustive (cover all scanner states), minimal (only necessary data), and extensible (easy to add new types).

**Requirements:**
- 18+ event variants (lifecycle, progress, discovery, diagnostic)
- All variants include `scan_id: Uuid` and `timestamp: SystemTime`
- Event data is minimal but sufficient for consumers
- Implements `Clone + Send + Sync + Debug + Serialize + Deserialize`
- Clear documentation for each variant (when emitted, what data means)

**Files to create:**
- `crates/prtip-core/src/events.rs` (new, ~400 lines)
- `crates/prtip-core/src/events/mod.rs` (module organization)
- `crates/prtip-core/src/events/types.rs` (event enum definition)

**Implementation Steps:**
1. Create `events` module in prtip-core
2. Define `ScanEvent` enum with 18+ variants:
   - **Lifecycle:** ScanStarted, ScanCompleted, ScanPaused, ScanResumed, ScanCancelled, ScanError
   - **Progress:** ProgressUpdate, StageChanged
   - **Discovery:** HostDiscovered, PortFound, ServiceDetected, OsDetected, BannerGrabbed, CertificateFound
   - **Diagnostic:** RateLimitTriggered, RetryScheduled, WarningIssued, MetricRecorded
3. Add supporting types:
   - `ScanStage` enum (7 stages: Initializing → Completed)
   - `Throughput` struct (pps, hpm, bandwidth_mbps)
   - `PauseReason` enum (UserRequested, RateLimited, Error)
   - `WarningSeverity` enum (Low, Medium, High, Critical)
   - `MetricType` enum (PacketsSent, PacketsReceived, BytesSent, BytesReceived, RTT, etc.)
4. Implement trait bounds:
   ```rust
   #[derive(Clone, Debug, Serialize, Deserialize)]
   pub enum ScanEvent { /* ... */ }
   ```
5. Add comprehensive rustdoc for each variant

**Example Code:**
```rust
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanEvent {
    /// Scan has started
    ///
    /// Emitted once at the beginning of a scan, includes configuration
    /// and target count for consumers to initialize.
    ScanStarted {
        scan_id: Uuid,
        config: ScanConfig,
        target_count: usize,
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
        timestamp: SystemTime,
    },

    // ... more variants ...
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanStage {
    Initializing,
    ResolvingTargets,
    DiscoveringHosts,
    ScanningPorts,
    DetectingServices,
    Finalizing,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Throughput {
    pub packets_per_second: f64,
    pub hosts_per_minute: f64,
    pub bandwidth_mbps: f64,
}
```

**Acceptance Criteria:**
- [ ] `ScanEvent` enum defined with 18+ variants
- [ ] All variants include `scan_id` and `timestamp`
- [ ] Supporting types defined (ScanStage, Throughput, etc.)
- [ ] Implements `Clone + Send + Sync + Debug + Serialize + Deserialize`
- [ ] Comprehensive rustdoc (when emitted, data meaning)
- [ ] Compiles without warnings: `cargo build --package prtip-core`

**Estimated Time:** 2 hours

---

### Task 1.2: Event Metadata & Helper Methods (1h)

**Description:**
Add common metadata to all events and implement helper methods for event inspection, validation, and formatting.

**Requirements:**
- All events accessible via common `scan_id()` and `timestamp()` methods
- Event validation (e.g., percentage must be 0.0-100.0)
- Event formatting for human-readable display
- Event type extraction (discriminant without data)

**Files to modify:**
- `crates/prtip-core/src/events/types.rs` (add impl blocks)

**Implementation Steps:**
1. Add `impl ScanEvent` with common methods:
   ```rust
   impl ScanEvent {
       /// Returns the scan ID for this event
       pub fn scan_id(&self) -> Uuid {
           match self {
               ScanEvent::ScanStarted { scan_id, .. } => *scan_id,
               ScanEvent::PortFound { scan_id, .. } => *scan_id,
               // ... all variants
           }
       }

       /// Returns the timestamp for this event
       pub fn timestamp(&self) -> SystemTime {
           match self {
               ScanEvent::ScanStarted { timestamp, .. } => *timestamp,
               // ... all variants
           }
       }

       /// Returns the event type (discriminant without data)
       pub fn event_type(&self) -> ScanEventType {
           match self {
               ScanEvent::ScanStarted { .. } => ScanEventType::ScanStarted,
               ScanEvent::PortFound { .. } => ScanEventType::PortFound,
               // ... all variants
           }
       }

       /// Validates event data (sanity checks)
       pub fn validate(&self) -> Result<(), ValidationError> {
           match self {
               ScanEvent::ProgressUpdate { percentage, .. } => {
                   if *percentage < 0.0 || *percentage > 100.0 {
                       return Err(ValidationError::InvalidPercentage(*percentage));
                   }
               }
               // ... more validations
           }
           Ok(())
       }

       /// Format event for human-readable display
       pub fn display(&self) -> String {
           match self {
               ScanEvent::PortFound { ip, port, state, .. } => {
                   format!("Port {}/tcp {} on {}", port, state, ip)
               }
               // ... all variants
           }
       }
   }
   ```

2. Add `ScanEventType` enum (lightweight discriminant):
   ```rust
   #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
   pub enum ScanEventType {
       ScanStarted,
       ScanCompleted,
       ProgressUpdate,
       PortFound,
       // ... all variants
   }
   ```

3. Add `ValidationError` enum for validation failures

**Acceptance Criteria:**
- [ ] `scan_id()` and `timestamp()` methods implemented for all variants
- [ ] `event_type()` returns lightweight discriminant
- [ ] `validate()` checks sanity (percentage range, non-negative counts, etc.)
- [ ] `display()` provides human-readable formatting
- [ ] All methods tested with unit tests

**Estimated Time:** 1 hour

---

### Task 1.3: Event Serialization & Testing (1-2h)

**Description:**
Implement JSON serialization for events (logging, replay) and create comprehensive unit tests for all event types.

**Requirements:**
- JSON serialization works for all variants
- Deserialization round-trips correctly
- Unit tests for each event variant (20+ tests)
- Property-based testing for validation logic

**Files to create:**
- `crates/prtip-core/src/events/tests.rs` (unit tests)

**Implementation Steps:**
1. Test serialization round-trips:
   ```rust
   #[test]
   fn test_scan_started_serialization() {
       let event = ScanEvent::ScanStarted {
           scan_id: Uuid::new_v4(),
           config: ScanConfig::default(),
           target_count: 1000,
           timestamp: SystemTime::now(),
       };

       let json = serde_json::to_string(&event).unwrap();
       let deserialized: ScanEvent = serde_json::from_str(&json).unwrap();

       assert_eq!(event.scan_id(), deserialized.scan_id());
   }
   ```

2. Test validation logic:
   ```rust
   #[test]
   fn test_progress_update_validation() {
       let event = ScanEvent::ProgressUpdate {
           scan_id: Uuid::new_v4(),
           stage: ScanStage::ScanningPorts,
           percentage: 150.0, // Invalid!
           throughput: Throughput::default(),
           eta: None,
           timestamp: SystemTime::now(),
       };

       assert!(event.validate().is_err());
   }
   ```

3. Test helper methods for all variants

4. Add property-based tests with `proptest`:
   ```rust
   proptest! {
       #[test]
       fn test_percentage_always_valid(percentage in 0.0f32..100.0f32) {
           let event = ScanEvent::ProgressUpdate {
               percentage,
               // ... other fields
           };
           assert!(event.validate().is_ok());
       }
   }
   ```

**Acceptance Criteria:**
- [ ] All event variants serialize/deserialize correctly (JSON)
- [ ] 20+ unit tests covering all variants
- [ ] Validation logic tested (valid and invalid inputs)
- [ ] Helper methods tested (scan_id, timestamp, event_type, display)
- [ ] Tests pass: `cargo test --package prtip-core events`

**Estimated Time:** 1-2 hours

---

## Task Area 2: Pub-Sub Event Bus (8-10 hours)

### Task 2.1: EventBus Architecture & Core Structure (2h)

**Description:**
Design and implement the core `EventBus` structure with thread-safe publishing and subscriber management.

**Requirements:**
- Thread-safe event bus (Arc<Mutex<>> or RwLock)
- Subscriber registry with UUID-based identification
- Event buffer (ring buffer, last 1000 events)
- Metrics tracking (total events published, active subscribers)

**Files to create:**
- `crates/prtip-core/src/event_bus.rs` (new, ~600 lines)
- `crates/prtip-core/src/event_bus/mod.rs` (module organization)
- `crates/prtip-core/src/event_bus/bus.rs` (EventBus implementation)

**Implementation Steps:**
1. Define core structures:
   ```rust
   use std::collections::HashMap;
   use std::sync::{Arc, Mutex};
   use tokio::sync::mpsc;
   use uuid::Uuid;

   pub struct EventBus {
       inner: Arc<Mutex<EventBusInner>>,
   }

   struct EventBusInner {
       subscribers: HashMap<Uuid, EventSubscriber>,
       buffer: RingBuffer<ScanEvent>, // Last 1000 events
       metrics: EventBusMetrics,
   }

   struct EventSubscriber {
       id: Uuid,
       filter: EventFilter,
       sender: mpsc::UnboundedSender<ScanEvent>,
   }

   #[derive(Clone)]
   pub enum EventFilter {
       All,
       ScanId(Uuid),
       EventType(Vec<ScanEventType>),
       Custom(Arc<dyn Fn(&ScanEvent) -> bool + Send + Sync>),
   }

   struct EventBusMetrics {
       total_published: usize,
       active_subscribers: usize,
       buffer_overflow_count: usize,
   }
   ```

2. Implement `EventBus::new()`:
   ```rust
   impl EventBus {
       pub fn new() -> Self {
           Self {
               inner: Arc::new(Mutex::new(EventBusInner {
                   subscribers: HashMap::new(),
                   buffer: RingBuffer::new(1000),
                   metrics: EventBusMetrics::default(),
               })),
           }
       }
   }
   ```

3. Implement `Clone` for `EventBus` (cheap Arc clone)

4. Add metrics accessors:
   ```rust
   impl EventBus {
       pub fn metrics(&self) -> EventBusMetrics {
           let inner = self.inner.lock().unwrap();
           inner.metrics.clone()
       }
   }
   ```

**Acceptance Criteria:**
- [ ] `EventBus` struct defined with thread-safe interior
- [ ] `EventSubscriber` struct with UUID, filter, sender
- [ ] `EventFilter` enum with 4 variants (All, ScanId, EventType, Custom)
- [ ] Ring buffer for 1000 events
- [ ] Metrics tracking structure
- [ ] Compiles without warnings

**Estimated Time:** 2 hours

---

### Task 2.2: Publish Method Implementation (1-2h)

**Description:**
Implement event publishing with broadcasting to all matching subscribers and history buffering.

**Requirements:**
- Non-blocking publish (async dispatch)
- Broadcast to all matching subscribers (filter check)
- Store in history buffer (ring buffer, drop oldest)
- Increment metrics (total published)
- Handle subscriber send failures gracefully

**Files to modify:**
- `crates/prtip-core/src/event_bus/bus.rs`

**Implementation Steps:**
1. Implement `EventBus::publish()`:
   ```rust
   impl EventBus {
       pub fn publish(&self, event: ScanEvent) {
           let mut inner = self.inner.lock().unwrap();

           // 1. Store in history buffer
           inner.buffer.push(event.clone());

           // 2. Broadcast to matching subscribers
           let mut failed_subscribers = Vec::new();
           for (id, subscriber) in &inner.subscribers {
               if subscriber.filter.matches(&event) {
                   if subscriber.sender.send(event.clone()).is_err() {
                       // Subscriber disconnected, mark for removal
                       failed_subscribers.push(*id);
                   }
               }
           }

           // 3. Remove failed subscribers
           for id in failed_subscribers {
               inner.subscribers.remove(&id);
               inner.metrics.active_subscribers -= 1;
           }

           // 4. Update metrics
           inner.metrics.total_published += 1;
       }
   }
   ```

2. Implement `EventFilter::matches()`:
   ```rust
   impl EventFilter {
       fn matches(&self, event: &ScanEvent) -> bool {
           match self {
               EventFilter::All => true,
               EventFilter::ScanId(id) => event.scan_id() == *id,
               EventFilter::EventType(types) => {
                   types.contains(&event.event_type())
               }
               EventFilter::Custom(predicate) => predicate(event),
           }
       }
   }
   ```

3. Add error handling for buffer overflow:
   ```rust
   if inner.buffer.is_full() {
       inner.metrics.buffer_overflow_count += 1;
       // Oldest event is automatically dropped (ring buffer)
   }
   ```

**Acceptance Criteria:**
- [ ] `publish()` broadcasts to all matching subscribers
- [ ] Events stored in history buffer (ring buffer)
- [ ] Failed subscribers auto-removed
- [ ] Metrics updated (total_published, active_subscribers)
- [ ] Non-blocking (no async sleeps or waits)
- [ ] Unit tests verify broadcasting and filtering

**Estimated Time:** 1-2 hours

---

### Task 2.3: Subscribe Method Implementation (2h)

**Description:**
Implement subscription mechanism allowing consumers to receive events via async channels.

**Requirements:**
- Returns `mpsc::UnboundedReceiver<ScanEvent>` for async receiving
- Supports event filtering (All, ScanId, EventType, Custom)
- Thread-safe subscriber registration
- Returns subscription ID for unsubscribe

**Files to modify:**
- `crates/prtip-core/src/event_bus/bus.rs`

**Implementation Steps:**
1. Implement `EventBus::subscribe()`:
   ```rust
   impl EventBus {
       pub fn subscribe(&self, filter: EventFilter) -> (Uuid, mpsc::UnboundedReceiver<ScanEvent>) {
           let (tx, rx) = mpsc::unbounded_channel();
           let id = Uuid::new_v4();

           let mut inner = self.inner.lock().unwrap();
           inner.subscribers.insert(
               id,
               EventSubscriber {
                   id,
                   filter,
                   sender: tx,
               },
           );
           inner.metrics.active_subscribers += 1;

           (id, rx)
       }
   }
   ```

2. Implement `EventBus::unsubscribe()`:
   ```rust
   impl EventBus {
       pub fn unsubscribe(&self, id: Uuid) -> bool {
           let mut inner = self.inner.lock().unwrap();
           if inner.subscribers.remove(&id).is_some() {
               inner.metrics.active_subscribers -= 1;
               true
           } else {
               false
           }
       }
   }
   ```

3. Add convenience methods:
   ```rust
   impl EventBus {
       /// Subscribe to all events
       pub fn subscribe_all(&self) -> (Uuid, mpsc::UnboundedReceiver<ScanEvent>) {
           self.subscribe(EventFilter::All)
       }

       /// Subscribe to specific scan
       pub fn subscribe_scan(&self, scan_id: Uuid) -> (Uuid, mpsc::UnboundedReceiver<ScanEvent>) {
           self.subscribe(EventFilter::ScanId(scan_id))
       }

       /// Subscribe to specific event types
       pub fn subscribe_types(&self, types: Vec<ScanEventType>) -> (Uuid, mpsc::UnboundedReceiver<ScanEvent>) {
           self.subscribe(EventFilter::EventType(types))
       }
   }
   ```

**Acceptance Criteria:**
- [ ] `subscribe()` returns (Uuid, UnboundedReceiver)
- [ ] Subscribers registered in thread-safe map
- [ ] Filters applied during publish (not subscribe)
- [ ] `unsubscribe()` removes subscriber and updates metrics
- [ ] Convenience methods for common filters
- [ ] Unit tests verify subscription and receiving

**Estimated Time:** 2 hours

---

### Task 2.4: Event History & Replay (2-3h)

**Description:**
Implement event history querying and replay capability for debugging and visualization.

**Requirements:**
- Query last N events (optionally filtered)
- Replay events within time range
- Stream historical events as if real-time
- Efficient ring buffer implementation

**Files to create:**
- `crates/prtip-core/src/event_bus/ring_buffer.rs` (ring buffer implementation)

**Files to modify:**
- `crates/prtip-core/src/event_bus/bus.rs` (add history methods)

**Implementation Steps:**
1. Implement `RingBuffer<T>`:
   ```rust
   pub struct RingBuffer<T> {
       buffer: Vec<Option<T>>,
       capacity: usize,
       head: usize,
       count: usize,
   }

   impl<T: Clone> RingBuffer<T> {
       pub fn new(capacity: usize) -> Self {
           Self {
               buffer: (0..capacity).map(|_| None).collect(),
               capacity,
               head: 0,
               count: 0,
           }
       }

       pub fn push(&mut self, item: T) {
           self.buffer[self.head] = Some(item);
           self.head = (self.head + 1) % self.capacity;
           if self.count < self.capacity {
               self.count += 1;
           }
       }

       pub fn iter(&self) -> impl Iterator<Item = &T> {
           let start = if self.count < self.capacity {
               0
           } else {
               self.head
           };
           (0..self.count)
               .map(move |i| (start + i) % self.capacity)
               .filter_map(move |idx| self.buffer[idx].as_ref())
       }
   }
   ```

2. Implement `EventBus::history()`:
   ```rust
   impl EventBus {
       pub fn history(&self, filter: EventFilter, limit: usize) -> Vec<ScanEvent> {
           let inner = self.inner.lock().unwrap();
           inner.buffer.iter()
               .filter(|event| filter.matches(event))
               .take(limit)
               .cloned()
               .collect()
       }
   }
   ```

3. Implement `EventBus::replay()` (async stream):
   ```rust
   use tokio_stream::wrappers::UnboundedReceiverStream;

   impl EventBus {
       pub fn replay(
           &self,
           from: SystemTime,
           to: SystemTime,
       ) -> UnboundedReceiverStream<ScanEvent> {
           let (tx, rx) = mpsc::unbounded_channel();
           let events: Vec<_> = self.history(EventFilter::All, usize::MAX)
               .into_iter()
               .filter(|e| {
                   let ts = e.timestamp();
                   ts >= from && ts <= to
               })
               .collect();

           tokio::spawn(async move {
               for event in events {
                   if tx.send(event).is_err() {
                       break;
                   }
                   // Optional: delay to simulate real-time
                   // tokio::time::sleep(Duration::from_millis(10)).await;
               }
           });

           UnboundedReceiverStream::new(rx)
       }
   }
   ```

**Acceptance Criteria:**
- [ ] `RingBuffer` implemented with O(1) push and iteration
- [ ] `history()` returns last N events (filtered)
- [ ] `replay()` streams events within time range
- [ ] Unit tests verify ring buffer wraparound
- [ ] Integration tests verify history querying

**Estimated Time:** 2-3 hours

---

### Task 2.5: Multi-Subscriber Testing (1-2h)

**Description:**
Test that event bus correctly handles multiple simultaneous subscribers with different filters.

**Requirements:**
- Test 10+ concurrent subscribers
- Test different filters per subscriber
- Test subscriber removal during event emission
- Test slow subscribers (don't block fast ones)

**Files to create:**
- `crates/prtip-core/src/event_bus/tests.rs`

**Implementation Steps:**
1. Test multi-subscriber broadcast:
   ```rust
   #[tokio::test]
   async fn test_multi_subscriber_broadcast() {
       let bus = EventBus::new();
       let mut receivers = Vec::new();

       // Subscribe 10 times
       for _ in 0..10 {
           let (_, rx) = bus.subscribe_all();
           receivers.push(rx);
       }

       // Publish event
       let event = ScanEvent::PortFound { /* ... */ };
       bus.publish(event.clone());

       // All subscribers should receive
       for rx in &mut receivers {
           let received = rx.recv().await.unwrap();
           assert_eq!(received.scan_id(), event.scan_id());
       }
   }
   ```

2. Test filtered subscribers:
   ```rust
   #[tokio::test]
   async fn test_filtered_subscribers() {
       let bus = EventBus::new();
       let scan_id = Uuid::new_v4();

       let (_, mut rx_all) = bus.subscribe_all();
       let (_, mut rx_filtered) = bus.subscribe_scan(scan_id);

       // Publish matching event
       bus.publish(ScanEvent::PortFound { scan_id, /* ... */ });

       // Both should receive
       assert!(rx_all.recv().await.is_some());
       assert!(rx_filtered.recv().await.is_some());

       // Publish non-matching event
       bus.publish(ScanEvent::PortFound { scan_id: Uuid::new_v4(), /* ... */ });

       // Only rx_all should receive
       assert!(rx_all.try_recv().is_ok());
       assert!(rx_filtered.try_recv().is_err());
   }
   ```

3. Test subscriber removal:
   ```rust
   #[tokio::test]
   async fn test_subscriber_removal() {
       let bus = EventBus::new();
       let (id, mut rx) = bus.subscribe_all();

       // Unsubscribe
       assert!(bus.unsubscribe(id));

       // Publish event
       bus.publish(ScanEvent::PortFound { /* ... */ });

       // Should not receive (channel closed)
       assert!(rx.recv().await.is_none());
   }
   ```

4. Test slow subscriber handling:
   ```rust
   #[tokio::test]
   async fn test_slow_subscriber_does_not_block() {
       let bus = EventBus::new();
       let (_id, _rx) = bus.subscribe_all();
       // Don't receive from _rx (simulate slow subscriber)

       let start = Instant::now();

       // Publish 1000 events
       for i in 0..1000 {
           bus.publish(ScanEvent::ProgressUpdate {
               percentage: i as f32 / 10.0,
               /* ... */
           });
       }

       let elapsed = start.elapsed();

       // Should be fast (<100ms) even with slow subscriber
       assert!(elapsed < Duration::from_millis(100));
   }
   ```

**Acceptance Criteria:**
- [ ] Multi-subscriber test passes (10+ subscribers)
- [ ] Filtered subscribers receive only matching events
- [ ] Subscriber removal prevents future events
- [ ] Slow subscribers don't block publishing
- [ ] All tests pass: `cargo test event_bus`

**Estimated Time:** 1-2 hours

---

### Task 2.6: Performance Benchmarking (1-2h)

**Description:**
Benchmark event bus performance to ensure <5% overhead with 10 subscribers.

**Requirements:**
- Measure publish latency (p50, p95, p99)
- Measure subscriber delivery latency
- Measure memory usage (buffer + channels)
- Measure throughput (events/sec)
- Compare against baseline (no events)

**Files to create:**
- `benches/event_bus_bench.rs` (Criterion benchmarks)

**Implementation Steps:**
1. Benchmark publish latency:
   ```rust
   use criterion::{black_box, criterion_group, criterion_main, Criterion};

   fn bench_publish_latency(c: &mut Criterion) {
       let bus = EventBus::new();
       let event = ScanEvent::PortFound { /* ... */ };

       c.bench_function("publish_no_subscribers", |b| {
           b.iter(|| {
               bus.publish(black_box(event.clone()));
           });
       });

       let mut _subscribers = Vec::new();
       for _ in 0..10 {
           let (_, rx) = bus.subscribe_all();
           _subscribers.push(rx);
       }

       c.bench_function("publish_10_subscribers", |b| {
           b.iter(|| {
               bus.publish(black_box(event.clone()));
           });
       });
   }
   ```

2. Benchmark throughput:
   ```rust
   fn bench_throughput(c: &mut Criterion) {
       let bus = EventBus::new();

       c.bench_function("throughput_10k_events", |b| {
           b.iter(|| {
               for i in 0..10_000 {
                   bus.publish(ScanEvent::ProgressUpdate {
                       percentage: (i % 100) as f32,
                       /* ... */
                   });
               }
           });
       });
   }
   ```

3. Measure memory usage:
   ```rust
   #[test]
   fn test_memory_usage() {
       let bus = EventBus::new();

       // Fill buffer
       for i in 0..1000 {
           bus.publish(ScanEvent::ProgressUpdate {
               percentage: i as f32 / 10.0,
               /* ... */
           });
       }

       let metrics = bus.metrics();
       assert_eq!(metrics.total_published, 1000);

       // Memory should be bounded (ring buffer)
       // Approximate check: <10MB for 1000 events
       let event_size = std::mem::size_of::<ScanEvent>();
       let buffer_size = event_size * 1000;
       assert!(buffer_size < 10_000_000); // <10MB
   }
   ```

**Acceptance Criteria:**
- [ ] Publish latency benchmarked (p50, p95, p99)
- [ ] Throughput benchmarked (events/sec)
- [ ] Memory usage validated (<10MB for buffer)
- [ ] Performance targets met:
  - [ ] p99 publish latency <10ms
  - [ ] Throughput >10,000 events/sec
  - [ ] <5% overhead vs. baseline
- [ ] Benchmarks run with: `cargo bench event_bus`

**Estimated Time:** 1-2 hours

---

## Task Area 3: Scanner Integration (6-8 hours)

### Task 3.1: Add EventBus to ScanConfig (1h)

**Description:**
Integrate `EventBus` into `ScanConfig` to make it available to all scanners, maintaining backward compatibility.

**Requirements:**
- `ScanConfig` includes optional `Arc<EventBus>`
- Backward compatible (existing code works without event bus)
- Thread-safe sharing via Arc

**Files to modify:**
- `crates/prtip-core/src/config.rs` (add event_bus field)
- `crates/prtip-core/src/lib.rs` (export EventBus)

**Implementation Steps:**
1. Add field to `ScanConfig`:
   ```rust
   pub struct ScanConfig {
       // ... existing fields ...
       
       /// Optional event bus for real-time progress updates
       ///
       /// If provided, scanners will emit events to this bus.
       /// If None, scanners operate in legacy mode (atomic counters only).
       pub event_bus: Option<Arc<EventBus>>,
   }
   ```

2. Update `Default` implementation:
   ```rust
   impl Default for ScanConfig {
       fn default() -> Self {
           Self {
               // ... existing defaults ...
               event_bus: None, // Backward compatible
           }
       }
   }
   ```

3. Add builder method:
   ```rust
   impl ScanConfig {
       pub fn with_event_bus(mut self, bus: Arc<EventBus>) -> Self {
           self.event_bus = Some(bus);
           self
       }
   }
   ```

4. Export EventBus from prtip-core:
   ```rust
   // crates/prtip-core/src/lib.rs
   pub mod event_bus;
   pub mod events;

   pub use event_bus::EventBus;
   pub use events::ScanEvent;
   ```

**Acceptance Criteria:**
- [ ] `ScanConfig::event_bus` field added (optional)
- [ ] Backward compatible (None by default)
- [ ] Builder method `with_event_bus()` added
- [ ] EventBus exported from prtip-core
- [ ] Compiles without warnings

**Estimated Time:** 1 hour

---

### Task 3.2: Emit Events from SynScanner (1-2h)

**Description:**
Modify `SynScanner` to emit events at key points: scan start, port discovery, progress updates, scan completion.

**Requirements:**
- Emit `ScanStarted` on scan initialization
- Emit `PortFound` when port discovered
- Emit `ProgressUpdate` every 5% completion
- Emit `ScanCompleted` on finish
- Maintain atomic counter updates (dual-write)

**Files to modify:**
- `crates/prtip-scanner/src/syn_scanner.rs`

**Implementation Steps:**
1. Emit `ScanStarted`:
   ```rust
   impl SynScanner {
       pub async fn scan(&mut self, targets: Vec<SocketAddr>) -> Result<Vec<ScanResult>> {
           let scan_id = Uuid::new_v4();
           
           if let Some(bus) = &self.config.event_bus {
               bus.publish(ScanEvent::ScanStarted {
                   scan_id,
                   config: self.config.clone(),
                   target_count: targets.len(),
                   timestamp: SystemTime::now(),
               });
           }

           // ... existing scan logic ...
       }
   }
   ```

2. Emit `PortFound`:
   ```rust
   // Inside scan loop when port discovered
   if response_indicates_open(&packet) {
       self.progress.increment_open(); // Keep atomic counter
       
       if let Some(bus) = &self.config.event_bus {
           bus.publish(ScanEvent::PortFound {
               scan_id,
               ip: target.ip(),
               port: target.port(),
               state: PortState::Open,
               protocol: Protocol::Tcp,
               timestamp: SystemTime::now(),
           });
       }
   }
   ```

3. Emit `ProgressUpdate` (every 5%):
   ```rust
   let last_reported = AtomicUsize::new(0);
   
   // Inside scan loop
   let completed = self.progress.completed();
   let total = self.progress.total();
   let percentage = (completed as f32 / total as f32) * 100.0;
   
   // Report every 5%
   if percentage as usize / 5 > last_reported.load(Ordering::Relaxed) {
       last_reported.store(percentage as usize / 5, Ordering::Relaxed);
       
       if let Some(bus) = &self.config.event_bus {
           bus.publish(ScanEvent::ProgressUpdate {
               scan_id,
               stage: ScanStage::ScanningPorts,
               percentage,
               throughput: calculate_throughput(),
               eta: calculate_eta(),
               timestamp: SystemTime::now(),
           });
       }
   }
   ```

4. Emit `ScanCompleted`:
   ```rust
   // After scan loop completes
   if let Some(bus) = &self.config.event_bus {
       bus.publish(ScanEvent::ScanCompleted {
           scan_id,
           duration: start_time.elapsed(),
           results: ScanSummary {
               total_targets: targets.len(),
               open_ports: self.progress.open_ports(),
               closed_ports: self.progress.closed_ports(),
               filtered_ports: self.progress.filtered_ports(),
           },
           timestamp: SystemTime::now(),
       });
   }
   ```

**Acceptance Criteria:**
- [ ] `ScanStarted` emitted at scan initialization
- [ ] `PortFound` emitted for each discovered port
- [ ] `ProgressUpdate` emitted every 5% completion
- [ ] `ScanCompleted` emitted on finish
- [ ] Atomic counters still updated (dual-write)
- [ ] Integration test verifies events emitted

**Estimated Time:** 1-2 hours

---

### Task 3.3: Emit Events from Remaining Scanners (2-3h)

**Description:**
Replicate event emission pattern from SynScanner to all remaining scanner types: ConnectScanner, UdpScanner, StealthScanner, IdleScanner, DecoyScanner, DiscoveryEngine.

**Requirements:**
- All 7 remaining scanners emit standard events
- Consistent event patterns across scanners
- Scanner-specific events where appropriate (e.g., zombie discovery for IdleScanner)

**Files to modify:**
- `crates/prtip-scanner/src/tcp_connect_scanner.rs`
- `crates/prtip-scanner/src/udp_scanner.rs`
- `crates/prtip-scanner/src/stealth_scanner.rs`
- `crates/prtip-scanner/src/idle/idle_scanner.rs`
- `crates/prtip-scanner/src/decoy_scanner.rs`
- `crates/prtip-scanner/src/discovery_engine.rs`

**Implementation Steps:**
1. For each scanner, add event emission at key points:
   - **Start:** `ScanStarted` with config and target count
   - **Discovery:** `PortFound`, `HostDiscovered` (for Discovery scanner)
   - **Progress:** `ProgressUpdate` every 5%
   - **Completion:** `ScanCompleted` with summary

2. Add scanner-specific events:
   - **IdleScanner:** `ZombieDiscovered` when suitable zombie found
   - **DiscoveryEngine:** `HostDiscovered` for each alive host
   - **UdpScanner:** `PortFound` with protocol UDP
   - **StealthScanner:** `PortFound` with scan type in metadata

3. Create helper function to reduce code duplication:
   ```rust
   fn emit_event_if_enabled(bus: &Option<Arc<EventBus>>, event: ScanEvent) {
       if let Some(bus) = bus {
           bus.publish(event);
       }
   }
   ```

4. Test each scanner independently:
   ```rust
   #[tokio::test]
   async fn test_tcp_connect_scanner_emits_events() {
       let bus = Arc::new(EventBus::new());
       let (_, mut rx) = bus.subscribe_all();
       
       let config = ScanConfig::default().with_event_bus(bus);
       let mut scanner = TcpConnectScanner::new(config);
       
       // Run scan
       scanner.scan(vec![/* targets */]).await.unwrap();
       
       // Verify events received
       let events: Vec<_> = collect_events(&mut rx).await;
       assert!(events.iter().any(|e| matches!(e, ScanEvent::ScanStarted { .. })));
       assert!(events.iter().any(|e| matches!(e, ScanEvent::ScanCompleted { .. })));
   }
   ```

**Acceptance Criteria:**
- [ ] All 7 scanners emit events at key points
- [ ] Consistent event patterns across scanners
- [ ] Scanner-specific events implemented (IdleScanner, DiscoveryEngine)
- [ ] Integration tests for each scanner
- [ ] All tests pass: `cargo test --package prtip-scanner`

**Estimated Time:** 2-3 hours

---

### Task 3.4: Emit Events from ServiceDetector & OsFingerprinter (1-2h)

**Description:**
Add event emission to high-level detection systems: service detection and OS fingerprinting.

**Requirements:**
- `ServiceDetected` event when service identified
- `OsDetected` event when OS fingerprinted
- `BannerGrabbed` event when banner retrieved
- `CertificateFound` event when TLS cert extracted

**Files to modify:**
- `crates/prtip-scanner/src/service_detector.rs`
- `crates/prtip-scanner/src/os_fingerprinter.rs`
- `crates/prtip-scanner/src/tls_certificate.rs`

**Implementation Steps:**
1. ServiceDetector events:
   ```rust
   impl ServiceDetector {
       pub async fn detect(&self, ip: IpAddr, port: u16) -> Result<Option<ServiceInfo>> {
           let service = self.probe_service(ip, port).await?;
           
           if let Some(service) = &service {
               if let Some(bus) = &self.config.event_bus {
                   bus.publish(ScanEvent::ServiceDetected {
                       scan_id: self.scan_id,
                       ip,
                       port,
                       service: service.clone(),
                       confidence: service.confidence,
                       timestamp: SystemTime::now(),
                   });
               }
           }
           
           Ok(service)
       }
   }
   ```

2. OsFingerprinter events:
   ```rust
   impl OsFingerprinter {
       pub async fn fingerprint(&self, ip: IpAddr) -> Result<Option<OsInfo>> {
           let os_info = self.probe_os(ip).await?;
           
           if let Some(os) = &os_info {
               if let Some(bus) = &self.config.event_bus {
                   bus.publish(ScanEvent::OsDetected {
                       scan_id: self.scan_id,
                       ip,
                       os: os.clone(),
                       confidence: os.confidence,
                       timestamp: SystemTime::now(),
                   });
               }
           }
           
           Ok(os_info)
       }
   }
   ```

3. TLS certificate events:
   ```rust
   impl TlsCertificateAnalyzer {
       pub async fn analyze(&self, ip: IpAddr, port: u16) -> Result<Option<CertificateInfo>> {
           let cert = self.extract_certificate(ip, port).await?;
           
           if let Some(cert) = &cert {
               if let Some(bus) = &self.config.event_bus {
                   bus.publish(ScanEvent::CertificateFound {
                       scan_id: self.scan_id,
                       ip,
                       port,
                       cert: cert.clone(),
                       timestamp: SystemTime::now(),
                   });
               }
           }
           
           Ok(cert)
       }
   }
   ```

4. Banner grabbing events:
   ```rust
   impl BannerGrabber {
       pub async fn grab_banner(&self, ip: IpAddr, port: u16) -> Result<String> {
           let banner = self.connect_and_read(ip, port).await?;
           
           if let Some(bus) = &self.config.event_bus {
               bus.publish(ScanEvent::BannerGrabbed {
                   scan_id: self.scan_id,
                   ip,
                   port,
                   banner: banner.clone(),
                   timestamp: SystemTime::now(),
               });
           }
           
           Ok(banner)
       }
   }
   ```

**Acceptance Criteria:**
- [ ] `ServiceDetected` emitted on successful detection
- [ ] `OsDetected` emitted on successful fingerprinting
- [ ] `BannerGrabbed` emitted on banner retrieval
- [ ] `CertificateFound` emitted on TLS cert extraction
- [ ] Integration tests verify events emitted
- [ ] All tests pass

**Estimated Time:** 1-2 hours

---

### Task 3.5: Integration Testing & Verification (1h)

**Description:**
Create comprehensive integration tests verifying end-to-end event flow from scanner through event bus to consumers.

**Requirements:**
- Test full scan emits all expected events
- Test event ordering (ScanStarted → ProgressUpdate → ... → ScanCompleted)
- Test multi-scanner event correlation (same scan_id)
- Test backward compatibility (scan works without event bus)

**Files to create:**
- `crates/prtip-scanner/tests/event_integration.rs`

**Implementation Steps:**
1. Test full scan event flow:
   ```rust
   #[tokio::test]
   async fn test_full_scan_event_flow() {
       let bus = Arc::new(EventBus::new());
       let (_, mut rx) = bus.subscribe_all();
       
       let config = ScanConfig::default().with_event_bus(bus);
       let mut scanner = SynScanner::new(config);
       
       let targets = vec![
           "127.0.0.1:80".parse().unwrap(),
           "127.0.0.1:443".parse().unwrap(),
       ];
       
       // Run scan in background
       let scan_task = tokio::spawn(async move {
           scanner.scan(targets).await.unwrap()
       });
       
       // Collect events
       let mut events = Vec::new();
       while let Some(event) = rx.recv().await {
           events.push(event.clone());
           if matches!(event, ScanEvent::ScanCompleted { .. }) {
               break;
           }
       }
       
       scan_task.await.unwrap();
       
       // Verify event sequence
       assert!(matches!(events[0], ScanEvent::ScanStarted { .. }));
       assert!(matches!(events.last().unwrap(), ScanEvent::ScanCompleted { .. }));
       
       // Verify all events have same scan_id
       let scan_id = events[0].scan_id();
       assert!(events.iter().all(|e| e.scan_id() == scan_id));
   }
   ```

2. Test backward compatibility:
   ```rust
   #[tokio::test]
   async fn test_scan_without_event_bus() {
       // No event bus
       let config = ScanConfig::default();
       let mut scanner = SynScanner::new(config);
       
       let targets = vec!["127.0.0.1:80".parse().unwrap()];
       
       // Should work fine
       let results = scanner.scan(targets).await.unwrap();
       assert!(!results.is_empty());
   }
   ```

3. Test event filtering:
   ```rust
   #[tokio::test]
   async fn test_event_filtering() {
       let bus = Arc::new(EventBus::new());
       let (_, mut rx_all) = bus.subscribe_all();
       let (_, mut rx_discovery) = bus.subscribe_types(vec![
           ScanEventType::PortFound,
           ScanEventType::HostDiscovered,
       ]);
       
       let config = ScanConfig::default().with_event_bus(bus);
       let mut scanner = SynScanner::new(config);
       
       scanner.scan(vec![/* ... */]).await.unwrap();
       
       // rx_all should receive all events
       let all_events = collect_events(&mut rx_all).await;
       assert!(all_events.len() > 5);
       
       // rx_discovery should receive only discovery events
       let discovery_events = collect_events(&mut rx_discovery).await;
       assert!(discovery_events.iter().all(|e| {
           matches!(e, ScanEvent::PortFound { .. } | ScanEvent::HostDiscovered { .. })
       }));
   }
   ```

**Acceptance Criteria:**
- [ ] Full scan event flow tested (start → progress → complete)
- [ ] Event ordering verified (correct sequence)
- [ ] Event correlation verified (same scan_id)
- [ ] Backward compatibility verified (works without event bus)
- [ ] Event filtering tested
- [ ] All integration tests pass

**Estimated Time:** 1 hour

---

## Task Area 4: Real-Time Progress Collection (6-8 hours)

### Task 4.1: Progress Aggregator Architecture (2-3h)

**Description:**
Create `ProgressAggregator` that subscribes to events and maintains real-time aggregated state.

**Requirements:**
- Subscribe to progress-related events
- Maintain queryable state (progress, throughput, counts)
- Non-blocking state reads
- Background task for state updates

**Files to create:**
- `crates/prtip-core/src/progress_aggregator.rs` (new, ~500 lines)

**Implementation Steps:**
1. Define aggregated state:
   ```rust
   use std::sync::{Arc, RwLock};
   use tokio::task::JoinHandle;

   pub struct ProgressAggregator {
       event_bus: Arc<EventBus>,
       state: Arc<RwLock<AggregatedState>>,
       _updater_task: JoinHandle<()>,
   }

   #[derive(Clone, Debug)]
   pub struct AggregatedState {
       pub scan_id: Option<Uuid>,
       pub current_stage: ScanStage,
       pub overall_progress: f32,
       pub stage_progress: f32,
       pub throughput: Throughput,
       pub eta: Option<Duration>,
       pub start_time: Option<SystemTime>,
       pub discovered_hosts: usize,
       pub open_ports: usize,
       pub closed_ports: usize,
       pub filtered_ports: usize,
       pub detected_services: usize,
       pub errors: Vec<ScanError>,
       pub warnings: Vec<String>,
   }

   impl Default for AggregatedState {
       fn default() -> Self {
           Self {
               scan_id: None,
               current_stage: ScanStage::Initializing,
               overall_progress: 0.0,
               stage_progress: 0.0,
               throughput: Throughput::default(),
               eta: None,
               start_time: None,
               discovered_hosts: 0,
               open_ports: 0,
               closed_ports: 0,
               filtered_ports: 0,
               detected_services: 0,
               errors: Vec::new(),
               warnings: Vec::new(),
           }
       }
   }
   ```

2. Implement constructor and background task:
   ```rust
   impl ProgressAggregator {
       pub fn new(event_bus: Arc<EventBus>) -> Self {
           let state = Arc::new(RwLock::new(AggregatedState::default()));
           let state_clone = state.clone();

           let (_, mut event_rx) = event_bus.subscribe_all();

           let updater_task = tokio::spawn(async move {
               while let Some(event) = event_rx.recv().await {
                   Self::update_state(&state_clone, event);
               }
           });

           Self {
               event_bus,
               state,
               _updater_task: updater_task,
           }
       }

       fn update_state(state: &Arc<RwLock<AggregatedState>>, event: ScanEvent) {
           let mut state = state.write().unwrap();

           match event {
               ScanEvent::ScanStarted { scan_id, timestamp, target_count, .. } => {
                   state.scan_id = Some(scan_id);
                   state.start_time = Some(timestamp);
                   state.current_stage = ScanStage::Initializing;
                   // Reset counters
                   state.open_ports = 0;
                   state.detected_services = 0;
               }
               ScanEvent::ProgressUpdate { percentage, throughput, eta, .. } => {
                   state.overall_progress = percentage;
                   state.throughput = throughput;
                   state.eta = eta;
               }
               ScanEvent::StageChanged { to_stage, .. } => {
                   state.current_stage = to_stage;
                   state.stage_progress = 0.0;
               }
               ScanEvent::HostDiscovered { .. } => {
                   state.discovered_hosts += 1;
               }
               ScanEvent::PortFound { state: port_state, .. } => {
                   match port_state {
                       PortState::Open => state.open_ports += 1,
                       PortState::Closed => state.closed_ports += 1,
                       PortState::Filtered => state.filtered_ports += 1,
                   }
               }
               ScanEvent::ServiceDetected { .. } => {
                   state.detected_services += 1;
               }
               ScanEvent::WarningIssued { message, .. } => {
                   state.warnings.push(message);
               }
               ScanEvent::ScanError { error, .. } => {
                   state.errors.push(error);
               }
               _ => {}
           }
       }

       pub fn get_state(&self) -> AggregatedState {
           self.state.read().unwrap().clone()
       }
   }
   ```

**Acceptance Criteria:**
- [ ] `ProgressAggregator` subscribes to all events
- [ ] Background task updates state on events
- [ ] `get_state()` returns current aggregated state
- [ ] Non-blocking reads (RwLock)
- [ ] Unit tests verify state updates

**Estimated Time:** 2-3 hours

---

### Task 4.2: ETA Calculation Algorithm (2h)

**Description:**
Implement adaptive ETA calculation using EWMA (Exponential Weighted Moving Average) for smoothing.

**Requirements:**
- Track completion rate over sliding window (60 seconds)
- Use EWMA for smoothing (α = 0.3)
- Handle edge cases (slow starts, rate changes, stalls)
- Provide confidence interval for ETA

**Files to modify:**
- `crates/prtip-core/src/progress_aggregator.rs`

**Implementation Steps:**
1. Add ETA calculator state:
   ```rust
   struct EtaCalculator {
       window_size: Duration,
       samples: VecDeque<(SystemTime, f32)>, // (time, completion percentage)
       smoothed_rate: f32, // EWMA smoothed rate
       alpha: f32, // EWMA smoothing factor (0.3)
   }

   impl EtaCalculator {
       fn new() -> Self {
           Self {
               window_size: Duration::from_secs(60),
               samples: VecDeque::new(),
               smoothed_rate: 0.0,
               alpha: 0.3,
           }
       }

       fn add_sample(&mut self, timestamp: SystemTime, percentage: f32) {
           // Remove old samples (outside window)
           let cutoff = timestamp - self.window_size;
           while let Some((time, _)) = self.samples.front() {
               if *time < cutoff {
                   self.samples.pop_front();
               } else {
                   break;
               }
           }

           // Add new sample
           self.samples.push_back((timestamp, percentage));

           // Calculate current rate (percentage points per second)
           if self.samples.len() >= 2 {
               let (first_time, first_pct) = self.samples.front().unwrap();
               let (last_time, last_pct) = self.samples.back().unwrap();
               
               let duration = last_time.duration_since(*first_time).unwrap();
               let pct_change = last_pct - first_pct;
               let current_rate = pct_change / duration.as_secs_f32();

               // EWMA smoothing
               self.smoothed_rate = self.alpha * current_rate + (1.0 - self.alpha) * self.smoothed_rate;
           }
       }

       fn calculate_eta(&self, current_percentage: f32) -> Option<Duration> {
           if self.smoothed_rate <= 0.0 {
               return None; // Can't estimate (stalled or negative rate)
           }

           let remaining = 100.0 - current_percentage;
           let seconds = remaining / self.smoothed_rate;

           Some(Duration::from_secs_f32(seconds))
       }
   }
   ```

2. Integrate into ProgressAggregator:
   ```rust
   pub struct AggregatedState {
       // ... existing fields ...
       eta_calculator: EtaCalculator,
   }

   impl ProgressAggregator {
       fn update_state(state: &Arc<RwLock<AggregatedState>>, event: ScanEvent) {
           let mut state = state.write().unwrap();

           match event {
               ScanEvent::ProgressUpdate { percentage, timestamp, .. } => {
                   state.eta_calculator.add_sample(timestamp, percentage);
                   state.eta = state.eta_calculator.calculate_eta(percentage);
                   state.overall_progress = percentage;
               }
               _ => {}
           }
       }
   }
   ```

**Acceptance Criteria:**
- [ ] ETA calculated using EWMA smoothing
- [ ] Sliding window (60 seconds) for rate calculation
- [ ] Edge cases handled (stalls, slow starts)
- [ ] Unit tests verify ETA accuracy (mock progress)
- [ ] ETA included in aggregated state

**Estimated Time:** 2 hours

---

### Task 4.3: Throughput Metrics Calculation (1-2h)

**Description:**
Calculate real-time throughput metrics (pps, hpm, bandwidth) from scan events.

**Requirements:**
- Packets per second (pps): Count packets sent/received
- Hosts per minute (hpm): Count hosts completed
- Bandwidth (Mbps): Sum packet sizes

**Files to modify:**
- `crates/prtip-core/src/progress_aggregator.rs`

**Implementation Steps:**
1. Add throughput tracker:
   ```rust
   struct ThroughputTracker {
       packets_sent: usize,
       packets_received: usize,
       bytes_sent: usize,
       bytes_received: usize,
       hosts_completed: usize,
       start_time: Instant,
   }

   impl ThroughputTracker {
       fn calculate(&self) -> Throughput {
           let elapsed = self.start_time.elapsed().as_secs_f64();
           if elapsed == 0.0 {
               return Throughput::default();
           }

           Throughput {
               packets_per_second: (self.packets_sent + self.packets_received) as f64 / elapsed,
               hosts_per_minute: (self.hosts_completed as f64 / elapsed) * 60.0,
               bandwidth_mbps: (self.bytes_sent + self.bytes_received) as f64 / elapsed / 1_000_000.0 * 8.0,
           }
       }

       fn on_packet_sent(&mut self, size: usize) {
           self.packets_sent += 1;
           self.bytes_sent += size;
       }

       fn on_packet_received(&mut self, size: usize) {
           self.packets_received += 1;
           self.bytes_received += size;
       }

       fn on_host_completed(&mut self) {
           self.hosts_completed += 1;
       }
   }
   ```

2. Integrate into ProgressAggregator:
   ```rust
   impl ProgressAggregator {
       fn update_state(state: &Arc<RwLock<AggregatedState>>, event: ScanEvent) {
           let mut state = state.write().unwrap();

           match event {
               ScanEvent::MetricRecorded { metric, value, .. } => {
                   match metric {
                       MetricType::PacketsSent => state.throughput_tracker.on_packet_sent(value as usize),
                       MetricType::PacketsReceived => state.throughput_tracker.on_packet_received(value as usize),
                       // ... more metrics
                   }
                   state.throughput = state.throughput_tracker.calculate();
               }
               _ => {}
           }
       }
   }
   ```

**Acceptance Criteria:**
- [ ] Throughput calculated (pps, hpm, bandwidth)
- [ ] Metrics updated from `MetricRecorded` events
- [ ] Real-time calculation (updated every event)
- [ ] Unit tests verify throughput calculations

**Estimated Time:** 1-2 hours

---

### Task 4.4: Progress Aggregator Testing (1h)

**Description:**
Test ProgressAggregator with simulated event streams to verify state updates and calculations.

**Requirements:**
- Test state updates from events
- Test ETA calculation accuracy
- Test throughput calculation accuracy
- Test concurrent access (multi-threaded reads)

**Files to create:**
- `crates/prtip-core/src/progress_aggregator/tests.rs`

**Implementation Steps:**
1. Test basic state updates:
   ```rust
   #[tokio::test]
   async fn test_progress_aggregator_updates() {
       let bus = Arc::new(EventBus::new());
       let aggregator = ProgressAggregator::new(bus.clone());

       // Emit events
       bus.publish(ScanEvent::ScanStarted { /* ... */ });
       bus.publish(ScanEvent::ProgressUpdate { percentage: 50.0, /* ... */ });
       bus.publish(ScanEvent::PortFound { state: PortState::Open, /* ... */ });

       // Wait for aggregator to process
       tokio::time::sleep(Duration::from_millis(100)).await;

       let state = aggregator.get_state();
       assert_eq!(state.overall_progress, 50.0);
       assert_eq!(state.open_ports, 1);
   }
   ```

2. Test ETA calculation:
   ```rust
   #[tokio::test]
   async fn test_eta_calculation() {
       let bus = Arc::new(EventBus::new());
       let aggregator = ProgressAggregator::new(bus.clone());

       // Simulate steady progress (10% per second)
       for i in 0..=10 {
           bus.publish(ScanEvent::ProgressUpdate {
               percentage: (i * 10) as f32,
               timestamp: SystemTime::now() + Duration::from_secs(i),
               /* ... */
           });
           tokio::time::sleep(Duration::from_millis(100)).await;
       }

       let state = aggregator.get_state();
       assert!(state.eta.is_some());
       
       // Should estimate ~9 seconds remaining (90% left at 10%/sec)
       let eta_secs = state.eta.unwrap().as_secs();
       assert!(eta_secs >= 8 && eta_secs <= 10);
   }
   ```

**Acceptance Criteria:**
- [ ] State updates tested
- [ ] ETA calculation tested
- [ ] Throughput calculation tested
- [ ] Concurrent access tested (multi-threaded)
- [ ] All tests pass

**Estimated Time:** 1 hour

---

## Task Area 5: CLI Integration (4-5 hours)

### Task 5.1: Convert CLI Progress Display to Event-Driven (2h)

**Description:**
Refactor `prtip-cli/src/progress.rs` from polling-based to event-driven using ProgressAggregator.

**Requirements:**
- Replace polling loop with event subscription
- Update display on `ProgressUpdate` events
- Maintain existing display styles (minimal, standard, verbose)
- Add 100ms debouncing to prevent UI flicker

**Files to modify:**
- `crates/prtip-cli/src/progress.rs`

**Implementation Steps:**
1. Refactor progress display:
   ```rust
   pub async fn display_progress_event_driven(
       aggregator: Arc<ProgressAggregator>,
       style: ProgressStyle,
   ) -> Result<()> {
       let mut last_update = Instant::now();
       let debounce_ms = 100;

       loop {
           // Get current state (non-blocking read)
           let state = aggregator.get_state();

           // Check if enough time passed (debounce)
           if last_update.elapsed() < Duration::from_millis(debounce_ms) {
               tokio::time::sleep(Duration::from_millis(10)).await;
               continue;
           }

           // Render progress based on style
           match style {
               ProgressStyle::Minimal => {
                   print!("\r{:.1}%", state.overall_progress);
               }
               ProgressStyle::Standard => {
                   print!(
                       "\r[{}] {:.1}% | ETA: {}",
                       render_stage(state.current_stage),
                       state.overall_progress,
                       render_eta(state.eta)
                   );
               }
               ProgressStyle::Verbose => {
                   print!(
                       "\r[{}] {:.1}% | ETA: {} | {:.0} pps | {} open",
                       render_stage(state.current_stage),
                       state.overall_progress,
                       render_eta(state.eta),
                       state.throughput.packets_per_second,
                       state.open_ports
                   );
               }
           }
           io::stdout().flush()?;

           last_update = Instant::now();

           // Exit if scan completed
           if state.current_stage == ScanStage::Completed {
               println!(); // Newline after progress
               break;
           }

           tokio::time::sleep(Duration::from_millis(10)).await;
       }

       Ok(())
   }
   ```

2. Remove old polling code:
   ```rust
   // DELETE THIS OLD CODE:
   // loop {
   //     let completed = progress.completed();
   //     let total = progress.total();
   //     // ...
   //     sleep(Duration::from_millis(500));
   // }
   ```

**Acceptance Criteria:**
- [ ] CLI progress display uses ProgressAggregator (no polling)
- [ ] Display updates on state changes (event-driven)
- [ ] 100ms debouncing prevents flicker
- [ ] All progress styles work (minimal, standard, verbose)
- [ ] Integration test verifies event-driven display

**Estimated Time:** 2 hours

---

### Task 5.2: Live Results Streaming (1-2h)

**Description:**
Implement `--live-results` flag to display discoveries in real-time as they occur.

**Requirements:**
- Subscribe to discovery events (PortFound, ServiceDetected)
- Display results immediately (streaming table)
- Option to disable (batch display at end)

**Files to modify:**
- `crates/prtip-cli/src/main.rs`
- `crates/prtip-cli/src/display.rs`

**Implementation Steps:**
1. Add CLI flag:
   ```rust
   #[derive(Parser)]
   struct Args {
       // ... existing flags ...
       
       /// Display results in real-time as they are discovered
       #[arg(long)]
       live_results: bool,
   }
   ```

2. Implement live display:
   ```rust
   pub async fn display_live_results(event_bus: Arc<EventBus>) -> Result<()> {
       let (_, mut event_rx) = event_bus.subscribe_types(vec![
           ScanEventType::PortFound,
           ScanEventType::ServiceDetected,
       ]);

       println!("{:<15} {:<6} {:<10} {:<30}", "IP", "Port", "State", "Service");
       println!("{}", "-".repeat(70));

       while let Some(event) = event_rx.recv().await {
           match event {
               ScanEvent::PortFound { ip, port, state, .. } => {
                   println!("{:<15} {:<6} {:<10} {:<30}", ip, port, state, "-");
               }
               ScanEvent::ServiceDetected { ip, port, service, .. } => {
                   println!(
                       "{:<15} {:<6} {:<10} {:<30}",
                       ip, port, "open", service.name
                   );
               }
               ScanEvent::ScanCompleted { .. } => break,
               _ => {}
           }
       }

       Ok(())
   }
   ```

3. Integrate into main:
   ```rust
   #[tokio::main]
   async fn main() -> Result<()> {
       let args = Args::parse();
       
       let event_bus = Arc::new(EventBus::new());
       let config = ScanConfig::default().with_event_bus(event_bus.clone());

       // Spawn live results display if enabled
       let live_display = if args.live_results {
           Some(tokio::spawn(display_live_results(event_bus.clone())))
       } else {
           None
       };

       // Run scan
       let results = run_scan(config).await?;

       // Wait for display to finish
       if let Some(handle) = live_display {
           handle.await??;
       }

       Ok(())
   }
   ```

**Acceptance Criteria:**
- [ ] `--live-results` flag implemented
- [ ] Results displayed immediately as discovered
- [ ] Table format (IP, Port, State, Service)
- [ ] Gracefully stops on scan completion
- [ ] Integration test verifies live display

**Estimated Time:** 1-2 hours

---

### Task 5.3: CLI Integration Testing (1h)

**Description:**
Test CLI integration with event system end-to-end.

**Requirements:**
- Test CLI progress display updates
- Test live results streaming
- Test multiple concurrent displays
- Test graceful shutdown

**Files to create:**
- `crates/prtip-cli/tests/event_integration.rs`

**Implementation Steps:**
1. Test progress display:
   ```rust
   #[tokio::test]
   async fn test_cli_progress_display() {
       let bus = Arc::new(EventBus::new());
       let aggregator = Arc::new(ProgressAggregator::new(bus.clone()));

       // Spawn progress display
       let display_task = tokio::spawn(display_progress_event_driven(
           aggregator.clone(),
           ProgressStyle::Standard,
       ));

       // Simulate scan events
       bus.publish(ScanEvent::ScanStarted { /* ... */ });
       for i in 0..=10 {
           bus.publish(ScanEvent::ProgressUpdate {
               percentage: (i * 10) as f32,
               /* ... */
           });
           tokio::time::sleep(Duration::from_millis(50)).await;
       }
       bus.publish(ScanEvent::ScanCompleted { /* ... */ });

       // Wait for display to finish
       tokio::time::timeout(Duration::from_secs(5), display_task).await.unwrap().unwrap();
   }
   ```

2. Test live results:
   ```rust
   #[tokio::test]
   async fn test_live_results_display() {
       let bus = Arc::new(EventBus::new());

       // Spawn live display
       let display_task = tokio::spawn(display_live_results(bus.clone()));

       // Emit discovery events
       bus.publish(ScanEvent::PortFound { /* ... */ });
       bus.publish(ScanEvent::ServiceDetected { /* ... */ });
       bus.publish(ScanEvent::ScanCompleted { /* ... */ });

       // Wait for display
       tokio::time::timeout(Duration::from_secs(5), display_task).await.unwrap().unwrap();
   }
   ```

**Acceptance Criteria:**
- [ ] CLI progress display tested
- [ ] Live results display tested
- [ ] Concurrent displays tested (progress + live results)
- [ ] Graceful shutdown tested
- [ ] All integration tests pass

**Estimated Time:** 1 hour

---

## Task Area 6: Event Logging (3-4 hours)

### Task 6.1: JSON Event Logger Implementation (2h)

**Description:**
Create event logger that subscribes to all events and writes them to JSON Lines format.

**Requirements:**
- Subscribe to all events
- Write to `~/.prtip/events/<scan_id>.jsonl`
- One event per line (JSON Lines format)
- Include scan metadata (header/footer)

**Files to create:**
- `crates/prtip-core/src/event_logger.rs` (new, ~300 lines)

**Implementation Steps:**
1. Define event logger:
   ```rust
   use std::fs::{File, create_dir_all};
   use std::io::{BufWriter, Write};
   use std::path::PathBuf;

   pub struct EventLogger {
       event_bus: Arc<EventBus>,
       log_dir: PathBuf,
       _logger_task: JoinHandle<()>,
   }

   impl EventLogger {
       pub fn new(event_bus: Arc<EventBus>) -> Result<Self> {
           let log_dir = dirs::home_dir()
               .ok_or(Error::NoHomeDir)?
               .join(".prtip")
               .join("events");
           
           create_dir_all(&log_dir)?;

           let (_, mut event_rx) = event_bus.subscribe_all();
           let log_dir_clone = log_dir.clone();

           let logger_task = tokio::spawn(async move {
               let mut current_file: Option<BufWriter<File>> = None;
               let mut current_scan_id: Option<Uuid> = None;

               while let Some(event) = event_rx.recv().await {
                   // Open new file on ScanStarted
                   if matches!(event, ScanEvent::ScanStarted { .. }) {
                       if let Some(mut file) = current_file.take() {
                           let _ = file.flush();
                       }

                       let scan_id = event.scan_id();
                       current_scan_id = Some(scan_id);

                       let path = log_dir_clone.join(format!("{}.jsonl", scan_id));
                       let file = File::create(path).unwrap();
                       current_file = Some(BufWriter::new(file));

                       // Write header
                       if let Some(writer) = &mut current_file {
                           let header = json!({
                               "type": "header",
                               "scan_id": scan_id,
                               "start_time": SystemTime::now(),
                               "prtip_version": env!("CARGO_PKG_VERSION"),
                           });
                           writeln!(writer, "{}", serde_json::to_string(&header).unwrap()).unwrap();
                       }
                   }

                   // Write event
                   if let Some(writer) = &mut current_file {
                       let json = serde_json::to_string(&event).unwrap();
                       writeln!(writer, "{}", json).unwrap();
                   }

                   // Write footer and close on ScanCompleted
                   if matches!(event, ScanEvent::ScanCompleted { .. }) {
                       if let Some(mut file) = current_file.take() {
                           let footer = json!({
                               "type": "footer",
                               "scan_id": current_scan_id,
                               "end_time": SystemTime::now(),
                           });
                           writeln!(file, "{}", serde_json::to_string(&footer).unwrap()).unwrap();
                           let _ = file.flush();
                       }
                       current_scan_id = None;
                   }
               }
           });

           Ok(Self {
               event_bus,
               log_dir,
               _logger_task: logger_task,
           })
       }

       pub fn log_dir(&self) -> &PathBuf {
           &self.log_dir
       }
   }
   ```

**Acceptance Criteria:**
- [ ] EventLogger subscribes to all events
- [ ] Events written to `~/.prtip/events/<scan_id>.jsonl`
- [ ] JSON Lines format (one event per line)
- [ ] Header and footer metadata
- [ ] Unit tests verify file creation and writing

**Estimated Time:** 2 hours

---

### Task 6.2: Log Rotation & Cleanup (1-2h)

**Description:**
Implement log rotation (max 100MB per file) and automatic cleanup (30-day retention).

**Requirements:**
- Rotate logs at 100MB
- Compress rotated logs (gzip)
- Keep last 10 log files
- Auto-delete logs older than 30 days

**Files to modify:**
- `crates/prtip-core/src/event_logger.rs`

**Implementation Steps:**
1. Add rotation logic:
   ```rust
   impl EventLogger {
       fn check_rotation(&mut self, file: &mut BufWriter<File>, scan_id: Uuid) -> Result<()> {
           let metadata = file.get_ref().metadata()?;
           if metadata.len() > 100_000_000 { // 100MB
               // Flush and close current file
               file.flush()?;

               // Compress old file
               let path = self.log_dir.join(format!("{}.jsonl", scan_id));
               let gz_path = self.log_dir.join(format!("{}.jsonl.gz", scan_id));
               Self::compress_file(&path, &gz_path)?;

               // Delete original
               std::fs::remove_file(path)?;

               // Open new file
               let new_path = self.log_dir.join(format!("{}-{}.jsonl", scan_id, Uuid::new_v4()));
               *file = BufWriter::new(File::create(new_path)?);
           }
           Ok(())
       }

       fn compress_file(src: &PathBuf, dst: &PathBuf) -> Result<()> {
           use flate2::write::GzEncoder;
           use flate2::Compression;

           let input = File::open(src)?;
           let output = File::create(dst)?;
           let mut encoder = GzEncoder::new(output, Compression::default());

           std::io::copy(&mut std::io::BufReader::new(input), &mut encoder)?;
           encoder.finish()?;
           Ok(())
       }
   }
   ```

2. Add cleanup task:
   ```rust
   impl EventLogger {
       pub fn cleanup_old_logs(&self) -> Result<()> {
           let cutoff = SystemTime::now() - Duration::from_secs(30 * 24 * 60 * 60); // 30 days

           for entry in std::fs::read_dir(&self.log_dir)? {
               let entry = entry?;
               let metadata = entry.metadata()?;

               if let Ok(modified) = metadata.modified() {
                   if modified < cutoff {
                       std::fs::remove_file(entry.path())?;
                   }
               }
           }

           Ok(())
       }
   }
   ```

**Acceptance Criteria:**
- [ ] Logs rotated at 100MB
- [ ] Rotated logs compressed (gzip)
- [ ] Last 10 files kept
- [ ] Logs older than 30 days deleted
- [ ] Tests verify rotation and cleanup

**Estimated Time:** 1-2 hours

---

## Task Area 7: Testing & Benchmarking (4-5 hours)

### Task 7.1: Comprehensive Unit Tests (2h)

**Description:**
Create unit tests for all event system components.

**Requirements:**
- Test event types (serialization, validation)
- Test event bus (publish, subscribe, filtering)
- Test progress aggregator (state updates, ETA)
- Test event logger (file writing, rotation)
- Achieve 90%+ test coverage for event system

**Files to create:**
- `crates/prtip-core/src/events/tests.rs`
- `crates/prtip-core/src/event_bus/tests.rs`
- `crates/prtip-core/src/progress_aggregator/tests.rs`
- `crates/prtip-core/src/event_logger/tests.rs`

**Implementation Steps:**
(Tests already covered in previous tasks, consolidate here)

**Acceptance Criteria:**
- [ ] 30+ unit tests for event system
- [ ] Test coverage >90% for:
  - [ ] events module
  - [ ] event_bus module
  - [ ] progress_aggregator module
  - [ ] event_logger module
- [ ] All tests pass: `cargo test --package prtip-core`
- [ ] No flaky tests (run 10x to verify)

**Estimated Time:** 2 hours

---

### Task 7.2: Performance Benchmarks (1-2h)

**Description:**
Benchmark event system performance to validate <5% overhead target.

**Requirements:**
- Measure baseline scan speed (no events)
- Measure with 1, 5, 10 subscribers
- Calculate overhead percentage
- Verify p99 latency <10ms

**Files to create:**
- `benches/event_system_overhead.rs`

**Implementation Steps:**
1. Benchmark baseline:
   ```rust
   fn bench_scan_no_events(c: &mut Criterion) {
       c.bench_function("scan_1000_ports_no_events", |b| {
           b.iter(|| {
               let runtime = tokio::runtime::Runtime::new().unwrap();
               runtime.block_on(async {
                   let config = ScanConfig::default();
                   let mut scanner = SynScanner::new(config);
                   scanner.scan(generate_targets(1000)).await.unwrap();
               });
           });
       });
   }
   ```

2. Benchmark with events:
   ```rust
   fn bench_scan_with_events(c: &mut Criterion) {
       for num_subscribers in [1, 5, 10] {
           c.bench_function(&format!("scan_1000_ports_{}_subscribers", num_subscribers), |b| {
               b.iter(|| {
                   let runtime = tokio::runtime::Runtime::new().unwrap();
                   runtime.block_on(async {
                       let bus = Arc::new(EventBus::new());
                       
                       // Create subscribers
                       let mut _subscribers = Vec::new();
                       for _ in 0..num_subscribers {
                           let (_, rx) = bus.subscribe_all();
                           _subscribers.push(rx);
                       }

                       let config = ScanConfig::default().with_event_bus(bus);
                       let mut scanner = SynScanner::new(config);
                       scanner.scan(generate_targets(1000)).await.unwrap();
                   });
               });
           });
       }
   }
   ```

3. Calculate overhead:
   ```rust
   // Run benchmarks and compare results
   // Target: <5% overhead with 10 subscribers
   ```

**Acceptance Criteria:**
- [ ] Baseline benchmark (no events)
- [ ] Event benchmarks (1, 5, 10 subscribers)
- [ ] Overhead <5% with 10 subscribers
- [ ] p99 publish latency <10ms
- [ ] Results documented in SPRINT-5.5.3-COMPLETE.md

**Estimated Time:** 1-2 hours

---

### Task 7.3: CI Integration & Regression Detection (1h)

**Description:**
Integrate event system tests and benchmarks into CI/CD pipeline with regression detection.

**Requirements:**
- Add event tests to GitHub Actions
- Add performance benchmarks (track over time)
- Fail CI if overhead >7% (5% target + 2% buffer)
- Generate benchmark comparison reports

**Files to modify:**
- `.github/workflows/ci.yml`
- `.github/workflows/benchmark.yml`

**Implementation Steps:**
1. Add event tests to CI:
   ```yaml
   # .github/workflows/ci.yml
   - name: Test Event System
     run: |
       cargo test --package prtip-core events
       cargo test --package prtip-core event_bus
       cargo test --package prtip-core progress_aggregator
       cargo test --package prtip-core event_logger
   ```

2. Add benchmark CI:
   ```yaml
   # .github/workflows/benchmark.yml
   name: Performance Benchmarks

   on:
     push:
       branches: [main]
     pull_request:

   jobs:
     benchmark:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - uses: actions-rs/toolchain@v1
           with:
             toolchain: stable
         
         - name: Run Benchmarks
           run: cargo bench --bench event_system_overhead -- --save-baseline main

         - name: Compare to Baseline
           run: |
             cargo bench --bench event_system_overhead -- --baseline main > bench_results.txt
             # Parse results and fail if regression >7%
   ```

**Acceptance Criteria:**
- [ ] Event tests run in CI
- [ ] Benchmarks run on every PR
- [ ] CI fails if overhead >7%
- [ ] Benchmark comparison reports generated
- [ ] Historical benchmark data tracked

**Estimated Time:** 1 hour

---

## Success Criteria

### Quantitative Metrics

**Event System:**
- [ ] 18+ event types defined (lifecycle, progress, discovery, diagnostic)
- [ ] Event bus supports multi-subscriber (10+ concurrent)
- [ ] Event history buffer: 1000 events (ring buffer)
- [ ] Event filtering: 4 types (All, ScanId, EventType, Custom)
- [ ] Event latency: p99 <10ms publish-to-receive
- [ ] Event overhead: <5% with 10 subscribers

**Scanner Integration:**
- [ ] All 8 scanners emit events (SYN, Connect, UDP, Stealth, Idle, Decoy, Discovery)
- [ ] Service detection emits `ServiceDetected` events
- [ ] OS fingerprinting emits `OsDetected` events
- [ ] TLS analysis emits `CertificateFound` events
- [ ] Backward compatible: scans work without event bus

**Progress Aggregation:**
- [ ] Real-time state queryable (<1ms read latency)
- [ ] ETA calculation with EWMA smoothing (α=0.3)
- [ ] Throughput metrics (pps, hpm, bandwidth)
- [ ] Multi-stage tracking (7 stages)

**CLI Integration:**
- [ ] Event-driven progress display (no polling)
- [ ] Live results streaming (--live-results flag)
- [ ] 100ms debouncing (prevents flicker)
- [ ] All progress styles work (minimal, standard, verbose)

**Event Logging:**
- [ ] JSON Lines format (.jsonl files)
- [ ] Log rotation at 100MB
- [ ] Compression (gzip)
- [ ] 30-day retention (auto-cleanup)

**Testing:**
- [ ] 30+ unit tests for event system
- [ ] Test coverage >90% (events, event_bus, aggregator, logger)
- [ ] Integration tests: scanner → event bus → CLI
- [ ] Performance benchmarks: baseline vs. with events

### Qualitative Metrics

**Architecture Quality:**
- [ ] Clean separation: scanner ↔ event bus ↔ consumers
- [ ] Non-blocking: slow subscribers don't block scans
- [ ] Extensible: easy to add new event types/subscribers
- [ ] Debuggable: event logs queryable, replay works
- [ ] Thread-safe: Arc/Mutex used correctly

**Code Quality:**
- [ ] Professional-grade code (A+ review)
- [ ] Comprehensive rustdoc (all public APIs)
- [ ] Zero clippy warnings
- [ ] Consistent patterns across codebase
- [ ] Error handling robust (no panics in production)

**TUI Readiness:**
- [ ] TUI can subscribe to events (real-time updates)
- [ ] Background scans work (decoupled from display)
- [ ] State queryable (TUI can get current state anytime)
- [ ] Multiple widgets supported (progress, results, logs)
- [ ] Zero polling needed (event-driven architecture)

**Documentation:**
- [ ] Architecture documented (event flow diagrams)
- [ ] API examples for common use cases
- [ ] Integration guide for consumers (TUI developers)
- [ ] Performance characteristics documented
- [ ] Troubleshooting guide (common issues)

---

## Documentation Requirements

### Files to Create

1. **Architecture Documentation:**
   - `docs/35-EVENT-SYSTEM-ARCHITECTURE.md` (1,500+ lines)
   - Event system design
   - Pub-sub pattern explanation
   - Thread safety considerations
   - Performance characteristics

2. **Integration Guide:**
   - `docs/36-EVENT-SYSTEM-INTEGRATION.md` (800+ lines)
   - How to subscribe to events
   - Event filtering examples
   - Building custom consumers
   - Common patterns and anti-patterns

3. **API Reference:**
   - Comprehensive rustdoc for all public APIs
   - `EventBus`, `ScanEvent`, `ProgressAggregator`, `EventLogger`
   - Code examples for each method

4. **Sprint Completion Report:**
   - `SPRINT-5.5.3-COMPLETE.md` (1,000+ lines)
   - Summary of accomplishments
   - Performance benchmark results
   - Challenges and solutions
   - Lessons learned

### Rustdoc Requirements

All public APIs must have:
- [ ] Module-level documentation (`//!`)
- [ ] Struct/enum documentation
- [ ] Method documentation
- [ ] Examples for complex APIs
- [ ] "See Also" cross-references

**Targets:**
- 0 rustdoc warnings: `cargo doc --no-deps`
- 100% public API coverage: `cargo doc --open` (manual review)

---

## Verification Checklist

### Pre-Implementation

- [ ] Sprint 5.5.2 COMPLETE (progress infrastructure exists)
- [ ] All 8 scanners implemented and tested
- [ ] Service detection, OS fingerprinting, TLS analysis ready
- [ ] Tokio 1.35+ available
- [ ] Serde 1.0+ available

### During Implementation

**After Each Task:**
- [ ] Code compiles: `cargo build --package prtip-core`
- [ ] Tests pass: `cargo test --package prtip-core <module>`
- [ ] No clippy warnings: `cargo clippy --package prtip-core`
- [ ] Documentation builds: `cargo doc --no-deps`

**After Each Task Area:**
- [ ] Integration tests pass
- [ ] Performance benchmarks run
- [ ] Documentation updated
- [ ] Examples tested

### Post-Implementation

**Code Quality:**
- [ ] All tasks complete (40/40 checkboxes)
- [ ] All tests pass: `cargo test --workspace`
- [ ] Zero clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Code formatted: `cargo fmt --all -- --check`

**Testing:**
- [ ] Unit tests: 30+ passing
- [ ] Integration tests: Scanner → EventBus → CLI
- [ ] Performance benchmarks: <5% overhead verified
- [ ] Test coverage: >90% for event system modules

**Documentation:**
- [ ] Rustdoc complete (0 warnings)
- [ ] Architecture docs written (35-EVENT-SYSTEM-ARCHITECTURE.md)
- [ ] Integration guide written (36-EVENT-SYSTEM-INTEGRATION.md)
- [ ] Sprint completion report (SPRINT-5.5.3-COMPLETE.md)

**Phase 6 Readiness:**
- [ ] TUI can subscribe to events
- [ ] Real-time updates work (<100ms latency)
- [ ] Background scans work (decoupled execution)
- [ ] State queryable (non-blocking reads)
- [ ] Multiple widgets can coexist

**Final Checks:**
- [ ] CI/CD passes: All GitHub Actions green
- [ ] Performance: <5% overhead with 10 subscribers verified
- [ ] Backward compatibility: Existing tests still pass
- [ ] No regressions: Baseline scan speed maintained

---

## Risk Mitigation

### Performance Risk: Event overhead degrades scan speed

**Mitigation:**
- Benchmark early (Task 2.6, 7.2)
- Profile hot paths with `cargo flamegraph`
- Optimize if overhead >5%

**Fallback:**
- Make event bus optional (already planned)
- Allow disabling events via config flag
- Document performance impact

**Target:**
- <5% overhead with 10 subscribers (measured)

---

### Technical Risk: Event buffer overflow

**Mitigation:**
- Ring buffer (oldest events dropped)
- Monitor buffer usage (warn at >90%)
- Configurable buffer size

**Fallback:**
- Increase default buffer size (1000 → 10000)
- Add overflow counter to metrics
- Log warnings on overflow

**Target:**
- <1% overflow rate during normal scans

---

### Concurrency Risk: Slow subscriber blocks others

**Mitigation:**
- Unbounded channels (buffering per-subscriber)
- Auto-remove subscribers >10 seconds behind
- Monitor subscriber lag

**Fallback:**
- Document max subscriber lag
- Add subscriber timeout config
- Provide subscriber health API

**Target:**
- All subscribers within 1 second of real-time

---

### Complexity Risk: Async debugging difficult

**Mitigation:**
- Comprehensive event logging (JSON)
- Event timeline visualizer (future)
- Property-based testing for races

**Fallback:**
- Sync event bus option (blocking)
- Debug mode with verbose logging
- Replay tool for debugging

**Target:**
- Event logs sufficient for debugging 95% of issues

---

### API Risk: Event schema changes break consumers

**Mitigation:**
- Version event types (EventV1, EventV2)
- Support multiple versions simultaneously
- Deprecation warnings for old versions

**Fallback:**
- Event schema migrations
- Compatibility layer
- Clear changelog

**Target:**
- Zero breaking changes within minor versions

---

## Estimated Timeline

**Day 1 (8 hours):**
- Task Area 1: Event Type Design (4-5h)
- Task Area 2.1-2.2: EventBus Architecture (3-4h)

**Day 2 (8 hours):**
- Task Area 2.3-2.6: EventBus Complete (6-7h)
- Task Area 3.1: ScanConfig Integration (1h)

**Day 3 (8 hours):**
- Task Area 3.2-3.4: Scanner Integration (4-6h)
- Task Area 3.5: Integration Testing (1h)
- Task Area 4.1: Progress Aggregator (2-3h)

**Day 4 (8 hours):**
- Task Area 4.2-4.4: Progress Complete (4-5h)
- Task Area 5.1-5.2: CLI Integration (3-4h)

**Day 5 (8 hours):**
- Task Area 5.3: CLI Testing (1h)
- Task Area 6: Event Logging (3-4h)
- Task Area 7: Testing & Benchmarking (4-5h)

**Buffer:** +0-8 hours for unexpected issues

**Total:** 32-40 hours (4-5 days)

---

## Completion Criteria

Sprint 5.5.3 is COMPLETE when:

✅ All 40 tasks checked off
✅ All tests passing (30+ unit, 10+ integration)
✅ Performance targets met (<5% overhead, p99 <10ms)
✅ Documentation complete (architecture, integration guide, rustdoc)
✅ CI/CD passing (all workflows green)
✅ Phase 6 readiness verified (TUI can consume events)
✅ Sprint completion report written (SPRINT-5.5.3-COMPLETE.md)
✅ Code review complete (A+ grade)

---

## Next Steps After Sprint 5.5.3

**Immediate (Sprint 5.5.4):**
- Performance audit with event system in place
- Establish performance baselines for TUI comparison
- Regression detection suite

**Near-term (Sprint 5.5.5):**
- Configuration profiles and state persistence
- Resume capability (save/load scan state)
- Advanced filtering and querying

**Phase 6 Preparation:**
- Ratatui research and prototyping
- TUI architecture design
- Widget component library selection

**Phase 6 TUI Development:**
- Use event system built in this sprint
- Focus purely on UI rendering
- Zero backend refactoring needed

---

**END OF SPRINT 5.5.3 TODO**

*This comprehensive TODO provides everything needed to execute Sprint 5.5.3 successfully. Follow the tasks sequentially, verify acceptance criteria, and maintain quality standards. The event system built here is the foundation for Phase 6 TUI and beyond.*
