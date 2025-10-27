# Core Module Tests

## Test Organization

- `test_circuit_breaker.rs` - Circuit breaker tests (18 tests)
- `test_retry.rs` - Retry logic tests (14 tests)
- `test_resource_monitor.rs` - Resource monitoring tests (15 tests)

## Running Tests

```bash
# All core tests
cargo test --package prtip-core

# Circuit breaker only
cargo test --package prtip-core --test test_circuit_breaker

# Retry logic only
cargo test --package prtip-core --test test_retry

# Resource monitor only
cargo test --package prtip-core --test test_resource_monitor
```

## Test Coverage

Sprint 4.22 Phase 7 added 47 tests for error handling infrastructure:

- **Circuit Breaker (18 tests):**
  - State transitions (CLOSED → OPEN → HALF_OPEN → CLOSED)
  - Failure threshold detection (5 consecutive failures)
  - Cooldown period (30s timeout)
  - Per-target isolation

- **Retry Logic (14 tests):**
  - Max retry attempts (3)
  - Exponential backoff (1s → 2s → 4s)
  - Transient error detection
  - Permanent error handling
  - Timing templates (T0-T5)

- **Resource Monitor (15 tests):**
  - Memory threshold detection (80%)
  - File descriptor limits (90% of ulimit)
  - Graceful degradation
  - Alert generation
  - Configuration presets (conservative, aggressive, default)
