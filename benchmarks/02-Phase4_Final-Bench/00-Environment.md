# Phase 4 Final Benchmark - Environment Configuration

**Date:** 2025-10-12
**Version:** v0.3.5
**Benchmark Suite:** Phase 4 Final (Post-Nmap Compatibility)

## System Information

Linux AB-i9 6.17.1-2-cachyos #1 SMP PREEMPT_DYNAMIC Mon, 06 Oct 2025 23:26:58 +0000 x86_64 GNU/Linux

## CPU Information

Architecture:                            x86_64
CPU(s):                                  20
Model name:                              Intel(R) Core(TM) i9-10850K CPU @ 3.60GHz
Thread(s) per core:                      2
Core(s) per socket:                      10
Socket(s):                               1
CPU(s) scaling MHz:                      77%

## Memory Information

               total        used        free      shared  buff/cache   available
Mem:            62Gi       5.7Gi        38Gi       573Mi        19Gi        56Gi
Swap:          126Gi          0B       126Gi

## Software Versions

- Rust: rustc 1.90.0 (1159e78c4 2025-09-14)
- Cargo: cargo 1.90.0 (840b83a10 2025-07-30)
- Hyperfine: hyperfine 1.19.0
- Perf: perf version 6.16-3
- Valgrind: valgrind-3.25.1
- Flamegraph: flamegraph 0.6.9

## Build Configuration

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
opt-level = 0
debug = true


## Target Binary

-rwxr-xr-x 2 parobek parobek 8.4M Oct 12 05:44 target/release/prtip

target/release/prtip: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, for GNU/Linux 6.1.0, BuildID[sha1]=c44d929bc79029281a1ec1e6bddc06486f48fb7c, stripped
