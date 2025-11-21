//! ProRT-IP Network Layer
//!
//! Cross-platform packet capture and privilege management for network scanning.
//!
//! # Examples
//!
//! ```no_run
//! use prtip_network::{capture::create_capture, check_privileges};
//!
//! // Check privileges first
//! check_privileges().expect("Insufficient privileges");
//!
//! // Create and open packet capture
//! let mut capture = create_capture().unwrap();
//! capture.open(Some("eth0")).unwrap();
//!
//! // Send a packet
//! let packet = vec![0u8; 64]; // Example packet
//! capture.send_packet(&packet).unwrap();
//!
//! // Receive a packet
//! if let Some(received) = capture.receive_packet(1000).unwrap() {
//!     println!("Received {} bytes", received.len());
//! }
//!
//! // Clean up
//! capture.close().unwrap();
//! ```

pub mod adaptive_batch;
pub mod batch_sender;
pub mod capture;
pub mod cdn_detector;
pub mod fragmentation;
pub mod icmpv6;
pub mod interface;
pub mod ipv6_packet;
pub mod large_buffer_pool;
pub mod numa;
pub mod packet_buffer;
pub mod packet_builder;
pub mod privilege;
pub mod protocol_payloads;

// Re-export commonly used items
pub use adaptive_batch::{AdaptiveBatchSizer, AdaptiveConfig, PerformanceMonitor};
pub use batch_sender::{
    BatchReceiver, BatchSender, PacketBatch, PlatformCapabilities, ReceivedPacket, MAX_BATCH_SIZE,
};
pub use capture::{create_capture, PacketCapture};
pub use cdn_detector::{CdnDetector, CdnProvider};
pub use fragmentation::{
    defragment_packets, fragment_tcp_packet, validate_mtu, MIN_MTU, NMAP_F_MTU,
};
pub use icmpv6::{Icmpv6PacketBuilder, Icmpv6ResponseParser};
pub use ipv6_packet::{parse_ipv6_header, ExtensionHeader, Ipv6Header, Ipv6PacketBuilder};
pub use large_buffer_pool::{
    BufferTier, LargeBufferPool, PoolStats, PooledBuffer, SharedPacket, TIER_1_SIZE, TIER_2_SIZE,
    TIER_3_SIZE,
};
pub use packet_buffer::{with_buffer, PacketBuffer};
pub use packet_builder::{TcpFlags, TcpOption, TcpPacketBuilder, UdpPacketBuilder};
pub use privilege::{check_privileges, drop_privileges, has_raw_socket_capability};
pub use protocol_payloads::get_udp_payload;
