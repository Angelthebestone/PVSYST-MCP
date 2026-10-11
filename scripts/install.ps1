# Build the MCP server (release) and install it to a SPACE-FREE path.
#
# Why: Command Code's MCP launcher does not quote the executable path, so a
# command containing spaces (e.g. "...\PVSYST MCP\...") fails to spawn. We
# therefore install the binary to C:\pvsyst-mcp\pvsyst-mcp.exe and point the
# MCP config at that stable location.

$ErrorActionPreference = "Stop"

$project = Split-Path -Parent $PSScriptRoot
$installDir = "C:\pvsyst-mcp"
$installExe = Join-Path $installDir "pvsyst-mcp.exe"

$mingw = "C:\Users\Lenovo\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin"
if (Test-Path $mingw) { $env:Path = "$env:USERPROFILE\.cargo\bin;$mingw;$env:Path" }

Write-Host "Building release..."
Push-Location $project
cargo build --release
Pop-Location

New-Item -ItemType Directory -Force -Path $installDir | Out-Null
$src = Join-Path $project "target\release\pvsyst-mcp.exe"

function Copy-WithUnlock {
    param($Src, $Dest)
    try { Copy-Item $Src $Dest -Force -ErrorAction Stop; return "copied" } catch {}
    # A running MCP server holds the .exe image; Windows allows renaming it aside.
    try {
        $bak = "$Dest.old-" + (Get-Date -Format "yyyyMMddHHmmss") + ".exe"
        Rename-Item $Dest $bak -ErrorAction Stop
        Copy-Item $Src $Dest -Force -ErrorAction Stop
        return "renamed running exe -> $bak, then copied"
    } catch {}
    # Last resort: stop the running server(s).
    $procs = Get-Process -Name "pvsyst-mcp" -ErrorAction SilentlyContinue
    if ($procs) { $procs | Stop-Process -Force; Start-Sleep -Milliseconds 800 }
    Copy-Item $Src $Dest -Force -ErrorAction Stop
    return "stopped running server(s), then copied"
}

$how = Copy-WithUnlock $src $installExe

Write-Host "Installed ($how): $installExe"
Write-Host "Register with:  cmdc mcp add --scope user --env PVSYST_WORKSPACE=C:\Users\Lenovo\PVsyst8.1_Data pvsyst -- `"$installExe`""
