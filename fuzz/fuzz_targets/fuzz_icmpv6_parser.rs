#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use pnet_packet::icmpv6::{Icmpv6Packet, checksum};
use pnet_packet::Packet;
use std::net::Ipv6Addr;

/// Structure-aware fuzzing input for ICMPv6 packets
#[derive(Arbitrary, Debug)]
struct FuzzIcmpv6Input {
    /// ICMPv6 type (0-255)
    /// Common types: 1 (Dest Unreachable), 128 (Echo Request), 129 (Echo Reply),
    /// 133 (Router Solicitation), 134 (Router Advertisement),
    /// 135 (Neighbor Solicitation), 136 (Neighbor Advertisement)
    icmp_type: u8,

    /// ICMPv6 code (0-255)
    /// For Type 1 (Dest Unreachable): 0-5 are defined codes
    code: u8,

    /// Payload data (0-1232 bytes, MTU minus headers)
    #[arbitrary(with = |u: &mut Unstructured| {
        let len = u.int_in_range(0..=1232)?;
        u.bytes(len).map(|b| b.to_vec())
    })]
    payload: Vec<u8>,

    /// Whether to use valid or invalid checksum
    use_bad_checksum: bool,

    /// For Echo Request/Reply: identifier
    echo_id: Option<u16>,

    /// For Echo Request/Reply: sequence number
    echo_seq: Option<u16>,

    /// For Neighbor Discovery: target IPv6 address
    nd_target: Option<[u8; 16]>,
}

/// Build ICMPv6 packet bytes from structured input
fn build_icmpv6_packet(input: &FuzzIcmpv6Input) -> Vec<u8> {
    // ICMPv6 header: type (1) + code (1) + checksum (2) = 4 bytes
    // Plus type-specific data (variable)
    let mut packet = Vec::new();

    // Type (byte 0)
    packet.push(input.icmp_type);

    // Code (byte 1)
    packet.push(input.code);

    // Checksum (bytes 2-3) - placeholder, will be calculated
    packet.extend_from_slice(&[0u8, 0u8]);

    // Type-specific data
    match input.icmp_type {
        1 => {
            // Destination Unreachable - 4 bytes unused + original packet
            packet.extend_from_slice(&[0u8, 0u8, 0u8, 0u8]);
            packet.extend_from_slice(&input.payload);
        }
        128 | 129 => {
            // Echo Request/Reply - identifier + sequence + data
            let id = input.echo_id.unwrap_or(0);
            let seq = input.echo_seq.unwrap_or(0);
            packet.extend_from_slice(&id.to_be_bytes());
            packet.extend_from_slice(&seq.to_be_bytes());
            packet.extend_from_slice(&input.payload);
        }
        133 | 134 => {
            // Router Solicitation/Advertisement
            packet.extend_from_slice(&[0u8, 0u8, 0u8, 0u8]); // Reserved
            packet.extend_from_slice(&input.payload);
        }
        135 | 136 => {
            // Neighbor Solicitation/Advertisement
            packet.extend_from_slice(&[0u8, 0u8, 0u8, 0u8]); // Reserved
            if let Some(target) = &input.nd_target {
                packet.extend_from_slice(target); // Target address (16 bytes)
            } else {
                packet.extend_from_slice(&[0u8; 16]); // Zero target
            }
            packet.extend_from_slice(&input.payload);
        }
        _ => {
            // Unknown type - just append payload
            packet.extend_from_slice(&input.payload);
        }
    }

    // Calculate checksum if requested
    if !input.use_bad_checksum {
        // Use placeholder IPv6 addresses for checksum
        let src = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1);
        let dst = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 2);
        // Create Icmpv6Packet from bytes to calculate checksum
        if let Some(icmpv6_pkt) = Icmpv6Packet::new(&packet) {
            let checksum_value = checksum(&icmpv6_pkt, &src, &dst);
            packet[2..4].copy_from_slice(&checksum_value.to_be_bytes());
        }
    }

    packet
}

fuzz_target!(|input: FuzzIcmpv6Input| {
    // Build ICMPv6 packet from structured input
    let packet_bytes = build_icmpv6_packet(&input);

    // Attempt to parse with pnet - should not panic
    if let Some(icmpv6_packet) = Icmpv6Packet::new(&packet_bytes) {
        // Exercise all accessor methods to find crashes
        let _ = icmpv6_packet.get_icmpv6_type();
        let _ = icmpv6_packet.get_icmpv6_code();
        let _ = icmpv6_packet.get_checksum();
        let _ = icmpv6_packet.payload();
        let _ = icmpv6_packet.packet();

        // Test checksum calculation (should not panic)
        let src = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1);
        let dst = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 2);
        let _ = checksum(&icmpv6_packet, &src, &dst);

        // Test type-specific parsing
        let icmp_type = icmpv6_packet.get_icmpv6_type();
        let payload = icmpv6_packet.payload();

        match icmp_type.0 {
            1 => {
                // Destination Unreachable
                let code = icmpv6_packet.get_icmpv6_code();
                // Codes 0-5 are valid for Type 1
                if code.0 <= 5 && payload.len() >= 4 {
                    let _unused = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                }
            }
            128 | 129 => {
                // Echo Request/Reply
                if payload.len() >= 4 {
                    let _id = u16::from_be_bytes([payload[0], payload[1]]);
                    let _seq = u16::from_be_bytes([payload[2], payload[3]]);
                }
            }
            135 | 136 => {
                // Neighbor Solicitation/Advertisement
                if payload.len() >= 20 {
                    let _reserved = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    let mut target = [0u8; 16];
                    target.copy_from_slice(&payload[4..20]);
                    let _target_addr = Ipv6Addr::from(target);
                }
            }
            _ => {
                // Unknown type - just access payload
                let _ = payload.len();
            }
        }
    }

    // Test minimum valid ICMPv6 packet (4 bytes header)
    if packet_bytes.len() >= 4 {
        if let Some(icmpv6_packet) = Icmpv6Packet::new(&packet_bytes) {
            let _ = icmpv6_packet.get_icmpv6_type();
        }
    }

    // Test malformed packets (shorter than 4 bytes)
    if packet_bytes.len() < 4 {
        // Should return None, not panic
        let result = Icmpv6Packet::new(&packet_bytes);
        assert!(result.is_none(), "Should reject packet shorter than 4 bytes");
    }

    // Test all Type 1 (Destination Unreachable) codes
    if input.icmp_type == 1 {
        for code in 0..=5 {
            let mut test_packet = packet_bytes.clone();
            if test_packet.len() >= 2 {
                test_packet[1] = code;
                if let Some(icmpv6) = Icmpv6Packet::new(&test_packet) {
                    let _ = icmpv6.get_icmpv6_code();
                }
            }
        }
    }

    // Test Echo Request/Reply edge cases
    if input.icmp_type == 128 || input.icmp_type == 129 {
        if packet_bytes.len() >= 8 {
            if let Some(icmpv6) = Icmpv6Packet::new(&packet_bytes) {
                let payload = icmpv6.payload();
                if payload.len() >= 4 {
                    // Identifier and sequence are first 4 bytes of payload
                    let _id = u16::from_be_bytes([payload[0], payload[1]]);
                    let _seq = u16::from_be_bytes([payload[2], payload[3]]);
                }
            }
        }
    }
});
