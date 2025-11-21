# Phase 4 Enhancements

This document details the feature enhancements implemented during Phase 4.

## Zero-Copy Packet Processing

### Implementation

Zero-copy processing was added for packets larger than 10KB threshold:

```rust
// Threshold for zero-copy
const ZERO_COPY_THRESHOLD: usize = 10 * 1024;

// Direct memory mapping for large packets
if packet.len() > ZERO_COPY_THRESHOLD {
    process_zero_copy(packet);
} else {
    process_standard(packet);
}
```

### Benefits

- 15-20% CPU overhead reduction
- Lower memory bandwidth usage
- Reduced allocation pressure

## NUMA Optimization

### Thread-Local Allocators

Each worker thread uses NUMA-local memory:

- Memory allocated on local node
- IRQ affinity configured
- Cross-socket penalties avoided

### Configuration

```bash
# Set IRQ affinity for network interface
sudo ethtool -L eth0 combined 4
sudo set_irq_affinity.sh eth0
```

## PCAPNG Output Format

### Features

- Interface description blocks
- Packet timestamps with microsecond precision
- Comment blocks for metadata
- Full Wireshark compatibility

### Usage

```bash
prtip -sS target -o scan.pcapng
```

## Evasion Techniques

### IP Fragmentation

Split packets into fragments to evade inspection:

```bash
prtip -sS -f target           # Aggressive fragmentation
prtip -sS --mtu 64 target     # Custom MTU
```

### Decoy Scanning

Hide among decoy source addresses:

```bash
prtip -sS -D RND:5 target     # 5 random decoys
```

## See Also

- [Phase 4 Archive](../archives/phase4.md)
- [Phase 4 Compliance](./phase4-compliance.md)
- [Evasion Techniques](../../features/evasion-techniques.md)
