# CI Root Cause Analysis - Windows Integration Test Failures (UPDATED)

**Analysis Date:** 2025-10-15  
**Run ID:** 18517056169  
**Commit:** 02037ad  
**Status:** WINDOWS FAILS (12/29 tests), macOS PASSES, Linux PASSES

## CRITICAL FINDING: Shell Environment Mismatch

**ROOT CAUSE IDENTIFIED:** The DLLs are extracted and copied successfully in PowerShell (pwsh), but the tests run in bash shell. The PATH environment variable set in PowerShell is NOT properly inherited by bash, causing the integration tests to fail with STATUS_INVALID_IMAGE_FORMAT.

## Evidence

### 1. DLL Extraction SUCCESS (PowerShell)
```
Install Npcap SDK step (shell: pwsh):
- Extracted x64 DLLs: Packet.dll (174,464 bytes), wpcap.dll (420,224 bytes)
- Copied from: D:\a\ProRT-IP\ProRT-IP\npcap-runtime\x64
- Set PATH: echo "PATH=$PWD\npcap-dlls;$env:PATH" >> $env:GITHUB_ENV
```

### 2. DLL Copy SUCCESS (PowerShell)  
```
Copy Npcap DLLs step (shell: pwsh):
- Source DLLs verified in npcap-dlls\
- Copied to target\debug\
- Verified in target\debug: Packet.dll (174,464), wpcap.dll (420,224)
```

### 3. Test Execution FAILURE (Bash)
```yaml
# Line 175-184 in ci.yml
- name: Run tests
  run: |
    if [ "${{ matrix.os }}" = "windows-latest" ]; then
      cargo test --workspace --locked --exclude prtip-network
    else
      cargo test --workspace --locked
    fi
  shell: bash  # <-- SHELL MISMATCH!
```

**Result:** bash shell does not see the DLLs in PATH, causing:
- Exit code: -1073741701 (STATUS_INVALID_IMAGE_FORMAT)
- Empty stderr (process crashes before output)
- Affects only tests that spawn prtip.exe subprocess

### 4. Test Failure Pattern

**PASS (no subprocess):**
- Unit tests: 97 + 125 tests (compile-time only)
- Integration tests without prtip.exe spawn: 17/29

**FAIL (subprocess spawn):**
- test_cli_help: `prtip.exe --help` (-1073741701)
- test_cli_version: `prtip.exe --version` (-1073741701)
- 10 other CLI validation tests: empty stderr

## Root Cause Analysis

### Issue 1: PowerShell $PWD vs Bash $PWD
PowerShell uses Windows-style paths:
```
PATH=D:\a\ProRT-IP\ProRT-IP\npcap-dlls;...
```

Bash on Windows uses POSIX-style paths:
```
PATH=/d/a/ProRT-IP/ProRT-IP/npcap-dlls:...
```

The PATH set in PowerShell may not translate correctly to bash.

### Issue 2: GITHUB_ENV Persistence
GitHub Actions GITHUB_ENV file SHOULD persist across steps, but there may be:
- Path translation issues (Windows \ vs Unix /)
- Environment variable inheritance differences between shells
- Timing issues (GITHUB_ENV not flushed before bash step)

### Issue 3: DLL Search Order
Windows DLL search order:
1. Application directory (target\debug\) - DLLs ARE here ✅
2. System32 directory - DLLs NOT here
3. PATH directories - May not be visible to bash subprocess

The integration tests spawn prtip.exe as a subprocess. If cargo test runs under bash, the spawned subprocess may not inherit the correct environment.

## Solution Options

### Option 1: Use Bash for Both DLL Setup and Tests (RECOMMENDED)
Change the DLL copy step to use bash shell to ensure environment consistency.

### Option 2: Use PowerShell for Tests
Change the test step to use PowerShell shell.

### Option 3: Copy DLLs to System32
Copy DLLs to C:\Windows\System32 which is always in DLL search path.

### Option 4: Set PATH in Test Step Directly
Explicitly set PATH in the test step using bash syntax.

## Recommended Fix (Option 4 - Minimal Change)

```yaml
- name: Run tests
  run: |
    # Add DLL directory to PATH for Windows (bash syntax)
    if [ "${{ matrix.os }}" = "windows-latest" ]; then
      export PATH="$(pwd)/target/debug:$PATH"
      cargo test --workspace --locked --exclude prtip-network
    else
      cargo test --workspace --locked
    fi
  shell: bash
```

**Rationale:**
- Minimal change (one line added)
- Uses existing DLLs in target\debug\
- Bash-native PATH syntax
- No need to change other steps
- Zero risk to macOS/Linux

## Expected Result
- Windows tests: 17/29 → 29/29 passing
- CI status: 6/7 → 7/7 passing
- Zero code changes
- Single line CI config change

---

**Next Step:** Implement Option 4 fix and test with GitHub Actions.
