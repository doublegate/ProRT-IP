//! ProRT-IP WarScan CLI
//!
//! Command-line interface for the ProRT-IP network scanner.

mod args;
mod banner;
mod output;

use anyhow::{Context, Result};
use args::Args;
use banner::Banner;
use clap::Parser;
use prtip_core::resource_limits::{adjust_and_get_limit, get_recommended_batch_size};
use prtip_core::{PortRange, ScanTarget};
use prtip_network::check_privileges;
#[cfg(target_os = "linux")]
use prtip_network::drop_privileges;
use prtip_network::interface::enumerate_interfaces;
use prtip_scanner::{ScanScheduler, ScanStorage, StorageBackend};
use std::sync::Arc;
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

    // Print banner unless quiet mode or piped output
    if !args.quiet && atty::is(atty::Stream::Stdout) {
        let banner = Banner::new(env!("CARGO_PKG_VERSION"));
        if args.compact_banner {
            banner.print_compact();
        } else {
            banner.print();
        }
    }

    // Handle --interface-list flag
    if args.interface_list {
        return handle_interface_list();
    }

    // Initialize logging based on verbosity
    init_logging(args.verbose);

    info!("ProRT-IP WarScan v{}", env!("CARGO_PKG_VERSION"));
    info!("High-performance network scanner");

    // Validate arguments
    args.validate().context("Invalid arguments")?;

    // Adjust ulimit if requested (before any other resource operations)
    if let Some(requested_ulimit) = args.ulimit {
        match adjust_and_get_limit(Some(requested_ulimit)) {
            Ok(new_limit) => {
                info!("Successfully adjusted ulimit to {}", new_limit);
            }
            Err(e) => {
                warn!("Failed to adjust ulimit to {}: {}", requested_ulimit, e);
                warn!(
                    "Continuing with current ulimit. You may need to run 'ulimit -n {}' manually.",
                    requested_ulimit
                );
            }
        }
    }

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
    let mut config = args.to_config();
    info!(
        "Scan configuration: type={:?}, timing={:?}, timeout={}ms",
        config.scan.scan_type, config.scan.timing_template, config.scan.timeout_ms
    );

    // Get recommended batch size based on ulimit
    let desired_batch = config.performance.batch_size.unwrap_or(1000);
    match get_recommended_batch_size(desired_batch as u64, config.performance.requested_ulimit) {
        Ok(recommended) => {
            if desired_batch as u64 > recommended {
                warn!(
                    "Batch size {} exceeds safe limit {} based on file descriptor limits",
                    desired_batch, recommended
                );
                warn!(
                    "Recommended: Use '-b {}' or increase ulimit with '--ulimit {}'",
                    recommended,
                    desired_batch * 2
                );
                // Auto-adjust to safe value
                config.performance.batch_size = Some(recommended as usize);
                info!("Auto-adjusted batch size to {}", recommended);
            } else if config.performance.batch_size.is_none() {
                // Set to recommended if not specified
                config.performance.batch_size = Some(recommended as usize);
                info!("Using recommended batch size: {}", recommended);
            }
        }
        Err(e) => {
            warn!("Failed to calculate recommended batch size: {}", e);
            if config.performance.batch_size.is_none() {
                config.performance.batch_size = Some(1000);
                info!("Using default batch size: 1000");
            }
        }
    }

    // Create storage backend
    let storage_backend = if args.with_db {
        let storage = Arc::new(ScanStorage::new(&args.database).await.context(format!(
            "Failed to create scan storage at '{}'",
            args.database
        ))?);
        info!("Connected to database: {} (async mode)", args.database);

        Arc::new(
            StorageBackend::async_database(
                storage,
                config.scan.scan_type,
                &format!("{:?}", targets),
            )
            .await
            .context("Failed to create async storage backend")?,
        )
    } else {
        info!("Database storage disabled (default in-memory mode)");

        // Calculate estimated capacity for memory backend
        let estimated_results = targets.len() * ports.count();
        let capacity = estimated_results.max(10000); // At least 10K capacity

        Arc::new(StorageBackend::memory(capacity))
    };

    // Create scheduler
    let scheduler = ScanScheduler::new(config.clone(), storage_backend)
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
    let scan_start = std::time::Instant::now();
    println!(
        "\n{}",
        format_scan_banner(&args, &config, ports.count(), &targets)
    );

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

    let scan_duration = scan_start.elapsed();
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

    // Print summary with scan statistics
    print_summary(&results, scan_duration);

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
    use colored::*;
    let mut targets = Vec::new();

    for spec in target_specs {
        let target =
            ScanTarget::parse(spec).context(format!("Invalid target specification: '{}'", spec))?;

        // Print DNS resolution feedback if hostname was resolved
        if let Some(hostname) = &target.hostname {
            let ip = target.network.ip();
            println!(
                "{} {} {} {}",
                "[DNS]".bright_blue(),
                "Resolved".green(),
                hostname.bright_yellow(),
                format!("-> {}", ip).bright_cyan()
            );
        }

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
fn format_scan_banner(
    args: &Args,
    config: &prtip_core::Config,
    port_count: usize,
    targets: &[prtip_core::ScanTarget],
) -> String {
    use colored::*;
    use prtip_scanner::adaptive_parallelism::calculate_parallelism;

    let mut banner = String::new();

    banner.push_str(&"=".repeat(60).bright_cyan().to_string());
    banner.push('\n');
    banner.push_str(&format!("{}\n", "ProRT-IP WarScan".bright_white().bold()));
    banner.push_str(&"=".repeat(60).bright_cyan().to_string());
    banner.push('\n');

    // Format targets with resolved IPs
    let target_display = if args.targets.len() == 1 && targets.len() == 1 {
        // Single target - show hostname (IP) if hostname was resolved
        if let Some(hostname) = &targets[0].hostname {
            let ip = targets[0].network.ip();
            format!("{} ({})", hostname, ip)
        } else {
            args.targets[0].clone()
        }
    } else {
        // Multiple targets - just show original input
        args.targets.join(", ")
    };

    banner.push_str(&format!("Targets:  {}\n", target_display));
    banner.push_str(&format!("Ports:    {}\n", args.ports));
    banner.push_str(&format!("Type:     {}\n", config.scan.scan_type));
    banner.push_str(&format!("Timing:   {}\n", config.scan.timing_template));
    banner.push_str(&format!("Timeout:  {}ms\n", config.scan.timeout_ms));

    if let Some(rate) = config.performance.max_rate {
        banner.push_str(&format!("Max Rate: {} pps\n", rate));
    }

    // Calculate actual parallelism (fix for "Parallel: 0" bug)
    let user_override = if config.performance.parallelism > 0 {
        Some(config.performance.parallelism)
    } else {
        None
    };
    let actual_parallelism = calculate_parallelism(
        port_count,
        user_override,
        config.performance.requested_ulimit,
    );

    banner.push_str(&format!(
        "Parallel: {}{}",
        actual_parallelism,
        if config.performance.parallelism == 0 {
            " (adaptive)".dimmed().to_string()
        } else {
            "".to_string()
        }
    ));
    banner.push('\n');
    banner.push_str(&"=".repeat(60).bright_cyan().to_string());
    banner.push('\n');

    banner
}

/// Print a summary of scan results with comprehensive statistics
fn print_summary(results: &[prtip_core::ScanResult], duration: std::time::Duration) {
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

    // Calculate services detected (ports with service name)
    let services_detected = results.iter().filter(|r| r.service().is_some()).count();

    // Calculate scan rate (ports/second)
    let duration_secs = duration.as_secs_f64();
    let scan_rate = if duration_secs > 0.0 {
        results.len() as f64 / duration_secs
    } else {
        0.0
    };

    // Format duration
    let duration_ms = duration.as_millis();
    let duration_str = if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else if duration_ms < 60_000 {
        format!("{:.2}s", duration_ms as f64 / 1000.0)
    } else {
        let mins = duration_ms / 60_000;
        let secs = (duration_ms % 60_000) / 1000;
        format!("{}m {}s", mins, secs)
    };

    println!("\n{}", "=".repeat(60).bright_cyan());
    println!("{}", "Scan Summary".bright_white().bold());
    println!("{}", "=".repeat(60).bright_cyan());

    // Scan statistics
    println!("{}", "Performance:".bright_white().bold());
    println!("  Duration:       {}", duration_str.bright_white());
    println!("  Scan Rate:      {:.0} ports/sec", scan_rate);

    println!();
    println!("{}", "Targets:".bright_white().bold());
    println!(
        "  Hosts Scanned:  {}",
        hosts.len().to_string().bright_white()
    );
    println!(
        "  Total Ports:    {}",
        results.len().to_string().bright_white()
    );

    println!();
    println!("{}", "Results:".bright_white().bold());
    println!(
        "  Open Ports:     {}",
        open_ports.to_string().green().bold()
    );
    println!("  Closed Ports:   {}", closed_ports.to_string().red());
    println!("  Filtered Ports: {}", filtered_ports.to_string().yellow());

    if services_detected > 0 {
        println!();
        println!("{}", "Detection:".bright_white().bold());
        println!("  Services:       {}", services_detected.to_string().cyan());
    }

    println!("{}", "=".repeat(60).bright_cyan());
}

/// Handle --interface-list flag
fn handle_interface_list() -> Result<()> {
    use colored::*;

    println!("\n{}", "Available Network Interfaces".bright_white().bold());
    println!("{}", "=".repeat(60).bright_cyan());

    let interfaces = enumerate_interfaces().context("Failed to enumerate network interfaces")?;

    if interfaces.is_empty() {
        println!("{}", "No network interfaces found".yellow());
        return Ok(());
    }

    let interface_count = interfaces.len();

    for iface in &interfaces {
        let status = if iface.is_up {
            "UP".green()
        } else {
            "DOWN".red()
        };

        let iface_type = if iface.is_loopback {
            " (loopback)".dimmed()
        } else {
            "".normal()
        };

        println!(
            "\n{}: {}{}",
            iface.name.bright_white().bold(),
            status,
            iface_type
        );

        if let Some(mac) = &iface.mac_address {
            let mac_str = mac
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(":");
            println!("  MAC:  {}", mac_str.dimmed());
        }

        if let Some(mtu) = iface.mtu {
            println!("  MTU:  {}", mtu.to_string().dimmed());
        }

        if !iface.ipv4_addresses.is_empty() {
            println!("  IPv4:");
            for addr in &iface.ipv4_addresses {
                println!("    - {}", addr.to_string().cyan());
            }
        }

        if !iface.ipv6_addresses.is_empty() {
            println!("  IPv6:");
            for addr in &iface.ipv6_addresses {
                println!("    - {}", addr.to_string().cyan());
            }
        }
    }

    println!("\n{}", "=".repeat(60).bright_cyan());
    println!("Total: {} interface(s)", interface_count);

    Ok(())
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
        // Should fail DNS resolution for invalid hostname
        assert!(targets.is_err());
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
        let targets = vec![ScanTarget::parse("192.168.1.1").unwrap()];
        let banner = format_scan_banner(&args, &config, 1, &targets);

        assert!(banner.contains("ProRT-IP"));
        assert!(banner.contains("192.168.1.1"));
        assert!(banner.contains("80"));
    }

    #[test]
    fn test_format_scan_banner_with_hostname() {
        let args = Args::parse_from(["prtip", "-p", "80", "127.0.0.1"]);
        let config = args.to_config();
        // Create a target with a hostname (simulate DNS resolution result)
        let mut target = ScanTarget::parse("127.0.0.1").unwrap();
        // Manually add hostname to simulate DNS resolution
        target.hostname = Some("example.com".to_string());

        let targets = vec![target];
        let banner = format_scan_banner(&args, &config, 1, &targets);

        assert!(banner.contains("ProRT-IP"));
        assert!(banner.contains("example.com"));
        assert!(banner.contains("127.0.0.1"));
        assert!(banner.contains("80"));
    }

    #[test]
    fn test_print_summary_empty() {
        let results = vec![];
        let duration = std::time::Duration::from_secs(1);
        // Should not panic
        print_summary(&results, duration);
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

        let duration = std::time::Duration::from_millis(100);
        // Should not panic
        print_summary(&results, duration);
    }

    #[test]
    fn test_expand_targets_with_ports() {
        let targets = vec![ScanTarget::parse("192.168.1.1").unwrap()];
        let ports = PortRange::parse("80,443").unwrap();
        let expanded = expand_targets_with_ports(targets, &ports).unwrap();
        assert_eq!(expanded.len(), 1);
    }
}
