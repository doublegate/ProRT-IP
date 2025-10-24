//! Database Command Handlers
//!
//! Provides command handlers for database operations:
//! - `prtip db list` - List all scans
//! - `prtip db query` - Query scan results with filters
//! - `prtip db export` - Export scan results to various formats
//! - `prtip db compare` - Compare two scans

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use prtip_core::PortState;
use prtip_scanner::DbReader;
use std::path::PathBuf;

/// Database operations
#[derive(Debug, Parser)]
#[command(name = "db", about = "Database operations")]
pub struct DbCommand {
    #[command(subcommand)]
    pub command: DbSubcommand,
}

/// Database subcommands
#[derive(Debug, Subcommand)]
pub enum DbSubcommand {
    /// List all scans in database
    List {
        /// Path to database file
        #[arg(value_name = "DB_PATH")]
        db_path: PathBuf,
    },

    /// Query scan results with filters
    Query {
        /// Path to database file
        #[arg(value_name = "DB_PATH")]
        db_path: PathBuf,

        /// Filter by scan ID
        #[arg(long, value_name = "ID")]
        scan_id: Option<i64>,

        /// Filter by target IP
        #[arg(long, value_name = "IP")]
        target: Option<String>,

        /// Filter by port number
        #[arg(long, value_name = "PORT")]
        port: Option<u16>,

        /// Filter by service name
        #[arg(long, value_name = "SERVICE")]
        service: Option<String>,

        /// Show only open ports
        #[arg(long)]
        open: bool,
    },

    /// Export scan results to file
    Export {
        /// Path to database file
        #[arg(value_name = "DB_PATH")]
        db_path: PathBuf,

        /// Scan ID to export
        #[arg(long, value_name = "ID", required = true)]
        scan_id: i64,

        /// Output format
        #[arg(long, value_name = "FORMAT", default_value = "json")]
        format: ExportFormat,

        /// Output file path
        #[arg(short = 'o', long, value_name = "FILE")]
        output: PathBuf,
    },

    /// Compare two scans
    Compare {
        /// Path to database file
        #[arg(value_name = "DB_PATH")]
        db_path: PathBuf,

        /// First scan ID
        #[arg(value_name = "SCAN_ID_1")]
        scan_id_1: i64,

        /// Second scan ID
        #[arg(value_name = "SCAN_ID_2")]
        scan_id_2: i64,
    },
}

/// Export format options
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ExportFormat {
    /// JSON format (machine-readable)
    Json,
    /// CSV format (spreadsheet-compatible)
    Csv,
    /// XML format (Nmap-compatible)
    Xml,
    /// Text format (human-readable)
    Text,
}

/// Handle database list command
pub async fn handle_list(db_path: PathBuf) -> Result<()> {
    let reader = DbReader::new(
        db_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid database path: {:?}", db_path))?,
    )
    .await
    .context("Failed to open database")?;

    let scans = reader.list_scans().await.context("Failed to list scans")?;

    if scans.is_empty() {
        println!("{}", "No scans found in database".yellow());
        return Ok(());
    }

    println!("\n{}", "Scans in Database".bright_white().bold());
    println!("{}", "=".repeat(80).bright_cyan());
    println!(
        "{:<8} {:<20} {:<20} {:<10}",
        "ID".bright_white().bold(),
        "Start Time".bright_white().bold(),
        "End Time".bright_white().bold(),
        "Results".bright_white().bold()
    );
    println!("{}", "=".repeat(80).bright_cyan());

    for scan in &scans {
        let end_time_str = scan
            .end_time
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "In Progress".dimmed().to_string());

        println!(
            "{:<8} {:<20} {:<20} {:<10}",
            scan.id.to_string().bright_cyan(),
            scan.start_time.format("%Y-%m-%d %H:%M:%S"),
            end_time_str,
            scan.result_count.to_string().green()
        );
    }

    println!("{}", "=".repeat(80).bright_cyan());
    println!("Total: {} scan(s)\n", scans.len());

    Ok(())
}

/// Handle database query command
pub async fn handle_query(
    db_path: PathBuf,
    scan_id: Option<i64>,
    target: Option<String>,
    port: Option<u16>,
    service: Option<String>,
    open: bool,
) -> Result<()> {
    let reader = DbReader::new(
        db_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid database path: {:?}", db_path))?,
    )
    .await
    .context("Failed to open database")?;

    // Validate that at least one filter is provided
    if scan_id.is_none() && target.is_none() && port.is_none() && service.is_none() {
        anyhow::bail!(
            "At least one filter must be provided (--scan-id, --target, --port, or --service)"
        );
    }

    // Query by scan ID
    if let Some(id) = scan_id {
        let results = reader
            .get_scan_results(id)
            .await
            .context(format!("Failed to get results for scan {}", id))?;

        if results.is_empty() {
            println!("{}", format!("No results found for scan {}", id).yellow());
            return Ok(());
        }

        println!(
            "\n{}",
            format!("Results for Scan {}", id).bright_white().bold()
        );
        println!("{}", "=".repeat(80).bright_cyan());

        print_results(&results, open);
        return Ok(());
    }

    // Query by target IP
    if let Some(ref ip_str) = target {
        // Validate IP address format
        let _ip: std::net::IpAddr = ip_str
            .parse()
            .context(format!("Invalid IP address: {}", ip_str))?;
        let ports = reader
            .query_open_ports(ip_str)
            .await
            .context(format!("Failed to query ports for {}", ip_str))?;

        if ports.is_empty() {
            println!("{}", format!("No open ports found for {}", ip_str).yellow());
            return Ok(());
        }

        println!(
            "\n{}",
            format!("Open Ports for {}", ip_str).bright_white().bold()
        );
        println!("{}", "=".repeat(80).bright_cyan());
        println!(
            "{:<8} {:<12} {:<20} {:<20} {:<10}",
            "Port".bright_white().bold(),
            "Protocol".bright_white().bold(),
            "Service".bright_white().bold(),
            "Version".bright_white().bold(),
            "RTT (ms)".bright_white().bold()
        );
        println!("{}", "=".repeat(80).bright_cyan());

        for port in ports {
            println!(
                "{:<8} {:<12} {:<20} {:<20} {:<10}",
                port.port.to_string().green(),
                port.protocol,
                port.service.as_deref().unwrap_or("-").cyan(),
                port.version.as_deref().unwrap_or("-"),
                port.response_time_ms
            );
        }

        println!("{}", "=".repeat(80).bright_cyan());
        return Ok(());
    }

    // Query by port
    if let Some(p) = port {
        let hosts = reader
            .query_by_port(p)
            .await
            .context(format!("Failed to query hosts with port {}", p))?;

        if hosts.is_empty() {
            println!(
                "{}",
                format!("No hosts found with port {} open", p).yellow()
            );
            return Ok(());
        }

        println!(
            "\n{}",
            format!("Hosts with Port {} Open", p).bright_white().bold()
        );
        println!("{}", "=".repeat(80).bright_cyan());
        println!(
            "{:<18} {:<8} {:<12} {:<20} {:<20}",
            "Target IP".bright_white().bold(),
            "Port".bright_white().bold(),
            "State".bright_white().bold(),
            "Service".bright_white().bold(),
            "Version".bright_white().bold()
        );
        println!("{}", "=".repeat(80).bright_cyan());

        for host in hosts {
            let state_str = format_port_state(host.state);
            println!(
                "{:<18} {:<8} {:<12} {:<20} {:<20}",
                host.target_ip.to_string().bright_cyan(),
                host.port.to_string().green(),
                state_str,
                host.service.as_deref().unwrap_or("-").cyan(),
                host.version.as_deref().unwrap_or("-")
            );
        }

        println!("{}", "=".repeat(80).bright_cyan());
        return Ok(());
    }

    // Query by service
    if let Some(ref svc) = service {
        let hosts = reader
            .query_by_service(svc)
            .await
            .context(format!("Failed to query hosts running {}", svc))?;

        if hosts.is_empty() {
            println!("{}", format!("No hosts found running {}", svc).yellow());
            return Ok(());
        }

        println!(
            "\n{}",
            format!("Hosts Running {}", svc).bright_white().bold()
        );
        println!("{}", "=".repeat(80).bright_cyan());
        println!(
            "{:<18} {:<8} {:<12} {:<20} {:<20}",
            "Target IP".bright_white().bold(),
            "Port".bright_white().bold(),
            "State".bright_white().bold(),
            "Service".bright_white().bold(),
            "Version".bright_white().bold()
        );
        println!("{}", "=".repeat(80).bright_cyan());

        for host in hosts {
            let state_str = format_port_state(host.state);
            println!(
                "{:<18} {:<8} {:<12} {:<20} {:<20}",
                host.target_ip.to_string().bright_cyan(),
                host.port.to_string().green(),
                state_str,
                host.service.as_deref().unwrap_or("-").cyan(),
                host.version.as_deref().unwrap_or("-")
            );
        }

        println!("{}", "=".repeat(80).bright_cyan());
        return Ok(());
    }

    Ok(())
}

/// Handle database export command
pub async fn handle_export(
    db_path: PathBuf,
    scan_id: i64,
    format: ExportFormat,
    output: PathBuf,
) -> Result<()> {
    let reader = DbReader::new(
        db_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid database path: {:?}", db_path))?,
    )
    .await
    .context("Failed to open database")?;

    let results = reader
        .get_scan_results(scan_id)
        .await
        .context(format!("Failed to get results for scan {}", scan_id))?;

    if results.is_empty() {
        println!(
            "{}",
            format!("No results found for scan {}", scan_id).yellow()
        );
        return Ok(());
    }

    let output_str = match format {
        ExportFormat::Json => crate::export::export_json(&results)
            .map_err(|e| anyhow::anyhow!("Failed to export to JSON: {}", e))?,
        ExportFormat::Csv => crate::export::export_csv(&results)
            .map_err(|e| anyhow::anyhow!("Failed to export to CSV: {}", e))?,
        ExportFormat::Xml => crate::export::export_xml(&results)
            .map_err(|e| anyhow::anyhow!("Failed to export to XML: {}", e))?,
        ExportFormat::Text => crate::export::export_text(&results)
            .map_err(|e| anyhow::anyhow!("Failed to export to text: {}", e))?,
    };

    std::fs::write(&output, output_str).context(format!("Failed to write to {:?}", output))?;

    println!(
        "{} Exported {} results from scan {} to {:?}",
        "[✓]".green(),
        results.len(),
        scan_id,
        output
    );

    Ok(())
}

/// Handle database compare command
pub async fn handle_compare(db_path: PathBuf, scan_id_1: i64, scan_id_2: i64) -> Result<()> {
    let reader = DbReader::new(
        db_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid database path: {:?}", db_path))?,
    )
    .await
    .context("Failed to open database")?;

    let comparison = reader
        .compare_scans(scan_id_1, scan_id_2)
        .await
        .context(format!(
            "Failed to compare scans {} and {}",
            scan_id_1, scan_id_2
        ))?;

    println!(
        "\n{}",
        format!("Comparing Scan {} vs Scan {}", scan_id_1, scan_id_2)
            .bright_white()
            .bold()
    );
    println!("{}", "=".repeat(80).bright_cyan());

    // New ports (in scan2 but not scan1)
    if !comparison.new_open_ports.is_empty() {
        println!("\n{}", "New Open Ports:".green().bold());
        println!("{}", "-".repeat(80));
        for result in &comparison.new_open_ports {
            println!(
                "  {} → Port {} {} {}",
                result.target_ip.to_string().bright_cyan(),
                result.port.to_string().green().bold(),
                result.service.as_deref().unwrap_or("unknown").cyan(),
                result
                    .version
                    .as_deref()
                    .map(|v| format!("({})", v))
                    .unwrap_or_default()
            );
        }
    }

    // Closed ports (in scan1 but not scan2)
    if !comparison.closed_ports.is_empty() {
        println!("\n{}", "Closed Ports:".red().bold());
        println!("{}", "-".repeat(80));
        for result in &comparison.closed_ports {
            println!(
                "  {} → Port {} {} {}",
                result.target_ip.to_string().bright_cyan(),
                result.port.to_string().red().bold(),
                result.service.as_deref().unwrap_or("unknown").dimmed(),
                result
                    .version
                    .as_deref()
                    .map(|v| format!("({})", v))
                    .unwrap_or_default()
                    .dimmed()
            );
        }
    }

    // Changed services
    if !comparison.changed_services.is_empty() {
        println!("\n{}", "Changed Services:".yellow().bold());
        println!("{}", "-".repeat(80));
        for (old_result, new_result) in &comparison.changed_services {
            println!(
                "  {} → Port {} {} → {}",
                old_result.target_ip.to_string().bright_cyan(),
                old_result.port.to_string().yellow(),
                old_result.service.as_deref().unwrap_or("unknown").dimmed(),
                new_result
                    .service
                    .as_deref()
                    .unwrap_or("unknown")
                    .cyan()
                    .bold()
            );
        }
    }

    // Summary
    println!("\n{}", "Summary:".bright_white().bold());
    println!("{}", "-".repeat(80));
    println!(
        "  New ports:        {}",
        comparison.new_open_ports.len().to_string().green()
    );
    println!(
        "  Closed ports:     {}",
        comparison.closed_ports.len().to_string().red()
    );
    println!(
        "  Changed services: {}",
        comparison.changed_services.len().to_string().yellow()
    );
    println!(
        "  New hosts:        {}",
        comparison.new_hosts.len().to_string().cyan()
    );
    println!(
        "  Disappeared hosts: {}",
        comparison.disappeared_hosts.len().to_string().dimmed()
    );
    println!("{}", "=".repeat(80).bright_cyan());

    Ok(())
}

/// Helper: Print scan results in table format
fn print_results(results: &[prtip_core::ScanResult], open_only: bool) {
    println!(
        "{:<18} {:<8} {:<12} {:<20} {:<20}",
        "Target IP".bright_white().bold(),
        "Port".bright_white().bold(),
        "State".bright_white().bold(),
        "Service".bright_white().bold(),
        "Version".bright_white().bold()
    );
    println!("{}", "=".repeat(80).bright_cyan());

    let mut count = 0;
    for result in results {
        // Filter by open_only flag
        if open_only && result.state != PortState::Open {
            continue;
        }

        let state_str = format_port_state(result.state);
        println!(
            "{:<18} {:<8} {:<12} {:<20} {:<20}",
            result.target_ip.to_string().bright_cyan(),
            result.port.to_string().green(),
            state_str,
            result.service.as_deref().unwrap_or("-").cyan(),
            result.version.as_deref().unwrap_or("-")
        );
        count += 1;
    }

    println!("{}", "=".repeat(80).bright_cyan());
    println!("Total: {} result(s)\n", count);
}

/// Helper: Format port state with color
fn format_port_state(state: PortState) -> colored::ColoredString {
    match state {
        PortState::Open => "open".green(),
        PortState::Closed => "closed".red(),
        PortState::Filtered => "filtered".yellow(),
        PortState::Unknown => "unknown".dimmed(),
    }
}
