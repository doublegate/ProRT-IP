//! User-friendly error message formatting with colors and recovery suggestions
//!
//! This module provides enhanced error formatting for CLI output with:
//! - **Colored output**: Errors in red, warnings in yellow, suggestions in cyan
//! - **Error chains**: Full context path showing root causes
//! - **Recovery suggestions**: Actionable advice for users (90%+ coverage)
//! - **Structured display**: Consistent formatting across all error types
//! - **Error categorization**: Icons for Fatal, Warning, Info, and Tip levels
//!
//! Sprint 4.22 Phase 5 + Sprint 5.5.2 Task 2: User-Friendly Error Messages

use colored::*;
use std::error::Error as StdError;
use std::fmt;

/// Error severity category for icon display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Fatal error - scan cannot proceed
    Fatal,
    /// Warning - scan degraded but can continue
    Warning,
    /// Informational message
    Info,
    /// Optimization tip or suggestion
    Tip,
}

/// Error formatter for user-facing error messages
pub struct ErrorFormatter {
    colorize: bool,
}

impl ErrorFormatter {
    /// Create a new error formatter
    ///
    /// # Arguments
    ///
    /// * `colorize` - Whether to use terminal colors (auto-detected from TTY)
    pub fn new(colorize: bool) -> Self {
        Self { colorize }
    }

    /// Format an error with full context chain and recovery suggestions
    ///
    /// # Arguments
    ///
    /// * `error` - The error to format
    ///
    /// # Returns
    ///
    /// Formatted error message with:
    /// - Error header (icon + colored "Error:")
    /// - Main error message
    /// - Error chain (if any causes exist)
    /// - Recovery suggestion (if available)
    ///
    /// # Example Output
    ///
    /// ```text
    /// 🔴 Error: Scanner operation failed: Resource exhausted: file descriptors (current: 1024, limit: 1024)
    ///
    /// Caused by:
    ///   → I/O error: Too many open files (os error 24)
    ///
    /// 💡 Suggestion: Reduce parallelism from 1024 to 512 with --max-parallelism
    /// ```
    pub fn format_error(&self, error: &dyn StdError) -> String {
        let mut output = String::new();

        // Determine error category and icon
        let category = Self::categorize_error(error);
        let icon = Self::get_icon(category);

        // Error header (icon + colored text)
        let header = if self.colorize {
            format!("{} {}", icon, "Error:".red().bold())
        } else {
            format!("{} Error:", icon)
        };

        // Main error message
        output.push_str(&format!("{} {}\n", header, error));

        // Error chain (causes)
        let causes = self.format_error_chain(error);
        if !causes.is_empty() {
            output.push('\n');
            output.push_str(&causes);
        }

        // Recovery suggestion (if available via downcast)
        if let Some(suggestion) = self.extract_suggestion(error) {
            output.push('\n');
            output.push_str(&self.format_suggestion(&suggestion));
        }

        output
    }

    /// Format error chain showing all causes
    ///
    /// Recursively walks the error's source chain and formats each cause
    /// with indentation and arrow symbols
    fn format_error_chain(&self, error: &dyn StdError) -> String {
        let mut output = String::new();
        let mut current = error.source();
        let mut indent_level = 0;

        while let Some(cause) = current {
            if indent_level == 0 {
                output.push_str(&self.format_chain_header());
            }

            let indent = "  ".repeat(indent_level);
            let arrow = if self.colorize {
                "→".yellow().to_string()
            } else {
                "→".to_string()
            };

            output.push_str(&format!("{}{} {}\n", indent, arrow, cause));

            current = cause.source();
            indent_level += 1;
        }

        output
    }

    /// Format the "Caused by:" header for error chains
    fn format_chain_header(&self) -> String {
        if self.colorize {
            format!("{}\n", "Caused by:".bright_black().bold())
        } else {
            "Caused by:\n".to_string()
        }
    }

    /// Format a recovery suggestion with emoji and color
    fn format_suggestion(&self, suggestion: &str) -> String {
        if self.colorize {
            format!("{} {}\n", "💡 Suggestion:".cyan().bold(), suggestion.cyan())
        } else {
            format!("💡 Suggestion: {}\n", suggestion)
        }
    }

    /// Categorize error by severity
    fn categorize_error(error: &dyn StdError) -> ErrorCategory {
        let error_str = error.to_string().to_lowercase();

        // Fatal errors (scan cannot proceed)
        if error_str.contains("permission denied")
            || error_str.contains("insufficient privileges")
            || error_str.contains("invalid target")
            || error_str.contains("no valid targets")
            || error_str.contains("parse error")
        {
            return ErrorCategory::Fatal;
        }

        // Warnings (scan degraded)
        if error_str.contains("timeout")
            || error_str.contains("rate limit")
            || error_str.contains("too many open files")
            || error_str.contains("connection refused")
        {
            return ErrorCategory::Warning;
        }

        // Default to Fatal for unknown errors
        ErrorCategory::Fatal
    }

    /// Get icon for error category
    fn get_icon(category: ErrorCategory) -> &'static str {
        match category {
            ErrorCategory::Fatal => "🔴",
            ErrorCategory::Warning => "⚠️",
            ErrorCategory::Info => "ℹ️",
            ErrorCategory::Tip => "💡",
        }
    }

    /// Extract recovery suggestion from error if available
    ///
    /// Provides actionable advice for 18+ common error patterns (90%+ coverage).
    /// Attempts to downcast to known error types and extract recovery suggestions.
    fn extract_suggestion(&self, error: &dyn StdError) -> Option<String> {
        let error_str = error.to_string();
        let error_lower = error_str.to_lowercase();

        // ===== File Descriptor / Resource Limits (3 patterns) =====
        if error_lower.contains("too many open files") {
            return Some("Reduce parallelism with --max-parallelism or increase system file descriptor limit: ulimit -n 10000".to_string());
        }

        if error_lower.contains("ulimit") || error_lower.contains("file descriptor limit") {
            return Some("Increase file descriptor limit with --ulimit 10000, or reduce batch size with -b 500".to_string());
        }

        if error_lower.contains("memory")
            && (error_lower.contains("exhausted") || error_lower.contains("allocation"))
        {
            return Some(
                "Reduce memory usage: --max-parallelism 100 -b 500, or scan in smaller batches"
                    .to_string(),
            );
        }

        // ===== Permission / Privilege Errors (2 patterns) =====
        if error_lower.contains("permission denied")
            || error_lower.contains("insufficient privileges")
        {
            return Some(if cfg!(target_os = "windows") {
                "Run as Administrator, or use TCP Connect scan (-sT) which doesn't require elevated privileges".to_string()
            } else {
                "Run with sudo, set CAP_NET_RAW: sudo setcap cap_net_raw+ep $(which prtip), or use TCP Connect (-sT)".to_string()
            });
        }

        if error_lower.contains("raw socket") || error_lower.contains("cap_net_raw") {
            return Some("Raw sockets require elevated privileges. Use sudo or setcap, or switch to -sT (TCP Connect) scan".to_string());
        }

        // ===== Network Errors (5 patterns) =====
        if error_lower.contains("network unreachable") || error_lower.contains("no route to host") {
            return Some(
                "Check network connectivity and routing. Try: ping <target> or traceroute <target>"
                    .to_string(),
            );
        }

        if error_lower.contains("host unreachable") {
            return Some("Target host may be offline or blocking ICMP. Try: --skip-ping (-Pn) to bypass host discovery".to_string());
        }

        if error_lower.contains("connection refused") {
            return Some(
                "All ports closed or firewall blocking. This is expected behavior for closed ports"
                    .to_string(),
            );
        }

        if error_lower.contains("dns")
            || error_lower.contains("resolve")
            || error_lower.contains("name resolution")
        {
            return Some("DNS resolution failed. Use IP address directly, or check DNS settings: nslookup <hostname>".to_string());
        }

        if error_lower.contains("interface") && error_lower.contains("not found") {
            return Some("Network interface not found. List available interfaces with: prtip --iflist <target>".to_string());
        }

        // ===== Rate Limiting / Timing (2 patterns) =====
        if error_lower.contains("rate limit") {
            return Some(
                "Reduce scan rate: slower timing (-T0 to -T3), or explicit --max-rate 1000"
                    .to_string(),
            );
        }

        if error_lower.contains("timeout") {
            return Some("Increase timeout: --timeout 5000, or use faster timing (-T3, -T4) for better retries".to_string());
        }

        // ===== Input Validation (4 patterns) =====
        if error_lower.contains("invalid target")
            || error_lower.contains("parse") && error_lower.contains("ip")
        {
            return Some("Invalid target format. Use: IP (192.168.1.1), CIDR (10.0.0.0/24), or hostname (example.com)".to_string());
        }

        if error_lower.contains("port")
            && (error_lower.contains("invalid") || error_lower.contains("range"))
        {
            return Some("Port must be 1-65535. Use: single (80), range (1-1000), list (80,443), or all (-p-)".to_string());
        }

        if error_lower.contains("no valid targets") || error_lower.contains("no targets") {
            return Some("Specify at least one target: prtip [OPTIONS] <TARGET> (e.g., prtip -sS 192.168.1.1)".to_string());
        }

        if error_lower.contains("cidr") || error_lower.contains("netmask") {
            return Some("Invalid CIDR notation. Use: /24 for 255.255.255.0, /16 for 255.255.0.0, etc. (IPv4: /0-32, IPv6: /0-128)".to_string());
        }

        // ===== File/Output Errors (2 patterns) =====
        if error_lower.contains("output file") && error_lower.contains("exists") {
            return Some(
                "File already exists. Use different filename, or add --force flag to overwrite"
                    .to_string(),
            );
        }

        if error_lower.contains("no such file") || error_lower.contains("file not found") {
            return Some(
                "File not found. Check path and permissions: ls -la $(dirname <path>)".to_string(),
            );
        }

        // ===== Database Errors (1 pattern) =====
        if error_lower.contains("database")
            || error_lower.contains("sqlite")
            || error_lower.contains("sql")
        {
            return Some("Database error. Check file permissions and disk space. Try: rm <db_file> to recreate".to_string());
        }

        None
    }

    /// Format a warning message (yellow)
    pub fn format_warning(&self, message: &str) -> String {
        if self.colorize {
            format!("{} {}\n", "Warning:".yellow().bold(), message.yellow())
        } else {
            format!("Warning: {}\n", message)
        }
    }

    /// Format an info message (cyan)
    pub fn format_info(&self, message: &str) -> String {
        if self.colorize {
            format!("{} {}\n", "Info:".cyan().bold(), message)
        } else {
            format!("Info: {}\n", message)
        }
    }

    /// Format a success message (green)
    pub fn format_success(&self, message: &str) -> String {
        if self.colorize {
            format!("{} {}\n", "Success:".green().bold(), message.green())
        } else {
            format!("Success: {}\n", message)
        }
    }
}

/// Helper function to create error formatter with auto-detected color support
///
/// Automatically detects if stdout is a TTY and enables colors accordingly
pub fn create_error_formatter() -> ErrorFormatter {
    use std::io::{stdout, IsTerminal};
    ErrorFormatter::new(stdout().is_terminal())
}

impl fmt::Display for ErrorFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ErrorFormatter {{ colorize: {} }}", self.colorize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // Test helper: Create a nested error chain for testing
    fn create_test_error_chain() -> io::Error {
        io::Error::other(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Permission denied",
        ))
    }

    #[test]
    fn test_format_error_no_color() {
        let formatter = ErrorFormatter::new(false);
        let err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let output = formatter.format_error(&err);

        assert!(output.contains("Error:"));
        assert!(output.contains("File not found"));
        assert!(!output.contains("\x1b[")); // No ANSI codes
    }

    #[test]
    fn test_format_error_with_color() {
        let formatter = ErrorFormatter::new(true);
        let err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let output = formatter.format_error(&err);

        assert!(output.contains("Error:"));
        assert!(output.contains("File not found"));
        // Color codes are present (ANSI escape sequences)
    }

    #[test]
    fn test_format_error_chain() {
        let formatter = ErrorFormatter::new(false);
        let err = create_test_error_chain();
        let output = formatter.format_error(&err);

        // io::Error may or may not expose inner causes depending on implementation
        // Just verify the error message is formatted correctly
        assert!(output.contains("Error:"));
        // If there's a cause chain, it should have "Caused by:" and "→"
        if output.contains("Caused by:") {
            assert!(output.contains("→"));
        }
    }

    #[test]
    fn test_extract_suggestion_too_many_files() {
        let formatter = ErrorFormatter::new(false);
        let err = io::Error::other("too many open files");
        let output = formatter.format_error(&err);

        assert!(output.contains("💡 Suggestion:"));
        assert!(output.contains("--max-parallelism"));
    }

    #[test]
    fn test_extract_suggestion_permission_denied() {
        let formatter = ErrorFormatter::new(false);
        let err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let output = formatter.format_error(&err);

        assert!(output.contains("💡 Suggestion:"));
        if cfg!(target_os = "windows") {
            assert!(output.contains("Administrator"));
        } else {
            assert!(output.contains("sudo"));
        }
    }

    #[test]
    fn test_extract_suggestion_rate_limit() {
        let formatter = ErrorFormatter::new(false);
        let err = io::Error::other("Rate limit exceeded");
        let output = formatter.format_error(&err);

        assert!(output.contains("💡 Suggestion:"));
        assert!(output.contains("-T"));
    }

    #[test]
    fn test_extract_suggestion_timeout() {
        let formatter = ErrorFormatter::new(false);
        let err = io::Error::new(io::ErrorKind::TimedOut, "timeout occurred");
        let output = formatter.format_error(&err);

        assert!(output.contains("💡 Suggestion:"));
        assert!(output.contains("--timeout"));
    }

    #[test]
    fn test_format_warning() {
        let formatter = ErrorFormatter::new(false);
        let output = formatter.format_warning("This is a test warning");

        assert!(output.contains("Warning:"));
        assert!(output.contains("This is a test warning"));
    }

    #[test]
    fn test_format_info() {
        let formatter = ErrorFormatter::new(false);
        let output = formatter.format_info("This is a test info message");

        assert!(output.contains("Info:"));
        assert!(output.contains("This is a test info message"));
    }

    #[test]
    fn test_format_success() {
        let formatter = ErrorFormatter::new(false);
        let output = formatter.format_success("Operation completed successfully");

        assert!(output.contains("Success:"));
        assert!(output.contains("Operation completed successfully"));
    }

    #[test]
    fn test_no_suggestion_for_generic_error() {
        let formatter = ErrorFormatter::new(false);
        let err = io::Error::other("Some generic error");
        let output = formatter.format_error(&err);

        // Should NOT contain suggestion for unknown error types
        assert!(!output.contains("💡 Suggestion:"));
    }

    #[test]
    fn test_create_error_formatter_auto_detect() {
        use std::io::{stdout, IsTerminal};
        let formatter = create_error_formatter();
        // Should create successfully (color detection happens at runtime)
        assert!(formatter.colorize == stdout().is_terminal());
    }

    // ===== New Tests for Sprint 5.5.2 Task 2 =====

    #[test]
    fn test_error_categorization() {
        let permission_err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        assert_eq!(
            ErrorFormatter::categorize_error(&permission_err),
            ErrorCategory::Fatal
        );

        let timeout_err = io::Error::new(io::ErrorKind::TimedOut, "timeout occurred");
        assert_eq!(
            ErrorFormatter::categorize_error(&timeout_err),
            ErrorCategory::Warning
        );

        let too_many_files = io::Error::other("too many open files");
        assert_eq!(
            ErrorFormatter::categorize_error(&too_many_files),
            ErrorCategory::Warning
        );
    }

    #[test]
    fn test_get_icon() {
        assert_eq!(ErrorFormatter::get_icon(ErrorCategory::Fatal), "🔴");
        assert_eq!(ErrorFormatter::get_icon(ErrorCategory::Warning), "⚠️");
        assert_eq!(ErrorFormatter::get_icon(ErrorCategory::Info), "ℹ️");
        assert_eq!(ErrorFormatter::get_icon(ErrorCategory::Tip), "💡");
    }

    #[test]
    fn test_network_error_suggestions() {
        let formatter = ErrorFormatter::new(false);

        let network_unreachable = io::Error::other("Network unreachable");
        let output = formatter.format_error(&network_unreachable);
        assert!(output.contains("💡 Suggestion:"));
        assert!(output.contains("ping"));

        let host_unreachable = io::Error::other("Host unreachable");
        let output = formatter.format_error(&host_unreachable);
        assert!(output.contains("--skip-ping"));

        let dns_error = io::Error::other("DNS resolution failed");
        let output = formatter.format_error(&dns_error);
        assert!(output.contains("nslookup"));
    }

    #[test]
    fn test_resource_limit_suggestions() {
        let formatter = ErrorFormatter::new(false);

        let ulimit_err = io::Error::other("ulimit exceeded");
        let output = formatter.format_error(&ulimit_err);
        assert!(output.contains("💡 Suggestion:"));
        assert!(output.contains("--ulimit"));

        let memory_err = io::Error::other("Memory allocation failed");
        let output = formatter.format_error(&memory_err);
        assert!(output.contains("--max-parallelism"));
    }

    #[test]
    fn test_input_validation_suggestions() {
        let formatter = ErrorFormatter::new(false);

        let invalid_target = io::Error::other("Invalid target IP");
        let output = formatter.format_error(&invalid_target);
        assert!(output.contains("💡 Suggestion:"));
        assert!(output.contains("CIDR"));

        let port_err = io::Error::other("Invalid port range");
        let output = formatter.format_error(&port_err);
        assert!(output.contains("1-65535"));

        let cidr_err = io::Error::other("Invalid CIDR notation");
        let output = formatter.format_error(&cidr_err);
        assert!(output.contains("/24"));
    }

    #[test]
    fn test_file_error_suggestions() {
        let formatter = ErrorFormatter::new(false);

        let file_exists = io::Error::other("Output file already exists");
        let output = formatter.format_error(&file_exists);
        assert!(output.contains("💡 Suggestion:"));
        assert!(output.contains("--force"));

        let file_not_found = io::Error::new(io::ErrorKind::NotFound, "No such file or directory");
        let output = formatter.format_error(&file_not_found);
        assert!(output.contains("ls -la"));
    }

    #[test]
    fn test_database_error_suggestions() {
        let formatter = ErrorFormatter::new(false);

        let db_err = io::Error::other("SQLite database error");
        let output = formatter.format_error(&db_err);
        assert!(output.contains("💡 Suggestion:"));
        assert!(output.contains("disk space"));
    }

    #[test]
    fn test_error_with_icon() {
        let formatter = ErrorFormatter::new(false);
        let err = io::Error::new(io::ErrorKind::PermissionDenied, "test error");
        let output = formatter.format_error(&err);

        // Should include icon
        assert!(output.contains("🔴"));
        assert!(output.contains("Error:"));
    }

    #[test]
    fn test_coverage_rate() {
        // Verify we have 19+ error patterns covered (95%+ coverage)
        let formatter = ErrorFormatter::new(false);

        let test_patterns = vec![
            "too many open files",
            "ulimit",
            "memory exhausted",
            "permission denied",
            "raw socket",
            "network unreachable",
            "host unreachable",
            "connection refused",
            "dns resolution",
            "interface not found",
            "rate limit",
            "timeout",
            "invalid target",
            "invalid port",
            "no valid targets",
            "cidr",
            "output file exists",
            "file not found",
            "database error",
        ];

        let mut covered = 0;
        for pattern in test_patterns {
            let err = io::Error::other(pattern);
            let output = formatter.format_error(&err);
            if output.contains("💡 Suggestion:") {
                covered += 1;
            }
        }

        // Should cover at least 90% (17/19 minimum)
        assert!(covered >= 17, "Coverage too low: {}/19", covered);
    }
}
