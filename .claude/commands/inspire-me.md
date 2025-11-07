# /inspire-me - AI-Powered Competitive Intelligence & Enhancement Planning

## Purpose

Execute comprehensive competitive analysis of ProRT-IP against industry leaders (Nmap, Masscan, RustScan, etc.), identify feature gaps and opportunities, and generate an 8-sprint enhancement roadmap for the current development phase.

**CRITICAL:** This command ALWAYS executes via dedicated sub-agent with full MCP server access for maximum research depth and autonomous execution.

**Use Case:** Run at end of each phase to discover competitive improvements before proceeding to next phase

**Duration:** ~3-4 hours for comprehensive analysis
**Output:** `docs/XX-PHASEXX-ENHANCEMENTS.md` + execution report + knowledge graph integration

---

## COMMAND INVOCATION

**User Triggers:**
```bash
/inspire-me
/inspire-me [phase_number]           # Optional: Specify phase (e.g., 6, 7)
/inspire-me --deep                    # Extended research mode (5-6h)
/inspire-me --quick                   # Quick analysis mode (1-2h, 4 sprints)
```

**Claude MUST Execute:**
```
/sub-agent (new, run separately) ultrathink: Execute comprehensive competitive
analysis and enhancement planning for ProRT-IP following the /inspire-me workflow
(all 6 phases) --> utilize ALL available MCP servers/tools for enhanced research:

1. GitHub MCP: Repository analysis, code search, issue tracking
2. Browser MCP: Online research, documentation, benchmarks
3. Knowledge Graph MCP: Track competitive insights, build relationship maps
4. Code Search MCP: Analyze reference implementations

Generate:
- docs/XX-PHASEXX-ENHANCEMENTS.md (>10,000 words)
- /tmp/ProRT-IP/inspire-me-report.md (execution summary)
- Knowledge graph entries (competitive landscape)
- CLAUDE.local.md session entry

Follow all 6 phases systematically with A+ quality standards.
```

---

## EXECUTION PHASES (Sub-Agent Instructions)

### PHASE 1: CONTEXT GATHERING (15 minutes)

**Objective:** Understand current project state + gather historical context

**Actions:**
```bash
# Read current documentation
Read: README.md, CLAUDE.md, CLAUDE.local.md
Read: docs/00-ARCHITECTURE.md, docs/01-ROADMAP.md
Read: docs/10-PROJECT-STATUS.md, CHANGELOG.md

# Use Knowledge Graph to recall previous competitive insights
mcp__MCP_DOCKER__search_nodes: query="competitive analysis" OR "enhancement" OR "gap"
mcp__MCP_DOCKER__open_nodes: ["ProRT-IP Competitors", "Feature Gaps", "Performance Metrics"]
```

**Extract Metrics:**
- Current version (from CHANGELOG.md)
- Current phase (from CLAUDE.local.md)
- Test count and coverage (from README.md / CLAUDE.local.md)
- Performance metrics (from README.md)
- Features implemented (from README.md / CHANGELOG.md)
- Recent completions (from CLAUDE.local.md sessions)
- Previous enhancement findings (from knowledge graph)

**Identify Phase Number:**
- Determine current phase (e.g., "Phase 4", "Phase 5") from PROJECT-STATUS
- Default: If unclear, use next phase number in sequence

**Verification Checklist:**
- [ ] Current version identified
- [ ] Phase number determined
- [ ] Metrics captured (tests, coverage, performance)
- [ ] Feature list documented
- [ ] Recent accomplishments noted
- [ ] Historical competitive insights retrieved

---

### PHASE 2: REFERENCE CODE ANALYSIS (45 minutes - ENHANCED)

**Objective:** Analyze competitive implementations using all available tools

**2.1 Local Code Analysis:**
```bash
# Scan code_ref/ directory
Glob: code_ref/**/*

# Use Code Search MCP for efficient analysis
mcp__MCP_DOCKER__search_code: "scanner implementation" language:c language:cpp language:rust language:go
mcp__MCP_DOCKER__search_code: "packet crafting" "raw socket"
mcp__MCP_DOCKER__search_code: "service detection" "banner grabbing"

# Read implementation files (prioritize by relevance)
# Focus: Scanner logic, packet crafting, detection engines, concurrency patterns

# Categorize findings:
# - Scanning techniques (SYN, stealth, idle, etc.)
# - Performance patterns (async, batching, zero-copy)
# - UI/UX approaches (help systems, progress bars)
# - Protocol implementations (TCP, UDP, ICMP)
# - Error handling strategies
# - Testing approaches
```

**2.2 GitHub Repository Research (MCP-Enhanced):**

**Primary Targets:**
1. **nmap/nmap** - Industry standard (C++)
   ```bash
   mcp__MCP_DOCKER__get_file_contents: owner="nmap" repo="nmap" path="/"
   mcp__MCP_DOCKER__search_repositories: "nmap network scanner"
   mcp__MCP_DOCKER__list_commits: owner="nmap" repo="nmap" perPage=20
   mcp__MCP_DOCKER__search_code: "service_scan" repo:nmap/nmap
   mcp__MCP_DOCKER__search_code: "idle_scan" repo:nmap/nmap
   ```
   - Focus: NSE scripts, OS fingerprinting database, service detection
   - Key files: NmapOps.h, service_scan.cc, idle_scan.cc
   - Recent features: Last 6 months of commits

2. **RustScan/RustScan** - Modern Rust alternative
   ```bash
   mcp__MCP_DOCKER__get_file_contents: owner="RustScan" repo="RustScan" path="/"
   mcp__MCP_DOCKER__search_code: "scanner" repo:RustScan/RustScan language:rust
   mcp__MCP_DOCKER__list_pull_requests: owner="RustScan" repo="RustScan" state="closed"
   ```
   - Focus: Async patterns, CLI design, performance optimizations
   - Key files: src/scanner/mod.rs, Cargo.toml dependencies

3. **robertdavidgraham/masscan** - Ultra-fast C scanner
   ```bash
   mcp__MCP_DOCKER__get_file_contents: owner="robertdavidgraham" repo="masscan" path="/"
   mcp__MCP_DOCKER__search_code: "transmit" OR "event loop" repo:robertdavidgraham/masscan
   ```
   - Focus: Stateless scanning, packet rate optimization, SipHash
   - Key files: transmit-linux.c, event loop patterns

4. **projectdiscovery/naabu** - Modern Go scanner
   ```bash
   mcp__MCP_DOCKER__get_file_contents: owner="projectdiscovery" repo="naabu" path="/"
   mcp__MCP_DOCKER__search_code: "scanner" repo:projectdiscovery/naabu language:go
   ```
   - Focus: Configuration design, output formats, integration patterns

**Research Focus for Each:**
- Star count and recent activity (GitHub MCP API)
- Recent features added (last 6 months via commits)
- Issue tracker for feature requests
- README for capability claims
- Performance benchmarks (if available)

**2.3 Online Community Research (Browser MCP):**
```bash
# Use Browser MCP for live research
mcp__MCP_DOCKER__browser_navigate: "https://www.google.com"
mcp__MCP_DOCKER__browser_snapshot  # Get current page state

# Search queries
WebSearch: "nmap vs masscan vs rustscan performance comparison 2024 2025"
WebSearch: "port scanner features comparison service detection"
WebSearch: "network scanner benchmarks 2025"
WebSearch: site:reddit.com "port scanning" OR "network scanner"
WebSearch: site:news.ycombinator.com "nmap" OR "port scanner"

# Fetch documentation
mcp__MCP_DOCKER__browser_navigate: "https://nmap.org/docs.html"
mcp__MCP_DOCKER__browser_snapshot

# Reddit research
mcp__MCP_DOCKER__browser_navigate: "https://www.reddit.com/r/netsec/search/?q=port+scanner"
mcp__MCP_DOCKER__browser_snapshot

# Hacker News
mcp__MCP_DOCKER__browser_navigate: "https://news.ycombinator.com/item?id=..."
mcp__MCP_DOCKER__browser_snapshot
```

**2.4 Knowledge Graph Integration:**
```bash
# Create competitive landscape entities
mcp__MCP_DOCKER__create_entities: [
  {
    name: "Nmap",
    entityType: "Competitor",
    observations: ["Industry standard scanner", "C++ implementation", "600+ NSE scripts", ...]
  },
  {
    name: "RustScan",
    entityType: "Competitor",
    observations: ["Modern Rust scanner", "3-8s full scan claim", ...]
  },
  ...
]

# Create relationships
mcp__MCP_DOCKER__create_relations: [
  {from: "ProRT-IP", to: "Nmap", relationType: "competes_with"},
  {from: "ProRT-IP", to: "RustScan", relationType: "competes_with"},
  {from: "Nmap", to: "NSE Scripts", relationType: "has_feature"},
  ...
]
```

**Verification Checklist:**
- [ ] All code_ref/ files analyzed (list key insights)
- [ ] 4+ GitHub repos researched (note stars, activity, recent commits)
- [ ] Community discussions reviewed (Reddit, HN, blogs, StackOverflow)
- [ ] Documentation fetched and analyzed
- [ ] Notes compiled by category
- [ ] Knowledge graph entities created (competitors, features, gaps)

---

### PHASE 3: GAP ANALYSIS (45 minutes)

**Objective:** Identify what ProRT-IP lacks vs competitors

**3.1 Create Feature Matrix:**

Build comprehensive comparison table:

| Feature Category | ProRT-IP | Nmap | Masscan | RustScan | Naabu | Gap Analysis |
|------------------|----------|------|---------|----------|-------|--------------|
| **Scan Types** | [List] | [List] | [List] | [List] | [List] | [Analysis] |
| **Service Detection** | [Rate %] | [Rate %] | [Status] | [Status] | [Status] | [Gap/Advantage] |
| **OS Fingerprinting** | [Details] | [2600+ sigs] | [None] | [None] | [None] | [Gap] |
| **Performance** | [Metrics] | [Metrics] | [10M pps] | [3-8s full] | [Metrics] | [Analysis] |
| **Output Formats** | [Count] | [Count] | [Count] | [Count] | [Count] | [Gap] |
| **Stealth Features** | [List] | [Comprehensive] | [Limited] | [Limited] | [Limited] | [Gap] |
| **IPv6 Support** | [Status] | [Full] | [Limited] | [Yes] | [Yes] | [Gap] |
| **Scripting** | [Status] | [NSE 600+] | [None] | [Multi-lang] | [None] | [Gap] |
| **CLI Design** | [Flags count] | [80+] | [~10] | [~15] | [~20] | [Gap] |
| **Documentation** | [Quality] | [Extensive] | [Basic] | [Good] | [Basic] | [Advantage?] |
| **Testing** | [Coverage %] | [Unknown] | [Unknown] | [Some] | [Unknown] | [Advantage?] |
| **Architecture** | [Rust/Tokio] | [C++] | [C] | [Rust] | [Go] | [Modern] |
| **Plugin System** | [Status] | [NSE] | [None] | [Lua/Python] | [None] | [Gap] |
| **TLS Analysis** | [Status] | [ssl-enum-ciphers] | [None] | [Limited] | [None] | [Gap/Advantage] |
| **Rate Limiting** | [Overhead %] | [Unknown] | [Stateless] | [Unknown] | [Unknown] | [Advantage?] |

**3.2 Performance Analysis:**

Compare benchmarks using available data:
- Packet rate (pps): ProRT-IP vs competitors
- Scan duration: Common ports (1-1000), full port range (1-65535)
- Memory usage: Peak, average
- CPU utilization: Single-core, multi-core
- Network efficiency: Packets sent vs results
- Accuracy: False positives/negatives

**3.3 Categorize Findings:**

**STRENGTHS** (Where ProRT-IP excels):
- [List 5-10 areas with evidence]
- Include: Rust safety, testing, modern architecture, specific features
- Provide metrics and comparisons
- Example: "54.92% code coverage vs competitors (unknown/minimal)"

**GAPS** (Where ProRT-IP is behind):
- [List 5-10 missing features/capabilities]
- Rate severity: HIGH / MEDIUM / LOW
- Estimate effort: EASY (1-2d) / MEDIUM (3-5d) / HARD (5-10d)
- Calculate ROI: (User Impact × Competitive Gap) / Effort
- Example: "NSE-equivalent scripting (HIGH severity, HARD effort, 7.5 ROI)"

**OPPORTUNITIES** (Where ProRT-IP can innovate):
- [List 3-5 unique advantages to pursue]
- Rust-specific benefits (memory safety, async/await, zero-cost abstractions)
- Modern architecture advantages (Tokio ecosystem, futures composability)
- Differentiation strategies (what can we do better than everyone?)
- Example: "Async plugin system with tokio (leverages Rust strengths)"

**3.4 Knowledge Graph Update:**
```bash
# Add gap analysis findings to knowledge graph
mcp__MCP_DOCKER__add_observations: [
  {
    entityName: "ProRT-IP Feature Gaps",
    contents: [
      "NSE-equivalent scripting system (HIGH priority)",
      "Advanced CLI help system (MEDIUM priority)",
      ...
    ]
  }
]
```

**Verification Checklist:**
- [ ] Feature matrix complete (15+ categories)
- [ ] Performance comparison documented
- [ ] Gaps categorized by severity (HIGH/MEDIUM/LOW)
- [ ] Effort estimated (EASY/MEDIUM/HARD)
- [ ] Opportunities identified (3-5 items)
- [ ] ROI scores calculated for potential enhancements
- [ ] Knowledge graph updated with findings

---

### PHASE 4: SPRINT PLANNING (60 minutes)

**Objective:** Design 8 enhancement sprints (or 4 for --quick mode)

**4.1 Prioritization Framework:**

Score each potential enhancement:

| Enhancement | User Impact (1-10) | Competitive Gap (1-10) | Effort (1-10, low=easy) | ROI Score | Priority |
|-------------|-------------------|------------------------|-------------------------|-----------|----------|
| [Feature 1] | X | Y | Z | (X × Y) / Z | HIGH/MED/LOW |
| [Feature 2] | X | Y | Z | (X × Y) / Z | HIGH/MED/LOW |
| ... | | | | | |

**ROI Formula:** `(User Impact × Competitive Gap) / Effort`

**Prioritization Rules:**
1. Sort by ROI score descending
2. Ensure dependency order (foundations before features)
3. Balance quick wins (HIGH ROI, EASY effort) with strategic work
4. Top 8 become sprints (top 4 for --quick mode)

**4.2 Sprint Structure Template:**

For each of 8 sprints:

**Sprint X.YY: [Title]**
- **Priority:** HIGH / MEDIUM / LOW (based on ROI)
- **Duration:** 3-5 days
- **Dependencies:** [List any prerequisites]
- **ROI Score:** X.X/10
- **Effort:** EASY / MEDIUM / HARD

**Objective:** [Clear, measurable goal in 1-2 sentences]

**Rationale:** [Why now? Competitive findings that justify this sprint]
- Current State: [What ProRT-IP has/lacks]
- Competitor State: [What others have with specifics]
- Impact: [User/competitive impact]
- Strategic Value: [Long-term benefit]

**Tasks:** [7-12 specific tasks with time estimates]
1. [ ] Research phase: Analyze X competitor implementation (2h)
2. [ ] Design: Architecture doc for Y feature (3h)
3. [ ] Core implementation: Z module (8h)
4. [ ] Integration: Connect with existing A subsystem (4h)
5. [ ] Testing: Unit tests (90%+ coverage) (4h)
6. [ ] Testing: Integration tests (3h)
7. [ ] Documentation: Technical guide (4h)
8. [ ] Performance: Benchmarks and optimization (3h)
9. [ ] Polish: Error handling, edge cases (2h)
10. [ ] Review: Code review and refinement (2h)

**Total Estimated Time:** 35h (7-day sprint with buffer)

**Deliverables:**
- [ ] Feature implementation (specific module/file)
- [ ] Tests (target: 90%+ coverage for new code)
- [ ] Documentation updates (list files: README, CHANGELOG, guide)
- [ ] Benchmarks (if performance-related)
- [ ] Examples (if user-facing feature)

**Success Criteria:**
- **Quantitative:** [3-5 measurable metrics]
  - Example: "Service detection rate 85% → 92% (+7pp)"
  - Example: "Plugin API supports 5+ hook types"
- **Qualitative:** [2-3 quality indicators]
  - Example: "Nmap parity for X feature"
  - Example: "Professional documentation quality (A+ grade)"

**References:**
- Code: [Relevant files in code_ref/ or ProRT-IP]
- Research: [Links to articles, docs, RFCs, GitHub repos]
- Competitors: [Specific implementations to reference]

**Risk Mitigation:**
- [List 2-3 potential risks and mitigations]
- Example: "Risk: Complex integration. Mitigation: Phased rollout with feature flag"

**4.3 Sprint Themes:**

Ensure balanced mix of:
- **Performance optimization** (1-2 sprints)
  - Example: "Zero-copy packet processing for 20% throughput improvement"
- **Feature parity** (2-3 sprints)
  - Example: "Advanced help system matching Nmap's multi-page design"
- **Innovation/differentiation** (1-2 sprints)
  - Example: "Async plugin system leveraging Tokio (unique to ProRT-IP)"
- **User experience** (1 sprint)
  - Example: "Interactive mode with real-time scan monitoring"
- **Documentation/polish** (1 sprint)
  - Example: "Professional example gallery with 50+ scenarios"

**4.4 Dependency Mapping:**
```
Sprint X.Y1 (Foundation: Plugin Infrastructure)
    ↓
Sprint X.Y2 (Builds on Y1: Example Plugins)
    ↓
Sprint X.Y3 (Independent: Help System)
    ↓
Sprint X.Y4 (Depends on Y2: Plugin Marketplace)
    ├─→ Sprint X.Y5 (Independent: Performance Optimization)
    └─→ Sprint X.Y6 (Independent: Output Format Enhancement)
    ↓
Sprint X.Y7 (Integration: TUI Interface Foundation)
    ↓
Sprint X.Y8 (Final: Documentation & Release Prep)
```

**Visualization:** Use ASCII art for dependency diagram

**4.5 Timeline Estimation:**
- Sprint duration: 3-5 days each
- Total: 24-40 days (4-8 weeks)
- Buffer: +20% for unexpected issues
- Final estimate: 5-10 weeks

**Verification Checklist:**
- [ ] 8 sprints defined with clear objectives (4 for --quick mode)
- [ ] Each sprint has 7-12 tasks with time estimates
- [ ] Total time per sprint realistic (35-50h = 5-7 days)
- [ ] Dependencies mapped (visual diagram)
- [ ] Effort realistic (24-40 days total → 5-8 weeks)
- [ ] Mix of quick wins (HIGH ROI) and strategic work
- [ ] Success criteria specific and measurable
- [ ] All sprints use consistent template format
- [ ] Risk mitigation documented

---

### PHASE 5: DOCUMENT GENERATION (60 minutes)

**Objective:** Create comprehensive `docs/XX-PHASEXX-ENHANCEMENTS.md`

**5.1 Document Structure:**

```markdown
# Phase X Post-Analysis: Enhancement Roadmap

**Created:** [DATE]
**Analyst:** Claude Code + Sub-Agent (AI-Powered Competitive Intelligence)
**Status:** Pre-Phase Y Enhancement Sprints
**Duration:** 8 sprints (5-8 weeks estimated)
**Goal:** Maximize competitive advantages before Phase Y

**Research Methodology:**
- 📚 Local code analysis (code_ref/)
- 🔍 GitHub repository deep-dive (MCP-powered)
- 🌐 Online community research (Browser MCP)
- 🧠 Knowledge graph integration (competitive landscape mapping)

---

## Executive Summary

[3-4 paragraphs summarizing findings and strategy]

**Phase X Status:**
- Current Version: [vX.Y.Z]
- Tests: [COUNT] passing ([COVERAGE]% coverage)
- Performance: [KEY METRICS]
- Features: [COUNT] scan types, [COUNT] protocols
- Recent Completions: [List 3-5 major recent achievements]

**Key Findings:**

**Strengths** (Where ProRT-IP Excels):
- 🎯 [Strength 1 with metric/evidence]
- 🎯 [Strength 2 with metric/evidence]
- 🎯 [Strength 3 with metric/evidence]
- 🎯 [Strength 4 with metric/evidence]
- 🎯 [Strength 5 with metric/evidence]
- 🎯 [Strength 6 with metric/evidence]
- 🎯 [Strength 7 with metric/evidence]

**Gaps** (Requiring Attention):
- ⚠️ [HIGH] [Gap 1 with competitive context]
- ⚠️ [HIGH] [Gap 2 with competitive context]
- ⚠️ [MEDIUM] [Gap 3 with competitive context]
- ⚠️ [MEDIUM] [Gap 4 with competitive context]
- ⚠️ [LOW] [Gap 5 with competitive context]

**Quick Wins** (High ROI, Low Effort):
- ⚡ [Quick Win 1: ROI X.X/10, Est. 2-3 days]
- ⚡ [Quick Win 2: ROI X.X/10, Est. 2-3 days]
- ⚡ [Quick Win 3: ROI X.X/10, Est. 3-4 days]

**Strategic Differentiators** (Innovation Opportunities):
- 💡 [Differentiator 1: Unique to ProRT-IP]
- 💡 [Differentiator 2: Leverage Rust strengths]
- 💡 [Differentiator 3: Modern architecture advantage]

**Recommended Sprint Roadmap:**

| Sprint | Title | Priority | Duration | ROI | Strategic Value |
|--------|-------|----------|----------|-----|-----------------|
| X.Y1 | [Title] | HIGH | 5d | 9.2/10 | Foundation for Y2-Y4 |
| X.Y2 | [Title] | HIGH | 4d | 8.8/10 | Feature parity with Nmap |
| X.Y3 | [Title] | MEDIUM | 5d | 8.5/10 | UX improvement |
| X.Y4 | [Title] | HIGH | 6d | 8.2/10 | Performance optimization |
| X.Y5 | [Title] | MEDIUM | 4d | 7.8/10 | Innovation differentiator |
| X.Y6 | [Title] | MEDIUM | 5d | 7.5/10 | Polish & quality |
| X.Y7 | [Title] | LOW | 3d | 7.0/10 | Nice-to-have enhancement |
| X.Y8 | [Title] | LOW | 4d | 6.5/10 | Documentation & release prep |

**Total Duration:** 36 days (7.2 weeks) + 20% buffer = **9 weeks estimated**

---

## Competitive Analysis Summary

### Research Methodology

**Analysis Period:** [START DATE] - [END DATE] (Duration: [HOURS])

**Sources Analyzed:**

**📁 Local Code References:**
- [File 1]: [Path] - [Key insights, 1-2 sentences]
- [File 2]: [Path] - [Key insights, 1-2 sentences]
- ...
- **Total:** [COUNT] files analyzed

**🔍 GitHub Repositories (MCP-Powered):**
- **nmap/nmap** ([STARS]⭐, last commit: [DATE])
  - Commits reviewed: [COUNT] (last 6 months)
  - Key features: NSE scripts, OS fingerprinting, service detection
  - Recent additions: [Feature 1], [Feature 2]
  - Repository URL: https://github.com/nmap/nmap
- **RustScan/RustScan** ([STARS]⭐, last commit: [DATE])
  - Commits reviewed: [COUNT]
  - Key features: Async Rust, 3-8s full scan claim
  - Recent additions: [Feature 1], [Feature 2]
  - Repository URL: https://github.com/RustScan/RustScan
- **robertdavidgraham/masscan** ([STARS]⭐, last commit: [DATE])
  - Key features: Stateless scanning, 10M pps claim
  - Architecture: Event loop, SipHash, batch transmit
  - Repository URL: https://github.com/robertdavidgraham/masscan
- **projectdiscovery/naabu** ([STARS]⭐, last commit: [DATE])
  - Key features: YAML config, multiple output formats
  - Repository URL: https://github.com/projectdiscovery/naabu
- **Total:** [COUNT] repositories analyzed

**🌐 Online Community Research:**
- Reddit discussions: [COUNT] threads (r/netsec, r/AskNetsec)
- Hacker News: [COUNT] discussions
- Blog posts: [COUNT] articles
- Stack Overflow: [COUNT] Q&A threads
- Documentation: [LIST official docs]
- **Total:** [COUNT] online sources

**🧠 Knowledge Graph Integration:**
- Entities created: [COUNT] (competitors, features, gaps, opportunities)
- Relations mapped: [COUNT] (competitive relationships, feature dependencies)
- Historical insights: [COUNT] previous analysis findings integrated

**Competitors Analyzed:** [COUNT] - Nmap, Masscan, RustScan, Naabu, [Others]

### Strengths Relative to Competitors

[Detailed analysis with evidence - 7-10 strengths]

**1. [Strength 1 Title]** (Impact: HIGH)
- **ProRT-IP Advantage:** [Specific details with metrics]
- **Competitor Comparison:**
  - Nmap: [Status]
  - Masscan: [Status]
  - RustScan: [Status]
  - Naabu: [Status]
- **Evidence:** [Metrics, comparisons, facts]
- **Strategic Value:** [Why this matters long-term]

[Repeat for each strength 2-7]

### Gaps Requiring Attention

[Detailed analysis with severity - 7-10 gaps]

**1. [Gap 1 Title]** (Severity: HIGH, Effort: MEDIUM, ROI: 8.5/10)
- **Current State:** [What ProRT-IP has/lacks with specifics]
- **Competitor State:**
  - Nmap: [Specific implementation details]
  - RustScan: [Specific implementation details]
  - Others: [Brief mention]
- **Impact:** [User/competitive impact - who benefits, how]
- **Effort Assessment:**
  - Time: [DAYS] days
  - Complexity: [EASY/MEDIUM/HARD]
  - Dependencies: [List prerequisites]
- **ROI Calculation:** (User Impact: X/10 × Competitive Gap: Y/10) / Effort: Z/10 = **[SCORE]/10**
- **Recommended Sprint:** [Sprint number that addresses this]

[Repeat for each gap 2-7]

### Innovation Opportunities

[Areas where ProRT-IP can lead - 3-5 opportunities]

**1. [Opportunity Title]** (Strategic Value: HIGH)
- **Market Opportunity:** [Why this is valuable - market needs, trends]
- **Competitive Landscape:** [What competitors lack]
- **ProRT-IP Position:** [How we can leverage our strengths]
- **Implementation Strategy:** [High-level approach]
- **Expected Impact:** [Outcome - differentiation, user benefit]
- **Risk/Considerations:** [Challenges to mitigate]
- **Recommended Sprint:** [Sprint number that pursues this]

[Repeat for each opportunity 2-5]

---

## Sprint Roadmap

[Detailed sprint descriptions - 8 sprints using template from Phase 4]

### Sprint X.Y1: [Title]

- **Priority:** HIGH
- **Duration:** 5 days (40h)
- **Dependencies:** None (Foundation sprint)
- **ROI Score:** 9.2/10
- **Effort:** MEDIUM

**Objective:** [Clear, measurable goal in 2-3 sentences]

**Rationale:** [Why now? Competitive findings - 2-3 paragraphs]

**Current State:**
- ProRT-IP: [What we have/lack]
- Metrics: [Current performance/features]

**Competitor State:**
- Nmap: [Specific implementation with details]
- RustScan: [Specific implementation with details]
- Others: [Brief mentions]

**Impact:**
- User Benefit: [How users benefit]
- Competitive Position: [How this closes gap]
- Strategic Value: [Long-term advantage]

**Tasks:** [7-12 specific tasks with time estimates]
1. [ ] Research: Analyze Nmap's X implementation (GitHub code review) (2h)
2. [ ] Research: Analyze RustScan's Y approach (code_ref/ review) (2h)
3. [ ] Design: Create architecture document (docs/DESIGN-X.md) (3h)
4. [ ] Design: API design and integration points (2h)
5. [ ] Implement: Core Z module (src/module/core.rs) (8h)
6. [ ] Implement: Integration with existing A subsystem (4h)
7. [ ] Testing: Unit tests (90%+ coverage target) (4h)
8. [ ] Testing: Integration tests (test scenarios) (3h)
9. [ ] Documentation: Technical guide (docs/XX-FEATURE-GUIDE.md) (4h)
10. [ ] Performance: Benchmarks and baseline measurement (2h)
11. [ ] Performance: Optimization based on benchmarks (3h)
12. [ ] Polish: Error handling, edge cases, validation (2h)
13. [ ] Review: Code review and refinement (2h)

**Total Time:** 41h (5-day sprint with buffer)

**Deliverables:**
- [ ] Core implementation: [Module/file paths]
- [ ] Tests: 90%+ coverage for new code (estimated: +[COUNT] tests)
- [ ] Documentation:
  - [ ] README.md update (features section)
  - [ ] CHANGELOG.md entry (sprint summary)
  - [ ] docs/XX-FEATURE-GUIDE.md ([PAGES] pages)
  - [ ] API documentation (rustdoc)
- [ ] Benchmarks: [Scenarios with baseline metrics]
- [ ] Examples: [COUNT] copy-paste ready examples

**Success Criteria:**

**Quantitative Metrics:**
- [ ] Feature X operational with [METRIC] (e.g., "5+ hook types supported")
- [ ] Performance: [METRIC] (e.g., "overhead <2%")
- [ ] Test coverage: 90%+ for new code
- [ ] Documentation: [METRIC] (e.g., "500+ line guide")
- [ ] Zero regression: All existing tests pass

**Qualitative Indicators:**
- [ ] Nmap parity: Feature matches Nmap's X functionality
- [ ] Professional quality: A+ grade documentation
- [ ] User experience: Intuitive API design
- [ ] Code quality: Zero clippy warnings

**References:**
- **Code:**
  - code_ref/nmap/[FILE]: [Specific implementation]
  - code_ref/rustscan/[FILE]: [Specific pattern]
  - ProRT-IP crates/prtip-scanner/src/[MODULE]: [Integration point]
- **Research:**
  - RFC [NUMBER]: [Title] - [URL]
  - Article: [Title] - [URL]
  - Nmap docs: [URL]
- **Competitors:**
  - Nmap implementation: [GitHub URL with line numbers]
  - RustScan approach: [GitHub URL]

**Risk Mitigation:**
- **Risk 1:** [Description]
  - **Mitigation:** [Strategy]
  - **Contingency:** [Fallback plan]
- **Risk 2:** [Description]
  - **Mitigation:** [Strategy]
  - **Contingency:** [Fallback plan]

**Dependencies:**
- None (foundation sprint)

**Enables Future Work:**
- Sprint X.Y2: [How this sprint enables]
- Sprint X.Y4: [How this sprint enables]

---

[Repeat full sprint template for Sprints X.Y2 through X.Y8]

---

## Sprint Dependency Diagram

```
Phase X Enhancement Roadmap
============================

Sprint X.Y1: [Title]
(Foundation: Plugin Infrastructure)
        ↓
Sprint X.Y2: [Title]
(Builds on Y1: Example Plugins)
        ↓
Sprint X.Y3: [Title]                Sprint X.Y5: [Title]
(Independent: Help System)          (Independent: Performance)
        ↓                                   ↓
Sprint X.Y4: [Title] ←──────────────────────┘
(Depends on Y2, Y3: Plugin Marketplace)
        ↓
Sprint X.Y6: [Title]                Sprint X.Y7: [Title]
(Independent: Output Format)        (Foundation: TUI)
        ↓                                   ↓
        └────────────→ Sprint X.Y8: [Title] ←┘
                    (Final: Documentation & Release Prep)

Legend:
  ↓  = Sequential dependency (must complete before)
  ←  = Parallel merge (both required)
```

---

## Appendix A: Competitive Feature Matrix

[Complete feature comparison table with 15+ categories]

| Feature Category | ProRT-IP | Nmap | Masscan | RustScan | Naabu | Gap Analysis |
|------------------|----------|------|---------|----------|-------|--------------|
| **Scan Types** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **Service Detection** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **OS Fingerprinting** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **Performance** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **Output Formats** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **Stealth Features** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **IPv6 Support** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **Scripting** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **CLI Design** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **Documentation** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **Testing** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **Architecture** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **Plugin System** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **TLS Analysis** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |
| **Rate Limiting** | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Detailed] | [Analysis] |

[Include row-by-row detailed analysis]

---

## Appendix B: Performance Benchmarks

### Current Performance (v[X.Y.Z])

**Test Environment:**
- System: [Specs: CPU, RAM, OS]
- Network: [Environment: local/cloud, bandwidth]
- Target: [Test target description]

**Benchmark Results:**

| Scenario | ProRT-IP | Nmap | Masscan | RustScan | Winner |
|----------|----------|------|---------|----------|--------|
| 1000 ports, 1 host | [TIME] | [TIME] | [TIME] | [TIME] | [WINNER] |
| Top 100 ports, /24 | [TIME] | [TIME] | [TIME] | [TIME] | [WINNER] |
| Full scan, 1 host | [TIME] | [TIME] | [TIME] | [TIME] | [WINNER] |
| Service detection | [TIME] | [TIME] | [N/A] | [TIME] | [WINNER] |
| Memory usage (avg) | [MB] | [MB] | [MB] | [MB] | [WINNER] |
| Packet rate (peak) | [pps] | [pps] | [pps] | [pps] | [WINNER] |

### Projected Performance (After Enhancement Sprints)

**Expected Improvements:**

| Metric | Current | Target | Improvement | Sprint |
|--------|---------|--------|-------------|--------|
| [Metric 1] | [VALUE] | [VALUE] | +[X]% | X.Y4 |
| [Metric 2] | [VALUE] | [VALUE] | +[X]% | X.Y5 |
| [Metric 3] | [VALUE] | [VALUE] | +[X]% | X.Y7 |

**Competitive Position After Enhancements:**

[Narrative: Where ProRT-IP will stand vs competitors after completing roadmap]

---

## Appendix C: Research Sources

### Code References (Local Analysis)

**code_ref/ Directory:**
- **[FILE 1]:** [Path]
  - **Size:** [LINES] lines
  - **Language:** [LANGUAGE]
  - **Key Insights:** [1-2 sentences]
  - **Applicable to Sprints:** [X.Y1, X.Y3]
- **[FILE 2]:** [Path]
  - **Size:** [LINES] lines
  - **Language:** [LANGUAGE]
  - **Key Insights:** [1-2 sentences]
  - **Applicable to Sprints:** [X.Y2]
- ...

**Total:** [COUNT] files analyzed

### GitHub Repositories (MCP-Powered Research)

**1. nmap/nmap**
- **URL:** https://github.com/nmap/nmap
- **Stars:** [COUNT] ⭐
- **Language:** C++
- **Last Commit:** [DATE]
- **Activity:** [Active/Moderate/Low]
- **Commits Reviewed:** [COUNT] (last 6 months)
  - [COMMIT SHA]: [Title] - [Key insight]
  - [COMMIT SHA]: [Title] - [Key insight]
  - ...
- **Files Analyzed:**
  - NmapOps.h: [Key insights, 2-3 sentences]
  - service_scan.cc: [Key insights]
  - idle_scan.cc: [Key insights]
- **Features Identified:**
  - NSE scripting system (600+ scripts)
  - OS fingerprinting (2,600+ signatures)
  - Service detection (nmap-service-probes)
  - Timing templates (T0-T5)
- **Insights for ProRT-IP:** [Paragraph summarizing applicable lessons]
- **Referenced in Sprints:** [X.Y1, X.Y2, X.Y4]

**2. RustScan/RustScan**
- [Same structure as above]

**3. robertdavidgraham/masscan**
- [Same structure as above]

**4. projectdiscovery/naabu**
- [Same structure as above]

### Online Resources (Browser MCP Research)

**Reddit Discussions:**
- **r/netsec: "Port Scanning Tools Comparison 2025"**
  - URL: [REDDIT URL]
  - Date: [DATE]
  - Upvotes: [COUNT]
  - Comments: [COUNT]
  - Key Insights: [Summary, 2-3 sentences]
  - Applicable to: [Sprints X.Y1, X.Y3]
- [More discussions...]

**Hacker News:**
- **"Show HN: RustScan - Faster than Nmap?"**
  - URL: [HN URL]
  - Date: [DATE]
  - Points: [COUNT]
  - Comments: [COUNT]
  - Key Insights: [Summary]
  - Applicable to: [Sprint X.Y5]
- [More discussions...]

**Articles & Blog Posts:**
- **"Network Scanning Performance Benchmarks"**
  - URL: [URL]
  - Author: [NAME]
  - Date: [DATE]
  - Key Insights: [Summary]
  - Applicable to: [Sprint X.Y4, X.Y5]
- [More articles...]

**Official Documentation:**
- **Nmap Reference Guide**
  - URL: https://nmap.org/book/man.html
  - Sections Reviewed: [List]
  - Key Insights: [Summary]
- [More docs...]

### Knowledge Graph (Competitive Intelligence)

**Entities Created:** [COUNT]
- Competitor entities: [LIST]
- Feature entities: [LIST]
- Gap entities: [LIST]
- Opportunity entities: [LIST]

**Relations Mapped:** [COUNT]
- Competitive relationships: [COUNT]
- Feature comparisons: [COUNT]
- Dependency chains: [COUNT]

**Historical Insights Integrated:** [COUNT]
- Previous /inspire-me analysis: [DATE]
- Sprint completion learnings: [SPRINTS]
- User feedback patterns: [THEMES]

---

## Appendix D: Decision Log

| Date | Decision | Rationale | Expected Impact | Alternative Considered |
|------|----------|-----------|-----------------|------------------------|
| [DATE] | Sprint prioritization order | ROI-based ranking with dependency constraints. Y1 foundation enables Y2-Y4. Y8 final polish before release. | Optimal value delivery sequence | Random order (rejected: violates dependencies) |
| [DATE] | Defer [Feature X] to Phase Y | Competitive gap is LOW severity, effort is HIGH, ROI only 4.2/10. Better opportunities available. | Focus resources on high-impact work | Include in Phase X (rejected: dilutes focus) |
| [DATE] | Pursue innovation opportunity: [Feature Z] | Unique differentiator leveraging Rust strengths. No competitor has equivalent. HIGH strategic value. | Market positioning advantage | Nmap parity approach (rejected: "me too" strategy) |
| [DATE] | 8 sprints vs 6 sprints | Additional 2 sprints enable comprehensive coverage of gaps. 9-week timeline acceptable for phase. | Thorough enhancement cycle | 6 sprints (rejected: leaves critical gaps) |
| [DATE] | Use MCP servers for research | GitHub MCP + Browser MCP provide deeper insights than manual research. Knowledge graph enables learning retention. | Higher quality analysis, reproducible methodology | Manual web research (rejected: time-consuming, less comprehensive) |

---

## Appendix E: ROI Scoring Details

### Scoring Methodology

**User Impact (1-10):**
- 9-10: Critical feature, blocks major use case
- 7-8: High value, significant UX improvement
- 5-6: Moderate value, nice-to-have enhancement
- 3-4: Low value, minor improvement
- 1-2: Minimal value, edge case

**Competitive Gap (1-10):**
- 9-10: All competitors have it, ProRT-IP doesn't
- 7-8: Most competitors have it
- 5-6: Some competitors have it
- 3-4: Few competitors have it
- 1-2: No competitor has it (innovation)

**Effort (1-10, lower is easier):**
- 9-10: >10 days, complex integration, high risk
- 7-8: 5-10 days, moderate complexity
- 5-6: 3-5 days, standard implementation
- 3-4: 2-3 days, straightforward
- 1-2: <2 days, trivial

**ROI Formula:** `(User Impact × Competitive Gap) / Effort`

### Enhancement Scores (Complete List)

| Enhancement | User Impact | Competitive Gap | Effort | ROI | Priority | Sprint |
|-------------|-------------|-----------------|--------|-----|----------|--------|
| [Enhancement 1] | 9 | 10 | 7 | 12.9 | HIGH | X.Y1 |
| [Enhancement 2] | 8 | 9 | 6 | 12.0 | HIGH | X.Y2 |
| [Enhancement 3] | 7 | 9 | 5 | 12.6 | HIGH | X.Y3 |
| [Enhancement 4] | 8 | 8 | 6 | 10.7 | MEDIUM | X.Y4 |
| [Enhancement 5] | 6 | 8 | 4 | 12.0 | MEDIUM | X.Y5 |
| [Enhancement 6] | 7 | 6 | 5 | 8.4 | MEDIUM | X.Y6 |
| [Enhancement 7] | 5 | 6 | 4 | 7.5 | LOW | X.Y7 |
| [Enhancement 8] | 6 | 5 | 5 | 6.0 | LOW | X.Y8 |
| [Not Selected 1] | 4 | 5 | 8 | 2.5 | DEFERRED | - |
| [Not Selected 2] | 3 | 4 | 7 | 1.7 | DEFERRED | - |
| ... | | | | | | |

**Total Enhancements Evaluated:** [COUNT]
**Selected for Roadmap:** 8
**Deferred to Future Phase:** [COUNT]

---

## Next Review & Success Criteria

**Review Trigger:** End of Sprint X.Y8 (before Phase Y planning)

**Success Criteria for Roadmap Complete:**

**Quantitative Metrics:**
- ✅ All 8 sprints delivered (100% completion)
- ✅ Test count: [CURRENT] → [TARGET] (+[INCREMENT] tests)
- ✅ Code coverage: [CURRENT]% → [TARGET]% (+[INCREMENT]%)
- ✅ Performance: [METRIC] improved by [TARGET]%
- ✅ Feature parity: [COUNT] gaps closed
- ✅ Zero critical bugs
- ✅ Zero regressions (all existing tests pass)
- ✅ Version [X.Y.Z] released

**Qualitative Indicators:**
- ✅ Professional documentation quality (A+ grade)
- ✅ Competitive position: [POSITION vs competitors]
- ✅ User feedback: [POSITIVE themes]
- ✅ Community engagement: [METRICS: stars, forks, discussions]
- ✅ Strategic differentiation: [UNIQUE advantages validated]

**Deliverables:**
- ✅ All sprint deliverables complete (see individual sprints)
- ✅ Comprehensive testing (unit, integration, fuzz, benchmarks)
- ✅ Documentation updates (README, CHANGELOG, guides, API)
- ✅ GitHub release with release notes
- ✅ Post-phase retrospective (lessons learned)

**Next Steps After Completion:**
1. Post-phase retrospective: Capture lessons learned
2. Phase Y planning: Use `/inspire-me` again for next cycle
3. Roadmap update: Mark Phase X complete, adjust Phase Y
4. Community announcement: Share achievements and next direction

---

**Document End**

**Status:** READY FOR EXECUTION
**Estimated Timeline:** [START DATE] → [END DATE] (9 weeks)
**Total Effort:** ~320 hours across 8 sprints

---

🤖 **Generated by Claude Code + Sub-Agent**
**Competitive Intelligence System:** AI-Powered Research & Enhancement Planning
**Tools Used:** GitHub MCP, Browser MCP, Knowledge Graph MCP, Code Search MCP
**Quality Grade:** A+ (Target achieved)

**📊 Document Statistics:**
- Word Count: [COUNT] words (Target: >10,000 ✓)
- Page Count: [COUNT] pages (Target: >30 ✓)
- Sprints: 8 detailed (Target: 8 ✓)
- Research Sources: [COUNT] (Target: >15 ✓)
- Feature Matrix Categories: [COUNT] (Target: >10 ✓)
```

**5.2 Quality Standards:**

Document MUST include:
- [ ] >10,000 words (comprehensive coverage)
- [ ] All 8 sprints fully detailed (tasks, estimates, success criteria, risks)
- [ ] Feature matrix with 15+ categories
- [ ] Performance benchmarks (current + projected)
- [ ] 20+ research sources cited with URLs/paths
- [ ] All research validated and factual
- [ ] Executive summary (3-4 paragraphs)
- [ ] Decision log with sprint prioritization rationale
- [ ] ROI scoring details (complete table)
- [ ] Sprint dependency diagram (ASCII art visualization)
- [ ] Professional formatting and proofreading
- [ ] Knowledge graph integration documented
- [ ] MCP tool usage documented

**5.3 File Naming:**

```bash
# Determine next number in docs/ directory
FILES=$(ls docs/*.md 2>/dev/null | grep -E "^docs/[0-9]+" | wc -l)
NEXT_NUM=$((FILES + 1))
NEXT_NUM_PADDED=$(printf "%02d" $NEXT_NUM)

# Determine phase number (user input or auto-detect)
PHASE_NUM="[X]"  # From Phase 1 context gathering

# Format: docs/XX-PHASEXX-ENHANCEMENTS.md
FILENAME="docs/${NEXT_NUM_PADDED}-PHASE${PHASE_NUM}-ENHANCEMENTS.md"

# Example: docs/35-PHASE6-ENHANCEMENTS.md
```

**5.4 Document Statistics Tracking:**

```bash
# Generate statistics
WORD_COUNT=$(wc -w < "$FILENAME")
PAGE_COUNT=$(echo "$WORD_COUNT / 300" | bc)  # Approx 300 words/page
LINE_COUNT=$(wc -l < "$FILENAME")

# Verify quality standards
if [ $WORD_COUNT -lt 10000 ]; then
  echo "⚠️ WARNING: Word count below target (${WORD_COUNT} < 10000)"
fi
```

---

### PHASE 6: VERIFICATION, REPORT & INTEGRATION (45 minutes)

**Objective:** Validate completeness, generate summary, integrate with knowledge systems

**6.1 Completeness Checklist:**

```markdown
## Enhancement Document Quality Check

**Structure:**
- [ ] Executive summary (3-4 paragraphs)
- [ ] Key findings (4 sections: Strengths, Gaps, Quick Wins, Differentiators)
- [ ] Competitive analysis (4 sections: Methodology, Strengths, Gaps, Opportunities)
- [ ] 8 sprints fully detailed (all using template from Phase 4.2)
- [ ] Sprint dependency diagram (ASCII art visualization)
- [ ] Feature matrix (15+ categories, 5+ competitors)
- [ ] Performance benchmarks (current + projected)
- [ ] Research sources (20+ items with URLs/paths)
- [ ] Decision log (sprint prioritization + alternatives)
- [ ] ROI scoring appendix (complete methodology + all scores)
- [ ] All appendices complete (A through E)

**Quality:**
- [ ] >10,000 words (comprehensive) - VERIFIED: [COUNT] words
- [ ] All research sources cited properly
- [ ] All metrics and comparisons accurate
- [ ] No broken links or invalid references
- [ ] Consistent formatting throughout
- [ ] Professional appearance (A+ grade)
- [ ] Proofread for typos and grammar
- [ ] All task time estimates realistic
- [ ] All ROI scores calculated correctly

**Verification:**
- [ ] Proofread entire document (2 passes minimum)
- [ ] Verify all file paths valid (code_ref/, docs/, crates/)
- [ ] Check all URLs accessible (GitHub, Reddit, blogs)
- [ ] Validate metrics against source data
- [ ] Review sprint estimates realistic (35-50h per sprint)
- [ ] Confirm dependency diagram matches sprint descriptions
- [ ] All sprints use consistent template format
- [ ] Knowledge graph integration documented

**MCP Integration:**
- [ ] All GitHub MCP searches documented
- [ ] Browser MCP research sessions logged
- [ ] Knowledge graph entities created
- [ ] Code search results incorporated
```

**6.2 Generate Completion Report:**

Create: `/tmp/ProRT-IP/inspire-me-report.md`

```markdown
# /inspire-me Execution Report

**Date:** [TIMESTAMP]
**Duration:** [ELAPSED_TIME] (Target: 3-4h)
**Status:** ✅ COMPLETE
**Quality Grade:** A+ (all standards met)

---

## Summary

Created comprehensive Phase X enhancement document analyzing [COUNT] competitors and identifying [COUNT] enhancement opportunities through AI-powered competitive intelligence.

**Document Created:** `docs/XX-PHASEXX-ENHANCEMENTS.md`
**Size:** [WORD_COUNT] words ([PAGE_COUNT] pages)
**Sprints Planned:** 8 sprints (9 weeks estimated, 320h total effort)
**Quality:** A+ grade (exceeded all targets)

---

## Research Statistics

**MCP-Powered Research:**

**GitHub MCP:**
- Repositories analyzed: [COUNT]
- Code searches performed: [COUNT]
- Commits reviewed: [COUNT]
- Files examined: [COUNT]
- Insights generated: [COUNT]

**Browser MCP:**
- Web pages analyzed: [COUNT]
- Reddit threads: [COUNT]
- Hacker News discussions: [COUNT]
- Blog posts: [COUNT]
- Documentation pages: [COUNT]

**Knowledge Graph MCP:**
- Entities created: [COUNT]
- Relations mapped: [COUNT]
- Historical insights integrated: [COUNT]
- Queries performed: [COUNT]

**Code Search MCP:**
- Searches executed: [COUNT]
- Code snippets analyzed: [COUNT]
- Patterns identified: [COUNT]

**Traditional Research:**
- Code references (local): [COUNT] files from code_ref/
- Performance benchmarks: [COUNT] metrics compared
- Feature comparisons: [COUNT] categories

**Total Sources:** [COUNT] (Target: >15 ✓ Exceeded)

**Competitive Analysis:**
- Competitors analyzed: [COUNT] (Nmap, Masscan, RustScan, Naabu, [Others])
- Features compared: [COUNT] categories (Target: >10 ✓)
- Performance benchmarks: [COUNT] metrics
- GitHub stars total: [COUNT]⭐
- Combined commits reviewed: [COUNT]
- Gaps identified: [COUNT] (HIGH: X, MEDIUM: Y, LOW: Z)
- Opportunities found: [COUNT]
- Quick wins: [COUNT] (HIGH ROI + LOW effort)

---

## Sprint Summary

### Roadmap Overview

**Total Duration:** 9 weeks (36 working days + 20% buffer)
**Total Effort:** ~320 hours across 8 sprints
**High-Priority Sprints:** [COUNT]
**Medium-Priority:** [COUNT]
**Low-Priority:** [COUNT]

### Sprint Breakdown

| Sprint | Title | Priority | Est. Days | Est. Hours | ROI Score | Strategic Value |
|--------|-------|----------|-----------|------------|-----------|-----------------|
| X.Y1 | [Title] | HIGH | 5 | 40h | 9.2/10 | Foundation for Y2-Y4 |
| X.Y2 | [Title] | HIGH | 4 | 35h | 8.8/10 | Nmap parity |
| X.Y3 | [Title] | MEDIUM | 5 | 40h | 8.5/10 | UX improvement |
| X.Y4 | [Title] | HIGH | 6 | 48h | 8.2/10 | Performance |
| X.Y5 | [Title] | MEDIUM | 4 | 32h | 7.8/10 | Innovation |
| X.Y6 | [Title] | MEDIUM | 5 | 40h | 7.5/10 | Polish |
| X.Y7 | [Title] | LOW | 3 | 28h | 7.0/10 | Enhancement |
| X.Y8 | [Title] | LOW | 4 | 32h | 6.5/10 | Release prep |

**Quick Wins** (HIGH ROI, LOW/MEDIUM Effort):
1. Sprint X.Y[N]: [TITLE] (ROI: X.X/10, Est: 3-4 days)
   - [Brief description of quick win value]
2. Sprint X.Y[N]: [TITLE] (ROI: X.X/10, Est: 4-5 days)
   - [Brief description]
3. Sprint X.Y[N]: [TITLE] (ROI: X.X/10, Est: 3-4 days)
   - [Brief description]

---

## Key Findings

### Strengths (ProRT-IP Advantages)

**Where ProRT-IP Excels:**
1. **[Strength 1]** (Impact: HIGH)
   - [1-2 sentence summary]
   - Evidence: [Metric or comparison]
2. **[Strength 2]** (Impact: HIGH)
   - [1-2 sentence summary]
   - Evidence: [Metric or comparison]
3. **[Strength 3]** (Impact: MEDIUM)
   - [1-2 sentence summary]
   - Evidence: [Metric or comparison]
4. **[Strength 4]** (Impact: MEDIUM)
   - [1-2 sentence summary]
   - Evidence: [Metric or comparison]
5. **[Strength 5]** (Impact: MEDIUM)
   - [1-2 sentence summary]
   - Evidence: [Metric or comparison]

**Total Strengths Identified:** [COUNT]

### Critical Gaps (Requiring Immediate Attention)

**HIGH Severity:**
1. **[Gap 1]** (ROI: X.X/10, Effort: [DAYS] days)
   - Competitor State: [Brief summary]
   - Impact: [User/competitive impact]
   - Addressed in: Sprint X.Y[N]
2. **[Gap 2]** (ROI: X.X/10, Effort: [DAYS] days)
   - Competitor State: [Brief summary]
   - Impact: [User/competitive impact]
   - Addressed in: Sprint X.Y[N]

**MEDIUM Severity:**
3. **[Gap 3]** (ROI: X.X/10, Effort: [DAYS] days)
   - [Brief summary]
   - Addressed in: Sprint X.Y[N]
4. **[Gap 4]** (ROI: X.X/10, Effort: [DAYS] days)
   - [Brief summary]
   - Addressed in: Sprint X.Y[N]

**Total Gaps Identified:** [COUNT]

### Innovation Opportunities (Strategic Differentiators)

**Where ProRT-IP Can Lead:**
1. **[Opportunity 1]** (Strategic Value: HIGH)
   - Market Opportunity: [1-2 sentences]
   - ProRT-IP Advantage: [How we leverage our strengths]
   - Addressed in: Sprint X.Y[N]
2. **[Opportunity 2]** (Strategic Value: HIGH)
   - Market Opportunity: [1-2 sentences]
   - ProRT-IP Advantage: [How we leverage our strengths]
   - Addressed in: Sprint X.Y[N]
3. **[Opportunity 3]** (Strategic Value: MEDIUM)
   - Market Opportunity: [1-2 sentences]
   - ProRT-IP Advantage: [How we leverage our strengths]
   - Addressed in: Sprint X.Y[N]

**Total Opportunities Identified:** [COUNT]

---

## Knowledge Graph Integration

**Competitive Intelligence Captured:**

**Entities Created:** [COUNT]
- Competitors: [LIST: Nmap, Masscan, RustScan, Naabu, Others]
- Features: [LIST: Top 5-7 features]
- Gaps: [LIST: Top 5-7 gaps]
- Opportunities: [LIST: Top 3-5 opportunities]
- Performance Metrics: [LIST: Key metrics]

**Relations Mapped:** [COUNT]
- Competitive relationships: [COUNT] (e.g., "competes_with", "exceeds", "lags_behind")
- Feature dependencies: [COUNT] (e.g., "requires", "enables", "enhances")
- Strategic connections: [COUNT] (e.g., "differentiates_via", "leverages")

**Historical Context:**
- Previous /inspire-me analysis: [DATE] (integrated: [COUNT] insights)
- Sprint learnings: [SPRINTS] (integrated: [COUNT] lessons)
- User feedback: [THEMES] (integrated: [COUNT] patterns)

**Query Capabilities:**
- "What are ProRT-IP's competitive advantages?" → [COUNT] results
- "Which gaps are HIGH priority?" → [COUNT] results
- "Show innovation opportunities" → [COUNT] results

**Knowledge Retention:**
All competitive intelligence persists in knowledge graph for:
- Future /inspire-me invocations (learning continuity)
- Sprint planning decisions (informed prioritization)
- Feature development (competitive context)
- Strategic planning (market positioning)

---

## Next Steps

### Immediate Actions

1. **Review:** `docs/XX-PHASEXX-ENHANCEMENTS.md`
   - Comprehensive analysis ([WORD_COUNT] words, [PAGE_COUNT] pages)
   - All 8 sprints detailed with tasks, metrics, timelines
   - Feature matrix, benchmarks, research sources

2. **Approve:** Sprint roadmap
   - 8 sprints, 9-week timeline, 320h effort
   - Dependencies mapped, ROI validated
   - Quick wins identified

3. **Begin:** Sprint X.Y1 [TITLE]
   - Highest priority (ROI: X.X/10)
   - Foundation for [LIST dependent sprints]
   - Estimated: [DAYS] days, [HOURS] hours

4. **Track:** Progress monitoring
   - Create: `to-dos/SPRINT-X.Y1-TODO.md`
   - Use: `/sprint-complete` upon completion
   - Update: PROJECT-STATUS.md with sprint progress

### Long-Term Strategy

**Phase X Enhancement Cycle:** [START DATE] → [END DATE] (9 weeks)
- Week 1-2: Sprints X.Y1-Y2 (foundations + high-priority)
- Week 3-5: Sprints X.Y3-Y5 (core enhancements)
- Week 6-7: Sprints X.Y6-Y7 (polish + quality)
- Week 8-9: Sprint X.Y8 (documentation + release)

**Phase Y Planning:** After Sprint X.Y8 completion
- Run `/inspire-me` again for next enhancement cycle
- Integrate lessons learned from Phase X
- Leverage knowledge graph insights

**Continuous Improvement:**
- Monthly competitive monitoring (GitHub activity, blog posts)
- Quarterly benchmark validation (performance regressions)
- Annual strategic review (market positioning, roadmap alignment)

---

## Files Created/Modified

### Created Files

1. **`docs/XX-PHASEXX-ENHANCEMENTS.md`**
   - Size: [WORD_COUNT] words ([LINE_COUNT] lines, [PAGE_COUNT] pages)
   - Sections: 8 (Executive Summary through Appendices)
   - Sprints: 8 detailed
   - Research sources: [COUNT]
   - Quality: A+ grade

2. **`/tmp/ProRT-IP/inspire-me-report.md`**
   - Size: [WORD_COUNT] words
   - Purpose: Execution summary (this file)

### Modified Files

1. **`CLAUDE.local.md`**
   - Added session entry: `/inspire-me Execution`
   - Updated: Recent Sessions table
   - Impact: Session history preserved

### Knowledge Graph Updates

- Entities: [COUNT] created/updated
- Relations: [COUNT] created/updated
- Observations: [COUNT] added
- Queries: Competitive intelligence now queryable

---

## MCP Server Usage Summary

**Tools Used:** 4 MCP servers, [COUNT] total tool invocations

### GitHub MCP (mcp__MCP_DOCKER__)
- `get_file_contents`: [COUNT] calls ([FILES] files)
- `search_repositories`: [COUNT] calls ([REPOS] found)
- `search_code`: [COUNT] calls ([RESULTS] matches)
- `list_commits`: [COUNT] calls ([COMMITS] analyzed)
- `list_pull_requests`: [COUNT] calls ([PRS] reviewed)
- **Insight:** [1-2 sentences about GitHub research value]

### Browser MCP (mcp__MCP_DOCKER__)
- `browser_navigate`: [COUNT] calls ([SITES] unique sites)
- `browser_snapshot`: [COUNT] calls ([PAGES] captured)
- **Sites Visited:** [LIST: reddit.com, news.ycombinator.com, nmap.org, ...]
- **Insight:** [1-2 sentences about web research value]

### Knowledge Graph MCP (mcp__MCP_DOCKER__)
- `create_entities`: [COUNT] calls ([ENTITIES] created)
- `create_relations`: [COUNT] calls ([RELATIONS] mapped)
- `search_nodes`: [COUNT] calls ([RESULTS] found)
- `open_nodes`: [COUNT] calls ([NODES] retrieved)
- `add_observations`: [COUNT] calls ([OBSERVATIONS] added)
- **Insight:** [1-2 sentences about knowledge retention value]

### Code Search MCP (mcp__MCP_DOCKER__)
- `search_code`: [COUNT] calls ([SNIPPETS] analyzed)
- **Languages:** [LIST: C++, Rust, Go, C]
- **Repositories:** [LIST: nmap/nmap, RustScan/RustScan, ...]
- **Insight:** [1-2 sentences about code analysis value]

---

## Performance Metrics

**Execution Performance:**
- Total duration: [ELAPSED_TIME] (Target: 3-4h)
- Phase 1 (Context): [TIME]
- Phase 2 (Research): [TIME] (MCP-accelerated)
- Phase 3 (Analysis): [TIME]
- Phase 4 (Planning): [TIME]
- Phase 5 (Documentation): [TIME]
- Phase 6 (Verification): [TIME]

**Output Metrics:**
- Document size: [WORD_COUNT] words (Target: >10,000 ✓)
- Research depth: [COUNT] sources (Target: >15 ✓)
- Sprint detail: 8 complete (Target: 8 ✓)
- Quality grade: A+ (Target: A+ ✓)

**Efficiency:**
- Sources per hour: [RATE]
- Words per hour: [RATE]
- Sprints planned per hour: [RATE]

---

## Quality Assessment

**Document Quality:** A+
- [✓] >10,000 words
- [✓] 8 detailed sprints
- [✓] 15+ categories in feature matrix
- [✓] 20+ research sources
- [✓] Professional formatting
- [✓] Zero broken links
- [✓] Proofread and polished

**Research Quality:** A+
- [✓] 4+ repositories analyzed
- [✓] Multiple online communities
- [✓] Official documentation
- [✓] MCP-powered depth
- [✓] Knowledge graph integration
- [✓] Historical context

**Sprint Planning Quality:** A+
- [✓] Dependencies mapped
- [✓] Per-task time estimates
- [✓] Quantitative + qualitative metrics
- [✓] 7-12 tasks per sprint
- [✓] Risk mitigation documented
- [✓] ROI scores validated

**Overall Grade:** A+ (Exceeds all standards)

---

## Recommendations

### Immediate (This Week)
1. ✅ Review enhancement document (user approval)
2. ✅ Create Sprint X.Y1 TODO: `/sprint-start X.Y1`
3. ✅ Begin Sprint X.Y1 execution
4. ✅ Set up progress tracking (PROJECT-STATUS.md)

### Short-Term (This Phase)
1. Execute all 8 sprints systematically
2. Track progress with `/sprint-complete` after each
3. Monitor competitive landscape monthly
4. Update knowledge graph with learnings

### Long-Term (Next Phase)
1. Run `/inspire-me` again before Phase Y
2. Leverage knowledge graph for continuity
3. Benchmark performance improvements
4. Validate competitive positioning

---

## Status

**✅ READY FOR REVIEW**

**User Action Required:** Review `docs/XX-PHASEXX-ENHANCEMENTS.md` and approve sprint roadmap

**Recommendation:** Begin Sprint X.Y1 [TITLE] immediately (highest priority, enables Y2-Y4)

**Next Command:** `/sprint-start X.Y1` to generate TODO and begin execution

---

🤖 **Generated by ProRT-IP Competitive Analysis System**
**Command:** `/inspire-me`
**Sub-Agent:** AI-Powered Research & Enhancement Planning
**MCP Integration:** GitHub, Browser, Knowledge Graph, Code Search
**Quality:** A+ Grade (All standards exceeded)
**Completion Time:** [TIMESTAMP]

---

**Thank you for using /inspire-me!** 🚀
```

**6.3 Update Memory Banks:**

Add to `CLAUDE.local.md` Recent Sessions:

```markdown
| [DATE] | `/inspire-me Execution` | ~[HOURS]h | Competitive analysis: docs/XX-PHASEXX-ENHANCEMENTS.md ([WORD_COUNT] words), analyzed [COUNT] competitors, planned 8 sprints (9 weeks), MCP-powered research ([COUNT] sources), knowledge graph integration ([COUNT] entities), A+ quality grade | ✅ |
```

**6.4 Knowledge Graph Final Update:**

```bash
# Create master entity for this analysis
mcp__MCP_DOCKER__create_entities: [
  {
    name: "Phase X Enhancement Roadmap",
    entityType: "Strategic Plan",
    observations: [
      "Created: [DATE]",
      "8 sprints planned (9-week timeline)",
      "Research: [COUNT] competitors, [COUNT] sources",
      "Gaps identified: [COUNT] (HIGH: X, MEDIUM: Y, LOW: Z)",
      "Opportunities: [COUNT]",
      "Quick wins: [COUNT]",
      "Quality: A+ grade"
    ]
  }
]

# Link to project context
mcp__MCP_DOCKER__create_relations: [
  {from: "ProRT-IP", to: "Phase X Enhancement Roadmap", relationType: "planned_enhancements"},
  {from: "Phase X Enhancement Roadmap", to: "Sprint X.Y1", relationType: "includes_sprint"},
  {from: "Phase X Enhancement Roadmap", to: "Sprint X.Y2", relationType: "includes_sprint"},
  ... (all 8 sprints)
]
```

---

## INTELLIGENCE & AUTOMATION FEATURES

### Smart Sprint Naming

Generate descriptive titles based on content:
- ✅ "NSE-Equivalent Plugin System" (not "Sprint 6.01")
- ✅ "Performance: Zero-Copy Packet Processing" (not "Optimization Sprint")
- ✅ "UX: Interactive Multi-Page Help System" (not "Documentation Sprint")

**Pattern:** `[Category]: [Specific Feature/Goal]`
- Categories: Performance, Feature, UX, Innovation, Security, Polish, Documentation, Integration

### Adaptive Research Depth

Adjust based on findings:
- **Major gap found (HIGH severity):**
  - Deep dive into competitor implementation (read code, analyze algorithms, benchmark performance)
  - Time allocation: +50% for research phase
  - Example: "NSE scripting system" → Analyze 10+ NSE scripts, study API design, explore Lua integration

- **Minor gap (LOW severity):**
  - Brief analysis sufficient (note feature exists, estimate effort, document API)
  - Time allocation: Standard research phase
  - Example: "Additional output format" → Review format spec, estimate implementation time

- **Innovation opportunity:**
  - Explore cutting-edge research (recent papers, blog posts, experimental features)
  - Time allocation: +30% for exploration phase
  - Example: "Async plugin system" → Research Rust async patterns, Tokio integration, plugin sandboxing

### Context-Aware Recommendations

Tailor sprints to ProRT-IP's architecture:
- **Rust-specific optimizations** (not C-style patterns)
  - Example: "Use zero-copy Bytes instead of Vec<u8> cloning"
- **Async/await patterns** (not thread pools)
  - Example: "tokio::spawn instead of std::thread::spawn"
- **Zero-copy techniques** (if ProRT-IP already uses this pattern)
  - Example: "Extend existing pnet_packet usage for new protocols"
- **Tokio ecosystem libraries** (not generic alternatives)
  - Example: "tokio-util for framing, not custom buffer management"

### ROI-Driven Prioritization

Automatic prioritization with clear rationale:
- Calculate ROI scores for all potential enhancements
- Sort by ROI descending
- Apply dependency constraints (foundations before features)
- Balance quick wins vs strategic work (60/40 split)
- Document all scoring decisions in Appendix E

### MCP-Powered Research

Leverage all available MCP servers:
- **GitHub MCP:** Deep repository analysis, code search, commit history
- **Browser MCP:** Live web research, documentation, community discussions
- **Knowledge Graph MCP:** Competitive intelligence persistence, relationship mapping
- **Code Search MCP:** Cross-repository pattern discovery

### Knowledge Graph Learning

Build persistent competitive intelligence:
- Create entities for competitors, features, gaps, opportunities
- Map relationships (competitive landscape, feature dependencies)
- Integrate historical insights (previous /inspire-me analysis, sprint learnings)
- Enable queries ("What are HIGH priority gaps?", "Show innovation opportunities")
- Retain context across phases (learning continuity)

### Automated Verification

Built-in quality checks:
- Word count validation (>10,000 target)
- Sprint task count (7-12 per sprint)
- Time estimate validation (35-50h per sprint)
- Link checking (all URLs accessible)
- Source citation (20+ sources minimum)
- Template consistency (all sprints use same format)

---

## ERROR HANDLING & EDGE CASES

**Common Issues & Solutions:**

| Issue | Solution | Fallback |
|-------|----------|----------|
| **code_ref/ directory empty** | Skip local analysis, focus on MCP-powered research. Note in report: "No local reference code available" | GitHub MCP code search for competitor implementations |
| **GitHub API rate limited** | Use Browser MCP for public repos (no auth needed). Fallback: Manual documentation review | WebFetch for repository READMEs, documentation |
| **Browser MCP unavailable** | Fall back to WebSearch + WebFetch for research | Warn user about reduced research depth |
| **Knowledge Graph MCP unavailable** | Continue without persistence. Warn: "Competitive intelligence won't persist across sessions" | Document findings in temporary notes |
| **Unclear current phase** | Ask user for clarification. Default: Use "Phase X" placeholder, update after confirmation | Infer from PROJECT-STATUS.md or ROADMAP.md |
| **Insufficient competitive data** | Focus on top 3-4 competitors (Nmap, Masscan, RustScan). Document limitations in report | Prioritize official documentation over community sources |
| **Sprint planning uncertainty** | Prioritize high-ROI items. Include "needs user input" section for unclear priorities | Conservative estimates with explicit buffers |
| **Performance data missing** | Use claimed/estimated values, mark as "unvalidated" or "claimed" in report | Compare feature lists instead of performance |
| **Document too short** | Expand sections: More detailed sprint descriptions, longer research summaries, additional appendices | Generate supplementary analysis documents |
| **Time budget overrun** | Continue to completion (quality over speed). Log actual time for future calibration | Use --quick mode for faster execution |

---

## MODE VARIANTS

### Standard Mode (Default)
- Duration: 3-4 hours
- Sprints: 8 detailed
- Research depth: Comprehensive (20+ sources)
- Quality target: A+

### Deep Mode (`--deep`)
- Duration: 5-6 hours
- Sprints: 8 detailed + 2 bonus
- Research depth: Extensive (30+ sources)
- Quality target: A++ (exceptional)
- Additional: Performance benchmarking suite, proof-of-concept implementations

### Quick Mode (`--quick`)
- Duration: 1-2 hours
- Sprints: 4 essential (highest ROI)
- Research depth: Focused (10+ sources)
- Quality target: A
- Use case: Mid-phase check-ins, rapid competitive updates

---

## QUALITY STANDARDS

**Document Quality Grades:**

| Grade | Word Count | Sprints | Detail Level | Research Depth | MCP Usage |
|-------|-----------|---------|--------------|----------------|-----------|
| **A++** | >12,000 | 8-10 detailed | Exceptional | 5+ repos, 30+ sources, deep code review | All 4 MCP servers |
| **A+** | >10,000 | 8 detailed | Comprehensive | 4+ repos, 20+ sources, code review | 3+ MCP servers |
| **A** | 7,000-10,000 | 7-8 | Good | 3-4 repos, 15+ sources | 2+ MCP servers |
| **B** | 5,000-7,000 | 6-8 | Basic | 2-3 repos, 10+ sources | 1 MCP server |
| **C** | <5,000 | <6 | Minimal | <2 repos, <5 sources | None |

**Target:** A+ grade minimum (A++ for --deep mode)

**Sprint Planning Quality:**

| Grade | Dependencies | Estimates | Success Metrics | Tasks | Risks |
|-------|-------------|-----------|-----------------|-------|-------|
| **A+** | Mapped visually | Per-task hours | Quant + Qual | 7-12 per sprint | Documented |
| **A** | Listed clearly | Per-sprint days | Quantitative | 5-8 per sprint | Mentioned |
| **B** | Basic mention | Sprint-level | Basic | 3-5 per sprint | None |

**Target:** A+ grade

---

## EXECUTION INSTRUCTIONS (For Sub-Agent)

**CRITICAL:** This command ALWAYS executes via sub-agent. When user invokes `/inspire-me`, Claude MUST respond with:

```
I'll execute comprehensive competitive analysis and enhancement planning using a dedicated sub-agent with full MCP server access.

/sub-agent (new, run separately) ultrathink: Execute comprehensive competitive
analysis and enhancement planning for ProRT-IP following the /inspire-me workflow
(all 6 phases) --> utilize ALL available MCP servers/tools for enhanced research:

1. GitHub MCP: Repository analysis, code search, issue tracking
2. Browser MCP: Online research, documentation, benchmarks
3. Knowledge Graph MCP: Track competitive insights, build relationship maps
4. Code Search MCP: Analyze reference implementations

Generate:
- docs/XX-PHASEXX-ENHANCEMENTS.md (>10,000 words)
- /tmp/ProRT-IP/inspire-me-report.md (execution summary)
- Knowledge graph entries (competitive landscape)
- CLAUDE.local.md session entry

Follow all 6 phases systematically with A+ quality standards.
```

**Systematic Execution (All 6 Phases):**

1. ✅ **CONTEXT GATHERING** - Understand ProRT-IP's current state (15 min)
   - Read project documentation
   - Query knowledge graph for historical insights
   - Extract current metrics and features

2. ✅ **REFERENCE ANALYSIS** - Analyze code and online resources (45 min)
   - Local code_ref/ analysis
   - GitHub MCP: Deep repository dives (4+ repos)
   - Browser MCP: Community research (Reddit, HN, blogs)
   - Code Search MCP: Pattern discovery
   - Knowledge Graph: Entity creation

3. ✅ **GAP ANALYSIS** - Compare and categorize findings (45 min)
   - Feature matrix (15+ categories)
   - Performance comparison
   - Categorize: Strengths, Gaps, Opportunities
   - ROI scoring
   - Knowledge Graph: Update with findings

4. ✅ **SPRINT PLANNING** - Design 8 enhancement sprints (60 min)
   - ROI-driven prioritization
   - Detailed sprint templates (7-12 tasks each)
   - Dependency mapping
   - Timeline estimation

5. ✅ **DOCUMENT GENERATION** - Create comprehensive enhancement doc (60 min)
   - 10,000+ word document
   - All sections complete (executive summary through appendices)
   - Professional formatting
   - Statistics tracking

6. ✅ **VERIFICATION & INTEGRATION** - Validate and generate summary (45 min)
   - Completeness checklist
   - Quality verification
   - Generate execution report
   - Update CLAUDE.local.md
   - Knowledge graph final update

**Work through each phase COMPLETELY before moving to next.**

**Critical Success Factors:**
- ✅ Enhancement document >10,000 words (A+ quality)
- ✅ All 8 sprints fully detailed with tasks, estimates, metrics, risks
- ✅ Research covers 4+ competitors thoroughly (MCP-powered)
- ✅ Feature matrix includes 15+ categories
- ✅ All research sources cited with URLs/paths (20+ sources)
- ✅ Document is production-quality (proofread, formatted)
- ✅ MCP servers utilized (GitHub, Browser, Knowledge Graph, Code Search)
- ✅ Knowledge graph integration (entities, relations, persistence)

**Final Deliverables:**
- ✅ `docs/XX-PHASEXX-ENHANCEMENTS.md` (comprehensive enhancement document)
- ✅ `/tmp/ProRT-IP/inspire-me-report.md` (execution summary)
- ✅ Updated `CLAUDE.local.md` (session entry)
- ✅ Knowledge graph (competitive intelligence captured)

---

## INTEGRATION WITH PROJECT WORKFLOW

**Before Phase Transition:**
1. Complete current phase (all sprints)
2. Run `/inspire-me` for next phase enhancement planning
3. Review and approve sprint roadmap
4. Update ROADMAP.md with planned sprints
5. Begin execution with `/sprint-start X.Y1`

**During Phase Execution:**
- Track progress with PROJECT-STATUS.md
- Complete sprints with `/sprint-complete`
- Monitor competitive landscape (monthly GitHub activity check)
- Update knowledge graph with learnings

**After Phase Completion:**
- Run `/phase-complete` for final verification
- Retrospective: Capture lessons learned
- Validate competitive positioning (benchmarks)
- Archive analysis for historical reference

**Knowledge Graph Continuity:**
- Each `/inspire-me` builds on previous analysis
- Competitive intelligence accumulates over time
- Sprint learnings inform future planning
- User feedback patterns tracked

---

**EXECUTE NOW - AI-Powered Competitive Intelligence & Enhancement Planning**

🚀 **Let's discover what's next for ProRT-IP!**
