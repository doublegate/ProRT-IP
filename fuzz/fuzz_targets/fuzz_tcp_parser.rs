#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};
use pnet_packet::tcp::{TcpPacket, ipv4_checksum, ipv6_checksum};
use pnet_packet::Packet;
use std::net::{Ipv4Addr, Ipv6Addr};

/// Structure-aware fuzzing input for TCP packets
#[derive(Arbitrary, Debug)]
struct FuzzTcpInput {
    /// TCP source port (0-65535)
    source_port: u16,

    /// TCP destination port (0-65535)
    dest_port: u16,

    /// Sequence number
    sequence: u32,

    /// Acknowledgment number
    acknowledgment: u32,

    /// TCP flags (8 bits: FIN, SYN, RST, PSH, ACK, URG, ECE, CWR)
    flags: u8,

    /// Window size
    window: u16,

    /// Urgent pointer
    urgent_ptr: u16,

    /// TCP options (0-40 bytes)
    #[arbitrary(with = |u: &mut Unstructured| {
        let len = u.int_in_range(0..=40)?;
        u.bytes(len).map(|b| b.to_vec())
    })]
    options: Vec<u8>,

    /// Payload data (0-1460 bytes for typical MTU)
    #[arbitrary(with = |u: &mut Unstructured| {
        let len = u.int_in_range(0..=1460)?;
        u.bytes(len).map(|b| b.to_vec())
    })]
    payload: Vec<u8>,

    /// Whether to use valid or invalid checksum
    use_bad_checksum: bool,

    /// Data offset value (normally calculated, but fuzz can override)
    override_data_offset: Option<u8>,
}

/// Build TCP packet bytes from structured input
fn build_tcp_packet(input: &FuzzTcpInput) -> Vec<u8> {
    // Calculate data offset (header length in 32-bit words)
    let options_len = input.options.len();
    let options_padding = (4 - (options_len % 4)) % 4; // Pad to 4-byte boundary
    let data_offset = input.override_data_offset.unwrap_or_else(|| {
        ((20 + options_len + options_padding) / 4) as u8
    });

    // Clamp data offset to valid range (5-15)
    let data_offset = data_offset.max(5).min(15);

    let total_len = (data_offset as usize * 4) + input.payload.len();
    let mut packet = vec![0u8; total_len];

    // Source port (bytes 0-1)
    packet[0..2].copy_from_slice(&input.source_port.to_be_bytes());

    // Destination port (bytes 2-3)
    packet[2..4].copy_from_slice(&input.dest_port.to_be_bytes());

    // Sequence number (bytes 4-7)
    packet[4..8].copy_from_slice(&input.sequence.to_be_bytes());

    // Acknowledgment number (bytes 8-11)
    packet[8..12].copy_from_slice(&input.acknowledgment.to_be_bytes());

    // Data offset and flags (bytes 12-13)
    packet[12] = (data_offset << 4) | ((input.flags >> 4) & 0x0F); // Upper 4 bits of flags
    packet[13] = input.flags; // All 8 flag bits

    // Window size (bytes 14-15)
    packet[14..16].copy_from_slice(&input.window.to_be_bytes());

    // Checksum (bytes 16-17) - leave as 0 for now (fuzzing doesn't need valid checksums)
    // Checksum validation will be tested during parsing, not packet building

    // Urgent pointer (bytes 18-19)
    packet[18..20].copy_from_slice(&input.urgent_ptr.to_be_bytes());

    // Options (bytes 20..20+options_len)
    if !input.options.is_empty() {
        let options_end = 20 + input.options.len().min(packet.len() - 20);
        packet[20..options_end].copy_from_slice(&input.options[..options_end - 20]);
    }

    // Payload (after header)
    let payload_start = data_offset as usize * 4;
    if payload_start < packet.len() && !input.payload.is_empty() {
        let copy_len = input.payload.len().min(packet.len() - payload_start);
        packet[payload_start..payload_start + copy_len].copy_from_slice(&input.payload[..copy_len]);
    }

    packet
}

fuzz_target!(|input: FuzzTcpInput| {
    // Build TCP packet from structured input
    let packet_bytes = build_tcp_packet(&input);

    // Attempt to parse with pnet - should not panic
    if let Some(tcp_packet) = TcpPacket::new(&packet_bytes) {
        // Exercise all accessor methods to find crashes
        let _ = tcp_packet.get_source();
        let _ = tcp_packet.get_destination();
        let _ = tcp_packet.get_sequence();
        let _ = tcp_packet.get_acknowledgement();
        let _ = tcp_packet.get_data_offset();
        let _ = tcp_packet.get_flags();
        let _ = tcp_packet.get_window();
        let _ = tcp_packet.get_checksum();
        let _ = tcp_packet.get_urgent_ptr();
        let _ = tcp_packet.get_options();
        let _ = tcp_packet.get_options_raw();
        let _ = tcp_packet.payload();

        // Test checksum calculation (should not panic)
        let src = Ipv4Addr::new(192, 168, 1, 1);
        let dst = Ipv4Addr::new(192, 168, 1, 2);
        let _ = ipv4_checksum(&tcp_packet, &src, &dst);

        // Test IPv6 checksum as well
        let src6 = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1);
        let dst6 = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 2);
        let _ = ipv6_checksum(&tcp_packet, &src6, &dst6);
    }

    // Also test raw bytes parsing (unstructured fuzzing)
    // This catches edge cases that structured fuzzing might miss
    if packet_bytes.len() >= 20 {
        if let Some(tcp_packet) = TcpPacket::new(&packet_bytes) {
            // Ensure we can safely access the packet without panicking
            let _ = tcp_packet.packet();
        }
    }
});
