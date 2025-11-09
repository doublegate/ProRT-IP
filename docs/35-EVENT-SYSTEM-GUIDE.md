# Event System Developer Guide

**Version:** 2.0.0
**Sprint:** 5.5.3 - Event System & Progress Integration
**Status:** Production Ready
**Last Updated:** 2025-11-09

---

## Table of Contents

1. [Overview](#overview)
   - [What is the Event System?](#what-is-the-event-system)
   - [Architecture Diagram](#architecture-diagram)
   - [Component Relationships](#component-relationships)
2. [Core Components](#core-components)
   - [EventBus](#eventbus)
   - [ScanEvent](#scanevent)
   - [ProgressAggregator](#progressaggregator)
   - [EventLogger](#eventlogger)
3. [Event Types Reference](#event-types-reference)
4. [EventBus API Guide](#eventbus-api-guide)
   - [Publishing Events](#publishing-events)
   - [Subscribing to Events](#subscribing-to-events)
   - [Event Filtering](#event-filtering)
5. [ProgressAggregator API Guide](#progressaggregator-api-guide)
   - [Basic Usage](#basic-usage-progressaggregator)
   - [Real-Time Metrics](#real-time-metrics)
   - [Multi-Scanner Scenarios](#multi-scanner-scenarios)
6. [EventLogger API Guide](#eventlogger-api-guide)
   - [Basic Logging](#basic-logging)
   - [JSON Lines Format](#json-lines-format)
   - [Log Management](#log-management)
7. [Integration Patterns](#integration-patterns)
   - [Scanner Integration](#scanner-integration)
   - [CLI Integration](#cli-integration)
   - [TUI Integration (Phase 6)](#tui-integration-phase-6)
8. [Performance Optimization](#performance-optimization)
   - [Publish Performance](#publish-performance)
   - [Subscription Performance](#subscription-performance)
   - [Memory Management](#memory-management)
9. [Error Handling](#error-handling)
   - [Common Errors](#common-errors)
   - [Error Recovery](#error-recovery)
   - [Best Practices](#best-practices-error-handling)
10. [Testing Strategies](#testing-strategies)
    - [Unit Testing](#unit-testing)
    - [Integration Testing](#integration-testing)
    - [Performance Testing](#performance-testing)
11. [Debugging Techniques](#debugging-techniques)
    - [Event Tracing](#event-tracing)
    - [Performance Profiling](#performance-profiling)
    - [Common Issues](#common-issues)
12. [Best Practices](#best-practices)
13. [Advanced Topics](#advanced-topics)
    - [Custom Event Types](#custom-event-types)
    - [Event Batching](#event-batching)
    - [Distributed Events](#distributed-events)
14. [Troubleshooting](#troubleshooting)
    - [Performance Issues](#performance-issues)
    - [Memory Issues](#memory-issues)
    - [Reliability Issues](#reliability-issues)
15. [API Quick Reference](#api-quick-reference)
16. [References & Related Documentation](#references--related-documentation)

---

## Overview

### What is the Event System?

The Event System is the central communication backbone of ProRT-IP, enabling **real-time monitoring**, **progress tracking**, and **result streaming** across all scanner types. It provides a **publish-subscribe** architecture that decouples event producers (scanners) from consumers (CLI, TUI, loggers).

**Key Benefits:**

- **Real-Time Visibility**: Immediate feedback on scan progress, discoveries, and issues
- **Decoupled Architecture**: Scanners don't need to know about consumers
- **Performance**: Sub-microsecond event delivery (40ns publish, 340ns end-to-end)
- **Flexibility**: Subscribe to exactly the events you need via powerful filtering
- **Persistence**: Optional event logging to JSON Lines format for analysis
- **Thread-Safe**: Safe concurrent access from multiple scanners and subscribers

**Use Cases:**

1. **Live Progress Displays**: Show scan progress in CLI or TUI with real-time ETA
2. **Result Streaming**: Display discovered hosts/ports as they're found
3. **Audit Logging**: Record all scan events for compliance or forensics
4. **Performance Monitoring**: Track throughput, latency, and resource usage
5. **Error Alerting**: Immediate notification of scan failures or warnings
6. **Integration**: Feed events to external monitoring systems (Prometheus, SIEM)

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        EventBus                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │  Ring Buffer (1,000 events history)                │     │
│  │  - ScanStarted, PortFound, ServiceDetected, ...    │     │
│  └────────────────────────────────────────────────────┘     │
│                          ▲                                   │
│                          │ publish()                         │
│          ┌───────────────┼───────────────────┐              │
│          │               │                   │              │
│     ┌────┴────┐     ┌────┴────┐      ┌──────┴──────┐       │
│     │ Scanner │     │ Scanner │      │   CLI/TUI   │       │
│     │  (SYN)  │     │  (UDP)  │      │  (Metrics)  │       │
│     └─────────┘     └─────────┘      └─────────────┘       │
│                                                              │
│                          │ subscribe()                      │
│                          ▼                                   │
│          ┌───────────────┼───────────────────┐              │
│          │               │                   │              │
│     ┌────┴────┐     ┌────┴────┐      ┌──────┴──────┐       │
│     │   TUI   │     │   CLI   │      │EventLogger  │       │
│     │Dashboard│     │Progress │      │ (scans.jsonl)│       │
│     └─────────┘     └─────────┘      └─────────────┘       │
└─────────────────────────────────────────────────────────────┘

Flow:
1. Scanners publish events to EventBus (async, non-blocking)
2. EventBus stores events in ring buffer (last 1,000)
3. EventBus broadcasts to all subscribers (filtered)
4. Subscribers receive events via MPSC channels
```

### Component Relationships

**EventBus** (Central Hub)
- Manages all event distribution
- Maintains event history (ring buffer)
- Thread-safe for concurrent access
- Supports filtering and time-range queries

**ScanEvent** (Data Model)
- Enum with 18 event variants
- All events have: scan_id, timestamp
- Categories: Lifecycle, Discovery, Detection, Progress, Diagnostic

**ProgressAggregator** (Metrics Collector)
- Subscribes to EventBus for progress events
- Calculates real-time metrics (%, ETA, throughput)
- Supports multi-scanner aggregation
- Thread-safe metrics access

**EventLogger** (Persistence)
- Subscribes to EventBus for all events
- Writes events to JSON Lines format
- Automatic log rotation and compression
- Query-friendly for analysis tools (jq, grep)

---

## Core Components

### EventBus

**Purpose:** Central event distribution hub implementing publish-subscribe pattern

**Key Features:**

1. **Publishing**: Non-blocking async event publication
2. **Subscribing**: Register multiple subscribers with custom filters
3. **History**: Ring buffer of last 1,000 events for late subscribers
4. **Filtering**: Subscribe to specific event types, scan IDs, or time ranges
5. **Thread Safety**: Safe concurrent access via Arc<Mutex<>>

**Performance Characteristics:**

- **Publish latency**: ~40ns (single event, no subscribers)
- **Subscribe latency**: ~1.2μs (any filter complexity)
- **End-to-end latency**: ~340ns (publish → receive)
- **Concurrent overhead**: 4.2% with 16 publishers
- **Max throughput**: >10M events/second

**Example: Creating and Using EventBus**

```rust
use prtip_core::event_bus::EventBus;
use std::sync::Arc;

// Create event bus with 1,000 event history
let event_bus = Arc::new(EventBus::new(1000));

// Share event_bus across components
let scanner_bus = event_bus.clone();
let ui_bus = event_bus.clone();
let logger_bus = event_bus.clone();
```

### ScanEvent

**Purpose:** Unified event type representing all scanner events

**Event Categories:**

1. **Lifecycle Events** (5 types)
   - `ScanStarted`: Scan initialization complete
   - `ScanCompleted`: Scan finished successfully
   - `ScanCancelled`: User requested cancellation
   - `ScanPaused` / `ScanResumed`: Pause/resume support

2. **Discovery Events** (3 types)
   - `HostDiscovered`: Live host found (ICMP, ARP, etc.)
   - `PortFound`: Open port detected
   - `IPv6PortFound`: IPv6-specific port discovery

3. **Detection Events** (4 types)
   - `ServiceDetected`: Service identification (HTTP, SSH, etc.)
   - `OSDetected`: Operating system fingerprinting
   - `BannerGrabbed`: Application banner retrieved
   - `CertificateFound`: TLS certificate discovered

4. **Progress Events** (2 types)
   - `ProgressUpdate`: Scan progress metrics (%, ETA, throughput)
   - `StageChanged`: Scan phase transition (discovery → port scan → service detection)

5. **Diagnostic Events** (4 types)
   - `MetricRecorded`: Performance metric (packets sent, errors, etc.)
   - `WarningIssued`: Non-fatal warning (rate limit, timeout)
   - `RateLimitTriggered`: Rate limiter activated
   - `RetryScheduled`: Failed operation retry planned

**Common Event Fields:**

All events include:
- `scan_id: Uuid` - Unique scan identifier
- `timestamp: SystemTime` - Event creation time

**Example: Creating Events**

```rust
use prtip_core::events::{ScanEvent, ScanStage, Throughput};
use prtip_core::types::ScanType;
use std::time::SystemTime;
use uuid::Uuid;

let scan_id = Uuid::new_v4();
let timestamp = SystemTime::now();

// Lifecycle event
let event = ScanEvent::ScanStarted {
    scan_id,
    scan_type: ScanType::Syn,
    target_count: 1000,
    port_count: 100,
    timestamp,
};

// Discovery event
let event = ScanEvent::PortFound {
    scan_id,
    target: "192.168.1.1".parse().unwrap(),
    port: 80,
    protocol: Protocol::Tcp,
    state: PortState::Open,
    timestamp,
};

// Progress event
let event = ScanEvent::ProgressUpdate {
    scan_id,
    stage: ScanStage::ScanningPorts,
    percentage: 50.0,
    completed: 500,
    total: 1000,
    throughput: Throughput::default(),
    eta: Some(Duration::from_secs(60)),
    timestamp,
};
```

### ProgressAggregator

**Purpose:** Real-time scan progress metrics collection and aggregation

**Key Features:**

1. **Automatic Tracking**: Subscribes to EventBus progress events
2. **ETA Calculation**: Estimates completion time based on throughput
3. **Throughput Monitoring**: Tracks packets/sec, ports/sec
4. **Multi-Scanner Support**: Aggregates metrics across multiple concurrent scans
5. **Thread-Safe Access**: Concurrent metric queries via Arc<RwLock<>>

**Metrics Provided:**

- **percentage**: Scan completion (0-100%)
- **completed**: Items processed
- **total**: Total items to process
- **throughput**: Current packets/sec, ports/sec
- **eta**: Estimated time to completion
- **stage**: Current scan phase

**Example: Using ProgressAggregator**

```rust
use prtip_core::progress::ProgressAggregator;
use std::sync::Arc;

// Create aggregator (subscribes to event_bus)
let aggregator = Arc::new(
    ProgressAggregator::new(event_bus.clone(), scan_id)
);

// Initialize with totals
aggregator.start(1000, 100).await; // 1000 targets, 100 ports

// Get current metrics (async, lock-free read)
let metrics = aggregator.get_current_metrics().await;
println!("Progress: {:.1}% ({}/{})",
    metrics.percentage,
    metrics.completed,
    metrics.total
);

if let Some(eta) = metrics.eta {
    println!("ETA: {:?}", eta);
}

println!("Throughput: {} packets/sec", metrics.throughput.packets_per_sec);
```

### EventLogger

**Purpose:** Persistent event logging to JSON Lines format

**Key Features:**

1. **JSON Lines Format**: One event per line, easy to parse
2. **Automatic Rotation**: Size-based or time-based rotation
3. **Compression**: Optional gzip compression of rotated logs
4. **Concurrent Scans**: Supports logging multiple scans to same file
5. **Query-Friendly**: Easy analysis with jq, grep, or log aggregators

**Log Structure:**

Each log file contains:
- Header line: `{"event": "log_started", "timestamp": "...", "version": "1.0"}`
- Event lines: One JSON object per event
- Footer line: `{"event": "log_ended", "timestamp": "..."}`

**Example: Using EventLogger**

```rust
use prtip_core::event_logger::EventLogger;

// Create logger (subscribes to event_bus)
let logger = EventLogger::new("scans.jsonl", event_bus.clone())?;

// Start logging a scan
logger.start_scan(scan_id, &scan_config).await?;

// Events are automatically logged as they occur
// ... scan runs ...

// End logging
logger.end_scan(scan_id).await?;

// Manually flush if needed
logger.flush().await?;
```

---

## Event Types Reference

| Event Type | Category | When Emitted | Key Fields | Example |
|------------|----------|--------------|------------|---------|
| **ScanStarted** | Lifecycle | Scan begins | `scan_type`, `target_count`, `port_count` | Starting SYN scan of 1000 hosts, 100 ports |
| **ScanCompleted** | Lifecycle | Scan finishes | `targets_scanned`, `ports_scanned`, `duration` | Completed in 120s, found 42 open ports |
| **ScanCancelled** | Lifecycle | User cancels | `reason` | User pressed Ctrl+C |
| **ScanPaused** | Lifecycle | Scan paused | `reason` | Rate limit reached |
| **ScanResumed** | Lifecycle | Scan resumes | - | Resuming after pause |
| **HostDiscovered** | Discovery | Live host found | `ip`, `method`, `latency_ms` | 192.168.1.1 via ICMP echo (10ms) |
| **PortFound** | Discovery | Open port detected | `target`, `port`, `protocol`, `state` | TCP 192.168.1.1:80 OPEN |
| **IPv6PortFound** | Discovery | IPv6 port found | `target`, `port`, `protocol`, `state` | TCP [fe80::1]:443 OPEN |
| **ServiceDetected** | Detection | Service identified | `service_name`, `version`, `confidence` | HTTP/1.1 (Apache/2.4.52, 95%) |
| **OSDetected** | Detection | OS fingerprinted | `os_name`, `accuracy`, `device_type` | Linux 5.x (98%, server) |
| **BannerGrabbed** | Detection | Banner retrieved | `banner` | "SSH-2.0-OpenSSH_8.9p1" |
| **CertificateFound** | Detection | TLS cert discovered | `subject`, `issuer`, `validity` | CN=example.com, Let's Encrypt |
| **ProgressUpdate** | Progress | Periodic progress | `percentage`, `completed`, `total`, `eta` | 50% (500/1000), ETA 60s |
| **StageChanged** | Progress | Phase transition | `old_stage`, `new_stage` | Discovery → Port Scanning |
| **MetricRecorded** | Diagnostic | Metric captured | `metric`, `value` | PacketsSent: 10000 |
| **WarningIssued** | Diagnostic | Non-fatal warning | `message` | "Timeout on 192.168.1.1:80" |
| **RateLimitTriggered** | Diagnostic | Rate limiter active | `current_rate`, `limit` | 1000 pps (limit: 1000) |
| **RetryScheduled** | Diagnostic | Retry planned | `target`, `port`, `attempt` | Retry 192.168.1.1:80 (3/5) |

**Usage Examples:**

```rust
use prtip_core::events::ScanEvent;

// Lifecycle: Starting a scan
let event = ScanEvent::ScanStarted {
    scan_id,
    scan_type: ScanType::Syn,
    target_count: 1000,
    port_count: 100,
    timestamp,
};
event_bus.publish(event).await;

// Discovery: Found an open port
let event = ScanEvent::PortFound {
    scan_id,
    target: "192.168.1.1".parse().unwrap(),
    port: 80,
    protocol: Protocol::Tcp,
    state: PortState::Open,
    timestamp,
};
event_bus.publish(event).await;

// Detection: Service identified
let event = ScanEvent::ServiceDetected {
    scan_id,
    target: "192.168.1.1".parse().unwrap(),
    port: 80,
    service_name: "http".to_string(),
    version: Some("Apache/2.4.52".to_string()),
    confidence: 95,
    timestamp,
};
event_bus.publish(event).await;

// Progress: 50% complete
let event = ScanEvent::ProgressUpdate {
    scan_id,
    stage: ScanStage::ScanningPorts,
    percentage: 50.0,
    completed: 500,
    total: 1000,
    throughput: Throughput {
        packets_per_sec: 1000,
        ports_per_sec: 10
    },
    eta: Some(Duration::from_secs(60)),
    timestamp,
};
event_bus.publish(event).await;
```

---

## EventBus API Guide

### Publishing Events

**Basic Publishing:**

```rust
use prtip_core::event_bus::EventBus;
use prtip_core::events::ScanEvent;
use std::sync::Arc;

let event_bus = Arc::new(EventBus::new(1000));

// Create event
let event = ScanEvent::ScanStarted {
    scan_id,
    scan_type: ScanType::Syn,
    target_count: 1000,
    port_count: 100,
    timestamp: SystemTime::now(),
};

// Publish (async, non-blocking)
event_bus.publish(event).await;
```

**Batch Publishing:**

```rust
// Collect events in a batch
let mut events = Vec::new();
for port in 1..=100 {
    events.push(ScanEvent::PortFound {
        scan_id,
        target: "192.168.1.1".parse().unwrap(),
        port,
        protocol: Protocol::Tcp,
        state: PortState::Open,
        timestamp: SystemTime::now(),
    });
}

// Publish all events
for event in events {
    event_bus.publish(event).await;
}
```

**Performance Tip:** Publishing is extremely fast (~40ns), so batching is typically unnecessary unless you have >1000 events/second.

### Subscribing to Events

**Basic Subscription:**

```rust
use tokio::sync::mpsc;
use prtip_core::event_bus::EventFilter;

// Create channel for receiving events
let (tx, mut rx) = mpsc::unbounded_channel();

// Subscribe to all events
event_bus.subscribe(tx, EventFilter::All).await;

// Receive events
tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        match event {
            ScanEvent::PortFound { target, port, .. } => {
                println!("Found: {}:{}", target, port);
            }
            ScanEvent::ScanCompleted { .. } => {
                println!("Scan complete!");
                break;
            }
            _ => {}
        }
    }
});
```

**Filtered Subscription:**

```rust
// Subscribe to only port discoveries
let (tx, mut rx) = mpsc::unbounded_channel();
event_bus.subscribe(tx, EventFilter::EventType(vec![
    ScanEventType::PortFound,
    ScanEventType::IPv6PortFound,
])).await;

// Receive only port events
while let Some(event) = rx.recv().await {
    if let ScanEvent::PortFound { target, port, .. } = event {
        println!("{}:{}", target, port);
    }
}
```

**Multiple Subscribers:**

```rust
// Progress display
let (progress_tx, mut progress_rx) = mpsc::unbounded_channel();
event_bus.subscribe(progress_tx, EventFilter::EventType(vec![
    ScanEventType::ProgressUpdate,
])).await;

// Results display
let (results_tx, mut results_rx) = mpsc::unbounded_channel();
event_bus.subscribe(results_tx, EventFilter::EventType(vec![
    ScanEventType::PortFound,
    ScanEventType::ServiceDetected,
])).await;

// Both run concurrently
tokio::spawn(async move {
    while let Some(event) = progress_rx.recv().await {
        // Update progress bar
    }
});

tokio::spawn(async move {
    while let Some(event) = results_rx.recv().await {
        // Display result
    }
});
```

### Event Filtering

**Filter Types:**

1. **EventFilter::All** - Receive all events
2. **EventFilter::EventType(Vec<ScanEventType>)** - Specific event types
3. **EventFilter::ScanId(Uuid)** - Events from specific scan
4. **EventFilter::And(Vec<EventFilter>)** - Multiple filters (AND logic)

**Filter Examples:**

```rust
// All events (no filtering)
let filter = EventFilter::All;

// Only lifecycle events
let filter = EventFilter::EventType(vec![
    ScanEventType::ScanStarted,
    ScanEventType::ScanCompleted,
    ScanEventType::ScanCancelled,
]);

// Only events from specific scan
let filter = EventFilter::ScanId(my_scan_id);

// Combined: Progress from specific scan
let filter = EventFilter::And(vec![
    EventFilter::ScanId(my_scan_id),
    EventFilter::EventType(vec![ScanEventType::ProgressUpdate]),
]);
```

**Performance Considerations:**

- Filter evaluation is very fast (~2-5ns per filter)
- EventFilter::All is fastest (no evaluation needed)
- EventType filters are nearly as fast (simple Vec::contains)
- Combined filters (And) have minimal overhead

**Best Practice:** Use the most specific filter possible to reduce unnecessary event delivery.

---

## ProgressAggregator API Guide

### Basic Usage (ProgressAggregator)

**Creating and Starting:**

```rust
use prtip_core::progress::ProgressAggregator;
use std::sync::Arc;

// Create aggregator (automatically subscribes to progress events)
let aggregator = Arc::new(
    ProgressAggregator::new(event_bus.clone(), scan_id)
);

// Initialize with totals
// start(total_targets, total_ports)
aggregator.start(1000, 100).await;

// Aggregator now tracks all ProgressUpdate events for this scan_id
```

**Getting Current Metrics:**

```rust
// Get metrics (async, non-blocking)
let metrics = aggregator.get_current_metrics().await;

println!("Progress: {:.1}%", metrics.percentage);
println!("Completed: {}/{}", metrics.completed, metrics.total);
println!("Throughput: {} packets/sec", metrics.throughput.packets_per_sec);
println!("Throughput: {} ports/sec", metrics.throughput.ports_per_sec);

if let Some(eta) = metrics.eta {
    println!("ETA: {} seconds", eta.as_secs());
}

println!("Stage: {:?}", metrics.stage);
```

**Complete Example:**

```rust
use prtip_core::progress::ProgressAggregator;
use std::sync::Arc;
use tokio::time::{interval, Duration};

async fn monitor_progress(
    event_bus: Arc<EventBus>,
    scan_id: Uuid,
) {
    let aggregator = Arc::new(
        ProgressAggregator::new(event_bus.clone(), scan_id)
    );

    aggregator.start(1000, 100).await;

    // Update every second
    let mut ticker = interval(Duration::from_secs(1));

    loop {
        ticker.tick().await;

        let metrics = aggregator.get_current_metrics().await;

        print!("\rProgress: {:.1}% ({}/{}) ",
            metrics.percentage,
            metrics.completed,
            metrics.total
        );

        if let Some(eta) = metrics.eta {
            print!("ETA: {}s ", eta.as_secs());
        }

        print!("| {} pps", metrics.throughput.packets_per_sec);

        if metrics.percentage >= 100.0 {
            println!("\nComplete!");
            break;
        }
    }
}
```

### Real-Time Metrics

**Available Metrics:**

```rust
pub struct ProgressMetrics {
    pub percentage: f64,           // 0-100
    pub completed: usize,          // Items processed
    pub total: usize,              // Total items
    pub throughput: Throughput,    // Current rates
    pub eta: Option<Duration>,     // Estimated completion
    pub stage: ScanStage,          // Current phase
}

pub struct Throughput {
    pub packets_per_sec: u64,
    pub ports_per_sec: u64,
}
```

**Metric Update Frequency:**

- **percentage/completed/total**: Updated on every ProgressUpdate event
- **throughput**: Calculated from last 5 seconds of events (moving average)
- **eta**: Recalculated every update based on current throughput
- **stage**: Updated on StageChanged events

**Accuracy Guarantees:**

- **percentage**: Accurate to 0.1%
- **throughput**: Average of last 5 seconds (smoothed)
- **eta**: Best estimate based on current rate (may vary with network conditions)

### Multi-Scanner Scenarios

**Concurrent Scans (Separate IDs):**

```rust
// Scanner 1
let scan_id_1 = Uuid::new_v4();
let aggregator_1 = Arc::new(
    ProgressAggregator::new(event_bus.clone(), scan_id_1)
);
aggregator_1.start(1000, 100).await;

// Scanner 2
let scan_id_2 = Uuid::new_v4();
let aggregator_2 = Arc::new(
    ProgressAggregator::new(event_bus.clone(), scan_id_2)
);
aggregator_2.start(500, 50).await;

// Each aggregator tracks its own scan independently
let metrics_1 = aggregator_1.get_current_metrics().await;
let metrics_2 = aggregator_2.get_current_metrics().await;
```

**Global Aggregation (All Scans):**

```rust
// Create aggregator without scan_id filter
// (Subscribe to all ProgressUpdate events)
let global_aggregator = Arc::new(
    ProgressAggregator::new_global(event_bus.clone())
);

// Get combined metrics across all scans
let total_metrics = global_aggregator.get_current_metrics().await;
println!("Total progress: {:.1}%", total_metrics.percentage);
```

**Thread Safety:**

- All methods are async and thread-safe
- Uses Arc<RwLock<>> for concurrent access
- Read locks are non-blocking and fast (~10ns overhead)
- No data races or inconsistent states

---

## EventLogger API Guide

### Basic Logging

**Creating and Starting:**

```rust
use prtip_core::event_logger::EventLogger;
use std::path::Path;

// Create logger (automatically subscribes to all events)
let logger = EventLogger::new(
    Path::new("scans.jsonl"),
    event_bus.clone()
)?;

// Start logging a specific scan
logger.start_scan(scan_id, &scan_config).await?;

// Events are automatically logged as they're published
// ... scan runs ...

// End logging for this scan
logger.end_scan(scan_id).await?;

// Manually flush to disk (optional, auto-flushed periodically)
logger.flush().await?;
```

**Complete Example:**

```rust
use prtip_core::event_logger::EventLogger;
use std::path::Path;

async fn log_scan(
    event_bus: Arc<EventBus>,
    scan_id: Uuid,
    config: &ScanConfig,
) -> Result<(), Error> {
    // Create logger
    let logger = EventLogger::new(
        Path::new("scans.jsonl"),
        event_bus.clone()
    )?;

    // Start logging
    logger.start_scan(scan_id, config).await?;

    // Run scan (events logged automatically)
    // ...

    // End logging
    logger.end_scan(scan_id).await?;

    Ok(())
}
```

### JSON Lines Format

**Format Specification:**

```jsonl
{"event":"log_started","timestamp":"2025-11-09T12:00:00Z","version":"1.0"}
{"event":"scan_started","scan_id":"123e4567-e89b-12d3-a456-426614174000","scan_type":"syn","target_count":1000,"port_count":100,"timestamp":"2025-11-09T12:00:01Z"}
{"event":"port_found","scan_id":"123e4567-e89b-12d3-a456-426614174000","target":"192.168.1.1","port":80,"protocol":"tcp","state":"open","timestamp":"2025-11-09T12:00:02Z"}
{"event":"service_detected","scan_id":"123e4567-e89b-12d3-a456-426614174000","target":"192.168.1.1","port":80,"service_name":"http","version":"Apache/2.4.52","confidence":95,"timestamp":"2025-11-09T12:00:03Z"}
{"event":"scan_completed","scan_id":"123e4567-e89b-12d3-a456-426614174000","targets_scanned":1000,"ports_scanned":100,"duration_secs":120,"timestamp":"2025-11-09T12:02:01Z"}
{"event":"log_ended","timestamp":"2025-11-09T12:02:01Z"}
```

**Parsing Examples:**

**Using jq (JSON query tool):**

```bash
# Get all open ports
jq 'select(.event == "port_found" and .state == "open")' scans.jsonl

# Count services by type
jq 'select(.event == "service_detected") | .service_name' scans.jsonl | sort | uniq -c

# Get scan duration
jq 'select(.event == "scan_completed") | .duration_secs' scans.jsonl

# Filter by time range
jq 'select(.timestamp >= "2025-11-09T12:00:00Z" and .timestamp < "2025-11-09T13:00:00Z")' scans.jsonl

# Extract specific scan
jq 'select(.scan_id == "123e4567-e89b-12d3-a456-426614174000")' scans.jsonl
```

**Using grep:**

```bash
# Find all HTTP services
grep '"service_name":"http"' scans.jsonl

# Find errors
grep '"event":"scan_error"' scans.jsonl

# Find specific IP
grep '"target":"192.168.1.1"' scans.jsonl
```

### Log Management

**Automatic Rotation:**

```rust
use prtip_core::event_logger::{EventLogger, RotationPolicy};

let logger = EventLogger::with_rotation(
    Path::new("scans.jsonl"),
    event_bus.clone(),
    RotationPolicy::Size(10 * 1024 * 1024) // 10 MB
)?;

// Rotated files: scans.jsonl.1, scans.jsonl.2, ...
```

**Compression:**

```rust
use prtip_core::event_logger::{EventLogger, CompressionPolicy};

let logger = EventLogger::with_compression(
    Path::new("scans.jsonl"),
    event_bus.clone(),
    CompressionPolicy::Gzip
)?;

// Compressed rotated files: scans.jsonl.1.gz, scans.jsonl.2.gz
```

**Cleanup Strategies:**

```rust
// Keep last 7 days of logs
logger.cleanup_old_logs(Duration::from_secs(7 * 24 * 3600)).await?;

// Keep only 100 most recent logs
logger.cleanup_by_count(100).await?;

// Delete logs older than specific timestamp
logger.cleanup_before(SystemTime::now() - Duration::from_secs(30 * 24 * 3600)).await?;
```

**Complete Example:**

```rust
use prtip_core::event_logger::{EventLogger, RotationPolicy, CompressionPolicy};
use std::path::Path;
use std::time::Duration;

async fn managed_logging(
    event_bus: Arc<EventBus>,
    scan_id: Uuid,
) -> Result<(), Error> {
    // Create logger with rotation and compression
    let logger = EventLogger::builder()
        .path(Path::new("scans.jsonl"))
        .event_bus(event_bus.clone())
        .rotation(RotationPolicy::Size(10 * 1024 * 1024)) // 10 MB
        .compression(CompressionPolicy::Gzip)
        .build()?;

    // Start logging
    logger.start_scan(scan_id, &config).await?;

    // Run scan
    // ...

    // End logging
    logger.end_scan(scan_id).await?;

    // Cleanup old logs (keep last 7 days)
    logger.cleanup_old_logs(Duration::from_secs(7 * 24 * 3600)).await?;

    Ok(())
}
```

---

## Integration Patterns

### Scanner Integration

**How Scanners Should Publish Events:**

1. **Initialization**: Publish `ScanStarted` before first packet
2. **Discovery**: Publish `HostDiscovered` / `PortFound` as discovered
3. **Progress**: Publish `ProgressUpdate` every N packets or N seconds
4. **Detection**: Publish `ServiceDetected` / `BannerGrabbed` as identified
5. **Completion**: Publish `ScanCompleted` after final packet
6. **Errors**: Publish `ScanError` / `WarningIssued` for issues

**Event Emission Pattern:**

```rust
use prtip_core::event_bus::EventBus;
use prtip_core::events::ScanEvent;
use std::sync::Arc;

pub struct SynScanner {
    event_bus: Arc<EventBus>,
    scan_id: Uuid,
    // ... other fields
}

impl SynScanner {
    pub async fn scan(&self) -> Result<(), Error> {
        // 1. Start event
        self.event_bus.publish(ScanEvent::ScanStarted {
            scan_id: self.scan_id,
            scan_type: ScanType::Syn,
            target_count: self.targets.len(),
            port_count: self.ports.len(),
            timestamp: SystemTime::now(),
        }).await;

        let mut completed = 0;
        let total = self.targets.len() * self.ports.len();

        // 2. Discovery loop
        for target in &self.targets {
            for port in &self.ports {
                // Send probe
                self.send_syn(target, port).await?;

                // Check for response
                if let Some(state) = self.check_response(target, port).await {
                    // 3. Discovery event
                    self.event_bus.publish(ScanEvent::PortFound {
                        scan_id: self.scan_id,
                        target: *target,
                        port: *port,
                        protocol: Protocol::Tcp,
                        state,
                        timestamp: SystemTime::now(),
                    }).await;
                }

                completed += 1;

                // 4. Progress event (every 1% or every second)
                if completed % (total / 100).max(1) == 0 {
                    self.event_bus.publish(ScanEvent::ProgressUpdate {
                        scan_id: self.scan_id,
                        stage: ScanStage::ScanningPorts,
                        percentage: (completed as f64 / total as f64) * 100.0,
                        completed,
                        total,
                        throughput: self.calculate_throughput(),
                        eta: self.estimate_eta(completed, total),
                        timestamp: SystemTime::now(),
                    }).await;
                }
            }
        }

        // 5. Completion event
        self.event_bus.publish(ScanEvent::ScanCompleted {
            scan_id: self.scan_id,
            targets_scanned: self.targets.len(),
            ports_scanned: completed,
            duration: self.start_time.elapsed(),
            timestamp: SystemTime::now(),
        }).await;

        Ok(())
    }
}
```

**Performance Considerations:**

- **Batch progress updates**: Don't publish every packet (every 1% or 1 second is fine)
- **Async publishing**: Use `.await` but don't block on event delivery
- **Error handling**: Always publish errors, never silently fail
- **Overhead**: Event publishing is ~40ns, negligible impact

### CLI Integration

**Progress Display:**

```rust
use tokio::sync::mpsc;
use prtip_core::event_bus::{EventBus, EventFilter};
use prtip_core::events::{ScanEvent, ScanEventType};

async fn cli_progress_display(
    event_bus: Arc<EventBus>,
    scan_id: Uuid,
) {
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Subscribe to progress and results
    event_bus.subscribe(tx, EventFilter::EventType(vec![
        ScanEventType::ProgressUpdate,
        ScanEventType::PortFound,
        ScanEventType::ScanCompleted,
    ])).await;

    while let Some(event) = rx.recv().await {
        match event {
            ScanEvent::ProgressUpdate { percentage, completed, total, eta, .. } => {
                print!("\rProgress: {:.1}% ({}/{}) ", percentage, completed, total);
                if let Some(eta) = eta {
                    print!("ETA: {}s", eta.as_secs());
                }
                std::io::stdout().flush().unwrap();
            }

            ScanEvent::PortFound { target, port, state, .. } => {
                println!("\n[+] {}:{} {}", target, port, state);
            }

            ScanEvent::ScanCompleted { .. } => {
                println!("\n[✓] Scan complete!");
                break;
            }

            _ => {}
        }
    }
}
```

**Live Results:**

```rust
async fn cli_live_results(
    event_bus: Arc<EventBus>,
    scan_id: Uuid,
) {
    let (tx, mut rx) = mpsc::unbounded_channel();

    event_bus.subscribe(tx, EventFilter::EventType(vec![
        ScanEventType::PortFound,
        ScanEventType::ServiceDetected,
    ])).await;

    println!("TARGET          PORT   STATE   SERVICE");
    println!("----------------------------------------");

    while let Some(event) = rx.recv().await {
        match event {
            ScanEvent::PortFound { target, port, state, .. } => {
                println!("{:<15} {:<6} {:<7}", target, port, state);
            }

            ScanEvent::ServiceDetected { target, port, service_name, version, .. } => {
                let service = match version {
                    Some(v) => format!("{} ({})", service_name, v),
                    None => service_name,
                };
                println!("{:<15} {:<6} {:<7} {}", target, port, "OPEN", service);
            }

            _ => {}
        }
    }
}
```

### TUI Integration (Phase 6)

**Real-Time Dashboard:**

```rust
use prtip_core::progress::ProgressAggregator;
use tokio::sync::mpsc;

struct TuiDashboard {
    event_bus: Arc<EventBus>,
    scan_id: Uuid,
    aggregator: Arc<ProgressAggregator>,
}

impl TuiDashboard {
    async fn new(event_bus: Arc<EventBus>, scan_id: Uuid) -> Self {
        let aggregator = Arc::new(
            ProgressAggregator::new(event_bus.clone(), scan_id)
        );

        Self {
            event_bus,
            scan_id,
            aggregator,
        }
    }

    async fn run(&self) {
        // Subscribe to all events
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.event_bus.subscribe(tx, EventFilter::All).await;

        // Update loop
        let mut update_interval = tokio::time::interval(
            tokio::time::Duration::from_millis(100)
        );

        loop {
            tokio::select! {
                // Process events
                Some(event) = rx.recv() => {
                    self.handle_event(event).await;
                }

                // Periodic UI update
                _ = update_interval.tick() => {
                    let metrics = self.aggregator.get_current_metrics().await;
                    self.update_ui(metrics);
                }
            }
        }
    }

    async fn handle_event(&self, event: ScanEvent) {
        match event {
            ScanEvent::PortFound { target, port, .. } => {
                // Add to results table
            }
            ScanEvent::ServiceDetected { service_name, .. } => {
                // Add to services table
            }
            ScanEvent::WarningIssued { message, .. } => {
                // Show notification
            }
            _ => {}
        }
    }

    fn update_ui(&self, metrics: ProgressMetrics) {
        // Render progress bar
        // Update throughput gauge
        // Update ETA label
        // Refresh display
    }
}
```

**State Management:**

```rust
use std::collections::HashMap;

struct TuiState {
    scans: HashMap<Uuid, ScanState>,
}

struct ScanState {
    config: ScanConfig,
    progress: ProgressMetrics,
    results: Vec<ScanResult>,
    warnings: Vec<String>,
}

impl TuiState {
    fn update_from_event(&mut self, event: ScanEvent) {
        let scan_id = event.scan_id();
        let state = self.scans.entry(scan_id).or_default();

        match event {
            ScanEvent::ProgressUpdate { percentage, .. } => {
                state.progress.percentage = percentage;
            }
            ScanEvent::PortFound { target, port, .. } => {
                state.results.push(ScanResult { target, port });
            }
            ScanEvent::WarningIssued { message, .. } => {
                state.warnings.push(message);
            }
            _ => {}
        }
    }
}
```

---

## Performance Optimization

### Publish Performance

**Current Performance:**
- ~40ns per event (no subscribers)
- ~340ns end-to-end (with subscribers)
- >10M events/second throughput

**Optimization Strategies:**

1. **Batch Progress Updates**

```rust
// ❌ Bad: Publish every packet
for packet in packets {
    send(packet);
    event_bus.publish(ProgressUpdate { ... }).await;  // 10,000/sec!
}

// ✅ Good: Publish every 1% or 1 second
let mut last_update = Instant::now();
for (i, packet) in packets.iter().enumerate() {
    send(packet);

    if i % (total / 100) == 0 || last_update.elapsed() > Duration::from_secs(1) {
        event_bus.publish(ProgressUpdate { ... }).await;
        last_update = Instant::now();
    }
}
```

2. **Avoid Unnecessary Clones**

```rust
// ❌ Bad: Clone before publish
let event = create_event();
let event_clone = event.clone();
event_bus.publish(event_clone).await;

// ✅ Good: Move into publish
let event = create_event();
event_bus.publish(event).await;  // No clone needed
```

3. **Pre-allocate Event Data**

```rust
// ❌ Bad: Allocate on every event
event_bus.publish(ScanEvent::ServiceDetected {
    service_name: format!("http-{}", version),  // Allocation!
    ...
}).await;

// ✅ Good: Pre-allocate strings
let service_name = format!("http-{}", version);
event_bus.publish(ScanEvent::ServiceDetected {
    service_name,
    ...
}).await;
```

### Subscription Performance

**Filter Optimization:**

```rust
// ❌ Bad: Subscribe to all, filter in handler
let (tx, mut rx) = mpsc::unbounded_channel();
event_bus.subscribe(tx, EventFilter::All).await;

while let Some(event) = rx.recv().await {
    if matches!(event, ScanEvent::PortFound { .. }) {
        handle(event);
    }
}

// ✅ Good: Use EventBus filter
let (tx, mut rx) = mpsc::unbounded_channel();
event_bus.subscribe(tx, EventFilter::EventType(vec![
    ScanEventType::PortFound,
])).await;

while let Some(event) = rx.recv().await {
    handle(event);  // Already filtered
}
```

**Channel Buffer Sizing:**

```rust
// For high-throughput subscriptions
let (tx, rx) = mpsc::channel(10000);  // Buffered channel

// For low-latency subscriptions
let (tx, rx) = mpsc::unbounded_channel();  // No buffering
```

### Memory Management

**Ring Buffer Tuning:**

```rust
// Default: 1,000 events (~200-500 KB)
let event_bus = EventBus::new(1000);

// Low memory: 100 events (~20-50 KB)
let event_bus = EventBus::new(100);

// High memory: 10,000 events (~2-5 MB)
let event_bus = EventBus::new(10000);
```

**Subscriber Cleanup:**

```rust
// Subscribers are automatically cleaned up when channel is dropped
{
    let (tx, mut rx) = mpsc::unbounded_channel();
    event_bus.subscribe(tx, EventFilter::All).await;

    // Use rx...
}  // rx dropped here, subscriber removed from EventBus
```

**Event Size Considerations:**

- `ScanStarted`: ~100 bytes
- `PortFound`: ~80 bytes
- `ServiceDetected`: ~200 bytes (has strings)
- `ProgressUpdate`: ~150 bytes

**Best Practice:** If logging all events, expect ~200 bytes/event average.

---

## Error Handling

### Common Errors

**Channel Closed:**

```rust
// Subscriber channel closed (normal on exit)
while let Some(event) = rx.recv().await {
    // Handle event
}
// rx.recv() returns None when channel closed
```

**Publish Failures:**

```rust
// EventBus.publish() is infallible (never fails)
// Events are delivered to all active subscribers
// Closed subscribers are automatically removed
```

### Error Recovery

**Reconnecting Subscriber:**

```rust
async fn resilient_subscriber(
    event_bus: Arc<EventBus>,
) {
    loop {
        let (tx, mut rx) = mpsc::unbounded_channel();
        event_bus.subscribe(tx, EventFilter::All).await;

        while let Some(event) = rx.recv().await {
            if let Err(e) = handle_event(event).await {
                eprintln!("Event handling error: {}", e);
                // Continue processing other events
            }
        }

        // Channel closed, reconnect after delay
        eprintln!("Subscriber disconnected, reconnecting...");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

**Graceful Degradation:**

```rust
async fn optional_logging(
    event_bus: Arc<EventBus>,
    scan_id: Uuid,
) {
    // Logging is optional, don't fail scan if logger fails
    match EventLogger::new(Path::new("scans.jsonl"), event_bus.clone()) {
        Ok(logger) => {
            logger.start_scan(scan_id, &config).await.ok();
            // Continue even if start_scan fails
        }
        Err(e) => {
            eprintln!("Logger initialization failed: {}", e);
            // Continue without logging
        }
    }
}
```

### Best Practices (Error Handling)

1. **Never Panic on Event Handling**

```rust
// ❌ Bad
while let Some(event) = rx.recv().await {
    handle_event(event).unwrap();  // Panics on error!
}

// ✅ Good
while let Some(event) = rx.recv().await {
    if let Err(e) = handle_event(event).await {
        eprintln!("Error handling event: {}", e);
    }
}
```

2. **Always Check Channel Closed**

```rust
// ✅ Good
while let Some(event) = rx.recv().await {
    // Handle event
}
println!("Channel closed, exiting gracefully");
```

3. **Log Errors Appropriately**

```rust
match event_logger.start_scan(scan_id, &config).await {
    Ok(_) => {},
    Err(e) => {
        eprintln!("Failed to start event logging: {}", e);
        // Continue without logging
    }
}
```

---

## Testing Strategies

### Unit Testing

**Mocking EventBus:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use prtip_core::event_bus::EventBus;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_scanner_publishes_events() {
        let event_bus = Arc::new(EventBus::new(100));
        let scan_id = Uuid::new_v4();

        let scanner = TestScanner::new(event_bus.clone(), scan_id);

        // Run scanner
        scanner.scan().await.unwrap();

        // Verify events were published
        let history = event_bus.get_history(100).await;
        assert!(history.iter().any(|e| matches!(e, ScanEvent::ScanStarted { .. })));
        assert!(history.iter().any(|e| matches!(e, ScanEvent::ScanCompleted { .. })));
    }
}
```

**Testing Event Emission:**

```rust
#[tokio::test]
async fn test_port_found_event() {
    let event_bus = Arc::new(EventBus::new(100));
    let scan_id = Uuid::new_v4();

    // Subscribe to events
    let (tx, mut rx) = mpsc::unbounded_channel();
    event_bus.subscribe(tx, EventFilter::EventType(vec![
        ScanEventType::PortFound,
    ])).await;

    // Publish event
    event_bus.publish(ScanEvent::PortFound {
        scan_id,
        target: "192.168.1.1".parse().unwrap(),
        port: 80,
        protocol: Protocol::Tcp,
        state: PortState::Open,
        timestamp: SystemTime::now(),
    }).await;

    // Verify received
    let event = tokio::time::timeout(
        Duration::from_secs(1),
        rx.recv()
    ).await.unwrap().unwrap();

    assert!(matches!(event, ScanEvent::PortFound { port: 80, .. }));
}
```

### Integration Testing

**End-to-End Event Flow:**

```rust
#[tokio::test]
async fn test_full_scan_event_flow() {
    let event_bus = Arc::new(EventBus::new(1000));
    let scan_id = Uuid::new_v4();

    // Create aggregator
    let aggregator = Arc::new(
        ProgressAggregator::new(event_bus.clone(), scan_id)
    );
    aggregator.start(100, 10).await;

    // Create logger
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let logger = EventLogger::new(
        temp_file.path(),
        event_bus.clone()
    ).unwrap();
    logger.start_scan(scan_id, &test_config()).await.unwrap();

    // Run scan
    let scanner = TestScanner::new(event_bus.clone(), scan_id);
    scanner.scan().await.unwrap();

    // Verify aggregator updated
    let metrics = aggregator.get_current_metrics().await;
    assert_eq!(metrics.percentage, 100.0);

    // Verify events logged
    logger.end_scan(scan_id).await.unwrap();
    let log_content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(log_content.contains("scan_started"));
    assert!(log_content.contains("scan_completed"));
}
```

### Performance Testing

**Using Benchmarks:**

```bash
# Run event system benchmarks
cargo bench --package prtip-core --bench event_system

# Check for regressions
# - Publish latency should stay <100ns
# - Subscribe latency should stay <5μs
# - Concurrent overhead should stay <10%
```

**Reference Baseline:**

See `benchmarks/event-system-baseline.md` for performance targets:
- Publish: ~40ns
- Subscribe: ~1.2μs
- End-to-end: ~340ns
- Concurrent (16 threads): 4.2% overhead

---

## Debugging Techniques

### Event Tracing

**Using History Queries:**

```rust
// Get last 100 events
let recent = event_bus.get_history(100).await;
for event in recent {
    println!("{:?}", event);
}

// Get events from specific scan
let scan_events = event_bus.query_history(
    EventFilter::ScanId(scan_id),
    1000
).await;

// Get events in time range
let start = SystemTime::now() - Duration::from_secs(60);
let end = SystemTime::now();
let time_range_events = event_bus.query_history(
    EventFilter::TimeRange(start, end),
    1000
).await;
```

**Debug Subscriber:**

```rust
// Subscribe to all events and log
async fn debug_subscriber(event_bus: Arc<EventBus>) {
    let (tx, mut rx) = mpsc::unbounded_channel();
    event_bus.subscribe(tx, EventFilter::All).await;

    while let Some(event) = rx.recv().await {
        println!("[DEBUG] {:?}", event);
    }
}
```

### Performance Profiling

**Identifying Bottlenecks:**

```rust
use std::time::Instant;

let start = Instant::now();
event_bus.publish(event).await;
let duration = start.elapsed();

if duration > Duration::from_micros(1000) {
    eprintln!("Slow publish: {:?}", duration);
}
```

**Throughput Monitoring:**

```rust
let mut count = 0;
let start = Instant::now();

for event in events {
    event_bus.publish(event).await;
    count += 1;
}

let duration = start.elapsed();
let throughput = count as f64 / duration.as_secs_f64();
println!("Throughput: {:.0} events/sec", throughput);
```

### Common Issues

**Slow Event Processing:**

```
Symptom: Events accumulating in channel buffer
Diagnosis: Subscriber handler is too slow
Solution: Offload heavy work to separate task

// ❌ Bad: Blocking handler
while let Some(event) = rx.recv().await {
    slow_operation(event);  // Blocks event processing!
}

// ✅ Good: Non-blocking handler
while let Some(event) = rx.recv().await {
    tokio::spawn(async move {
        slow_operation(event);
    });
}
```

**Memory Leaks:**

```
Symptom: Growing memory usage
Diagnosis: EventBus history growing unbounded
Solution: Reduce history size or clear periodically

event_bus.clear_history().await;
```

**Subscriber Hangs:**

```
Symptom: Subscriber not receiving events
Diagnosis: Channel full or closed
Solution: Check channel status and buffer size

// Use unbounded channel for high-throughput
let (tx, rx) = mpsc::unbounded_channel();

// Or increase buffer size
let (tx, rx) = mpsc::channel(10000);
```

---

## Best Practices

**Event Naming Conventions:**

- Use descriptive event types: `PortFound`, not `Found`
- Include context in event data: `target`, `port`, `timestamp`
- Consistent field naming: `scan_id`, not `id` or `scanId`

**When to Emit Events:**

- **Always**: Lifecycle events (Started, Completed, Error)
- **Frequently**: Discovery events (HostDiscovered, PortFound)
- **Periodically**: Progress events (every 1% or 1 second)
- **Sparingly**: Diagnostic events (only when relevant)

**Filter Design Guidelines:**

- Use most specific filter possible to reduce overhead
- Combine filters with `And` for complex scenarios
- Prefer `EventType` filter over `All` when possible

**Performance Considerations:**

- Batch progress updates (every 1% or 1 second, not every packet)
- Avoid unnecessary clones (move events when possible)
- Use async/await but don't block on event delivery
- Monitor event throughput in high-volume scenarios

**Error Handling Patterns:**

- Never panic on event handling errors
- Always check channel closed (rx.recv() returns None)
- Log errors appropriately but continue processing
- Graceful degradation for optional components (logging)

**Testing Recommendations:**

- Unit test: Event emission and filtering
- Integration test: End-to-end event flow (scanner → bus → subscriber)
- Performance test: Use benchmarks to detect regressions
- Mock EventBus for isolated component testing

---

## Advanced Topics

### Custom Event Types

**Extending ScanEvent:**

```rust
// Current: 18 event variants
pub enum ScanEvent {
    ScanStarted { ... },
    PortFound { ... },
    // ...
}

// Future: Add new event type
pub enum ScanEvent {
    // Existing events...

    // New event
    VulnerabilityFound {
        scan_id: Uuid,
        target: IpAddr,
        port: u16,
        cve_id: String,
        severity: Severity,
        timestamp: SystemTime,
    },
}
```

**Backwards Compatibility:**

- Add new variants to end of enum
- Never remove or reorder existing variants
- Use non-exhaustive match patterns in subscribers

```rust
match event {
    ScanEvent::PortFound { .. } => { /* handle */ }
    ScanEvent::ServiceDetected { .. } => { /* handle */ }
    _ => { /* ignore unknown events */ }
}
```

### Event Batching

**When to Batch:**

- High-frequency events (>1000/second)
- Network transport (reduce protocol overhead)
- Bulk import/export scenarios

**Implementation:**

```rust
// Collect events in batch
let mut batch = Vec::new();
for event in events {
    batch.push(event);

    if batch.len() >= 100 {
        // Publish batch
        for event in batch.drain(..) {
            event_bus.publish(event).await;
        }
    }
}

// Publish remaining
for event in batch {
    event_bus.publish(event).await;
}
```

**Trade-offs:**

- **Pro**: Reduced overhead, higher throughput
- **Con**: Increased latency, more complex code
- **Recommendation**: Only batch if >1000 events/second

### Distributed Events

**Multi-Process Scenarios:**

Currently, EventBus is in-process only. For distributed scanning:

1. **Shared Memory**: Use shared memory IPC (unsafe, complex)
2. **Network Protocol**: Implement event streaming over TCP/gRPC
3. **Message Broker**: Use Redis/RabbitMQ/Kafka for event distribution

**Future Work:**

- Serializable event format (already JSON-compatible)
- Network event transport (gRPC/protobuf)
- Event aggregation across nodes
- Distributed progress tracking

---

## Troubleshooting

### Performance Issues

**Symptom:** Slow event processing, growing channel buffer

**Diagnosis:**

1. Check subscriber handler speed:
   ```rust
   let start = Instant::now();
   handle_event(event).await;
   println!("Handler took: {:?}", start.elapsed());
   ```

2. Check event throughput:
   ```rust
   let history = event_bus.get_history(1000).await;
   println!("Events in last 1000: {}", history.len());
   ```

**Solutions:**

1. Offload heavy work to separate tasks:
   ```rust
   tokio::spawn(async move {
       slow_operation(event);
   });
   ```

2. Use buffered channel:
   ```rust
   let (tx, rx) = mpsc::channel(10000);
   ```

3. Reduce event frequency (batch progress updates)

### Memory Issues

**Symptom:** Growing memory usage, OOM errors

**Diagnosis:**

1. Check EventBus history size:
   ```rust
   let history = event_bus.get_history(10000).await;
   let memory = history.len() * 200; // ~200 bytes/event
   println!("EventBus memory: {} KB", memory / 1024);
   ```

2. Check subscriber count:
   ```rust
   let count = event_bus.subscriber_count().await;
   println!("Subscribers: {}", count);
   ```

**Solutions:**

1. Reduce history size:
   ```rust
   let event_bus = EventBus::new(100); // Instead of 1000
   ```

2. Clear history periodically:
   ```rust
   event_bus.clear_history().await;
   ```

3. Remove closed subscribers (automatic, but verify)

### Reliability Issues

**Symptom:** Lost events, missing updates

**Diagnosis:**

1. Check channel status:
   ```rust
   if rx.is_closed() {
       eprintln!("Channel closed!");
   }
   ```

2. Check filter configuration:
   ```rust
   // Verify filter is correct
   let filter = EventFilter::EventType(vec![
       ScanEventType::PortFound,
   ]);
   ```

**Solutions:**

1. Use `EventFilter::All` temporarily to debug:
   ```rust
   event_bus.subscribe(tx, EventFilter::All).await;
   ```

2. Check event publishing:
   ```rust
   // Add debug logging
   eprintln!("Publishing event: {:?}", event);
   event_bus.publish(event).await;
   ```

3. Verify scan_id matches:
   ```rust
   // Ensure using same scan_id
   assert_eq!(event.scan_id(), aggregator.scan_id);
   ```

---

## API Quick Reference

| API | Purpose | Example | Notes |
|-----|---------|---------|-------|
| `EventBus::new(size)` | Create event bus | `EventBus::new(1000)` | Ring buffer size |
| `bus.publish(event)` | Publish event | `bus.publish(event).await` | Async, non-blocking |
| `bus.subscribe(tx, filter)` | Subscribe | `bus.subscribe(tx, EventFilter::All).await` | Returns subscriber ID |
| `bus.get_history(count)` | Get recent events | `bus.get_history(100).await` | Last N events |
| `bus.query_history(filter, count)` | Query events | `bus.query_history(filter, 100).await` | Filtered history |
| `bus.clear_history()` | Clear history | `bus.clear_history().await` | Free memory |
| `ProgressAggregator::new(bus, id)` | Create aggregator | `ProgressAggregator::new(bus, id)` | Auto-subscribes |
| `aggregator.start(targets, ports)` | Initialize | `aggregator.start(1000, 100).await` | Set totals |
| `aggregator.get_current_metrics()` | Get metrics | `aggregator.get_current_metrics().await` | Thread-safe |
| `EventLogger::new(path, bus)` | Create logger | `EventLogger::new(path, bus)?` | Auto-subscribes |
| `logger.start_scan(id, config)` | Start logging | `logger.start_scan(id, &config).await?` | Writes header |
| `logger.end_scan(id)` | End logging | `logger.end_scan(id).await?` | Writes footer |
| `logger.flush()` | Flush to disk | `logger.flush().await?` | Manual flush |
| `EventFilter::All` | All events | `EventFilter::All` | No filtering |
| `EventFilter::EventType(types)` | Specific types | `EventFilter::EventType(vec![...])` | Common pattern |
| `EventFilter::ScanId(id)` | Specific scan | `EventFilter::ScanId(uuid)` | Single scan |
| `EventFilter::And(filters)` | Combined | `EventFilter::And(vec![...])` | Multiple filters |

---

## References & Related Documentation

### Source Code

- **EventBus**: `crates/prtip-core/src/event_bus.rs`
- **ScanEvent**: `crates/prtip-core/src/events/mod.rs`
- **ProgressAggregator**: `crates/prtip-core/src/progress/mod.rs`
- **EventLogger**: `crates/prtip-core/src/event_logger.rs`

### Benchmarks

- **Benchmark Suite**: `crates/prtip-core/benches/event_system.rs`
- **Performance Baseline**: `benchmarks/event-system-baseline.md`

### Tests

- **104 Event System Tests**: See test modules in source files
  - EventBus: 9 tests
  - Events: 43 tests
  - Progress: 42 tests
  - EventLogger: 10 tests

### Related Guides

- **Architecture**: `docs/00-ARCHITECTURE.md` - System design overview
- **Implementation**: `docs/04-IMPLEMENTATION-GUIDE.md` - Code structure
- **Testing**: `docs/06-TESTING.md` - Testing strategy
- **Progress Tracking**: `docs/10-PROJECT-STATUS.md` - Project status

### External Resources

- **Tokio Documentation**: https://tokio.rs/
- **MPSC Channels**: https://docs.rs/tokio/latest/tokio/sync/mpsc/
- **JSON Lines Format**: https://jsonlines.org/
- **jq Manual**: https://stedolan.github.io/jq/manual/

---

**Document Version:** 2.0.0
**Last Updated:** 2025-11-09
**Maintainer:** ProRT-IP Development Team
**License:** GPL-3.0

For questions or contributions, see `CONTRIBUTING.md` or open an issue on GitHub.
