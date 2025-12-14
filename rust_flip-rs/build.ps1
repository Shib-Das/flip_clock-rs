# Automate the build process for the Windows Screensaver

# 1. Build release
Write-Host "Building release..."
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed."
    exit 1
}

# 2. Create dist folder
$dist = "dist"
if (-not (Test-Path $dist)) {
    New-Item -ItemType Directory -Force -Path $dist | Out-Null
}

# 3. Copy executable to .scr
$srcParams = "target/release/rust_flip_clock.exe"
$destParams = "$dist/rust_flip_clock.scr"

if (Test-Path $srcParams) {
    Copy-Item -Path $srcParams -Destination $destParams -Force
    Write-Host "Copied executable to $destParams"
} else {
    Write-Error "Could not find executable at $srcParams"
    exit 1
}

# 4. Copy font.ttf if exists
if (Test-Path "font.ttf") {
    Copy-Item -Path "font.ttf" -Destination "$dist/" -Force
    Write-Host "Copied font.ttf to dist/"
}

# 5. Output success message
Write-Host "Build complete! Right-click $destParams and select 'Install' to use as screensaver."
