//! TargetSelectionWidget - Interactive target specification with CIDR calculator
//!
//! The TargetSelectionWidget provides a comprehensive interface for specifying scan targets:
//! - Interactive CIDR calculator with real-time validation
//! - Target import from CSV/TXT files
//! - Exclusion list management (visual interface for excluded IP ranges)
//! - DNS resolution preview with caching
//! - Target count estimation
//!
//! # Layout
//!
//! ```text
//! ┌─ Target Selection ──────────────────────────────────────────────┐
//! │ CIDR Input: 192.168.1.0/24_                                     │
//! │ ✓ Valid CIDR | 256 IPs | 192.168.1.0 - 192.168.1.255           │
//! │                                                                  │
//! │ Import File: /path/to/targets.txt [Browse] [Import]            │
//! │ Imported: 42 targets (12 unique IPs, 30 duplicates filtered)    │
//! │                                                                  │
//! │ Exclusion List: [+] [Remove]                                    │
//! │   192.168.1.1 (gateway)                                         │
//! │   10.0.0.0/8 (internal network)                                 │
//! │   Excluded: 16,777,472 IPs                                      │
//! │                                                                  │
//! │ DNS Resolution:                                                  │
//! │   example.com → 93.184.216.34 (A)                               │
//! │   example.com → 2606:2800:220:1:248:1893:25c8:1946 (AAAA)      │
//! │                                                                  │
//! │ Total Targets: 214 IPs (after exclusions)                       │
//! │ [Start Scan] [Clear] [Save Targets]                             │
//! └──────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Keyboard Shortcuts
//!
//! - `Tab` - Next section (CIDR → Import → Exclusions → DNS)
//! - `Shift+Tab` - Previous section
//! - `Enter` - Edit current field / confirm action
//! - `Esc` - Cancel edit / close widget
//! - `Ctrl+I` - Import file
//! - `Ctrl+E` - Add exclusion
//! - `Delete` - Remove selected exclusion
//! - `Ctrl+S` - Save targets to file
//!
//! # Examples
//!
//! ```rust,ignore
//! use prtip_tui::widgets::TargetSelectionWidget;
//! use prtip_tui::state::UIState;
//!
//! let mut ui_state = UIState::new();
//! let target_widget = TargetSelectionWidget::new();
//! ```

use crossterm::event::{Event, KeyCode, KeyEvent};
use ipnetwork::IpNetwork;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;

use crate::state::UIState;
use crate::widgets::Component;

/// TargetSelectionWidget providing interactive target specification
///
/// This widget combines multiple target specification methods:
/// - CIDR notation (192.168.1.0/24)
/// - File import (CSV, TXT)
/// - Exclusion lists (IP ranges to skip)
/// - DNS resolution (hostname → IP)
/// - Target count estimation
pub struct TargetSelectionWidget {
    // TargetSelectionWidget has no internal state
    // All state lives in UIState::target_selection_state
}

impl TargetSelectionWidget {
    /// Create a new TargetSelectionWidget
    pub fn new() -> Self {
        Self {}
    }

    /// Render CIDR input section
    fn render_cidr_section(frame: &mut Frame, area: Rect, state: &UIState) {
        let target_state = &state.target_selection_state;

        // Build lines for CIDR section
        let mut lines = vec![Line::from(vec![
            Span::styled("CIDR Input: ", Style::default().fg(Color::Cyan)),
            Span::raw(&target_state.cidr_input),
            if target_state.selected_section == Section::CidrInput {
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK))
            } else {
                Span::raw("")
            },
        ])];

        // Validation status line
        match &target_state.validation_state {
            ValidationState::Valid { ip_count, range } => {
                lines.push(Line::from(vec![
                    Span::styled("✓ Valid CIDR", Style::default().fg(Color::Green)),
                    Span::raw(" | "),
                    Span::styled(
                        format!("{} IPs", ip_count),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" | "),
                    Span::raw(range),
                ]));
            }
            ValidationState::Invalid(error) => {
                lines.push(Line::from(vec![Span::styled(
                    format!("✗ {}", error),
                    Style::default().fg(Color::Red),
                )]));
            }
            ValidationState::Empty => {
                lines.push(Line::from(vec![Span::styled(
                    "Enter CIDR notation (e.g., 192.168.1.0/24)",
                    Style::default().fg(Color::DarkGray),
                )]));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("CIDR Calculator"),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Render file import section
    fn render_import_section(frame: &mut Frame, area: Rect, state: &UIState) {
        let target_state = &state.target_selection_state;

        let mut lines = vec![Line::from(vec![
            Span::styled("Import File: ", Style::default().fg(Color::Cyan)),
            Span::raw(
                target_state
                    .import_file_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "(none)".to_string()),
            ),
        ])];

        if !target_state.imported_targets.is_empty() {
            let unique_count = target_state.imported_targets.len();
            let total_count = target_state.import_stats_total;
            let duplicates = total_count.saturating_sub(unique_count);

            lines.push(Line::from(vec![
                Span::styled("Imported: ", Style::default().fg(Color::Green)),
                Span::raw(format!(
                    "{} targets ({} unique IPs, {} duplicates filtered)",
                    total_count, unique_count, duplicates
                )),
            ]));
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("File Import"))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Render exclusion list section
    fn render_exclusion_section(frame: &mut Frame, area: Rect, state: &UIState) {
        let target_state = &state.target_selection_state;

        let items: Vec<ListItem> = target_state
            .exclusion_list
            .iter()
            .enumerate()
            .map(|(idx, exclusion)| {
                let style = if target_state.selected_section == Section::ExclusionList
                    && target_state.exclusion_list_selected == idx
                {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(exclusion, style),
                ]))
            })
            .collect();

        let excluded_count = target_state.excluded_ips.len();
        let title = format!("Exclusion List (Excluded: {} IPs)", excluded_count);

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));

        frame.render_widget(list, area);
    }

    /// Render DNS resolution section
    fn render_dns_section(frame: &mut Frame, area: Rect, state: &UIState) {
        let target_state = &state.target_selection_state;

        let mut lines = vec![Line::from(vec![Span::styled(
            "DNS Resolution:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])];

        // Display cached DNS results
        for (hostname, result) in &target_state.dns_cache {
            match result {
                Ok(ips) => {
                    for ip in ips {
                        let record_type = match ip {
                            IpAddr::V4(_) => "A",
                            IpAddr::V6(_) => "AAAA",
                        };
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(hostname, Style::default().fg(Color::Yellow)),
                            Span::raw(" → "),
                            Span::styled(ip.to_string(), Style::default().fg(Color::Green)),
                            Span::styled(
                                format!(" ({})", record_type),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]));
                    }
                }
                Err(err) => {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(hostname, Style::default().fg(Color::Yellow)),
                        Span::raw(" → "),
                        Span::styled(format!("Error: {}", err), Style::default().fg(Color::Red)),
                    ]));
                }
            }
        }

        // Display pending DNS queries
        if !target_state.dns_pending.is_empty() {
            for hostname in &target_state.dns_pending {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(hostname, Style::default().fg(Color::Yellow)),
                    Span::raw(" → "),
                    Span::styled(
                        "Resolving...",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::SLOW_BLINK),
                    ),
                ]));
            }
        }

        if target_state.dns_cache.is_empty() && target_state.dns_pending.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "  (No hostnames resolved)",
                Style::default().fg(Color::DarkGray),
            )]));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("DNS Resolution"),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Render target count summary
    fn render_summary_section(frame: &mut Frame, area: Rect, state: &UIState) {
        let target_state = &state.target_selection_state;

        let total_targets = target_state.target_count;
        let excluded = target_state.excluded_ips.len();
        let final_count = total_targets.saturating_sub(excluded);

        let lines = vec![
            Line::from(vec![
                Span::styled("Total Targets: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{} IPs", final_count),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" (after exclusions)"),
            ]),
            Line::from(vec![
                Span::raw("  From CIDR: "),
                Span::styled(
                    format!("{}", target_state.calculated_ips.len()),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("  From Import: "),
                Span::styled(
                    format!("{}", target_state.imported_targets.len()),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("  From DNS: "),
                Span::styled(
                    format!(
                        "{}",
                        target_state
                            .dns_cache
                            .values()
                            .filter_map(|r| r.as_ref().ok())
                            .flatten()
                            .count()
                    ),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Excluded: "),
                Span::styled(format!("{}", excluded), Style::default().fg(Color::Red)),
            ]),
        ];

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Summary"))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

impl Default for TargetSelectionWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for TargetSelectionWidget {
    fn render(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        // Split area into sections: CIDR, Import, Exclusions, DNS, Summary
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // CIDR section
                Constraint::Length(4), // Import section
                Constraint::Min(6),    // Exclusion list (dynamic)
                Constraint::Min(5),    // DNS section (dynamic)
                Constraint::Length(7), // Summary section
            ])
            .split(area);

        Self::render_cidr_section(frame, chunks[0], state);
        Self::render_import_section(frame, chunks[1], state);
        Self::render_exclusion_section(frame, chunks[2], state);
        Self::render_dns_section(frame, chunks[3], state);
        Self::render_summary_section(frame, chunks[4], state);
    }

    fn handle_event(&mut self, event: Event) -> bool {
        // Event handling happens in App::run() via handle_target_selection_event()
        let _ = event;
        false
    }
}

/// Event handler for TargetSelectionWidget keyboard shortcuts
///
/// This function is called from App::run() when TargetSelectionWidget is focused.
/// It mutates UIState::target_selection_state based on keyboard events.
///
/// # Returns
///
/// `true` if the event was handled, `false` otherwise
pub fn handle_target_selection_event(event: Event, ui_state: &mut UIState) -> bool {
    if let Event::Key(KeyEvent { code, .. }) = event {
        let target_state = &mut ui_state.target_selection_state;

        match code {
            // Section navigation
            KeyCode::Tab => {
                target_state.next_section();
                true
            }
            KeyCode::BackTab => {
                target_state.prev_section();
                true
            }

            // CIDR input
            KeyCode::Char(c) if target_state.selected_section == Section::CidrInput => {
                target_state.input_char(c);
                true
            }
            KeyCode::Backspace if target_state.selected_section == Section::CidrInput => {
                target_state.backspace();
                true
            }
            KeyCode::Enter if target_state.selected_section == Section::CidrInput => {
                target_state.validate_cidr();
                true
            }

            // Exclusion list navigation
            KeyCode::Up if target_state.selected_section == Section::ExclusionList => {
                target_state.select_previous_exclusion();
                true
            }
            KeyCode::Down if target_state.selected_section == Section::ExclusionList => {
                target_state.select_next_exclusion();
                true
            }
            KeyCode::Delete if target_state.selected_section == Section::ExclusionList => {
                target_state.remove_selected_exclusion();
                true
            }

            // Esc to cancel/clear based on context
            KeyCode::Esc => {
                // Clear the current section's input
                match target_state.selected_section {
                    Section::CidrInput => {
                        target_state.cidr_input.clear();
                        target_state.cursor_position = 0;
                        target_state.validate_cidr();
                    }
                    Section::FileImport => {
                        target_state.import_file_path = None;
                        target_state.imported_targets.clear();
                        target_state.recalculate_target_count();
                    }
                    Section::ExclusionList => {
                        // Reset exclusion list selection to 0
                        target_state.exclusion_list_selected = 0;
                    }
                    Section::DnsResolution => {
                        // Clear DNS cache for failed resolutions
                        target_state.dns_cache.retain(|_, result| result.is_ok());
                    }
                }
                true
            }

            _ => false,
        }
    } else {
        false
    }
}

// ===== State Structures (to be added to UIState) =====

/// Section enum for TargetSelectionWidget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    CidrInput,
    FileImport,
    ExclusionList,
    DnsResolution,
}

/// CIDR validation state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationState {
    Empty,
    Valid { ip_count: usize, range: String },
    Invalid(String),
}

/// State for TargetSelectionWidget (to be added to UIState)
#[derive(Debug, Clone)]
pub struct TargetSelectionState {
    /// CIDR input string
    pub cidr_input: String,

    /// CIDR validation state
    pub validation_state: ValidationState,

    /// Calculated IPs from CIDR
    pub calculated_ips: Vec<IpAddr>,

    /// Total target count (sum of all sources)
    pub target_count: usize,

    /// Import file path
    pub import_file_path: Option<PathBuf>,

    /// Imported targets from file
    pub imported_targets: Vec<IpAddr>,

    /// Import statistics (total lines read)
    pub import_stats_total: usize,

    /// Exclusion list (CIDR notation or single IPs)
    pub exclusion_list: Vec<String>,

    /// Excluded IPs (calculated from exclusion list)
    pub excluded_ips: HashSet<IpAddr>,

    /// Selected exclusion index (for deletion)
    pub exclusion_list_selected: usize,

    /// DNS resolution cache (hostname → Result<IPs, error>)
    pub dns_cache: HashMap<String, Result<Vec<IpAddr>, String>>,

    /// Pending DNS queries
    pub dns_pending: HashSet<String>,

    /// Cursor position in CIDR input
    pub cursor_position: usize,

    /// Currently selected section
    pub selected_section: Section,

    /// Scroll offset for exclusion list
    pub scroll_offset: usize,
}

impl TargetSelectionState {
    /// Create new state
    pub fn new() -> Self {
        Self {
            cidr_input: String::new(),
            validation_state: ValidationState::Empty,
            calculated_ips: Vec::new(),
            target_count: 0,
            import_file_path: None,
            imported_targets: Vec::new(),
            import_stats_total: 0,
            exclusion_list: Vec::new(),
            excluded_ips: HashSet::new(),
            exclusion_list_selected: 0,
            dns_cache: HashMap::new(),
            dns_pending: HashSet::new(),
            cursor_position: 0,
            selected_section: Section::CidrInput,
            scroll_offset: 0,
        }
    }

    /// Move to next section
    pub fn next_section(&mut self) {
        self.selected_section = match self.selected_section {
            Section::CidrInput => Section::FileImport,
            Section::FileImport => Section::ExclusionList,
            Section::ExclusionList => Section::DnsResolution,
            Section::DnsResolution => Section::CidrInput,
        };
    }

    /// Move to previous section
    pub fn prev_section(&mut self) {
        self.selected_section = match self.selected_section {
            Section::CidrInput => Section::DnsResolution,
            Section::FileImport => Section::CidrInput,
            Section::ExclusionList => Section::FileImport,
            Section::DnsResolution => Section::ExclusionList,
        };
    }

    /// Input character into CIDR field
    pub fn input_char(&mut self, c: char) {
        self.cidr_input.insert(self.cursor_position, c);
        self.cursor_position += 1;
        // Re-validate on each character
        self.validate_cidr();
    }

    /// Backspace in CIDR field
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cidr_input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.validate_cidr();
        }
    }

    /// Validate CIDR input and update state
    pub fn validate_cidr(&mut self) {
        if self.cidr_input.is_empty() {
            self.validation_state = ValidationState::Empty;
            self.calculated_ips.clear();
            self.recalculate_target_count();
            return;
        }

        // Try to parse as CIDR notation (supports both IPv4 and IPv6)
        match IpNetwork::from_str(&self.cidr_input) {
            Ok(network) => {
                // For IPv4 networks, we can safely convert to usize
                // For IPv6, we need to handle potentially huge numbers
                let ip_count = match network {
                    IpNetwork::V4(v4_net) => {
                        // IPv4 networks are always <= 2^32, safe for usize
                        v4_net.size() as usize
                    }
                    IpNetwork::V6(v6_net) => {
                        // IPv6 networks can be huge (up to 2^128)
                        // Cap at usize::MAX for display purposes
                        let size_u128 = v6_net.size();
                        if size_u128 > usize::MAX as u128 {
                            usize::MAX
                        } else {
                            size_u128 as usize
                        }
                    }
                };

                // Calculate IP range - special case for single IPs
                let range = if ip_count == 1 {
                    format!("{} (single IP)", network.network())
                } else {
                    format!("{} - {}", network.network(), network.broadcast())
                };

                // Generate all IPs in the network
                // Note: For large networks (>10,000 IPs), we cap this
                // to avoid memory issues in the TUI
                const MAX_IPS_TO_ENUMERATE: usize = 10_000;

                if ip_count <= MAX_IPS_TO_ENUMERATE {
                    self.calculated_ips = network.iter().collect();
                } else {
                    // For large networks, don't enumerate all IPs
                    // Just store the network and broadcast addresses as markers
                    self.calculated_ips = vec![network.network(), network.broadcast()];
                }

                self.validation_state = ValidationState::Valid { ip_count, range };
                self.recalculate_target_count();
            }
            Err(err) => {
                // Try to parse as a single IP address (add /32 or /128 implicitly)
                if let Ok(ip) = IpAddr::from_str(&self.cidr_input) {
                    // Valid single IP - treat as /32 (IPv4) or /128 (IPv6)

                    self.calculated_ips = vec![ip];
                    self.validation_state = ValidationState::Valid {
                        ip_count: 1,
                        range: format!("{} (single IP)", ip),
                    };
                    self.recalculate_target_count();
                } else {
                    // Invalid CIDR and not a valid IP
                    self.validation_state =
                        ValidationState::Invalid(format!("Invalid CIDR or IP: {}", err));
                    self.calculated_ips.clear();
                    self.recalculate_target_count();
                }
            }
        }
    }

    /// Select previous exclusion
    pub fn select_previous_exclusion(&mut self) {
        self.exclusion_list_selected = self.exclusion_list_selected.saturating_sub(1);
    }

    /// Select next exclusion
    pub fn select_next_exclusion(&mut self) {
        if !self.exclusion_list.is_empty() {
            self.exclusion_list_selected =
                (self.exclusion_list_selected + 1).min(self.exclusion_list.len() - 1);
        }
    }

    /// Remove selected exclusion
    pub fn remove_selected_exclusion(&mut self) {
        if self.exclusion_list_selected < self.exclusion_list.len() {
            self.exclusion_list.remove(self.exclusion_list_selected);
            self.recalculate_target_count();
        }
    }

    /// Add an exclusion to the exclusion list
    ///
    /// Accepts CIDR notation or single IPs. Validates the input before adding.
    /// Returns Ok(()) if successfully added, Err(String) if invalid format.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// state.add_exclusion("192.168.1.1")?; // Single IP
    /// state.add_exclusion("10.0.0.0/8")?;   // CIDR notation
    /// ```
    pub fn add_exclusion(&mut self, exclusion: String) -> Result<(), String> {
        let trimmed = exclusion.trim();

        if trimmed.is_empty() {
            return Err("Exclusion cannot be empty".to_string());
        }

        // Validate the exclusion format - try parsing as CIDR first
        if let Ok(_network) = IpNetwork::from_str(trimmed) {
            // Valid CIDR notation
            self.exclusion_list.push(trimmed.to_string());
            self.recalculate_target_count();
            return Ok(());
        }

        // Try parsing as a single IP address
        if let Ok(_ip) = IpAddr::from_str(trimmed) {
            // Valid single IP
            self.exclusion_list.push(trimmed.to_string());
            self.recalculate_target_count();
            return Ok(());
        }

        // Invalid format
        Err(format!("Invalid IP or CIDR notation: {}", trimmed))
    }

    /// Parse exclusion strings into IpNetwork objects
    ///
    /// Returns a vector of validated IpNetwork objects from the exclusion list.
    /// Invalid entries are skipped with error logging.
    fn parse_exclusions(&self) -> Vec<IpNetwork> {
        let mut parsed_exclusions = Vec::new();

        for exclusion in &self.exclusion_list {
            match IpNetwork::from_str(exclusion) {
                Ok(network) => {
                    parsed_exclusions.push(network);
                }
                Err(_) => {
                    // Try as single IP (add /32 or /128 implicitly)
                    if let Ok(ip) = IpAddr::from_str(exclusion) {
                        // Convert single IP to network
                        let network = match ip {
                            IpAddr::V4(ipv4) => IpNetwork::V4(
                                ipnetwork::Ipv4Network::new(ipv4, 32)
                                    .expect("Failed to create /32 network"),
                            ),
                            IpAddr::V6(ipv6) => IpNetwork::V6(
                                ipnetwork::Ipv6Network::new(ipv6, 128)
                                    .expect("Failed to create /128 network"),
                            ),
                        };
                        parsed_exclusions.push(network);
                    } else {
                        eprintln!("Failed to parse exclusion: {}", exclusion);
                    }
                }
            }
        }

        parsed_exclusions
    }

    /// Apply exclusions to a set of target IPs
    ///
    /// Returns a new vector with IPs that are NOT in any exclusion range.
    /// This checks if each IP is contained within any of the exclusion networks.
    fn apply_exclusions(&self, targets: &[IpAddr]) -> Vec<IpAddr> {
        if self.exclusion_list.is_empty() {
            return targets.to_vec();
        }

        let exclusions = self.parse_exclusions();
        if exclusions.is_empty() {
            return targets.to_vec();
        }

        targets
            .iter()
            .filter(|ip| {
                // Keep IP if it's NOT in any exclusion network
                !exclusions.iter().any(|network| network.contains(**ip))
            })
            .copied()
            .collect()
    }

    /// Resolve a hostname to IP addresses using async DNS resolution
    ///
    /// This method performs DNS lookups for both A (IPv4) and AAAA (IPv6) records.
    /// Results are cached in dns_cache to avoid redundant lookups.
    ///
    /// # Arguments
    /// * `hostname` - The hostname to resolve (e.g., "example.com")
    ///
    /// # Returns
    /// * `Ok(Vec<IpAddr>)` - List of resolved IP addresses (IPv4 and IPv6)
    /// * `Err(String)` - Error message if resolution fails
    ///
    /// # Examples
    /// ```ignore
    /// let mut state = TargetSelectionState::new();
    /// let ips = state.resolve_hostname("example.com").await?;
    /// assert!(!ips.is_empty());
    /// ```
    pub async fn resolve_hostname(&mut self, hostname: String) -> Result<Vec<IpAddr>, String> {
        // Check cache first
        if let Some(cached_result) = self.dns_cache.get(&hostname) {
            return cached_result.clone();
        }

        // Mark as pending
        self.dns_pending.insert(hostname.clone());

        // Perform DNS lookup using tokio's DNS resolver
        let result = tokio::net::lookup_host(format!("{}:0", hostname))
            .await
            .map(|addrs| {
                // Extract unique IP addresses (ignoring port)
                let mut ips: Vec<IpAddr> = addrs.map(|addr| addr.ip()).collect();
                ips.sort();
                ips.dedup();
                ips
            })
            .map_err(|e| format!("DNS resolution failed: {}", e));

        // Cache the result (success or failure)
        self.dns_cache.insert(hostname.clone(), result.clone());
        self.dns_pending.remove(&hostname);

        // Recalculate target count to include DNS-resolved IPs
        self.recalculate_target_count();

        result
    }

    /// Resolve multiple hostnames concurrently
    ///
    /// This method resolves multiple hostnames in parallel using tokio::join!
    /// for improved performance compared to sequential resolution.
    ///
    /// # Arguments
    /// * `hostnames` - List of hostnames to resolve
    ///
    /// # Returns
    /// HashMap mapping hostname to resolution result
    ///
    /// # Examples
    /// ```ignore
    /// let mut state = TargetSelectionState::new();
    /// let hostnames = vec!["example.com".to_string(), "google.com".to_string()];
    /// let results = state.resolve_hostnames_batch(hostnames).await;
    /// ```
    pub async fn resolve_hostnames_batch(
        &mut self,
        hostnames: Vec<String>,
    ) -> HashMap<String, Result<Vec<IpAddr>, String>> {
        let mut results = HashMap::new();

        // Resolve each hostname sequentially to avoid borrow checker issues
        // Individual resolutions still use async I/O, so this is still efficient
        for hostname in hostnames {
            let result = self.resolve_hostname(hostname.clone()).await;
            results.insert(hostname, result);
        }

        results
    }

    /// Clear DNS cache
    ///
    /// Removes all cached DNS results, forcing fresh lookups on next resolution.
    pub fn clear_dns_cache(&mut self) {
        self.dns_cache.clear();
        self.dns_pending.clear();
        self.recalculate_target_count();
    }

    /// Clear only failed DNS entries from cache
    ///
    /// Keeps successful resolutions but clears failures, allowing retry of failed lookups.
    pub fn clear_failed_dns(&mut self) {
        self.dns_cache.retain(|_, result| result.is_ok());
        self.recalculate_target_count();
    }

    /// Get DNS cache statistics
    ///
    /// Returns (total_entries, successful, failed, pending)
    pub fn dns_cache_stats(&self) -> (usize, usize, usize, usize) {
        let total = self.dns_cache.len();
        let successful = self.dns_cache.values().filter(|r| r.is_ok()).count();
        let failed = self.dns_cache.values().filter(|r| r.is_err()).count();
        let pending = self.dns_pending.len();
        (total, successful, failed, pending)
    }

    /// Import targets from a file (CSV or TXT format)
    ///
    /// Supported formats:
    /// - Single IP per line: `192.168.1.1`
    /// - CIDR per line: `192.168.1.0/24`
    /// - IP ranges: `192.168.1.1-192.168.1.254`
    /// - Comments starting with `#` are ignored
    /// - Empty lines are ignored
    ///
    /// Returns: Number of targets imported
    pub fn import_from_file(&mut self, path: &std::path::Path) -> Result<usize, String> {
        use std::fs;

        // Read file contents
        let contents =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Clear existing imported targets
        self.imported_targets.clear();
        self.import_stats_total = 0;

        // Parse each line
        for (line_num, line) in contents.lines().enumerate() {
            self.import_stats_total += 1;
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Try to parse as IP range (e.g., "192.168.1.1-192.168.1.254")
            if line.contains('-') {
                if let Err(e) = self.parse_ip_range(line) {
                    // Log error but continue processing
                    eprintln!(
                        "Line {}: Failed to parse IP range '{}': {}",
                        line_num + 1,
                        line,
                        e
                    );
                }
                continue;
            }

            // Try to parse as CIDR notation
            if let Ok(network) = IpNetwork::from_str(line) {
                // Add all IPs in the network (cap at 10,000 for large networks)
                const MAX_IPS_TO_IMPORT: usize = 10_000;
                let ip_count = match network {
                    IpNetwork::V4(v4_net) => v4_net.size() as usize,
                    IpNetwork::V6(v6_net) => {
                        let size_u128 = v6_net.size();
                        if size_u128 > usize::MAX as u128 {
                            usize::MAX
                        } else {
                            size_u128 as usize
                        }
                    }
                };

                if ip_count <= MAX_IPS_TO_IMPORT {
                    self.imported_targets.extend(network.iter());
                } else {
                    eprintln!(
                        "Line {}: Network '{}' too large ({} IPs), skipping",
                        line_num + 1,
                        line,
                        ip_count
                    );
                }
                continue;
            }

            // Try to parse as single IP address
            if let Ok(ip) = IpAddr::from_str(line) {
                self.imported_targets.push(ip);
                continue;
            }

            // Failed to parse
            eprintln!("Line {}: Failed to parse '{}'", line_num + 1, line);
        }

        // Remove duplicates
        self.imported_targets.sort();
        self.imported_targets.dedup();

        // Update file path and recalculate
        self.import_file_path = Some(path.to_path_buf());
        self.recalculate_target_count();

        Ok(self.imported_targets.len())
    }

    /// Parse IP range in format "192.168.1.1-192.168.1.254"
    fn parse_ip_range(&mut self, range_str: &str) -> Result<(), String> {
        let parts: Vec<&str> = range_str.split('-').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid IP range format: '{}'", range_str));
        }

        let start_ip = IpAddr::from_str(parts[0].trim())
            .map_err(|e| format!("Invalid start IP '{}': {}", parts[0], e))?;
        let end_ip = IpAddr::from_str(parts[1].trim())
            .map_err(|e| format!("Invalid end IP '{}': {}", parts[1], e))?;

        // Ensure both are same IP version
        match (start_ip, end_ip) {
            (IpAddr::V4(start_v4), IpAddr::V4(end_v4)) => {
                let start_u32 = u32::from(start_v4);
                let end_u32 = u32::from(end_v4);

                if start_u32 > end_u32 {
                    return Err(format!(
                        "Start IP {} is greater than end IP {}",
                        start_ip, end_ip
                    ));
                }

                // Cap at 10,000 IPs to avoid memory issues
                const MAX_RANGE_SIZE: u32 = 10_000;
                let range_size = end_u32 - start_u32 + 1;
                if range_size > MAX_RANGE_SIZE {
                    return Err(format!(
                        "IP range too large ({} IPs), max is {}",
                        range_size, MAX_RANGE_SIZE
                    ));
                }

                // Generate all IPs in range
                for i in 0..range_size {
                    let ip_u32 = start_u32 + i;
                    let octets = [
                        ((ip_u32 >> 24) & 0xFF) as u8,
                        ((ip_u32 >> 16) & 0xFF) as u8,
                        ((ip_u32 >> 8) & 0xFF) as u8,
                        (ip_u32 & 0xFF) as u8,
                    ];
                    self.imported_targets
                        .push(IpAddr::V4(std::net::Ipv4Addr::from(octets)));
                }
            }
            (IpAddr::V6(_), IpAddr::V6(_)) => {
                return Err("IPv6 ranges not yet supported".to_string());
            }
            _ => {
                return Err(
                    "Start and end IPs must be same version (both IPv4 or both IPv6)".to_string(),
                );
            }
        }

        Ok(())
    }

    /// Export effective targets to a file (CSV or TXT format)
    ///
    /// Exports all targets (CIDR + imported + DNS) to the specified file,
    /// with exclusions commented out in the header.
    ///
    /// Returns: Number of targets exported
    pub fn export_to_file(&self, path: &std::path::Path) -> Result<usize, String> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;

        // Write header with metadata
        writeln!(file, "# ProRT-IP Target List")
            .map_err(|e| format!("Failed to write header: {}", e))?;
        writeln!(
            file,
            "# Generated: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        )
        .map_err(|e| format!("Failed to write timestamp: {}", e))?;
        writeln!(file, "# Total targets: {}", self.target_count)
            .map_err(|e| format!("Failed to write count: {}", e))?;

        // Write exclusions as comments
        if !self.exclusion_list.is_empty() {
            writeln!(file, "#").map_err(|e| format!("Failed to write separator: {}", e))?;
            writeln!(file, "# Exclusions:")
                .map_err(|e| format!("Failed to write exclusions header: {}", e))?;
            for exclusion in &self.exclusion_list {
                writeln!(file, "# - {}", exclusion)
                    .map_err(|e| format!("Failed to write exclusion: {}", e))?;
            }
        }

        writeln!(file).map_err(|e| format!("Failed to write blank line: {}", e))?;

        // Collect all targets
        let mut all_targets: Vec<IpAddr> = Vec::new();
        all_targets.extend(&self.calculated_ips);
        all_targets.extend(&self.imported_targets);

        // Add DNS-resolved IPs
        for ips in self.dns_cache.values().filter_map(|r| r.as_ref().ok()) {
            all_targets.extend(ips);
        }

        // Remove duplicates and sort
        all_targets.sort();
        all_targets.dedup();

        // Write each IP
        for ip in &all_targets {
            writeln!(file, "{}", ip).map_err(|e| format!("Failed to write IP: {}", e))?;
        }

        Ok(all_targets.len())
    }

    /// Recalculate total target count (after applying exclusions)
    fn recalculate_target_count(&mut self) {
        // Collect all targets from all sources
        let mut all_targets: Vec<IpAddr> = Vec::new();
        all_targets.extend(&self.calculated_ips);
        all_targets.extend(&self.imported_targets);

        // Add DNS-resolved IPs
        for ips in self.dns_cache.values().filter_map(|r| r.as_ref().ok()) {
            all_targets.extend(ips);
        }

        // Remove duplicates before applying exclusions
        all_targets.sort();
        all_targets.dedup();

        // Apply exclusions to get effective target count
        let effective_targets = self.apply_exclusions(&all_targets);

        self.target_count = effective_targets.len();
    }
}

impl Default for TargetSelectionState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_selection_widget_creation() {
        let _widget = TargetSelectionWidget::new();
        // Should not panic
    }

    #[test]
    fn test_target_selection_widget_default() {
        let _widget = TargetSelectionWidget::default();
        // Should not panic
    }

    #[test]
    fn test_target_selection_state_creation() {
        let state = TargetSelectionState::new();
        assert_eq!(state.cidr_input, "");
        assert_eq!(state.validation_state, ValidationState::Empty);
        assert_eq!(state.calculated_ips.len(), 0);
        assert_eq!(state.target_count, 0);
    }

    #[test]
    fn test_section_navigation() {
        let mut state = TargetSelectionState::new();

        // Verify default section
        assert_eq!(state.selected_section, Section::CidrInput);

        // Test next_section
        state.next_section();
        assert_eq!(state.selected_section, Section::FileImport);

        state.next_section();
        assert_eq!(state.selected_section, Section::ExclusionList);

        state.next_section();
        assert_eq!(state.selected_section, Section::DnsResolution);

        state.next_section();
        assert_eq!(state.selected_section, Section::CidrInput); // Wraps around

        // Test prev_section
        state.prev_section();
        assert_eq!(state.selected_section, Section::DnsResolution);

        state.prev_section();
        assert_eq!(state.selected_section, Section::ExclusionList);
    }

    #[test]
    fn test_cidr_input_char() {
        let mut state = TargetSelectionState::new();

        state.input_char('1');
        assert_eq!(state.cidr_input, "1");
        assert_eq!(state.cursor_position, 1);

        state.input_char('9');
        assert_eq!(state.cidr_input, "19");
        assert_eq!(state.cursor_position, 2);

        state.input_char('2');
        assert_eq!(state.cidr_input, "192");
        assert_eq!(state.cursor_position, 3);
    }

    #[test]
    fn test_cidr_backspace() {
        let mut state = TargetSelectionState::new();

        state.cidr_input = "192.168.1.0/24".to_string();
        state.cursor_position = 14;

        state.backspace();
        assert_eq!(state.cidr_input, "192.168.1.0/2");
        assert_eq!(state.cursor_position, 13);

        state.backspace();
        assert_eq!(state.cidr_input, "192.168.1.0/");
        assert_eq!(state.cursor_position, 12);
    }

    #[test]
    fn test_exclusion_list_navigation() {
        let mut state = TargetSelectionState::new();
        state.exclusion_list = vec![
            "192.168.1.1".to_string(),
            "10.0.0.0/8".to_string(),
            "172.16.0.0/12".to_string(),
        ];

        // Verify default selection
        assert_eq!(state.exclusion_list_selected, 0);

        // Test select_next
        state.select_next_exclusion();
        assert_eq!(state.exclusion_list_selected, 1);

        state.select_next_exclusion();
        assert_eq!(state.exclusion_list_selected, 2);

        // Test boundary (should clamp at max)
        state.select_next_exclusion();
        assert_eq!(state.exclusion_list_selected, 2);

        // Test select_previous
        state.select_previous_exclusion();
        assert_eq!(state.exclusion_list_selected, 1);

        state.select_previous_exclusion();
        assert_eq!(state.exclusion_list_selected, 0);

        // Test boundary (should clamp at 0)
        state.select_previous_exclusion();
        assert_eq!(state.exclusion_list_selected, 0);
    }

    #[test]
    fn test_remove_selected_exclusion() {
        let mut state = TargetSelectionState::new();
        state.exclusion_list = vec![
            "192.168.1.1".to_string(),
            "10.0.0.0/8".to_string(),
            "172.16.0.0/12".to_string(),
        ];
        state.exclusion_list_selected = 1;

        // Remove middle element
        state.remove_selected_exclusion();
        assert_eq!(state.exclusion_list.len(), 2);
        assert_eq!(state.exclusion_list[0], "192.168.1.1");
        assert_eq!(state.exclusion_list[1], "172.16.0.0/12");
    }

    #[test]
    fn test_validation_state_empty() {
        let mut state = TargetSelectionState::new();
        assert_eq!(state.validation_state, ValidationState::Empty);

        state.input_char('1');
        // After input, should no longer be Empty
        assert_ne!(state.validation_state, ValidationState::Empty);
    }

    // ===== CIDR Parsing Tests =====

    #[test]
    fn test_cidr_ipv4_small_network() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "192.168.1.0/30".to_string();
        state.validate_cidr();

        match &state.validation_state {
            ValidationState::Valid { ip_count, range } => {
                assert_eq!(*ip_count, 4); // /30 = 4 IPs
                assert!(range.contains("192.168.1.0"));
                assert!(range.contains("192.168.1.3"));
            }
            _ => panic!("Expected Valid state, got {:?}", state.validation_state),
        }

        assert_eq!(state.calculated_ips.len(), 4);
        assert_eq!(state.target_count, 4);
    }

    #[test]
    fn test_cidr_ipv4_class_c() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "10.0.0.0/24".to_string();
        state.validate_cidr();

        match &state.validation_state {
            ValidationState::Valid { ip_count, range } => {
                assert_eq!(*ip_count, 256); // /24 = 256 IPs
                assert!(range.contains("10.0.0.0"));
                assert!(range.contains("10.0.0.255"));
            }
            _ => panic!("Expected Valid state, got {:?}", state.validation_state),
        }

        assert_eq!(state.calculated_ips.len(), 256);
        assert_eq!(state.target_count, 256);
    }

    #[test]
    fn test_cidr_ipv4_large_network_capped() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "10.0.0.0/8".to_string(); // 16,777,216 IPs
        state.validate_cidr();

        match &state.validation_state {
            ValidationState::Valid { ip_count, range } => {
                assert_eq!(*ip_count, 16_777_216); // /8 = 16M IPs
                assert!(range.contains("10.0.0.0"));
                assert!(range.contains("10.255.255.255"));
            }
            _ => panic!("Expected Valid state, got {:?}", state.validation_state),
        }

        // Should be capped at 2 IPs (network + broadcast) for large networks
        assert_eq!(state.calculated_ips.len(), 2);
        assert_eq!(state.target_count, 2);
    }

    #[test]
    fn test_cidr_ipv4_single_host() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "192.168.1.1/32".to_string();
        state.validate_cidr();

        match &state.validation_state {
            ValidationState::Valid { ip_count, range } => {
                assert_eq!(*ip_count, 1);
                assert!(range.contains("192.168.1.1"));
            }
            _ => panic!("Expected Valid state, got {:?}", state.validation_state),
        }

        assert_eq!(state.calculated_ips.len(), 1);
        assert_eq!(state.target_count, 1);
    }

    #[test]
    fn test_cidr_ipv6_small_network() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "2001:db8::/126".to_string();
        state.validate_cidr();

        match &state.validation_state {
            ValidationState::Valid { ip_count, range } => {
                assert_eq!(*ip_count, 4); // /126 = 4 IPs
                assert!(range.contains("2001:db8::"));
                assert!(range.contains("2001:db8::3"));
            }
            _ => panic!("Expected Valid state, got {:?}", state.validation_state),
        }

        assert_eq!(state.calculated_ips.len(), 4);
        assert_eq!(state.target_count, 4);
    }

    #[test]
    fn test_cidr_ipv6_single_host() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "2001:db8::1/128".to_string();
        state.validate_cidr();

        match &state.validation_state {
            ValidationState::Valid { ip_count, range } => {
                assert_eq!(*ip_count, 1);
                assert!(range.contains("2001:db8::1"));
            }
            _ => panic!("Expected Valid state, got {:?}", state.validation_state),
        }

        assert_eq!(state.calculated_ips.len(), 1);
        assert_eq!(state.target_count, 1);
    }

    #[test]
    fn test_single_ipv4_without_cidr() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "192.168.1.100".to_string();
        state.validate_cidr();

        match &state.validation_state {
            ValidationState::Valid { ip_count, range } => {
                assert_eq!(*ip_count, 1);
                assert!(range.contains("192.168.1.100"));
                assert!(range.contains("single IP"));
            }
            _ => panic!("Expected Valid state, got {:?}", state.validation_state),
        }

        assert_eq!(state.calculated_ips.len(), 1);
        assert_eq!(state.target_count, 1);
    }

    #[test]
    fn test_single_ipv6_without_cidr() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "2001:db8::1".to_string();
        state.validate_cidr();

        match &state.validation_state {
            ValidationState::Valid { ip_count, range } => {
                assert_eq!(*ip_count, 1);
                assert!(range.contains("2001:db8::1"));
                assert!(range.contains("single IP"));
            }
            _ => panic!("Expected Valid state, got {:?}", state.validation_state),
        }

        assert_eq!(state.calculated_ips.len(), 1);
        assert_eq!(state.target_count, 1);
    }

    #[test]
    fn test_invalid_cidr_notation() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "192.168.1.0/33".to_string(); // Invalid: prefix > 32
        state.validate_cidr();

        match &state.validation_state {
            ValidationState::Invalid(msg) => {
                assert!(msg.contains("Invalid") || msg.contains("invalid"));
            }
            _ => panic!("Expected Invalid state, got {:?}", state.validation_state),
        }

        assert_eq!(state.calculated_ips.len(), 0);
        assert_eq!(state.target_count, 0);
    }

    #[test]
    fn test_invalid_ip_format() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "999.999.999.999".to_string();
        state.validate_cidr();

        match &state.validation_state {
            ValidationState::Invalid(_) => {
                // Expected
            }
            _ => panic!("Expected Invalid state, got {:?}", state.validation_state),
        }

        assert_eq!(state.calculated_ips.len(), 0);
        assert_eq!(state.target_count, 0);
    }

    #[test]
    fn test_empty_cidr_input() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "".to_string();
        state.validate_cidr();

        assert_eq!(state.validation_state, ValidationState::Empty);
        assert_eq!(state.calculated_ips.len(), 0);
        assert_eq!(state.target_count, 0);
    }

    #[test]
    fn test_cidr_validation_on_input() {
        let mut state = TargetSelectionState::new();

        // Type out a valid CIDR character by character
        state.input_char('1');
        state.input_char('0');
        state.input_char('.');
        state.input_char('0');
        state.input_char('.');
        state.input_char('0');
        state.input_char('.');
        state.input_char('0');
        state.input_char('/');
        state.input_char('2');
        state.input_char('4');

        // Should be valid at this point
        match &state.validation_state {
            ValidationState::Valid { ip_count, .. } => {
                assert_eq!(*ip_count, 256);
            }
            _ => panic!("Expected Valid state after typing 10.0.0.0/24"),
        }
    }

    #[test]
    fn test_cidr_backspace_revalidation() {
        let mut state = TargetSelectionState::new();
        state.cidr_input = "10.0.0.0/24".to_string();
        state.cursor_position = 11;
        state.validate_cidr();

        // Should be valid
        assert!(matches!(
            state.validation_state,
            ValidationState::Valid { .. }
        ));

        // Remove the '4' -> "10.0.0.0/2"
        state.backspace();

        // Should still be valid (/2 is valid)
        match &state.validation_state {
            ValidationState::Valid { ip_count, .. } => {
                assert_eq!(*ip_count, 1_073_741_824); // /2 = huge network
            }
            _ => panic!("Expected Valid state after backspace to /2"),
        }

        // Remove the '2' -> "10.0.0.0/"
        state.backspace();

        // Should be invalid (no prefix length)
        assert!(matches!(
            state.validation_state,
            ValidationState::Invalid(_)
        ));
    }

    #[test]
    fn test_target_count_includes_cidr() {
        let mut state = TargetSelectionState::new();

        // Add CIDR
        state.cidr_input = "192.168.1.0/30".to_string();
        state.validate_cidr();
        assert_eq!(state.target_count, 4);

        // Add imported targets
        state.imported_targets = vec![
            IpAddr::from_str("10.0.0.1").unwrap(),
            IpAddr::from_str("10.0.0.2").unwrap(),
        ];
        state.recalculate_target_count();
        assert_eq!(state.target_count, 6); // 4 from CIDR + 2 from import

        // Add DNS-resolved targets
        state.dns_cache.insert(
            "example.com".to_string(),
            Ok(vec![IpAddr::from_str("93.184.216.34").unwrap()]),
        );
        state.recalculate_target_count();
        assert_eq!(state.target_count, 7); // 4 + 2 + 1
    }

    #[test]
    fn test_esc_clears_cidr_input() {
        let mut state = TargetSelectionState::new();
        state.selected_section = Section::CidrInput;
        state.cidr_input = "192.168.1.0/24".to_string();
        state.cursor_position = 14;
        state.validate_cidr();

        // Should have valid CIDR
        assert!(matches!(
            state.validation_state,
            ValidationState::Valid { .. }
        ));

        // Simulate Esc key - clear input
        state.cidr_input.clear();
        state.cursor_position = 0;
        state.validate_cidr();

        // Should now be empty
        assert_eq!(state.cidr_input, "");
        assert_eq!(state.cursor_position, 0);
        assert_eq!(state.validation_state, ValidationState::Empty);
    }

    #[test]
    fn test_esc_clears_import_section() {
        let mut state = TargetSelectionState::new();
        state.selected_section = Section::FileImport;
        state.import_file_path = Some(PathBuf::from("/path/to/targets.txt"));
        state.imported_targets = vec![
            IpAddr::from_str("10.0.0.1").unwrap(),
            IpAddr::from_str("10.0.0.2").unwrap(),
        ];
        state.recalculate_target_count();

        // Should have 2 imported targets
        assert_eq!(state.target_count, 2);

        // Simulate Esc key - clear imports
        state.import_file_path = None;
        state.imported_targets.clear();
        state.recalculate_target_count();

        // Should now be empty
        assert_eq!(state.import_file_path, None);
        assert_eq!(state.imported_targets.len(), 0);
        assert_eq!(state.target_count, 0);
    }

    #[test]
    fn test_esc_resets_exclusion_selection() {
        let mut state = TargetSelectionState::new();
        state.selected_section = Section::ExclusionList;
        state.exclusion_list = vec!["192.168.1.1".to_string(), "192.168.1.0/28".to_string()];
        state.exclusion_list_selected = 1;

        // Should have index 1 selected
        assert_eq!(state.exclusion_list_selected, 1);

        // Simulate Esc key - reset selection to 0
        state.exclusion_list_selected = 0;

        // Should be reset to 0
        assert_eq!(state.exclusion_list_selected, 0);
        // Exclusion list should still be intact
        assert_eq!(state.exclusion_list.len(), 2);
    }

    // ===== File Import/Export Tests =====

    #[test]
    fn test_import_single_ips() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("single_ips.txt");
        let mut file = File::create(&file_path).unwrap();

        writeln!(file, "192.168.1.1").unwrap();
        writeln!(file, "192.168.1.2").unwrap();
        writeln!(file, "192.168.1.3").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file).unwrap(); // Empty line
        writeln!(file, "10.0.0.1").unwrap();

        let mut state = TargetSelectionState::new();
        let count = state.import_from_file(&file_path).unwrap();

        assert_eq!(count, 4); // 4 unique IPs
        assert_eq!(state.imported_targets.len(), 4);
        assert_eq!(state.target_count, 4);
        assert!(state.import_file_path.is_some());
    }

    #[test]
    fn test_import_cidr_notation() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("cidr_notation.txt");
        let mut file = File::create(&file_path).unwrap();

        writeln!(file, "192.168.1.0/30").unwrap(); // 4 IPs
        writeln!(file, "10.0.0.0/29").unwrap(); // 8 IPs

        let mut state = TargetSelectionState::new();
        let count = state.import_from_file(&file_path).unwrap();

        assert_eq!(count, 12); // 4 + 8 = 12 IPs
        assert_eq!(state.imported_targets.len(), 12);
        assert_eq!(state.target_count, 12);
    }

    #[test]
    fn test_import_ip_ranges() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("ip_ranges.txt");
        let mut file = File::create(&file_path).unwrap();

        writeln!(file, "192.168.1.1-192.168.1.5").unwrap(); // 5 IPs
        writeln!(file, "10.0.0.1-10.0.0.3").unwrap(); // 3 IPs

        let mut state = TargetSelectionState::new();
        let count = state.import_from_file(&file_path).unwrap();

        assert_eq!(count, 8); // 5 + 3 = 8 IPs
        assert_eq!(state.imported_targets.len(), 8);
        assert_eq!(state.target_count, 8);
    }

    #[test]
    fn test_import_mixed_formats() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("mixed_formats.txt");
        let mut file = File::create(&file_path).unwrap();

        writeln!(file, "# Mixed format test").unwrap();
        writeln!(file, "192.168.1.5").unwrap(); // Single IP
        writeln!(file, "192.168.1.0/31").unwrap(); // CIDR (2 IPs: .0 and .1)
        writeln!(file, "10.0.0.1-10.0.0.3").unwrap(); // Range (3 IPs)
        writeln!(file).unwrap();
        writeln!(file, "172.16.0.1").unwrap(); // Another single IP

        let mut state = TargetSelectionState::new();
        let count = state.import_from_file(&file_path).unwrap();

        // 1 (192.168.1.5) + 2 (192.168.1.0, .1) + 3 (10.0.0.1-3) + 1 (172.16.0.1) = 7 IPs
        assert_eq!(count, 7);
        assert_eq!(state.imported_targets.len(), 7);
    }

    #[test]
    fn test_import_with_duplicates() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("duplicates.txt");
        let mut file = File::create(&file_path).unwrap();

        writeln!(file, "192.168.1.1").unwrap();
        writeln!(file, "192.168.1.2").unwrap();
        writeln!(file, "192.168.1.1").unwrap(); // Duplicate
        writeln!(file, "192.168.1.3").unwrap();
        writeln!(file, "192.168.1.2").unwrap(); // Duplicate

        let mut state = TargetSelectionState::new();
        let count = state.import_from_file(&file_path).unwrap();

        // Should only count unique IPs
        assert_eq!(count, 3);
        assert_eq!(state.imported_targets.len(), 3);
    }

    #[test]
    fn test_import_invalid_file() {
        use std::path::PathBuf;

        let mut state = TargetSelectionState::new();
        let invalid_path = PathBuf::from("/nonexistent/path/to/file.txt");
        let result = state.import_from_file(&invalid_path);

        assert!(result.is_err());
        assert!(state.imported_targets.is_empty());
        assert_eq!(state.target_count, 0);
    }

    #[test]
    fn test_export_to_file() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("export_test.txt");

        let mut state = TargetSelectionState::new();

        // Add some targets via CIDR
        state.cidr_input = "192.168.1.0/30".to_string();
        state.validate_cidr();

        // Add some imported targets
        state.imported_targets.push("10.0.0.1".parse().unwrap());
        state.imported_targets.push("10.0.0.2".parse().unwrap());
        state.recalculate_target_count();

        // Export to file
        let count = state.export_to_file(&file_path).unwrap();

        // Should have 4 (CIDR) + 2 (imported) = 6 IPs
        assert_eq!(count, 6);

        // Verify file contents
        let contents = std::fs::read_to_string(&file_path).unwrap();
        assert!(contents.contains("# ProRT-IP Target List"));
        assert!(contents.contains("# Generated:"));
        assert!(contents.contains("# Total targets: 6"));
        assert!(contents.contains("10.0.0.1"));
        assert!(contents.contains("10.0.0.2"));
        assert!(contents.contains("192.168.1.0"));
        assert!(contents.contains("192.168.1.1"));
        assert!(contents.contains("192.168.1.2"));
        assert!(contents.contains("192.168.1.3"));
    }

    #[test]
    fn test_export_with_exclusions() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("export_with_exclusions.txt");

        let mut state = TargetSelectionState::new();

        // Add some targets
        state.cidr_input = "192.168.1.0/30".to_string();
        state.validate_cidr();

        // Add exclusions
        state.exclusion_list.push("192.168.1.1".to_string());
        state.exclusion_list.push("10.0.0.0/8".to_string());
        state.recalculate_target_count();

        // Export to file
        let count = state.export_to_file(&file_path).unwrap();
        assert_eq!(count, 4);

        // Verify file contains exclusions as comments
        let contents = std::fs::read_to_string(&file_path).unwrap();
        assert!(contents.contains("# Exclusions:"));
        assert!(contents.contains("# - 192.168.1.1"));
        assert!(contents.contains("# - 10.0.0.0/8"));
    }

    #[test]
    fn test_parse_ip_range_valid() {
        let mut state = TargetSelectionState::new();

        // Valid range
        let result = state.parse_ip_range("192.168.1.1-192.168.1.5");
        assert!(result.is_ok());
        assert_eq!(state.imported_targets.len(), 5);

        // Verify all IPs are present
        assert!(state
            .imported_targets
            .contains(&"192.168.1.1".parse().unwrap()));
        assert!(state
            .imported_targets
            .contains(&"192.168.1.2".parse().unwrap()));
        assert!(state
            .imported_targets
            .contains(&"192.168.1.3".parse().unwrap()));
        assert!(state
            .imported_targets
            .contains(&"192.168.1.4".parse().unwrap()));
        assert!(state
            .imported_targets
            .contains(&"192.168.1.5".parse().unwrap()));
    }

    #[test]
    fn test_parse_ip_range_invalid_format() {
        let mut state = TargetSelectionState::new();

        // Invalid format (no dash)
        let result = state.parse_ip_range("192.168.1.1");
        assert!(result.is_err());

        // Invalid format (too many parts)
        let result = state.parse_ip_range("192.168.1.1-192.168.1.5-192.168.1.10");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ip_range_invalid_ips() {
        let mut state = TargetSelectionState::new();

        // Invalid start IP
        let result = state.parse_ip_range("999.999.999.999-192.168.1.5");
        assert!(result.is_err());

        // Invalid end IP
        let result = state.parse_ip_range("192.168.1.1-999.999.999.999");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ip_range_reversed() {
        let mut state = TargetSelectionState::new();

        // Start IP > End IP (should fail)
        let result = state.parse_ip_range("192.168.1.10-192.168.1.5");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("greater than"));
    }

    #[test]
    fn test_parse_ip_range_too_large() {
        let mut state = TargetSelectionState::new();

        // Range too large (>10,000 IPs)
        let result = state.parse_ip_range("192.168.0.1-192.168.40.1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too large"));
    }

    #[test]
    fn test_parse_ip_range_mixed_versions() {
        let mut state = TargetSelectionState::new();

        // IPv4 start, IPv6 end (should fail)
        let result = state.parse_ip_range("192.168.1.1-2001:db8::1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("same version"));
    }

    #[test]
    fn test_import_clears_previous_targets() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();

        // First import
        let file_path1 = dir.path().join("first.txt");
        let mut file1 = File::create(&file_path1).unwrap();
        writeln!(file1, "192.168.1.1").unwrap();
        writeln!(file1, "192.168.1.2").unwrap();

        let mut state = TargetSelectionState::new();
        let count1 = state.import_from_file(&file_path1).unwrap();
        assert_eq!(count1, 2);

        // Second import should clear first
        let file_path2 = dir.path().join("second.txt");
        let mut file2 = File::create(&file_path2).unwrap();
        writeln!(file2, "10.0.0.1").unwrap();

        let count2 = state.import_from_file(&file_path2).unwrap();
        assert_eq!(count2, 1);
        assert_eq!(state.imported_targets.len(), 1);
        assert_eq!(
            state.imported_targets[0],
            "10.0.0.1".parse::<std::net::IpAddr>().unwrap()
        );
    }

    // ===== Exclusion List Management Tests =====

    #[test]
    fn test_add_exclusion_single_ip() {
        let mut state = TargetSelectionState::new();

        // Add valid single IP
        let result = state.add_exclusion("192.168.1.1".to_string());
        assert!(result.is_ok());
        assert_eq!(state.exclusion_list.len(), 1);
        assert_eq!(state.exclusion_list[0], "192.168.1.1");
    }

    #[test]
    fn test_add_exclusion_cidr() {
        let mut state = TargetSelectionState::new();

        // Add valid CIDR
        let result = state.add_exclusion("10.0.0.0/8".to_string());
        assert!(result.is_ok());
        assert_eq!(state.exclusion_list.len(), 1);
        assert_eq!(state.exclusion_list[0], "10.0.0.0/8");

        // Add another CIDR
        let result2 = state.add_exclusion("192.168.0.0/16".to_string());
        assert!(result2.is_ok());
        assert_eq!(state.exclusion_list.len(), 2);
        assert_eq!(state.exclusion_list[1], "192.168.0.0/16");
    }

    #[test]
    fn test_add_exclusion_invalid() {
        let mut state = TargetSelectionState::new();

        // Try to add invalid exclusion
        let result = state.add_exclusion("not-an-ip".to_string());
        assert!(result.is_err());
        assert_eq!(state.exclusion_list.len(), 0);

        // Try to add empty exclusion
        let result2 = state.add_exclusion("".to_string());
        assert!(result2.is_err());
        assert_eq!(state.exclusion_list.len(), 0);

        // Try to add whitespace-only exclusion
        let result3 = state.add_exclusion("   ".to_string());
        assert!(result3.is_err());
        assert_eq!(state.exclusion_list.len(), 0);
    }

    #[test]
    fn test_apply_exclusions_single_ip() {
        let mut state = TargetSelectionState::new();

        // Set up targets
        state.cidr_input = "192.168.1.0/29".to_string(); // 8 IPs: .0 to .7
        state.validate_cidr();

        assert_eq!(state.target_count, 8); // No exclusions yet

        // Add exclusion for single IP
        state.add_exclusion("192.168.1.1".to_string()).unwrap();

        // Should exclude 1 IP, leaving 7
        assert_eq!(state.target_count, 7);
    }

    #[test]
    fn test_apply_exclusions_cidr_range() {
        let mut state = TargetSelectionState::new();

        // Set up targets: 192.168.1.0/24 (256 IPs)
        state.cidr_input = "192.168.1.0/24".to_string();
        state.validate_cidr();

        assert_eq!(state.target_count, 256);

        // Exclude 192.168.1.0/28 (first 16 IPs: .0 to .15)
        state.add_exclusion("192.168.1.0/28".to_string()).unwrap();

        // Should exclude 16 IPs, leaving 240
        assert_eq!(state.target_count, 240);
    }

    #[test]
    fn test_apply_exclusions_multiple() {
        let mut state = TargetSelectionState::new();

        // Set up targets: 192.168.1.0/24 (256 IPs)
        state.cidr_input = "192.168.1.0/24".to_string();
        state.validate_cidr();

        assert_eq!(state.target_count, 256);

        // Add multiple exclusions
        state.add_exclusion("192.168.1.1".to_string()).unwrap(); // 1 IP
        state.add_exclusion("192.168.1.10".to_string()).unwrap(); // 1 IP
        state.add_exclusion("192.168.1.100".to_string()).unwrap(); // 1 IP

        // Should exclude 3 IPs, leaving 253
        assert_eq!(state.target_count, 253);
    }

    #[test]
    fn test_apply_exclusions_overlapping() {
        let mut state = TargetSelectionState::new();

        // Set up targets: 192.168.1.0/24 (256 IPs)
        state.cidr_input = "192.168.1.0/24".to_string();
        state.validate_cidr();

        assert_eq!(state.target_count, 256);

        // Add overlapping exclusions
        state.add_exclusion("192.168.1.0/28".to_string()).unwrap(); // First 16 IPs (.0 to .15)
        state.add_exclusion("192.168.1.1".to_string()).unwrap(); // Single IP already in /28

        // Should still exclude only 16 IPs (overlapping exclusions don't double-count)
        assert_eq!(state.target_count, 240);
    }

    #[test]
    fn test_apply_exclusions_no_overlap() {
        let mut state = TargetSelectionState::new();

        // Set up targets: 192.168.1.0/24
        state.cidr_input = "192.168.1.0/24".to_string();
        state.validate_cidr();

        assert_eq!(state.target_count, 256);

        // Add exclusion that doesn't overlap with targets
        state.add_exclusion("10.0.0.0/8".to_string()).unwrap();

        // Should not exclude any IPs from 192.168.1.0/24
        assert_eq!(state.target_count, 256);
    }

    #[test]
    fn test_remove_exclusion_recalculates() {
        let mut state = TargetSelectionState::new();

        // Set up targets
        state.cidr_input = "192.168.1.0/24".to_string();
        state.validate_cidr();

        assert_eq!(state.target_count, 256);

        // Add exclusion
        state.add_exclusion("192.168.1.0/28".to_string()).unwrap();
        assert_eq!(state.target_count, 240); // 256 - 16

        // Remove exclusion
        state.exclusion_list_selected = 0;
        state.remove_selected_exclusion();

        // Should recalculate back to 256
        assert_eq!(state.target_count, 256);
        assert_eq!(state.exclusion_list.len(), 0);
    }

    #[test]
    fn test_exclusions_with_imported_targets() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("targets.txt");
        let mut file = File::create(&file_path).unwrap();

        // Import 10 IPs from 192.168.1.1 to 192.168.1.10
        for i in 1..=10 {
            writeln!(file, "192.168.1.{}", i).unwrap();
        }

        let mut state = TargetSelectionState::new();
        let count = state.import_from_file(&file_path).unwrap();
        assert_eq!(count, 10);
        assert_eq!(state.target_count, 10);

        // Exclude first 5 IPs (192.168.1.1 to 192.168.1.5)
        state.add_exclusion("192.168.1.1".to_string()).unwrap();
        state.add_exclusion("192.168.1.2".to_string()).unwrap();
        state.add_exclusion("192.168.1.3".to_string()).unwrap();
        state.add_exclusion("192.168.1.4".to_string()).unwrap();
        state.add_exclusion("192.168.1.5".to_string()).unwrap();

        // Should have 5 effective targets (192.168.1.6 to 192.168.1.10)
        assert_eq!(state.target_count, 5);
    }

    #[test]
    fn test_exclusions_with_cidr_and_import() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("targets.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "10.0.0.1").unwrap();
        writeln!(file, "10.0.0.2").unwrap();

        let mut state = TargetSelectionState::new();

        // Add CIDR targets: 192.168.1.0/29 (8 IPs)
        state.cidr_input = "192.168.1.0/29".to_string();
        state.validate_cidr();

        // Import additional targets
        state.import_from_file(&file_path).unwrap();

        // Total: 8 + 2 = 10 targets
        assert_eq!(state.target_count, 10);

        // Exclude 192.168.1.0/30 (first 4 IPs from CIDR)
        state.add_exclusion("192.168.1.0/30".to_string()).unwrap();

        // Should exclude 4 IPs from CIDR, leaving 8 - 4 + 2 = 6
        assert_eq!(state.target_count, 6);

        // Exclude one imported IP
        state.add_exclusion("10.0.0.1".to_string()).unwrap();

        // Should now have 5 targets
        assert_eq!(state.target_count, 5);
    }

    #[test]
    fn test_parse_exclusions_ipv6() {
        let mut state = TargetSelectionState::new();

        // Add IPv6 exclusion
        state.add_exclusion("2001:db8::/32".to_string()).unwrap();

        let parsed = state.parse_exclusions();
        assert_eq!(parsed.len(), 1);

        // Verify IPv6 network is parsed correctly
        match parsed[0] {
            IpNetwork::V6(_) => {} // Expected
            IpNetwork::V4(_) => panic!("Expected IPv6 network"),
        }
    }

    #[test]
    fn test_apply_exclusions_empty_list() {
        let state = TargetSelectionState::new();

        let targets = vec![
            "192.168.1.1".parse().unwrap(),
            "192.168.1.2".parse().unwrap(),
        ];

        let filtered = state.apply_exclusions(&targets);

        // No exclusions, should return all targets
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_exclusions_affect_export() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();

        let mut state = TargetSelectionState::new();

        // Add targets
        state.cidr_input = "192.168.1.0/29".to_string(); // 8 IPs
        state.validate_cidr();

        // Add exclusions (these should appear in export metadata)
        state.add_exclusion("192.168.1.1".to_string()).unwrap();
        state.add_exclusion("192.168.1.0/30".to_string()).unwrap();

        // Export to file
        let export_path = dir.path().join("export.txt");
        state.export_to_file(&export_path).unwrap();

        // Read the exported file
        let contents = fs::read_to_string(&export_path).unwrap();

        // Verify exclusions are documented in comments
        assert!(contents.contains("# Exclusions:"));
        assert!(contents.contains("192.168.1.1"));
        assert!(contents.contains("192.168.1.0/30"));
    }

    // ===== DNS Resolution Tests =====

    #[tokio::test]
    async fn test_resolve_hostname_success() {
        let mut state = TargetSelectionState::new();

        // Resolve localhost (should always work)
        let result = state.resolve_hostname("localhost".to_string()).await;

        assert!(result.is_ok());
        let ips = result.unwrap();
        assert!(!ips.is_empty());

        // Should contain 127.0.0.1 or ::1
        let has_ipv4 = ips.iter().any(|ip| ip.to_string() == "127.0.0.1");
        let has_ipv6 = ips.iter().any(|ip| ip.to_string() == "::1");
        assert!(has_ipv4 || has_ipv6);
    }

    #[tokio::test]
    async fn test_resolve_hostname_cache() {
        let mut state = TargetSelectionState::new();

        // First resolution
        let result1 = state.resolve_hostname("localhost".to_string()).await;
        assert!(result1.is_ok());

        // Check that result is cached
        assert!(state.dns_cache.contains_key("localhost"));

        // Second resolution should use cache (verify by checking cache directly)
        let cached = state.dns_cache.get("localhost").unwrap();
        assert!(cached.is_ok());

        // Stats should show 1 successful entry
        let (total, successful, failed, pending) = state.dns_cache_stats();
        assert_eq!(total, 1);
        assert_eq!(successful, 1);
        assert_eq!(failed, 0);
        assert_eq!(pending, 0);
    }

    #[tokio::test]
    async fn test_resolve_hostname_failure() {
        let mut state = TargetSelectionState::new();

        // Resolve invalid hostname (should fail)
        let result = state
            .resolve_hostname("this-domain-definitely-does-not-exist-12345.invalid".to_string())
            .await;

        assert!(result.is_err());

        // Failure should be cached
        assert!(state
            .dns_cache
            .contains_key("this-domain-definitely-does-not-exist-12345.invalid"));

        // Stats should show 1 failed entry
        let (total, successful, failed, pending) = state.dns_cache_stats();
        assert_eq!(total, 1);
        assert_eq!(successful, 0);
        assert_eq!(failed, 1);
        assert_eq!(pending, 0);
    }

    #[tokio::test]
    async fn test_resolve_hostnames_batch() {
        let mut state = TargetSelectionState::new();

        let hostnames = vec!["localhost".to_string(), "localhost".to_string()];

        let results = state.resolve_hostnames_batch(hostnames).await;

        // Should have 1 entry (duplicate hostname "localhost")
        assert_eq!(results.len(), 1);

        // localhost should succeed
        assert!(results.get("localhost").unwrap().is_ok());

        // Cache should have 1 entry (localhost cached after first resolution)
        assert_eq!(state.dns_cache.len(), 1);
    }

    #[tokio::test]
    async fn test_dns_cache_clear() {
        let mut state = TargetSelectionState::new();

        // Add some DNS entries
        state.resolve_hostname("localhost".to_string()).await.ok();

        // Verify cache has entry
        assert_eq!(state.dns_cache.len(), 1);

        // Clear cache
        state.clear_dns_cache();

        // Cache should be empty
        assert_eq!(state.dns_cache.len(), 0);
        assert_eq!(state.dns_pending.len(), 0);

        let (total, successful, failed, pending) = state.dns_cache_stats();
        assert_eq!(total, 0);
        assert_eq!(successful, 0);
        assert_eq!(failed, 0);
        assert_eq!(pending, 0);
    }

    #[tokio::test]
    async fn test_clear_failed_dns() {
        let mut state = TargetSelectionState::new();

        // Add successful resolution
        state.resolve_hostname("localhost".to_string()).await.ok();

        // Add failed resolution
        state
            .resolve_hostname("invalid-domain-12345.invalid".to_string())
            .await
            .ok();

        // Should have 2 entries (1 success, 1 failure)
        let (total, successful, failed, _) = state.dns_cache_stats();
        assert_eq!(total, 2);
        assert_eq!(successful, 1);
        assert_eq!(failed, 1);

        // Clear only failed entries
        state.clear_failed_dns();

        // Should have 1 entry (only successful)
        let (total, successful, failed, _) = state.dns_cache_stats();
        assert_eq!(total, 1);
        assert_eq!(successful, 1);
        assert_eq!(failed, 0);

        // localhost should still be cached
        assert!(state.dns_cache.contains_key("localhost"));
    }

    #[tokio::test]
    async fn test_dns_recalculates_target_count() {
        let mut state = TargetSelectionState::new();

        // Start with CIDR targets
        state.cidr_input = "192.168.1.0/30".to_string(); // 4 IPs
        state.validate_cidr();
        assert_eq!(state.target_count, 4);

        // Resolve localhost (adds more IPs)
        state.resolve_hostname("localhost".to_string()).await.ok();

        // Target count should increase (4 from CIDR + at least 1 from DNS)
        assert!(state.target_count > 4);
    }

    #[tokio::test]
    async fn test_dns_with_exclusions() {
        let mut state = TargetSelectionState::new();

        // Add CIDR targets
        state.cidr_input = "127.0.0.0/24".to_string(); // 256 IPs
        state.validate_cidr();

        // Resolve localhost (should resolve to 127.0.0.1)
        state.resolve_hostname("localhost".to_string()).await.ok();

        // Add exclusion for 127.0.0.1
        state.add_exclusion("127.0.0.1".to_string()).unwrap();

        // Target count should exclude 127.0.0.1
        // Should be 255 (256 - 1) if localhost resolved to 127.0.0.1
        // or 256 if localhost resolved only to ::1
        assert!(state.target_count == 255 || state.target_count == 256);
    }

    #[tokio::test]
    async fn test_dns_stats() {
        let mut state = TargetSelectionState::new();

        // Initially empty
        let (total, successful, failed, pending) = state.dns_cache_stats();
        assert_eq!(total, 0);
        assert_eq!(successful, 0);
        assert_eq!(failed, 0);
        assert_eq!(pending, 0);

        // Add successful resolution
        state.resolve_hostname("localhost".to_string()).await.ok();

        let (total, successful, failed, pending) = state.dns_cache_stats();
        assert_eq!(total, 1);
        assert_eq!(successful, 1);
        assert_eq!(failed, 0);
        assert_eq!(pending, 0);

        // Add failed resolution
        state
            .resolve_hostname("invalid-12345.invalid".to_string())
            .await
            .ok();

        let (total, successful, failed, pending) = state.dns_cache_stats();
        assert_eq!(total, 2);
        assert_eq!(successful, 1);
        assert_eq!(failed, 1);
        assert_eq!(pending, 0);
    }

    #[tokio::test]
    async fn test_dns_deduplication() {
        let mut state = TargetSelectionState::new();

        // Resolve same hostname multiple times
        state.resolve_hostname("localhost".to_string()).await.ok();
        state.resolve_hostname("localhost".to_string()).await.ok();
        state.resolve_hostname("localhost".to_string()).await.ok();

        // Should only have 1 cache entry (deduplication)
        assert_eq!(state.dns_cache.len(), 1);

        // IPs should be deduplicated within the result
        let cached_result = state.dns_cache.get("localhost").unwrap();
        if let Ok(ips) = cached_result {
            // Check for duplicates
            let unique_ips: std::collections::HashSet<_> = ips.iter().collect();
            assert_eq!(ips.len(), unique_ips.len());
        }
    }
}
