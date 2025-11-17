# Automation

Automate ProRT-IP scans for continuous monitoring, compliance, and security workflows.

## What is Automation?

**Automation** transforms ProRT-IP from an interactive tool into a continuous security monitoring platform. Automated workflows enable scheduled scans, event-driven responses, compliance reporting, and integration with existing infrastructure.

**ProRT-IP Automation Capabilities:**
- **Scheduled Scanning** - Cron/systemd timers for periodic network scans
- **Event-Driven Workflows** - Webhook triggers for real-time responses
- **CI/CD Integration** - Automated security testing in build pipelines
- **Change Detection** - Automatic alerts on network topology changes
- **Compliance Automation** - Continuous validation against security policies
- **Incident Response** - Automated scanning on security events
- **Asset Inventory** - Maintain up-to-date network infrastructure databases
- **Vulnerability Tracking** - Automated correlation with CVE feeds
- **Report Generation** - Automated compliance and audit reports

**Use Cases:**
- **24/7 Network Monitoring** - Detect unauthorized services or devices
- **DevSecOps Pipelines** - Shift-left security scanning in CI/CD
- **Compliance Auditing** - Automated PCI-DSS, HIPAA, SOC2 compliance checks
- **Threat Hunting** - Continuous scanning for IOCs and anomalies
- **Change Management** - Validate network state before/after deployments
- **Disaster Recovery** - Automated baseline scans for recovery validation

---

## Cron-based Automation

### Basic Scheduled Scans

**Daily Network Scan:**
```bash
# /etc/cron.daily/prtip-scan

#!/bin/bash
set -e

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
TARGET="192.168.1.0/24"
RESULTS_DIR="/var/log/prtip"
RESULTS_FILE="${RESULTS_DIR}/scan-${TIMESTAMP}.json"

# Create results directory
mkdir -p "${RESULTS_DIR}"

# Run scan
prtip -sS -sV -p 1-1000 "${TARGET}" \
  -oJ "${RESULTS_FILE}" \
  -oX "${RESULTS_DIR}/scan-${TIMESTAMP}.xml" \
  --with-db --database "${RESULTS_DIR}/scans.db"

# Compress old results (>7 days)
find "${RESULTS_DIR}" -name "scan-*.json" -mtime +7 -exec gzip {} \;

# Delete very old results (>30 days)
find "${RESULTS_DIR}" -name "scan-*.json.gz" -mtime +30 -delete

echo "Scan complete: ${RESULTS_FILE}"
```

**Make executable:**
```bash
sudo chmod +x /etc/cron.daily/prtip-scan
```

### Hourly Monitoring

**Monitor Critical Hosts (Crontab):**
```bash
# Edit crontab
crontab -e

# Add entry (hourly scan of critical servers)
0 * * * * /usr/local/bin/prtip-monitor-critical.sh >> /var/log/prtip/monitor.log 2>&1
```

**Script: /usr/local/bin/prtip-monitor-critical.sh:**
```bash
#!/bin/bash

CRITICAL_HOSTS=(
  "192.168.1.10"  # Database server
  "192.168.1.20"  # Web server
  "192.168.1.30"  # Mail server
)

TIMESTAMP=$(date +%Y-%m-%d_%H:%M:%S)

for host in "${CRITICAL_HOSTS[@]}"; do
  echo "[${TIMESTAMP}] Scanning ${host}..."

  # Quick scan (top 100 ports)
  prtip -F "${host}" -oJ "/tmp/prtip-${host}-${TIMESTAMP}.json"

  # Check for unexpected open ports
  UNEXPECTED=$(jq '[.hosts[].ports[] | select(.state == "Open" and (.port | IN(22, 80, 443, 3306, 25, 143) | not))] | length' "/tmp/prtip-${host}-${TIMESTAMP}.json")

  if [ "$UNEXPECTED" -gt 0 ]; then
    # Alert on unexpected ports
    echo "ALERT: Found ${UNEXPECTED} unexpected open ports on ${host}" | \
      mail -s "Security Alert: ${host}" security@example.com
  fi

  # Cleanup
  rm "/tmp/prtip-${host}-${TIMESTAMP}.json"
done
```

### Weekly Full Scans

**Comprehensive Subnet Scan:**
```bash
# /etc/cron.weekly/prtip-full-scan

#!/bin/bash
set -e

TIMESTAMP=$(date +%Y%m%d)
TARGET="10.0.0.0/16"
RESULTS_DIR="/var/lib/prtip/weekly"
DB_FILE="${RESULTS_DIR}/scans-weekly.db"

mkdir -p "${RESULTS_DIR}"

echo "Starting weekly full scan: ${TIMESTAMP}"

# Full scan (all 65,535 ports + service detection + OS detection)
prtip -sS -sV -O -p- -T4 "${TARGET}" \
  --with-db --database "${DB_FILE}" \
  -oJ "${RESULTS_DIR}/scan-${TIMESTAMP}.json" \
  -oX "${RESULTS_DIR}/scan-${TIMESTAMP}.xml" \
  2>&1 | tee "${RESULTS_DIR}/scan-${TIMESTAMP}.log"

# Generate HTML report
python3 /usr/local/bin/generate-report.py \
  --input "${RESULTS_DIR}/scan-${TIMESTAMP}.json" \
  --output "${RESULTS_DIR}/report-${TIMESTAMP}.html"

# Email report to team
mutt -s "Weekly Network Scan Report - ${TIMESTAMP}" \
  -a "${RESULTS_DIR}/report-${TIMESTAMP}.html" \
  -- security-team@example.com < /dev/null

echo "Weekly scan complete: ${TIMESTAMP}"
```

### Change Detection

**Detect Network Changes:**
```bash
#!/bin/bash
# /etc/cron.daily/prtip-change-detection

TARGET="192.168.1.0/24"
DB="/var/lib/prtip/changes.db"

# Run today's scan
prtip -sS -p 1-1000 "${TARGET}" --with-db --database "${DB}"

# Get last two scan IDs
SCAN1=$(sqlite3 "${DB}" "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1 OFFSET 1;")
SCAN2=$(sqlite3 "${DB}" "SELECT id FROM scans ORDER BY start_time DESC LIMIT 1;")

if [ -z "$SCAN1" ] || [ -z "$SCAN2" ]; then
  echo "Not enough scans to compare (need at least 2)"
  exit 0
fi

# Compare scans
DIFF=$(prtip db compare "${DB}" "${SCAN1}" "${SCAN2}")

# Check for changes
if echo "${DIFF}" | grep -q "New Open Ports\|Closed Ports\|Changed Services"; then
  # Email alert
  echo "${DIFF}" | mail -s "ALERT: Network Changes Detected" security@example.com

  # Log to syslog
  logger -t prtip-change-detection "Network changes detected between scan ${SCAN1} and ${SCAN2}"
fi
```

---

## Shell Scripting Automation

### Parallel Scanning

**Multi-Target Scanning with GNU Parallel:**
```bash
#!/bin/bash
# parallel-scan.sh

# List of targets
TARGETS=(
  "192.168.1.0/24"
  "192.168.2.0/24"
  "192.168.3.0/24"
  "10.0.0.0/16"
)

# Function to scan single target
scan_target() {
  local target=$1
  local timestamp=$(date +%Y%m%d-%H%M%S)
  local safe_target=$(echo "${target}" | tr '/' '-')

  echo "Scanning ${target}..."

  prtip -sS -p 1-1000 "${target}" \
    -oJ "results-${safe_target}-${timestamp}.json" \
    2>&1 | tee "scan-${safe_target}-${timestamp}.log"

  echo "Completed: ${target}"
}

export -f scan_target

# Run scans in parallel (4 concurrent)
printf '%s\n' "${TARGETS[@]}" | parallel -j 4 scan_target
```

### Result Processing Pipeline

**Extract and Process Scan Results:**
```bash
#!/bin/bash
# process-results.sh

RESULTS_FILE=$1

if [ ! -f "$RESULTS_FILE" ]; then
  echo "Usage: $0 <scan-results.json>"
  exit 1
fi

# Extract open ports
echo "=== Open Ports ==="
jq -r '.hosts[] | "\(.ip): " + ([.ports[] | select(.state == "Open") | .port] | map(tostring) | join(", "))' "$RESULTS_FILE"

# Extract services with versions
echo -e "\n=== Services ==="
jq -r '.hosts[].ports[] | select(.state == "Open" and .service != null) | "\(.port)/\(.protocol): \(.service.name) \(.service.version // "unknown")"' "$RESULTS_FILE" | sort -u

# Count port states
echo -e "\n=== Port State Summary ==="
jq -r '.hosts[].ports[] | .state' "$RESULTS_FILE" | sort | uniq -c

# Find outdated services (example: SSH < 8.0)
echo -e "\n=== Outdated SSH Versions ==="
jq -r '.hosts[] | select(.ports[].service.name == "ssh") | .ip as $ip | .ports[] | select(.service.name == "ssh" and (.service.version // "" | test("^[0-7]\\."))) | "\($ip): SSH \(.service.version)"' "$RESULTS_FILE"
```

### Compliance Validation

**PCI-DSS Port Validation:**
```bash
#!/bin/bash
# pci-compliance-check.sh

RESULTS_FILE=$1
ALLOWED_PORTS=(80 443 22)

echo "PCI-DSS Compliance Check"
echo "========================="

# Extract all open ports
OPEN_PORTS=$(jq -r '.hosts[].ports[] | select(.state == "Open") | .port' "$RESULTS_FILE" | sort -u)

VIOLATIONS=0

for port in $OPEN_PORTS; do
  # Check if port is in allowed list
  if ! echo "${ALLOWED_PORTS[@]}" | grep -qw "$port"; then
    VIOLATIONS=$((VIOLATIONS + 1))
    echo "VIOLATION: Port $port is open but not in allowed list"

    # Get details
    jq -r --arg port "$port" '.hosts[] | select(.ports[].port == ($port | tonumber)) | "  Host: \(.ip) - Service: \(.ports[] | select(.port == ($port | tonumber)) | .service.name // "unknown")"' "$RESULTS_FILE"
  fi
done

echo ""
if [ $VIOLATIONS -eq 0 ]; then
  echo "✓ PASS: All open ports are compliant"
  exit 0
else
  echo "✗ FAIL: Found $VIOLATIONS compliance violations"
  exit 1
fi
```

### Multi-Format Export

**Export to Multiple Formats:**
```bash
#!/bin/bash
# export-all-formats.sh

TARGET=$1
OUTPUT_PREFIX="scan-$(date +%Y%m%d-%H%M%S)"

if [ -z "$TARGET" ]; then
  echo "Usage: $0 <target>"
  exit 1
fi

echo "Scanning ${TARGET}..."

# Run single scan, output to all formats
prtip -sS -p 1-1000 "${TARGET}" \
  -oN "${OUTPUT_PREFIX}.txt" \
  -oJ "${OUTPUT_PREFIX}.json" \
  -oX "${OUTPUT_PREFIX}.xml" \
  -oG "${OUTPUT_PREFIX}.gnmap" \
  --with-db --database "${OUTPUT_PREFIX}.db"

echo "Results saved:"
echo "  Text:      ${OUTPUT_PREFIX}.txt"
echo "  JSON:      ${OUTPUT_PREFIX}.json"
echo "  XML:       ${OUTPUT_PREFIX}.xml"
echo "  Greppable: ${OUTPUT_PREFIX}.gnmap"
echo "  Database:  ${OUTPUT_PREFIX}.db"

# Generate summary
echo -e "\nScan Summary:"
jq -r '"Hosts scanned: " + (.hosts | length | tostring)' "${OUTPUT_PREFIX}.json"
jq -r '"Open ports: " + ([.hosts[].ports[] | select(.state == "Open")] | length | tostring)' "${OUTPUT_PREFIX}.json"
jq -r '"Services detected: " + ([.hosts[].ports[] | select(.service != null)] | length | tostring)' "${OUTPUT_PREFIX}.json"
```

---

## Python Automation

### Automated Vulnerability Correlation

**Correlate Scan Results with CVE Database:**
```python
#!/usr/bin/env python3
"""
Automated vulnerability correlation using ProRT-IP scan results.
"""
import json
import requests
import sys
from datetime import datetime

def fetch_cves_for_service(service_name, version):
    """Fetch CVEs for a specific service version"""

    # Example: Query CVE database (NVD, VulnDB, etc.)
    # This is a simplified example - real implementation would use proper CVE API
    api_url = f"https://services.nvd.nist.gov/rest/json/cves/2.0"

    params = {
        'keywordSearch': f"{service_name} {version}",
        'resultsPerPage': 10
    }

    try:
        response = requests.get(api_url, params=params, timeout=10)
        response.raise_for_status()
        return response.json().get('vulnerabilities', [])
    except Exception as e:
        print(f"Error fetching CVEs: {e}", file=sys.stderr)
        return []

def analyze_scan_results(results_file):
    """Analyze scan results for potential vulnerabilities"""

    with open(results_file) as f:
        data = json.load(f)

    vulnerabilities = []

    for host in data.get('hosts', []):
        for port in host.get('ports', []):
            if port['state'] != 'Open' or 'service' not in port:
                continue

            service = port['service']
            service_name = service.get('name')
            version = service.get('version')

            if not version:
                continue

            print(f"Checking {host['ip']}:{port['port']} - {service_name} {version}")

            # Fetch CVEs
            cves = fetch_cves_for_service(service_name, version)

            if cves:
                vulnerabilities.append({
                    'host': host['ip'],
                    'port': port['port'],
                    'service': service_name,
                    'version': version,
                    'cves': cves
                })

    return vulnerabilities

def generate_report(vulnerabilities, output_file):
    """Generate vulnerability report"""

    report = {
        'timestamp': datetime.utcnow().isoformat(),
        'total_vulnerabilities': len(vulnerabilities),
        'findings': vulnerabilities
    }

    with open(output_file, 'w') as f:
        json.dump(report, f, indent=2)

    print(f"\nVulnerability Report: {output_file}")
    print(f"Total findings: {len(vulnerabilities)}")

    # Print summary
    for vuln in vulnerabilities:
        print(f"\n{vuln['host']}:{vuln['port']} - {vuln['service']} {vuln['version']}")
        print(f"  CVEs found: {len(vuln['cves'])}")

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print("Usage: vulnerability-correlation.py <scan.json> <report.json>")
        sys.exit(1)

    vulnerabilities = analyze_scan_results(sys.argv[1])
    generate_report(vulnerabilities, sys.argv[2])
```

### Automated Remediation Workflows

**Trigger Remediation on Findings:**
```python
#!/usr/bin/env python3
"""
Automated remediation workflow based on scan findings.
"""
import json
import subprocess
import smtplib
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart

class RemediationOrchestrator:
    def __init__(self, config_file):
        with open(config_file) as f:
            self.config = json.load(f)

    def analyze_results(self, results_file):
        """Analyze scan results and trigger remediation"""

        with open(results_file) as f:
            data = json.load(f)

        findings = []

        for host in data['hosts']:
            for port in host.get('ports', []):
                if port['state'] != 'Open':
                    continue

                # Check against policy
                finding = self.check_policy(host['ip'], port)
                if finding:
                    findings.append(finding)

        return findings

    def check_policy(self, ip, port):
        """Check port/service against security policy"""

        # Prohibited ports (example)
        prohibited = {
            23: "Telnet is prohibited (use SSH instead)",
            21: "FTP is prohibited (use SFTP/FTPS instead)",
            3389: "RDP should not be internet-facing"
        }

        if port['port'] in prohibited:
            return {
                'host': ip,
                'port': port['port'],
                'service': port.get('service', {}).get('name', 'unknown'),
                'severity': 'high',
                'reason': prohibited[port['port']],
                'action': 'block'
            }

        # Outdated service versions (example)
        if port.get('service', {}).get('name') == 'ssh':
            version = port.get('service', {}).get('version', '')
            if version.startswith('OpenSSH 6.') or version.startswith('OpenSSH 7.'):
                return {
                    'host': ip,
                    'port': port['port'],
                    'service': 'ssh',
                    'version': version,
                    'severity': 'medium',
                    'reason': 'Outdated SSH version',
                    'action': 'notify'
                }

        return None

    def remediate(self, findings):
        """Execute remediation actions"""

        for finding in findings:
            action = finding['action']

            if action == 'block':
                self.block_port(finding)
            elif action == 'notify':
                self.send_notification(finding)
            elif action == 'patch':
                self.trigger_patch(finding)

    def block_port(self, finding):
        """Add firewall rule to block port"""

        print(f"Blocking {finding['host']}:{finding['port']}")

        # Example: Add iptables rule
        cmd = [
            'iptables', '-A', 'INPUT',
            '-s', finding['host'],
            '-p', 'tcp', '--dport', str(finding['port']),
            '-j', 'DROP'
        ]

        try:
            subprocess.run(cmd, check=True)
            print(f"  Firewall rule added")
        except subprocess.CalledProcessError as e:
            print(f"  Error: {e}")

    def send_notification(self, finding):
        """Send email notification"""

        smtp_config = self.config['smtp']

        msg = MIMEMultipart()
        msg['From'] = smtp_config['from']
        msg['To'] = ', '.join(smtp_config['to'])
        msg['Subject'] = f"Security Finding: {finding['host']}:{finding['port']}"

        body = f"""
Security Finding Detected:

Host: {finding['host']}
Port: {finding['port']}
Service: {finding.get('service', 'unknown')}
Severity: {finding['severity']}
Reason: {finding['reason']}

Recommended Action: {finding['action']}
        """

        msg.attach(MIMEText(body, 'plain'))

        try:
            with smtplib.SMTP(smtp_config['server'], smtp_config['port']) as server:
                server.starttls()
                server.login(smtp_config['username'], smtp_config['password'])
                server.send_message(msg)
            print(f"  Notification sent to {smtp_config['to']}")
        except Exception as e:
            print(f"  Email error: {e}")

    def trigger_patch(self, finding):
        """Trigger patch management system"""

        # Example: Ansible playbook execution
        print(f"Triggering patch for {finding['host']}")

        cmd = [
            'ansible-playbook',
            '/etc/ansible/playbooks/patch-ssh.yml',
            '-i', f"{finding['host']},"
        ]

        try:
            subprocess.run(cmd, check=True)
            print(f"  Patch playbook executed")
        except subprocess.CalledProcessError as e:
            print(f"  Error: {e}")

if __name__ == '__main__':
    import sys
    if len(sys.argv) != 3:
        print("Usage: remediation.py <config.json> <scan.json>")
        sys.exit(1)

    orchestrator = RemediationOrchestrator(sys.argv[1])
    findings = orchestrator.analyze_results(sys.argv[2])

    print(f"Found {len(findings)} policy violations")

    if findings:
        orchestrator.remediate(findings)
```

**Configuration (config.json):**
```json
{
  "smtp": {
    "server": "smtp.example.com",
    "port": 587,
    "username": "alerts@example.com",
    "password": "YOUR_PASSWORD",
    "from": "security-automation@example.com",
    "to": ["security-team@example.com", "ops-team@example.com"]
  },
  "remediation": {
    "auto_block": true,
    "auto_patch": false,
    "notify_on_severity": ["high", "critical"]
  }
}
```

### Continuous Monitoring Dashboard

**Real-Time Monitoring with Flask:**
```python
#!/usr/bin/env python3
"""
Real-time network monitoring dashboard.
"""
from flask import Flask, render_template, jsonify
import subprocess
import json
import sqlite3
from datetime import datetime, timedelta

app = Flask(__name__)
DB_FILE = '/var/lib/prtip/monitor.db'

@app.route('/')
def dashboard():
    """Main dashboard page"""
    return render_template('dashboard.html')

@app.route('/api/scans/recent')
def recent_scans():
    """Get recent scan results"""

    conn = sqlite3.connect(DB_FILE)
    cur = conn.cursor()

    # Last 24 hours
    cutoff = datetime.utcnow() - timedelta(hours=24)

    cur.execute("""
        SELECT
            s.id,
            s.start_time,
            s.end_time,
            COUNT(sr.id) as total_results,
            SUM(CASE WHEN sr.state = 'open' THEN 1 ELSE 0 END) as open_ports
        FROM scans s
        LEFT JOIN scan_results sr ON s.id = sr.scan_id
        WHERE s.start_time > ?
        GROUP BY s.id
        ORDER BY s.start_time DESC
    """, (cutoff,))

    scans = []
    for row in cur.fetchall():
        scans.append({
            'id': row[0],
            'start_time': row[1],
            'end_time': row[2],
            'total_results': row[3],
            'open_ports': row[4]
        })

    conn.close()
    return jsonify(scans)

@app.route('/api/changes')
def network_changes():
    """Detect network changes"""

    conn = sqlite3.connect(DB_FILE)
    cur = conn.cursor()

    # Compare last two scans
    cur.execute("SELECT id FROM scans ORDER BY start_time DESC LIMIT 2")
    scan_ids = [row[0] for row in cur.fetchall()]

    if len(scan_ids) < 2:
        return jsonify({'changes': []})

    # New open ports
    cur.execute("""
        SELECT DISTINCT sr2.target_ip, sr2.port, sr2.service
        FROM scan_results sr2
        WHERE sr2.scan_id = ?
          AND sr2.state = 'open'
          AND NOT EXISTS (
              SELECT 1 FROM scan_results sr1
              WHERE sr1.scan_id = ?
                AND sr1.target_ip = sr2.target_ip
                AND sr1.port = sr2.port
                AND sr1.state = 'open'
          )
    """, (scan_ids[0], scan_ids[1]))

    changes = []
    for row in cur.fetchall():
        changes.append({
            'type': 'new_open_port',
            'ip': row[0],
            'port': row[1],
            'service': row[2]
        })

    conn.close()
    return jsonify({'changes': changes})

@app.route('/api/trigger-scan', methods=['POST'])
def trigger_scan():
    """Trigger on-demand scan"""

    # Run scan in background
    subprocess.Popen([
        'prtip', '-sS', '-p', '1-1000',
        '192.168.1.0/24',
        '--with-db', '--database', DB_FILE
    ])

    return jsonify({'status': 'scan_started'})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000, debug=False)
```

---

## Systemd Timers

Modern alternative to cron with better logging and service management.

### Timer Unit File

**/etc/systemd/system/prtip-scan.timer:**
```ini
[Unit]
Description=ProRT-IP Network Scan Timer
Documentation=https://github.com/doublegate/ProRT-IP

[Timer]
# Run daily at 2 AM
OnCalendar=*-*-* 02:00:00

# Run 5 minutes after boot
OnBootSec=5min

# Randomize start time by up to 1 hour
RandomizedDelaySec=1h

# Persistent (run missed timers after boot)
Persistent=true

[Install]
WantedBy=timers.target
```

### Service Unit File

**/etc/systemd/system/prtip-scan.service:**
```ini
[Unit]
Description=ProRT-IP Network Scan
Documentation=https://github.com/doublegate/ProRT-IP
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
User=prtip
Group=prtip

# Environment
Environment="RESULTS_DIR=/var/lib/prtip"
Environment="TARGET=192.168.1.0/24"

# Pre-execution
ExecStartPre=/bin/mkdir -p ${RESULTS_DIR}

# Main scan
ExecStart=/usr/local/bin/prtip \
  -sS -sV -p 1-1000 \
  ${TARGET} \
  --with-db --database ${RESULTS_DIR}/scans.db \
  -oJ ${RESULTS_DIR}/scan-%%Y%%m%%d-%%H%%M%%S.json

# Post-execution cleanup (remove old scans)
ExecStartPost=/usr/bin/find ${RESULTS_DIR} -name "scan-*.json" -mtime +30 -delete

# Resource limits
MemoryMax=2G
CPUQuota=50%

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=prtip-scan

[Install]
WantedBy=multi-user.target
```

### Enable and Manage Timer

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable timer (start on boot)
sudo systemctl enable prtip-scan.timer

# Start timer immediately
sudo systemctl start prtip-scan.timer

# Check timer status
sudo systemctl status prtip-scan.timer

# List all timers
systemctl list-timers --all

# View logs
sudo journalctl -u prtip-scan.service -f

# Manually trigger service (for testing)
sudo systemctl start prtip-scan.service
```

---

## Orchestration Tools

### Ansible Playbook

**Distributed Scanning Orchestration:**

**playbook.yml:**
```yaml
---
- name: Automated ProRT-IP Network Scanning
  hosts: scanners
  become: yes
  vars:
    prtip_version: "0.5.2"
    results_dir: "/var/lib/prtip"
    target_networks:
      - "192.168.1.0/24"
      - "192.168.2.0/24"
      - "10.0.0.0/16"

  tasks:
    - name: Install ProRT-IP
      get_url:
        url: "https://github.com/doublegate/ProRT-IP/releases/download/v{{ prtip_version }}/prtip-linux-x86_64"
        dest: /usr/local/bin/prtip
        mode: '0755'

    - name: Create results directory
      file:
        path: "{{ results_dir }}"
        state: directory
        mode: '0755'

    - name: Run parallel scans
      shell: |
        prtip -sS -p 1-1000 "{{ item }}" \
          --with-db --database "{{ results_dir }}/scans.db" \
          -oJ "{{ results_dir }}/scan-{{ item | replace('/', '-') }}-{{ ansible_date_time.epoch }}.json"
      loop: "{{ target_networks }}"
      async: 3600  # 1 hour timeout
      poll: 0
      register: scan_jobs

    - name: Wait for scans to complete
      async_status:
        jid: "{{ item.ansible_job_id }}"
      register: scan_results
      until: scan_results.finished
      retries: 60
      delay: 60
      loop: "{{ scan_jobs.results }}"

    - name: Fetch scan results
      fetch:
        src: "{{ results_dir }}/scan-{{ item | replace('/', '-') }}-{{ ansible_date_time.epoch }}.json"
        dest: "./collected-scans/"
        flat: yes
      loop: "{{ target_networks }}"

    - name: Send notification
      mail:
        host: smtp.example.com
        port: 587
        username: alerts@example.com
        password: "{{ smtp_password }}"
        to: security-team@example.com
        subject: "Automated Scan Complete - {{ ansible_date_time.iso8601 }}"
        body: "Network scans completed successfully. Results attached."
```

**Run playbook:**
```bash
ansible-playbook -i inventory.ini playbook.yml
```

### Terraform (Infrastructure as Code)

**Deploy Scanning Infrastructure:**

**main.tf:**
```hcl
terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# Security group for scanner instances
resource "aws_security_group" "prtip_scanner" {
  name        = "prtip-scanner-sg"
  description = "ProRT-IP scanner security group"
  vpc_id      = var.vpc_id

  egress {
    from_port   = 0
    to_port     = 65535
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "prtip-scanner"
  }
}

# IAM role for scanner instances
resource "aws_iam_role" "prtip_scanner" {
  name = "prtip-scanner-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "ec2.amazonaws.com"
      }
    }]
  })
}

# EC2 instance for scanning
resource "aws_instance" "scanner" {
  count         = var.scanner_count
  ami           = var.ami_id
  instance_type = "t3.medium"
  key_name      = var.ssh_key_name

  vpc_security_group_ids = [aws_security_group.prtip_scanner.id]
  subnet_id              = var.subnet_id
  iam_instance_profile   = aws_iam_instance_profile.scanner.name

  user_data = templatefile("${path.module}/user-data.sh", {
    prtip_version = var.prtip_version
    s3_bucket     = aws_s3_bucket.scan_results.bucket
  })

  tags = {
    Name = "prtip-scanner-${count.index + 1}"
    Role = "security-scanner"
  }
}

# S3 bucket for scan results
resource "aws_s3_bucket" "scan_results" {
  bucket = "prtip-scan-results-${var.environment}"

  tags = {
    Name = "prtip-scan-results"
  }
}

# CloudWatch log group
resource "aws_cloudwatch_log_group" "prtip_scans" {
  name              = "/aws/prtip/scans"
  retention_in_days = 30
}

# EventBridge rule for scheduled scans
resource "aws_cloudwatch_event_rule" "daily_scan" {
  name                = "prtip-daily-scan"
  description         = "Trigger daily network scan"
  schedule_expression = "cron(0 2 * * ? *)"  # 2 AM daily
}

resource "aws_cloudwatch_event_target" "scanner_instance" {
  rule      = aws_cloudwatch_event_rule.daily_scan.name
  target_id = "TriggerScan"
  arn       = aws_ssm_document.run_scan.arn
  role_arn  = aws_iam_role.eventbridge.arn
}

output "scanner_ips" {
  value = aws_instance.scanner[*].public_ip
}
```

**user-data.sh:**
```bash
#!/bin/bash
set -e

# Install ProRT-IP
wget https://github.com/doublegate/ProRT-IP/releases/download/v${prtip_version}/prtip-linux-x86_64
chmod +x prtip-linux-x86_64
mv prtip-linux-x86_64 /usr/local/bin/prtip

# Install AWS CLI
apt-get update
apt-get install -y awscli jq

# Create scan script
cat > /usr/local/bin/automated-scan.sh << 'EOF'
#!/bin/bash
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RESULTS_FILE="/tmp/scan-${TIMESTAMP}.json"

# Run scan
prtip -sS -p 1-1000 ${TARGET_NETWORK} -oJ "${RESULTS_FILE}"

# Upload to S3
aws s3 cp "${RESULTS_FILE}" "s3://${s3_bucket}/scans/scan-${TIMESTAMP}.json"

# Send metrics to CloudWatch
OPEN_PORTS=$(jq '[.hosts[].ports[] | select(.state == "Open")] | length' "${RESULTS_FILE}")
aws cloudwatch put-metric-data \
  --namespace ProRT-IP \
  --metric-name OpenPorts \
  --value ${OPEN_PORTS} \
  --timestamp $(date -u +%Y-%m-%dT%H:%M:%S)

# Cleanup
rm "${RESULTS_FILE}"
EOF

chmod +x /usr/local/bin/automated-scan.sh
```

---

## Monitoring Integration

### Prometheus Exporter

**Export Scan Metrics to Prometheus:**

**prtip_exporter.py:**
```python
#!/usr/bin/env python3
"""
Prometheus exporter for ProRT-IP scan metrics.
"""
from prometheus_client import start_http_server, Gauge, Counter
import sqlite3
import time
from datetime import datetime, timedelta

# Metrics
OPEN_PORTS = Gauge('prtip_open_ports_total', 'Total number of open ports')
SCANS_TOTAL = Counter('prtip_scans_total', 'Total number of scans completed')
SCAN_DURATION = Gauge('prtip_scan_duration_seconds', 'Last scan duration')
HOSTS_SCANNED = Gauge('prtip_hosts_scanned_total', 'Total hosts scanned')
SERVICES_DETECTED = Gauge('prtip_services_detected_total', 'Services with version detected')

DB_FILE = '/var/lib/prtip/scans.db'

def collect_metrics():
    """Collect metrics from ProRT-IP database"""

    conn = sqlite3.connect(DB_FILE)
    cur = conn.cursor()

    # Last scan
    cur.execute("""
        SELECT id, start_time, end_time
        FROM scans
        ORDER BY start_time DESC
        LIMIT 1
    """)
    last_scan = cur.fetchone()

    if last_scan:
        scan_id, start_time, end_time = last_scan

        # Calculate duration
        start = datetime.fromisoformat(start_time)
        end = datetime.fromisoformat(end_time)
        duration = (end - start).total_seconds()
        SCAN_DURATION.set(duration)

        # Count open ports
        cur.execute("""
            SELECT COUNT(*)
            FROM scan_results
            WHERE scan_id = ? AND state = 'open'
        """, (scan_id,))
        OPEN_PORTS.set(cur.fetchone()[0])

        # Count hosts
        cur.execute("""
            SELECT COUNT(DISTINCT target_ip)
            FROM scan_results
            WHERE scan_id = ?
        """, (scan_id,))
        HOSTS_SCANNED.set(cur.fetchone()[0])

        # Count services with versions
        cur.execute("""
            SELECT COUNT(*)
            FROM scan_results
            WHERE scan_id = ? AND version IS NOT NULL
        """, (scan_id,))
        SERVICES_DETECTED.set(cur.fetchone()[0])

        SCANS_TOTAL.inc()

    conn.close()

if __name__ == '__main__':
    # Start HTTP server for Prometheus to scrape
    start_http_server(9090)
    print("ProRT-IP Prometheus exporter running on :9090")

    # Collect metrics every 60 seconds
    while True:
        try:
            collect_metrics()
        except Exception as e:
            print(f"Error collecting metrics: {e}")

        time.sleep(60)
```

**Prometheus Configuration (prometheus.yml):**
```yaml
scrape_configs:
  - job_name: 'prtip-exporter'
    scrape_interval: 60s
    static_configs:
      - targets: ['localhost:9090']
        labels:
          instance: 'prtip-scanner-01'
```

**Run exporter:**
```bash
python3 prtip_exporter.py &
```

### Grafana Dashboard

**Dashboard JSON Configuration:**
```json
{
  "dashboard": {
    "title": "ProRT-IP Network Monitoring",
    "panels": [
      {
        "title": "Open Ports Over Time",
        "type": "graph",
        "targets": [
          {
            "expr": "prtip_open_ports_total",
            "legendFormat": "Open Ports"
          }
        ]
      },
      {
        "title": "Scan Duration",
        "type": "graph",
        "targets": [
          {
            "expr": "prtip_scan_duration_seconds",
            "legendFormat": "Duration (seconds)"
          }
        ]
      },
      {
        "title": "Total Scans",
        "type": "stat",
        "targets": [
          {
            "expr": "prtip_scans_total"
          }
        ]
      },
      {
        "title": "Services Detected",
        "type": "stat",
        "targets": [
          {
            "expr": "prtip_services_detected_total"
          }
        ]
      }
    ]
  }
}
```

### Alerting Rules

**Prometheus Alerting (alerts.yml):**
```yaml
groups:
  - name: prtip_alerts
    interval: 60s
    rules:
      - alert: UnexpectedOpenPorts
        expr: prtip_open_ports_total > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High number of open ports detected"
          description: "{{ $value }} open ports found (threshold: 100)"

      - alert: NewServicesDetected
        expr: increase(prtip_services_detected_total[1h]) > 10
        for: 5m
        labels:
          severity: info
        annotations:
          summary: "New services detected"
          description: "{{ $value }} new services detected in last hour"

      - alert: ScanDurationHigh
        expr: prtip_scan_duration_seconds > 3600
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Scan taking too long"
          description: "Last scan took {{ $value }}s (threshold: 3600s)"
```

---

## Best Practices

### 1. Error Handling and Retry Logic

**Robust Scan Wrapper:**
```bash
#!/bin/bash
# robust-scan.sh

MAX_RETRIES=3
RETRY_DELAY=60

run_scan() {
  local target=$1
  local output=$2
  local attempt=1

  while [ $attempt -le $MAX_RETRIES ]; do
    echo "Attempt $attempt of $MAX_RETRIES..."

    if prtip -sS -p 1-1000 "${target}" -oJ "${output}" 2>&1 | tee scan.log; then
      echo "Scan succeeded"
      return 0
    else
      exit_code=$?
      echo "Scan failed with exit code $exit_code"

      if [ $attempt -lt $MAX_RETRIES ]; then
        echo "Retrying in ${RETRY_DELAY}s..."
        sleep $RETRY_DELAY
      fi

      attempt=$((attempt + 1))
    fi
  done

  echo "All retry attempts exhausted"
  return 1
}

# Usage
if run_scan "192.168.1.0/24" "results.json"; then
  echo "Scan completed successfully"
else
  echo "Scan failed after $MAX_RETRIES attempts" >&2
  # Send alert
  mail -s "Scan Failed" admin@example.com < scan.log
  exit 1
fi
```

### 2. Idempotency

**Idempotent Scan Script:**
```bash
#!/bin/bash
# idempotent-scan.sh

LOCKFILE="/var/run/prtip-scan.lock"
RESULTS_DIR="/var/lib/prtip"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Acquire lock
exec 200>"$LOCKFILE"
flock -n 200 || {
  echo "Another scan is already running"
  exit 1
}

# Cleanup function
cleanup() {
  rm -f "$LOCKFILE"
}
trap cleanup EXIT

# Check if recent scan exists (within last hour)
RECENT_SCAN=$(find "$RESULTS_DIR" -name "scan-*.json" -mmin -60 | head -n 1)

if [ -n "$RECENT_SCAN" ]; then
  echo "Recent scan found: $RECENT_SCAN"
  echo "Skipping duplicate scan"
  exit 0
fi

# Run scan (guaranteed to run only once)
echo "Running scan..."
prtip -sS -p 1-1000 192.168.1.0/24 \
  -oJ "$RESULTS_DIR/scan-$TIMESTAMP.json" \
  --with-db --database "$RESULTS_DIR/scans.db"

echo "Scan complete: $RESULTS_DIR/scan-$TIMESTAMP.json"
```

### 3. Comprehensive Logging

**Structured Logging:**
```python
#!/usr/bin/env python3
import logging
import logging.handlers
import json
from datetime import datetime

class StructuredLogger:
    def __init__(self, log_file):
        self.logger = logging.getLogger('prtip-automation')
        self.logger.setLevel(logging.DEBUG)

        # Syslog handler
        syslog = logging.handlers.SysLogHandler(address='/dev/log')
        syslog.setLevel(logging.INFO)

        # File handler (JSON structured logs)
        file_handler = logging.FileHandler(log_file)
        file_handler.setLevel(logging.DEBUG)

        # Custom JSON formatter
        class JSONFormatter(logging.Formatter):
            def format(self, record):
                log_entry = {
                    'timestamp': datetime.utcnow().isoformat(),
                    'level': record.levelname,
                    'logger': record.name,
                    'message': record.getMessage(),
                    'module': record.module,
                    'function': record.funcName,
                    'line': record.lineno
                }

                if record.exc_info:
                    log_entry['exception'] = self.formatException(record.exc_info)

                return json.dumps(log_entry)

        file_handler.setFormatter(JSONFormatter())

        self.logger.addHandler(syslog)
        self.logger.addHandler(file_handler)

    def log_scan_start(self, target, scan_type):
        self.logger.info(f"Scan started: {target} (type: {scan_type})")

    def log_scan_complete(self, target, duration, results):
        self.logger.info(
            f"Scan complete: {target} - "
            f"duration={duration:.2f}s - "
            f"results={results}"
        )

    def log_error(self, message, exception=None):
        self.logger.error(message, exc_info=exception)

# Usage
logger = StructuredLogger('/var/log/prtip/automation.json.log')
logger.log_scan_start('192.168.1.0/24', 'syn')
```

### 4. Resource Management

**Control Resource Usage:**
```bash
#!/bin/bash
# resource-limited-scan.sh

# Limit CPU usage (50% of one core)
cpulimit -l 50 -e prtip &

# Set memory limit (2 GB)
ulimit -v 2097152  # 2 GB in KB

# Set maximum number of open files
ulimit -n 10000

# Set nice level (lower priority)
renice -n 10 $$

# Run scan
prtip -sS -p 1-1000 192.168.1.0/24 -oJ results.json

# Cleanup cpulimit
pkill cpulimit
```

### 5. Security Considerations

**Secure Automation Configuration:**
```bash
#!/bin/bash
# secure-automation.sh

# 1. Use dedicated service account
if [ "$USER" != "prtip-scanner" ]; then
  echo "ERROR: Must run as prtip-scanner user"
  exit 1
fi

# 2. Restrict file permissions
umask 077  # Files: 600, Directories: 700

# 3. Encrypt sensitive data
encrypt_config() {
  gpg --encrypt --recipient security@example.com config.json
  rm config.json
  echo "Configuration encrypted"
}

# 4. Validate inputs
validate_target() {
  local target=$1

  # Check if valid IP/CIDR
  if ! [[ "$target" =~ ^[0-9./]+$ ]]; then
    echo "ERROR: Invalid target format"
    return 1
  fi

  # Check against allowlist
  if ! grep -q "^$target$" /etc/prtip/allowed-targets.txt; then
    echo "ERROR: Target not in allowlist"
    return 1
  fi

  return 0
}

# 5. Audit logging
log_audit() {
  local action=$1
  logger -t prtip-audit -p auth.info "User=$USER Action=$action Target=$TARGET"
}

# Example usage
TARGET="192.168.1.0/24"

if validate_target "$TARGET"; then
  log_audit "scan_start"
  prtip -sS -p 1-1000 "$TARGET" -oJ results.json
  log_audit "scan_complete"
else
  log_audit "scan_blocked"
  exit 1
fi
```

---

## See Also

- **[Integration](./integration.md)** - CI/CD pipelines, SIEM integration, webhook automation
- **[Database Usage](./database-usage.md)** - Persistent storage for automated scan results
- **[Large-Scale Scanning](./large-scale-scanning.md)** - Performance optimization for automation workloads
- **[Distributed Scanning](./distributed-scanning.md)** - Multi-instance coordination for automated workflows
- **[Plugin System](../features/plugin-system.md)** - Custom plugins for specialized automation
- **[Output Formats](../reference/output-formats.md)** - JSON, XML parsing for automation scripts
- **[User Guide: Basic Usage](../user-guide/basic-usage.md)** - Command-line examples for scripting
- **[Examples Gallery](../34-EXAMPLES-GALLERY.md)** - 65 runnable examples for automation patterns

**External Resources:**
- **Cron Documentation**: https://man7.org/linux/man-pages/man5/crontab.5.html
- **Systemd Timers**: https://www.freedesktop.org/software/systemd/man/systemd.timer.html
- **Ansible Docs**: https://docs.ansible.com/ansible/latest/
- **Terraform AWS Provider**: https://registry.terraform.io/providers/hashicorp/aws/latest/docs
- **Prometheus Exporters**: https://prometheus.io/docs/instrumenting/exporters/
- **Grafana Dashboards**: https://grafana.com/docs/grafana/latest/dashboards/

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
