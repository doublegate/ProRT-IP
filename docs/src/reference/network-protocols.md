# Network Protocols Reference

ProRT-IP implements multiple network protocols for scanning including TCP, UDP, ICMP, ICMPv6, and application-layer protocols. This document provides comprehensive technical reference for protocol implementations, packet structures, and RFC compliance.

## Protocol Architecture

### Layer Model

```
┌─────────────────────────────────────────┐
│        Application Layer                │
│    (DNS, SNMP, NTP, NetBIOS, etc.)     │
├─────────────────────────────────────────┤
│        Transport Layer                  │
│         (TCP / UDP)                     │
├─────────────────────────────────────────┤
│        Network Layer                    │
│    (IPv4 / IPv6 / ICMP / ICMPv6)       │
├─────────────────────────────────────────┤
│        Data Link Layer                  │
│         (Ethernet)                      │
└─────────────────────────────────────────┘
```

### Implementation Overview

| Protocol | Module | RFC Compliance | Key Features |
|----------|--------|----------------|--------------|
| TCP | `packet_builder.rs` | RFC 793, 7323 | All flags, options (MSS, WScale, SACK, Timestamp) |
| UDP | `packet_builder.rs` | RFC 768 | Protocol-specific payloads |
| IPv4 | `packet_builder.rs` | RFC 791 | Fragmentation, TTL control |
| IPv6 | `ipv6_packet.rs` | RFC 8200 | Extension headers, flow labels |
| ICMPv6 | `icmpv6.rs` | RFC 4443 | Echo, NDP, Router Discovery |
| ICMP | `pnet` crate | RFC 792 | Echo, Unreachable |

## TCP Protocol Implementation

### Header Structure

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
├─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┤
│          Source Port          │       Destination Port        │
├─────────────────────────────────────────────────────────────────┤
│                        Sequence Number                          │
├─────────────────────────────────────────────────────────────────┤
│                    Acknowledgment Number                        │
├───────────┬───────┬─┬─┬─┬─┬─┬─┬─────────────────────────────────┤
│  Data     │       │C│E│U│A│P│R│S│F│                               │
│  Offset   │ Res.  │W│C│R│C│S│S│Y│I│           Window              │
│           │       │R│E│G│K│H│T│N│N│                               │
├───────────┴───────┴─┴─┴─┴─┴─┴─┴─┴───────────────────────────────┤
│           Checksum            │         Urgent Pointer          │
├─────────────────────────────────────────────────────────────────┤
│                    Options (if data offset > 5)                 │
├─────────────────────────────────────────────────────────────────┤
│                             Payload                             │
└─────────────────────────────────────────────────────────────────┘
```

### TCP Flags

ProRT-IP implements all 8 TCP flags defined in RFC 793 and RFC 3168:

| Flag | Bitmask | Description | Scan Usage |
|------|---------|-------------|------------|
| FIN | `0x01` | Finish - graceful close | FIN scan (stealth) |
| SYN | `0x02` | Synchronize - connection initiation | SYN scan (default) |
| RST | `0x04` | Reset - abort connection | Response detection |
| PSH | `0x08` | Push - immediate delivery | - |
| ACK | `0x10` | Acknowledge - data receipt | ACK scan (firewall mapping) |
| URG | `0x20` | Urgent - priority data | - |
| ECE | `0x40` | ECN-Echo (RFC 3168) | - |
| CWR | `0x80` | Congestion Window Reduced | - |

**Flag Combinations for Stealth Scans:**

| Scan Type | Flags | Expected Response (Open) | Expected Response (Closed) |
|-----------|-------|--------------------------|----------------------------|
| SYN | `0x02` | SYN+ACK | RST |
| FIN | `0x01` | No response | RST |
| NULL | `0x00` | No response | RST |
| Xmas | `0x29` (FIN+PSH+URG) | No response | RST |
| ACK | `0x10` | RST (unfiltered) | RST (unfiltered) |

### TCP Options

ProRT-IP supports all standard TCP options for fingerprinting and evasion:

```rust
pub enum TcpOption {
    Mss(u16),                    // Maximum Segment Size (kind=2, len=4)
    WindowScale(u8),             // Window Scale factor (kind=3, len=3)
    SackPermitted,               // SACK Permitted (kind=4, len=2)
    Timestamp { tsval, tsecr },  // Timestamps (kind=8, len=10)
    Nop,                         // Padding (kind=1, len=1)
    Eol,                         // End of list (kind=0, len=1)
}
```

**Option Details:**

| Option | Kind | Length | RFC | Purpose |
|--------|------|--------|-----|---------|
| MSS | 2 | 4 | RFC 879 | Maximum segment size negotiation |
| Window Scale | 3 | 3 | RFC 7323 | Large window support (up to 1GB) |
| SACK Permitted | 4 | 2 | RFC 2018 | Selective acknowledgment negotiation |
| Timestamp | 8 | 10 | RFC 7323 | RTT measurement, PAWS |
| NOP | 1 | 1 | RFC 793 | Option padding/alignment |
| EOL | 0 | 1 | RFC 793 | End of options list |

### TcpPacketBuilder Usage

```rust
use prtip_network::{TcpPacketBuilder, TcpFlags, TcpOption};
use std::net::Ipv4Addr;

// Basic SYN packet
let packet = TcpPacketBuilder::new()
    .source_ip(Ipv4Addr::new(10, 0, 0, 1))
    .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
    .source_port(12345)
    .dest_port(80)
    .flags(TcpFlags::SYN)
    .window(65535)
    .build()
    .expect("Failed to build packet");

// SYN with TCP options (mimics real OS)
let packet = TcpPacketBuilder::new()
    .source_ip(Ipv4Addr::new(10, 0, 0, 1))
    .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
    .source_port(12345)
    .dest_port(443)
    .flags(TcpFlags::SYN)
    .window(65535)
    .add_option(TcpOption::Mss(1460))
    .add_option(TcpOption::WindowScale(7))
    .add_option(TcpOption::SackPermitted)
    .build()
    .expect("Failed to build packet");

// IPv6 TCP packet
let src_v6 = "2001:db8::1".parse().unwrap();
let dst_v6 = "2001:db8::2".parse().unwrap();

let packet = TcpPacketBuilder::new()
    .source_port(12345)
    .dest_port(80)
    .flags(TcpFlags::SYN)
    .build_ipv6_packet(src_v6, dst_v6)
    .expect("Failed to build IPv6 packet");
```

### Zero-Copy Packet Building

For high-performance scenarios (>100K pps), use buffer pools:

```rust
use prtip_network::{TcpPacketBuilder, TcpFlags, packet_buffer::with_buffer};

with_buffer(|pool| {
    let packet = TcpPacketBuilder::new()
        .source_ip(Ipv4Addr::new(10, 0, 0, 1))
        .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
        .source_port(12345)
        .dest_port(80)
        .flags(TcpFlags::SYN)
        .build_with_buffer(pool)
        .expect("Failed to build packet");

    // Packet slice is valid within this closure
    send_packet(packet);

    pool.reset();
});
```

**Performance Comparison:**

| Method | Allocation | Typical Time |
|--------|------------|--------------|
| `build()` | 1 Vec per packet | ~2-5µs |
| `build_with_buffer()` | Zero | <1µs |

## UDP Protocol Implementation

### Header Structure

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
├─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┤
│          Source Port          │       Destination Port        │
├─────────────────────────────────────────────────────────────────┤
│            Length             │           Checksum            │
├─────────────────────────────────────────────────────────────────┤
│                             Payload                             │
└─────────────────────────────────────────────────────────────────┘
```

### UdpPacketBuilder Usage

```rust
use prtip_network::UdpPacketBuilder;
use std::net::Ipv4Addr;

// Basic UDP packet
let packet = UdpPacketBuilder::new()
    .source_ip(Ipv4Addr::new(10, 0, 0, 1))
    .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
    .source_port(12345)
    .dest_port(53)
    .payload(dns_query.to_vec())
    .build()
    .expect("Failed to build packet");

// IPv6 UDP packet
let packet = UdpPacketBuilder::new()
    .source_port(12345)
    .dest_port(53)
    .payload(dns_query.to_vec())
    .build_ipv6_packet(src_v6, dst_v6)
    .expect("Failed to build packet");
```

### Protocol-Specific Payloads

ProRT-IP provides well-formed payloads for common UDP protocols to improve detection rates:

| Port | Protocol | Payload Description |
|------|----------|---------------------|
| 53 | DNS | Standard query for root domain |
| 123 | NTP | Version 3 client request (48 bytes) |
| 137 | NetBIOS | Name Service query for `*<00><00>` |
| 161 | SNMP | GetRequest for `sysDescr.0` with community "public" |
| 111 | RPC | Sun RPC NULL call (portmapper query) |
| 500 | IKE | IPSec Main Mode SA payload |
| 1900 | SSDP | M-SEARCH discovery request |
| 5353 | mDNS | Query for `_services._dns-sd._udp.local` |

**Usage:**

```rust
use prtip_network::protocol_payloads::get_udp_payload;

if let Some(payload) = get_udp_payload(53) {
    // Use DNS-specific payload for better detection
    let packet = UdpPacketBuilder::new()
        .source_port(12345)
        .dest_port(53)
        .payload(payload)
        .build();
}
```

### UDP Scan Behavior

UDP scanning is fundamentally different from TCP:

| Response | Interpretation |
|----------|----------------|
| UDP response | Port is open |
| ICMP Port Unreachable | Port is closed |
| ICMP Other Unreachable | Port is filtered |
| No response | Open or filtered |

**Timing Considerations:**

- UDP scans are 10-100x slower than TCP scans
- ICMP rate limiting affects response timing
- Retransmissions required for reliability
- Protocol-specific payloads improve response rates

## IPv4 Protocol Implementation

### Header Fields

ProRT-IP provides full control over IPv4 header fields:

| Field | Size | Default | Configurable |
|-------|------|---------|--------------|
| Version | 4 bits | 4 | No |
| IHL | 4 bits | 5 (20 bytes) | Auto-calculated |
| DSCP/ECN | 8 bits | 0 | No |
| Total Length | 16 bits | Auto | Auto-calculated |
| Identification | 16 bits | Random | Yes (`ip_id()`) |
| Flags | 3 bits | Don't Fragment | Via fragmentation |
| Fragment Offset | 13 bits | 0 | Via fragmentation |
| TTL | 8 bits | 64 | Yes (`ttl()`) |
| Protocol | 8 bits | 6 (TCP) or 17 (UDP) | Auto |
| Checksum | 16 bits | Auto | Auto-calculated |
| Source IP | 32 bits | Required | Yes |
| Destination IP | 32 bits | Required | Yes |

### Checksum Algorithm

IPv4 and TCP/UDP checksums use the Internet checksum algorithm (RFC 1071):

```
1. Sum all 16-bit words with carry
2. Add any carry overflow
3. Take one's complement
```

**Implementation:**
- IPv4 header checksum: Covers only IP header
- TCP/UDP checksum: Includes pseudo-header (src IP, dst IP, protocol, length)
- ICMPv6 checksum: Includes 40-byte IPv6 pseudo-header

## IPv6 Protocol Implementation

### Header Structure

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
├─┬─┬─┬─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┤
│Version│  Traffic Class  │             Flow Label              │
├───────┴─────────────────┴─────────────────────────────────────┤
│         Payload Length        │  Next Header  │   Hop Limit   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                         Source Address                          │
│                          (128 bits)                             │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                      Destination Address                        │
│                          (128 bits)                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Ipv6PacketBuilder Usage

```rust
use prtip_network::ipv6_packet::Ipv6PacketBuilder;
use std::net::Ipv6Addr;

let src = "2001:db8::1".parse::<Ipv6Addr>().unwrap();
let dst = "2001:db8::2".parse::<Ipv6Addr>().unwrap();

let packet = Ipv6PacketBuilder::new(src, dst)
    .hop_limit(64)
    .next_header(6)  // TCP
    .payload(tcp_data)
    .build()
    .expect("Failed to build IPv6 packet");
```

### IPv6 vs IPv4 Size Comparison

| Component | IPv4 | IPv6 | Difference |
|-----------|------|------|------------|
| IP Header | 20 bytes | 40 bytes | +20 bytes |
| TCP Header | 20 bytes | 20 bytes | 0 |
| Minimum Packet | 40 bytes | 60 bytes | +20 bytes |

## ICMPv6 Protocol Implementation

### Supported Message Types

| Type | Name | Usage |
|------|------|-------|
| 128 | Echo Request | Host discovery (ping) |
| 129 | Echo Reply | Response to ping |
| 133 | Router Solicitation | Router discovery |
| 134 | Router Advertisement | Router announcement |
| 135 | Neighbor Solicitation | Address resolution (replaces ARP) |
| 136 | Neighbor Advertisement | Address resolution response |
| 1 | Destination Unreachable | Error reporting |

### Icmpv6PacketBuilder Usage

```rust
use prtip_network::icmpv6::Icmpv6PacketBuilder;
use std::net::Ipv6Addr;

let src = "2001:db8::1".parse().unwrap();
let dst = "2001:db8::2".parse().unwrap();

// Echo Request (ping)
let packet = Icmpv6PacketBuilder::echo_request(1234, 1, vec![0xDE, 0xAD])
    .build(src, dst)
    .unwrap();

// Neighbor Solicitation (address resolution)
let target = "fe80::2".parse().unwrap();
let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
let packet = Icmpv6PacketBuilder::neighbor_solicitation(target, Some(mac))
    .build(src, "ff02::1:ff00:2".parse().unwrap())
    .unwrap();

// Router Solicitation
let packet = Icmpv6PacketBuilder::router_solicitation(Some(mac))
    .build(src, "ff02::2".parse().unwrap())
    .unwrap();
```

### ICMPv6 Checksum

ICMPv6 checksums include a 40-byte pseudo-header (unlike IPv4 ICMP):

```
Pseudo-header format:
├─ Source Address (16 bytes)
├─ Destination Address (16 bytes)
├─ Upper-Layer Packet Length (4 bytes)
├─ Zero padding (3 bytes)
└─ Next Header: 58 (1 byte)
```

### ICMPv6 Response Parsing

```rust
use prtip_network::icmpv6::Icmpv6ResponseParser;

// Parse Echo Reply
if let Some((identifier, sequence)) = Icmpv6ResponseParser::parse_echo_reply(&packet) {
    println!("Reply from id={} seq={}", identifier, sequence);
}

// Parse Port Unreachable (for UDP scanning)
if let Some((dest_addr, port)) = Icmpv6ResponseParser::parse_port_unreachable(&packet) {
    println!("Port {} on {} is closed", port, dest_addr);
}

// Quick type check
if Icmpv6ResponseParser::is_icmpv6(&packet) {
    let (typ, code) = Icmpv6ResponseParser::get_type_code(&packet).unwrap();
    println!("ICMPv6 type={} code={}", typ, code);
}
```

## Evasion Techniques

### Bad Checksum

Test firewall/IDS checksum validation:

```rust
// TCP with invalid checksum
let packet = TcpPacketBuilder::new()
    .source_ip(src)
    .dest_ip(dst)
    .source_port(12345)
    .dest_port(80)
    .flags(TcpFlags::SYN)
    .bad_checksum(true)  // Sets checksum to 0x0000
    .build();

// UDP with invalid checksum
let packet = UdpPacketBuilder::new()
    .source_ip(src)
    .dest_ip(dst)
    .source_port(12345)
    .dest_port(53)
    .bad_checksum(true)
    .build();
```

### TTL Control

Control packet hop limit for traceroute-style probes:

```rust
let packet = TcpPacketBuilder::new()
    .source_ip(src)
    .dest_ip(dst)
    .source_port(12345)
    .dest_port(80)
    .ttl(10)  // Only traverse 10 hops
    .flags(TcpFlags::SYN)
    .build();
```

## RFC Compliance Matrix

| RFC | Title | Implementation Status |
|-----|-------|----------------------|
| RFC 768 | UDP | ✅ Full |
| RFC 791 | IPv4 | ✅ Full |
| RFC 792 | ICMP | ✅ Via pnet |
| RFC 793 | TCP | ✅ Full |
| RFC 879 | TCP MSS | ✅ Full |
| RFC 1071 | Internet Checksum | ✅ Full |
| RFC 2018 | TCP SACK | ✅ Full |
| RFC 3168 | ECN | ✅ Flags only |
| RFC 4443 | ICMPv6 | ✅ Full |
| RFC 4861 | NDP | ✅ NS/NA/RS |
| RFC 5681 | TCP Congestion | ⚠️ Partial (timing) |
| RFC 6298 | TCP RTO | ✅ Via timing |
| RFC 7323 | TCP Extensions | ✅ Full |
| RFC 8200 | IPv6 | ✅ Full |

## Performance Characteristics

### Packet Building Performance

| Operation | Time | Allocations |
|-----------|------|-------------|
| TCP SYN (basic) | ~2µs | 1 |
| TCP SYN (with options) | ~3µs | 1 |
| TCP SYN (zero-copy) | <1µs | 0 |
| UDP (basic) | ~1.5µs | 1 |
| UDP (with payload) | ~2µs | 1 |
| ICMPv6 Echo | ~2µs | 1 |

### Throughput Limits

| Scenario | Max Packets/sec | Notes |
|----------|-----------------|-------|
| SYN scan (standard) | ~500K | Single-threaded |
| SYN scan (zero-copy) | ~1M | Buffer pool |
| UDP scan | ~100K | ICMP rate limiting |
| ICMPv6 scan | ~200K | Host discovery |

## See Also

- [Scan Types Guide](../user-guide/scan-types.md) - TCP/UDP scan technique documentation
- [IPv6 Guide](../features/ipv6.md) - IPv6 and ICMPv6 protocol details
- [Technical Specifications](./tech-spec-v2.md) - Network stack architecture
- [Timing Templates](./timing-templates.md) - TCP congestion control (AIMD, RTT estimation)
- [Evasion Techniques](../advanced/evasion-techniques.md) - Firewall bypass methods

---

**Last Updated:** 2025-11-21
**ProRT-IP Version:** v0.5.4
