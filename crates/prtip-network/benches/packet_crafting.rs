//! Packet Crafting Performance Benchmarks
//!
//! Sprint 4.17 Phase 3: Measures performance of packet building operations
//! comparing OLD API (with allocations) vs NEW zero-copy API.
//!
//! ## Benchmark Groups
//!
//! 1. **tcp_packet_old_api** - TcpPacketBuilder::build() (allocates Vec)
//! 2. **tcp_packet_zero_copy** - TcpPacketBuilder::build_with_buffer() (zero-copy)
//! 3. **packet_throughput** - Packets per second (1K packet batches)
//!
//! ## Expected Results (Phase 2 predictions)
//!
//! - Old API: ~5µs per packet (3-7 allocations)
//! - Zero-copy API: ~800ns per packet (0 allocations)
//! - Speedup: ~5-6x improvement
//!
//! ## Running Benchmarks
//!
//! ```bash
//! cargo bench --bench packet_crafting -- --save-baseline phase3
//! ```

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use prtip_network::{packet_buffer::with_buffer, TcpFlags, TcpOption, TcpPacketBuilder};
use std::net::Ipv4Addr;

/// Benchmark OLD API (with allocations)
///
/// Uses TcpPacketBuilder::build() which allocates a Vec<u8> per packet.
/// Expected: ~5µs per packet due to allocation overhead.
fn bench_tcp_build_old_api(c: &mut Criterion) {
    let mut group = c.benchmark_group("tcp_packet_old_api");

    for count in [10, 100, 1000] {
        group.throughput(Throughput::Elements(count));

        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                for _ in 0..count {
                    let packet = TcpPacketBuilder::new()
                        .source_ip(Ipv4Addr::new(10, 0, 0, 1))
                        .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
                        .source_port(12345)
                        .dest_port(80)
                        .flags(TcpFlags::SYN)
                        .window(65535)
                        .add_option(TcpOption::Mss(1460))
                        .build()
                        .unwrap();

                    black_box(packet);
                }
            });
        });
    }

    group.finish();
}

/// Benchmark NEW zero-copy API (no allocations)
///
/// Uses TcpPacketBuilder::build_with_buffer() which writes directly to
/// thread-local buffer without allocating.
/// Expected: ~800ns per packet (5-6x faster than old API).
fn bench_tcp_build_zero_copy(c: &mut Criterion) {
    let mut group = c.benchmark_group("tcp_packet_zero_copy");

    for count in [10, 100, 1000] {
        group.throughput(Throughput::Elements(count));

        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                with_buffer(|pool| {
                    for _ in 0..count {
                        let packet = TcpPacketBuilder::new()
                            .source_ip(Ipv4Addr::new(10, 0, 0, 1))
                            .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
                            .source_port(12345)
                            .dest_port(80)
                            .flags(TcpFlags::SYN)
                            .window(65535)
                            .add_option(TcpOption::Mss(1460))
                            .build_with_buffer(pool)
                            .unwrap();

                        black_box(packet);
                        pool.reset();
                    }
                });
            });
        });
    }

    group.finish();
}

/// Benchmark packet crafting throughput (packets per second)
///
/// Measures end-to-end packet crafting performance for 1K packet batches.
/// Validates 1M+ pps target (would need <1µs per packet).
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("packet_throughput");
    group.sample_size(50);

    // Old API throughput (1K packets)
    group.bench_function("old_api_1k_packets", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let packet = TcpPacketBuilder::new()
                    .source_ip(Ipv4Addr::new(10, 0, 0, 1))
                    .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
                    .source_port(12345)
                    .dest_port(80)
                    .flags(TcpFlags::SYN)
                    .window(65535)
                    .build()
                    .unwrap();
                black_box(packet);
            }
        });
    });

    // Zero-copy API throughput (1K packets)
    group.bench_function("zero_copy_1k_packets", |b| {
        b.iter(|| {
            with_buffer(|pool| {
                for _ in 0..1000 {
                    let packet = TcpPacketBuilder::new()
                        .source_ip(Ipv4Addr::new(10, 0, 0, 1))
                        .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
                        .source_port(12345)
                        .dest_port(80)
                        .flags(TcpFlags::SYN)
                        .window(65535)
                        .build_with_buffer(pool)
                        .unwrap();
                    black_box(packet);
                    pool.reset();
                }
            });
        });
    });

    group.finish();
}

/// Benchmark with TCP options (realistic SYN packet)
///
/// Tests performance with typical TCP options (MSS, window scale, SACK).
/// This matches real-world SYN scanner packet structure.
fn bench_with_options(c: &mut Criterion) {
    let mut group = c.benchmark_group("packet_with_options");

    // Old API with options
    group.bench_function("old_api_with_options", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let packet = TcpPacketBuilder::new()
                    .source_ip(Ipv4Addr::new(10, 0, 0, 1))
                    .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
                    .source_port(12345)
                    .dest_port(80)
                    .flags(TcpFlags::SYN)
                    .window(65535)
                    .add_option(TcpOption::Mss(1460))
                    .add_option(TcpOption::WindowScale(7))
                    .add_option(TcpOption::SackPermitted)
                    .build()
                    .unwrap();
                black_box(packet);
            }
        });
    });

    // Zero-copy API with options
    group.bench_function("zero_copy_with_options", |b| {
        b.iter(|| {
            with_buffer(|pool| {
                for _ in 0..100 {
                    let packet = TcpPacketBuilder::new()
                        .source_ip(Ipv4Addr::new(10, 0, 0, 1))
                        .dest_ip(Ipv4Addr::new(10, 0, 0, 2))
                        .source_port(12345)
                        .dest_port(80)
                        .flags(TcpFlags::SYN)
                        .window(65535)
                        .add_option(TcpOption::Mss(1460))
                        .add_option(TcpOption::WindowScale(7))
                        .add_option(TcpOption::SackPermitted)
                        .build_with_buffer(pool)
                        .unwrap();
                    black_box(packet);
                    pool.reset();
                }
            });
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tcp_build_old_api,
    bench_tcp_build_zero_copy,
    bench_throughput,
    bench_with_options
);
criterion_main!(benches);
