//! Centralized keyboard shortcut system for ProRT-IP TUI
//!
//! This module provides a unified system for managing keyboard shortcuts across all widgets:
//! - Global shortcuts (quit, help, navigation)
//! - Context-specific shortcuts (widget-dependent actions)
//! - Shortcut conflict detection
//! - Help text generation
//!
//! # Examples
//!
//! ```rust,ignore
//! use prtip_tui::shortcuts::ShortcutManager;
//!
//! let shortcuts = ShortcutManager::new();
//! let help_text = shortcuts.get_help_text_for_context("port_table");
//! ```

use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

/// Keyboard shortcut definition
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shortcut {
    /// Key code
    pub key: KeyCode,
    /// Modifiers (Ctrl, Alt, Shift, etc.)
    pub modifiers: KeyModifiers,
    /// Human-readable description
    pub description: String,
    /// Context (widget name or "global")
    pub context: String,
}

impl Shortcut {
    /// Create a new shortcut
    pub fn new(
        key: KeyCode,
        modifiers: KeyModifiers,
        description: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            key,
            modifiers,
            description: description.into(),
            context: context.into(),
        }
    }

    /// Format shortcut for display (e.g., "Ctrl+Q", "Shift+Tab")
    pub fn format_key(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }

        let key_str = match self.key {
            KeyCode::Char(c) => c.to_uppercase().to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::BackTab => "Shift+Tab".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Delete => "Del".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::PageUp => "PgUp".to_string(),
            KeyCode::PageDown => "PgDn".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            _ => format!("{:?}", self.key),
        };

        if parts.is_empty() {
            key_str
        } else {
            format!("{}+{}", parts.join("+"), key_str)
        }
    }
}

/// Centralized shortcut manager
pub struct ShortcutManager {
    /// All registered shortcuts
    shortcuts: Vec<Shortcut>,
    /// Shortcuts grouped by context
    context_shortcuts: HashMap<String, Vec<Shortcut>>,
}

impl ShortcutManager {
    /// Create a new shortcut manager with default shortcuts
    pub fn new() -> Self {
        let mut manager = Self {
            shortcuts: Vec::new(),
            context_shortcuts: HashMap::new(),
        };

        // Register default shortcuts
        manager.register_defaults();

        manager
    }

    /// Register a new shortcut
    pub fn register(&mut self, shortcut: Shortcut) {
        // Check for conflicts in the same context
        if let Some(conflict) = self.find_conflict(&shortcut) {
            eprintln!(
                "Warning: Shortcut conflict detected: {} conflicts with {} in context '{}'",
                shortcut.format_key(),
                conflict.description,
                shortcut.context
            );
        }

        // Add to context map
        self.context_shortcuts
            .entry(shortcut.context.clone())
            .or_default()
            .push(shortcut.clone());

        // Add to global list
        self.shortcuts.push(shortcut);
    }

    /// Find conflicting shortcut (same key+modifiers in same context)
    fn find_conflict(&self, shortcut: &Shortcut) -> Option<&Shortcut> {
        self.context_shortcuts
            .get(&shortcut.context)
            .and_then(|shortcuts| {
                shortcuts.iter().find(|s| {
                    s.key == shortcut.key
                        && s.modifiers == shortcut.modifiers
                        && s.description != shortcut.description
                })
            })
    }

    /// Get all shortcuts for a specific context (includes global shortcuts)
    pub fn get_shortcuts_for_context(&self, context: &str) -> Vec<&Shortcut> {
        let mut result = Vec::new();

        // Add global shortcuts
        if let Some(global) = self.context_shortcuts.get("global") {
            result.extend(global.iter());
        }

        // Add context-specific shortcuts
        if let Some(context_shortcuts) = self.context_shortcuts.get(context) {
            result.extend(context_shortcuts.iter());
        }

        result
    }

    /// Get help text for a specific context
    pub fn get_help_text_for_context(&self, context: &str) -> Vec<(String, String)> {
        self.get_shortcuts_for_context(context)
            .into_iter()
            .map(|s| (s.format_key(), s.description.clone()))
            .collect()
    }

    /// Get all contexts
    pub fn get_contexts(&self) -> Vec<String> {
        self.context_shortcuts.keys().cloned().collect()
    }

    /// Register default shortcuts
    fn register_defaults(&mut self) {
        // === Global Shortcuts ===
        self.register(Shortcut::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
            "Quit application",
            "global",
        ));
        self.register(Shortcut::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
            "Cancel/Exit current view",
            "global",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('?'),
            KeyModifiers::NONE,
            "Toggle help screen",
            "global",
        ));
        self.register(Shortcut::new(
            KeyCode::Tab,
            KeyModifiers::NONE,
            "Next pane/section",
            "global",
        ));
        self.register(Shortcut::new(
            KeyCode::BackTab,
            KeyModifiers::SHIFT,
            "Previous pane/section",
            "global",
        ));

        // === Port Table Shortcuts ===
        self.register(Shortcut::new(
            KeyCode::Up,
            KeyModifiers::NONE,
            "Select previous row",
            "port_table",
        ));
        self.register(Shortcut::new(
            KeyCode::Down,
            KeyModifiers::NONE,
            "Select next row",
            "port_table",
        ));
        self.register(Shortcut::new(
            KeyCode::PageUp,
            KeyModifiers::NONE,
            "Scroll page up",
            "port_table",
        ));
        self.register(Shortcut::new(
            KeyCode::PageDown,
            KeyModifiers::NONE,
            "Scroll page down",
            "port_table",
        ));
        self.register(Shortcut::new(
            KeyCode::Home,
            KeyModifiers::NONE,
            "Jump to first row",
            "port_table",
        ));
        self.register(Shortcut::new(
            KeyCode::End,
            KeyModifiers::NONE,
            "Jump to last row",
            "port_table",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('p'),
            KeyModifiers::NONE,
            "Sort by Port",
            "port_table",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('s'),
            KeyModifiers::NONE,
            "Sort by State",
            "port_table",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('r'),
            KeyModifiers::NONE,
            "Sort by Protocol",
            "port_table",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('v'),
            KeyModifiers::NONE,
            "Sort by Service",
            "port_table",
        ));

        // === Log Widget Shortcuts ===
        self.register(Shortcut::new(
            KeyCode::Up,
            KeyModifiers::NONE,
            "Scroll up one line",
            "log_widget",
        ));
        self.register(Shortcut::new(
            KeyCode::Down,
            KeyModifiers::NONE,
            "Scroll down one line",
            "log_widget",
        ));
        self.register(Shortcut::new(
            KeyCode::PageUp,
            KeyModifiers::NONE,
            "Scroll page up",
            "log_widget",
        ));
        self.register(Shortcut::new(
            KeyCode::PageDown,
            KeyModifiers::NONE,
            "Scroll page down",
            "log_widget",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('s'),
            KeyModifiers::NONE,
            "Toggle auto-scroll",
            "log_widget",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('c'),
            KeyModifiers::NONE,
            "Clear log",
            "log_widget",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('1'),
            KeyModifiers::NONE,
            "Filter: All events",
            "log_widget",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('2'),
            KeyModifiers::NONE,
            "Filter: Ports",
            "log_widget",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('3'),
            KeyModifiers::NONE,
            "Filter: Hosts",
            "log_widget",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('4'),
            KeyModifiers::NONE,
            "Filter: Services",
            "log_widget",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('5'),
            KeyModifiers::NONE,
            "Filter: Errors",
            "log_widget",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('6'),
            KeyModifiers::NONE,
            "Filter: Warnings",
            "log_widget",
        ));

        // === Target Selection Shortcuts ===
        self.register(Shortcut::new(
            KeyCode::Char('b'),
            KeyModifiers::CONTROL,
            "Browse for file",
            "target_selection",
        ));
        self.register(Shortcut::new(
            KeyCode::Delete,
            KeyModifiers::NONE,
            "Remove selected target/exclusion",
            "target_selection",
        ));

        // === Template Selection Shortcuts ===
        self.register(Shortcut::new(
            KeyCode::Char('e'),
            KeyModifiers::NONE,
            "Edit template",
            "template_selection",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('d'),
            KeyModifiers::NONE,
            "Duplicate template",
            "template_selection",
        ));
        self.register(Shortcut::new(
            KeyCode::Delete,
            KeyModifiers::NONE,
            "Delete custom template",
            "template_selection",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('/'),
            KeyModifiers::NONE,
            "Focus filter",
            "template_selection",
        ));

        // === File Browser Shortcuts ===
        self.register(Shortcut::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
            "Select file/Enter directory",
            "file_browser",
        ));
        self.register(Shortcut::new(
            KeyCode::Backspace,
            KeyModifiers::NONE,
            "Navigate to parent directory",
            "file_browser",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('1'),
            KeyModifiers::NONE,
            "Filter: Text files",
            "file_browser",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('2'),
            KeyModifiers::NONE,
            "Filter: CSV files",
            "file_browser",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('3'),
            KeyModifiers::NONE,
            "Filter: JSON files",
            "file_browser",
        ));
        self.register(Shortcut::new(
            KeyCode::Char('4'),
            KeyModifiers::NONE,
            "Filter: All files",
            "file_browser",
        ));
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_creation() {
        let shortcut = Shortcut::new(KeyCode::Char('q'), KeyModifiers::NONE, "Quit", "global");
        assert_eq!(shortcut.key, KeyCode::Char('q'));
        assert_eq!(shortcut.modifiers, KeyModifiers::NONE);
        assert_eq!(shortcut.description, "Quit");
        assert_eq!(shortcut.context, "global");
    }

    #[test]
    fn test_shortcut_formatting() {
        let s1 = Shortcut::new(KeyCode::Char('q'), KeyModifiers::NONE, "Quit", "global");
        assert_eq!(s1.format_key(), "Q");

        let s2 = Shortcut::new(KeyCode::Char('c'), KeyModifiers::CONTROL, "Copy", "global");
        assert_eq!(s2.format_key(), "Ctrl+C");

        let s3 = Shortcut::new(KeyCode::Tab, KeyModifiers::SHIFT, "Previous", "global");
        assert_eq!(s3.format_key(), "Shift+Tab");

        let s4 = Shortcut::new(KeyCode::Up, KeyModifiers::NONE, "Up", "global");
        assert_eq!(s4.format_key(), "↑");
    }

    #[test]
    fn test_manager_creation() {
        let manager = ShortcutManager::new();
        assert!(!manager.shortcuts.is_empty());
        assert!(manager.context_shortcuts.contains_key("global"));
    }

    #[test]
    fn test_get_shortcuts_for_context() {
        let manager = ShortcutManager::new();

        // Global shortcuts should always be included
        let port_table_shortcuts = manager.get_shortcuts_for_context("port_table");
        assert!(port_table_shortcuts
            .iter()
            .any(|s| s.description == "Quit application"));
        assert!(port_table_shortcuts
            .iter()
            .any(|s| s.description == "Sort by Port"));

        // Log widget shortcuts
        let log_shortcuts = manager.get_shortcuts_for_context("log_widget");
        assert!(log_shortcuts.iter().any(|s| s.description == "Clear log"));
        assert!(log_shortcuts
            .iter()
            .any(|s| s.description == "Filter: All events"));
    }

    #[test]
    fn test_get_help_text() {
        let manager = ShortcutManager::new();
        let help_text = manager.get_help_text_for_context("port_table");
        assert!(!help_text.is_empty());

        // Check that quit shortcut is present
        assert!(help_text.iter().any(|(_, desc)| desc == "Quit application"));
    }

    #[test]
    fn test_conflict_detection() {
        let mut manager = ShortcutManager::new();

        // Try to register conflicting shortcut
        let conflicting = Shortcut::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
            "Different action",
            "global",
        );

        // This should trigger a warning (but still register)
        manager.register(conflicting);
    }

    #[test]
    fn test_get_contexts() {
        let manager = ShortcutManager::new();
        let contexts = manager.get_contexts();
        assert!(contexts.contains(&"global".to_string()));
        assert!(contexts.contains(&"port_table".to_string()));
        assert!(contexts.contains(&"log_widget".to_string()));
    }

    #[test]
    fn test_shortcut_equality() {
        let s1 = Shortcut::new(KeyCode::Char('q'), KeyModifiers::NONE, "Quit", "global");
        let s2 = Shortcut::new(KeyCode::Char('q'), KeyModifiers::NONE, "Quit", "global");
        let s3 = Shortcut::new(KeyCode::Char('q'), KeyModifiers::CONTROL, "Quit", "global");

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_special_key_formatting() {
        let shortcuts = vec![
            (KeyCode::Enter, "Enter"),
            (KeyCode::Esc, "Esc"),
            (KeyCode::Backspace, "Backspace"),
            (KeyCode::Delete, "Del"),
            (KeyCode::PageUp, "PgUp"),
            (KeyCode::PageDown, "PgDn"),
            (KeyCode::Home, "Home"),
            (KeyCode::End, "End"),
        ];

        for (key, expected) in shortcuts {
            let s = Shortcut::new(key, KeyModifiers::NONE, "Test", "global");
            assert_eq!(s.format_key(), expected);
        }
    }

    #[test]
    fn test_function_key_formatting() {
        let s = Shortcut::new(KeyCode::F(1), KeyModifiers::NONE, "F1 Help", "global");
        assert_eq!(s.format_key(), "F1");

        let s = Shortcut::new(KeyCode::F(12), KeyModifiers::NONE, "F12", "global");
        assert_eq!(s.format_key(), "F12");
    }

    #[test]
    fn test_multiple_modifiers() {
        let s = Shortcut::new(
            KeyCode::Char('s'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            "Save As",
            "global",
        );
        let formatted = s.format_key();
        assert!(formatted.contains("Ctrl"));
        assert!(formatted.contains("Shift"));
        assert!(formatted.contains("S"));
    }
}
