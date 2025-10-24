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
pub const MIN_MTU: usize = 68;

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
    if mtu % 8 != 0 {
        return Err(FragmentationError::MtuNotMultipleOf8 { mtu });
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
        let result = validate_mtu(64);
        assert!(matches!(
            result,
            Err(FragmentationError::MtuTooSmall { .. })
        ));
    }

    #[test]
    fn test_validate_mtu_not_multiple_of_8() {
        let result = validate_mtu(100); // Not multiple of 8
        assert!(matches!(
            result,
            Err(FragmentationError::MtuNotMultipleOf8 { .. })
        ));
    }

    #[test]
    fn test_calculate_fragment_data_size() {
        assert_eq!(calculate_fragment_data_size(68), 48); // 68 - 20 = 48
        assert_eq!(calculate_fragment_data_size(200), 176); // 200 - 20 = 180, round down to 176
        assert_eq!(calculate_fragment_data_size(1500), 1480); // 1500 - 20 = 1480
    }

    // Additional tests would go here (fragment_simple_packet, defragmentation, etc.)
}
