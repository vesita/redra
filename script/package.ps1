Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Set-Location "$PSScriptRoot\.."

Write-Host "=== Build release ===" -ForegroundColor Cyan
cargo build --release

Write-Host "=== Package to .\redra ===" -ForegroundColor Cyan
if (Test-Path redra) { Remove-Item redra -Recurse -Force }
New-Item -ItemType Directory -Path redra | Out-Null

# Executable
Copy-Item -Path "target\release\redra.exe" -Destination "redra\redra.exe"

# Assets
Copy-Item -Recurse -Path "assets\fonts"     -Destination "redra\assets\fonts"
Copy-Item -Recurse -Path "assets\init"      -Destination "redra\assets\init"
Copy-Item -Recurse -Path "assets\materials" -Destination "redra\assets\materials"

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
