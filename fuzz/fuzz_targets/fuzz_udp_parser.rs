#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use pnet_packet::udp::{UdpPacket, ipv4_checksum, ipv6_checksum};
use pnet_packet::Packet;
use std::net::{Ipv4Addr, Ipv6Addr};

/// Structure-aware fuzzing input for UDP packets
#[derive(Arbitrary, Debug)]
struct FuzzUdpInput {
    /// UDP source port (0-65535)
    source_port: u16,

    /// UDP destination port (0-65535)
    dest_port: u16,

    /// Payload data (0-65507 bytes maximum for UDP)
    /// In practice, limit to 1472 bytes (typical MTU - headers)
    #[arbitrary(with = |u: &mut Unstructured| {
        let len = u.int_in_range(0..=1472)?;
        u.bytes(len).map(|b| b.to_vec())
    })]
    payload: Vec<u8>,

    /// Whether to use valid or invalid checksum
    use_bad_checksum: bool,

    /// Override length field (normally payload + 8 bytes header)
    override_length: Option<u16>,
}

/// Build UDP packet bytes from structured input
fn build_udp_packet(input: &FuzzUdpInput) -> Vec<u8> {
    let header_len = 8;
    let payload_len = input.payload.len();
    let total_len = header_len + payload_len;

    // UDP length field (header + payload)
    let length = input.override_length.unwrap_or(total_len as u16);

    let mut packet = vec![0u8; total_len];

    // Source port (bytes 0-1)
    packet[0..2].copy_from_slice(&input.source_port.to_be_bytes());

    // Destination port (bytes 2-3)
    packet[2..4].copy_from_slice(&input.dest_port.to_be_bytes());

    // Length (bytes 4-5)
    packet[4..6].copy_from_slice(&length.to_be_bytes());

    // Checksum (bytes 6-7) - leave as 0 (checksum optional for IPv4 UDP)
    // Checksum validation will be tested during parsing, not packet building

    // Payload (bytes 8..end)
    if !input.payload.is_empty() && payload_len <= packet.len() - header_len {
        packet[header_len..].copy_from_slice(&input.payload);
    }

    packet
}

fuzz_target!(|input: FuzzUdpInput| {
    // Build UDP packet from structured input
    let packet_bytes = build_udp_packet(&input);

    // Attempt to parse with pnet - should not panic
    if let Some(udp_packet) = UdpPacket::new(&packet_bytes) {
        // Exercise all accessor methods to find crashes
        let _ = udp_packet.get_source();
        let _ = udp_packet.get_destination();
        let _ = udp_packet.get_length();
        let _ = udp_packet.get_checksum();
        let _ = udp_packet.payload();
        let _ = udp_packet.packet();

        // Test checksum calculation (should not panic)
        let src = Ipv4Addr::new(192, 168, 1, 1);
        let dst = Ipv4Addr::new(192, 168, 1, 2);
        let _ = ipv4_checksum(&udp_packet, &src, &dst);

        // Test IPv6 checksum as well
        let src6 = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1);
        let dst6 = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 2);
        let _ = ipv6_checksum(&udp_packet, &src6, &dst6);

        // Test common UDP protocol payloads
        let payload = udp_packet.payload();

        // DNS (port 53) - try to parse DNS header (12 bytes minimum)
        if (udp_packet.get_source() == 53 || udp_packet.get_destination() == 53) && payload.len() >= 12 {
            // DNS header: ID(2) + Flags(2) + Questions(2) + Answers(2) + Authority(2) + Additional(2)
            let _id = u16::from_be_bytes([payload[0], payload[1]]);
            let _flags = u16::from_be_bytes([payload[2], payload[3]]);
            let _questions = u16::from_be_bytes([payload[4], payload[5]]);
        }

        // SNMP (port 161/162) - ASN.1 BER encoding
        if (udp_packet.get_destination() == 161 || udp_packet.get_destination() == 162) && !payload.is_empty() {
            // SNMP messages start with SEQUENCE tag (0x30)
            let _tag = payload[0];
        }

        // NetBIOS (port 137/138/139)
        if (135..=139).contains(&udp_packet.get_destination()) && payload.len() >= 12 {
            // NetBIOS name service header
            let _transaction_id = u16::from_be_bytes([payload[0], payload[1]]);
        }
    }

    // Also test raw bytes parsing for edge cases
    if packet_bytes.len() >= 8 {
        if let Some(udp_packet) = UdpPacket::new(&packet_bytes) {
            // Verify we can access the packet without panicking
            let _ = udp_packet.packet();

            // Test zero-length payload (valid UDP)
            if udp_packet.get_length() == 8 && udp_packet.payload().is_empty() {
                let _ = udp_packet.get_source();
            }
        }
    }

    // Test malformed packets (shorter than 8 bytes)
    if packet_bytes.len() < 8 {
        // Should return None, not panic
        let result = UdpPacket::new(&packet_bytes);
        assert!(result.is_none(), "Should reject packet shorter than 8 bytes");
    }
});
