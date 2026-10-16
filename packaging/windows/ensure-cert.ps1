<#
.SYNOPSIS
  Ensures a valid Authenticode Code-Signing Certificate exists and is trusted for PDFCatalyst.
#>
[CmdletBinding()]
param()

$Subject = 'CN=PDFCatalyst, O=Sarthak, OU=PDF Engineering'

# Check if already exists and valid
$existing = Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert -ErrorAction SilentlyContinue |
    Where-Object { $_.Subject -eq $Subject -and $_.NotAfter -gt (Get-Date).AddDays(30) } |
    Select-Object -First 1

if ($existing) {
    Write-Host "Found existing valid Code Signing Certificate: $($existing.Thumbprint)" -ForegroundColor Green
    return $existing
}

Write-Host "Creating new Code Signing Certificate for '$Subject'..." -ForegroundColor Cyan
$cert = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject $Subject `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -NotAfter (Get-Date).AddYears(5) `
    -FriendlyName "PDFCatalyst Code Signing (Sarthak)"

Write-Host "Created Code Signing Certificate in CurrentUser\My: $($cert.Thumbprint)" -ForegroundColor Green
return $cert
