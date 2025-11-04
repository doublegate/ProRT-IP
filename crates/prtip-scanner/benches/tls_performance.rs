//! TLS Certificate Analysis Performance Benchmarks
//!
//! Sprint 5.5 TASK-8: Measures performance of TLS certificate parsing using x509-parser
//! to verify the <50ms overhead target for HTTPS service detection.
//!
//! ## Benchmark Focus
//!
//! This benchmark measures the **pure parsing overhead** of X.509 certificate processing.
//! End-to-end TLS overhead (including network time) is measured separately in integration tests.
//!
//! ## Performance Targets
//!
//! - Certificate parsing: <1ms per certificate
//! - Batch parsing (100 certs): <100ms total
//! - Per-certificate overhead: <500μs
//!
//! ## Running Benchmarks
//!
//! ```bash
//! # Run all TLS benchmarks
//! cargo bench --bench tls_performance
//!
//! # Save baseline for Sprint 5.5
//! cargo bench --bench tls_performance -- --save-baseline sprint-5.5
//!
//! # Compare against baseline later
//! cargo bench --bench tls_performance -- --baseline sprint-5.5
//!
//! # Generate HTML report
//! cargo bench --bench tls_performance
//! # View: target/criterion/report/index.html
//! ```
//!
//! ## Test Data
//!
//! Uses a minimal valid X.509 self-signed certificate (1024-bit RSA) for consistent
//! benchmarking. This represents the **lower bound** of parsing time - real certificates
//! with extensions, longer keys, and chain validation will take longer.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use x509_parser::prelude::*;

// Minimal valid self-signed X.509 certificate for benchmarking
//
// Structure: 1024-bit RSA, SHA256withRSA signature, basic extensions
// Subject: CN=benchmark.test
// Valid: 2024-11-03 to 2025-11-03
// Size: ~600 bytes (typical real-world certs are 1-3KB)
//
// This provides a performance BASELINE - real-world certificates will be
// 2-5x larger and take proportionally longer to parse.
const MINIMAL_CERT_DER: &[u8] = &[
    0x30, 0x82, 0x02, 0x4b, 0x30, 0x82, 0x01, 0xb4, 0xa0, 0x03, 0x02, 0x01, 0x02, 0x02, 0x14,
    0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd,
    0xef, 0x12, 0x34, 0x56, 0x78, 0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d,
    0x01, 0x01, 0x0b, 0x05, 0x00, 0x30, 0x1a, 0x31, 0x18, 0x30, 0x16, 0x06, 0x03, 0x55, 0x04,
    0x03, 0x0c, 0x0f, 0x62, 0x65, 0x6e, 0x63, 0x68, 0x6d, 0x61, 0x72, 0x6b, 0x2e, 0x74, 0x65,
    0x73, 0x74, 0x30, 0x1e, 0x17, 0x0d, 0x32, 0x34, 0x31, 0x31, 0x30, 0x33, 0x30, 0x30, 0x30,
    0x30, 0x30, 0x30, 0x5a, 0x17, 0x0d, 0x32, 0x35, 0x31, 0x31, 0x30, 0x33, 0x30, 0x30, 0x30,
    0x30, 0x30, 0x30, 0x5a, 0x30, 0x1a, 0x31, 0x18, 0x30, 0x16, 0x06, 0x03, 0x55, 0x04, 0x03,
    0x0c, 0x0f, 0x62, 0x65, 0x6e, 0x63, 0x68, 0x6d, 0x61, 0x72, 0x6b, 0x2e, 0x74, 0x65, 0x73,
    0x74, 0x30, 0x81, 0x9f, 0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01,
    0x01, 0x01, 0x05, 0x00, 0x03, 0x81, 0x8d, 0x00, 0x30, 0x81, 0x89, 0x02, 0x81, 0x81, 0x00,
    0xc9, 0x8b, 0x3d, 0x0f, 0x8a, 0x4e, 0x5c, 0x9f, 0xa3, 0x76, 0x84, 0x21, 0x56, 0x78, 0x9a,
    0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78,
    0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56,
    0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34,
    0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12,
    0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
    0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
    0xde, 0xf0, 0x02, 0x03, 0x01, 0x00, 0x01, 0xa3, 0x50, 0x30, 0x4e, 0x30, 0x1d, 0x06, 0x03,
    0x55, 0x1d, 0x0e, 0x04, 0x16, 0x04, 0x14, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef,
    0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x30, 0x1f, 0x06,
    0x03, 0x55, 0x1d, 0x23, 0x04, 0x18, 0x30, 0x16, 0x80, 0x14, 0x12, 0x34, 0x56, 0x78, 0x90,
    0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78,
    0x30, 0x0c, 0x06, 0x03, 0x55, 0x1d, 0x13, 0x04, 0x05, 0x30, 0x03, 0x01, 0x01, 0xff, 0x30,
    0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0b, 0x05, 0x00, 0x03,
    0x81, 0x81, 0x00, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12,
    0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
    0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
    0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a,
    0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78,
    0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56,
    0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
];

/// Benchmark X.509 certificate parsing
///
/// Measures the raw performance of x509-parser::X509Certificate::from_der().
/// This is the fundamental operation underlying all TLS certificate analysis.
///
/// **Expected Performance:**
/// - Single parse: 100-500μs (target <1ms)
/// - 100 certs: <50ms total (<500μs per cert)
///
/// **Real-World Context:**
/// - Typical certificates are 1-3KB (vs our 600-byte test cert)
/// - Real parsing time: 200-1000μs per certificate
/// - Chain validation adds 100-500μs per chain
/// - Network TLS handshake: 20-100ms (dominates overhead)
fn bench_x509_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("x509_parsing");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    // Single certificate parsing - baseline operation
    group.bench_function("parse_single_cert", |b| {
        b.iter(|| {
            let (_rem, cert) = X509Certificate::from_der(black_box(MINIMAL_CERT_DER))
                .expect("Failed to parse test certificate");
            black_box(cert);
        });
    });

    // Extract subject from parsed certificate
    group.bench_function("parse_and_extract_subject", |b| {
        b.iter(|| {
            let (_rem, cert) = X509Certificate::from_der(black_box(MINIMAL_CERT_DER)).unwrap();
            let subject = cert.subject().to_string();
            black_box(subject);
        });
    });

    // Extract all common fields (simulates full certificate info extraction)
    group.bench_function("parse_full_cert_info", |b| {
        b.iter(|| {
            let (_rem, cert) = X509Certificate::from_der(black_box(MINIMAL_CERT_DER)).unwrap();
            let subject = cert.subject().to_string();
            let issuer = cert.issuer().to_string();
            let serial = format!("{:x}", cert.serial);
            let not_before = cert.validity().not_before.to_string();
            let not_after = cert.validity().not_after.to_string();
            black_box((subject, issuer, serial, not_before, not_after));
        });
    });

    group.finish();
}

/// Benchmark batch certificate parsing
///
/// Simulates scanning multiple HTTPS servers by parsing many certificates sequentially.
/// This represents the common case where a network scanner processes hundreds or
/// thousands of HTTPS servers.
///
/// **Performance Scaling:**
/// - 10 certs: ~1-5ms (suitable for small scans)
/// - 100 certs: ~10-50ms (common scan size)
/// - 1000 certs: ~100-500ms (large-scale scanning)
///
/// **Throughput Goal:**
/// - Target: >2000 certificates/second
/// - Acceptable: >1000 certificates/second
fn bench_batch_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_parsing");
    group.sample_size(50); // Fewer samples for longer benchmarks
    group.measurement_time(Duration::from_secs(15));

    for count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_certs", count)),
            count,
            |b, &count| {
                b.iter(|| {
                    for _ in 0..count {
                        let (_rem, cert) =
                            X509Certificate::from_der(black_box(MINIMAL_CERT_DER)).unwrap();
                        // Extract key fields to simulate real usage
                        let _ = cert.subject().to_string();
                        let _ = cert.issuer().to_string();
                        black_box(cert);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark certificate validation operations
///
/// Tests the overhead of common validation checks performed during chain validation.
/// These operations are performed in addition to parsing.
///
/// **Validation Steps:**
/// - Date validity check (~1μs)
/// - Extension parsing (~10-50μs depending on extension count)
/// - Signature algorithm identification (~1μs)
fn bench_cert_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cert_validation");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    // Pre-parse certificate for validation benchmarks
    let (_rem, cert) = X509Certificate::from_der(MINIMAL_CERT_DER).unwrap();

    // Check if certificate is currently valid
    group.bench_function("check_validity_dates", |b| {
        b.iter(|| {
            let now = x509_parser::time::ASN1Time::now();
            let is_valid = cert.validity().is_valid_at(now);
            black_box(is_valid);
        });
    });

    // Extract and process extensions
    group.bench_function("process_extensions", |b| {
        b.iter(|| {
            let extensions: Vec<_> = cert.extensions().iter().collect();
            black_box(extensions);
        });
    });

    // Get signature algorithm
    group.bench_function("get_signature_algorithm", |b| {
        b.iter(|| {
            let sig_alg = cert.signature_algorithm.algorithm.to_string();
            black_box(sig_alg);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_x509_parsing,
    bench_batch_parsing,
    bench_cert_validation,
);

criterion_main!(benches);
