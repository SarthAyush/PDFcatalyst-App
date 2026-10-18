<#
.SYNOPSIS
  Ensures a valid Authenticode Code-Signing Certificate exists and is trusted for PDFCatalyst.
#>
[CmdletBinding()]
param()

$Subject = 'CN=PDFCatalyst, O=Sarthak, OU=PDF Engineering'

# Check if already exists and valid
$cert = Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert -ErrorAction SilentlyContinue |
    Where-Object { $_.Subject -eq $Subject -and $_.NotAfter -gt (Get-Date).AddDays(30) } |
    Select-Object -First 1

if (-not $cert) {
    Write-Host "Creating new Code Signing Certificate for '$Subject'..." -ForegroundColor Cyan
    $cert = New-SelfSignedCertificate `
        -Type CodeSigningCert `
        -Subject $Subject `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -NotAfter (Get-Date).AddYears(5) `
        -FriendlyName "PDFCatalyst Code Signing (Sarthak)"
    Write-Host "Created Code Signing Certificate in CurrentUser\My: $($cert.Thumbprint)" -ForegroundColor Green
} else {
    Write-Host "Found existing valid Code Signing Certificate: $($cert.Thumbprint)" -ForegroundColor Green
}

# Ensure certificate is trusted in CurrentUser\Root and CurrentUser\TrustedPublisher so Windows SmartScreen recognizes the publisher
try {
    $rootStore = New-Object System.Security.Cryptography.X509Certificates.X509Store "Root", "CurrentUser"
    $rootStore.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
    if (-not ($rootStore.Certificates | Where-Object { $_.Thumbprint -eq $cert.Thumbprint })) {
        $rootStore.Add($cert)
        Write-Host "Trusted in Cert:\CurrentUser\Root" -ForegroundColor Green
    }
    $rootStore.Close()

    $pubStore = New-Object System.Security.Cryptography.X509Certificates.X509Store "TrustedPublisher", "CurrentUser"
    $pubStore.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
    if (-not ($pubStore.Certificates | Where-Object { $_.Thumbprint -eq $cert.Thumbprint })) {
        $pubStore.Add($cert)
        Write-Host "Trusted in Cert:\CurrentUser\TrustedPublisher" -ForegroundColor Green
    }
    $pubStore.Close()
} catch {
    Write-Warning "Could not register root trust: $_"
}

# Export public certificate (.cer)
$CerPath = Join-Path $PSScriptRoot 'PDFCatalyst-Certificate.cer'
Export-Certificate -Cert $cert -FilePath $CerPath -Force | Out-Null
Write-Host "Exported public certificate to $CerPath" -ForegroundColor Gray

return $cert
