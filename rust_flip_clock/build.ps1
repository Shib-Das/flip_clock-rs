# Automate the build process for rust_flip_clock

$scriptPath = $PSScriptRoot
if (-not $scriptPath) {
    $scriptPath = Get-Location
}

# Ensure we are in the project root relative to this script if needed,
# or just target the cargo file correctly.
# Assuming this script is inside rust_flip_clock/ or the root?
# The user asked for "build.ps1". The previous tool put it in `rust_flip_clock/build.ps1`.
# So $PSScriptRoot is .../rust_flip_clock/

Write-Host "Building rust_flip_clock..."
# Run cargo build from the script's directory
Push-Location $scriptPath

cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed." -ForegroundColor Red
    Pop-Location
    exit 1
}

$dist = "dist"
if (-not (Test-Path $dist)) {
    New-Item -ItemType Directory -Force -Path $dist | Out-Null
}

$sourceExe = "target/release/rust_flip_clock.exe"
$destScr = "$dist/rust_flip_clock.scr"

if (Test-Path $sourceExe) {
    Copy-Item -Path $sourceExe -Destination $destScr -Force
    Write-Host "Copied executable to $destScr"
} else {
    Write-Host "Error: Could not find compiled executable at $sourceExe" -ForegroundColor Red
    Pop-Location
    exit 1
}

# Check for font.ttf in the root (assuming root of the repo/workspace, or adjacent to script?)
# "If a file named font.ttf exists in the root..."
# Let's check both current dir and up one level just in case.
$fontSource = "font.ttf"
if (Test-Path $fontSource) {
    Copy-Item -Path $fontSource -Destination "$dist/font.ttf" -Force
    Write-Host "Copied font.ttf to dist folder."
} else {
     # Check one level up (repo root)
     $fontSourceRoot = "../font.ttf"
     if (Test-Path $fontSourceRoot) {
        Copy-Item -Path $fontSourceRoot -Destination "$dist/font.ttf" -Force
        Write-Host "Copied font.ttf (from ../) to dist folder."
     }
}

Pop-Location
Write-Host "Build Complete!" -ForegroundColor Green
Write-Host "To install, right-click '$destScr' and select 'Install'." -ForegroundColor Cyan
