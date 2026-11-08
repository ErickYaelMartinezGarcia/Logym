param(
    [Parameter(Mandatory=$true)][string]$Ejecutable,
    [Parameter(Mandatory=$true)][ValidateSet('respaldar','restaurar','importar_legacy')][string]$Operacion,
    [Parameter(Mandatory=$true)][string]$Base,
    [Parameter(Mandatory=$true)][string]$Respaldo
)
$ErrorActionPreference = 'Stop'
function Texto-Seguro($Valor) {
    $ptr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($Valor)
    try { [Runtime.InteropServices.Marshal]::PtrToStringBSTR($ptr) }
    finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($ptr) }
}
if (-not [IO.Path]::IsPathRooted($Base) -or -not [IO.Path]::IsPathRooted($Respaldo)) { throw 'Usa rutas absolutas.' }
$admin = Read-Host 'Contrasena administrativa (o nueva para importar un legacy sin administrador)' -AsSecureString
$backup = if ($Operacion -eq 'importar_legacy') { New-Object Security.SecureString } else { Read-Host 'Contrasena del respaldo (minimo 16 caracteres)' -AsSecureString }
$request = @{ operacion=$Operacion; base=$Base; respaldo=$Respaldo; password_admin=(Texto-Seguro $admin); password_respaldo=(Texto-Seguro $backup) }
$start = New-Object Diagnostics.ProcessStartInfo
$start.FileName = (Resolve-Path -LiteralPath $Ejecutable).Path
$start.Arguments = '--mantenimiento'
$start.UseShellExecute = $false
$start.CreateNoWindow = $true
$start.RedirectStandardInput = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$process = New-Object Diagnostics.Process
$process.StartInfo = $start
try {
    [void]$process.Start()
    $process.StandardInput.Write(($request | ConvertTo-Json -Compress))
    $process.StandardInput.Close()
    $salida = $process.StandardOutput.ReadToEnd()
    $errorProceso = $process.StandardError.ReadToEnd()
    $process.WaitForExit()
    if ($process.ExitCode -ne 0) { throw $errorProceso }
    Write-Output $salida
} finally {
    $request.Clear(); $admin.Dispose(); $backup.Dispose(); $process.Dispose()
}
