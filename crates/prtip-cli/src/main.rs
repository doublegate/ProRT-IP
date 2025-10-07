//! ProRT-IP WarScan CLI
//!
//! Command-line interface for the ProRT-IP network scanner.

mod args;
mod output;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use prtip_core::{PortRange, ScanTarget};
use prtip_network::{check_privileges, drop_privileges};
use prtip_scanner::{ScanScheduler, ScanStorage};
use tracing::{info, warn};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);

        // Print error chain if available
        if let Some(cause) = e.source() {
            eprintln!("\nCaused by:");
            let mut current_cause = Some(cause);
            let mut level = 1;
            while let Some(cause) = current_cause {
                eprintln!("  {}: {}", level, cause);
                current_cause = cause.source();
                level += 1;
            }
        }

        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    // Parse arguments
    let args = Args::parse();

    // Initialize logging based on verbosity
    init_logging(args.verbose);

    info!("ProRT-IP WarScan v{}", env!("CARGO_PKG_VERSION"));
    info!("High-performance network scanner");

    // Validate arguments
    args.validate().context("Invalid arguments")?;

    // Check privileges (needed for raw sockets in future phases)
    match check_privileges() {
        Ok(()) => {
            info!("Privilege check passed");
        }
        Err(e) => {
            warn!(
                "Insufficient privileges for raw sockets: {}. Using TCP connect scan fallback.",
                e
            );
            // For Phase 1, we only have TCP connect scan, so this is OK
            // In future phases, raw socket scans will require privileges
        }
    }

    // Parse targets
    let targets = parse_targets(&args.targets)?;
    info!("Parsed {} scan target(s)", targets.len());

    // Parse ports
    let ports = PortRange::parse(&args.ports).context(format!(
        "Failed to parse port specification '{}'",
        args.ports
    ))?;
    info!("Scanning {} port(s) per host", ports.count());

    // Create config from arguments
    let config = args.to_config();
    info!(
        "Scan configuration: type={:?}, timing={:?}, timeout={}ms",
        config.scan.scan_type, config.scan.timing_template, config.scan.timeout_ms
    );

    // Create storage
    let storage = ScanStorage::new(&args.database).await.context(format!(
        "Failed to create scan storage at '{}'",
        args.database
    ))?;
    info!("Connected to database: {}", args.database);

    // Create scheduler
    let scheduler = ScanScheduler::new(config.clone(), storage)
        .await
        .context("Failed to create scan scheduler")?;

    // Drop privileges after creating privileged resources (if we had any)
    // For Phase 1, TCP connect scan doesn't need raw sockets
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = drop_privileges("nobody", "nogroup") {
            warn!(
                "Failed to drop privileges: {}. Continuing as current user.",
                e
            );
            // Not fatal for Phase 1 since we're using connect scan
        } else {
            info!("Successfully dropped privileges");
        }
    }

    // Execute scan
    info!("Starting scan...");
    println!("\n{}", format_scan_banner(&args, &config));

    let results = if args.host_discovery {
        info!("Performing host discovery before port scanning");
        scheduler.execute_scan_with_discovery(targets).await?
    } else {
        // For Phase 1, we need to expand targets with ports
        let expanded_targets = expand_targets_with_ports(targets, &ports)?;
        scheduler
            .execute_scan_ports(expanded_targets, &ports)
            .await?
    };

    info!("Scan complete: {} results", results.len());

    // Format and output results
    let is_terminal = atty::is(atty::Stream::Stdout);
    let formatter = output::create_formatter(config.output.format, is_terminal);
    let formatted = formatter
        .format_results(&results, &config)
        .context("Failed to format scan results")?;

    // Write output
    match &config.output.file {
        Some(path) => {
            std::fs::write(path, &formatted)
                .context(format!("Failed to write output to {:?}", path))?;
            println!("\nResults written to: {:?}", path);
            println!("Total results: {}", results.len());
        }
        None => {
            println!("{}", formatted);
        }
    }

    // Print summary
    print_summary(&results);

    Ok(())
}

/// Initialize tracing/logging based on verbosity level
fn init_logging(verbose: u8) {
    use tracing_subscriber::{fmt, EnvFilter};

    let level = match verbose {
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .compact()
        .init();
}

/// Parse target specifications into ScanTarget structures
fn parse_targets(target_specs: &[String]) -> Result<Vec<ScanTarget>> {
    let mut targets = Vec::new();

    for spec in target_specs {
        let target =
            ScanTarget::parse(spec).context(format!("Invalid target specification: '{}'", spec))?;
        targets.push(target);
    }

    if targets.is_empty() {
        anyhow::bail!("No valid targets specified");
    }

    Ok(targets)
}

/// Expand targets with port information
///
/// This is a temporary helper for Phase 1 which doesn't have the full
/// port specification in ScanTarget yet.
fn expand_targets_with_ports(
    targets: Vec<ScanTarget>,
    _ports: &PortRange,
) -> Result<Vec<ScanTarget>> {
    // For Phase 1, just return targets as-is
    // In future phases, we'll properly associate ports with targets
    Ok(targets)
}

/// Format a nice banner for the scan
fn format_scan_banner(args: &Args, config: &prtip_core::Config) -> String {
    use colored::*;

    let mut banner = String::new();

    banner.push_str(&"=".repeat(60).bright_cyan().to_string());
    banner.push('\n');
    banner.push_str(&format!("{}\n", "ProRT-IP WarScan".bright_white().bold()));
    banner.push_str(&"=".repeat(60).bright_cyan().to_string());
    banner.push('\n');

    banner.push_str(&format!("Targets:  {}\n", args.targets.join(", ")));
    banner.push_str(&format!("Ports:    {}\n", args.ports));
    banner.push_str(&format!("Type:     {}\n", config.scan.scan_type));
    banner.push_str(&format!("Timing:   {}\n", config.scan.timing_template));
    banner.push_str(&format!("Timeout:  {}ms\n", config.scan.timeout_ms));

    if let Some(rate) = config.performance.max_rate {
        banner.push_str(&format!("Max Rate: {} pps\n", rate));
    }

    banner.push_str(&format!("Parallel: {}\n", config.performance.parallelism));
    banner.push_str(&"=".repeat(60).bright_cyan().to_string());
    banner.push('\n');

    banner
}

/// Print a summary of scan results
fn print_summary(results: &[prtip_core::ScanResult]) {
    use colored::*;
    use std::collections::HashSet;

    if results.is_empty() {
        return;
    }

    let hosts: HashSet<_> = results.iter().map(|r| r.target_ip()).collect();
    let open_ports = results
        .iter()
        .filter(|r| r.state() == prtip_core::PortState::Open)
        .count();
    let closed_ports = results
        .iter()
        .filter(|r| r.state() == prtip_core::PortState::Closed)
        .count();
    let filtered_ports = results
        .iter()
        .filter(|r| r.state() == prtip_core::PortState::Filtered)
        .count();

    println!("\n{}", "=".repeat(60).bright_cyan());
    println!("{}", "Scan Summary".bright_white().bold());
    println!("{}", "=".repeat(60).bright_cyan());
    println!(
        "Hosts Scanned:    {}",
        hosts.len().to_string().bright_white().bold()
    );
    println!(
        "Total Ports:      {}",
        results.len().to_string().bright_white().bold()
    );
    println!(
        "Open Ports:       {}",
        open_ports.to_string().green().bold()
    );
    println!("Closed Ports:     {}", closed_ports.to_string().red());
    println!("Filtered Ports:   {}", filtered_ports.to_string().yellow());
    println!("{}", "=".repeat(60).bright_cyan());
}

// Add atty dependency for terminal detection
mod atty {
    pub enum Stream {
        Stdout,
    }

    pub fn is(_stream: Stream) -> bool {
        // Simple check if stdout is a tty
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = std::io::stdout().as_raw_fd();
            unsafe { libc::isatty(fd) != 0 }
        }

        #[cfg(windows)]
        {
            // On Windows, assume terminal for now
            true
        }

        #[cfg(not(any(unix, windows)))]
        {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_targets_single() {
        let specs = vec!["192.168.1.1".to_string()];
        let targets = parse_targets(&specs).unwrap();
        assert_eq!(targets.len(), 1);
    }

    #[test]
    fn test_parse_targets_multiple() {
        let specs = vec![
            "192.168.1.1".to_string(),
            "10.0.0.0/24".to_string(),
            "example.com".to_string(),
        ];
        let targets = parse_targets(&specs).unwrap();
        assert_eq!(targets.len(), 3);
    }

    #[test]
    fn test_parse_targets_invalid() {
        let specs = vec!["not-a-valid-target!!!".to_string()];
        let targets = parse_targets(&specs);
        // Should still parse as hostname
        assert!(targets.is_ok());
    }

    #[test]
    fn test_parse_targets_empty() {
        let specs: Vec<String> = vec![];
        let result = parse_targets(&specs);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_scan_banner() {
        let args = Args::parse_from(["prtip", "-p", "80", "192.168.1.1"]);
        let config = args.to_config();
        let banner = format_scan_banner(&args, &config);

        assert!(banner.contains("ProRT-IP"));
        assert!(banner.contains("192.168.1.1"));
        assert!(banner.contains("80"));
    }

    #[test]
    fn test_print_summary_empty() {
        let results = vec![];
        // Should not panic
        print_summary(&results);
    }

    #[test]
    fn test_print_summary_with_results() {
        use prtip_core::{PortState, ScanResult};
        use std::net::IpAddr;

        let results = vec![
            ScanResult::new(
                "192.168.1.1".parse::<IpAddr>().unwrap(),
                80,
                PortState::Open,
            ),
            ScanResult::new(
                "192.168.1.1".parse::<IpAddr>().unwrap(),
                81,
                PortState::Closed,
            ),
        ];

        // Should not panic
        print_summary(&results);
    }

    #[test]
    fn test_expand_targets_with_ports() {
        let targets = vec![ScanTarget::parse("192.168.1.1").unwrap()];
        let ports = PortRange::parse("80,443").unwrap();
        let expanded = expand_targets_with_ports(targets, &ports).unwrap();
        assert_eq!(expanded.len(), 1);
    }
}
