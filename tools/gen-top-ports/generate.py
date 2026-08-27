#!/usr/bin/env python3
"""Generate ProRT-IP's port priority lists from the committed IANA snapshot.

The rule this implements is specified in RULE.md, which is authoritative. Read
that first; the code below is a transcription of it and nothing more.

The output is an editorial ranking of IANA assignments. It is NOT a frequency
ranking, and nothing here reads a measurement of any kind. In particular this
generator must never read nmap-services, nmap-service-probes, nmap-os-db, or
any other NPSL-covered file, and must never consult a previous version of the
generated arrays.

Usage:
    python3 tools/gen-top-ports/generate.py            # rewrite the managed block
    python3 tools/gen-top-ports/generate.py --check     # CI gate: fail if stale
    python3 tools/gen-top-ports/generate.py --explain N # audit the first N entries

Pure stdlib Python 3.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent

# The snapshot is shared with tools/gen-service-probes rather than duplicated:
# one registry snapshot, one fetch script, one thing to review on a refresh.
SNAPSHOT = REPO / "tools" / "gen-service-probes" / "data" / "iana-ports.tsv"
TARGET = REPO / "crates" / "prtip-core" / "src" / "top_ports.rs"

BEGIN = "// BEGIN GENERATED: gen-top-ports -- edit tools/gen-top-ports/, not this block"
END = "// END GENERATED: gen-top-ports"

# ---------------------------------------------------------------------------
# RULE.md Step 5 -- the service family table.
#
# Ordered. First family with a matching pattern wins, so a port belongs to
# exactly one family. Patterns are case-insensitive regexes matched against the
# lowercased concatenation of every IANA service name and description for the
# port. Every pattern must be justifiable from published protocol
# documentation; none of them encodes an observation about deployment.
#
# The ORDER of this table is the rule's editorial judgement and the rationale
# for it is in RULE.md. Changing the order changes the output; changing it
# needs a matching RULE.md edit.
# ---------------------------------------------------------------------------
FAMILIES: list[tuple[str, list[str]]] = [
    (
        "remote-access",
        [
            r"\bssh\b",
            r"secure shell",
            r"\btelnet",
            r"\brlogin\b",
            r"remote login",
            r"remote shell",
            r"remote process execution",
            r"\brexec\b",
            r"\brsh\b",
            r"remote desktop",
            r"\brdp\b",
            r"ms-wbt",
            r"wbt server",
            r"\bvnc\b",
            r"remote framebuffer",
            r"\brfb\b",
            r"\bx11\b",
            r"x window",
            r"terminal server",
            r"remote terminal",
            r"remote access",
            r"remote control",
            r"remote admin",
            r"pcanywhere",
            r"citrix",
            r"\bica\b",
        ],
    ),
    (
        "web",
        [
            r"\bhttp\b",
            r"\bhttps\b",
            r"http-alt",
            r"http alternate",
            r"world wide web",
            r"\bwww\b",
            r"web server",
            r"web-server",
            r"web proxy",
            r"websocket",
            r"web socket",
            r"web service",
            r"web-based",
            r"\bhttp/",
            r"\bhttpd\b",
        ],
    ),
    (
        "file-sharing",
        [
            r"\bftp\b",
            r"ftp-data",
            r"\bftps\b",
            r"file transfer",
            r"\btftp\b",
            r"trivial file transfer",
            r"\bsftp\b",
            r"\bnfs\b",
            r"network file system",
            r"\bsmb\b",
            r"\bcifs\b",
            r"microsoft-ds",
            r"netbios-ssn",
            r"netbios session",
            r"\bafp\b",
            r"appleshare",
            r"apple filing",
            r"\brsync\b",
            r"webdav",
            r"file server",
            r"file service",
            r"file sharing",
            r"file access",
            r"\bsamba\b",
        ],
    ),
    (
        "database",
        [
            r"\bsql\b",
            r"mysql",
            r"mariadb",
            r"postgres",
            r"\boracle\b",
            r"\btns\b",
            r"sybase",
            r"\bdb2\b",
            r"mongodb",
            r"\bredis\b",
            r"cassandra",
            r"memcache",
            r"couchdb",
            r"couchbase",
            r"\bdatabase\b",
            r"informix",
            r"\bodbc\b",
            r"\bjdbc\b",
            r"elasticsearch",
            r"clickhouse",
            r"\bneo4j\b",
            r"influxdb",
            r"\bdbms\b",
            r"data warehouse",
        ],
    ),
    (
        "mail",
        [
            r"\bsmtp\b",
            r"simple mail transfer",
            r"\bpop2\b",
            r"\bpop3\b",
            r"post office protocol",
            r"\bimap\b",
            r"message access protocol",
            r"\bmail\b",
            r"message submission",
            r"\bsubmission\b",
            r"\bmta\b",
            r"\bmilter\b",
            r"\bsieve\b",
        ],
    ),
    (
        "name-service",
        [
            r"domain name",
            r"\bdns\b",
            r"\bmdns\b",
            r"multicast dns",
            r"\bllmnr\b",
            r"\bwhois\b",
            r"netbios-ns",
            r"netbios name",
            r"netbios-dgm",
            r"netbios datagram",
            r"name server",
            r"nameserver",
            r"name service",
            r"\brwhois\b",
            r"\bnicname\b",
        ],
    ),
    (
        "directory-auth",
        [
            r"\bldap\b",
            r"\bldaps\b",
            r"directory access",
            r"kerberos",
            r"\bkpasswd\b",
            r"\bradius\b",
            r"tacacs",
            r"\bnis\b",
            r"network information service",
            r"\bident\b",
            r"authentication",
            r"\bauth\b",
            r"\boauth\b",
            r"\bsaml\b",
            r"single sign",
            r"credential",
        ],
    ),
    (
        "management-discovery",
        [
            r"\bsnmp\b",
            r"network management",
            r"\bipmi\b",
            r"\brmcp\b",
            r"\basf\b",
            r"\bsyslog\b",
            r"\bntp\b",
            r"network time",
            r"\bwbem\b",
            r"ws-management",
            r"\bwinrm\b",
            r"\bwmi\b",
            r"remote management",
            r"monitoring",
            r"\bupnp\b",
            r"\bssdp\b",
            r"simple service discovery",
            r"\bnetconf\b",
            r"\brestconf\b",
            r"\bipfix\b",
            r"\bnetflow\b",
            r"remote procedure call",
            r"\brpc\b",
            r"portmapper",
            r"endpoint resolution",
            r"endpoint mapper",
            r"service location",
            r"\bslp\b",
            r"service discovery",
        ],
    ),
    (
        "voice-video",
        [
            r"\bsip\b",
            r"session initiation",
            r"\brtsp\b",
            r"\brtp\b",
            r"\brtcp\b",
            r"\bh\.?323\b",
            r"real time streaming",
            r"real-time streaming",
            r"\bvoip\b",
            r"\biax\b",
            r"\bmgcp\b",
            r"\bsccp\b",
            r"\bstreaming\b",
            r"video conference",
            r"videoconference",
            r"\bsdp\b",
        ],
    ),
    (
        "messaging",
        [
            r"\bamqp\b",
            r"\bmqtt\b",
            r"\bxmpp\b",
            r"jabber",
            r"\birc\b",
            r"internet relay chat",
            r"\bstomp\b",
            r"\bkafka\b",
            r"message queue",
            r"messaging",
            r"\bnntp\b",
            r"network news",
            r"\bnats\b",
            r"publish/subscribe",
            r"instant messag",
        ],
    ),
    (
        "print",
        [
            r"\bipp\b",
            r"internet printing",
            r"\blpd\b",
            r"\blpr\b",
            r"\bprinter\b",
            r"\bprinting\b",
            r"print server",
            r"print spooler",
            r"jetdirect",
            r"\bcups\b",
        ],
    ),
    (
        "network-infra",
        [
            r"\bbgp\b",
            r"border gateway",
            r"\bdhcp\b",
            r"\bdhcpv6\b",
            r"\bbootp\b",
            r"bootstrap protocol",
            r"\bospf\b",
            r"routing information protocol",
            r"\bisakmp\b",
            r"\bike\b",
            r"\bipsec\b",
            r"\bl2tp\b",
            r"\bpptp\b",
            r"openvpn",
            r"\bvpn\b",
            r"\bwireguard\b",
            r"\bsocks\b",
            r"\btunnel\b",
            r"\bpxe\b",
            r"\bgre\b",
            r"\bmpls\b",
            r"\bvrrp\b",
            r"\bhsrp\b",
        ],
    ),
]

OTHER = "other"

# System / User / Dynamic, per RFC 6335 section 6.
WELL_KNOWN_MAX = 1023
REGISTERED_MAX = 49151


def range_class(port: int) -> int:
    """RULE.md Step 3."""
    if port <= WELL_KNOWN_MAX:
        return 0
    if port <= REGISTERED_MAX:
        return 1
    return 2


def load_snapshot(path: Path) -> dict[int, tuple[str, bool]]:
    """RULE.md Steps 1-2. Returns {port: (haystack, has_tcp)}."""
    text: dict[int, list[str]] = {}
    tcp: dict[int, bool] = {}
    with path.open(encoding="utf-8") as fh:
        for line in fh:
            if line.startswith("#") or not line.strip():
                continue
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 3:
                continue
            name, port_s, transport = parts[0], parts[1], parts[2]
            desc = parts[3] if len(parts) > 3 else ""
            try:
                port = int(port_s)
            except ValueError:
                continue
            if not 0 < port < 65536:
                continue
            if transport not in ("tcp", "udp"):
                continue
            text.setdefault(port, []).extend([name, desc])
            tcp[port] = tcp.get(port, False) or transport == "tcp"
    return {p: (" ".join(v).lower(), tcp[p]) for p, v in text.items()}


def compile_families() -> list[tuple[str, re.Pattern[str]]]:
    return [
        (name, re.compile("|".join(pats), re.IGNORECASE)) for name, pats in FAMILIES
    ]


def classify(haystack: str, compiled: list[tuple[str, re.Pattern[str]]]) -> str:
    """RULE.md Step 5. First family in table order whose pattern matches."""
    for name, rx in compiled:
        if rx.search(haystack):
            return name
    return OTHER


def build_sequence(records: dict[int, tuple[str, bool]]) -> list[tuple[int, str]]:
    """RULE.md Steps 4-7. Returns the ranked [(port, family)] sequence."""
    compiled = compile_families()
    buckets: dict[str, list[int]] = {name: [] for name, _ in FAMILIES}
    buckets[OTHER] = []

    for port, (haystack, has_tcp) in records.items():
        buckets[classify(haystack, compiled)].append(port)

    def sort_key(port: int) -> tuple[int, int, int]:
        # Step 6: (range class, transport class, port number). The port number
        # is unique, so this is a strict total order and every tie is broken by
        # ascending port number and nothing else.
        _, has_tcp = records[port]
        return (range_class(port), 0 if has_tcp else 1, port)

    for name in buckets:
        buckets[name].sort(key=sort_key)

    # Step 7: round-robin over the classified families, in table order.
    classified = [name for name, _ in FAMILIES]
    sequence: list[tuple[int, str]] = []
    round_index = 0
    while True:
        emitted = False
        for name in classified:
            bucket = buckets[name]
            if round_index < len(bucket):
                sequence.append((bucket[round_index], name))
                emitted = True
        if not emitted:
            break
        round_index += 1

    # Step 7 continued: `other` never interleaves; it is appended after every
    # classified assignment.
    sequence.extend((port, OTHER) for port in buckets[OTHER])
    return sequence


def format_array(name: str, doc: list[str], ports: list[int]) -> str:
    lines = [f"/// {d}".rstrip() for d in doc]
    lines.append(f"pub const {name}: &[u16] = &[")
    row: list[str] = []
    width = 4
    for port in ports:
        item = f"{port},"
        # rustfmt reflows this array itself, so the generator must reproduce
        # rustfmt's own greedy packing exactly or `--check` and `cargo fmt
        # --check` will fight each other. Empirically rustfmt (max_width = 100)
        # accepts a 99-column line and breaks at 100.
        if len(row) and width + 1 + len(item) > 99:
            lines.append("    " + " ".join(row))
            row = []
            width = 4
        row.append(item)
        width += (1 if width > 4 else 0) + len(item)
    if row:
        lines.append("    " + " ".join(row))
    lines.append("];")
    return "\n".join(lines)


def render_block(sequence: list[tuple[int, str]]) -> str:
    ports = [p for p, _ in sequence]
    if len(ports) < 1000:
        raise SystemExit(
            f"snapshot yields only {len(ports)} ranked ports; need at least 1000"
        )
    top100 = ports[:100]
    top1000 = ports[:1000]

    header = [
        BEGIN,
        "//",
        "// Regenerate with:  python3 tools/gen-top-ports/generate.py",
        "// CI gate:          python3 tools/gen-top-ports/generate.py --check",
        "//",
        "// Source: tools/gen-service-probes/data/iana-ports.tsv (committed IANA",
        "// Service Name and Transport Protocol Port Number Registry snapshot).",
        "// Rule:   tools/gen-top-ports/RULE.md",
        "//",
        "// This is an editorial ranking of registry assignments, not a frequency",
        "// ranking. No input to it is a measurement.",
    ]

    a = format_array(
        "PRIORITY_PORTS_100",
        [
            "The first 100 ports of ProRT-IP's editorial priority ordering (`-F`).",
            "",
            "Produced by applying `tools/gen-top-ports/RULE.md` to a committed snapshot",
            "of the IANA Service Name and Transport Protocol Port Number Registry.",
            "",
            "**This is not a frequency ranking.** No claim is made about how often any",
            "of these ports is open in practice, and the selection differs from Nmap's",
            "`-F` and from every other scanner whose list comes from measured data.",
        ],
        top100,
    )
    b = format_array(
        "PRIORITY_PORTS_1000",
        [
            "The first 1000 ports of the same ordering (`--top-ports 1000`).",
            "",
            "`PRIORITY_PORTS_100` is a prefix of this list by construction, so",
            "[`get_priority_ports`] can serve any `n <= 1000` as a simple prefix.",
            "",
            "**This is not a frequency ranking.** See [`PRIORITY_PORTS_100`].",
        ],
        top1000,
    )
    return "\n".join(header) + "\n\n" + a + "\n\n" + b + "\n\n" + END + "\n"


def splice(existing: str, block: str) -> str:
    start = existing.find(BEGIN)
    end = existing.find(END)
    if start == -1 or end == -1:
        raise SystemExit(
            f"managed block markers not found in {TARGET}\n"
            f"  expected a line {BEGIN!r}\n"
            f"  and a line {END!r}"
        )
    end += len(END) + 1  # include the trailing newline
    return existing[:start] + block + existing[end:]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--check",
        action="store_true",
        help="exit non-zero if the committed block is stale (CI gate)",
    )
    ap.add_argument(
        "--explain",
        type=int,
        metavar="N",
        help="print the first N ranked entries with family and IANA name, then exit",
    )
    args = ap.parse_args()

    if not SNAPSHOT.is_file():
        raise SystemExit(f"missing snapshot: {SNAPSHOT}")

    records = load_snapshot(SNAPSHOT)
    sequence = build_sequence(records)

    if args.explain is not None:
        for i, (port, family) in enumerate(sequence[: args.explain], start=1):
            haystack, has_tcp = records[port]
            rc = ("well-known", "registered", "dynamic")[range_class(port)]
            tc = "tcp" if has_tcp else "udp-only"
            print(
                f"{i:4d}  {port:5d}  {family:<18} {rc:<10} {tc:<8} "
                f"{haystack.split()[0]}"
            )
        return 0

    block = render_block(sequence)
    existing = TARGET.read_text(encoding="utf-8")
    updated = splice(existing, block)

    if args.check:
        if updated != existing:
            print(
                f"STALE: {TARGET.relative_to(REPO)} does not match a fresh run of "
                f"{Path(__file__).relative_to(REPO)}.\n"
                "Run: python3 tools/gen-top-ports/generate.py",
                file=sys.stderr,
            )
            return 1
        print(f"OK: {TARGET.relative_to(REPO)} is up to date")
        return 0

    if updated == existing:
        print(f"unchanged: {TARGET.relative_to(REPO)}")
        return 0

    TARGET.write_text(updated, encoding="utf-8")
    counts: dict[str, int] = {}
    for _, family in sequence[:1000]:
        counts[family] = counts.get(family, 0) + 1
    print(f"wrote {TARGET.relative_to(REPO)}")
    print(f"  candidate ports in snapshot: {len(sequence)}")
    print("  family shares of the top 1000:")
    for name, _ in FAMILIES:
        print(f"    {name:<20} {counts.get(name, 0):4d}")
    print(f"    {OTHER:<20} {counts.get(OTHER, 0):4d}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
