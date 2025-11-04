# Repository Guidelines

## Current Snapshot

- **Project**: ProRT-IP WarScan, a high-performance network scanner implemented in Rust.
- **Workspace version**: `0.3.8` (`Cargo.toml:68`); roadmap and release docs are synced to the 2025-10-13 tag.
- **Development phase**: Phase 4 performance optimization is complete (Sprints 4.1–4.19). Phase 4 stretch work (4.18 output expansion, 4.19 stealth/NUMA validation) is active ahead of the Phase 5 feature push.
- **CI health**: `ci.yml` workflow green with 803 tests passing (see `README.md` badge and Project Status). NUMA feature flags introduced in Sprint 4.19 have unit coverage across core crates.
- **Test surface**: Fast unit coverage sits in each crate, while cross-crate scenarios live under `tests/`.
- **Knowledge base**: Notion source of truth—“ProRT-IP Knowledge Map”—mirrors this document; keep both in lockstep during updates.

## Active Focus Areas

- Sprint 4.18: finish PCAPNG writer, SQLite streaming exporter, validation harnesses, and documentation in `docs/20-SPRINT-4.18-DEFERRED.md`.
- Sprint 4.19 Phase 2: integrate the NUMA manager into the scheduler, extend zero-copy to decoy and OS probe scanners, capture benchmarks, and document usage in `docs/07-PERFORMANCE.md` and `docs/15-PLATFORM-SUPPORT.md`.
- Preserve Phase 4 benchmark gains—rerun suites in `benchmarks/01-Phase4_PreFinal-Bench/` and keep `docs/07-PERFORMANCE.md` tables current after every NUMA/output change.
- Prepare Phase 5 foundations: idle scan design notes, decoy configuration UX, Lua plugin architecture sketches, and audit logging roadmap (see `docs/19-PHASE4-ENHANCEMENTS.md`).
- Update external comms (README badges, CLAUDE memory, marketing decks) whenever metrics change to avoid drift with the Notion map.

## Immediate Action Items

- Wrap Sprint 4.18 deliverables: implement/export PCAPNG writer, finalize SQLite streaming output, and update docs/tests accordingly.
- Validate Sprint 4.19 Phase 1 NUMA changes on multi-socket hardware (or documented simulation) and record metrics in `docs/07-PERFORMANCE.md`.
- Sync auxiliary guides (`CLAUDE.md`, marketing decks) with the 803-test count, Phase 4 completion messaging, and new NUMA flags.
- Plan Phase 5 kickoff doc updates (idle scan spec, plugin API outline) once Sprint 4.18/4.19 artifacts merge.

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
- `docs/01-ROADMAP.md` for sprint-level tasks, acceptance criteria, and performance targets (Phase 4 logged, Phase 5 planning in progress).
- `docs/08-SECURITY.md` for mandatory operational controls.
- `docs/07-PERFORMANCE.md` and `benchmarks/` for Phase 4 profiling output and harnesses.
- `scripts/` and `benchmarks/` for profiling helpers and reproducible performance experiments.

## Notion Knowledge Map Maintenance

- Keep the “ProRT-IP Knowledge Map” (Section 13 metrics, Section 16 Phase Timeline, Section 17 Future Development Breakdown) in sync with this document after every status change.
- Ensure Section 16 retains the single consolidated Phase Timeline table (Phase/Status/Focus/Timeline/Key Notes). Archive any duplicate tables if they reappear.
- Update Section 17’s Future Development Breakdown table when sprint scopes shift, mirroring active sprint objectives and dependencies.

## Metrics Snapshot

| Metric | Current Value | Source |
| --- | --- | --- |
| Automated tests | 803 total (70 integration) | README.md: Project Statistics |
| Coverage | 61.92% (15,397 / 24,814 lines) | README.md: Project Statistics |
| Latest release | v0.3.8 (tagged 2025-10-13) | CHANGELOG.md / GitHub Releases |
| CI cadence | `ci.yml` 3–6 min PR builds; `release.yml` publishes 8 artifacts | `.github/workflows/ci.yml` |
| Documentation corpus | ~600 KB across 307 files (`docs/`, `benchmarks/`, `bug_fix/`) | README.md: Documentation |
| Active sprints | 4.18 output expansion, 4.19 Phase 2 NUMA validation | README.md: Phase 4 Summary |

Update the table above—and the mirrored Notion block “13. Metrics Snapshot”—any time a value shifts.
