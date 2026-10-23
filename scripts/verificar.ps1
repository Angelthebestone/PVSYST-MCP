<#
.SYNOPSIS
    Script de verificacion completa de pvsyst-mcp (§4b.8).
    Ejecuta fmt, clippy, test --lib, cada binario de integracion y doctests con timeout.
    Redirige la salida de cada binario a .scratch/verificacion/ y mata procesos huerfanos PVsyst.exe nuevos.
#>

$ErrorActionPreference = "Continue"

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$ScratchDir = Join-Path $Root ".scratch\verificacion"
if (-not (Test-Path $ScratchDir)) {
    New-Item -ItemType Directory -Path $ScratchDir -Force | Out-Null
}

function Get-PVsystPids {
    $pids = @()
    $procs = Get-Process -Name "PVsyst" -ErrorAction SilentlyContinue
    if ($procs) {
        foreach ($p in $procs) {
            $pids += $p.Id
        }
    }
    return $pids
}

function Kill-NewPVsystPids($initialPids) {
    $currentPids = Get-PVsystPids
    # $pid es la variable automatica (solo lectura) del propio host: no sirve de iterador.
    foreach ($idHuerfano in $currentPids) {
        if ($initialPids -notcontains $idHuerfano) {
            Write-Host "Matando proceso huerfano PVsyst.exe nuevo (PID: $idHuerfano)..." -ForegroundColor Yellow
            Stop-Process -Id $idHuerfano -Force -ErrorAction SilentlyContinue
        }
    }
}
$PvsystIniciales = Get-PVsystPids
if ($PvsystIniciales.Count -gt 0) {
    Write-Host "AVISO: Se detecto PVsyst.exe en ejecucion previamente (PIDs: $($PvsystIniciales -join ', '))." -ForegroundColor Yellow
    Write-Host "AVISO: Esto invalida las pruebas de GUI/procesos huerfanos. No se cerrara para proteger la sesion del usuario." -ForegroundColor Yellow
}

# Start-Process redirige a archivo: los manejadores DataReceived con scriptblock abortan
# Windows PowerShell 5.1 al ejecutarse fuera de su runspace.
function Run-StepWithTimeout($name, $cmd, $argsList, $timeoutSeconds, $logFile) {
    Write-Host "`n=== [$name] ===" -ForegroundColor Cyan
    $initialPids = Get-PVsystPids
    $errFile = "$logFile.stderr"

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $proc = Start-Process -FilePath $cmd -ArgumentList $argsList -WorkingDirectory $Root `
        -NoNewWindow -PassThru -RedirectStandardOutput $logFile -RedirectStandardError $errFile
    # Sin abrir el handle ahora, ExitCode llega vacio cuando el proceso ya termino.
    $null = $proc.Handle
    $completed = $proc.WaitForExit($timeoutSeconds * 1000)
    $sw.Stop()

    if ($completed) {
        $proc.WaitForExit()
        $exitCode = $proc.ExitCode
    } else {
        Write-Host "TIMEOUT tras $timeoutSeconds s en $name!" -ForegroundColor Red
        # /T mata tambien el binario de tests hijo de cargo, que si no quedaria huerfano.
        taskkill /T /F /PID $proc.Id | Out-Null
        $exitCode = -1
    }

    Kill-NewPVsystPids $initialPids

    $segundos = [math]::Round($sw.Elapsed.TotalSeconds, 2)
    if ($exitCode -eq 0) {
        Write-Host "OK ($segundos s)" -ForegroundColor Green
    } else {
        Write-Host "FALLO (ExitCode: $exitCode, $segundos s; ver $errFile)" -ForegroundColor Red
    }

    return $exitCode
}

$GlobalExit = 0

# 1. Cargo fmt --check
$res = Run-StepWithTimeout "cargo fmt --check" "cargo" @("fmt", "--check") 120 (Join-Path $ScratchDir "fmt.log")
if ($res -ne 0) { $GlobalExit = $res }

# 2. Cargo clippy --all-targets -D warnings
$res = Run-StepWithTimeout "cargo clippy --all-targets -D warnings" "cargo" @("clippy", "--all-targets", "--", "-D", "warnings") 300 (Join-Path $ScratchDir "clippy.log")
if ($res -ne 0) { $GlobalExit = $res }

# 3. Cargo test --lib
$res = Run-StepWithTimeout "cargo test --lib" "cargo" @("test", "--lib") 600 (Join-Path $ScratchDir "test_lib.log")
if ($res -ne 0) { $GlobalExit = $res }

# 4. Cada binario de integracion: uno por tests/*.rs, para que uno nuevo no quede fuera.
$integracionTests = Get-ChildItem -Path (Join-Path $Root "tests") -Filter "*.rs" -File |
    ForEach-Object { $_.BaseName } | Sort-Object

foreach ($t in $integracionTests) {
    $log = Join-Path $ScratchDir "test_$t.log"
    $res = Run-StepWithTimeout "cargo test --test $t" "cargo" @("test", "--test", $t) 600 $log
    if ($res -ne 0) { $GlobalExit = $res }
}

# 5. Doctests
$res = Run-StepWithTimeout "cargo test --doc" "cargo" @("test", "--doc") 300 (Join-Path $ScratchDir "test_doc.log")
if ($res -ne 0) { $GlobalExit = $res }

Write-Host "`n=========================================="
if ($GlobalExit -eq 0) {
    Write-Host "Verificacion de tests completada exitosamente!" -ForegroundColor Green
} else {
    Write-Host "Verificacion termino con codigo: $GlobalExit" -ForegroundColor Red
}
Write-Host "=========================================="

exit $GlobalExit
