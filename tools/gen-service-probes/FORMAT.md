# ProRT-IP service probe format

The grammar accepted by `prtip_core::service_db::ServiceProbeDb::parse`. It is a
line-oriented plain-text format; blank lines and `#` comments are ignored, and
every line is trimmed before parsing.

```text
Probe <TCP|UDP> <Name> q|<payload>|
ports <port-list>
sslports <port-list>
rarity <1-9>
match <service> m|<regex>| [p/product/] [v/version/] [i/info/] [h/hostname/] [o/ostype/] [d/devicetype/] [cpe:/...]
softmatch <service> m|<regex>| [fields...]
```

* `Probe` opens a block. Every following `ports` / `sslports` / `rarity` /
  `match` / `softmatch` line belongs to it until the next `Probe`.
* `<payload>` is the bytes sent on connect. An empty payload (`q||`) means
  "connect and read whatever the server sends first".
* `ports` is a comma-separated list; ranges like `80-85` are expanded. A probe
  with no `ports` line is treated as generic and is attempted against every port.
* `rarity` orders probes (1 first). Probes with `rarity <= 3` are also used as a
  fallback on ports that matched no probe, so keep low rarities for broadly
  applicable probes only.
* `match` rules are evaluated in file order; all `softmatch` rules are evaluated
  only after every `match` rule has failed. The first hit wins, so order from
  most specific to least.

## Payload escapes

`\r` `\n` `\t` `\0` `\\` and `\xHH` (exactly two hex digits). Every other
character is emitted as its literal byte, so payloads must be ASCII apart from
`\xHH` escapes.

## Constraints (violating any of these makes the parser drop the rule silently)

1. **No literal `|`** in a payload or a regex. The delimiters are found by
   scanning for the first `|`. Use `\x7c` in a payload; restructure the regex
   (character class, or a second `match` line) instead of using alternation.
2. **No spaces inside `p/…/ v/…/ i/…/ h/…/ o/…/ d/…/`.** The tail of a match
   line is split on whitespace, so a value containing a space loses its trailing
   `/` and the whole field is discarded. Use `-` between words.
3. **Regexes are Rust `regex` syntax.** No backreferences, no lookaround. A
   pattern that fails to compile causes the rule to be dropped without warning —
   `test_embedded_corpus_fully_parses` in `service_db.rs` exists to catch that.
4. **`.` does not match a newline.** Use `[\s\S]` to span lines.
5. **Responses are matched after `String::from_utf8_lossy`.** Every byte that is
   not valid UTF-8 becomes U+FFFD, and a *run* of invalid bytes may collapse to a
   *single* U+FFFD. Anchor binary patterns on ASCII-range bytes and use `[\s\S]`
   (never a fixed count of high bytes) where non-ASCII can appear.

## Capture substitution

`$1` … `$9` in `p/`, `v/` and `i/` values are replaced with the corresponding
regex capture group from the matched response.

## Relationship to other formats

The layout is a deliberately small, independently reimplemented line format. It
is *not* Nmap's format, ProRT-IP does not read Nmap's data files, and no rule in
the corpus is derived from one. See `crates/prtip-core/data/ATTRIBUTION.md`.
