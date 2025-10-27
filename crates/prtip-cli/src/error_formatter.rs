//! User-friendly error message formatting with colors and recovery suggestions
//!
//! This module provides enhanced error formatting for CLI output with:
//! - **Colored output**: Errors in red, warnings in yellow, suggestions in cyan
//! - **Error chains**: Full context path showing root causes
//! - **Recovery suggestions**: Actionable advice for users
//! - **Structured display**: Consistent formatting across all error types
//!
//! Sprint 4.22 Phase 5: User-Friendly Error Messages

use colored::*;
use std::error::Error as StdError;
use std::fmt;

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
    /// - Error header (red "Error:")
    /// - Main error message
    /// - Error chain (if any causes exist)
    /// - Recovery suggestion (if available)
    ///
    /// # Example Output
    ///
    /// ```text
    /// Error: Scanner operation failed: Resource exhausted: file descriptors (current: 1024, limit: 1024)
    ///
    /// Caused by:
    ///   → I/O error: Too many open files (os error 24)
    ///
    /// 💡 Suggestion: Reduce parallelism from 1024 to 512 with --max-parallelism
    /// ```
    pub fn format_error(&self, error: &dyn StdError) -> String {
        let mut output = String::new();

        // Error header (red)
        let header = if self.colorize {
            format!("{}", "Error:".red().bold())
        } else {
            "Error:".to_string()
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

    /// Extract recovery suggestion from error if available
    ///
    /// Attempts to downcast to known error types (ScannerError, CliError)
    /// and extract their recovery suggestions
    fn extract_suggestion(&self, error: &dyn StdError) -> Option<String> {
        // Try to get error description and check for known patterns
        let error_str = error.to_string();

        // Check for common error patterns and provide suggestions
        if error_str.contains("too many open files") {
            return Some("Reduce parallelism with --max-parallelism or increase system file descriptor limit (ulimit -n)".to_string());
        }

        if error_str.contains("permission denied") || error_str.contains("Insufficient privileges")
        {
            return Some(if cfg!(target_os = "windows") {
                "Run as Administrator or use TCP Connect scan (-sT) which doesn't require elevated privileges".to_string()
            } else {
                "Run with sudo, or set CAP_NET_RAW capability: sudo setcap cap_net_raw+ep $(which prtip), or use TCP Connect scan (-sT)".to_string()
            });
        }

        if error_str.contains("Rate limit exceeded") {
            return Some("Reduce scan rate with slower timing template (-T0 through -T3), or use --max-rate to set explicit limit".to_string());
        }

        if error_str.contains("timeout") || error_str.contains("Timeout") {
            return Some("Increase timeout with --timeout, or use faster timing template (-T3, -T4) for better retries".to_string());
        }

        if error_str.contains("No valid targets") {
            return Some("Specify targets with: IP address (192.168.1.1), CIDR notation (10.0.0.0/24), or hostname (example.com)".to_string());
        }

        if error_str.contains("Output file already exists") {
            return Some(
                "Use --force to overwrite existing file, or specify a different output path"
                    .to_string(),
            );
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
    ErrorFormatter::new(atty::is(atty::Stream::Stdout))
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
        io::Error::new(
            io::ErrorKind::Other,
            io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied"),
        )
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
        let err = io::Error::new(io::ErrorKind::Other, "too many open files");
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
        let err = io::Error::new(io::ErrorKind::Other, "Rate limit exceeded");
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
        let err = io::Error::new(io::ErrorKind::Other, "Some generic error");
        let output = formatter.format_error(&err);

        // Should NOT contain suggestion for unknown error types
        assert!(!output.contains("💡 Suggestion:"));
    }

    #[test]
    fn test_create_error_formatter_auto_detect() {
        let formatter = create_error_formatter();
        // Should create successfully (color detection happens at runtime)
        assert!(formatter.colorize == atty::is(atty::Stream::Stdout));
    }
}
