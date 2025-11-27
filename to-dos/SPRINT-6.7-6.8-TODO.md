# Sprints 6.7-6.8: Interactive Selection & TUI Polish

**Status:** 📋 PLANNING
**Timeline:** 5-7 days (40-56 hours)
**Dependencies:** Sprint 6.6 (Memory-Mapped I/O) COMPLETE ✅
**Priority:** HIGH (Critical Path to Phase 6 Completion)

---

## Overview

**Goal:** Complete Phase 6 TUI implementation by adding interactive selection widgets and polishing the user experience with a comprehensive keyboard shortcut system and context-sensitive help.

### Sprint Objectives

**Sprint 6.7: Interactive Selection Widgets (3-4 days, 24-32 hours)**
- Enhance `target_selection.rs` with advanced features
- Create new `port_selection.rs` widget for port range selection
- Enhance `template_selection.rs` with improved UX
- Integration testing and state synchronization

**Sprint 6.8: TUI Polish + Help System (2-3 days, 16-24 hours)**
- Create centralized `shortcuts.rs` keyboard shortcut system
- Enhance `help_widget.rs` with context-sensitive help
- UX polish across all widgets (focus indicators, transitions, error handling)
- Final integration, documentation, and Phase 6 completion

### Strategic Value
- Transforms ProRT-IP TUI from display-only to fully interactive
- Keyboard-first navigation enables rapid workflow for penetration testers
- Context-sensitive help reduces learning curve
- Polished UX differentiates from command-line-only tools

---

## Sprint 6.7: Interactive Selection Widgets

**Duration:** 3-4 days (24-32 hours)
**Focus:** Interactive target/port/template selection with full keyboard navigation

### Task 6.7.1: Enhanced Target Selection Widget (6-8 hours)

**Goal:** Enhance existing `target_selection.rs` with interactive features

**File:** `crates/prtip-tui/src/widgets/target_selection.rs` (currently ~1,300 lines)

**Current State Analysis:**
- ✅ CIDR input with validation (lines 90-142)
- ✅ File import section (lines 144-178)
- ✅ Exclusion list management (lines 180-212)
- ✅ DNS resolution display (lines 214-290)
- ✅ Target count summary (lines 292-351)
- ✅ Basic keyboard navigation (lines 396-480)
- ✅ `TargetSelectionState` structure (lines 494-1147)

**Enhancements Needed:**

#### 6.7.1.1: File Browser Dialog (2h)
**Location:** `crates/prtip-tui/src/widgets/file_browser.rs` (NEW - ~300 lines)

```rust
/// FileBrowserWidget for selecting target import files
pub struct FileBrowserWidget {
    current_path: PathBuf,
    entries: Vec<DirEntry>,
    selected_index: usize,
    file_filter: FileFilter,  // *.txt, *.csv, *.json, *
}

pub enum FileFilter {
    TextFiles,  // *.txt
    CsvFiles,   // *.csv
    JsonFiles,  // *.json
    All,        // *
}

impl FileBrowserWidget {
    pub fn navigate_up(&mut self) -> io::Result<()>
    pub fn navigate_into(&mut self) -> io::Result<()>
    pub fn select_file(&self) -> Option<PathBuf>
    pub fn filter_entries(&mut self, filter: FileFilter)
}
```

**Keyboard Shortcuts:**
- `Ctrl+I` - Open file browser
- `↑/↓` - Navigate entries
- `Enter` - Select directory or file
- `Backspace` - Navigate to parent directory
- `1-4` - Quick filter (Text/CSV/JSON/All)
- `Esc` - Cancel file browser

**Integration:**
- Add `file_browser_state: Option<FileBrowserState>` to `UIState`
- Open modal file browser when `Ctrl+I` pressed
- Return selected path to `TargetSelectionState.import_file_path`
- Trigger automatic file import

**Tests:** 8 tests
- Directory navigation (up, into, parent)
- File selection (txt, csv, json)
- Filtering (4 filter types)
- Edge cases (empty dir, unreadable)

---

#### 6.7.1.2: Async DNS Resolution with Progress (2h)
**Location:** Lines 800-900 in `target_selection.rs`

**Current Implementation:**
- `resolve_hostname()` - single hostname resolution (lines 810-848)
- `resolve_hostnames_batch()` - batch resolution (lines 858-910)
- DNS cache with error handling

**Enhancements:**
```rust
/// Add to TargetSelectionState
pub struct DnsProgress {
    pub total_hostnames: usize,
    pub resolved_count: usize,
    pub failed_count: usize,
    pub pending_count: usize,
    pub elapsed: Duration,
    pub eta: Option<Duration>,
}

impl TargetSelectionState {
    /// Resolve hostnames with progress tracking
    pub async fn resolve_with_progress<F>(&mut self,
        hostnames: Vec<String>,
        progress_callback: F
    ) -> Vec<(String, Result<Vec<IpAddr>, String>)>
    where F: Fn(&DnsProgress)
    {
        // Tokio JoinSet for concurrent resolution
        // Track progress via AtomicUsize counters
        // Call progress_callback every 100ms
        // Return results with success/error breakdown
    }
}
```

**UI Integration:**
- Display progress bar during bulk DNS resolution
- Show "Resolving 42/100 hostnames (15 failed, ETA: 5s)"
- Real-time update of DNS section (lines 214-290)

**Tests:** 6 tests
- Single hostname resolution
- Batch resolution (10 hostnames)
- Progress tracking (callbacks)
- Error handling (DNS failures)
- Cache hit/miss rates
- Cancellation support

---

#### 6.7.1.3: Target List Management (2-4h)
**Location:** New section in `target_selection.rs` (lines ~350-500, +150 lines)

**New Features:**
- Add "Target List" widget showing combined targets from all sources
- Display deduplicated final target list (CIDR + Import + DNS)
- Interactive selection (checkbox per IP)
- Bulk operations (Select All, Deselect All, Invert)
- Export selected targets to file

**Implementation:**
```rust
/// Add to TargetSelectionState
pub struct TargetListState {
    pub all_targets: HashSet<IpAddr>,  // Deduplicated
    pub selected_targets: HashSet<IpAddr>,
    pub scroll_offset: usize,
    pub selected_row: usize,
}

impl TargetSelectionState {
    /// Render target list with checkboxes
    fn render_target_list(frame: &mut Frame, area: Rect, state: &UIState) {
        // Table widget: [✓] IP Address | Source (CIDR/Import/DNS)
        // Keyboard: Space=toggle, a=all, n=none, i=invert
    }

    /// Get final target list (after exclusions)
    pub fn get_final_targets(&self) -> Vec<IpAddr> {
        self.all_targets.iter()
            .filter(|ip| !self.excluded_ips.contains(ip))
            .copied()
            .collect()
    }

    /// Export selected targets to file
    pub fn export_targets(&self, path: &str, format: ExportFormat) -> io::Result<()> {
        // Plain text: one IP per line
        // JSON: {"targets": ["192.168.1.1", ...]}
        // CSV: ip,source\n192.168.1.1,CIDR
    }
}

pub enum ExportFormat {
    PlainText,
    Json,
    Csv,
}
```

**Keyboard Shortcuts:**
- `Space` - Toggle current IP selection
- `a` - Select all targets
- `n` - Deselect all targets
- `i` - Invert selection
- `Ctrl+E` - Export selected targets
- `Delete` - Remove selected from list

**Tests:** 10 tests
- Target deduplication (CIDR+Import+DNS)
- Selection operations (toggle, all, none, invert)
- Export (plain text, JSON, CSV)
- Exclusion filtering
- Scrolling (large lists 1000+ IPs)

---

### Task 6.7.2: Port Range Selection Widget (8-10 hours)

**Goal:** Create new interactive port selection widget

**File:** `crates/prtip-tui/src/widgets/port_selection.rs` (NEW - ~800 lines)

**Widget Design:**
```
┌─ Port Selection ────────────────────────────────────────────┐
│ Quick Presets:                                              │
│   [Top 100] [Top 1000] [All Ports] [Common Services]       │
│                                                              │
│ Custom Range:                                               │
│   Ports: 80,443,8080-8090,3000_                            │
│   ✓ Valid | 13 ports selected                              │
│                                                              │
│ Selected Ports (13):                                        │
│   80, 443, 3000, 8080-8090                                 │
│                                                              │
│ Port Categories: (toggle with number keys 1-9)             │
│   [✓] Web (80, 443, 8080, 8443)                4 ports     │
│   [ ] SSH (22)                                 1 port      │
│   [✓] Database (3306, 5432, 27017, 6379)      4 ports     │
│   [ ] Mail (25, 110, 143, 465, 587, 993)      6 ports     │
│   [ ] File Sharing (21, 139, 445, 2049)       4 ports     │
│   [ ] Remote Access (23, 3389, 5900)          3 ports     │
│   [ ] Custom Range (8000-9000)                1001 ports   │
│                                                              │
│ Total: 13 ports selected                                    │
│ [Start Scan] [Clear] [Save Preset]                          │
└──────────────────────────────────────────────────────────────┘
```

#### 6.7.2.1: Port Parser & Validator (2h)
```rust
/// Port specification parser
pub struct PortSpec {
    ports: HashSet<u16>,
}

impl PortSpec {
    /// Parse port specification: "80,443,8080-8090,3000"
    pub fn parse(input: &str) -> Result<Self, String> {
        // Comma-separated values
        // Ranges: "8080-8090" (inclusive)
        // Single ports: "80"
        // Validation: 1-65535, no duplicates
    }

    /// Quick presets
    pub fn top_100() -> Self
    pub fn top_1000() -> Self
    pub fn all_ports() -> Self  // 1-65535
    pub fn common_services() -> Self  // Web, SSH, DB, Mail
}

/// Port categories for quick selection
pub enum PortCategory {
    Web,          // 80, 443, 8080, 8443, 8000, 8888
    Ssh,          // 22
    Database,     // 3306, 5432, 27017, 6379, 1433
    Mail,         // 25, 110, 143, 465, 587, 993, 995
    FileSharing,  // 21, 139, 445, 2049, 111
    RemoteAccess, // 23, 3389, 5900, 5800
    CustomRange,  // User-defined range
}

impl PortCategory {
    pub fn ports(&self) -> Vec<u16>
    pub fn description(&self) -> &'static str
}
```

**Tests:** 12 tests
- Parse single port ("80" → {80})
- Parse comma-separated ("80,443" → {80, 443})
- Parse range ("8080-8090" → {8080..=8090})
- Parse mixed ("80,443,8000-9000")
- Invalid input ("abc", "70000", "-10")
- Category port lists (Web=6 ports, Database=5 ports)

---

#### 6.7.2.2: Interactive Port Widget (4h)
```rust
/// PortSelectionWidget state
pub struct PortSelectionState {
    /// Current port input string
    pub port_input: String,

    /// Parsed port set (validated)
    pub selected_ports: HashSet<u16>,

    /// Active preset (if any)
    pub active_preset: Option<PortPreset>,

    /// Port category toggles
    pub category_toggles: HashMap<PortCategory, bool>,

    /// Custom range input
    pub custom_range: Option<(u16, u16)>,

    /// Input validation state
    pub validation_state: ValidationState,

    /// Cursor position in input
    pub cursor_position: usize,
}

impl PortSelectionState {
    pub fn toggle_preset(&mut self, preset: PortPreset)
    pub fn toggle_category(&mut self, category: PortCategory)
    pub fn add_custom_range(&mut self, start: u16, end: u16)
    pub fn clear_selection(&mut self)
    pub fn validate_input(&mut self)
    pub fn get_port_list(&self) -> Vec<u16>  // Sorted
}

pub enum PortPreset {
    Top100,
    Top1000,
    AllPorts,
    CommonServices,
}
```

**Keyboard Shortcuts:**
- `1-6` - Toggle port categories (Web, SSH, DB, Mail, File, Remote)
- `F1-F4` - Quick presets (Top 100, Top 1000, All, Common)
- `Enter` - Confirm selection
- `Esc` - Cancel changes
- `Ctrl+C` - Clear all selections
- `Ctrl+S` - Save as custom preset
- `↑/↓` - Navigate categories
- `Space` - Toggle category
- `/` - Focus custom range input

**Tests:** 15 tests
- Preset selection (Top 100, Top 1000, All, Common)
- Category toggling (Web, SSH, DB, etc.)
- Custom range input ("8000-9000")
- Port deduplication (multiple sources)
- Input validation (real-time)
- State persistence (save/load)

---

#### 6.7.2.3: Port Preset Manager (2-4h)
```rust
/// Port preset manager (save/load custom presets)
pub struct PortPresetManager {
    presets: HashMap<String, PortSpec>,
    config_path: PathBuf,  // ~/.prtip/port-presets.toml
}

impl PortPresetManager {
    pub fn load() -> io::Result<Self>
    pub fn save(&self) -> io::Result<()>
    pub fn add_preset(&mut self, name: String, spec: PortSpec)
    pub fn remove_preset(&mut self, name: &str) -> Option<PortSpec>
    pub fn list_presets(&self) -> Vec<&String>
    pub fn get_preset(&self, name: &str) -> Option<&PortSpec>
}

// File format: ~/.prtip/port-presets.toml
// [presets]
// web-dev = "80,443,3000,5000,8080-8090"
// databases = "3306,5432,27017,6379"
```

**Tests:** 8 tests
- Save preset to file
- Load presets from file
- Add/remove preset
- List all presets
- Get preset by name
- Invalid TOML handling

---

### Task 6.7.3: Enhanced Template Selection (2-3 hours)

**Goal:** Improve template selection UX with parameter preview and quick actions

**File:** `crates/prtip-tui/src/widgets/template_selection.rs` (currently ~740 lines)

**Current State:**
- ✅ Template browsing with filter (lines 80-102)
- ✅ Built-in + custom template support (lines 395-426)
- ✅ Keyboard navigation (lines 277-363)
- ✅ 18 unit tests (lines 525-739)

**Enhancements:**

#### 6.7.3.1: Template Preview Panel (1-1.5h)
**Location:** Lines ~100-150 (new rendering section)

```rust
/// Render template preview panel (right side)
fn render_template_preview(frame: &mut Frame, area: Rect, state: &UIState) {
    let template_state = &state.template_selection_state;

    if let Some((name, template, is_custom)) = template_state.get_selected_template() {
        // Split area: Title (3 lines) | Parameters (remaining)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Template name + badge
                Constraint::Min(0),      // Parameter details
            ])
            .split(area);

        // Render template header
        let header = vec![
            Line::from(vec![
                Span::styled(name, Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)),
                if is_custom {
                    Span::styled(" [Custom]", Style::default().fg(Color::Magenta))
                } else {
                    Span::styled(" [Built-in]", Style::default().fg(Color::Green))
                },
            ]),
            Line::from(template.description.clone()),
        ];

        // Render parameter table
        let params = vec![
            format!("Scan Type:        {}", template.scan_type.unwrap_or_default()),
            format!("Ports:            {}", format_ports(&template.ports)),
            format!("Timing:           {}", template.timing.unwrap_or_default()),
            format!("Service Detect:   {}", yes_no(template.service_detection)),
            format!("OS Detection:     {}", yes_no(template.os_detection)),
            format!("TLS Analysis:     {}", yes_no(template.tls_analysis)),
            format!("Scripts:          {}", template.scripts.len()),
        ];

        // ... render both sections
    }
}
```

**Tests:** 3 tests
- Preview rendering (built-in template)
- Preview rendering (custom template)
- No preview when no selection

---

#### 6.7.3.2: Template Quick Actions (1-1.5h)
**Location:** Lines 277-363 (keyboard handler enhancement)

**New Keyboard Shortcuts:**
- `d` - Duplicate template (create custom variant)
- `e` - Edit template (open editor)
- `x` - Delete custom template
- `Enter` - Apply template to scan config
- `Ctrl+S` - Save current config as template

```rust
/// Handle template quick actions
pub fn handle_template_action(
    action: TemplateAction,
    state: &mut UIState,
    config: &mut ScanConfig
) -> Result<(), String> {
    match action {
        TemplateAction::Duplicate => {
            // Create copy of selected template with "(copy)" suffix
            // Switch to edit mode
        }
        TemplateAction::Edit => {
            // Open template editor modal
            // Allow editing all parameters
        }
        TemplateAction::Delete => {
            // Confirm deletion (built-in templates cannot be deleted)
            // Remove from TemplateManager
            // Refresh template list
        }
        TemplateAction::Apply => {
            // Apply template to ScanConfig
            // Merge template params into config
            // Close template selector
        }
        TemplateAction::SaveCurrent => {
            // Prompt for template name
            // Save current ScanConfig as template
            // Add to custom templates
        }
    }
}
```

**Tests:** 6 tests
- Duplicate template
- Delete custom template (confirm built-in rejected)
- Apply template to config
- Save current config as template
- Edit template parameters

---

### Task 6.7.4: Integration & State Synchronization (4-6 hours)

**Goal:** Wire all widgets to scan configuration and ensure state synchronization

#### 6.7.4.1: ScanConfig Integration (2h)
**File:** `crates/prtip-tui/src/state/scan_config.rs` (NEW - ~300 lines)

```rust
/// Scan configuration state (maps from TUI widgets → scanner params)
pub struct ScanConfig {
    // From TargetSelectionWidget
    pub targets: Vec<IpAddr>,
    pub exclusions: Vec<IpAddr>,

    // From PortSelectionWidget
    pub ports: Vec<u16>,
    pub port_spec: PortSpec,

    // From TemplateSelectionWidget
    pub scan_type: ScanType,
    pub timing: TimingTemplate,
    pub service_detection: bool,
    pub os_detection: bool,
    pub tls_analysis: bool,
    pub scripts: Vec<String>,

    // Computed fields
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
}

impl ScanConfig {
    pub fn new() -> Self
    pub fn validate(&mut self) -> bool
    pub fn apply_template(&mut self, template: &ScanTemplate)
    pub fn to_cli_args(&self) -> Vec<String>  // For subprocess execution
}
```

**Integration Points:**
- `TargetSelectionState.get_final_targets()` → `ScanConfig.targets`
- `PortSelectionState.get_port_list()` → `ScanConfig.ports`
- `TemplateSelectionState.get_selected_template()` → `ScanConfig.apply_template()`

**Tests:** 8 tests
- Config validation (empty targets/ports)
- Template application
- CLI args generation
- State synchronization

---

#### 6.7.4.2: Widget Event Routing (2h)
**File:** `crates/prtip-tui/src/events/widget_events.rs` (NEW - ~200 lines)

```rust
/// Widget-specific events
pub enum WidgetEvent {
    TargetSelection(TargetSelectionEvent),
    PortSelection(PortSelectionEvent),
    TemplateSelection(TemplateSelectionEvent),
}

pub enum TargetSelectionEvent {
    TargetsChanged(Vec<IpAddr>),
    ExclusionsChanged(Vec<IpAddr>),
    FileImported(PathBuf, usize),
    DnsResolved(String, Vec<IpAddr>),
}

pub enum PortSelectionEvent {
    PortsChanged(Vec<u16>),
    PresetApplied(PortPreset),
    CategoryToggled(PortCategory, bool),
}

pub enum TemplateSelectionEvent {
    TemplateSelected(String),
    TemplateApplied(ScanTemplate),
    CustomTemplateSaved(String, ScanTemplate),
}

/// Event dispatcher
pub struct WidgetEventDispatcher {
    scan_config: Arc<RwLock<ScanConfig>>,
}

impl WidgetEventDispatcher {
    pub fn handle_event(&mut self, event: WidgetEvent) {
        // Route events to appropriate handlers
        // Update ScanConfig based on widget changes
        // Emit EventBus notifications for other widgets
    }
}
```

**Tests:** 6 tests
- Event routing (Target → Config)
- Event routing (Port → Config)
- Event routing (Template → Config)
- Cross-widget synchronization
- EventBus integration

---

#### 6.7.4.3: Full Workflow Testing (2-4h)
**File:** `crates/prtip-tui/tests/integration/widget_workflow.rs` (NEW - ~400 lines)

**Test Scenarios:**
1. **Basic Workflow** (5 tests)
   - Select targets (CIDR)
   - Select ports (Top 100)
   - Select template (Quick Discovery)
   - Validate config
   - Generate CLI args

2. **Advanced Workflow** (5 tests)
   - Import targets from file
   - Add exclusions
   - Custom port range
   - Custom template
   - DNS resolution batch

3. **Error Handling** (4 tests)
   - Empty targets validation
   - Empty ports validation
   - Invalid CIDR input
   - File import failure

4. **State Synchronization** (4 tests)
   - Target change → Config update
   - Port change → Config update
   - Template change → Config update
   - Cross-widget consistency

**Total Sprint 6.7 Tests:** ~90 tests
- File Browser: 8
- DNS Progress: 6
- Target List: 10
- Port Parser: 12
- Port Widget: 15
- Port Presets: 8
- Template Preview: 3
- Template Actions: 6
- ScanConfig: 8
- Widget Events: 6
- Workflow: 18

---

## Sprint 6.8: TUI Polish + Help System

**Duration:** 2-3 days (16-24 hours)
**Focus:** Keyboard shortcuts, context-sensitive help, UX refinements

### Task 6.8.1: Centralized Keyboard Shortcut System (6-8 hours)

**Goal:** Create unified keyboard shortcut management with conflict detection and customization

**File:** `crates/prtip-tui/src/shortcuts.rs` (NEW - ~600 lines)

#### 6.8.1.1: Shortcut Manager (3-4h)
```rust
/// Global keyboard shortcut manager
pub struct ShortcutManager {
    /// Global shortcuts (always active)
    global_shortcuts: HashMap<KeyBinding, ShortcutAction>,

    /// Context-specific shortcuts (widget-dependent)
    context_shortcuts: HashMap<ShortcutContext, HashMap<KeyBinding, ShortcutAction>>,

    /// User customizations (loaded from ~/.prtip/shortcuts.toml)
    custom_bindings: HashMap<KeyBinding, ShortcutAction>,

    /// Conflict detection cache
    conflicts: Vec<ShortcutConflict>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutContext {
    Global,           // Always active
    TargetSelection,  // TargetSelectionWidget focused
    PortSelection,    // PortSelectionWidget focused
    TemplateSelection,// TemplateSelectionWidget focused
    Dashboard,        // Dashboard view
    Help,             // Help screen
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutAction {
    // Global
    Quit,
    ToggleHelp,
    NextPane,
    PrevPane,

    // Navigation
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    Home,
    End,

    // Selection
    ToggleSelection,
    SelectAll,
    DeselectAll,
    InvertSelection,

    // Widget-Specific
    OpenFileBrowser,
    AddExclusion,
    RemoveExclusion,
    ToggleCategory(usize),
    ApplyTemplate,
    SaveTemplate,

    // Custom (user-defined)
    Custom(String),
}

impl ShortcutManager {
    /// Create default shortcut manager
    pub fn new() -> Self

    /// Load custom bindings from config file
    pub fn load_custom_bindings(&mut self) -> io::Result<()>

    /// Save custom bindings to config file
    pub fn save_custom_bindings(&self) -> io::Result<()>

    /// Get action for key binding in context
    pub fn get_action(&self, key: KeyBinding, context: ShortcutContext) -> Option<ShortcutAction>

    /// Register custom binding
    pub fn register_binding(&mut self, key: KeyBinding, action: ShortcutAction) -> Result<(), ShortcutConflict>

    /// Detect conflicts (same key, different contexts)
    pub fn detect_conflicts(&self) -> Vec<ShortcutConflict>

    /// Get all bindings for context
    pub fn get_context_bindings(&self, context: ShortcutContext) -> Vec<(KeyBinding, ShortcutAction)>
}

#[derive(Debug, Clone)]
pub struct ShortcutConflict {
    pub key: KeyBinding,
    pub context1: ShortcutContext,
    pub action1: ShortcutAction,
    pub context2: ShortcutContext,
    pub action2: ShortcutAction,
}
```

**Default Bindings:**

**Global Shortcuts:**
- `q`, `Ctrl+C`, `Esc` - Quit application
- `?`, `F1` - Toggle help
- `Tab` - Next pane/widget
- `Shift+Tab` - Previous pane/widget

**Navigation (all contexts):**
- `↑/↓` or `j/k` - Scroll up/down
- `Page Up/Down` - Scroll page
- `Home/End` - First/last item
- `Ctrl+U/D` - Half-page scroll

**Target Selection:**
- `Ctrl+I` - Open file browser
- `Ctrl+E` - Add exclusion
- `Delete` - Remove exclusion
- `Space` - Toggle IP selection
- `a` - Select all IPs
- `n` - Deselect all
- `i` - Invert selection

**Port Selection:**
- `1-6` - Toggle categories (Web, SSH, DB, Mail, File, Remote)
- `F1-F4` - Presets (Top 100, Top 1000, All, Common)
- `Ctrl+C` - Clear selection
- `Ctrl+S` - Save preset
- `/` - Focus custom range input

**Template Selection:**
- `/` - Filter templates
- `d` - Duplicate template
- `e` - Edit template
- `x` - Delete template
- `Enter` - Apply template
- `Ctrl+S` - Save current config as template

**Dashboard:**
- `1-4` - Switch tabs (Port/Service/Metrics/Network)
- `p/s/r/v` - Sort columns
- `f` - Cycle filters
- `a` - Toggle auto-scroll

**Tests:** 15 tests
- Default bindings load
- Custom bindings save/load
- Conflict detection (2 tests)
- Context filtering (get bindings for context)
- Key lookup (find action for key+context)
- Invalid TOML handling
- Modifier combinations (Ctrl+, Shift+, etc.)

---

#### 6.8.1.2: Shortcut Customization UI (2-3h)
**File:** `crates/prtip-tui/src/widgets/shortcut_editor.rs` (NEW - ~400 lines)

```rust
/// Interactive shortcut editor widget
pub struct ShortcutEditorWidget {
    manager: ShortcutManager,
    selected_context: ShortcutContext,
    selected_row: usize,
    editing_binding: Option<EditingBinding>,
}

struct EditingBinding {
    action: ShortcutAction,
    old_key: KeyBinding,
    new_key: Option<KeyBinding>,
    conflict: Option<ShortcutConflict>,
}

impl ShortcutEditorWidget {
    /// Render shortcut editor
    ///
    /// Layout:
    /// ┌─ Keyboard Shortcuts Editor ────────────────────────┐
    /// │ Context: [Global] ◀ ▶                              │
    /// │                                                     │
    /// │ Action                 | Binding                   │
    /// │ ─────────────────────────────────────────────────  │
    /// │ Quit                   | q, Ctrl+C, Esc           │
    /// │ Toggle Help            | ?, F1                     │
    /// │ > Next Pane            | Tab                 [Edit]│
    /// │ Previous Pane          | Shift+Tab                 │
    /// │ ...                                                │
    /// │                                                     │
    /// │ [Save] [Reset to Defaults] [Cancel]                │
    /// │ Press any key to bind, or Esc to cancel            │
    /// └─────────────────────────────────────────────────────┘
    pub fn render(&self, frame: &mut Frame, area: Rect)

    /// Enter edit mode for selected binding
    pub fn edit_binding(&mut self, action: ShortcutAction)

    /// Capture next key press as new binding
    pub fn capture_key(&mut self, key: KeyBinding) -> Result<(), ShortcutConflict>

    /// Save changes to config file
    pub fn save_changes(&mut self) -> io::Result<()>

    /// Reset to default bindings
    pub fn reset_to_defaults(&mut self)
}
```

**Keyboard Shortcuts:**
- `←/→` - Switch context (Global, Target, Port, Template, etc.)
- `↑/↓` - Select action
- `Enter` - Edit binding (capture mode)
- `Delete` - Remove binding
- `Ctrl+S` - Save changes
- `Ctrl+R` - Reset to defaults
- `Esc` - Cancel edit/close editor

**Tests:** 8 tests
- Context switching
- Edit binding (capture key)
- Conflict detection during edit
- Save changes
- Reset to defaults
- Remove binding
- Multiple bindings per action

---

#### 6.8.1.3: Cheat Sheet Overlay (1-2h)
**File:** `crates/prtip-tui/src/widgets/cheat_sheet.rs` (NEW - ~200 lines)

```rust
/// Compact keyboard shortcut cheat sheet (overlay)
pub struct CheatSheetWidget {
    context: ShortcutContext,
    manager: ShortcutManager,
}

impl CheatSheetWidget {
    /// Render compact cheat sheet
    ///
    /// Layout (centered overlay):
    /// ┌─ Keyboard Shortcuts ────────────────────┐
    /// │ Global                                   │
    /// │   q / Ctrl+C - Quit                     │
    /// │   ? / F1     - Help                     │
    /// │   Tab        - Next pane                │
    /// │                                          │
    /// │ Navigation                               │
    /// │   ↑/↓        - Scroll                   │
    /// │   PgUp/PgDn  - Page                     │
    /// │   Home/End   - First/Last               │
    /// │                                          │
    /// │ [Context: Global] [Press ? to close]    │
    /// └──────────────────────────────────────────┘
    pub fn render(&self, frame: &mut Frame, area: Rect)

    /// Switch displayed context
    pub fn set_context(&mut self, context: ShortcutContext)
}
```

**Integration:**
- `F1` or `?` - Toggle cheat sheet overlay
- Auto-set context based on active widget
- Semi-transparent background (Block widget)

**Tests:** 3 tests
- Render for each context
- Context switching
- Overlay positioning

---

### Task 6.8.2: Enhanced Help System (4-6 hours)

**Goal:** Transform `help_widget.rs` into comprehensive, context-sensitive help system

**File:** `crates/prtip-tui/src/widgets/help_widget.rs` (currently ~408 lines)

**Current State:**
- ✅ Basic help text (lines 70-200)
- ✅ Scrolling support (lines 256-290)
- ✅ Context mode toggle (lines 280-282)
- ✅ 12 unit tests (lines 293-407)

#### 6.8.2.1: Tabbed Help System (2-3h)
**Enhancement:** Lines 70-200 (replace with tabbed interface)

```rust
/// Help tab categories
pub enum HelpTab {
    QuickStart,      // Getting started guide
    Shortcuts,       // Keyboard shortcuts by context
    Widgets,         // Widget-specific help
    Workflows,       // Common workflow examples
    Troubleshooting, // FAQ and common issues
}

impl HelpWidget {
    /// Render tabbed help system
    fn render_tabbed_help(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        let help_state = &state.help_widget_state;

        // Create tab bar (top)
        let tabs = vec![
            "Quick Start", "Shortcuts", "Widgets",
            "Workflows", "Troubleshooting"
        ];

        // Render active tab content
        match help_state.active_tab {
            HelpTab::QuickStart => self.render_quick_start(frame, area),
            HelpTab::Shortcuts => self.render_shortcuts_help(frame, area, state),
            HelpTab::Widgets => self.render_widgets_help(frame, area),
            HelpTab::Workflows => self.render_workflows_help(frame, area),
            HelpTab::Troubleshooting => self.render_troubleshooting(frame, area),
        }
    }

    /// Quick Start tab (beginner-friendly)
    fn render_quick_start(&self, frame: &mut Frame, area: Rect) {
        // 1. What is ProRT-IP?
        // 2. Basic scan workflow (5 steps)
        // 3. Essential keyboard shortcuts (10 keys)
        // 4. Next steps (advanced features)
    }

    /// Shortcuts tab (by context, searchable)
    fn render_shortcuts_help(&self, frame: &mut Frame, area: Rect, state: &UIState) {
        // Use ShortcutManager to get current bindings
        // Group by context (Global, Target, Port, Template, etc.)
        // Highlight customized bindings
        // Show conflicts (if any)
    }

    /// Widgets tab (detailed widget help)
    fn render_widgets_help(&self, frame: &mut Frame, area: Rect) {
        // Explain each widget:
        // - TargetSelectionWidget (CIDR, import, exclusions, DNS)
        // - PortSelectionWidget (presets, categories, custom ranges)
        // - TemplateSelectionWidget (built-in, custom, apply)
        // - Dashboard widgets (Port Table, Service Table, Metrics, Network)
    }

    /// Workflows tab (step-by-step examples)
    fn render_workflows_help(&self, frame: &mut Frame, area: Rect) {
        // Example workflows:
        // 1. Quick network discovery (5 steps)
        // 2. Service version detection (7 steps)
        // 3. Stealth scan with exclusions (8 steps)
        // 4. Custom template creation (6 steps)
        // 5. Bulk target import (4 steps)
    }

    /// Troubleshooting tab (FAQ)
    fn render_troubleshooting(&self, frame: &mut Frame, area: Rect) {
        // Common issues:
        // Q: "No ports found" - Check firewall, timing, target reachability
        // Q: "Permission denied" - Root/admin required for raw sockets
        // Q: "DNS resolution slow" - Use --no-dns or increase timeout
        // Q: "TUI not responding" - Check terminal size (min 80x24)
        // ... 10-15 FAQ entries
    }
}
```

**New Keyboard Shortcuts:**
- `1-5` - Switch tabs (Quick Start, Shortcuts, Widgets, Workflows, Troubleshooting)
- `Tab` - Next tab
- `Shift+Tab` - Previous tab
- `/` - Search help content (future enhancement)

**Tests:** 10 tests
- Tab switching (1-5 keys)
- Quick Start rendering
- Shortcuts tab (reads from ShortcutManager)
- Widgets tab rendering
- Workflows tab rendering
- Troubleshooting tab rendering
- Scrolling within tabs
- Context filtering (show only relevant shortcuts)

---

#### 6.8.2.2: Context-Sensitive Help Tooltips (1-2h)
**File:** `crates/prtip-tui/src/widgets/tooltip.rs` (NEW - ~250 lines)

```rust
/// Tooltip widget for inline help
pub struct TooltipWidget {
    text: String,
    position: TooltipPosition,
    style: TooltipStyle,
}

pub enum TooltipPosition {
    Below,    // Below the widget
    Above,    // Above the widget
    Right,    // To the right
    Cursor,   // At cursor position
}

pub enum TooltipStyle {
    Info,     // Blue background
    Warning,  // Yellow background
    Error,    // Red background
    Hint,     // Gray background
}

impl TooltipWidget {
    pub fn new(text: String, position: TooltipPosition) -> Self

    /// Render tooltip at calculated position
    pub fn render(&self, frame: &mut Frame, anchor: Rect)
}

/// Tooltip manager (tracks active tooltips)
pub struct TooltipManager {
    tooltips: HashMap<String, Tooltip>,
    active_tooltip: Option<String>,
}

impl TooltipManager {
    /// Show tooltip for widget/action
    pub fn show(&mut self, id: &str, text: String, position: TooltipPosition)

    /// Hide active tooltip
    pub fn hide(&mut self)

    /// Get active tooltip (if any)
    pub fn get_active(&self) -> Option<&Tooltip>
}
```

**Usage Examples:**
- Hover over CIDR input → "Enter IP/CIDR (e.g., 192.168.1.0/24)"
- Hover over preset button → "Scan top 100 most common ports"
- Hover over exclusion list → "IPs to skip during scan"
- Invalid input → "Invalid CIDR notation" (error style)

**Integration:**
- Add `tooltip_manager: TooltipManager` to `UIState`
- Trigger tooltips on:
  - Widget focus change
  - Mouse hover (if mouse support enabled)
  - Input validation errors
- Auto-hide after 5 seconds or on next keypress

**Tests:** 6 tests
- Show tooltip (4 positions)
- Hide tooltip
- Multiple tooltips (only one active)
- Auto-hide timeout
- Style variants (Info, Warning, Error, Hint)

---

#### 6.8.2.3: Help Search (1h)
**Enhancement:** Add search functionality to help system

```rust
/// Add to HelpWidgetState
pub struct HelpWidgetState {
    // Existing fields...
    pub active_tab: HelpTab,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub search_focused: bool,
}

pub struct SearchResult {
    pub tab: HelpTab,
    pub line_number: usize,
    pub matched_text: String,
    pub context: String,  // Surrounding text
}

impl HelpWidgetState {
    /// Search help content
    pub fn search(&mut self, query: &str) {
        // Fuzzy search across all help tabs
        // Return matches with context (2 lines before/after)
        // Highlight query terms in results
    }

    /// Jump to search result
    pub fn jump_to_result(&mut self, result: &SearchResult) {
        // Switch to result tab
        // Scroll to line number
        // Highlight matched text
    }
}
```

**Keyboard Shortcuts:**
- `/` - Focus search input
- `Enter` - Execute search
- `↑/↓` - Navigate results
- `Enter` - Jump to result
- `Esc` - Clear search / exit search mode

**Tests:** 4 tests
- Search single word
- Search phrase
- No results handling
- Jump to result

---

### Task 6.8.3: UX Polish (4-6 hours)

**Goal:** Enhance visual feedback, transitions, and error handling across all widgets

#### 6.8.3.1: Focus Indicators (2h)
**Files:** All widget files

**Enhancements:**
- Visual focus ring (border color change: Gray → Yellow)
- Focus breadcrumb trail (show widget path: "Dashboard > Port Table > Row 42")
- Focus history (Alt+←/→ to navigate focus history)

```rust
/// Add to UIState
pub struct FocusManager {
    focus_stack: Vec<FocusedWidget>,
    focus_history: VecDeque<FocusedWidget>,
    max_history: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusedWidget {
    TargetSelection,
    PortSelection,
    TemplateSelection,
    Dashboard(DashboardTab),
    Help(HelpTab),
}

impl FocusManager {
    pub fn push_focus(&mut self, widget: FocusedWidget)
    pub fn pop_focus(&mut self) -> Option<FocusedWidget>
    pub fn navigate_back(&mut self) -> Option<FocusedWidget>
    pub fn navigate_forward(&mut self) -> Option<FocusedWidget>
    pub fn get_breadcrumb(&self) -> String  // "Dashboard > Port Table"
}
```

**Visual Indicators:**
- Focused widget: Yellow border
- Unfocused widget: Gray border
- Breadcrumb: Status bar (top right)
- Focus trail: Dimmed previous focus (for context)

**Tests:** 6 tests
- Focus push/pop
- Focus history navigation (back/forward)
- Breadcrumb generation
- Visual indicator rendering

---

#### 6.8.3.2: Loading States & Spinners (1-2h)
**File:** `crates/prtip-tui/src/widgets/loading.rs` (NEW - ~150 lines)

```rust
/// Loading spinner widget
pub struct LoadingSpinner {
    text: String,
    frame: usize,
    style: SpinnerStyle,
}

pub enum SpinnerStyle {
    Dots,      // ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
    Line,      // - \ | /
    Arrows,    // ← ↑ → ↓
    Clock,     // 🕐 🕑 🕒 🕓 🕔 🕕
}

impl LoadingSpinner {
    pub fn new(text: String, style: SpinnerStyle) -> Self

    /// Advance spinner frame
    pub fn tick(&mut self)

    /// Render spinner with text
    pub fn render(&self, frame: &mut Frame, area: Rect)
}

/// Progress bar widget
pub struct ProgressBar {
    progress: f32,      // 0.0 - 1.0
    text: String,
    style: ProgressStyle,
}

pub enum ProgressStyle {
    Bar,      // [████████░░░░░░░░] 50%
    Blocks,   // ▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯ 50%
    Percent,  // 50% (42/100)
}

impl ProgressBar {
    pub fn new(progress: f32, text: String) -> Self

    /// Update progress
    pub fn set_progress(&mut self, progress: f32)

    /// Render progress bar
    pub fn render(&self, frame: &mut Frame, area: Rect)
}
```

**Usage:**
- File import: "Loading targets... ⠋ (42/100)"
- DNS resolution: "Resolving hostnames... [████████░░░░] 50%"
- Template loading: "Loading templates... →"
- Scan execution: "Scanning... [▮▮▮▮▮▮▯▯] 75% (750/1000 ports)"

**Tests:** 6 tests
- Spinner tick (frame advancement)
- Spinner styles (4 styles)
- Progress bar rendering
- Progress updates

---

#### 6.8.3.3: Error Handling & Toast Messages (1-2h)
**File:** `crates/prtip-tui/src/widgets/toast.rs` (NEW - ~200 lines)

```rust
/// Toast notification widget
pub struct ToastWidget {
    message: String,
    toast_type: ToastType,
    duration: Duration,
    created_at: Instant,
}

pub enum ToastType {
    Success,  // Green, ✓ icon
    Info,     // Blue, ℹ icon
    Warning,  // Yellow, ⚠ icon
    Error,    // Red, ✗ icon
}

impl ToastWidget {
    pub fn new(message: String, toast_type: ToastType, duration: Duration) -> Self

    /// Check if toast expired
    pub fn is_expired(&self) -> bool

    /// Render toast (top-right corner)
    pub fn render(&self, frame: &mut Frame, area: Rect)
}

/// Toast manager (queue multiple toasts)
pub struct ToastManager {
    toasts: VecDeque<ToastWidget>,
    max_toasts: usize,
}

impl ToastManager {
    pub fn show(&mut self, message: String, toast_type: ToastType)
    pub fn show_success(&mut self, message: String)
    pub fn show_error(&mut self, message: String)
    pub fn show_warning(&mut self, message: String)
    pub fn show_info(&mut self, message: String)

    /// Remove expired toasts
    pub fn cleanup_expired(&mut self)

    /// Render all active toasts
    pub fn render_all(&self, frame: &mut Frame, area: Rect)
}
```

**Usage Examples:**
- File imported: "✓ Imported 42 targets from file.txt" (Success)
- Invalid input: "✗ Invalid CIDR notation" (Error)
- DNS failure: "⚠ Failed to resolve example.com" (Warning)
- Template saved: "ℹ Template 'my-scan' saved" (Info)

**Behavior:**
- Toast appears in top-right corner
- Auto-dismiss after 3-5 seconds
- Max 3 toasts visible at once
- Click to dismiss (optional mouse support)

**Tests:** 8 tests
- Show toast (4 types)
- Queue multiple toasts
- Auto-expire (duration)
- Max toasts limit
- Rendering (position, styling)

---

### Task 6.8.4: Final Integration & Documentation (2-4 hours)

#### 6.8.4.1: Integration Testing (1-2h)
**File:** `crates/prtip-tui/tests/integration/phase6_complete.rs` (NEW - ~500 lines)

**Test Scenarios:**
1. **Complete Workflow** (10 tests)
   - Navigate to Target Selection
   - Enter CIDR + import file + DNS
   - Navigate to Port Selection
   - Select preset + custom range
   - Navigate to Template Selection
   - Apply template
   - Validate config
   - Generate CLI args
   - Execute scan (mocked)
   - Display results

2. **Keyboard Navigation** (8 tests)
   - Focus traversal (Tab key)
   - Context switching (1-5 keys in Dashboard)
   - Shortcut execution (all global shortcuts)
   - Help overlay toggle
   - Cheat sheet toggle

3. **State Persistence** (6 tests)
   - Target selection state preserved on tab switch
   - Port selection state preserved
   - Template selection state preserved
   - Shortcut customizations loaded on restart
   - Port presets loaded on restart

4. **Error Handling** (6 tests)
   - Invalid CIDR → Error toast
   - File import failure → Error toast + log
   - Empty targets validation → Warning toast
   - Empty ports validation → Warning toast
   - DNS timeout → Warning toast + partial results
   - Shortcut conflict → Warning + block registration

**Total Sprint 6.8 Tests:** ~70 tests
- Shortcut Manager: 15
- Shortcut Editor: 8
- Cheat Sheet: 3
- Help Tabs: 10
- Help Tooltips: 6
- Help Search: 4
- Focus Manager: 6
- Loading Spinners: 6
- Toast Messages: 8
- Integration: 30

---

#### 6.8.4.2: Documentation Updates (1-2h)

**File 1:** `docs/TUI-ARCHITECTURE.md` (UPDATE - add 200-300 lines)
- Add Sprint 6.7-6.8 sections
- Update widget inventory (11 widgets → 18 widgets)
- Document keyboard shortcut system
- Document help system architecture

**File 2:** `docs/TUI-USER-GUIDE.md` (NEW - ~1,500 lines)
- Getting Started (Quick Start)
- Widget Reference (Target, Port, Template, Dashboard, Help)
- Keyboard Shortcuts (all contexts)
- Workflows (5 common workflows)
- Troubleshooting (10-15 FAQ entries)
- Customization (shortcuts, themes)

**File 3:** `CHANGELOG.md` (UPDATE - add Sprint 6.7-6.8 entry)
```markdown
## [v0.5.7] - 2025-11-XX - Sprint 6.7-6.8: Interactive Selection & TUI Polish

### Added (Sprint 6.7)
- **TargetSelectionWidget Enhancements**:
  - File browser dialog for target import (Ctrl+I)
  - Async DNS resolution with progress tracking
  - Interactive target list with checkbox selection
  - Export selected targets to file (Ctrl+E)
- **PortSelectionWidget** (NEW):
  - Quick presets (Top 100, Top 1000, All, Common Services)
  - Port categories (Web, SSH, Database, Mail, File Sharing, Remote Access)
  - Custom port range input (80,443,8000-9000)
  - Port preset manager (save/load custom presets)
- **TemplateSelectionWidget Enhancements**:
  - Template preview panel (parameters, description)
  - Quick actions (Duplicate, Edit, Delete, Apply, Save)
- **Integration**:
  - ScanConfig state (maps widgets → scanner params)
  - Widget event routing (cross-widget synchronization)
  - Full workflow testing (90 integration tests)

### Added (Sprint 6.8)
- **Keyboard Shortcut System**:
  - Centralized ShortcutManager (global + context-specific bindings)
  - Shortcut customization UI (edit bindings, detect conflicts)
  - Cheat sheet overlay (context-sensitive)
  - Custom bindings persistence (~/.prtip/shortcuts.toml)
- **Enhanced Help System**:
  - Tabbed help (Quick Start, Shortcuts, Widgets, Workflows, Troubleshooting)
  - Context-sensitive tooltips (inline help hints)
  - Help search (fuzzy search across all tabs)
- **UX Polish**:
  - Focus indicators (yellow border, breadcrumb trail, focus history)
  - Loading states (spinners, progress bars)
  - Toast notifications (Success, Info, Warning, Error)
  - Smooth transitions (fade-in/out, slide animations)

### Changed
- Widget count: 11 → 18 (+7 new widgets)
- Keyboard shortcuts: ~30 → 80+ (context-specific)
- Help content: 200 lines → 1,500 lines (comprehensive guide)

### Tests
- Sprint 6.7: +90 tests (File Browser, DNS, Target List, Port Selection, etc.)
- Sprint 6.8: +70 tests (Shortcuts, Help, Focus, Loading, Toasts, Integration)
- Total: 2,246 → 2,406 (+160, +7.1%)
- Coverage: 54.92% → 62% (target: 80%)

### Documentation
- TUI-ARCHITECTURE.md updated (+200 lines)
- TUI-USER-GUIDE.md created (1,500 lines)
- Phase 6 COMPLETE (8/8 sprints)
```

**File 4:** `docs/10-PROJECT-STATUS.md` (UPDATE)
- Mark Phase 6 as COMPLETE
- Update test count (2,246 → 2,406)
- Update Sprint summary (6.7-6.8 entries)
- Update version (v0.5.7)

---

## Definition of Done

### Functional Requirements (Sprint 6.7)
- [ ] File browser dialog working (navigate, select, import)
- [ ] Async DNS resolution with progress tracking
- [ ] Interactive target list with checkbox selection
- [ ] Export targets to file (plain text, JSON, CSV)
- [ ] Port selection widget with presets and categories
- [ ] Port preset manager (save/load custom presets)
- [ ] Template preview panel with parameter details
- [ ] Template quick actions (duplicate, edit, delete, apply, save)
- [ ] ScanConfig integration (widgets → scanner params)
- [ ] Widget event routing (cross-widget synchronization)

### Functional Requirements (Sprint 6.8)
- [ ] Keyboard shortcut system (global + context-specific)
- [ ] Shortcut customization UI (edit, conflict detection)
- [ ] Cheat sheet overlay (context-sensitive)
- [ ] Tabbed help system (5 tabs)
- [ ] Context-sensitive tooltips
- [ ] Help search (fuzzy search)
- [ ] Focus indicators (border, breadcrumb, history)
- [ ] Loading states (spinners, progress bars)
- [ ] Toast notifications (4 types)

### Quality Requirements
- [ ] 160 new tests passing (90 Sprint 6.7 + 70 Sprint 6.8)
- [ ] Zero clippy warnings (--deny warnings)
- [ ] Zero rustdoc warnings
- [ ] Code formatted with `cargo fmt`
- [ ] TUI responsive (<100ms input latency)
- [ ] 60 FPS rendering (16.67ms frame budget)

### Documentation Requirements
- [ ] TUI-ARCHITECTURE.md updated (+200 lines)
- [ ] TUI-USER-GUIDE.md created (1,500 lines)
- [ ] CHANGELOG.md updated (Sprint 6.7-6.8 entry)
- [ ] docs/10-PROJECT-STATUS.md updated (Phase 6 COMPLETE)
- [ ] Rustdoc comments for all new public APIs

### User Experience Requirements
- [ ] All keyboard shortcuts documented and working
- [ ] Focus indicators visible and consistent
- [ ] Error messages clear and actionable
- [ ] Loading states prevent perceived lag
- [ ] Toast notifications auto-dismiss (3-5s)
- [ ] Help system comprehensive and searchable

---

## Risk Mitigation

### Risk 1: File Browser Complexity
**Impact:** Medium | **Probability:** Medium
**Mitigation:**
- Use `std::fs::read_dir()` for directory traversal
- Limit depth (max 10 levels) to prevent infinite loops
- Handle permissions errors gracefully (skip unreadable directories)
- Test on Windows (different path separators)

### Risk 2: Keyboard Shortcut Conflicts
**Impact:** High | **Probability:** Medium
**Mitigation:**
- Conflict detection algorithm (detect overlaps early)
- Warn user before registering conflicting binding
- Allow overrides with confirmation dialog
- Reset to defaults option (escape hatch)

### Risk 3: Help Content Maintenance
**Impact:** Low | **Probability:** High
**Mitigation:**
- Auto-generate shortcut documentation from ShortcutManager
- Use markdown for help content (easy to edit)
- Separate help content into modules (scalable)
- Version help content with code releases

---

## Dependencies

### External Crates
- `crossterm = "0.27"` - Keyboard input, terminal manipulation
- `ratatui = "0.29"` - TUI framework
- `tokio = "1.35"` - Async runtime (DNS resolution)
- `toml = "0.8"` - Config file parsing (shortcuts, presets)

### Internal Dependencies
- **Sprint 6.1 (TUI Framework):** App state, event loop ✅
- **Sprint 6.2 (Live Dashboard):** Widget framework ✅
- **Sprint 6.6 (Memory-Mapped I/O):** Result streaming ✅
- **prtip-core:** ScanConfig, TargetSpec, ScanType, TimingTemplate

---

## Resources

### Documentation
- **ratatui Widgets:** https://ratatui.rs/widgets/
- **crossterm Events:** https://docs.rs/crossterm/latest/crossterm/event/
- **TOML Specification:** https://toml.io/
- **Tokio JoinSet:** https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html

### Reference Implementations
- **Vim Keyboard Shortcuts:** Consistent keybinding conventions
- **Nmap TUI (Zenmap):** GUI workflow inspiration
- **htop:** Focus indicators and keyboard navigation

---

## Sprint Completion Checklist

### Sprint 6.7 Checklist
- [ ] Task 6.7.1: Enhanced Target Selection (6-8h)
  - [ ] 6.7.1.1: File Browser Dialog (2h)
  - [ ] 6.7.1.2: Async DNS Resolution (2h)
  - [ ] 6.7.1.3: Target List Management (2-4h)
- [ ] Task 6.7.2: Port Range Selection Widget (8-10h)
  - [ ] 6.7.2.1: Port Parser & Validator (2h)
  - [ ] 6.7.2.2: Interactive Port Widget (4h)
  - [ ] 6.7.2.3: Port Preset Manager (2-4h)
- [ ] Task 6.7.3: Enhanced Template Selection (2-3h)
  - [ ] 6.7.3.1: Template Preview Panel (1-1.5h)
  - [ ] 6.7.3.2: Template Quick Actions (1-1.5h)
- [ ] Task 6.7.4: Integration & State Sync (4-6h)
  - [ ] 6.7.4.1: ScanConfig Integration (2h)
  - [ ] 6.7.4.2: Widget Event Routing (2h)
  - [ ] 6.7.4.3: Full Workflow Testing (2-4h)

### Sprint 6.8 Checklist
- [ ] Task 6.8.1: Keyboard Shortcut System (6-8h)
  - [ ] 6.8.1.1: Shortcut Manager (3-4h)
  - [ ] 6.8.1.2: Customization UI (2-3h)
  - [ ] 6.8.1.3: Cheat Sheet Overlay (1-2h)
- [ ] Task 6.8.2: Enhanced Help System (4-6h)
  - [ ] 6.8.2.1: Tabbed Help System (2-3h)
  - [ ] 6.8.2.2: Context-Sensitive Tooltips (1-2h)
  - [ ] 6.8.2.3: Help Search (1h)
- [ ] Task 6.8.3: UX Polish (4-6h)
  - [ ] 6.8.3.1: Focus Indicators (2h)
  - [ ] 6.8.3.2: Loading States & Spinners (1-2h)
  - [ ] 6.8.3.3: Error Handling & Toasts (1-2h)
- [ ] Task 6.8.4: Final Integration & Documentation (2-4h)
  - [ ] 6.8.4.1: Integration Testing (1-2h)
  - [ ] 6.8.4.2: Documentation Updates (1-2h)

### Final Verification
- [ ] All 160 new tests passing (100% success rate)
- [ ] cargo clippy -- -D warnings (zero warnings)
- [ ] cargo fmt --check (all files formatted)
- [ ] TUI responsive at 60 FPS (no frame drops)
- [ ] Keyboard navigation complete (all widgets accessible)
- [ ] Help system comprehensive (all features documented)
- [ ] Phase 6 marked COMPLETE (8/8 sprints)

---

**Phase 6 completion marks the end of TUI development - ProRT-IP will have a production-ready terminal interface rivaling commercial network scanners. Prioritize user experience polish over raw feature count.**
