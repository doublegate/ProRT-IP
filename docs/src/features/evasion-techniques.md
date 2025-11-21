# Firewall Evasion

ProRT-IP implements advanced firewall and IDS evasion techniques for authorized penetration testing and security assessments.

## Overview

Firewall and Intrusion Detection System (IDS) evasion refers to techniques used to bypass security controls that monitor or block network traffic. These techniques are essential for:

- **Penetration Testing**: Assessing security defenses by simulating attacker behaviors
- **Red Team Operations**: Testing blue team detection capabilities
- **Security Research**: Understanding how malicious actors evade detection
- **Network Troubleshooting**: Diagnosing firewall/IDS misconfigurations

## Legal and Ethical Warning

**WARNING**: Use these techniques ONLY on networks you own or have explicit written permission to test. Unauthorized use is illegal and may result in federal prosecution under the Computer Fraud and Abuse Act (CFAA), civil liability, and imprisonment.

## Evasion Techniques

ProRT-IP implements 5 primary evasion techniques, all nmap-compatible:

| Technique | Flag | Purpose | Detection Risk |
|-----------|------|---------|----------------|
| **IP Fragmentation** | `-f` | Split packets into tiny fragments | Low-Medium |
| **Custom MTU** | `--mtu <SIZE>` | Control fragment sizes | Low |
| **TTL Manipulation** | `--ttl <VALUE>` | Set IP Time-To-Live | Low |
| **Decoy Scanning** | `-D <DECOYS>` | Hide among fake sources | Low-High |
| **Bad Checksums** | `--badsum` | Use invalid checksums | Medium |

## Packet Fragmentation

IP packet fragmentation splits network packets into smaller fragments, evading firewalls and IDS that don't properly reassemble fragments before inspection.

### How It Works

```
Normal Packet (80 bytes):
+--------------------------------------------------+
| IP Header (20) | TCP Header (20) | Data (40)     |
+--------------------------------------------------+

Fragmented (-f flag, MTU 28):
Fragment 1: | IP Header (20) | 8 data |
Fragment 2: | IP Header (20) | 8 data |
Fragment 3: | IP Header (20) | 8 data |
...and so on
```

### Usage

```bash
# Aggressive fragmentation (smallest fragments)
prtip -sS -f -p 1-1000 192.168.1.0/24

# Custom MTU (control fragment size)
prtip -sS --mtu 64 -p 1-1000 192.168.1.0/24
```

**When to Use:**
- Evading stateless firewalls
- Bypassing simple packet filters
- Testing fragment reassembly capabilities

**Trade-offs:**
- Slower scan speed (more packets to send)
- Higher bandwidth usage
- May trigger fragmentation alerts

## Decoy Scanning

Decoy scanning hides your real source IP address among fake decoy addresses.

### Usage

```bash
# Use 3 random decoys
prtip -sS -D RND:3 -p 80,443 target.com

# Specific decoys (your IP inserted randomly)
prtip -sS -D 10.0.0.1,10.0.0.2,ME,10.0.0.3 -p 80,443 target.com
```

**How It Works:**
1. ProRT-IP sends packets from decoy addresses
2. Your real scan packets are interleaved
3. Target sees traffic from N+1 sources
4. Attribution becomes difficult

**Best Practices:**
- Use routable IP addresses (avoid private ranges for internet scans)
- Use IPs that won't raise suspicion (same subnet, ISP)
- Keep decoy count reasonable (3-5 recommended)
- Ensure decoys won't be harmed by response traffic

## Source Port Manipulation

Use a trusted source port to bypass firewall rules that allow certain ports.

```bash
# Use DNS source port (often allowed through firewalls)
prtip -sS -g 53 -p 1-1000 192.168.1.1

# Use HTTPS source port
prtip -sS --source-port 443 -p 1-1000 target.com
```

**Common Trusted Ports:**
- 20 (FTP data)
- 53 (DNS)
- 67 (DHCP)
- 80 (HTTP)
- 443 (HTTPS)

## Timing Manipulation

Slow down scans to avoid detection by rate-based IDS.

```bash
# Paranoid timing (extremely slow, maximum stealth)
prtip -T0 -sS -p 80,443 target.com

# Sneaky timing (slow, IDS evasion)
prtip -T1 -sS -p 1-1000 target.com

# Polite timing (reduced speed)
prtip -T2 -sS -p 1-1000 target.com
```

| Template | Speed | Use Case |
|----------|-------|----------|
| T0 (Paranoid) | 1-10 pps | Maximum stealth, IDS evasion |
| T1 (Sneaky) | 10-50 pps | Slow evasion scanning |
| T2 (Polite) | 50-200 pps | Production networks |
| T3 (Normal) | 1-5K pps | Default balanced |
| T4 (Aggressive) | 5-10K pps | Fast LANs |
| T5 (Insane) | 10-50K pps | Maximum speed |

## Performance Impact

| Technique | Overhead | Notes |
|-----------|----------|-------|
| Fragmentation (-f) | +18% | More packets to craft |
| Decoys (-D 3) | +300% | 4x traffic (3 decoys + real) |
| Source Port (-g) | <1% | Minimal overhead |
| Timing (T0 vs T3) | +50,000% | Extreme slowdown |

## Combined Techniques

For maximum evasion, combine multiple techniques:

```bash
# Fragment + Decoy + Slow timing
prtip -sS -f -D RND:3 -T2 --ttl 64 -p 80,443 target.com

# Full evasion suite
prtip -sS -f --mtu 24 -D RND:5 -g 53 -T1 --ttl 128 -p 80,443 target.com
```

## Detection Considerations

### What Triggers Alerts

| Indicator | Detection Likelihood | Mitigation |
|-----------|---------------------|------------|
| Port scan patterns | High | Use slow timing (T0-T2) |
| SYN flood detection | Medium | Use rate limiting |
| Fragment reassembly | Low-Medium | Use reasonable MTU |
| Decoy traffic | Low | Use realistic decoys |
| Bad checksums | Medium | Use only for testing |

### Avoiding Detection

1. **Reconnaissance first**: Understand target's security posture
2. **Start slow**: Begin with T2, escalate only if needed
3. **Limit port count**: Target specific ports, not full range
4. **Use timing jitter**: Random delays between packets
5. **Test in phases**: Verify each technique works before combining

## See Also

- [Stealth Scanning](./stealth-scanning.md) - FIN/NULL/Xmas scan types
- [Rate Limiting](./rate-limiting.md) - Adaptive rate control
- [Security Best Practices](../advanced/security-best-practices.md) - Safe scanning guidelines
