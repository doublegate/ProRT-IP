# Event System

ProRT-IP's event system provides real-time monitoring, progress tracking, and audit logging through a high-performance publish-subscribe architecture.

## Overview

The event system enables:
- **Real-Time Visibility**: Immediate feedback on scan progress and discoveries
- **Live Progress Displays**: Accurate ETA calculations with current throughput metrics
- **Audit Logging**: Comprehensive event recording for compliance and forensics
- **Performance Monitoring**: Track throughput, latency, and resource usage
- **TUI Integration**: Powers the live dashboard with real-time updates

**Performance**: Sub-microsecond event delivery (40ns publish, 340ns end-to-end) with industry-leading -4.1% overhead.

## Event Types

ProRT-IP tracks 18 event types across 5 categories:

### Lifecycle Events

Track scan execution state:

- **ScanStarted**: Initialization complete, scanning begins
- **ScanCompleted**: Scan finished successfully
- **ScanCancelled**: User requested cancellation
- **ScanPaused** / **ScanResumed**: Pause/resume operations

### Discovery Events

Report discovered hosts and ports:

- **HostDiscovered**: Live host found via ICMP, ARP, or probe
- **PortFound**: Open port detected (IPv4)
- **IPv6PortFound**: Open port discovered on IPv6 address

### Detection Events

Provide service and OS identification results:

- **ServiceDetected**: Service identified with version and confidence
- **OSDetected**: Operating system fingerprinted
- **BannerGrabbed**: Application banner retrieved
- **CertificateFound**: TLS certificate discovered

### Progress Events

Enable real-time progress tracking:

- **ProgressUpdate**: Percentage, throughput, ETA calculations
- **StageChanged**: Scan phase transitions (discovery → scanning → detection)

### Diagnostic Events

Monitor performance and issues:

- **MetricRecorded**: Performance metrics (packets sent, errors)
- **WarningIssued**: Non-fatal warnings (timeouts, rate limits)
- **RateLimitTriggered**: Rate limiter activation
- **RetryScheduled**: Failed operation retry planned

## Event Bus

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        EventBus                              │
│  ┌────────────────────────────────────────────────────┐     │
│  │  Ring Buffer (1,000 events history)                │     │
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
```

### Features

- **Non-Blocking**: Asynchronous event publication
- **History**: Ring buffer of last 1,000 events
- **Filtering**: Subscribe to specific event types or time ranges
- **Thread-Safe**: Safe concurrent access from multiple scanners
- **High Performance**: >10M events/second throughput

## Progress Tracking

### Real-Time Metrics

The event system enables accurate progress tracking with:

**Percentage Complete**: Current scan progress (0-100%)

**ETA Calculation**: Estimated time to completion based on:
- Current throughput (packets/sec, ports/sec)
- Work remaining
- Historical performance

**Throughput Monitoring**:
- Packets per second
- Ports per second
- Targets per minute
- Bandwidth utilization

**Stage Tracking**: Current scan phase
- Stage 1: Target Resolution
- Stage 2: Host Discovery
- Stage 3: Port Scanning
- Stage 4: Service Detection
- Stage 5: Finalization

### CLI Progress Display

**Compact Mode (Default)**:

```
[Stage 3/5] Port Scanning ▓▓▓▓▓▓▓▓▓░ 87% | ETA: 3m 24s
```

**Detailed Mode**:

```bash
prtip --progress-style detailed -sS -p- 192.168.1.0/24
```

Shows:
- Percentage complete
- ETA with color-coded accuracy
- Packets per second
- Hosts per minute
- Bandwidth usage

**Multi-Stage Bars**:

```bash
prtip --progress-style bars -sS -sV -p 1-1000 192.168.1.0/24
```

```
Stage 1: Target Resolution   ▓▓▓▓▓▓▓▓▓▓ 100%
Stage 2: Host Discovery      ▓▓▓▓▓▓▓▓▓▓ 100%
Stage 3: Port Scanning        ▓▓▓▓▓▓▓▓░░  87%
Stage 4: Service Detection    ▓░░░░░░░░░  10%
Stage 5: Finalization         ░░░░░░░░░░   0%
Overall ▓▓▓▓▓░░░░░ 52% | ETA: 3m 24s | 1,240 pps | 42 hpm
```

### ETA Algorithms

**Linear ETA**: Simple current-rate projection

```
ETA = (total - completed) / current_rate
```

**EWMA ETA**: Exponentially Weighted Moving Average (α=0.2)

```
rate_ewma = α × current_rate + (1 - α) × previous_rate_ewma
ETA = (total - completed) / rate_ewma
```

Smooths out fluctuations for more stable estimates.

**Multi-Stage ETA**: Weighted prediction across 5 scan stages

Each stage contributes to overall completion estimate based on typical time distribution.

## Event Logging

### JSON Lines Format

Events are logged to JSON Lines format (one JSON object per line) for easy parsing and analysis.

**Example Log File** (`~/.prtip/events/scan-2024-11-15.jsonl`):

```json
{"event":"log_started","timestamp":"2024-11-15T10:30:00Z","version":"1.0"}
{"event":"ScanStarted","scan_id":"a1b2c3...","scan_type":"Syn","target_count":1000,"port_count":100,"timestamp":"2024-11-15T10:30:01Z"}
{"event":"HostDiscovered","scan_id":"a1b2c3...","ip":"192.168.1.1","method":"ICMP","latency_ms":10,"timestamp":"2024-11-15T10:30:02Z"}
{"event":"PortFound","scan_id":"a1b2c3...","target":"192.168.1.1","port":80,"protocol":"Tcp","state":"Open","timestamp":"2024-11-15T10:30:03Z"}
{"event":"ServiceDetected","scan_id":"a1b2c3...","target":"192.168.1.1","port":80,"service":"HTTP","version":"Apache/2.4.52","confidence":95,"timestamp":"2024-11-15T10:30:04Z"}
{"event":"ProgressUpdate","scan_id":"a1b2c3...","percentage":50.0,"completed":500,"total":1000,"timestamp":"2024-11-15T10:32:00Z"}
{"event":"ScanCompleted","scan_id":"a1b2c3...","targets_scanned":1000,"ports_scanned":100,"duration":120,"timestamp":"2024-11-15T10:34:00Z"}
{"event":"log_ended","timestamp":"2024-11-15T10:34:01Z"}
```

### Enabling Event Logging

**CLI Flag**:

```bash
prtip -sS -p 80,443 --event-log scans.jsonl 192.168.1.0/24
```

**Configuration File**:

```toml
[logging]
event_log = "~/.prtip/events/scan-%Y-%m-%d.jsonl"
event_log_rotation = "daily"
event_log_compression = true
```

### Log Analysis

**Query with jq**:

```bash
# Count port discoveries
jq -r 'select(.event == "PortFound") | .port' scans.jsonl | sort -n | uniq -c

# Find all HTTP services
jq -r 'select(.event == "ServiceDetected" and .service == "HTTP")' scans.jsonl

# Calculate average scan duration
jq -r 'select(.event == "ScanCompleted") | .duration' scans.jsonl | \
  awk '{sum+=$1; count++} END {print sum/count}'

# Extract all warnings
jq -r 'select(.event == "WarningIssued") | .message' scans.jsonl
```

**Query with grep**:

```bash
# Find all events for specific scan ID
grep 'a1b2c3d4-e5f6-7890' scans.jsonl

# Find failed connection attempts
grep '"state":"Filtered"' scans.jsonl

# Extract all discovered hosts
grep 'HostDiscovered' scans.jsonl | jq -r '.ip'
```

### Log Rotation

Automatic rotation prevents log files from growing indefinitely:

**Size-Based**:
```toml
[logging]
event_log_rotation = "size"
event_log_max_size = 104857600  # 100 MB
event_log_max_files = 10
```

**Time-Based**:
```toml
[logging]
event_log_rotation = "daily"  # or "hourly", "weekly"
event_log_pattern = "scan-%Y-%m-%d.jsonl"
```

**Compression**:
```toml
[logging]
event_log_compression = true  # gzip older logs
```

## Performance

### Event System Overhead

Comprehensive benchmarking shows industry-leading performance:

| Metric | Value | Impact |
|--------|-------|--------|
| Publish latency | 40ns | Negligible |
| End-to-end latency | 340ns | Sub-microsecond |
| Max throughput | >10M events/sec | Scales to largest scans |
| Concurrent overhead | -4.1% | Faster with events enabled! |

**Why Negative Overhead?**
- Better CPU optimization with predictable event patterns
- Improved cache efficiency from event batching
- Reduced memory contention via async channels

### Memory Usage

Ring buffer maintains last 1,000 events:
- **Memory**: ~100 KB (1,000 events × 100 bytes/event)
- **Retention**: Last 1,000 events only (auto-cleanup)
- **Growth**: Bounded (no unbounded growth)

Event logging:
- **Buffered writes**: 8 KB buffer (reduces I/O)
- **Async I/O**: Non-blocking writes
- **Compression**: ~70% size reduction with gzip

## Integration

### TUI Dashboard

The event system powers the live TUI dashboard:

```bash
prtip --live -sS -p 1-10000 192.168.1.0/24
```

**Real-Time Updates**:
- Port discoveries as they happen
- Service detection results streaming
- Live throughput graphs
- Error warnings immediately visible

**Event-Driven Architecture**:
- 60 FPS rendering
- <5ms frame time
- 10K+ events/sec throughput
- Zero dropped events

### API Integration

Custom integrations can subscribe to events:

```rust
use prtip_core::event_bus::EventBus;
use std::sync::Arc;

// Create event bus
let event_bus = Arc::new(EventBus::new(1000));

// Subscribe to port discoveries
let mut rx = event_bus.subscribe(
    |event| matches!(event, ScanEvent::PortFound { .. })
);

// Process events
while let Some(event) = rx.recv().await {
    if let ScanEvent::PortFound { target, port, .. } = event {
        println!("Found: {}:{}", target, port);
    }
}
```

## Best Practices

### Enable Progress Display

Always use progress display for interactive scans:

```bash
# Default: compact progress
prtip -sS -p 1-10000 192.168.1.0/24

# Detailed metrics
prtip --progress-style detailed -sS -p- 192.168.1.0/24

# Multi-stage visualization
prtip --progress-style bars -sS -sV -p 1-1000 192.168.1.0/24
```

### Use Event Logging for Audits

Enable event logging for compliance and forensics:

```bash
# Single scan log
prtip -sS -p 80,443 --event-log audit-scan.jsonl targets.txt

# Daily rotation with compression
prtip -sS -p- --event-log ~/.prtip/events/scan-%Y-%m-%d.jsonl \
  --event-log-rotation daily \
  --event-log-compression \
  10.0.0.0/8
```

### Disable for Automation

Disable progress display in CI/automation:

```bash
# No progress output
prtip --no-progress -sS -p 80,443 192.168.1.0/24

# Minimal output (errors only)
prtip -q -sS -p 80,443 192.168.1.0/24
```

## See Also

- [TUI Architecture](../advanced/tui-architecture.md) - Live dashboard implementation
- [Performance Tuning](../advanced/performance-tuning.md) - Optimization guide
- [CLI Reference](../user-guide/cli-reference.md#verbosity-progress) - Progress flags
- [Benchmarking](../31-BENCHMARKING-GUIDE.md) - Performance validation
