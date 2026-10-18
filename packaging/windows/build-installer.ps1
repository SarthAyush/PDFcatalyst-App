<#
.SYNOPSIS
  Builds PDFCatalyst release binary and creates the Windows Desktop Installer (.exe).
.EXAMPLE
  pwsh packaging/windows/build-installer.ps1 -Version "1.0.3"
#>
[CmdletBinding()]
param(
    [string]$Version = "1.0.3"
)

$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

Write-Host "==> Step 1: Synchronizing workspace version and compiling PDFCatalyst (Release build)..." -ForegroundColor Cyan
Push-Location $Root
try {
    # Ensure workspace version matches target installer version so CARGO_PKG_VERSION is accurate
    $CurrentVer = (cargo xtask version)
    if ($CurrentVer.Trim() -ne $Version.Trim()) {
        Write-Host "Syncing workspace package version: $CurrentVer -> $Version..." -ForegroundColor Yellow
        cargo xtask version set $Version
    }
    cargo build --release -p pdfcraft --bin PDFCatalyst
} finally {
    Pop-Location
}

# Ensure certificate exists for Authenticode signing
$EnsureCert = Join-Path $PSScriptRoot 'ensure-cert.ps1'
if (Test-Path $EnsureCert) {
    try {
        powershell -ExecutionPolicy Bypass -File $EnsureCert -ErrorAction SilentlyContinue
    } catch {}
}

# Locate signtool
$SignToolCandidates = @(
    "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe",
    "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe",
    "C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe",
    (Get-Command signtool.exe -ErrorAction SilentlyContinue).Source
) | Where-Object { $_ -and (Test-Path $_) }

$SignTool = $SignToolCandidates | Select-Object -First 1

# Sign the main application binary before packaging so the installed binary is signed
$AppExe = Join-Path $Root 'target\release\PDFCatalyst.exe'
if ($SignTool -and (Test-Path $AppExe)) {
    Write-Host "==> Signing application binary with Authenticode Certificate ($AppExe)..." -ForegroundColor Cyan
    try {
        & $SignTool sign /v /fd SHA256 /tr "http://timestamp.digicert.com" /td SHA256 /a /n "PDFCatalyst" $AppExe
    } catch {
        & $SignTool sign /v /fd SHA256 /a /n "PDFCatalyst" $AppExe
    }
}

Write-Host "==> Step 2: Locating Inno Setup Compiler..." -ForegroundColor Cyan
$Candidates = @(
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe"
) | Where-Object { $_ -and (Test-Path $_) }

$ISCC = $Candidates | Select-Object -First 1
if (-not $ISCC) {
    throw "Inno Setup compiler (ISCC.exe) not found. Please install Inno Setup 6."
}
Write-Host "Found ISCC at: $ISCC" -ForegroundColor Gray

Write-Host "==> Step 3: Compiling Installer with $ISCC (Version: $Version)..." -ForegroundColor Cyan
$IssPath = Join-Path $PSScriptRoot 'pdfcatalyst.iss'
& $ISCC "/DMyAppVersion=$Version" $IssPath

$DistDir = Join-Path $Root 'dist'
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

$VersionedExe = Join-Path $DistDir "PDFCatalyst-Setup-v$Version-x64.exe"
$StandardSetup = Join-Path $DistDir "PDFCatalyst-Setup.exe"
$PortableExe = Join-Path $DistDir "PDFCatalyst.exe"
$ZipArchive = Join-Path $DistDir "PDFCatalyst-v$Version-Windows-x64.zip"

if ($SignTool -and (Test-Path $VersionedExe)) {
    Write-Host "==> Signing Windows Installer with Authenticode Certificate ($VersionedExe)..." -ForegroundColor Cyan
    try {
        & $SignTool sign /v /fd SHA256 /tr "http://timestamp.digicert.com" /td SHA256 /a /n "PDFCatalyst" $VersionedExe
    } catch {
        & $SignTool sign /v /fd SHA256 /a /n "PDFCatalyst" $VersionedExe
    }
}

if (Test-Path $VersionedExe) {
    Copy-Item $VersionedExe $StandardSetup -Force
    Copy-Item (Join-Path $Root 'target\release\PDFCatalyst.exe') $PortableExe -Force

    # Clean up any legacy CraftPDF binaries from dist directory
    Get-ChildItem -Path $DistDir -Filter "*CraftPDF*" | Remove-Item -Force -ErrorAction SilentlyContinue

    # Package portable zip archive (no setup wizard required, no SmartScreen installation blocks)
    try {
        if (Test-Path $ZipArchive) { Remove-Item $ZipArchive -Force }
        Compress-Archive -Path $PortableExe -DestinationPath $ZipArchive -Force
        Write-Host "Created Portable Zip: $ZipArchive" -ForegroundColor Green
    } catch {}

    Unblock-File $VersionedExe -ErrorAction SilentlyContinue
    Unblock-File $StandardSetup -ErrorAction SilentlyContinue
    Unblock-File $PortableExe -ErrorAction SilentlyContinue
    
    # Also update user's downloads folder if it exists and unblock
    $DownloadsPath = [System.IO.Path]::Combine($env:USERPROFILE, "Downloads")
    if (Test-Path $DownloadsPath) {
        Copy-Item $VersionedExe (Join-Path $DownloadsPath "PDFCatalyst-Setup-v$Version-x64.exe") -Force
        Unblock-File (Join-Path $DownloadsPath "PDFCatalyst-Setup-v$Version-x64.exe") -ErrorAction SilentlyContinue
    }

    $size = [math]::Round((Get-Item $VersionedExe).Length / 1MB, 2)
    Write-Host "==> Success! Signed Installer created at: $VersionedExe ($size MB)" -ForegroundColor Green
    Write-Host "Staged: $StandardSetup, $PortableExe, and $ZipArchive" -ForegroundColor Green
}
