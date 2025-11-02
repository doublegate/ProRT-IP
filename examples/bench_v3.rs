//! AdaptiveRateLimiterV3 Benchmark Example

use prtip_scanner::AdaptiveRateLimiterV3;
use std::env;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let rate: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100_000);
    let packets: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1_000);

    // Create limiter
    let limiter = AdaptiveRateLimiterV3::new(Some(rate));

    // Warmup
    for _ in 0..100 {
        limiter.acquire().await.unwrap();
    }

    // Let monitor stabilize
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Benchmark
    let start = Instant::now();
    for _ in 0..packets {
        limiter.acquire().await.unwrap();
    }
    let elapsed = start.elapsed();

    // Output: rate,duration_ms
    println!("{},{:.3}", rate, elapsed.as_secs_f64() * 1000.0);
}
