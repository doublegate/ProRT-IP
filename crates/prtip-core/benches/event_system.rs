//! Event System Performance Benchmarks
//!
//! Validates EventBus meets performance targets:
//! - Publish latency: <10ms p99 (preferably <1ms)
//! - Subscribe latency: <100μs
//! - Concurrent overhead: <5% vs single-threaded
//! - History query: <100μs for 100 events

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use prtip_core::event_bus::{EventBus, EventFilter};
use prtip_core::events::{
    DiscoveryMethod, MetricType, ScanEvent, ScanEventType, ScanStage, Throughput,
};
use prtip_core::types::ScanType;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Helper to create test events of different types
fn create_test_event(event_type: &str) -> ScanEvent {
    let scan_id = Uuid::new_v4();
    let timestamp = SystemTime::now();

    match event_type {
        "lifecycle" => ScanEvent::ScanStarted {
            scan_id,
            scan_type: ScanType::Syn,
            target_count: 1000,
            port_count: 1000,
            timestamp,
        },
        "progress" => ScanEvent::ProgressUpdate {
            scan_id,
            stage: ScanStage::ScanningPorts,
            percentage: 50.0,
            completed: 500,
            total: 1000,
            throughput: Throughput::default(),
            eta: None,
            timestamp,
        },
        "discovery" => ScanEvent::HostDiscovered {
            scan_id,
            ip: "192.168.1.1".parse().unwrap(),
            method: DiscoveryMethod::IcmpEcho,
            latency_ms: Some(10),
            timestamp,
        },
        "diagnostic" => ScanEvent::MetricRecorded {
            scan_id,
            metric: MetricType::PacketsSent,
            value: 1000.0,
            timestamp,
        },
        _ => panic!("Unknown event type"),
    }
}

/// Helper to create filters with varying complexity
fn create_complex_filter(complexity: usize) -> EventFilter {
    match complexity {
        1 => EventFilter::All,
        5 => EventFilter::EventType(vec![
            ScanEventType::ScanStarted,
            ScanEventType::ScanCompleted,
            ScanEventType::ProgressUpdate,
            ScanEventType::PortFound,
            ScanEventType::HostDiscovered,
        ]),
        10 => EventFilter::EventType(vec![
            ScanEventType::ScanStarted,
            ScanEventType::ScanCompleted,
            ScanEventType::ProgressUpdate,
            ScanEventType::PortFound,
            ScanEventType::HostDiscovered,
            ScanEventType::StageChanged,
            ScanEventType::ServiceDetected,
            ScanEventType::MetricRecorded,
            ScanEventType::WarningIssued,
            ScanEventType::RateLimitTriggered,
        ]),
        _ => EventFilter::All,
    }
}

/// Benchmark single event publication
fn benchmark_publish(c: &mut Criterion) {
    let mut group = c.benchmark_group("publish");

    let rt = tokio::runtime::Runtime::new().unwrap();

    for event_type in ["lifecycle", "progress", "discovery", "diagnostic"] {
        group.bench_with_input(
            BenchmarkId::new("single_event", event_type),
            &event_type,
            |b, event_type| {
                let bus = EventBus::new(1000);
                let event = create_test_event(event_type);

                b.to_async(&rt).iter(|| async {
                    bus.publish(black_box(event.clone())).await;
                });
            },
        );
    }

    group.finish();
}

/// Benchmark subscription with different filter complexities
fn benchmark_subscribe(c: &mut Criterion) {
    let mut group = c.benchmark_group("subscribe");

    let rt = tokio::runtime::Runtime::new().unwrap();

    for filter_complexity in [1, 5, 10] {
        group.bench_with_input(
            BenchmarkId::new("filter_complexity", filter_complexity),
            &filter_complexity,
            |b, &complexity| {
                let bus = EventBus::new(1000);
                let filter = create_complex_filter(complexity);

                b.to_async(&rt).iter(|| async {
                    let (tx, _rx) = mpsc::unbounded_channel();
                    bus.subscribe(tx, black_box(filter.clone())).await;
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent publishing from multiple threads
fn benchmark_concurrent_publish(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent");

    let rt = tokio::runtime::Runtime::new().unwrap();

    for num_publishers in [1, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("publishers", num_publishers),
            &num_publishers,
            |b, &num| {
                let bus = Arc::new(EventBus::new(1000));

                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();

                    for _ in 0..num {
                        let bus = bus.clone();
                        handles.push(tokio::spawn(async move {
                            for _ in 0..100 {
                                bus.publish(create_test_event("progress"))
                                    .await;
                            }
                        }));
                    }

                    for h in handles {
                        h.await.unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark history query operations
fn benchmark_history_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("history");

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Pre-populate history with 1,000 events
    let bus = Arc::new(EventBus::new(1000));
    rt.block_on(async {
        for _ in 0..1000 {
            bus.publish(create_test_event("discovery")).await;
        }
    });

    group.bench_function("get_recent_100", |b| {
        b.to_async(&rt).iter(|| async {
            bus.get_history(black_box(100)).await;
        });
    });

    group.bench_function("get_all", |b| {
        b.to_async(&rt).iter(|| async {
            bus.get_history(black_box(1000)).await;
        });
    });

    group.bench_function("query_filtered", |b| {
        b.to_async(&rt).iter(|| async {
            bus.query_history(
                black_box(EventFilter::EventType(vec![
                    ScanEventType::HostDiscovered,
                ])),
                black_box(100),
            )
            .await;
        });
    });

    group.finish();
}

/// Benchmark publish-to-receive latency (end-to-end)
fn benchmark_publish_receive_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency");

    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("publish_to_receive", |b| {
        b.to_async(&rt).iter(|| async {
            let bus = EventBus::new(1000);
            let (tx, mut rx) = mpsc::unbounded_channel();
            bus.subscribe(tx, EventFilter::All).await;

            let event = create_test_event("progress");
            bus.publish(event).await;

            rx.recv().await.unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_publish,
    benchmark_subscribe,
    benchmark_concurrent_publish,
    benchmark_history_query,
    benchmark_publish_receive_latency
);
criterion_main!(benches);
