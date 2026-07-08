param(
    [switch]$SkipBuild,
    [switch]$CliOnly
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $PSScriptRoot
$ReleaseDir = Join-Path $RepoRoot "target\release"
$CliExe = Join-Path $ReleaseDir "rarust.exe"
$GuiExe = Join-Path $ReleaseDir "rarust-gui.exe"

function Get-CargoPath {
    $cmd = Get-Command cargo -ErrorAction SilentlyContinue
    if ($cmd) {
        return $cmd.Source
    }

    $fallback = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (Test-Path -LiteralPath $fallback) {
        return $fallback
    }

    throw "cargo.exe was not found on PATH or at $fallback"
}

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,
        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$Arguments
    )

    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code ${LASTEXITCODE}: $FilePath $($Arguments -join ' ')"
    }
}

$cargo = Get-CargoPath
Write-Host "Repo: $RepoRoot"
Write-Host "Cargo: $cargo"

if (-not $SkipBuild) {
    Push-Location $RepoRoot
    try {
        Invoke-Checked $cargo build --release --workspace
    }
    finally {
        Pop-Location
    }
}

if (-not (Test-Path -LiteralPath $CliExe)) {
    throw "Missing release CLI binary: $CliExe"
}

if (-not $CliOnly -and -not (Test-Path -LiteralPath $GuiExe)) {
    throw "Missing release GUI binary: $GuiExe"
}

Invoke-Checked $CliExe --version

Write-Host "Release artifacts:"
Get-Item -LiteralPath $CliExe | Select-Object FullName, Length, LastWriteTime
if (-not $CliOnly) {
    Get-Item -LiteralPath $GuiExe | Select-Object FullName, Length, LastWriteTime
}
