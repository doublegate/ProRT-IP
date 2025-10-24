# Database Operations Guide

**Version:** 0.4.0 (Sprint 4.18.1)
**Last Updated:** 2025-10-24

## Overview

ProRT-IP WarScan includes a powerful SQLite database system for storing, querying, and comparing scan results over time. This guide covers all database operations including storage, queries, exports, and historical analysis.

## Table of Contents

- [Quick Start](#quick-start)
- [Database Schema](#database-schema)
- [Storing Scan Results](#storing-scan-results)
- [Querying Database](#querying-database)
- [Exporting Results](#exporting-results)
- [Comparing Scans](#comparing-scans)
- [Performance Tips](#performance-tips)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Store Scan Results

```bash
# Scan and save to database
prtip -p 1-1000 192.168.1.0/24 --with-db --database results.db

# Scan with service detection and save
prtip -sV -p 80,443 target.com --with-db --database security-audit.db
```

### List All Scans

```bash
prtip db list results.db
```

### Query Results

```bash
# Get all results for scan ID 1
prtip db query results.db --scan-id 1

# Find all open ports on specific target
prtip db query results.db --target 192.168.1.100

# Find all hosts with SSH open
prtip db query results.db --port 22

# Find all hosts running Apache
prtip db query results.db --service apache
```

### Export Results

```bash
# Export to JSON
prtip db export results.db --scan-id 1 --format json -o scan-results.json

# Export to CSV for spreadsheet analysis
prtip db export results.db --scan-id 1 --format csv -o scan-results.csv

# Export to Nmap-compatible XML
prtip db export results.db --scan-id 1 --format xml -o scan-results.xml
```

### Compare Scans

```bash
# Compare two scans to detect changes
prtip db compare results.db 1 2
```

## Database Schema

ProRT-IP uses SQLite with the following schema:

### `scans` Table

Stores metadata about each scan.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique scan identifier |
| `start_time` | TIMESTAMP | Scan start time (UTC) |
| `end_time` | TIMESTAMP | Scan completion time (NULL if in progress) |
| `config_json` | TEXT | Scan configuration (JSON format) |

### `scan_results` Table

Stores individual port scan results.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique result identifier |
| `scan_id` | INTEGER | Foreign key to `scans.id` |
| `target_ip` | TEXT | Target IP address |
| `port` | INTEGER | Port number (1-65535) |
| `state` | TEXT | Port state: 'open', 'closed', 'filtered', 'unknown' |
| `service` | TEXT | Detected service name (NULL if unknown) |
| `version` | TEXT | Service version (NULL if undetected) |
| `banner` | TEXT | Service banner (NULL if unavailable) |
| `response_time_ms` | INTEGER | Response time in milliseconds |
| `timestamp` | TIMESTAMP | Timestamp of this specific check |

### Indexes

- `idx_scan_results_scan_id` on `scan_results(scan_id)`
- `idx_scan_results_target_ip` on `scan_results(target_ip)`
- `idx_scan_results_port` on `scan_results(port)`
- `idx_scan_results_state` on `scan_results(state)`

## Storing Scan Results

### Basic Storage

Use the `--with-db` flag to enable database storage:

```bash
# Default database location (./scans.db)
prtip -p 80,443 192.168.1.1 --with-db

# Custom database location
prtip -p 80,443 192.168.1.1 --with-db --database /path/to/results.db
```

### Best Practices

1. **Organize by Purpose**
   ```bash
   # Separate databases for different purposes
   prtip -p- network.com --with-db --database full-scan.db
   prtip -sV -p 22,80,443 network.com --with-db --database service-audit.db
   ```

2. **Periodic Scans**
   ```bash
   # Daily security scans
   prtip -sV -p 22,23,3389 192.168.1.0/24 --with-db --database daily-$(date +%Y%m%d).db

   # Or append to same database for trending
   prtip -sV -p 22,23,3389 192.168.1.0/24 --with-db --database security-monitor.db
   ```

3. **Compliance Audits**
   ```bash
   # PCI DSS scan
   prtip -p 21,22,23,135-139,445,1433,3306,3389 \
     192.168.1.0/24 \
     --with-db --database pci-audit-$(date +%Y%m%d).db
   ```

## Querying Database

### List Scans

View all scans in a database:

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

### Query by Scan ID

Retrieve all results for a specific scan:

```bash
prtip db query results.db --scan-id 1
```

### Query by Target

Find all open ports on a specific host:

```bash
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

### Query by Port

Find all hosts with a specific port open:

```bash
prtip db query results.db --port 22
```

**Example Output:**
```
Hosts with Port 22 Open
================================================================================
Target IP          Port     State        Service              Version
================================================================================
192.168.1.10       22       open         ssh                  OpenSSH 8.9
192.168.1.25       22       open         ssh                  OpenSSH 7.4
192.168.1.100      22       open         ssh                  OpenSSH 8.9
================================================================================
```

### Query by Service

Find all hosts running a specific service:

```bash
prtip db query results.db --service apache
```

### Filter Open Ports Only

Add `--open` flag to show only open ports:

```bash
prtip db query results.db --scan-id 1 --open
```

## Exporting Results

ProRT-IP supports exporting scan results to multiple formats for analysis and reporting.

### Export Formats

#### JSON

Machine-readable format, preserves all data:

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
    "version": "OpenSSH 8.9",
    "raw_response": null
  }
]
```

#### CSV

Spreadsheet-compatible format:

```bash
prtip db export results.db --scan-id 1 --format csv -o scan1.csv
```

**Example CSV:**
```
Target IP,Port,State,Service,Version,Banner,Response Time (ms),Timestamp
192.168.1.100,22,Open,ssh,OpenSSH 8.9,SSH-2.0-OpenSSH_8.9,2,2025-10-24T10:30:15Z
192.168.1.100,80,Open,http,Apache 2.4.52,,5,2025-10-24T10:30:16Z
```

#### XML (Nmap-Compatible)

Compatible with Nmap XML parsers:

```bash
prtip db export results.db --scan-id 1 --format xml -o scan1.xml
```

**Example XML:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE nmaprun>
<nmaprun scanner="prtip" version="0.4.0" xmloutputversion="1.05">
  <host>
    <address addr="192.168.1.100" addrtype="ipv4"/>
    <ports>
      <port protocol="tcp" portid="22">
        <state state="open"/>
        <service name="ssh" product="OpenSSH" version="8.9"/>
      </port>
    </ports>
  </host>
</nmaprun>
```

#### Text

Human-readable summary:

```bash
prtip db export results.db --scan-id 1 --format text -o scan1.txt
```

### Export Workflows

#### Security Reporting

```bash
# Generate report for management
prtip db export audit.db --scan-id 1 --format text -o security-report.txt

# Generate spreadsheet for analysis
prtip db export audit.db --scan-id 1 --format csv -o security-data.csv
```

#### Integration with Other Tools

```bash
# Export to Nmap XML for tool compatibility
prtip db export results.db --scan-id 1 --format xml -o nmap-format.xml

# Process with existing Nmap XML parsers
nmap-vulners nmap-format.xml
```

## Comparing Scans

Compare two scans to identify changes in network security posture.

### Basic Comparison

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

### Use Cases

#### Detect Unauthorized Services

```bash
# Compare weekly scans
prtip db compare weekly-scans.db 1 2

# Alert on new ports
prtip db compare weekly-scans.db 1 2 | grep "New Open Ports" -A 10
```

#### Track Patch Management

```bash
# Compare before/after patching
prtip db compare patch-validation.db 1 2

# Verify service versions updated
prtip db compare patch-validation.db 1 2 | grep "Changed Services"
```

#### Compliance Monitoring

```bash
# Daily PCI scans
for i in {1..30}; do
  prtip db compare compliance.db $i $((i+1))
done
```

## Performance Tips

### Database Optimization

1. **WAL Mode**

   SQLite Write-Ahead Logging (WAL) mode provides better concurrency. ProRT-IP enables this automatically.

2. **Batch Inserts**

   ProRT-IP uses batch inserts (1,000-10,000 results per transaction) for optimal performance.

3. **Indexes**

   All critical columns are indexed. Query performance scales logarithmically with database size.

### Large Scan Performance

For scans with >100K results:

```bash
# Use adaptive parallelism
prtip -p- 10.0.0.0/16 --with-db --database large-scan.db

# Results stream to disk immediately (no memory buffering)
# Database remains responsive during scan
```

### Query Performance

```bash
# Fast: Uses index on target_ip
prtip db query results.db --target 192.168.1.100

# Fast: Uses index on port
prtip db query results.db --port 22

# Fast: Uses index on scan_id
prtip db query results.db --scan-id 1

# Slower: Service name is not indexed (requires full scan)
prtip db query results.db --service apache
```

### Database Maintenance

```bash
# Vacuum database to reclaim space (after deleting old scans)
sqlite3 results.db "VACUUM;"

# Analyze database for query optimization
sqlite3 results.db "ANALYZE;"

# Check database integrity
sqlite3 results.db "PRAGMA integrity_check;"
```

## Troubleshooting

### Common Issues

#### Database Locked

**Problem:** `database is locked` error

**Solution:**
```bash
# Ensure no other prtip processes are using the database
ps aux | grep prtip

# If necessary, enable timeout in SQLite
sqlite3 results.db "PRAGMA busy_timeout = 30000;"
```

#### Database Corruption

**Problem:** Database file corrupted

**Solution:**
```bash
# Check integrity
sqlite3 results.db "PRAGMA integrity_check;"

# Attempt recovery
sqlite3 results.db ".recover" | sqlite3 recovered.db

# If unrecoverable, restore from backup
cp results.db.backup results.db
```

#### No Results Found

**Problem:** Query returns no results

**Solution:**
```bash
# Verify scan completed
prtip db list results.db

# Check scan has results
prtip db query results.db --scan-id 1

# Verify target IP format
prtip db query results.db --target "192.168.1.100" # NOT "192.168.1.100/32"
```

#### Export Fails

**Problem:** Export command fails

**Solution:**
```bash
# Verify output directory exists
mkdir -p /path/to/exports

# Check disk space
df -h

# Verify scan ID exists
prtip db list results.db
```

### Getting Help

```bash
# Database command help
prtip db --help

# Query command help
prtip db query --help

# Export command help
prtip db export --help

# Compare command help
prtip db compare --help
```

## Advanced Usage

### Combining with Other Tools

#### Diff Analysis

```bash
# Export both scans
prtip db export results.db --scan-id 1 --format json -o scan1.json
prtip db export results.db --scan-id 2 --format json -o scan2.json

# Use jq for detailed diff
jq -S . scan1.json > scan1.sorted.json
jq -S . scan2.json > scan2.sorted.json
diff scan1.sorted.json scan2.sorted.json
```

#### Automated Monitoring

```bash
#!/bin/bash
# Daily scan and comparison script

DB="security-monitor.db"
TARGET="192.168.1.0/24"

# Run today's scan
prtip -sV -p 22,23,80,443,3389 $TARGET --with-db --database $DB

# Get last two scan IDs
SCAN1=$(sqlite3 $DB "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1 OFFSET 1;")
SCAN2=$(sqlite3 $DB "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1;")

# Compare and alert if changes
if prtip db compare $DB $SCAN1 $SCAN2 | grep -q "New Open Ports"; then
  echo "ALERT: New services detected!" | mail -s "Security Alert" security@company.com
fi
```

### Direct SQL Access

For advanced queries, access the database directly with SQLite:

```bash
# Find all hosts with high-value ports open
sqlite3 results.db "
  SELECT DISTINCT target_ip, port, service
  FROM scan_results
  WHERE state = 'open'
  AND port IN (21, 22, 23, 3389, 5900)
  ORDER BY target_ip, port;
"

# Count results by state
sqlite3 results.db "
  SELECT state, COUNT(*) as count
  FROM scan_results
  GROUP BY state;
"

# Find services with known versions
sqlite3 results.db "
  SELECT target_ip, port, service, version
  FROM scan_results
  WHERE version IS NOT NULL
  ORDER BY service, version;
"
```

## See Also

- [Architecture Documentation](00-ARCHITECTURE.md) - System design
- [API Reference](05-API-REFERENCE.md) - Programmatic access
- [Nmap Compatibility](14-NMAP_COMPATIBILITY.md) - Nmap flag reference
- [Performance Guide](21-PERFORMANCE-GUIDE.md) - Optimization techniques

---

**Sprint 4.18.1 - SQLite Query Interface & Export Utilities**
Generated with ProRT-IP WarScan v0.4.0
