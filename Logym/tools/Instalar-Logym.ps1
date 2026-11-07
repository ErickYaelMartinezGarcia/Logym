param([Parameter(Mandatory=$true)][string]$Carpeta)
$ErrorActionPreference = 'Stop'
$Carpeta = (Resolve-Path -LiteralPath $Carpeta).Path
$manifest = Get-Content -LiteralPath (Join-Path $Carpeta 'manifest.json') -Raw | ConvertFrom-Json
foreach ($file in $manifest.archivos) {
    if ([IO.Path]::GetFileName($file.nombre) -ne $file.nombre) { throw 'Nombre de archivo no permitido en manifiesto.' }
    $ruta = Join-Path $Carpeta $file.nombre
    if ((Get-FileHash -LiteralPath $ruta -Algorithm SHA256).Hash -ne $file.sha256) { throw "Hash incorrecto: $($file.nombre)" }
}
foreach ($entry in @(@{nombre='vc_redist.x64.exe';emisor='Microsoft'},@{nombre='SecuGenDriver.exe';emisor='SecuGen'})) {
    $signature = Get-AuthenticodeSignature -LiteralPath (Join-Path $Carpeta $entry.nombre)
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch $entry.emisor) { throw "Firma no verificada: $($entry.nombre)" }
}
# Estas dependencias requieren elevacion; el instalador de la app es por usuario.
$runtime = Start-Process -FilePath (Join-Path $Carpeta 'vc_redist.x64.exe') -ArgumentList '/install /quiet /norestart' -Verb RunAs -Wait -PassThru -WindowStyle Hidden
if ($runtime.ExitCode -notin @(0,1638,3010)) { throw "VC++ fallo: $($runtime.ExitCode)" }
if ($runtime.ExitCode -eq 3010) { throw 'Windows requiere reinicio. Reinicia y repite la instalacion.' }
# No se inventan parametros silenciosos del proveedor: seguir su asistente.
$driver = Start-Process -FilePath (Join-Path $Carpeta 'SecuGenDriver.exe') -Verb RunAs -Wait -PassThru
if ($driver.ExitCode -ne 0) { throw "Controlador pendiente; revisar el asistente SecuGen: $($driver.ExitCode)" }
$app = Start-Process -FilePath (Join-Path $Carpeta 'Logym-setup.exe') -Wait -PassThru
if ($app.ExitCode -ne 0) { throw "Instalador Logym fallo: $($app.ExitCode)" }
Write-Output 'Instalacion completada. Faltan activacion y pruebas de aceptacion con el lector.'
