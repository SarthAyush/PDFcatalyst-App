<#
.SYNOPSIS
  Ensures a valid Authenticode Code-Signing Certificate exists and is trusted for PDFix.
#>
[CmdletBinding()]
param()

$Subject = 'CN=PDFix, O=Sarthak, OU=PDF Engineering'

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
    -FriendlyName "PDFix Code Signing (Sarthak)"

$tempCer = Join-Path ([IO.Path]::GetTempPath()) "pdfix-$($cert.Thumbprint).cer"
Export-Certificate -Cert $cert -FilePath $tempCer | Out-Null

try {
    Import-Certificate -FilePath $tempCer -CertStoreLocation "Cert:\CurrentUser\Root" | Out-Null
    Import-Certificate -FilePath $tempCer -CertStoreLocation "Cert:\CurrentUser\TrustedPublisher" | Out-Null
    Write-Host "Certificate installed into CurrentUser Root and TrustedPublisher stores." -ForegroundColor Green
} finally {
    if (Test-Path $tempCer) { Remove-Item $tempCer -Force -ErrorAction SilentlyContinue }
}

Write-Host "Created and trusted Code Signing Certificate: $($cert.Thumbprint)" -ForegroundColor Green
return $cert
