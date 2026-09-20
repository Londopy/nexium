# Build `nx` from source with no Rust (see build.sh for the stages). Run from
# the repository root in PowerShell:
#
#     .\bootstrap\build.ps1            # -> nx-out\bootstrap\nx2.exe
#
# $env:CC overrides the C compiler (default: zig cc).
$ErrorActionPreference = "Stop"
$out = "nx-out\bootstrap"
New-Item -ItemType Directory -Force $out | Out-Null
$cc = if ($env:CC) { $env:CC } else { "zig cc" }
$ccExe, $ccArgs = $cc.Split(" ", 2)
function Invoke-CC { param([string[]]$rest) & $ccExe @($ccArgs -split " " | Where-Object { $_ }) @rest; if ($LASTEXITCODE -ne 0) { throw "C compilation failed" } }
Write-Host "stage 0: $cc builds nx0 from bootstrap\nx.c"
Invoke-CC @("-std=gnu11", "-O2", "-w", "-fno-strict-aliasing", "-o", "$out\nx0.exe", "bootstrap\nx.c", "-lws2_32")
Write-Host "stage 1: nx0 builds self\nx.nx"
& "$out\nx0.exe" build self/nx.nx --mode safe -o "$out\nx1.exe" --out-dir $out; if ($LASTEXITCODE -ne 0) { throw "nx0 could not build self/nx.nx" }
Write-Host "stage 2: nx1 emits itself, $cc builds nx2, nx2 emits itself"
& "$out\nx1.exe" emit-c self/nx.nx --mode safe | Set-Content -NoNewline -Encoding utf8 "$out\nx1.c"
Invoke-CC @("-std=gnu11", "-O2", "-w", "-fno-strict-aliasing", "-o", "$out\nx2.exe", "$out\nx1.c", "-lws2_32")
& "$out\nx2.exe" emit-c self/nx.nx --mode safe | Set-Content -NoNewline -Encoding utf8 "$out\nx2.c"
if ((Get-FileHash "$out\nx1.c").Hash -ne (Get-FileHash "$out\nx2.c").Hash) { throw "nx1 and nx2 emit different C" }
Write-Host "fixed point: nx1 and nx2 emit the same C"
if ((Get-FileHash "bootstrap\nx.c").Hash -ne (Get-FileHash "$out\nx1.c").Hash) { Write-Host "note: bootstrap\nx.c is behind self\; at the release: copy $out\nx1.c bootstrap\nx.c" }
# nx1 was linked against the runtime the seed carries; nx2 against runtime\nx_rt.h
Write-Host "built $out\nx2.exe"
