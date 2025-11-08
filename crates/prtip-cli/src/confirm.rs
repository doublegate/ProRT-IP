//! Interactive confirmation system for dangerous operations
//!
//! This module provides safety confirmations for operations that could have
//! significant impact on networks or systems. Confirmations are:
//! - Skipped on non-interactive terminals (CI/CD environments)
//! - Skipped with `--yes` flag for automation
//! - Smart about what operations are truly dangerous
//!
//! # Examples
//!
//! ```no_run
//! use prtip_cli::confirm::{ConfirmationManager, ConfirmConfig};
//!
//! let manager = ConfirmationManager::new(ConfirmConfig {
//!     auto_yes: false,
//!     is_interactive: true,
//! });
//!
//! // Check for dangerous scan before executing
//! manager.confirm_scan(&config, &targets)?;
//! ```

use anyhow::{bail, Result};
use colored::*;
use prtip_core::{Config, ScanTarget, TimingTemplate};
use std::io::{stdin, stdout, Write};
use std::net::IpAddr;

/// Configuration for the confirmation system
#[derive(Debug, Clone)]
pub struct ConfirmConfig {
    /// Skip all confirmations (--yes flag)
    pub auto_yes: bool,
    /// Whether the terminal is interactive
    pub is_interactive: bool,
}

impl Default for ConfirmConfig {
    fn default() -> Self {
        Self {
            auto_yes: false,
            is_interactive: is_terminal_interactive(),
        }
    }
}

/// Manages interactive confirmations for dangerous operations
pub struct ConfirmationManager {
    config: ConfirmConfig,
}

impl ConfirmationManager {
    /// Create a new confirmation manager
    pub fn new(config: ConfirmConfig) -> Self {
        Self { config }
    }

    /// Confirm before executing a scan based on its characteristics
    ///
    /// This checks multiple danger indicators:
    /// - Internet-scale scans (large target sets or global IPs)
    /// - Aggressive timing (T5 insane mode)
    /// - Evasion techniques (fragmentation, decoys)
    /// - Large target counts (>10,000 hosts)
    /// - Running as root
    ///
    /// Returns `Ok(())` if user confirms or confirmation not needed,
    /// `Err` if user declines.
    pub fn confirm_scan(&self, config: &Config, targets: &[ScanTarget]) -> Result<()> {
        // Skip if non-interactive (CI/CD) or --yes flag
        if !self.config.is_interactive || self.config.auto_yes {
            return Ok(());
        }

        // Check for internet-scale scan
        if self.is_internet_scale_scan(targets) {
            self.confirm_internet_scale()?;
        }

        // Check for large target set
        let target_count = estimate_target_count(targets);
        if target_count > 10_000 {
            self.confirm_large_target_set(target_count, config)?;
        }

        // Check for aggressive timing
        if config.scan.timing_template == TimingTemplate::Insane {
            self.confirm_aggressive_timing()?;
        }

        // Check for evasion techniques
        if config.evasion.fragment_packets || config.evasion.decoys.is_some() {
            self.confirm_evasion_techniques(&config.evasion)?;
        }

        // Check if running as root (Unix only)
        #[cfg(unix)]
        if self.is_running_as_root() {
            self.confirm_root_privileges()?;
        }

        Ok(())
    }

    /// Check if this is an internet-scale scan (non-RFC1918 targets or huge ranges)
    fn is_internet_scale_scan(&self, targets: &[ScanTarget]) -> bool {
        // Check for non-private IPs or very large CIDR ranges
        targets.iter().any(|target| {
            let ip = target.network.ip();

            // Check if IP is public (not RFC1918 private)
            let is_public = match ip {
                IpAddr::V4(ipv4) => !is_private_ipv4(&ipv4),
                IpAddr::V6(ipv6) => {
                    // Check if private: link-local (fe80::/10) or unique local (fc00::/7) or localhost
                    let segments = ipv6.segments();
                    let is_private = (segments[0] >= 0xfe80 && segments[0] <= 0xfebf) // link-local
                        || (segments[0] >= 0xfc00 && segments[0] <= 0xfdff) // unique local
                        || ipv6 == std::net::Ipv6Addr::LOCALHOST;
                    !is_private
                }
            };

            // Or check for large CIDR ranges (> /16 for IPv4, > /48 for IPv6)
            let is_large_range = match ip {
                IpAddr::V4(_) => target.network.prefix() < 16,
                IpAddr::V6(_) => target.network.prefix() < 48,
            };

            is_public || is_large_range
        })
    }

    /// Confirm internet-scale scan
    fn confirm_internet_scale(&self) -> Result<()> {
        let warning = format!(
            "{} {}",
            "⚠".yellow().bold(),
            "Internet-Scale Scan Detected".bright_white().bold()
        );

        eprintln!("\n{}", warning);
        eprintln!("{}", "─".repeat(60).bright_yellow());
        eprintln!(
            "Scanning public IPs or large address ranges may:\n\
             • Violate your ISP's terms of service\n\
             • Trigger intrusion detection systems\n\
             • Be illegal in some jurisdictions\n\
             • Generate significant network traffic"
        );
        eprintln!("{}", "─".repeat(60).bright_yellow());

        self.prompt_user("Continue with internet-scale scan?", false)
    }

    /// Confirm large target set
    fn confirm_large_target_set(&self, count: usize, config: &Config) -> Result<()> {
        let eta = estimate_scan_duration(count, config);

        let warning = format!(
            "{} {}",
            "ℹ".bright_blue().bold(),
            "Large Target Set".bright_white().bold()
        );

        eprintln!("\n{}", warning);
        eprintln!("{}", "─".repeat(60).bright_cyan());
        eprintln!(
            "Scanning {} hosts (estimated time: {})\n\
             This will generate significant network traffic.",
            count.to_string().bright_yellow(),
            eta.bright_yellow()
        );
        eprintln!("{}", "─".repeat(60).bright_cyan());

        self.prompt_user("Continue with large scan?", true)
    }

    /// Confirm aggressive timing
    fn confirm_aggressive_timing(&self) -> Result<()> {
        let warning = format!(
            "{} {}",
            "⚠".yellow().bold(),
            "Aggressive Timing (T5) Selected".bright_white().bold()
        );

        eprintln!("\n{}", warning);
        eprintln!("{}", "─".repeat(60).bright_yellow());
        eprintln!(
            "T5 (Insane) timing is VERY aggressive and may:\n\
             • Trigger IDS/IPS systems\n\
             • Cause network congestion\n\
             • Miss ports on slower hosts\n\
             • Be detected easily"
        );
        eprintln!("{}", "─".repeat(60).bright_yellow());

        self.prompt_user("Continue with T5 timing?", false)
    }

    /// Confirm evasion techniques
    fn confirm_evasion_techniques(&self, evasion: &prtip_core::EvasionConfig) -> Result<()> {
        let warning = format!(
            "{} {}",
            "⚠".yellow().bold(),
            "Evasion Techniques Enabled".bright_white().bold()
        );

        eprintln!("\n{}", warning);
        eprintln!("{}", "─".repeat(60).bright_yellow());

        let mut techniques = Vec::new();
        if evasion.fragment_packets {
            techniques.push("• Packet fragmentation");
        }
        if evasion.decoys.is_some() {
            techniques.push("• Decoy scanning");
        }
        if evasion.bad_checksums {
            techniques.push("• Bad checksums");
        }

        eprintln!("Active evasion techniques:");
        for technique in &techniques {
            eprintln!("{}", technique);
        }
        eprintln!();
        eprintln!(
            "These techniques may be illegal in your jurisdiction and\n\
             could be interpreted as attempts to evade detection."
        );
        eprintln!("{}", "─".repeat(60).bright_yellow());

        self.prompt_user("Continue with evasion techniques?", false)
    }

    /// Confirm running as root
    #[cfg(unix)]
    fn confirm_root_privileges(&self) -> Result<()> {
        let warning = format!(
            "{} {}",
            "⚠".yellow().bold(),
            "Running as Root".bright_white().bold()
        );

        eprintln!("\n{}", warning);
        eprintln!("{}", "─".repeat(60).bright_yellow());
        eprintln!(
            "Running as root/superuser is recommended for raw socket access.\n\
             ProRT-IP will drop privileges after creating raw sockets.\n\n\
             Privileges will be dropped to 'nobody:nogroup' for safety."
        );
        eprintln!("{}", "─".repeat(60).bright_yellow());

        self.prompt_user("Drop privileges after socket creation?", true)
    }

    /// Check if running as root
    #[cfg(unix)]
    fn is_running_as_root(&self) -> bool {
        unsafe { libc::geteuid() == 0 }
    }

    /// Prompt user for yes/no confirmation
    ///
    /// # Arguments
    /// * `prompt` - The question to ask
    /// * `default_yes` - If true, default is 'Y' ([Y/n]); if false, default is 'N' ([y/N])
    fn prompt_user(&self, prompt: &str, default_yes: bool) -> Result<()> {
        let prompt_suffix = if default_yes { "[Y/n]" } else { "[y/N]" };

        eprint!("\n{} {} ", prompt.bright_white(), prompt_suffix.dimmed());

        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;

        let response = input.trim().to_lowercase();

        let confirmed = if response.is_empty() {
            default_yes // Enter key uses default
        } else {
            matches!(response.as_str(), "y" | "yes")
        };

        if !confirmed {
            eprintln!("\n{}", "❌ Scan cancelled by user.".bright_red());
            bail!("User declined to continue");
        }

        eprintln!(); // Add blank line after confirmation
        Ok(())
    }
}

/// Check if an IPv4 address is private (RFC1918)
fn is_private_ipv4(ip: &std::net::Ipv4Addr) -> bool {
    let octets = ip.octets();

    // 10.0.0.0/8
    octets[0] == 10
        // 172.16.0.0/12
        || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
        // 192.168.0.0/16
        || (octets[0] == 192 && octets[1] == 168)
        // Loopback 127.0.0.0/8
        || octets[0] == 127
}

/// Estimate the total number of targets (hosts) from target specifications
fn estimate_target_count(targets: &[ScanTarget]) -> usize {
    targets
        .iter()
        .map(|target| {
            let prefix = target.network.prefix();
            match target.network.ip() {
                IpAddr::V4(_) => {
                    // IPv4: 2^(32 - prefix) addresses
                    if prefix >= 32 {
                        1
                    } else {
                        1 << (32 - prefix)
                    }
                }
                IpAddr::V6(_) => {
                    // IPv6: For /64 and larger, estimate based on typical scan patterns
                    // Full IPv6 ranges are impractically large, so we estimate conservatively
                    if prefix >= 64 {
                        // Typical /64 scan targets ~1000 hosts
                        1000
                    } else if prefix >= 48 {
                        // /48 to /63: scale linearly
                        1000 * (1 << (64 - prefix))
                    } else {
                        // Larger than /48: cap at 1M hosts for estimation
                        1_000_000
                    }
                }
            }
        })
        .sum()
}

/// Estimate scan duration based on target count and configuration
fn estimate_scan_duration(target_count: usize, config: &Config) -> String {
    // Base scan rate: ports per second per target
    let base_pps_per_target = match config.scan.timing_template {
        TimingTemplate::Paranoid => 10,     // T0: very slow
        TimingTemplate::Sneaky => 50,       // T1: slow
        TimingTemplate::Polite => 200,      // T2: polite
        TimingTemplate::Normal => 500,      // T3: normal
        TimingTemplate::Aggressive => 1000, // T4: aggressive
        TimingTemplate::Insane => 2000,     // T5: insane
    };

    // Assume scanning 1000 ports (common default)
    let ports_to_scan = 1000;
    let total_ports = target_count * ports_to_scan;

    // Calculate estimated seconds
    let estimated_seconds = total_ports as f64 / base_pps_per_target as f64;

    // Format duration nicely
    if estimated_seconds < 60.0 {
        format!("{:.0}s", estimated_seconds)
    } else if estimated_seconds < 3600.0 {
        format!("{:.1}m", estimated_seconds / 60.0)
    } else if estimated_seconds < 86400.0 {
        format!("{:.1}h", estimated_seconds / 3600.0)
    } else {
        format!("{:.1}d", estimated_seconds / 86400.0)
    }
}

/// Check if terminal is interactive (supports stdin/stdout)
fn is_terminal_interactive() -> bool {
    use std::io::IsTerminal;
    stdin().is_terminal() && stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_is_private_ipv4() {
        // Private ranges
        assert!(is_private_ipv4(&Ipv4Addr::new(10, 0, 0, 1)));
        assert!(is_private_ipv4(&Ipv4Addr::new(172, 16, 0, 1)));
        assert!(is_private_ipv4(&Ipv4Addr::new(172, 31, 255, 255)));
        assert!(is_private_ipv4(&Ipv4Addr::new(192, 168, 1, 1)));
        assert!(is_private_ipv4(&Ipv4Addr::new(127, 0, 0, 1)));

        // Public IPs
        assert!(!is_private_ipv4(&Ipv4Addr::new(8, 8, 8, 8)));
        assert!(!is_private_ipv4(&Ipv4Addr::new(1, 1, 1, 1)));
        assert!(!is_private_ipv4(&Ipv4Addr::new(172, 15, 0, 1)));
        assert!(!is_private_ipv4(&Ipv4Addr::new(172, 32, 0, 1)));
    }

    #[test]
    fn test_estimate_target_count_ipv4() {
        let target = ScanTarget::parse("192.168.1.1/32").unwrap();
        assert_eq!(estimate_target_count(&[target]), 1);

        let target = ScanTarget::parse("192.168.1.0/24").unwrap();
        assert_eq!(estimate_target_count(&[target]), 256);

        let target = ScanTarget::parse("10.0.0.0/16").unwrap();
        assert_eq!(estimate_target_count(&[target]), 65536);
    }

    #[test]
    fn test_estimate_target_count_multiple() {
        let targets = vec![
            ScanTarget::parse("192.168.1.1").unwrap(),
            ScanTarget::parse("192.168.2.1").unwrap(),
            ScanTarget::parse("10.0.0.0/24").unwrap(),
        ];
        assert_eq!(estimate_target_count(&targets), 1 + 1 + 256);
    }

    #[test]
    fn test_estimate_scan_duration() {
        let mut config = Config::default();

        // T3 (Normal) timing
        config.scan.timing_template = TimingTemplate::Normal;
        let duration = estimate_scan_duration(1, &config);
        assert!(duration.contains('s')); // Should be in seconds

        // T0 (Paranoid) timing - much slower
        config.scan.timing_template = TimingTemplate::Paranoid;
        let duration = estimate_scan_duration(100, &config);
        assert!(duration.contains('h') || duration.contains('m')); // Should be hours or minutes

        // T5 (Insane) timing - very fast
        config.scan.timing_template = TimingTemplate::Insane;
        let duration = estimate_scan_duration(10, &config);
        assert!(duration.contains('s')); // Should be seconds
    }

    #[test]
    fn test_confirmation_manager_auto_yes() {
        let config = ConfirmConfig {
            auto_yes: true,
            is_interactive: true,
        };
        let manager = ConfirmationManager::new(config);

        // Should always succeed with auto_yes
        let scan_config = Config::default();
        let targets = vec![ScanTarget::parse("8.8.8.8").unwrap()];

        assert!(manager.confirm_scan(&scan_config, &targets).is_ok());
    }

    #[test]
    fn test_confirmation_manager_non_interactive() {
        let config = ConfirmConfig {
            auto_yes: false,
            is_interactive: false,
        };
        let manager = ConfirmationManager::new(config);

        // Should always succeed when non-interactive (CI/CD)
        let scan_config = Config::default();
        let targets = vec![ScanTarget::parse("8.8.8.8").unwrap()];

        assert!(manager.confirm_scan(&scan_config, &targets).is_ok());
    }

    #[test]
    fn test_is_internet_scale_private_ips() {
        let manager = ConfirmationManager::new(ConfirmConfig::default());

        // Private IPs should NOT be internet-scale
        let targets = vec![
            ScanTarget::parse("192.168.1.1").unwrap(),
            ScanTarget::parse("10.0.0.0/24").unwrap(),
        ];
        assert!(!manager.is_internet_scale_scan(&targets));
    }

    #[test]
    fn test_is_internet_scale_public_ips() {
        let manager = ConfirmationManager::new(ConfirmConfig::default());

        // Public IPs should be internet-scale
        let targets = vec![ScanTarget::parse("8.8.8.8").unwrap()];
        assert!(manager.is_internet_scale_scan(&targets));
    }

    #[test]
    fn test_is_internet_scale_large_ranges() {
        let manager = ConfirmationManager::new(ConfirmConfig::default());

        // Large CIDR ranges should be internet-scale even if private
        let targets = vec![
            ScanTarget::parse("10.0.0.0/8").unwrap(), // /8 is huge
        ];
        assert!(manager.is_internet_scale_scan(&targets));
    }

    #[test]
    fn test_is_terminal_interactive() {
        // Can't reliably test this in automated tests, but ensure function exists
        let _interactive = is_terminal_interactive();
    }
}
