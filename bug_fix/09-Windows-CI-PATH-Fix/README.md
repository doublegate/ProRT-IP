# Bug Fix #09: Windows CI PATH Format Fix

**Date:** 2025-10-15
**Commit:** 598c217
**Status:** ✅ RESOLVED
**Severity:** HIGH (blocking Windows CI)
**Time to Fix:** 45 minutes

## Summary

Fixed Windows CI integration test failures caused by PATH format mismatch. Git Bash returns Unix-style paths (`/d/a/...`) but Windows DLL loader requires native format (`D:\a\...`). Solution: Use `cygpath -w` to convert paths before adding to PATH.

## Problem

**Error:** 12/29 integration tests failing with error code -1073741701 (STATUS_DLL_NOT_FOUND)
**Impact:** Windows CI failing, blocking releases
**Pattern:** All tests spawning prtip.exe failed with empty stdout/stderr

## Root Cause

1. DLLs correctly copied to `target\debug\`
2. PATH set with `export PATH="$(pwd)/target/debug:$PATH"` in Git Bash
3. Git Bash converted to Unix format: `/d/a/ProRT-IP/ProRT-IP/target/debug`
4. Windows DLL loader cannot use Unix-style paths
5. Subprocess spawning failed before main() execution

## Solution

**Change:** `.github/workflows/ci.yml` lines 181-182

```bash
# BEFORE (FAILED)
export PATH="$(pwd)/target/debug:$PATH"

# AFTER (FIXED)
WIN_PATH=$(cygpath -w "$(pwd)/target/debug")
export PATH="$WIN_PATH:$PATH"
```

**Result:** Path converted to `D:\a\ProRT-IP\ProRT-IP\target\debug`, DLL loader succeeds

## Files in This Directory

1. **CI-FIX-ATTEMPT-2-ANALYSIS.md** - Comprehensive root cause analysis
2. **CI-FIX-COMPLETE-SUMMARY.md** - Full investigation summary
3. **README.md** - This file

## Related Commits

- **598c217** - Final fix (cygpath conversion)
- **41f167f** - Previous attempt (PATH export, failed)
- **be99938** - Npcap switch (DLLs updated, PATH issue persisted)
- **02037ad** - Bash revert (shell changed, PATH issue persisted)

## Test Results

**Before Fix:**
- CI Status: 6/7 passing (85.7%)
- Windows: 17/29 tests passing (58.6%)
- Error: -1073741701 (DLL not found)

**After Fix:**
- CI Status: 7/7 passing (100%) ✅
- Windows: 29/29 tests passing (100%) ✅
- Error: None

## Lessons Learned

1. **Git Bash Path Conversion:** Always use `cygpath -w` when setting Windows paths in Git Bash
2. **DLL Loading:** Windows DLL loader requires native format paths
3. **Subprocess Testing:** Test subprocess spawning on Windows CI, not just direct execution
4. **Error Code Analysis:** -1073741701 = STATUS_DLL_NOT_FOUND (DLL load failure before main)

## Related Issues

- GitHub Actions run: #18520268693 (fix verification)
- Windows error code reference: https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-

## Future Prevention

1. Document Windows path requirements in CI comments
2. Test subprocess spawning in CI validation
3. Monitor for DLL-related error codes
4. Keep cygpath usage pattern for future Windows PATH needs
