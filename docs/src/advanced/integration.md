# Integration

Integrate ProRT-IP with external systems and automation workflows.

## What is ProRT-IP Integration?

**Integration** enables ProRT-IP to work seamlessly with existing security infrastructure, automation tools, and monitoring platforms. This transforms ProRT-IP from a standalone scanner into a component of your security ecosystem.

**ProRT-IP Integration Capabilities:**
- **Programmatic API** - Embed ProRT-IP as a Rust library in your applications
- **CI/CD pipelines** - Automated security scanning in build workflows
- **SIEM platforms** - Real-time event forwarding to Splunk, ELK, QRadar
- **Vulnerability management** - Integration with Nessus, OpenVAS, Rapid7
- **Network monitoring** - Feed data to Nagios, Zabbix, Prometheus
- **Webhook notifications** - Real-time alerts via HTTP callbacks
- **Database storage** - Persistent results in PostgreSQL, MySQL, SQLite, ClickHouse
- **Cloud platforms** - AWS Security Hub, Azure Sentinel, GCP Security Command Center
- **Custom plugins** - Lua-based extensions for specialized integrations

**Use Cases:**
- **DevSecOps** - Shift-left security scanning in CI/CD pipelines
- **SOC Automation** - Automated threat hunting and incident response
- **Compliance Reporting** - Continuous compliance validation with audit trails
- **Network Inventory** - Maintain up-to-date asset databases
- **Vulnerability Correlation** - Match scan results to CVE databases
- **Threat Intelligence** - Enrich scan data with IOC feeds

---

## Programmatic API Integration

### Using ProRT-IP as a Rust Library

Embed ProRT-IP scanner directly in your Rust applications.

#### Adding Dependency

**Cargo.toml:**
```toml
[dependencies]
prtip-core = "0.5"
prtip-net = "0.5"
prtip-detect = "0.5"
tokio = { version = "1.35", features = ["full"] }
```

#### Basic Scan Execution

```rust
use prtip_core::{Scanner, ScanConfig, ScanType, Target, PortRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure scan
    let config = ScanConfig {
        targets: vec!["192.168.1.0/24".parse()?],
        ports: PortRange::new(1, 1000),
        scan_type: ScanType::Syn,
        service_detection: true,
        os_detection: true,
        ..Default::default()
    };

    // Execute scan
    let scanner = Scanner::new(config)?;
    let report = scanner.execute().await?;

    // Process results
    println!("Scanned {} hosts in {:?}",
        report.hosts.len(),
        report.duration());

    for host in &report.hosts {
        println!("Host: {}", host.ip);
        for port in &host.ports {
            if port.state == PortState::Open {
                println!("  Port {}: {} ({})",
                    port.port,
                    port.service.as_ref().map(|s| s.name.as_str()).unwrap_or("unknown"),
                    port.state);
            }
        }
    }

    Ok(())
}
```

#### Progress Tracking

```rust
use prtip_core::{Scanner, ScanProgress};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = Scanner::new(config)?;

    // Execute with progress callback
    let report = scanner.execute_with_progress(|progress: ScanProgress| {
        println!("Progress: {:.1}% ({}/{} hosts)",
            progress.percentage(),
            progress.completed,
            progress.total);

        if let Some(eta) = progress.eta() {
            println!("ETA: {:?}", eta);
        }
    }).await?;

    Ok(())
}
```

#### Scan Control (Pause/Resume/Stop)

```rust
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = Arc::new(Scanner::new(config)?);
    let scanner_clone = Arc::clone(&scanner);

    // Spawn scan in background
    let scan_handle = tokio::spawn(async move {
        scanner_clone.execute().await
    });

    // Wait 10 seconds, then pause
    sleep(Duration::from_secs(10)).await;
    scanner.pause()?;
    println!("Scan paused");

    // Resume after 5 seconds
    sleep(Duration::from_secs(5)).await;
    scanner.resume()?;
    println!("Scan resumed");

    // Wait for completion
    let report = scan_handle.await??;
    println!("Scan complete: {} results", report.hosts.len());

    Ok(())
}
```

#### Custom Service Detection

```rust
use prtip_detect::{ServiceDetector, ServiceInfo};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut detector = ServiceDetector::new(7); // Intensity 7
    detector.load_probes("probes/service-probes.txt")?;

    let target: SocketAddr = "192.168.1.1:80".parse()?;

    if let Some(service) = detector.detect(target).await? {
        println!("Service: {} {}",
            service.name,
            service.version.unwrap_or_default());

        if let Some(cpe) = service.cpe {
            println!("CPE: {}", cpe);
        }
    }

    Ok(())
}
```

---

## CI/CD Integration

### GitHub Actions

Automated scanning on every push/pull request.

#### Workflow File (.github/workflows/security-scan.yml)

```yaml
name: Security Scan

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  network-scan:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install ProRT-IP
        run: |
          wget https://github.com/doublegate/ProRT-IP/releases/latest/download/prtip-linux-x86_64
          chmod +x prtip-linux-x86_64
          sudo mv prtip-linux-x86_64 /usr/local/bin/prtip

      - name: Scan staging environment
        run: |
          sudo prtip -sS -p 1-1000 \
            staging.example.com \
            -oJ scan-results.json \
            -oX scan-results.xml

      - name: Parse results
        id: parse
        run: |
          # Count open ports
          OPEN_PORTS=$(jq '[.hosts[].ports[] | select(.state == "Open")] | length' scan-results.json)
          echo "open_ports=$OPEN_PORTS" >> $GITHUB_OUTPUT

          # Check for unexpected open ports
          UNEXPECTED=$(jq '[.hosts[].ports[] | select(.state == "Open" and (.port | IN(80, 443) | not))] | length' scan-results.json)
          echo "unexpected_ports=$UNEXPECTED" >> $GITHUB_OUTPUT

      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: scan-results
          path: |
            scan-results.json
            scan-results.xml

      - name: Comment on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const openPorts = ${{ steps.parse.outputs.open_ports }};
            const unexpected = ${{ steps.parse.outputs.unexpected_ports }};

            const body = `## Security Scan Results

            - **Open Ports**: ${openPorts}
            - **Unexpected Open Ports**: ${unexpected}

            ${unexpected > 0 ? '⚠️ **Warning**: Unexpected ports detected!' : '✅ All ports expected'}
            `;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });

      - name: Fail on unexpected ports
        if: steps.parse.outputs.unexpected_ports > 0
        run: |
          echo "ERROR: Found ${{ steps.parse.outputs.unexpected_ports }} unexpected open ports"
          exit 1
```

#### Advanced: Multi-Environment Scanning

```yaml
jobs:
  scan-matrix:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        environment:
          - name: staging
            target: staging.example.com
            allowed_ports: [80, 443, 22]
          - name: production
            target: prod.example.com
            allowed_ports: [80, 443]

    steps:
      - name: Scan ${{ matrix.environment.name }}
        run: |
          sudo prtip -sS -sV -p 1-1000 \
            ${{ matrix.environment.target }} \
            -oJ ${{ matrix.environment.name }}-results.json

      - name: Validate ports
        run: |
          python3 validate-ports.py \
            --results ${{ matrix.environment.name }}-results.json \
            --allowed "${{ join(matrix.environment.allowed_ports, ',') }}"
```

**validate-ports.py:**
```python
import json
import sys
import argparse

parser = argparse.ArgumentParser()
parser.add_argument('--results', required=True)
parser.add_argument('--allowed', required=True)
args = parser.parse_args()

allowed_ports = set(map(int, args.allowed.split(',')))

with open(args.results) as f:
    data = json.load(f)

unexpected = []
for host in data['hosts']:
    for port in host.get('ports', []):
        if port['state'] == 'Open' and port['port'] not in allowed_ports:
            unexpected.append(port['port'])

if unexpected:
    print(f"ERROR: Unexpected open ports: {unexpected}")
    sys.exit(1)
else:
    print("✅ All open ports are allowed")
```

### GitLab CI

**.gitlab-ci.yml:**
```yaml
stages:
  - security

network-scan:
  stage: security
  image: ubuntu:22.04
  before_script:
    - apt-get update
    - apt-get install -y wget jq
    - wget https://github.com/doublegate/ProRT-IP/releases/latest/download/prtip-linux-x86_64
    - chmod +x prtip-linux-x86_64
    - mv prtip-linux-x86_64 /usr/local/bin/prtip

  script:
    - prtip -sS -p 1-1000 $CI_ENVIRONMENT_URL -oJ scan-results.json
    - |
      UNEXPECTED=$(jq '[.hosts[].ports[] | select(.state == "Open" and (.port | IN(80, 443) | not))] | length' scan-results.json)
      if [ "$UNEXPECTED" -gt 0 ]; then
        echo "ERROR: Found $UNEXPECTED unexpected open ports"
        exit 1
      fi

  artifacts:
    reports:
      junit: scan-results.xml
    paths:
      - scan-results.json

  only:
    - merge_requests
    - main
```

### Jenkins Pipeline

**Jenkinsfile:**
```groovy
pipeline {
    agent any

    parameters {
        string(name: 'TARGET', defaultValue: 'staging.example.com', description: 'Target to scan')
        choice(name: 'SCAN_TYPE', choices: ['quick', 'full'], description: 'Scan type')
    }

    environment {
        PRTIP_VERSION = '0.5.2'
        SCAN_RESULTS = 'scan-results.json'
    }

    stages {
        stage('Install ProRT-IP') {
            steps {
                sh '''
                    wget https://github.com/doublegate/ProRT-IP/releases/download/v${PRTIP_VERSION}/prtip-linux-x86_64
                    chmod +x prtip-linux-x86_64
                    sudo mv prtip-linux-x86_64 /usr/local/bin/prtip
                '''
            }
        }

        stage('Network Scan') {
            steps {
                script {
                    def ports = params.SCAN_TYPE == 'full' ? '1-65535' : '1-1000'

                    sh """
                        sudo prtip -sS -sV -p ${ports} \
                            ${params.TARGET} \
                            -oJ ${env.SCAN_RESULTS} \
                            -oX scan-results.xml
                    """
                }
            }
        }

        stage('Parse Results') {
            steps {
                script {
                    def results = readJSON file: env.SCAN_RESULTS
                    def openPorts = results.hosts.collectMany { it.ports }
                        .findAll { it.state == 'Open' }
                        .collect { it.port }

                    echo "Open ports: ${openPorts}"

                    def unexpected = openPorts.findAll { !(it in [80, 443, 22]) }
                    if (unexpected) {
                        error("Unexpected open ports: ${unexpected}")
                    }
                }
            }
        }

        stage('Archive Results') {
            steps {
                archiveArtifacts artifacts: 'scan-results.*', fingerprint: true
                junit 'scan-results.xml'
            }
        }
    }

    post {
        failure {
            emailext(
                subject: "Network Scan Failed: ${params.TARGET}",
                body: "Scan of ${params.TARGET} failed. Check console output.",
                to: 'security@example.com'
            )
        }
    }
}
```

---

## SIEM Integration

### Splunk Integration

Forward ProRT-IP scan results to Splunk for correlation and alerting.

#### Forwarder Script (Python)

**splunk_forwarder.py:**
```python
#!/usr/bin/env python3
import json
import requests
import argparse
from datetime import datetime

def send_to_splunk(results_file, hec_url, hec_token):
    """Send ProRT-IP results to Splunk HTTP Event Collector"""

    with open(results_file) as f:
        scan_data = json.load(f)

    headers = {
        'Authorization': f'Splunk {hec_token}',
        'Content-Type': 'application/json'
    }

    events = []
    for host in scan_data['hosts']:
        for port in host.get('ports', []):
            event = {
                'time': datetime.utcnow().timestamp(),
                'sourcetype': 'prtip:scan',
                'source': 'prtip',
                'event': {
                    'scan_id': scan_data.get('scan_id'),
                    'target_ip': host['ip'],
                    'port': port['port'],
                    'protocol': port['protocol'],
                    'state': port['state'],
                    'service': port.get('service', {}).get('name'),
                    'version': port.get('service', {}).get('version'),
                    'banner': port.get('banner'),
                }
            }
            events.append(event)

    # Send in batches of 100
    for i in range(0, len(events), 100):
        batch = events[i:i+100]
        payload = '\n'.join(json.dumps({'event': e['event'], 'time': e['time']})
                           for e in batch)

        response = requests.post(
            hec_url,
            headers=headers,
            data=payload,
            verify=True
        )

        if response.status_code != 200:
            print(f"Error: {response.status_code} - {response.text}")
        else:
            print(f"Sent batch {i//100 + 1}: {len(batch)} events")

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--results', required=True, help='Scan results JSON file')
    parser.add_argument('--hec-url', required=True, help='Splunk HEC URL')
    parser.add_argument('--hec-token', required=True, help='Splunk HEC token')
    args = parser.parse_args()

    send_to_splunk(args.results, args.hec_url, args.hec_token)
```

**Usage:**
```bash
# Run scan
sudo prtip -sS -p 1-1000 10.0.0.0/24 -oJ scan-results.json

# Forward to Splunk
python3 splunk_forwarder.py \
    --results scan-results.json \
    --hec-url https://splunk.example.com:8088/services/collector \
    --hec-token YOUR_HEC_TOKEN
```

#### Splunk Search Queries

**Find new open ports:**
```splunk
sourcetype=prtip:scan state=Open
| stats latest(_time) as last_seen by target_ip, port, service
| where last_seen > relative_time(now(), "-1d")
```

**Alert on unexpected services:**
```splunk
sourcetype=prtip:scan state=Open service!=http service!=https service!=ssh
| stats count by target_ip, port, service
| where count > 0
```

### ELK Stack (Elasticsearch, Logstash, Kibana)

#### Logstash Configuration

**logstash-prtip.conf:**
```ruby
input {
  file {
    path => "/var/log/prtip/scan-*.json"
    start_position => "beginning"
    codec => "json"
  }
}

filter {
  if [hosts] {
    split {
      field => "hosts"
    }

    split {
      field => "[hosts][ports]"
    }

    mutate {
      add_field => {
        "target_ip" => "%{[hosts][ip]}"
        "port" => "%{[hosts][ports][port]}"
        "state" => "%{[hosts][ports][state]}"
        "service" => "%{[hosts][ports][service][name]}"
      }

      remove_field => ["hosts"]
    }
  }
}

output {
  elasticsearch {
    hosts => ["http://elasticsearch:9200"]
    index => "prtip-scans-%{+YYYY.MM.dd}"
    document_type => "_doc"
  }

  stdout {
    codec => rubydebug
  }
}
```

**Run Logstash:**
```bash
/usr/share/logstash/bin/logstash -f /etc/logstash/conf.d/logstash-prtip.conf
```

#### Kibana Dashboard

**Create index pattern:**
```
Index pattern: prtip-scans-*
Time field: @timestamp
```

**Visualizations:**
1. **Open Ports Over Time** - Line chart, count of state:Open
2. **Top Services** - Pie chart, terms aggregation on service field
3. **Port Distribution** - Bar chart, terms aggregation on port field
4. **Network Map** - Coordinate map, geolocation of target IPs

### QRadar Integration

**QRadar Custom Property:**
```xml
<CustomProperty name="prtip_scan">
  <Description>ProRT-IP network scan event</Description>
  <Property name="target_ip" type="IP" />
  <Property name="port" type="numeric" />
  <Property name="service" type="AlphaNumeric" />
  <Property name="state" type="AlphaNumeric" />
</CustomProperty>
```

**Syslog Forwarder:**
```bash
# Run scan and send to QRadar via syslog
sudo prtip -sS -p 1-1000 10.0.0.0/24 -oJ scan-results.json

# Parse and forward
jq -c '.hosts[].ports[] | {ip: .target_ip, port: .port, service: .service.name, state: .state}' scan-results.json \
  | while read event; do
      logger -n qradar.example.com -P 514 -t prtip "$event"
    done
```

---

## Vulnerability Management Integration

### Nessus Integration

**Export to Nessus format:**
```python
#!/usr/bin/env python3
import json
import xml.etree.ElementTree as ET
from datetime import datetime

def prtip_to_nessus(prtip_json, output_file):
    """Convert ProRT-IP JSON to Nessus .nessus format"""

    with open(prtip_json) as f:
        data = json.load(f)

    # Create Nessus XML structure
    root = ET.Element('NessusClientData_v2')
    policy = ET.SubElement(root, 'Policy')
    ET.SubElement(policy, 'policyName').text = 'ProRT-IP Scan'

    report = ET.SubElement(root, 'Report', name='ProRT-IP Scan')

    for host in data['hosts']:
        host_properties = ET.SubElement(
            report, 'ReportHost', name=host['ip']
        )

        # Host properties
        ET.SubElement(host_properties, 'HostProperties')
        tag = ET.SubElement(host_properties, 'tag', name='HOST_END')
        tag.text = datetime.utcnow().isoformat()

        # Port findings
        for port_info in host.get('ports', []):
            if port_info['state'] == 'Open':
                item = ET.SubElement(host_properties, 'ReportItem',
                    port=str(port_info['port']),
                    svc_name=port_info.get('service', {}).get('name', 'unknown'),
                    protocol=port_info['protocol'],
                    severity='0',
                    pluginID='0',
                    pluginName='Port Scanner'
                )

                ET.SubElement(item, 'description').text = \
                    f"Port {port_info['port']} is open"

                if 'service' in port_info:
                    ET.SubElement(item, 'plugin_output').text = \
                        f"Service: {port_info['service'].get('name')}\n" \
                        f"Version: {port_info['service'].get('version', 'unknown')}"

    # Write XML
    tree = ET.ElementTree(root)
    tree.write(output_file, encoding='utf-8', xml_declaration=True)
    print(f"Exported to {output_file}")

if __name__ == '__main__':
    import sys
    if len(sys.argv) != 3:
        print("Usage: prtip_to_nessus.py <input.json> <output.nessus>")
        sys.exit(1)

    prtip_to_nessus(sys.argv[1], sys.argv[2])
```

**Usage:**
```bash
sudo prtip -sS -sV -p 1-65535 target.com -oJ scan.json
python3 prtip_to_nessus.py scan.json scan.nessus
# Import scan.nessus into Nessus
```

### OpenVAS Integration

**GMP (Greenbone Management Protocol) integration:**
```python
#!/usr/bin/env python3
from gvm.connections import TLSConnection
from gvm.protocols.gmp import Gmp
from gvm.transforms import EtreeTransform
import json

def import_to_openvas(prtip_json, openvas_host, username, password):
    connection = TLSConnection(hostname=openvas_host)
    transform = EtreeTransform()

    with Gmp(connection, transform=transform) as gmp:
        gmp.authenticate(username, password)

        with open(prtip_json) as f:
            data = json.load(f)

        # Create target from ProRT-IP results
        for host in data['hosts']:
            target_name = f"ProRT-IP: {host['ip']}"

            # Create target
            response = gmp.create_target(
                name=target_name,
                hosts=[host['ip']],
                port_list_id='33d0cd82-57c6-11e1-8ed1-406186ea4fc5'  # All TCP
            )
            target_id = response.get('id')

            # Create task
            task_response = gmp.create_task(
                name=f"Scan {host['ip']}",
                config_id='daba56c8-73ec-11df-a475-002264764cea',  # Full and fast
                target_id=target_id,
                scanner_id='08b69003-5fc2-4037-a479-93b440211c73'  # OpenVAS Default
            )

            print(f"Created task for {host['ip']}: {task_response.get('id')}")

if __name__ == '__main__':
    import sys
    if len(sys.argv) != 5:
        print("Usage: import_to_openvas.py <scan.json> <host> <user> <pass>")
        sys.exit(1)

    import_to_openvas(sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4])
```

---

## Webhook Integration

Real-time notifications via HTTP callbacks.

### Webhook Server Configuration

**webhook-config.json:**
```json
{
  "webhooks": [
    {
      "name": "slack-alerts",
      "url": "https://hooks.slack.com/services/YOUR/WEBHOOK/URL",
      "events": ["port_discovered", "scan_complete"],
      "filter": {
        "min_severity": "medium"
      }
    },
    {
      "name": "security-dashboard",
      "url": "https://api.example.com/security/events",
      "events": ["all"],
      "headers": {
        "Authorization": "Bearer YOUR_API_TOKEN"
      }
    }
  ]
}
```

### Webhook Sender (Python)

**webhook_sender.py:**
```python
#!/usr/bin/env python3
import json
import requests
import argparse
from datetime import datetime

def send_webhook(event_type, payload, webhook_config):
    """Send webhook notification"""

    with open(webhook_config) as f:
        config = json.load(f)

    for webhook in config['webhooks']:
        # Check if this webhook subscribes to this event
        if event_type not in webhook['events'] and 'all' not in webhook['events']:
            continue

        # Apply filters
        if 'filter' in webhook:
            if 'min_severity' in webhook['filter']:
                # Filter logic here
                pass

        # Prepare payload
        notification = {
            'event': event_type,
            'timestamp': datetime.utcnow().isoformat(),
            'data': payload
        }

        # Send webhook
        headers = webhook.get('headers', {})
        headers['Content-Type'] = 'application/json'

        try:
            response = requests.post(
                webhook['url'],
                json=notification,
                headers=headers,
                timeout=10
            )

            if response.status_code == 200:
                print(f"✓ Sent to {webhook['name']}")
            else:
                print(f"✗ Failed to send to {webhook['name']}: {response.status_code}")

        except Exception as e:
            print(f"✗ Error sending to {webhook['name']}: {e}")

def process_scan_results(results_file, webhook_config):
    """Process ProRT-IP results and send webhooks"""

    with open(results_file) as f:
        data = json.load(f)

    # Send port discoveries
    for host in data['hosts']:
        for port in host.get('ports', []):
            if port['state'] == 'Open':
                send_webhook('port_discovered', {
                    'target_ip': host['ip'],
                    'port': port['port'],
                    'service': port.get('service', {}).get('name'),
                    'version': port.get('service', {}).get('version')
                }, webhook_config)

    # Send scan complete
    send_webhook('scan_complete', {
        'total_hosts': len(data['hosts']),
        'duration': data.get('duration'),
        'open_ports': sum(
            1 for h in data['hosts']
            for p in h.get('ports', [])
            if p['state'] == 'Open'
        )
    }, webhook_config)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--results', required=True)
    parser.add_argument('--config', default='webhook-config.json')
    args = parser.parse_args()

    process_scan_results(args.results, args.config)
```

**Usage:**
```bash
# Run scan
sudo prtip -sS -p 1-1000 10.0.0.0/24 -oJ scan.json

# Send webhooks
python3 webhook_sender.py --results scan.json --config webhook-config.json
```

### Slack Integration Example

**slack_notifier.py:**
```python
import requests
import json

def send_slack_alert(webhook_url, target, open_ports):
    """Send Slack notification for scan results"""

    message = {
        "text": f"🔍 Network Scan Complete: {target}",
        "attachments": [
            {
                "color": "warning" if open_ports > 10 else "good",
                "fields": [
                    {
                        "title": "Target",
                        "value": target,
                        "short": True
                    },
                    {
                        "title": "Open Ports",
                        "value": str(open_ports),
                        "short": True
                    }
                ]
            }
        ]
    }

    requests.post(webhook_url, json=message)
```

---

## Database Integration

### PostgreSQL Storage

**Schema:**
```sql
-- Create schema
CREATE SCHEMA prtip;

-- Scans table
CREATE TABLE prtip.scans (
    scan_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    scan_type VARCHAR(20),
    status VARCHAR(20),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Scan results table
CREATE TABLE prtip.scan_results (
    result_id BIGSERIAL PRIMARY KEY,
    scan_id UUID REFERENCES prtip.scans(scan_id) ON DELETE CASCADE,
    target_ip INET NOT NULL,
    port INTEGER NOT NULL,
    protocol VARCHAR(10),
    state VARCHAR(20),
    service VARCHAR(100),
    version VARCHAR(200),
    banner TEXT,
    discovered_at TIMESTAMP DEFAULT NOW(),

    UNIQUE(scan_id, target_ip, port)
);

-- Indexes
CREATE INDEX idx_target_ip ON prtip.scan_results(target_ip);
CREATE INDEX idx_port ON prtip.scan_results(port);
CREATE INDEX idx_service ON prtip.scan_results(service);
CREATE INDEX idx_discovered_at ON prtip.scan_results(discovered_at);

-- View: Current network state
CREATE VIEW prtip.current_state AS
SELECT DISTINCT ON (target_ip, port)
    target_ip,
    port,
    protocol,
    state,
    service,
    version,
    discovered_at
FROM prtip.scan_results
ORDER BY target_ip, port, discovered_at DESC;
```

**Import Script (Python):**
```python
#!/usr/bin/env python3
import json
import psycopg2
from datetime import datetime
import uuid

def import_to_postgres(results_file, db_config):
    """Import ProRT-IP results to PostgreSQL"""

    conn = psycopg2.connect(**db_config)
    cur = conn.cursor()

    with open(results_file) as f:
        data = json.load(f)

    # Create scan record
    scan_id = str(uuid.uuid4())
    cur.execute("""
        INSERT INTO prtip.scans (scan_id, start_time, end_time, scan_type, status)
        VALUES (%s, %s, %s, %s, %s)
    """, (
        scan_id,
        data.get('start_time', datetime.utcnow()),
        data.get('end_time', datetime.utcnow()),
        data.get('scan_type', 'syn'),
        'complete'
    ))

    # Insert results
    for host in data['hosts']:
        for port in host.get('ports', []):
            cur.execute("""
                INSERT INTO prtip.scan_results
                (scan_id, target_ip, port, protocol, state, service, version, banner)
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (scan_id, target_ip, port) DO UPDATE
                SET state = EXCLUDED.state,
                    service = EXCLUDED.service,
                    version = EXCLUDED.version,
                    banner = EXCLUDED.banner
            """, (
                scan_id,
                host['ip'],
                port['port'],
                port['protocol'],
                port['state'],
                port.get('service', {}).get('name'),
                port.get('service', {}).get('version'),
                port.get('banner')
            ))

    conn.commit()
    cur.close()
    conn.close()

    print(f"Imported scan {scan_id} to PostgreSQL")

if __name__ == '__main__':
    db_config = {
        'host': 'localhost',
        'database': 'security',
        'user': 'prtip',
        'password': 'YOUR_PASSWORD'
    }

    import sys
    if len(sys.argv) != 2:
        print("Usage: import_to_postgres.py <scan.json>")
        sys.exit(1)

    import_to_postgres(sys.argv[1], db_config)
```

**Useful Queries:**
```sql
-- Find new open ports in last 24 hours
SELECT target_ip, port, service, version
FROM prtip.scan_results
WHERE discovered_at > NOW() - INTERVAL '24 hours'
  AND state = 'Open';

-- Port change history for a specific host
SELECT target_ip, port, state, service, discovered_at
FROM prtip.scan_results
WHERE target_ip = '192.168.1.10'
ORDER BY discovered_at DESC;

-- Most common open ports
SELECT port, COUNT(*) as count
FROM prtip.scan_results
WHERE state = 'Open'
GROUP BY port
ORDER BY count DESC
LIMIT 10;
```

---

## Best Practices

### 1. Authentication & Authorization

**API Key Management:**
```python
import os
from cryptography.fernet import Fernet

class SecureConfig:
    def __init__(self):
        # Load encryption key from environment
        key = os.environ.get('CONFIG_ENCRYPTION_KEY')
        if not key:
            raise ValueError("CONFIG_ENCRYPTION_KEY not set")

        self.cipher = Fernet(key.encode())

    def encrypt_api_key(self, api_key):
        return self.cipher.encrypt(api_key.encode()).decode()

    def decrypt_api_key(self, encrypted_key):
        return self.cipher.decrypt(encrypted_key.encode()).decode()

# Usage
config = SecureConfig()
encrypted = config.encrypt_api_key("your-api-key")
# Store encrypted key in config file
```

### 2. Rate Limiting

**Prevent API Abuse:**
```python
import time
from functools import wraps

class RateLimiter:
    def __init__(self, max_calls, period):
        self.max_calls = max_calls
        self.period = period
        self.calls = []

    def __call__(self, func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            now = time.time()

            # Remove old calls outside period
            self.calls = [c for c in self.calls if c > now - self.period]

            if len(self.calls) >= self.max_calls:
                wait_time = self.period - (now - self.calls[0])
                raise Exception(f"Rate limit exceeded. Retry in {wait_time:.1f}s")

            self.calls.append(now)
            return func(*args, **kwargs)

        return wrapper

# Usage
@RateLimiter(max_calls=10, period=60)  # 10 calls per minute
def send_to_api(data):
    requests.post('https://api.example.com/events', json=data)
```

### 3. Error Handling

**Robust Integration Code:**
```python
import logging
from tenacity import retry, stop_after_attempt, wait_exponential

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@retry(
    stop=stop_after_attempt(3),
    wait=wait_exponential(multiplier=1, min=4, max=10)
)
def send_with_retry(url, payload):
    """Send HTTP request with exponential backoff retry"""
    try:
        response = requests.post(url, json=payload, timeout=30)
        response.raise_for_status()
        return response.json()

    except requests.exceptions.Timeout:
        logger.error(f"Timeout sending to {url}")
        raise

    except requests.exceptions.HTTPError as e:
        if e.response.status_code >= 500:
            # Retry on server errors
            logger.warning(f"Server error: {e.response.status_code}")
            raise
        else:
            # Don't retry on client errors
            logger.error(f"Client error: {e.response.status_code}")
            return None

    except Exception as e:
        logger.exception(f"Unexpected error: {e}")
        raise
```

### 4. Data Sanitization

**Prevent Injection Attacks:**
```python
import re
from html import escape

def sanitize_input(data):
    """Sanitize data before sending to external systems"""

    if isinstance(data, dict):
        return {k: sanitize_input(v) for k, v in data.items()}

    elif isinstance(data, list):
        return [sanitize_input(item) for item in data]

    elif isinstance(data, str):
        # Remove control characters
        data = re.sub(r'[\x00-\x1F\x7F]', '', data)

        # HTML escape
        data = escape(data)

        # Truncate long strings
        if len(data) > 1000:
            data = data[:1000] + '...'

        return data

    else:
        return data
```

### 5. Logging & Monitoring

**Comprehensive Logging:**
```python
import logging
from datetime import datetime

class IntegrationLogger:
    def __init__(self, log_file):
        self.logger = logging.getLogger('prtip-integration')
        self.logger.setLevel(logging.DEBUG)

        # File handler
        fh = logging.FileHandler(log_file)
        fh.setLevel(logging.DEBUG)

        # Console handler
        ch = logging.StreamHandler()
        ch.setLevel(logging.INFO)

        # Formatter
        formatter = logging.Formatter(
            '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        fh.setFormatter(formatter)
        ch.setFormatter(formatter)

        self.logger.addHandler(fh)
        self.logger.addHandler(ch)

    def log_scan_start(self, target):
        self.logger.info(f"Starting scan: {target}")

    def log_scan_complete(self, target, duration, results_count):
        self.logger.info(
            f"Scan complete: {target} - "
            f"Duration: {duration:.2f}s - "
            f"Results: {results_count}"
        )

    def log_integration_event(self, integration, event, status):
        self.logger.info(
            f"Integration: {integration} - "
            f"Event: {event} - "
            f"Status: {status}"
        )

    def log_error(self, message, exception=None):
        self.logger.error(message)
        if exception:
            self.logger.exception(exception)

# Usage
logger = IntegrationLogger('/var/log/prtip-integration.log')
logger.log_scan_start('192.168.1.0/24')
```

---

## See Also

- **[Plugin System](../features/plugin-system.md)** - Custom plugins for specialized integrations
- **[Distributed Scanning](./distributed-scanning.md)** - Multi-instance coordination for large-scale integrations
- **[Large-Scale Scanning](./large-scale-scanning.md)** - Performance optimization for integration workloads
- **[Database Usage](./database-usage.md)** - SQL database integration patterns
- **[Automation](./automation.md)** - Scripting and automation examples
- **[Output Formats](../reference/output-formats.md)** - JSON, XML, Greppable output for parsing
- **[API Reference](../../05-API-REFERENCE.md)** - Complete Rust API documentation
- **[User Guide: Basic Usage](../user-guide/basic-usage.md)** - Command-line examples for integration testing

**External Resources:**
- **GitHub Actions Docs**: https://docs.github.com/en/actions
- **GitLab CI/CD**: https://docs.gitlab.com/ee/ci/
- **Splunk HEC**: https://docs.splunk.com/Documentation/Splunk/latest/Data/UsetheHTTPEventCollector
- **ELK Stack**: https://www.elastic.co/what-is/elk-stack
- **OpenVAS GMP**: https://community.greenbone.net/t/about-gvm-architecture/1231

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
