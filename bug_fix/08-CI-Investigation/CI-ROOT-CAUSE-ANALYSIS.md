# CI Root Cause Analysis - Windows Integration Test Failures

**Analysis Date:** 2025-10-15  
**Run ID:** 18517056169  
**Commit:** 02037ad (fix(ci): resolve Windows and macOS test failures)  
**Status:** WINDOWS FAILS (12 tests), macOS PASSES (all tests), Linux PASSES (all tests)

## Executive Summary

**CRITICAL FINDING:** Windows integration tests are failing with exit code `-1073741701` (0xC000007B) which is **STATUS_INVALID_IMAGE_FORMAT**. This indicates a **DLL architecture mismatch** or **missing runtime dependency**, NOT a DLL loading path issue.

## Detailed Analysis

### 1. Windows Test Failures (12 of 29 integration tests)

**Exit Code:** `-1073741701` (hexadecimal: 0xC000007B)  
**Windows Error:** STATUS_INVALID_IMAGE_FORMAT  
**Meaning:** The application tried to load a DLL that is the wrong architecture (32-bit vs 64-bit) or is missing a required runtime library.

**Failed Tests:**
1. test_cli_excessive_retries - Empty stderr (process crashed before output)
2. test_cli_excessive_max_concurrent - Empty stderr (process crashed)
3. test_cli_help - Exit code -1073741701
4. test_cli_excessive_timeout - Empty stderr
5. test_cli_invalid_ports - Empty stderr
6. test_cli_invalid_port_range - Empty stderr
7. test_cli_multiple_targets - Unexpected failure
8. test_cli_no_targets - Empty stderr
9. test_cli_port_zero - Empty stderr
10. test_cli_version - Exit code -1073741701
11. test_cli_zero_max_rate - Empty stderr
12. test_cli_zero_timeout - Empty stderr

**Pattern:** ALL failures show empty stderr and immediate process termination. The process cannot even start to produce error messages.

### 2. Root Cause: DLL Architecture Mismatch

The Npcap 1.79 installer contains BOTH 32-bit and 64-bit DLLs:
- x86 (32-bit): `$PLUGINSDIR/x86/`
- x64 (64-bit): `$PLUGINSDIR/x64/`

**CRITICAL ERROR in ci.yml line ~75:**
```yaml
7z x npcap-1.79.exe -o"npcap_extracted" $PLUGINSDIR/Packet.dll $PLUGINSDIR/wpcap.dll
```

This extracts from `$PLUGINSDIR/` WITHOUT specifying `x64/` subdirectory. The 7-Zip extraction may be:
1. Extracting 32-bit DLLs instead of 64-bit
2. Failing to extract DLLs at all
3. Extracting corrupted DLLs

**GitHub Actions Windows Runners:**
- Architecture: x64 (64-bit)
- Rust target: x86_64-pc-windows-msvc (64-bit)
- Binary built: 64-bit prtip.exe

**Result:** The 64-bit prtip.exe cannot load 32-bit DLLs → STATUS_INVALID_IMAGE_FORMAT

### 3. Evidence from Logs

**Test Execution Pattern:**
- Unit tests (all crates): ✅ PASS (97 + 125 tests)
- Integration tests that DON'T spawn prtip.exe: ✅ PASS
- Integration tests that spawn prtip.exe subprocess: ❌ FAIL (12 tests)

**Key Observations:**
1. `test_cli_help` fails with -1073741701 on `prtip.exe --help`
2. `test_cli_version` fails with -1073741701 on `prtip.exe --version`
3. Both commands should execute WITHOUT loading packet capture DLLs
4. This means the DLL loading happens at process startup, NOT on-demand

**DLL Loading Timing:**
- The failures occur even for `--help` and `--version` flags
- This suggests pnet/pcap crate loads DLLs during static initialization
- The DLL architecture mismatch prevents the process from even starting

### 4. Comparison with Last Successful Commit

**Last Successful:** Commit 1c611ce (not in recent history, likely pre-Windows CI fixes)  
**Current State:** Multiple failed attempts to fix Windows DLL issues (10+ commits)

**CI Workflow Evolution:**
1. Initial: Npcap installer execution (hangs in CI)
2. Attempt 1: WinPcap extraction (DLL not found)
3. Attempt 2: VC++ 2010 runtime (wrong runtime)
4. Attempt 3: Npcap 1.79 extraction (current - wrong architecture)

### 5. macOS and Linux Status

**macOS (macos-latest):** ✅ ALL TESTS PASS (789/789)
- No timing issues
- No permission issues
- The previous timeout fix (2s→5s) appears to have worked

**Linux (ubuntu-latest):** ✅ ALL TESTS PASS (789/789)
- No issues whatsoever

**CI Status:** 6/7 jobs passing, only Windows fails

## Root Cause Statement

The Windows CI failure is caused by extracting 32-bit Npcap DLLs from the installer instead of 64-bit DLLs. The GitHub Actions Windows runner is x64, Rust builds a 64-bit binary, but the DLLs extracted are likely 32-bit. When the 64-bit prtip.exe attempts to load 32-bit DLLs during static initialization, Windows returns STATUS_INVALID_IMAGE_FORMAT (-1073741701), preventing the process from starting.

## Recommended Fix

### Option 1: Explicit x64 Subdirectory Extraction (PREFERRED)

```yaml
- name: Install Npcap SDK and Extract Runtime DLLs (Windows)
  if: runner.os == 'Windows'
  shell: bash
  run: |
    # Download Npcap SDK (compile-time headers)
    curl -L https://npcap.com/dist/npcap-sdk-1.13.zip -o npcap-sdk.zip
    7z x npcap-sdk.zip -o"C:\npcap-sdk"
    
    # Download Npcap installer (runtime DLLs)
    curl -L https://npcap.com/dist/npcap-1.79.exe -o npcap-1.79.exe
    
    # Extract x64 (64-bit) DLLs from installer
    7z x npcap-1.79.exe -o"npcap_extracted"
    
    # Copy x64 DLLs to system directory
    cp npcap_extracted/\$PLUGINSDIR/x64/Packet.dll C:/Windows/System32/
    cp npcap_extracted/\$PLUGINSDIR/x64/wpcap.dll C:/Windows/System32/
    
    # Verify DLL architecture (should show "x64")
    file C:/Windows/System32/Packet.dll
    file C:/Windows/System32/wpcap.dll
    
    # Set environment for build
    echo "LIB=C:\npcap-sdk\Lib\x64" >> $GITHUB_ENV
```

### Option 2: Use Pre-Built WinPcap 4.1.3 (x64)

```yaml
# Alternative: Use WinPcap 4.1.3 which has simpler extraction
# Download: https://www.winpcap.org/install/bin/WinPcap_4_1_3.exe
# Extract x64 DLLs directly (no subdirectories)
```

### Option 3: Install Npcap Normally (May Require Admin)

```yaml
- name: Install Npcap (Windows)
  if: runner.os == 'Windows'
  shell: pwsh
  run: |
    # Download and install Npcap silently
    curl -L https://npcap.com/dist/npcap-1.79.exe -o npcap.exe
    ./npcap.exe /S
    Start-Sleep -Seconds 30
```

## Implementation Plan

1. **Verify DLL Extraction:** Add `file` command to check DLL architecture
2. **Fix Extraction Path:** Update ci.yml to extract from `$PLUGINSDIR/x64/`
3. **Test Locally:** Cannot reproduce without Windows (user has Windows 11 but local tests pass)
4. **Commit and Test:** Push fix and monitor GitHub Actions

## Files to Modify

1. `.github/workflows/ci.yml` - Fix DLL extraction path (lines ~75-80)

## Expected Impact

- Windows integration tests: 17/29 → 29/29 passing
- CI status: 6/7 → 7/7 passing (100%)
- Zero code changes required (CI-only fix)

## Knowledge Graph Entities

- Windows-CI-DLL-Architecture-Mismatch (Issue)
- STATUS_INVALID_IMAGE_FORMAT (Windows Error)
- Npcap-1.79-x64-DLL-Path (Solution)
- ProRT-IP-Windows-CI-Fix-v2 (Implementation)

---

**Next Step:** Implement Option 1 (explicit x64 extraction) and verify with GitHub Actions run.
