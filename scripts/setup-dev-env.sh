#!/usr/bin/env bash
#
# Script Name: setup-dev-env.sh
# Purpose: One-command development environment setup for ProRT-IP
# Version: 1.0.0
# Usage: ./setup-dev-env.sh [options]
# Prerequisites: None (script installs everything)
# Exit Codes:
#   0 - Success
#   1 - General error
#   2 - Unsupported platform
#
# Examples:
#   ./setup-dev-env.sh                 # Full setup
#   ./setup-dev-env.sh --quick         # Skip optional tools
#   ./setup-dev-env.sh --no-hooks      # Skip git hooks setup
#

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Options
QUICK_MODE=false
INSTALL_HOOKS=true
VERBOSE=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Minimum versions
MIN_RUST_VERSION="1.85"
MIN_LIBPCAP_VERSION="1.9"

# Helper functions
usage() {
    cat <<EOF
${GREEN}ProRT-IP Development Environment Setup v1.0.0${NC}

${BLUE}Usage:${NC}
  $(basename "$0") [options]

${BLUE}Options:${NC}
  --help              Show this help message
  --quick             Skip optional tools (benchmarks, profiling)
  --no-hooks          Skip git hooks setup
  --verbose           Enable verbose output

${BLUE}What This Script Does:${NC}
  1. Detect platform (Linux/macOS/FreeBSD/Windows WSL)
  2. Install Rust toolchain (${MIN_RUST_VERSION}+)
  3. Install system dependencies (libpcap, OpenSSL, pkg-config)
  4. Install development tools (clippy, rustfmt, cargo-audit)
  5. Install optional tools (hyperfine, perf, valgrind, etc.)
  6. Setup git hooks (pre-commit formatting/linting)
  7. Run smoke test to verify setup

${BLUE}Supported Platforms:${NC}
  - Linux (Debian/Ubuntu, Fedora/RHEL, Arch)
  - macOS (Homebrew required)
  - FreeBSD
  - Windows WSL2

${BLUE}Examples:${NC}
  # Full setup with all tools
  ./setup-dev-env.sh

  # Quick setup (skip benchmarking/profiling tools)
  ./setup-dev-env.sh --quick

  # Setup without git hooks
  ./setup-dev-env.sh --no-hooks

${YELLOW}Note:${NC} This script may require sudo privileges for system packages.
You will be prompted when needed.
EOF
    exit 0
}

error() {
    echo -e "${RED}ERROR: $*${NC}" >&2
    exit 1
}

warn() {
    echo -e "${YELLOW}WARN: $*${NC}"
}

info() {
    echo -e "${BLUE}INFO: $*${NC}"
}

success() {
    echo -e "${GREEN}✓ $*${NC}"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            usage
            ;;
        --quick|-q)
            QUICK_MODE=true
            shift
            ;;
        --no-hooks)
            INSTALL_HOOKS=false
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        *)
            error "Unknown option: $1. Use --help for usage."
            ;;
    esac
done

# Platform detection
detect_platform() {
    info "Detecting platform..."

    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        PLATFORM="linux"

        # Detect Linux distribution
        if [[ -f /etc/os-release ]]; then
            source /etc/os-release
            case "$ID" in
                ubuntu|debian)
                    DISTRO="debian"
                    PKG_MANAGER="apt-get"
                    ;;
                fedora|rhel|centos)
                    DISTRO="fedora"
                    PKG_MANAGER="dnf"
                    ;;
                arch|manjaro)
                    DISTRO="arch"
                    PKG_MANAGER="pacman"
                    ;;
                *)
                    warn "Unknown Linux distribution: $ID"
                    DISTRO="unknown"
                    ;;
            esac
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        PLATFORM="macos"
        PKG_MANAGER="brew"
    elif [[ "$OSTYPE" == "freebsd"* ]]; then
        PLATFORM="freebsd"
        PKG_MANAGER="pkg"
    else
        error "Unsupported platform: $OSTYPE"
    fi

    success "Platform detected: $PLATFORM${DISTRO:+ ($DISTRO)}"
}

# Check if command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Install Rust toolchain
install_rust() {
    info "Checking Rust installation..."

    if command_exists rustc && command_exists cargo; then
        local rust_version
        rust_version=$(rustc --version | awk '{print $2}')
        success "Rust already installed: $rust_version"

        # Check if version is sufficient
        if [[ "$(printf '%s\n' "$MIN_RUST_VERSION" "$rust_version" | sort -V | head -n1)" != "$MIN_RUST_VERSION" ]]; then
            warn "Rust version $rust_version is below minimum $MIN_RUST_VERSION"
            info "Updating Rust..."
            rustup update stable
            success "Rust updated"
        fi
    else
        info "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

        # Source cargo env
        source "$HOME/.cargo/env"
        success "Rust installed: $(rustc --version)"
    fi
}

# Install development tools
install_dev_tools() {
    info "Installing Rust development tools..."

    local tools=(
        "clippy"
        "rustfmt"
    )

    for tool in "${tools[@]}"; do
        if rustup component list --installed | grep -q "^${tool}"; then
            success "$tool already installed"
        else
            info "Installing $tool..."
            rustup component add "$tool"
            success "$tool installed"
        fi
    done

    # Install cargo-audit
    if command_exists cargo-audit; then
        success "cargo-audit already installed"
    else
        info "Installing cargo-audit..."
        cargo install cargo-audit --quiet
        success "cargo-audit installed"
    fi
}

# Install system dependencies
install_system_deps() {
    info "Installing system dependencies..."

    case "$PLATFORM" in
        linux)
            case "$DISTRO" in
                debian)
                    sudo apt-get update
                    sudo apt-get install -y \
                        libpcap-dev \
                        libssl-dev \
                        pkg-config \
                        build-essential
                    ;;
                fedora)
                    sudo dnf install -y \
                        libpcap-devel \
                        openssl-devel \
                        pkg-config \
                        gcc
                    ;;
                arch)
                    sudo pacman -S --noconfirm \
                        libpcap \
                        openssl \
                        pkg-config \
                        base-devel
                    ;;
                *)
                    warn "Unknown distro, skipping system packages"
                    return
                    ;;
            esac
            ;;
        macos)
            if ! command_exists brew; then
                error "Homebrew not found. Install from: https://brew.sh"
            fi

            # Check and install only if not present
            for pkg in libpcap openssl pkg-config; do
                if brew list "$pkg" &>/dev/null; then
                    success "$pkg already installed"
                else
                    info "Installing $pkg..."
                    brew install "$pkg"
                fi
            done
            ;;
        freebsd)
            sudo pkg install -y \
                libpcap \
                openssl \
                pkgconf
            ;;
    esac

    success "System dependencies installed"
}

# Install optional tools (benchmarking, profiling)
install_optional_tools() {
    if [[ "$QUICK_MODE" == true ]]; then
        info "Skipping optional tools (--quick mode)"
        return
    fi

    info "Installing optional tools..."

    # hyperfine (benchmarking)
    if command_exists hyperfine; then
        success "hyperfine already installed"
    else
        info "Installing hyperfine..."
        cargo install hyperfine --quiet
        success "hyperfine installed"
    fi

    # Platform-specific profiling tools
    case "$PLATFORM" in
        linux)
            # perf (Linux only)
            if command_exists perf; then
                success "perf already installed"
            else
                case "$DISTRO" in
                    debian)
                        sudo apt-get install -y linux-tools-generic
                        ;;
                    fedora)
                        sudo dnf install -y perf
                        ;;
                    arch)
                        sudo pacman -S --noconfirm perf
                        ;;
                esac
                success "perf installed"
            fi

            # valgrind
            if command_exists valgrind; then
                success "valgrind already installed"
            else
                case "$DISTRO" in
                    debian)
                        sudo apt-get install -y valgrind
                        ;;
                    fedora)
                        sudo dnf install -y valgrind
                        ;;
                    arch)
                        sudo pacman -S --noconfirm valgrind
                        ;;
                esac
                success "valgrind installed"
            fi
            ;;
        macos)
            # valgrind not officially supported on macOS
            warn "valgrind not available on macOS"
            ;;
    esac
}

# Setup git hooks
setup_git_hooks() {
    if [[ "$INSTALL_HOOKS" == false ]]; then
        info "Skipping git hooks (--no-hooks specified)"
        return
    fi

    info "Setting up git hooks..."

    local hooks_dir="$PROJECT_ROOT/.git/hooks"
    mkdir -p "$hooks_dir"

    # Pre-commit hook: format and lint
    cat > "$hooks_dir/pre-commit" <<'EOF'
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
echo "Checking formatting..."
cargo fmt --all --check || {
    echo "Formatting issues found. Run: cargo fmt --all"
    exit 1
}

# Clippy check
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings || {
    echo "Clippy warnings found. Fix before committing."
    exit 1
}

echo "Pre-commit checks passed!"
EOF

    chmod +x "$hooks_dir/pre-commit"
    success "Git hooks installed"
}

# Run smoke test
run_smoke_test() {
    info "Running smoke test..."

    cd "$PROJECT_ROOT"

    # Build in debug mode
    info "Building project (debug)..."
    if cargo build --quiet 2>&1 | grep -i error; then
        error "Build failed"
    fi
    success "Build successful"

    # Run quick test
    info "Running tests..."
    if cargo test --lib --quiet 2>&1 | tail -1 | grep -q "test result: ok"; then
        success "Tests passed"
    else
        error "Tests failed"
    fi

    success "Smoke test complete"
}

# Print summary
print_summary() {
    echo ""
    echo "=========================================="
    echo -e "${GREEN}Development Environment Setup Complete!${NC}"
    echo "=========================================="
    echo ""
    echo "Installed:"
    echo "  - Rust: $(rustc --version | awk '{print $2}')"
    echo "  - Cargo: $(cargo --version | awk '{print $2}')"
    echo "  - Clippy: $(rustup component list --installed | grep clippy || echo "installed")"
    echo "  - Rustfmt: $(rustup component list --installed | grep rustfmt || echo "installed")"
    echo "  - Cargo-audit: $(command_exists cargo-audit && echo "installed" || echo "skipped")"

    if [[ "$QUICK_MODE" == false ]]; then
        echo "  - Hyperfine: $(command_exists hyperfine && echo "installed" || echo "skipped")"
        [[ "$PLATFORM" == "linux" ]] && echo "  - Perf: $(command_exists perf && echo "installed" || echo "skipped")"
        echo "  - Valgrind: $(command_exists valgrind && echo "installed" || echo "skipped")"
    fi

    echo ""
    echo "Next Steps:"
    echo "  1. Build release binary:"
    echo "     ${BLUE}cd $PROJECT_ROOT && cargo build --release${NC}"
    echo ""
    echo "  2. Run tests:"
    echo "     ${BLUE}cargo test${NC}"
    echo ""
    echo "  3. Run benchmarks:"
    echo "     ${BLUE}./scripts/run-benchmarks.sh${NC}"
    echo ""
    echo "  4. Check documentation:"
    echo "     ${BLUE}cargo doc --open${NC}"
    echo ""
    echo "Happy hacking! 🚀"
}

# Main execution
main() {
    echo "=========================================="
    echo "ProRT-IP Development Environment Setup"
    echo "=========================================="
    echo ""

    detect_platform
    install_rust
    install_dev_tools
    install_system_deps
    install_optional_tools
    setup_git_hooks
    run_smoke_test
    print_summary
}

main "$@"
