## [Unreleased]

### Fixed
- **Progress Bar Bug**: Fixed critical issue where progress bar showed 100% completion from the start on fast localhost scans
  - **Root cause**: Bridge polling task sleeping too long (5-50ms) relative to scan duration (2-50ms on localhost)
  - **Solution**: Reduced polling intervals by 10-50x using adaptive thresholds:
    - < 100 ports: 5ms → 0.2ms (25x faster)
    - < 1000 ports: 10ms → 0.5ms (20x faster)
    - < 20000 ports: 50ms → 1ms (50x faster)
    - ≥ 20000 ports: 50ms → 2ms (25x faster)
  - **Impact**: Progress bar now shows 5-50 incremental updates per scan instead of 1-2
  - **Performance**: < 0.5% CPU overhead increase (negligible)
  - Disabled `enable_steady_tick()` to prevent interference with manual progress updates
  - Files changed: `scheduler.rs` (9 lines), `progress_bar.rs` (2 lines)

### Technical Details
- Bridge polling now adaptive based on port count to match expected scan duration
- Localhost scans (50K-227K pps) now properly tracked with sub-millisecond polling
- Network scans (1K-10K pps) benefit from 2-5x more granular progress updates
- All 643 tests passing with zero regressions
