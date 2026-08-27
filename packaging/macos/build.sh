#!/bin/bash
# ProRT-IP macOS Package Build Script
# Creates a .pkg installer for macOS

set -e

VERSION="${1:-1.0.0}"
SOURCE_DIR="${2:-../../target/release}"
OUTPUT_DIR="${3:-./output}"
IDENTIFIER="com.prorti.prtip"

echo "Building ProRT-IP macOS Package v$VERSION"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check for binary
if [ ! -f "$SOURCE_DIR/prtip" ]; then
    echo "Error: prtip binary not found at $SOURCE_DIR"
    echo "Run 'cargo build --release' first."
    exit 1
fi

# Create package structure
PACKAGE_ROOT="$OUTPUT_DIR/package-root"
rm -rf "$PACKAGE_ROOT"
mkdir -p "$PACKAGE_ROOT/usr/local/bin"
mkdir -p "$PACKAGE_ROOT/usr/local/share/man/man1"
mkdir -p "$PACKAGE_ROOT/usr/local/share/doc/prtip"

# Copy files
cp "$SOURCE_DIR/prtip" "$PACKAGE_ROOT/usr/local/bin/"
chmod 755 "$PACKAGE_ROOT/usr/local/bin/prtip"

# Copy documentation
cp "../../README.md" "$PACKAGE_ROOT/usr/local/share/doc/prtip/"
cp "../../LICENSE" "$PACKAGE_ROOT/usr/local/share/doc/prtip/"
cp "../../CHANGELOG.md" "$PACKAGE_ROOT/usr/local/share/doc/prtip/"

# Copy man page if exists
if [ -f "../../man/prtip.1" ]; then
    cp "../../man/prtip.1" "$PACKAGE_ROOT/usr/local/share/man/man1/"
fi

# Create component package
COMPONENT_PKG="$OUTPUT_DIR/prtip-component.pkg"
pkgbuild \
    --root "$PACKAGE_ROOT" \
    --identifier "$IDENTIFIER" \
    --version "$VERSION" \
    --install-location "/" \
    "$COMPONENT_PKG"

# Create distribution.xml
cat > "$OUTPUT_DIR/distribution.xml" << EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>ProRT-IP WarScan</title>
    <organization>com.prorti</organization>
    <domains enable_localSystem="true" enable_anywhere="false"/>
    <options customize="never" require-scripts="false" hostArchitectures="x86_64,arm64"/>

    <welcome file="welcome.html"/>
    <license file="LICENSE"/>
    <conclusion file="conclusion.html"/>

    <choices-outline>
        <line choice="default">
            <line choice="prtip"/>
        </line>
    </choices-outline>

    <choice id="default"/>
    <choice id="prtip" visible="false">
        <pkg-ref id="$IDENTIFIER"/>
    </choice>

    <pkg-ref id="$IDENTIFIER"
        version="$VERSION"
        onConclusion="none">prtip-component.pkg</pkg-ref>
</installer-gui-script>
EOF

# Create HTML files for installer
cat > "$OUTPUT_DIR/welcome.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; padding: 20px; }
        h1 { color: #333; }
        .highlight { background: #f0f0f0; padding: 10px; border-radius: 5px; }
    </style>
</head>
<body>
    <h1>ProRT-IP WarScan v$VERSION</h1>
    <p>Modern network scanner combining Masscan speed with Nmap detection depth.</p>

    <h2>Features</h2>
    <ul>
        <li>8 scan types (SYN, Connect, UDP, FIN, NULL, Xmas, ACK, Idle)</li>
        <li>Service detection (37-probe ProRT-IP corpus, 226 match rules)</li>
        <li>OS fingerprinting (caller-supplied signature set)</li>
        <li>TUI dashboard with real-time visualization</li>
        <li>Full IPv6 support</li>
    </ul>

    <h2>Requirements</h2>
    <ul>
        <li>macOS 11.0 (Big Sur) or later</li>
        <li>Administrator access for raw packet scanning</li>
    </ul>

    <div class="highlight">
        <strong>Note:</strong> After installation, you may need to run with <code>sudo</code>
        or configure ChmodBPF for raw packet access.
    </div>
</body>
</html>
EOF

cat > "$OUTPUT_DIR/conclusion.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; padding: 20px; }
        h1 { color: #333; }
        code { background: #f0f0f0; padding: 2px 6px; border-radius: 3px; }
        .command { background: #1e1e1e; color: #f0f0f0; padding: 10px; border-radius: 5px; font-family: monospace; }
    </style>
</head>
<body>
    <h1>Installation Complete!</h1>
    <p>ProRT-IP WarScan has been installed to <code>/usr/local/bin/prtip</code></p>

    <h2>Quick Start</h2>
    <div class="command">
        # Verify installation<br>
        prtip --version<br><br>
        # Run a quick scan<br>
        sudo prtip -F 192.168.1.1<br><br>
        # TCP Connect scan (no sudo required)<br>
        prtip -sT -p 80,443 example.com
    </div>

    <h2>Documentation</h2>
    <p>
        <a href="https://github.com/doublegate/ProRT-IP">GitHub Repository</a> |
        <a href="https://github.com/doublegate/ProRT-IP/tree/main/docs">Documentation</a>
    </p>

    <h2>Important</h2>
    <p>
        <strong>Legal Notice:</strong> Only scan networks you own or have explicit permission to test.
    </p>
</body>
</html>
EOF

# Copy LICENSE for installer
cp "../../LICENSE" "$OUTPUT_DIR/LICENSE"

# Create final distribution package
FINAL_PKG="$OUTPUT_DIR/prtip-$VERSION-macos.pkg"
productbuild \
    --distribution "$OUTPUT_DIR/distribution.xml" \
    --resources "$OUTPUT_DIR" \
    --package-path "$OUTPUT_DIR" \
    "$FINAL_PKG"

# Clean up intermediate files
rm -f "$COMPONENT_PKG"
rm -rf "$PACKAGE_ROOT"
rm -f "$OUTPUT_DIR/distribution.xml"
rm -f "$OUTPUT_DIR/welcome.html"
rm -f "$OUTPUT_DIR/conclusion.html"
rm -f "$OUTPUT_DIR/LICENSE"

# Calculate checksum
shasum -a 256 "$FINAL_PKG" > "$FINAL_PKG.sha256"

echo ""
echo "Build complete: $FINAL_PKG"
echo "SHA256: $(cat "$FINAL_PKG.sha256" | cut -d' ' -f1)"
