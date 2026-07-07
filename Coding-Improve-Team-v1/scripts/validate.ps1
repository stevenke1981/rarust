param(
    [switch]$Strict
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Python = Get-Command python -ErrorAction SilentlyContinue

if (-not $Python) {
    $Python = Get-Command py -ErrorAction SilentlyContinue
}

if (-not $Python) {
    Write-Error "Python is required to run scripts/validate.py"
    exit 1
}

Push-Location $Root
try {
    if ($Python.Name -eq "py.exe" -or $Python.Name -eq "py") {
        py scripts\validate.py
    } else {
        python scripts\validate.py
    }
} finally {
    Pop-Location
}
