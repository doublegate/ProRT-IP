# The ProRT-IP port priority rule

This document is the specification. `generate.py` implements it; if the two
disagree, this file is the bug report and the code is the bug.

## What this list is, and what it is not

`PRIORITY_PORTS_100` and `PRIORITY_PORTS_1000` are an **editorial ranking of IANA
port assignments**. They are produced by applying the deterministic rule below to
a committed snapshot of the IANA Service Name and Transport Protocol Port Number
Registry.

They are **not** a frequency ranking. ProRT-IP makes no claim about how often any
port in this list is actually open on the Internet, and none of the inputs to the
rule is a measurement. Two consequences follow, and both are intentional:

- `-F` and `--top-ports N` select a **different set of ports** than Nmap,
  Masscan, RustScan or any other scanner whose list comes from measured
  frequency data. Results are not comparable port-for-port.
- The ordering carries no hit-rate or coverage guarantee. If you need a specific
  port scanned, name it with `-p`.

### Why there is no measured ordering

A measured ordering would require a port-frequency dataset that ProRT-IP may
redistribute under GPL-3.0. The obvious candidates do not qualify:

- `nmap-services` is covered by the Nmap Public Source License. The NPSL treats a
  work that reads or includes Nmap's data files as a Derivative Work and forbids
  distributing such a work under plain GPL terms. ProRT-IP therefore neither
  ships it, reads it, nor derives an ordering from it.
- scans.io restricts its data to non-commercial use. Censys restricts research
  access to non-commercial use and forbids redistribution. GPL-3.0 guarantees
  commercial use and redistribution, so a non-commercial restriction cannot be
  carried by a GPL-3.0 work.

No lawfully redistributable measured-frequency dataset was available, so the
project chose an ordering it can fully justify from first principles instead of
one it cannot ship.

It is worth noting that a frequency-ordered list is a weaker heuristic than its
reputation suggests. Izhikevich, Teixeira and Durumeric, *LZR: Identifying
Unexpected Internet Services* (USENIX Security 2021,
<https://www.usenix.org/conference/usenixsecurity21/presentation/izhikevich>),
found that only about 3% of HTTP services run on port 80, and that protocol
deployment is far more diffuse across ports than port-based assumptions allow
for. Ordering by popularity helps; it does not substitute for scanning the ports
you actually care about, and no ordering of 100 ports out of 65,535 is a
substitute either.

## Input

The single input is `tools/gen-service-probes/data/iana-ports.tsv`, the committed
snapshot shared with the service-probe generator. Columns:
`service`, `port`, `transport`, `description`. The snapshot already excludes port
ranges, non-TCP/UDP transports, and rows whose description is `Reserved`,
`Unassigned` or `De-registered`.

No other input exists. In particular the rule does not read, and must never read,
`nmap-services`, `nmap-service-probes`, `nmap-os-db`, or any other NPSL-covered
file, and does not consult any previous version of the port arrays.

## The rule

### Step 1 — Universe

The candidate set is every distinct **port number** that carries at least one TCP
or UDP assignment in the snapshot. A port with no assignment is not a candidate
at any length.

### Step 2 — Port record

For each candidate port `p`, take every snapshot row for `p` in file order and
build:

- `haystack(p)` — the lowercased concatenation of every `service` and
  `description` value for `p`, separated by spaces.
- `has_tcp(p)` — true if any row for `p` has transport `tcp`.

### Step 3 — Range class (RFC 6335 section 6)

| class | range | IANA name |
|---|---|---|
| 0 | 1–1023 | System Ports (well-known) |
| 1 | 1024–49151 | User Ports (registered) |
| 2 | 49152–65535 | Dynamic / Private Ports |

Lower is better. A System Port assignment went through IETF Review or IESG
Approval; a User Port assignment went through Expert Review; a Dynamic Port is
never assigned at all. That is a statement about the registry's own process, not
about deployment.

### Step 4 — Transport class

| class | condition |
|---|---|
| 0 | `has_tcp(p)` |
| 1 | UDP-only assignment |

Lower is better. ProRT-IP's default scan types and every one of its stealth scan
types are TCP; a UDP-only assignment is a weaker candidate for a list whose main
consumer is a TCP scan.

### Step 5 — Service family

The rule carries an **ordered table of thirteen service families**. Each family
owns an ordered list of case-insensitive regular expressions. Port `p` is
assigned to the **first family in table order** that has any pattern matching
`haystack(p)`. First match wins, so a port belongs to exactly one family and the
assignment is total and deterministic. A port matched by no family is assigned to
the catch-all family `other`.

The families, in table order:

| # | family | what it covers | why it ranks here |
|---|---|---|---|
| 1 | `remote-access` | SSH, Telnet, RDP, VNC, X11, rlogin/rexec, Citrix | Interactive control of the host. Finding one open is the single most consequential result a port scan can produce, for operator and adversary alike. |
| 2 | `web` | HTTP, HTTPS, HTTP-alt, web proxies, WebSocket | The largest application surface on the modern Internet, and the one most likely to carry further attack surface behind it. |
| 3 | `file-sharing` | SMB/CIFS, NFS, FTP/FTPS/TFTP/SFTP, AFP, rsync, WebDAV | Direct data exposure and the classic lateral-movement path. |
| 4 | `database` | MySQL, PostgreSQL, MSSQL, Oracle, MongoDB, Redis, memcached, Elasticsearch, ... | A database reachable from the scanning vantage point is nearly always a misconfiguration worth reporting. |
| 5 | `mail` | SMTP, POP3, IMAP, submission, and their TLS variants | Ubiquitous, externally reachable by design, and a standing abuse target. |
| 6 | `name-service` | DNS, mDNS, LLMNR, NetBIOS name/datagram, WHOIS | Infrastructure services that map a network for you once found. |
| 7 | `directory-auth` | LDAP, Kerberos, RADIUS, TACACS+, NIS, ident | Identity infrastructure; high value, narrow deployment. |
| 8 | `management-discovery` | SNMP, IPMI, syslog, NTP, WBEM, WS-Management/WinRM, UPnP/SSDP, SLP, and RPC endpoint mappers (sunrpc/portmapper, DCE `epmap`) | Out-of-band management and the service-location surfaces that enumerate everything else on the host. |
| 9 | `voice-video` | SIP, RTSP, RTP/RTCP, H.323, MGCP/SCCP, streaming | A large, well-defined family with a distinct operator population. |
| 10 | `messaging` | AMQP, MQTT, XMPP, IRC, STOMP, Kafka, NNTP | Broker and queue surfaces, increasingly exposed by IoT and microservice deployments. |
| 11 | `print` | IPP, LPD/LPR, JetDirect, CUPS | Small, unambiguous family; historically an easy foothold. |
| 12 | `network-infra` | BGP, DHCP/BOOTP, OSPF, IKE/ISAKMP/IPsec, L2TP/PPTP/OpenVPN/WireGuard, SOCKS, tunnels | Routing and tunnelling endpoints. Ranks last of the classified families because most of it is UDP or unreachable from a generic vantage point. |
| 13 | `other` | everything else | Not interleaved — see Step 7. |

The family order is the rule's editorial judgement and it is the only place
judgement enters. It is ordered by **how much an operator learns from finding a
port of that family open**, which is a triage question, not a frequency question.
Someone else could reasonably order these differently; what matters is that the
order is fixed, written down here, and applied mechanically.

Family membership is decided from the registry's own `service` and `description`
text and nothing else. The pattern tables live in `generate.py` under
`FAMILIES`, are the file a reviewer should read, and must be justifiable from
published protocol documentation.

### Step 6 — Order within a family

Sort the family's ports ascending by the key

```
(range_class, transport_class, port_number)
```

The port number is unique within the candidate set, so this is a strict total
order. **Every tie is ultimately broken by ascending port number and by nothing
else.**

### Step 7 — Interleave

Emit the twelve classified families in **rounds**. Round `i` (0-based) emits, in
family-table order, the `i`-th port of every family that still has one. A family
that has run out is skipped for the remainder.

The interleave is what makes a short prefix useful: the first round is one port
per family, so `--top-ports 12` spans all twelve families rather than exhausting
`remote-access`. Concatenating the families instead would put roughly a hundred
remote-access ports ahead of the first web port.

`other` does **not** participate in the interleave. It is appended after every
classified port, ordered by the Step 6 key. A registry entry whose service name
and description match none of the thirteen families gives the scanner no
articulable reason to prefer it, so it ranks below every classified assignment.

### Step 8 — Output

- `PRIORITY_PORTS_100` = the first 100 entries of the sequence.
- `PRIORITY_PORTS_1000` = the first 1000 entries of the sequence.

By construction the 100-list is a prefix of the 1000-list, so
`get_priority_ports(n)` is a simple prefix for every `n <= 1000`.

## What the rule actually yields

The first 20 entries against the 2026-08-27 snapshot, which is the whole rule
visible at once — one port per family, family-table order, repeating:

| # | port | family | class | IANA name / description |
|---|---|---|---|---|
| 1 | 22 | remote-access | well-known | `ssh` — The Secure Shell (SSH) Protocol |
| 2 | 80 | web | well-known | `http` — World Wide Web HTTP |
| 3 | 20 | file-sharing | well-known | `ftp-data` — File Transfer [Default Data] |
| 4 | 65 | database | well-known | `tacacs-ds` — TACACS-Database Service |
| 5 | 25 | mail | well-known | `smtp` — Simple Mail Transfer |
| 6 | 42 | name-service | well-known | `name` — Host Name Server |
| 7 | 31 | directory-auth | well-known | `msg-auth` — MSG Authentication |
| 8 | 111 | management-discovery | well-known | `sunrpc` — SUN Remote Procedure Call |
| 9 | 537 | voice-video | well-known | `nmsp` — Networked Media Streaming Protocol |
| 10 | 119 | messaging | well-known | `nntp` — Network News Transfer Protocol |
| 11 | 92 | print | well-known | `npp` — Network Printing Protocol |
| 12 | 67 | network-infra | well-known | `bootps` — Bootstrap Protocol Server |
| 13 | 23 | remote-access | well-known | `telnet` — Telnet |
| 14 | 280 | web | well-known | `http-mgmt` |
| 15 | 21 | file-sharing | well-known | `ftp` — File Transfer Protocol [Control] |
| 16 | 66 | database | well-known | `sql*net` — Oracle SQL*NET |
| 17 | 50 | mail | well-known | `re-mail-ck` — Remote Mail Checking Protocol |
| 18 | 43 | name-service | well-known | `nicname` — WHOIS |
| 19 | 49 | directory-auth | well-known | `tacacs` — Login Host Protocol (TACACS) |
| 20 | 123 | management-discovery | well-known | `ntp` — Network Time Protocol |

Reproduce this at any length with:

```bash
python3 tools/gen-top-ports/generate.py --explain 20
```

Two things this table makes obvious, and both are the rule working as specified
rather than defects to patch:

- **Rank 4 is port 65, not port 3306.** `tacacs-ds` is described in the registry
  as "TACACS-Database Service", so it matches the `database` family before the
  lower-ranked `directory-auth` family gets a look. First-match-wins over an
  ordered table is what makes the classification total and deterministic; the
  cost is occasional counter-intuitive placements like this one. Special-casing
  it would mean encoding an opinion about which service is *really* important,
  which is precisely the frequency judgement this rule refuses to make.
- **Some obviously important ports rank far down.** Port 3389 (RDP) lands at 309
  and port 8080 (HTTP-alt) at 532, because within a family the well-known range
  is exhausted before the registered range and the registered range is walked in
  ascending numeric order. A frequency-derived list would put both in its first
  hundred. This one does not, and that is the visible, intended consequence of
  having no frequency data. Both are still inside `--top-ports 1000`.

## Determinism contract

Same snapshot plus same `generate.py` produces byte-identical output. Nothing in
the rule reads the clock, the filesystem beyond the snapshot, an environment
variable, or an unordered iteration. `generate.py --check` is the CI gate that
asserts the committed arrays match a fresh run.

Refreshing the snapshot with `tools/gen-service-probes/fetch-iana.sh` will change
the output whenever IANA adds, removes or re-describes an assignment. That is the
intended way for the list to evolve, and the diff is reviewable.
