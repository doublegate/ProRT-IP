# Database Usage

Persistent storage and advanced querying of ProRT-IP scan results.

## What is Database Usage?

**Database Usage** enables persistent storage of scan results for historical analysis, trend detection, compliance reporting, and automated monitoring. ProRT-IP supports multiple database backends optimized for different use cases.

**Database Storage Capabilities:**
- **Persistent Results** - Store scan results for long-term analysis
- **Historical Comparison** - Track network changes over time
- **Compliance Reporting** - Generate audit trails for regulatory requirements
- **Trend Analysis** - Identify patterns across multiple scans
- **Automated Monitoring** - Continuous security posture assessment
- **Query Optimization** - Indexed searches across millions of results
- **Export Flexibility** - Generate reports in JSON, CSV, XML, Text formats

**Supported Databases:**
- **SQLite** - Embedded database, zero-configuration, file-based storage
- **PostgreSQL** - Production deployments, advanced analytics, high concurrency
- **MySQL** - Legacy compatibility, widespread tooling support
- **ClickHouse** - Analytical queries, time-series data, OLAP workloads

**Use Cases:**
- Security Operations Centers (SOC) - Continuous network monitoring
- Compliance Audits - PCI DSS, HIPAA, SOC 2 evidence collection
- Vulnerability Management - Tracking remediation progress
- Network Inventory - Asset discovery and change detection
- Penetration Testing - Engagement documentation and reporting

---

## SQLite Integration

**SQLite** is ProRT-IP's default database backend, providing zero-configuration persistent storage with excellent performance for most use cases.

### Schema Design

ProRT-IP uses a normalized schema optimized for query performance:

**`scans` Table:**
```sql
CREATE TABLE scans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    end_time TIMESTAMP,
    config_json TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**Fields:**
- `id` - Unique scan identifier (auto-increment)
- `start_time` - Scan start timestamp (UTC)
- `end_time` - Scan completion timestamp (NULL if in progress)
- `config_json` - Scan configuration (JSON format: targets, ports, scan type, timing)
- `created_at` - Database insertion timestamp

**`scan_results` Table:**
```sql
CREATE TABLE scan_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id INTEGER NOT NULL,
    target_ip TEXT NOT NULL,
    port INTEGER NOT NULL CHECK(port >= 1 AND port <= 65535),
    state TEXT NOT NULL CHECK(state IN ('open', 'closed', 'filtered', 'unknown')),
    service TEXT,
    version TEXT,
    banner TEXT,
    response_time_ms INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (scan_id) REFERENCES scans(id) ON DELETE CASCADE,
    UNIQUE(scan_id, target_ip, port)
);
```

**Fields:**
- `id` - Unique result identifier
- `scan_id` - Foreign key to scans table (cascade delete)
- `target_ip` - Target IP address (TEXT for IPv4/IPv6 compatibility)
- `port` - Port number (1-65535, CHECK constraint)
- `state` - Port state (open/closed/filtered/unknown, CHECK constraint)
- `service` - Detected service name (NULL if unknown)
- `version` - Service version string (NULL if undetected)
- `banner` - Service banner/greeting (NULL if unavailable)
- `response_time_ms` - Response time in milliseconds
- `timestamp` - Timestamp of this specific check (UTC)

**Indexes:**
```sql
-- Performance indexes
CREATE INDEX idx_scan_results_scan_id ON scan_results(scan_id);
CREATE INDEX idx_scan_results_target_ip ON scan_results(target_ip);
CREATE INDEX idx_scan_results_port ON scan_results(port);
CREATE INDEX idx_scan_results_state ON scan_results(state);
CREATE INDEX idx_scan_results_service ON scan_results(service);
CREATE INDEX idx_scan_results_timestamp ON scan_results(timestamp);

-- Composite index for common queries
CREATE INDEX idx_scan_results_target_port ON scan_results(target_ip, port);
CREATE INDEX idx_scan_results_state_port ON scan_results(state, port);
```

### Basic Storage

**Store Scan Results:**
```bash
# Default database location (./scans.db)
prtip -p 80,443 192.168.1.1 --with-db

# Custom database location
prtip -p 80,443 192.168.1.1 --with-db --database /path/to/results.db

# Scan with service detection and database storage
prtip -sV -p 1-1000 192.168.1.0/24 --with-db --database security-audit.db
```

**Organize by Purpose:**
```bash
# Separate databases for different purposes
prtip -p- network.com --with-db --database full-scan.db
prtip -sV -p 22,80,443 network.com --with-db --database service-audit.db

# Daily security scans with date-stamped databases
prtip -sV -p 22,23,3389 192.168.1.0/24 \
  --with-db --database daily-$(date +%Y%m%d).db

# Continuous monitoring (append to same database for trending)
prtip -sV -p 22,23,3389 192.168.1.0/24 \
  --with-db --database security-monitor.db
```

### Querying Database

**List All Scans:**
```bash
prtip db list results.db
```

**Example Output:**
```
Scans in Database
================================================================================
ID       Start Time           End Time             Results
================================================================================
3        2025-10-24 10:30:15  2025-10-24 10:32:45  156
2        2025-10-23 14:22:10  2025-10-23 14:25:33  243
1        2025-10-22 09:15:00  2025-10-22 09:18:12  189
================================================================================
Total: 3 scan(s)
```

**Query by Scan ID:**
```bash
# Get all results for specific scan
prtip db query results.db --scan-id 1
```

**Query by Target:**
```bash
# Find all open ports on specific host
prtip db query results.db --target 192.168.1.100
```

**Example Output:**
```
Open Ports for 192.168.1.100
================================================================================
Port     Protocol     Service              Version              RTT (ms)
================================================================================
22       TCP          ssh                  OpenSSH 8.9          2
80       TCP          http                 Apache 2.4.52        5
443      TCP          https                Apache 2.4.52        6
================================================================================
```

**Query by Port:**
```bash
# Find all hosts with specific port open
prtip db query results.db --port 22
```

**Query by Service:**
```bash
# Find all hosts running specific service
prtip db query results.db --service apache

# Filter open ports only
prtip db query results.db --scan-id 1 --open
```

### Exporting Results

**Export to JSON:**
```bash
prtip db export results.db --scan-id 1 --format json -o scan1.json
```

**Example JSON:**
```json
[
  {
    "target_ip": "192.168.1.100",
    "port": 22,
    "state": "Open",
    "response_time": { "secs": 0, "nanos": 2000000 },
    "timestamp": "2025-10-24T10:30:15Z",
    "banner": "SSH-2.0-OpenSSH_8.9",
    "service": "ssh",
    "version": "OpenSSH 8.9"
  }
]
```

**Export to CSV:**
```bash
prtip db export results.db --scan-id 1 --format csv -o scan1.csv
```

**Example CSV:**
```
Target IP,Port,State,Service,Version,Banner,Response Time (ms),Timestamp
192.168.1.100,22,Open,ssh,OpenSSH 8.9,SSH-2.0-OpenSSH_8.9,2,2025-10-24T10:30:15Z
192.168.1.100,80,Open,http,Apache 2.4.52,,5,2025-10-24T10:30:16Z
```

**Export to XML (Nmap-compatible):**
```bash
prtip db export results.db --scan-id 1 --format xml -o scan1.xml
```

**Export to Text:**
```bash
prtip db export results.db --scan-id 1 --format text -o scan1.txt
```

### Comparing Scans

**Basic Comparison:**
```bash
prtip db compare results.db 1 2
```

**Example Output:**
```
Comparing Scan 1 vs Scan 2
================================================================================

New Open Ports:
--------------------------------------------------------------------------------
  192.168.1.150 → Port 3306 mysql (MySQL 5.7)
  192.168.1.200 → Port 8080 http (Apache Tomcat)

Closed Ports:
--------------------------------------------------------------------------------
  192.168.1.100 → Port 23 telnet ()

Changed Services:
--------------------------------------------------------------------------------
  192.168.1.100 → Port 80 Apache 2.4.41 → Apache 2.4.52

Summary:
--------------------------------------------------------------------------------
  New ports:        2
  Closed ports:     1
  Changed services: 1
  New hosts:        1
  Disappeared hosts: 0
================================================================================
```

**Use Cases:**

**1. Detect Unauthorized Services:**
```bash
# Compare weekly scans
prtip db compare weekly-scans.db 1 2

# Alert on new ports
prtip db compare weekly-scans.db 1 2 | grep "New Open Ports" -A 10
```

**2. Track Patch Management:**
```bash
# Compare before/after patching
prtip db compare patch-validation.db 1 2

# Verify service versions updated
prtip db compare patch-validation.db 1 2 | grep "Changed Services"
```

**3. Compliance Monitoring:**
```bash
# Daily PCI scans with 30-day comparison
for i in {1..30}; do
  prtip db compare compliance.db $i $((i+1))
done
```

### Performance Optimization

**WAL Mode (Write-Ahead Logging):**

ProRT-IP automatically enables SQLite WAL mode for better concurrency:
```sql
PRAGMA journal_mode=WAL;
```

**Benefits:**
- Concurrent readers while writing
- Better crash recovery
- Faster write performance (no fsync per transaction)

**Batch Inserts:**

ProRT-IP uses batch inserts (1,000-10,000 results per transaction):
```sql
BEGIN TRANSACTION;
INSERT INTO scan_results (scan_id, target_ip, port, state, service)
VALUES (1, '192.168.1.1', 22, 'open', 'ssh'),
       (1, '192.168.1.1', 80, 'open', 'http'),
       -- ... (1,000-10,000 rows per batch)
COMMIT;
```

**Performance:** ~50,000 inserts/second on modern SSD

**Query Performance:**

```bash
# Fast: Uses index on target_ip
prtip db query results.db --target 192.168.1.100

# Fast: Uses index on port
prtip db query results.db --port 22

# Fast: Uses index on scan_id
prtip db query results.db --scan-id 1

# Slower: Service name index requires full table scan without specific state filter
prtip db query results.db --service apache
```

**Database Maintenance:**
```bash
# Vacuum database to reclaim space (after deleting old scans)
sqlite3 results.db "VACUUM;"

# Analyze database for query optimization
sqlite3 results.db "ANALYZE;"

# Check database integrity
sqlite3 results.db "PRAGMA integrity_check;"

# Enable auto-vacuum (reclaim space automatically)
sqlite3 results.db "PRAGMA auto_vacuum = FULL;"
```

---

## PostgreSQL Integration

**PostgreSQL** is recommended for production deployments requiring:
- High concurrency (multiple concurrent scans)
- Advanced analytics (complex queries, aggregations)
- Replication and high availability
- JSONB query capabilities
- Massive scale (billions of results)

### Schema Design

**Production Schema:**
```sql
-- Create schema for ProRT-IP data
CREATE SCHEMA prtip;

-- Scans table
CREATE TABLE prtip.scans (
    scan_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    start_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    end_time TIMESTAMPTZ,
    config_jsonb JSONB NOT NULL,
    scan_type VARCHAR(20),
    status VARCHAR(20) CHECK(status IN ('running', 'complete', 'failed')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Scan results table
CREATE TABLE prtip.scan_results (
    result_id BIGSERIAL PRIMARY KEY,
    scan_id UUID NOT NULL REFERENCES prtip.scans(scan_id) ON DELETE CASCADE,
    target_ip INET NOT NULL,
    port INTEGER NOT NULL CHECK(port >= 1 AND port <= 65535),
    protocol VARCHAR(10) NOT NULL CHECK(protocol IN ('tcp', 'udp')),
    state VARCHAR(20) NOT NULL CHECK(state IN ('open', 'closed', 'filtered', 'unknown')),
    service VARCHAR(100),
    version VARCHAR(200),
    banner TEXT,
    response_time_ms INTEGER,
    discovered_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(scan_id, target_ip, port, protocol)
);

-- Performance indexes
CREATE INDEX idx_scan_results_scan_id ON prtip.scan_results(scan_id);
CREATE INDEX idx_scan_results_target_ip ON prtip.scan_results USING BTREE(target_ip);
CREATE INDEX idx_scan_results_port ON prtip.scan_results(port);
CREATE INDEX idx_scan_results_state ON prtip.scan_results(state);
CREATE INDEX idx_scan_results_service ON prtip.scan_results USING BTREE(service);
CREATE INDEX idx_scan_results_discovered_at ON prtip.scan_results(discovered_at);

-- Composite indexes for common queries
CREATE INDEX idx_scan_results_state_port ON prtip.scan_results(state, port);
CREATE INDEX idx_scan_results_target_port ON prtip.scan_results(target_ip, port);

-- GIN index for JSONB config queries
CREATE INDEX idx_scans_config ON prtip.scans USING GIN(config_jsonb);

-- Partial index for open ports (70-90% of queries filter for open state)
CREATE INDEX idx_scan_results_open_ports ON prtip.scan_results(target_ip, port)
WHERE state = 'open';
```

**Materialized View - Current Network State:**
```sql
CREATE MATERIALIZED VIEW prtip.current_state AS
SELECT DISTINCT ON (target_ip, port, protocol)
    target_ip,
    port,
    protocol,
    state,
    service,
    version,
    discovered_at
FROM prtip.scan_results
ORDER BY target_ip, port, protocol, discovered_at DESC;

-- Index for fast queries
CREATE INDEX idx_current_state_target ON prtip.current_state(target_ip);
CREATE INDEX idx_current_state_port ON prtip.current_state(port);

-- Refresh daily (or after each scan)
REFRESH MATERIALIZED VIEW CONCURRENTLY prtip.current_state;
```

### Data Import

**Python Import Script:**
```python
#!/usr/bin/env python3
import json
import psycopg2
from psycopg2.extras import execute_batch
from datetime import datetime
import uuid

def import_to_postgres(results_file, db_config):
    """Import ProRT-IP JSON results to PostgreSQL"""

    conn = psycopg2.connect(**db_config)
    cur = conn.cursor()

    with open(results_file) as f:
        data = json.load(f)

    # Create scan record
    scan_id = str(uuid.uuid4())
    cur.execute("""
        INSERT INTO prtip.scans (scan_id, start_time, end_time, config_jsonb, scan_type, status)
        VALUES (%s, %s, %s, %s, %s, %s)
    """, (
        scan_id,
        data.get('scan_metadata', {}).get('start_time', datetime.utcnow()),
        data.get('scan_metadata', {}).get('end_time', datetime.utcnow()),
        json.dumps(data.get('scan_metadata', {})),
        data.get('scan_metadata', {}).get('scan_type', 'syn'),
        'complete'
    ))

    # Prepare batch insert data
    results = []
    for host in data.get('hosts', []):
        for port in host.get('ports', []):
            results.append((
                scan_id,
                host['address'],
                port['port'],
                port.get('protocol', 'tcp'),
                port['state'],
                port.get('service'),
                port.get('version'),
                port.get('banner'),
                int(port.get('response_time', {}).get('nanos', 0) / 1_000_000)  # Convert to ms
            ))

    # Batch insert for performance
    execute_batch(cur, """
        INSERT INTO prtip.scan_results
        (scan_id, target_ip, port, protocol, state, service, version, banner, response_time_ms)
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s)
        ON CONFLICT (scan_id, target_ip, port, protocol) DO UPDATE
        SET state = EXCLUDED.state,
            service = EXCLUDED.service,
            version = EXCLUDED.version,
            banner = EXCLUDED.banner,
            response_time_ms = EXCLUDED.response_time_ms
    """, results, page_size=1000)

    conn.commit()
    cur.close()
    conn.close()

    print(f"Imported scan {scan_id} to PostgreSQL ({len(results)} results)")

# Usage
db_config = {
    'host': 'localhost',
    'port': 5432,
    'database': 'security',
    'user': 'prtip',
    'password': 'secure-password'
}

import_to_postgres('scan-results.json', db_config)
```

### Advanced Queries

**Find New Ports (Last 24 Hours):**
```sql
SELECT target_ip, port, service, version
FROM prtip.scan_results
WHERE discovered_at > NOW() - INTERVAL '24 hours'
  AND state = 'open'
ORDER BY discovered_at DESC;
```

**Port Change History:**
```sql
SELECT
    target_ip,
    port,
    state,
    service,
    version,
    discovered_at,
    LAG(state) OVER (PARTITION BY target_ip, port ORDER BY discovered_at) AS previous_state
FROM prtip.scan_results
WHERE target_ip = '192.168.1.100' AND port = 22
ORDER BY discovered_at DESC;
```

**Top Ports by Frequency:**
```sql
SELECT
    port,
    service,
    COUNT(*) AS occurrences,
    COUNT(DISTINCT target_ip) AS unique_hosts
FROM prtip.scan_results
WHERE state = 'open'
GROUP BY port, service
ORDER BY occurrences DESC
LIMIT 20;
```

**Vulnerable Service Versions:**
```sql
SELECT
    target_ip,
    port,
    service,
    version
FROM prtip.scan_results
WHERE state = 'open'
  AND (
      (service = 'ssh' AND version ~ 'OpenSSH [0-7]\.[0-9]')
      OR (service = 'http' AND version ~ 'Apache 2\.[0-3]')
  )
ORDER BY target_ip, port;
```

**Scan Coverage Statistics:**
```sql
SELECT
    DATE_TRUNC('day', start_time) AS scan_date,
    COUNT(*) AS num_scans,
    SUM(CASE WHEN status = 'complete' THEN 1 ELSE 0 END) AS successful_scans,
    AVG(EXTRACT(EPOCH FROM (end_time - start_time))) AS avg_duration_seconds
FROM prtip.scans
GROUP BY DATE_TRUNC('day', start_time)
ORDER BY scan_date DESC;
```

---

## MySQL Integration

**MySQL** provides compatibility with existing infrastructure and widespread tooling support.

### Schema Design

**MySQL 8.0+ Schema:**
```sql
-- Create database
CREATE DATABASE prtip CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE prtip;

-- Scans table
CREATE TABLE scans (
    scan_id CHAR(36) PRIMARY KEY,  -- UUID as CHAR(36)
    start_time DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    end_time DATETIME(6),
    config_json JSON NOT NULL,
    scan_type VARCHAR(20),
    status ENUM('running', 'complete', 'failed'),
    created_at DATETIME(6) DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),

    INDEX idx_start_time (start_time),
    INDEX idx_status (status)
) ENGINE=InnoDB;

-- Scan results table
CREATE TABLE scan_results (
    result_id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    scan_id CHAR(36) NOT NULL,
    target_ip VARCHAR(45) NOT NULL,  -- IPv6-compatible (39 chars + 6 buffer)
    port INT UNSIGNED NOT NULL CHECK (port >= 1 AND port <= 65535),
    protocol ENUM('tcp', 'udp') NOT NULL,
    state ENUM('open', 'closed', 'filtered', 'unknown') NOT NULL,
    service VARCHAR(100),
    version VARCHAR(200),
    banner TEXT,
    response_time_ms INT UNSIGNED,
    discovered_at DATETIME(6) DEFAULT CURRENT_TIMESTAMP(6),

    FOREIGN KEY (scan_id) REFERENCES scans(scan_id) ON DELETE CASCADE,
    UNIQUE KEY unique_scan_target_port (scan_id, target_ip, port, protocol),

    INDEX idx_scan_id (scan_id),
    INDEX idx_target_ip (target_ip),
    INDEX idx_port (port),
    INDEX idx_state (state),
    INDEX idx_service (service),
    INDEX idx_discovered_at (discovered_at),
    INDEX idx_state_port (state, port)
) ENGINE=InnoDB;
```

### Best Practices

**Connection Pooling:**
```python
import mysql.connector.pooling

# Create connection pool
db_pool = mysql.connector.pooling.MySQLConnectionPool(
    pool_name="prtip_pool",
    pool_size=10,
    host='localhost',
    database='prtip',
    user='prtip',
    password='secure-password'
)

def get_connection():
    return db_pool.get_connection()
```

**Prepared Statements:**
```python
conn = get_connection()
cursor = conn.cursor(prepared=True)

# Prepared statement (prevents SQL injection, improves performance)
insert_query = """
INSERT INTO scan_results
(scan_id, target_ip, port, protocol, state, service, version)
VALUES (%s, %s, %s, %s, %s, %s, %s)
ON DUPLICATE KEY UPDATE
    state = VALUES(state),
    service = VALUES(service),
    version = VALUES(version)
"""

cursor.execute(insert_query, (
    scan_id, target_ip, port, protocol, state, service, version
))
conn.commit()
```

---

## ClickHouse Integration

**ClickHouse** excels at analytical queries on time-series data with exceptional compression and query performance.

### Schema Design

**ClickHouse Table:**
```sql
CREATE DATABASE prtip;

CREATE TABLE prtip.scan_results (
    scan_id UUID,
    target_ip IPv4,  -- Or IPv6 for IPv6 addresses
    port UInt16,
    protocol LowCardinality(String),
    state LowCardinality(String),
    service LowCardinality(Nullable(String)),
    version Nullable(String),
    banner Nullable(String),
    response_time_ms UInt32,
    discovered_at DateTime
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(discovered_at)
ORDER BY (target_ip, port, discovered_at)
SETTINGS index_granularity = 8192;

-- Materialized view for current state
CREATE MATERIALIZED VIEW prtip.current_state_mv
ENGINE = ReplacingMergeTree(discovered_at)
ORDER BY (target_ip, port)
AS SELECT
    target_ip,
    port,
    protocol,
    state,
    service,
    version,
    discovered_at
FROM prtip.scan_results;
```

### Time-Series Queries

**Port Open/Close Events Over Time:**
```sql
SELECT
    toStartOfDay(discovered_at) AS day,
    target_ip,
    port,
    groupArray(state) AS state_history
FROM prtip.scan_results
WHERE target_ip = toIPv4('192.168.1.100')
  AND port = 22
  AND discovered_at > now() - INTERVAL 30 DAY
GROUP BY day, target_ip, port
ORDER BY day DESC;
```

**Service Version Trends:**
```sql
SELECT
    service,
    version,
    COUNT(*) AS occurrences,
    MIN(discovered_at) AS first_seen,
    MAX(discovered_at) AS last_seen
FROM prtip.scan_results
WHERE state = 'open'
  AND service = 'ssh'
GROUP BY service, version
ORDER BY occurrences DESC;
```

---

## Schema Design Patterns

### Normalized vs Denormalized

**Normalized (Recommended for Transactional):**

Separate tables for scans, scan_results, services, hosts:
```sql
CREATE TABLE hosts (
    host_id SERIAL PRIMARY KEY,
    ip_address INET UNIQUE,
    hostname VARCHAR(255),
    os_type VARCHAR(100),
    last_seen TIMESTAMPTZ
);

CREATE TABLE services (
    service_id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE,
    description TEXT,
    default_port INTEGER
);

CREATE TABLE scan_results (
    result_id BIGSERIAL PRIMARY KEY,
    scan_id UUID,
    host_id INTEGER REFERENCES hosts(host_id),
    service_id INTEGER REFERENCES services(service_id),
    port INTEGER,
    state VARCHAR(20),
    version VARCHAR(200),
    discovered_at TIMESTAMPTZ
);
```

**Benefits:**
- Data integrity (referential constraints)
- Storage efficiency (no duplication)
- Easier updates (single source of truth)

**Denormalized (Recommended for Analytical):**

Single wide table with all data:
```sql
CREATE TABLE scan_results_denormalized (
    scan_id UUID,
    target_ip INET,
    hostname VARCHAR(255),
    os_type VARCHAR(100),
    port INTEGER,
    protocol VARCHAR(10),
    state VARCHAR(20),
    service_name VARCHAR(100),
    service_description TEXT,
    version VARCHAR(200),
    banner TEXT,
    response_time_ms INTEGER,
    discovered_at TIMESTAMPTZ
) PARTITION BY RANGE (discovered_at);
```

**Benefits:**
- Faster queries (no joins)
- Better compression (columnar storage)
- Optimized for read-heavy workloads

### Partitioning

**Time-Based Partitioning (PostgreSQL):**
```sql
-- Parent table
CREATE TABLE prtip.scan_results_partitioned (
    result_id BIGSERIAL,
    scan_id UUID,
    target_ip INET,
    port INTEGER,
    state VARCHAR(20),
    service VARCHAR(100),
    version VARCHAR(200),
    discovered_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (result_id, discovered_at)
) PARTITION BY RANGE (discovered_at);

-- Monthly partitions
CREATE TABLE prtip.scan_results_2025_01 PARTITION OF prtip.scan_results_partitioned
FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');

CREATE TABLE prtip.scan_results_2025_02 PARTITION OF prtip.scan_results_partitioned
FOR VALUES FROM ('2025-02-01') TO ('2025-03-01');

-- Automatic partition creation (PostgreSQL 11+)
CREATE TABLE prtip.scan_results_default PARTITION OF prtip.scan_results_partitioned DEFAULT;
```

**Benefits:**
- Faster queries (partition pruning)
- Easier archival (drop old partitions)
- Better maintenance (vacuum/analyze individual partitions)
- Improved concurrency (locks per partition)

---

## Query Optimization

### Index Strategies

**B-Tree Indexes (Default):**
```sql
-- Equality and range queries
CREATE INDEX idx_target_ip ON scan_results(target_ip);
CREATE INDEX idx_port_range ON scan_results(port) WHERE port BETWEEN 1 AND 1024;
```

**GIN/GiST Indexes (PostgreSQL):**
```sql
-- JSONB queries
CREATE INDEX idx_config_jsonb ON scans USING GIN(config_jsonb);

-- Array queries
CREATE INDEX idx_services_array ON hosts USING GIN(services);

-- Full-text search
CREATE INDEX idx_banner_fulltext ON scan_results USING GIN(to_tsvector('english', banner));
```

**Partial Indexes:**
```sql
-- Index only open ports (70-90% of queries filter for open state)
CREATE INDEX idx_open_ports ON scan_results(target_ip, port)
WHERE state = 'open';

-- Index only recent scans (last 30 days)
CREATE INDEX idx_recent_scans ON scan_results(discovered_at)
WHERE discovered_at > NOW() - INTERVAL '30 days';
```

### Query Analysis

**PostgreSQL EXPLAIN:**
```sql
EXPLAIN (ANALYZE, BUFFERS, VERBOSE)
SELECT target_ip, port, service
FROM prtip.scan_results
WHERE state = 'open'
  AND port IN (22, 80, 443)
  AND discovered_at > NOW() - INTERVAL '7 days';
```

**Optimization Tips:**
1. **Use indexes** - Verify `Index Scan` vs `Seq Scan`
2. **Limit results** - Add `LIMIT` clause for pagination
3. **Avoid SELECT *** - Fetch only required columns
4. **Use prepared statements** - Reuse query plans
5. **Monitor pg_stat_statements** - Identify slow queries

---

## Data Retention Policies

### Archival Strategy

**Age-Based Archival:**
```sql
-- Archive scans older than 1 year
CREATE TABLE prtip.scan_results_archive (
    LIKE prtip.scan_results INCLUDING ALL
);

-- Move old data to archive
INSERT INTO prtip.scan_results_archive
SELECT * FROM prtip.scan_results
WHERE discovered_at < NOW() - INTERVAL '1 year';

-- Delete archived data from main table
DELETE FROM prtip.scan_results
WHERE discovered_at < NOW() - INTERVAL '1 year';

-- Vacuum to reclaim space
VACUUM FULL prtip.scan_results;
```

**Automated Archival (Cron Job):**
```bash
#!/bin/bash
# /etc/cron.monthly/prtip-archival

PGPASSWORD=secure-password psql -h localhost -U prtip -d security <<EOF
-- Archive old scans
INSERT INTO prtip.scan_results_archive
SELECT * FROM prtip.scan_results
WHERE discovered_at < NOW() - INTERVAL '1 year'
ON CONFLICT DO NOTHING;

-- Delete archived data
DELETE FROM prtip.scan_results
WHERE discovered_at < NOW() - INTERVAL '1 year';

-- Vacuum
VACUUM ANALYZE prtip.scan_results;
EOF

echo "$(date): Archival complete" >> /var/log/prtip-archival.log
```

### Purging Old Data

**Purge Script:**
```python
#!/usr/bin/env python3
import psycopg2
from datetime import datetime, timedelta

def purge_old_scans(db_config, retention_days=90):
    """Delete scans older than retention period"""

    conn = psycopg2.connect(**db_config)
    cur = conn.cursor()

    cutoff_date = datetime.now() - timedelta(days=retention_days)

    # Count scans to delete
    cur.execute("""
        SELECT COUNT(*) FROM prtip.scans
        WHERE start_time < %s
    """, (cutoff_date,))
    count = cur.fetchone()[0]

    if count == 0:
        print(f"No scans older than {retention_days} days")
        return

    print(f"Purging {count} scans older than {retention_days} days...")

    # Delete scans (cascade deletes scan_results)
    cur.execute("""
        DELETE FROM prtip.scans
        WHERE start_time < %s
    """, (cutoff_date,))

    conn.commit()
    print(f"Purged {cur.rowcount} scans")

    # Vacuum to reclaim space
    cur.execute("VACUUM ANALYZE prtip.scans, prtip.scan_results")

    cur.close()
    conn.close()

# Usage
db_config = {
    'host': 'localhost',
    'database': 'security',
    'user': 'prtip',
    'password': 'secure-password'
}

purge_old_scans(db_config, retention_days=90)
```

---

## Backup & Recovery

### PostgreSQL Backup

**Logical Backup (pg_dump):**
```bash
# Full database backup
pg_dump -h localhost -U prtip -d security -F c -f prtip-backup-$(date +%Y%m%d).dump

# Schema-only backup
pg_dump -h localhost -U prtip -d security -s -f prtip-schema.sql

# Data-only backup
pg_dump -h localhost -U prtip -d security -a -F c -f prtip-data.dump

# Compressed backup
pg_dump -h localhost -U prtip -d security | gzip > prtip-backup-$(date +%Y%m%d).sql.gz
```

**Physical Backup (pg_basebackup):**
```bash
# Continuous archiving + point-in-time recovery
pg_basebackup -h localhost -U replication -D /backup/prtip-base -Fp -Xs -P

# Restore from base backup
pg_ctl stop -D /var/lib/postgresql/14/main
rm -rf /var/lib/postgresql/14/main/*
cp -r /backup/prtip-base/* /var/lib/postgresql/14/main/
pg_ctl start -D /var/lib/postgresql/14/main
```

**Automated Daily Backups (Cron):**
```bash
#!/bin/bash
# /etc/cron.daily/prtip-backup

BACKUP_DIR="/backup/prtip"
RETENTION_DAYS=30

# Create backup
pg_dump -h localhost -U prtip -d security -F c -f ${BACKUP_DIR}/prtip-$(date +%Y%m%d).dump

# Compress
gzip ${BACKUP_DIR}/prtip-$(date +%Y%m%d).dump

# Delete old backups
find ${BACKUP_DIR} -name "prtip-*.dump.gz" -mtime +${RETENTION_DAYS} -delete

# Sync to remote storage (S3, rsync, etc.)
# aws s3 cp ${BACKUP_DIR}/prtip-$(date +%Y%m%d).dump.gz s3://backups/prtip/

echo "$(date): Backup complete" >> /var/log/prtip-backup.log
```

### SQLite Backup

**File Copy (Offline):**
```bash
# Stop all ProRT-IP processes
pkill prtip

# Copy database file
cp /path/to/results.db /backup/results-$(date +%Y%m%d).db

# Restart scans
```

**Online Backup (SQLite):**
```bash
# Backup while database is in use
sqlite3 results.db ".backup /backup/results-$(date +%Y%m%d).db"

# Verify backup integrity
sqlite3 /backup/results-$(date +%Y%m%d).db "PRAGMA integrity_check;"
```

---

## Real-World Examples

### Example 1: Continuous Monitoring

**Goal:** Track network changes with daily scans and automated alerting

**Workflow:**
```bash
#!/bin/bash
# /etc/cron.daily/prtip-monitor

DB="/var/lib/prtip/monitor.db"
TARGET="192.168.1.0/24"
ALERT_EMAIL="security@example.com"

# Run today's scan
prtip -sV -p 22,23,80,443,3389 ${TARGET} --with-db --database ${DB}

# Get last two scan IDs
SCAN1=$(sqlite3 ${DB} "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1 OFFSET 1;")
SCAN2=$(sqlite3 ${DB} "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1;")

# Compare and alert if changes
DIFF=$(prtip db compare ${DB} ${SCAN1} ${SCAN2})

if echo "${DIFF}" | grep -q "New Open Ports"; then
    echo "${DIFF}" | mail -s "ALERT: New Services Detected" ${ALERT_EMAIL}
fi

if echo "${DIFF}" | grep -q "Changed Services"; then
    echo "${DIFF}" | mail -s "INFO: Service Versions Changed" ${ALERT_EMAIL}
fi
```

### Example 2: Compliance Reporting

**Goal:** Generate monthly PCI DSS compliance reports

**Workflow:**
```bash
#!/bin/bash
# Monthly PCI compliance scan

DB="/var/lib/prtip/pci-compliance.db"
REPORT_DIR="/var/reports/pci"
MONTH=$(date +%Y-%m)

# PCI DSS prohibited ports: FTP(21), Telnet(23), RDP(3389), unencrypted MySQL(3306)
PROHIBITED_PORTS="21,22,23,135-139,445,1433,3306,3389"

# Scan entire network
prtip -sV -p ${PROHIBITED_PORTS} 192.168.0.0/16 10.0.0.0/16 \
  --with-db --database ${DB}

# Export findings
SCAN_ID=$(sqlite3 ${DB} "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1;")
prtip db export ${DB} --scan-id ${SCAN_ID} --format json -o ${REPORT_DIR}/pci-${MONTH}.json
prtip db export ${DB} --scan-id ${SCAN_ID} --format csv -o ${REPORT_DIR}/pci-${MONTH}.csv

# Generate summary
sqlite3 ${DB} "
  SELECT target_ip, port, service, version
  FROM scan_results
  WHERE scan_id = ${SCAN_ID}
    AND state = 'open'
  ORDER BY target_ip, port;
" > ${REPORT_DIR}/pci-${MONTH}-summary.txt

# Email report to compliance team
mail -s "PCI Compliance Report ${MONTH}" compliance@example.com < ${REPORT_DIR}/pci-${MONTH}-summary.txt
```

### Example 3: Vulnerability Tracking

**Goal:** Track remediation progress for identified vulnerabilities

**PostgreSQL Schema:**
```sql
CREATE TABLE prtip.vulnerabilities (
    vuln_id SERIAL PRIMARY KEY,
    target_ip INET NOT NULL,
    port INTEGER NOT NULL,
    service VARCHAR(100),
    version VARCHAR(200),
    cve_id VARCHAR(20),
    severity VARCHAR(10) CHECK(severity IN ('low', 'medium', 'high', 'critical')),
    discovered_at TIMESTAMPTZ DEFAULT NOW(),
    remediated_at TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'open' CHECK(status IN ('open', 'in_progress', 'remediated', 'false_positive'))
);

-- Track remediation progress
SELECT
    severity,
    status,
    COUNT(*) AS count
FROM prtip.vulnerabilities
GROUP BY severity, status
ORDER BY
    CASE severity
        WHEN 'critical' THEN 1
        WHEN 'high' THEN 2
        WHEN 'medium' THEN 3
        WHEN 'low' THEN 4
    END,
    status;
```

---

## Best Practices

### 1. Connection Pooling

**PostgreSQL with pgbouncer:**
```ini
# /etc/pgbouncer/pgbouncer.ini
[databases]
security = host=localhost port=5432 dbname=security

[pgbouncer]
listen_addr = 127.0.0.1
listen_port = 6432
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt
pool_mode = transaction
max_client_conn = 100
default_pool_size = 25
```

**Python with psycopg2:**
```python
from psycopg2 import pool

# Create connection pool (application startup)
db_pool = pool.SimpleConnectionPool(
    minconn=5,
    maxconn=20,
    host='localhost',
    port=6432,  # pgbouncer port
    database='security',
    user='prtip',
    password='secure-password'
)

def execute_query(query, params):
    conn = db_pool.getconn()
    try:
        cur = conn.cursor()
        cur.execute(query, params)
        result = cur.fetchall()
        conn.commit()
        return result
    finally:
        db_pool.putconn(conn)
```

### 2. Prepared Statements

**Prevent SQL Injection + Performance:**
```python
import psycopg2

conn = psycopg2.connect(...)
cur = conn.cursor()

# BAD: String interpolation (SQL injection risk)
target_ip = "192.168.1.100'; DROP TABLE scan_results; --"
cur.execute(f"SELECT * FROM scan_results WHERE target_ip = '{target_ip}'")  # DANGEROUS!

# GOOD: Parameterized query (safe + prepared statement caching)
target_ip = "192.168.1.100"
cur.execute("SELECT * FROM scan_results WHERE target_ip = %s", (target_ip,))
```

### 3. Transaction Management

**ACID Guarantees:**
```python
conn = psycopg2.connect(...)
conn.autocommit = False  # Explicit transaction control

try:
    cur = conn.cursor()

    # Insert scan metadata
    cur.execute("""
        INSERT INTO scans (scan_id, start_time, config_jsonb)
        VALUES (%s, %s, %s)
    """, (scan_id, start_time, config))

    # Insert scan results (batch)
    execute_batch(cur, """
        INSERT INTO scan_results (scan_id, target_ip, port, state, service)
        VALUES (%s, %s, %s, %s, %s)
    """, results, page_size=1000)

    # Commit transaction (all-or-nothing)
    conn.commit()

except Exception as e:
    # Rollback on error (atomic operation)
    conn.rollback()
    raise e
finally:
    cur.close()
    conn.close()
```

### 4. Error Handling

**Graceful Degradation:**
```python
import logging

logger = logging.getLogger(__name__)

def store_scan_results(scan_id, results, db_config):
    """Store scan results with error handling"""

    try:
        conn = psycopg2.connect(**db_config)
        cur = conn.cursor()

        # Insert results
        execute_batch(cur, """
            INSERT INTO scan_results (scan_id, target_ip, port, state, service)
            VALUES (%s, %s, %s, %s, %s)
        """, results, page_size=1000)

        conn.commit()
        logger.info(f"Stored {len(results)} results for scan {scan_id}")

    except psycopg2.OperationalError as e:
        logger.error(f"Database connection failed: {e}")
        # Fallback: write to JSON file
        with open(f'/tmp/scan-{scan_id}.json', 'w') as f:
            json.dump(results, f)
        logger.warning(f"Results written to /tmp/scan-{scan_id}.json")

    except psycopg2.IntegrityError as e:
        logger.error(f"Constraint violation: {e}")
        conn.rollback()

    except Exception as e:
        logger.error(f"Unexpected error: {e}")
        conn.rollback()
        raise

    finally:
        if cur:
            cur.close()
        if conn:
            conn.close()
```

### 5. Monitoring & Logging

**PostgreSQL Logging:**
```sql
-- Enable statement logging (postgresql.conf)
log_statement = 'all'
log_duration = on
log_min_duration_statement = 1000  -- Log queries > 1 second

-- Track slow queries
SELECT
    query,
    calls,
    total_time / 1000 AS total_seconds,
    mean_time / 1000 AS mean_seconds
FROM pg_stat_statements
ORDER BY total_time DESC
LIMIT 20;
```

**Application Logging:**
```python
import logging
from pythonjsonlogger import jsonlogger

# Structured logging
handler = logging.FileHandler('/var/log/prtip-db.log')
formatter = jsonlogger.JsonFormatter()
handler.setFormatter(formatter)

logger = logging.getLogger()
logger.addHandler(handler)
logger.setLevel(logging.INFO)

# Log database operations
logger.info('Database insert', extra={
    'scan_id': scan_id,
    'results_count': len(results),
    'duration_ms': duration,
    'target_network': target_network
})
```

---

## See Also

- **[Integration](./integration.md)** - API integration, CI/CD, SIEM forwarding
- **[Output Formats](../user-guide/output-formats.md)** - JSON, CSV, XML, Text export formats
- **[Large-Scale Scanning](./large-scale-scanning.md)** - Performance tuning for massive scans
- **[Distributed Scanning](./distributed-scanning.md)** - Multi-host scanning architecture
- **[Performance Tuning](./performance-tuning.md)** - Query optimization strategies

**External Resources:**
- **PostgreSQL Documentation**: [https://www.postgresql.org/docs/](https://www.postgresql.org/docs/)
- **SQLite Documentation**: [https://www.sqlite.org/docs.html](https://www.sqlite.org/docs.html)
- **ClickHouse Documentation**: [https://clickhouse.com/docs/](https://clickhouse.com/docs/)
- **psycopg2 Tutorial**: [https://www.psycopg.org/docs/](https://www.psycopg.org/docs/)

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
