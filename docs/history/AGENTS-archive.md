# AGENTS.md archive

## Moved from AGENTS.md on 2026-10-07

Status snapshot sections that described the project as of v0.6.0 (Phase 6 active, docs synced 2025-11-17).
They were superseded by v1.0.0 (see `CLAUDE.md`, `Cargo.toml`, `docs/10-PROJECT-STATUS.md`) and are kept
here verbatim for history.

### Current Snapshot (verbatim)

- **Project**: ProRT-IP WarScan, a high-performance network scanner implemented in Rust.
- **Workspace version**: `0.6.0` (Phase 6 active; docs synced 2025-11-17).
- **Latest release**: v0.6.0 (Sprint 6.3 complete; performance O(N) connection state optimization, batch I/O defaults). **Tests:** 2,151 passing (100%). **Coverage:** 54.92%. **Artifacts:** 8/8 via `release.yml`.
- **Development phase**: Phase 6 (TUI + network optimizations) in progress — 3/8 sprints complete (37.5%). Phases 1–5 (plus 5.5) are complete.
- **CI health**: `ci.yml` green; tarpaulin coverage upload and Codecov wired. One flaky macOS job noted in CLAUDE.local; Linux/macOS coverage runs enabled.
- **Test surface**: Extensive unit coverage within crates; cross-crate and UI flows in `tests/`. Fuzz targets (5) with 230M+ execs, zero crashes.
- **Knowledge base**: Notion “ProRT-IP Knowledge Map” mirrors this doc; CLAUDE.local.md holds session-by-session deltas.

### Active Focus Areas (verbatim)

- Phase 6 Sprints:
  - **6.4 Zero-Copy Rework:** Extend O(N) connection-state gains with zero-copy integration across scanners; validate against perf/cgroup limits.
  - **6.5 Interactive Target Selection:** TUI-driven target selection, keyboard navigation refinements.
  - **6.6 TUI Polish & UX:** Tooltips, accessibility, theming, and export flows.
  - **6.7 Configuration Profiles:** Presets for speed/stealth/coverage.
  - **6.8 Help System & Tooltips:** Inline docs, context-aware hints.
- Preserve performance gains: rerun Phase 6 benchmark suites (CDN filtering, batch I/O) and archive results under `benchmarks/sprint-6.3-*`.
- Keep IPv6/NDP, idle/zombie, and rate limiting changes reflected in `docs/23-IPv6-GUIDE.md`, `docs/25-IDLE-SCAN-GUIDE.md`, and `docs/26-RATE-LIMITING-GUIDE.md`.
- Maintain Notion and archives: update Phase 6 tables in Notion and `docs/archive/PHASE-6-README-ARCHIVE.md` after each sprint.

### Immediate Action Items (verbatim)

- Sprint 6.4 kickoff: finalize zero-copy path validation and performance benches; ensure CI covers batch + zero-copy permutations.
- Benchmark + doc sync: propagate Sprint 6.3 results (O(N) connection state, adaptive batch) into `docs/07-PERFORMANCE.md`, README tables, and Notion metrics.
- Address macOS workflow flake noted in CLAUDE.local (scanner.initialize path); keep `ci.yml` green across matrices.
- Keep `/tmp/ProRT-IP/` scratch assets organized per CLAUDE.local.md (release notes, sprint decks) and clean up stragglers after merges.

### Outstanding Tracking (verbatim)

- Phase 6: Sprint 6.3 COMPLETE (O(N) connection state, adaptive batch I/O, CDN dedup). Next sprints 6.4–6.8 as listed above.
- Phase 5/5.5: COMPLETE. Archives in `docs/archive/PHASE-5-README-ARCHIVE.md`.
- Phase 4: COMPLETE (performance optimization). Archive in `docs/archive/PHASE-4-README-ARCHIVE.md`.
- Coverage + fuzz: 2,151 tests, 54.92% coverage; 230M+ fuzz executions, 0 crashes.

### Metrics Snapshot (verbatim)

| Metric | Current Value | Source |
| --- | --- | --- |
| Automated tests | 2,151 total (73 ignored) | README.md badge / CLAUDE.local.md |
| Coverage | 54.92% | README badge / tarpaulin CI |
| Latest release | v0.6.0 (Sprint 6.3 complete) | CHANGELOG.md / README.md |
| CI cadence | `ci.yml` ~3–6 min PR builds; coverage+Codecov enabled; minor macOS flake noted | `.github/workflows/ci.yml` |
| Documentation corpus | 50K+ lines (mdBook + archives; Phase 4–6 archives) | CLAUDE.local.md / docs/ |
| Active sprints | Phase 6 (3/8 complete; working on 6.4–6.8) | README.md Phase 6 roadmap |

Update the table above—and the mirrored Notion block “13. Metrics Snapshot”—any time a value shifts.
