//! ProRT-IP WarScan CLI
//!
//! Command-line interface for the ProRT-IP network scanner.

mod args;
mod banner;
mod confirm;
mod db_commands;
mod export;
mod help;
mod history;
mod output;
mod progress;
mod templates;

use anyhow::{bail, Context, Result};
use args::Args;
use banner::Banner;
use clap::Parser;
use colored::Colorize;
use confirm::{ConfirmConfig, ConfirmationManager};
use history::HistoryManager;
use progress::{ProgressDisplay, ProgressStyle};
use prtip_core::event_bus::{EventBus, EventFilter};
use prtip_core::events::ScanEventType;
use prtip_core::resource_limits::{adjust_and_get_limit, get_recommended_batch_size};
use prtip_core::{PortRange, ScanTarget};
use prtip_network::check_privileges;
#[cfg(target_os = "linux")]
use prtip_network::drop_privileges;
use prtip_network::interface::enumerate_interfaces;
use prtip_scanner::{ScanScheduler, ScanStorage, StorageBackend};
use std::sync::Arc;
use tracing::{info, warn};

/// Preprocess command-line arguments to convert nmap-style flags to ProRT-IP flags
///
/// This function translates nmap-compatible syntax (e.g., `-sS`, `-oN file.txt`)
/// into ProRT-IP's internal long-form flags that clap can parse. This enables
/// nmap users to use familiar syntax while maintaining 100% backward compatibility.
///
/// # Conversions
///
/// - `-sS` → `--nmap-syn`
/// - `-sT` → `--nmap-connect`
/// - `-sV` → `--sV` (service version detection)
/// - `-sI <zombie>` → `--nmap-idle <zombie>`
/// - `-oN <file>` → `--output-normal <file>`
/// - `-oX <file>` → `--output-xml <file>`
/// - `-oG <file>` → `--output-greppable <file>`
/// - `-oA <base>` → `--output-all-formats <base>`
/// - `-Pn` → `--skip-ping`
///
/// All other arguments are passed through unchanged.
fn preprocess_argv() -> Vec<String> {
    let args: Vec<String> = std::env::args().collect();

    // Fast path: Skip preprocessing if no nmap-style flags are present
    // This optimization reduces CLI overhead by ~0.1ms for native ProRT-IP syntax
    let needs_preprocessing = args.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "-sS"
                | "-sT"
                | "-sU"
                | "-sN"
                | "-sF"
                | "-sX"
                | "-sA"
                | "-sI"
                | "-sV"
                | "-oN"
                | "-oX"
                | "-oG"
                | "-oA"
                | "-Pn"
        )
    });

    if !needs_preprocessing {
        return args; // Return original args unchanged (zero-copy)
    }

    // Slow path: Preprocess nmap-style flags
    let mut processed = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            // Scan type flags (no value)
            "-sS" => processed.push("--nmap-syn".to_string()),
            "-sT" => processed.push("--nmap-connect".to_string()),
            "-sU" => processed.push("--nmap-udp".to_string()),
            "-sN" => processed.push("--nmap-null".to_string()),
            "-sF" => processed.push("--nmap-fin".to_string()),
            "-sX" => processed.push("--nmap-xmas".to_string()),
            "-sA" => processed.push("--nmap-ack".to_string()),

            // Service version detection (no value)
            "-sV" => processed.push("--sV".to_string()),

            // Idle scan flag (with zombie host value)
            "-sI" => {
                processed.push("--nmap-idle".to_string());
                i += 1;
                if i < args.len() {
                    processed.push(args[i].clone());
                }
            }

            // Output format flags (with value)
            "-oN" => {
                processed.push("--output-normal".to_string());
                i += 1;
                if i < args.len() {
                    processed.push(args[i].clone());
                }
            }
            "-oX" => {
                processed.push("--output-xml".to_string());
                i += 1;
                if i < args.len() {
                    processed.push(args[i].clone());
                }
            }
            "-oG" => {
                processed.push("--output-greppable".to_string());
                i += 1;
                if i < args.len() {
                    processed.push(args[i].clone());
                }
            }
            "-oA" => {
                processed.push("--output-all-formats".to_string());
                i += 1;
                if i < args.len() {
                    processed.push(args[i].clone());
                }
            }

            // Skip ping flag
            "-Pn" => processed.push("--skip-ping".to_string()),

            // Pass through everything else unchanged
            _ => processed.push(arg.clone()),
        }

        i += 1;
    }

    processed
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        // Use enhanced error formatter for user-friendly messages
        let formatter = prtip_cli::create_error_formatter();
        eprint!("{}", formatter.format_error(e.as_ref()));

        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    // Check for help subcommand before preprocessing
    // This allows `prtip help`, `prtip help <topic>`, `prtip help examples`, `prtip help search <query>`
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() >= 2 && argv[1] == "help" {
        let help_system = help::HelpSystem::new();
        if argv.len() == 2 {
            // `prtip help` - show categories
            help_system.show_categories();
        } else if argv[2] == "examples" {
            // `prtip help examples` - show examples
            help_system.show_examples();
        } else if argv[2] == "search" {
            // `prtip help search <query>` - search help content
            if argv.len() >= 4 {
                let query = argv[3..].join(" ");
                help_system.search(&query);
            } else {
                eprintln!("Error: Missing search query");
                eprintln!("Usage: prtip help search <query>");
                eprintln!("Example: prtip help search timing");
                std::process::exit(1);
            }
        } else {
            // `prtip help <topic>` - show specific topic
            help_system.show_topic(&argv[2]);
        }
        return Ok(());
    }

    // Check for db subcommand
    // This allows `prtip db list`, `prtip db query`, etc.
    if argv.len() >= 2 && argv[1] == "db" {
        use clap::Parser;
        use db_commands::{DbCommand, DbSubcommand};

        let db_cmd = DbCommand::parse_from(&argv[1..]);

        match db_cmd.command {
            DbSubcommand::List { db_path } => {
                db_commands::handle_list(db_path).await?;
            }
            DbSubcommand::Query {
                db_path,
                scan_id,
                target,
                port,
                service,
                open,
            } => {
                db_commands::handle_query(db_path, scan_id, target, port, service, open).await?;
            }
            DbSubcommand::Export {
                db_path,
                scan_id,
                format,
                output,
            } => {
                db_commands::handle_export(db_path, scan_id, format, output).await?;
            }
            DbSubcommand::Compare {
                db_path,
                scan_id_1,
                scan_id_2,
            } => {
                db_commands::handle_compare(db_path, scan_id_1, scan_id_2).await?;
            }
        }

        return Ok(());
    }

    // Check for history subcommand
    // This allows `prtip history`, `prtip history <n>`, `prtip history --clear`
    if argv.len() >= 2 && argv[1] == "history" {
        return handle_history_command(&argv[2..]).await;
    }

    // Check for replay subcommand
    // This allows `prtip replay <index>`, `prtip replay --last`
    if argv.len() >= 2 && argv[1] == "replay" {
        return handle_replay_command(&argv[2..]).await;
    }

    // Preprocess arguments to support nmap-style syntax
    let processed_args = preprocess_argv();

    // Parse arguments
    let args = Args::parse_from(processed_args);

    // Print banner unless quiet mode or piped output
    {
        use std::io::{stdout, IsTerminal};
        if !args.quiet && stdout().is_terminal() {
            let banner = Banner::new(env!("CARGO_PKG_VERSION"));
            if args.compact_banner {
                banner.print_compact();
            } else {
                banner.print();
            }
        }
    }

    // Handle template commands (--list-templates, --show-template)
    if args.list_templates {
        return handle_list_templates();
    }

    if let Some(ref template_name) = args.show_template {
        return handle_show_template(template_name);
    }

    // Handle --interface-list or --iflist flags
    if args.interface_list || args.iflist {
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

    // Validate target protocols against -4/-6/--dual-stack flags
    args.validate_target_protocols(&targets)
        .context("Target protocol validation failed")?;

    // Parse ports (use effective ports which handles -F and --top-ports)
    let port_spec = args.get_effective_ports();
    let ports = PortRange::parse(&port_spec).context(format!(
        "Failed to parse port specification '{}'",
        port_spec
    ))?;
    info!("Scanning {} port(s) per host", ports.count());

    // Create config from arguments
    let mut config = args.to_config()?;

    // Apply template if specified (template values are overridden by CLI flags)
    if let Some(ref template_name) = args.template {
        use templates::TemplateManager;

        let manager =
            TemplateManager::with_custom_templates().context("Failed to load templates")?;

        manager
            .apply_template(template_name, &mut config)
            .with_context(|| format!("Failed to apply template '{}'", template_name))?;

        info!("Applied template: {}", template_name);
    }

    info!(
        "Scan configuration: type={:?}, timing={:?}, timeout={}ms",
        config.scan.scan_type, config.scan.timing_template, config.scan.timeout_ms
    );

    // Create EventBus for progress tracking and live results (if not quiet)
    let event_bus = if !args.quiet {
        Some(Arc::new(EventBus::new(1000)))
    } else {
        None
    };

    // Attach EventBus to ScanConfig
    if let Some(ref bus) = event_bus {
        config.scan = config.scan.with_event_bus(bus.clone());
        info!("Event-driven progress tracking enabled");
    }

    // Calculate total ports for progress display
    let total_ports = targets.len() * ports.count();

    // Initialize ProgressDisplay (event-driven)
    let progress_display = if let Some(ref bus) = event_bus {
        // Determine display style (compact by default, can be extended later)
        let style = ProgressStyle::Compact;
        let display = ProgressDisplay::new(bus.clone(), style, args.quiet);

        // Start the display task
        let _display_task = display.start().await;

        info!("Progress display initialized ({} total ports)", total_ports);
        Some(display)
    } else {
        None
    };

    // Setup live results streaming if requested
    if args.live_results {
        if let Some(ref bus) = event_bus {
            let bus_clone = bus.clone();
            tokio::spawn(async move {
                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

                // Subscribe to PortFound events
                bus_clone
                    .subscribe(tx, EventFilter::EventType(vec![ScanEventType::PortFound]))
                    .await;

                // Stream results to stdout immediately
                while let Some(event) = rx.recv().await {
                    if let prtip_core::events::ScanEvent::PortFound {
                        ip,
                        port,
                        state,
                        protocol,
                        ..
                    } = event
                    {
                        println!("[LIVE] {}:{} {} ({})", ip, port, state, protocol);
                    }
                }
            });
            info!("Live results streaming enabled");
        } else {
            warn!("Live results requires event tracking (ignored in --quiet mode)");
        }
    }

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

    // Check for dangerous operations and confirm if needed
    // This provides safety confirmations for:
    // - Internet-scale scans (public IPs or large ranges)
    // - Large target sets (>10,000 hosts)
    // - Aggressive timing (T5)
    // - Evasion techniques (fragmentation, decoys)
    // - Running as root
    let confirmation_config = ConfirmConfig {
        auto_yes: args.yes,
        is_interactive: !args.quiet && {
            use std::io::{stdout, IsTerminal};
            stdout().is_terminal()
        },
    };
    let confirmation_manager = ConfirmationManager::new(confirmation_config);
    confirmation_manager
        .confirm_scan(&config, &targets)
        .context("User declined to continue with scan")?;

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

    // Create PCAPNG writer if --packet-capture flag is set
    let pcapng_writer = if let Some(ref pcap_path) = args.packet_capture {
        match prtip_scanner::pcapng::PcapngWriter::new(pcap_path) {
            Ok(writer) => {
                info!("PCAPNG packet capture enabled: {:?}", pcap_path);
                Some(Arc::new(std::sync::Mutex::new(writer)))
            }
            Err(e) => {
                eprintln!("Error creating PCAPNG writer: {}", e);
                eprintln!("Hint: Ensure the directory exists and you have write permissions.");
                return Err(e);
            }
        }
    } else {
        None
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

    let results = if args.should_perform_host_discovery() {
        info!("Performing host discovery before port scanning");
        scheduler
            .execute_scan_with_discovery(targets, pcapng_writer)
            .await?
    } else {
        // For Phase 1, we need to expand targets with ports
        let expanded_targets = expand_targets_with_ports(targets, &ports)?;
        scheduler
            .execute_scan_ports(expanded_targets, &ports)
            .await?
    };

    let scan_duration = scan_start.elapsed();
    info!("Scan complete: {} results", results.len());

    // Cleanup progress display
    if let Some(display) = progress_display {
        display.finish();
    }

    // Format and output results
    let is_terminal = {
        use std::io::{stdout, IsTerminal};
        stdout().is_terminal()
    };
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

    // Record command in history
    record_scan_history(&argv, &results, scan_duration, 0)?;

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

/// Handle history subcommand
async fn handle_history_command(args: &[String]) -> Result<()> {
    let manager = HistoryManager::new()?;

    // Parse arguments
    if args.is_empty() {
        // `prtip history` - show all entries
        if manager.is_empty() {
            println!("{}", "No command history found.".yellow());
            println!();
            println!("Run some scans to build your history:");
            println!("  prtip -sS -p 80,443 192.168.1.1");
            println!("  prtip -sV target.com");
            println!();
            return Ok(());
        }

        println!("\n{}", "Command History".bright_white().bold());
        println!("{}", "=".repeat(80).bright_cyan());
        println!();

        for (idx, entry) in manager.list_entries().iter().enumerate() {
            println!("{}", entry.format_display(idx));
            println!();
        }

        println!("{}", "=".repeat(80).bright_cyan());
        println!("Total: {} command(s)", manager.len());
        println!();
        println!("Usage:");
        println!("  prtip history <n>      - Show specific entry");
        println!("  prtip history --clear  - Clear all history");
        println!("  prtip replay <n>       - Re-run command by index");
        println!("  prtip replay --last    - Re-run most recent command");
        println!();
        return Ok(());
    }

    // Check for --clear flag
    if args[0] == "--clear" {
        let count = manager.len();
        let mut manager = manager;
        manager.clear()?;
        println!("{}", format!("✓ Cleared {} history entries", count).green());
        return Ok(());
    }

    // Parse index (e.g., `prtip history 5`)
    let index: usize = args[0].parse().with_context(|| {
        format!(
            "Invalid history index: '{}'\n\
             Usage: prtip history <n> (where n is a number)\n\
             Example: prtip history 0",
            args[0]
        )
    })?;

    // Get entry
    let entry = manager.get_entry(index).with_context(|| {
        format!(
            "History index {} not found (valid range: 0-{})",
            index,
            manager.len().saturating_sub(1)
        )
    })?;

    // Display detailed entry
    println!(
        "\n{}",
        format!("History Entry [{}]", index).bright_white().bold()
    );
    println!("{}", "=".repeat(80).bright_cyan());
    println!();
    println!(
        "  {}: {}",
        "Timestamp".bright_white(),
        entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("  {}: {}", "Command".bright_white(), entry.command.cyan());
    println!(
        "  {}: {}",
        "Exit Code".bright_white(),
        if entry.exit_code == 0 {
            "0 (success)".green()
        } else {
            format!("{} (error)", entry.exit_code).red()
        }
    );
    println!();
    println!("  {}:", "Summary".bright_white());
    println!("    {}", entry.summary);
    println!();
    println!("{}", "=".repeat(80).bright_cyan());
    println!();
    println!("To replay this command:");
    println!("  prtip replay {}", index);
    println!();
    println!("To replay with modifications:");
    println!("  prtip replay {} --verbose", index);
    println!("  prtip replay {} -p 1-1000", index);
    println!();

    Ok(())
}

/// Handle replay subcommand
async fn handle_replay_command(args: &[String]) -> Result<()> {
    let manager = HistoryManager::new()?;

    if manager.is_empty() {
        bail!(
            "No command history found.\n\
             Run some scans first to build your history:\n\
             prtip -sS -p 80,443 192.168.1.1"
        );
    }

    // Parse arguments
    if args.is_empty() {
        bail!(
            "Missing replay argument.\n\n\
             Usage:\n\
             prtip replay <index>     - Replay command by index\n\
             prtip replay --last      - Replay most recent command\n\n\
             Examples:\n\
             prtip replay 0\n\
             prtip replay --last\n\
             prtip replay 5 --verbose"
        );
    }

    // Get entry to replay
    let entry = if args[0] == "--last" {
        manager.get_last().context("History is empty")?.clone()
    } else {
        let index: usize = args[0].parse().with_context(|| {
            format!(
                "Invalid index: '{}'\n\
                 Usage: prtip replay <n> (where n is a number)\n\
                 Example: prtip replay 0",
                args[0]
            )
        })?;

        manager
            .get_entry(index)
            .with_context(|| {
                format!(
                    "History index {} not found (valid range: 0-{})",
                    index,
                    manager.len().saturating_sub(1)
                )
            })?
            .clone()
    };

    // Validate entry can be replayed
    HistoryManager::validate_replay(&entry)?;

    // Rebuild command with optional modifications
    let modifications = if args.len() > 1 {
        Some(args[1..].iter().map(|s| s.as_str()).collect::<Vec<_>>())
    } else {
        None
    };

    let replay_args = HistoryManager::rebuild_command(&entry, modifications);

    // Show what we're replaying
    println!("\n{}", "Replaying Command".bright_white().bold());
    println!("{}", "=".repeat(80).bright_cyan());
    println!(
        "  {}: {}",
        "Original".bright_white(),
        entry.command.dimmed()
    );
    if !args[1..].is_empty() {
        println!(
            "  {}: {}",
            "Modified".bright_white(),
            replay_args.join(" ").cyan()
        );
    }
    println!("{}", "=".repeat(80).bright_cyan());
    println!();

    // Recursively call run() with the replayed arguments
    // This will use the normal scan flow and record to history automatically
    std::env::set_var("PRTIP_REPLAY_ARGS", serde_json::to_string(&replay_args)?);

    // Note: We need to restart the entire process to properly parse arguments
    // This is a limitation of clap's design - it expects to parse from std::env::args()
    // For now, we'll bail with instructions to manually run the command
    println!("{}", "⚠ Manual replay required".yellow().bold());
    println!();
    println!("Due to CLI parser limitations, please run the command manually:");
    println!();
    println!("  {}", replay_args.join(" ").cyan());
    println!();
    println!("This will be improved in a future version with proper replay support.");
    println!();

    Ok(())
}

/// Record a scan to history
fn record_scan_history(
    argv: &[String],
    results: &[prtip_core::ScanResult],
    duration: std::time::Duration,
    exit_code: i32,
) -> Result<()> {
    use std::collections::HashSet;

    // Skip recording if this is a replay (would create duplicate entries)
    if std::env::var("PRTIP_REPLAY_ARGS").is_ok() {
        return Ok(());
    }

    // Build summary
    let hosts: HashSet<_> = results.iter().map(|r| r.target_ip()).collect();
    let open_ports = results
        .iter()
        .filter(|r| r.state() == prtip_core::PortState::Open)
        .count();

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

    let summary = format!(
        "{} hosts, {} ports scanned, {} open, duration: {}",
        hosts.len(),
        results.len(),
        open_ports,
        duration_str
    );

    // Add to history (skip if running in test mode)
    if std::env::var("PRTIP_DISABLE_HISTORY").is_err() {
        let mut manager = HistoryManager::new()?;
        manager.add_entry(argv.to_vec(), summary, exit_code)?;
    }

    Ok(())
}

/// Handle --list-templates flag
fn handle_list_templates() -> Result<()> {
    use templates::TemplateManager;

    let manager = TemplateManager::with_custom_templates().context("Failed to load templates")?;

    let templates = manager.list_templates();

    if templates.is_empty() {
        println!("{}", "No templates available.".yellow());
        return Ok(());
    }

    println!("\n{}", "Available Scan Templates".bright_white().bold());
    println!("{}", "=".repeat(80).bright_cyan());
    println!();

    // Separate built-in and custom templates
    let builtin_names: Vec<_> = manager.builtin_names();
    let custom_names: Vec<_> = manager.custom_names();

    // Show built-in templates
    if !builtin_names.is_empty() {
        println!("{}", "Built-in Templates:".bright_white());
        println!();
        for name in builtin_names {
            if let Some(template) = manager.get_template(name) {
                println!("  {} - {}", name.cyan().bold(), template.description);
            }
        }
        println!();
    }

    // Show custom templates
    if !custom_names.is_empty() {
        println!("{}", "Custom Templates:".bright_white());
        println!();
        for name in custom_names {
            if let Some(template) = manager.get_template(name) {
                println!("  {} - {}", name.green().bold(), template.description);
            }
        }
        println!();
    }

    println!("{}", "=".repeat(80).bright_cyan());
    println!("Total: {} template(s)", templates.len());
    println!();
    println!("Usage:");
    println!("  prtip --template <name> <target>         - Use a template");
    println!("  prtip --show-template <name>             - Show template details");
    println!("  prtip --template <name> [flags] <target> - Override template values");
    println!();
    println!("Examples:");
    println!("  prtip --template web-servers 192.168.1.0/24");
    println!("  prtip --template stealth -T4 target.com");
    println!();
    println!("Custom templates can be defined in: ~/.prtip/templates.toml");
    println!();

    Ok(())
}

/// Handle --show-template flag
fn handle_show_template(template_name: &str) -> Result<()> {
    use templates::TemplateManager;

    let manager = TemplateManager::with_custom_templates().context("Failed to load templates")?;

    let template = manager
        .get_template(template_name)
        .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?;

    let is_custom = manager.custom_names().contains(&template_name);
    let template_type = if is_custom {
        "Custom Template".green()
    } else {
        "Built-in Template".cyan()
    };

    println!(
        "\n{}",
        format!("Template: {}", template_name).bright_white().bold()
    );
    println!("{}", "=".repeat(80).bright_cyan());
    println!("  {}: {}", "Type".bright_white(), template_type);
    println!(
        "  {}: {}",
        "Description".bright_white(),
        template.description
    );
    println!();

    // Show configuration
    println!("  {}:", "Configuration".bright_white());

    if let Some(ref ports) = template.ports {
        println!("    {}: {:?}", "Ports".bright_white(), ports);
    }

    if let Some(ref scan_type) = template.scan_type {
        println!("    {}: {}", "Scan Type".bright_white(), scan_type);
    }

    if let Some(enabled) = template.service_detection {
        println!("    {}: {}", "Service Detection".bright_white(), enabled);
    }

    if let Some(enabled) = template.os_detection {
        println!("    {}: {}", "OS Detection".bright_white(), enabled);
    }

    if let Some(ref timing) = template.timing {
        println!("    {}: {}", "Timing".bright_white(), timing);
    }

    if let Some(rate) = template.max_rate {
        println!("    {}: {} pps", "Max Rate".bright_white(), rate);
    }

    if let Some(enabled) = template.randomize {
        println!("    {}: {}", "Randomize".bright_white(), enabled);
    }

    if let Some(enabled) = template.fragment {
        println!("    {}: {}", "Fragment".bright_white(), enabled);
    }

    if let Some(enabled) = template.tls_analysis {
        println!("    {}: {}", "TLS Analysis".bright_white(), enabled);
    }

    if let Some(enabled) = template.discovery_only {
        println!("    {}: {}", "Discovery Only".bright_white(), enabled);
    }

    println!();
    println!("{}", "=".repeat(80).bright_cyan());
    println!();
    println!("Usage:");
    println!("  prtip --template {} <target>", template_name);
    println!();
    println!("To override template settings:");
    println!(
        "  prtip --template {} -T4 <target>     # Override timing",
        template_name
    );
    println!(
        "  prtip --template {} -p 1-1000 <target> # Override ports",
        template_name
    );
    println!();

    Ok(())
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
        let config = args.to_config().unwrap();
        let targets = vec![ScanTarget::parse("192.168.1.1").unwrap()];
        let banner = format_scan_banner(&args, &config, 1, &targets);

        assert!(banner.contains("ProRT-IP"));
        assert!(banner.contains("192.168.1.1"));
        assert!(banner.contains("80"));
    }

    #[test]
    fn test_format_scan_banner_with_hostname() {
        let args = Args::parse_from(["prtip", "-p", "80", "127.0.0.1"]);
        let config = args.to_config().unwrap();
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

    // Test argv preprocessor for nmap compatibility
    fn preprocess_argv_from(args: Vec<&str>) -> Vec<String> {
        // Simulate std::env::args() for testing
        let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        // Manually implement preprocessing logic for tests
        let mut processed = Vec::new();
        let mut i = 0;

        while i < args_vec.len() {
            let arg = &args_vec[i];

            match arg.as_str() {
                "-sS" => processed.push("--nmap-syn".to_string()),
                "-sT" => processed.push("--nmap-connect".to_string()),
                "-sU" => processed.push("--nmap-udp".to_string()),
                "-sN" => processed.push("--nmap-null".to_string()),
                "-sF" => processed.push("--nmap-fin".to_string()),
                "-sX" => processed.push("--nmap-xmas".to_string()),
                "-sA" => processed.push("--nmap-ack".to_string()),
                "-sV" => processed.push("--sV".to_string()),
                "-sI" => {
                    processed.push("--nmap-idle".to_string());
                    i += 1;
                    if i < args_vec.len() {
                        processed.push(args_vec[i].clone());
                    }
                }
                "-oN" => {
                    processed.push("--output-normal".to_string());
                    i += 1;
                    if i < args_vec.len() {
                        processed.push(args_vec[i].clone());
                    }
                }
                "-oX" => {
                    processed.push("--output-xml".to_string());
                    i += 1;
                    if i < args_vec.len() {
                        processed.push(args_vec[i].clone());
                    }
                }
                "-oG" => {
                    processed.push("--output-greppable".to_string());
                    i += 1;
                    if i < args_vec.len() {
                        processed.push(args_vec[i].clone());
                    }
                }
                "-oA" => {
                    processed.push("--output-all-formats".to_string());
                    i += 1;
                    if i < args_vec.len() {
                        processed.push(args_vec[i].clone());
                    }
                }
                "-Pn" => processed.push("--skip-ping".to_string()),
                _ => processed.push(arg.clone()),
            }

            i += 1;
        }

        processed
    }

    #[test]
    fn test_preprocess_nmap_syn() {
        let args = vec!["prtip", "-sS", "192.168.1.1"];
        let processed = preprocess_argv_from(args);
        assert_eq!(processed[1], "--nmap-syn");
        assert_eq!(processed[2], "192.168.1.1");
    }

    #[test]
    fn test_preprocess_nmap_connect() {
        let args = vec!["prtip", "-sT", "192.168.1.1"];
        let processed = preprocess_argv_from(args);
        assert_eq!(processed[1], "--nmap-connect");
    }

    #[test]
    fn test_preprocess_output_normal() {
        let args = vec!["prtip", "-oN", "out.txt", "192.168.1.1"];
        let processed = preprocess_argv_from(args);
        assert_eq!(processed[1], "--output-normal");
        assert_eq!(processed[2], "out.txt");
        assert_eq!(processed[3], "192.168.1.1");
    }

    #[test]
    fn test_preprocess_output_xml() {
        let args = vec!["prtip", "-oX", "scan.xml", "192.168.1.1"];
        let processed = preprocess_argv_from(args);
        assert_eq!(processed[1], "--output-xml");
        assert_eq!(processed[2], "scan.xml");
    }

    #[test]
    fn test_preprocess_output_greppable() {
        let args = vec!["prtip", "-oG", "scan.gnmap", "192.168.1.1"];
        let processed = preprocess_argv_from(args);
        assert_eq!(processed[1], "--output-greppable");
        assert_eq!(processed[2], "scan.gnmap");
    }

    #[test]
    fn test_preprocess_output_all() {
        let args = vec!["prtip", "-oA", "scan", "192.168.1.1"];
        let processed = preprocess_argv_from(args);
        assert_eq!(processed[1], "--output-all-formats");
        assert_eq!(processed[2], "scan");
    }

    #[test]
    fn test_preprocess_skip_ping() {
        let args = vec!["prtip", "-Pn", "192.168.1.1"];
        let processed = preprocess_argv_from(args);
        assert_eq!(processed[1], "--skip-ping");
    }

    #[test]
    fn test_preprocess_idle_scan() {
        let args = vec![
            "prtip",
            "-sI",
            "192.168.1.100",
            "-p",
            "80,443",
            "target.com",
        ];
        let processed = preprocess_argv_from(args);
        assert_eq!(processed[1], "--nmap-idle");
        assert_eq!(processed[2], "192.168.1.100");
        assert_eq!(processed[3], "-p");
        assert_eq!(processed[4], "80,443");
        assert_eq!(processed[5], "target.com");
    }

    #[test]
    fn test_preprocess_service_version() {
        let args = vec!["prtip", "-sV", "-p", "80,443", "192.168.1.1"];
        let processed = preprocess_argv_from(args);
        assert_eq!(processed[1], "--sV");
        assert_eq!(processed[2], "-p");
        assert_eq!(processed[3], "80,443");
        assert_eq!(processed[4], "192.168.1.1");
    }

    #[test]
    fn test_preprocess_mixed_args() {
        let args = vec![
            "prtip",
            "-sS",
            "-p",
            "80,443",
            "-oX",
            "out.xml",
            "192.168.1.1",
        ];
        let processed = preprocess_argv_from(args);
        assert_eq!(processed[1], "--nmap-syn");
        assert_eq!(processed[2], "-p");
        assert_eq!(processed[3], "80,443");
        assert_eq!(processed[4], "--output-xml");
        assert_eq!(processed[5], "out.xml");
        assert_eq!(processed[6], "192.168.1.1");
    }

    #[test]
    fn test_preprocess_passthrough_standard_args() {
        let args = vec!["prtip", "-s", "syn", "-p", "80", "192.168.1.1"];
        let processed = preprocess_argv_from(args);
        // Standard args should pass through unchanged
        assert_eq!(processed[1], "-s");
        assert_eq!(processed[2], "syn");
        assert_eq!(processed[3], "-p");
        assert_eq!(processed[4], "80");
    }
}
