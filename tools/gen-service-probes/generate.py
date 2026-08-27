#!/usr/bin/env python3
"""Rebuild crates/prtip-core/data/service-probes.txt.

Inputs
------
  data/iana-ports.tsv  Reduced snapshot of the IANA Service Name and Transport
                       Protocol Port Number Registry (see fetch-iana.sh).
  overlay.txt          Hand-authored probe payloads and match expressions,
                       written from published protocol specifications.

Output
------
  ../../crates/prtip-core/data/service-probes.txt

The overlay declares which IANA service names each probe applies to; this script
expands those names into the concrete port list the ProRT-IP probe format wants.
That keeps port assignments authoritative and refreshable (re-run fetch-iana.sh)
while the protocol knowledge stays in one reviewable hand-written file.

Usage:
    python3 generate.py [--check]

    --check  regenerate into memory and diff against the committed corpus;
             exit non-zero if they differ (suitable for CI).
"""

from __future__ import annotations

import argparse
import datetime
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
IANA_TSV = os.path.join(HERE, "data", "iana-ports.tsv")
OVERLAY = os.path.join(HERE, "overlay.txt")
OUTPUT = os.path.normpath(
    os.path.join(HERE, "..", "..", "crates", "prtip-core", "data", "service-probes.txt")
)

FIELD_RE = re.compile(r"^(?:[pvihod]/[^/]*/|cpe:/\S*)$")


class OverlayError(Exception):
    pass


def load_iana(path: str) -> tuple[dict[tuple[str, str], list[int]], str]:
    """Return {(service_name, transport): [ports]} plus the retrieval date."""
    table: dict[tuple[str, str], list[int]] = {}
    retrieved = "unknown"
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            if line.startswith("#"):
                if line.startswith("# Retrieved:"):
                    retrieved = line.split(":", 1)[1].strip()
                continue
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 3:
                continue
            name, port, proto = parts[0], parts[1], parts[2]
            table.setdefault((name, proto), []).append(int(port))
    return table, retrieved


class Probe:
    def __init__(self, transport: str, name: str, payload: str, lineno: int):
        self.transport = transport
        self.name = name
        self.payload = payload
        self.lineno = lineno
        self.rarity: int | None = None
        self.iana: list[str] = []
        self.extra_ports: list[int] = []
        self.ssl_ports: list[int] = []
        self.rules: list[str] = []  # match / softmatch lines, verbatim, in order

    @property
    def matches(self) -> int:
        return sum(1 for r in self.rules if r.startswith("match "))

    @property
    def softmatches(self) -> int:
        return sum(1 for r in self.rules if r.startswith("softmatch "))


def lint_rule(rule: str, lineno: int) -> None:
    """Reject anything the ProRT-IP probe parser would silently drop."""
    kind, rest = rule.split(" ", 1)
    if " m|" not in rest:
        raise OverlayError(f"line {lineno}: {kind} rule has no m|...| pattern")
    _service, remainder = rest.split(" ", 1)
    if not remainder.startswith("m|"):
        raise OverlayError(f"line {lineno}: pattern must follow the service name")
    body = remainder[2:]
    end = body.find("|")
    if end < 0:
        raise OverlayError(f"line {lineno}: unterminated m|...| pattern")
    tail = body[end + 1 :].strip()
    for field in tail.split():
        if not FIELD_RE.match(field):
            raise OverlayError(
                f"line {lineno}: field {field!r} is unparseable "
                "(values must be x/value/ with no spaces, or cpe:/...)"
            )


def parse_overlay(path: str) -> list[Probe]:
    probes: list[Probe] = []
    current: Probe | None = None
    with open(path, encoding="utf-8") as fh:
        for lineno, raw in enumerate(fh, 1):
            line = raw.rstrip("\n")
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue

            if stripped.startswith("probe "):
                rest = stripped[len("probe ") :]
                try:
                    transport, rest = rest.split(" ", 1)
                    name, payload = rest.split(" q|", 1)
                except ValueError:
                    raise OverlayError(f"line {lineno}: malformed probe header")
                if transport not in ("TCP", "UDP"):
                    raise OverlayError(f"line {lineno}: transport must be TCP or UDP")
                if not payload.endswith("|"):
                    raise OverlayError(f"line {lineno}: probe payload must end with |")
                payload = payload[:-1]
                if "|" in payload:
                    raise OverlayError(
                        f"line {lineno}: payload contains a literal | (use \\x7c)"
                    )
                current = Probe(transport, name, payload, lineno)
                probes.append(current)
                continue

            if current is None:
                raise OverlayError(f"line {lineno}: directive outside a probe block")

            key, _, value = stripped.partition(" ")
            value = value.strip()
            if key == "rarity":
                current.rarity = int(value)
            elif key == "iana":
                current.iana.extend(value.split())
            elif key == "extra-ports":
                current.extra_ports.extend(int(p) for p in value.split(",") if p.strip())
            elif key == "sslports":
                current.ssl_ports.extend(int(p) for p in value.split(",") if p.strip())
            elif key in ("match", "softmatch"):
                lint_rule(stripped, lineno)
                current.rules.append(stripped)
            else:
                raise OverlayError(f"line {lineno}: unknown directive {key!r}")
    return probes


def resolve_ports(probe: Probe, iana: dict[tuple[str, str], list[int]]) -> list[int]:
    transport = probe.transport.lower()
    ports: set[int] = set(probe.extra_ports)
    for name in probe.iana:
        found = iana.get((name, transport))
        if not found:
            raise OverlayError(
                f"probe {probe.name}: IANA registry has no {transport} assignment "
                f"for service name {name!r}"
            )
        ports.update(found)
    return sorted(ports)


def render(probes: list[Probe], iana: dict, retrieved: str) -> str:
    total_matches = sum(p.matches for p in probes)
    total_soft = sum(p.softmatches for p in probes)
    tcp = sum(1 for p in probes if p.transport == "TCP")
    udp = sum(1 for p in probes if p.transport == "UDP")
    resolved = {p.name: resolve_ports(p, iana) for p in probes}
    covered = sorted({port for ports in resolved.values() for port in ports})

    out: list[str] = []
    w = out.append
    w("# ProRT-IP service probe corpus")
    w("#")
    w("# GENERATED FILE -- do not edit by hand.")
    w("# Regenerate with: python3 tools/gen-service-probes/generate.py")
    w("# Probe payloads and match expressions live in tools/gen-service-probes/overlay.txt")
    w("#")
    w("# Copyright (c) ProRT-IP contributors.")
    w("# SPDX-License-Identifier: GPL-3.0-or-later")
    w("# Licensed under the GNU General Public License v3.0, the same licence as")
    w("# the rest of ProRT-IP. See LICENSE at the repository root.")
    w("#")
    w("# Sources")
    w("# -------")
    w("#   * IANA Service Name and Transport Protocol Port Number Registry")
    w("#     https://www.iana.org/assignments/service-names-port-numbers/"
      "service-names-port-numbers.csv")
    w("#     retrieved %s -- supplies every port assignment below." % retrieved)
    w("#   * Published protocol specifications (IETF RFCs, OASIS MQTT, UPnP Device")
    w("#     Architecture, Microsoft Open Specifications, and vendor protocol")
    w("#     documentation) -- supply every probe payload and match expression.")
    w("#")
    w("# This corpus contains NO content derived from any Nmap Public Source License")
    w("# work (nmap-service-probes, nmap-os-db) or from any other source whose")
    w("# licence is incompatible with GPL-3.0. See")
    w("# crates/prtip-core/data/ATTRIBUTION.md for the full provenance record.")
    w("#")
    w("# Coverage (honest counts, not aspirational)")
    w("# ------------------------------------------")
    w("#   probes .......... %d (%d TCP, %d UDP)" % (len(probes), tcp, udp))
    w("#   match rules ..... %d" % total_matches)
    w("#   softmatch rules . %d" % total_soft)
    w("#   distinct ports .. %d" % len(covered))
    w("#")
    w("# This is a young corpus. It targets the services that dominate real scan")
    w("# results; it does not attempt exhaustive coverage and it will miss")
    w("# uncommon or proprietary services.")
    w("#")
    w("# Format is documented in tools/gen-service-probes/FORMAT.md.")
    w("")

    for probe in probes:
        ports = resolved[probe.name]
        w("Probe %s %s q|%s|" % (probe.transport, probe.name, probe.payload))
        if ports:
            w("ports %s" % ",".join(str(p) for p in ports))
        if probe.ssl_ports:
            w("sslports %s" % ",".join(str(p) for p in sorted(set(probe.ssl_ports))))
        if probe.rarity is not None:
            w("rarity %d" % probe.rarity)
        for rule in probe.rules:
            w(rule)
        w("")

    return "\n".join(out)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--check",
        action="store_true",
        help="fail if the committed corpus differs from a fresh generation",
    )
    args = ap.parse_args()

    try:
        iana, retrieved = load_iana(IANA_TSV)
        probes = parse_overlay(OVERLAY)
        text = render(probes, iana, retrieved)
    except OverlayError as exc:
        print("error: %s" % exc, file=sys.stderr)
        return 2

    if args.check:
        try:
            with open(OUTPUT, encoding="utf-8") as fh:
                existing = fh.read()
        except FileNotFoundError:
            print("error: %s does not exist" % OUTPUT, file=sys.stderr)
            return 1
        if existing != text:
            print(
                "error: %s is out of date; re-run tools/gen-service-probes/generate.py"
                % OUTPUT,
                file=sys.stderr,
            )
            return 1
        print("ok: corpus is up to date")
        return 0

    os.makedirs(os.path.dirname(OUTPUT), exist_ok=True)
    with open(OUTPUT, "w", encoding="utf-8") as fh:
        fh.write(text)

    total_matches = sum(p.matches for p in probes)
    total_soft = sum(p.softmatches for p in probes)
    ports = sorted({p for probe in probes for p in resolve_ports(probe, iana)})
    print(
        "wrote %s\n  %d probes, %d match rules, %d softmatch rules, %d distinct ports"
        % (OUTPUT, len(probes), total_matches, total_soft, len(ports)),
        file=sys.stderr,
    )
    print("  generated %s" % datetime.date.today().isoformat(), file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
