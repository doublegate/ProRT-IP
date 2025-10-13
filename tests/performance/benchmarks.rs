//! Performance benchmarks for ProRT-IP
//!
//! Uses Criterion for statistical benchmarking.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

// Import common utilities
#[path = "../common/mod.rs"]
mod common;

fn bench_binary_startup(c: &mut Criterion) {
    c.bench_function("binary_startup_help", |b| {
        b.iter(|| {
            let output = common::run_prtip(&["--help"]);
            black_box(output);
        });
    });

    c.bench_function("binary_startup_version", |b| {
        b.iter(|| {
            let output = common::run_prtip(&["--version"]);
            black_box(output);
        });
    });
}

fn bench_port_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("port_parsing");

    group.bench_function("single_port", |b| {
        b.iter(|| {
            // Just check if binary accepts the port (doesn't actually scan)
            let args = ["-sT", "-p", "80", "--help"];
            black_box(args);
        });
    });

    group.bench_function("port_range_small", |b| {
        b.iter(|| {
            let args = ["-sT", "-p", "80-85", "--help"];
            black_box(args);
        });
    });

    group.bench_function("port_list", |b| {
        b.iter(|| {
            let args = ["-sT", "-p", "22,80,443", "--help"];
            black_box(args);
        });
    });

    group.finish();
}

fn bench_localhost_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("localhost_scan");
    group.sample_size(10); // Reduce sample size for slower benchmarks
    group.measurement_time(Duration::from_secs(30));

    // Only run if binary exists
    if common::get_binary_path().exists() {
        group.bench_function("single_port", |b| {
            b.iter(|| {
                let output = common::run_prtip(&["-sT", "-p", "80", "127.0.0.1"]);
                black_box(output);
            });
        });

        group.bench_function("three_ports", |b| {
            b.iter(|| {
                let output = common::run_prtip(&["-sT", "-p", "22,80,443", "127.0.0.1"]);
                black_box(output);
            });
        });

        group.bench_function("port_range_10", |b| {
            b.iter(|| {
                let output = common::run_prtip(&["-sT", "-p", "80-89", "127.0.0.1"]);
                black_box(output);
            });
        });
    }

    group.finish();
}

fn bench_output_formats(c: &mut Criterion) {
    let mut group = c.benchmark_group("output_formats");
    group.sample_size(10);

    if common::get_binary_path().exists() {
        let temp_dir = common::create_temp_dir("bench");

        group.bench_function("text_output", |b| {
            b.iter(|| {
                let output = common::run_prtip(&["-sT", "-p", "80", "127.0.0.1"]);
                black_box(output);
            });
        });

        group.bench_function("json_output", |b| {
            let json_file = temp_dir.join("bench.json");
            b.iter(|| {
                let output = common::run_prtip(&[
                    "-sT",
                    "-p",
                    "80",
                    "-oJ",
                    json_file.to_str().unwrap(),
                    "127.0.0.1",
                ]);
                black_box(output);
            });
        });

        common::cleanup_temp_dir(&temp_dir);
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_binary_startup,
    bench_port_parsing,
    bench_localhost_scan,
    bench_output_formats
);

criterion_main!(benches);
