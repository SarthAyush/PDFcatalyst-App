<#
.SYNOPSIS
  Builds PDFCatalyst release binary and creates the Windows Desktop Installer (.exe).
.EXAMPLE
  pwsh packaging/windows/build-installer.ps1 -Version "1.0.0"
#>
[CmdletBinding()]
param(
    [string]$Version = "1.0.0"
)

$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

Write-Host "==> Step 1: Compiling PDFCatalyst (Release build)..." -ForegroundColor Cyan
Push-Location $Root
try {
    cargo build --release -p pdfcraft --bin PDFCatalyst
} finally {
    Pop-Location
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
if (-not (Test-Path $VersionedExe)) {
    $LegacyExe = Join-Path $DistDir "CraftPDF-Setup-v$Version-x64.exe"
    if (Test-Path $LegacyExe) {
        Move-Item $LegacyExe $VersionedExe -Force
    }
}

$StandardSetup = Join-Path $DistDir "PDFCatalyst-Setup.exe"
$PortableExe = Join-Path $DistDir "PDFCatalyst.exe"

# Ensure certificate exists for signing
$EnsureCert = Join-Path $PSScriptRoot 'ensure-cert.ps1'
if (Test-Path $EnsureCert) {
    try {
        & pwsh -ExecutionPolicy Bypass -File $EnsureCert -ErrorAction SilentlyContinue
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
if ($SignTool) {
    Write-Host "==> Signing binaries with Authenticode Certificate via $SignTool..." -ForegroundColor Cyan
    $AppExe = Join-Path $Root 'target\release\PDFCatalyst.exe'
    if (Test-Path $AppExe) {
        & $SignTool sign /v /fd SHA256 /a /n "PDFCatalyst" $AppExe
    }
    if (Test-Path $VersionedExe) {
        & $SignTool sign /v /fd SHA256 /a /n "PDFCatalyst" $VersionedExe
    }
}

if (Test-Path $VersionedExe) {
    Copy-Item $VersionedExe $StandardSetup -Force
    Copy-Item (Join-Path $Root 'target\release\PDFCatalyst.exe') $PortableExe -Force
    
    # Also stage legacy named binaries so both names exist in dist
    Copy-Item $VersionedExe (Join-Path $DistDir "CraftPDF-Setup-v$Version-x64.exe") -Force
    Copy-Item $StandardSetup (Join-Path $DistDir "CraftPDF-Setup.exe") -Force
    Copy-Item $PortableExe (Join-Path $DistDir "CraftPDF.exe") -Force

    Unblock-File $VersionedExe -ErrorAction SilentlyContinue
    Unblock-File $StandardSetup -ErrorAction SilentlyContinue
    
    # Also update user's downloads folder if it exists
    $DownloadsPath = [System.IO.Path]::Combine($env:USERPROFILE, "Downloads")
    if (Test-Path $DownloadsPath) {
        Copy-Item $VersionedExe (Join-Path $DownloadsPath "PDFCatalyst-Setup-v$Version-x64.exe") -Force
        Unblock-File (Join-Path $DownloadsPath "PDFCatalyst-Setup-v$Version-x64.exe") -ErrorAction SilentlyContinue
        # Also unblock any existing old installer in Downloads
        $OldInDownloads = Join-Path $DownloadsPath "CraftPDF-Setup-v$Version-x64.exe"
        if (Test-Path $OldInDownloads) {
            Unblock-File $OldInDownloads -ErrorAction SilentlyContinue
        }
    }

    $size = [math]::Round((Get-Item $VersionedExe).Length / 1MB, 2)
    Write-Host "==> Success! Signed Installer created at: $VersionedExe ($size MB)" -ForegroundColor Green
    Write-Host "Staged: $StandardSetup and $PortableExe" -ForegroundColor Green
}
