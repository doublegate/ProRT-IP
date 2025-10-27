# CLI Integration Tests

## Test Organization

- `test_error_messages.rs` - Error message validation (20 tests)
- `test_error_integration.rs` - End-to-end error handling (15 tests)
- `test_edge_cases.rs` - Boundary conditions (18 tests)
- `test_cli_args.rs` - CLI argument parsing and validation
- `integration_tests.rs` - Full CLI integration scenarios

## Running Tests

```bash
# All CLI tests
cargo test --package prtip-cli

# Error messages only
cargo test --package prtip-cli --test test_error_messages

# Error integration only
cargo test --package prtip-cli --test test_error_integration

# Edge cases only
cargo test --package prtip-cli --test test_edge_cases
```

## Integration Tests

Integration tests use `Command` to spawn the CLI binary and verify:
- Exit codes (0=success, 1=error)
- Error message formatting
- User-facing clarity
- Recovery suggestions

## Test Coverage

Sprint 4.22 Phase 7 added 53 tests for error handling:

- **Error Messages (20 tests):**
  - Network errors (5 tests): unreachable, timeout, refused, reset
  - Permission errors (3 tests): raw socket, privileges, resource limits
  - Input validation (5 tests): invalid config, rate limits, probe failures
  - Configuration errors (4 tests): conflicting options, invalid targets
  - Error categories (3 tests): category classification, retriability

- **Error Integration (15 tests):**
  - Input validation (4 tests): invalid IP, port range, CIDR notation
  - Permission handling (2 tests): SYN/FIN without root
  - Network failures (3 tests): unreachable hosts, connection timeout
  - Configuration errors (4 tests): conflicting scan types, invalid timing
  - Exit codes (2 tests): success/failure code verification

- **Edge Cases (18 tests):**
  - Port boundaries (6 tests): port 0, 65535, 65536, ranges
  - CIDR extremes (6 tests): /0, /31, /32, /33 networks
  - Timeouts (3 tests): zero, negative, very large
  - Parallelism (3 tests): zero, 1, very large (10000+)
