Create new Rust module with boilerplate and integration: $*

---

## MODULE CREATION WORKFLOW

**Purpose:** Generate new Rust module with comprehensive boilerplate, tests, documentation, and integration into lib.rs

**Usage:** `/module-create <crate> <module-name> <description>`
- **crate:** Target crate (core, network, scanner, cli)
- **module-name:** Module filename (e.g., "packet_fragmentation", "zombie_scanner")
- **description:** Brief module description

**Example:** `/module-create scanner idle_scanner "Idle/zombie scanning for ultimate anonymity"`

---

## Phase 1: VALIDATE PARAMETERS AND CRATE STRUCTURE

**Objective:** Ensure parameters are valid and target crate exists

### Step 1.1: Parse Arguments

```bash
CRATE="$1"
MODULE_NAME="$2"
DESCRIPTION="${@:3}"

if [ -z "$CRATE" ] || [ -z "$MODULE_NAME" ] || [ -z "$DESCRIPTION" ]; then
  echo "ERROR: Missing required parameters"
  echo "Usage: /module-create <crate> <module-name> <description>"
  echo "Example: /module-create scanner idle_scanner \"Idle scanning implementation\""
  exit 1
fi
```

### Step 1.2: Validate Crate Name

```bash
VALID_CRATES=("core" "network" "scanner" "cli")

if [[ ! " ${VALID_CRATES[@]} " =~ " ${CRATE} " ]]; then
  echo "ERROR: Invalid crate '$CRATE'"
  echo "Valid crates: core, network, scanner, cli"
  exit 1
fi
```

### Step 1.3: Determine Crate Directory

```bash
CRATE_DIR="crates/prtip-${CRATE}"

if [ ! -d "$CRATE_DIR" ]; then
  echo "ERROR: Crate directory not found: $CRATE_DIR"
  exit 1
fi

MODULE_PATH="${CRATE_DIR}/src/${MODULE_NAME}.rs"
```

### Step 1.4: Check for Existing Module

```bash
if [ -f "$MODULE_PATH" ]; then
  echo "WARNING: Module already exists: $MODULE_PATH"
  read -p "Overwrite? (y/n) " -n 1 -r
  echo ""
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
  fi
fi
```

---

## Phase 2: GENERATE MODULE BOILERPLATE

**Objective:** Create comprehensive module file with documentation, imports, structure, tests

### Step 2.1: Determine Module Category

**Categories Based on Crate:**
- **core:** Data types, configuration, utilities
- **network:** Packet handling, socket operations
- **scanner:** Scanning algorithms, detection engines
- **cli:** User interface, output formatting

### Step 2.2: Generate Module File

**Template:** `$MODULE_PATH`

**Module Structure (~200 lines):**

```rust
//! $DESCRIPTION
//!
//! This module provides [detailed description of functionality].
//!
//! # Features
//!
//! - [Feature 1]
//! - [Feature 2]
//! - [Feature 3]
//!
//! # Architecture
//!
//! [High-level architecture description]
//!
//! # Examples
//!
//! ```rust
//! use prtip_${CRATE}::${MODULE_NAME}::*;
//!
//! // Example usage
//! let example = ExampleStruct::new();
//! ```
//!
//! # Performance
//!
//! [Performance characteristics, if applicable]
//!
//! # References
//!
//! - [Link to external documentation]
//! - [Link to RFC/specification]

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

// Crate-specific imports based on category
[Auto-generate based on crate type]

/// [Brief description of main struct/type]
///
/// # Fields
///
/// - `field1` - [Description]
/// - `field2` - [Description]
#[derive(Debug, Clone)]
pub struct ExampleStruct {
    /// [Field documentation]
    field1: String,
    /// [Field documentation]
    field2: usize,
}

impl ExampleStruct {
    /// Creates a new instance
    ///
    /// # Arguments
    ///
    /// * `param1` - [Description]
    ///
    /// # Returns
    ///
    /// Returns a new `ExampleStruct` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// let example = ExampleStruct::new("test".to_string());
    /// ```
    pub fn new(param1: String) -> Self {
        Self {
            field1: param1,
            field2: 0,
        }
    }

    /// [Method description]
    ///
    /// # Arguments
    ///
    /// * `arg1` - [Description]
    ///
    /// # Returns
    ///
    /// Returns [description]
    ///
    /// # Errors
    ///
    /// Returns error if [condition]
    pub fn example_method(&self, arg1: usize) -> Result<(), String> {
        debug!("Executing example_method with arg1={}", arg1);

        // Implementation placeholder
        Ok(())
    }
}

impl Default for ExampleStruct {
    fn default() -> Self {
        Self::new(String::new())
    }
}

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let example = ExampleStruct::new("test".to_string());
        assert_eq!(example.field1, "test");
        assert_eq!(example.field2, 0);
    }

    #[test]
    fn test_default() {
        let example = ExampleStruct::default();
        assert_eq!(example.field1, "");
        assert_eq!(example.field2, 0);
    }

    #[test]
    fn test_example_method() {
        let example = ExampleStruct::new("test".to_string());
        let result = example.example_method(42);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_functionality() {
        // Async test template
        let example = ExampleStruct::new("async".to_string());
        assert!(example.example_method(1).is_ok());
    }
}
```

### Step 2.3: Customize Imports Based on Crate

**prtip-core:**
```rust
use crate::types::*;
use crate::config::*;
```

**prtip-network:**
```rust
use pnet::packet::{Packet, MutablePacket};
use socket2::{Socket, Domain, Type, Protocol};
```

**prtip-scanner:**
```rust
use prtip_core::types::*;
use prtip_network::*;
use tokio::task::JoinHandle;
use futures::stream::{FuturesUnordered, StreamExt};
```

**prtip-cli:**
```rust
use clap::Parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
```

---

## Phase 3: INTEGRATE INTO LIB.RS

**Objective:** Add module declaration and public exports to lib.rs

### Step 3.1: Read Current lib.rs

```bash
LIB_RS="${CRATE_DIR}/src/lib.rs"

if [ ! -f "$LIB_RS" ]; then
  echo "ERROR: lib.rs not found: $LIB_RS"
  exit 1
fi
```

### Step 3.2: Add Module Declaration

**Insertion Point:** After existing module declarations

**Format:**
```rust
/// $DESCRIPTION
pub mod ${MODULE_NAME};
```

**Implementation:**
```bash
# Find last "pub mod" line
LAST_MOD_LINE=$(grep -n "^pub mod" "$LIB_RS" | tail -1 | cut -d: -f1)

# Insert new module declaration after last mod
sed -i "${LAST_MOD_LINE}a\\/// $DESCRIPTION\\npub mod ${MODULE_NAME};" "$LIB_RS"
```

### Step 3.3: Add Public Re-exports (if applicable)

**For core types/structs:**
```rust
pub use ${MODULE_NAME}::{ExampleStruct, ExampleEnum};
```

### Step 3.4: Verify lib.rs Syntax

```bash
echo "Verifying lib.rs syntax..."
cargo check --package prtip-${CRATE}

if [ $? -ne 0 ]; then
  echo "ERROR: lib.rs syntax error after integration"
  exit 1
fi

echo "✅ lib.rs integration successful"
```

---

## Phase 4: GENERATE INTEGRATION GUIDE

**Objective:** Create comprehensive guide for using the new module

### Step 4.1: Create Integration Guide

**Template Location:** `/tmp/ProRT-IP/${MODULE_NAME}-integration-guide.md`

**Document Structure:**

```markdown
# $MODULE_NAME Integration Guide

**Module:** prtip-${CRATE}::${MODULE_NAME}
**Description:** $DESCRIPTION
**Created:** $(date +%Y-%m-%d)

## Overview

[Auto-generated overview based on module category]

## Module Location

**File:** `${MODULE_PATH}`
**Lines:** [Count lines in generated file]
**Tests:** [Count test functions]

## Public API

### Structs

#### ExampleStruct

**Purpose:** [Auto-generated description]

**Fields:**
- `field1: String` - [Description]
- `field2: usize` - [Description]

**Methods:**
- `new(param1: String) -> Self` - Constructor
- `example_method(&self, arg1: usize) -> Result<(), String>` - [Method description]

**Example Usage:**
```rust
use prtip_${CRATE}::${MODULE_NAME}::ExampleStruct;

let example = ExampleStruct::new("test".to_string());
let result = example.example_method(42)?;
```

## Integration Points

### Where to Use This Module

[Auto-generated suggestions based on crate type]

**For prtip-scanner modules:**
- Import in `scheduler.rs` for scan orchestration
- Use in scan type implementations (tcp_connect.rs, syn_scanner.rs, etc.)

**For prtip-network modules:**
- Import in scanner modules for packet operations
- Use in network-level scan implementations

**For prtip-core modules:**
- Import across all crates for shared types/config
- Use in configuration parsing and validation

**For prtip-cli modules:**
- Import in `main.rs` for user interface
- Use in output formatting and progress tracking

## Testing Strategy

**Unit Tests:** [Count] tests included in module
**Integration Tests:** Recommendations for integration testing

**Run Tests:**
```bash
cargo test --package prtip-${CRATE} ${MODULE_NAME}
```

## Performance Considerations

[Auto-generated performance guidance based on module type]

## Next Steps

1. **Implement Core Functionality:**
   - Replace placeholder implementation in `example_method()`
   - Add additional methods as needed
   - Document complex logic with inline comments

2. **Add Comprehensive Tests:**
   - Increase test coverage to >90%
   - Add edge case tests (empty input, invalid data, etc.)
   - Add async tests if module uses async operations

3. **Integration:**
   - Import module in relevant scanner/network files
   - Wire into existing workflows (scheduler, scanners, etc.)
   - Update configuration if new parameters needed

4. **Documentation:**
   - Add usage examples to README.md
   - Update ARCHITECTURE.md with new component
   - Add API documentation to docs/05-API-REFERENCE.md

5. **Benchmarking:**
   - Create benchmark scenarios if performance-critical
   - Compare performance vs existing implementations
   - Document performance characteristics

## References

- [Link to relevant RFCs/specifications]
- [Link to similar implementations (Nmap, Masscan, etc.)]
- [Link to documentation]

---

**Generated:** $(date)
```

---

## Phase 5: DISPLAY SUMMARY AND NEXT STEPS

**Objective:** Provide comprehensive creation summary

### Step 5.1: Count Generated Code

```bash
MODULE_LINES=$(wc -l < "$MODULE_PATH")
TEST_COUNT=$(grep -c "#\[test\]" "$MODULE_PATH")
ASYNC_TEST_COUNT=$(grep -c "#\[tokio::test\]" "$MODULE_PATH")
TOTAL_TESTS=$((TEST_COUNT + ASYNC_TEST_COUNT))
```

### Step 5.2: Display Summary

```bash
echo "=========================================="
echo "Module Created Successfully"
echo "=========================================="
echo ""
echo "📦 MODULE DETAILS"
echo "  Crate: prtip-${CRATE}"
echo "  Module: ${MODULE_NAME}"
echo "  Description: ${DESCRIPTION}"
echo ""
echo "📝 GENERATED FILES"
echo "  ✅ ${MODULE_PATH} (${MODULE_LINES} lines)"
echo "  ✅ ${LIB_RS} (updated with module declaration)"
echo "  ✅ /tmp/ProRT-IP/${MODULE_NAME}-integration-guide.md"
echo ""
echo "🧪 TESTS"
echo "  Unit Tests: ${TEST_COUNT}"
echo "  Async Tests: ${ASYNC_TEST_COUNT}"
echo "  Total Tests: ${TOTAL_TESTS}"
echo ""
echo "🚀 NEXT STEPS"
echo "  1. Review generated module: ${MODULE_PATH}"
echo "  2. Implement core functionality (replace placeholders)"
echo "  3. Add comprehensive tests (target >90% coverage)"
echo "  4. Run tests: cargo test --package prtip-${CRATE} ${MODULE_NAME}"
echo "  5. Integrate into existing workflows (see integration guide)"
echo "  6. Update documentation (README, ARCHITECTURE, API-REFERENCE)"
echo ""
echo "📚 INTEGRATION GUIDE"
echo "  Location: /tmp/ProRT-IP/${MODULE_NAME}-integration-guide.md"
echo "  Read this for detailed integration instructions"
echo ""
```

---

## SUCCESS CRITERIA

✅ Parameters validated (crate, module name, description)
✅ Module file generated with comprehensive boilerplate (~200 lines)
✅ lib.rs updated with module declaration
✅ Integration guide created with usage examples
✅ Syntax verified (cargo check passes)
✅ Summary displayed with next steps

---

## DELIVERABLES

1. **Module File:** `${MODULE_PATH}` (~200 lines, 4 tests)
2. **Updated lib.rs:** Module declaration added
3. **Integration Guide:** Comprehensive usage documentation
4. **Summary Report:** Console output with next steps

---

**Create module: $***
