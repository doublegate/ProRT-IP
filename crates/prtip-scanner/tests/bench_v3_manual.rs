//! Manual V3 Benchmark Test
//!
//! This test can be run with: cargo test --release bench_v3_ -- --nocapture --test-threads=1

use prtip_scanner::AdaptiveRateLimiterV3;
use std::time::Instant;

#[tokio::test]
async fn bench_v3_10k() {
    run_bench(10_000, 1_000, "10K").await;
}

#[tokio::test]
async fn bench_v3_50k() {
    run_bench(50_000, 1_000, "50K").await;
}

#[tokio::test]
async fn bench_v3_100k() {
    run_bench(100_000, 1_000, "100K").await;
}

#[tokio::test]
async fn bench_v3_500k() {
    run_bench(500_000, 1_000, "500K").await;
}

#[tokio::test]
async fn bench_v3_1m() {
    run_bench(1_000_000, 1_000, "1M").await;
}

async fn run_bench(rate: u64, packets: u64, label: &str) {
    let limiter = AdaptiveRateLimiterV3::new(Some(rate));

    // Warmup
    for _ in 0..100 {
        limiter.acquire().await.unwrap();
    }

    // Let monitor stabilize
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Benchmark (single run)
    let start = Instant::now();
    for _ in 0..packets {
        limiter.acquire().await.unwrap();
    }
    let elapsed = start.elapsed();

    println!("BENCH_RESULT: {},{},{:.3}",
        label,
        rate,
        elapsed.as_secs_f64() * 1000.0
    );
}
