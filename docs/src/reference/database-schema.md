# Database Schema Reference

This document provides complete database schema documentation for ProRT-IP's SQLite-based scan result storage, including table structures, relationships, indexes, performance optimizations, and query examples.

## Overview

ProRT-IP uses SQLite for persistent storage of scan results with the following features:

- **Transaction-based batch inserts** - Multi-row VALUES for 100-1000x faster writes
- **Indexed queries** - Fast retrieval by scan ID, target IP, or port
- **WAL mode** - Write-Ahead Logging for concurrent access
- **Automatic schema initialization** - Tables created on first use
- **Performance-optimized pragmas** - Tuned for high-throughput scanning

## Database Configuration

### Connection Options

```rust
// In-memory database (testing)
let storage = ScanStorage::new(":memory:").await?;

// File-based database
let storage = ScanStorage::new("results.db").await?;

// Absolute path
let storage = ScanStorage::new("/var/lib/prtip/scans.db").await?;
```

### SQLite Pragmas

ProRT-IP automatically applies these performance optimizations:

| Pragma | Value | Purpose |
|--------|-------|---------|
| `journal_mode` | `WAL` | Concurrent reads/writes |
| `synchronous` | `NORMAL` | Safe for WAL, better performance |
| `cache_size` | `-64000` | 64MB cache (vs 2MB default) |
| `busy_timeout` | `10000` | 10-second timeout |

## Schema Definition

### Entity Relationship Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                          scans                              │
├─────────────────────────────────────────────────────────────┤
│ id          INTEGER PRIMARY KEY AUTOINCREMENT               │
│ start_time  TIMESTAMP NOT NULL                              │
│ end_time    TIMESTAMP                                       │
│ config_json TEXT NOT NULL                                   │
│ created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP             │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ 1:N
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      scan_results                           │
├─────────────────────────────────────────────────────────────┤
│ id               INTEGER PRIMARY KEY AUTOINCREMENT          │
│ scan_id          INTEGER NOT NULL (FK → scans.id)           │
│ target_ip        TEXT NOT NULL                              │
│ port             INTEGER NOT NULL                           │
│ state            TEXT NOT NULL                              │
│ service          TEXT                                       │
│ banner           TEXT                                       │
│ response_time_ms INTEGER NOT NULL                           │
│ timestamp        TIMESTAMP NOT NULL                         │
└─────────────────────────────────────────────────────────────┘
```

### scans Table

Stores metadata about scan executions.

```sql
CREATE TABLE IF NOT EXISTS scans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    config_json TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `id` | INTEGER | No | Auto-incrementing primary key |
| `start_time` | TIMESTAMP | No | Scan start timestamp (UTC) |
| `end_time` | TIMESTAMP | Yes | Scan completion timestamp (UTC) |
| `config_json` | TEXT | No | JSON-encoded scan configuration |
| `created_at` | TIMESTAMP | No | Record creation timestamp |

**config_json Schema:**

```json
{
  "targets": "192.168.1.0/24",
  "ports": "1-1000",
  "scan_type": "Syn",
  "timing": "Aggressive",
  "service_detection": true
}
```

### scan_results Table

Stores individual port scan results.

```sql
CREATE TABLE IF NOT EXISTS scan_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id INTEGER NOT NULL,
    target_ip TEXT NOT NULL,
    port INTEGER NOT NULL,
    state TEXT NOT NULL,
    service TEXT,
    banner TEXT,
    response_time_ms INTEGER NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE
);
```

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `id` | INTEGER | No | Auto-incrementing primary key |
| `scan_id` | INTEGER | No | Foreign key to scans.id |
| `target_ip` | TEXT | No | Target IP address (IPv4 or IPv6) |
| `port` | INTEGER | No | Port number (1-65535) |
| `state` | TEXT | No | Port state: open, closed, filtered, unknown |
| `service` | TEXT | Yes | Detected service name |
| `banner` | TEXT | Yes | Service banner/version info |
| `response_time_ms` | INTEGER | No | Response time in milliseconds |
| `timestamp` | TIMESTAMP | No | Result timestamp (UTC) |

**State Values:**

| Value | Description |
|-------|-------------|
| `open` | Port accepting connections |
| `closed` | Port responding with RST |
| `filtered` | No response or ICMP unreachable |
| `unknown` | State could not be determined |

### Indexes

```sql
-- Fast lookups by scan ID (most common query)
CREATE INDEX IF NOT EXISTS idx_scan_id ON scan_results(scan_id);

-- Fast lookups by target IP
CREATE INDEX IF NOT EXISTS idx_target_ip ON scan_results(target_ip);

-- Fast lookups by port number
CREATE INDEX IF NOT EXISTS idx_port ON scan_results(port);
```

| Index | Column(s) | Use Case |
|-------|-----------|----------|
| `idx_scan_id` | `scan_id` | Retrieving all results for a scan |
| `idx_target_ip` | `target_ip` | Finding all ports for a host |
| `idx_port` | `port` | Finding all hosts with a port open |

## Data Types

### IP Address Storage

IP addresses are stored as TEXT for maximum compatibility:

| Format | Example |
|--------|---------|
| IPv4 | `"192.168.1.1"` |
| IPv6 | `"2001:db8::1"` |
| IPv6 (compressed) | `"::1"` |

### Timestamp Format

All timestamps use ISO 8601 format with UTC timezone:

```
2025-11-21T14:30:00.000000Z
```

### Port State Mapping

| Rust Enum | Database Value |
|-----------|----------------|
| `PortState::Open` | `"open"` |
| `PortState::Closed` | `"closed"` |
| `PortState::Filtered` | `"filtered"` |
| `PortState::Unknown` | `"unknown"` |

## Query Examples

### Basic Queries

**Get all results for a scan:**

```sql
SELECT target_ip, port, state, service, banner, response_time_ms, timestamp
FROM scan_results
WHERE scan_id = ?
ORDER BY target_ip, port;
```

**Count results by state:**

```sql
SELECT state, COUNT(*) as count
FROM scan_results
WHERE scan_id = ?
GROUP BY state
ORDER BY count DESC;
```

**Find all open ports:**

```sql
SELECT target_ip, port, service, banner
FROM scan_results
WHERE scan_id = ? AND state = 'open'
ORDER BY target_ip, port;
```

### Analysis Queries

**Top 10 most common open ports:**

```sql
SELECT port, COUNT(*) as count, service
FROM scan_results
WHERE scan_id = ? AND state = 'open'
GROUP BY port
ORDER BY count DESC
LIMIT 10;
```

**Hosts with specific service:**

```sql
SELECT DISTINCT target_ip
FROM scan_results
WHERE scan_id = ? AND service LIKE '%http%'
ORDER BY target_ip;
```

**Average response time by port:**

```sql
SELECT port, AVG(response_time_ms) as avg_ms
FROM scan_results
WHERE scan_id = ? AND state = 'open'
GROUP BY port
ORDER BY avg_ms;
```

**Scan duration:**

```sql
SELECT
    id,
    start_time,
    end_time,
    ROUND((JULIANDAY(end_time) - JULIANDAY(start_time)) * 86400, 2) as duration_seconds
FROM scans
WHERE id = ?;
```

### Cross-Scan Queries

**Compare results between two scans:**

```sql
SELECT
    r1.target_ip,
    r1.port,
    r1.state as state_scan1,
    r2.state as state_scan2
FROM scan_results r1
LEFT JOIN scan_results r2
    ON r1.target_ip = r2.target_ip
    AND r1.port = r2.port
    AND r2.scan_id = ?
WHERE r1.scan_id = ?
    AND (r1.state != r2.state OR r2.state IS NULL);
```

**Find newly opened ports:**

```sql
SELECT r2.target_ip, r2.port, r2.service
FROM scan_results r2
LEFT JOIN scan_results r1
    ON r1.target_ip = r2.target_ip
    AND r1.port = r2.port
    AND r1.scan_id = ?
WHERE r2.scan_id = ?
    AND r2.state = 'open'
    AND (r1.state IS NULL OR r1.state != 'open');
```

### Reporting Queries

**Summary report:**

```sql
SELECT
    COUNT(DISTINCT target_ip) as hosts_scanned,
    COUNT(*) as total_results,
    SUM(CASE WHEN state = 'open' THEN 1 ELSE 0 END) as open_ports,
    SUM(CASE WHEN state = 'closed' THEN 1 ELSE 0 END) as closed_ports,
    SUM(CASE WHEN state = 'filtered' THEN 1 ELSE 0 END) as filtered_ports,
    AVG(response_time_ms) as avg_response_ms
FROM scan_results
WHERE scan_id = ?;
```

**Service distribution:**

```sql
SELECT
    COALESCE(service, 'unknown') as service,
    COUNT(*) as count,
    GROUP_CONCAT(DISTINCT port) as ports
FROM scan_results
WHERE scan_id = ? AND state = 'open'
GROUP BY service
ORDER BY count DESC;
```

## Performance Optimization

### Batch Insert Performance

ProRT-IP uses multi-row INSERT for optimal write performance:

| Batch Size | INSERT Method | Performance |
|------------|---------------|-------------|
| 1 | Individual | ~100 inserts/sec |
| 100 | Multi-row VALUES | ~10,000 inserts/sec |
| 1000 | Multi-row + Transaction | ~50,000 inserts/sec |

**SQLite Parameter Limit:**

SQLite has a 999 parameter limit. With 8 columns per row:
- Maximum rows per statement: 124 (999 ÷ 8)
- ProRT-IP uses 100 rows per statement for safety

### Index Usage

Ensure queries use indexes efficiently:

```sql
-- Uses idx_scan_id
SELECT * FROM scan_results WHERE scan_id = 123;

-- Uses idx_target_ip
SELECT * FROM scan_results WHERE target_ip = '192.168.1.1';

-- Uses idx_port
SELECT * FROM scan_results WHERE port = 80;

-- Full table scan (avoid for large datasets)
SELECT * FROM scan_results WHERE banner LIKE '%Apache%';
```

### Connection Pooling

ProRT-IP uses a connection pool with 5 connections:

```rust
SqlitePoolOptions::new()
    .max_connections(5)
    .connect_with(options)
```

## API Usage

### Creating Storage

```rust
use prtip_scanner::ScanStorage;

// Create or open database
let storage = ScanStorage::new("results.db").await?;
```

### Creating a Scan

```rust
// Create scan with configuration JSON
let config_json = serde_json::json!({
    "targets": "192.168.1.0/24",
    "ports": "1-1000",
    "scan_type": "Syn"
}).to_string();

let scan_id = storage.create_scan(&config_json).await?;
```

### Storing Results

```rust
use prtip_core::{ScanResult, PortState};

// Single result
let result = ScanResult::new(
    "192.168.1.1".parse()?,
    80,
    PortState::Open,
).with_service("http".to_string());

storage.store_result(scan_id, &result).await?;

// Batch results (100-1000x faster)
let results: Vec<ScanResult> = /* ... */;
storage.store_results_batch(scan_id, &results).await?;
```

### Completing a Scan

```rust
// Mark scan as complete (sets end_time)
storage.complete_scan(scan_id).await?;
```

### Retrieving Results

```rust
// Get all results for a scan
let results = storage.get_scan_results(scan_id).await?;

// Get counts
let scan_count = storage.get_scan_count().await?;
let result_count = storage.get_result_count(scan_id).await?;
```

### Closing Connection

```rust
// Graceful shutdown
storage.close().await;
```

## CLI Integration

### Enabling Database Storage

```bash
# Store results in SQLite database
prtip --with-db results.db 192.168.1.0/24

# Combine with other output formats
prtip --with-db results.db -oJ results.json 192.168.1.0/24
```

### Querying Results

```bash
# Using sqlite3 CLI
sqlite3 results.db "SELECT * FROM scan_results WHERE state='open'"

# Export to CSV
sqlite3 -csv results.db "SELECT target_ip,port,service FROM scan_results WHERE state='open'" > open_ports.csv
```

## Migration and Maintenance

### Schema Versioning

Current schema version: **1.0**

ProRT-IP uses `CREATE TABLE IF NOT EXISTS` for forward compatibility. Future migrations will be handled via schema version tracking.

### Database Maintenance

**Analyze for query optimization:**

```sql
ANALYZE;
```

**Vacuum to reclaim space:**

```sql
VACUUM;
```

**Check integrity:**

```sql
PRAGMA integrity_check;
```

### Backup

```bash
# Simple file copy (ensure WAL is checkpointed)
sqlite3 results.db "PRAGMA wal_checkpoint(TRUNCATE);"
cp results.db results.db.backup

# Or use .backup command
sqlite3 results.db ".backup 'results.db.backup'"
```

## PostgreSQL Support (Planned)

PostgreSQL support is planned for future releases. The schema will be compatible with these differences:

| Feature | SQLite | PostgreSQL |
|---------|--------|------------|
| Auto-increment | `AUTOINCREMENT` | `SERIAL` |
| Timestamp | `TIMESTAMP` | `TIMESTAMPTZ` |
| JSON | `TEXT` | `JSONB` |
| Connection | File-based | Network |

## See Also

- [Database Storage Guide](../features/database-storage.md) - Feature documentation
- [Output Formats](../user-guide/output-formats.md) - Alternative output options
- [CLI Reference](./command-reference.md) - Command-line options

---

**Last Updated:** 2025-11-21
**ProRT-IP Version:** v0.5.4
