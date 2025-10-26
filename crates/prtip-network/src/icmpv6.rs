//! ICMPv6 packet types and builders
//!
//! This module implements ICMPv6 (Internet Control Message Protocol version 6)
//! as defined in RFC 4443. Supports:
//! - Echo Request/Reply (ping) for host discovery
//! - Neighbor Discovery Protocol (NDP) - replaces ARP in IPv6
//! - Router Discovery for network configuration
//! - Destination Unreachable (port unreachable for UDP scanning)
//!
//! # Examples
//!
//! ```no_run
//! use prtip_network::icmpv6::Icmpv6PacketBuilder;
//! use std::net::Ipv6Addr;
//!
//! let src = "2001:db8::1".parse().unwrap();
//! let dst = "2001:db8::2".parse().unwrap();
//!
//! // Create Echo Request (ping)
//! let packet = Icmpv6PacketBuilder::echo_request(1234, 1, vec![0xDE, 0xAD])
//!     .build(src, dst)
//!     .unwrap();
//! ```

use pnet::packet::icmpv6::{Icmpv6Code, Icmpv6Packet, Icmpv6Type, MutableIcmpv6Packet};
use pnet::packet::{MutablePacket, Packet};
use std::net::Ipv6Addr;
use thiserror::Error;

/// Errors that can occur during ICMPv6 packet construction
#[derive(Debug, Error)]
pub enum Icmpv6Error {
    #[error("Failed to create ICMPv6 packet: {0}")]
    PacketBuild(String),

    #[error("Invalid packet length: {0} bytes")]
    InvalidLength(usize),
}

pub type Result<T> = std::result::Result<T, Icmpv6Error>;

/// ICMPv6 packet builder
///
/// Provides methods for constructing various ICMPv6 packet types with
/// proper checksums calculated using the IPv6 pseudo-header.
#[derive(Debug, Clone)]
pub struct Icmpv6PacketBuilder {
    icmp_type: Icmpv6Type,
    code: u8,
    payload: Vec<u8>,
}

impl Icmpv6PacketBuilder {
    /// Create Echo Request (Type 128, for ping/host discovery)
    ///
    /// Used to test if a host is reachable. The host should respond with
    /// an Echo Reply (Type 129) containing the same identifier and sequence.
    ///
    /// # Arguments
    ///
    /// * `identifier` - 16-bit identifier to match request/reply pairs
    /// * `sequence` - 16-bit sequence number (incremented per request)
    /// * `data` - Optional payload data (often timestamp or pattern)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::icmpv6::Icmpv6PacketBuilder;
    /// use std::net::Ipv6Addr;
    ///
    /// let src = "fe80::1".parse().unwrap();
    /// let dst = "fe80::2".parse().unwrap();
    ///
    /// let packet = Icmpv6PacketBuilder::echo_request(1234, 1, vec![0xDE, 0xAD])
    ///     .build(src, dst)
    ///     .unwrap();
    /// ```
    pub fn echo_request(identifier: u16, sequence: u16, data: Vec<u8>) -> Self {
        let mut payload = Vec::with_capacity(4 + data.len());
        payload.extend_from_slice(&identifier.to_be_bytes());
        payload.extend_from_slice(&sequence.to_be_bytes());
        payload.extend_from_slice(&data);

        Self {
            icmp_type: Icmpv6Type::new(128), // Echo Request
            code: 0,
            payload,
        }
    }

    /// Create Neighbor Solicitation (Type 135, NDP - replaces ARP)
    ///
    /// Used to resolve link-layer addresses from IPv6 addresses.
    /// This replaces ARP from IPv4 networks.
    ///
    /// # Arguments
    ///
    /// * `target` - Target IPv6 address to resolve
    /// * `source_ll_addr` - Optional source link-layer (MAC) address
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::icmpv6::Icmpv6PacketBuilder;
    /// use std::net::Ipv6Addr;
    ///
    /// let src = "fe80::1".parse().unwrap();
    /// let dst = "ff02::1:ff00:2".parse().unwrap(); // Solicited-node multicast
    /// let target = "fe80::2".parse().unwrap();
    /// let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    ///
    /// let packet = Icmpv6PacketBuilder::neighbor_solicitation(target, Some(mac))
    ///     .build(src, dst)
    ///     .unwrap();
    /// ```
    pub fn neighbor_solicitation(target: Ipv6Addr, source_ll_addr: Option<[u8; 6]>) -> Self {
        let mut payload = Vec::with_capacity(24);

        // Reserved (4 bytes)
        payload.extend_from_slice(&[0, 0, 0, 0]);

        // Target Address (16 bytes)
        payload.extend_from_slice(&target.octets());

        // Source Link-Layer Address option (if provided)
        if let Some(ll_addr) = source_ll_addr {
            payload.push(1); // Option Type: Source Link-Layer Address
            payload.push(1); // Option Length: 1 (units of 8 bytes)
            payload.extend_from_slice(&ll_addr);
        }

        Self {
            icmp_type: Icmpv6Type::new(135), // Neighbor Solicitation
            code: 0,
            payload,
        }
    }

    /// Create Router Solicitation (Type 133, router discovery)
    ///
    /// Sent by hosts to discover routers on the local network.
    /// Routers respond with Router Advertisement (Type 134).
    ///
    /// # Arguments
    ///
    /// * `source_ll_addr` - Optional source link-layer (MAC) address
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::icmpv6::Icmpv6PacketBuilder;
    /// use std::net::Ipv6Addr;
    ///
    /// let src = "fe80::1".parse().unwrap();
    /// let dst = "ff02::2".parse().unwrap(); // All-routers multicast
    /// let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    ///
    /// let packet = Icmpv6PacketBuilder::router_solicitation(Some(mac))
    ///     .build(src, dst)
    ///     .unwrap();
    /// ```
    pub fn router_solicitation(source_ll_addr: Option<[u8; 6]>) -> Self {
        let mut payload = Vec::with_capacity(8);

        // Reserved (4 bytes)
        payload.extend_from_slice(&[0, 0, 0, 0]);

        // Source Link-Layer Address option (if provided)
        if let Some(ll_addr) = source_ll_addr {
            payload.push(1); // Option Type
            payload.push(1); // Option Length
            payload.extend_from_slice(&ll_addr);
        }

        Self {
            icmp_type: Icmpv6Type::new(133), // Router Solicitation
            code: 0,
            payload,
        }
    }

    /// Build ICMPv6 packet with calculated checksum
    ///
    /// Constructs the complete ICMPv6 packet including header and payload.
    /// The checksum is calculated using the IPv6 pseudo-header (40 bytes).
    ///
    /// # Arguments
    ///
    /// * `src` - Source IPv6 address (for pseudo-header checksum)
    /// * `dst` - Destination IPv6 address (for pseudo-header checksum)
    ///
    /// # Returns
    ///
    /// Complete ICMPv6 packet bytes ready for transmission.
    pub fn build(self, src: Ipv6Addr, dst: Ipv6Addr) -> Result<Vec<u8>> {
        let packet_len = 8 + self.payload.len(); // 4 bytes (type, code, checksum) + 4 bytes (varies) + payload
        let mut buffer = vec![0u8; packet_len];

        // First pass: set type, code, and payload (checksum is 0)
        {
            let mut icmpv6_packet = MutableIcmpv6Packet::new(&mut buffer)
                .ok_or_else(|| Icmpv6Error::PacketBuild("Failed to create ICMPv6 packet".into()))?;

            icmpv6_packet.set_icmpv6_type(self.icmp_type);
            icmpv6_packet.set_icmpv6_code(Icmpv6Code(self.code));
            icmpv6_packet.set_checksum(0); // Temporary

            // Copy payload (pnet handles the "rest of header" field internally)
            if !self.payload.is_empty() {
                let payload_slice = icmpv6_packet.payload_mut();
                if payload_slice.len() >= self.payload.len() {
                    payload_slice[..self.payload.len()].copy_from_slice(&self.payload);
                }
            }
        }

        // Second pass: calculate and set checksum
        let checksum = Self::calculate_checksum(&buffer, src, dst)?;
        {
            let mut icmpv6_packet = MutableIcmpv6Packet::new(&mut buffer)
                .ok_or_else(|| Icmpv6Error::PacketBuild("Failed to create ICMPv6 packet".into()))?;
            icmpv6_packet.set_checksum(checksum);
        }

        Ok(buffer)
    }

    /// Calculate ICMPv6 checksum with IPv6 pseudo-header
    ///
    /// ICMPv6 checksums include a 40-byte pseudo-header containing source/dest
    /// addresses and packet length. This is different from IPv4 ICMP which does
    /// not include a pseudo-header.
    ///
    /// Pseudo-header format:
    /// - Source address (16 bytes)
    /// - Destination address (16 bytes)
    /// - Upper-Layer Packet Length (4 bytes)
    /// - Zero padding (3 bytes)
    /// - Next Header: 58 for ICMPv6 (1 byte)
    fn calculate_checksum(icmpv6_packet: &[u8], src: Ipv6Addr, dst: Ipv6Addr) -> Result<u16> {
        // Build pseudo-header (40 bytes)
        let mut pseudo_header = Vec::with_capacity(40 + icmpv6_packet.len());

        // Source address (16 bytes)
        pseudo_header.extend_from_slice(&src.octets());
        // Destination address (16 bytes)
        pseudo_header.extend_from_slice(&dst.octets());
        // Upper-Layer Packet Length (4 bytes)
        pseudo_header.extend_from_slice(&(icmpv6_packet.len() as u32).to_be_bytes());
        // Zero padding (3 bytes)
        pseudo_header.extend_from_slice(&[0, 0, 0]);
        // Next Header: ICMPv6 = 58 (1 byte)
        pseudo_header.push(58);

        // Append ICMPv6 packet (with checksum field set to 0)
        pseudo_header.extend_from_slice(icmpv6_packet);

        Ok(pnet::util::checksum(&pseudo_header, 1))
    }
}

/// Parse ICMPv6 response packets
///
/// Provides methods for parsing various ICMPv6 response types,
/// useful for interpreting scan results.
pub struct Icmpv6ResponseParser;

impl Icmpv6ResponseParser {
    /// Parse Echo Reply (Type 129)
    ///
    /// Extracts identifier and sequence number from Echo Reply packets.
    /// Returns None if packet is not a valid Echo Reply.
    ///
    /// # Returns
    ///
    /// Tuple of (identifier, sequence) or None if invalid.
    pub fn parse_echo_reply(packet: &[u8]) -> Option<(u16, u16)> {
        if packet.len() < 8 {
            return None;
        }

        let icmpv6 = Icmpv6Packet::new(packet)?;

        if icmpv6.get_icmpv6_type().0 != 129 {
            // Echo Reply
            return None;
        }

        let payload = icmpv6.payload();
        if payload.len() < 4 {
            return None;
        }

        let identifier = u16::from_be_bytes([payload[0], payload[1]]);
        let sequence = u16::from_be_bytes([payload[2], payload[3]]);

        Some((identifier, sequence))
    }

    /// Parse Neighbor Advertisement (Type 136)
    ///
    /// Extracts target address from Neighbor Advertisement packets.
    /// Used for NDP (Neighbor Discovery Protocol) responses.
    ///
    /// # Returns
    ///
    /// Target IPv6 address or None if invalid.
    pub fn parse_neighbor_advertisement(packet: &[u8]) -> Option<Ipv6Addr> {
        if packet.len() < 24 {
            return None;
        }

        let icmpv6 = Icmpv6Packet::new(packet)?;

        if icmpv6.get_icmpv6_type().0 != 136 {
            // Neighbor Advertisement
            return None;
        }

        let payload = icmpv6.payload();
        if payload.len() < 20 {
            return None;
        }

        // Skip flags and reserved (4 bytes), extract target address (16 bytes)
        let mut addr_bytes = [0u8; 16];
        addr_bytes.copy_from_slice(&payload[4..20]);

        Some(Ipv6Addr::from(addr_bytes))
    }

    /// Parse Destination Unreachable - Port Unreachable (Type 1, Code 4)
    ///
    /// Critical for UDP scanning - indicates the port is closed.
    /// Returns destination address and port from the original packet.
    ///
    /// # Returns
    ///
    /// Tuple of (IPv6 address, port) or None if invalid or not port unreachable.
    pub fn parse_port_unreachable(packet: &[u8]) -> Option<(Ipv6Addr, u16)> {
        if packet.len() < 48 {
            return None; // Need at least ICMP header + IPv6 header + partial transport header
        }

        let icmpv6 = Icmpv6Packet::new(packet)?;

        // Type 1 = Destination Unreachable, Code 4 = Port Unreachable
        if icmpv6.get_icmpv6_type().0 != 1 || icmpv6.get_icmpv6_code().0 != 4 {
            return None;
        }

        // Extract original IPv6 packet from ICMPv6 payload
        let payload = icmpv6.payload();
        if payload.len() < 44 {
            return None; // Need at least 4 bytes unused + 40 bytes IPv6 header
        }

        // Skip 4 bytes (unused), then IPv6 header starts at offset 4
        let ipv6_start = 4;

        // Extract destination address from original IPv6 header (bytes 24-39 of IPv6 header)
        let mut dest_bytes = [0u8; 16];
        dest_bytes.copy_from_slice(&payload[ipv6_start + 24..ipv6_start + 40]);
        let dest_addr = Ipv6Addr::from(dest_bytes);

        // Extract port from UDP/TCP header (starts at offset 40 in original IPv6 packet)
        // For both UDP and TCP, destination port is at offset 2-3 of transport header
        let transport_start = ipv6_start + 40;
        if payload.len() >= transport_start + 4 {
            let dest_port =
                u16::from_be_bytes([payload[transport_start + 2], payload[transport_start + 3]]);
            Some((dest_addr, dest_port))
        } else {
            None
        }
    }

    /// Check if packet is ICMPv6
    ///
    /// Quick validation that packet is ICMPv6 (types start at 128).
    pub fn is_icmpv6(packet: &[u8]) -> bool {
        packet.len() >= 8 && packet[0] >= 128 // ICMPv6 types start at 128
    }

    /// Get ICMPv6 type and code
    ///
    /// Extracts type and code from raw packet bytes.
    ///
    /// # Returns
    ///
    /// Tuple of (type, code) or None if packet too short.
    pub fn get_type_code(packet: &[u8]) -> Option<(u8, u8)> {
        if packet.len() < 2 {
            return None;
        }
        Some((packet[0], packet[1]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_request_build() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let packet = Icmpv6PacketBuilder::echo_request(1234, 5678, vec![0xDE, 0xAD])
            .build(src, dst)
            .unwrap();

        assert!(packet.len() >= 12); // 8 (ICMPv6 header) + 4 (id+seq) + variable data
        assert_eq!(packet[0], 128); // Type: Echo Request
        assert_eq!(packet[1], 0); // Code: 0

        // Verify checksum is non-zero (indicates it was calculated)
        let checksum = u16::from_be_bytes([packet[2], packet[3]]);
        assert_ne!(checksum, 0);
    }

    #[test]
    fn test_neighbor_solicitation_build() {
        let src = "fe80::1".parse().unwrap();
        let dst = "ff02::1:ff00:2".parse().unwrap(); // Solicited-node multicast
        let target = "fe80::2".parse().unwrap();
        let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];

        let packet = Icmpv6PacketBuilder::neighbor_solicitation(target, Some(mac))
            .build(src, dst)
            .unwrap();

        assert!(packet.len() >= 32); // 8 (ICMPv6 header) + 4 (reserved) + 16 (target) + 8 (option)
        assert_eq!(packet[0], 135); // Type: Neighbor Solicitation
        assert_eq!(packet[1], 0); // Code: 0

        // Verify checksum is non-zero
        let checksum = u16::from_be_bytes([packet[2], packet[3]]);
        assert_ne!(checksum, 0);
    }

    #[test]
    fn test_neighbor_solicitation_no_ll_addr() {
        let src = "fe80::1".parse().unwrap();
        let dst = "ff02::1:ff00:2".parse().unwrap();
        let target = "fe80::2".parse().unwrap();

        let packet = Icmpv6PacketBuilder::neighbor_solicitation(target, None)
            .build(src, dst)
            .unwrap();

        // Without link-layer option: 8 (ICMPv6) + 4 (reserved) + 16 (target) = 28 bytes
        assert_eq!(packet.len(), 28);
        assert_eq!(packet[0], 135); // Type: Neighbor Solicitation
    }

    #[test]
    fn test_router_solicitation_build() {
        let src = "fe80::1".parse().unwrap();
        let dst = "ff02::2".parse().unwrap(); // All-routers multicast
        let mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];

        let packet = Icmpv6PacketBuilder::router_solicitation(Some(mac))
            .build(src, dst)
            .unwrap();

        assert!(packet.len() >= 16); // 8 (ICMPv6) + 4 (reserved) + 8 (option)
        assert_eq!(packet[0], 133); // Type: Router Solicitation
        assert_eq!(packet[1], 0); // Code: 0
    }

    #[test]
    fn test_echo_reply_parsing() {
        // Create an Echo Request, then simulate parsing a reply
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let packet = Icmpv6PacketBuilder::echo_request(1234, 5678, vec![0xDE, 0xAD])
            .build(src, dst)
            .unwrap();

        // Manually change type to Echo Reply for testing parser
        let mut reply_packet = packet.clone();
        reply_packet[0] = 129; // Type: Echo Reply

        // Recalculate checksum for the modified packet
        let checksum = Icmpv6PacketBuilder::calculate_checksum(&reply_packet, src, dst).unwrap();
        reply_packet[2] = (checksum >> 8) as u8;
        reply_packet[3] = checksum as u8;

        let (id, seq) = Icmpv6ResponseParser::parse_echo_reply(&reply_packet).unwrap();
        assert_eq!(id, 1234);
        assert_eq!(seq, 5678);
    }

    #[test]
    fn test_is_icmpv6() {
        let mut packet = vec![0u8; 8];
        packet[0] = 128; // Echo Request type
        assert!(Icmpv6ResponseParser::is_icmpv6(&packet));

        packet[0] = 64; // Not ICMPv6 (< 128)
        assert!(!Icmpv6ResponseParser::is_icmpv6(&packet));
    }

    #[test]
    fn test_get_type_code() {
        let mut packet = vec![0u8; 8];
        packet[0] = 128; // Type: Echo Request
        packet[1] = 0; // Code: 0

        let (typ, code) = Icmpv6ResponseParser::get_type_code(&packet).unwrap();
        assert_eq!(typ, 128);
        assert_eq!(code, 0);
    }

    #[test]
    fn test_get_type_code_too_short() {
        let packet = vec![0u8; 1];
        assert!(Icmpv6ResponseParser::get_type_code(&packet).is_none());
    }

    #[test]
    fn test_echo_request_empty_data() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let packet = Icmpv6PacketBuilder::echo_request(100, 200, vec![])
            .build(src, dst)
            .unwrap();

        // Minimum size: 8 bytes ICMPv6 header + 4 bytes (id + seq) = 12 bytes
        assert_eq!(packet.len(), 12);
        assert_eq!(packet[0], 128); // Echo Request
    }

    #[test]
    fn test_checksum_different_addresses() {
        let src1 = "2001:db8::1".parse().unwrap();
        let src2 = "2001:db8::ff".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let packet1 = Icmpv6PacketBuilder::echo_request(100, 200, vec![0xAB])
            .build(src1, dst)
            .unwrap();

        let packet2 = Icmpv6PacketBuilder::echo_request(100, 200, vec![0xAB])
            .build(src2, dst)
            .unwrap();

        // Checksums should be different for different source addresses
        let checksum1 = u16::from_be_bytes([packet1[2], packet1[3]]);
        let checksum2 = u16::from_be_bytes([packet2[2], packet2[3]]);
        assert_ne!(checksum1, checksum2);
    }
}
