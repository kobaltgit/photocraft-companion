<#
.SYNOPSIS
    Packages PhotoCraft Companion for Windows (Portable .zip and Inno Setup .exe installer).
#>

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent $PSScriptRoot
Set-Location $RootDir

Write-Host ">>> Building release binary (photocraft-companion)..." -ForegroundColor Cyan
cargo build --package companion_app --release -j 1

# Ensure official Microsoft ONNX Runtime dynamic library and DirectML are available for packaging
$OrtDll = Join-Path $RootDir "target\release\onnxruntime.dll"
$DmlDll = Join-Path $RootDir "target\release\DirectML.dll"
New-Item -ItemType Directory -Force -Path (Join-Path $RootDir "target\release") | Out-Null

if (-not (Test-Path $OrtDll)) {
    Write-Host ">>> Fetching official Microsoft ONNX Runtime DLL..." -ForegroundColor Cyan
    $NupkgUrl = "https://api.nuget.org/v3-flatcontainer/microsoft.ml.onnxruntime.directml/1.20.0/microsoft.ml.onnxruntime.directml.1.20.0.nupkg"
    $TempZip = Join-Path $RootDir "target\ort.zip"
    curl.exe -s -L -o $TempZip $NupkgUrl
    tar.exe -xf $TempZip -C (Join-Path $RootDir "target") runtimes/win-x64/native/onnxruntime.dll
    Copy-Item (Join-Path $RootDir "target\runtimes\win-x64\native\onnxruntime.dll") -Destination (Join-Path $RootDir "target\release\") -Force
    Remove-Item $TempZip -Force -ErrorAction SilentlyContinue
    Remove-Item -Recurse (Join-Path $RootDir "target\runtimes") -Force -ErrorAction SilentlyContinue
}

if (-not (Test-Path $DmlDll)) {
    Write-Host ">>> Fetching official Microsoft DirectML DLL..." -ForegroundColor Cyan
    $DmlNupkgUrl = "https://api.nuget.org/v3-flatcontainer/microsoft.ai.directml/1.15.4/microsoft.ai.directml.1.15.4.nupkg"
    $TempZip = Join-Path $RootDir "target\dml.zip"
    curl.exe -s -L -o $TempZip $DmlNupkgUrl
    tar.exe -xf $TempZip -C (Join-Path $RootDir "target") bin/x64-win/DirectML.dll
    Copy-Item (Join-Path $RootDir "target\bin\x64-win\DirectML.dll") -Destination (Join-Path $RootDir "target\release\") -Force
    Remove-Item $TempZip -Force -ErrorAction SilentlyContinue
    Remove-Item -Recurse (Join-Path $RootDir "target\bin") -Force -ErrorAction SilentlyContinue
}

$Version = "2.2.5"
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
if (Test-Path (Join-Path $RootDir "target\release\onnxruntime.dll")) {
    Copy-Item (Join-Path $RootDir "target\release\onnxruntime.dll") -Destination $PortableDir
}
if (Test-Path (Join-Path $RootDir "target\release\DirectML.dll")) {
    Copy-Item (Join-Path $RootDir "target\release\DirectML.dll") -Destination $PortableDir
}
Copy-Item -Recurse (Join-Path $RootDir "plugins") -Destination $PortableDir
if (Test-Path (Join-Path $DistDir "registry.json")) {
    Copy-Item (Join-Path $DistDir "registry.json") -Destination $PortableDir
}
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