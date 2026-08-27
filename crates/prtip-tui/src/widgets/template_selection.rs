//! TemplateSelectionWidget - Interactive scan template browser and selector
//!
//! The TemplateSelectionWidget provides a comprehensive interface for browsing and selecting
//! scan templates (both built-in and custom):
//! - Browse 10 built-in templates (web-servers, databases, quick, thorough, stealth, etc.)
//! - Load and display custom templates from ~/.prtip/templates.toml
//! - Filter templates by name or description
//! - View detailed template configuration before selection
//! - Keyboard navigation with arrow keys
//!
//! # Layout
//!
//! ```text
//! ┌─ Template Selection ────────────────────────────────────────┐
//! │ Filter: web_                                                 │
//! │                                                              │
//! │ > web-servers                                  [Built-in]   │
//! │   Scan common web server ports with service detection        │
//! │   Ports: 80, 443, 8080, 8443, 3000, 5000, 8000, 8888        │
//! │   Scan: SYN | Timing: T3 | Service Detection: Yes           │
//! │                                                              │
//! │   databases                                    [Built-in]   │
//! │   Scan common database ports (MySQL, PostgreSQL, etc.)       │
//! │   Ports: 3306, 5432, 27017, 6379, 1433, 5984, 9042          │
//! │   Scan: Connect | Timing: T3 | Service Detection: Yes       │
//! │                                                              │
//! │ 10 templates (2 visible, 8 filtered)                         │
//! │ [↑/↓] Navigate [Enter] Select [Esc] Cancel [/] Filter       │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Keyboard Shortcuts
//!
//! - `↑`/`↓` - Navigate template list
//! - `Enter` - Select template and apply
//! - `Esc` - Cancel template selection
//! - `/` - Focus filter input
//! - `Tab` - Toggle between filter and template list
//! - `PgUp`/`PgDn` - Page up/down in template list
//!
//! # Examples
//!
//! ```rust,ignore
//! use prtip_tui::widgets::TemplateSelectionWidget;
//! use prtip_tui::state::UIState;
//!
//! let mut ui_state = UIState::new();
//! let template_widget = TemplateSelectionWidget::new();
//! ```

use crossterm::event::{Event, KeyCode, KeyModifiers};
use prtip_core::templates::{ScanTemplate, TemplateManager};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::state::UIState;
use crate::widgets::Component;

/// TemplateSelectionWidget providing interactive template browser
///
/// This widget allows users to browse built-in and custom scan templates,
/// filter by name/description, and select templates to apply to their scan configuration.
pub struct TemplateSelectionWidget {
    // TemplateSelectionWidget has no internal state
    // All state lives in UIState::template_selection_state
}

impl TemplateSelectionWidget {
    /// Create a new TemplateSelectionWidget
    pub fn new() -> Self {
        Self {}
    }

    /// Render the template selection widget
    fn render_templates(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        let _template_state = &state.template_selection_state;

        // Create main layout (filter input + template list + footer)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Filter input
                Constraint::Min(5),    // Template list
                Constraint::Length(2), // Footer (stats + help)
            ])
            .split(area);

        // Render filter input
        self.render_filter(frame, chunks[0], state);

        // Render template list
        self.render_template_list(frame, chunks[1], state);

        // Render footer (stats + help)
        self.render_footer(frame, chunks[2], state);
    }

    /// Render filter input section
    fn render_filter(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        let template_state = &state.template_selection_state;

        let filter_text = if template_state.filter_focused {
            format!("Filter: {}█", template_state.filter_input)
        } else {
            format!("Filter: {} (Press / to edit)", template_state.filter_input)
        };

        let filter_style = if template_state.filter_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let filter_block = Block::default()
            .borders(Borders::ALL)
            .border_style(filter_style)
            .title(" Filter ");

        let filter_paragraph = Paragraph::new(filter_text)
            .block(filter_block)
            .style(Style::default());

        frame.render_widget(filter_paragraph, area);
    }

    /// Render template list section
    fn render_template_list(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        let template_state = &state.template_selection_state;

        // Split area: template list (left) + preview panel (right)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Template list
                Constraint::Percentage(40), // Preview panel
            ])
            .split(area);

        // Render template list (left side)
        let items: Vec<ListItem> = template_state
            .filtered_templates
            .iter()
            .enumerate()
            .map(|(index, (name, template, is_custom))| {
                let is_selected = index == template_state.selected_index;

                // Format template type badge
                let badge = if *is_custom {
                    Span::styled(" [Custom] ", Style::default().fg(Color::Magenta))
                } else {
                    Span::styled(" [Built-in] ", Style::default().fg(Color::Cyan))
                };

                // Format template name with selection indicator
                let name_span = if is_selected {
                    Span::styled(
                        format!("> {}", name),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(format!("  {}", name), Style::default().fg(Color::White))
                };

                // Build lines for this template
                let mut lines = vec![Line::from(vec![name_span, badge])];

                // Add description (indented)
                lines.push(Line::from(vec![Span::styled(
                    format!("  {}", template.description),
                    Style::default().fg(Color::Gray),
                )]));

                // Add spacing between templates
                lines.push(Line::from(vec![]));

                ListItem::new(lines)
            })
            .collect();

        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(if template_state.filter_focused {
                Style::default().fg(Color::Gray)
            } else {
                Style::default().fg(Color::Yellow)
            })
            .title(format!(
                " Templates ({} of {}) ",
                template_state.filtered_templates.len(),
                template_state.total_templates
            ));

        let list = List::new(items).block(list_block);
        frame.render_widget(list, chunks[0]);

        // Render preview panel (right side)
        self.render_preview_panel(frame, chunks[1], state);
    }

    /// Render preview panel showing selected template details
    fn render_preview_panel(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        let template_state = &state.template_selection_state;

        let mut lines = vec![Line::from(Span::styled(
            "Template Preview",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))];

        if let Some((name, template, is_custom)) = template_state.get_selected_template() {
            lines.push(Line::from(""));

            // Template name and type
            let type_badge = if is_custom {
                Span::styled(" [Custom] ", Style::default().fg(Color::Magenta))
            } else {
                Span::styled(" [Built-in] ", Style::default().fg(Color::Cyan))
            };
            lines.push(Line::from(vec![
                Span::styled(name, Style::default().fg(Color::Yellow)),
                type_badge,
            ]));

            lines.push(Line::from(""));

            // Description
            lines.push(Line::from(Span::styled(
                "Description:",
                Style::default().fg(Color::Green),
            )));
            lines.push(Line::from(format!("  {}", template.description)));
            lines.push(Line::from(""));

            // Ports
            if let Some(ref ports) = template.ports {
                let ports_vec: &Vec<u16> = ports;
                lines.push(Line::from(Span::styled(
                    "Ports:",
                    Style::default().fg(Color::Green),
                )));
                if ports_vec.len() > 10 {
                    lines.push(Line::from(format!("  {} ports total", ports_vec.len())));
                    lines.push(Line::from(format!(
                        "  {}...",
                        Self::format_port_list(&ports_vec[..10])
                    )));
                } else {
                    lines.push(Line::from(format!(
                        "  {}",
                        Self::format_port_list(ports_vec)
                    )));
                }
                lines.push(Line::from(""));
            }

            // Scan configuration
            lines.push(Line::from(Span::styled(
                "Configuration:",
                Style::default().fg(Color::Green),
            )));

            if let Some(ref scan_type) = template.scan_type {
                lines.push(Line::from(format!("  Scan Type: {}", scan_type)));
            }
            if let Some(ref timing) = template.timing {
                lines.push(Line::from(format!("  Timing: {}", timing)));
            }
            if let Some(true) = template.service_detection {
                lines.push(Line::from("  Service Detection: Enabled"));
            }
            if let Some(true) = template.os_detection {
                lines.push(Line::from("  OS Detection: Enabled"));
            }
            if let Some(true) = template.tls_analysis {
                lines.push(Line::from("  TLS Analysis: Enabled"));
            }

            // Quick actions
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Quick Actions:",
                Style::default().fg(Color::Green),
            )));
            lines.push(Line::from(vec![
                Span::styled("  [Enter]", Style::default().fg(Color::Yellow)),
                Span::raw(" Apply  "),
                Span::styled("[e]", Style::default().fg(Color::Yellow)),
                Span::raw(" Edit"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  [d]", Style::default().fg(Color::Yellow)),
                Span::raw(" Duplicate  "),
                Span::styled("[Del]", Style::default().fg(Color::Yellow)),
                Span::raw(" Delete"),
            ]));
        } else {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "No template selected",
                Style::default().fg(Color::DarkGray),
            )));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Preview ")
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Render footer (stats + help text)
    fn render_footer(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        let template_state = &state.template_selection_state;

        let footer_text = if template_state.filtered_templates.is_empty() {
            "No templates match filter. Press ESC to clear filter."
        } else {
            "[↑/↓] Navigate [Enter] Select [Esc] Cancel [/] Filter [Tab] Toggle Focus"
        };

        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: false });

        frame.render_widget(footer, area);
    }

    /// Format port list for display
    fn format_port_list(ports: &[u16]) -> String {
        ports
            .iter()
            .take(8)
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Component for TemplateSelectionWidget {
    fn render(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        self.render_templates(frame, area, state);
    }
}

impl Default for TemplateSelectionWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle keyboard events for TemplateSelectionWidget
///
/// Returns true if the event was handled, false otherwise
pub fn handle_template_selection_event(event: &Event, state: &mut UIState) -> bool {
    if let Event::Key(key_event) = event {
        let template_state = &mut state.template_selection_state;

        match (key_event.code, key_event.modifiers) {
            // Filter focus toggle
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                template_state.filter_focused = true;
                true
            }

            // Tab - toggle focus between filter and list
            (KeyCode::Tab, KeyModifiers::NONE) => {
                template_state.filter_focused = !template_state.filter_focused;
                true
            }

            // Escape - cancel or clear filter
            (KeyCode::Esc, KeyModifiers::NONE) => {
                if template_state.filter_focused && !template_state.filter_input.is_empty() {
                    template_state.filter_input.clear();
                    template_state.apply_filter();
                } else {
                    template_state.filter_focused = false;
                }
                true
            }

            // Filter input handling
            (KeyCode::Char(c), KeyModifiers::NONE) if template_state.filter_focused => {
                template_state.filter_input.push(c);
                template_state.apply_filter();
                true
            }

            (KeyCode::Backspace, KeyModifiers::NONE) if template_state.filter_focused => {
                template_state.filter_input.pop();
                template_state.apply_filter();
                true
            }

            // Navigation (when filter not focused)
            (KeyCode::Up, KeyModifiers::NONE) if !template_state.filter_focused => {
                template_state.move_selection_up();
                true
            }

            (KeyCode::Down, KeyModifiers::NONE) if !template_state.filter_focused => {
                template_state.move_selection_down();
                true
            }

            (KeyCode::PageUp, KeyModifiers::NONE) if !template_state.filter_focused => {
                template_state.page_up();
                true
            }

            (KeyCode::PageDown, KeyModifiers::NONE) if !template_state.filter_focused => {
                template_state.page_down();
                true
            }

            (KeyCode::Home, KeyModifiers::NONE) if !template_state.filter_focused => {
                template_state.move_to_first();
                true
            }

            (KeyCode::End, KeyModifiers::NONE) if !template_state.filter_focused => {
                template_state.move_to_last();
                true
            }

            // Enter - select template (apply)
            (KeyCode::Enter, KeyModifiers::NONE) if !template_state.filter_focused => {
                if let Some(selected) = template_state.get_selected_template() {
                    template_state.selected_template_name = Some(selected.0.clone());
                    // Template application to Config would happen in parent widget/app
                }
                true
            }

            // Quick action: Edit (e key)
            (KeyCode::Char('e'), KeyModifiers::NONE) if !template_state.filter_focused => {
                if let Some(selected) = template_state.get_selected_template() {
                    template_state.action_pending = Some(TemplateAction::Edit(selected.0.clone()));
                }
                true
            }

            // Quick action: Duplicate (d key)
            (KeyCode::Char('d'), KeyModifiers::NONE) if !template_state.filter_focused => {
                if let Some(selected) = template_state.get_selected_template() {
                    template_state.action_pending =
                        Some(TemplateAction::Duplicate(selected.0.clone()));
                }
                true
            }

            // Quick action: Delete (Delete key)
            (KeyCode::Delete, KeyModifiers::NONE) if !template_state.filter_focused => {
                if let Some(selected) = template_state.get_selected_template() {
                    // Only allow deletion of custom templates
                    if selected.2 {
                        template_state.action_pending =
                            Some(TemplateAction::Delete(selected.0.clone()));
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

/// Template actions that can be triggered from the widget
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateAction {
    Edit(String),
    Duplicate(String),
    Delete(String),
}

// ===== State Structures =====

/// State for TemplateSelectionWidget
#[derive(Debug, Clone)]
pub struct TemplateSelectionState {
    /// Template manager (loads built-in and custom templates)
    template_manager: TemplateManager,

    /// All templates (name, template, is_custom)
    all_templates: Vec<(String, ScanTemplate, bool)>,

    /// Filtered templates (name, template, is_custom)
    filtered_templates: Vec<(String, ScanTemplate, bool)>,

    /// Total template count (for display)
    total_templates: usize,

    /// Selected template index in filtered list
    selected_index: usize,

    /// Filter input string
    filter_input: String,

    /// Whether filter input is focused (vs template list)
    filter_focused: bool,

    /// Selected template name (after Enter key)
    selected_template_name: Option<String>,

    /// Pending action (Edit, Duplicate, Delete)
    action_pending: Option<TemplateAction>,
}

impl TemplateSelectionState {
    /// Create new state and load templates
    pub fn new() -> Self {
        let template_manager = TemplateManager::with_custom_templates().unwrap_or_else(|_| {
            // Fall back to built-in only if custom template loading fails
            TemplateManager::new()
        });

        // Load all templates
        let all_templates: Vec<(String, ScanTemplate, bool)> = template_manager
            .list_templates()
            .into_iter()
            .map(|(name, template)| {
                let is_custom = !template_manager.builtin_names().contains(&name);
                (name.to_string(), template.clone(), is_custom)
            })
            .collect();

        let total_templates = all_templates.len();
        let filtered_templates = all_templates.clone();

        Self {
            template_manager,
            all_templates,
            filtered_templates,
            total_templates,
            selected_index: 0,
            filter_input: String::new(),
            filter_focused: false,
            selected_template_name: None,
            action_pending: None,
        }
    }

    /// Apply filter to template list
    ///
    /// Filters by name or description (case-insensitive substring match)
    pub fn apply_filter(&mut self) {
        let filter_lower = self.filter_input.to_lowercase();

        if filter_lower.is_empty() {
            // No filter - show all templates
            self.filtered_templates = self.all_templates.clone();
        } else {
            // Filter by name or description
            self.filtered_templates = self
                .all_templates
                .iter()
                .filter(|(name, template, _)| {
                    name.to_lowercase().contains(&filter_lower)
                        || template.description.to_lowercase().contains(&filter_lower)
                })
                .cloned()
                .collect();
        }

        // Reset selection to first item
        self.selected_index = 0;
    }

    /// Move selection up
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn move_selection_down(&mut self) {
        if self.selected_index + 1 < self.filtered_templates.len() {
            self.selected_index += 1;
        }
    }

    /// Page up (10 items)
    pub fn page_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(10);
    }

    /// Page down (10 items)
    pub fn page_down(&mut self) {
        self.selected_index = (self.selected_index + 10).min(self.filtered_templates.len() - 1);
    }

    /// Move to first template
    pub fn move_to_first(&mut self) {
        self.selected_index = 0;
    }

    /// Move to last template
    pub fn move_to_last(&mut self) {
        if !self.filtered_templates.is_empty() {
            self.selected_index = self.filtered_templates.len() - 1;
        }
    }

    /// Get currently selected template (if any)
    pub fn get_selected_template(&self) -> Option<(&String, &ScanTemplate, bool)> {
        self.filtered_templates
            .get(self.selected_index)
            .map(|(name, template, is_custom)| (name, template, *is_custom))
    }

    /// Get selected template name (after Enter key)
    pub fn get_selected_template_name(&self) -> Option<&str> {
        self.selected_template_name.as_deref()
    }

    /// Clear selected template name
    pub fn clear_selection(&mut self) {
        self.selected_template_name = None;
    }

    /// Get template by name (case-insensitive)
    pub fn get_template(&self, name: &str) -> Option<&ScanTemplate> {
        self.template_manager.get_template(name)
    }

    /// Get template manager (for advanced usage)
    pub fn template_manager(&self) -> &TemplateManager {
        &self.template_manager
    }

    /// Get pending action (if any)
    pub fn get_pending_action(&self) -> Option<&TemplateAction> {
        self.action_pending.as_ref()
    }

    /// Clear pending action
    pub fn clear_pending_action(&mut self) {
        self.action_pending = None;
    }
}

impl Default for TemplateSelectionState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_state_new() {
        let state = TemplateSelectionState::new();

        // Should load at least 10 built-in templates
        assert!(state.total_templates >= 10);
        assert_eq!(state.all_templates.len(), state.total_templates);
        assert_eq!(state.filtered_templates.len(), state.total_templates);
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.filter_input, "");
        assert!(!state.filter_focused);
        assert!(state.selected_template_name.is_none());
    }

    #[test]
    fn test_template_filter_by_name() {
        let mut state = TemplateSelectionState::new();

        // Filter for "web"
        state.filter_input = "web".to_string();
        state.apply_filter();

        // Should find web-servers template
        assert!(!state.filtered_templates.is_empty());
        assert!(state
            .filtered_templates
            .iter()
            .any(|(name, _, _)| name.contains("web")));
    }

    #[test]
    fn test_template_filter_by_description() {
        let mut state = TemplateSelectionState::new();

        // Filter for "database"
        state.filter_input = "database".to_string();
        state.apply_filter();

        // Should find databases template
        assert!(!state.filtered_templates.is_empty());
        assert!(state
            .filtered_templates
            .iter()
            .any(|(_, template, _)| template.description.to_lowercase().contains("database")));
    }

    #[test]
    fn test_template_filter_case_insensitive() {
        let mut state = TemplateSelectionState::new();

        // Filter for "WEB" (uppercase)
        state.filter_input = "WEB".to_string();
        state.apply_filter();

        // Should find web-servers (case-insensitive)
        assert!(!state.filtered_templates.is_empty());
    }

    #[test]
    fn test_template_filter_empty_restores_all() {
        let mut state = TemplateSelectionState::new();
        let original_count = state.total_templates;

        // Apply filter
        state.filter_input = "stealth".to_string();
        state.apply_filter();
        assert!(state.filtered_templates.len() < original_count);

        // Clear filter
        state.filter_input.clear();
        state.apply_filter();
        assert_eq!(state.filtered_templates.len(), original_count);
    }

    #[test]
    fn test_template_navigation_up_down() {
        let mut state = TemplateSelectionState::new();
        assert_eq!(state.selected_index, 0);

        // Move down
        state.move_selection_down();
        assert_eq!(state.selected_index, 1);

        state.move_selection_down();
        assert_eq!(state.selected_index, 2);

        // Move up
        state.move_selection_up();
        assert_eq!(state.selected_index, 1);

        state.move_selection_up();
        assert_eq!(state.selected_index, 0);

        // Move up at boundary (should stay at 0)
        state.move_selection_up();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_template_navigation_bounds() {
        let mut state = TemplateSelectionState::new();
        let max_index = state.filtered_templates.len() - 1;

        // Move to last
        state.move_to_last();
        assert_eq!(state.selected_index, max_index);

        // Move down at boundary (should stay at max)
        state.move_selection_down();
        assert_eq!(state.selected_index, max_index);

        // Move to first
        state.move_to_first();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_template_page_navigation() {
        let mut state = TemplateSelectionState::new();

        // Page down
        state.page_down();
        assert_eq!(
            state.selected_index,
            10.min(state.filtered_templates.len() - 1)
        );

        // Page up
        state.page_up();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_template_selection() {
        let state = TemplateSelectionState::new();

        // Get first template
        let selected = state.get_selected_template();
        assert!(selected.is_some());

        let (name, template, _) = selected.unwrap();
        assert!(!name.is_empty());
        assert!(!template.description.is_empty());
    }

    #[test]
    fn test_template_selection_after_filter() {
        let mut state = TemplateSelectionState::new();

        // Filter for specific template
        state.filter_input = "stealth".to_string();
        state.apply_filter();

        // Selection should reset to 0
        assert_eq!(state.selected_index, 0);

        // Get selected template
        let selected = state.get_selected_template();
        assert!(selected.is_some());

        let (name, _, _) = selected.unwrap();
        assert!(name.to_lowercase().contains("stealth"));
    }

    #[test]
    fn test_template_get_by_name() {
        let state = TemplateSelectionState::new();

        // Get built-in template
        let web_servers = state.get_template("web-servers");
        assert!(web_servers.is_some());

        let template = web_servers.unwrap();
        assert_eq!(template.name, "web-servers");
        assert!(template.ports.is_some());
        assert!(template.service_detection.unwrap_or(false));
    }

    #[test]
    fn test_template_selected_name() {
        let mut state = TemplateSelectionState::new();

        // No selection initially
        assert!(state.get_selected_template_name().is_none());

        // Simulate Enter key (select template)
        if let Some((name, _, _)) = state.get_selected_template() {
            state.selected_template_name = Some(name.clone());
        }

        // Should have selected name
        assert!(state.get_selected_template_name().is_some());

        // Clear selection
        state.clear_selection();
        assert!(state.get_selected_template_name().is_none());
    }

    #[test]
    fn test_template_manager_access() {
        let state = TemplateSelectionState::new();

        let manager = state.template_manager();

        // Should have built-in templates
        let builtin_names = manager.builtin_names();
        assert!(builtin_names.len() >= 10);
        assert!(builtin_names.contains(&"web-servers"));
        assert!(builtin_names.contains(&"databases"));
        assert!(builtin_names.contains(&"stealth"));
    }
}
