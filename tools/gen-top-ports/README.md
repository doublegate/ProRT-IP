# gen-top-ports

Rebuilds `PRIORITY_PORTS_100` and `PRIORITY_PORTS_1000` in
`crates/prtip-core/src/top_ports.rs` — the port lists behind `-F` and
`--top-ports`.

**The lists are an editorial ranking of IANA port assignments, not a frequency
ranking.** Nothing here reads a measurement. Read [`RULE.md`](RULE.md) before
touching anything in this directory; it is the specification, this file is only
the operating manual.

## Layout

| Path | Role |
|---|---|
| `RULE.md` | The ranking rule, its rationale, and the licensing reasoning behind it. **Authoritative.** |
| `generate.py` | Implements `RULE.md`. Pure stdlib Python 3. Contains the family pattern table. |
| *(input)* | `../gen-service-probes/data/iana-ports.tsv` — the committed IANA registry snapshot, shared rather than duplicated. |
| *(output)* | The managed block in `crates/prtip-core/src/top_ports.rs`, between the `BEGIN GENERATED: gen-top-ports` / `END GENERATED` markers. Everything outside those markers is hand-written and is preserved. |

## Why a sibling tool rather than part of `gen-service-probes`

The two generators share one input and nothing else. `gen-service-probes` emits a
runtime data file (`crates/prtip-core/data/service-probes.txt`) built from an
overlay of probe payloads; this one emits Rust constants from a ranking rule with
no overlay at all. Merging them would put two unrelated rules and two unrelated
output contracts in one file for the sake of a shared `open()`.

The snapshot itself is deliberately **not** duplicated: one file, one
`fetch-iana.sh`, one diff to review when the registry is refreshed. That is why
`generate.py` reaches across to `../gen-service-probes/data/`.

## Usage

```bash
# regenerate the arrays after editing generate.py or refreshing the snapshot
python3 tools/gen-top-ports/generate.py

# CI gate: fail if the committed arrays are stale
python3 tools/gen-top-ports/generate.py --check

# audit the ordering by eye -- port, family, range class, transport, IANA name
python3 tools/gen-top-ports/generate.py --explain 20
```

After regenerating, run `cargo test -p prtip-core --lib top_ports`. Two of those
tests are honesty gates rather than ordinary assertions:

- `test_expected_ports_in_priority_100` pins the ports the rule currently ranks
  into the first hundred. If a registry refresh moves one, read the diff — do not
  pin the port back.
- `test_ordering_is_not_frequency_derived` asserts that 3389 and 8080 are *not*
  in the first hundred. A frequency-derived list would rank both highly; this one
  must not. If that test starts failing, either the rule changed or a
  frequency-derived ordering was reintroduced, and both need review.

## Refreshing the registry snapshot

```bash
tools/gen-service-probes/fetch-iana.sh     # refresh the shared snapshot
python3 tools/gen-service-probes/generate.py
python3 tools/gen-top-ports/generate.py
```

Then review both diffs. A registry refresh is the intended way for the ordering
to evolve.

## Changing the rule

Edit `RULE.md` first, then `generate.py` to match, then regenerate. The family
table order in `generate.py` (`FAMILIES`) is the rule's only editorial input and
its rationale lives in `RULE.md`; changing the order without changing the
rationale leaves the project unable to defend its own list.

## Licensing

Everything here is GPL-3.0-or-later, the same as the rest of ProRT-IP.

The only input is the IANA Service Name and Transport Protocol Port Number
Registry, which IANA publishes without redistribution restriction.

**Do not** derive any part of the ordering from `nmap-services`,
`nmap-service-probes`, `nmap-os-db`, or any other database under a licence
incompatible with GPL-3.0 — the NPSL classifies a work that merely *reads* those
files as a Derivative Work. **Do not** derive it from scans.io or Censys data
either: both restrict use to non-commercial purposes, which a GPL-3.0 work cannot
carry.

See `crates/prtip-core/data/ATTRIBUTION.md` for the full provenance record.
