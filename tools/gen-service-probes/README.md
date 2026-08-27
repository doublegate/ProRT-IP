# gen-service-probes

Rebuilds `crates/prtip-core/data/service-probes.txt`, the service-detection
corpus that is `include_str!`-embedded into every ProRT-IP binary.

The corpus is generated rather than hand-maintained so that port assignments
stay authoritative and refreshable, and so that the only file a human edits is a
single reviewable overlay of protocol knowledge.

## Layout

| Path | Role |
|---|---|
| `fetch-iana.sh` | Downloads the IANA port registry and reduces it to `data/iana-ports.tsv`. |
| `data/iana-ports.tsv` | Committed snapshot of the registry (service, port, transport, description). The generator's only external input. |
| `overlay.txt` | Hand-authored probe payloads and match expressions. **The file to edit.** |
| `generate.py` | Merges the two into the corpus. Pure stdlib Python 3. |
| `FORMAT.md` | The probe-file grammar and its constraints. |

## Design

1. `overlay.txt` declares each probe: transport, name, payload, rarity, its
   `match`/`softmatch` rules, and — instead of a hardcoded port list — the set of
   **IANA service names** it applies to (`iana http https http-alt`).
2. `generate.py` expands those names against `data/iana-ports.tsv`, filtered to
   the probe's transport, and unions the result with any `extra-ports` (de-facto
   assignments IANA does not carry, e.g. 2181 for ZooKeeper or 8080 aliases).
3. The rendered corpus carries a provenance header and the real coverage counts.

This split means a registry refresh (`./fetch-iana.sh`) can widen port coverage
without anyone editing a probe, while protocol knowledge lives in one place that
a reviewer can read end to end.

`generate.py` also lints the overlay against the parser's quirks (see
`FORMAT.md`): a literal `|`, or a `p/…/` value containing a space, is a hard
error rather than a rule that vanishes silently at runtime.

## Usage

```bash
# regenerate the corpus after editing overlay.txt
python3 tools/gen-service-probes/generate.py

# refresh the IANA snapshot, then regenerate and review the diff
tools/gen-service-probes/fetch-iana.sh
python3 tools/gen-service-probes/generate.py

# CI / pre-commit: fail if the committed corpus is stale
python3 tools/gen-service-probes/generate.py --check
```

After regenerating, run `cargo test -p prtip-core --lib service_db`.
`test_embedded_corpus_fully_parses` asserts that every `match` and `softmatch`
line in the corpus was actually loaded, which is what catches a regex that
failed to compile.

## Licensing

Everything here is GPL-3.0-or-later, the same as the rest of ProRT-IP.

Contributions to `overlay.txt` must be written from published protocol
specifications, from a vendor's own documentation of its banner, or from
first-hand observation of a service you control. **Do not** copy, transcribe or
adapt rules from `nmap-service-probes`, `nmap-os-db`, or any other database
under a licence incompatible with GPL-3.0. The Nmap Public Source License
classifies a work that merely *reads* those data files as a Derivative Work, so
ProRT-IP neither ships them nor looks for them at runtime.

See `crates/prtip-core/data/ATTRIBUTION.md` for the full provenance record.
