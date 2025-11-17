# Large-Scale Scanning

Strategies and architecture for internet-scale network reconnaissance with ProRT-IP.

## What is Large-Scale Scanning?

**Large-scale scanning** refers to network reconnaissance operations targeting thousands to millions of hosts, requiring specialized architectures and techniques to complete efficiently while minimizing system impact and detection risk.

**ProRT-IP Large-Scale Capabilities:**
- **10M+ packets/second** capable (stateless mode)
- **Internet-scale IPv4 sweep** in <6 minutes (appropriate hardware)
- **Hybrid scanning** balances speed and depth (stateless discovery → stateful enumeration)
- **Streaming results** enables scans with unlimited target counts
- **Adaptive rate limiting** prevents network disruption (-1.8% overhead)

**Use Cases:**
- **Internet-wide discovery** (port 80/443 across IPv4 space)
- **ASN-based reconnaissance** (scan specific autonomous systems)
- **Cloud infrastructure mapping** (AWS, Azure, GCP IP ranges)
- **Vulnerability research** (identify exposed services at scale)
- **Network census** (track exposed services over time)

**Challenges:**
- **Memory constraints:** Stateful scanning requires per-host state
- **Network bandwidth:** Packet rates must not saturate links
- **Detection risk:** IDS/IPS systems detect high-rate scans
- **ISP rate limiting:** Internet providers throttle or block scans
- **Legal considerations:** Unauthorized scanning may violate laws

**Ethical and Legal Requirements:**

⚠️ **CRITICAL: Authorized Use Only**

Large-scale scanning raises significant legal and ethical concerns:

✅ **Legal Uses:**
- Security research with written authorization
- Scanning owned infrastructure (cloud VPCs, corporate networks)
- Bug bounty programs with explicit scope
- Academic research with IRB approval
- Compliance validation (internal networks)

❌ **Illegal Uses:**
- Unauthorized internet-wide scans
- Scanning without explicit permission
- Violating ISP terms of service
- Targeting critical infrastructure
- Evading detection for malicious purposes

**Legal Frameworks:**
- **United States:** Computer Fraud and Abuse Act (CFAA), 18 U.S.C. § 1030
- **United Kingdom:** Computer Misuse Act 1990
- **European Union:** Network and Information Systems Directive
- **International:** Budapest Convention on Cybercrime

**Responsible Scanning Principles:**
1. Obtain written authorization before scanning
2. Respect rate limits and network capacity
3. Implement proper identification (reverse DNS, abuse contact)
4. Honor exclusion lists (military, government, critical infrastructure)
5. Respond to abuse complaints promptly
6. Document methodology and findings responsibly

---

## Architecture for Scale

### Scanning Modes Comparison

ProRT-IP provides three architectural approaches for different scale requirements:

| Mode | State | Speed | Memory | Use Case |
|------|-------|-------|--------|----------|
| **Stateless** | O(1) | ⚡⚡⚡⚡⚡ (10M+ pps) | <1 MB | Internet-wide discovery |
| **Stateful** | O(n×p) | ⚡⚡⚡ (6K pps) | ~10 KB/host | Detailed enumeration |
| **Hybrid** | O(responsive) | ⚡⚡⚡⚡ (9K pps avg) | Variable | Recommended (90%+ faster) |

### Stateless Mode (Masscan-Style)

**Architecture Overview:**

Stateless scanning eliminates per-target state, enabling internet-scale throughput:

**Key Characteristics:**
- **No connection state:** Zero memory per target
- **SipHash-based validation:** Encode target identity in sequence numbers
- **Maximum throughput:** 10M+ packets/second capable
- **O(1) memory complexity:** Independent of target count
- **Target randomization:** Permutation functions for distributed load

**How It Works:**

```
1. Generate random sequence number encoding target identity:
   SEQ = SipHash(target_ip, target_port, secret_key)

2. Send SYN packet with encoded sequence number:
   TCP SYN: src_port=random, seq=SEQ, flags=SYN

3. Validate response without state lookup:
   Expected ACK = SEQ + 1
   If received ACK matches → target identified without lookup

4. Stream result to disk immediately (no buffering)
```

**Validation Without State:**

```rust
// Encode target in sequence number (stateless probe)
fn generate_seq(ip: Ipv4Addr, port: u16, key: (u64, u64)) -> u32 {
    let mut hasher = SipHasher::new_with_keys(key.0, key.1);
    hasher.write(&ip.octets());
    hasher.write_u16(port);
    (hasher.finish() & 0xFFFFFFFF) as u32
}

// Validate response without lookup (O(1) validation)
fn validate_response(packet: &TcpPacket, key: (u64, u64)) -> Option<TargetId> {
    let expected_ack = generate_seq(
        packet.dest_ip(),
        packet.dest_port(),
        key
    ).wrapping_add(1);

    if packet.ack() == expected_ack {
        Some(TargetId { ip: packet.dest_ip(), port: packet.dest_port() })
    } else {
        None // Invalid response, discard
    }
}
```

**Performance Characteristics:**

| Metric | Value | Notes |
|--------|-------|-------|
| **Throughput** | 10,200 pps (localhost) | Kernel processing limited |
| **Memory** | <1 MB | Independent of target count |
| **CPU** | 10-20% single core | Packet crafting overhead |
| **Scaling** | O(n × p) time, O(1) space | Linear time, constant memory |

**When to Use:**
- ✅ Internet-wide discovery (1M+ hosts, limited ports)
- ✅ ASN-based scanning (10K-100K hosts)
- ✅ Initial reconnaissance phase (find responsive hosts)
- ✅ Time-constrained situations (fast results required)

**Limitations:**
- ❌ No service detection (banner grabbing requires stateful connections)
- ❌ No OS fingerprinting (requires multiple probe sequences)
- ❌ Single scan type (SYN only in pure stateless mode)
- ❌ Limited error handling (no retransmissions)

### Stateful Mode (Nmap-Style)

**Architecture Overview:**

Stateful scanning maintains per-connection state, enabling comprehensive enumeration:

**Key Characteristics:**
- **Connection tracking:** Per-host state in hash map
- **Retransmission support:** Exponential backoff for lost packets
- **Congestion control:** RTT-based adaptive rate limiting
- **Multiple scan types:** SYN, FIN, NULL, Xmas, ACK, Window, Maimon
- **Deep inspection:** Service detection, OS fingerprinting, banner grabbing

**State Machine:**

```rust
enum ConnectionState {
    Pending,                                  // Initial state
    SynSent { sent_at: Instant, seq: u32 },  // SYN sent, awaiting SYN-ACK
    SynAckReceived { rtt: Duration },        // SYN-ACK received, calculating RTT
    RstReceived,                             // RST received, port closed
    Timeout { attempts: u8 },                // No response, retrying
}
```

**Memory Overhead:**

```
Memory per host = Base overhead + (Ports × Per-port overhead)
                = ~2 KB + (n × 8 bytes)
                = ~2 KB + (1,000 × 8 bytes) = ~10 KB per host

For 10,000 hosts scanning 1,000 ports each:
Memory = 10,000 × 10 KB = 100 MB
```

**Performance Characteristics:**

| Metric | Value | Notes |
|--------|-------|-------|
| **Throughput** | 6,600 pps (localhost) | Full TCP handshake overhead |
| **Memory** | ~10 KB per host | Connection state tracking |
| **CPU** | 40-60% multi-core | Async I/O overhead |
| **Scaling** | O(n × p) time and space | Linear time and memory |

**When to Use:**
- ✅ Detailed enumeration (service detection required)
- ✅ Small-medium scale (<10K hosts)
- ✅ Stealth scanning (FIN/NULL/Xmas scans)
- ✅ Comprehensive reports (OS fingerprinting, banners)

**Limitations:**
- ❌ High memory usage (10 KB per host)
- ❌ Slower throughput (6.6K pps vs 10K+ pps stateless)
- ❌ Doesn't scale to internet-wide (memory constraints)

### Hybrid Mode (Recommended)

**Architecture Overview:**

Hybrid mode combines stateless discovery with stateful enumeration for optimal efficiency:

**Workflow:**

```
Phase 1: Fast Discovery (Stateless)
  ├─ Stateless SYN sweep across all targets
  ├─ Stream responsive hosts to file
  └─ Memory: O(1), Time: O(n × p)
       │
       ▼
  [Responsive Hosts List] (e.g., 5% of total targets)
       │
       ▼
Phase 2: Stateful Enumeration
  ├─ Stateful scan of responsive hosts only
  ├─ Service detection, OS fingerprinting
  ├─ Banner grabbing, deep inspection
  └─ Memory: O(responsive), Time: O(responsive × p')
       │
       ▼
  [Complete Scan Report]
```

**Performance Comparison:**

| Mode | Targets | Ports | Time | Memory | Notes |
|------|---------|-------|------|--------|-------|
| **Stateless Only** | 100K | 10 | 163s | <1 MB | Fast, no depth |
| **Stateful Only** | 100K | 10 | 2,520s | 1 GB | Slow, comprehensive |
| **Hybrid** | 100K | 10 | 250s | 50 MB | **90% faster, 95% less memory** |

**Why 90% Faster:**

```
Full stateful scan: 100,000 hosts × 10 ports = 1,000,000 probes
Stateless discovery: 100,000 hosts × 10 ports = 1,000,000 probes (fast, 163s)
Assume 5% response rate: 100,000 × 0.05 = 5,000 responsive hosts
Stateful enumeration: 5,000 hosts × 10 ports = 50,000 probes (detailed, 87s)

Total hybrid time: 163s + 87s = 250s
vs. Full stateful: 2,520s
Speedup: 2,520s / 250s = 10x faster (90% reduction)
```

**Configuration:**

```bash
# Enable hybrid mode (automatic if discovery + enumeration flags)
prtip -sS -sV -p 80,443,8080 0.0.0.0/8

# Explicit hybrid workflow
prtip -sS -p 80,443 0.0.0.0/8 -oN discovery.txt  # Phase 1
prtip -sS -sV -iL discovery.txt                   # Phase 2
```

**When to Use:**
- ✅ **Always preferred** for large-scale scans with service detection
- ✅ Internet-scale discovery + targeted enumeration
- ✅ Memory-constrained environments
- ✅ Time-sensitive operations (maximize speed and depth)

**Benefits:**
- **90%+ time reduction** vs. full stateful scan
- **95%+ memory reduction** (only track responsive hosts)
- **Maintains accuracy** for service detection
- **Automatic fallback** to stateful if response rate is high

---

## Capacity Planning

### Memory-Based Capacity

**Stateless Scans (SYN/FIN/NULL/Xmas/ACK):**

| Available RAM | Max Hosts | Ports per Host | Notes |
|---------------|-----------|----------------|-------|
| 512 MB | Unlimited | 10-100 | Streaming results (O(1) memory) |
| 1 GB | Unlimited | 10-100 | Stateless architecture |
| 4 GB | Unlimited | 10-100 | No memory constraints |

**Stateful Scans (Connect, Service Detection):**

| Available RAM | Max Hosts | Ports per Host | Notes |
|---------------|-----------|----------------|-------|
| 1 GB | 10,000 | 100 | Minimal overhead |
| 4 GB | 50,000 | 1,000 | Typical desktop |
| 16 GB | 200,000 | 1,000 | Server-class |
| 64 GB | 1,000,000 | 100 | Internet-scale |

**Formula:**

```
Memory (MB) = Base_overhead + (Hosts × Ports × Per_connection_overhead)
            = 2 MB + (Hosts × Ports × 8 bytes) / 1,048,576

Example: 10,000 hosts × 1,000 ports
Memory = 2 MB + (10,000 × 1,000 × 8) / 1,048,576
       = 2 MB + 76.3 MB
       = 78.3 MB ≈ 80 MB
```

### Network-Based Capacity

**Bandwidth Requirements:**

| Bandwidth | Packet Size | Max PPS | Hosts/Min (1K ports) | Hosts/Hour |
|-----------|-------------|---------|---------------------|------------|
| 1 Mbps | 60 bytes | 2,083 pps | 2 hosts/min | 120 hosts |
| 10 Mbps | 60 bytes | 20,833 pps | 20 hosts/min | 1,200 hosts |
| 100 Mbps | 60 bytes | 208,333 pps | 200 hosts/min | 12,000 hosts |
| 1 Gbps | 60 bytes | 2,083,333 pps | 2,000 hosts/min | 120,000 hosts |
| 10 Gbps | 60 bytes | 20,833,333 pps | 20,000 hosts/min | 1,200,000 hosts |

**Formula:**

```
Max PPS = Bandwidth (bps) / (Packet_Size (bytes) × 8)

Hosts per minute = Max_PPS / Ports_per_host

Example: 100 Mbps, 1,000 ports per host
Max PPS = 100,000,000 / (60 × 8) = 208,333 pps
Hosts/min = 208,333 / 1,000 = 208 hosts/min
```

### Scan Duration Estimation

**Basic Formula:**

```
Duration (seconds) = (Hosts × Ports) / Throughput_pps
```

**Example Calculations:**

| Scenario | Hosts | Ports | Throughput | Duration |
|----------|-------|-------|------------|----------|
| Home Network | 10 | 1,000 | 10,000 pps | 1 second |
| Small Office | 100 | 1,000 | 10,000 pps | 10 seconds |
| Data Center | 1,000 | 100 | 10,000 pps | 10 seconds |
| Internet /24 | 256 | 10 | 5,000 pps | <1 second |
| Internet /16 | 65,536 | 10 | 5,000 pps | 131 seconds (~2 min) |
| Internet /8 | 16,777,216 | 2 | 100,000 pps | 335 seconds (~6 min) |

**Adjust for Features:**

| Feature | Duration Multiplier | Example Impact |
|---------|-------------------|----------------|
| Service Detection (-sV) | 1.5-2x | 2 min → 3-4 min |
| OS Fingerprinting (-O) | 1.3-1.5x | 2 min → 2.6-3 min |
| Decoy Scanning (-D 3 decoys) | 4x | 2 min → 8 min |
| Timing T0 (Paranoid) | 500x | 2 min → 16.7 hours |
| Timing T2 (Polite) | 5x | 2 min → 10 min |
| Timing T4 (Aggressive) | 0.8x | 2 min → 1.6 min |
| Timing T5 (Insane) | 0.6x | 2 min → 1.2 min |

### Hardware Requirements

**CPU Recommendations:**

| Scan Scale | Min CPU | Recommended CPU | Notes |
|------------|---------|-----------------|-------|
| Small (<1K hosts) | 1 core, 2 GHz | 2 cores, 3 GHz | Single-threaded sufficient |
| Medium (<10K hosts) | 2 cores, 2 GHz | 4 cores, 3 GHz | Parallel scanning benefits |
| Large (<100K hosts) | 4 cores, 3 GHz | 8 cores, 3.5 GHz | Multi-threaded required |
| Internet-Scale (1M+) | 8 cores, 3 GHz | 16+ cores, 3.5 GHz | NUMA multi-socket preferred |

**RAM Recommendations:**

| Scan Scale | Min RAM | Recommended RAM | Notes |
|------------|---------|-----------------|-------|
| Small (<1K hosts) | 512 MB | 1 GB | Minimal overhead |
| Medium (<10K hosts) | 1 GB | 4 GB | Comfortable buffer |
| Large (<100K hosts) | 4 GB | 16 GB | Batch processing |
| Internet-Scale (1M+) | 8 GB | 64 GB | Streaming required |

**Network Interface:**

| Scan Scale | Min NIC | Recommended NIC | Notes |
|------------|---------|-----------------|-------|
| Small (<1K hosts) | 100 Mbps | 1 Gbps | Any modern NIC |
| Medium (<10K hosts) | 1 Gbps | 1 Gbps | Gigabit sufficient |
| Large (<100K hosts) | 1 Gbps | 10 Gbps | High throughput |
| Internet-Scale (1M+) | 10 Gbps | 10 Gbps+ | Dedicated NIC |

**Storage Requirements:**

| Result Format | Storage per Host | Storage for 100K Hosts | Storage for 1M Hosts |
|---------------|------------------|----------------------|---------------------|
| Text | ~500 bytes | 50 MB | 500 MB |
| JSON | ~1 KB | 100 MB | 1 GB |
| XML (Nmap) | ~1.5 KB | 150 MB | 1.5 GB |
| PCAPNG | ~50 KB | 5 GB | 50 GB |
| SQLite | ~800 bytes | 80 MB | 800 MB |

---

## Target Segmentation Strategies

### CIDR-Based Segmentation

**Subnet Planning:**

| CIDR | Hosts | Scan Strategy | Memory | Time (@ 10K pps) |
|------|-------|---------------|--------|------------------|
| /32 | 1 | Direct scan | <1 MB | <1 second |
| /24 | 256 | Single batch | <10 MB | 26 seconds |
| /20 | 4,096 | Batch processing | <50 MB | 7 minutes |
| /16 | 65,536 | Streaming results | <100 MB | 1.8 hours |
| /12 | 1,048,576 | Distributed scan | <500 MB | 29 hours |
| /8 | 16,777,216 | Multi-instance | <1 GB | 19 days |

**Best Practices:**

1. **Split Large Scans into Subnets:**

```bash
# Scan /16 as 256 separate /24 subnets
for i in {0..255}; do
    prtip -sS -p 80,443 10.0.$i.0/24 -oN scan-10.0.$i.0.txt &
done
wait
```

2. **Use Target Randomization:**

```bash
# Randomize target order (avoid sequential scanning)
prtip --randomize-hosts -sS -p 80,443 10.0.0.0/16
```

3. **Parallel Instance Strategy:**

```bash
# Split /8 into 16 parallel /12 scans
for i in {0..15}; do
    prtip -sS -p 80,443 10.$((i*16)).0.0/12 -oN scan-$i.txt &
done
wait
```

### ASN-Based Segmentation

**Autonomous System Number (ASN) Targeting:**

Target specific organizations or cloud providers by ASN:

```bash
# Example: Scan Amazon AWS IP ranges (AS16509)
# 1. Download ASN prefixes from BGP route tables
wget https://bgp.tools/asn/16509/prefixes.json

# 2. Extract CIDR blocks
cat prefixes.json | jq -r '.ipv4[]' > aws-prefixes.txt

# 3. Scan each prefix
while read prefix; do
    prtip -sS -p 80,443,8080 $prefix -oN aws-$prefix.txt
done < aws-prefixes.txt
```

**Common ASNs:**

| Organization | ASN | Approximate IPs | Example CIDR |
|--------------|-----|----------------|--------------|
| Amazon AWS | AS16509 | 100M+ | 52.0.0.0/8 |
| Google Cloud | AS15169 | 50M+ | 35.0.0.0/8 |
| Microsoft Azure | AS8075 | 80M+ | 40.0.0.0/8 |
| Cloudflare | AS13335 | 10M+ | 104.16.0.0/13 |
| Akamai | AS16625 | 20M+ | 23.0.0.0/8 |

### Cloud Infrastructure Segmentation

**Cloud Provider IP Ranges:**

Major cloud providers publish official IP ranges:

**AWS:**
```bash
# Download AWS IP ranges
wget https://ip-ranges.amazonaws.com/ip-ranges.json

# Extract EC2 ranges for us-east-1
cat ip-ranges.json | jq -r '.prefixes[] | select(.service=="EC2" and .region=="us-east-1") | .ip_prefix' > aws-ec2-us-east-1.txt

# Scan AWS EC2 instances
prtip -sS -p 80,443,22 -iL aws-ec2-us-east-1.txt -oN aws-ec2-scan.txt
```

**Azure:**
```bash
# Download Azure IP ranges
wget https://www.microsoft.com/en-us/download/details.aspx?id=56519

# Extract regions and scan
# (Azure JSON format varies, adjust accordingly)
```

**GCP:**
```bash
# Google Cloud publishes IP ranges via DNS
# Use dig or nslookup to resolve _cloud-netblocks.googleusercontent.com
```

### Geographic Segmentation

**Regional Internet Registries (RIRs):**

| RIR | Region | IP Blocks | Example CIDR |
|-----|--------|-----------|--------------|
| **ARIN** | North America | 1.5B IPs | 23.0.0.0/8, 44.0.0.0/8 |
| **RIPE NCC** | Europe, Middle East | 800M IPs | 2.0.0.0/8, 31.0.0.0/8 |
| **APNIC** | Asia-Pacific | 900M IPs | 1.0.0.0/8, 14.0.0.0/8 |
| **LACNIC** | Latin America | 200M IPs | 177.0.0.0/8, 179.0.0.0/8 |
| **AFRINIC** | Africa | 100M IPs | 41.0.0.0/8, 102.0.0.0/8 |

**Example: Scan ARIN-allocated space:**

```bash
# Scan representative /8 blocks from ARIN
for block in 23 44 52 54 72; do
    prtip -sS -p 80,443 $block.0.0.0/8 -oN arin-$block.txt &
done
```

---

## Network Capacity Management

### Rate Limiting for Internet Scans

**ISP Rate Limiting:**

Most ISPs implement rate limiting to prevent abuse:

| Connection Type | Typical Limit | Recommended Rate | Notes |
|-----------------|---------------|------------------|-------|
| Home Broadband | 10-50K pps | 5K pps | Conservative |
| Business Fiber | 50-200K pps | 20K pps | Moderate |
| Data Center | 200K-1M pps | 100K pps | Aggressive |
| Cloud Instance | Varies | 50K pps | Cloud provider limits |

**ProRT-IP Rate Limiting:**

```bash
# Conservative (home broadband)
prtip --max-rate 5000 -sS -p 80,443 target-range

# Moderate (business fiber)
prtip --max-rate 20000 -T3 -sS -p 80,443 target-range

# Aggressive (data center, 10GbE)
prtip --max-rate 100000 -T4 -sS -p 80,443 target-range

# Maximum (internet-scale, dedicated infrastructure)
prtip --max-rate 500000 -T5 -sS -p 80 0.0.0.0/8
```

**Adaptive Rate Limiting:**

ProRT-IP's V3 rate limiter automatically adapts to network conditions:

- **Convergence algorithm:** Adjusts batch size based on observed throughput
- **ICMP error monitoring:** Backs off if network unreachable errors detected
- **-1.8% overhead:** Faster than unlimited mode (industry-leading)

**Configuration:**

```bash
# Enable adaptive rate limiting (default)
prtip -sS -p 80,443 target-range

# Manual rate limit (override adaptive)
prtip --max-rate 10000 -sS -p 80,443 target-range

# Timing templates (set rate limits)
prtip -T3 -sS -p 80,443 target-range  # Normal: 1-5K pps
prtip -T4 -sS -p 80,443 target-range  # Aggressive: 5-10K pps
prtip -T5 -sS -p 80,443 target-range  # Insane: 10-50K pps
```

### Bandwidth Capacity Planning

**Packet Size Estimation:**

```
Ethernet Frame:
  ├─ Ethernet Header: 14 bytes
  ├─ IP Header: 20 bytes (IPv4) or 40 bytes (IPv6)
  ├─ TCP Header: 20 bytes (minimal) or 32-60 bytes (with options)
  ├─ Payload: 0 bytes (SYN scan)
  └─ Ethernet CRC: 4 bytes

Total: 58 bytes (IPv4 SYN, minimal TCP options)
Total: 78 bytes (IPv6 SYN, minimal TCP options)
```

**Bandwidth Formula:**

```
Bandwidth (Mbps) = Packet_Rate (pps) × Packet_Size (bytes) × 8 / 1,000,000

Examples:
10,000 pps × 60 bytes × 8 = 4.8 Mbps
100,000 pps × 60 bytes × 8 = 48 Mbps
1,000,000 pps × 60 bytes × 8 = 480 Mbps
```

**Bandwidth Requirements Table:**

| Packet Rate | IPv4 Bandwidth | IPv6 Bandwidth | NIC Requirement |
|-------------|----------------|----------------|-----------------|
| 1K pps | 0.5 Mbps | 0.6 Mbps | 1 Mbps+ |
| 10K pps | 4.8 Mbps | 6.2 Mbps | 10 Mbps+ |
| 100K pps | 48 Mbps | 62 Mbps | 100 Mbps+ |
| 1M pps | 480 Mbps | 624 Mbps | 1 Gbps+ |
| 10M pps | 4.8 Gbps | 6.2 Gbps | 10 Gbps+ |

### Handling ICMP Rate Limiting

**ICMP Unreachable Throttling:**

Linux kernel limits ICMP unreachable messages to ~200/second by default:

```bash
# Check current ICMP rate limit
sysctl net.ipv4.icmp_ratelimit
# Output: net.ipv4.icmp_ratelimit = 1000 (milliseconds between ICMP messages)

# Disable ICMP rate limiting (caution: can overwhelm network)
sudo sysctl -w net.ipv4.icmp_ratelimit=0

# Reduce rate limiting (100ms = 10/second)
sudo sysctl -w net.ipv4.icmp_ratelimit=100
```

**Impact on UDP Scans:**

UDP scanning relies on ICMP port unreachable messages to detect closed ports:

```
Rate Limit: 200 ICMP/second
UDP Scan Throughput: Max 200 ports/second (10-100x slower than TCP)

Example: 1,000 UDP ports
TCP SYN scan: 1,000 ports / 10,000 pps = 0.1 seconds
UDP scan: 1,000 ports / 200 pps = 5 seconds (50x slower)
```

**Mitigation Strategies:**

1. **Focus on Known UDP Services:**
```bash
# Scan only DNS, SNMP, NTP (high-value UDP services)
prtip -sU -p 53,161,123,500 target-range
```

2. **Increase Timeout for UDP:**
```bash
# Allow more time for ICMP responses
prtip -sU -p 53,161,123 --host-timeout 30s target-range
```

3. **Accept Slower Scan Rates:**
```bash
# UDP scans inherently slow, plan accordingly
prtip -sU -p 1-1000 target-range  # Expect 5-50 seconds per host
```

---

## State Management at Scale

### Streaming Results to Disk

**Architecture:**

Streaming results eliminates memory constraints for internet-scale scans:

```
Scan Engine → Result Channel → Async Writer → Disk
     ↓
  (No buffering, immediate write)
```

**Implementation:**

```bash
# Stream to JSON file (recommended for large scans)
prtip -sS -p 80,443 0.0.0.0/8 -oN scan-results.json

# Stream to database (SQLite, batched transactions)
prtip -sS -p 80,443 0.0.0.0/8 --output-db scan.db

# Stream to multiple formats
prtip -sS -p 80,443 0.0.0.0/8 -oA scan  # Creates .txt, .json, .xml, .gnmap
```

**Memory Footprint:**

| Mode | Buffering | Memory Usage | Notes |
|------|-----------|--------------|-------|
| **In-Memory** | All results buffered | O(n × p) | Limited to available RAM |
| **Streaming** | Async write buffer (1K results) | O(1) | Unlimited target count |
| **Database** | Transaction batch (10K results) | O(1) | Indexed queries |

**Database Batching:**

```
Transaction batching reduces disk I/O overhead:

Without batching: 1,000,000 results × 5ms/write = 5,000 seconds
With batching (10K/tx): 100 transactions × 50ms/tx = 5 seconds

Speedup: 1,000x faster
```

**Configuration:**

```bash
# Adjust batch size (default: 10,000 results/transaction)
prtip -sS -p 80,443 target-range --db-batch-size 5000
```

### Memory Optimization Techniques

**Buffer Pooling:**

Pre-allocated buffer pools eliminate allocation overhead:

```
Packet Buffer Pool:
  ├─ 1,500-byte buffers (MTU size)
  ├─ Pre-allocated at startup (1,000 buffers)
  ├─ Reused across packets (zero allocation)
  └─ 30-40% faster than dynamic allocation
```

**Zero-Copy Packet Building:**

```rust
// Zero-copy packet crafting (v0.3.8+)
let packet = packet_pool.acquire();  // Borrow from pool
craft_syn_packet(&mut packet, target_ip, target_port);
send_packet(packet);                 // Automatically returned to pool
```

**Batch Processing:**

Process targets in batches to release memory incrementally:

```bash
# Process 64 hosts at a time (default)
prtip -sS -p 1-1000 192.168.0.0/16

# Larger batches (faster, more memory)
prtip --max-hostgroup 256 -sS -p 1-1000 192.168.0.0/16

# Smaller batches (slower, less memory)
prtip --max-hostgroup 16 -sS -p 1-1000 192.168.0.0/16
```

**Memory Usage Formula:**

```
Peak Memory = Base_overhead + (Batch_size × Ports × Per_connection_overhead)

Example: 256 batch size, 1,000 ports
Peak Memory = 2 MB + (256 × 1,000 × 8 bytes) / 1,048,576
            = 2 MB + 1.95 MB
            = 3.95 MB ≈ 4 MB
```

---

## Performance Optimization

### Timing Templates for Large-Scale Scans

**Template Selection Matrix:**

| Environment | Recommended Template | Rate | Duration (1M hosts, 10 ports) |
|-------------|---------------------|------|------------------------------|
| **Localhost** | T5 (Insane) | 50K pps | 200 seconds (~3 min) |
| **LAN (1 Gbps)** | T4 (Aggressive) | 10K pps | 1,000 seconds (~17 min) |
| **Internet** | T3 (Normal) | 5K pps | 2,000 seconds (~33 min) |
| **Stealth** | T2 (Polite) | 1K pps | 10,000 seconds (~2.8 hours) |
| **IDS Evasion** | T0 (Paranoid) | 10 pps | 1,000,000 seconds (~11 days) |

**Usage:**

```bash
# Internet-wide HTTP/HTTPS discovery (aggressive, 6-minute full IPv4 sweep)
prtip -T5 --max-rate 100000 -sS -p 80,443 0.0.0.0/8

# ASN-based scan (moderate, minimize detection)
prtip -T3 --max-rate 10000 -sS -p 80,443,8080 <ASN-prefixes.txt>

# Stealth scan (slow, evade IDS/IPS)
prtip -T2 --max-rate 1000 --randomize-hosts -sS -p 80,443 target-range
```

### NUMA Optimization for Multi-Socket Systems

**When to Use NUMA:**

✅ **Use NUMA if:**
- Dual-socket or quad-socket system (2+ physical CPUs)
- High-throughput scans (>100K packets/second)
- Long-running scans (>1 hour)
- CPU/memory intensive workloads (service detection, OS fingerprinting)

❌ **Don't Use NUMA if:**
- Single-socket system (no benefit, slight overhead)
- Low-throughput scans (<10K packets/second)
- Short scans (<1 minute)

**Performance Benefits:**

| System Type | Expected Improvement | Cache Miss Reduction |
|-------------|---------------------|---------------------|
| Single-Socket | <5% (negligible) | <2% |
| Dual-Socket | 20-30% faster | 15-25% |
| Quad-Socket | 30-40% faster | 25-35% |

**Configuration:**

```bash
# Enable NUMA optimization (auto-detects topology)
prtip --numa -sS -p 1-65535 10.0.0.0/16 --max-rate 1000000

# Check NUMA availability
numactl --hardware
```

**How It Works:**

```
NUMA Node 0 (Cores 0-7):   TX Thread (core 0) + Workers 0, 2, 4, 6, ...
NUMA Node 1 (Cores 8-15):  Workers 1, 3, 5, 7, ...

Benefits:
- TX thread has local access to NIC (node 0)
- Workers evenly distributed (8 per node)
- Memory bandwidth: 2x aggregate (both nodes utilized)
```

### Batch System Calls (Linux)

**sendmmsg/recvmmsg Batching:**

Linux supports batched system calls for efficient I/O:

```
Without batching: 1,000 packets = 1,000 syscalls
With batching (64 packets/batch): 1,000 packets = 16 syscalls

Syscall reduction: 98.4% (1,000 → 16)
Throughput improvement: 2-5x at high packet rates
```

**Configuration:**

```bash
# Adjust batch size (default: 64 packets)
prtip --batch-size 128 -sS -p 80,443 target-range

# Disable batching (compatibility mode)
prtip --batch-size 1 -sS -p 80,443 target-range
```

**Optimal Batch Sizes:**

| Batch Size | Syscall Reduction | Latency | Use Case |
|------------|------------------|---------|----------|
| 16 | ~95% | Low | Latency-sensitive |
| 64 | ~98% | Balanced | **Recommended** |
| 128 | ~99% | Higher | Maximum throughput |

---

## Error Handling and Recovery

### Timeout Management

**Timeout Hierarchy:**

ProRT-IP implements multi-level timeouts for reliability:

```
Global Scan Timeout (--scan-timeout)
  ├─ Host Timeout (--host-timeout)
  │   ├─ Port Timeout (--port-timeout)
  │   └─ Retries (--max-retries)
  └─ Probe Timeout (service detection)
```

**Configuration:**

```bash
# Internet scan with conservative timeouts
prtip -sS -p 80,443 target-range \
    --scan-timeout 6h \
    --host-timeout 5m \
    --port-timeout 3s \
    --max-retries 2

# LAN scan with aggressive timeouts
prtip -sS -p 1-1000 192.168.1.0/24 \
    --scan-timeout 10m \
    --host-timeout 30s \
    --port-timeout 500ms \
    --max-retries 1
```

**Timeout Recommendations:**

| Environment | Scan Timeout | Host Timeout | Port Timeout | Retries |
|-------------|--------------|--------------|--------------|---------|
| Localhost | 1 hour | 1 minute | 100ms | 0 |
| LAN | 2 hours | 2 minutes | 500ms | 1 |
| Internet | 12 hours | 5 minutes | 3 seconds | 2 |
| Satellite/Mobile | 24 hours | 10 minutes | 10 seconds | 3 |

### Retry Logic

**Exponential Backoff:**

```
Retry 1: Wait 1 second
Retry 2: Wait 2 seconds (exponential)
Retry 3: Wait 4 seconds
Retry 4: Wait 8 seconds (max: 8 seconds)
```

**Configuration:**

```bash
# Enable retries with exponential backoff
prtip -sS -p 80,443 target-range --max-retries 3

# Disable retries (fast, may miss results)
prtip -sS -p 80,443 target-range --max-retries 0
```

**Retry Strategy:**

| Scenario | Recommended Retries | Reasoning |
|----------|-------------------|-----------|
| Localhost | 0 | No packet loss expected |
| LAN | 1 | Minimal packet loss |
| Internet | 2 | Moderate packet loss |
| Unreliable Link | 3 | High packet loss |

### Partial Result Preservation

**Graceful Shutdown:**

ProRT-IP preserves partial results on interruption:

```bash
# Start scan (Ctrl+C to interrupt)
prtip -sS -p 1-65535 0.0.0.0/8 -oN scan.txt

# Interrupt with Ctrl+C
^C
[!] Received SIGINT, shutting down gracefully...
[+] Flushing 1,234 results to disk...
[✓] Partial results saved to scan.txt (1,234 hosts)
```

**Resume Capabilities:**

```bash
# Save progress to checkpoint
prtip -sS -p 80,443 0.0.0.0/8 --checkpoint scan.checkpoint

# Resume from checkpoint (if interrupted)
prtip --resume scan.checkpoint
```

---

## Real-World Examples

### Example 1: Internet-Wide HTTP/HTTPS Discovery

**Objective:** Find all web servers on the public internet (IPv4 space)

**Command:**

```bash
# Phase 1: Stateless discovery (6 minutes @ 100K pps)
prtip -T5 --max-rate 100000 -sS -p 80,443 0.0.0.0/0 \
    -oN internet-http-discovery.txt \
    --randomize-hosts \
    --scan-timeout 12h
```

**Expected Results:**

```
Total IPs: 4,294,967,296 (all IPv4)
Scan duration: ~360 seconds (6 minutes) @ 100K pps
Expected responsive hosts: ~50M (1-2% response rate)
Result file size: ~5 GB (text format)
```

**Considerations:**

- **Legal:** Requires authorization or research exemption
- **ISP:** May throttle or block high-rate scans
- **Detection:** Will trigger IDS/IPS systems globally
- **Ethics:** Consider impact on target networks

### Example 2: AWS EC2 Instance Discovery

**Objective:** Map exposed services on Amazon AWS EC2 instances

**Command:**

```bash
# Step 1: Download AWS IP ranges
wget https://ip-ranges.amazonaws.com/ip-ranges.json

# Step 2: Extract EC2 ranges (all regions)
cat ip-ranges.json | jq -r '.prefixes[] | select(.service=="EC2") | .ip_prefix' > aws-ec2-prefixes.txt

# Step 3: Scan for common services
prtip -sS -p 22,80,443,3389,8080,8443 -iL aws-ec2-prefixes.txt \
    -oN aws-ec2-scan.txt \
    --max-rate 20000 \
    -T3

# Step 4: Service detection on responsive hosts
prtip -sV -iL aws-ec2-scan.txt -oN aws-ec2-services.txt
```

**Expected Duration:**

```
AWS EC2 IPs: ~100M prefixes
Scan rate: 20,000 pps
Ports: 6 ports
Duration: (100M × 6) / 20,000 = 30,000 seconds (~8.3 hours)
```

### Example 3: Data Center Subnet Audit

**Objective:** Comprehensive audit of corporate data center (/16 subnet)

**Command:**

```bash
# Phase 1: Host discovery (find live hosts)
prtip -sn 10.0.0.0/16 -oN dc-live-hosts.txt

# Phase 2: Port scan (top 1000 ports)
prtip -sS -p 1-1000 -iL dc-live-hosts.txt -oN dc-port-scan.txt

# Phase 3: Service detection
prtip -sV -iL dc-port-scan.txt -oN dc-services.txt

# Phase 4: OS fingerprinting
prtip -O -iL dc-live-hosts.txt -oN dc-os-fingerprints.txt
```

**Expected Duration:**

```
Phase 1 (Discovery): 65,536 hosts / 10,000 pps = 7 seconds
Phase 2 (Port Scan): Assume 10% live (6,554 × 1,000 ports / 10,000 pps) = 655 seconds (~11 min)
Phase 3 (Service Detection): 6,554 hosts × 1.5x overhead = 983 seconds (~16 min)
Phase 4 (OS Fingerprinting): 6,554 hosts × 1.3x overhead = 852 seconds (~14 min)

Total: ~42 minutes (hybrid mode efficiency)
```

### Example 4: Continuous Internet Monitoring

**Objective:** Track changes in exposed services over time

**Architecture:**

```
Cron Job (Daily)
  ├─ Scan internet-wide (0.0.0.0/0, ports 80,443)
  ├─ Store results in database (timestamped)
  ├─ Compare with previous day's results
  └─ Generate diff report (new services, removed services)
```

**Implementation:**

```bash
#!/bin/bash
# daily-internet-scan.sh

DATE=$(date +%Y-%m-%d)
OUTPUT_DIR="/data/internet-scans/$DATE"
mkdir -p $OUTPUT_DIR

# Run scan
prtip -T5 --max-rate 100000 -sS -p 80,443 0.0.0.0/0 \
    --output-db $OUTPUT_DIR/scan.db \
    --scan-timeout 12h

# Import to database
sqlite3 /data/internet-monitor.db <<EOF
ATTACH DATABASE '$OUTPUT_DIR/scan.db' AS daily;
INSERT INTO scans (date, results) SELECT '$DATE', * FROM daily.results;
DETACH DATABASE daily;
EOF

# Generate diff report
python3 /scripts/generate-diff.py --date $DATE
```

**Cron Configuration:**

```bash
# Run daily at 2 AM UTC
0 2 * * * /scripts/daily-internet-scan.sh >> /var/log/internet-scan.log 2>&1
```

---

## Monitoring and Progress Tracking

### Real-Time Statistics

**Verbose Mode:**

```bash
# Enable verbose output for real-time statistics
prtip -v -sS -p 80,443 target-range

# Very verbose (packet-level details)
prtip -vv -sS -p 1-1000 target-range

# Extremely verbose (debug logging)
prtip -vvv -sS -p 80 target-range
```

**Output Example:**

```
[✓] Starting TCP SYN scan of 0.0.0.0/8
[+] Targets: 16,777,216 hosts, Ports: 2 (80, 443)
[+] Expected duration: ~5.6 minutes @ 100,000 pps
[+] Progress: 1,234,567 / 16,777,216 (7.4%) | Rate: 98,234 pps | ETA: 4m 23s
[+] Results: 12,345 open, 45,678 closed, 1,176,544 filtered
[+] Responsive hosts: 12,345 (1.0% response rate)
```

### Event Logging

**Event System (Sprint 5.5.3):**

ProRT-IP provides comprehensive event logging for monitoring:

```bash
# Enable event logging to file
prtip -sS -p 80,443 target-range --event-log scan-events.jsonl

# Enable event logging to database
prtip -sS -p 80,443 target-range --event-db scan-events.db
```

**Event Types:**

- **Scan Events:** Start, stop, pause, resume
- **Host Events:** Discovery, timeout, error
- **Port Events:** State change (open, closed, filtered)
- **Service Events:** Detection, version identified
- **Error Events:** Network errors, timeouts, exceptions

**Event Format (JSONL):**

```json
{"timestamp":"2025-11-15T12:34:56Z","type":"scan_start","targets":16777216,"ports":[80,443]}
{"timestamp":"2025-11-15T12:35:01Z","type":"host_discovered","ip":"1.2.3.4","rtt":23}
{"timestamp":"2025-11-15T12:35:01Z","type":"port_open","ip":"1.2.3.4","port":80,"service":"http"}
{"timestamp":"2025-11-15T12:40:23Z","type":"scan_complete","duration":267,"responsive_hosts":12345}
```

### Progress Estimation

**ETA Calculation:**

```
Current Progress: 1,234,567 hosts scanned
Total Targets: 16,777,216 hosts
Scan Rate: 98,234 pps (measured over last 10 seconds)

Remaining: 16,777,216 - 1,234,567 = 15,542,649 hosts
ETA: 15,542,649 / 98,234 = 158 seconds ≈ 2m 38s
```

**Adaptive ETA:**

ProRT-IP recalculates ETA every 10 seconds based on recent throughput:

```
Initial estimate: 5m 36s (@ 100K pps)
After 1 minute: 4m 52s (@ 97K pps, congestion detected)
After 2 minutes: 4m 23s (@ 101K pps, congestion cleared)
After 3 minutes: 4m 10s (@ 103K pps, optimal rate achieved)
```

---

## Best Practices

### Pre-Scan Checklist

**Before Large-Scale Scans:**

- [ ] **Authorization:** Written permission for all target networks
- [ ] **Scope Definition:** CIDR blocks, port lists, scan types documented
- [ ] **Rate Limits:** Configured for network type (5K-100K pps)
- [ ] **Exclusion Lists:** Military, government, critical infrastructure excluded
- [ ] **System Resources:** ulimit increased, NUMA enabled (multi-socket)
- [ ] **Monitoring:** Event logging, verbose mode, progress tracking enabled
- [ ] **Error Handling:** Timeouts, retries, checkpoint/resume configured
- [ ] **Results Storage:** Streaming to disk, database batching enabled
- [ ] **Test Run:** Small subset scan (1K hosts) to validate configuration
- [ ] **Abuse Contact:** Reverse DNS, abuse email, website configured

### During Scan Checklist

**While Scanning:**

- [ ] **Monitor Throughput:** Watch for rate drops (congestion, filtering)
- [ ] **Check Errors:** Review event log for ICMP errors, timeouts
- [ ] **Verify Results:** Spot-check responsive hosts for accuracy
- [ ] **Network Health:** Monitor NIC utilization, packet loss, errors
- [ ] **System Health:** Monitor CPU, memory, disk I/O, swap usage
- [ ] **Respect Limits:** Reduce rate if abuse complaints received
- [ ] **Progress Tracking:** Verify ETA accuracy, adjust if needed
- [ ] **Partial Results:** Periodically flush results to disk (checkpoint)

### Post-Scan Checklist

**After Completion:**

- [ ] **Verify Completion:** Confirm all targets scanned (check event log)
- [ ] **Analyze Results:** Review response rate, service distribution
- [ ] **Error Review:** Identify failed hosts, network issues
- [ ] **Performance Analysis:** Compare actual vs. estimated duration
- [ ] **Result Validation:** Spot-check open ports, service detections
- [ ] **Archive Results:** Store with metadata (date, scope, configuration)
- [ ] **Documentation:** Record findings, anomalies, lessons learned
- [ ] **Cleanup:** Remove temporary files, close database connections

### Responsible Scanning

**Ethical Principles:**

1. **Minimize Impact:** Use rate limiting to avoid network disruption
2. **Respect Privacy:** Don't collect unnecessary data (banners only)
3. **Honor Exclusions:** Respect robots.txt, security.txt, abuse requests
4. **Respond to Complaints:** Monitor abuse email, respond within 24 hours
5. **Document Methodology:** Publish scan parameters, research goals
6. **Disclose Responsibly:** Follow coordinated disclosure for vulnerabilities
7. **Respect Laws:** Comply with CFAA, CMA, and local regulations

**Exclusion Lists:**

```bash
# U.S. Department of Defense (.mil)
--exclude 6.0.0.0/8,7.0.0.0/8,11.0.0.0/8,21.0.0.0/8,22.0.0.0/8

# U.S. Government (.gov)
--exclude 3.0.0.0/8,12.0.0.0/8,13.0.0.0/8,14.0.0.0/8

# Critical Infrastructure (power grids, water systems)
--exclude-file critical-infrastructure.txt

# Private Networks (RFC 1918)
--exclude 10.0.0.0/8,172.16.0.0/12,192.168.0.0/16
```

---

## See Also

**User Guide:**
- [Basic Usage](../user-guide/basic-usage.md) - Command-line fundamentals
- [Timing & Performance](../user-guide/timing-performance.md) - T0-T5 timing templates
- [Output Formats](../user-guide/output-formats.md) - JSON, XML, database output

**Feature Guides:**
- [Port Scanning](../features/index.md) - Port specification and strategies
- [Rate Limiting](../features/rate-limiting.md) - Adaptive rate limiting V3
- [IPv6 Support](../features/ipv6.md) - IPv6 scanning capabilities

**Advanced Topics:**
- [Performance Tuning](./performance-tuning.md) - System and ProRT-IP optimization
- [Distributed Scanning](./distributed-scanning.md) - Multi-instance coordination
- [Database Usage](./database-usage.md) - SQLite/PostgreSQL integration
- [Automation](./automation.md) - Scripting and CI/CD integration

**Reference:**
- [Architecture Documentation](../00-ARCHITECTURE.md) - Stateless/stateful/hybrid modes
- [Performance Characteristics](../34-PERFORMANCE-CHARACTERISTICS.md) - Scaling formulas and benchmarks
- [Capacity Planning](../21-PERFORMANCE-GUIDE.md) - Hardware requirements

**External Resources:**
- **Masscan:** https://github.com/robertdavidgraham/masscan (stateless architecture inspiration)
- **Nmap:** https://nmap.org/book/performance.html (performance optimization)
- **RIR Statistics:** https://www.arin.net/reference/research/statistics/ (IP allocation data)
- **ASN Lookup:** https://bgp.tools/ (autonomous system number database)
- **CFAA:** https://www.law.cornell.edu/uscode/text/18/1030 (U.S. computer fraud law)

---

**Last Updated:** 2025-11-15
**ProRT-IP Version:** v0.5.2
