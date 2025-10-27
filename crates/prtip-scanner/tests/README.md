# Scanner Integration Tests

## Test Organization

- `common/` - Shared test utilities
  - `error_injection.rs` - Failure simulation framework
- `test_error_injection.rs` - Error injection tests (22 tests)
- `integration_scanner.rs` - Scanner integration tests
- `integration_pcapng.rs` - PCAPNG capture tests
- `test_source_port.rs` - Source port manipulation tests

## Running Tests

```bash
# All scanner tests
cargo test --package prtip-scanner

# Specific test file
cargo test --package prtip-scanner --test test_error_injection

# Single test
cargo test --package prtip-scanner --test test_error_injection test_timeout_error_conversion
```

## Error Injection Framework

The error injection framework (`common/error_injection.rs`) provides deterministic failure simulation.

See Sprint 4.22 Phase 7 documentation for details.
