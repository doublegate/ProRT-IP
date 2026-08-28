# Repository history

This repository's git history was rewritten once, on 2026-08-27, before the
project was first published. This document records what was removed and why, so
that anyone auditing the history knows it is not the original object graph.

## Why

ProRT-IP declared `license = "GPL-3.0"` while vendoring, and compiling in, data
covered by the **Nmap Public Source License**. The NPSL states that a work which
"Reads or includes Covered Software data files, such as `nmap-os-db` or
`nmap-service-probes`" is a Derivative Work, and that Derivative Works may not be
distributed under plain GPL terms without the licensor's permission.

Removing that data from the working tree was not sufficient. Git history is
published content: a clone of a repository carries every past revision of every
file. Publishing a history that still contained the NPSL-covered material would
have distributed it just as surely as shipping it at the tip.

## What was removed from every commit

| Path | Reason |
|---|---|
| `crates/prtip-core/data/nmap-service-probes` | The vendored NPSL corpus, 17,128 lines, 2.57 MB. |
| `crates/prtip-core/src/top_ports.rs` | Held `TOP_100_PORTS` / `TOP_1000_PORTS`, which its own module documentation described as "based on nmap-services frequency data". `nmap-services` is NPSL-covered exactly as `nmap-service-probes` is. |
| `crates/prtip-scanner/examples/common_fast_scan.rs` | Carried a hand-copied duplicate of that 100-port array. |
| `crates/prtip-tui/src/widgets/port_selection.rs` | Carried a third copy of the same array. |
| `docs/src/reference/port-specification.md` | Documented a `--top-ports 10` example that was verbatim Nmap's top ten. |
| `benchmarks/archive/*/perf/*` | Not a licensing matter. 35 MB of regenerable `perf` captures, including a single 24 MB `.data` file, removed to keep clone size reasonable. |

All five of the first group have been recreated from clean-room work and are
present at the tip. Their pre-rewrite history lives only in the private archive
described below.

## What was kept

Everything else, including all 452 commits and all 28 release tags. The rewrite
removed paths, not commits: no commit became empty, and the commit graph's shape
is unchanged. Commit hashes did change, because removing a blob changes every
tree and therefore every commit that contains it.

Some Nmap-shaped material was deliberately **not** treated as Covered Software.
A truncated TLS ClientHello (`\x16\x03\x00\x00S\x01\x00\x00O\x03\x00`) appeared
in a few examples and test fixtures; every byte of it is dictated by the TLS
record and handshake layout (RFC 6101 / RFC 2246), so there is no authorship in
it to license. Those lines were rewritten anyway, because after the corpus was
replaced they no longer described what ProRT-IP sends — an accuracy fix, not a
licensing one. `crates/prtip-core/data/ATTRIBUTION.md` records that reasoning in
full.

## Consequences you may notice

- **Some historical commits do not build.** Commits predating the remediation
  reference a `crates/prtip-core/src/top_ports.rs` that no longer exists at that
  point in history. `git bisect` across the boundary will hit compile failures.
  The tip and every release tag's tree are consistent and build.
- **The five recreated files have no history before the remediation commit.**
  `git log --follow` on them stops there.
- **Pre-rewrite commit hashes do not resolve.** Any hash cited in an old issue,
  release note or external link refers to the pre-rewrite graph.

## Where the original history is

The unmodified repository — full object graph, original hashes, and the
NPSL-covered files as they were — is preserved in a **private, archived**
repository, `doublegate/ProRT-IP_NPSL`. It is private precisely because it
contains material that cannot be redistributed under this project's licence, and
archived so that it is read-only.

## Related documents

- `crates/prtip-core/data/ATTRIBUTION.md` — per-source provenance for the
  replacement corpus and the port ordering, and the full record of what was
  rejected and why.
- `tools/gen-top-ports/RULE.md` — the ranking rule that replaced the
  frequency-derived ordering, and the licensing analysis behind it.
- `CHANGELOG.md` — the `[Unreleased]` section covering the remediation.
