//! Output formatters for scan results

use anyhow::Result;
use chrono::Utc;
use colored::*;
use prtip_core::{Config, PortState, ScanResult};
use serde::Serialize;
use std::collections::BTreeMap;
use std::net::IpAddr;

/// Output formatter trait
pub trait OutputFormatter {
    /// Format scan results into a string
    fn format_results(&self, results: &[ScanResult], config: &Config) -> Result<String>;
}

/// Text output formatter (human-readable with colors)
pub struct TextFormatter {
    colorize: bool,
}

impl TextFormatter {
    /// Create a new text formatter
    ///
    /// # Arguments
    ///
    /// * `colorize` - Whether to use terminal colors
    pub fn new(colorize: bool) -> Self {
        Self { colorize }
    }

    /// Format a port state with optional colorization
    fn format_state(&self, state: PortState) -> String {
        if !self.colorize {
            return state.to_string();
        }

        match state {
            PortState::Open => "open".green().bold().to_string(),
            PortState::Closed => "closed".red().to_string(),
            PortState::Filtered => "filtered".yellow().to_string(),
            PortState::Unknown => "unknown".white().to_string(),
        }
    }

    /// Format a number with color (if enabled)
    fn format_number(&self, n: usize) -> String {
        if self.colorize {
            n.to_string().cyan().bold().to_string()
        } else {
            n.to_string()
        }
    }

    /// Format an IP address with color (if enabled)
    fn format_ip(&self, ip: &IpAddr) -> String {
        if self.colorize {
            ip.to_string().bright_blue().bold().to_string()
        } else {
            ip.to_string()
        }
    }

    /// Format section header
    fn format_header(&self, text: &str) -> String {
        if self.colorize {
            format!("\n{}\n", text.bright_white().bold())
        } else {
            format!("\n{}\n", text)
        }
    }
}

impl OutputFormatter for TextFormatter {
    fn format_results(&self, results: &[ScanResult], config: &Config) -> Result<String> {
        let mut output = String::new();

        // Header
        output.push_str(&self.format_header("=== ProRT-IP Scan Results ==="));
        output.push_str(&format!(
            "Scan Time: {}\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!("Scan Type: {}\n", config.scan.scan_type));
        output.push_str(&format!(
            "Timing Template: {}\n",
            config.scan.timing_template
        ));
        output.push_str(&format!(
            "Total Results: {}\n",
            self.format_number(results.len())
        ));

        if results.is_empty() {
            output.push_str("\nNo results found.\n");
            return Ok(output);
        }

        // Group results by host
        let mut by_host: BTreeMap<IpAddr, Vec<&ScanResult>> = BTreeMap::new();
        for result in results {
            by_host.entry(result.target_ip).or_default().push(result);
        }

        // Statistics
        let total_hosts = by_host.len();
        let total_open = results
            .iter()
            .filter(|r| r.state == PortState::Open)
            .count();
        let total_closed = results
            .iter()
            .filter(|r| r.state == PortState::Closed)
            .count();
        let total_filtered = results
            .iter()
            .filter(|r| r.state == PortState::Filtered)
            .count();

        output.push_str(&format!(
            "\nHosts Scanned: {}\n",
            self.format_number(total_hosts)
        ));
        output.push_str(&format!(
            "Ports: {} open, {} closed, {} filtered\n",
            self.format_number(total_open),
            self.format_number(total_closed),
            self.format_number(total_filtered)
        ));

        // Output by host
        for (host, host_results) in &by_host {
            output.push_str(&self.format_header(&format!("Host: {}", self.format_ip(host))));

            // Count states for this host
            let open_count = host_results
                .iter()
                .filter(|r| r.state == PortState::Open)
                .count();
            let closed_count = host_results
                .iter()
                .filter(|r| r.state == PortState::Closed)
                .count();
            let filtered_count = host_results
                .iter()
                .filter(|r| r.state == PortState::Filtered)
                .count();

            output.push_str(&format!(
                "Ports: {} open, {} closed, {} filtered\n",
                self.format_number(open_count),
                self.format_number(closed_count),
                self.format_number(filtered_count)
            ));

            // List open ports prominently
            if open_count > 0 {
                output.push_str("\nOpen Ports:\n");
                for result in host_results.iter().filter(|r| r.state == PortState::Open) {
                    output.push_str(&format!(
                        "  {:5} {:12} ({:6.2}ms)",
                        result.port,
                        self.format_state(result.state),
                        result.response_time.as_secs_f64() * 1000.0
                    ));

                    if let Some(service) = &result.service {
                        if let Some(version) = &result.version {
                            output.push_str(&format!(" [{} ({})]", service, version));
                        } else {
                            output.push_str(&format!(" [{}]", service));
                        }
                    } else if let Some(version) = &result.version {
                        output.push_str(&format!(" [version: {}]", version));
                    }

                    output.push('\n');

                    if let Some(banner) = &result.banner {
                        let truncated = if banner.len() > 70 {
                            format!("{}...", &banner[..67])
                        } else {
                            banner.clone()
                        };
                        output.push_str(&format!("        Banner: {}\n", truncated));
                    }

                    if let Some(raw_response) = &result.raw_response {
                        if !raw_response.is_empty() {
                            output.push_str(&format!("        Raw Response: {:?}\n", raw_response));
                        }
                    }
                }
            }

            // List filtered ports if verbose
            if config.output.verbose > 0 && filtered_count > 0 {
                output.push_str("\nFiltered Ports:\n");
                for result in host_results
                    .iter()
                    .filter(|r| r.state == PortState::Filtered)
                {
                    output.push_str(&format!(
                        "  {:5} {:12}\n",
                        result.port,
                        self.format_state(result.state)
                    ));
                }
            }

            // List closed ports if very verbose
            if config.output.verbose > 1 && closed_count > 0 {
                output.push_str("\nClosed Ports:\n");
                let mut count = 0;
                for result in host_results.iter().filter(|r| r.state == PortState::Closed) {
                    output.push_str(&format!("{:5} ", result.port));
                    count += 1;
                    if count % 10 == 0 {
                        output.push('\n');
                    }
                }
                if count % 10 != 0 {
                    output.push('\n');
                }
            }

            output.push('\n');
        }

        Ok(output)
    }
}

/// JSON output formatter
pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format_results(&self, results: &[ScanResult], config: &Config) -> Result<String> {
        #[derive(Serialize)]
        struct JsonOutput<'a> {
            scan_time: chrono::DateTime<Utc>,
            scan_type: String,
            timing_template: String,
            timeout_ms: u64,
            total_results: usize,
            statistics: Statistics,
            results: &'a [ScanResult],
        }

        #[derive(Serialize)]
        struct Statistics {
            hosts_scanned: usize,
            ports_open: usize,
            ports_closed: usize,
            ports_filtered: usize,
            ports_unknown: usize,
        }

        // Calculate statistics
        let hosts: std::collections::HashSet<_> = results.iter().map(|r| r.target_ip).collect();

        let stats = Statistics {
            hosts_scanned: hosts.len(),
            ports_open: results
                .iter()
                .filter(|r| r.state == PortState::Open)
                .count(),
            ports_closed: results
                .iter()
                .filter(|r| r.state == PortState::Closed)
                .count(),
            ports_filtered: results
                .iter()
                .filter(|r| r.state == PortState::Filtered)
                .count(),
            ports_unknown: results
                .iter()
                .filter(|r| r.state == PortState::Unknown)
                .count(),
        };

        let output = JsonOutput {
            scan_time: Utc::now(),
            scan_type: format!("{:?}", config.scan.scan_type),
            timing_template: format!("{:?}", config.scan.timing_template),
            timeout_ms: config.scan.timeout_ms,
            total_results: results.len(),
            statistics: stats,
            results,
        };

        let json = serde_json::to_string_pretty(&output)?;
        Ok(json)
    }
}

/// XML output formatter (Nmap-compatible)
pub struct XmlFormatter;

impl OutputFormatter for XmlFormatter {
    fn format_results(&self, results: &[ScanResult], config: &Config) -> Result<String> {
        let mut output = String::new();

        // XML header
        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        output.push_str("<!DOCTYPE nmaprun>\n");
        output.push_str(&format!(
            "<nmaprun scanner=\"prtip\" version=\"{}\" start=\"{}\">\n",
            env!("CARGO_PKG_VERSION"),
            Utc::now().timestamp()
        ));

        // Scan info
        output.push_str(&format!(
            "  <scaninfo type=\"{:?}\" protocol=\"tcp\" timeout=\"{}\" />\n",
            config.scan.scan_type, config.scan.timeout_ms
        ));

        // Verbose output
        output.push_str(&format!(
            "  <verbose level=\"{}\" />\n",
            config.output.verbose
        ));

        // Group by host
        let mut by_host: BTreeMap<IpAddr, Vec<&ScanResult>> = BTreeMap::new();
        for result in results {
            by_host.entry(result.target_ip).or_default().push(result);
        }

        // Output hosts
        for (host, host_results) in &by_host {
            let start_time = host_results
                .iter()
                .map(|r| r.timestamp.timestamp())
                .min()
                .unwrap_or_else(|| Utc::now().timestamp());

            let end_time = host_results
                .iter()
                .map(|r| r.timestamp.timestamp())
                .max()
                .unwrap_or_else(|| Utc::now().timestamp());

            output.push_str(&format!(
                "  <host starttime=\"{}\" endtime=\"{}\">\n",
                start_time, end_time
            ));

            // Address
            let addr_type = match host {
                IpAddr::V4(_) => "ipv4",
                IpAddr::V6(_) => "ipv6",
            };
            output.push_str(&format!(
                "    <address addr=\"{}\" addrtype=\"{}\" />\n",
                host, addr_type
            ));

            // Status (assume up if we have results)
            output.push_str("    <status state=\"up\" reason=\"echo-reply\" />\n");

            // Ports
            output.push_str("    <ports>\n");

            for result in host_results {
                let protocol = match config.scan.scan_type {
                    prtip_core::ScanType::Udp => "udp",
                    _ => "tcp",
                };

                output.push_str(&format!(
                    "      <port protocol=\"{}\" portid=\"{}\">\n",
                    protocol, result.port
                ));

                let reason = match result.state {
                    PortState::Open => "syn-ack",
                    PortState::Closed => "reset",
                    PortState::Filtered => "no-response",
                    PortState::Unknown => "unknown",
                };

                output.push_str(&format!(
                    "        <state state=\"{}\" reason=\"{}\" />\n",
                    result.state, reason
                ));

                // Service info if available
                if let Some(service) = &result.service {
                    output.push_str(&format!("        <service name=\"{}\"", service));
                    if let Some(banner) = &result.banner {
                        // Escape XML special characters
                        let escaped_banner = banner
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;")
                            .replace('"', "&quot;")
                            .replace('\'', "&apos;");
                        output.push_str(&format!(" product=\"{}\"", escaped_banner));
                    }
                    output.push_str(" />\n");
                }

                output.push_str("      </port>\n");
            }

            output.push_str("    </ports>\n");

            // Timing information
            let total_time = (end_time - start_time) as f64;
            output.push_str(&format!(
                "    <times srtt=\"{}\" rttvar=\"{}\" to=\"{}\" />\n",
                (total_time * 1000.0) as u64,
                0,
                config.scan.timeout_ms
            ));

            output.push_str("  </host>\n");
        }

        // Run statistics
        let total_time = Utc::now().timestamp();
        output.push_str(&format!(
            "  <runstats>\n    <finished time=\"{}\" />\n",
            total_time
        ));
        output.push_str(&format!(
            "    <hosts up=\"{}\" down=\"0\" total=\"{}\" />\n",
            by_host.len(),
            by_host.len()
        ));
        output.push_str("  </runstats>\n");

        output.push_str("</nmaprun>\n");

        Ok(output)
    }
}

/// Greppable output formatter (Nmap -oG compatible)
pub struct GreppableFormatter;

impl OutputFormatter for GreppableFormatter {
    fn format_results(&self, results: &[ScanResult], config: &Config) -> Result<String> {
        let mut output = String::new();

        // Header comment
        output.push_str(&format!(
            "# Nmap-style greppable output (ProRT-IP v{})\n",
            env!("CARGO_PKG_VERSION")
        ));
        output.push_str(&format!(
            "# Started {} UTC\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));

        if results.is_empty() {
            output.push_str("# No results found\n");
            return Ok(output);
        }

        // Group by host
        let mut by_host: BTreeMap<IpAddr, Vec<&ScanResult>> = BTreeMap::new();
        for result in results {
            by_host.entry(result.target_ip).or_default().push(result);
        }

        // Output each host
        for (host, host_results) in &by_host {
            // Host line: Host: <ip> () Status: Up
            output.push_str(&format!("Host: {} ()\tStatus: Up\n", host));

            // Ports line: Ports: <port>/<state>/<proto>/<owner>/<service>/<rpc>/<version>
            // Simplified format: <port>/<state>/<proto>/<service>
            let ports_str: Vec<String> = host_results
                .iter()
                .map(|r| {
                    let protocol = match config.scan.scan_type {
                        prtip_core::ScanType::Udp => "udp",
                        _ => "tcp",
                    };

                    let service = r.service.as_deref().unwrap_or("");

                    format!("{}/{}/{}/{}", r.port, r.state, protocol, service)
                })
                .collect();

            if !ports_str.is_empty() {
                output.push_str(&format!("Ports: {}\n", ports_str.join(", ")));
            }
        }

        // Footer
        output.push_str(&format!(
            "# Nmap done at {} UTC -- {} IP address{} ({} host{} up) scanned\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S"),
            by_host.len(),
            if by_host.len() == 1 { "" } else { "es" },
            by_host.len(),
            if by_host.len() == 1 { "" } else { "s" }
        ));

        Ok(output)
    }
}

/// Create formatter based on format type
///
/// # Arguments
///
/// * `format` - Output format enum
/// * `colorize` - Whether to colorize text output (only applies to Text format)
pub fn create_formatter(
    format: prtip_core::OutputFormat,
    colorize: bool,
) -> Box<dyn OutputFormatter> {
    match format {
        prtip_core::OutputFormat::Text => Box::new(TextFormatter::new(colorize)),
        prtip_core::OutputFormat::Json => Box::new(JsonFormatter),
        prtip_core::OutputFormat::Xml => Box::new(XmlFormatter),
        prtip_core::OutputFormat::Greppable => Box::new(GreppableFormatter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prtip_core::{
        NetworkConfig, OutputConfig, OutputFormat, PerformanceConfig, PortState, ScanConfig,
        ScanResult, ScanType, TimingTemplate,
    };
    use std::net::IpAddr;
    use std::time::Duration;

    fn create_test_config() -> Config {
        Config {
            scan: ScanConfig {
                scan_type: ScanType::Connect,
                timing_template: TimingTemplate::Normal,
                timeout_ms: 3000,
                retries: 0,
                scan_delay_ms: 0,
                host_delay_ms: 0,
                service_detection: Default::default(),
                progress: false,
            },
            network: NetworkConfig {
                interface: None,
                source_port: None,
            },
            output: OutputConfig {
                format: OutputFormat::Text,
                file: None,
                verbose: 0,
            },
            performance: PerformanceConfig {
                max_rate: None,
                parallelism: 10,
                batch_size: None,
                requested_ulimit: None,
                numa_enabled: false,
            },
            evasion: prtip_core::EvasionConfig {
                fragment_packets: false,
                mtu: None,
                ttl: None,
                decoys: None,
                bad_checksums: false,
            },
        }
    }

    fn create_test_result(ip: &str, port: u16, state: PortState) -> ScanResult {
        ScanResult::new(ip.parse::<IpAddr>().unwrap(), port, state)
            .with_response_time(Duration::from_millis(100))
    }

    #[test]
    fn test_text_formatter_basic() {
        let results = vec![
            create_test_result("192.168.1.1", 80, PortState::Open),
            create_test_result("192.168.1.1", 443, PortState::Open),
        ];

        let formatter = TextFormatter::new(false);
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("192.168.1.1"));
        assert!(output.contains("80"));
        assert!(output.contains("443"));
        assert!(output.contains("open"));
        assert!(output.contains("Total Results: 2"));
    }

    #[test]
    fn test_text_formatter_with_service() {
        let mut result = create_test_result("192.168.1.1", 80, PortState::Open);
        result.service = Some("http".to_string());
        result.banner = Some("Apache/2.4.41".to_string());

        let results = vec![result];
        let formatter = TextFormatter::new(false);
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("http"));
        assert!(output.contains("Apache/2.4.41"));
    }

    #[test]
    fn test_text_formatter_empty() {
        let results = vec![];
        let formatter = TextFormatter::new(false);
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("No results found"));
    }

    #[test]
    fn test_text_formatter_multiple_states() {
        let results = vec![
            create_test_result("192.168.1.1", 80, PortState::Open),
            create_test_result("192.168.1.1", 81, PortState::Closed),
            create_test_result("192.168.1.1", 82, PortState::Filtered),
        ];

        let formatter = TextFormatter::new(false);
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("open"));
        assert!(output.contains("1 open, 1 closed, 1 filtered"));
    }

    #[test]
    fn test_text_formatter_multiple_hosts() {
        let results = vec![
            create_test_result("192.168.1.1", 80, PortState::Open),
            create_test_result("192.168.1.2", 80, PortState::Open),
        ];

        let formatter = TextFormatter::new(false);
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("192.168.1.1"));
        assert!(output.contains("192.168.1.2"));
        assert!(output.contains("Hosts Scanned: 2"));
    }

    #[test]
    fn test_json_formatter_basic() {
        let results = vec![create_test_result("192.168.1.1", 80, PortState::Open)];

        let formatter = JsonFormatter;
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("\"total_results\": 1"));
        assert!(output.contains("\"target_ip\""));
        assert!(output.contains("\"port\": 80"));

        // Verify valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["total_results"], 1);
    }

    #[test]
    fn test_json_formatter_statistics() {
        let results = vec![
            create_test_result("192.168.1.1", 80, PortState::Open),
            create_test_result("192.168.1.1", 81, PortState::Closed),
            create_test_result("192.168.1.1", 82, PortState::Filtered),
        ];

        let formatter = JsonFormatter;
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["statistics"]["ports_open"], 1);
        assert_eq!(parsed["statistics"]["ports_closed"], 1);
        assert_eq!(parsed["statistics"]["ports_filtered"], 1);
        assert_eq!(parsed["statistics"]["hosts_scanned"], 1);
    }

    #[test]
    fn test_xml_formatter_basic() {
        let results = vec![create_test_result("192.168.1.1", 80, PortState::Open)];

        let formatter = XmlFormatter;
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("<?xml version"));
        assert!(output.contains("<nmaprun"));
        assert!(output.contains("192.168.1.1"));
        assert!(output.contains("portid=\"80\""));
        assert!(output.contains("state=\"open\""));
        assert!(output.contains("</nmaprun>"));
    }

    #[test]
    fn test_xml_formatter_with_service() {
        let mut result = create_test_result("192.168.1.1", 80, PortState::Open);
        result.service = Some("http".to_string());
        result.banner = Some("Apache/2.4.41".to_string());

        let results = vec![result];
        let formatter = XmlFormatter;
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("<service name=\"http\""));
        assert!(output.contains("Apache/2.4.41"));
    }

    #[test]
    fn test_xml_formatter_escape_special_chars() {
        let mut result = create_test_result("192.168.1.1", 80, PortState::Open);
        result.service = Some("http".to_string());
        result.banner = Some("<test>&\"foo\"</test>".to_string());

        let results = vec![result];
        let formatter = XmlFormatter;
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("&lt;"));
        assert!(output.contains("&gt;"));
        assert!(output.contains("&amp;"));
        assert!(output.contains("&quot;"));
    }

    #[test]
    fn test_xml_formatter_ipv6() {
        let results = vec![ScanResult::new(
            "::1".parse::<IpAddr>().unwrap(),
            80,
            PortState::Open,
        )];

        let formatter = XmlFormatter;
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("addrtype=\"ipv6\""));
        assert!(output.contains("::1"));
    }

    #[test]
    fn test_create_formatter_text() {
        let formatter = create_formatter(OutputFormat::Text, false);
        let results = vec![];
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();
        assert!(output.contains("ProRT-IP"));
    }

    #[test]
    fn test_create_formatter_json() {
        let formatter = create_formatter(OutputFormat::Json, false);
        let results = vec![];
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&output).is_ok());
    }

    #[test]
    fn test_create_formatter_xml() {
        let formatter = create_formatter(OutputFormat::Xml, false);
        let results = vec![];
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();
        assert!(output.contains("<?xml"));
    }

    #[test]
    fn test_text_formatter_colorize() {
        let results = vec![create_test_result("192.168.1.1", 80, PortState::Open)];

        let formatter = TextFormatter::new(true);
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        // With colorize=true, formatting methods will attempt to colorize
        // Actual ANSI codes depend on terminal detection by `colored` crate
        // So we just verify the output contains the IP and state
        assert!(output.contains("192.168.1.1"));
        assert!(output.contains("open"));
    }

    #[test]
    fn test_text_formatter_verbose_levels() {
        let results = vec![
            create_test_result("192.168.1.1", 80, PortState::Open),
            create_test_result("192.168.1.1", 81, PortState::Filtered),
            create_test_result("192.168.1.1", 82, PortState::Closed),
        ];

        let formatter = TextFormatter::new(false);

        // Verbose 0: no filtered/closed
        let mut config = create_test_config();
        config.output.verbose = 0;
        let output = formatter.format_results(&results, &config).unwrap();
        assert!(!output.contains("Filtered Ports:"));
        assert!(!output.contains("Closed Ports:"));

        // Verbose 1: show filtered
        config.output.verbose = 1;
        let output = formatter.format_results(&results, &config).unwrap();
        assert!(output.contains("Filtered Ports:"));
        assert!(!output.contains("Closed Ports:"));

        // Verbose 2: show filtered and closed
        config.output.verbose = 2;
        let output = formatter.format_results(&results, &config).unwrap();
        assert!(output.contains("Filtered Ports:"));
        assert!(output.contains("Closed Ports:"));
    }

    #[test]
    fn test_text_formatter_long_banner() {
        let mut result = create_test_result("192.168.1.1", 80, PortState::Open);
        result.banner = Some("A".repeat(100));

        let results = vec![result];
        let formatter = TextFormatter::new(false);
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        // Should be truncated
        assert!(output.contains("..."));
    }

    #[test]
    fn test_greppable_formatter_basic() {
        let results = vec![create_test_result("192.168.1.1", 80, PortState::Open)];

        let formatter = GreppableFormatter;
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("# Nmap-style greppable output"));
        assert!(output.contains("Host: 192.168.1.1"));
        assert!(output.contains("Status: Up"));
        assert!(output.contains("Ports:"));
        assert!(output.contains("80/open/tcp"));
    }

    #[test]
    fn test_greppable_formatter_with_service() {
        let mut result = create_test_result("192.168.1.1", 80, PortState::Open);
        result.service = Some("http".to_string());

        let results = vec![result];
        let formatter = GreppableFormatter;
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("80/open/tcp/http"));
    }

    #[test]
    fn test_greppable_formatter_multiple_ports() {
        let results = vec![
            create_test_result("192.168.1.1", 80, PortState::Open),
            create_test_result("192.168.1.1", 443, PortState::Open),
            create_test_result("192.168.1.1", 22, PortState::Closed),
        ];

        let formatter = GreppableFormatter;
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("80/open/tcp"));
        assert!(output.contains("443/open/tcp"));
        assert!(output.contains("22/closed/tcp"));
    }

    #[test]
    fn test_greppable_formatter_empty() {
        let results = vec![];
        let formatter = GreppableFormatter;
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();

        assert!(output.contains("# No results found"));
    }

    #[test]
    fn test_create_formatter_greppable() {
        let formatter = create_formatter(OutputFormat::Greppable, false);
        let results = vec![];
        let config = create_test_config();
        let output = formatter.format_results(&results, &config).unwrap();
        assert!(output.contains("# Nmap-style"));
    }
}
