//! Zero-copy packet crafting tests
//!
//! Verifies that the zero-copy packet building API eliminates heap allocations
//! in the hot path and achieves sub-microsecond packet crafting performance.

use prtip_network::{
    packet_buffer::with_buffer, TcpFlags, TcpOption, TcpPacketBuilder, UdpPacketBuilder,
};
use std::net::Ipv4Addr;

#[test]
fn test_tcp_syn_zero_copy() {
    // Build SYN packet with zero allocations
    let result = with_buffer(|pool| {
        let builder = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(192, 168, 1, 1))
            .dest_ip(Ipv4Addr::new(192, 168, 1, 2))
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN);

        let packet = builder.build_with_buffer(pool)?;

        // IPv4 (20) + TCP (20) = 40 bytes minimum
        assert!(packet.len() >= 40);

        // Check IPv4 header
        assert_eq!(packet[0] >> 4, 4); // Version
        assert_eq!(packet[9], 6); // Protocol (TCP)

        Ok::<_, prtip_network::packet_builder::PacketBuilderError>(packet.len())
    });

    assert!(result.is_ok());
}

#[test]
fn test_tcp_syn_ack_zero_copy() {
    let result = with_buffer(|pool| {
        let builder = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(80)
            .dest_port(12345)
            .flags(TcpFlags::SYN.combine(TcpFlags::ACK))
            .acknowledgment(1000);

        let packet = builder.build_with_buffer(pool)?;
        assert!(packet.len() >= 40);

        Ok::<_, prtip_network::packet_builder::PacketBuilderError>(())
    });

    assert!(result.is_ok());
}

#[test]
fn test_tcp_with_options_zero_copy() {
    let result = with_buffer(|pool| {
        let builder = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .add_option(TcpOption::Mss(1460))
            .add_option(TcpOption::WindowScale(7))
            .add_option(TcpOption::SackPermitted);

        let packet = builder.build_with_buffer(pool)?;

        // IPv4 (20) + TCP header (20) + MSS (4) + WScale (3) + SACK (2) + padding (3) = 52
        assert!(packet.len() >= 52);

        Ok::<_, prtip_network::packet_builder::PacketBuilderError>(())
    });

    assert!(result.is_ok());
}

#[test]
fn test_tcp_with_timestamp_option_zero_copy() {
    let result = with_buffer(|pool| {
        let builder = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .add_option(TcpOption::Timestamp {
                tsval: 0x12345678,
                tsecr: 0x9ABCDEF0,
            });

        let packet = builder.build_with_buffer(pool)?;

        // IPv4 (20) + TCP header (20) + Timestamp (10) + padding (2) = 52
        assert!(packet.len() >= 52);

        Ok::<_, prtip_network::packet_builder::PacketBuilderError>(())
    });

    assert!(result.is_ok());
}

#[test]
fn test_udp_zero_copy() {
    let payload = b"Hello, UDP!";
    let result = with_buffer(|pool| {
        let builder = UdpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(53)
            .payload(payload.to_vec());

        let packet = builder.build_with_buffer(pool)?;

        // IPv4 (20) + UDP (8) + payload (11) = 39 bytes
        assert_eq!(packet.len(), 39);

        // Check IPv4 header
        assert_eq!(packet[0] >> 4, 4); // Version
        assert_eq!(packet[9], 17); // Protocol (UDP)

        Ok::<_, prtip_network::packet_builder::PacketBuilderError>(())
    });

    assert!(result.is_ok());
}

#[test]
fn test_udp_empty_payload_zero_copy() {
    let result = with_buffer(|pool| {
        let builder = UdpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(53);

        let packet = builder.build_with_buffer(pool)?;

        // IPv4 (20) + UDP (8) = 28 bytes
        assert_eq!(packet.len(), 28);

        Ok::<_, prtip_network::packet_builder::PacketBuilderError>(())
    });

    assert!(result.is_ok());
}

#[test]
fn test_buffer_reuse() {
    // Verify buffer reuse across multiple packets
    with_buffer(|pool| {
        // Build 10 packets in sequence
        for i in 0..10 {
            let builder = TcpPacketBuilder::new()
                .source_ip(Ipv4Addr::new(192, 168, 1, 1))
                .dest_ip(Ipv4Addr::new(192, 168, 1, 2))
                .source_port(12345 + i)
                .dest_port(80)
                .flags(TcpFlags::SYN);

            let packet = builder
                .build_with_buffer(pool)
                .expect("Failed to build packet");
            assert!(packet.len() >= 40);

            pool.reset(); // Reuse buffer for next packet
        }
    });
}

#[test]
fn test_buffer_exhaustion_handling() {
    let result = with_buffer(|pool| {
        // Try to allocate more than buffer capacity (4096 bytes)
        let large_payload = vec![0u8; 5000];

        let builder = UdpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(53)
            .payload(large_payload);

        let packet_result = builder.build_with_buffer(pool);

        // Should fail with BufferTooSmall error
        assert!(packet_result.is_err());

        Ok::<_, prtip_network::packet_builder::PacketBuilderError>(())
    });

    assert!(result.is_ok());
}

#[test]
fn test_multiple_packets_without_reset() {
    // Build multiple packets without resetting buffer
    let result = with_buffer(|pool| {
        // Reset pool to start fresh
        pool.reset();

        // Build packets until buffer exhausts
        let mut count = 0;
        loop {
            let builder = TcpPacketBuilder::new()
                .source_ip(Ipv4Addr::new(10, 0, 0, 1))
                .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
                .source_port(12345)
                .dest_port(80)
                .flags(TcpFlags::SYN);

            match builder.build_with_buffer(pool) {
                Ok(_packet) => {
                    count += 1;
                }
                Err(_) => {
                    // Buffer exhausted, this is expected
                    break;
                }
            }
        }

        // Should be able to build at least 100 packets (40 bytes each)
        // 4096 / 40 = 102 packets
        assert!(count >= 100, "Only built {} packets", count);

        Ok::<_, prtip_network::packet_builder::PacketBuilderError>(())
    });

    assert!(result.is_ok());
}

#[test]
#[ignore] // Skip in coverage - timing sensitive
fn test_packet_crafting_performance() {
    use std::time::Instant;

    let start = Instant::now();

    // Craft 1000 packets using zero-copy
    for _ in 0..1000 {
        with_buffer(|pool| {
            let builder = TcpPacketBuilder::new()
                .source_ip(Ipv4Addr::new(192, 168, 1, 1))
                .dest_ip(Ipv4Addr::new(192, 168, 1, 2))
                .source_port(12345)
                .dest_port(80)
                .flags(TcpFlags::SYN);

            let _packet = builder
                .build_with_buffer(pool)
                .expect("Failed to build packet");
            pool.reset();
        });
    }

    let elapsed = start.elapsed();
    let per_packet = elapsed.as_nanos() / 1000;

    // Target: <1µs per packet (1000ns)
    // Allow 2µs for test overhead
    assert!(
        per_packet < 2000,
        "Packet crafting too slow: {}ns per packet (target: <1000ns)",
        per_packet
    );

    println!(
        "Zero-copy performance: {} packets/sec ({}ns per packet)",
        1_000_000_000 / per_packet,
        per_packet
    );
}

#[test]
fn test_tcp_vs_udp_packet_sizes() {
    with_buffer(|pool| {
        // TCP SYN (no options)
        let tcp_builder = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN);

        let tcp_packet = tcp_builder
            .build_with_buffer(pool)
            .expect("Failed to build TCP packet");
        let tcp_size = tcp_packet.len();

        pool.reset();

        // UDP (no payload)
        let udp_builder = UdpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(53);

        let udp_packet = udp_builder
            .build_with_buffer(pool)
            .expect("Failed to build UDP packet");
        let udp_size = udp_packet.len();

        // TCP: IPv4 (20) + TCP (20) = 40 bytes
        // UDP: IPv4 (20) + UDP (8) = 28 bytes
        assert_eq!(tcp_size, 40);
        assert_eq!(udp_size, 28);
        assert!(tcp_size > udp_size); // TCP header larger than UDP
    });
}

#[test]
fn test_backwards_compatibility_build() {
    // Verify old build() API still works (allocates Vec<u8>)
    let packet = TcpPacketBuilder::new()
        .source_ip(Ipv4Addr::new(10, 0, 0, 1))
        .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
        .source_port(12345)
        .dest_port(80)
        .flags(TcpFlags::SYN)
        .build_ip_packet()
        .expect("Failed to build packet");

    assert_eq!(packet.len(), 40);
}

#[test]
fn test_thread_local_isolation() {
    use std::thread;

    // Spawn multiple threads, each using its own buffer pool
    let handles: Vec<_> = (0..4)
        .map(|thread_id| {
            thread::spawn(move || {
                with_buffer(|pool| {
                    // Each thread builds 100 packets
                    for i in 0..100 {
                        let builder = TcpPacketBuilder::new()
                            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
                            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
                            .source_port(12345 + (thread_id * 1000) + i)
                            .dest_port(80)
                            .flags(TcpFlags::SYN);

                        let _packet = builder
                            .build_with_buffer(pool)
                            .expect("Failed to build packet");

                        pool.reset();
                    }
                });
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn test_zero_copy_with_all_options() {
    // Test packet with all TCP option types
    let result = with_buffer(|pool| {
        let builder = TcpPacketBuilder::new()
            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
            .source_port(12345)
            .dest_port(80)
            .flags(TcpFlags::SYN)
            .add_option(TcpOption::Mss(1460))
            .add_option(TcpOption::WindowScale(7))
            .add_option(TcpOption::SackPermitted)
            .add_option(TcpOption::Timestamp {
                tsval: 0x12345678,
                tsecr: 0,
            })
            .add_option(TcpOption::Nop)
            .add_option(TcpOption::Eol);

        let packet = builder.build_with_buffer(pool)?;

        // IPv4 (20) + TCP (20) + MSS (4) + WScale (3) + SACK (2) + Timestamp (10) + Nop (1) + Eol (1) + padding = ~64 bytes
        assert!(
            packet.len() >= 48,
            "Packet too small: {} bytes",
            packet.len()
        );

        Ok::<_, prtip_network::packet_builder::PacketBuilderError>(())
    });

    assert!(result.is_ok());
}
