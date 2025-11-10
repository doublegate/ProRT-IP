# Sprint 6.5: Interactive Target Selection & Scan Templates (QW-5)

**Status:** 📋 Planned (Q2 2026)
**Effort Estimate:** 14-18 hours
**Timeline:** Weeks 9-10 (2 weeks)
**Dependencies:** Sprint 6.2 (Live Dashboard) COMPLETE
**Priority:** HIGH (Critical Path)

## Sprint Overview

### Deliverables
1. **Interactive Target Selector** - TUI-based multi-select for discovered hosts
2. **QW-5: Scan Preset Templates** - Common scan profiles (ROI 3.33)
3. **Template Management System** - Create, save, load custom templates
4. **Target Import/Export** - Load from file, save discovered hosts
5. **TUI Integration** - Keyboard navigation, visual selection

### Strategic Value
- Transforms ProRT-IP into an interactive exploration tool (not just batch scanner)
- Scan templates reduce operator error and improve reproducibility
- Enables multi-stage workflows: discovery → selection → deep scan
- Differentiates from batch-only tools (Masscan, ZMap)

### Integration Points
- **TUI Framework (Sprint 6.1):** App state, event loop, keyboard handling
- **Live Dashboard (Sprint 6.2):** Port table widget for selection UI
- **EventBus:** TargetSelectionEvent, TemplateLoadEvent
- **Configuration System:** Template storage and loading

---

## Task Breakdown

### Task Area 1: Interactive Target Selector (5-6 hours)

**Task 1.1: Design TargetSelectorWidget**
- File: `prtip-tui/src/widgets/target_selector.rs`
- Multi-select table: IP addresses with checkbox selection
- Columns: [ ] IP Address, Open Ports, Services, OS Hint
- Keyboard navigation:
  - `Space`: Toggle selection for current row
  - `a`: Select all
  - `n`: Select none
  - `i`: Invert selection
  - `Enter`: Confirm selection and proceed to next scan phase
```rust
pub struct TargetSelectorWidget {
    discovered_hosts: Vec<DiscoveredHost>,
    selected_indices: HashSet<usize>,
    current_row: usize,
    filter: Option<TargetFilter>,
}

pub struct DiscoveredHost {
    pub ip: IpAddr,
    pub open_ports: Vec<u16>,
    pub services: Vec<String>,
    pub os_hint: Option<String>,
}

impl TargetSelectorWidget {
    pub fn toggle_selection(&mut self, index: usize) {
        if self.selected_indices.contains(&index) {
            self.selected_indices.remove(&index);
        } else {
            self.selected_indices.insert(index);
        }
    }
    
    pub fn select_all(&mut self) {
        self.selected_indices = (0..self.discovered_hosts.len()).collect();
    }
    
    pub fn select_none(&mut self) {
        self.selected_indices.clear();
    }
    
    pub fn invert_selection(&mut self) {
        let all: HashSet<usize> = (0..self.discovered_hosts.len()).collect();
        self.selected_indices = all.difference(&self.selected_indices).copied().collect();
    }
    
    pub fn get_selected_targets(&self) -> Vec<IpAddr> {
        self.selected_indices.iter()
            .map(|&i| self.discovered_hosts[i].ip)
            .collect()
    }
}
```
- **Estimated Time:** 2.5h

**Task 1.2: Populate from scan results**
```rust
// Subscribe to port discovery events
let mut port_rx = app.event_bus.subscribe_typed::<PortDiscoveryEvent>();

// Aggregate by host
let mut hosts: HashMap<IpAddr, DiscoveredHost> = HashMap::new();

tokio::select! {
    Some(discovery) = port_rx.recv() => {
        hosts.entry(discovery.target)
            .or_insert_with(|| DiscoveredHost::new(discovery.target))
            .add_port(discovery.port, discovery.service);
    }
}

// Update selector widget when discovery phase complete
if scan_state.phase == ScanPhase::DiscoveryComplete {
    app.ui_state.target_selector.set_hosts(hosts.into_values().collect());
}
```
- **Estimated Time:** 1.5h

**Task 1.3: Add filtering capabilities**
- Filter by: port count (≥ N open ports), service (has SSH/HTTP/etc), OS family
- Filter UI: `f` key opens filter dialog
```rust
pub enum TargetFilter {
    MinOpenPorts(usize),          // ≥ N open ports
    HasService(String),            // Has service matching regex
    OsFamily(String),              // Linux, Windows, etc
    PortRange(u16, u16),          // Has ports in range
    Combined(Vec<TargetFilter>),  // AND of multiple filters
}

impl TargetFilter {
    pub fn matches(&self, host: &DiscoveredHost) -> bool {
        match self {
            Self::MinOpenPorts(n) => host.open_ports.len() >= *n,
            Self::HasService(regex) => {
                let re = Regex::new(regex).unwrap();
                host.services.iter().any(|s| re.is_match(s))
            }
            Self::OsFamily(family) => {
                host.os_hint.as_ref().map_or(false, |hint| hint.contains(family))
            }
            Self::PortRange(min, max) => {
                host.open_ports.iter().any(|&p| p >= *min && p <= *max)
            }
            Self::Combined(filters) => {
                filters.iter().all(|f| f.matches(host))
            }
        }
    }
}
```
- Display filter status in widget header: "Filtered: 42/100 hosts (SSH servers)"
- **Estimated Time:** 2h

**Task 1.4: Render widget with selection indicators**
```rust
use ratatui::widgets::{Table, Row, Cell};

impl Widget for TargetSelectorWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows: Vec<Row> = self.discovered_hosts.iter().enumerate()
            .filter(|(_, host)| self.filter.as_ref().map_or(true, |f| f.matches(host)))
            .map(|(i, host)| {
                let checkbox = if self.selected_indices.contains(&i) {
                    "[✓]"
                } else {
                    "[ ]"
                };
                
                let style = if i == self.current_row {
                    Style::default().bg(Color::Blue)
                } else {
                    Style::default()
                };
                
                Row::new(vec![
                    Cell::from(checkbox),
                    Cell::from(host.ip.to_string()),
                    Cell::from(host.open_ports.len().to_string()),
                    Cell::from(host.services.join(", ")),
                    Cell::from(host.os_hint.clone().unwrap_or("-".to_string())),
                ]).style(style)
            })
            .collect();
        
        let table = Table::new(rows, &[
            Constraint::Length(3),  // Checkbox
            Constraint::Length(20), // IP
            Constraint::Length(10), // Port count
            Constraint::Min(20),    // Services
            Constraint::Min(15),    // OS
        ]);
        
        table.render(area, buf);
    }
}
```
- **Estimated Time:** 1h

**Task 1.5: Write unit tests**
- Test selection operations (toggle, select all, invert)
- Test filtering (min ports, service regex, OS family)
- Test keyboard navigation (arrow keys, space, enter)
- **Target:** 10-12 tests
- **Estimated Time:** 1h

---

### Task Area 2: Scan Preset Templates (QW-5) (4-5 hours)

**Task 2.1: Design template system**
- File: `prtip-core/src/config/scan_templates.rs`
- Built-in templates:
  1. **Quick Discovery:** -sS -F --top-ports 100
  2. **Full TCP:** -sS -p1-65535 -T4
  3. **Service Detection:** -sV -sC --version-intensity 5
  4. **Stealth Scan:** -sS -f --mtu 24 -g 53 --ttl 64 -D RND:5
  5. **UDP Scan:** -sU --top-ports 100
  6. **IPv6 Discovery:** -6 -sS -F
  7. **Aggressive:** -A -T4 (OS + version + scripts)
  8. **Polite:** -T0 -p80,443,22 (slow, low impact)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTemplate {
    pub name: String,
    pub description: String,
    pub scan_type: ScanType,
    pub ports: PortSpec,
    pub timing: TimingTemplate,
    pub enable_service_detection: bool,
    pub enable_os_detection: bool,
    pub enable_scripts: bool,
    pub evasion: EvasionConfig,
    pub tags: Vec<String>,  // "fast", "stealth", "comprehensive"
}

impl ScanTemplate {
    pub fn quick_discovery() -> Self {
        Self {
            name: "Quick Discovery".to_string(),
            description: "Fast SYN scan of top 100 ports".to_string(),
            scan_type: ScanType::Syn,
            ports: PortSpec::Top(100),
            timing: TimingTemplate::T4,
            enable_service_detection: false,
            enable_os_detection: false,
            enable_scripts: false,
            evasion: EvasionConfig::default(),
            tags: vec!["fast".to_string(), "discovery".to_string()],
        }
    }
    
    // ... other built-in templates
}
```
- **Estimated Time:** 2h

**Task 2.2: Template management CLI**
- File: `prtip/src/cli.rs`
- Commands:
  - `prtip template list` - Show available templates
  - `prtip template show <name>` - Display template details
  - `prtip template save <name>` - Save current config as template
  - `prtip template load <name> <target>` - Run scan with template
```rust
#[derive(Parser)]
pub enum TemplateCommand {
    /// List available scan templates
    List,
    
    /// Show template details
    Show { name: String },
    
    /// Save current configuration as template
    Save {
        name: String,
        #[arg(long)]
        description: Option<String>,
    },
    
    /// Load and run template
    Load {
        name: String,
        target: String,
    },
}
```
- **Estimated Time:** 1.5h

**Task 2.3: Template storage (TOML format)**
- File: `~/.prtip/templates/` directory
- Each template: `~/.prtip/templates/<name>.toml`
```toml
# ~/.prtip/templates/web-server-audit.toml
name = "Web Server Audit"
description = "Comprehensive scan of web servers (HTTP/HTTPS)"
scan_type = "Syn"
timing = "T4"

[ports]
type = "List"
ports = [80, 443, 8080, 8443, 8000, 8888]

[service_detection]
enabled = true
version_intensity = 5

[scripts]
enabled = true
scripts = ["http-title", "http-headers", "ssl-cert"]

[evasion]
fragment_packets = false
source_port = null

[tags]
tags = ["web", "comprehensive"]
```
- **Estimated Time:** 1.5h

**Task 2.4: TUI template selector**
- File: `prtip-tui/src/widgets/template_selector.rs`
- Display templates in sidebar (during scan setup phase)
- Keyboard: `t` opens template selector, arrow keys navigate, Enter selects
- Preview template details in right panel
- **Estimated Time:** 1.5h

**Task 2.5: Write unit tests**
- Test built-in templates (8 templates load correctly)
- Test template save/load round-trip
- Test TOML serialization/deserialization
- **Target:** 8-10 tests
- **Estimated Time:** 1h

---

### Task Area 3: Target Import/Export (2-3 hours)

**Task 3.1: Target file formats**
- File: `prtip-core/src/io/target_io.rs`
- Supported formats:
  1. Plain text (one IP/CIDR per line)
  2. Nmap XML (`-iX` flag)
  3. Greppable (`-iG` flag)
  4. JSON (ProRT-IP native)
```rust
pub enum TargetFileFormat {
    PlainText,  // 192.168.1.1\n10.0.0.0/24
    NmapXml,    // <host><address addr="192.168.1.1"/></host>
    Greppable,  // Host: 192.168.1.1 () Ports: 80/open/tcp//http///
    Json,       // [{"ip": "192.168.1.1", "ports": [80, 443]}]
}

pub struct TargetImporter {
    format: TargetFileFormat,
}

impl TargetImporter {
    pub fn import(&self, path: &str) -> io::Result<Vec<TargetSpec>> {
        match self.format {
            TargetFileFormat::PlainText => self.import_plain_text(path),
            TargetFileFormat::NmapXml => self.import_nmap_xml(path),
            TargetFileFormat::Greppable => self.import_greppable(path),
            TargetFileFormat::Json => self.import_json(path),
        }
    }
}
```
- **Estimated Time:** 1.5h

**Task 3.2: Export discovered targets**
```rust
pub struct TargetExporter {
    format: TargetFileFormat,
}

impl TargetExporter {
    pub fn export(&self, targets: &[DiscoveredHost], path: &str) -> io::Result<()> {
        match self.format {
            TargetFileFormat::PlainText => {
                let content = targets.iter()
                    .map(|h| h.ip.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                std::fs::write(path, content)?;
            }
            TargetFileFormat::Json => {
                let json = serde_json::to_string_pretty(targets)?;
                std::fs::write(path, json)?;
            }
            _ => unimplemented!("Export format not supported"),
        }
        Ok(())
    }
}
```
- **Estimated Time:** 1h

**Task 3.3: TUI export dialog**
- Keyboard: `e` opens export dialog
- Select format (plain text, JSON)
- Enter filename (default: `prtip-targets-<timestamp>.txt`)
- Display success message: "Exported 42 targets to targets.txt"
- **Estimated Time:** 1h

**Task 3.4: Write unit tests**
- Test import plain text (10 IPs)
- Test import Nmap XML (parse 5 hosts)
- Test export plain text (round-trip)
- Test export JSON (verify format)
- **Target:** 6-8 tests
- **Estimated Time:** 0.5h

---

### Task Area 4: TUI Integration & Keyboard Navigation (2-3 hours)

**Task 4.1: Implement multi-phase scan workflow**
```rust
pub enum ScanPhase {
    Setup,             // Configure scan, select template
    Discovery,         // Running discovery scan
    TargetSelection,   // Interactive selection of discovered hosts
    DeepScan,          // Running deep scan on selected targets
    Results,           // Display final results
}

impl App {
    pub fn handle_keyboard(&mut self, key: KeyEvent) -> io::Result<()> {
        match self.phase {
            ScanPhase::Setup => {
                match key.code {
                    KeyCode::Char('t') => self.open_template_selector(),
                    KeyCode::Enter => self.start_discovery_scan(),
                    _ => {}
                }
            }
            ScanPhase::TargetSelection => {
                match key.code {
                    KeyCode::Char(' ') => self.target_selector.toggle_selection(),
                    KeyCode::Char('a') => self.target_selector.select_all(),
                    KeyCode::Char('n') => self.target_selector.select_none(),
                    KeyCode::Char('i') => self.target_selector.invert_selection(),
                    KeyCode::Char('f') => self.open_filter_dialog(),
                    KeyCode::Char('e') => self.export_targets(),
                    KeyCode::Enter => self.start_deep_scan(),
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```
- **Estimated Time:** 1.5h

**Task 4.2: Add keyboard shortcut help overlay**
- Keyboard: `?` toggles help overlay
- Context-sensitive shortcuts (different for each phase)
- Display as centered popup dialog
- **Estimated Time:** 1h

**Task 4.3: Write integration tests**
- Test phase transitions (Setup → Discovery → Selection → DeepScan)
- Test keyboard shortcuts in each phase
- **Target:** 5-6 tests
- **Estimated Time:** 1h

---

### Task Area 5: Documentation (1-2 hours)

**Task 5.1: Create interactive selection guide**
- File: `docs/29-INTERACTIVE-SELECTION-GUIDE.md` (1,000-1,200 lines)
- Sections:
  1. Overview (multi-phase workflow)
  2. Target Selection Widget (keyboard shortcuts)
  3. Scan Templates (built-in + custom)
  4. Import/Export (supported formats)
  5. Filtering Targets (port count, services, OS)
  6. Examples (typical workflows)
- **Estimated Time:** 1.5h

**Task 5.2: Update CHANGELOG.md**
- Add entry for Sprint 6.5 completion
- Highlight: Interactive target selection, 8 built-in templates
- **Estimated Time:** 0.5h

---

## Definition of Done

### Functional Requirements
- [ ] Interactive target selector working (multi-select, keyboard navigation)
- [ ] 8 built-in scan templates available
- [ ] Template save/load functional (TOML storage)
- [ ] Target import from plain text, Nmap XML, JSON
- [ ] Target export to plain text, JSON
- [ ] Multi-phase scan workflow (Setup → Discovery → Selection → DeepScan)
- [ ] Keyboard shortcuts documented and working

### Quality Requirements
- [ ] 39-42 tests passing (100% success rate)
- [ ] Zero clippy warnings (--deny warnings)
- [ ] Zero rustdoc warnings
- [ ] Code formatted with `cargo fmt`
- [ ] TUI responsive (<100ms input latency)

### Documentation Requirements
- [ ] 29-INTERACTIVE-SELECTION-GUIDE.md complete (1,000-1,200 lines)
- [ ] Rustdoc comments for all public APIs
- [ ] Template examples in guide
- [ ] Workflow diagrams (phase transitions)

### User Experience Requirements
- [ ] Selection changes visible immediately (<100ms)
- [ ] Filter results update in real-time
- [ ] Template preview shows all configuration details
- [ ] Export completes in <1s for 10K targets

---

## Testing Plan

### Unit Tests (25-28 tests)
```bash
cargo test -p prtip-tui widgets::target_selector
cargo test -p prtip-core config::scan_templates
cargo test -p prtip-core io::target_io
```

**Test Cases:**
1-5. TargetSelector: toggle, select all, select none, invert, get selected
6-9. TargetFilter: min ports, has service, OS family, combined
10-17. ScanTemplate: 8 built-in templates load correctly
18. Template: save to TOML
19. Template: load from TOML (round-trip)
20-23. TargetImporter: plain text, Nmap XML, greppable, JSON
24-25. TargetExporter: plain text, JSON
26-28. Keyboard handling: space, a, n, i, f, e, Enter

### Integration Tests (14-16 tests)
```bash
cargo test -p prtip --test integration_interactive
```

**Test Cases:**
1. Full Workflow: Setup → Discovery → Selection → DeepScan
2. Target Selection: select 10/100 hosts, verify correct IPs
3. Filtering: filter by ≥5 open ports, verify count
4. Filtering: filter by SSH service, verify matches
5. Template Load: load "Quick Discovery", verify config
6. Template Save: save custom template, reload, verify equality
7. Import: load targets.txt (100 IPs), verify count
8. Export: save 50 discovered hosts to JSON, verify format
9. EventBus: TargetSelectionEvent emitted on selection change
10. EventBus: TemplateLoadEvent emitted on template load
11. Phase Transitions: verify correct phase after each action
12. Keyboard Shortcuts: test all shortcuts in each phase
13. Help Overlay: verify context-sensitive shortcuts
14. TUI Responsiveness: input latency <100ms

### Manual Testing Checklist
- [ ] **Selection:** Space key toggles checkbox
- [ ] **Selection:** `a` selects all hosts
- [ ] **Selection:** `n` clears all selections
- [ ] **Selection:** `i` inverts selection
- [ ] **Selection:** Selected count displays correctly (e.g., "42/100 selected")
- [ ] **Filtering:** `f` opens filter dialog
- [ ] **Filtering:** Filter by ≥5 ports shows only matching hosts
- [ ] **Templates:** `t` opens template selector
- [ ] **Templates:** All 8 built-in templates listed
- [ ] **Templates:** Template preview shows full configuration
- [ ] **Import:** `prtip -iL targets.txt` loads targets correctly
- [ ] **Export:** `e` key saves selected targets to file
- [ ] **Workflow:** Discovery scan → selection UI → deep scan (end-to-end)
- [ ] **Help:** `?` displays keyboard shortcuts for current phase

---

## Dependencies

### External Crates
- `serde_json = "1.0"` - JSON import/export
- `quick-xml = "0.31"` - Nmap XML parsing
- `toml = "0.8"` - Template storage

### Internal Dependencies
- **Sprint 6.1 (TUI Framework):** App state, keyboard handling
- **Sprint 6.2 (Live Dashboard):** Widget framework
- **prtip-core:** ScanConfig, TargetSpec

---

## Risk Mitigation

### Risk 1: Nmap XML Parsing Complexity
**Impact:** Medium | **Probability:** Medium
**Mitigation:**
- Use quick-xml crate (well-tested)
- Support subset of Nmap XML (hosts, ports only)
- Provide examples in documentation

### Risk 2: TUI Selection Performance (Large Host Lists)
**Impact:** Medium | **Probability:** Low
**Mitigation:**
- Paginate host list (100 hosts per page)
- Render only visible rows (scrolling viewport)
- Test with 10K host list

### Risk 3: Template Storage Location
**Impact:** Low | **Probability:** Low
**Mitigation:**
- Use XDG Base Directory specification (`~/.config/prtip/templates/`)
- Fallback to `~/.prtip/templates/` on non-Linux
- Document location in guide

---

## Resources

### Documentation
- **XDG Base Directory:** https://specifications.freedesktop.org/basedir-spec/
- **Nmap XML Format:** https://nmap.org/book/nmap-dtd.html
- **TOML Specification:** https://toml.io/

### Reference Implementations
- **Nmap Templates:** NSE scripts (scan profiles)
- **Masscan Config:** masscan.conf (simple key=value)

---

## Sprint Completion Report Template

```markdown
# Sprint 6.5 Completion Report

**Date:** [YYYY-MM-DD]
**Actual Duration:** [X] hours
**Status:** ✅ COMPLETE / ⚠️ PARTIAL / ❌ INCOMPLETE

## Deliverables Status
- [ ] Interactive Target Selector
- [ ] 8 Built-in Scan Templates
- [ ] Template Management System
- [ ] Target Import/Export
- [ ] TUI Integration & Keyboard Navigation

## Test Results
- Unit Tests: [X/28] passing
- Integration Tests: [X/16] passing

## Performance Metrics
- Selection Latency: [X]ms (target: <100ms)
- Export Time (10K targets): [X]s (target: <1s)

## Issues Encountered
1. [Issue description] - **Resolution:** [How resolved]

## Lessons Learned
- [Key insight from TUI development]

## Next Sprint Preparation
- Dependencies ready for Sprint 6.6: ✅/❌
```

---

**This sprint transforms ProRT-IP into an interactive tool - operators can now explore and refine scan targets visually. Prioritize UX polish (smooth animations, clear feedback) over raw performance.**
