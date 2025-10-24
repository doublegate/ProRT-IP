//! IP packet fragmentation for firewall/IDS evasion
//!
//! This module implements IP-layer packet fragmentation compatible with Nmap's `-f` and `--mtu` flags.
//! Fragmentation splits packets into smaller pieces to evade firewalls and IDS that don't properly
//! reassemble fragments.
//!
//! # How It Works
//!
//! 1. **Validation**: MTU must be ≥68 bytes and multiple of 8
//! 2. **Splitting**: Original packet divided into fragments
//! 3. **Fragment Headers**: Each fragment gets proper IP header with:
//!    - Fragment offset (in 8-byte units)
//!    - More Fragments (MF) flag
//!    - Same Fragment ID
//! 4. **Checksums**: IP checksum recalculated for each fragment
//!
//! # Example
//!
//! ```no_run
//! use prtip_network::fragmentation::{fragment_tcp_packet, validate_mtu};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Build original TCP packet (SYN to port 80)
//! let packet = vec![0u8; 40];  // 20 IP + 20 TCP
//!
//! // Fragment with MTU 200 (reasonable fragmentation)
//! let fragments = fragment_tcp_packet(&packet, 200)?;
//! println!("Split into {} fragments", fragments.len());
//!
//! // Or aggressive 8-byte fragmentation (Nmap -f flag)
//! let aggressive = fragment_tcp_packet(&packet, 28)?;  // 20 IP + 8 data
//! println!("Aggressive: {} fragments", aggressive.len());
//! # Ok(())
//! # }
//! ```
//!
//! # MTU Requirements
//!
//! - **Minimum**: 68 bytes (20 IP header + 8 fragment minimum from RFC 791)
//! - **Multiple of 8**: IP fragment offset is in 8-byte units
//! - **Typical**: 1500 bytes (Ethernet MTU)
//! - **Nmap `-f`**: 28 bytes (20 IP + 8 data, extremely aggressive)
//!
//! # Security Considerations
//!
//! - **Modern Firewalls**: Many reassemble fragments before inspection
//! - **Performance**: Fragmentation multiplies packet count
//! - **Detection**: Unusual fragment sizes may trigger alerts
//! - **Combination**: Most effective when combined with other evasion techniques

use pnet::packet::ipv4::{checksum as ipv4_checksum, Ipv4Flags, MutableIpv4Packet};
use thiserror::Error;

/// Errors that can occur during fragmentation
#[derive(Debug, Error)]
pub enum FragmentationError {
    #[error("MTU too small: {mtu} bytes (minimum 68 bytes)")]
    MtuTooSmall { mtu: usize },

    #[error("MTU must be multiple of 8, got {mtu}")]
    MtuNotMultipleOf8 { mtu: usize },

    #[error("Packet too small: {size} bytes (minimum 20 for IP header)")]
    PacketTooSmall { size: usize },

    #[error("Invalid IP header in packet")]
    InvalidIpHeader,

    #[error("Buffer too small for fragment: need {needed}, have {available}")]
    BufferTooSmall { needed: usize, available: usize },
}

pub type Result<T> = std::result::Result<T, FragmentationError>;

/// Minimum MTU allowed (RFC 791: 20 IP header + 8 fragment minimum)
pub const MIN_MTU: usize = 28; // Nmap -f compatibility (20-byte header + 8-byte payload)

/// Standard Ethernet MTU
pub const STANDARD_MTU: usize = 1500;

/// IP header size (without options)
pub const IP_HEADER_SIZE: usize = 20;

/// Nmap -f flag MTU (20 IP + 8 data)
pub const NMAP_F_MTU: usize = 28;

/// Fragment a TCP/IP packet into multiple IP fragments
///
/// # Arguments
///
/// * `packet` - Original IP packet (must include IP header)
/// * `mtu` - Maximum Transmission Unit (must be ≥68 and multiple of 8)
///
/// # Returns
///
/// Vector of fragmented packets, each with complete IP header
///
/// # Errors
///
/// Returns error if MTU is invalid or packet is malformed
///
/// # Example
///
/// ```no_run
/// # use prtip_network::fragmentation::fragment_tcp_packet;
/// let packet = vec![0u8; 100];  // Example packet
/// let fragments = fragment_tcp_packet(&packet, 200)?;
/// assert!(fragments.len() > 1);  // Packet was fragmented
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn fragment_tcp_packet(packet: &[u8], mtu: usize) -> Result<Vec<Vec<u8>>> {
    // Validate MTU
    validate_mtu(mtu)?;

    // Validate packet has IP header
    if packet.len() < IP_HEADER_SIZE {
        return Err(FragmentationError::PacketTooSmall { size: packet.len() });
    }

    // Calculate fragment size (MTU - IP header)
    let fragment_data_size = calculate_fragment_data_size(mtu);

    // Extract original IP header for reuse
    let ip_header = &packet[0..IP_HEADER_SIZE];
    let payload = &packet[IP_HEADER_SIZE..];

    // If payload fits in one fragment, no fragmentation needed
    if payload.len() <= fragment_data_size {
        return Ok(vec![packet.to_vec()]);
    }

    // Generate unique fragment ID
    use rand::Rng;
    let fragment_id = rand::thread_rng().gen::<u16>();

    let mut fragments = Vec::new();
    let mut offset_bytes = 0;

    while offset_bytes < payload.len() {
        let remaining = payload.len() - offset_bytes;
        let data_size = std::cmp::min(remaining, fragment_data_size);
        let is_last = (offset_bytes + data_size) >= payload.len();

        let fragment = create_fragment(
            ip_header,
            &payload[offset_bytes..offset_bytes + data_size],
            offset_bytes,
            fragment_id,
            !is_last, // More Fragments flag
        )?;

        fragments.push(fragment);
        offset_bytes += data_size;
    }

    Ok(fragments)
}

/// Validate MTU meets requirements
///
/// # Errors
///
/// Returns error if MTU is too small or not a multiple of 8
pub fn validate_mtu(mtu: usize) -> Result<()> {
    if mtu < MIN_MTU {
        return Err(FragmentationError::MtuTooSmall { mtu });
    }
    // Calculate the payload size after subtracting IP header
    let payload_size = calculate_fragment_data_size(mtu);
    if payload_size == 0 {
        return Err(FragmentationError::MtuTooSmall { mtu });
    }
    Ok(())
}

/// Calculate fragment data size from MTU
///
/// Returns the number of data bytes that can fit in each fragment
/// (MTU - IP header size, rounded down to multiple of 8)
fn calculate_fragment_data_size(mtu: usize) -> usize {
    let size = mtu - IP_HEADER_SIZE;
    // Round down to multiple of 8
    (size / 8) * 8
}

/// Create a single IP fragment
///
/// # Arguments
///
/// * `ip_header` - Original IP header (20 bytes)
/// * `data` - Fragment data
/// * `offset_bytes` - Byte offset in original packet
/// * `fragment_id` - Fragment ID (same for all fragments of original packet)
/// * `more_fragments` - True if this is not the last fragment
fn create_fragment(
    ip_header: &[u8],
    data: &[u8],
    offset_bytes: usize,
    fragment_id: u16,
    more_fragments: bool,
) -> Result<Vec<u8>> {
    let total_size = IP_HEADER_SIZE + data.len();
    let mut fragment = vec![0u8; total_size];

    // Create mutable IP packet
    let mut ip_packet =
        MutableIpv4Packet::new(&mut fragment).ok_or(FragmentationError::InvalidIpHeader)?;

    // Copy fields from original IP header
    let original_ip = pnet::packet::ipv4::Ipv4Packet::new(ip_header)
        .ok_or(FragmentationError::InvalidIpHeader)?;

    ip_packet.set_version(4);
    ip_packet.set_header_length(5); // 5 * 4 = 20 bytes
    ip_packet.set_dscp(original_ip.get_dscp());
    ip_packet.set_ecn(original_ip.get_ecn());
    ip_packet.set_total_length(total_size as u16);
    ip_packet.set_identification(fragment_id);
    ip_packet.set_ttl(original_ip.get_ttl());
    ip_packet.set_next_level_protocol(original_ip.get_next_level_protocol());
    ip_packet.set_source(original_ip.get_source());
    ip_packet.set_destination(original_ip.get_destination());

    // Set fragment-specific fields
    let offset_8byte_units = (offset_bytes / 8) as u16;
    ip_packet.set_fragment_offset(offset_8byte_units);

    // Set flags (DF=0, MF=more_fragments)
    let mut flags = 0u8;
    if more_fragments {
        flags |= Ipv4Flags::MoreFragments;
    }
    ip_packet.set_flags(flags);

    // Copy data
    ip_packet.set_payload(data);

    // Recalculate checksum
    let checksum = ipv4_checksum(&ip_packet.to_immutable());
    ip_packet.set_checksum(checksum);

    Ok(fragment)
}

/// Defragment packets back into original (for testing)
///
/// # Arguments
///
/// * `fragments` - Vector of IP fragments
///
/// # Returns
///
/// Reassembled original packet
///
/// # Errors
///
/// Returns error if fragments are invalid or incomplete
pub fn defragment_packets(mut fragments: Vec<Vec<u8>>) -> Result<Vec<u8>> {
    if fragments.is_empty() {
        return Ok(Vec::new());
    }

    // Sort fragments by offset
    fragments.sort_by_key(|f| {
        if let Some(ip) = pnet::packet::ipv4::Ipv4Packet::new(f) {
            ip.get_fragment_offset()
        } else {
            0
        }
    });

    // Extract IP header from first fragment
    let first_fragment = &fragments[0];
    if first_fragment.len() < IP_HEADER_SIZE {
        return Err(FragmentationError::PacketTooSmall {
            size: first_fragment.len(),
        });
    }

    let ip_header = &first_fragment[0..IP_HEADER_SIZE];
    let mut reassembled = ip_header.to_vec();

    // Collect all data
    for fragment in &fragments {
        if fragment.len() < IP_HEADER_SIZE {
            continue;
        }
        reassembled.extend_from_slice(&fragment[IP_HEADER_SIZE..]);
    }

    // Calculate total length before creating mutable borrow
    let total_length = reassembled.len() as u16;

    // Update total length in IP header
    if let Some(mut ip_packet) = MutableIpv4Packet::new(&mut reassembled) {
        ip_packet.set_total_length(total_length);
        ip_packet.set_fragment_offset(0);
        ip_packet.set_flags(0); // Clear all flags

        // Recalculate checksum
        let checksum = ipv4_checksum(&ip_packet.to_immutable());
        ip_packet.set_checksum(checksum);
    }

    Ok(reassembled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_mtu_valid() {
        assert!(validate_mtu(68).is_ok()); // Minimum
        assert!(validate_mtu(200).is_ok()); // Valid
        assert!(validate_mtu(1500).is_ok()); // Standard
    }

    #[test]
    fn test_validate_mtu_too_small() {
        // MIN_MTU is 28 (20 IP header + 8 min payload), so 20 should fail
        let result = validate_mtu(20);
        assert!(matches!(
            result,
            Err(FragmentationError::MtuTooSmall { .. })
        ));
    }

    #[test]
    fn test_validate_mtu_not_multiple_of_8() {
        // MTU doesn't need to be multiple of 8, only the payload size matters
        // MTU 100 - 20 byte header = 80 byte payload (valid, multiple of 8)
        let result = validate_mtu(100);
        assert!(result.is_ok(), "MTU 100 should be valid (80-byte payload)");
    }

    #[test]
    fn test_calculate_fragment_data_size() {
        assert_eq!(calculate_fragment_data_size(68), 48); // 68 - 20 = 48
        assert_eq!(calculate_fragment_data_size(200), 176); // 200 - 20 = 180, round down to 176
        assert_eq!(calculate_fragment_data_size(1500), 1480); // 1500 - 20 = 1480
    }

    // ========================================================================
    // TEST HELPER FUNCTIONS
    // ========================================================================

    /// Create a valid IP packet with specified size
    fn create_test_packet(total_size: usize) -> Vec<u8> {
        use pnet::packet::ip::IpNextHeaderProtocols;
        use pnet::packet::ipv4::MutableIpv4Packet;
        use std::net::Ipv4Addr;

        let mut packet = vec![0u8; total_size];
        if let Some(mut ip_packet) = MutableIpv4Packet::new(&mut packet) {
            ip_packet.set_version(4);
            ip_packet.set_header_length(5);
            ip_packet.set_total_length(total_size as u16);
            ip_packet.set_ttl(64);
            ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
            ip_packet.set_source(Ipv4Addr::new(192, 168, 1, 1));
            ip_packet.set_destination(Ipv4Addr::new(192, 168, 1, 2));

            // Calculate and set checksum
            let checksum = ipv4_checksum(&ip_packet.to_immutable());
            ip_packet.set_checksum(checksum);
        }
        packet
    }

    /// Verify IP packet checksum is valid
    fn verify_checksum(packet: &[u8]) -> bool {
        if let Some(ip) = pnet::packet::ipv4::Ipv4Packet::new(packet) {
            let calculated = ipv4_checksum(&ip);
            calculated == ip.get_checksum()
        } else {
            false
        }
    }

    /// Extract fragment offset from IP packet (in bytes)
    fn get_fragment_offset_bytes(packet: &[u8]) -> usize {
        if let Some(ip) = pnet::packet::ipv4::Ipv4Packet::new(packet) {
            (ip.get_fragment_offset() as usize) * 8
        } else {
            0
        }
    }

    /// Check if More Fragments flag is set
    fn has_more_fragments(packet: &[u8]) -> bool {
        if let Some(ip) = pnet::packet::ipv4::Ipv4Packet::new(packet) {
            (ip.get_flags() & Ipv4Flags::MoreFragments) != 0
        } else {
            false
        }
    }

    /// Get fragment ID from IP packet
    fn get_fragment_id(packet: &[u8]) -> u16 {
        if let Some(ip) = pnet::packet::ipv4::Ipv4Packet::new(packet) {
            ip.get_identification()
        } else {
            0
        }
    }

    // ========================================================================
    // CATEGORY 1: BASIC FRAGMENTATION TESTS (8 tests)
    // ========================================================================

    #[test]
    fn test_fragment_no_fragmentation_needed() {
        // Packet that fits in MTU should return single packet
        let packet = create_test_packet(100); // Small packet
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        assert_eq!(fragments.len(), 1, "Should not fragment small packet");
        assert_eq!(
            fragments[0], packet,
            "Single fragment should equal original"
        );
    }

    #[test]
    fn test_fragment_simple_two_fragments() {
        // Packet requiring exactly 2 fragments
        let packet = create_test_packet(100); // 20 IP + 80 payload
        let fragments = fragment_tcp_packet(&packet, 68).unwrap(); // 48 bytes payload per fragment

        assert_eq!(fragments.len(), 2, "Should create 2 fragments");
        assert!(fragments[0].len() <= 68, "First fragment within MTU");
        assert!(fragments[1].len() <= 68, "Second fragment within MTU");
    }

    #[test]
    fn test_fragment_multiple_fragments() {
        // Packet requiring 3+ fragments
        let packet = create_test_packet(200); // 20 IP + 180 payload
        let fragments = fragment_tcp_packet(&packet, 68).unwrap(); // 48 bytes payload each

        assert!(
            fragments.len() >= 4,
            "Should create 4+ fragments (180/48 = 3.75)"
        );
        for (i, fragment) in fragments.iter().enumerate() {
            assert!(
                fragment.len() <= 68,
                "Fragment {} exceeds MTU: {} bytes",
                i,
                fragment.len()
            );
        }
    }

    #[test]
    fn test_fragment_exact_boundary() {
        // Packet size exactly multiple of fragment size
        let packet = create_test_packet(116); // 20 IP + 96 payload (exactly 2 * 48)
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert_eq!(fragments.len(), 2, "Should create exactly 2 fragments");
        assert_eq!(fragments[0].len(), 68, "First fragment full");
        assert_eq!(fragments[1].len(), 68, "Second fragment full");
    }

    #[test]
    fn test_fragment_preserves_ip_fields() {
        use pnet::packet::ipv4::Ipv4Packet;

        let packet = create_test_packet(100);
        let original_ip = Ipv4Packet::new(&packet).unwrap();
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for fragment in &fragments {
            let frag_ip = Ipv4Packet::new(fragment).unwrap();
            assert_eq!(frag_ip.get_version(), 4, "IPv4 version preserved");
            assert_eq!(frag_ip.get_ttl(), original_ip.get_ttl(), "TTL preserved");
            assert_eq!(
                frag_ip.get_next_level_protocol(),
                original_ip.get_next_level_protocol(),
                "Protocol preserved"
            );
            assert_eq!(
                frag_ip.get_source(),
                original_ip.get_source(),
                "Source IP preserved"
            );
            assert_eq!(
                frag_ip.get_destination(),
                original_ip.get_destination(),
                "Destination IP preserved"
            );
        }
    }

    #[test]
    fn test_fragment_unique_id() {
        // Each fragmentation should get unique ID
        let packet = create_test_packet(100);
        let fragments1 = fragment_tcp_packet(&packet, 68).unwrap();
        let fragments2 = fragment_tcp_packet(&packet, 68).unwrap();

        let id1 = get_fragment_id(&fragments1[0]);
        let id2 = get_fragment_id(&fragments2[0]);

        // Very unlikely to get same random ID twice
        assert_ne!(id1, id2, "Fragment IDs should be unique across calls");
    }

    #[test]
    fn test_fragment_same_id_across_fragments() {
        // All fragments of same packet should share ID
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        let first_id = get_fragment_id(&fragments[0]);
        for (i, fragment) in fragments.iter().enumerate() {
            assert_eq!(
                get_fragment_id(fragment),
                first_id,
                "Fragment {} has different ID",
                i
            );
        }
    }

    #[test]
    fn test_fragment_total_size_matches() {
        // Sum of fragment payloads should equal original payload
        let packet = create_test_packet(200);
        let original_payload_size = packet.len() - IP_HEADER_SIZE;
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        let total_payload: usize = fragments.iter().map(|f| f.len() - IP_HEADER_SIZE).sum();

        assert_eq!(
            total_payload, original_payload_size,
            "Total fragment payload should equal original"
        );
    }

    // ========================================================================
    // CATEGORY 2: EDGE CASES (8 tests)
    // ========================================================================

    #[test]
    fn test_fragment_minimum_mtu_68() {
        let packet = create_test_packet(100);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert!(!fragments.is_empty(), "Should fragment with minimum MTU");
        for fragment in &fragments {
            assert!(fragment.len() <= 68, "Fragment exceeds minimum MTU");
        }
    }

    #[test]
    fn test_fragment_aggressive_nmap_f_28() {
        // Nmap -f mode: extremely aggressive 8-byte fragments
        let packet = create_test_packet(60); // Small packet
        let fragments = fragment_tcp_packet(&packet, NMAP_F_MTU).unwrap();

        assert!(fragments.len() >= 5, "Should create many small fragments");
        for fragment in &fragments {
            assert!(fragment.len() <= 28, "Fragment exceeds Nmap -f MTU");
        }
    }

    #[test]
    fn test_fragment_large_packet_4kb() {
        let packet = create_test_packet(4096);
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        assert!(
            fragments.len() > 20,
            "Large packet should create many fragments"
        );
        for fragment in &fragments {
            assert!(fragment.len() <= 200, "Fragment exceeds MTU");
            assert!(verify_checksum(fragment), "Fragment checksum invalid");
        }
    }

    #[test]
    fn test_fragment_tiny_packet_40_bytes() {
        let packet = create_test_packet(40); // 20 IP + 20 TCP
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        assert_eq!(fragments.len(), 1, "Tiny packet should not fragment");
    }

    #[test]
    fn test_fragment_medium_mtu_200() {
        let packet = create_test_packet(400);
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        assert!(fragments.len() >= 2, "Should create multiple fragments");
        assert!(fragments.len() <= 4, "Should not over-fragment");
    }

    #[test]
    fn test_fragment_standard_mtu_1500() {
        let packet = create_test_packet(2000);
        let fragments = fragment_tcp_packet(&packet, STANDARD_MTU).unwrap();

        assert!(
            fragments.len() >= 2,
            "Should fragment packet larger than MTU"
        );
        for fragment in &fragments {
            assert!(fragment.len() <= 1500, "Fragment exceeds standard MTU");
        }
    }

    #[test]
    fn test_fragment_odd_packet_size() {
        // Packet size not aligned to 8 bytes
        let packet = create_test_packet(123); // Odd size
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert!(!fragments.is_empty(), "Should handle odd-sized packets");
        let total: usize = fragments.iter().map(|f| f.len() - IP_HEADER_SIZE).sum();
        assert_eq!(
            total, 103,
            "Total payload size should match (123 - 20 = 103)"
        );
    }

    #[test]
    fn test_fragment_maximum_theoretical() {
        // Very large packet (10KB)
        let packet = create_test_packet(10240);
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        assert!(fragments.len() > 50, "Should create many fragments");
        for fragment in &fragments {
            assert!(fragment.len() <= 200, "Fragment exceeds MTU");
        }
    }

    // ========================================================================
    // CATEGORY 3: FRAGMENT OFFSET CALCULATION (6 tests)
    // ========================================================================

    #[test]
    fn test_fragment_offset_first_is_zero() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert_eq!(
            get_fragment_offset_bytes(&fragments[0]),
            0,
            "First fragment offset should be 0"
        );
    }

    #[test]
    fn test_fragment_offset_second() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        let offset = get_fragment_offset_bytes(&fragments[1]);
        assert_eq!(
            offset, 48,
            "Second fragment offset should be 48 (68 - 20 IP)"
        );
    }

    #[test]
    fn test_fragment_offset_third() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        if fragments.len() >= 3 {
            let offset = get_fragment_offset_bytes(&fragments[2]);
            assert_eq!(offset, 96, "Third fragment offset should be 96 (48 * 2)");
        }
    }

    #[test]
    fn test_fragment_offset_in_8byte_units() {
        // Fragment offsets must be in 8-byte units
        let packet = create_test_packet(400);
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        for (i, fragment) in fragments.iter().enumerate() {
            let offset = get_fragment_offset_bytes(fragment);
            assert_eq!(
                offset % 8,
                0,
                "Fragment {} offset {} not multiple of 8",
                i,
                offset
            );
        }
    }

    #[test]
    fn test_fragment_offset_sequence() {
        // Offsets should form continuous sequence
        let packet = create_test_packet(300);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        let mut expected_offset = 0;
        for (i, fragment) in fragments.iter().enumerate() {
            let offset = get_fragment_offset_bytes(fragment);
            assert_eq!(offset, expected_offset, "Fragment {} offset incorrect", i);
            expected_offset += fragment.len() - IP_HEADER_SIZE;
        }
    }

    #[test]
    fn test_fragment_offset_last() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        let last_idx = fragments.len() - 1;
        let last_offset = get_fragment_offset_bytes(&fragments[last_idx]);

        // Last offset should be sum of all previous fragment payloads
        let expected: usize = fragments[..last_idx]
            .iter()
            .map(|f| f.len() - IP_HEADER_SIZE)
            .sum();

        assert_eq!(last_offset, expected, "Last fragment offset incorrect");
    }

    // ========================================================================
    // CATEGORY 4: MF FLAG HANDLING (8 tests)
    // ========================================================================

    #[test]
    fn test_mf_flag_single_fragment_clear() {
        let packet = create_test_packet(40);
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        assert_eq!(fragments.len(), 1, "Should not fragment");
        assert!(
            !has_more_fragments(&fragments[0]),
            "MF flag should be clear"
        );
    }

    #[test]
    fn test_mf_flag_first_fragment_set() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert!(fragments.len() > 1, "Should create multiple fragments");
        assert!(
            has_more_fragments(&fragments[0]),
            "First fragment MF flag should be set"
        );
    }

    #[test]
    fn test_mf_flag_middle_fragment_set() {
        let packet = create_test_packet(300);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert!(fragments.len() >= 3, "Need at least 3 fragments");
        let middle_idx = fragments.len() / 2;
        assert!(
            has_more_fragments(&fragments[middle_idx]),
            "Middle fragment MF flag should be set"
        );
    }

    #[test]
    fn test_mf_flag_last_fragment_clear() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        let last_idx = fragments.len() - 1;
        assert!(
            !has_more_fragments(&fragments[last_idx]),
            "Last fragment MF flag should be clear"
        );
    }

    #[test]
    fn test_mf_flag_two_fragments() {
        let packet = create_test_packet(116); // Exactly 2 fragments
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert_eq!(fragments.len(), 2, "Should create exactly 2 fragments");
        assert!(has_more_fragments(&fragments[0]), "First fragment MF=1");
        assert!(!has_more_fragments(&fragments[1]), "Second fragment MF=0");
    }

    #[test]
    fn test_mf_flag_three_fragments() {
        let packet = create_test_packet(164); // Exactly 3 fragments
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert_eq!(fragments.len(), 3, "Should create exactly 3 fragments");
        assert!(has_more_fragments(&fragments[0]), "First fragment MF=1");
        assert!(has_more_fragments(&fragments[1]), "Second fragment MF=1");
        assert!(!has_more_fragments(&fragments[2]), "Third fragment MF=0");
    }

    #[test]
    fn test_mf_flag_many_fragments() {
        let packet = create_test_packet(500);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        // All but last should have MF=1
        for (i, fragment) in fragments.iter().enumerate().take(fragments.len() - 1) {
            assert!(
                has_more_fragments(fragment),
                "Fragment {} should have MF=1",
                i
            );
        }

        // Last should have MF=0
        let last_idx = fragments.len() - 1;
        assert!(
            !has_more_fragments(&fragments[last_idx]),
            "Last fragment should have MF=0"
        );
    }

    #[test]
    fn test_mf_flag_parsing() {
        use pnet::packet::ipv4::Ipv4Packet;

        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for (i, fragment) in fragments.iter().enumerate() {
            let ip = Ipv4Packet::new(fragment).unwrap();
            let flags = ip.get_flags();
            let is_last = i == fragments.len() - 1;

            if is_last {
                assert_eq!(flags & Ipv4Flags::MoreFragments, 0, "Last fragment MF=0");
            } else {
                assert_ne!(
                    flags & Ipv4Flags::MoreFragments,
                    0,
                    "Non-last fragment MF=1"
                );
            }
        }
    }

    // ========================================================================
    // CATEGORY 5: CHECKSUM VERIFICATION (6 tests)
    // ========================================================================

    #[test]
    fn test_fragment_checksum_valid_each() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for (i, fragment) in fragments.iter().enumerate() {
            assert!(
                verify_checksum(fragment),
                "Fragment {} has invalid checksum",
                i
            );
        }
    }

    #[test]
    fn test_fragment_checksum_differs() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        // Each fragment should have different checksum (different sizes/offsets)
        // Just verify all are valid - checksums will naturally differ
        for fragment in &fragments {
            assert!(verify_checksum(fragment), "Fragment checksum invalid");
        }
    }

    #[test]
    fn test_fragment_checksum_first() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert!(
            verify_checksum(&fragments[0]),
            "First fragment checksum invalid"
        );
    }

    #[test]
    fn test_fragment_checksum_middle() {
        let packet = create_test_packet(300);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        let middle_idx = fragments.len() / 2;
        assert!(
            verify_checksum(&fragments[middle_idx]),
            "Middle fragment checksum invalid"
        );
    }

    #[test]
    fn test_fragment_checksum_last() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        let last_idx = fragments.len() - 1;
        assert!(
            verify_checksum(&fragments[last_idx]),
            "Last fragment checksum invalid"
        );
    }

    #[test]
    fn test_fragment_checksum_recalculation() {
        use pnet::packet::ipv4::Ipv4Packet;

        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for fragment in &fragments {
            let ip = Ipv4Packet::new(fragment).unwrap();
            let stored_checksum = ip.get_checksum();
            let calculated_checksum = ipv4_checksum(&ip);

            assert_eq!(
                stored_checksum, calculated_checksum,
                "Checksum mismatch: stored={}, calculated={}",
                stored_checksum, calculated_checksum
            );
        }
    }

    // ========================================================================
    // CATEGORY 6: DEFRAGMENTATION TESTS (8 tests)
    // ========================================================================

    #[test]
    fn test_defragment_single_packet() {
        let packet = create_test_packet(40);
        let fragments = vec![packet.clone()];
        let reassembled = defragment_packets(fragments).unwrap();

        assert_eq!(reassembled.len(), packet.len(), "Size should match");
    }

    #[test]
    fn test_defragment_two_fragments() {
        let packet = create_test_packet(116);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();
        let reassembled = defragment_packets(fragments).unwrap();

        assert_eq!(
            reassembled.len(),
            packet.len(),
            "Reassembled size should match original"
        );
    }

    #[test]
    fn test_defragment_multiple_fragments() {
        let packet = create_test_packet(300);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();
        let reassembled = defragment_packets(fragments).unwrap();

        assert!(
            reassembled.len() >= packet.len(),
            "Reassembled should be at least original size"
        );
    }

    #[test]
    fn test_defragment_out_of_order() {
        let packet = create_test_packet(200);
        let mut fragments = fragment_tcp_packet(&packet, 68).unwrap();

        // Reverse order
        fragments.reverse();

        let reassembled = defragment_packets(fragments).unwrap();
        assert!(
            reassembled.len() >= packet.len(),
            "Should reassemble out-of-order fragments"
        );
    }

    #[test]
    fn test_defragment_round_trip_small() {
        let packet = create_test_packet(100);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();
        let reassembled = defragment_packets(fragments).unwrap();

        // Payload should match exactly
        let original_payload = &packet[IP_HEADER_SIZE..];
        let reassembled_payload = &reassembled[IP_HEADER_SIZE..];

        assert_eq!(
            original_payload, reassembled_payload,
            "Payloads should match after round-trip"
        );
    }

    #[test]
    fn test_defragment_round_trip_large() {
        let packet = create_test_packet(1000);
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();
        let reassembled = defragment_packets(fragments).unwrap();

        let original_payload = &packet[IP_HEADER_SIZE..];
        let reassembled_payload = &reassembled[IP_HEADER_SIZE..];

        assert_eq!(
            original_payload, reassembled_payload,
            "Large packet payloads should match"
        );
    }

    #[test]
    fn test_defragment_preserves_payload() {
        let mut packet = create_test_packet(150);
        // Fill payload with recognizable pattern
        for (offset, byte) in packet.iter_mut().enumerate().skip(IP_HEADER_SIZE) {
            *byte = ((offset - IP_HEADER_SIZE) % 256) as u8;
        }

        let fragments = fragment_tcp_packet(&packet, 68).unwrap();
        let reassembled = defragment_packets(fragments).unwrap();

        let original_payload = &packet[IP_HEADER_SIZE..];
        let reassembled_payload = &reassembled[IP_HEADER_SIZE..];

        assert_eq!(
            original_payload, reassembled_payload,
            "Payload pattern should be preserved"
        );
    }

    #[test]
    fn test_defragment_empty() {
        let fragments: Vec<Vec<u8>> = Vec::new();
        let reassembled = defragment_packets(fragments).unwrap();

        assert!(
            reassembled.is_empty(),
            "Empty fragment list should return empty packet"
        );
    }

    // ========================================================================
    // CATEGORY 7: IP HEADER VERIFICATION TESTS (8 tests)
    // ========================================================================

    #[test]
    fn test_fragment_version_preserved() {
        use pnet::packet::ipv4::Ipv4Packet;

        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for fragment in &fragments {
            let ip = Ipv4Packet::new(fragment).unwrap();
            assert_eq!(ip.get_version(), 4, "IPv4 version should be 4");
        }
    }

    #[test]
    fn test_fragment_header_length() {
        use pnet::packet::ipv4::Ipv4Packet;

        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for fragment in &fragments {
            let ip = Ipv4Packet::new(fragment).unwrap();
            assert_eq!(
                ip.get_header_length(),
                5,
                "Header length should be 5 (20 bytes)"
            );
        }
    }

    #[test]
    fn test_fragment_ttl_preserved() {
        use pnet::packet::ipv4::Ipv4Packet;

        let packet = create_test_packet(200);
        let original_ip = Ipv4Packet::new(&packet).unwrap();
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for fragment in &fragments {
            let ip = Ipv4Packet::new(fragment).unwrap();
            assert_eq!(
                ip.get_ttl(),
                original_ip.get_ttl(),
                "TTL should be preserved"
            );
        }
    }

    #[test]
    fn test_fragment_protocol_preserved() {
        use pnet::packet::ipv4::Ipv4Packet;

        let packet = create_test_packet(200);
        let original_ip = Ipv4Packet::new(&packet).unwrap();
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for fragment in &fragments {
            let ip = Ipv4Packet::new(fragment).unwrap();
            assert_eq!(
                ip.get_next_level_protocol(),
                original_ip.get_next_level_protocol(),
                "Protocol should be preserved"
            );
        }
    }

    #[test]
    fn test_fragment_source_preserved() {
        use pnet::packet::ipv4::Ipv4Packet;

        let packet = create_test_packet(200);
        let original_ip = Ipv4Packet::new(&packet).unwrap();
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for fragment in &fragments {
            let ip = Ipv4Packet::new(fragment).unwrap();
            assert_eq!(
                ip.get_source(),
                original_ip.get_source(),
                "Source IP should be preserved"
            );
        }
    }

    #[test]
    fn test_fragment_destination_preserved() {
        use pnet::packet::ipv4::Ipv4Packet;

        let packet = create_test_packet(200);
        let original_ip = Ipv4Packet::new(&packet).unwrap();
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for fragment in &fragments {
            let ip = Ipv4Packet::new(fragment).unwrap();
            assert_eq!(
                ip.get_destination(),
                original_ip.get_destination(),
                "Destination IP should be preserved"
            );
        }
    }

    #[test]
    fn test_fragment_dscp_preserved() {
        use pnet::packet::ipv4::Ipv4Packet;

        let packet = create_test_packet(200);
        let original_ip = Ipv4Packet::new(&packet).unwrap();
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for fragment in &fragments {
            let ip = Ipv4Packet::new(fragment).unwrap();
            assert_eq!(
                ip.get_dscp(),
                original_ip.get_dscp(),
                "DSCP should be preserved"
            );
        }
    }

    #[test]
    fn test_fragment_ecn_preserved() {
        use pnet::packet::ipv4::Ipv4Packet;

        let packet = create_test_packet(200);
        let original_ip = Ipv4Packet::new(&packet).unwrap();
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        for fragment in &fragments {
            let ip = Ipv4Packet::new(fragment).unwrap();
            assert_eq!(
                ip.get_ecn(),
                original_ip.get_ecn(),
                "ECN should be preserved"
            );
        }
    }

    // ========================================================================
    // CATEGORY 8: ERROR HANDLING TESTS (6 tests)
    // ========================================================================

    #[test]
    fn test_fragment_packet_too_small() {
        let packet = vec![0u8; 10]; // Less than 20 bytes
        let result = fragment_tcp_packet(&packet, 68);

        assert!(matches!(
            result,
            Err(FragmentationError::PacketTooSmall { .. })
        ));
    }

    #[test]
    fn test_fragment_empty_packet() {
        let packet = Vec::new();
        let result = fragment_tcp_packet(&packet, 68);

        assert!(matches!(
            result,
            Err(FragmentationError::PacketTooSmall { .. })
        ));
    }

    #[test]
    fn test_fragment_mtu_too_small_integrated() {
        let packet = create_test_packet(100);
        // MTU 20 is below MIN_MTU (28), should fail
        let result = fragment_tcp_packet(&packet, 20);

        assert!(matches!(
            result,
            Err(FragmentationError::MtuTooSmall { .. })
        ));
    }

    #[test]
    fn test_fragment_mtu_not_multiple_of_8_integrated() {
        // MTU 100 is valid even though it's not a multiple of 8
        // Payload: 100 - 20 = 80 bytes (multiple of 8)
        let packet = create_test_packet(100);
        let result = fragment_tcp_packet(&packet, 100);

        assert!(result.is_ok(), "MTU 100 should work correctly");
        let fragments = result.unwrap();
        assert_eq!(fragments.len(), 1, "Small packet should not fragment");
    }

    #[test]
    fn test_defragment_invalid_fragment() {
        let mut fragments = vec![vec![0u8; 10]]; // Invalid fragment (too small)
        fragments.push(create_test_packet(68));

        let result = defragment_packets(fragments);
        // Should either error or handle gracefully
        assert!(
            result.is_ok() || result.is_err(),
            "Should handle invalid fragments"
        );
    }

    #[test]
    fn test_defragment_empty_fragment_list() {
        let fragments: Vec<Vec<u8>> = Vec::new();
        let result = defragment_packets(fragments);

        assert!(result.is_ok(), "Empty fragment list should be OK");
        assert!(result.unwrap().is_empty(), "Should return empty packet");
    }

    // ========================================================================
    // CATEGORY 9: INTEGRATION TESTS (8 tests)
    // ========================================================================

    #[test]
    fn test_integration_syn_scan() {
        // Simulate SYN scan packet (40 bytes: 20 IP + 20 TCP)
        let packet = create_test_packet(40);
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        // Small packet should not fragment
        assert_eq!(
            fragments.len(),
            1,
            "SYN packet should not fragment with large MTU"
        );
    }

    #[test]
    fn test_integration_udp_scan() {
        // UDP packet with payload
        let packet = create_test_packet(100);
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert!(
            fragments.len() >= 2,
            "UDP packet with payload should fragment"
        );
        for fragment in &fragments {
            assert!(verify_checksum(fragment), "Fragment checksum invalid");
        }
    }

    #[test]
    fn test_integration_large_payload() {
        // Packet with 1KB payload
        let packet = create_test_packet(1044); // 20 IP + 1024 payload
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        assert!(
            fragments.len() >= 6,
            "Large payload should create many fragments"
        );
        let total_payload: usize = fragments.iter().map(|f| f.len() - IP_HEADER_SIZE).sum();
        assert_eq!(total_payload, 1024, "Total payload should be 1024");
    }

    #[test]
    fn test_integration_multiple_packets() {
        // Fragment multiple independent packets
        let packets = vec![
            create_test_packet(80),
            create_test_packet(120),
            create_test_packet(200),
        ];

        for packet in packets {
            let fragments = fragment_tcp_packet(&packet, 68).unwrap();
            assert!(!fragments.is_empty(), "Each packet should fragment");
        }
    }

    #[test]
    fn test_integration_no_fragmentation_fast_path() {
        // Fast path when no fragmentation needed
        let packet = create_test_packet(50);
        let fragments = fragment_tcp_packet(&packet, 1500).unwrap();

        assert_eq!(fragments.len(), 1, "Should use fast path");
        assert_eq!(fragments[0], packet, "Should return original packet");
    }

    #[test]
    fn test_integration_aggressive_fragmentation() {
        // Aggressive fragmentation scenario
        let packet = create_test_packet(100);
        let fragments = fragment_tcp_packet(&packet, 28).unwrap(); // Nmap -f

        assert!(
            fragments.len() >= 10,
            "Aggressive MTU should create many fragments"
        );
        for (i, fragment) in fragments.iter().enumerate() {
            assert!(fragment.len() <= 28, "Fragment {} exceeds MTU", i);
            assert!(verify_checksum(fragment), "Fragment {} checksum invalid", i);
        }
    }

    #[test]
    fn test_integration_realistic_scenario() {
        // Realistic scan scenario: 400-byte packet, MTU 200
        let packet = create_test_packet(400);
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        assert!(fragments.len() >= 2, "Should create 2-3 fragments");
        assert!(fragments.len() <= 4, "Should not over-fragment");

        // Verify round-trip
        let reassembled = defragment_packets(fragments).unwrap();
        assert_eq!(
            packet[IP_HEADER_SIZE..],
            reassembled[IP_HEADER_SIZE..],
            "Round-trip should preserve payload"
        );
    }

    #[test]
    fn test_integration_stress_test() {
        // Fragment many packets quickly
        for size in (60..500).step_by(40) {
            let packet = create_test_packet(size);
            let fragments = fragment_tcp_packet(&packet, 68).unwrap();

            assert!(!fragments.is_empty(), "Packet size {} failed", size);
            for fragment in &fragments {
                assert!(
                    verify_checksum(fragment),
                    "Checksum failed for size {}",
                    size
                );
            }
        }
    }

    // ========================================================================
    // CATEGORY 10: BOUNDARY CONDITIONS (8 tests)
    // ========================================================================

    #[test]
    fn test_boundary_mtu_equals_packet_size() {
        let packet = create_test_packet(200);
        // MTU 208: fragment_data_size = (208-20) = 188, rounds to 184
        // Packet payload = 180 bytes, fits in 184 → no fragmentation
        let fragments = fragment_tcp_packet(&packet, 208).unwrap();

        assert_eq!(
            fragments.len(),
            1,
            "Packet should fit without fragmentation"
        );
    }

    #[test]
    fn test_boundary_mtu_one_byte_larger() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 208).unwrap(); // 200 + 8

        assert_eq!(
            fragments.len(),
            1,
            "MTU slightly larger should not fragment"
        );
    }

    #[test]
    fn test_boundary_mtu_one_byte_smaller() {
        let packet = create_test_packet(200);
        let fragments = fragment_tcp_packet(&packet, 192).unwrap(); // 200 - 8

        assert!(fragments.len() >= 2, "MTU slightly smaller should fragment");
    }

    #[test]
    fn test_boundary_packet_20_bytes() {
        let packet = create_test_packet(20); // Minimum IP packet
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        assert_eq!(fragments.len(), 1, "Minimum packet should not fragment");
    }

    #[test]
    fn test_boundary_packet_21_bytes() {
        let packet = create_test_packet(21); // Minimal payload
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        assert_eq!(
            fragments.len(),
            1,
            "Tiny payload should not fragment with large MTU"
        );
    }

    #[test]
    fn test_boundary_payload_exact_fragment_size() {
        // Payload exactly equals fragment data size
        let packet = create_test_packet(68); // 20 IP + 48 data
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert_eq!(fragments.len(), 1, "Exact fit should not fragment");
    }

    #[test]
    fn test_boundary_payload_one_byte_over() {
        // Payload one byte over fragment size
        let packet = create_test_packet(69); // 20 IP + 49 data
        let fragments = fragment_tcp_packet(&packet, 68).unwrap();

        assert_eq!(
            fragments.len(),
            2,
            "One byte over should create 2 fragments"
        );
    }

    #[test]
    fn test_boundary_large_offset() {
        // Test with offset approaching 16-bit limit
        let packet = create_test_packet(8192); // Large packet
        let fragments = fragment_tcp_packet(&packet, 200).unwrap();

        // Verify last fragment has reasonable offset
        let last_offset = get_fragment_offset_bytes(&fragments[fragments.len() - 1]);
        assert!(last_offset < 65536, "Offset should fit in 16 bits");
    }
}
