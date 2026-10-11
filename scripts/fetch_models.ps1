# PhotoCraft Companion — Developer Script: Fetch Neural Network Models
# Downloads required ONNX model weights declared in plugins/*/plugin.json if they are not already present.

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot

Write-Host "=== PhotoCraft Plugins: Fetching Neural Network Models ===" -ForegroundColor Cyan

$pluginDirs = Get-ChildItem -Directory -Path (Join-Path $Root "plugins")
$foundCount = 0
$downloadedCount = 0

foreach ($dir in $pluginDirs) {
    $manifestPath = Join-Path $dir.FullName "plugin.json"
    if (Test-Path $manifestPath) {
        $json = Get-Content $manifestPath -Raw | ConvertFrom-Json
        if ($json.models -and $json.models.Count -gt 0) {
            $modelsDir = Join-Path $dir.FullName "models"
            if (-not (Test-Path $modelsDir)) {
                New-Item -ItemType Directory -Path $modelsDir -Force | Out-Null
            }

            foreach ($model in $json.models) {
                $targetFile = Join-Path $modelsDir $model.name
                if (Test-Path $targetFile) {
                    $size = [math]::Round((Get-Item $targetFile).Length / 1MB, 1)
                    Write-Host "  [OK] $($json.name): $($model.name) ($size MB) already exists." -ForegroundColor Green
                    $foundCount++
                } else {
                    if ($model.url) {
                        Write-Host "  [DOWNLOADING] $($json.name): $($model.name) from $($model.url)..." -ForegroundColor Yellow
                        curl.exe -L -o $targetFile $model.url --progress-bar
                        if (Test-Path $targetFile) {
                            $size = [math]::Round((Get-Item $targetFile).Length / 1MB, 1)
                            Write-Host "  [DONE] Successfully downloaded $($model.name) ($size MB)." -ForegroundColor Green
                            $downloadedCount++
                        }
                    } else {
                        Write-Host "  [WARNING] $($model.name) missing and no download URL specified in manifest." -ForegroundColor Red
                    }
                }
            }
        }
    }
}

Write-Host "`nSummary: $foundCount model(s) verified, $downloadedCount model(s) downloaded." -ForegroundColor Cyan
