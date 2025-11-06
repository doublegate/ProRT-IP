#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use pnet_packet::ipv6::Ipv6Packet;
use pnet_packet::Packet;

/// IPv6 extension header types
#[derive(Arbitrary, Debug, Clone, Copy)]
enum ExtensionHeaderType {
    HopByHop = 0,
    Routing = 43,
    Fragment = 44,
    DestinationOptions = 60,
}

/// Structure-aware fuzzing input for IPv6 packets
#[derive(Arbitrary, Debug)]
struct FuzzIpv6Input {
    /// Traffic class (8 bits)
    traffic_class: u8,

    /// Flow label (20 bits)
    flow_label: u32,

    /// Hop limit (TTL equivalent)
    hop_limit: u8,

    /// Source IPv6 address (16 bytes)
    source: [u8; 16],

    /// Destination IPv6 address (16 bytes)
    destination: [u8; 16],

    /// Next header protocol number
    next_header: u8,

    /// Extension headers (0-3 headers, variable length)
    #[arbitrary(with = |u: &mut Unstructured| {
        let count = u.int_in_range(0..=3)?;
        (0..count).map(|_| {
            let header_type = u.choose(&[0u8, 43, 44, 60])?;
            let len = u.int_in_range(0..=40)?;
            let data = u.bytes(len)?.to_vec();
            Ok::<(u8, Vec<u8>), arbitrary::Error>((*header_type, data))
        }).collect::<Result<Vec<(u8, Vec<u8>)>, arbitrary::Error>>()
    })]
    extension_headers: Vec<(u8, Vec<u8>)>,

    /// Payload data (0-1280 bytes, minimum IPv6 MTU)
    #[arbitrary(with = |u: &mut Unstructured| {
        let len = u.int_in_range(0..=1280)?;
        u.bytes(len).map(|b| b.to_vec())
    })]
    payload: Vec<u8>,

    /// Override payload length field
    override_payload_length: Option<u16>,
}

/// Build IPv6 packet bytes from structured input
fn build_ipv6_packet(input: &FuzzIpv6Input) -> Vec<u8> {
    // Calculate extension headers size
    let ext_headers_size: usize = input.extension_headers.iter()
        .map(|(_, data)| {
            // Extension headers are 8 bytes + length field * 8 bytes
            // Simplified: just use actual data length + 2 bytes overhead
            data.len() + 2
        })
        .sum();

    // Calculate total payload length (extension headers + payload)
    let payload_length = input.override_payload_length
        .unwrap_or((ext_headers_size + input.payload.len()) as u16);

    let total_len = 40 + ext_headers_size + input.payload.len();
    let mut packet = vec![0u8; total_len];

    // IPv6 header (40 bytes)
    // Version (4 bits) = 6, Traffic Class (8 bits), Flow Label (20 bits)
    let version_tc_flow = (6u32 << 28)
        | ((input.traffic_class as u32) << 20)
        | (input.flow_label & 0xFFFFF);
    packet[0..4].copy_from_slice(&version_tc_flow.to_be_bytes());

    // Payload length (bytes 4-5)
    packet[4..6].copy_from_slice(&payload_length.to_be_bytes());

    // Next header (byte 6)
    packet[6] = if !input.extension_headers.is_empty() {
        input.extension_headers[0].0
    } else {
        input.next_header
    };

    // Hop limit (byte 7)
    packet[7] = input.hop_limit;

    // Source address (bytes 8-23)
    packet[8..24].copy_from_slice(&input.source);

    // Destination address (bytes 24-39)
    packet[24..40].copy_from_slice(&input.destination);

    // Extension headers (after main header)
    let mut offset = 40;
    for (i, (_header_type, data)) in input.extension_headers.iter().enumerate() {
        if offset + data.len() + 2 > packet.len() {
            break; // Prevent overflow
        }

        // Next header field
        packet[offset] = if i + 1 < input.extension_headers.len() {
            input.extension_headers[i + 1].0
        } else {
            input.next_header
        };

        // Header extension length (in 8-byte units, minus first 8 bytes)
        let hdr_len = ((data.len() + 7) / 8) as u8;
        packet[offset + 1] = hdr_len;

        // Header data
        if !data.is_empty() && offset + 2 + data.len() <= packet.len() {
            packet[offset + 2..offset + 2 + data.len()].copy_from_slice(data);
        }

        offset += 2 + data.len();
    }

    // Payload (after extension headers)
    if !input.payload.is_empty() && offset + input.payload.len() <= packet.len() {
        packet[offset..offset + input.payload.len()].copy_from_slice(&input.payload);
    }

    packet
}

fuzz_target!(|input: FuzzIpv6Input| {
    // Build IPv6 packet from structured input
    let packet_bytes = build_ipv6_packet(&input);

    // Attempt to parse with pnet - should not panic
    if let Some(ipv6_packet) = Ipv6Packet::new(&packet_bytes) {
        // Exercise all accessor methods to find crashes
        let _ = ipv6_packet.get_version();
        let _ = ipv6_packet.get_traffic_class();
        let _ = ipv6_packet.get_flow_label();
        let _ = ipv6_packet.get_payload_length();
        let _ = ipv6_packet.get_next_header();
        let _ = ipv6_packet.get_hop_limit();
        let _ = ipv6_packet.get_source();
        let _ = ipv6_packet.get_destination();
        let _ = ipv6_packet.payload();
        let _ = ipv6_packet.packet();

        // Test extension header parsing (simplified)
        let next_header = ipv6_packet.get_next_header().0;
        let payload = ipv6_packet.payload();

        // Check if next header is an extension header type
        let is_extension = matches!(next_header, 0 | 43 | 44 | 60);
        if is_extension && payload.len() >= 2 {
            // Parse first two bytes of extension header
            let _next_hdr = payload[0];
            let _hdr_ext_len = payload[1];
            // Just accessing the bytes is enough for fuzzing coverage
        }
    }

    // Test minimum valid IPv6 packet (40 bytes header only)
    if packet_bytes.len() >= 40 {
        if let Some(ipv6_packet) = Ipv6Packet::new(&packet_bytes) {
            // Verify version is 6
            let version = ipv6_packet.get_version();
            if version == 6 {
                let _ = ipv6_packet.get_source();
            }
        }
    }

    // Test malformed packets (shorter than 40 bytes)
    if packet_bytes.len() < 40 {
        // Should return None, not panic
        let result = Ipv6Packet::new(&packet_bytes);
        assert!(result.is_none(), "Should reject packet shorter than 40 bytes");
    }

    // Test fragment header handling (next_header = 44)
    if packet_bytes.len() >= 48 && packet_bytes[6] == 44 {
        if let Some(ipv6_packet) = Ipv6Packet::new(&packet_bytes) {
            let payload = ipv6_packet.payload();
            if payload.len() >= 8 {
                // Fragment header: next header (1) + reserved (1) + fragment offset (2) + res + M flag (1) + identification (4)
                let _next_hdr = payload[0];
                let _frag_offset = u16::from_be_bytes([payload[2], payload[3]]);
                let _more_fragments = (payload[3] & 0x01) != 0;
                let _identification = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
            }
        }
    }
});
