Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Fix UTF-8 encoding for console output
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

Set-Location "$PSScriptRoot\.."

Write-Host "=== Build release ===" -ForegroundColor Cyan
cargo build --release

Write-Host "=== Package to .\redra ===" -ForegroundColor Cyan

# Clean old
if (Test-Path redra) { Remove-Item redra -Recurse -Force }
$null = New-Item -ItemType Directory -Path redra

# Check binary exists
if (-not (Test-Path "target\release\redra.exe")) {
    Write-Host "ERROR: target\release\redra.exe not found" -ForegroundColor Red
    exit 1
}

# Copy binary
Copy-Item "target\release\redra.exe" "redra\redra.exe"

# Copy assets directories (robocopy handles recursive dir copy reliably)
$dirs = @("fonts", "init", "materials")
foreach ($dir in $dirs) {
    robocopy "assets\$dir" "redra\assets\$dir" /E /NJH /NJS /NDL /NFL
    if ($LASTEXITCODE -ge 8) {
        Write-Host "ERROR: failed to copy assets\$dir" -ForegroundColor Red
        exit 1
    }
}

# Verify
if (-not (Test-Path "redra\redra.exe")) {
    Write-Host "ERROR: redra.exe not copied" -ForegroundColor Red
    exit 1
}

Write-Host "=== Done ===" -ForegroundColor Green
Write-Host "Contents:"
Get-ChildItem redra -Recurse -File | Select-Object -First 30 | ForEach-Object { $_.FullName.Replace((Get-Location).Path + "\", "") }
Write-Host ""
Write-Host "Run: .\redra\redra.exe" -ForegroundColor Yellow
