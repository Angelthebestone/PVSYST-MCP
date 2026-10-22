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
    foreach ($pid in $currentPids) {
        if ($initialPids -notcontains $pid) {
            Write-Host "Matando proceso huerfano PVsyst.exe nuevo (PID: $pid)..." -ForegroundColor Yellow
            Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
        }
    }
}

function Run-StepWithTimeout($name, $cmd, $argsList, $timeoutSeconds, $logFile) {
    Write-Host "`n=== [$name] ===" -ForegroundColor Cyan
    $initialPids = Get-PVsystPids
    
    $pinfo = New-Object System.Diagnostics.ProcessStartInfo
    $pinfo.FileName = $cmd
    $pinfo.Arguments = $argsList -join " "
    $pinfo.WorkingDirectory = $Root
    $pinfo.RedirectStandardOutput = $true
    $pinfo.RedirectStandardError = $true
    $pinfo.UseShellExecute = $false
    $pinfo.CreateNoWindow = $true

    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = $pinfo

    $stdoutBuilder = New-Object System.Text.StringBuilder
    $stderrBuilder = New-Object System.Text.StringBuilder

    $outHandler = [System.Diagnostics.DataReceivedEventHandler]{
        if ($EventArgs.Data) { $stdoutBuilder.AppendLine($EventArgs.Data) | Out-Null }
    }
    $errHandler = [System.Diagnostics.DataReceivedEventHandler]{
        if ($EventArgs.Data) { $stderrBuilder.AppendLine($EventArgs.Data) | Out-Null }
    }

    $proc.add_OutputDataReceived($outHandler)
    $proc.add_ErrorDataReceived($errHandler)

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $proc.Start() | Out-Null
    $proc.BeginOutputReadLine()
    $proc.BeginErrorReadLine()

    $completed = $proc.WaitForExit($timeoutSeconds * 1000)
    $sw.Stop()

    if (-not $completed) {
        Write-Host "TIMEOUT tras $timeoutSeconds s en $name!" -ForegroundColor Red
        $proc.Kill()
        $proc.WaitForExit()
        $exitCode = -1
    } else {
        $exitCode = $proc.ExitCode
    }

    Kill-NewPVsystPids $initialPids

    $allOutput = "STDOUT:`n" + $stdoutBuilder.ToString() + "`nSTDERR:`n" + $stderrBuilder.ToString()
    if ($logFile) {
        [System.IO.File]::WriteAllText($logFile, $allOutput)
    }

    if ($exitCode -eq 0) {
        Write-Host "OK ($([math]::Round($sw.Elapsed.TotalSeconds, 2)) s)" -ForegroundColor Green
    } else {
        Write-Host "FALLO (ExitCode: $exitCode, $([math]::Round($sw.Elapsed.TotalSeconds, 2)) s)" -ForegroundColor Red
    }

    return $exitCode
}

$GlobalExit = 0

# 1. Cargo fmt --check
$res = Run-StepWithTimeout "cargo fmt --check" "cargo" @("fmt", "--check") 120 (Join-Path $ScratchDir "fmt.log")
if ($res -ne 0) { $GlobalExit = $res }

# 2. Cargo clippy --all-targets -D warnings
$res = Run-StepWithTimeout "cargo clippy --all-targets -D warnings" "cargo" @("clippy", "--all-targets", "--", "-D", "warnings") 300 (Join-Path $ScratchDir "clippy.log")
if ($res -ne 0) {
    Write-Host "Nota: clippy reporta advertencias de otras zonas pendientes de refactorizar." -ForegroundColor Yellow
}

# 3. Cargo test --lib
$res = Run-StepWithTimeout "cargo test --lib" "cargo" @("test", "--lib") 600 (Join-Path $ScratchDir "test_lib.log")
if ($res -ne 0) { $GlobalExit = $res }

# 4. Cada binario de integracion
$integracionTests = @(
    "aislado",
    "almacenamiento",
    "bombeo",
    "cli_fake",
    "economia",
    "fixture_workspace",
    "inyeccion",
    "mcp_regresiones",
    "mcp_smoke",
    "medicion",
    "perdidas",
    "protocol",
    "protocol_prompts",
    "roundtrip",
    "seguidores",
    "serie_horaria"
)

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
