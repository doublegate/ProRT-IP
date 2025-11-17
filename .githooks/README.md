# Git Hooks

This directory contains custom Git hooks for the ProRT-IP project.

## Available Hooks

### pre-commit

Automatically checks markdown links in staged files before allowing commits.

**Features:**
- Fast: Only checks staged `.md` files
- Consistent: Uses same config as CI workflow
- Helpful: Shows which links are broken
- Graceful: Skips if markdown-link-check not installed

**Installation:**

```bash
# Method 1: Use custom hooks directory (recommended)
git config core.hooksPath .githooks

# Method 2: Copy to .git/hooks/
cp .githooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

**Requirements:**

```bash
npm install -g markdown-link-check
```

**Usage:**

Hooks run automatically on `git commit`. To skip (not recommended):

```bash
git commit --no-verify
```

## Configuration

Link checking uses `.mlc_config.json` in the project root. See that file for ignore patterns and timeout settings.

## Troubleshooting

**Hook not running:**
- Check: `git config core.hooksPath` (should show `.githooks`)
- OR check: `.git/hooks/pre-commit` exists and is executable

**markdown-link-check not found:**
- Install: `npm install -g markdown-link-check`
- Or skip hook temporarily with `--no-verify`

**Slow on large commits:**
- Hook only checks staged files, should be <5 seconds
- If slow, check for external URL timeouts in config
