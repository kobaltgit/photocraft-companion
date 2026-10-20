<#
.SYNOPSIS
    Packages PhotoCraft Companion for Windows (Portable .zip and Inno Setup .exe installer).
#>

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent $PSScriptRoot
Set-Location $RootDir

Write-Host ">>> Building release binary (photocraft-companion)..." -ForegroundColor Cyan
cargo build --package companion_app --release -j 1

$Version = "1.2.0"
$DistDir = Join-Path $RootDir "dist"
$PortableDir = Join-Path $DistDir "photocraft-companion-v$Version-windows-x64-portable"
$ZipPath = Join-Path $DistDir "PhotoCraft-Companion-v$Version-windows-x64-portable.zip"

if (Test-Path $DistDir) {
    Remove-Item -Recurse -Force $DistDir
}
New-Item -ItemType Directory -Force -Path $PortableDir | Out-Null

Write-Host ">>> Building plugin registry and .pcpl packages..." -ForegroundColor Cyan
cargo run --package plugin_validator -j 1 -- --validate plugins --build-registry --package-plugins

Write-Host ">>> Assembling portable distribution bundle..." -ForegroundColor Cyan
Copy-Item (Join-Path $RootDir "target\release\photocraft-companion.exe") -Destination $PortableDir
Copy-Item -Recurse (Join-Path $RootDir "plugins") -Destination $PortableDir
if (Test-Path (Join-Path $DistDir "registry.json")) {
    Copy-Item (Join-Path $DistDir "registry.json") -Destination $PortableDir
}
New-Item -ItemType Directory -Force -Path (Join-Path $PortableDir "scripts") | Out-Null
Copy-Item (Join-Path $RootDir "scripts\fetch_models.ps1") -Destination (Join-Path $PortableDir "scripts")
Copy-Item (Join-Path $RootDir "README.md") -Destination $PortableDir
Copy-Item (Join-Path $RootDir "LICENSE") -Destination $PortableDir

# Exclude heavy onnx models from the base archive distribution (they are fetched on demand or during setup)
Get-ChildItem -Path $PortableDir -Recurse -Filter "*.onnx" | Remove-Item -Force

Write-Host ">>> Creating portable zip: $ZipPath..." -ForegroundColor Green
Compress-Archive -Path "$PortableDir\*" -DestinationPath $ZipPath -Force

Write-Host ">>> Checking for Inno Setup compiler (ISCC.exe)..." -ForegroundColor Cyan
$IsccCandidates = @(
    "ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
)

$IsccPath = $null
foreach ($cand in $IsccCandidates) {
    if (Get-Command $cand -ErrorAction SilentlyContinue) {
        $IsccPath = $cand
        break
    } elseif (Test-Path $cand) {
        $IsccPath = $cand
        break
    }
}

if ($IsccPath) {
    Write-Host ">>> Compiling Windows Setup installer via Inno Setup..." -ForegroundColor Cyan
    & $IsccPath (Join-Path $RootDir "packaging\windows\installer.iss")
    Write-Host ">>> Installer created at: dist\PhotoCraft-Companion-Setup-Windows-x64.exe" -ForegroundColor Green
} else {
    Write-Host "(!) ISCC.exe not found on PATH. Portable zip is ready, to build installer install Inno Setup 6." -ForegroundColor Yellow
}

Write-Host "`n=== Packaging Complete ===" -ForegroundColor Green
Get-ChildItem $DistDir | Select-Object Name, Length
