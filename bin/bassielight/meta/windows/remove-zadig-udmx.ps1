param(
    [switch]$Apply
)

$ErrorActionPreference = "Stop"

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Run this script from an elevated PowerShell session."
}

$hardwareIdPattern = '(?i)USB\\VID_16C0&PID_05DC(?![&A-Z0-9_])'
$providerPattern = '(?im)^\s*(Provider|VendorName)\s*=.*(libwdi|zadig|libusb)'
$servicePattern = '(?im)^\s*AddService\s*=\s*(WinUSB|libusbK|libusb0)\s*,'
$packages = @()

Get-ChildItem -LiteralPath "$env:SystemRoot\INF" -Filter "oem*.inf" -File | ForEach-Object {
    $content = Get-Content -LiteralPath $_.FullName -Raw
    if ($content -notmatch $hardwareIdPattern -or
        $content -notmatch $providerPattern -or
        $content -notmatch $servicePattern) {
        return
    }

    $provider = [regex]::Match($content, $providerPattern).Value.Trim()
    $service = [regex]::Match($content, $servicePattern).Groups[1].Value
    $packages += [pscustomobject]@{
        Provider = $provider
        Service = $service
        PublishedInf = $_.Name
        Command = "pnputil /delete-driver $($_.Name) /uninstall /force"
    }
}

if ($packages.Count -eq 0) {
    Write-Host "No Zadig/libwdi uDMX driver packages matched."
    exit 0
}

$packages | Format-Table Provider, Service, PublishedInf, Command -AutoSize

if (-not $Apply) {
    Write-Host "Dry run only. Inspect every match, then rerun with -Apply to remove it."
    exit 0
}

foreach ($package in $packages) {
    Write-Host "Running: $($package.Command)"
    & pnputil /delete-driver $package.PublishedInf /uninstall /force
    if ($LASTEXITCODE -ne 0) {
        throw "pnputil failed for $($package.PublishedInf) with exit code $LASTEXITCODE"
    }
}
