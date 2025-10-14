//! Batch I/O Performance Benchmarks
//!
//! Measures performance of sendmmsg/recvmmsg batch packet operations.
//!
//! # Running Benchmarks
//!
//! ```bash
//! # Build first
//! cargo build --release
//!
//! # Run batch I/O benchmarks (requires root on Linux for raw sockets)
//! sudo cargo bench --bench batch_io
//! ```
//!
//! # Platform Support
//!
//! - **Linux**: Full sendmmsg/recvmmsg benchmarks with actual syscalls
//! - **macOS/Windows**: Fallback benchmarks (API overhead only, no actual sending)
//!
//! # Metrics Measured
//!
//! - **Batch Assembly Time**: Time to prepare mmsghdr structures
//! - **Syscall Overhead**: Time spent in sendmmsg/recvmmsg calls
//! - **Throughput**: Packets per second at various batch sizes
//! - **Latency**: Per-packet processing time

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use prtip_network::{BatchSender, PacketBatch};
use std::time::Duration;

/// Benchmark packet batch creation and management
fn bench_packet_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("packet_batch");

    for batch_size in [16, 32, 64, 128, 256, 512, 1024] {
        group.throughput(Throughput::Elements(batch_size as u64));

        group.bench_with_input(
            BenchmarkId::new("create_and_fill", batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    let mut batch = PacketBatch::new(size);
                    for _ in 0..size {
                        // Simulate typical SYN packet (54 bytes: Ethernet + IP + TCP)
                        let packet = vec![0u8; 54];
                        let _ = batch.add(packet);
                    }
                    black_box(batch);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("reuse_with_clear", batch_size),
            &batch_size,
            |b, &size| {
                let mut batch = PacketBatch::new(size);
                b.iter(|| {
                    batch.clear();
                    for _ in 0..size {
                        let packet = vec![0u8; 54];
                        let _ = batch.add(packet);
                    }
                    black_box(&batch);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark batch sender creation and packet addition
fn bench_batch_sender_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_sender_overhead");

    // Note: This doesn't require root as we're not actually creating raw sockets
    // We're just testing the API overhead

    for batch_size in [16, 32, 64, 128] {
        group.throughput(Throughput::Elements(batch_size as u64));

        group.bench_with_input(
            BenchmarkId::new("add_packet", batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    // Note: BatchSender::new() will fail without root, so we test
                    // just the batch management logic via PacketBatch
                    let mut batch = PacketBatch::new(size);
                    for _ in 0..size {
                        let packet = vec![0u8; 54];
                        let _ = batch.add(packet);
                    }
                    black_box(batch);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark syscall overhead estimation (Linux only)
///
/// This benchmark requires root privileges to create raw sockets.
/// On non-Linux platforms or without root, it will be skipped.
#[cfg(target_os = "linux")]
fn bench_sendmmsg_syscall(c: &mut Criterion) {
    // Check if we have raw socket capability
    let has_cap = std::process::Command::new("sh")
        .arg("-c")
        .arg("capsh --print | grep -q cap_net_raw")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !has_cap && unsafe { libc::geteuid() } != 0 {
        eprintln!("⚠️  Skipping sendmmsg benchmarks: requires root or CAP_NET_RAW");
        return;
    }

    let mut group = c.benchmark_group("sendmmsg_syscall");
    group.sample_size(20); // Reduce sample size for syscall benchmarks
    group.measurement_time(Duration::from_secs(10));

    // Try to create a batch sender (requires root)
    let sender_result = BatchSender::new("lo", 64);

    if let Ok(mut sender) = sender_result {
        for batch_size in [16, 32, 64, 128] {
            group.throughput(Throughput::Elements(batch_size as u64));

            group.bench_with_input(
                BenchmarkId::new("flush", batch_size),
                &batch_size,
                |b, &size| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    b.iter(|| {
                        rt.block_on(async {
                            // Fill batch
                            for _ in 0..size {
                                let packet = vec![0u8; 54];
                                let _ = sender.add_packet(packet);
                            }

                            // Measure flush time (includes sendmmsg syscall)
                            let result = sender.flush(3).await;
                            let _ = black_box(result);
                        })
                    });
                },
            );
        }
    } else {
        eprintln!("⚠️  Could not create BatchSender (requires root)");
    }

    group.finish();
}

/// Benchmark packet crafting + batch assembly pipeline
fn bench_end_to_end_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_pipeline");

    for batch_size in [16, 32, 64, 128] {
        group.throughput(Throughput::Elements(batch_size as u64));

        group.bench_with_input(
            BenchmarkId::new("craft_and_batch", batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    let mut batch = PacketBatch::new(size);

                    for _ in 0..size {
                        // Simulate packet crafting (vec allocation)
                        let mut packet = vec![0; 54];
                        packet[0] = 0x45; // IPv4 version + IHL
                        packet[9] = 6; // TCP protocol

                        let _ = batch.add(packet);
                    }

                    black_box(batch);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark various packet sizes
fn bench_packet_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("packet_sizes");

    let batch_size = 64; // Fixed batch size

    for packet_size in [54, 128, 256, 512, 1024, 1500] {
        group.throughput(Throughput::Bytes((batch_size * packet_size) as u64));

        group.bench_with_input(
            BenchmarkId::new("batch_fill", packet_size),
            &packet_size,
            |b, &pkt_size| {
                b.iter(|| {
                    let mut batch = PacketBatch::new(batch_size);
                    for _ in 0..batch_size {
                        let packet = vec![0u8; pkt_size];
                        let _ = batch.add(packet);
                    }
                    black_box(batch);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark batch size selection (simulated adaptive sizing)
fn bench_adaptive_batch_sizing(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_batch_sizing");

    // Simulate different packet rates to test adaptive batch size selection
    for pps in [1_000, 10_000, 100_000, 1_000_000] {
        group.bench_with_input(BenchmarkId::from_parameter(pps), &pps, |b, &rate| {
            b.iter(|| {
                // Adaptive batch size logic
                let batch_size = if rate < 10_000 {
                    16
                } else if rate < 100_000 {
                    32
                } else if rate < 500_000 {
                    64
                } else {
                    128
                };

                let mut batch = PacketBatch::new(batch_size);
                for _ in 0..batch_size {
                    let packet = vec![0u8; 54];
                    let _ = batch.add(packet);
                }
                black_box(batch);
            });
        });
    }

    group.finish();
}

// Criterion benchmark groups
#[cfg(target_os = "linux")]
criterion_group!(
    benches,
    bench_packet_batch,
    bench_batch_sender_overhead,
    bench_sendmmsg_syscall,
    bench_end_to_end_pipeline,
    bench_packet_sizes,
    bench_adaptive_batch_sizing
);

#[cfg(not(target_os = "linux"))]
criterion_group!(
    benches,
    bench_packet_batch,
    bench_batch_sender_overhead,
    bench_end_to_end_pipeline,
    bench_packet_sizes,
    bench_adaptive_batch_sizing
);

criterion_main!(benches);
