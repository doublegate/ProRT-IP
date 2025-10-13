# Test Fixtures

This directory contains test data files used by integration tests.

## Files

### sample_targets.json
Sample network targets for testing:
- Localhost (IPv4 and IPv6)
- Expected port states
- Common port ranges

### nmap_compatible_flags.json
Comprehensive list of nmap-compatible flags for CLI testing:
- Scan types (-sS, -sT, -sU, etc.)
- Port specifications (-p, -F, --top-ports)
- Output formats (-oN, -oX, -oG)
- Timing templates (-T0 through -T5)
- Detection options (-sV, -O, -A)

### expected_outputs.json
Expected output patterns for validation:
- Text output format patterns
- JSON schema requirements
- XML output patterns
- Greppable output format
- Valid port states

## Usage

Load fixtures in tests using the common utilities:

```rust
use crate::common::{load_fixture, load_json_fixture};

// Load as string
let content = load_fixture("sample_targets.json");

// Load and parse JSON
let targets = load_json_fixture("sample_targets.json");
let localhost_ip = targets["targets"]["localhost"]["ip"]
    .as_str()
    .unwrap();
```

## Adding New Fixtures

1. Create JSON file with descriptive name
2. Include a "comment" field explaining purpose
3. Use consistent structure with existing fixtures
4. Update this README with description
5. Add validation test if needed
