## Description

<!-- Provide a clear and concise description of your changes -->

## Type of Change

<!-- Check all that apply -->

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature (non-breaking change which adds functionality)
- [ ] 💥 Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] 📝 Documentation update
- [ ] 🔧 Configuration / infrastructure change
- [ ] ⚡ Performance improvement
- [ ] ♻️ Refactoring (no functional changes)
- [ ] ✅ Test improvements

## Related Issues

<!-- Link related issues using keywords: Fixes #123, Closes #456, Related to #789 -->

Fixes #

## Changes Made

<!-- Provide a detailed list of changes -->

-
-
-

## Testing Performed

<!-- Describe the testing you've done -->

### Manual Testing

- [ ] Tested on Linux
- [ ] Tested on Windows
- [ ] Tested on macOS
- [ ] Tested on FreeBSD

### Automated Testing

- [ ] All existing tests pass (`cargo test --workspace`)
- [ ] Added new unit tests
- [ ] Added new integration tests
- [ ] Tested with different scan types (SYN, Connect, UDP, etc.)
- [ ] Benchmarked performance (if applicable)

### Test Commands

<!-- Provide specific commands used for testing -->

```bash
# Example test commands
cargo test --workspace
cargo run -- -sS -p 80,443 scanme.nmap.org
```

### Test Results

<!-- Paste relevant test output or summarize results -->

```
Tests: 643/643 passed
Performance: [metrics if applicable]
```

## Quality Checklist

<!-- Ensure all items are checked before requesting review -->

- [ ] Code follows project style guidelines (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy --workspace --all-targets -- -D warnings`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] MSRV compatibility maintained (Rust 1.85+)
- [ ] Documentation updated (if needed):
  - [ ] README.md statistics/features updated
  - [ ] CHANGELOG.md entry added
  - [ ] Code comments/rustdoc updated
  - [ ] API documentation complete (`cargo doc`)
- [ ] No new compiler warnings
- [ ] Commit messages follow conventional commits format
- [ ] No TODO/FIXME/WIP markers left uncommitted

## Performance Impact

<!-- For performance-related changes, provide metrics -->

**Before:**
- Metric: [value]

**After:**
- Metric: [value]

**Improvement:** [X% faster / Y seconds reduction / no change / N/A]

## Breaking Changes

<!-- If this is a breaking change, describe the impact and migration path -->

- [ ] This PR includes breaking changes
- [ ] Migration guide provided (if applicable)

**Impact:**
<!-- Describe what breaks and how users should update their code/usage -->

## Security Considerations

<!-- For security-related changes, describe the security implications -->

- [ ] This PR has security implications
- [ ] Security review requested
- [ ] Input validation added/updated
- [ ] Privilege escalation considered
- [ ] No sensitive data exposed in logs/output

## Platform Compatibility

<!-- Check platforms this PR affects or has been tested on -->

### Tested Platforms

- [ ] Linux x86_64 (glibc)
- [ ] Windows x86_64
- [ ] macOS Intel (x86_64)
- [ ] macOS Apple Silicon (ARM64)
- [ ] FreeBSD x86_64

### Platform-Specific Changes

<!-- Describe any platform-specific code or considerations -->

- [ ] No platform-specific code
- [ ] Platform-specific code properly isolated with `cfg` attributes
- [ ] All platforms tested (or CI will test)

## Code Coverage

<!-- If adding new code, what's the test coverage? -->

**Coverage:**
- New code: [X%] (target: >80%)
- Overall: [Y%] (target: >80%)

**Uncovered lines:** [List any uncovered lines and why they're acceptable]

## Additional Context

<!-- Add any other context, screenshots, or information about the PR -->

## Checklist for Reviewers

<!-- This helps reviewers know what to focus on -->

**Focus Areas:**
- [ ] Code quality and style
- [ ] Test coverage and correctness
- [ ] Documentation completeness
- [ ] Performance implications
- [ ] Security considerations
- [ ] Cross-platform compatibility

**Special Attention:**
<!-- Highlight any areas that need extra scrutiny -->

---

**By submitting this PR, I confirm:**
- [ ] I have read and followed the [Contributing Guidelines](../CONTRIBUTING.md)
- [ ] I have tested my changes thoroughly
- [ ] I agree to license my contributions under GPL-3.0
- [ ] My commits follow the conventional commits format (feat:, fix:, docs:, etc.)
