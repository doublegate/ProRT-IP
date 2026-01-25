# ProRT-IP Windows Package Build Script
# Requires: WiX Toolset v4 (https://wixtoolset.org/)

param(
    [string]$Version = "1.0.0",
    [string]$SourceDir = "..\..\target\release",
    [string]$OutputDir = ".\output"
)

$ErrorActionPreference = "Stop"

Write-Host "Building ProRT-IP Windows Package v$Version" -ForegroundColor Cyan

# Ensure output directory exists
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Check for WiX
$wix = Get-Command "wix" -ErrorAction SilentlyContinue
if (-not $wix) {
    Write-Error "WiX Toolset not found. Install from: https://wixtoolset.org/"
    exit 1
}

# Check for binary
if (-not (Test-Path "$SourceDir\prtip.exe")) {
    Write-Error "prtip.exe not found at $SourceDir. Run 'cargo build --release' first."
    exit 1
}

# Create LICENSE.rtf for WiX UI
$licenseText = Get-Content "..\..\LICENSE" -Raw
$rtfContent = "{\rtf1\ansi\deff0 {\fonttbl {\f0 Consolas;}}
\f0\fs18
$($licenseText -replace '\n', '\par ')
}"
$rtfContent | Out-File -FilePath "$SourceDir\LICENSE.rtf" -Encoding ASCII

# Copy documentation
Copy-Item "..\..\README.md" "$SourceDir\" -Force
Copy-Item "..\..\LICENSE" "$SourceDir\" -Force
Copy-Item "..\..\CHANGELOG.md" "$SourceDir\" -Force

# Build MSI
Write-Host "Running WiX build..." -ForegroundColor Yellow
wix build `
    -d SourceDir="$SourceDir" `
    -d Version=$Version `
    -o "$OutputDir\prtip-$Version-x86_64-windows.msi" `
    prtip.wxs

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful: $OutputDir\prtip-$Version-x86_64-windows.msi" -ForegroundColor Green
} else {
    Write-Error "Build failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}

# Calculate checksum
$hash = Get-FileHash "$OutputDir\prtip-$Version-x86_64-windows.msi" -Algorithm SHA256
Write-Host "SHA256: $($hash.Hash)" -ForegroundColor Cyan
$hash.Hash | Out-File "$OutputDir\prtip-$Version-x86_64-windows.msi.sha256"
