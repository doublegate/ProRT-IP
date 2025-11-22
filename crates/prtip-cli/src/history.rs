//! Command history management and replay functionality
//!
//! This module provides command history tracking and replay capabilities for
//! the ProRT-IP scanner. All executed commands are stored in a JSON file at
//! `~/.prtip/history.json` with timestamps, arguments, and result summaries.
//!
//! # Features
//!
//! - **Persistent Storage**: JSON-based history stored in user's home directory
//! - **Auto-rotation**: Automatically limits history to 1000 most recent entries
//! - **Replay**: Re-execute previous commands by index
//! - **Modification**: Replay commands with flag modifications
//! - **Atomic Writes**: Uses temp file + rename to prevent corruption
//!
//! # Examples
//!
//! ```no_run
//! use prtip_cli::history::{HistoryManager, HistoryEntry};
//! use chrono::Utc;
//!
//! let mut manager = HistoryManager::new(true)?;
//!
//! // Add a scan to history
//! manager.add_entry(
//!     vec!["prtip".to_string(), "-sS".to_string(), "-p".to_string(), "80,443".to_string(), "192.168.1.0/24".to_string()],
//!     "SYN scan of 192.168.1.0/24: 5 hosts, 2 open ports",
//!     0,
//! )?;
//!
//! // List all entries
//! for (idx, entry) in manager.list_entries().iter().enumerate() {
//!     println!("[{}] {}: {}", idx, entry.timestamp, entry.summary);
//! }
//!
//! // Get specific entry
//! if let Some(entry) = manager.get_entry(0) {
//!     println!("Command: {}", entry.command);
//! }
//!
//! // Clear history
//! manager.clear()?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Maximum number of history entries to keep
const MAX_HISTORY_ENTRIES: usize = 1000;

/// A single command history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// When the command was executed
    pub timestamp: DateTime<Utc>,
    /// Full command string (e.g., "prtip -sS -p 80,443 target.com")
    pub command: String,
    /// Individual arguments (e.g., ["prtip", "-sS", "-p", "80,443", "target.com"])
    pub args: Vec<String>,
    /// Human-readable summary (e.g., "SYN scan of 192.168.1.0/24: 5 hosts, 2 open ports")
    pub summary: String,
    /// Exit code (0 = success, non-zero = error)
    pub exit_code: i32,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new<S: Into<String>>(args: Vec<String>, summary: S, exit_code: i32) -> Self {
        let command = args.join(" ");
        Self {
            timestamp: Utc::now(),
            command,
            args,
            summary: summary.into(),
            exit_code,
        }
    }

    /// Format the entry for display
    pub fn format_display(&self, index: usize) -> String {
        let timestamp = self.timestamp.format("%Y-%m-%d %H:%M:%S");
        let status = if self.exit_code == 0 {
            "✓".to_string()
        } else {
            format!("✗({})", self.exit_code)
        };
        format!(
            "[{}] {} {} {}\n    {}",
            index, timestamp, status, self.command, self.summary
        )
    }
}

/// Manages command history storage and retrieval
pub struct HistoryManager {
    /// Path to history file (~/.prtip/history.json)
    history_path: PathBuf,
    /// In-memory history entries
    entries: Vec<HistoryEntry>,
}

impl HistoryManager {
    /// Create a new history manager
    ///
    /// This will:
    /// 1. Create `~/.prtip/` directory if it doesn't exist (if enabled)
    /// 2. Load existing history from `~/.prtip/history.json` (if enabled)
    /// 3. Create empty history file if it doesn't exist (if enabled)
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to actually save history to disk. If false, operates in memory-only mode.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Home directory cannot be determined
    /// - Directory creation fails
    /// - History file is corrupted (invalid JSON)
    pub fn new(enabled: bool) -> Result<Self> {
        // Check if history is disabled (for testing, CI environments, or user preference)
        // This prevents file I/O conflicts when tests run in parallel
        if std::env::var("PRTIP_DISABLE_HISTORY").is_ok() || !enabled {
            // Return in-memory-only manager with dummy path
            return Ok(Self {
                history_path: PathBuf::from("/dev/null"),
                entries: Vec::new(),
            });
        }

        let history_path = Self::get_history_path()?;

        // Create ~/.prtip/ directory if it doesn't exist
        if let Some(parent) = history_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Load existing history or create empty
        let entries = if history_path.exists() {
            Self::load_from_file(&history_path)?
        } else {
            Vec::new()
        };

        Ok(Self {
            history_path,
            entries,
        })
    }

    /// Create a history manager with custom path (for testing)
    ///
    /// This is exposed for integration tests.
    #[doc(hidden)]
    #[allow(dead_code)] // Used in integration tests
    pub fn with_path(history_path: PathBuf) -> Self {
        Self {
            history_path,
            entries: Vec::new(),
        }
    }

    /// Get the path to the history file
    fn get_history_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".prtip").join("history.json"))
    }

    /// Load history entries from JSON file
    fn load_from_file(path: &PathBuf) -> Result<Vec<HistoryEntry>> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read history file: {}", path.display()))?;

        if contents.trim().is_empty() {
            return Ok(Vec::new());
        }

        serde_json::from_str(&contents).with_context(|| {
            format!(
                "History file is corrupted (invalid JSON): {}\n\
                 Consider running 'prtip history --clear' to reset",
                path.display()
            )
        })
    }

    /// Save history entries to JSON file using atomic write
    ///
    /// This uses the write-to-temp-then-rename pattern to prevent corruption
    /// from partial writes (e.g., power loss, CTRL+C).
    fn save_to_file(&self) -> Result<()> {
        // Skip file operations if history is disabled (dummy path indicates this)
        if self.history_path == Path::new("/dev/null") {
            return Ok(());
        }

        // Serialize to JSON with pretty formatting
        let json = serde_json::to_string_pretty(&self.entries)
            .context("Failed to serialize history to JSON")?;

        // Write to temporary file first
        let temp_path = self.history_path.with_extension("json.tmp");
        let mut file = File::create(&temp_path)
            .with_context(|| format!("Failed to create temp file: {}", temp_path.display()))?;

        file.write_all(json.as_bytes())
            .context("Failed to write history to temp file")?;

        file.sync_all()
            .context("Failed to sync temp file to disk")?;

        // Atomic rename (this is atomic on all platforms)
        fs::rename(&temp_path, &self.history_path).with_context(|| {
            format!(
                "Failed to rename {} to {}",
                temp_path.display(),
                self.history_path.display()
            )
        })?;

        Ok(())
    }

    /// Add a new history entry
    ///
    /// This will:
    /// 1. Create a new entry with current timestamp
    /// 2. Add to in-memory list
    /// 3. Auto-rotate if exceeding MAX_HISTORY_ENTRIES (keeps newest 1000)
    /// 4. Save to disk atomically
    ///
    /// # Arguments
    ///
    /// * `args` - Full command arguments (e.g., ["prtip", "-sS", "target.com"])
    /// * `summary` - Human-readable summary (e.g., "SYN scan: 5 open ports")
    /// * `exit_code` - Exit code (0 = success, non-zero = error)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_cli::history::HistoryManager;
    /// let mut manager = HistoryManager::new(true)?;
    /// manager.add_entry(
    ///     vec!["prtip".to_string(), "-sS".to_string(), "-p".to_string(), "80,443".to_string(), "192.168.1.1".to_string()],
    ///     "SYN scan of 192.168.1.1: 2 open ports",
    ///     0,
    /// )?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn add_entry<S: Into<String>>(
        &mut self,
        args: Vec<String>,
        summary: S,
        exit_code: i32,
    ) -> Result<()> {
        let entry = HistoryEntry::new(args, summary.into(), exit_code);
        self.entries.push(entry);

        // Auto-rotate if exceeding limit
        if self.entries.len() > MAX_HISTORY_ENTRIES {
            // Keep only the newest MAX_HISTORY_ENTRIES
            let start = self.entries.len() - MAX_HISTORY_ENTRIES;
            self.entries = self.entries[start..].to_vec();
        }

        self.save_to_file()
    }

    /// Get a specific history entry by index (0 = oldest)
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index (0 = oldest entry)
    ///
    /// # Returns
    ///
    /// Returns `Some(&HistoryEntry)` if index is valid, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_cli::history::HistoryManager;
    /// let manager = HistoryManager::new(true)?;
    /// if let Some(entry) = manager.get_entry(0) {
    ///     println!("Command: {}", entry.command);
    /// } else {
    ///     println!("No entry at index 0");
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn get_entry(&self, index: usize) -> Option<&HistoryEntry> {
        self.entries.get(index)
    }

    /// Get the most recent history entry
    ///
    /// # Returns
    ///
    /// Returns `Some(&HistoryEntry)` if history is not empty, `None` otherwise.
    pub fn get_last(&self) -> Option<&HistoryEntry> {
        self.entries.last()
    }

    /// List all history entries
    ///
    /// # Returns
    ///
    /// Returns a slice of all history entries, ordered from oldest to newest.
    pub fn list_entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Get the number of history entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all history entries
    ///
    /// This removes all entries from memory and deletes the history file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_cli::history::HistoryManager;
    /// let mut manager = HistoryManager::new(true)?;
    /// manager.clear()?;
    /// assert!(manager.is_empty());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();

        // Skip file operations if history is disabled (dummy path indicates this)
        if self.history_path == Path::new("/dev/null") {
            return Ok(());
        }

        // Delete history file if it exists
        if self.history_path.exists() {
            fs::remove_file(&self.history_path).with_context(|| {
                format!(
                    "Failed to delete history file: {}",
                    self.history_path.display()
                )
            })?;
        }

        Ok(())
    }

    /// Rebuild command arguments from a history entry with optional modifications
    ///
    /// This is used for the replay functionality. It takes a history entry and
    /// optionally adds or replaces flags.
    ///
    /// # Arguments
    ///
    /// * `entry` - The history entry to replay
    /// * `modifications` - Optional additional arguments to append/override
    ///
    /// # Returns
    ///
    /// Returns a vector of command arguments ready for execution.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use prtip_cli::history::HistoryManager;
    /// let manager = HistoryManager::new(true)?;
    /// if let Some(entry) = manager.get_entry(0) {
    ///     // Replay with modifications
    ///     let args = HistoryManager::rebuild_command(entry, Some(vec!["--verbose"]));
    ///     // args = ["prtip", "-sS", "target.com", "--verbose"]
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn rebuild_command(entry: &HistoryEntry, modifications: Option<Vec<&str>>) -> Vec<String> {
        let mut args = entry.args.clone();

        if let Some(mods) = modifications {
            args.extend(mods.iter().map(|s| s.to_string()));
        }

        args
    }

    /// Validate that a history entry can be replayed
    ///
    /// This checks for basic sanity (non-empty args, starts with "prtip").
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Entry has no arguments
    /// - First argument is not "prtip"
    pub fn validate_replay(entry: &HistoryEntry) -> Result<()> {
        if entry.args.is_empty() {
            bail!("History entry has no arguments");
        }

        if !entry.args[0].ends_with("prtip") {
            bail!("History entry does not start with 'prtip' command");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a temporary history manager for testing
    fn create_test_manager() -> (HistoryManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.json");

        let manager = HistoryManager {
            history_path,
            entries: Vec::new(),
        };

        (manager, temp_dir)
    }

    #[test]
    fn test_new_history_entry() {
        let args = ["prtip", "-sS", "target.com"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let entry = HistoryEntry::new(args, "Test summary", 0);

        assert_eq!(entry.command, "prtip -sS target.com");
        assert_eq!(entry.args.len(), 3);
        assert_eq!(entry.summary, "Test summary");
        assert_eq!(entry.exit_code, 0);
    }

    #[test]
    fn test_add_and_retrieve_entry() {
        let (mut manager, _temp) = create_test_manager();

        manager
            .add_entry(
                ["prtip", "-sS", "target.com"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "Test scan",
                0,
            )
            .unwrap();

        assert_eq!(manager.len(), 1);
        assert!(!manager.is_empty());

        let entry = manager.get_entry(0).unwrap();
        assert_eq!(entry.command, "prtip -sS target.com");
        assert_eq!(entry.summary, "Test scan");
    }

    #[test]
    fn test_get_last() {
        let (mut manager, _temp) = create_test_manager();

        manager
            .add_entry(
                ["prtip", "-sS", "target1"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "Scan 1",
                0,
            )
            .unwrap();
        manager
            .add_entry(
                ["prtip", "-sT", "target2"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "Scan 2",
                0,
            )
            .unwrap();

        let last = manager.get_last().unwrap();
        assert_eq!(last.command, "prtip -sT target2");
        assert_eq!(last.summary, "Scan 2");
    }

    #[test]
    fn test_list_entries() {
        let (mut manager, _temp) = create_test_manager();

        manager
            .add_entry(
                ["prtip", "-sS", "target1"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "Scan 1",
                0,
            )
            .unwrap();
        manager
            .add_entry(
                ["prtip", "-sT", "target2"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "Scan 2",
                0,
            )
            .unwrap();

        let entries = manager.list_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].summary, "Scan 1");
        assert_eq!(entries[1].summary, "Scan 2");
    }

    #[test]
    fn test_clear_history() {
        let (mut manager, _temp) = create_test_manager();

        manager
            .add_entry(
                ["prtip", "-sS", "target"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "Test",
                0,
            )
            .unwrap();
        assert_eq!(manager.len(), 1);

        manager.clear().unwrap();
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_auto_rotation() {
        let (mut manager, _temp) = create_test_manager();

        // Add more than MAX_HISTORY_ENTRIES
        for i in 0..(MAX_HISTORY_ENTRIES + 100) {
            manager
                .add_entry(
                    ["prtip", "-sS", &format!("target{}", i)]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    format!("Scan {}", i),
                    0,
                )
                .unwrap();
        }

        // Should be capped at MAX_HISTORY_ENTRIES
        assert_eq!(manager.len(), MAX_HISTORY_ENTRIES);

        // First entry should be the 100th scan (0-99 were rotated out)
        let first = manager.get_entry(0).unwrap();
        assert_eq!(first.summary, "Scan 100");
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.json");

        // Create and populate manager
        {
            let mut manager = HistoryManager {
                history_path: history_path.clone(),
                entries: Vec::new(),
            };

            manager
                .add_entry(
                    ["prtip", "-sS", "target1"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    "Scan 1",
                    0,
                )
                .unwrap();
            manager
                .add_entry(
                    ["prtip", "-sT", "target2"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    "Scan 2",
                    1,
                )
                .unwrap();
        }

        // Load in new manager
        let loaded_entries = HistoryManager::load_from_file(&history_path).unwrap();
        assert_eq!(loaded_entries.len(), 2);
        assert_eq!(loaded_entries[0].summary, "Scan 1");
        assert_eq!(loaded_entries[1].summary, "Scan 2");
        assert_eq!(loaded_entries[1].exit_code, 1);
    }

    #[test]
    fn test_rebuild_command_without_modifications() {
        let entry = HistoryEntry::new(
            ["prtip", "-sS", "-p", "80,443", "target.com"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            "Test",
            0,
        );

        let rebuilt = HistoryManager::rebuild_command(&entry, None);
        assert_eq!(rebuilt, entry.args);
    }

    #[test]
    fn test_rebuild_command_with_modifications() {
        let entry = HistoryEntry::new(
            ["prtip", "-sS", "target.com"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            "Test",
            0,
        );

        let rebuilt = HistoryManager::rebuild_command(&entry, Some(vec!["-p", "80,443"]));
        assert_eq!(rebuilt.len(), 5);
        assert_eq!(rebuilt[3], "-p");
        assert_eq!(rebuilt[4], "80,443");
    }

    #[test]
    fn test_validate_replay_valid() {
        let entry = HistoryEntry::new(
            ["prtip", "-sS", "target.com"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            "Test",
            0,
        );

        assert!(HistoryManager::validate_replay(&entry).is_ok());
    }

    #[test]
    fn test_validate_replay_empty_args() {
        let entry = HistoryEntry::new(Vec::new(), "Test", 0);
        assert!(HistoryManager::validate_replay(&entry).is_err());
    }

    #[test]
    fn test_validate_replay_wrong_command() {
        let entry = HistoryEntry::new(
            ["nmap", "-sS", "target.com"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            "Test",
            0,
        );

        assert!(HistoryManager::validate_replay(&entry).is_err());
    }

    #[test]
    fn test_format_display_success() {
        let entry = HistoryEntry::new(
            ["prtip", "-sS", "target.com"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            "SYN scan: 5 open ports",
            0,
        );

        let display = entry.format_display(0);
        assert!(display.contains("[0]"));
        assert!(display.contains("✓"));
        assert!(display.contains("prtip -sS target.com"));
        assert!(display.contains("SYN scan: 5 open ports"));
    }

    #[test]
    fn test_format_display_error() {
        let entry = HistoryEntry::new(
            ["prtip", "-sS", "invalid-target"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            "Failed to resolve hostname",
            1,
        );

        let display = entry.format_display(0);
        assert!(display.contains("✗(1)"));
        assert!(display.contains("Failed to resolve hostname"));
    }

    #[test]
    fn test_empty_history_file_loads_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.json");

        // Create empty file
        File::create(&history_path).unwrap();

        let entries = HistoryManager::load_from_file(&history_path).unwrap();
        assert_eq!(entries.len(), 0);
    }
}
