<#
.SYNOPSIS
    Packages all PhotoCraft plugins from 'plugins/' into '.pcpl' archives in 'dist/packages/'.
.DESCRIPTION
    A .pcpl file is a standard ZIP archive containing plugin.json, assets, and optional models.
    Can be run independently without building or modifying the companion core.
#>

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$PluginsDir = Join-Path $RootDir "plugins"
$DistPackagesDir = Join-Path $RootDir "dist\packages"

if (!(Test-Path $DistPackagesDir)) {
    New-Item -ItemType Directory -Force -Path $DistPackagesDir | Out-Null
}

Write-Host ">>> Packaging PhotoCraft Plugins from '$PluginsDir' into '$DistPackagesDir'..." -ForegroundColor Cyan

Get-ChildItem -Directory $PluginsDir | ForEach-Object {
    $pluginFolder = $_.FullName
    $manifestPath = Join-Path $pluginFolder "plugin.json"
    
    if (Test-Path $manifestPath) {
        $name = $_.Name
        $targetPcpl = Join-Path $DistPackagesDir "$name.pcpl"
        
        if (Test-Path $targetPcpl) {
            Remove-Item -Force $targetPcpl
        }
        
        $tempZip = Join-Path $DistPackagesDir "$name.tmp.zip"
        if (Test-Path $tempZip) { Remove-Item -Force $tempZip }
        
        Write-Host "  Packing plugin '$name' -> '$targetPcpl'..." -ForegroundColor Green
        Compress-Archive -Path "$pluginFolder\*" -DestinationPath $tempZip -Force
        Move-Item -Path $tempZip -Destination $targetPcpl -Force
        
        $sizeKb = [math]::Round(((Get-Item $targetPcpl).Length / 1KB), 1)
        Write-Host "  Done: $name.pcpl ($sizeKb KB)" -ForegroundColor DarkGray
    }
}

Write-Host "`nAll plugins packaged successfully into: $DistPackagesDir" -ForegroundColor Yellow
