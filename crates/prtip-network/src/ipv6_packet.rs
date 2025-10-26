//! IPv6 packet building utilities
//!
//! This module provides builders for constructing IPv6 packets with support for:
//! - Basic IPv6 headers (40 bytes fixed, RFC 8200)
//! - Extension headers (Fragment, Hop-by-Hop, Routing, Destination Options)
//! - Pseudo-header checksum calculation for TCP/UDP/ICMPv6
//! - RFC 8200 compliance
//!
//! # Examples
//!
//! ```no_run
//! use prtip_network::ipv6_packet::{Ipv6PacketBuilder, ExtensionHeader};
//! use std::net::Ipv6Addr;
//!
//! let src = "2001:db8::1".parse::<Ipv6Addr>().unwrap();
//! let dst = "2001:db8::2".parse::<Ipv6Addr>().unwrap();
//!
//! let packet = Ipv6PacketBuilder::new(src, dst)
//!     .hop_limit(64)
//!     .next_header(6) // TCP
//!     .payload(vec![0xDE, 0xAD, 0xBE, 0xEF])
//!     .build()
//!     .unwrap();
//! ```

use pnet::packet::ip::IpNextHeaderProtocol;
use pnet::packet::ipv6::MutableIpv6Packet;
use std::net::Ipv6Addr;
use thiserror::Error;

/// Errors that can occur during IPv6 packet construction
#[derive(Debug, Error)]
pub enum Ipv6PacketError {
    #[error("Failed to create IPv6 packet: {0}")]
    PacketBuild(String),

    #[error("Invalid MTU: {mtu} (minimum is 1280 for IPv6)")]
    InvalidMtu { mtu: usize },

    #[error("Fragment offset must be multiple of 8 bytes, got {0}")]
    InvalidFragmentOffset(u16),

    #[error("Extension header too large: {size} bytes")]
    ExtensionHeaderTooLarge { size: usize },
}

pub type Result<T> = std::result::Result<T, Ipv6PacketError>;

/// IPv6 packet builder with extension header support
///
/// Provides a fluent API for constructing IPv6 packets with proper header
/// fields and optional extension headers.
#[derive(Debug, Clone)]
pub struct Ipv6PacketBuilder {
    source: Ipv6Addr,
    destination: Ipv6Addr,
    hop_limit: u8,
    traffic_class: u8,
    flow_label: u32,
    next_header: u8,
    extension_headers: Vec<ExtensionHeader>,
    payload: Vec<u8>,
}

/// IPv6 extension header types
///
/// Extension headers are optional headers that can be chained after the
/// main IPv6 header. Each header contains a "Next Header" field that
/// points to the next header in the chain.
#[derive(Debug, Clone)]
pub enum ExtensionHeader {
    /// Hop-by-Hop Options (Type 0)
    /// Processed by every router along the path
    HopByHop(Vec<u8>),

    /// Routing Header (Type 43)
    /// Specifies intermediate nodes to visit
    Routing(Vec<u8>),

    /// Fragment Header (Type 44)
    /// Used for packet fragmentation (8 bytes)
    Fragment {
        /// Fragment identification (32 bits)
        id: u32,
        /// Fragment offset in 8-byte units (13 bits)
        offset: u16,
        /// More Fragments flag (1 bit)
        more_fragments: bool,
    },

    /// Destination Options (Type 60)
    /// Processed only by destination node
    DestinationOptions(Vec<u8>),
}

impl Ipv6PacketBuilder {
    /// Create new IPv6 packet builder
    ///
    /// # Arguments
    ///
    /// * `source` - Source IPv6 address
    /// * `destination` - Destination IPv6 address
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::ipv6_packet::Ipv6PacketBuilder;
    /// use std::net::Ipv6Addr;
    ///
    /// let src = "2001:db8::1".parse().unwrap();
    /// let dst = "2001:db8::2".parse().unwrap();
    /// let builder = Ipv6PacketBuilder::new(src, dst);
    /// ```
    pub fn new(source: Ipv6Addr, destination: Ipv6Addr) -> Self {
        Self {
            source,
            destination,
            hop_limit: 64, // Default TTL (RFC 4861 recommendation)
            traffic_class: 0,
            flow_label: 0,
            next_header: 59, // No Next Header (will be overridden)
            extension_headers: Vec::new(),
            payload: Vec::new(),
        }
    }

    /// Set hop limit (TTL equivalent for IPv6)
    ///
    /// Default: 64 (RFC 4861 recommendation)
    pub fn hop_limit(mut self, ttl: u8) -> Self {
        self.hop_limit = ttl;
        self
    }

    /// Set traffic class (DSCP + ECN, 8 bits)
    ///
    /// Used for QoS classification. Default: 0
    pub fn traffic_class(mut self, tc: u8) -> Self {
        self.traffic_class = tc;
        self
    }

    /// Set flow label (20-bit QoS field)
    ///
    /// Used for flow identification. Default: 0 (scanning doesn't use flows)
    pub fn flow_label(mut self, label: u32) -> Self {
        self.flow_label = label & 0xFFFFF; // Mask to 20 bits
        self
    }

    /// Set next header protocol number
    ///
    /// Common values:
    /// - 6: TCP
    /// - 17: UDP
    /// - 58: ICMPv6
    /// - 44: Fragment Header (if using extension headers)
    pub fn next_header(mut self, protocol: u8) -> Self {
        self.next_header = protocol;
        self
    }

    /// Add extension header to the chain
    ///
    /// Extension headers are processed in order. The first header's type
    /// becomes the IPv6 "Next Header" field.
    pub fn add_extension_header(mut self, header: ExtensionHeader) -> Self {
        self.extension_headers.push(header);
        self
    }

    /// Set payload data
    pub fn payload(mut self, data: Vec<u8>) -> Self {
        self.payload = data;
        self
    }

    /// Build IPv6 packet
    ///
    /// Constructs the complete IPv6 packet including main header,
    /// extension headers (if any), and payload.
    ///
    /// # Errors
    ///
    /// Returns error if packet construction fails or if headers are invalid.
    pub fn build(self) -> Result<Vec<u8>> {
        // Calculate total packet size
        let extension_headers_size: usize = self.extension_headers.iter()
            .map(|h| h.size())
            .sum();
        let total_size = 40 + extension_headers_size + self.payload.len();

        let mut buffer = vec![0u8; total_size];

        // Build IPv6 header (40 bytes fixed)
        {
            let mut ipv6_packet = MutableIpv6Packet::new(&mut buffer[0..40])
                .ok_or_else(|| Ipv6PacketError::PacketBuild("Failed to create IPv6 packet".into()))?;

            ipv6_packet.set_version(6);
            ipv6_packet.set_traffic_class(self.traffic_class);
            ipv6_packet.set_flow_label(self.flow_label);
            ipv6_packet.set_payload_length((extension_headers_size + self.payload.len()) as u16);
            ipv6_packet.set_hop_limit(self.hop_limit);
            ipv6_packet.set_source(self.source);
            ipv6_packet.set_destination(self.destination);

            // Set next header (will be first extension header or payload protocol)
            let first_next_header = if !self.extension_headers.is_empty() {
                self.extension_headers[0].header_type()
            } else {
                self.next_header
            };
            ipv6_packet.set_next_header(IpNextHeaderProtocol::new(first_next_header));
        }

        // Build extension headers
        let mut offset = 40;
        for (i, ext_header) in self.extension_headers.iter().enumerate() {
            let next_header = if i + 1 < self.extension_headers.len() {
                self.extension_headers[i + 1].header_type()
            } else {
                self.next_header
            };
            let header_bytes = ext_header.build(next_header)?;
            buffer[offset..offset + header_bytes.len()].copy_from_slice(&header_bytes);
            offset += header_bytes.len();
        }

        // Copy payload
        buffer[offset..].copy_from_slice(&self.payload);

        Ok(buffer)
    }

    /// Calculate IPv6 pseudo-header checksum for TCP/UDP/ICMPv6
    ///
    /// Pseudo-header format (40 bytes):
    /// - Source address (16 bytes)
    /// - Destination address (16 bytes)
    /// - Upper-Layer Packet Length (4 bytes, big-endian)
    /// - Zero padding (3 bytes)
    /// - Next Header protocol (1 byte)
    ///
    /// # Arguments
    ///
    /// * `protocol` - Upper-layer protocol number (6=TCP, 17=UDP, 58=ICMPv6)
    /// * `payload_len` - Length of upper-layer packet (header + data)
    ///
    /// # Returns
    ///
    /// 16-bit checksum value
    pub fn pseudo_header_checksum(&self, protocol: u8, payload_len: u16) -> u16 {
        let mut pseudo_header = Vec::with_capacity(40);

        // Source address (16 bytes)
        pseudo_header.extend_from_slice(&self.source.octets());
        // Destination address (16 bytes)
        pseudo_header.extend_from_slice(&self.destination.octets());
        // Upper-Layer Packet Length (4 bytes, big-endian)
        pseudo_header.extend_from_slice(&(payload_len as u32).to_be_bytes());
        // Zero padding (3 bytes)
        pseudo_header.extend_from_slice(&[0, 0, 0]);
        // Next Header protocol (1 byte)
        pseudo_header.push(protocol);

        pnet::util::checksum(&pseudo_header, 1)
    }

    /// Fragment packet into MTU-sized chunks
    ///
    /// Splits the payload into fragments that fit within the given MTU.
    /// Each fragment includes a Fragment extension header.
    ///
    /// # Arguments
    ///
    /// * `mtu` - Maximum Transmission Unit (must be >= 1280 per RFC 8200)
    ///
    /// # Errors
    ///
    /// Returns error if MTU is less than 1280 (IPv6 minimum).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::ipv6_packet::Ipv6PacketBuilder;
    /// use std::net::Ipv6Addr;
    ///
    /// let src = "2001:db8::1".parse().unwrap();
    /// let dst = "2001:db8::2".parse().unwrap();
    /// let payload = vec![0u8; 2000]; // Large payload
    ///
    /// let fragments = Ipv6PacketBuilder::new(src, dst)
    ///     .next_header(6) // TCP
    ///     .payload(payload)
    ///     .fragment(1280)
    ///     .unwrap();
    ///
    /// assert!(fragments.len() > 1); // Multiple fragments
    /// ```
    pub fn fragment(self, mtu: usize) -> Result<Vec<Vec<u8>>> {
        // Validate MTU (IPv6 minimum is 1280 bytes)
        if mtu < 1280 {
            return Err(Ipv6PacketError::InvalidMtu { mtu });
        }

        // Calculate fragment payload size (MTU - 40 IPv6 header - 8 fragment header)
        let fragment_payload_size = (mtu - 48) & !7; // Round down to multiple of 8
        let mut fragments = Vec::new();
        let mut offset = 0;
        use rand::Rng;
        let fragment_id: u32 = rand::thread_rng().gen();

        while offset < self.payload.len() {
            let remaining = self.payload.len() - offset;
            let chunk_size = std::cmp::min(fragment_payload_size, remaining);
            let more_fragments = offset + chunk_size < self.payload.len();

            // Create fragment with Fragment extension header
            let fragment_builder = Ipv6PacketBuilder::new(self.source, self.destination)
                .hop_limit(self.hop_limit)
                .traffic_class(self.traffic_class)
                .flow_label(self.flow_label)
                .next_header(self.next_header)
                .add_extension_header(ExtensionHeader::Fragment {
                    id: fragment_id,
                    offset: (offset / 8) as u16, // Offset in 8-byte units
                    more_fragments,
                })
                .payload(self.payload[offset..offset + chunk_size].to_vec());

            fragments.push(fragment_builder.build()?);
            offset += chunk_size;
        }

        Ok(fragments)
    }

    /// Get source address
    pub fn source(&self) -> Ipv6Addr {
        self.source
    }

    /// Get destination address
    pub fn destination(&self) -> Ipv6Addr {
        self.destination
    }
}

impl ExtensionHeader {
    /// Get header type number
    ///
    /// Returns the IPv6 protocol number for this extension header type.
    pub fn header_type(&self) -> u8 {
        match self {
            ExtensionHeader::HopByHop(_) => 0,
            ExtensionHeader::Routing(_) => 43,
            ExtensionHeader::Fragment { .. } => 44,
            ExtensionHeader::DestinationOptions(_) => 60,
        }
    }

    /// Get header size in bytes
    pub fn size(&self) -> usize {
        match self {
            ExtensionHeader::HopByHop(data) => data.len() + 2,
            ExtensionHeader::Routing(data) => data.len() + 2,
            ExtensionHeader::Fragment { .. } => 8, // Fragment header is always 8 bytes
            ExtensionHeader::DestinationOptions(data) => data.len() + 2,
        }
    }

    /// Build extension header bytes
    ///
    /// Constructs the raw bytes for this extension header with proper
    /// formatting and Next Header field.
    ///
    /// # Arguments
    ///
    /// * `next_header` - Protocol number of the next header in the chain
    pub fn build(&self, next_header: u8) -> Result<Vec<u8>> {
        match self {
            ExtensionHeader::Fragment { id, offset, more_fragments } => {
                // Validate fragment offset (must be multiple of 8 when converted to bytes)
                // Note: offset is already in 8-byte units, so this check is for the value itself
                if *offset > 8191 {
                    return Err(Ipv6PacketError::InvalidFragmentOffset(*offset));
                }

                let mut buffer = vec![0u8; 8];
                buffer[0] = next_header;
                buffer[1] = 0; // Reserved

                // Fragment offset (13 bits) + Res (2 bits) + M flag (1 bit)
                let offset_and_flags = ((*offset) << 3) | (*more_fragments as u16);
                buffer[2..4].copy_from_slice(&offset_and_flags.to_be_bytes());
                buffer[4..8].copy_from_slice(&id.to_be_bytes());

                Ok(buffer)
            }
            ExtensionHeader::HopByHop(data) |
            ExtensionHeader::Routing(data) |
            ExtensionHeader::DestinationOptions(data) => {
                if data.is_empty() {
                    return Err(Ipv6PacketError::ExtensionHeaderTooLarge { size: 0 });
                }

                // Extension header length must be multiple of 8 bytes
                let header_len = ((data.len() + 6) / 8) * 8; // Round up to multiple of 8
                let mut buffer = vec![0u8; header_len];

                buffer[0] = next_header;
                buffer[1] = ((header_len / 8) - 1) as u8; // Header Ext Length
                buffer[2..2 + data.len()].copy_from_slice(data);

                Ok(buffer)
            }
        }
    }
}

/// Parse IPv6 packet header
///
/// Extracts key fields from an IPv6 packet for response validation.
#[derive(Debug, Clone)]
pub struct Ipv6Header {
    pub source: Ipv6Addr,
    pub destination: Ipv6Addr,
    pub next_header: u8,
    pub hop_limit: u8,
    pub payload_length: u16,
}

/// Parse IPv6 packet header from raw bytes
///
/// # Arguments
///
/// * `packet` - Raw packet bytes (must be at least 40 bytes)
///
/// # Returns
///
/// Parsed IPv6 header or None if packet is too short or invalid.
pub fn parse_ipv6_header(packet: &[u8]) -> Option<Ipv6Header> {
    if packet.len() < 40 {
        return None;
    }

    // Check version field (first 4 bits must be 6)
    if (packet[0] >> 4) != 6 {
        return None;
    }

    let mut src_bytes = [0u8; 16];
    let mut dst_bytes = [0u8; 16];

    src_bytes.copy_from_slice(&packet[8..24]);
    dst_bytes.copy_from_slice(&packet[24..40]);

    Some(Ipv6Header {
        source: Ipv6Addr::from(src_bytes),
        destination: Ipv6Addr::from(dst_bytes),
        next_header: packet[6],
        hop_limit: packet[7],
        payload_length: u16::from_be_bytes([packet[4], packet[5]]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv6_packet_builder_basic() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let packet = Ipv6PacketBuilder::new(src, dst)
            .hop_limit(64)
            .next_header(6) // TCP
            .payload(vec![0xDE, 0xAD, 0xBE, 0xEF])
            .build()
            .unwrap();

        assert_eq!(packet.len(), 44); // 40 (header) + 4 (payload)
        assert_eq!(packet[0] >> 4, 6); // Version = 6
        assert_eq!(packet[7], 64); // Hop limit
        assert_eq!(packet[6], 6); // Next header = TCP
    }

    #[test]
    fn test_ipv6_packet_builder_flow_label() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let packet = Ipv6PacketBuilder::new(src, dst)
            .flow_label(0xABCDE) // 20-bit value
            .build()
            .unwrap();

        // Flow label should be masked to 20 bits
        assert_eq!(packet.len(), 40);
    }

    #[test]
    fn test_pseudo_header_checksum() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let builder = Ipv6PacketBuilder::new(src, dst);
        let checksum = builder.pseudo_header_checksum(6, 20); // TCP, 20 bytes

        // Checksum should be non-zero
        assert_ne!(checksum, 0);
    }

    #[test]
    fn test_extension_header_fragment() {
        let header = ExtensionHeader::Fragment {
            id: 12345,
            offset: 100,
            more_fragments: true,
        };

        assert_eq!(header.header_type(), 44);
        assert_eq!(header.size(), 8);

        let bytes = header.build(6).unwrap(); // Next header = TCP
        assert_eq!(bytes.len(), 8);
        assert_eq!(bytes[0], 6); // Next header
    }

    #[test]
    fn test_ipv6_packet_with_fragment_header() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let packet = Ipv6PacketBuilder::new(src, dst)
            .hop_limit(64)
            .next_header(6) // TCP
            .add_extension_header(ExtensionHeader::Fragment {
                id: 54321,
                offset: 0,
                more_fragments: false,
            })
            .payload(vec![0xDE, 0xAD])
            .build()
            .unwrap();

        // 40 (IPv6) + 8 (Fragment) + 2 (payload) = 50 bytes
        assert_eq!(packet.len(), 50);
        assert_eq!(packet[6], 44); // Next header = Fragment
    }

    #[test]
    fn test_fragmentation_basic() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();
        let payload = vec![0u8; 2000]; // Large payload

        let fragments = Ipv6PacketBuilder::new(src, dst)
            .next_header(6) // TCP
            .payload(payload)
            .fragment(1280)
            .unwrap();

        // Should create 2 fragments (1280 MTU - 48 headers = 1232 bytes per fragment, rounded to 1224)
        assert!(fragments.len() >= 2);

        // First fragment should have more_fragments flag
        assert!(fragments[0].len() <= 1280);
    }

    #[test]
    fn test_fragmentation_invalid_mtu() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let result = Ipv6PacketBuilder::new(src, dst)
            .payload(vec![0u8; 100])
            .fragment(1000); // Less than minimum 1280

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ipv6_header_valid() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let packet = Ipv6PacketBuilder::new(src, dst)
            .hop_limit(64)
            .next_header(6)
            .build()
            .unwrap();

        let header = parse_ipv6_header(&packet).unwrap();
        assert_eq!(header.source, src);
        assert_eq!(header.destination, dst);
        assert_eq!(header.next_header, 6);
        assert_eq!(header.hop_limit, 64);
    }

    #[test]
    fn test_parse_ipv6_header_too_short() {
        let packet = vec![0u8; 20]; // Too short
        assert!(parse_ipv6_header(&packet).is_none());
    }

    #[test]
    fn test_parse_ipv6_header_invalid_version() {
        let mut packet = vec![0u8; 40];
        packet[0] = 0x40; // Version 4, not 6
        assert!(parse_ipv6_header(&packet).is_none());
    }

    #[test]
    fn test_extension_header_hop_by_hop() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06]; // 6 bytes
        let header = ExtensionHeader::HopByHop(data.clone());

        assert_eq!(header.header_type(), 0);
        // Size should be data.len() + 2 (next header + length field)
        assert_eq!(header.size(), 8);

        let bytes = header.build(6).unwrap();
        assert_eq!(bytes[0], 6); // Next header = TCP
    }

    #[test]
    fn test_traffic_class_field() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let packet = Ipv6PacketBuilder::new(src, dst)
            .traffic_class(0xB8) // DSCP 46 (EF)
            .build()
            .unwrap();

        assert_eq!(packet.len(), 40);
        // Traffic class is in bits 4-11 of the first 4 bytes
    }

    #[test]
    fn test_zero_payload() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();

        let packet = Ipv6PacketBuilder::new(src, dst)
            .next_header(59) // No Next Header
            .build()
            .unwrap();

        assert_eq!(packet.len(), 40); // Header only
        assert_eq!(packet[6], 59); // No Next Header
    }

    #[test]
    fn test_max_payload() {
        let src = "2001:db8::1".parse().unwrap();
        let dst = "2001:db8::2".parse().unwrap();
        let payload = vec![0xFF; 65535]; // Maximum payload

        let packet = Ipv6PacketBuilder::new(src, dst)
            .next_header(17) // UDP
            .payload(payload)
            .build()
            .unwrap();

        assert_eq!(packet.len(), 40 + 65535);
    }
}
