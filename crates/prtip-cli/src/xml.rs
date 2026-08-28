//! XML escaping for the `-oX` output formats.
//!
//! ProRT-IP builds its Nmap-compatible XML by hand rather than through an XML
//! library, so escaping is this module's responsibility and nothing else's.
//! Both XML emitters — [`crate::export`] and [`crate::output`] — must route
//! every dynamic value through [`escape_xml`].
//!
//! # Why this is security-relevant
//!
//! Service names and banners are **bytes chosen by the host being scanned**. A
//! hostile target that gets an unescaped `"` or `<` into a banner can close an
//! attribute and inject arbitrary elements into the report, which is a document
//! forgery against whatever consumes the report — a SIEM, a parser, a colleague
//! reading the file. Escaping here is the only thing preventing that.
//!
//! # What this handles beyond the five metacharacters
//!
//! Escaping `& < > " '` is necessary but not sufficient. XML 1.0 (§2.2) permits
//! only these characters:
//!
//! ```text
//! #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
//! ```
//!
//! Control characters outside that set are illegal **even as numeric character
//! references** — `&#x0;` is not valid XML. A banner containing a raw NUL, which
//! is entirely ordinary from a binary protocol, would therefore produce a
//! document that conforming parsers reject outright. Escaping only the five
//! metacharacters yields well-formed-looking output that fails to parse.
//!
//! Such characters are rendered as visible `\xNN` text, matching how Nmap
//! presents non-printable banner bytes. That keeps the document valid without
//! silently discarding evidence — which matters when the banner is the finding.

use std::fmt::Write as _;

/// Escape a string for inclusion in an XML attribute value or text node.
///
/// Escapes the five XML metacharacters, preserves the three legal whitespace
/// controls (tab, LF, CR), and renders every other character that XML 1.0
/// forbids as visible `\xNN` text.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(escape_xml("a & b"), "a &amp; b");
/// assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
/// // A NUL byte would make the document unparseable, so it is shown instead.
/// assert_eq!(escape_xml("a\0b"), "a\\x00b");
/// ```
pub fn escape_xml(s: &str) -> String {
    // Most inputs need no escaping at all; size for the common case.
    let mut out = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),

            // The only control characters XML 1.0 allows.
            '\t' | '\n' | '\r' => out.push(c),

            // C0 controls and DEL. Illegal in XML 1.0 even as numeric
            // references, so they are rendered rather than emitted.
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                // Writing to a String is infallible.
                let _ = write!(out, "\\x{:02x}", c as u32);
            }

            // Non-characters, likewise illegal.
            '\u{FFFE}' | '\u{FFFF}' => out.push_str("\\u{fffd}"),

            c => out.push(c),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_through_ordinary_text() {
        assert_eq!(escape_xml("test"), "test");
        assert_eq!(escape_xml(""), "");
        assert_eq!(escape_xml("OpenSSH 9.6p1 Ubuntu"), "OpenSSH 9.6p1 Ubuntu");
    }

    #[test]
    fn escapes_the_five_metacharacters() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"quote\""), "&quot;quote&quot;");
        assert_eq!(escape_xml("it's"), "it&apos;s");
    }

    #[test]
    fn escapes_ampersand_first() {
        // Escaping `<` before `&` would turn "&lt;" into "&amp;lt;" on a second
        // pass. A char-by-char walk cannot make that mistake, and this test
        // pins the property so a future rewrite to chained `replace` cannot
        // reintroduce it.
        assert_eq!(escape_xml("&lt;"), "&amp;lt;");
        assert_eq!(escape_xml("&amp;"), "&amp;amp;");
    }

    #[test]
    fn preserves_legal_whitespace_controls() {
        assert_eq!(escape_xml("a\tb\nc\rd"), "a\tb\nc\rd");
    }

    #[test]
    fn renders_illegal_control_characters() {
        // These are legal in a Rust string and legal on the wire, but illegal
        // in XML 1.0 -- even as `&#x0;`. Emitting them raw produces a document
        // that conforming parsers reject.
        assert_eq!(escape_xml("a\0b"), "a\\x00b");
        assert_eq!(escape_xml("\x01\x02"), "\\x01\\x02");
        assert_eq!(escape_xml("bell\x07"), "bell\\x07");
        assert_eq!(escape_xml("esc\x1b[0m"), "esc\\x1b[0m");
        assert_eq!(escape_xml("del\x7f"), "del\\x7f");
    }

    #[test]
    fn output_contains_no_illegal_characters_for_any_input() {
        // Negative control: walk every code point a byte-oriented banner can
        // produce and assert the escaped form is XML 1.0 clean.
        let hostile: String = (0u8..=0xff).map(|b| b as char).collect();
        let escaped = escape_xml(&hostile);

        for c in escaped.chars() {
            let cp = c as u32;
            let legal = cp == 0x9
                || cp == 0xA
                || cp == 0xD
                || (0x20..=0xD7FF).contains(&cp)
                || (0xE000..=0xFFFD).contains(&cp)
                || (0x10000..=0x10FFFF).contains(&cp);
            assert!(legal, "illegal XML 1.0 character in output: U+{:04X}", cp);
        }
    }

    #[test]
    fn hostile_banner_cannot_break_out_of_an_attribute() {
        // The actual attack: end the attribute, close the element, inject.
        let attack = r#"" /><script>alert(1)</script><x a=""#;
        let escaped = escape_xml(attack);

        assert!(
            !escaped.contains('"'),
            "unescaped quote closes the attribute"
        );
        assert!(
            !escaped.contains('<'),
            "unescaped angle bracket opens a tag"
        );
        assert!(
            !escaped.contains('>'),
            "unescaped angle bracket closes a tag"
        );
    }

    #[test]
    fn renders_non_characters() {
        assert_eq!(escape_xml("a\u{FFFE}b"), "a\\u{fffd}b");
        assert_eq!(escape_xml("a\u{FFFF}b"), "a\\u{fffd}b");
    }

    #[test]
    fn preserves_legitimate_multibyte_text() {
        // U+D7FF and U+E000 sit either side of the surrogate hole; both legal.
        assert_eq!(escape_xml("héllo wörld"), "héllo wörld");
        assert_eq!(escape_xml("日本語"), "日本語");
        assert_eq!(escape_xml("\u{D7FF}\u{E000}"), "\u{D7FF}\u{E000}");
        assert_eq!(escape_xml("emoji 🦀"), "emoji 🦀");
    }
}
