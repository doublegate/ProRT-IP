# ProRT-IP Architecture Diagrams

This document captures high-level relationships and runtime flows inside the ProRT-IP WarScan workspace. All diagrams use Mermaid syntax so they can be previewed in compatible renderers.

## Workspace Module Relationships

```mermaid
graph LR
    subgraph CLI Layer
        CLI[prtip-cli]
    end
    subgraph Scanner Engine
        Scheduler[ScanScheduler]
        Scanners[Scan Implementations]
        Storage[ScanStorage]
    end
    subgraph Networking & System
        Network[prtip-network]
        Core[prtip-core]
    end

    CLI -->|parses args| Core
    CLI -->|builds config| Scheduler
    Scheduler -->|reads/writes| Storage
    Scheduler -->|invokes| Scanners
    Scanners -->|craft packets| Network
    Network -->|uses types/errors| Core
    Storage -->|serialize results| Core
```

## CLI Execution Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI as prtip-cli
    participant Core as prtip-core
    participant Scanner as prtip-scanner
    participant Network as prtip-network
    participant DB as SQLite (ScanStorage)

    User->>CLI: Run `prtip ...`
    CLI->>CLI: Parse & validate arguments
    CLI->>Core: Build Config & PortRange
    CLI->>Core: Adjust resource limits
    CLI->>Scanner: Create ScanScheduler + Storage
    CLI->>Scanner: Execute scan (targets, ports)
    Scanner->>Network: Check/drop privileges
    Scanner->>Network: Send/receive packets
    Scanner->>DB: Persist scan records/results
    DB-->>Scanner: Ack writes
    Scanner-->>CLI: Return ScanResult list
    CLI->>Core: Format output (text/json/xml)
    CLI-->>User: Render formatted results
```

## Scan Scheduler Orchestration

```mermaid
graph TD
    Scheduler[ScanScheduler] --> Config[Config Validation]
    Scheduler --> Discovery[DiscoveryEngine]
    Scheduler --> RateLimiter[RateLimiter]
    Scheduler --> Parallelism[Adaptive Parallelism]
    Scheduler --> TCP[TcpConnectScanner]
    Scheduler --> SYN[SynScanner]
    Scheduler --> UDP[UdpScanner]
    Scheduler --> Stealth[StealthScanner]
    Scheduler --> Service[ServiceDetector]
    Scheduler --> OS[OsFingerprinter]
    Scheduler --> Storage
    Storage --> DB[(SQLite)]
    RateLimiter --> Core
    Parallelism --> Core
```

## Result Aggregation Pipeline

```mermaid
flowchart LR
    subgraph Producers
        Workers[Scan Workers]
    end
    Workers -->|ScanResult| Aggregator["LockFreeAggregator (SegQueue)"]
    Aggregator -->|Batch drain| Writer[Storage Writer Task]
    Writer -->|Transaction| SQLite[(SQLite WAL)]
    SQLite --> Reports[Formatted Output]
    Reports --> CLI
```

## Packet Lifecycle (SYN Scan Path)

```mermaid
sequenceDiagram
    participant Syn as SynScanner
    participant Builder as TcpPacketBuilder
    participant Capture as PacketCapture
    participant Target as Target Host

    Syn->>Builder: Build SYN frame\n(local IP/port + target)
    Builder-->>Syn: Raw packet bytes
    Syn->>Capture: send_packet(bytes)
    Capture-->>Target: Transmit over wire
    Target-->>Capture: SYN/ACK or RST
    Capture-->>Syn: receive_packet()
    Syn->>Syn: Update connection state
    Syn->>Builder: Optional RST teardown
    Syn-->>Scheduler: PortState + timing
```
