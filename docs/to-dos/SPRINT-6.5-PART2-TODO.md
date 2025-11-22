# Sprint 6.5 Part 2: Interactive Target Selection & Templates

**Sprint ID:** 6.5 Part 2
**Phase:** 6 (TUI + Network Optimizations)
**Priority:** HIGH
**Status:** IN PROGRESS
**Created:** 2025-11-21
**Estimated Duration:** 16-24 hours (2-3 days)

---

## Executive Summary

### Sprint Goals

Sprint 6.5 Part 2 implements **Interactive Target Selection & Scan Template Management** for ProRT-IP's TUI interface, enabling users to:

1. **Interactive CIDR Calculator** - Visual IP range specification with real-time validation
2. **Target Import/Export** - Load targets from CSV/TXT files, export target lists
3. **Exclusion List Management** - Visual exclude/include target management
4. **DNS Resolution Preview** - Preview DNS-to-IP resolution before scanning
5. **Scan Template Browser** - Visual selection of built-in and custom templates
6. **Target Count Estimation** - Real-time display of total targets to scan

**Strategic Impact:** Transforms ProRT-IP from CLI-only to professional TUI-first network scanner with enhanced usability and feature discoverability.

### Context from Sprint 6.5 Part 1

Sprint 6.5 Part 1 COMPLETE (Nov 21, 2025):
- ✅ Plugin System Lua Callbacks (6 callbacks functional)
- ✅ Idle Scan IPID Tracking (real IPID values, not stub 0)
- ✅ Decoy Scanner Integration (BatchSender/BatchReceiver)
- ✅ 27 new tests, 2,260 total tests passing
- ✅ Quality gates: 0 clippy warnings, clean formatting

Part 2 builds on established TUI architecture from Sprint 6.1-6.3.

### Success Criteria

✅ Interactive target selection widget with CIDR calculator
✅ File import/export for target lists (CSV, TXT formats)
✅ Exclusion list visual management
✅ DNS resolution preview with caching
✅ Scan template browser with 10 built-in templates
✅ Target count estimation with validation
✅ 30+ new tests (widget unit tests + integration tests)
✅ 2,260+ tests passing (zero regressions)
✅ 0 clippy warnings across all crates
✅ Documentation comprehensive and updated

### Effort Estimates

| Scenario | Hours | Probability |
|----------|-------|-------------|
| **Optimistic** | 16 | 25% - Straightforward widget implementation |
| **Realistic** | 20 | 60% - Standard TUI integration challenges |
| **Pessimistic** | 24 | 15% - Complex state management issues |

**Recommended Time Budget:** 20-22 hours over 2-3 days

### Risk Assessment

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| Complex state management | MEDIUM | 30% | Follow existing PortTableWidget patterns |
| File I/O errors | MEDIUM | 25% | Comprehensive error handling, user feedback |
| DNS resolution blocking | LOW | 15% | Async resolution with tokio, timeout handling |
| Template parsing errors | LOW | 10% | Existing validation in template system |

---

## Requirements Analysis

### Feature Specification from Roadmap

**Source:** docs/01-ROADMAP.md lines 832-846

Sprint 6.5 Requirements:
1. Interactive CIDR calculator
2. Target import from files (CSV, TXT)
3. Exclusion list management
4. DNS resolution preview
5. Target count estimation

**Additional Context from Scan Templates:**
- 10 built-in templates (web-servers, databases, quick, thorough, stealth, discovery, ssl-only, admin-panels, mail-servers, file-shares)
- Custom template support via `~/.prtip/templates.toml`
- Template validation and override behavior

### User Stories

**US-1: As a security analyst, I want to visually specify CIDR ranges so I can avoid typos and validate network ranges before scanning.**

**Acceptance Criteria:**
- CIDR input field with real-time validation
- Visual feedback for valid/invalid CIDR notation
- Display calculated IP range (e.g., "192.168.1.0/24 = 256 hosts")
- Support for IPv4 and IPv6 CIDR notation
- Error messages for invalid input

**US-2: As a penetration tester, I want to load target lists from files so I can reuse target specifications across multiple scans.**

**Acceptance Criteria:**
- File browser/selector for CSV and TXT files
- Preview loaded targets before import
- Support for multiple formats: IP per line, CIDR ranges, IP ranges
- Error handling for invalid file formats
- Import confirmation dialog

**US-3: As a network administrator, I want to exclude specific IPs/ranges from scans so I can avoid critical infrastructure.**

**Acceptance Criteria:**
- Visual list of excluded targets
- Add/remove exclusion rules
- Validate exclusions against target list
- Display effective target count after exclusions
- Save/load exclusion lists

**US-4: As a user, I want to preview DNS resolution results so I can verify targets before scanning.**

**Acceptance Criteria:**
- Display hostname → IP resolution
- Async resolution with progress indicator
- Caching to avoid repeated lookups
- Error handling for failed resolutions
- Timeout for slow DNS servers

**US-5: As a scanner user, I want to browse and select scan templates visually so I can discover pre-configured scanning scenarios.**

**Acceptance Criteria:**
- List of 10 built-in templates
- Display custom templates from ~/.prtip/templates.toml
- Show template details (ports, scan type, timing, description)
- Template selection with visual highlight
- Override template settings with custom values

### Technical Requirements

**TR-1: State Management**
- New UIState fields: `target_input`, `selected_template`, `exclusion_list`, `dns_cache`
- Validation state for CIDR inputs
- File import state (path, preview data)

**TR-2: Widget Architecture**
- TargetSelectionWidget: CIDR calculator, file import, exclusion management
- TemplateSelectionWidget: Template browser with details panel
- Follow existing Component trait pattern (render, handle_key, update)

**TR-3: Integration Points**
- EventBus integration for target updates
- File system integration for template loading
- DNS resolver integration (tokio_dns or trust-dns-resolver)
- Scanner configuration updates

**TR-4: Performance**
- Widget rendering: <5ms (within 16.67ms frame budget)
- DNS resolution: Async, non-blocking, 5s timeout
- Template loading: <100ms (cached after first load)
- File import: <200ms for 10,000 targets

---

## Architecture Design

### Widget Structure

#### TargetSelectionWidget

**Purpose:** Interactive target specification with CIDR calculator, file import, and exclusion management

**Location:** `crates/prtip-tui/src/widgets/target_selection.rs` (~800 lines estimated)

**Data Structure:**
```rust
pub struct TargetSelectionWidget {
    // Input state
    cidr_input: String,              // Current CIDR input text
    validation_state: ValidationState, // Valid, Invalid(error_msg), Empty

    // Calculated targets
    calculated_ips: Vec<IpAddr>,      // IPs from current CIDR
    target_count: usize,              // Total targets after exclusions

    // File import
    import_file_path: Option<PathBuf>,
    imported_targets: Vec<IpAddr>,

    // Exclusion management
    exclusion_list: Vec<String>,      // CIDR ranges to exclude
    excluded_ips: HashSet<IpAddr>,    // Calculated excluded IPs

    // DNS resolution
    dns_cache: HashMap<String, Result<Vec<IpAddr>, String>>,
    dns_pending: HashSet<String>,

    // UI state
    cursor_position: usize,
    selected_section: Section,        // CIDR | Import | Exclusions | DNS
    scroll_offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationState {
    Empty,
    Valid { range: String, count: usize },
    Invalid { error: String },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Section {
    CidrInput,
    FileImport,
    ExclusionList,
    DnsResolution,
}
```

**Key Methods:**
```rust
impl TargetSelectionWidget {
    pub fn new() -> Self;
    pub fn set_cidr(&mut self, cidr: &str) -> Result<()>;
    pub fn validate_cidr(&self, cidr: &str) -> ValidationState;
    pub fn calculate_ip_range(&self, cidr: &str) -> Result<Vec<IpAddr>>;
    pub fn import_from_file(&mut self, path: &Path) -> Result<usize>;
    pub fn export_to_file(&self, path: &Path) -> Result<()>;
    pub fn add_exclusion(&mut self, exclusion: String);
    pub fn remove_exclusion(&mut self, index: usize);
    pub fn resolve_hostname(&mut self, hostname: String) -> tokio::task::JoinHandle<Result<Vec<IpAddr>>>;
    pub fn get_effective_targets(&self) -> Vec<IpAddr>;
}

impl Component for TargetSelectionWidget {
    fn render(&mut self, frame: &mut Frame, area: Rect);
    fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<()>;
    fn update(&mut self) -> anyhow::Result<()>;
}
```

**Layout:**
```
┌─────────────────────────────────────────────────────────────┐
│ Target Selection                                            │
├─────────────────────────────────────────────────────────────┤
│ CIDR Input:                                                 │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 192.168.1.0/24                                          │ │
│ └─────────────────────────────────────────────────────────┘ │
│ ✓ Valid: 192.168.1.0 - 192.168.1.255 (256 hosts)            │
│                                                             │
│ Import Targets: [F] File Browser                            │
│ Loaded: 150 targets from targets.txt                        │
│                                                             │
│ Exclusions: (2 ranges)                                      │
│ - 192.168.1.1 (gateway)                                     │
│ - 192.168.1.0/28 (management subnet)                        │
│                                                             │
│ DNS Resolution:                                             │
│ example.com → 93.184.216.34 ✓                               │
│ target.local → Resolving... ⏳                              │
│                                                             │
│ Effective Targets: 238 hosts                                │
│ [Tab] Switch Section | [Enter] Confirm | [Esc] Cancel       │
└─────────────────────────────────────────────────────────────┘
```

**Keyboard Shortcuts:**
- `Tab`: Switch between sections (CIDR → Import → Exclusions → DNS)
- `Enter`: Confirm target selection, start scan
- `f`: Open file browser for import
- `e`: Add exclusion rule
- `d`: Remove selected exclusion
- `r`: Resolve hostname (DNS section)
- `Esc`: Cancel/close widget

#### TemplateSelectionWidget

**Purpose:** Visual browser for built-in and custom scan templates

**Location:** `crates/prtip-tui/src/widgets/template_selection.rs` (~600 lines estimated)

**Data Structure:**
```rust
pub struct TemplateSelectionWidget {
    // Template data
    builtin_templates: Vec<ScanTemplate>,  // 10 built-in templates
    custom_templates: Vec<ScanTemplate>,   // From ~/.prtip/templates.toml

    // UI state
    selected_index: usize,
    scroll_offset: usize,
    show_details: bool,                     // Show full template details

    // Filter state
    filter_text: String,
    filtered_templates: Vec<usize>,         // Indices of visible templates
}

#[derive(Debug, Clone)]
pub struct ScanTemplate {
    pub name: String,
    pub description: String,
    pub ports: Option<Vec<u16>>,
    pub scan_type: Option<ScanType>,
    pub service_detection: Option<bool>,
    pub os_detection: Option<bool>,
    pub timing: Option<String>,
    pub max_rate: Option<u64>,
    pub randomize: Option<bool>,
    pub fragment: Option<bool>,
    pub tls_analysis: Option<bool>,
    pub discovery_only: Option<bool>,
    pub is_custom: bool,                     // Built-in vs custom
}
```

**Key Methods:**
```rust
impl TemplateSelectionWidget {
    pub fn new() -> Self;
    pub fn load_builtin_templates() -> Vec<ScanTemplate>;
    pub fn load_custom_templates() -> Result<Vec<ScanTemplate>>;
    pub fn reload_templates(&mut self) -> Result<()>;
    pub fn filter(&mut self, text: &str);
    pub fn get_selected_template(&self) -> Option<&ScanTemplate>;
    pub fn apply_template(&self, config: &mut ScanConfig);
}

impl Component for TemplateSelectionWidget {
    fn render(&mut self, frame: &mut Frame, area: Rect);
    fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<()>;
    fn update(&mut self) -> anyhow::Result<()>;
}
```

**Layout:**
```
┌─────────────────────────────────────────────────────────────┐
│ Scan Templates                                 [10 Built-in]│
├─────────────────┬───────────────────────────────────────────┤
│ Templates       │ Template Details                          │
├─────────────────┼───────────────────────────────────────────┤
│ web-servers  ●  │ Name: web-servers                         │
│ databases       │ Description: Scan common web server ports │
│ quick           │ with service and TLS detection            │
│ thorough        │                                           │
│ stealth         │ Ports: 80, 443, 8080, 8443, 3000, 5000,   │
│ discovery       │        8000, 8888                         │
│ ssl-only        │ Scan Type: SYN                            │
│ admin-panels    │ Service Detection: ✓ Enabled              │
│ mail-servers    │ TLS Analysis: ✓ Enabled                   │
│ file-shares     │ Timing: T3 (Normal)                       │
│                 │                                           │
│ [Custom]        │ Estimated Time: 2-5 seconds/host          │
│ internal-svcs   │ Network Impact: Very Low                  │
│                 │                                           │
├─────────────────┴───────────────────────────────────────────┤
│ [↑/↓] Navigate | [Enter] Select | [d] Details | [/] Filter  │
└─────────────────────────────────────────────────────────────┘
```

**Keyboard Shortcuts:**
- `↑/↓` or `j/k`: Navigate template list
- `Enter`: Select template and apply to scan configuration
- `d`: Toggle detailed view
- `/`: Filter templates by name/description
- `Esc`: Close template selector

### State Management Approach

**UIState Extensions:**
```rust
// crates/prtip-tui/src/state/ui_state.rs

pub struct UIState {
    // ... existing fields ...

    // NEW FIELDS for Sprint 6.5 Part 2:
    pub target_selection_active: bool,
    pub template_selection_active: bool,
    pub target_widget: TargetSelectionWidget,
    pub template_widget: TemplateSelectionWidget,
}
```

**Integration with App Event Loop:**
- Modal widget pattern: Overlay target/template selection on main dashboard
- Keyboard event routing: Active widget receives events first
- State updates: Widgets update ScanConfig upon confirmation
- EventBus integration: Publish TargetSelectionComplete, TemplateSelectionComplete events

### Event Handling Flow

```
User presses 't' (target selection)
    ↓
App::handle_key() detects 't'
    ↓
ui_state.target_selection_active = true
    ↓
Event loop routes keys to TargetSelectionWidget
    ↓
User interacts with widget (CIDR input, file import, exclusions)
    ↓
User presses Enter to confirm
    ↓
TargetSelectionWidget::apply_to_config(scan_config)
    ↓
EventBus publishes TargetSelectionComplete { targets: Vec<IpAddr> }
    ↓
ui_state.target_selection_active = false
    ↓
Return to main dashboard
```

### Integration Points

**1. Scanner Configuration:**
```rust
// Apply target selection to scan configuration
let targets = target_widget.get_effective_targets();
scan_config.set_targets(targets);

// Apply template to scan configuration
template_widget.apply_template(&mut scan_config);
```

**2. File System Integration:**
```rust
// Load custom templates from ~/.prtip/templates.toml
let custom_templates = TemplateLoader::load_from_config()?;

// Import targets from file
let imported = TargetImporter::import_from_csv(path)?;
let imported = TargetImporter::import_from_txt(path)?;
```

**3. DNS Resolution Integration:**
```rust
// Async DNS resolution with tokio
use tokio_dns::TokioAsyncResolver;

async fn resolve_hostname(hostname: String) -> Result<Vec<IpAddr>> {
    let resolver = TokioAsyncResolver::tokio_from_system_conf()?;
    let response = resolver.lookup_ip(hostname).await?;
    Ok(response.iter().collect())
}
```

**4. EventBus Integration:**
```rust
// Publish events for target/template selection
event_bus.publish(ScanEvent::TargetSelectionComplete {
    target_count: targets.len(),
    excluded_count: exclusions.len(),
}).await;

event_bus.publish(ScanEvent::TemplateApplied {
    template_name: template.name.clone(),
    settings: template.to_settings(),
}).await;
```

---

## Implementation Plan

### Task 1: TargetSelectionWidget - CIDR Calculator (6-8 hours)

**Sub-Task 1.1: Basic widget structure** (1.5 hours)
- Create `crates/prtip-tui/src/widgets/target_selection.rs`
- Implement Component trait skeleton
- Add to `widgets/mod.rs` exports
- Create basic render method with placeholder layout

**Sub-Task 1.2: CIDR input and validation** (2 hours)
- Text input field with cursor management
- Real-time CIDR validation using `ipnetwork` crate
- Calculate IP range from valid CIDR
- Display validation state (Valid, Invalid, Empty)
- Error messages for invalid CIDR notation

**Sub-Task 1.3: IP range calculation and display** (1.5 hours)
- Calculate all IPs in CIDR range (IPv4 and IPv6)
- Display range summary (e.g., "192.168.1.0 - 192.168.1.255 (256 hosts)")
- Handle edge cases: /32 (single IP), /0 (all IPs - warn user)
- Efficient calculation for large ranges (don't enumerate all IPs for /8)

**Sub-Task 1.4: Keyboard input handling** (1 hour)
- Character input for CIDR field
- Backspace, delete, arrow keys
- Tab navigation between sections
- Enter to confirm, Esc to cancel

**Sub-Task 1.5: Unit tests** (2 hours)
- Test CIDR validation logic (valid/invalid cases)
- Test IP range calculation (IPv4 and IPv6)
- Test edge cases (/32, /0, invalid notation)
- Test keyboard input handling
- **Minimum 8 tests**

### Task 2: TargetSelectionWidget - File Import/Export (4-5 hours)

**Sub-Task 2.1: File format parsing** (2 hours)
- CSV parser: IP per line, CIDR per line
- TXT parser: Same formats, flexible whitespace
- IP range parser: "192.168.1.1-192.168.1.254"
- Error handling for invalid formats
- Preview loaded targets (first 100 for large files)

**Sub-Task 2.2: File browser integration** (1 hour)
- Simple file path input (no visual browser initially)
- Path validation and error messages
- Display loaded file path and target count
- Support for relative and absolute paths

**Sub-Task 2.3: Export functionality** (1 hour)
- Export effective targets to CSV
- Export effective targets to TXT
- Include exclusions in export header (commented)
- Success/failure feedback to user

**Sub-Task 2.4: Unit tests** (1.5 hours)
- Test CSV parsing (valid and invalid files)
- Test TXT parsing
- Test IP range parsing
- Test export functionality
- **Minimum 6 tests**

### Task 3: TargetSelectionWidget - Exclusion Management (3-4 hours)

**Sub-Task 3.1: Exclusion list data structure** (1 hour)
- Vec<String> for exclusion rules (CIDR notation)
- HashSet<IpAddr> for efficient lookup
- Recalculate excluded IPs when rules change
- Display exclusion list in widget

**Sub-Task 3.2: Add/remove exclusions** (1.5 hours)
- Input field for new exclusion rule
- Validate exclusion CIDR before adding
- Remove selected exclusion with keyboard shortcut
- Display effective target count after exclusions

**Sub-Task 3.3: Exclusion application logic** (1 hour)
- Filter targets by exclusion list
- Display excluded count vs total
- Warn if all targets excluded

**Sub-Task 3.4: Unit tests** (1 hour)
- Test exclusion rule validation
- Test exclusion application (filtering)
- Test add/remove operations
- **Minimum 4 tests**

### Task 4: TargetSelectionWidget - DNS Resolution (2-3 hours)

**Sub-Task 4.1: Async DNS integration** (1.5 hours)
- Add `trust-dns-resolver` dependency
- Implement async hostname resolution
- Timeout handling (5 seconds default)
- Caching resolved IPs in HashMap

**Sub-Task 4.2: Resolution UI** (1 hour)
- Input field for hostname
- Display resolution status (Pending, Success, Failed)
- Show resolved IPs for successful resolutions
- Error messages for failed resolutions

**Sub-Task 4.3: Unit tests** (1 hour)
- Test resolution caching
- Test timeout handling (mock)
- Test error handling (invalid hostname)
- **Minimum 3 tests**

### Task 5: TemplateSelectionWidget (5-6 hours)

**Sub-Task 5.1: Template data loading** (2 hours)
- Load 10 built-in templates (hardcoded ScanTemplate structs)
- Load custom templates from ~/.prtip/templates.toml
- TOML parsing with `toml` crate
- Template validation (existing validation logic)

**Sub-Task 5.2: Template browser widget** (2 hours)
- List rendering with scroll support
- Template selection with keyboard navigation
- Highlight selected template
- Details panel showing template configuration

**Sub-Task 5.3: Template application** (1 hour)
- Apply selected template to ScanConfig
- Merge template settings with existing config
- Override behavior (CLI flags > custom > built-in)

**Sub-Task 5.4: Unit tests** (1.5 hours)
- Test built-in template loading
- Test custom template parsing
- Test template application to config
- Test filtering by name/description
- **Minimum 6 tests**

### Task 6: Integration and Testing (3-4 hours)

**Sub-Task 6.1: App integration** (2 hours)
- Add target_selection_active and template_selection_active to UIState
- Keyboard shortcuts to open widgets ('t' for targets, 'p' for templates)
- Event routing to active widget
- Modal overlay rendering

**Sub-Task 6.2: EventBus integration** (1 hour)
- Publish TargetSelectionComplete event
- Publish TemplateApplied event
- Update ScanState with target count, template name

**Sub-Task 6.3: Integration tests** (1.5 hours)
- Test full widget lifecycle (open → interact → confirm → close)
- Test event publishing
- Test state updates
- **Minimum 3 integration tests**

### Task 7: Documentation and Completion (2-3 hours)

**Sub-Task 7.1: Rustdoc comments** (1 hour)
- Document all public methods
- Add usage examples for key methods
- Document keyboard shortcuts

**Sub-Task 7.2: Completion report** (1 hour)
- Create SPRINT-6.5-PART2-COMPLETE.md
- Document implementation details
- Include test results and quality metrics
- Strategic value and impact

**Sub-Task 7.3: Documentation updates** (1 hour)
- Update CHANGELOG.md with Sprint 6.5 Part 2 section
- Update README.md if TUI features mentioned
- Update CLAUDE.local.md (Recent Decisions, Recent Sessions)
- Update TUI-ARCHITECTURE.md with new widgets

---

## Quality Gates

### Code Quality

```bash
# Formatting (must be clean)
cargo fmt --all -- --check

# Linting (zero warnings required)
cargo clippy --workspace --all-targets --locked -- -D warnings

# Build (must succeed)
cargo build --release --workspace --locked

# Documentation (no warnings)
cargo doc --workspace --no-deps --locked
```

**Success Criteria:** All commands exit with code 0, zero warnings/errors.

### Test Suite

```bash
# Full test suite (no root required)
cargo test --workspace --lib --bins --tests --locked

# Target: 2,260+ tests passing (maintain v0.6.1 baseline)
# New tests: 30+ (TargetSelectionWidget + TemplateSelectionWidget)
```

**Test Breakdown:**
- TargetSelectionWidget unit tests: 21 minimum
  - CIDR validation: 8 tests
  - File import/export: 6 tests
  - Exclusion management: 4 tests
  - DNS resolution: 3 tests
- TemplateSelectionWidget unit tests: 6 minimum
- Integration tests: 3 minimum
- **Total new tests: 30 minimum**

**Success Criteria:**
- 2,290+ tests passing (2,260 baseline + 30 new)
- No regressions from Sprint 6.5 Part 1
- Test execution time ≤120 seconds (CI/CD timeout)

### Coverage

```bash
# Generate coverage report
cargo tarpaulin --workspace --locked --timeout 300 --out Cobertura
```

**Success Criteria:**
- Overall coverage: ≥54.92% (maintain baseline)
- New widgets: ≥70% coverage
- Core methods: ≥80% coverage

### Performance

**Widget Rendering Benchmarks:**
- TargetSelectionWidget render: <5ms
- TemplateSelectionWidget render: <5ms
- CIDR calculation (10,000 IPs): <10ms
- File import (10,000 targets): <200ms
- DNS resolution: 5s timeout (async, non-blocking)
- Template loading: <100ms (first load), <1ms (cached)

**Success Criteria:**
- All rendering within 16.67ms frame budget (60 FPS)
- No UI blocking during file I/O or DNS resolution
- Smooth scrolling with large lists (1,000+ items)

---

## File Structure

### New Files

```
crates/prtip-tui/src/widgets/target_selection.rs    (~800 lines)
crates/prtip-tui/src/widgets/template_selection.rs  (~600 lines)
docs/to-dos/SPRINT-6.5-PART2-TODO.md                (this file)
/tmp/ProRT-IP/SPRINT-6.5-PART2-COMPLETE.md          (completion report)
```

### Modified Files

```
crates/prtip-tui/src/widgets/mod.rs                 (exports)
crates/prtip-tui/src/state/ui_state.rs              (new fields)
crates/prtip-tui/src/app.rs                         (keyboard shortcuts)
crates/prtip-tui/src/events/loop.rs                 (event routing)
crates/prtip-tui/src/ui/renderer.rs                 (modal rendering)
crates/prtip-tui/Cargo.toml                         (dependencies)
CHANGELOG.md                                         (Sprint 6.5 Part 2)
README.md                                            (TUI features update)
docs/TUI-ARCHITECTURE.md                            (widget documentation)
docs/CLAUDE.local.md                                (Recent Decisions, Sessions)
```

### Dependencies

```toml
# crates/prtip-tui/Cargo.toml

[dependencies]
# ... existing dependencies ...

# NEW for Sprint 6.5 Part 2:
ipnetwork = "0.20"           # CIDR parsing and validation
trust-dns-resolver = "0.23"  # Async DNS resolution
csv = "1.3"                  # CSV file parsing
toml = "0.8"                 # TOML template parsing
```

---

## Testing Strategy

### Unit Tests

**TargetSelectionWidget Tests (21 minimum):**

**CIDR Validation (8 tests):**
```rust
#[test]
fn test_cidr_validation_ipv4_valid() {
    let widget = TargetSelectionWidget::new();
    let state = widget.validate_cidr("192.168.1.0/24");
    assert!(matches!(state, ValidationState::Valid { .. }));
}

#[test]
fn test_cidr_validation_ipv4_invalid() {
    let widget = TargetSelectionWidget::new();
    let state = widget.validate_cidr("192.168.1.0/33"); // Invalid prefix
    assert!(matches!(state, ValidationState::Invalid { .. }));
}

#[test]
fn test_cidr_validation_ipv6_valid() {
    let widget = TargetSelectionWidget::new();
    let state = widget.validate_cidr("2001:db8::/32");
    assert!(matches!(state, ValidationState::Valid { .. }));
}

#[test]
fn test_cidr_calculation_small_range() {
    let widget = TargetSelectionWidget::new();
    let ips = widget.calculate_ip_range("192.168.1.0/30").unwrap();
    assert_eq!(ips.len(), 4); // 192.168.1.0-3
}

#[test]
fn test_cidr_calculation_single_ip() {
    let widget = TargetSelectionWidget::new();
    let ips = widget.calculate_ip_range("192.168.1.1/32").unwrap();
    assert_eq!(ips.len(), 1);
}

#[test]
fn test_cidr_empty_input() {
    let widget = TargetSelectionWidget::new();
    let state = widget.validate_cidr("");
    assert!(matches!(state, ValidationState::Empty));
}

#[test]
fn test_cidr_invalid_format() {
    let widget = TargetSelectionWidget::new();
    let state = widget.validate_cidr("not-a-cidr");
    assert!(matches!(state, ValidationState::Invalid { .. }));
}

#[test]
fn test_cidr_ipv6_calculation() {
    let widget = TargetSelectionWidget::new();
    let ips = widget.calculate_ip_range("2001:db8::/126").unwrap();
    assert_eq!(ips.len(), 4);
}
```

**File Import/Export (6 tests):**
```rust
#[test]
fn test_csv_import_valid_file() {
    let mut widget = TargetSelectionWidget::new();
    let count = widget.import_from_file(Path::new("tests/fixtures/targets.csv")).unwrap();
    assert!(count > 0);
}

#[test]
fn test_txt_import_cidr_format() {
    let mut widget = TargetSelectionWidget::new();
    let count = widget.import_from_file(Path::new("tests/fixtures/targets_cidr.txt")).unwrap();
    assert!(count > 0);
}

#[test]
fn test_import_invalid_file() {
    let mut widget = TargetSelectionWidget::new();
    let result = widget.import_from_file(Path::new("nonexistent.csv"));
    assert!(result.is_err());
}

#[test]
fn test_export_to_csv() {
    let widget = TargetSelectionWidget::new();
    widget.set_cidr("192.168.1.0/30").unwrap();
    let result = widget.export_to_file(Path::new("/tmp/export.csv"));
    assert!(result.is_ok());
}

#[test]
fn test_import_ip_range_format() {
    let mut widget = TargetSelectionWidget::new();
    // Test "192.168.1.1-192.168.1.10" format
    // Implementation needed in import logic
}

#[test]
fn test_import_mixed_formats() {
    let mut widget = TargetSelectionWidget::new();
    // Test file with CIDR, IP ranges, and single IPs
}
```

**Exclusion Management (4 tests):**
```rust
#[test]
fn test_add_exclusion() {
    let mut widget = TargetSelectionWidget::new();
    widget.add_exclusion("192.168.1.1".to_string());
    assert_eq!(widget.exclusion_list.len(), 1);
}

#[test]
fn test_remove_exclusion() {
    let mut widget = TargetSelectionWidget::new();
    widget.add_exclusion("192.168.1.1".to_string());
    widget.remove_exclusion(0);
    assert_eq!(widget.exclusion_list.len(), 0);
}

#[test]
fn test_exclusion_filtering() {
    let mut widget = TargetSelectionWidget::new();
    widget.set_cidr("192.168.1.0/24").unwrap();
    widget.add_exclusion("192.168.1.0/28".to_string()); // Exclude first 16 IPs
    let targets = widget.get_effective_targets();
    assert_eq!(targets.len(), 256 - 16); // 240 targets
}

#[test]
fn test_exclusion_all_targets() {
    let mut widget = TargetSelectionWidget::new();
    widget.set_cidr("192.168.1.0/30").unwrap();
    widget.add_exclusion("192.168.1.0/30".to_string()); // Exclude all
    let targets = widget.get_effective_targets();
    assert_eq!(targets.len(), 0);
}
```

**DNS Resolution (3 tests):**
```rust
#[test]
fn test_dns_cache_hit() {
    let mut widget = TargetSelectionWidget::new();
    widget.dns_cache.insert("example.com".to_string(), Ok(vec!["93.184.216.34".parse().unwrap()]));
    // Verify cache lookup instead of resolution
}

#[test]
fn test_dns_cache_miss() {
    let widget = TargetSelectionWidget::new();
    assert!(widget.dns_cache.get("newhost.com").is_none());
}

#[test]
fn test_dns_error_handling() {
    let mut widget = TargetSelectionWidget::new();
    widget.dns_cache.insert("invalid.local".to_string(), Err("Resolution failed".to_string()));
    // Verify error state displayed in UI
}
```

**TemplateSelectionWidget Tests (6 minimum):**
```rust
#[test]
fn test_builtin_templates_loaded() {
    let widget = TemplateSelectionWidget::new();
    assert_eq!(widget.builtin_templates.len(), 10);
}

#[test]
fn test_custom_template_loading() {
    // Mock ~/.prtip/templates.toml
    let widget = TemplateSelectionWidget::new();
    widget.reload_templates().unwrap();
    // Verify custom templates loaded
}

#[test]
fn test_template_filtering() {
    let mut widget = TemplateSelectionWidget::new();
    widget.filter("web");
    // Verify only web-related templates visible
}

#[test]
fn test_template_selection() {
    let widget = TemplateSelectionWidget::new();
    let template = widget.get_selected_template().unwrap();
    assert_eq!(template.name, "web-servers"); // Default first template
}

#[test]
fn test_template_application() {
    let widget = TemplateSelectionWidget::new();
    let mut config = ScanConfig::default();
    widget.apply_template(&mut config);
    // Verify config updated with template settings
}

#[test]
fn test_template_validation() {
    // Test invalid template TOML parsing
    // Verify validation errors caught
}
```

### Integration Tests

**Location:** `crates/prtip-tui/tests/target_template_integration.rs`

**Test 1: Full target selection workflow**
```rust
#[tokio::test]
async fn test_target_selection_workflow() {
    let mut app = App::new(Arc::new(EventBus::new(1000)));

    // Open target selection
    app.ui_state.target_selection_active = true;

    // Input CIDR
    app.ui_state.target_widget.set_cidr("192.168.1.0/24").unwrap();

    // Add exclusion
    app.ui_state.target_widget.add_exclusion("192.168.1.1".to_string());

    // Confirm selection
    let targets = app.ui_state.target_widget.get_effective_targets();
    assert_eq!(targets.len(), 255); // 256 - 1 excluded

    // Close widget
    app.ui_state.target_selection_active = false;
}
```

**Test 2: Template selection workflow**
```rust
#[tokio::test]
async fn test_template_selection_workflow() {
    let mut app = App::new(Arc::new(EventBus::new(1000)));

    // Open template selection
    app.ui_state.template_selection_active = true;

    // Navigate to template
    app.ui_state.template_widget.selected_index = 2; // "quick" template

    // Apply template
    let mut config = ScanConfig::default();
    app.ui_state.template_widget.apply_template(&mut config);

    // Verify template applied
    assert_eq!(config.timing, Some("T4".to_string()));

    // Close widget
    app.ui_state.template_selection_active = false;
}
```

**Test 3: EventBus integration**
```rust
#[tokio::test]
async fn test_eventbus_integration() {
    let event_bus = Arc::new(EventBus::new(1000));
    let mut subscriber = event_bus.subscribe().await;

    // Simulate target selection confirmation
    event_bus.publish(ScanEvent::TargetSelectionComplete {
        target_count: 256,
        excluded_count: 1,
    }).await;

    // Verify event received
    let event = subscriber.recv().await.unwrap();
    assert!(matches!(event, ScanEvent::TargetSelectionComplete { .. }));
}
```

---

## Dependencies and Prerequisites

### Rust Crates

**Required:**
- `ipnetwork = "0.20"` - CIDR parsing and validation
- `trust-dns-resolver = "0.23"` - Async DNS resolution
- `csv = "1.3"` - CSV file parsing
- `toml = "0.8"` - TOML template parsing

**Already Available:**
- `ratatui = "0.29"` - TUI framework
- `crossterm` - Terminal manipulation
- `tokio` - Async runtime
- `parking_lot` - RwLock for state

### System Requirements

**Development:**
- Rust 1.70+ (MSRV)
- tokio async runtime
- Access to filesystem (for templates, target files)
- DNS resolver (system DNS configuration)

**Testing:**
- No root privileges required for unit tests
- Filesystem access for test fixtures
- Mock DNS responses for deterministic tests

### Existing Infrastructure

**TUI Framework (Sprint 6.1-6.3):**
- Component trait pattern established
- Event loop with tokio::select!
- State management with Arc<RwLock<>>
- 60 FPS rendering pipeline
- 7 existing widgets (StatusBar, MainWidget, LogWidget, HelpWidget, PortTableWidget, ServiceTableWidget, MetricsDashboardWidget)

**Template System (Phase 5.5.2):**
- 10 built-in templates already implemented
- Template validation logic exists
- TOML parsing for custom templates
- Template override behavior

**Scanner Configuration:**
- ScanConfig struct with all scan parameters
- CLI flag parsing and merging
- Config file support

---

## Acceptance Criteria Checklist

### Functionality

- [ ] TargetSelectionWidget renders correctly
- [ ] CIDR input accepts and validates IPv4 and IPv6
- [ ] IP range calculation accurate (verified with tests)
- [ ] File import supports CSV and TXT formats
- [ ] Exclusion list management functional
- [ ] DNS resolution async and non-blocking
- [ ] TemplateSelectionWidget renders correctly
- [ ] 10 built-in templates loaded
- [ ] Custom templates loaded from ~/.prtip/templates.toml
- [ ] Template details panel displays all settings
- [ ] Template application updates ScanConfig correctly

### Quality

- [ ] All unit tests passing (30+ new tests)
- [ ] All integration tests passing (3 minimum)
- [ ] 2,290+ total tests passing
- [ ] 0 clippy warnings
- [ ] Clean formatting (cargo fmt)
- [ ] Coverage ≥70% for new widgets
- [ ] Rustdoc comments for all public methods

### Performance

- [ ] Widget rendering <5ms (both widgets)
- [ ] CIDR calculation <10ms for 10,000 IPs
- [ ] File import <200ms for 10,000 targets
- [ ] DNS resolution timeout 5s
- [ ] Template loading <100ms first load, <1ms cached
- [ ] No UI blocking during I/O operations

### User Experience

- [ ] Keyboard navigation intuitive
- [ ] Visual feedback for all actions
- [ ] Error messages clear and actionable
- [ ] Help text available for shortcuts
- [ ] Confirmation dialogs for destructive actions
- [ ] Smooth scrolling with large lists

### Integration

- [ ] EventBus integration working
- [ ] ScanConfig updates correctly
- [ ] Modal overlay rendering correctly
- [ ] No conflicts with existing widgets
- [ ] State persistence across widget open/close

### Documentation

- [ ] SPRINT-6.5-PART2-COMPLETE.md created
- [ ] CHANGELOG.md updated
- [ ] README.md updated (TUI features)
- [ ] TUI-ARCHITECTURE.md updated
- [ ] CLAUDE.local.md updated (Recent Decisions, Sessions)
- [ ] All rustdoc comments added
- [ ] Usage examples in documentation

---

## Risk Mitigation

### Risk 1: Complex State Management

**Severity:** MEDIUM
**Probability:** 30%

**Mitigation:**
- Follow existing PortTableWidget state management patterns
- Use established Arc<RwLock<>> pattern for shared state
- Keep widget state isolated (no cross-widget dependencies)
- Comprehensive unit tests for state transitions

**Fallback:**
- Simplify state to essential fields only
- Defer advanced features (DNS caching) to future sprint

### Risk 2: File I/O Errors

**Severity:** MEDIUM
**Probability:** 25%

**Mitigation:**
- Comprehensive error handling with anyhow::Result
- Clear error messages with context (file path, error reason)
- File format validation before parsing
- Preview functionality to validate before import

**Fallback:**
- Basic format support initially (simple IP per line)
- Advanced formats (CIDR, ranges) in follow-up sprint

### Risk 3: DNS Resolution Blocking

**Severity:** LOW
**Probability:** 15%

**Mitigation:**
- Async DNS resolution with tokio::spawn
- Timeout handling (5 seconds default)
- Caching to avoid repeated lookups
- Clear UI feedback (Pending, Success, Failed states)

**Fallback:**
- Disable DNS resolution feature initially
- Basic hostname validation only (no actual resolution)

### Risk 4: Template Parsing Errors

**Severity:** LOW
**Probability:** 10%

**Mitigation:**
- Existing template validation logic reused
- TOML parsing with serde and toml crate
- Clear validation error messages
- Fallback to built-in templates on custom template errors

**Fallback:**
- Built-in templates only initially
- Custom template support in follow-up sprint

---

## Timeline

### Optimistic (16 hours, 25% probability)

**Day 1 (8 hours):**
- Task 1: CIDR Calculator (6 hours)
- Task 2: File Import/Export (2 hours)

**Day 2 (8 hours):**
- Task 2: File Import/Export (3 hours)
- Task 3: Exclusion Management (4 hours)
- Task 4: DNS Resolution (1 hour)

**Day 3 (4 hours):**
- Task 4: DNS Resolution (2 hours)
- Task 5: TemplateSelectionWidget (2 hours)

**Day 4 (4 hours):**
- Task 5: TemplateSelectionWidget (4 hours)
- Task 6: Integration (partial)

**Day 5 (4 hours):**
- Task 6: Integration (complete)
- Task 7: Documentation

### Realistic (20 hours, 60% probability)

**Day 1 (8 hours):**
- Task 1: CIDR Calculator (7 hours)
- Task 2: File Import/Export (1 hour)

**Day 2 (8 hours):**
- Task 2: File Import/Export (4 hours)
- Task 3: Exclusion Management (4 hours)

**Day 3 (8 hours):**
- Task 4: DNS Resolution (3 hours)
- Task 5: TemplateSelectionWidget (5 hours)

**Day 4 (8 hours):**
- Task 5: TemplateSelectionWidget (1 hour)
- Task 6: Integration (4 hours)
- Task 7: Documentation (3 hours)

**Day 5 (4 hours):**
- Final testing and quality gates
- Git commit

### Pessimistic (24 hours, 15% probability)

**Day 1 (8 hours):**
- Task 1: CIDR Calculator (8 hours, debugging)

**Day 2 (8 hours):**
- Task 2: File Import/Export (5 hours)
- Task 3: Exclusion Management (3 hours)

**Day 3 (8 hours):**
- Task 3: Exclusion Management (1 hour)
- Task 4: DNS Resolution (3 hours)
- Task 5: TemplateSelectionWidget (4 hours)

**Day 4 (8 hours):**
- Task 5: TemplateSelectionWidget (2 hours)
- Task 6: Integration (6 hours, debugging)

**Day 5 (8 hours):**
- Task 6: Integration (2 hours)
- Task 7: Documentation (4 hours)
- Final testing (2 hours)

**Day 6 (4 hours):**
- Quality gate fixes
- Git commit

**Recommended Planning:** Allocate 2-3 days (20-22 hours) with buffer for integration testing.

---

## Deliverables

### Phase 1: Planning ✓ COMPLETE
- [x] SPRINT-6.5-PART2-TODO.md (this document)

### Phase 2: Implementation
- [ ] TargetSelectionWidget implementation (~800 lines)
- [ ] TemplateSelectionWidget implementation (~600 lines)
- [ ] UIState extensions
- [ ] App integration (keyboard shortcuts, event routing)
- [ ] 30+ new tests (21 target + 6 template + 3 integration)

### Phase 3: Quality
- [ ] All quality gates passing (fmt, clippy, build, tests)
- [ ] Coverage ≥70% on new widgets
- [ ] Performance validated (<5ms rendering, <200ms file I/O)

### Phase 4: Documentation
- [ ] SPRINT-6.5-PART2-COMPLETE.md (600+ lines)
- [ ] CHANGELOG.md updated
- [ ] README.md updated
- [ ] TUI-ARCHITECTURE.md updated
- [ ] CLAUDE.local.md updated
- [ ] All rustdoc comments added

### Phase 5: Git Commit
- [ ] Comprehensive commit message (150-200 lines)
- [ ] All changes staged
- [ ] Commit created
- [ ] Ready for review

---

## Success Metrics

### Functional Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| CIDR Validation Accuracy | 100% | Unit tests (valid/invalid cases) |
| File Import Success Rate | 100% (valid files) | Integration tests |
| DNS Resolution Timeout | ≤5 seconds | Async timeout tests |
| Template Loading Time | <100ms first, <1ms cached | Performance benchmarks |
| Target Count Calculation | 100% accurate | Unit tests (edge cases) |

### Quality Metrics

| Metric | Target | Current Baseline |
|--------|--------|------------------|
| Tests Passing | 2,290+ | 2,260 (v0.6.1) |
| Clippy Warnings | 0 | 0 (v0.6.1) |
| Coverage Overall | ≥54.92% | 54.92% (v0.6.1) |
| Coverage New Code | ≥70% | N/A |
| Build Time | ≤120s | ~60s (v0.6.1) |
| Test Time | ≤120s | ~45s (v0.6.1) |

### Performance Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Widget Rendering | <5ms | Frame time profiling |
| CIDR Calculation | <10ms (10K IPs) | Benchmark tests |
| File Import | <200ms (10K targets) | Integration tests |
| DNS Resolution | 5s timeout | Async test with delay |
| Template Loading | <100ms | Benchmark on first load |

---

## Notes

### Platform Considerations

**Linux:**
- Full support for all features
- Native file path handling
- System DNS resolver

**macOS:**
- Full support
- Different file path conventions (~/ expansion)
- System DNS resolver

**Windows:**
- Full support
- Windows path separators (\\ vs /)
- System DNS resolver

### Future Enhancements (Post-Sprint)

**Not in Scope for Sprint 6.5 Part 2:**
- Advanced file browser UI (visual directory navigation)
- Drag-and-drop file import
- Real-time target validation (background task)
- Template editor (create/modify templates in TUI)
- Target grouping/tagging
- Import from cloud sources (S3, HTTP)

**Deferred to Later Sprints:**
- Interactive port specification widget
- Scan schedule/automation UI
- Results comparison between scans

### Lessons from Sprint 6.5 Part 1

**What Worked Well:**
- Systematic implementation order (Plugin → Idle → Decoy)
- Comprehensive testing at each task boundary
- Early quality gate validation
- Detailed planning documents

**What to Apply Here:**
- Start with simplest widget (TemplateSelection) for confidence
- Test each sub-task before moving to next
- Validate rendering performance early
- Incremental integration (one widget at a time)

---

**Document Version:** 1.0
**Status:** IN PROGRESS
**Created:** 2025-11-21
**Review Required:** Before implementation starts, verify:
- [ ] TUI architecture understood (Component trait pattern)
- [ ] Existing template system analyzed
- [ ] Dependencies identified
- [ ] Test strategy clear
- [ ] Time estimates realistic

---

**END OF SPRINT 6.5 PART 2 PLANNING DOCUMENT**
