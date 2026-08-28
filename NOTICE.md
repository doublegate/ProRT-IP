# Notice

## Copyright and licence

ProRT-IP WarScan — Modern Network Scanner
Copyright (C) 2025 ProRT-IP Contributors

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. A complete copy is in [`LICENSE`](LICENSE) at the root of this
repository; it is also available at <https://www.gnu.org/licenses/>.

The manifest declares `license = "GPL-3.0-or-later"`, matching the "or (at your
option) any later version" grant above.

## Acceptable use

This is a security research and penetration testing tool. Users must:

- Only use it on networks and systems they own or have explicit authorisation to
  test.
- Comply with all applicable laws and regulations.
- Use it responsibly and ethically.

Unauthorised network scanning may be illegal in your jurisdiction. The authors
and contributors assume no liability for misuse of this software.

See [`SECURITY.md`](SECURITY.md) for vulnerability reporting.

## Third-party material

ProRT-IP vendors no third-party data files. Its service-detection corpus and its
`-F` / `--top-ports` ordering are its own work, generated from the IANA Service
Name and Transport Protocol Port Number Registry and from published protocol
specifications. Per-source provenance, licences and retrieval dates are recorded
in [`crates/prtip-core/data/ATTRIBUTION.md`](crates/prtip-core/data/ATTRIBUTION.md).

ProRT-IP is interface-compatible with a number of Nmap command-line flags
(`-sS`, `-sV`, `-oX`, and others). That is an independent reimplementation of a
command-line interface and involves no Nmap code or data. This project is not
affiliated with, endorsed by, or derived from Nmap or Insecure.Com LLC.

Rust dependency licences are resolvable from `Cargo.lock`; run `cargo license` or
`cargo about` to produce a full manifest.

---

*This file exists because `LICENSE` must contain the unmodified text of the GNU
General Public License and nothing else — both because GPLv3 section 4 requires
conveying a verbatim copy, and because licence scanners cannot identify a file
that mixes the licence with project-specific notices. Everything that previously
sat alongside the licence text now lives here.*
