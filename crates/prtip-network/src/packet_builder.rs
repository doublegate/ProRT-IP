//! Raw packet construction for various scan types
//!
//! This module provides low-level packet building capabilities for TCP, UDP, and ICMP
//! protocols. It handles Ethernet framing, IPv4 headers, and transport layer headers
//! with proper checksum calculation.
//!
//! # Safety
//!
//! All packet construction is bounds-checked and does not use unsafe code.
//! The `pnet` crate provides safe abstractions over raw packet manipulation.
//!
//! # Example
//!
//! ```no_run
//! use prtip_network::packet_builder::{TcpPacketBuilder, TcpFlags};
//! use std::net::Ipv4Addr;
//!
//! let packet = TcpPacketBuilder::new()
//!     .source_ip(Ipv4Addr::new(10, 0, 0, 1))
//!     .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
//!     .source_port(12345)
//!     .dest_port(80)
//!     .flags(TcpFlags::SYN)
//!     .window(65535)
//!     .build()
//!     .expect("Failed to build packet");
//! ```

use pnet::packet::{
    ethernet::{EtherTypes, MutableEthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::{checksum as ipv4_checksum, MutableIpv4Packet},
    tcp::{ipv4_checksum as tcp_ipv4_checksum, ipv6_checksum as tcp_ipv6_checksum, MutableTcpPacket},
    udp::{ipv4_checksum as udp_ipv4_checksum, ipv6_checksum as udp_ipv6_checksum, MutableUdpPacket},
};
use pnet::util::MacAddr;
use std::net::{Ipv4Addr, Ipv6Addr};
use thiserror::Error;

/// Errors that can occur during packet construction
#[derive(Debug, Error)]
pub enum PacketBuilderError {
    #[error("Buffer too small for packet: need {needed} bytes, have {available}")]
    BufferTooSmall { needed: usize, available: usize },

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

// Allow PacketBuilderError to be converted to prtip_core::Error
impl From<PacketBuilderError> for prtip_core::Error {
    fn from(err: PacketBuilderError) -> Self {
        prtip_core::Error::Network(format!("Packet builder error: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, PacketBuilderError>;

/// TCP flags as a bitmask
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcpFlags(pub u8);

impl TcpFlags {
    pub const FIN: TcpFlags = TcpFlags(0b0000_0001);
    pub const SYN: TcpFlags = TcpFlags(0b0000_0010);
    pub const RST: TcpFlags = TcpFlags(0b0000_0100);
    pub const PSH: TcpFlags = TcpFlags(0b0000_1000);
    pub const ACK: TcpFlags = TcpFlags(0b0001_0000);
    pub const URG: TcpFlags = TcpFlags(0b0010_0000);
    pub const ECE: TcpFlags = TcpFlags(0b0100_0000);
    pub const CWR: TcpFlags = TcpFlags(0b1000_0000);

    /// Create flags with no bits set
    pub const fn empty() -> Self {
        TcpFlags(0)
    }

    /// Combine multiple flags
    pub fn combine(&self, other: TcpFlags) -> Self {
        TcpFlags(self.0 | other.0)
    }

    /// Check if a flag is set
    pub fn has(&self, flag: TcpFlags) -> bool {
        (self.0 & flag.0) != 0
    }
}

/// TCP options that can be included in TCP headers
#[derive(Debug, Clone)]
pub enum TcpOption {
    /// Maximum Segment Size (kind=2, len=4)
    Mss(u16),
    /// Window Scale (kind=3, len=3)
    WindowScale(u8),
    /// SACK Permitted (kind=4, len=2)
    SackPermitted,
    /// Timestamp (kind=8, len=10)
    Timestamp { tsval: u32, tsecr: u32 },
    /// No Operation (kind=1, len=1) - used for padding
    Nop,
    /// End of Option List (kind=0, len=1)
    Eol,
}

impl TcpOption {
    /// Get the total length of this option in bytes
    pub fn len(&self) -> usize {
        match self {
            TcpOption::Eol => 1,
            TcpOption::Nop => 1,
            TcpOption::Mss(_) => 4,
            TcpOption::WindowScale(_) => 3,
            TcpOption::SackPermitted => 2,
            TcpOption::Timestamp { .. } => 10,
        }
    }

    /// Check if option is empty (always false - all options have data)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Serialize this option to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            TcpOption::Eol => vec![0],
            TcpOption::Nop => vec![1],
            TcpOption::Mss(mss) => vec![2, 4, (*mss >> 8) as u8, *mss as u8],
            TcpOption::WindowScale(scale) => vec![3, 3, *scale],
            TcpOption::SackPermitted => vec![4, 2],
            TcpOption::Timestamp { tsval, tsecr } => {
                let mut bytes = vec![8, 10];
                bytes.extend_from_slice(&tsval.to_be_bytes());
                bytes.extend_from_slice(&tsecr.to_be_bytes());
                bytes
            }
        }
    }
}

/// Builder for TCP packets with full control over headers
#[derive(Debug, Clone)]
pub struct TcpPacketBuilder {
    // Ethernet layer
    src_mac: Option<MacAddr>,
    dst_mac: Option<MacAddr>,

    // IP layer
    src_ip: Option<Ipv4Addr>,
    dst_ip: Option<Ipv4Addr>,
    ttl: u8,
    ip_id: u16,

    // TCP layer
    src_port: Option<u16>,
    dst_port: Option<u16>,
    seq: u32,
    ack: u32,
    flags: TcpFlags,
    window: u16,
    urgent_ptr: u16,
    options: Vec<TcpOption>,

    // Payload
    payload: Vec<u8>,

    // Evasion
    bad_checksum: bool,
}

impl Default for TcpPacketBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpPacketBuilder {
    /// Create a new TCP packet builder with default values
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            src_mac: None,
            dst_mac: None,
            src_ip: None,
            dst_ip: None,
            ttl: 64, // Standard Linux default
            ip_id: rng.gen(),
            src_port: None,
            dst_port: None,
            seq: rng.gen(),
            ack: 0,
            flags: TcpFlags::empty(),
            window: 65535,
            urgent_ptr: 0,
            options: Vec::new(),
            payload: Vec::new(),
            bad_checksum: false,
        }
    }

    /// Set source MAC address
    pub fn source_mac(mut self, mac: MacAddr) -> Self {
        self.src_mac = Some(mac);
        self
    }

    /// Set destination MAC address
    pub fn dest_mac(mut self, mac: MacAddr) -> Self {
        self.dst_mac = Some(mac);
        self
    }

    /// Set source IP address
    pub fn source_ip(mut self, ip: Ipv4Addr) -> Self {
        self.src_ip = Some(ip);
        self
    }

    /// Set destination IP address
    pub fn dest_ip(mut self, ip: Ipv4Addr) -> Self {
        self.dst_ip = Some(ip);
        self
    }

    /// Set TTL (Time To Live)
    pub fn ttl(mut self, ttl: u8) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set IP identification field
    pub fn ip_id(mut self, id: u16) -> Self {
        self.ip_id = id;
        self
    }

    /// Set source port
    pub fn source_port(mut self, port: u16) -> Self {
        self.src_port = Some(port);
        self
    }

    /// Set destination port
    pub fn dest_port(mut self, port: u16) -> Self {
        self.dst_port = Some(port);
        self
    }

    /// Set sequence number
    pub fn sequence(mut self, seq: u32) -> Self {
        self.seq = seq;
        self
    }

    /// Set acknowledgment number
    pub fn acknowledgment(mut self, ack: u32) -> Self {
        self.ack = ack;
        self
    }

    /// Set TCP flags
    pub fn flags(mut self, flags: TcpFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set window size
    pub fn window(mut self, window: u16) -> Self {
        self.window = window;
        self
    }

    /// Set urgent pointer
    pub fn urgent_pointer(mut self, ptr: u16) -> Self {
        self.urgent_ptr = ptr;
        self
    }

    /// Add a TCP option
    pub fn add_option(mut self, option: TcpOption) -> Self {
        self.options.push(option);
        self
    }

    /// Set payload data
    pub fn payload(mut self, data: Vec<u8>) -> Self {
        self.payload = data;
        self
    }

    /// Enable bad checksum for testing/evasion (nmap --badsum)
    ///
    /// When enabled, sets TCP checksum to 0x0000 (invalid) instead of calculating
    /// the correct checksum. Used for testing firewall/IDS checksum validation.
    ///
    /// Default: false (calculate correct checksum)
    pub fn bad_checksum(mut self, enabled: bool) -> Self {
        self.bad_checksum = enabled;
        self
    }

    /// Calculate the total size needed for options (padded to 4-byte boundary)
    fn options_size(&self) -> usize {
        let raw_size: usize = self.options.iter().map(|opt| opt.len()).sum();
        // Round up to multiple of 4
        (raw_size + 3) & !3
    }

    /// Serialize options directly to buffer (zero-copy)
    ///
    /// Writes TCP options directly to the provided buffer without intermediate
    /// allocations. Returns the number of bytes written.
    ///
    /// # Performance
    ///
    /// This is a zero-allocation operation. All option data is written directly
    /// to the destination buffer in-place.
    fn serialize_options_to_buffer(&self, buffer: &mut [u8]) -> usize {
        let mut offset = 0;

        for option in &self.options {
            match option {
                TcpOption::Eol => {
                    buffer[offset] = 0;
                    offset += 1;
                }
                TcpOption::Nop => {
                    buffer[offset] = 1;
                    offset += 1;
                }
                TcpOption::Mss(mss) => {
                    buffer[offset] = 2; // Kind
                    buffer[offset + 1] = 4; // Length
                    buffer[offset + 2] = (*mss >> 8) as u8; // MSS high byte
                    buffer[offset + 3] = *mss as u8; // MSS low byte
                    offset += 4;
                }
                TcpOption::WindowScale(scale) => {
                    buffer[offset] = 3; // Kind
                    buffer[offset + 1] = 3; // Length
                    buffer[offset + 2] = *scale; // Scale value
                    offset += 3;
                }
                TcpOption::SackPermitted => {
                    buffer[offset] = 4; // Kind
                    buffer[offset + 1] = 2; // Length
                    offset += 2;
                }
                TcpOption::Timestamp { tsval, tsecr } => {
                    buffer[offset] = 8; // Kind
                    buffer[offset + 1] = 10; // Length
                    buffer[offset + 2..offset + 6].copy_from_slice(&tsval.to_be_bytes());
                    buffer[offset + 6..offset + 10].copy_from_slice(&tsecr.to_be_bytes());
                    offset += 10;
                }
            }
        }

        // Pad to 4-byte boundary with NOPs
        while offset % 4 != 0 {
            buffer[offset] = 1; // NOP
            offset += 1;
        }

        offset
    }

    /// Serialize options to bytes with padding (legacy, kept for backwards compatibility)
    #[deprecated(
        since = "0.3.8",
        note = "Use serialize_options_to_buffer for zero-copy performance"
    )]
    #[allow(dead_code)] // Kept for API compatibility
    fn serialize_options(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for option in &self.options {
            bytes.extend_from_slice(&option.to_bytes());
        }

        // Pad to 4-byte boundary with NOPs
        while bytes.len() % 4 != 0 {
            bytes.push(1); // NOP
        }

        bytes
    }

    /// Build packet using zero-copy buffer pool (high-performance)
    ///
    /// This is the high-performance packet building method that uses thread-local
    /// buffer pools to eliminate heap allocations. Ideal for high packet rate
    /// scenarios (>100K pps).
    ///
    /// # Performance
    ///
    /// - **Zero allocations**: Uses pre-allocated buffer pool
    /// - **Zero contention**: Thread-local storage
    /// - **Sub-microsecond**: Typical packet crafting <1µs
    ///
    /// # Returns
    ///
    /// Returns a byte slice borrowed from the thread-local buffer pool.
    /// The slice is valid until the next call to `with_buffer()` or `reset()`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::{TcpPacketBuilder, TcpFlags, packet_buffer::with_buffer};
    /// use std::net::Ipv4Addr;
    ///
    /// with_buffer(|pool| {
    ///     let packet = TcpPacketBuilder::new()
    ///         .source_ip(Ipv4Addr::new(10, 0, 0, 1))
    ///         .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
    ///         .source_port(12345)
    ///         .dest_port(80)
    ///         .flags(TcpFlags::SYN)
    ///         .build_with_buffer(pool)
    ///         .expect("Failed to build packet");
    ///
    ///     // Use packet slice here (valid within this closure)
    ///     assert!(packet.len() >= 40);
    /// });
    /// ```
    #[allow(clippy::needless_lifetimes)] // Lifetime needed for clarity
    pub fn build_with_buffer<'a>(
        self,
        buffer_pool: &'a mut crate::packet_buffer::PacketBuffer,
    ) -> Result<&'a [u8]> {
        // Validate required fields
        let src_ip = self
            .src_ip
            .ok_or_else(|| PacketBuilderError::MissingField("source_ip".to_string()))?;
        let dst_ip = self
            .dst_ip
            .ok_or_else(|| PacketBuilderError::MissingField("dest_ip".to_string()))?;
        let src_port = self
            .src_port
            .ok_or_else(|| PacketBuilderError::MissingField("source_port".to_string()))?;
        let dst_port = self
            .dst_port
            .ok_or_else(|| PacketBuilderError::MissingField("dest_port".to_string()))?;

        // Calculate sizes
        let options_size = self.options_size();
        let tcp_header_size = 20 + options_size;
        let tcp_total_size = tcp_header_size + self.payload.len();
        let ip_total_size = 20 + tcp_total_size;

        // Decide whether to include Ethernet header
        let (total_size, has_ethernet) = if self.src_mac.is_some() && self.dst_mac.is_some() {
            (14 + ip_total_size, true)
        } else {
            (ip_total_size, false)
        };

        // Get buffer from pool (zero-copy)
        let remaining = buffer_pool.remaining(); // Capture before borrow
        let buffer = buffer_pool
            .get_mut(total_size)
            .ok_or(PacketBuilderError::BufferTooSmall {
                needed: total_size,
                available: remaining,
            })?;

        let buffer_len = buffer.len(); // Capture before mutable borrow
        let mut offset = 0;

        // Build Ethernet header if MAC addresses provided
        if has_ethernet {
            let src_mac = self.src_mac.unwrap();
            let dst_mac = self.dst_mac.unwrap();

            let mut eth_packet = MutableEthernetPacket::new(&mut buffer[offset..offset + 14])
                .ok_or(PacketBuilderError::BufferTooSmall {
                    needed: 14,
                    available: buffer_len,
                })?;

            eth_packet.set_destination(dst_mac);
            eth_packet.set_source(src_mac);
            eth_packet.set_ethertype(EtherTypes::Ipv4);

            offset += 14;
        }

        // Build IPv4 header
        {
            let mut ip_packet = MutableIpv4Packet::new(&mut buffer[offset..offset + 20]).ok_or(
                PacketBuilderError::BufferTooSmall {
                    needed: 20,
                    available: buffer_len - offset,
                },
            )?;

            ip_packet.set_version(4);
            ip_packet.set_header_length(5); // 5 * 4 = 20 bytes
            ip_packet.set_dscp(0);
            ip_packet.set_ecn(0);
            ip_packet.set_total_length(ip_total_size as u16);
            ip_packet.set_identification(self.ip_id);
            ip_packet.set_flags(2); // Don't Fragment
            ip_packet.set_fragment_offset(0);
            ip_packet.set_ttl(self.ttl);
            ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
            ip_packet.set_source(src_ip);
            ip_packet.set_destination(dst_ip);

            // Calculate and set checksum
            let checksum = ipv4_checksum(&ip_packet.to_immutable());
            ip_packet.set_checksum(checksum);

            offset += 20;
        }

        // Build TCP header
        {
            let tcp_size = tcp_header_size + self.payload.len();
            let mut tcp_packet = MutableTcpPacket::new(&mut buffer[offset..offset + tcp_size])
                .ok_or(PacketBuilderError::BufferTooSmall {
                    needed: tcp_size,
                    available: buffer_len - offset,
                })?;

            tcp_packet.set_source(src_port);
            tcp_packet.set_destination(dst_port);
            tcp_packet.set_sequence(self.seq);
            tcp_packet.set_acknowledgement(self.ack);
            tcp_packet.set_data_offset((tcp_header_size / 4) as u8);
            tcp_packet.set_reserved(0);
            tcp_packet.set_flags(self.flags.0);
            tcp_packet.set_window(self.window);
            tcp_packet.set_urgent_ptr(self.urgent_ptr);

            // Set options if any (zero-copy)
            if !self.options.is_empty() {
                let opts_slice = tcp_packet.get_options_raw_mut();
                let _ = self.serialize_options_to_buffer(opts_slice);
            }

            // Set payload if any
            if !self.payload.is_empty() {
                tcp_packet.set_payload(&self.payload);
            }

            // Calculate and set checksum (or use bad checksum for testing)
            if self.bad_checksum {
                tcp_packet.set_checksum(0x0000); // Invalid checksum for testing
            } else {
                let checksum = tcp_ipv4_checksum(&tcp_packet.to_immutable(), &src_ip, &dst_ip);
                tcp_packet.set_checksum(checksum);
            }
        }

        Ok(&buffer[..total_size])
    }

    /// Build the complete packet (Ethernet + IPv4 + TCP)
    ///
    /// This is the traditional packet building method that allocates a new Vec<u8>
    /// for each packet. For high packet rates (>100K pps), consider using
    /// `build_with_buffer()` instead for zero-allocation performance.
    pub fn build(self) -> Result<Vec<u8>> {
        // Validate required fields
        let src_ip = self
            .src_ip
            .ok_or_else(|| PacketBuilderError::MissingField("source_ip".to_string()))?;
        let dst_ip = self
            .dst_ip
            .ok_or_else(|| PacketBuilderError::MissingField("dest_ip".to_string()))?;
        let src_port = self
            .src_port
            .ok_or_else(|| PacketBuilderError::MissingField("source_port".to_string()))?;
        let dst_port = self
            .dst_port
            .ok_or_else(|| PacketBuilderError::MissingField("dest_port".to_string()))?;

        // Calculate sizes
        let options_size = self.options_size();
        let tcp_header_size = 20 + options_size;
        let tcp_total_size = tcp_header_size + self.payload.len();
        let ip_total_size = 20 + tcp_total_size;

        // Decide whether to include Ethernet header
        let (total_size, has_ethernet) = if self.src_mac.is_some() && self.dst_mac.is_some() {
            (14 + ip_total_size, true)
        } else {
            (ip_total_size, false)
        };

        let mut buffer = vec![0u8; total_size];
        let mut offset = 0;

        // Build Ethernet header if MAC addresses provided
        if has_ethernet {
            let src_mac = self.src_mac.unwrap();
            let dst_mac = self.dst_mac.unwrap();
            let buffer_len = buffer.len(); // Capture before mutable borrow

            let mut eth_packet = MutableEthernetPacket::new(&mut buffer[offset..offset + 14])
                .ok_or(PacketBuilderError::BufferTooSmall {
                    needed: 14,
                    available: buffer_len,
                })?;

            eth_packet.set_destination(dst_mac);
            eth_packet.set_source(src_mac);
            eth_packet.set_ethertype(EtherTypes::Ipv4);

            offset += 14;
        }

        // Build IPv4 header
        {
            let buffer_len = buffer.len(); // Capture before mutable borrow
            let mut ip_packet = MutableIpv4Packet::new(&mut buffer[offset..offset + 20]).ok_or(
                PacketBuilderError::BufferTooSmall {
                    needed: 20,
                    available: buffer_len - offset,
                },
            )?;

            ip_packet.set_version(4);
            ip_packet.set_header_length(5); // 5 * 4 = 20 bytes
            ip_packet.set_dscp(0);
            ip_packet.set_ecn(0);
            ip_packet.set_total_length(ip_total_size as u16);
            ip_packet.set_identification(self.ip_id);
            ip_packet.set_flags(2); // Don't Fragment
            ip_packet.set_fragment_offset(0);
            ip_packet.set_ttl(self.ttl);
            ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
            ip_packet.set_source(src_ip);
            ip_packet.set_destination(dst_ip);

            // Calculate and set checksum
            let checksum = ipv4_checksum(&ip_packet.to_immutable());
            ip_packet.set_checksum(checksum);

            offset += 20;
        }

        // Build TCP header
        {
            let tcp_size = tcp_header_size + self.payload.len();
            let buffer_len = buffer.len(); // Capture before mutable borrow
            let mut tcp_packet = MutableTcpPacket::new(&mut buffer[offset..offset + tcp_size])
                .ok_or(PacketBuilderError::BufferTooSmall {
                    needed: tcp_size,
                    available: buffer_len - offset,
                })?;

            tcp_packet.set_source(src_port);
            tcp_packet.set_destination(dst_port);
            tcp_packet.set_sequence(self.seq);
            tcp_packet.set_acknowledgement(self.ack);
            tcp_packet.set_data_offset((tcp_header_size / 4) as u8);
            tcp_packet.set_reserved(0);
            tcp_packet.set_flags(self.flags.0);
            tcp_packet.set_window(self.window);
            tcp_packet.set_urgent_ptr(self.urgent_ptr);

            // Set options if any (zero-copy)
            if !self.options.is_empty() {
                let opts_slice = tcp_packet.get_options_raw_mut();
                let _ = self.serialize_options_to_buffer(opts_slice);
            }

            // Set payload if any
            if !self.payload.is_empty() {
                tcp_packet.set_payload(&self.payload);
            }

            // Calculate and set checksum (or use bad checksum for testing)
            if self.bad_checksum {
                tcp_packet.set_checksum(0x0000); // Invalid checksum for testing
            } else {
                let checksum = tcp_ipv4_checksum(&tcp_packet.to_immutable(), &src_ip, &dst_ip);
                tcp_packet.set_checksum(checksum);
            }
        }

        Ok(buffer)
    }

    /// Build just the IP+TCP packet (no Ethernet header)
    pub fn build_ip_packet(self) -> Result<Vec<u8>> {
        // Ensure no MAC addresses are set
        let builder = Self {
            src_mac: None,
            dst_mac: None,
            ..self
        };
        builder.build()
    }

    /// Build IPv6 TCP packet (SYN, FIN, ACK, etc.)
    ///
    /// Creates a complete IPv6+TCP packet with the specified source and destination
    /// IPv6 addresses. The TCP header is built with all configured options.
    ///
    /// # Arguments
    ///
    /// * `src_ipv6` - Source IPv6 address
    /// * `dst_ipv6` - Destination IPv6 address
    ///
    /// # Returns
    ///
    /// Complete IPv6+TCP packet bytes (40 bytes IPv6 header + TCP header + payload)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::packet_builder::{TcpPacketBuilder, TcpFlags};
    /// use std::net::Ipv6Addr;
    ///
    /// let src = "2001:db8::1".parse().unwrap();
    /// let dst = "2001:db8::2".parse().unwrap();
    ///
    /// let packet = TcpPacketBuilder::new()
    ///     .source_port(12345)
    ///     .dest_port(80)
    ///     .flags(TcpFlags::SYN)
    ///     .build_ipv6_packet(src, dst)
    ///     .unwrap();
    /// ```
    pub fn build_ipv6_packet(
        self,
        src_ipv6: Ipv6Addr,
        dst_ipv6: Ipv6Addr,
    ) -> Result<Vec<u8>> {
        // Validate required fields
        let src_port = self
            .src_port
            .ok_or_else(|| PacketBuilderError::MissingField("source_port".to_string()))?;
        let dst_port = self
            .dst_port
            .ok_or_else(|| PacketBuilderError::MissingField("dest_port".to_string()))?;

        // Calculate sizes
        let options_size = self.options_size();
        let tcp_header_size = 20 + options_size;
        let tcp_total_size = tcp_header_size + self.payload.len();

        // Allocate buffer for TCP packet only (no IPv6 header, we'll add that separately)
        let mut tcp_buffer = vec![0u8; tcp_total_size];

        // Build TCP header
        {
            let buffer_len = tcp_buffer.len(); // Capture before mutable borrow
            let mut tcp_packet = MutableTcpPacket::new(&mut tcp_buffer)
                .ok_or(PacketBuilderError::BufferTooSmall {
                    needed: tcp_total_size,
                    available: buffer_len,
                })?;

            tcp_packet.set_source(src_port);
            tcp_packet.set_destination(dst_port);
            tcp_packet.set_sequence(self.seq);
            tcp_packet.set_acknowledgement(self.ack);
            tcp_packet.set_data_offset((tcp_header_size / 4) as u8);
            tcp_packet.set_reserved(0);
            tcp_packet.set_flags(self.flags.0);
            tcp_packet.set_window(self.window);
            tcp_packet.set_urgent_ptr(self.urgent_ptr);

            // Set options if any
            if !self.options.is_empty() {
                let opts_slice = tcp_packet.get_options_raw_mut();
                let _ = self.serialize_options_to_buffer(opts_slice);
            }

            // Set payload if any
            if !self.payload.is_empty() {
                tcp_packet.set_payload(&self.payload);
            }

            // Calculate and set checksum with IPv6 pseudo-header
            if self.bad_checksum {
                tcp_packet.set_checksum(0x0000); // Invalid checksum for testing
            } else {
                let checksum = tcp_ipv6_checksum(&tcp_packet.to_immutable(), &src_ipv6, &dst_ipv6);
                tcp_packet.set_checksum(checksum);
            }
        }

        // Build IPv6 packet with TCP payload
        use crate::ipv6_packet::Ipv6PacketBuilder;
        let ipv6_packet = Ipv6PacketBuilder::new(src_ipv6, dst_ipv6)
            .hop_limit(self.ttl)
            .next_header(6) // TCP
            .payload(tcp_buffer)
            .build()
            .map_err(|e| PacketBuilderError::InvalidParameter(format!("IPv6 build failed: {}", e)))?;

        Ok(ipv6_packet)
    }

    /// Build just the IP+TCP packet (no Ethernet header) using zero-copy buffer
    ///
    /// This is the zero-copy equivalent of `build_ip_packet()`. Instead of
    /// allocating a new Vec<u8>, it writes directly to the provided buffer pool.
    ///
    /// # Arguments
    ///
    /// * `buffer_pool` - Mutable reference to thread-local buffer pool
    ///
    /// # Returns
    ///
    /// A byte slice reference (`&[u8]`) valid for the lifetime of the closure.
    /// The slice becomes invalid after `buffer_pool.reset()` is called.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::{TcpPacketBuilder, TcpFlags, packet_buffer::with_buffer};
    /// use std::net::Ipv4Addr;
    ///
    /// with_buffer(|pool| {
    ///     let packet = TcpPacketBuilder::new()
    ///         .source_ip(Ipv4Addr::new(10, 0, 0, 1))
    ///         .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
    ///         .source_port(12345)
    ///         .dest_port(80)
    ///         .flags(TcpFlags::SYN)
    ///         .build_ip_packet_with_buffer(pool)
    ///         .expect("Failed to build packet");
    ///
    ///     // Use packet slice here (e.g., send via raw socket)
    ///     println!("Packet size: {} bytes", packet.len());
    ///
    ///     pool.reset();
    ///     Ok::<(), Box<dyn std::error::Error>>(())
    /// }).unwrap();
    /// ```
    #[allow(clippy::needless_lifetimes)]
    pub fn build_ip_packet_with_buffer<'a>(
        self,
        buffer_pool: &'a mut crate::packet_buffer::PacketBuffer,
    ) -> Result<&'a [u8]> {
        // Ensure no MAC addresses are set (IP packet only)
        let builder = Self {
            src_mac: None,
            dst_mac: None,
            ..self
        };
        builder.build_with_buffer(buffer_pool)
    }
}

/// Builder for UDP packets
#[derive(Debug, Clone)]
pub struct UdpPacketBuilder {
    // Ethernet layer
    src_mac: Option<MacAddr>,
    dst_mac: Option<MacAddr>,

    // IP layer
    src_ip: Option<Ipv4Addr>,
    dst_ip: Option<Ipv4Addr>,
    ttl: u8,
    ip_id: u16,

    // UDP layer
    src_port: Option<u16>,
    dst_port: Option<u16>,

    // Payload
    payload: Vec<u8>,

    // Evasion
    bad_checksum: bool,
}

impl Default for UdpPacketBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl UdpPacketBuilder {
    /// Create a new UDP packet builder with default values
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            src_mac: None,
            dst_mac: None,
            src_ip: None,
            dst_ip: None,
            ttl: 64,
            ip_id: rng.gen(),
            src_port: None,
            dst_port: None,
            payload: Vec::new(),
            bad_checksum: false,
        }
    }

    /// Set source MAC address
    pub fn source_mac(mut self, mac: MacAddr) -> Self {
        self.src_mac = Some(mac);
        self
    }

    /// Set destination MAC address
    pub fn dest_mac(mut self, mac: MacAddr) -> Self {
        self.dst_mac = Some(mac);
        self
    }

    /// Set source IP address
    pub fn source_ip(mut self, ip: Ipv4Addr) -> Self {
        self.src_ip = Some(ip);
        self
    }

    /// Set destination IP address
    pub fn dest_ip(mut self, ip: Ipv4Addr) -> Self {
        self.dst_ip = Some(ip);
        self
    }

    /// Set TTL (Time To Live)
    pub fn ttl(mut self, ttl: u8) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set IP identification field
    pub fn ip_id(mut self, id: u16) -> Self {
        self.ip_id = id;
        self
    }

    /// Set source port
    pub fn source_port(mut self, port: u16) -> Self {
        self.src_port = Some(port);
        self
    }

    /// Set destination port
    pub fn dest_port(mut self, port: u16) -> Self {
        self.dst_port = Some(port);
        self
    }

    /// Set payload data
    pub fn payload(mut self, data: Vec<u8>) -> Self {
        self.payload = data;
        self
    }

    /// Enable bad checksum for testing/evasion (nmap --badsum)
    ///
    /// When enabled, sets UDP checksum to 0x0000 (invalid) instead of calculating
    /// the correct checksum. Used for testing firewall/IDS checksum validation.
    ///
    /// Default: false (calculate correct checksum)
    pub fn bad_checksum(mut self, enabled: bool) -> Self {
        self.bad_checksum = enabled;
        self
    }

    /// Build packet using zero-copy buffer pool (high-performance)
    ///
    /// This is the high-performance packet building method that uses thread-local
    /// buffer pools to eliminate heap allocations. Ideal for high packet rate
    /// scenarios (>100K pps).
    ///
    /// # Performance
    ///
    /// - **Zero allocations**: Uses pre-allocated buffer pool
    /// - **Zero contention**: Thread-local storage
    /// - **Sub-microsecond**: Typical packet crafting <1µs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::{UdpPacketBuilder, packet_buffer::with_buffer};
    /// use std::net::Ipv4Addr;
    ///
    /// with_buffer(|pool| {
    ///     let packet = UdpPacketBuilder::new()
    ///         .source_ip(Ipv4Addr::new(10, 0, 0, 1))
    ///         .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
    ///         .source_port(12345)
    ///         .dest_port(53)
    ///         .build_with_buffer(pool)
    ///         .expect("Failed to build packet");
    ///
    ///     // Use packet slice here (valid within this closure)
    ///     assert_eq!(packet.len(), 28);
    /// });
    /// ```
    #[allow(clippy::needless_lifetimes)] // Lifetime needed for clarity
    pub fn build_with_buffer<'a>(
        self,
        buffer_pool: &'a mut crate::packet_buffer::PacketBuffer,
    ) -> Result<&'a [u8]> {
        // Validate required fields
        let src_ip = self
            .src_ip
            .ok_or_else(|| PacketBuilderError::MissingField("source_ip".to_string()))?;
        let dst_ip = self
            .dst_ip
            .ok_or_else(|| PacketBuilderError::MissingField("dest_ip".to_string()))?;
        let src_port = self
            .src_port
            .ok_or_else(|| PacketBuilderError::MissingField("source_port".to_string()))?;
        let dst_port = self
            .dst_port
            .ok_or_else(|| PacketBuilderError::MissingField("dest_port".to_string()))?;

        // Calculate sizes
        let udp_total_size = 8 + self.payload.len();
        let ip_total_size = 20 + udp_total_size;

        // Decide whether to include Ethernet header
        let (total_size, has_ethernet) = if self.src_mac.is_some() && self.dst_mac.is_some() {
            (14 + ip_total_size, true)
        } else {
            (ip_total_size, false)
        };

        // Get buffer from pool (zero-copy)
        let remaining = buffer_pool.remaining(); // Capture before borrow
        let buffer = buffer_pool
            .get_mut(total_size)
            .ok_or(PacketBuilderError::BufferTooSmall {
                needed: total_size,
                available: remaining,
            })?;

        let buffer_len = buffer.len(); // Capture before mutable borrow
        let mut offset = 0;

        // Build Ethernet header if MAC addresses provided
        if has_ethernet {
            let src_mac = self.src_mac.unwrap();
            let dst_mac = self.dst_mac.unwrap();

            let mut eth_packet = MutableEthernetPacket::new(&mut buffer[offset..offset + 14])
                .ok_or(PacketBuilderError::BufferTooSmall {
                    needed: 14,
                    available: buffer_len,
                })?;

            eth_packet.set_destination(dst_mac);
            eth_packet.set_source(src_mac);
            eth_packet.set_ethertype(EtherTypes::Ipv4);

            offset += 14;
        }

        // Build IPv4 header
        {
            let mut ip_packet = MutableIpv4Packet::new(&mut buffer[offset..offset + 20]).ok_or(
                PacketBuilderError::BufferTooSmall {
                    needed: 20,
                    available: buffer_len - offset,
                },
            )?;

            ip_packet.set_version(4);
            ip_packet.set_header_length(5);
            ip_packet.set_dscp(0);
            ip_packet.set_ecn(0);
            ip_packet.set_total_length(ip_total_size as u16);
            ip_packet.set_identification(self.ip_id);
            ip_packet.set_flags(2); // Don't Fragment
            ip_packet.set_fragment_offset(0);
            ip_packet.set_ttl(self.ttl);
            ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Udp);
            ip_packet.set_source(src_ip);
            ip_packet.set_destination(dst_ip);

            // Calculate and set checksum
            let checksum = ipv4_checksum(&ip_packet.to_immutable());
            ip_packet.set_checksum(checksum);

            offset += 20;
        }

        // Build UDP header
        {
            let udp_size = 8 + self.payload.len();
            let mut udp_packet = MutableUdpPacket::new(&mut buffer[offset..offset + udp_size])
                .ok_or(PacketBuilderError::BufferTooSmall {
                    needed: udp_size,
                    available: buffer_len - offset,
                })?;

            udp_packet.set_source(src_port);
            udp_packet.set_destination(dst_port);
            udp_packet.set_length(udp_size as u16);

            // Set payload if any
            if !self.payload.is_empty() {
                udp_packet.set_payload(&self.payload);
            }

            // Calculate and set checksum (or use bad checksum for testing)
            if self.bad_checksum {
                udp_packet.set_checksum(0x0000); // Invalid checksum for testing
            } else {
                let checksum = udp_ipv4_checksum(&udp_packet.to_immutable(), &src_ip, &dst_ip);
                udp_packet.set_checksum(checksum);
            }
        }

        Ok(&buffer[..total_size])
    }

    /// Build the complete packet (Ethernet + IPv4 + UDP)
    ///
    /// This is the traditional packet building method that allocates a new Vec<u8>
    /// for each packet. For high packet rates (>100K pps), consider using
    /// `build_with_buffer()` instead for zero-allocation performance.
    pub fn build(self) -> Result<Vec<u8>> {
        // Validate required fields
        let src_ip = self
            .src_ip
            .ok_or_else(|| PacketBuilderError::MissingField("source_ip".to_string()))?;
        let dst_ip = self
            .dst_ip
            .ok_or_else(|| PacketBuilderError::MissingField("dest_ip".to_string()))?;
        let src_port = self
            .src_port
            .ok_or_else(|| PacketBuilderError::MissingField("source_port".to_string()))?;
        let dst_port = self
            .dst_port
            .ok_or_else(|| PacketBuilderError::MissingField("dest_port".to_string()))?;

        // Calculate sizes
        let udp_total_size = 8 + self.payload.len();
        let ip_total_size = 20 + udp_total_size;

        // Decide whether to include Ethernet header
        let (total_size, has_ethernet) = if self.src_mac.is_some() && self.dst_mac.is_some() {
            (14 + ip_total_size, true)
        } else {
            (ip_total_size, false)
        };

        let mut buffer = vec![0u8; total_size];
        let mut offset = 0;

        // Build Ethernet header if MAC addresses provided
        if has_ethernet {
            let src_mac = self.src_mac.unwrap();
            let dst_mac = self.dst_mac.unwrap();
            let buffer_len = buffer.len(); // Capture before mutable borrow

            let mut eth_packet = MutableEthernetPacket::new(&mut buffer[offset..offset + 14])
                .ok_or(PacketBuilderError::BufferTooSmall {
                    needed: 14,
                    available: buffer_len,
                })?;

            eth_packet.set_destination(dst_mac);
            eth_packet.set_source(src_mac);
            eth_packet.set_ethertype(EtherTypes::Ipv4);

            offset += 14;
        }

        // Build IPv4 header
        {
            let buffer_len = buffer.len(); // Capture before mutable borrow
            let mut ip_packet = MutableIpv4Packet::new(&mut buffer[offset..offset + 20]).ok_or(
                PacketBuilderError::BufferTooSmall {
                    needed: 20,
                    available: buffer_len - offset,
                },
            )?;

            ip_packet.set_version(4);
            ip_packet.set_header_length(5);
            ip_packet.set_dscp(0);
            ip_packet.set_ecn(0);
            ip_packet.set_total_length(ip_total_size as u16);
            ip_packet.set_identification(self.ip_id);
            ip_packet.set_flags(2); // Don't Fragment
            ip_packet.set_fragment_offset(0);
            ip_packet.set_ttl(self.ttl);
            ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Udp);
            ip_packet.set_source(src_ip);
            ip_packet.set_destination(dst_ip);

            // Calculate and set checksum
            let checksum = ipv4_checksum(&ip_packet.to_immutable());
            ip_packet.set_checksum(checksum);

            offset += 20;
        }

        // Build UDP header
        {
            let udp_size = 8 + self.payload.len();
            let buffer_len = buffer.len(); // Capture before mutable borrow
            let mut udp_packet = MutableUdpPacket::new(&mut buffer[offset..offset + udp_size])
                .ok_or(PacketBuilderError::BufferTooSmall {
                    needed: udp_size,
                    available: buffer_len - offset,
                })?;

            udp_packet.set_source(src_port);
            udp_packet.set_destination(dst_port);
            udp_packet.set_length(udp_size as u16);

            // Set payload if any
            if !self.payload.is_empty() {
                udp_packet.set_payload(&self.payload);
            }

            // Calculate and set checksum (or use bad checksum for testing)
            if self.bad_checksum {
                udp_packet.set_checksum(0x0000); // Invalid checksum for testing
            } else {
                let checksum = udp_ipv4_checksum(&udp_packet.to_immutable(), &src_ip, &dst_ip);
                udp_packet.set_checksum(checksum);
            }
        }

        Ok(buffer)
    }

    /// Build just the IP+UDP packet (no Ethernet header)
    pub fn build_ip_packet(self) -> Result<Vec<u8>> {
        // Ensure no MAC addresses are set
        let builder = Self {
            src_mac: None,
            dst_mac: None,
            ..self
        };
        builder.build()
    }

    /// Build IPv6 UDP packet
    ///
    /// Creates a complete IPv6+UDP packet with the specified source and destination
    /// IPv6 addresses. The UDP header is built with proper checksum.
    ///
    /// # Arguments
    ///
    /// * `src_ipv6` - Source IPv6 address
    /// * `dst_ipv6` - Destination IPv6 address
    ///
    /// # Returns
    ///
    /// Complete IPv6+UDP packet bytes (40 bytes IPv6 header + 8 bytes UDP header + payload)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::packet_builder::UdpPacketBuilder;
    /// use std::net::Ipv6Addr;
    ///
    /// let src = "2001:db8::1".parse().unwrap();
    /// let dst = "2001:db8::2".parse().unwrap();
    ///
    /// let packet = UdpPacketBuilder::new()
    ///     .source_port(12345)
    ///     .dest_port(53)
    ///     .payload(b"DNS query".to_vec())
    ///     .build_ipv6_packet(src, dst)
    ///     .unwrap();
    /// ```
    pub fn build_ipv6_packet(
        self,
        src_ipv6: Ipv6Addr,
        dst_ipv6: Ipv6Addr,
    ) -> Result<Vec<u8>> {
        // Validate required fields
        let src_port = self
            .src_port
            .ok_or_else(|| PacketBuilderError::MissingField("source_port".to_string()))?;
        let dst_port = self
            .dst_port
            .ok_or_else(|| PacketBuilderError::MissingField("dest_port".to_string()))?;

        // Calculate sizes
        let udp_total_size = 8 + self.payload.len();

        // Allocate buffer for UDP packet only
        let mut udp_buffer = vec![0u8; udp_total_size];

        // Build UDP header
        {
            let buffer_len = udp_buffer.len(); // Capture before mutable borrow
            let mut udp_packet = MutableUdpPacket::new(&mut udp_buffer)
                .ok_or(PacketBuilderError::BufferTooSmall {
                    needed: udp_total_size,
                    available: buffer_len,
                })?;

            udp_packet.set_source(src_port);
            udp_packet.set_destination(dst_port);
            udp_packet.set_length(udp_total_size as u16);

            // Set payload if any
            if !self.payload.is_empty() {
                udp_packet.set_payload(&self.payload);
            }

            // Calculate and set checksum with IPv6 pseudo-header
            if self.bad_checksum {
                udp_packet.set_checksum(0x0000); // Invalid checksum for testing
            } else {
                let checksum = udp_ipv6_checksum(&udp_packet.to_immutable(), &src_ipv6, &dst_ipv6);
                udp_packet.set_checksum(checksum);
            }
        }

        // Build IPv6 packet with UDP payload
        use crate::ipv6_packet::Ipv6PacketBuilder;
        let ipv6_packet = Ipv6PacketBuilder::new(src_ipv6, dst_ipv6)
            .hop_limit(self.ttl)
            .next_header(17) // UDP
            .payload(udp_buffer)
            .build()
            .map_err(|e| PacketBuilderError::InvalidParameter(format!("IPv6 build failed: {}", e)))?;

        Ok(ipv6_packet)
    }

    /// Build just the IP+UDP packet (no Ethernet header) using zero-copy buffer
    ///
    /// This is the zero-copy equivalent of `build_ip_packet()`. Instead of
    /// allocating a new Vec<u8>, it writes directly to the provided buffer pool.
    ///
    /// # Arguments
    ///
    /// * `buffer_pool` - Mutable reference to thread-local buffer pool
    ///
    /// # Returns
    ///
    /// A byte slice reference (`&[u8]`) valid for the lifetime of the closure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::{UdpPacketBuilder, packet_buffer::with_buffer};
    /// use std::net::Ipv4Addr;
    ///
    /// with_buffer(|pool| {
    ///     let packet = UdpPacketBuilder::new()
    ///         .source_ip(Ipv4Addr::new(10, 0, 0, 1))
    ///         .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
    ///         .source_port(12345)
    ///         .dest_port(53)
    ///         .build_ip_packet_with_buffer(pool)
    ///         .expect("Failed to build packet");
    ///
    ///     // Use packet slice here (e.g., send via raw socket)
    ///     println!("Packet size: {} bytes", packet.len());
    ///
    ///     pool.reset();
    ///     Ok::<(), Box<dyn std::error::Error>>(())
    /// }).unwrap();
    /// ```
    #[allow(clippy::needless_lifetimes)]
    pub fn build_ip_packet_with_buffer<'a>(
        self,
        buffer_pool: &'a mut crate::packet_buffer::PacketBuffer,
    ) -> Result<&'a [u8]> {
        // Ensure no MAC addresses are set (IP packet only)
        let builder = Self {
            src_mac: None,
            dst_mac: None,
            ..self
        };
        builder.build_with_buffer(buffer_pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_flags() {
        let syn = TcpFlags::SYN;
        assert_eq!(syn.0, 0b0000_0010);
        assert!(syn.has(TcpFlags::SYN));
        assert!(!syn.has(TcpFlags::ACK));

        let syn_ack = syn.combine(TcpFlags::ACK);
        assert_eq!(syn_ack.0, 0b0001_0010);
        assert!(syn_ack.has(TcpFlags::SYN));
        assert!(syn_ack.has(TcpFlags::ACK));
    }

    #[test]
    fn test_tcp_option_mss() {
        let opt = TcpOption::Mss(1460);
        assert_eq!(opt.len(), 4);
        assert_eq!(opt.to_bytes(), vec![2, 4, 0x05, 0xB4]);
    }

    #[test]
    fn test_tcp_option_window_scale() {
        let opt = TcpOption::WindowScale(7);
        assert_eq!(opt.len(), 3);
        assert_eq!(opt.to_bytes(), vec![3, 3, 7]);
    }

    #[test]
    fn test_tcp_option_timestamp() {
        let opt = TcpOption::Timestamp {
            tsval: 0x12345678,
            tsecr: 0x9ABCDEF0,
        };
        assert_eq!(opt.len(), 10);
        let bytes = opt.to_bytes();
        assert_eq!(bytes[0], 8); // Kind
        assert_eq!(bytes[1], 10); // Length
    }

    #[test]
    fn test_tcp_packet_builder_basic() {
        let packet = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .build_ip_packet()
            .expect("Failed to build packet");

        // Should have IPv4 (20) + TCP (20) = 40 bytes minimum
        assert!(packet.len() >= 40);

        // Check IPv4 header
        assert_eq!(packet[0] >> 4, 4); // Version
        assert_eq!(packet[0] & 0x0F, 5); // IHL
        assert_eq!(packet[9], 6); // Protocol (TCP)
    }

    #[test]
    fn test_tcp_packet_builder_with_options() {
        let packet = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .add_option(TcpOption::Mss(1460))
            .add_option(TcpOption::WindowScale(7))
            .build_ip_packet()
            .expect("Failed to build packet");

        // IPv4 (20) + TCP header (20) + MSS (4) + WScale (3) + padding (1) = 48
        assert!(packet.len() >= 48);
    }

    #[test]
    fn test_tcp_packet_builder_missing_fields() {
        let result = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_port(80)
            .build_ip_packet();

        assert!(result.is_err());
    }

    #[test]
    fn test_udp_packet_builder_basic() {
        let payload = b"Hello, UDP!";
        let packet = UdpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(53)
            .payload(payload.to_vec())
            .build_ip_packet()
            .expect("Failed to build packet");

        // Should have IPv4 (20) + UDP (8) + payload (11) = 39 bytes
        assert_eq!(packet.len(), 39);

        // Check IPv4 header
        assert_eq!(packet[0] >> 4, 4); // Version
        assert_eq!(packet[9], 17); // Protocol (UDP)
    }

    #[test]
    fn test_udp_packet_builder_empty_payload() {
        let packet = UdpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(53)
            .build_ip_packet()
            .expect("Failed to build packet");

        // IPv4 (20) + UDP (8) = 28 bytes
        assert_eq!(packet.len(), 28);
    }

    // Sprint 4.20 Phase 6: Bad checksum tests
    #[test]
    fn test_tcp_bad_checksum() {
        let packet = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .bad_checksum(true)
            .build_ip_packet()
            .expect("Failed to build packet");

        // TCP checksum is at offset 36-37 (IP header 20 bytes + TCP offset 16)
        let checksum = u16::from_be_bytes([packet[36], packet[37]]);
        assert_eq!(checksum, 0x0000, "Bad checksum should be 0x0000");
    }

    #[test]
    fn test_tcp_valid_checksum_default() {
        let packet = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .build_ip_packet()
            .expect("Failed to build packet");

        // TCP checksum should NOT be 0x0000 (valid checksum)
        let checksum = u16::from_be_bytes([packet[36], packet[37]]);
        assert_ne!(checksum, 0x0000, "Valid checksum should not be 0x0000");
    }

    #[test]
    fn test_tcp_bad_checksum_false() {
        let packet = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .bad_checksum(false)
            .build_ip_packet()
            .expect("Failed to build packet");

        // TCP checksum should NOT be 0x0000 when bad_checksum=false
        let checksum = u16::from_be_bytes([packet[36], packet[37]]);
        assert_ne!(checksum, 0x0000, "Valid checksum should not be 0x0000");
    }

    #[test]
    fn test_udp_bad_checksum() {
        let packet = UdpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(53)
            .payload(b"test".to_vec())
            .bad_checksum(true)
            .build_ip_packet()
            .expect("Failed to build packet");

        // UDP checksum is at offset 26-27 (IP header 20 bytes + UDP offset 6)
        let checksum = u16::from_be_bytes([packet[26], packet[27]]);
        assert_eq!(checksum, 0x0000, "Bad checksum should be 0x0000");
    }

    #[test]
    fn test_udp_valid_checksum_default() {
        let packet = UdpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(53)
            .payload(b"test".to_vec())
            .build_ip_packet()
            .expect("Failed to build packet");

        // UDP checksum should NOT be 0x0000 (valid checksum)
        let checksum = u16::from_be_bytes([packet[26], packet[27]]);
        assert_ne!(checksum, 0x0000, "Valid checksum should not be 0x0000");
    }

    // IPv6 Integration Tests

    #[test]
    fn test_tcp_ipv6_syn_packet() {
        use std::net::Ipv6Addr;

        let src = "2001:db8::1".parse::<Ipv6Addr>().unwrap();
        let dst = "2001:db8::2".parse::<Ipv6Addr>().unwrap();

        let packet = TcpPacketBuilder::new()
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .build_ipv6_packet(src, dst)
            .expect("Failed to build IPv6 TCP packet");

        // IPv6 header (40 bytes) + TCP header (20 bytes minimum) = 60 bytes minimum
        assert!(packet.len() >= 60);

        // Check IPv6 version (first 4 bits should be 6)
        assert_eq!(packet[0] >> 4, 6);

        // Check next header field (should be 6 for TCP)
        assert_eq!(packet[6], 6);

        // Verify checksum is non-zero (indicates it was calculated)
        let tcp_checksum_offset = 40 + 16; // IPv6 header + TCP checksum offset
        let checksum = u16::from_be_bytes([packet[tcp_checksum_offset], packet[tcp_checksum_offset + 1]]);
        assert_ne!(checksum, 0, "TCP checksum should be calculated");
    }

    #[test]
    fn test_udp_ipv6_packet() {
        use std::net::Ipv6Addr;

        let src = "2001:db8::1".parse::<Ipv6Addr>().unwrap();
        let dst = "2001:db8::2".parse::<Ipv6Addr>().unwrap();
        let payload = b"Hello, IPv6!";

        let packet = UdpPacketBuilder::new()
            .source_port(12345)
            .dest_port(53)
            .payload(payload.to_vec())
            .build_ipv6_packet(src, dst)
            .expect("Failed to build IPv6 UDP packet");

        // IPv6 header (40) + UDP header (8) + payload (12) = 60 bytes
        assert_eq!(packet.len(), 60);

        // Check IPv6 version
        assert_eq!(packet[0] >> 4, 6);

        // Check next header field (should be 17 for UDP)
        assert_eq!(packet[6], 17);

        // Verify checksum is non-zero
        let udp_checksum_offset = 40 + 6; // IPv6 header + UDP checksum offset
        let checksum = u16::from_be_bytes([packet[udp_checksum_offset], packet[udp_checksum_offset + 1]]);
        assert_ne!(checksum, 0, "UDP checksum should be calculated");
    }

    #[test]
    fn test_tcp_ipv6_with_options() {
        use std::net::Ipv6Addr;

        let src = "2001:db8::1".parse::<Ipv6Addr>().unwrap();
        let dst = "2001:db8::2".parse::<Ipv6Addr>().unwrap();

        let packet = TcpPacketBuilder::new()
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .add_option(TcpOption::Mss(1460))
            .add_option(TcpOption::WindowScale(7))
            .build_ipv6_packet(src, dst)
            .expect("Failed to build IPv6 TCP packet with options");

        // IPv6 (40) + TCP header (20) + MSS (4) + WScale (3) + padding (1) = 68 bytes
        assert!(packet.len() >= 68);
        assert_eq!(packet[0] >> 4, 6); // IPv6 version
    }

    #[test]
    fn test_ipv6_packet_size_comparison() {
        use std::net::Ipv6Addr;

        let src_v6 = "2001:db8::1".parse::<Ipv6Addr>().unwrap();
        let dst_v6 = "2001:db8::2".parse::<Ipv6Addr>().unwrap();

        let packet_v6 = TcpPacketBuilder::new()
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .build_ipv6_packet(src_v6, dst_v6)
            .expect("Failed to build IPv6 packet");

        let packet_v4 = TcpPacketBuilder::new()
            .source_ip(std::net::Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(std::net::Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .build_ip_packet()
            .expect("Failed to build IPv4 packet");

        // IPv6 header is 20 bytes larger than IPv4 (40 vs 20)
        assert_eq!(packet_v6.len(), packet_v4.len() + 20);
    }

    #[test]
    fn test_ipv6_tcp_bad_checksum() {
        use std::net::Ipv6Addr;

        let src = "2001:db8::1".parse::<Ipv6Addr>().unwrap();
        let dst = "2001:db8::2".parse::<Ipv6Addr>().unwrap();

        let packet = TcpPacketBuilder::new()
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .bad_checksum(true)
            .build_ipv6_packet(src, dst)
            .expect("Failed to build IPv6 TCP packet");

        // TCP checksum is at offset 40 (IPv6 header) + 16 (TCP checksum offset)
        let tcp_checksum_offset = 40 + 16;
        let checksum = u16::from_be_bytes([packet[tcp_checksum_offset], packet[tcp_checksum_offset + 1]]);
        assert_eq!(checksum, 0x0000, "Bad checksum should be 0x0000");
    }
}
