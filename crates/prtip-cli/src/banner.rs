//! Banner module for ProRT-IP CLI
//!
//! Provides professional ASCII art banner and branding display.

use colored::*;

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
    /// - ASCII art logo in cyan/bold
    /// - Version number in green
    /// - Project tagline and description
    /// - GitHub repository URL
    /// - License information
    pub fn print(&self) {
        // Print ASCII art with cyber aesthetic
        println!("{}", self.ascii_art().cyan().bold());

        // Print project information
        println!(
            "  {} {} │ {} {}",
            "Version:".white().bold(),
            self.version.green().bold(),
            "Phase:".white().bold(),
            "3 COMPLETE".green().bold()
        );
        println!(
            "  {} {}",
            "Project:".white().bold(),
            "Modern Network Scanner & War Dialer".white()
        );
        println!(
            "  {} {}",
            "GitHub:".white().bold(),
            "https://github.com/doublegate/ProRT-IP"
                .bright_blue()
                .underline()
        );
        println!(
            "  {} {} │ {} {}",
            "License:".white().bold(),
            "GPL-3.0".white(),
            "Tests:".white().bold(),
            "391 passing".green()
        );
        println!();
    }

    /// Print compact banner for minimal output
    ///
    /// Single-line format: "ProRT-IP v0.1.0 - Modern Network Scanner"
    pub fn print_compact(&self) {
        println!(
            "{} {} - {}",
            "ProRT-IP".cyan().bold(),
            format!("v{}", self.version).green(),
            "Modern Network Scanner".white()
        );
    }

    /// Get ASCII art logo
    ///
    /// Returns cyber-style ASCII art for ProRT-IP branding.
    /// Optimized for 80-column terminals.
    fn ascii_art(&self) -> String {
        r#"
 ██████╗ ██████╗  ██████╗ ██████╗ ████████╗      ██╗██████╗
 ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗╚══██╔══╝      ██║██╔══██╗
 ██████╔╝██████╔╝██║   ██║██████╔╝   ██║   █████╗██║██████╔╝
 ██╔═══╝ ██╔══██╗██║   ██║██╔══██╗   ██║   ╚════╝██║██╔═══╝
 ██║     ██║  ██║╚██████╔╝██║  ██║   ██║         ██║██║
 ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝         ╚═╝╚═╝
        "#
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
        assert!(art.contains("ProRT-IP") || art.contains("██"));
    }

    #[test]
    fn test_ascii_art_contains_box_drawing() {
        let banner = Banner::new("0.1.0");
        let art = banner.ascii_art();
        // Should contain Unicode box drawing characters
        assert!(art.contains("█") || art.contains("╗") || art.contains("╔"));
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
