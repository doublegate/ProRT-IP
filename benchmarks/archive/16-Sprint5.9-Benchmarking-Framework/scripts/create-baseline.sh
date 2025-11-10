#!/usr/bin/env bash
#
# Create Performance Baseline
# Runs full benchmark suite and saves as version-tagged baseline
#
# Usage:
#   ./create-baseline.sh <version>
#
# Example:
#   ./create-baseline.sh v0.5.1
#

set -euo pipefail

# Validate argument
if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <version>"
    echo ""
    echo "Example:"
    echo "  $0 v0.5.1"
    exit 1
fi

VERSION="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
BASELINES_DIR="${PROJECT_ROOT}/benchmarks/baselines"
RESULTS_DIR="${SCRIPT_DIR}/../results"
DATE=$(date +%Y%m%d-%H%M%S)
BASELINE_DIR="${BASELINES_DIR}/${VERSION}"

# Validate version format
if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Invalid version format: $VERSION"
    echo "Expected format: v0.5.1"
    exit 1
fi

# Check if baseline already exists
if [[ -d "$BASELINE_DIR" ]]; then
    echo "Warning: Baseline for $VERSION already exists at $BASELINE_DIR"
    read -p "Overwrite? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 0
    fi
    rm -rf "$BASELINE_DIR"
fi

# Ensure directories exist
mkdir -p "$BASELINE_DIR"
mkdir -p "$RESULTS_DIR"

echo "============================================="
echo "Creating Performance Baseline"
echo "============================================="
echo "Version:      $VERSION"
echo "Baseline Dir: $BASELINE_DIR"
echo "Date:         $(date)"
echo ""

# Validate binary exists
BINARY="${PROJECT_ROOT}/target/release/prtip"
if [[ ! -f "${BINARY}" ]]; then
    echo "Error: ProRT-IP binary not found at ${BINARY}"
    echo "Run: cargo build --release"
    exit 1
fi

# Get binary info
BINARY_VERSION=$("${BINARY}" --version 2>/dev/null | head -n 1 || echo "unknown")
echo "Binary:       ${BINARY}"
echo "Binary Info:  ${BINARY_VERSION}"
echo ""

# Get system info
OS=$(uname -s)
OS_VERSION=$(uname -r)
ARCH=$(uname -m)
CPU_COUNT=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "unknown")
RAM_GB=$(free -g 2>/dev/null | awk '/^Mem:/{print $2}' || sysctl -n hw.memsize 2>/dev/null | awk '{print int($1/1024/1024/1024)}' || echo "unknown")

echo "System Info:"
echo "  OS:       $OS $OS_VERSION"
echo "  Arch:     $ARCH"
echo "  CPU:      $CPU_COUNT cores"
echo "  RAM:      ${RAM_GB}GB"
echo ""

# Get git info
GIT_COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
GIT_DIRTY=$(git diff --quiet 2>/dev/null || echo "dirty")

echo "Git Info:"
echo "  Commit:   $GIT_COMMIT"
echo "  Branch:   $GIT_BRANCH"
echo "  Status:   ${GIT_DIRTY:-clean}"
echo ""

# Run full benchmark suite
echo "Running full benchmark suite..."
echo "This may take several minutes..."
echo ""

cd "$SCRIPT_DIR"
chmod +x run-all-benchmarks.sh
./run-all-benchmarks.sh

# Copy results to baseline directory
echo ""
echo "Copying results to baseline directory..."
cp "${RESULTS_DIR}"/*.json "$BASELINE_DIR/"
cp "${RESULTS_DIR}"/*.md "$BASELINE_DIR/" 2>/dev/null || true

# Generate metadata file
METADATA_FILE="${BASELINE_DIR}/baseline-metadata.md"

cat > "$METADATA_FILE" << EOF
# Baseline Metadata: ${VERSION}

**Created:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")

## Version Information

- **ProRT-IP Version:** ${VERSION}
- **Binary:** ${BINARY_VERSION}

## System Information

- **Operating System:** ${OS} ${OS_VERSION}
- **Architecture:** ${ARCH}
- **CPU Cores:** ${CPU_COUNT}
- **RAM:** ${RAM_GB}GB

## Git Information

- **Commit:** ${GIT_COMMIT}
- **Branch:** ${GIT_BRANCH}
- **Status:** ${GIT_DIRTY:-clean}

## Benchmark Results

$(ls -1 "$BASELINE_DIR"/*.json | wc -l) benchmark scenarios captured.

### Scenarios

EOF

# List all benchmark scenarios
for json_file in "$BASELINE_DIR"/*.json; do
    if [[ -f "$json_file" ]]; then
        filename=$(basename "$json_file")
        scenario_name=$(echo "$filename" | sed 's/-[0-9]\{8\}-[0-9]\{6\}\.json$//' | sed 's/\.json$//')
        mean=$(jq -r '.results[0].mean' "$json_file" 2>/dev/null || echo "N/A")
        mean_ms=$(awk "BEGIN {printf \"%.2f\", $mean * 1000}" 2>/dev/null || echo "N/A")

        echo "- **${scenario_name}:** ${mean_ms}ms" >> "$METADATA_FILE"
    fi
done

cat >> "$METADATA_FILE" << EOF

## Usage

This baseline can be used for regression detection:

\`\`\`bash
# Compare current results against this baseline
./scripts/analyze-results.sh benchmarks/baselines/${VERSION} results/
\`\`\`

## Notes

- Baselines should be created on consistent hardware
- Ensure system is idle during baseline creation
- Results may vary across different systems
- Update baseline after significant optimizations

EOF

echo ""
echo "============================================="
echo "Baseline Created Successfully"
echo "============================================="
echo "Location:     $BASELINE_DIR"
echo "Metadata:     $METADATA_FILE"
echo "Scenarios:    $(ls -1 "$BASELINE_DIR"/*.json | wc -l)"
echo ""
echo "Next Steps:"
echo "  1. Review metadata: cat $METADATA_FILE"
echo "  2. Commit baseline: git add benchmarks/baselines/ && git commit -m \"chore: Add ${VERSION} performance baseline\""
echo "  3. Use for regression detection: ./scripts/analyze-results.sh $BASELINE_DIR results/"
echo ""
