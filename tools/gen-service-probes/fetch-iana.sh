#!/usr/bin/env bash
# Fetch the IANA Service Name and Transport Protocol Port Number Registry and
# reduce it to the four columns the generator consumes.
#
# Source : https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.csv
# Terms  : IANA protocol-parameter registries are published by IANA/IETF and are
#          freely usable and redistributable (BCP 78 / RFC 5378 applies to the
#          documents; the registry data itself carries no redistribution
#          restriction). See https://www.iana.org/protocols
#
# Usage: ./fetch-iana.sh          # refresh data/iana-ports.tsv in place
#
# The reduced TSV is committed so that `generate.py` is reproducible offline.
# Re-run this script to refresh it, then re-run generate.py and review the diff.

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
url="https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.csv"
raw="$(mktemp)"
trap 'rm -f "$raw"' EXIT

echo "Fetching $url" >&2
curl -sSL --fail -o "$raw" "$url"

mkdir -p "$here/data"
python3 - "$raw" "$here/data/iana-ports.tsv" <<'PY'
import csv, sys, datetime

src, dst = sys.argv[1], sys.argv[2]
rows = []
with open(src, newline="", encoding="utf-8") as fh:
    for r in csv.DictReader(fh):
        name = (r.get("Service Name") or "").strip()
        port = (r.get("Port Number") or "").strip()
        proto = (r.get("Transport Protocol") or "").strip().lower()
        desc = " ".join((r.get("Description") or "").split())
        if not name or not port or proto not in ("tcp", "udp"):
            continue
        if "-" in port:          # ranges carry no single assignment
            continue
        if not port.isdigit():
            continue
        n = int(port)
        if not 0 < n < 65536:
            continue
        if desc.lower() in ("reserved", "unassigned", "de-registered"):
            continue
        rows.append((name, n, proto, desc))

rows.sort(key=lambda t: (t[1], t[2], t[0]))
with open(dst, "w", encoding="utf-8") as fh:
    fh.write("# IANA Service Name and Transport Protocol Port Number Registry (reduced)\n")
    fh.write("# Retrieved: %s\n" % datetime.date.today().isoformat())
    fh.write("# Source: https://www.iana.org/assignments/service-names-port-numbers/"
             "service-names-port-numbers.csv\n")
    fh.write("# Columns: service\tport\ttransport\tdescription\n")
    for name, n, proto, desc in rows:
        fh.write("%s\t%d\t%s\t%s\n" % (name, n, proto, desc))
print("wrote %d rows to %s" % (len(rows), dst), file=sys.stderr)
PY
