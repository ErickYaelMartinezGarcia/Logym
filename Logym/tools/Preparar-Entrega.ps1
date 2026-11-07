param(
    [Parameter(Mandatory=$true)][string]$ControladorOficial,
    [Parameter(Mandatory=$true)][string]$RuntimeMicrosoft,
    [Parameter(Mandatory=$true)][string]$InstaladorLogym
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $root ('.artifacts/entrega-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
foreach ($entry in @(@{ruta=$RuntimeMicrosoft;emisor='Microsoft'},@{ruta=$ControladorOficial;emisor='SecuGen'})) {
    $firma = Get-AuthenticodeSignature -LiteralPath $entry.ruta
    if ($firma.Status -ne 'Valid' -or $firma.SignerCertificate.Subject -notmatch $entry.emisor) { throw "No se pudo verificar el proveedor: $($entry.ruta)" }
}
New-Item -ItemType Directory -Path $out | Out-Null
Copy-Item -LiteralPath $ControladorOficial -Destination (Join-Path $out 'SecuGenDriver.exe')
Copy-Item -LiteralPath $RuntimeMicrosoft -Destination (Join-Path $out 'vc_redist.x64.exe')
Copy-Item -LiteralPath $InstaladorLogym -Destination (Join-Path $out 'Logym-setup.exe')
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'Instalar-Logym.ps1') -Destination $out
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'Mantenimiento.ps1') -Destination $out
$files = Get-ChildItem -LiteralPath $out -File | ForEach-Object { @{ nombre=$_.Name;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash } }
@{ archivos=@($files);generado=(Get-Date -Format o);arquitectura='x64' } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $out 'manifest.json') -Encoding UTF8
Write-Output $out
