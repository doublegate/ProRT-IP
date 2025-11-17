# Distributed Scanning

Deploy ProRT-IP across multiple instances for internet-scale scanning with coordinated work distribution and result aggregation.

## What is Distributed Scanning?

**Distributed scanning** enables multiple ProRT-IP instances to collaborate on large-scale network reconnaissance by dividing work across compute nodes and aggregating results. This architecture achieves scan rates impossible from a single host.

**ProRT-IP Distributed Capabilities:**
- **Horizontal scaling** - Add more scanners to increase throughput linearly
- **Multiple coordination strategies** - Shared state, message queues, or lockless approaches
- **Automatic work distribution** - Target sharding, port splitting, or dynamic task allocation
- **Result aggregation** - Merge results with deduplication and consistency checks
- **Fault tolerance** - Continue scanning despite node failures
- **Cloud-native deployment** - Kubernetes, Docker Swarm, AWS ECS support

**Use Cases:**
- **Internet-wide surveys** - Scan entire IPv4 space (4.3B addresses)
- **Continuous monitoring** - Maintain up-to-date network inventory across global infrastructure
- **Geographic distribution** - Deploy scanners in multiple regions for network diversity
- **Compliance scanning** - Meet time-bounded audit requirements for massive networks
- **Research projects** - Academic/security research requiring comprehensive data collection

---

## Coordination Strategies

### Strategy 1: Shared Database (PostgreSQL/MySQL)

**Architecture:**

```
┌─────────────────────────────────────────────────────┐
│          Shared PostgreSQL Database                 │
│  ┌──────────────┬───────────────┬─────────────┐   │
│  │ work_queue   │ scan_results  │ locks       │   │
│  │ (targets)    │ (ports/svcs)  │ (coordination)│  │
│  └──────────────┴───────────────┴─────────────┘   │
└─────────────────────────────────────────────────────┘
         ▲                ▲                ▲
         │                │                │
    ┌────┴────┐      ┌───┴────┐      ┌───┴────┐
    │ Scanner │      │Scanner │      │Scanner │
    │ Node 1  │      │ Node 2 │      │ Node 3 │
    └─────────┘      └────────┘      └────────┘
```

**Work Distribution:**

```sql
-- Scanner node claims work (atomic transaction)
BEGIN;
SELECT id, target FROM work_queue
WHERE status = 'pending'
ORDER BY id
LIMIT 1000
FOR UPDATE SKIP LOCKED;

UPDATE work_queue
SET status = 'in_progress', worker = 'node-1'
WHERE id IN (...);
COMMIT;
```

**Result Storage:**

```sql
-- Insert results (batch operation)
INSERT INTO scan_results (target, port, state, service, timestamp, worker)
VALUES
  ('1.2.3.4', 80, 'open', 'http', NOW(), 'node-1'),
  ('1.2.3.4', 443, 'open', 'https', NOW(), 'node-1'),
  ...
ON CONFLICT (target, port) DO UPDATE
  SET state = EXCLUDED.state,
      service = EXCLUDED.service,
      timestamp = EXCLUDED.timestamp;
```

**Advantages:**
- ✅ Simple implementation (ProRT-IP already supports PostgreSQL/SQLite)
- ✅ ACID transactions guarantee work uniqueness
- ✅ Familiar SQL tooling for monitoring and queries
- ✅ No additional infrastructure beyond database
- ✅ Easy result deduplication via primary keys

**Disadvantages:**
- ❌ Database becomes single point of failure (mitigated by replication)
- ❌ Network latency to database affects work claim rate
- ❌ Lock contention at extreme scale (>100 scanners)
- ❌ Database write throughput limits overall scan rate

**When to Use:**
- <20 scanner nodes
- Existing PostgreSQL infrastructure
- Need for SQL-based result analysis
- Moderate scan rate requirements (<500K pps aggregate)

---

### Strategy 2: Message Queue (Redis/RabbitMQ)

**Architecture:**

```
┌───────────────────────────────────────────────┐
│        Redis Message Queue                    │
│  ┌──────────────┬────────────────────────┐   │
│  │ work_list    │ results_stream         │   │
│  │ (LPUSH/RPOP) │ (XADD/XREAD)          │   │
│  └──────────────┴────────────────────────┘   │
└───────────────────────────────────────────────┘
         ▲                      │
         │ claim work           │ publish results
    ┌────┴────┐            ┌───▼────┐
    │ Scanner │            │Result  │
    │ Nodes   │            │Aggreg- │
    │ (1-100) │            │ator    │
    └─────────┘            └────────┘
```

**Work Distribution (Redis List):**

```bash
# Populate work queue (coordinator)
redis-cli LPUSH work_queue "10.0.0.0/24:80,443"
redis-cli LPUSH work_queue "10.0.1.0/24:80,443"
redis-cli LPUSH work_queue "10.0.2.0/24:80,443"

# Claim work (scanner node, atomic operation)
redis-cli RPOP work_queue
# Returns: "10.0.0.0/24:80,443"
```

**Scanner Node Script:**

```bash
#!/bin/bash
# Scanner node work loop

while true; do
    # Claim work atomically
    WORK=$(redis-cli RPOP work_queue)

    if [ -z "$WORK" ]; then
        echo "No work available, sleeping..."
        sleep 10
        continue
    fi

    # Parse work: "10.0.0.0/24:80,443"
    TARGET=$(echo "$WORK" | cut -d: -f1)
    PORTS=$(echo "$WORK" | cut -d: -f2)

    # Execute scan
    prtip -sS -p "$PORTS" "$TARGET" -oJ "/tmp/results-$(date +%s).json"

    # Publish results to stream
    redis-cli XADD results_stream "*" \
        "node" "$HOSTNAME" \
        "target" "$TARGET" \
        "file" "/tmp/results-$(date +%s).json"
done
```

**Result Aggregation (Redis Streams):**

```bash
# Aggregator node reads results
redis-cli XREAD COUNT 100 BLOCK 1000 STREAMS results_stream 0

# Example output:
# 1) 1) "results_stream"
#    2) 1) 1) "1234567890-0"
#          2) 1) "node"
#             2) "scanner-1"
#             3) "target"
#             4) "10.0.0.0/24"
#             5) "file"
#             6) "/tmp/results-123.json"
```

**Advantages:**
- ✅ High throughput (100K+ claims/second)
- ✅ Built-in pub-sub for result streaming
- ✅ No polling overhead (blocking reads)
- ✅ Horizontal scaling to 100+ nodes
- ✅ Redis Cluster for fault tolerance

**Disadvantages:**
- ❌ Additional infrastructure (Redis server)
- ❌ Requires result persistence strategy
- ❌ Network latency to Redis affects work claim rate
- ❌ Stream backlog management needed

**When to Use:**
- 20-100 scanner nodes
- High throughput requirements (>500K pps)
- Need for real-time result streaming
- Existing Redis infrastructure

---

### Strategy 3: Lockless Target Sharding

**Architecture:**

```
Total target space divided into N shards at startup
Each scanner assigned S shards deterministically

Example: 256 /24 subnets, 4 scanners, 64 shards each

┌─────────────────────────────────────────────┐
│   Target Space: 0.0.0.0/8 (16,777,216 IPs) │
└─────────────────────────────────────────────┘
         │
         ├──── Scanner 1: Shards 0-63
         │     (10.0.0.0/24 - 10.0.63.0/24)
         │
         ├──── Scanner 2: Shards 64-127
         │     (10.0.64.0/24 - 10.0.127.0/24)
         │
         ├──── Scanner 3: Shards 128-191
         │     (10.0.128.0/24 - 10.0.191.0/24)
         │
         └──── Scanner 4: Shards 192-255
               (10.0.192.0/24 - 10.0.255.0/24)
```

**Shard Assignment Algorithm:**

```rust
use std::net::Ipv4Addr;

pub struct ShardConfig {
    pub total_shards: usize,
    pub node_id: usize,
    pub total_nodes: usize,
}

impl ShardConfig {
    /// Calculate which shards this node is responsible for
    pub fn my_shards(&self) -> Vec<usize> {
        let shards_per_node = self.total_shards / self.total_nodes;
        let start = self.node_id * shards_per_node;
        let end = start + shards_per_node;
        (start..end).collect()
    }

    /// Check if this node should scan a given IP
    pub fn should_scan(&self, ip: Ipv4Addr) -> bool {
        let shard = self.ip_to_shard(ip);
        self.my_shards().contains(&shard)
    }

    /// Map IP to shard number
    fn ip_to_shard(&self, ip: Ipv4Addr) -> usize {
        let octets = ip.octets();
        // Hash third octet to shard (for /16 target space)
        (octets[2] as usize) % self.total_shards
    }
}
```

**Scanner Node Command:**

```bash
# Node 1 of 4 (scans shards 0-63)
prtip -sS -p 80,443 10.0.0.0/16 \
    --distributed \
    --node-id 0 \
    --total-nodes 4 \
    --shard-count 256 \
    -oN results-node1.txt

# Node 2 of 4 (scans shards 64-127)
prtip -sS -p 80,443 10.0.0.0/16 \
    --distributed \
    --node-id 1 \
    --total-nodes 4 \
    --shard-count 256 \
    -oN results-node2.txt
```

**Advantages:**
- ✅ Zero coordination overhead (no shared state)
- ✅ Perfect linear scaling (no contention)
- ✅ Simple deployment (no infrastructure)
- ✅ Deterministic work assignment (reproducible)
- ✅ Fault tolerance via re-assignment

**Disadvantages:**
- ❌ Requires manual result merging
- ❌ Load imbalance if shards unequal
- ❌ No dynamic work stealing
- ❌ Re-sharding on node count change

**When to Use:**
- 4-50 scanner nodes
- Predictable target distribution
- Want to avoid shared infrastructure
- Offline result aggregation acceptable

---

## Work Distribution Patterns

### Pattern 1: Target Sharding (Horizontal Division)

Divide target IP space across scanners.

**Example: 10.0.0.0/8 across 4 nodes**

```
Node 1: 10.0.0.0   - 10.63.255.255  (  4,194,304 IPs, 25%)
Node 2: 10.64.0.0  - 10.127.255.255 (  4,194,304 IPs, 25%)
Node 3: 10.128.0.0 - 10.191.255.255 (  4,194,304 IPs, 25%)
Node 4: 10.192.0.0 - 10.255.255.255 (  4,194,304 IPs, 25%)
```

**Advantages:**
- Perfect load balance (equal IP counts)
- No coordination overhead
- Scan results naturally partitioned

**Disadvantages:**
- Assumes uniform host distribution (often false)
- Geographic clustering may create hotspots

---

### Pattern 2: Port Splitting (Vertical Division)

Divide port space across scanners while all scan same targets.

**Example: 65,535 ports across 4 nodes**

```
Node 1: Ports 1-16383      (25% of ports)
Node 2: Ports 16384-32767  (25% of ports)
Node 3: Ports 32768-49151  (25% of ports)
Node 4: Ports 49152-65535  (25% of ports)
```

**Advantages:**
- Better load balance than target sharding (port distribution more uniform)
- Useful for focused deep scans

**Disadvantages:**
- All nodes scan same targets (network amplification)
- Requires result merging by target IP

---

### Pattern 3: Task Queue (Dynamic Allocation)

Central task queue distributes work dynamically as nodes become available.

**Task Granularity:**

```
Coarse-grained (better for large-scale):
- Task = /24 subnet × port list
- Example: "10.0.0.0/24:80,443"
- Task count: 256 tasks for /16
- Duration: ~1-5 minutes per task

Fine-grained (better load balance):
- Task = Single host × port list
- Example: "10.0.0.1:80,443"
- Task count: 65,536 tasks for /16
- Duration: ~1-10 seconds per task
```

**Optimal Granularity Formula:**

```
Optimal task duration = (Total_scan_time / Total_nodes) / 10

Example: 10-hour scan, 10 nodes
Optimal task = (10 hours / 10 nodes) / 10 = 6 minutes

This ensures 10+ tasks per node for good load balance
```

---

## Result Aggregation

### Aggregation Pattern 1: Streaming Merge

Each scanner streams results to central aggregator in real-time.

**Architecture:**

```
Scanner 1 ──┐
            ├──> Aggregator ──> Database / File
Scanner 2 ──┤    (dedup +       (final results)
            ├──> merge)
Scanner 3 ──┘
```

**Aggregator Implementation (Pseudo-Code):**

```python
import redis
import json

def aggregate_results():
    r = redis.Redis(host='localhost', port=6379)
    seen = set()  # Deduplication: (ip, port) tuples

    while True:
        # Read from results stream
        results = r.xread({'results_stream': '0'}, count=100, block=1000)

        for stream, messages in results:
            for msg_id, data in messages:
                result = json.loads(data['json'])

                # Deduplication
                key = (result['ip'], result['port'])
                if key in seen:
                    continue
                seen.add(key)

                # Store result
                save_to_database(result)

                # Acknowledge message
                r.xdel('results_stream', msg_id)
```

**Advantages:**
- ✅ Real-time visibility into scan progress
- ✅ Early deduplication reduces storage
- ✅ Single source of truth

**Disadvantages:**
- ❌ Aggregator becomes bottleneck at scale
- ❌ Aggregator failure halts entire scan

---

### Aggregation Pattern 2: Offline Merge

Each scanner writes results locally, merge after completion.

**Workflow:**

```bash
# Phase 1: Distributed scanning (parallel)
Node 1: prtip ... -oN results-node1.txt &
Node 2: prtip ... -oN results-node2.txt &
Node 3: prtip ... -oN results-node3.txt &
wait

# Phase 2: Collect results to coordinator
scp scanner1:/tmp/results-node1.txt ./
scp scanner2:/tmp/results-node2.txt ./
scp scanner3:/tmp/results-node3.txt ./

# Phase 3: Merge and deduplicate
cat results-node*.txt | sort -u > results-merged.txt

# Phase 4: Import to database
prtip-import results-merged.txt --database scan.db
```

**Deduplication Script:**

```bash
#!/bin/bash
# Merge ProRT-IP scan results with deduplication

# Combine all result files
cat results-node*.txt > combined.txt

# Sort by IP:PORT (ensures duplicates are adjacent)
sort -t: -k1,1V -k2,2n combined.txt > sorted.txt

# Remove duplicates (keep first occurrence)
awk -F: '!seen[$1":"$2]++' sorted.txt > deduped.txt

# Statistics
echo "Total results: $(wc -l < combined.txt)"
echo "Unique results: $(wc -l < deduped.txt)"
echo "Duplicates removed: $(($(wc -l < combined.txt) - $(wc -l < deduped.txt)))"
```

**Advantages:**
- ✅ No real-time coordination required
- ✅ Scanners run at full speed (no network overhead)
- ✅ Simple implementation

**Disadvantages:**
- ❌ No progress visibility until completion
- ❌ Requires post-processing step
- ❌ Storage overhead (duplicates until merge)

---

## Multi-Instance Deployment

### Deployment 1: Same-Host Multi-Instance

Run multiple ProRT-IP instances on single powerful server to maximize hardware utilization.

**Use Case:** Maximize throughput from single high-core-count server (32-128 cores)

**Configuration:**

```bash
# Server specs: 64 cores, 256GB RAM, dual 10GbE NICs

# Instance 1: Cores 0-15, NIC eth0
sudo numactl --physcpubind=0-15 prtip -sS -p 80,443 10.0.0.0/16 \
    --distributed --node-id 0 --total-nodes 4 \
    --interface eth0 -oN results-instance1.txt &

# Instance 2: Cores 16-31, NIC eth0
sudo numactl --physcpubind=16-31 prtip -sS -p 80,443 10.0.0.0/16 \
    --distributed --node-id 1 --total-nodes 4 \
    --interface eth0 -oN results-instance2.txt &

# Instance 3: Cores 32-47, NIC eth1
sudo numactl --physcpubind=32-47 prtip -sS -p 80,443 10.0.0.0/16 \
    --distributed --node-id 2 --total-nodes 4 \
    --interface eth1 -oN results-instance3.txt &

# Instance 4: Cores 48-63, NIC eth1
sudo numactl --physcpubind=48-63 prtip -sS -p 80,443 10.0.0.0/16 \
    --distributed --node-id 3 --total-nodes 4 \
    --interface eth1 -oN results-instance4.txt &
```

**Benefits:**
- Bypass per-process file descriptor limits (typically 1M)
- Leverage multiple NICs for bandwidth
- NUMA-aware CPU pinning for cache locality

**Expected Throughput:**
- Per-instance: 100K-500K pps
- Total: 400K-2M pps (4 instances)

---

### Deployment 2: Multi-Host Cloud Deployment

Deploy scanners across multiple cloud instances for geographic distribution and cost optimization.

**Architecture (AWS Example):**

```
┌────────────────────────────────────────────────┐
│         Coordinator (EC2 t3.small)             │
│  - Work queue (Redis)                          │
│  - Result aggregator                           │
│  - Monitoring dashboard                        │
└────────────────────────────────────────────────┘
         │
         ├─── Scanner 1 (EC2 c6i.xlarge, us-east-1)
         │    - 4 vCPUs, 8GB RAM
         │    - 10 Gbps network
         │
         ├─── Scanner 2 (EC2 c6i.xlarge, us-west-2)
         │    - 4 vCPUs, 8GB RAM
         │    - 10 Gbps network
         │
         └─── Scanner 3 (EC2 c6i.xlarge, eu-west-1)
              - 4 vCPUs, 8GB RAM
              - 10 Gbps network
```

**Terraform Configuration:**

```hcl
# terraform/scanner-node.tf

resource "aws_instance" "scanner" {
  count         = 3
  ami           = "ami-0c55b159cbfafe1f0" # Ubuntu 22.04 LTS
  instance_type = "c6i.xlarge"

  user_data = <<-EOF
    #!/bin/bash
    apt-get update
    apt-get install -y redis-tools jq

    # Install ProRT-IP
    wget https://github.com/doublegate/ProRT-IP/releases/latest/download/prtip-linux-x86_64
    chmod +x prtip-linux-x86_64
    mv prtip-linux-x86_64 /usr/local/bin/prtip

    # Start scanner worker
    /usr/local/bin/scanner-worker.sh &
  EOF

  tags = {
    Name = "prtip-scanner-${count.index}"
    Role = "scanner"
  }
}

resource "aws_instance" "coordinator" {
  ami           = "ami-0c55b159cbfafe1f0"
  instance_type = "t3.small"

  user_data = <<-EOF
    #!/bin/bash
    apt-get update
    apt-get install -y redis-server

    # Configure Redis for network access
    sed -i 's/bind 127.0.0.1/bind 0.0.0.0/' /etc/redis/redis.conf
    systemctl restart redis-server
  EOF

  tags = {
    Name = "prtip-coordinator"
    Role = "coordinator"
  }
}
```

**Cost Estimation (AWS):**

| Component | Type | Hourly | Daily | Monthly |
|-----------|------|--------|-------|---------|
| 3× Scanner | c6i.xlarge | $0.51 | $12.24 | $367.20 |
| 1× Coordinator | t3.small | $0.02 | $0.48 | $14.40 |
| **Total** | | **$0.55/hr** | **$13.20/day** | **$396/month** |

**Expected Throughput:** 300K-1.5M pps aggregate (3 scanners × 100K-500K pps)

---

### Deployment 3: Container Orchestration (Kubernetes)

Deploy scanners as Kubernetes pods for dynamic scaling and fault tolerance.

**Kubernetes Manifests:**

```yaml
# coordinator-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: prtip-coordinator
spec:
  replicas: 1
  selector:
    matchLabels:
      app: prtip-coordinator
  template:
    metadata:
      labels:
        app: prtip-coordinator
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        ports:
        - containerPort: 6379

---
# scanner-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: prtip-scanner
spec:
  replicas: 10  # Scale to 10 scanner pods
  selector:
    matchLabels:
      app: prtip-scanner
  template:
    metadata:
      labels:
        app: prtip-scanner
    spec:
      hostNetwork: true  # Required for raw socket access
      containers:
      - name: scanner
        image: doublegate/prtip:latest
        securityContext:
          privileged: true  # Required for raw sockets
        env:
        - name: REDIS_HOST
          value: "prtip-coordinator"
        - name: REDIS_PORT
          value: "6379"
        command: ["/usr/local/bin/scanner-worker.sh"]
        resources:
          requests:
            cpu: "2"
            memory: "4Gi"
          limits:
            cpu: "4"
            memory: "8Gi"

---
# scanner-hpa.yaml (Horizontal Pod Autoscaler)
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: prtip-scanner-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: prtip-scanner
  minReplicas: 5
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 80
```

**Scanner Worker Script:**

```bash
#!/bin/bash
# /usr/local/bin/scanner-worker.sh

set -e

REDIS_HOST=${REDIS_HOST:-localhost}
REDIS_PORT=${REDIS_PORT:-6379}

echo "Starting ProRT-IP scanner worker..."
echo "Coordinator: $REDIS_HOST:$REDIS_PORT"

while true; do
    # Claim work from Redis
    WORK=$(redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" RPOP work_queue)

    if [ -z "$WORK" ]; then
        echo "No work available, sleeping 10s..."
        sleep 10
        continue
    fi

    # Parse work: "10.0.0.0/24:80,443"
    TARGET=$(echo "$WORK" | cut -d: -f1)
    PORTS=$(echo "$WORK" | cut -d: -f2)

    echo "Scanning $TARGET ports $PORTS..."

    # Execute scan
    RESULT_FILE="/tmp/results-$(date +%s)-$$-random.json"
    prtip -sS -p "$PORTS" "$TARGET" -oJ "$RESULT_FILE"

    # Publish results
    if [ -f "$RESULT_FILE" ]; then
        RESULT_DATA=$(cat "$RESULT_FILE" | jq -c .)
        redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" \
            XADD results_stream "*" \
            "node" "$HOSTNAME" \
            "target" "$TARGET" \
            "data" "$RESULT_DATA"
        rm "$RESULT_FILE"
    fi

    echo "Scan complete: $TARGET"
done
```

**Deployment Commands:**

```bash
# Create namespace
kubectl create namespace prtip

# Deploy coordinator (Redis)
kubectl apply -f coordinator-deployment.yaml -n prtip

# Deploy scanners (10 pods initially)
kubectl apply -f scanner-deployment.yaml -n prtip

# Enable autoscaling (5-50 pods)
kubectl apply -f scanner-hpa.yaml -n prtip

# Populate work queue
kubectl run -it --rm redis-cli --image=redis:7-alpine --restart=Never -n prtip -- \
    redis-cli -h prtip-coordinator LPUSH work_queue "10.0.0.0/24:80,443"

# Monitor progress
kubectl logs -f deployment/prtip-scanner -n prtip
```

**Scaling Characteristics:**

| Pods | CPU Cores | Expected Throughput |
|------|-----------|---------------------|
| 5 | 10-20 | 500K-2.5M pps |
| 10 | 20-40 | 1M-5M pps |
| 25 | 50-100 | 2.5M-12.5M pps |
| 50 | 100-200 | 5M-25M pps |

---

## Real-World Examples

### Example 1: Academic Research - Internet Census

**Objective:** Scan entire IPv4 space (4.3 billion addresses) for HTTP/HTTPS servers

**Setup:**

- **Cluster:** 20 AWS c6i.8xlarge instances (32 vCPUs, 64GB RAM each)
- **Coordinator:** 1 AWS t3.medium (Redis + aggregator)
- **Network:** 20× 12.5 Gbps = 250 Gbps aggregate bandwidth
- **Storage:** S3 for results (estimated 500GB)

**Configuration:**

```bash
# Coordinator: Populate work queue with 16,777,216 /24 subnets
for i in {0..255}; do
    for j in {0..255}; do
        for k in {0..255}; do
            redis-cli LPUSH work_queue "$i.$j.$k.0/24:80,443"
        done
    done
done

# Scanner nodes (20 instances, identical command)
prtip-distributed-worker \
    --redis-host coordinator.internal \
    --max-rate 500000 \
    --timing T4 \
    --output s3://research-scans/results-$(hostname)/
```

**Expected Results:**

| Metric | Value |
|--------|-------|
| Total targets | 4,294,967,296 IPs |
| Scan rate | 10M pps aggregate (20 nodes × 500K pps) |
| Duration | 429 seconds (~7 minutes theoretical, 15-20 minutes realistic) |
| Responsive hosts | ~100M (2-3% response rate) |
| Total results | ~200M port records (100M hosts × 2 ports avg) |
| Storage | ~400GB (JSON format) |

---

### Example 2: Continuous Monitoring - Enterprise Network

**Objective:** Daily scan of 500K enterprise hosts (10 data centers)

**Setup:**

- **Cluster:** 10 on-premise servers (1 per data center)
  - Dell PowerEdge R750 (64 cores, 256GB RAM, dual 25GbE)
- **Coordinator:** PostgreSQL database (central IT infrastructure)
- **Scheduling:** Cron job daily at 02:00 local time

**Configuration:**

```bash
# PostgreSQL schema (coordinator)
CREATE TABLE work_queue (
    id SERIAL PRIMARY KEY,
    target VARCHAR(18),
    ports VARCHAR(100),
    status VARCHAR(20) DEFAULT 'pending',
    worker VARCHAR(50),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE scan_results (
    id SERIAL PRIMARY KEY,
    target VARCHAR(15),
    port INTEGER,
    state VARCHAR(10),
    service VARCHAR(50),
    version VARCHAR(100),
    timestamp TIMESTAMP DEFAULT NOW(),
    worker VARCHAR(50),
    UNIQUE(target, port)
);

# Populate daily work queue (50K /24 subnets)
INSERT INTO work_queue (target, ports)
SELECT network || '/24', '21,22,23,80,443,445,3389,8080'
FROM enterprise_networks;
```

**Scanner Node (Systemd Service):**

```ini
# /etc/systemd/system/prtip-scanner.service
[Unit]
Description=ProRT-IP Distributed Scanner
After=network.target

[Service]
Type=simple
User=scanner
ExecStart=/usr/local/bin/prtip-scanner-worker.sh
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

**Worker Script:**

```bash
#!/bin/bash
# /usr/local/bin/prtip-scanner-worker.sh

DB_HOST="postgres.corp.internal"
DB_NAME="scanning"
DB_USER="scanner"

while true; do
    # Claim work (atomic SQL)
    WORK=$(psql -h "$DB_HOST" -d "$DB_NAME" -U "$DB_USER" -tAc "
        WITH work AS (
            SELECT id, target, ports FROM work_queue
            WHERE status = 'pending'
            ORDER BY id
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        UPDATE work_queue SET status = 'in_progress', worker = '$(hostname)'
        WHERE id = (SELECT id FROM work)
        RETURNING target, ports;
    ")

    if [ -z "$WORK" ]; then
        sleep 60
        continue
    fi

    TARGET=$(echo "$WORK" | cut -d'|' -f1)
    PORTS=$(echo "$WORK" | cut -d'|' -f2)

    # Scan and import to database
    prtip -sS -sV -p "$PORTS" "$TARGET" --database "postgresql://$DB_USER@$DB_HOST/$DB_NAME"
done
```

**Monitoring Dashboard (Grafana + PostgreSQL):**

```sql
-- Active scanners
SELECT worker, COUNT(*) as tasks_active
FROM work_queue
WHERE status = 'in_progress'
GROUP BY worker;

-- Scan progress
SELECT
    COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending,
    COUNT(CASE WHEN status = 'in_progress' THEN 1 END) as in_progress,
    COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed,
    ROUND(100.0 * COUNT(CASE WHEN status = 'completed' THEN 1 END) / COUNT(*), 2) as percent_complete
FROM work_queue;

-- New discoveries (last 24h)
SELECT state, COUNT(*) as count
FROM scan_results
WHERE timestamp > NOW() - INTERVAL '24 hours'
GROUP BY state;
```

**Expected Results:**

| Metric | Value |
|--------|-------|
| Total work units | 50,000 /24 subnets |
| Scan duration | 2-4 hours (10 scanners) |
| Throughput | 50K-100K pps per scanner |
| Results | ~5M port records daily |
| Change detection | ~1,000 new/changed services daily |

---

### Example 3: Security Research - Vulnerability Survey

**Objective:** Identify all exposed instances of vulnerable service (e.g., Log4Shell)

**Setup:**

- **Cluster:** 50 GCP n2-highcpu-16 instances (16 vCPUs, 16GB RAM each)
- **Coordinator:** Cloud Memorystore (Redis)
- **Target:** Entire internet IPv4 space (exclude IANA reserved)
- **Timeline:** 2-hour time window for rapid assessment

**Configuration:**

```bash
# Generate work queue (exclude reserved ranges)
cat <<EOF | redis-cli --pipe
LPUSH work_queue "1.0.0.0/8:8080,8443,9200,9300,8009"
LPUSH work_queue "2.0.0.0/8:8080,8443,9200,9300,8009"
...
LPUSH work_queue "223.0.0.0/8:8080,8443,9200,9300,8009"
EOF

# Scanner command (50 nodes)
prtip -sS -p 8080,8443,9200,9300,8009 \
    --distributed --use-redis \
    --redis-url redis://coordinator:6379 \
    --max-rate 1000000 \
    --timing T5 \
    --service-detection \
    --match-regex "log4j" \
    --output-format json \
    --stream-to redis://coordinator:6379/results
```

**Vulnerability Detection (Post-Scan):**

```python
import redis
import json

r = redis.Redis(host='coordinator', port=6379)

vulnerable_hosts = []

# Read results stream
for result in r.xread({'results': '0'}, count=1000):
    data = json.loads(result['data'])

    # Check for Log4j indicators
    if any(indicator in data.get('banner', '').lower()
           for indicator in ['log4j', 'apache-log4j', 'logback']):
        vulnerable_hosts.append({
            'ip': data['ip'],
            'port': data['port'],
            'service': data['service'],
            'version': data['version'],
            'banner': data['banner']
        })

print(f"Vulnerable hosts found: {len(vulnerable_hosts)}")
```

**Expected Results:**

| Metric | Value |
|--------|-------|
| Target space | ~3.7B IPs (excluding reserved) |
| Scan rate | 50M pps (50 nodes × 1M pps) |
| Duration | 74 seconds theoretical, ~2 hours realistic |
| Responsive hosts | ~50M (1-2% response rate) |
| Vulnerable hosts | ~100K-1M (0.2-2% of responsive) |
| Cost | ~$50 (50 instances × 2 hours × $0.50/hr) |

---

## Best Practices

### 1. Work Granularity

**Too Coarse (Bad):**
```bash
# Single task = entire /8 subnet
work_queue: ["10.0.0.0/8:80,443"]
# Problem: 1 scanner handles 16M IPs, others idle
```

**Too Fine (Bad):**
```bash
# Single task = 1 IP × 1 port
work_queue: ["10.0.0.1:80", "10.0.0.1:443", "10.0.0.2:80", ...]
# Problem: 33M tasks, coordination overhead exceeds scan time
```

**Optimal (Good):**
```bash
# Single task = /24 subnet × port list
work_queue: ["10.0.0.0/24:80,443", "10.0.1.0/24:80,443", ...]
# Sweet spot: ~65K tasks for /8, 1-5 minutes per task
```

**Granularity Formula:**

```
Ideal task count = Total_nodes × 100

Example: 10 scanners → 1,000 tasks
For /8 (16M IPs): Task size = 16M / 1,000 = 16K IPs = ~/ 18 subnets
```

---

### 2. Fault Tolerance

**Heartbeat Monitoring:**

```sql
-- Heartbeat table (coordinator database)
CREATE TABLE scanner_heartbeats (
    worker VARCHAR(50) PRIMARY KEY,
    last_seen TIMESTAMP DEFAULT NOW()
);

-- Scanner node updates heartbeat every 30 seconds
UPDATE scanner_heartbeats SET last_seen = NOW() WHERE worker = 'node-1';

-- Coordinator detects dead scanners (>5 minutes since heartbeat)
SELECT worker FROM scanner_heartbeats
WHERE last_seen < NOW() - INTERVAL '5 minutes';

-- Reassign work from dead scanners
UPDATE work_queue SET status = 'pending', worker = NULL
WHERE status = 'in_progress' AND worker IN (
    SELECT worker FROM scanner_heartbeats
    WHERE last_seen < NOW() - INTERVAL '5 minutes'
);
```

**Advantages:**
- Automatic recovery from scanner failures
- No human intervention required
- Work automatically reassigned to healthy nodes

---

### 3. Rate Limiting (Prevent Network Saturation)

**Per-Scanner Rate Limits:**

```bash
# Conservative (100K pps per scanner)
prtip --max-rate 100000 ...

# Aggressive (500K pps per scanner)
prtip --max-rate 500000 ...

# Dynamic (based on network capacity)
BANDWIDTH_GBPS=10
PACKET_SIZE=60
MAX_RATE=$((BANDWIDTH_GBPS * 1000000000 / (PACKET_SIZE * 8)))
prtip --max-rate $MAX_RATE ...
```

**Aggregate Rate Coordination:**

```python
# Coordinator enforces global rate limit across all scanners
# Example: 5M pps global limit, 10 scanners = 500K pps each

GLOBAL_RATE_LIMIT = 5000000  # 5M pps
SCANNER_COUNT = 10
PER_SCANNER_LIMIT = GLOBAL_RATE_LIMIT // SCANNER_COUNT

# Publish per-scanner limit via Redis
redis_client.set('rate_limit_per_scanner', PER_SCANNER_LIMIT)

# Scanners read limit at startup
rate_limit = int(redis_client.get('rate_limit_per_scanner'))
```

---

### 4. Result Deduplication

**Why Deduplication Needed:**
- Network packet loss may trigger retransmissions
- Scanner restarts may re-scan targets
- Work reassignment may cause overlapping scans

**Deduplication Strategies:**

**Strategy 1: Database Primary Keys**
```sql
CREATE TABLE scan_results (
    target VARCHAR(15),
    port INTEGER,
    state VARCHAR(10),
    PRIMARY KEY (target, port)  -- Automatic deduplication
);

INSERT INTO scan_results VALUES ('1.2.3.4', 80, 'open')
ON CONFLICT (target, port) DO UPDATE
    SET state = EXCLUDED.state;  -- Update if newer
```

**Strategy 2: Bloom Filter (Memory-Efficient)**
```python
from pybloom_live import BloomFilter

# 100M entries, 0.1% false positive rate
bloom = BloomFilter(capacity=100000000, error_rate=0.001)

def is_duplicate(ip, port):
    key = f"{ip}:{port}"
    if key in bloom:
        return True  # Likely duplicate (0.1% false positive)
    bloom.add(key)
    return False

# Memory usage: ~120MB for 100M entries (vs 2GB for hash set)
```

---

### 5. Monitoring and Observability

**Key Metrics:**

```python
# Prometheus metrics (exported by coordinator)
from prometheus_client import Counter, Gauge, Histogram

# Work queue metrics
work_queue_pending = Gauge('work_queue_pending', 'Pending tasks')
work_queue_in_progress = Gauge('work_queue_in_progress', 'In-progress tasks')
work_queue_completed = Counter('work_queue_completed_total', 'Completed tasks')

# Scanner metrics
scanner_active = Gauge('scanner_active', 'Active scanners')
scanner_throughput = Gauge('scanner_throughput_pps', 'Packets per second', ['node'])

# Result metrics
results_total = Counter('results_total', 'Total results')
results_open_ports = Counter('results_open_ports_total', 'Open ports found')

# Performance metrics
scan_duration = Histogram('scan_duration_seconds', 'Scan duration', ['target_size'])
```

**Grafana Dashboard Queries:**

```promql
# Aggregate throughput (all scanners)
sum(scanner_throughput_pps)

# Completion percentage
100 * (work_queue_completed_total / (work_queue_pending + work_queue_in_progress + work_queue_completed_total))

# Average scan duration (last 1 hour)
avg_over_time(scan_duration_seconds[1h])

# Results per second
rate(results_total[1m])
```

---

## Troubleshooting

### Issue 1: Scanners Idle Despite Pending Work

**Symptoms:**
- Work queue has pending tasks
- Scanner nodes show 0% CPU utilization
- No errors in scanner logs

**Root Cause:** Work claim failures (database locks, network issues)

**Solutions:**

```bash
# 1. Check database connection
psql -h coordinator -U scanner -c "SELECT 1;"

# 2. Check Redis connection
redis-cli -h coordinator PING

# 3. Verify work queue visibility
redis-cli -h coordinator LLEN work_queue

# 4. Check scanner logs for errors
tail -f /var/log/prtip-scanner.log

# 5. Increase work claim timeout
# In scanner config: claim_timeout = 10s → 30s
```

---

### Issue 2: Duplicate Results

**Symptoms:**
- Same (IP, port) pair appears multiple times in results
- Result count exceeds expected maximum

**Root Cause:** Insufficient deduplication or scanner restarts

**Solutions:**

```sql
-- 1. Identify duplicates
SELECT target, port, COUNT(*) as count
FROM scan_results
GROUP BY target, port
HAVING COUNT(*) > 1;

-- 2. Remove duplicates (keep first occurrence)
DELETE FROM scan_results WHERE id NOT IN (
    SELECT MIN(id) FROM scan_results
    GROUP BY target, port
);

-- 3. Add unique constraint to prevent future duplicates
ALTER TABLE scan_results
ADD CONSTRAINT unique_target_port UNIQUE (target, port);
```

**Prevention:**
- Use database unique constraints
- Implement idempotent result storage (UPSERT)
- Add Bloom filter pre-check before database insert

---

### Issue 3: Uneven Load Distribution

**Symptoms:**
- Some scanners 100% CPU, others idle
- Total throughput below expected aggregate

**Root Cause:** Coarse work granularity or skewed target distribution

**Solutions:**

```bash
# 1. Check work distribution
SELECT worker, COUNT(*) as tasks
FROM work_queue
WHERE status = 'in_progress'
GROUP BY worker;

# 2. Reduce task granularity
# Before: 1 task = /16 subnet (65,536 IPs, 10+ minutes)
# After:  1 task = /24 subnet (256 IPs, 10-30 seconds)

# 3. Increase task count
# Target: 100+ tasks per scanner for good load balance
```

**Prevention:**
- Use fine-grained work units (<5 minute duration)
- Aim for 100× more tasks than scanners
- Monitor per-scanner CPU utilization

---

## See Also

**Advanced Topics:**
- [Large-Scale Scanning](./large-scale-scanning.md) - Capacity planning, architecture patterns
- [Performance Tuning](./performance-tuning.md) - System optimization for distributed deployment
- [Database Usage](./database-usage.md) - PostgreSQL/Redis integration patterns

**User Guide:**
- [Basic Usage](../user-guide/basic-usage.md) - Single-instance scanning fundamentals
- [Output Formats](../user-guide/output-formats.md) - Result formats for aggregation

**Reference:**
- [Command Reference](../reference/command-reference.md) - Distributed scanning CLI flags
- [Configuration Files](../user-guide/configuration.md) - Multi-instance configuration

**External Resources:**
- **Kubernetes:** https://kubernetes.io/docs/ (container orchestration)
- **Redis:** https://redis.io/docs/ (message queue and coordination)
- **PostgreSQL:** https://www.postgresql.org/docs/ (result storage and work queue)
- **Terraform:** https://www.terraform.io/docs/ (cloud infrastructure provisioning)

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
