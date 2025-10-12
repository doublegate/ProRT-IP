# Repository Guidelines

## Current Snapshot

- **Project**: ProRT-IP WarScan, a high-performance network scanner implemented in Rust.
- **Workspace version**: `0.3.5` (`Cargo.toml:68`); keep crate versions aligned before cutting releases.
- **Development phase**: Phase 4 performance optimization is complete (see `CHANGELOG.md:508`); Phase 5 advanced feature work is next. `docs/01-ROADMAP.md` still lists Phase 4 as planned and needs a follow-up refresh.
- **CI health**: `ci.yml` workflow remains green; the README badge tracks 677 tests passing.
- **Test surface**: Fast unit coverage sits in each crate, while cross-crate scenarios live under `tests/`.

## Active Focus Areas

- Kick off Phase 5 by tackling idle scans, decoys, and fragmentation per the checklists in `docs/01-ROADMAP.md:360`.
- Draft the Lua plugin architecture early (interfaces, sandbox boundaries) to unblock `mlua` integration and example plugins.
- Preserve Phase 4 benchmark gains—use tooling in `benchmarks/` and `docs/07-PERFORMANCE.md` to guard regressions.
- Coordinate roadmap doc updates so Phase 4 completion and new priorities stay consistent across `ROADMAP.md` and the `docs/` set.

## Project Structure & Module Organization

This repository is a Rust workspace managed from `Cargo.toml`, with core functionality split across `crates/prtip-core`, packet and I/O logic in `crates/prtip-network`, scanning orchestration in `crates/prtip-scanner`, and the CLI packaged in `crates/prtip-cli`. Shared documentation and reference material live under `docs/`, while marketing assets reside in `images/` and `html/`. Use the `tests/` directory for integration scenarios that exercise multiple crates together.

## Build, Test & Development Commands

- `cargo build` compiles the full workspace; add `--release` for optimized output in `target/release/`.
- `cargo run -p prtip-cli -- --help` launches the CLI crate locally for quick smoke checks.
- `cargo fmt` and `cargo fmt --check` apply or verify canonical formatting across the workspace.
- `cargo clippy --workspace --all-targets -- -D warnings` enforces lint cleanliness; submissions must pass with zero warnings.
- `cargo test --workspace --all-targets` runs unit, integration, and doc tests; pair with `cargo test --doc` when updating public APIs.

## Coding Style & Naming Conventions

Code targets Rust 2021 with `rustfmt` defaults (four-space indentation, trailing commas for multiline literals). Prefer expressive `snake_case` for functions, `SCREAMING_SNAKE_CASE` for constants, and `CamelCase` for types and traits. Document all public items with `///` doc comments and include runnable examples where practical. Handle errors with `Result` or `anyhow::Result`; avoid `unwrap` outside controlled tests. When interacting with networking primitives, add brief comments explaining protocol assumptions.

## Testing Guidelines

Place fast unit tests alongside modules in `#[cfg(test)]` blocks, naming functions `test_*` for clarity. Cross-crate and async scenarios belong in `tests/` using `#[tokio::test]` where concurrency is required. New features should include coverage for failure paths and ensure doc examples compile. Before opening a PR, run the full clippy and test suite exactly as CI does: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace --all-targets`.

## Commit & Pull Request Guidelines

Follow Conventional Commits (`feat(scanner): add UDP probes`) so automation can categorize changes. Group related work into focused commits and keep branches current with `upstream/main` before review. PR descriptions should summarize motivation, list concrete changes, note testing performed, and link to any issues. Update `CHANGELOG.md` and relevant docs when behavior or user workflows change, and confirm the PR checklist in `CONTRIBUTING.md` before requesting review.

## Security & Operational Notes

This project ships offensive security tooling; only run scans against systems you are authorized to probe. Adhere to `docs/08-SECURITY.md` for privilege dropping, safe packet parsing, and rate limiting expectations when touching network-facing code. Verify that contributions remain cross-platform and respect resource limits to avoid introducing denial-of-service regressions.

## Helpful References

- `ROADMAP.md` for a high-level milestone summary and current status.
- `docs/01-ROADMAP.md` for sprint-level tasks, acceptance criteria, and performance targets (refresh pending for Phase 4 completion).
- `docs/08-SECURITY.md` for mandatory operational controls.
- `docs/07-PERFORMANCE.md` and `benchmarks/` for Phase 4 profiling output and harnesses.
- `scripts/` and `benchmarks/` for profiling helpers and reproducible performance experiments.
