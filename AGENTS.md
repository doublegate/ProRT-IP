# Repository Guidelines

## Current Snapshot

- **Project**: ProRT-IP WarScan, a high-performance network scanner implemented in Rust.
- **Workspace version**: `0.6.0` (Phase 6 active; docs synced 2025-11-17).
- **Latest release**: v0.6.0 (Sprint 6.3 complete; performance O(N) connection state optimization, batch I/O defaults). **Tests:** 2,151 passing (100%). **Coverage:** 54.92%. **Artifacts:** 8/8 via `release.yml`.
- **Development phase**: Phase 6 (TUI + network optimizations) in progress — 3/8 sprints complete (37.5%). Phases 1–5 (plus 5.5) are complete.
- **CI health**: `ci.yml` green; tarpaulin coverage upload and Codecov wired. One flaky macOS job noted in CLAUDE.local; Linux/macOS coverage runs enabled.
- **Test surface**: Extensive unit coverage within crates; cross-crate and UI flows in `tests/`. Fuzz targets (5) with 230M+ execs, zero crashes.
- **Knowledge base**: Notion “ProRT-IP Knowledge Map” mirrors this doc; CLAUDE.local.md holds session-by-session deltas.

## Active Focus Areas

- Phase 6 Sprints:
  - **6.4 Zero-Copy Rework:** Extend O(N) connection-state gains with zero-copy integration across scanners; validate against perf/cgroup limits.
  - **6.5 Interactive Target Selection:** TUI-driven target selection, keyboard navigation refinements.
  - **6.6 TUI Polish & UX:** Tooltips, accessibility, theming, and export flows.
  - **6.7 Configuration Profiles:** Presets for speed/stealth/coverage.
  - **6.8 Help System & Tooltips:** Inline docs, context-aware hints.
- Preserve performance gains: rerun Phase 6 benchmark suites (CDN filtering, batch I/O) and archive results under `benchmarks/sprint-6.3-*`.
- Keep IPv6/NDP, idle/zombie, and rate limiting changes reflected in `docs/23-IPv6-GUIDE.md`, `docs/25-IDLE-SCAN-GUIDE.md`, and `docs/26-RATE-LIMITING-GUIDE.md`.
- Maintain Notion and archives: update Phase 6 tables in Notion and `docs/archive/PHASE-6-README-ARCHIVE.md` after each sprint.

## Immediate Action Items

- Sprint 6.4 kickoff: finalize zero-copy path validation and performance benches; ensure CI covers batch + zero-copy permutations.
- Benchmark + doc sync: propagate Sprint 6.3 results (O(N) connection state, adaptive batch) into `docs/07-PERFORMANCE.md`, README tables, and Notion metrics.
- Address macOS workflow flake noted in CLAUDE.local (scanner.initialize path); keep `ci.yml` green across matrices.
- Keep `/tmp/ProRT-IP/` scratch assets organized per CLAUDE.local.md (release notes, sprint decks) and clean up stragglers after merges.

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

## Outstanding Tracking

- Phase 6: Sprint 6.3 COMPLETE (O(N) connection state, adaptive batch I/O, CDN dedup). Next sprints 6.4–6.8 as listed above.
- Phase 5/5.5: COMPLETE. Archives in `docs/archive/PHASE-5-README-ARCHIVE.md`.
- Phase 4: COMPLETE (performance optimization). Archive in `docs/archive/PHASE-4-README-ARCHIVE.md`.
- Coverage + fuzz: 2,151 tests, 54.92% coverage; 230M+ fuzz executions, 0 crashes.

## Metrics Snapshot

| Metric | Current Value | Source |
| --- | --- | --- |
| Automated tests | 2,151 total (73 ignored) | README.md badge / CLAUDE.local.md |
| Coverage | 54.92% | README badge / tarpaulin CI |
| Latest release | v0.6.0 (Sprint 6.3 complete) | CHANGELOG.md / README.md |
| CI cadence | `ci.yml` ~3–6 min PR builds; coverage+Codecov enabled; minor macOS flake noted | `.github/workflows/ci.yml` |
| Documentation corpus | 50K+ lines (mdBook + archives; Phase 4–6 archives) | CLAUDE.local.md / docs/ |
| Active sprints | Phase 6 (3/8 complete; working on 6.4–6.8) | README.md Phase 6 roadmap |

Update the table above—and the mirrored Notion block “13. Metrics Snapshot”—any time a value shifts.
