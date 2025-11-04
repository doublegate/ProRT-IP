//! Export Utilities for Scan Results
//!
//! Provides functions to export scan results to various formats:
//! - JSON: Machine-readable, preserves all data
//! - CSV: Spreadsheet-compatible, tabular format
//! - XML: Nmap-compatible output
//! - Text: Human-readable summary

use prtip_core::{PortState, ScanResult};
use std::collections::HashMap;

/// Export scan results to JSON format
///
/// Returns a pretty-printed JSON string with all scan result fields.
pub fn export_json(results: &[ScanResult]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(results)
}

/// Export scan results to CSV format
///
/// Returns a CSV string with headers and one row per scan result.
pub fn export_csv(results: &[ScanResult]) -> Result<String, Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_writer(vec![]);

    // Write header
    wtr.write_record([
        "Target IP",
        "Port",
        "State",
        "Service",
        "Version",
        "Banner",
        "Response Time (ms)",
        "Timestamp",
    ])?;

    // Write results
    for result in results {
        let service_name = result.service.as_deref().unwrap_or("");
        let service_version = result.version.as_deref().unwrap_or("");

        let banner = result
            .banner
            .as_ref()
            .map(|b| {
                // Truncate long banners and escape newlines
                let truncated = if b.len() > 100 {
                    format!("{}...", &b[..100])
                } else {
                    b.to_string()
                };
                truncated.replace('\n', "\\n").replace('\r', "\\r")
            })
            .unwrap_or_default();

        let response_time_ms = result.response_time.as_millis().to_string();
        let timestamp = result.timestamp.to_rfc3339();

        wtr.write_record([
            &result.target_ip.to_string(),
            &result.port.to_string(),
            &format!("{:?}", result.state),
            service_name,
            service_version,
            &banner,
            &response_time_ms,
            &timestamp,
        ])?;
    }

    let bytes = wtr.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

/// Export scan results to XML format (Nmap-compatible)
///
/// Returns an XML string in Nmap output format for compatibility
/// with tools that parse Nmap XML.
pub fn export_xml(results: &[ScanResult]) -> Result<String, Box<dyn std::error::Error>> {
    let mut xml = String::new();

    // XML declaration
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<!DOCTYPE nmaprun>\n");
    xml.push_str("<?xml-stylesheet href=\"file:///usr/share/nmap/nmap.xsl\" type=\"text/xsl\"?>\n");
    xml.push_str(&format!("<nmaprun scanner=\"prtip\" version=\"{}\" xmloutputversion=\"1.05\">\n", env!("CARGO_PKG_VERSION")));

    // Group results by host
    let mut hosts: HashMap<String, Vec<&ScanResult>> = HashMap::new();
    for result in results {
        hosts
            .entry(result.target_ip.to_string())
            .or_default()
            .push(result);
    }

    // Write each host
    for (ip, ports) in hosts.iter() {
        xml.push_str("  <host>\n");
        xml.push_str(&format!(
            "    <address addr=\"{}\" addrtype=\"ipv4\"/>\n",
            ip
        ));

        xml.push_str("    <ports>\n");

        for port in ports {
            // Assume TCP protocol (can be extended later)
            let protocol = "tcp";
            let portid = port.port;

            xml.push_str(&format!(
                "      <port protocol=\"{}\" portid=\"{}\">\n",
                protocol, portid
            ));

            // Port state
            let state = match port.state {
                PortState::Open => "open",
                PortState::Closed => "closed",
                PortState::Filtered => "filtered",
                PortState::Unknown => "unknown",
            };
            xml.push_str(&format!("        <state state=\"{}\"/>\n", state));

            // Service info
            if let Some(service_name) = &port.service {
                xml.push_str("        <service");
                xml.push_str(&format!(" name=\"{}\"", escape_xml(service_name)));

                if let Some(version) = &port.version {
                    xml.push_str(&format!(" version=\"{}\"", escape_xml(version)));
                }

                if let Some(banner) = &port.banner {
                    let truncated = if banner.len() > 200 {
                        format!("{}...", &banner[..200])
                    } else {
                        banner.clone()
                    };
                    xml.push_str(&format!(" extrainfo=\"{}\"", escape_xml(&truncated)));
                }

                xml.push_str("/>\n");
            }

            xml.push_str("      </port>\n");
        }

        xml.push_str("    </ports>\n");
        xml.push_str("  </host>\n");
    }

    xml.push_str("</nmaprun>\n");

    Ok(xml)
}

/// Export scan results to human-readable text format
///
/// Returns a formatted text string with scan results organized by host.
pub fn export_text(results: &[ScanResult]) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = String::new();

    // Header
    output.push_str("Scan Results Summary\n");
    output.push_str("====================\n\n");

    // Group by host
    let mut hosts: HashMap<String, Vec<&ScanResult>> = HashMap::new();
    for result in results {
        hosts
            .entry(result.target_ip.to_string())
            .or_default()
            .push(result);
    }

    // Write each host
    for (ip, ports) in hosts.iter() {
        output.push_str(&format!("Host: {}\n", ip));
        output.push_str(&format!("{}\n", "-".repeat(ip.len() + 6)));

        // Table header
        output.push_str(&format!(
            "{:<10} {:<12} {:<16} {}\n",
            "PORT", "STATE", "SERVICE", "VERSION"
        ));
        output.push_str(&format!("{}\n", "-".repeat(60)));

        // Sort ports numerically
        let mut sorted_ports = ports.clone();
        sorted_ports.sort_by_key(|r| r.port);

        // Write ports
        for port in sorted_ports {
            let port_proto = format!("{}/tcp", port.port);
            let state = format!("{:?}", port.state).to_lowercase();

            let service_name = port.service.as_deref().unwrap_or("unknown");
            let service_version = port.version.as_deref().unwrap_or("");

            output.push_str(&format!(
                "{:<10} {:<12} {:<16} {}\n",
                port_proto, state, service_name, service_version
            ));
        }

        output.push('\n');
    }

    // Summary statistics
    output.push_str("Summary\n");
    output.push_str("=======\n");
    output.push_str(&format!("Total hosts: {}\n", hosts.len()));
    output.push_str(&format!("Total ports: {}\n", results.len()));

    let open_count = results
        .iter()
        .filter(|r| r.state == PortState::Open)
        .count();
    output.push_str(&format!("Open ports: {}\n", open_count));

    Ok(output)
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;

    fn test_scan_result() -> ScanResult {
        ScanResult {
            target_ip: "192.168.1.1".parse().unwrap(),
            port: 80,
            state: PortState::Open,
            response_time: Duration::from_millis(10),
            timestamp: Utc::now(),
            banner: Some("HTTP/1.1 200 OK".to_string()),
            service: Some("http".to_string()),
            version: Some("nginx 1.18.0".to_string()),
            raw_response: None,
        }
    }

    #[test]
    fn test_export_json() {
        let results = vec![test_scan_result()];
        let json = export_json(&results).unwrap();

        assert!(json.contains("\"port\": 80"));
        assert!(json.contains("\"target_ip\""));
        assert!(json.contains("http"));
    }

    #[test]
    fn test_export_csv() {
        let results = vec![test_scan_result()];
        let csv = export_csv(&results).unwrap();

        assert!(csv.contains("Target IP,Port,State"));
        assert!(csv.contains("192.168.1.1,80,Open"));
        assert!(csv.contains("http"));
    }

    #[test]
    fn test_export_xml() {
        let results = vec![test_scan_result()];
        let xml = export_xml(&results).unwrap();

        assert!(xml.contains("<?xml version"));
        assert!(xml.contains("<nmaprun"));
        assert!(xml.contains("192.168.1.1"));
        assert!(xml.contains("portid=\"80\""));
        assert!(xml.contains("name=\"http\""));
    }

    #[test]
    fn test_export_text() {
        let results = vec![test_scan_result()];
        let text = export_text(&results).unwrap();

        assert!(text.contains("Scan Results Summary"));
        assert!(text.contains("Host: 192.168.1.1"));
        assert!(text.contains("80/tcp"));
        assert!(text.contains("open"));
        assert!(text.contains("http"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("test"), "test");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn test_export_multiple_hosts() {
        let mut results = vec![];
        for i in 1..=3 {
            results.push(ScanResult {
                target_ip: format!("192.168.1.{}", i).parse().unwrap(),
                port: 80,
                state: PortState::Open,
                response_time: Duration::from_millis(10),
                timestamp: Utc::now(),
                banner: None,
                service: None,
                version: None,
                raw_response: None,
            });
        }

        let text = export_text(&results).unwrap();
        assert!(text.contains("Total hosts: 3"));
        assert!(text.contains("Total ports: 3"));
    }
}
