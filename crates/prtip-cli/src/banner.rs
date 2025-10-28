//! Banner module for ProRT-IP CLI
//!
//! Provides aggressive cyber-punk graffiti-style ASCII art banner and branding display.

use colored::Colorize;

/// Banner display for ProRT-IP WarScan
///
/// Manages cyber-punk ASCII art branding, version information, and project details.
/// Features aggressive multi-color graffiti aesthetic with block characters.
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

    /// Print the full banner with cyber-punk ASCII art, version, and project information
    ///
    /// Displays:
    /// - Multi-color cyber-punk graffiti ASCII art (cyan → magenta → red → yellow → green)
    /// - Heavy block characters (██, ╔, ╗, ║, ═) for aggressive aesthetic
    /// - Version number and project status
    /// - GitHub repository URL
    /// - License and test information
    /// - Cyber-punk separators and tech symbols (━, ▸, │, ⚡)
    pub fn print(&self) {
        println!("{}", self.ascii_art());
        println!();

        // Cyber-punk styled info with tech separators
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                .bright_cyan()
        );
        println!(
            "  {} {} {} {} {} {}",
            "▸".bright_cyan().bold(),
            "Version:".bright_white().bold(),
            self.version.bright_green(),
            "│".bright_black(),
            "Phase:".bright_white().bold(),
            "4 COMPLETE".bright_green().bold()
        );
        println!(
            "  {} {} {} {} {} {}",
            "▸".bright_magenta().bold(),
            "GitHub:".bright_white().bold(),
            "https://github.com/doublegate/ProRT-IP"
                .bright_blue()
                .underline(),
            "│".bright_black(),
            "Tests:".bright_white().bold(),
            "1,338 passing".bright_green()
        );
        println!(
            "  {} {} {}",
            "▸".bright_red().bold(),
            "License:".bright_white().bold(),
            "GPL-3.0".bright_yellow()
        );
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                .bright_cyan()
        );
        println!();
        println!(
            "{}",
            "  ⚡ Protocol/Port Real-Time War Scanner for IP Networks"
                .bright_white()
                .bold()
        );
    }

    /// Print compact banner for minimal output
    ///
    /// Single-line format: "⟨ProRT-IP⟩ v0.1.0 ─ Network Scanner"
    pub fn print_compact(&self) {
        println!(
            "{} {} {} {}",
            "⟨ProRT-IP⟩".bright_cyan().bold(),
            self.version.bright_green(),
            "─".bright_black(),
            "Network Scanner".bright_white()
        );
    }

    /// Get cyber-punk graffiti ASCII art logo
    ///
    /// Returns aggressive multi-color ASCII art with heavy block characters.
    /// Cyber-punk aesthetic: NOT bubbly, edgy and aggressive.
    /// Color gradient: cyan → magenta → red → yellow → green
    /// Uses box drawing characters (╔, ╗, ║, ═) and blocks (██)
    fn ascii_art(&self) -> String {
        // Cyber-punk multi-color graffiti style with heavy blocks
        let line1 = " ██████╗ ██████╗  ██████╗ ██████╗ ████████╗     ██╗██████╗ "
            .bright_cyan()
            .bold();
        let line2 = " ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗╚══██╔══╝     ██║██╔══██╗"
            .bright_magenta()
            .bold();
        let line3 = " ██████╔╝██████╔╝██║   ██║██████╔╝   ██║  █████╗██║██████╔╝"
            .bright_red()
            .bold();
        let line4 = " ██╔═══╝ ██╔══██╗██║   ██║██╔══██╗   ██║  ╚════╝██║██╔═══╝ "
            .bright_yellow()
            .bold();
        let line5 = " ██║     ██║  ██║╚██████╔╝██║  ██║   ██║        ██║██║     "
            .bright_green()
            .bold();
        let line6 = " ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝        ╚═╝╚═╝     "
            .white()
            .dimmed();

        let line7 = " ██╗    ██╗ █████╗ ██████╗ ███████╗ ██████╗ █████╗ ███╗   ██╗"
            .bright_cyan()
            .bold();
        let line8 = " ██║    ██║██╔══██╗██╔══██╗██╔════╝██╔════╝██╔══██╗████╗  ██║"
            .bright_magenta()
            .bold();
        let line9 = " ██║ █╗ ██║███████║██████╔╝███████╗██║     ███████║██╔██╗ ██║"
            .bright_red()
            .bold();
        let line10 = " ██║███╗██║██╔══██║██╔══██╗╚════██║██║     ██╔══██║██║╚██╗██║"
            .bright_yellow()
            .bold();
        let line11 = " ╚███╔███╔╝██║  ██║██║  ██║███████║╚██████╗██║  ██║██║ ╚████║"
            .bright_green()
            .bold();
        let line12 = "  ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ ╚═════╝╚═╝  ╚═╝╚═╝  ╚═══╝"
            .white()
            .dimmed();

        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n\n{}\n{}\n{}\n{}\n{}\n{}",
            line1, line2, line3, line4, line5, line6, line7, line8, line9, line10, line11, line12
        )
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
    fn test_ascii_art_multicolor() {
        use colored::control;

        // Force colors for testing
        control::set_override(true);

        let banner = Banner::new("0.1.0");
        let art = banner.ascii_art();
        // Should contain ANSI color escape codes
        assert!(art.contains('\x1b'));

        // Reset color override
        control::unset_override();
    }

    #[test]
    fn test_ascii_art_contains_blocks() {
        let banner = Banner::new("0.1.0");
        let art = banner.ascii_art();
        // Should contain block characters for cyber-punk graffiti style
        assert!(art.contains('█'));
        // Should contain box drawing characters
        assert!(art.contains('╔') || art.contains('╗') || art.contains('║') || art.contains('═'));
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
        // ASCII art should be multi-line (12 lines for cyber-punk design)
        assert!(art.lines().count() >= 12);
    }

    #[test]
    fn test_ascii_art_cyber_punk_style() {
        let banner = Banner::new("0.1.0");
        let art = banner.ascii_art();
        // Should NOT contain old RustScan-style characters
        assert!(!art.contains("{}"));
        // Should contain cyber-punk block characters
        assert!(art.contains('█'));
    }
}
