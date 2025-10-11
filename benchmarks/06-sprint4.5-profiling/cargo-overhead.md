| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `cargo run --release -- --scan-type syn -p 1-1000 127.0.0.1` | 163.7 ± 14.2 | 147.4 | 183.7 | 4.95 ± 1.12 |
| `./target/release/prtip --scan-type syn -p 1-1000 127.0.0.1` | 33.1 ± 6.9 | 24.7 | 39.0 | 1.00 |
