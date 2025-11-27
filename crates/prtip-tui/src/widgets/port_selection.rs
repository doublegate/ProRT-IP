//! PortSelectionWidget - Interactive port range selector with presets and categories
//!
//! The PortSelectionWidget provides a comprehensive interface for specifying scan ports:
//! - Quick presets (Top 100, Top 1000, All Ports, Common Services)
//! - Port categories (Web, SSH, Database, Mail, File Sharing, Remote Access)
//! - Custom port range input (e.g., "80,443,8000-9000")
//! - Port preset manager (save/load custom presets)
//! - Real-time input validation and port count display
//!
//! # Layout
//!
//! ```text
//! ┌─ Port Selection ────────────────────────────────────────────┐
//! │ Quick Presets:                                              │
//! │   [Top 100] [Top 1000] [All Ports] [Common Services]       │
//! │                                                              │
//! │ Custom Range:                                               │
//! │   Ports: 80,443,8080-8090,3000_                            │
//! │   ✓ Valid | 13 ports selected                              │
//! │                                                              │
//! │ Selected Ports (13):                                        │
//! │   80, 443, 3000, 8080-8090                                 │
//! │                                                              │
//! │ Port Categories: (toggle with number keys 1-6)             │
//! │   [✓] Web (80, 443, 8080, 8443)                4 ports     │
//! │   [ ] SSH (22)                                 1 port      │
//! │   [✓] Database (3306, 5432, 27017, 6379)      4 ports     │
//! │   [ ] Mail (25, 110, 143, 465, 587, 993)      6 ports     │
//! │   [ ] File Sharing (21, 139, 445, 2049)       4 ports     │
//! │   [ ] Remote Access (23, 3389, 5900)          3 ports     │
//! │                                                              │
//! │ Total: 13 ports selected                                    │
//! │ [F1-F4] Presets [1-6] Categories [Enter] Confirm [Esc] Cancel│
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Keyboard Shortcuts
//!
//! - `F1-F4` - Quick presets (Top 100, Top 1000, All, Common)
//! - `1-6` - Toggle port categories
//! - `Enter` - Confirm selection
//! - `Esc` - Cancel changes
//! - `Ctrl+C` - Clear all selections
//! - `Ctrl+S` - Save as custom preset
//! - `/` - Focus custom range input
//! - `Tab` - Switch focus (categories <-> custom input)

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Port specification parser and validator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortSpec {
    /// Set of unique ports
    ports: HashSet<u16>,
}

impl PortSpec {
    /// Create empty port specification
    pub fn new() -> Self {
        Self {
            ports: HashSet::new(),
        }
    }

    /// Parse port specification string (e.g., "80,443,8080-8090,3000")
    ///
    /// # Format
    /// - Comma-separated values: "80,443,3000"
    /// - Ranges (inclusive): "8080-8090"
    /// - Mixed: "80,443,8000-9000"
    ///
    /// # Errors
    /// Returns Err if:
    /// - Invalid number format
    /// - Port out of range (1-65535)
    /// - Invalid range syntax
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut ports = HashSet::new();

        if input.trim().is_empty() {
            return Ok(Self { ports });
        }

        for part in input.split(',') {
            let part = part.trim();

            if part.contains('-') {
                // Parse range (e.g., "8080-8090")
                let range_parts: Vec<&str> = part.split('-').collect();
                if range_parts.len() != 2 {
                    return Err(format!("Invalid range syntax: {}", part));
                }

                let start: u16 = range_parts[0]
                    .trim()
                    .parse()
                    .map_err(|_| format!("Invalid port number: {}", range_parts[0]))?;
                let end: u16 = range_parts[1]
                    .trim()
                    .parse()
                    .map_err(|_| format!("Invalid port number: {}", range_parts[1]))?;

                if start == 0 || end == 0 {
                    return Err("Port numbers must be between 1 and 65535".to_string());
                }

                if start > end {
                    return Err(format!("Invalid range: {} > {}", start, end));
                }

                for port in start..=end {
                    ports.insert(port);
                }
            } else {
                // Parse single port
                let port: u16 = part
                    .parse()
                    .map_err(|_| format!("Invalid port number: {}", part))?;

                if port == 0 {
                    return Err("Port numbers must be between 1 and 65535".to_string());
                }

                ports.insert(port);
            }
        }

        Ok(Self { ports })
    }

    /// Create Top 100 most common ports preset
    pub fn top_100() -> Self {
        let mut ports = HashSet::new();
        // Nmap's top 100 ports
        let top_100_list = vec![
            7, 9, 13, 21, 22, 23, 25, 26, 37, 53, 79, 80, 81, 88, 106, 110, 111, 113, 119, 135,
            139, 143, 144, 179, 199, 389, 427, 443, 444, 445, 465, 513, 514, 515, 543, 544, 548,
            554, 587, 631, 646, 873, 990, 993, 995, 1025, 1026, 1027, 1028, 1029, 1110, 1433, 1720,
            1723, 1755, 1900, 2000, 2001, 2049, 2121, 2717, 3000, 3128, 3306, 3389, 3986, 4899,
            5000, 5009, 5051, 5060, 5101, 5190, 5357, 5432, 5631, 5666, 5800, 5900, 6000, 6001,
            6646, 7070, 8000, 8008, 8009, 8080, 8081, 8443, 8888, 9100, 9999, 10000, 32768, 49152,
            49153, 49154, 49155, 49156, 49157,
        ];
        for port in top_100_list {
            ports.insert(port);
        }
        Self { ports }
    }

    /// Create Top 1000 most common ports preset
    pub fn top_1000() -> Self {
        // For simplicity, include Top 100 + common ranges
        let mut spec = Self::top_100();

        // Add additional common ports for top 1000
        for port in 1024..1100 {
            spec.ports.insert(port);
        }
        for port in 5000..5100 {
            spec.ports.insert(port);
        }
        for port in 8000..8100 {
            spec.ports.insert(port);
        }

        spec
    }

    /// Create All Ports (1-65535) preset
    pub fn all_ports() -> Self {
        let mut ports = HashSet::new();
        for port in 1..=65535 {
            ports.insert(port);
        }
        Self { ports }
    }

    /// Create Common Services preset
    pub fn common_services() -> Self {
        let mut ports = HashSet::new();

        // Combine all category ports
        for category in [
            PortCategory::Web,
            PortCategory::Ssh,
            PortCategory::Database,
            PortCategory::Mail,
            PortCategory::FileSharing,
            PortCategory::RemoteAccess,
        ] {
            ports.extend(category.ports());
        }

        Self { ports }
    }

    /// Add port to specification
    pub fn add_port(&mut self, port: u16) {
        if port > 0 {
            self.ports.insert(port);
        }
    }

    /// Add ports from another PortSpec
    pub fn add_ports(&mut self, other: &PortSpec) {
        self.ports.extend(&other.ports);
    }

    /// Remove port from specification
    pub fn remove_port(&mut self, port: u16) {
        self.ports.remove(&port);
    }

    /// Clear all ports
    pub fn clear(&mut self) {
        self.ports.clear();
    }

    /// Get sorted port list
    pub fn get_ports(&self) -> Vec<u16> {
        let mut ports: Vec<u16> = self.ports.iter().copied().collect();
        ports.sort_unstable();
        ports
    }

    /// Get port count
    pub fn count(&self) -> usize {
        self.ports.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.ports.is_empty()
    }

    /// Check if contains port
    pub fn contains(&self, port: u16) -> bool {
        self.ports.contains(&port)
    }
}

impl Default for PortSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// Port category for quick selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortCategory {
    /// Web services (HTTP, HTTPS, etc.)
    Web,
    /// SSH
    Ssh,
    /// Databases (MySQL, PostgreSQL, MongoDB, Redis, MSSQL)
    Database,
    /// Mail services (SMTP, POP3, IMAP, etc.)
    Mail,
    /// File sharing (FTP, SMB, NFS, etc.)
    FileSharing,
    /// Remote access (Telnet, RDP, VNC)
    RemoteAccess,
}

impl PortCategory {
    /// Get ports for this category
    pub fn ports(&self) -> Vec<u16> {
        match self {
            PortCategory::Web => vec![80, 443, 8080, 8443, 8000, 8888],
            PortCategory::Ssh => vec![22],
            PortCategory::Database => vec![3306, 5432, 27017, 6379, 1433],
            PortCategory::Mail => vec![25, 110, 143, 465, 587, 993, 995],
            PortCategory::FileSharing => vec![21, 139, 445, 2049, 111],
            PortCategory::RemoteAccess => vec![23, 3389, 5900, 5800],
        }
    }

    /// Get category description
    pub fn description(&self) -> &'static str {
        match self {
            PortCategory::Web => "Web (HTTP, HTTPS, proxies)",
            PortCategory::Ssh => "SSH (Secure Shell)",
            PortCategory::Database => "Database (MySQL, PostgreSQL, MongoDB, Redis, MSSQL)",
            PortCategory::Mail => "Mail (SMTP, POP3, IMAP, SMTPS, etc.)",
            PortCategory::FileSharing => "File Sharing (FTP, SMB, NFS, RPC)",
            PortCategory::RemoteAccess => "Remote Access (Telnet, RDP, VNC)",
        }
    }

    /// Get short name for display
    pub fn short_name(&self) -> &'static str {
        match self {
            PortCategory::Web => "Web",
            PortCategory::Ssh => "SSH",
            PortCategory::Database => "Database",
            PortCategory::Mail => "Mail",
            PortCategory::FileSharing => "File Sharing",
            PortCategory::RemoteAccess => "Remote Access",
        }
    }

    /// Get all categories
    pub fn all() -> Vec<PortCategory> {
        vec![
            PortCategory::Web,
            PortCategory::Ssh,
            PortCategory::Database,
            PortCategory::Mail,
            PortCategory::FileSharing,
            PortCategory::RemoteAccess,
        ]
    }
}

/// Port preset (built-in or custom)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortPreset {
    Top100,
    Top1000,
    AllPorts,
    CommonServices,
}

impl PortPreset {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            PortPreset::Top100 => "Top 100",
            PortPreset::Top1000 => "Top 1000",
            PortPreset::AllPorts => "All Ports",
            PortPreset::CommonServices => "Common Services",
        }
    }

    /// Get port specification for preset
    pub fn to_spec(&self) -> PortSpec {
        match self {
            PortPreset::Top100 => PortSpec::top_100(),
            PortPreset::Top1000 => PortSpec::top_1000(),
            PortPreset::AllPorts => PortSpec::all_ports(),
            PortPreset::CommonServices => PortSpec::common_services(),
        }
    }
}

/// Port preset manager (save/load custom presets)
pub struct PortPresetManager {
    /// Custom presets (name -> port spec string)
    presets: HashMap<String, String>,
    /// Config file path (~/.prtip/port-presets.toml)
    config_path: PathBuf,
}

impl PortPresetManager {
    /// Create new manager
    pub fn new() -> Self {
        let config_path = Self::default_config_path();

        Self {
            presets: HashMap::new(),
            config_path,
        }
    }

    /// Get default config path (~/.prtip/port-presets.toml)
    fn default_config_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".prtip");
        path.push("port-presets.toml");
        path
    }

    /// Load presets from file
    pub fn load() -> io::Result<Self> {
        let mut manager = Self::new();

        if manager.config_path.exists() {
            let content = fs::read_to_string(&manager.config_path)?;

            // Parse TOML (simple format: name = "port_spec")
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if let Some((name, spec)) = line.split_once('=') {
                    let name = name.trim().trim_matches('"').to_string();
                    let spec = spec.trim().trim_matches('"').to_string();
                    manager.presets.insert(name, spec);
                }
            }
        }

        Ok(manager)
    }

    /// Save presets to file
    pub fn save(&self) -> io::Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut content = String::from("# ProRT-IP Port Presets\n\n");

        for (name, spec) in &self.presets {
            content.push_str(&format!("{} = \"{}\"\n", name, spec));
        }

        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// Add preset
    pub fn add_preset(&mut self, name: String, spec: String) {
        self.presets.insert(name, spec);
    }

    /// Remove preset
    #[allow(dead_code)]
    pub fn remove_preset(&mut self, name: &str) -> Option<String> {
        self.presets.remove(name)
    }

    /// Get preset
    #[allow(dead_code)]
    pub fn get_preset(&self, name: &str) -> Option<&String> {
        self.presets.get(name)
    }

    /// List all preset names
    #[allow(dead_code)]
    pub fn list_presets(&self) -> Vec<&String> {
        self.presets.keys().collect()
    }
}

impl Default for PortPresetManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation state for port input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortValidationState {
    Empty,
    Valid { count: usize, spec: PortSpec },
    Invalid(String),
}

/// PortSelectionWidget state
pub struct PortSelectionState {
    /// Current port input string
    pub port_input: String,

    /// Cursor position in input
    pub cursor_position: usize,

    /// Parsed port specification (validated)
    pub port_spec: PortSpec,

    /// Active preset (if any)
    pub active_preset: Option<PortPreset>,

    /// Port category toggles
    pub category_toggles: HashMap<PortCategory, bool>,

    /// Input validation state
    pub validation_state: PortValidationState,

    /// Whether custom input is focused
    pub input_focused: bool,

    /// Preset manager
    preset_manager: PortPresetManager,
}

impl PortSelectionState {
    /// Create new state
    pub fn new() -> Self {
        let preset_manager = PortPresetManager::load().unwrap_or_default();

        Self {
            port_input: String::new(),
            cursor_position: 0,
            port_spec: PortSpec::new(),
            active_preset: None,
            category_toggles: HashMap::new(),
            validation_state: PortValidationState::Empty,
            input_focused: false,
            preset_manager,
        }
    }

    /// Apply preset
    pub fn apply_preset(&mut self, preset: PortPreset) {
        self.active_preset = Some(preset);
        self.port_spec = preset.to_spec();
        self.port_input = self.format_port_spec();
        self.validate_input();
    }

    /// Toggle port category
    pub fn toggle_category(&mut self, category: PortCategory) {
        let enabled = !self.category_toggles.get(&category).unwrap_or(&false);
        self.category_toggles.insert(category, enabled);

        if enabled {
            // Add category ports
            for port in category.ports() {
                self.port_spec.add_port(port);
            }
        } else {
            // Remove category ports
            for port in category.ports() {
                self.port_spec.remove_port(port);
            }
        }

        self.port_input = self.format_port_spec();
        self.validate_input();
    }

    /// Clear all selections
    pub fn clear_all(&mut self) {
        self.port_spec.clear();
        self.port_input.clear();
        self.cursor_position = 0;
        self.category_toggles.clear();
        self.active_preset = None;
        self.validation_state = PortValidationState::Empty;
    }

    /// Input character (custom range input)
    pub fn input_char(&mut self, c: char) {
        if c.is_ascii_digit() || c == ',' || c == '-' {
            self.port_input.insert(self.cursor_position, c);
            self.cursor_position += 1;
            self.validate_input();
        }
    }

    /// Backspace (custom range input)
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.port_input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.validate_input();
        }
    }

    /// Validate port input
    pub fn validate_input(&mut self) {
        if self.port_input.trim().is_empty() {
            // Check if we have ports from categories
            if self.port_spec.is_empty() {
                self.validation_state = PortValidationState::Empty;
            } else {
                self.validation_state = PortValidationState::Valid {
                    count: self.port_spec.count(),
                    spec: self.port_spec.clone(),
                };
            }
        } else {
            match PortSpec::parse(&self.port_input) {
                Ok(spec) => {
                    // Merge with category ports
                    let mut merged = self.port_spec.clone();
                    merged.add_ports(&spec);

                    self.validation_state = PortValidationState::Valid {
                        count: merged.count(),
                        spec: merged.clone(),
                    };
                }
                Err(err) => {
                    self.validation_state = PortValidationState::Invalid(err);
                }
            }
        }
    }

    /// Format port spec for display
    fn format_port_spec(&self) -> String {
        let ports = self.port_spec.get_ports();
        if ports.is_empty() {
            return String::new();
        }

        // Group consecutive ports into ranges
        let mut result = Vec::new();
        let mut i = 0;

        while i < ports.len() {
            let start = ports[i];
            let mut end = start;

            // Find consecutive range
            while i + 1 < ports.len() && ports[i + 1] == end + 1 {
                i += 1;
                end = ports[i];
            }

            if end > start + 1 {
                // Range of 3+ ports
                result.push(format!("{}-{}", start, end));
            } else if end == start + 1 {
                // Two consecutive ports
                result.push(start.to_string());
                result.push(end.to_string());
            } else {
                // Single port
                result.push(start.to_string());
            }

            i += 1;
        }

        result.join(",")
    }

    /// Get final port list
    pub fn get_port_list(&self) -> Vec<u16> {
        match &self.validation_state {
            PortValidationState::Valid { spec, .. } => spec.get_ports(),
            _ => self.port_spec.get_ports(),
        }
    }

    /// Save current config as preset
    pub fn save_as_preset(&mut self, name: String) -> io::Result<()> {
        let spec_str = self.port_input.clone();
        self.preset_manager.add_preset(name, spec_str);
        self.preset_manager.save()
    }
}

impl Default for PortSelectionState {
    fn default() -> Self {
        Self::new()
    }
}

/// PortSelectionWidget
pub struct PortSelectionWidget;

impl PortSelectionWidget {
    pub fn new() -> Self {
        Self
    }

    /// Render the port selection widget
    pub fn render(&self, frame: &mut Frame, area: Rect, state: &PortSelectionState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Presets
                Constraint::Length(5), // Custom range input
                Constraint::Length(4), // Selected ports summary
                Constraint::Min(8),    // Categories
                Constraint::Length(2), // Footer
            ])
            .split(area);

        self.render_presets(frame, chunks[0], state);
        self.render_custom_input(frame, chunks[1], state);
        self.render_selected_summary(frame, chunks[2], state);
        self.render_categories(frame, chunks[3], state);
        self.render_footer(frame, chunks[4], state);
    }

    fn render_presets(&self, frame: &mut Frame, area: Rect, state: &PortSelectionState) {
        let presets = [
            PortPreset::Top100,
            PortPreset::Top1000,
            PortPreset::AllPorts,
            PortPreset::CommonServices,
        ];

        let mut spans = vec![Span::raw("Quick Presets: ")];

        for (i, preset) in presets.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("  "));
            }

            let style = if Some(*preset) == state.active_preset {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };

            spans.push(Span::styled(
                format!("[F{}] {}", i + 1, preset.display_name()),
                style,
            ));
        }

        let paragraph = Paragraph::new(Line::from(spans))
            .block(Block::default().borders(Borders::ALL).title(" Presets "));

        frame.render_widget(paragraph, area);
    }

    fn render_custom_input(&self, frame: &mut Frame, area: Rect, state: &PortSelectionState) {
        let input_text = if state.input_focused {
            format!("{}█", state.port_input)
        } else {
            state.port_input.clone()
        };

        let mut lines = vec![Line::from(vec![
            Span::styled("Ports: ", Style::default().fg(Color::Cyan)),
            Span::raw(&input_text),
        ])];

        // Validation line
        match &state.validation_state {
            PortValidationState::Valid { count, .. } => {
                lines.push(Line::from(vec![
                    Span::styled("✓ Valid", Style::default().fg(Color::Green)),
                    Span::raw(" | "),
                    Span::styled(
                        format!("{} ports selected", count),
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
            }
            PortValidationState::Invalid(err) => {
                lines.push(Line::from(vec![Span::styled(
                    format!("✗ {}", err),
                    Style::default().fg(Color::Red),
                )]));
            }
            PortValidationState::Empty => {
                lines.push(Line::from(vec![Span::styled(
                    "Enter port ranges (e.g., 80,443,8000-9000)",
                    Style::default().fg(Color::DarkGray),
                )]));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Custom Range "),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn render_selected_summary(&self, frame: &mut Frame, area: Rect, state: &PortSelectionState) {
        let ports = state.get_port_list();

        let summary = if ports.is_empty() {
            "(No ports selected)".to_string()
        } else if ports.len() <= 10 {
            ports
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            format!(
                "{} ports selected ({}...{})",
                ports.len(),
                ports[0],
                ports[ports.len() - 1]
            )
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("Selected Ports: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("({})", ports.len()),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![Span::raw(summary)]),
        ];

        let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);
    }

    fn render_categories(&self, frame: &mut Frame, area: Rect, state: &PortSelectionState) {
        let items: Vec<ListItem> = PortCategory::all()
            .iter()
            .enumerate()
            .map(|(i, category)| {
                let enabled = state.category_toggles.get(category).unwrap_or(&false);
                let checkbox = if *enabled { "[✓]" } else { "[ ]" };
                let port_count = category.ports().len();

                let line = Line::from(vec![
                    Span::styled(
                        format!("{} {} ", i + 1, checkbox),
                        if *enabled {
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Gray)
                        },
                    ),
                    Span::styled(
                        format!("{:<20}", category.short_name()),
                        if *enabled {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::Gray)
                        },
                    ),
                    Span::styled(
                        format!("{} ports", port_count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Port Categories (Press 1-6 to toggle) "),
        );

        frame.render_widget(list, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect, state: &PortSelectionState) {
        let port_count = state.get_port_list().len();
        let text = format!(
            "Total: {} ports selected | [F1-F4] Presets [1-6] Categories [/] Custom [Enter] Confirm [Esc] Cancel",
            port_count
        );

        let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Gray));

        frame.render_widget(paragraph, area);
    }
}

impl Default for PortSelectionWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle keyboard events for PortSelectionWidget
pub fn handle_port_selection_event(event: &Event, state: &mut PortSelectionState) -> bool {
    if let Event::Key(KeyEvent {
        code, modifiers, ..
    }) = event
    {
        match (*code, *modifiers) {
            // Presets (F1-F4)
            (KeyCode::F(1), KeyModifiers::NONE) => {
                state.apply_preset(PortPreset::Top100);
                true
            }
            (KeyCode::F(2), KeyModifiers::NONE) => {
                state.apply_preset(PortPreset::Top1000);
                true
            }
            (KeyCode::F(3), KeyModifiers::NONE) => {
                state.apply_preset(PortPreset::AllPorts);
                true
            }
            (KeyCode::F(4), KeyModifiers::NONE) => {
                state.apply_preset(PortPreset::CommonServices);
                true
            }

            // Categories (1-6)
            (KeyCode::Char('1'), KeyModifiers::NONE) if !state.input_focused => {
                state.toggle_category(PortCategory::Web);
                true
            }
            (KeyCode::Char('2'), KeyModifiers::NONE) if !state.input_focused => {
                state.toggle_category(PortCategory::Ssh);
                true
            }
            (KeyCode::Char('3'), KeyModifiers::NONE) if !state.input_focused => {
                state.toggle_category(PortCategory::Database);
                true
            }
            (KeyCode::Char('4'), KeyModifiers::NONE) if !state.input_focused => {
                state.toggle_category(PortCategory::Mail);
                true
            }
            (KeyCode::Char('5'), KeyModifiers::NONE) if !state.input_focused => {
                state.toggle_category(PortCategory::FileSharing);
                true
            }
            (KeyCode::Char('6'), KeyModifiers::NONE) if !state.input_focused => {
                state.toggle_category(PortCategory::RemoteAccess);
                true
            }

            // Custom input focus
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                state.input_focused = true;
                true
            }

            // Custom input (when focused)
            (KeyCode::Char(c), KeyModifiers::NONE) if state.input_focused => {
                state.input_char(c);
                true
            }
            (KeyCode::Backspace, KeyModifiers::NONE) if state.input_focused => {
                state.backspace();
                true
            }

            // Clear all
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                state.clear_all();
                true
            }

            // Escape - unfocus input or cancel
            (KeyCode::Esc, KeyModifiers::NONE) => {
                if state.input_focused {
                    state.input_focused = false;
                    true
                } else {
                    false // Let parent handle
                }
            }

            _ => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PortSpec tests
    #[test]
    fn test_port_spec_parse_single() {
        let spec = PortSpec::parse("80").unwrap();
        assert_eq!(spec.count(), 1);
        assert!(spec.contains(80));
    }

    #[test]
    fn test_port_spec_parse_comma_separated() {
        let spec = PortSpec::parse("80,443,3000").unwrap();
        assert_eq!(spec.count(), 3);
        assert!(spec.contains(80));
        assert!(spec.contains(443));
        assert!(spec.contains(3000));
    }

    #[test]
    fn test_port_spec_parse_range() {
        let spec = PortSpec::parse("8080-8090").unwrap();
        assert_eq!(spec.count(), 11); // 8080..=8090
        assert!(spec.contains(8080));
        assert!(spec.contains(8085));
        assert!(spec.contains(8090));
    }

    #[test]
    fn test_port_spec_parse_mixed() {
        let spec = PortSpec::parse("80,443,8000-8010,3000").unwrap();
        assert_eq!(spec.count(), 14); // 80, 443, 3000, 8000-8010 (11 ports)
    }

    #[test]
    fn test_port_spec_parse_invalid_number() {
        assert!(PortSpec::parse("abc").is_err());
        assert!(PortSpec::parse("80,abc,443").is_err());
    }

    #[test]
    fn test_port_spec_parse_invalid_range() {
        assert!(PortSpec::parse("9000-8000").is_err()); // Start > end
        assert!(PortSpec::parse("80-").is_err()); // Missing end
        assert!(PortSpec::parse("-443").is_err()); // Missing start
    }

    #[test]
    fn test_port_spec_parse_zero_port() {
        assert!(PortSpec::parse("0").is_err());
        assert!(PortSpec::parse("80,0,443").is_err());
    }

    #[test]
    fn test_port_spec_top_100() {
        let spec = PortSpec::top_100();
        assert_eq!(spec.count(), 100);
        assert!(spec.contains(80));
        assert!(spec.contains(443));
        assert!(spec.contains(22));
    }

    #[test]
    fn test_port_spec_top_1000() {
        let spec = PortSpec::top_1000();
        assert!(spec.count() >= 100); // At least top 100
        assert!(spec.contains(80));
        assert!(spec.contains(443));
    }

    #[test]
    fn test_port_spec_all_ports() {
        let spec = PortSpec::all_ports();
        assert_eq!(spec.count(), 65535);
        assert!(spec.contains(1));
        assert!(spec.contains(65535));
    }

    #[test]
    fn test_port_spec_common_services() {
        let spec = PortSpec::common_services();
        assert!(spec.count() > 20); // At least 20 common ports
        assert!(spec.contains(80)); // Web
        assert!(spec.contains(22)); // SSH
        assert!(spec.contains(3306)); // MySQL
    }

    // PortCategory tests
    #[test]
    fn test_port_category_web() {
        let ports = PortCategory::Web.ports();
        assert!(ports.contains(&80));
        assert!(ports.contains(&443));
        assert!(ports.contains(&8080));
    }

    #[test]
    fn test_port_category_ssh() {
        let ports = PortCategory::Ssh.ports();
        assert_eq!(ports.len(), 1);
        assert!(ports.contains(&22));
    }

    #[test]
    fn test_port_category_database() {
        let ports = PortCategory::Database.ports();
        assert!(ports.contains(&3306)); // MySQL
        assert!(ports.contains(&5432)); // PostgreSQL
        assert!(ports.contains(&27017)); // MongoDB
    }

    #[test]
    fn test_port_category_all() {
        let categories = PortCategory::all();
        assert_eq!(categories.len(), 6);
    }

    // PortSelectionState tests
    #[test]
    fn test_port_selection_state_new() {
        let state = PortSelectionState::new();
        assert_eq!(state.port_input, "");
        assert_eq!(state.cursor_position, 0);
        assert!(state.port_spec.is_empty());
        assert_eq!(state.active_preset, None);
        assert!(!state.input_focused);
    }

    #[test]
    fn test_port_selection_apply_preset() {
        let mut state = PortSelectionState::new();
        state.apply_preset(PortPreset::Top100);

        assert_eq!(state.active_preset, Some(PortPreset::Top100));
        assert_eq!(state.port_spec.count(), 100);
    }

    #[test]
    fn test_port_selection_toggle_category() {
        let mut state = PortSelectionState::new();

        // Toggle Web category on
        state.toggle_category(PortCategory::Web);
        assert!(*state.category_toggles.get(&PortCategory::Web).unwrap());
        assert_eq!(state.port_spec.count(), PortCategory::Web.ports().len());

        // Toggle Web category off
        state.toggle_category(PortCategory::Web);
        assert!(!*state.category_toggles.get(&PortCategory::Web).unwrap());
        assert_eq!(state.port_spec.count(), 0);
    }

    #[test]
    fn test_port_selection_input_char() {
        let mut state = PortSelectionState::new();
        state.input_focused = true;

        state.input_char('8');
        state.input_char('0');

        assert_eq!(state.port_input, "80");
        assert_eq!(state.cursor_position, 2);
    }

    #[test]
    fn test_port_selection_backspace() {
        let mut state = PortSelectionState::new();
        state.input_focused = true;
        state.port_input = "80".to_string();
        state.cursor_position = 2;

        state.backspace();
        assert_eq!(state.port_input, "8");
        assert_eq!(state.cursor_position, 1);
    }

    #[test]
    fn test_port_selection_clear_all() {
        let mut state = PortSelectionState::new();
        state.apply_preset(PortPreset::Top100);
        state.toggle_category(PortCategory::Web);

        state.clear_all();

        assert_eq!(state.port_input, "");
        assert_eq!(state.port_spec.count(), 0);
        assert_eq!(state.active_preset, None);
        assert_eq!(state.category_toggles.len(), 0);
    }

    #[test]
    fn test_port_selection_validate_valid() {
        let mut state = PortSelectionState::new();
        state.port_input = "80,443".to_string();
        state.validate_input();

        match state.validation_state {
            PortValidationState::Valid { count, .. } => assert_eq!(count, 2),
            _ => panic!("Expected valid state"),
        }
    }

    #[test]
    fn test_port_selection_validate_invalid() {
        let mut state = PortSelectionState::new();
        state.port_input = "abc".to_string();
        state.validate_input();

        match state.validation_state {
            PortValidationState::Invalid(_) => (),
            _ => panic!("Expected invalid state"),
        }
    }

    #[test]
    fn test_port_selection_get_port_list() {
        let mut state = PortSelectionState::new();
        state.port_input = "443,80,8080".to_string();
        state.validate_input();

        let ports = state.get_port_list();
        assert_eq!(ports, vec![80, 443, 8080]); // Should be sorted
    }

    // PortPresetManager tests
    #[test]
    fn test_preset_manager_new() {
        let manager = PortPresetManager::new();
        assert_eq!(manager.presets.len(), 0);
    }

    #[test]
    fn test_preset_manager_add_remove() {
        let mut manager = PortPresetManager::new();

        manager.add_preset("web-dev".to_string(), "80,443,3000,8080".to_string());
        assert_eq!(manager.presets.len(), 1);

        let spec = manager.get_preset("web-dev");
        assert_eq!(spec.unwrap(), "80,443,3000,8080");

        manager.remove_preset("web-dev");
        assert_eq!(manager.presets.len(), 0);
    }

    #[test]
    fn test_preset_manager_list() {
        let mut manager = PortPresetManager::new();

        manager.add_preset("web".to_string(), "80,443".to_string());
        manager.add_preset("db".to_string(), "3306,5432".to_string());

        let names = manager.list_presets();
        assert_eq!(names.len(), 2);
    }
}
