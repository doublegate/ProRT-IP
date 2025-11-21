# Examples (Legacy)

This document contains original command-line examples from earlier ProRT-IP versions.

## Basic Scans (Phase 3)

### Simple SYN Scan

```bash
# Original syntax
prtip --scan-type syn --target 192.168.1.1 --ports 1-1000

# Current equivalent
prtip -sS -p 1-1000 192.168.1.1
```

### Connect Scan

```bash
# Original syntax
prtip --scan-type connect --target example.com --ports 80,443

# Current equivalent
prtip -sT -p 80,443 example.com
```

## Phase 4 Examples

### Zero-Copy Mode

```bash
# Enable zero-copy (automatic for large packets)
prtip -sS -p 1-65535 target --buffer-size 65536
```

### PCAPNG Output

```bash
# Save to PCAPNG format
prtip -sS target -o scan.pcapng
```

### Evasion Examples

```bash
# IP fragmentation
prtip -sS -f target

# Custom MTU
prtip -sS --mtu 24 target

# Decoy scanning
prtip -sS -D 10.0.0.1,10.0.0.2,ME target
```

## Migration Notes

### Changed Flags

| Old | New | Purpose |
|-----|-----|---------|
| `--scan-type syn` | `-sS` | SYN scan |
| `--scan-type connect` | `-sT` | Connect scan |
| `--target` | positional | Target specification |
| `--ports` | `-p` | Port specification |

### Deprecated Options

These options are no longer available:

- `--legacy-output` - Use `-oN` instead
- `--no-color` - Set `NO_COLOR=1` environment variable
- `--quiet-mode` - Use `-q` instead

## Current Documentation

For current examples, see:

- [Quick Start Guide](../../getting-started/quick-start.md)
- [Tutorials](../../getting-started/tutorials.md)
- [Example Scans Gallery](../../getting-started/examples.md)
