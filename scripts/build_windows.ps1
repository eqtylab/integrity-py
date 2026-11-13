# Builds the eqty_sdk wheel natively on Windows (the Justfile needs a POSIX shell).
#
# Needs Python 3.10+, Rust (rustup) and the Visual Studio Build Tools with the C++ workload.
#   powershell -ExecutionPolicy Bypass -File scripts\build_windows.ps1 [-Test]
# -Test installs the wheel into .venv and runs the unit tests and example manifest checks.
param([switch]$Test)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

if (-not (Test-Path .venv)) { python -m venv .venv }
$py = ".venv\Scripts\python.exe"
& $py -m pip install --quiet --upgrade pip maturin
if ($LASTEXITCODE) { exit $LASTEXITCODE }

& .venv\Scripts\maturin.exe build --release --out dist
if ($LASTEXITCODE) { exit $LASTEXITCODE }
$wheel = Get-ChildItem dist\*win_amd64.whl | Sort-Object LastWriteTime | Select-Object -Last 1
Write-Host "Built $($wheel.FullName)"

if ($Test) {
    $env:VIRTUAL_ENV = (Resolve-Path .venv).Path
    & $py -m pip install --quiet base58 model-signing==1.1.1
    & .venv\Scripts\maturin.exe develop --release
    if ($LASTEXITCODE) { exit $LASTEXITCODE }
    & $py -m unittest discover tests
    if ($LASTEXITCODE) { exit $LASTEXITCODE }
    $env:EQTY_SKIP_PROOF = "true"
    & $py integration-tests\_example_manifests_test.py
    exit $LASTEXITCODE
}
