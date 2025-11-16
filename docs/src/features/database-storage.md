# Database Storage

ProRT-IP provides comprehensive SQLite database support for storing, querying, and analyzing scan results over time. The database system enables historical tracking, change detection, and integration with external analysis tools.

## Overview

The database system enables:
- **Persistent Storage**: Save scan results for long-term analysis
- **Historical Tracking**: Monitor network changes over time
- **Change Detection**: Compare scans to identify new services, closed ports, or version updates
- **Export Integration**: Export to JSON, CSV, XML (Nmap-compatible), or text formats
- **Query Interface**: Search by scan ID, target, port, or service
- **Performance Optimized**: WAL mode, batch inserts, comprehensive indexes

**Database Engine**: SQLite 3.x with Write-Ahead Logging (WAL) for concurrent access

## Database Schema

### Tables

**`scans` Table** - Scan metadata:

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique scan identifier |
| `start_time` | TIMESTAMP | Scan start time (UTC) |
| `end_time` | TIMESTAMP | Scan completion time (NULL if in progress) |
| `config_json` | TEXT | Scan configuration (JSON format) |

**`scan_results` Table** - Individual port results:

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

Comprehensive indexes for fast queries:

- `idx_scan_results_scan_id` on `scan_results(scan_id)` - Query by scan
- `idx_scan_results_target_ip` on `scan_results(target_ip)` - Query by host
- `idx_scan_results_port` on `scan_results(port)` - Query by port
- `idx_scan_results_state` on `scan_results(state)` - Filter by state

**Query Performance**: Logarithmic scaling with database size (O(log n))

## Storing Scan Results

### Basic Storage

Enable database storage with the `--with-db` flag:

```bash
# Default location (./scans.db)
prtip -p 80,443 192.168.1.1 --with-db

# Custom database location
prtip -p 80,443 192.168.1.1 --with-db --database /path/to/results.db

# Scan with service detection
prtip -sV -p 1-1000 target.com --with-db --database security-audit.db
```

### Organizational Strategies

**Purpose-Based Databases:**

```bash
# Full network scans
prtip -p- network.com --with-db --database full-scan.db

# Service-specific audits
prtip -sV -p 22,80,443 network.com --with-db --database service-audit.db

# Vulnerability scanning
prtip -sV -p 21,22,23,3389 192.168.1.0/24 --with-db --database vuln-scan.db
```

**Time-Based Tracking:**

```bash
# Daily scans with date stamping
prtip -sV -p 22,23,3389 192.168.1.0/24 --with-db --database daily-$(date +%Y%m%d).db

# Continuous monitoring (single database)
prtip -sV -p 22,23,3389 192.168.1.0/24 --with-db --database security-monitor.db
```

**Compliance Audits:**

```bash
# PCI DSS scan
prtip -p 21,22,23,135-139,445,1433,3306,3389 \
  192.168.1.0/24 \
  --with-db --database pci-audit-$(date +%Y%m%d).db

# SOC 2 quarterly scan
prtip -sV -p- critical-systems.txt --with-db --database soc2-q$(date +%q)-2025.db
```

## Querying Results

### List All Scans

View scan history:

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
prtip db query results.db --service mysql
prtip db query results.db --service ssh
```

### Filter Open Ports

Show only open ports:

```bash
prtip db query results.db --scan-id 1 --open
prtip db query results.db --target 192.168.1.100 --open
```

## Exporting Results

ProRT-IP supports exporting to multiple formats for analysis and reporting.

### Export Formats

**JSON** - Machine-readable, preserves all data:

```bash
prtip db export results.db --scan-id 1 --format json -o scan1.json
```

**Example:**
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

**CSV** - Spreadsheet-compatible:

```bash
prtip db export results.db --scan-id 1 --format csv -o scan1.csv
```

**Example:**
```
Target IP,Port,State,Service,Version,Banner,Response Time (ms),Timestamp
192.168.1.100,22,Open,ssh,OpenSSH 8.9,SSH-2.0-OpenSSH_8.9,2,2025-10-24T10:30:15Z
192.168.1.100,80,Open,http,Apache 2.4.52,,5,2025-10-24T10:30:16Z
```

**XML** - Nmap-compatible:

```bash
prtip db export results.db --scan-id 1 --format xml -o scan1.xml
```

**Example:**
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

**Text** - Human-readable summary:

```bash
prtip db export results.db --scan-id 1 --format text -o scan1.txt
```

### Export Workflows

**Security Reporting:**

```bash
# Management report
prtip db export audit.db --scan-id 1 --format text -o security-report.txt

# Data analysis spreadsheet
prtip db export audit.db --scan-id 1 --format csv -o security-data.csv
```

**Tool Integration:**

```bash
# Export to Nmap XML for compatibility
prtip db export results.db --scan-id 1 --format xml -o nmap-format.xml

# Process with Nmap XML tools
nmap-vulners nmap-format.xml
```

## Comparing Scans

Compare two scans to identify network changes.

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

**Detect Unauthorized Services:**

```bash
# Weekly comparison
prtip db compare weekly-scans.db 1 2

# Alert on new ports
prtip db compare weekly-scans.db 1 2 | grep "New Open Ports" -A 10
```

**Track Patch Management:**

```bash
# Compare before/after patching
prtip db compare patch-validation.db 1 2

# Verify service versions updated
prtip db compare patch-validation.db 1 2 | grep "Changed Services"
```

**Compliance Monitoring:**

```bash
# Daily PCI DSS comparison
for i in {1..30}; do
  prtip db compare compliance.db $i $((i+1))
done
```

## Performance

### Database Optimization

ProRT-IP automatically optimizes database performance:

1. **WAL Mode**: Write-Ahead Logging enabled for better concurrency
2. **Batch Inserts**: 1,000-10,000 results per transaction
3. **Comprehensive Indexes**: All critical columns indexed
4. **Stream-to-Disk**: Results written immediately (no memory buffering)

### Large Scan Performance

For scans with >100K results:

```bash
# Adaptive parallelism handles large scans efficiently
prtip -p- 10.0.0.0/16 --with-db --database large-scan.db

# Database remains responsive during scan (streaming writes)
```

### Query Performance

**Fast Queries** (uses indexes):

```bash
prtip db query results.db --target 192.168.1.100  # O(log n)
prtip db query results.db --port 22               # O(log n)
prtip db query results.db --scan-id 1             # O(log n)
```

**Slower Queries** (requires full scan):

```bash
prtip db query results.db --service apache        # O(n) - no index on service
```

### Database Maintenance

```bash
# Reclaim space after deleting old scans
sqlite3 results.db "VACUUM;"

# Optimize query performance
sqlite3 results.db "ANALYZE;"

# Check database integrity
sqlite3 results.db "PRAGMA integrity_check;"
```

## Advanced Usage

### Direct SQL Access

For advanced queries, use SQLite directly:

```bash
# Find all hosts with high-risk ports open
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

### Automated Monitoring

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

# Compare and alert if changes detected
if prtip db compare $DB $SCAN1 $SCAN2 | grep -q "New Open Ports"; then
  echo "ALERT: New services detected!" | mail -s "Security Alert" security@company.com
fi
```

### Diff Analysis

```bash
# Export both scans
prtip db export results.db --scan-id 1 --format json -o scan1.json
prtip db export results.db --scan-id 2 --format json -o scan2.json

# Use jq for detailed diff
jq -S . scan1.json > scan1.sorted.json
jq -S . scan2.json > scan2.sorted.json
diff scan1.sorted.json scan2.sorted.json
```

## Troubleshooting

### Database Locked

**Problem**: `database is locked` error

**Solution**:
```bash
# Check for other prtip processes
ps aux | grep prtip

# Enable timeout in SQLite (30 seconds)
sqlite3 results.db "PRAGMA busy_timeout = 30000;"
```

### Database Corruption

**Problem**: Database file corrupted

**Solution**:
```bash
# Check integrity
sqlite3 results.db "PRAGMA integrity_check;"

# Attempt recovery
sqlite3 results.db ".recover" | sqlite3 recovered.db

# Restore from backup
cp results.db.backup results.db
```

### No Results Found

**Problem**: Query returns no results

**Solution**:
```bash
# Verify scan completed
prtip db list results.db

# Check scan has results
prtip db query results.db --scan-id 1

# Verify target format (no CIDR notation)
prtip db query results.db --target "192.168.1.100"  # NOT "192.168.1.100/32"
```

### Export Fails

**Problem**: Export command fails

**Solution**:
```bash
# Verify output directory exists
mkdir -p /path/to/exports

# Check disk space
df -h

# Verify scan ID exists
prtip db list results.db
```

## Best Practices

### Organize by Purpose

Use separate databases for different purposes:

```bash
# Development scanning
prtip -p 80,443 dev.example.com --with-db --database dev-scans.db

# Production audits
prtip -sV -p- prod.example.com --with-db --database prod-audits.db

# Security assessments
prtip -A external-targets.txt --with-db --database security-assessments.db
```

### Regular Backups

```bash
# Automated backup before each scan
cp security-monitor.db security-monitor.db.backup
prtip -sV -p 22,80,443 192.168.1.0/24 --with-db --database security-monitor.db
```

### Archive Old Scans

```bash
# Export old scans before deletion
prtip db export results.db --scan-id 1 --format json -o archive/scan-1.json

# Delete from database
sqlite3 results.db "DELETE FROM scan_results WHERE scan_id = 1;"
sqlite3 results.db "DELETE FROM scans WHERE id = 1;"

# Reclaim space
sqlite3 results.db "VACUUM;"
```

### Compliance Documentation

```bash
# Generate compliance reports
prtip db export pci-audit.db --scan-id 1 --format text -o reports/pci-audit-$(date +%Y%m%d).txt
prtip db export pci-audit.db --scan-id 1 --format csv -o reports/pci-audit-$(date +%Y%m%d).csv

# Store for audit trail
tar -czf pci-audit-$(date +%Y%m).tar.gz reports/*.txt reports/*.csv
```

## See Also

- [Output Formats](../user-guide/output-formats.md) - Export format specifications
- [CLI Reference](../user-guide/cli-reference.md#database-storage) - Database flags
- [Performance Tuning](../advanced/performance-tuning.md) - Optimization guide
- [Nmap Compatibility](./nmap-compatibility.md) - XML export compatibility
