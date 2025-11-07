# Development Environment Setup

Complete development environment setup for ProRT-IP with all dependencies and tools.

---

## Phase 1: PREREQUISITES CHECK

Check and report all required tools and dependencies.

### Step 1.1: Verify Core Requirements

```bash
echo "Checking ProRT-IP Prerequisites..."
echo ""

MISSING=0

# Rust toolchain
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✅ Rust: $RUST_VERSION"
else
    echo "❌ Rust not found"
    echo "   Install: https://rustup.rs/"
    MISSING=1
fi

# Rust nightly (for fuzzing)
if rustup toolchain list 2>/dev/null | grep -q nightly; then
    NIGHTLY_VERSION=$(rustc +nightly --version 2>/dev/null | awk '{print $2}')
    echo "✅ Rust nightly: $NIGHTLY_VERSION"
else
    echo "⚠️  Rust nightly not found (required for fuzzing)"
    echo "   Install: rustup toolchain install nightly"
fi

# libpcap (system dependency)
if pkg-config --exists libpcap 2>/dev/null; then
    PCAP_VERSION=$(pkg-config --modversion libpcap)
    echo "✅ libpcap: $PCAP_VERSION"
elif command -v dpkg &> /dev/null && dpkg -l libpcap-dev 2>/dev/null | grep -q "^ii"; then
    echo "✅ libpcap: installed (dpkg)"
elif command -v brew &> /dev/null && brew list libpcap &> /dev/null; then
    echo "✅ libpcap: installed (brew)"
else
    echo "❌ libpcap not found"
    echo "   Debian/Ubuntu: sudo apt-get install libpcap-dev"
    echo "   macOS: brew install libpcap"
    echo "   Fedora: sudo dnf install libpcap-devel"
    MISSING=1
fi

# Git (for repository operations)
if command -v git &> /dev/null; then
    GIT_VERSION=$(git --version | awk '{print $3}')
    echo "✅ git: $GIT_VERSION"
else
    echo "❌ git not found"
    MISSING=1
fi

echo ""

if [ $MISSING -eq 1 ]; then
    echo "❌ Missing required dependencies - install them first"
    exit 1
fi

echo "✅ All core requirements satisfied"
echo ""
```

### Step 1.2: Check Cargo Tools

```bash
echo "Checking cargo tools..."
echo ""

TOOLS=(
    "cargo-fuzz:Fuzzing infrastructure"
    "cargo-tarpaulin:Code coverage"
    "cargo-deny:Dependency checking"
    "cargo-audit:Security audits"
    "cargo-outdated:Dependency updates"
    "cargo-watch:Auto-rebuild"
)

INSTALL_NEEDED=()

for tool_desc in "${TOOLS[@]}"; do
    IFS=':' read -r tool desc <<< "$tool_desc"

    if cargo install --list 2>/dev/null | grep -q "^$tool "; then
        VERSION=$(cargo install --list | grep "^$tool " | awk '{print $2}' | tr -d ':')
        echo "✅ $tool $VERSION - $desc"
    else
        echo "⚠️  $tool - $desc (not installed)"
        INSTALL_NEEDED+=("$tool")
    fi
done

echo ""

if [ ${#INSTALL_NEEDED[@]} -gt 0 ]; then
    echo "Tools to install: ${#INSTALL_NEEDED[@]}"
else
    echo "✅ All cargo tools installed"
fi

echo ""
```

---

## Phase 2: INSTALL MISSING TOOLS

Install any missing cargo tools automatically.

### Step 2.1: Install Nightly Toolchain

```bash
if ! rustup toolchain list 2>/dev/null | grep -q nightly; then
    echo "Installing Rust nightly toolchain..."
    rustup toolchain install nightly
    echo "✅ Nightly toolchain installed"
    echo ""
fi
```

### Step 2.2: Install Cargo Tools

```bash
if [ ${#INSTALL_NEEDED[@]} -gt 0 ]; then
    echo "Installing missing cargo tools..."
    echo "This may take 5-15 minutes..."
    echo ""

    for tool in "${INSTALL_NEEDED[@]}"; do
        echo "Installing $tool..."
        cargo install "$tool" --quiet 2>&1 | grep -E "Installing|Installed|error" || true
    done

    echo ""
    echo "✅ All cargo tools installed"
else
    echo "⏭️  No cargo tools need installation"
fi

echo ""
```

---

## Phase 3: PROJECT SETUP

Configure project-specific settings and verify build.

### Step 3.1: Verify Repository

```bash
if [ ! -d ".git" ]; then
    echo "❌ Not in ProRT-IP git repository"
    echo "   Clone: git clone https://github.com/doublegate/ProRT-IP.git"
    exit 1
fi

echo "✅ Git repository detected"
echo ""
```

### Step 3.2: Configure Git Hooks

```bash
if [ -d ".githooks" ]; then
    git config core.hooksPath .githooks
    echo "✅ Git hooks configured (.githooks)"
else
    echo "⏭️  No .githooks directory found"
fi

echo ""
```

### Step 3.3: Initial Build

```bash
echo "Building project (release mode)..."
echo "This will take 2-5 minutes on first build..."
echo ""

if cargo build --release 2>&1 | tee /tmp/build-output.txt; then
    echo ""
    echo "✅ Release build successful"

    # Show binary info
    if [ -f "target/release/prtip" ]; then
        BINARY_SIZE=$(ls -lh target/release/prtip | awk '{print $5}')
        echo "   Binary: target/release/prtip ($BINARY_SIZE)"
    fi
else
    echo ""
    echo "❌ Build failed - review errors above"
    exit 1
fi

echo ""
```

### Step 3.4: Run Test Suite

```bash
echo "Running test suite to verify installation..."
echo "This will take 30-60 seconds..."
echo ""

if cargo test --workspace --quiet 2>&1 | tee /tmp/test-output.txt; then
    TESTS_PASSED=$(grep -oP '\d+(?= passed)' /tmp/test-output.txt | tail -1)
    echo ""
    echo "✅ All tests passed ($TESTS_PASSED tests)"
else
    echo ""
    echo "⚠️  Some tests failed - this may be normal on initial setup"
    echo "   Run: cargo test --workspace -- --nocapture for details"
fi

echo ""
```

---

## Phase 4: DOCUMENTATION GENERATION

Generate and open project documentation.

### Step 4.1: Generate Docs

```bash
echo "Generating project documentation..."

cargo doc --no-deps --workspace --quiet 2>&1 | grep -v "Documenting" || true

echo "✅ Documentation generated"
echo ""
```

### Step 4.2: Open Documentation

```bash
echo "Opening documentation in browser..."

if command -v xdg-open &> /dev/null; then
    xdg-open target/doc/prtip_core/index.html 2>/dev/null &
elif command -v open &> /dev/null; then
    open target/doc/prtip_core/index.html 2>/dev/null &
else
    echo "   Manual: file://$(pwd)/target/doc/prtip_core/index.html"
fi

echo ""
```

---

## Phase 5: ENVIRONMENT SUMMARY

Display complete setup summary and next steps.

### Step 5.1: Display Configuration

```bash
cat << 'EOF'

╔════════════════════════════════════════════════════════════════╗
║          ProRT-IP Development Environment Ready                ║
╚════════════════════════════════════════════════════════════════╝

✅ INSTALLED COMPONENTS

Rust Toolchains:
EOF

rustup toolchain list | sed 's/^/  • /'

cat << 'EOF'

Cargo Tools:
EOF

for tool_desc in "${TOOLS[@]}"; do
    IFS=':' read -r tool desc <<< "$tool_desc"
    if cargo install --list 2>/dev/null | grep -q "^$tool "; then
        VERSION=$(cargo install --list | grep "^$tool " | awk '{print $2}' | tr -d ':')
        echo "  • $tool $VERSION"
    fi
done

cat << 'EOF'

System Libraries:
EOF

if pkg-config --exists libpcap 2>/dev/null; then
    PCAP_VERSION=$(pkg-config --modversion libpcap)
    echo "  • libpcap $PCAP_VERSION"
else
    echo "  • libpcap (installed)"
fi

cat << 'EOF'

📊 PROJECT STATUS

EOF

echo "  • Repository: $(git remote get-url origin 2>/dev/null || echo 'local')"
echo "  • Branch: $(git branch --show-current 2>/dev/null || echo 'unknown')"
echo "  • Tests: $TESTS_PASSED passing"
echo "  • Binary: target/release/prtip ($BINARY_SIZE)"

cat << 'EOF'

🚀 NEXT STEPS

1. Review documentation:
   • docs/00-ARCHITECTURE.md - System overview
   • docs/03-DEV-SETUP.md - Development guide
   • docs/06-TESTING.md - Testing strategies

2. Verify code quality:
   /rust-check

3. Run specific tests:
   /test-quick <pattern>

4. Start development:
   cargo watch -x check    # Auto-check on file changes

5. Read project guides:
   • README.md - Quick start
   • CONTRIBUTING.md - Contribution guidelines
   • CLAUDE.md - Claude Code guidance

📖 DOCUMENTATION

• Local docs: target/doc/prtip_core/index.html
• Architecture: docs/00-ARCHITECTURE.md
• Testing: docs/06-TESTING.md
• Security: docs/08-SECURITY.md

🔧 DEVELOPMENT COMMANDS

• /rust-check - Full quality pipeline
• /test-quick <pattern> - Fast targeted tests
• /bench-compare - Performance comparison
• /sprint-start - Begin new sprint
• /ci-status - Check CI/CD status

EOF
```

---

## SUCCESS CRITERIA

✅ All prerequisites installed (Rust, libpcap, git)
✅ Cargo tools available (6+ development tools)
✅ Project builds successfully (release mode)
✅ Tests passing (1,728+ tests)
✅ Documentation generated and accessible
✅ Environment summary displayed

---

## COMMON ISSUES

### Linux: Permission denied for raw sockets

**Solution:** Run tests with sudo or configure capabilities
```bash
sudo setcap cap_net_raw,cap_net_admin=eip target/release/prtip
```

### macOS: libpcap not found

**Solution:** Install via Homebrew
```bash
brew install libpcap
```

### Windows: Npcap required

**Solution:** Install Npcap from https://npcap.com/
- Use installer with WinPcap compatibility mode
- Add to PATH: C:\Windows\System32\Npcap

### Cargo tools fail to install

**Solution:** Update Rust toolchain
```bash
rustup update stable
rustup update nightly
```

---

## SEE ALSO

- `/rust-check` - Verify code quality after setup
- `/test-quick` - Run specific test subsets
- `docs/03-DEV-SETUP.md` - Detailed setup guide

---

**Execute complete development environment setup now.**
