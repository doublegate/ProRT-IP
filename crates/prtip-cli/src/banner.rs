//! Banner module for ProRT-IP CLI
//!
//! Provides professional ASCII art banner and branding display.

use colorful::{Color, Colorful};

/// Banner display for ProRT-IP WarScan
///
/// Manages ASCII art branding, version information, and project details.
/// Automatically suppresses banner in quiet mode or piped output.
pub struct Banner {
    version: String,
}

impl Banner {
    /// Create a new banner with version information
    ///
    /// # Arguments
    ///
    /// * `version` - Version string (typically from CARGO_PKG_VERSION)
    ///
    /// # Examples
    ///
    /// ```
    /// use prtip_cli::banner::Banner;
    ///
    /// let banner = Banner::new(env!("CARGO_PKG_VERSION"));
    /// banner.print();
    /// ```
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
        }
    }

    /// Print the full banner with ASCII art, version, and project information
    ///
    /// Displays:
    /// - ASCII art logo in green gradient (RustScan style)
    /// - Version number in green
    /// - Project tagline and description
    /// - GitHub repository URL
    /// - License information
    pub fn print(&self) {
        use colored::Colorize as C;

        // Print ASCII art with RustScan-style green gradient
        println!("{}", self.ascii_art().gradient(Color::Green).bold());
        println!(
            "{}",
            C::bright_white("The Modern Network Scanner & War Dialer")
        );
        println!();

        // Print project information
        println!(
            "  {} {} │ {} {}",
            C::white("Version:").bold(),
            C::green(self.version.as_str()).bold(),
            C::white("Phase:").bold(),
            C::green("3 COMPLETE").bold()
        );
        println!(
            "  {} {}",
            C::white("GitHub:").bold(),
            C::bright_blue("https://github.com/doublegate/ProRT-IP").underline()
        );
        println!(
            "  {} {} │ {} {}",
            C::white("License:").bold(),
            C::white("GPL-3.0"),
            C::white("Tests:").bold(),
            C::green("391 passing")
        );
        println!();
    }

    /// Print compact banner for minimal output
    ///
    /// Single-line format: "ProRT-IP v0.1.0 - Modern Network Scanner"
    pub fn print_compact(&self) {
        use colored::Colorize as C;
        let version_str = format!("v{}", self.version);
        println!(
            "{} {} - {}",
            C::cyan("ProRT-IP").bold(),
            C::green(version_str.as_str()),
            C::white("Modern Network Scanner")
        );
    }

    /// Get ASCII art logo
    ///
    /// Returns RustScan-style ASCII art for ProRT-IP branding.
    /// Uses only ASCII characters (no Unicode) for maximum compatibility.
    /// Optimized for 80-column terminals.
    fn ascii_art(&self) -> String {
        r#".----. .---. .----.  .---. .----.     .-. .----.
| {}  }| {}  }| {} \ | {} \{}  {}     | | | {}  }
|  __/ |     /| {} / |    /{}  {} --- | | |  __/
`-'    `-' `-'`-' `-'`-' `-'  `--'    `-' `-'    "#
            .to_string()
    }
}

impl Default for Banner {
    fn default() -> Self {
        Self::new(env!("CARGO_PKG_VERSION"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banner_creation() {
        let banner = Banner::new("0.1.0");
        assert_eq!(banner.version, "0.1.0");
    }

    #[test]
    fn test_banner_creation_with_version() {
        let banner = Banner::new("1.2.3");
        assert_eq!(banner.version, "1.2.3");
    }

    #[test]
    fn test_ascii_art_not_empty() {
        let banner = Banner::new("0.1.0");
        let art = banner.ascii_art();
        assert!(!art.is_empty());
    }

    #[test]
    fn test_ascii_art_contains_ascii_only() {
        let banner = Banner::new("0.1.0");
        let art = banner.ascii_art();
        // Should contain only ASCII characters (no Unicode)
        assert!(art.chars().all(|c| c.is_ascii()));
    }

    #[test]
    fn test_ascii_art_rustscan_style() {
        let banner = Banner::new("0.1.0");
        let art = banner.ascii_art();
        // Should contain RustScan-style ASCII characters
        assert!(art.contains('.'));
        assert!(art.contains('-'));
        assert!(art.contains('|'));
        assert!(art.contains('{'));
        assert!(art.contains('}'));
    }

    #[test]
    fn test_default_banner() {
        let banner = Banner::default();
        assert!(!banner.version.is_empty());
    }

    #[test]
    fn test_banner_version_storage() {
        let test_version = "2.5.8";
        let banner = Banner::new(test_version);
        assert_eq!(banner.version, test_version);
    }

    #[test]
    fn test_ascii_art_multiline() {
        let banner = Banner::new("0.1.0");
        let art = banner.ascii_art();
        // ASCII art should be multi-line
        assert!(art.lines().count() > 1);
    }

    #[test]
    fn test_ascii_art_width() {
        let banner = Banner::new("0.1.0");
        let art = banner.ascii_art();
        // Each line should fit in reasonable terminal width (check longest line)
        for line in art.lines() {
            assert!(line.len() <= 100, "Line too long: {} chars", line.len());
        }
    }
}
