# Documentation Standards

Comprehensive documentation standards, organization guidelines, and maintenance procedures for ProRT-IP.

---

## Quick Reference

**Documentation System**: mdBook (hierarchical, folder-based)
**Legacy System**: Numbered prefixes (00-29) in `docs/` directory (being phased out)
**Naming Convention**: kebab-case (e.g., `service-detection.md`, `quick-start.md`)
**Build Tool**: mdBook 0.4+
**Theme**: Rust (with navy dark mode)
**Review Schedule**: Weekly during active development, monthly during maintenance
**Link Validation**: Automated via mdBook preprocessor + manual verification

---

## Documentation Organization

### Current System (mdBook - Hierarchical)

**Location**: `docs/src/`

**Structure**:
```
docs/src/
├── getting-started/     # New user onboarding
│   ├── installation.md
│   ├── quick-start.md
│   ├── tutorials.md
│   └── examples.md
├── user-guide/          # Basic usage and CLI reference
│   ├── basic-usage.md
│   ├── scan-types.md
│   ├── cli-reference.md
│   ├── configuration.md
│   └── output-formats.md
├── features/            # Feature-specific documentation
│   ├── service-detection.md
│   ├── os-fingerprinting.md
│   ├── ipv6.md
│   ├── stealth-scanning.md
│   ├── rate-limiting.md
│   ├── event-system.md
│   ├── plugin-system.md
│   └── database-storage.md
├── advanced/            # Performance, optimization, advanced topics
│   ├── performance-tuning.md
│   ├── tui-architecture.md
│   ├── performance-characteristics.md
│   ├── benchmarking.md
│   ├── evasion-techniques.md
│   ├── security-best-practices.md
│   └── efficiency-analysis.md
├── development/         # Developer documentation
│   ├── architecture.md
│   ├── implementation.md
│   ├── technical-specs.md
│   ├── testing.md
│   ├── testing-infrastructure.md
│   ├── fuzzing.md
│   ├── ci-cd.md
│   ├── release-process.md
│   ├── doc-standards.md (THIS FILE)
│   └── contributing.md
├── reference/           # Technical references and comparisons
│   ├── tech-spec-v2.md
│   ├── api-reference.md
│   ├── faq.md
│   ├── troubleshooting.md
│   ├── index.md
│   └── comparisons/
│       ├── overview.md
│       ├── nmap.md
│       ├── masscan.md
│       ├── zmap.md
│       ├── rustscan.md
│       └── naabu.md
├── project-management/  # Project tracking and planning
│   ├── phases.md
│   ├── sprints.md
│   └── tracking.md
├── security/            # Security documentation
│   ├── security-model.md
│   ├── vulnerability-disclosure.md
│   ├── audit-log.md
│   └── secure-configuration.md
└── appendices/          # Supplemental materials
    ├── glossary.md
    ├── references.md
    └── changelog-archive.md
```

**Principles**:
1. **Hierarchical organization**: No numbering prefixes, folders group related content
2. **Audience-based structure**: getting-started → user-guide → features → advanced → development → reference
3. **Descriptive folders**: Folder names indicate content type and target audience
4. **Nested subdirectories**: reference/comparisons/ for specialized content groupings
5. **Clear navigation**: Table of contents auto-generated from folder structure

### Legacy System (Numbered - Deprecated)

**Location**: `docs/` (root directory)

**Structure**:
```
docs/
├── 00-ARCHITECTURE.md
├── 00-DOCUMENTATION-INDEX.md
├── 01-ROADMAP.md
├── 02-TECHNICAL-SPECS.md
├── 03-DEV-SETUP.md
├── 04-IMPLEMENTATION-GUIDE.md
├── 06-TESTING.md
├── 08-SECURITY.md
├── 10-PROJECT-STATUS.md
├── 11-RELEASE-PROCESS.md
...
├── 28-CI-CD-COVERAGE.md
└── 29-FUZZING-GUIDE.md
```

**Characteristics**:
- **Numbered prefixes**: 00-29 for organization
- **CAPS-WITH-HYPHENS**: File naming convention
- **Multiple files per prefix**: 00-ARCHITECTURE.md, 00-DOCUMENTATION-INDEX.md
- **Flat structure**: All files in single directory (no subfolders)

**Migration Status**: ⚠️ Being phased out in favor of mdBook hierarchical structure. Legacy docs remain for reference until all content migrated.

---

## Naming Conventions

### File Names

**Format**: `kebab-case.md`

**Rules**:
1. **Use lowercase letters**: Never use uppercase in file names
2. **Separate words with hyphens**: `service-detection.md`, NOT `service_detection.md` or `ServiceDetection.md`
3. **Descriptive names**: File name should clearly indicate content (e.g., `quick-start.md`, `performance-tuning.md`)
4. **No numbering prefixes**: Hierarchical structure provides organization (exception: legacy docs/)
5. **Consistent terminology**: Use project vocabulary (e.g., `ipv6.md` not `ipv6-scanning.md`)

**Examples**:
```
✅ Good:
- installation.md
- quick-start.md
- service-detection.md
- performance-characteristics.md
- tui-architecture.md

❌ Bad:
- Installation.md (uppercase)
- quick_start.md (underscore)
- service_version_detection.md (verbose, use service-detection.md)
- 01-Installation.md (numbered prefix in mdBook)
- ipv6-scanning.md (redundant, use ipv6.md)
```

### Folder Names

**Format**: `kebab-case/`

**Rules**:
1. **Plural for collections**: `features/`, `comparisons/`, `appendices/`
2. **Singular for processes**: `development/`, `security/`, `project-management/`
3. **Descriptive but concise**: `getting-started/` not `new-user-onboarding/`
4. **No special characters**: Only lowercase letters and hyphens

**Examples**:
```
✅ Good:
- getting-started/
- user-guide/
- features/
- advanced/
- reference/comparisons/

❌ Bad:
- GettingStarted/ (camelCase)
- user_guide/ (underscore)
- refs/ (unclear abbreviation)
- reference-comparisons/ (flat, use nested reference/comparisons/)
```

### Section Headings

**Format**: Title Case for H1, Sentence case for H2+

**Rules**:
1. **H1 (Title)**: Title Case with Major Words Capitalized
2. **H2-H6**: Sentence case with only first word capitalized
3. **Consistent hierarchy**: Never skip heading levels (H1 → H2 → H3, not H1 → H3)
4. **Unique anchors**: Ensure heading text is unique within document for link anchors

**Examples**:
```markdown
✅ Good:
# Service Detection
## Detection methodology
### HTTP banner extraction
#### Version string parsing

❌ Bad:
# service detection (lowercase H1)
## Detection Methodology (Title Case H2)
### Version String Parsing (skipped H2)
```

---

## File Structure Standards

### Document Template

Every documentation file should follow this structure:

```markdown
# [Title]

[One-sentence description of the document's purpose]

---

## Quick Reference

[2-5 bullet points with key information for quick lookups]

---

## [Main Content Section 1]

### [Subsection 1.1]

[Content with code examples, tables, diagrams]

### [Subsection 1.2]

[Content]

---

## [Main Content Section 2]

...

---

## See Also

- [Related Document 1](../path/to/doc1.md) - Brief description
- [Related Document 2](../path/to/doc2.md) - Brief description
- [External Resource](https://example.com) - Brief description
```

### Required Sections

**All documentation files must include**:

1. **Title (H1)**: Single top-level heading
2. **One-sentence description**: Immediately after title
3. **Horizontal rule separator**: `---` after description
4. **Quick Reference section**: Key information in bullet points or table
5. **Main content sections**: Organized with H2-H6 headings
6. **See Also section**: Cross-references to related documentation
7. **Markdown file extension**: `.md`

### Optional Sections

**Include when relevant**:

1. **Prerequisites**: Required knowledge or setup before reading
2. **Examples**: Code snippets, command examples, use cases
3. **Troubleshooting**: Common issues and solutions
4. **Performance considerations**: Timing, resource usage, optimization tips
5. **Version compatibility**: Feature availability across versions
6. **External references**: Links to RFCs, specifications, research papers

---

## Content Organization Principles

### Audience Hierarchy

**Progressive disclosure**: Organize content from beginner to advanced:

```
1. getting-started/     # Beginners (first-time users)
   ↓
2. user-guide/          # Regular users (basic usage)
   ↓
3. features/            # Feature exploration (specific capabilities)
   ↓
4. advanced/            # Power users (optimization, tuning)
   ↓
5. development/         # Contributors (architecture, internals)
   ↓
6. reference/           # Experts (API docs, specifications)
```

**Each level builds on previous**:
- Don't assume advanced knowledge in getting-started/
- Reference technical details in development/ without repeating basics
- Cross-link to prerequisites when necessary

### Content Types

**Tutorials** (getting-started/):
- Step-by-step instructions
- Concrete examples with expected output
- Clear learning objectives
- Minimal theory, maximum practical guidance

**Guides** (user-guide/):
- Conceptual explanations
- Multiple approaches to common tasks
- Background context and "why"
- Comparison of options

**References** (reference/):
- Comprehensive API documentation
- Complete option listings
- Technical specifications
- Searchable, not necessarily readable sequentially

**How-To** (advanced/):
- Solution-oriented
- Assumes basic knowledge
- Focused on specific problems
- Performance and optimization tips

### Document Length Guidelines

**Target lengths** (approximate):

| Document Type | Target Length | Maximum Length | Notes |
|--------------|---------------|----------------|-------|
| **Quick Start** | 300-500 lines | 800 lines | Focus on essential path |
| **Tutorial** | 500-800 lines | 1,200 lines | Step-by-step with examples |
| **User Guide** | 800-1,500 lines | 2,500 lines | Comprehensive coverage |
| **Feature Documentation** | 600-1,000 lines | 1,800 lines | Complete feature reference |
| **Architecture** | 1,000-1,500 lines | 2,500 lines | System design details |
| **API Reference** | 1,500-3,000 lines | 5,000 lines | Complete API surface |
| **FAQ** | 400-800 lines | 1,500 lines | Question/answer pairs |
| **Troubleshooting** | 600-1,200 lines | 2,000 lines | Problem/solution catalog |

**When documents exceed maximum**:
1. Split into logical subdocuments
2. Create overview with links to details
3. Move advanced topics to separate files
4. Extract examples to examples gallery

---

## Writing Style Guidelines

### Voice and Tone

**Use**:
- **Active voice**: "The scanner sends packets" NOT "Packets are sent by the scanner"
- **Second person**: "You can configure..." NOT "Users can configure..."
- **Present tense**: "The system validates..." NOT "The system will validate..."
- **Imperative for instructions**: "Run the command" NOT "You should run the command"

**Examples**:
```markdown
✅ Good:
Run the following command to start a SYN scan:
prtip -sS -p 80,443 target.com

❌ Bad:
You should run the following command if you want to start a SYN scan:
prtip -sS -p 80,443 target.com
```

### Clarity and Conciseness

**Rules**:
1. **One idea per sentence**: Keep sentences focused and short
2. **Remove filler words**: "basically", "essentially", "obviously"
3. **Use simple words**: "use" not "utilize", "help" not "facilitate"
4. **Avoid jargon**: Define technical terms on first use
5. **Be specific**: "20 seconds" not "a short time"

**Examples**:
```markdown
✅ Good:
The scanner waits 5 seconds for responses before timing out.

❌ Bad:
Basically, the scanner will essentially wait for a relatively short period of time (approximately 5 seconds) before it determines that a timeout has occurred.
```

### Technical Accuracy

**Requirements**:
1. **Test all commands**: Verify every code example executes successfully
2. **Use correct version**: Document feature availability by version (e.g., "Available since v0.5.0")
3. **Cite sources**: Link to RFCs, research papers, external documentation
4. **Indicate limitations**: Document known issues, edge cases, unsupported scenarios
5. **Update regularly**: Review documentation with each release

---

## Code Examples Standards

### Command Examples

**Format**:
````markdown
```bash
# Description of what this command does
prtip -sS -p 80,443 target.com
```
````

**Rules**:
1. **Include comment**: Explain what the command does
2. **Show output**: Include expected output for clarity
3. **Use realistic targets**: `target.com`, `scanme.nmap.org`, `192.168.1.0/24`
4. **Highlight key flags**: Use bold or inline code for important options
5. **Test thoroughly**: Every example must execute successfully

**Examples**:
````markdown
✅ Good:
```bash
# Scan common web ports on local network
prtip -sS -p 80,443,8080 192.168.1.0/24
```

Expected output:
```
Scanning 256 hosts, 3 ports each (768 total combinations)
Progress: [========================================] 100%

192.168.1.1:80    open    HTTP 1.1
192.168.1.1:443   open    HTTPS (TLS 1.3)
192.168.1.10:8080 open    HTTP 1.1

Scan complete: 3 open ports found in 2.3 seconds
```

❌ Bad:
```bash
prtip -sS -p 80,443 192.168.1.0/24
```
(No comment, no expected output, unclear purpose)
````

### Rust Code Examples

**Format**:
````markdown
```rust
// Description of code functionality
use prtip_scanner::SynScanner;

let scanner = SynScanner::new(config)?;
scanner.scan_target("192.168.1.1").await?;
```
````

**Rules**:
1. **Imports first**: Show necessary `use` statements
2. **Error handling**: Use `?` or proper error handling, never `.unwrap()`
3. **Type annotations**: Include types when not obvious from context
4. **Comments**: Explain non-obvious code sections
5. **Compilable**: Code must compile (use `cargo test --doc`)

**Examples**:
````markdown
✅ Good:
```rust
use prtip_scanner::{SynScanner, ScanConfig};

// Configure scanner with custom timing
let config = ScanConfig {
    timeout_ms: 5000,
    max_retries: 3,
    ..Default::default()
};

let scanner = SynScanner::new(config)?;
let results = scanner.scan_ports(target, ports).await?;
```

❌ Bad:
```rust
let scanner = SynScanner::new(config).unwrap();
let results = scanner.scan_ports(target, ports).await.unwrap();
```
(No imports, uses unwrap(), no comments, unclear context)
````

### Configuration Examples

**Format**:
````markdown
```toml
# Description of configuration purpose
[scanner]
timeout_ms = 5000
max_rate = 100000
parallel_threads = 16
```
````

**Rules**:
1. **Show full section**: Include section header (`[scanner]`)
2. **Default values**: Indicate which values are defaults vs custom
3. **Units**: Specify units in comments (ms, seconds, bytes)
4. **Valid syntax**: Configuration must parse successfully
5. **Realistic values**: Use reasonable, production-ready values

---

## Cross-Reference Standards

### Internal Links

**Format**: `[Link Text](../path/to/file.md#anchor)`

**Rules**:
1. **Relative paths**: Use `../` for navigation, not absolute paths
2. **Descriptive text**: Link text should describe destination
3. **Section anchors**: Link to specific sections when relevant
4. **Verify links**: All links must resolve (validated in CI/CD)
5. **Update on moves**: Update all cross-references when moving files

**Examples**:
```markdown
✅ Good:
For installation instructions, see [Installation Guide](../getting-started/installation.md).

For TCP SYN scan details, see [Scan Types: SYN Scan](../user-guide/scan-types.md#syn-scan).

❌ Bad:
See installation guide (no link).
See [here](../getting-started/installation.md) (vague link text).
See [Installation](/docs/src/getting-started/installation.md) (absolute path).
```

### External Links

**Format**: `[Link Text](https://example.com)`

**Rules**:
1. **HTTPS preferred**: Use `https://` when available
2. **Stable URLs**: Link to permanent, versioned documentation
3. **Include description**: Explain what user will find at link
4. **Archive important links**: Use Web Archive for critical references
5. **Check regularly**: Validate external links quarterly

**Examples**:
```markdown
✅ Good:
ProRT-IP implements TCP SYN scanning as described in [RFC 793: Transmission Control Protocol](https://www.rfc-editor.org/rfc/rfc793).

For Nmap comparison, see [Nmap Project](https://nmap.org/).

❌ Bad:
See RFC 793 (no link).
See [this page](http://example.com) (HTTP, vague description).
```

### "See Also" Section

**Format**:
```markdown
## See Also

- [Document 1](path/to/doc1.md) - Brief description of content
- [Document 2](path/to/doc2.md) - Brief description of content
- [External Resource](https://example.com) - Brief description of content
```

**Rules**:
1. **Always include**: Every document must have "See Also" section
2. **3-7 links**: Enough to be useful, not overwhelming
3. **Related content**: Link to prerequisite, next steps, related topics
4. **Brief descriptions**: One sentence explaining link relevance
5. **Logical order**: Prerequisites first, next steps last

---

## mdBook Configuration

### book.toml Settings

**Current configuration** (`docs/book.toml`):

```toml
[book]
title = "ProRT-IP WarScan Documentation"
authors = ["ProRT-IP Contributors"]
description = "Modern network scanner combining Masscan speed with Nmap depth"
language = "en"
src = "src"

[build]
build-dir = "book"
create-missing = true

[preprocessor.links]
# Enable link checking

[output.html]
default-theme = "rust"
preferred-dark-theme = "navy"
git-repository-url = "https://github.com/doublegate/ProRT-IP"
edit-url-template = "https://github.com/doublegate/ProRT-IP/edit/main/{path}"

[output.html.fold]
enable = true
level = 1

[output.html.search]
enable = true
limit-results = 30
use-boolean-and = true
boost-title = 2
boost-hierarchy = 1

[output.html.code]
line-numbers = true
copyable = true
```

**Key Features**:

1. **Theme**: Rust (official Rust book theme)
   - Clean, professional appearance
   - Excellent code syntax highlighting
   - Responsive design for mobile/desktop

2. **Dark Mode**: Navy theme preferred
   - Reduces eye strain for extended reading
   - Professional appearance
   - Consistent with developer tools

3. **Link Checking**: Enabled via preprocessor
   - Validates all internal links
   - Catches broken cross-references
   - Prevents dead links in production

4. **Search**: Full-text search with Boolean AND
   - 30 results limit for performance
   - Title boost (2x) for better relevance
   - Hierarchy boost for section matching

5. **Code Features**:
   - Line numbers for reference
   - Copyable code blocks
   - Syntax highlighting via highlight.js

6. **GitHub Integration**:
   - Edit links for contributions
   - Repository URL for context
   - Automatic "Edit this page" buttons

### Building Documentation

**Local build**:
```bash
# Install mdBook (first time only)
cargo install mdbook

# Build documentation
cd docs/
mdbook build

# Serve with live reload (development)
mdbook serve --open
```

**CI/CD build** (GitHub Actions):
```bash
# Production build with optimizations
mdbook build --dest-dir ./book

# Test all code examples
mdbook test

# Validate links
mdbook build 2>&1 | grep -i "error\|warning"
```

**Output**:
- Generated HTML: `docs/book/` directory
- Deployable to GitHub Pages, Netlify, or static hosting
- Search index: `docs/book/searchindex.json`
- Assets: CSS, JavaScript, fonts in `docs/book/` subdirectories

---

## Documentation Review Schedule

### Regular Reviews

**Weekly** (during active development):
- Review new documentation for sprint features
- Update Quick Reference sections with new capabilities
- Validate code examples against latest codebase
- Fix broken links from file moves/renames

**Monthly** (during maintenance periods):
- Comprehensive documentation audit
- Update version references and compatibility notes
- Review external links for validity
- Refresh performance benchmarks and metrics

**Per Release**:
- Update all version references (e.g., "Available since v0.5.0")
- Regenerate API documentation with `cargo doc`
- Validate all code examples compile and execute
- Update CHANGELOG references in documentation

### Review Checklist

**Content Review**:
- [ ] Technical accuracy verified
- [ ] Code examples tested and working
- [ ] Version compatibility documented
- [ ] Performance claims validated with benchmarks
- [ ] Security considerations documented
- [ ] Cross-references up-to-date

**Style Review**:
- [ ] Consistent terminology used
- [ ] Active voice and present tense
- [ ] Clear, concise sentences
- [ ] Proper heading hierarchy (H1 → H2 → H3)
- [ ] Code formatting consistent
- [ ] No spelling or grammar errors

**Structure Review**:
- [ ] Quick Reference section present
- [ ] Logical section organization
- [ ] "See Also" section complete
- [ ] Examples follow standards
- [ ] Document length appropriate
- [ ] File naming follows conventions

**Link Review**:
- [ ] All internal links resolve
- [ ] External links valid (HTTPS preferred)
- [ ] Section anchors correct
- [ ] No broken cross-references
- [ ] GitHub edit links functional

---

## Link Validation Procedures

### Automated Validation

**mdBook preprocessor** (built-in):
```bash
# Build with link checking enabled
mdbook build

# Output shows broken links:
# ERROR: Broken link: ../non-existent/file.md
```

**GitHub Actions CI/CD**:
```yaml
- name: Build documentation
  run: |
    cd docs/
    mdbook build 2>&1 | tee build.log

- name: Check for broken links
  run: |
    if grep -qi "error.*broken link" docs/build.log; then
      echo "❌ Broken links detected"
      exit 1
    fi
```

### Manual Validation

**grep-based link checking**:
```bash
# Find all markdown links
grep -r "\[.*\](.*\.md" docs/src/ | grep -v "http"

# Extract relative paths and verify files exist
for link in $(grep -roh "(\.\./[^)]*\.md)" docs/src/ | sort -u); do
  file=$(echo $link | tr -d '()')
  if [ ! -f "docs/src/$file" ]; then
    echo "Broken: $file"
  fi
done
```

**Section anchor validation**:
```bash
# Find section links (#anchor)
grep -roh "\[.*\](#[^)]*)" docs/src/

# Verify anchors exist in target files
# (Manual review - anchors generated from headings)
```

### Link Update Procedures

**When moving files**:
1. **Search for all references**:
   ```bash
   grep -r "old-filename.md" docs/src/
   ```

2. **Update all cross-references**:
   - Use find-and-replace with care
   - Verify each update manually
   - Test links after changes

3. **Update SUMMARY.md**:
   - Update table of contents entry
   - Verify chapter hierarchy

4. **Rebuild and verify**:
   ```bash
   mdbook build
   # Check for broken link errors
   ```

**When renaming sections**:
1. **Identify anchor changes**:
   - Heading "Service Detection" → anchor `#service-detection`
   - Update all section links referencing old anchor

2. **Search for anchor references**:
   ```bash
   grep -r "#old-anchor" docs/src/
   ```

3. **Update and test**:
   - Update all references
   - Rebuild documentation
   - Manually verify navigation

---

## Content Update Procedures

### Adding New Features

**When documenting a new feature**:

1. **Create feature documentation**:
   - Add file to appropriate section (usually `features/`)
   - Follow document template structure
   - Include Quick Reference, examples, "See Also"

2. **Update cross-references**:
   - Add to related documents' "See Also" sections
   - Update CLI Reference if new flags added
   - Update Quick Start if core workflow affected

3. **Update navigation**:
   - Add entry to `SUMMARY.md` table of contents
   - Place in logical position within hierarchy

4. **Validate integration**:
   - Build documentation: `mdbook build`
   - Test code examples: `mdbook test`
   - Verify links: Manual review + CI/CD

5. **Update version notes**:
   - Add "Available since vX.Y.Z" note
   - Update feature comparison tables
   - Update README.md feature list

### Deprecating Features

**When deprecating a feature**:

1. **Add deprecation notice**:
   ```markdown
   > **⚠️ Deprecated**: This feature is deprecated as of v0.6.0 and will be removed in v1.0.0.
   > Use [New Feature](../features/new-feature.md) instead.
   ```

2. **Update documentation**:
   - Mark sections with deprecation warnings
   - Provide migration guide
   - Link to replacement feature

3. **Update cross-references**:
   - Remove from Quick Start guides
   - Update CLI Reference with deprecation note
   - Update comparison tables

4. **Archive after removal**:
   - Move to `appendices/deprecated-features.md`
   - Maintain for historical reference
   - Update all links to archived location

### Updating Code Examples

**When codebase changes affect examples**:

1. **Identify affected examples**:
   ```bash
   # Search for specific API usage
   grep -r "OldAPI::method" docs/src/
   ```

2. **Update examples**:
   - Modify code to use new API
   - Update comments and descriptions
   - Verify syntax highlighting still works

3. **Test examples**:
   ```bash
   # Test Rust code examples
   cargo test --doc

   # Manually test shell commands
   # (Copy-paste each command, verify output)
   ```

4. **Update output**:
   - Regenerate expected output if changed
   - Update version-specific behavior notes
   - Verify backward compatibility notes

---

## Glossary and Terminology

### Project-Specific Terms

**Use consistent terminology throughout documentation**:

| Term | Definition | Use Instead Of |
|------|------------|----------------|
| **ProRT-IP** | Project name | "the scanner", "this tool" |
| **SYN scan** | TCP SYN scanning technique | "SYN scanning", "stealth scan" (ambiguous) |
| **Service detection** | Banner grabbing + version identification | "service fingerprinting", "version detection" |
| **OS fingerprinting** | Operating system detection | "OS detection", "TCP/IP stack fingerprinting" |
| **Rate limiting** | Packet transmission throttling | "rate control", "throttling" |
| **Idle scan** | Zombie-based stealth scanning | "zombie scan", "IPID scan" |
| **TUI** | Terminal User Interface | "CLI interface" (confusing), "text UI" |
| **Event system** | Pub-sub architecture for scan events | "event bus", "message bus" |
| **Plugin system** | Lua-based extensibility | "scripting", "extensions" |

### Technical Acronyms

**Define on first use, then use acronym**:

| Acronym | Full Term | Definition |
|---------|-----------|------------|
| **TLS** | Transport Layer Security | Cryptographic protocol for secure communications |
| **SNI** | Server Name Indication | TLS extension for virtual hosting |
| **PCAPNG** | Packet Capture Next Generation | Modern packet capture file format |
| **NUMA** | Non-Uniform Memory Access | Multi-processor memory architecture |
| **BPF** | Berkeley Packet Filter | Packet filtering mechanism |
| **ICMP** | Internet Control Message Protocol | Network diagnostic protocol |
| **TTL** | Time To Live | Packet hop limit field |
| **MTU** | Maximum Transmission Unit | Maximum packet size |

**Example usage**:
```markdown
ProRT-IP uses Server Name Indication (SNI) to extract TLS certificates from virtual hosts. The SNI extension allows multiple HTTPS sites to share a single IP address.
```

---

## Quality Metrics

### Documentation Coverage

**Target metrics**:

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| **Feature documentation** | 100% of public features | ~95% | 🟢 Good |
| **API documentation** | 100% of public APIs | ~90% | 🟡 Needs improvement |
| **Code example coverage** | ≥3 examples per major feature | ~85% | 🟡 Needs improvement |
| **Tutorial coverage** | 1 tutorial per user journey | 100% | 🟢 Good |
| **External links validity** | ≥95% valid links | ~98% | 🟢 Excellent |
| **Internal links validity** | 100% valid links | 100% | 🟢 Excellent |

### Documentation Quality Indicators

**Positive indicators**:
- Documentation referenced in GitHub issues
- Low rate of "documentation unclear" issues
- High documentation search usage (analytics)
- Quick user onboarding (time to first successful scan)
- Positive community feedback

**Negative indicators**:
- Frequent documentation-related issues
- High rate of "how do I..." questions
- Low documentation page views (analytics)
- Users defaulting to reading source code
- Outdated examples or screenshots

---

## Future Improvements

### Planned Enhancements

**Short-term** (Phase 7, Q1-Q2 2026):
1. **Interactive tutorials**: Web-based interactive examples with live output
2. **Video guides**: Screencast tutorials for complex workflows
3. **Expanded examples gallery**: 100+ production-ready examples
4. **Multilingual documentation**: Spanish, Chinese, Japanese translations

**Medium-term** (Phase 8, Q3-Q4 2026):
1. **API playground**: Interactive API documentation with try-it-now functionality
2. **Architecture diagrams**: Interactive SVG diagrams with tooltips
3. **Performance calculator**: Interactive tool for estimating scan times
4. **Documentation analytics**: Track most-viewed pages, search queries

**Long-term** (Post-v1.0, 2027+):
1. **Community contributions**: User-submitted tutorials and guides
2. **Version-specific docs**: Separate documentation for each major version
3. **Integration guides**: Third-party tool integration examples
4. **Advanced search**: AI-powered semantic search

---

## See Also

- [Architecture](architecture.md) - System design and component structure
- [Implementation](implementation.md) - Code organization and design patterns
- [CI/CD](ci-cd.md) - Automated testing and deployment
- [Release Process](release-process.md) - Versioning and distribution
- [Contributing](contributing.md) - Contribution guidelines and code of conduct
- [mdBook Documentation](https://rust-lang.github.io/mdBook/) - Official mdBook guide
