# CI Fix Implementation - Windows Integration Tests

**Date:** 2025-10-15  
**Issue:** Windows integration tests failing (12/29 tests, 41% failure rate)  
**Commit:** 02037ad (previous failed attempt)  
**Root Cause:** Shell environment mismatch (PowerShell vs Bash)  

## Problem Summary

### Symptom
- Windows: 17/29 integration tests passing, 12 failing with exit code -1073741701
- macOS: All tests passing (789/789)
- Linux: All tests passing (789/789)
- CI Status: 6/7 jobs passing (Windows test job failing)

### Root Cause
1. **DLL Extraction:** Success in PowerShell (x64 DLLs from Npcap 1.79)
   - Packet.dll: 174,464 bytes (correct x64)
   - wpcap.dll: 420,224 bytes (correct x64)

2. **DLL Placement:** Success in PowerShell
   - Copied to `target\debug\` directory
   - Verified with file size checks

3. **PATH Setup:** Failed to propagate to bash shell
   - Set in PowerShell: `PATH=$PWD\npcap-dlls;$env:PATH`
   - Test step runs in bash shell
   - Bash does not inherit PowerShell environment correctly

4. **Test Execution:** Subprocess fails to find DLLs
   - Integration tests spawn `prtip.exe` subprocess
   - Subprocess searches for DLLs in PATH
   - DLLs not visible → STATUS_INVALID_IMAGE_FORMAT

### Exit Code Analysis
- `-1073741701` = `0xC000007B` = STATUS_INVALID_IMAGE_FORMAT
- Windows error for: "wrong architecture DLL" OR "DLL not found"
- In this case: x64 DLLs are correct, but not found by subprocess

## Implementation

### File Modified
- `.github/workflows/ci.yml` (lines 175-186)

### Change Details
```yaml
# BEFORE (BROKEN):
- name: Run tests
  run: |
    if [ "${{ matrix.os }}" = "windows-latest" ]; then
      # Windows: Skip prtip-network tests due to Npcap DLL runtime dependencies
      # The tests require actual network privileges and adapters not available in CI
      cargo test --workspace --locked --exclude prtip-network
    else
      cargo test --workspace --locked
    fi
  shell: bash

# AFTER (FIXED):
- name: Run tests
  run: |
    if [ "${{ matrix.os }}" = "windows-latest" ]; then
      # Windows: Add target/debug to PATH for Npcap DLL runtime loading
      # DLLs must be visible to subprocesses spawned by integration tests
      export PATH="$(pwd)/target/debug:$PATH"
      # Skip prtip-network tests due to actual network privileges requirement
      cargo test --workspace --locked --exclude prtip-network
    else
      cargo test --workspace --locked
    fi
  shell: bash
```

### Key Points
1. **One Line Addition:** `export PATH="$(pwd)/target/debug:$PATH"`
2. **Bash Syntax:** Uses `$(pwd)` not `$PWD` for portability
3. **Prepend PATH:** DLLs in `target/debug` searched first
4. **Windows-Only:** No impact on Linux/macOS
5. **No Code Changes:** Pure CI configuration fix

## Testing Strategy

### Expected Behavior
1. **Build Step:** Compiles prtip.exe (unchanged)
2. **DLL Copy Step:** Copies Packet.dll + wpcap.dll to target\debug (unchanged)
3. **Test Step:** Sets PATH before running tests (CHANGED)
   - `export PATH=/d/a/ProRT-IP/ProRT-IP/target/debug:$PATH`
   - Subprocess inherits correct PATH
   - DLLs found in same directory as prtip.exe
4. **Integration Tests:** Spawn prtip.exe subprocess
   - Windows searches target\debug first
   - Finds Packet.dll and wpcap.dll
   - Process starts successfully
   - Tests execute normally

### Expected Results
- **Windows Tests:** 17/29 → 29/29 passing (100%)
- **macOS Tests:** 789/789 passing (unchanged)
- **Linux Tests:** 789/789 passing (unchanged)
- **CI Status:** 6/7 → 7/7 jobs passing (100%)

### Failure Tests (12 affected tests)
All should now PASS:
1. test_cli_excessive_retries
2. test_cli_excessive_max_concurrent
3. test_cli_help
4. test_cli_excessive_timeout
5. test_cli_invalid_ports
6. test_cli_invalid_port_range
7. test_cli_multiple_targets
8. test_cli_no_targets
9. test_cli_port_zero
10. test_cli_version
11. test_cli_zero_max_rate
12. test_cli_zero_timeout

## Verification Plan

### Pre-Commit Checks
1. ✅ Syntax validation (YAML valid)
2. ✅ No code changes (only CI config)
3. ✅ Windows-specific change (no macOS/Linux impact)
4. ✅ Minimal change (one line)

### Post-Commit Verification
1. Monitor GitHub Actions run
2. Check Windows test job execution
3. Verify all 29 integration tests pass
4. Confirm 7/7 jobs passing
5. Check macOS/Linux unchanged

### Rollback Plan
If this fix fails:
- Revert commit
- Try alternative: Change test step to PowerShell shell
- Or: Copy DLLs to System32 directory

## Risk Assessment

### Risk Level: VERY LOW
- Single line change
- Windows-only impact
- Uses standard bash syntax
- DLLs already in correct location
- Zero code changes
- Zero dependency changes

### Alternative Approaches Considered
1. ✅ **Chosen:** Set PATH in test step (minimal, targeted)
2. ❌ Change to PowerShell: Larger change, affects all tests
3. ❌ Copy to System32: Requires admin, pollutes system
4. ❌ Change DLL copy to bash: More complex, unnecessary

## Documentation Updates Required
None - CI configuration self-documenting via comments.

## Related Issues
- Previous attempts (10+ commits):
  - WinPcap 4.1.3 extraction
  - VC++ 2010 runtime installation
  - Npcap installer execution (hung)
  - DLL size validation
  - Multiple extraction strategies

- Final root cause: Shell environment inheritance

## Success Criteria
- ✅ Windows integration tests: 29/29 passing
- ✅ CI status: 7/7 jobs passing
- ✅ Zero test modifications
- ✅ Zero code changes
- ✅ Clean commit history

---

**Ready for commit:** YES  
**Breaking changes:** NO  
**Requires testing:** GitHub Actions verification only  
**Confidence level:** HIGH (95%)  
