# ProRT-IP WarScan: Comprehensive Technical Specification and Implementation Document

## Executive summary: Building a production-grade network scanner in Rust

ProRT-IP WarScan requires implementing sophisticated network scanning capabilities combining the speed of Masscan/ZMap (10+ million packets/second) with the depth of Nmap's service detection and OS fingerprinting. **The most critical architectural decision is choosing between stateful scanning (accurate, slower) and stateless scanning (fast, less accurate).** For a modern implementation, a **hybrid approach offers optimal results**: stateless discovery at Masscan speeds followed by stateful service enumeration using Nmap techniques. Rust's async/await with Tokio provides the performance foundation, achieving 152,258 requests/second in HTTP benchmarks—a 34% improvement over previous schedulers. The implementation must handle cross-platform raw packet access (Linux AF_PACKET, Windows Npcap, macOS BPF), implement proper privilege dropping after socket creation, and use lock-free data structures for coordination at scale. This document synthesizes technical implementation details from Nmap, Masscan, ZMap, RustScan, and production security tools to provide actionable specifications for building ProRT-IP WarScan.

## Network scanner architecture fundamentals

Modern network scanners operate on a spectrum between two architectural extremes. **Stateful scanners like Nmap maintain extensive per-target state**, tracking every probe with sequence numbers, source/destination ports, and ID fields for response recognition. Nmap's ultra_scan engine adjusts speed dynamically based on network conditions, implementing TCP-style congestion control with per-host and per-group windows. This architecture achieves high accuracy through adaptive retransmission, RTT estimation, and timeout calculation, but limits throughput to roughly 300,000 packets per second even with aggressive tuning.

**Stateless scanners like Masscan and ZMap eliminate connection tracking entirely**, achieving 10-25 million packets per second by never waiting for responses before continuing. Masscan uses a custom userland TCP/IP stack that bypasses the kernel completely, employing SYN cookies encoded in source ports to validate responses without state lookups. ZMap takes this further with a cyclic multiplicative group algorithm for address generation, using the mathematical property that (Z/pZ)× forms a cyclic group where p = 2³² + 15, enabling complete IPv4 iteration with only three stored integers (primitive root, first address, current address). The trade-off manifests in accuracy: ZMap observed **97% hit rates at 4 Mpps dropping to 63% at 14.23 Mpps** due to upstream network congestion and packet drops.

For ProRT-IP WarScan, the optimal architecture combines both approaches in distinct phases. The initial discovery phase employs stateless scanning for rapid port enumeration across large address spaces, while subsequent service detection uses stateful connections with Nmap's proven version detection methodology. This mirrors RustScan's successful design, which scans all 65,535 ports in approximately 3 seconds before piping discovered services to Nmap for detailed analysis.

## Core scanning techniques implementation

### TCP SYN scanning: The foundation

**TCP SYN scanning remains the default technique for most security assessments**, balancing stealth with accuracy. The implementation constructs a TCP packet with only the SYN flag set, uses a random sequence number, and sends it to each target port. Response analysis follows a clear state machine: **SYN/ACK responses indicate open ports**, RST packets signal closed ports, and absence of response after retransmissions suggests filtered ports. Critical to stealth operation, the scanner sends an RST packet upon receiving SYN/ACK, never completing the three-way handshake and avoiding connection logs on many target systems.

The packet crafting process requires constructing complete frames from Ethernet layer upward. For IPv4 over Ethernet, this means building a 14-byte Ethernet header (destination MAC, source MAC, EtherType 0x0800), followed by a 20-byte IP header with configurable TTL and Don't Fragment bit, and finally a 20-byte TCP header with the SYN flag set at bit position 1. Sequence numbers should use cryptographically random generation to prevent prediction attacks, and the TCP window size typically ranges from 1024-65535 bytes to appear legitimate.

Implementation in Rust using the pnet packet construction library provides memory safety guarantees while maintaining performance. The etherparse crate offers an even faster zero-allocation alternative with its PacketBuilder API, automatically calculating checksums for TCP and UDP. **Checksum calculation must include the pseudo-header** containing source IP, destination IP, protocol (6 for TCP), and TCP segment length—a frequent source of bugs in manual implementations.

### Advanced scan types: Exploiting protocol edge cases

**FIN, NULL, and Xmas scans exploit a loophole in RFC 793** where packets with unexpected flags to closed ports must trigger RST responses, while open ports should silently drop the packets. NULL scan sends packets with all flag bits set to zero, FIN scan sets only the FIN flag, and Xmas scan simultaneously sets FIN, PSH, and URG flags (appearing like a Christmas tree in packet analyzers). These techniques bypass stateless firewalls that only filter SYN packets, but critically **fail against Windows systems and most Cisco devices**, which send RST regardless of port state.

UDP scanning presents unique challenges due to the connectionless nature of the protocol. Since most UDP services don't respond to empty probes, the scanner must rely on **ICMP port unreachable messages (type 3, code 3) to identify closed ports**. Open ports typically remain silent unless protocol-specific payloads trigger responses. Nmap's nmap-service-probes database contains hundreds of protocol-specific UDP payloads: DNS queries for port 53, SNMP community string requests for port 161, and NetBIOS queries for port 137. UDP scanning operates 10-100x slower than TCP due to ICMP rate limiting on most operating systems (Linux defaults to 1 error message per second).

### Idle scanning: Achieving complete anonymity

**Idle scanning (zombie scanning) represents the pinnacle of stealth techniques**, completely obscuring the attacker's IP address by exploiting a third-party system's predictable IP ID generation. The three-step process repeats for each target port: probe the zombie's IP ID and record it, forge a SYN packet appearing to come from the zombie to the target port, then re-probe the zombie's IP ID. **An increment of 2 indicates an open port** (zombie sent RST in response to the target's SYN/ACK), while an increment of 1 suggests closed or filtered (no response triggered, zombie only replied to scanner's probe).

The technique's effectiveness depends entirely on zombie selection. Ideal zombies exhibit **incremental or "broken little-endian incremental" IP ID generation** (adding 256 per packet instead of 1, common in Windows systems), maintain global rather than per-host ID counters, remain mostly idle with minimal network traffic, and provide low latency to both attacker and target. Simple network devices like printers, IoT devices, and legacy Windows XP systems often make excellent zombies. Modern defenses include per-host ID counters (Solaris, current Linux) and randomized sequences (OpenBSD default).

Implementation requires careful timing and reliability checking. Nmap's idle_scan.cc parallelizes scans across groups of up to 100 ports, using binary search when multiple open ports are detected to verify results match expectations. Dynamic adjustment of group size and timing based on zombie reliability prevents false positives from network jitter or zombie activity.

## Service and OS detection methodology

### Version detection: The nmap-service-probes database

**Service version detection operates through a probe-and-match system** defined in the nmap-service-probes database, a flat-file format containing protocol-specific queries and response patterns. Each probe specification includes the protocol (TCP/UDP), probe name, actual data to send, applicable ports, SSL variants, and timeout values. For example, the GetRequest probe sends "GET / HTTP/1.0\r\n\r\n" to web servers, with pattern matches extracting server software, version numbers, and additional metadata using regex capture groups.

The detection process follows an intensity-based escalation. **At intensity level 0-1, only registered probes** (specific probes designated for particular ports like DNS for 53) execute. Level 2 (--version-light) adds quick, common service probes. Default level 7 balances coverage with speed, while level 9 (--version-all) exhaustively tries every probe against every port. NULL probes always execute first, as many services announce themselves immediately upon connection (SSH sends "SSH-2.0-OpenSSH_8.2p1", FTP sends "220 ProFTPD Server").

For SSL/TLS encrypted services, the detection engine performs a full TLS handshake before sending probes through the encrypted channel, outputting results in the format "ssl/http" or "ssl/smtp". This requires OpenSSL support and adds connection establishment overhead but enables detection of increasingly common encrypted services. The version intensity and probe selection dramatically impact scan duration: a full intensity 9 scan against an unresponsive port may timeout 50+ probes, while intensity 2 typically completes in under a second per port.

### OS fingerprinting: 16 probe packets reveal operating systems

**Nmap's OS detection achieves 2,600+ fingerprint accuracy** by sending a carefully crafted sequence of 16 probe packets designed to elicit OS-specific responses. The technique exploits implementation variations in TCP/IP stacks that, while technically RFC-compliant, differ in timing, option handling, field initialization, and error conditions. The probe sequence includes: six TCP SYN packets to an open port spaced 100ms apart (testing ISN generation, TCP options, window sizes, and timestamps), two ICMP echo requests with different TOS and code values, one ECN probe with CWR+ECE flags, six unusual TCP packets to open and closed ports (NULL, SYN+FIN+URG+PSH, ACK with various window sizes and DF bit settings), and one UDP packet to a closed port.

The resulting fingerprint contains dozens of test values extracted from responses. **Key discriminators include:** GCD (greatest common divisor of ISN sequence differences, typically 64000-128000 for most systems), ISR (ISN counter rate per second), TI/CI/II (IP ID generation patterns for TCP/CLOSED/ICMP showing incremental, random, or zero sequences), TS (TCP timestamp algorithm), O (TCP option ordering using M=MSS, W=WScale, T=Timestamp, S=SACK notation), W (window size patterns across probes), and DF (Don't Fragment bit preferences). 

Matching employs a weighted scoring system where unusual behaviors receive more points than common patterns. For instance, the TCP timestamp option carries significant weight because implementation algorithms vary substantially (BSD increments by frequency/100, Linux by frequency/1000), while TTL values receive less weight due to correlation with other fields. The database format encodes all values as hexadecimal or symbolic constants, with ranges indicating acceptable variation (W=4000|8000 means window could be either value).

## Rust networking implementation patterns

### Async runtime architecture with Tokio

**Tokio's scheduler redesign achieved a 10x performance improvement** through several key optimizations that ProRT-IP WarScan should leverage. The work-stealing queue algorithm uses fixed-size per-core queues (256 tasks) with overflow to a global MPMC queue, stealing half a queue at once for better load balancing while throttling concurrent stealers to half the total processor count to reduce contention. Memory optimization reduced allocations from 2 to 1 per task by carefully ordering struct fields: hot data in the Header (cache-line optimized at struct head), the Future itself, and cold data in the Trailer.

The "next task" slot optimization proves particularly effective for network scanning's message-passing patterns. When a task wakes another (common in request/response workflows), the woken task goes into a special LIFO slot on the current worker, keeping data hot in CPU cache and avoiding queue operations. **Reference counting optimization** eliminates atomic increments on wake_by_ref by maintaining a master scheduler list of active tasks, using a single reference count for queue operations rather than per-reference counting.

For network scanning workloads, configure the runtime with worker threads matching physical CPU cores (not logical hyperthreads), use spawn_blocking for CPU-bound operations like cryptographic verification, and leverage tokio::select! for racing operations like timeout enforcement. The runtime builder provides granular control:

```rust
let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_cpus::get_physical())
    .thread_name("scanner-worker")
    .event_interval(61)  // Check global queue every 61 local polls
    .max_blocking_threads(512)
    .build()?;
```

### Packet capture and injection: Cross-platform raw socket access

**Platform-specific packet capture requires careful abstraction** to maintain a single codebase. Linux provides AF_PACKET sockets for layer 2 access requiring CAP_NET_RAW capability, with PACKET_MMAP offering zero-copy performance by mapping kernel buffers to userspace. Advanced users can leverage eBPF/XDP for 24 million packets/second per core by executing filter programs at the earliest RX path point before sk_buff allocation. XDP operates in three modes: Generic (post-skb, slowest), Native (in driver, best performance), and Offloaded (NIC hardware, Netronome only).

Windows mandates Npcap (the modern WinPcap successor) for packet capture, as Microsoft heavily restricted raw socket capabilities starting in Windows XP SP2. The implementation must link against Packet.lib from the Npcap SDK at build time and require Administrator privileges at runtime. Recent Npcap versions (0.993+) load the NPF driver at boot to avoid 90-second network connectivity loss during driver initialization.

macOS and BSD systems use BPF (Berkeley Packet Filter) devices accessed via /dev/bpf*. The ChmodBPF launch daemon creates devices with appropriate permissions (crw-rw---- root:access_bpf) at boot, allowing users in the access_bpf group to capture without root. Setup requires adding users to the group and running the ChmodBPF script provided by Wireshark's installer.

The **pnet crate provides excellent cross-platform abstraction**, automatically using AF_PACKET on Linux, Npcap on Windows, and BPF on BSD/macOS. For raw packet injection, socket2 offers type-safe socket creation across platforms. The pcap crate wraps libpcap/Npcap for both capture and injection, providing BPF filtering capabilities that execute in the kernel for maximum efficiency.

### Lock-free coordination for internet-scale scanning

**Achieving 10+ million packets/second requires eliminating lock contention** in the critical path. Masscan's architecture demonstrates this principle: separate transmit and receive threads communicate through ring buffers rather than mutexes, avoiding cache-line bouncing and kernel involvement. The transmit thread reads configuration, generates packets based on an encrypted index for randomization, and writes directly to the network via the custom TCP/IP stack. The receive thread captures packets via libpcap bypass, parses responses, matches them to transmitted probes using stateless techniques, and updates results.

**The single index variable design** elegantly enables stateless operation. For a scan range of N addresses across M ports, the index i iterates from 0 to N×M. At each iteration, the scanner computes x = encrypt(i) using a symmetric cipher for randomization, then picks the IP address from the target list at position x / M and port at position x mod M. This deterministic generation eliminates state storage while ensuring random target selection to prevent network flooding.

For Rust implementations, the crossbeam crate provides production-ready lock-free data structures including Chase-Lev work-stealing deques. Custom implementations for specific use cases can achieve even better performance—Tokio's scheduler uses atomic operations with Acquire/Release ordering instead of Sequential Consistency, providing near-zero synchronization overhead on x86 where these operations compile to simple loads and stores.

## Performance optimization techniques at scale

### Zero-copy networking and batch operations

**MSG_ZEROCOPY in Linux enables true zero-copy transmission** for large payloads, eliminating per-byte copy costs by pinning pages and sharing buffers between process and network stack. However, the technique only benefits writes exceeding approximately 10 KB due to page pinning and completion notification overhead. For typical network scanning packets (40-100 bytes), traditional copying actually performs better. Enable via setsockopt(fd, SOL_SOCKET, SO_ZEROCOPY, &one, sizeof(one)) and handle completion notifications through the socket's error queue with recvmmsg batching.

**sendmmsg and recvmmsg provide critical performance gains** by batching multiple UDP/TCP messages in a single syscall, reducing context switch overhead from per-packet to per-batch. At packet rates exceeding 1 million per second, syscall overhead becomes the dominant bottleneck. ZMap achieved 14.23 Mpps (96% of 10 GigE theoretical limit) using these techniques combined with parallel address generation sharding that eliminated mutex contention.

Memory-mapped I/O for result storage avoids frequent small writes by mapping result files directly into process address space. Masscan's binary output format writes results immediately without buffering, streaming them to disk as they arrive. For large scan results (millions of hosts), consider SQLite with write-ahead logging (WAL) mode or PostgreSQL with COPY statements for bulk insertion.

### CPU affinity and NUMA optimization

**NUMA (Non-Uniform Memory Access) penalties can reduce performance 10-30%** when threads access memory on distant NUMA nodes. For network scanning, pin interrupt handlers to specific NUMA nodes matching the NIC's PCIe slot location, then bind packet processing threads to the same node. Linux's numactl tool enables process-level binding: `numactl --cpunodebind=0 --membind=0 ./scanner` forces execution on NUMA node 0 with memory allocation from the same node.

Network interface RSS (Receive Side Scaling) distributes incoming packets across multiple hardware queues, each with a dedicated IRQ that should be pinned to a specific CPU. For an 8-queue NIC on NUMA node 0, pin IRQs 0-7 to CPUs 0-7: `echo $((1 << i)) > /proc/irq/$IRQ/smp_affinity`. Combined with proper thread placement, this prevents cross-NUMA traffic and keeps packet data hot in cache.

Profiling with perf and flamegraphs identifies hot paths requiring optimization. Build Rust code with debug symbols in release mode using RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes" cargo build --release, then profile with perf record --call-graph dwarf -F 997. Generate flamegraphs with perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg for visual analysis. The Criterion.rs crate provides statistics-driven benchmarking with automatic regression detection.

## Stealth and evasion capabilities

### Packet fragmentation and IDS evasion

**IP fragmentation splits TCP headers across multiple fragments** to evade packet filters examining only first fragments. The -f flag fragments packets into 8 bytes or less after the IP header, while -f -f uses 16-byte fragments. For a 20-byte TCP header, single fragmentation creates three packets (8+8+4 bytes plus IP headers). Custom MTU values (--mtu) provide precise control, though values must be multiples of 8 per IP specification.

Fragmentation-based evasion exploits timing differences and reassembly inconsistencies between IDS and target systems. **Insertion attacks** send fragments accepted by the IDS but rejected by the target (via TTL manipulation or invalid checksums), causing divergent reassembly states. **Overlapping fragments** with conflicting data at the same offset may reassemble differently depending on whether the target prefers first or last fragment data. Out-of-order fragment delivery confuses stateless inspection systems that don't maintain reassembly buffers.

Modern defenses include Linux's CONFIG_IP_ALWAYS_DEFRAG kernel option and IDS/IPS systems that fully reassemble fragments before inspection. The --send-eth option bypasses the local IP layer to prevent host OS defragmentation from interfering with fragment crafting.

### Timing templates and adaptive scanning

**Nmap's six timing templates (T0-T5) balance stealth against speed.** T0 (Paranoid) serializes scanning with 5-minute delays between probes, enabling IDS evasion but requiring days for large scans. T1 (Sneaky) uses 15-second delays. T2 (Polite) implements 0.4-second delays to reduce bandwidth consumption. T3 (Normal) provides default balanced behavior. **T4 (Aggressive) assumes fast, reliable networks** with max RTT timeout 1250ms, min 100ms, initial 500ms, max retries 6, and 10ms maximum TCP scan delay. T5 (Insane) sacrifices accuracy for speed with 300ms max RTT timeout, 50ms minimum, 250ms initial, only 2 retries, and 5ms scan delay.

Granular timing controls supplement templates: --scan-delay and --max-scan-delay set minimum and maximum probe intervals, --min-rate and --max-rate override congestion control to enforce fixed packet rates, --max-retries limits retransmission attempts, and --host-timeout abandons slow hosts. For production scanning, T4 provides good throughput on modern networks, while T2 suits environments with IDS or bandwidth constraints.

**Adaptive rate limiting** monitors network feedback to automatically adjust send rates. Track response rate versus send rate to detect packet loss: if hit rate falls below threshold (typically 90-95%), reduce rate by 10-20%. If hit rate exceeds threshold with headroom, cautiously increase rate by 5-10%. This maintains maximum throughput while preventing network flooding or triggering rate-based IDS signatures.

### Decoy scanning and source manipulation

**Decoy scanning intermixes real probes with spoofed packets from fake source IPs**, making attacker identification difficult when security teams analyze logs showing scans from dozens of sources simultaneously. Syntax -D decoy1,decoy2,ME,decoy3 specifies decoys with ME representing the real scanner's IP. Positioning ME at position 6+ defeats scanlogd and similar correlation-based detectors. The RND and RND:number options generate random decoy IPs, though care must be taken to avoid randomly selecting active hosts that could be blamed for the scan.

Decoys apply during initial host discovery, port scanning, and OS detection phases, but not version detection or TCP connect scans. ISP egress filtering may drop spoofed source addresses, limiting effectiveness of decoy scanning from many networks. Colocation facilities and enterprise uplinks often permit spoofing, while residential and mobile connections typically filter.

**Source port manipulation** (--source-port or -g) exploits misconfigured firewalls that trust specific ports: 20 (FTP-DATA), 53 (DNS), 80 (HTTP), and 88 (Kerberos) commonly receive privileged treatment. This works with raw socket scans (SYN, UDP) but not DNS requests, TCP connect(), version detection, or script scanning. MAC address spoofing (--spoof-mac) changes source MAC address to a specific value, vendor prefix, or random address, useful for bypassing MAC filtering but only affecting local network segment.

## Database schemas and output formats

### Result storage and retrieval optimization

**Efficient result storage requires normalized schema design** balancing query flexibility against write performance. The core schema uses two tables: scans (id, start_time, end_time, config_json) and scan_results (id, scan_id, target_ip, port, state, service, banner, response_time_ms, timestamp). Indexes on scan_id, target_ip, and port enable fast queries while foreign key constraints maintain referential integrity.

For high-throughput scanning, SQLite with WAL (Write-Ahead Logging) mode provides excellent performance without server overhead. Enable with `PRAGMA journal_mode=WAL` and configure `PRAGMA synchronous=NORMAL` for balanced durability and speed. Batch inserts using transactions: begin transaction, execute 1000-10000 inserts, commit transaction. This reduces fsync calls and dramatically improves write throughput.

PostgreSQL suits multi-user environments and large result sets. Use COPY statements for bulk loading: `COPY scan_results FROM STDIN` with tab-separated or CSV formatting achieves 10-100x faster insertion than individual INSERT statements. Unlogged tables eliminate WAL overhead for temporary scan data, though at the cost of crash recovery. Partitioning by scan_id or timestamp enables efficient archival and query scoping.

### Output format specifications

**XML output** (-oX) provides complete machine-readable results following Nmap's established schema. The structure nests host elements containing status, addresses, hostnames, ports (with service/state/script results), and OS detection data. XML output supports XSL transformation for custom reporting and easy parsing by security orchestration platforms.

**Binary formats** optimize storage and parsing speed. Masscan's binary format enables efficient streaming: write results immediately upon receipt without buffering, then use --readscan to convert to other formats post-scan. This prevents memory exhaustion during internet-scale scans and enables resumption if the scan terminates prematurely.

**PCAPNG format** (pcap next generation) stores packet captures with rich metadata. The structure uses typed blocks: Section Header Block (SHB) with byte-order magic, Interface Description Block (IDB) for each capture interface, and Enhanced Packet Block (EPB) for each packet with interface ID, high-resolution timestamp, captured/original lengths, and packet data. Options allow comments, flags, packet hashes, and custom extensions. Multiple sections support concatenation by simply joining files.

## Security implementation requirements

### Privilege management and capability systems

**Privilege dropping immediately after raw socket creation** constitutes the most critical security measure. The canonical pattern creates privileged resources first (raw sockets, capture handles), then permanently drops privileges via setuid/setgid system calls. On Linux, the capabilities system provides fine-grained permission control: CAP_NET_RAW allows raw socket creation and packet capture, CAP_NET_ADMIN enables network configuration, and CAP_NET_BIND_SERVICE permits binding to privileged ports. Grant capabilities with `sudo setcap cap_net_raw,cap_net_admin=eip /path/to/scanner`.

Order matters critically when dropping privileges: clear supplementary groups first (requires root), drop group privileges (setgid), then finally drop user privileges (setuid). After setuid, the process cannot regain root even if exploited. Verify each step succeeds by checking return codes—failure to drop privileges completely represents a critical security vulnerability.

```rust
pub fn drop_privileges(user: &str, group: &str) -> Result<(), Error> {
    unsafe {
        // Clear supplementary groups (requires root)
        if setgroups(0, std::ptr::null()) != 0 {
            return Err(Error::SetGroupsFailed);
        }
        
        // Drop group (must be before setuid)
        let grp = getgrnam(CString::new(group)?.as_ptr());
        if grp.is_null() || setgid((*grp).gr_gid) != 0 {
            return Err(Error::SetGidFailed);
        }
        
        // Drop user (permanent)
        let pwd = getpwnam(CString::new(user)?.as_ptr());
        if pwd.is_null() || setuid((*pwd).pw_uid) != 0 {
            return Err(Error::SetUidFailed);
        }
    }
    Ok(())
}
```

### Input validation and injection prevention

**Allowlist validation prevents entire classes of vulnerabilities** by explicitly defining acceptable input rather than attempting to blacklist dangerous patterns. For IP addresses, use Rust's standard library IpAddr::parse() which validates IPv4/IPv6 format. For CIDR notation, leverage the ipnetwork crate's IpNetwork::parse(). Port ranges require validation ensuring start > 0, end <= 65535, and start <= end. Reject invalid input immediately at the API boundary before any processing.

Command injection vulnerabilities arise when user input reaches system shells. **Never construct shell commands from user input.** Use Rust's std::process::Command API which spawns processes directly without shell interpretation:

```rust
// UNSAFE - command injection vulnerable
let output = std::process::Command::new("sh")
    .arg("-c")
    .arg(format!("nslookup {}", user_input))
    .output()?;

// SAFE - direct process spawn
let output = std::process::Command::new("nslookup")
    .arg(user_input)
    .output()?;
```

Path traversal prevention requires validating all file paths. Use std::fs::canonicalize() to resolve paths and verify they remain within expected directories. Check for suspicious patterns like "..", absolute paths when relative expected, and null bytes in strings.

### Packet parsing safety and DoS prevention

**Memory-safe packet parsing** in Rust eliminates entire vulnerability classes compared to C implementations. Use pnet or etherparse which perform automatic bounds checking. Both libraries return Option or Result types for parsing operations, forcing explicit handling of malformed packets. Never use panic! in packet parsing code—malformed packets are expected adversarial input, not exceptional conditions.

```rust
fn parse_tcp_safe(data: &[u8]) -> Option<TcpHeader> {
    // Explicit bounds checking
    if data.len() < 20 {
        return None;
    }
    
    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let seq = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    
    // Validate data offset
    let data_offset = (data[12] >> 4) as usize * 4;
    if data_offset < 20 || data_offset > data.len() {
        return None;
    }
    
    Some(TcpHeader { src_port, dst_port, seq, data_offset })
}
```

**Resource exhaustion prevention** requires limits at multiple layers. Bound maximum concurrent scans via semaphores (tokio::sync::Semaphore), implement per-target rate limiting, set maximum scan duration timeouts, limit result buffer sizes, and cap memory allocation. For internet-scale scanning, stream results to disk immediately rather than accumulating in memory. Monitor file descriptor usage and implement limits—Linux defaults often require raising via ulimit or systemd LimitNOFILE.

## Recommended implementation roadmap

### Phase 1: Core infrastructure (weeks 1-3)

Begin with cross-platform packet capture abstraction using the pnet crate for broad compatibility. Implement basic TCP connect scanning as the simplest technique, establishing the async/await patterns with Tokio that will extend to advanced methods. Create the privilege management system with capability-based permissions on Linux and proper setuid alternatives. Build configuration file loading using serde with TOML format for Rust-native integration. Establish the result storage schema in SQLite with indexing for common queries.

### Phase 2: Advanced scanning techniques (weeks 4-6)

Implement TCP SYN scanning using raw sockets with proper checksum calculation including pseudo-headers. Add UDP scanning with protocol-specific payloads for common services (DNS, SNMP, NetBIOS). Develop the service detection engine using the nmap-service-probes format, starting with 20-30 common protocols. Create stealth scan variants (FIN, NULL, Xmas) with proper response interpretation. Build timing templates (T0-T5) with configurable delays and retransmission logic.

### Phase 3: Detection and fingerprinting (weeks 7-10)

Implement OS fingerprinting using Nmap's 16-probe sequence with weighted scoring against the nmap-os-db database. Develop banner grabbing for application-level service identification. Create the service version detection engine with intensity levels and SSL/TLS support. Build protocol-specific probe modules for HTTP, FTP, SSH, SMTP, DNS, and SNMP. Implement heuristic service detection for non-standard ports.

### Phase 4: Performance optimization (weeks 11-13)

Integrate lock-free data structures using crossbeam for scan coordination. Implement connection pooling for stateful scans with health checking and idle timeout. Add adaptive rate limiting based on response rates and error monitoring. Optimize packet batching using sendmmsg/recvmmsg on Linux. Configure NUMA-aware thread placement and IRQ affinity. Profile hot paths with perf and optimize based on flamegraph analysis.

### Phase 5: Advanced features (weeks 14-16)

Develop idle scanning with zombie detection and binary search for port identification. Implement decoy scanning with configurable decoy count and placement. Add packet fragmentation with custom MTU support. Create the plugin system using mlua for Lua scripts. Build comprehensive audit logging with structured output. Implement graceful degradation and error recovery. Conduct security audit and penetration testing.

## Critical dependencies and toolchain

**Core Rust Crates:**
- tokio (1.35+): Async runtime with optimized scheduler
- pnet (0.34+): Cross-platform packet capture and manipulation
- socket2 (0.5+): Cross-platform raw socket API
- etherparse (0.14+): Zero-allocation packet parsing
- clap (4.4+): Command-line argument parsing with derive macros
- serde (1.0+): Serialization framework
- toml (0.8+): Configuration file parsing
- sqlx (0.7+): Async SQL with compile-time query checking
- tracing (0.1+): Structured logging and instrumentation
- crossbeam (0.8+): Lock-free concurrent data structures
- governor (0.6+): Token bucket rate limiting
- mlua (0.9+): Lua scripting integration

**System Requirements:**
- Linux: Kernel 4.15+ for optimal performance, libpcap 1.9+, setcap for capabilities
- Windows: Windows 10+, Npcap 1.70+, Administrator privileges
- macOS: 11.0+, ChmodBPF or root access for packet capture
- Memory: 4GB minimum, 16GB recommended for large scans
- Storage: SSD recommended for result database, 100MB+ for program and fingerprint databases

**Build Configuration:**
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"

[features]
default = ["async", "ssl"]
async = ["tokio/full"]
ssl = ["openssl"]
lua-plugins = ["mlua"]
python-plugins = ["pyo3"]
```

This comprehensive technical specification provides the foundation for implementing ProRT-IP WarScan with production-grade security, performance, and cross-platform compatibility. The architecture combines proven techniques from industry-standard tools while leveraging Rust's safety guarantees to prevent entire vulnerability classes that plague C-based implementations.