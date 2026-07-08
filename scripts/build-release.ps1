param(
    [switch]$SkipBuild,
    [switch]$CliOnly,
    [switch]$Package
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

function Invoke-CaptureChecked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,
        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$Arguments
    )

    $output = & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code ${LASTEXITCODE}: $FilePath $($Arguments -join ' ')"
    }
    return $output
}

function New-ReleasePackage {
    param(
        [Parameter(Mandatory = $true)]
        [string]$VersionText
    )

    $version = ($VersionText -split '\s+')[-1]
    if (-not $version) {
        throw "Could not parse version from: $VersionText"
    }

    $packageName = "rarust-v$version-windows-x64"
    $stagingDir = Join-Path $ReleaseDir $packageName
    $zipPath = Join-Path $ReleaseDir "$packageName.zip"
    $hashPath = "$zipPath.sha256"

    if (Test-Path -LiteralPath $stagingDir) {
        Remove-Item -LiteralPath $stagingDir -Recurse -Force
    }
    if (Test-Path -LiteralPath $zipPath) {
        Remove-Item -LiteralPath $zipPath -Force
    }
    if (Test-Path -LiteralPath $hashPath) {
        Remove-Item -LiteralPath $hashPath -Force
    }

    New-Item -ItemType Directory -Path $stagingDir | Out-Null
    Copy-Item -LiteralPath $CliExe -Destination $stagingDir
    if (-not $CliOnly) {
        Copy-Item -LiteralPath $GuiExe -Destination $stagingDir
    }

    Compress-Archive -Path (Join-Path $stagingDir '*') -DestinationPath $zipPath -CompressionLevel Optimal
    $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $zipPath).Hash.ToLowerInvariant()
    "$hash  $(Split-Path -Leaf $zipPath)" | Set-Content -LiteralPath $hashPath -Encoding ascii
    Remove-Item -LiteralPath $stagingDir -Recurse -Force

    Write-Host "Release package:"
    Get-Item -LiteralPath $zipPath | Select-Object FullName, Length, LastWriteTime
    Write-Host "SHA256: $hashPath"
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

$versionText = Invoke-CaptureChecked $CliExe --version
Write-Host $versionText

Write-Host "Release artifacts:"
Get-Item -LiteralPath $CliExe | Select-Object FullName, Length, LastWriteTime
if (-not $CliOnly) {
    Get-Item -LiteralPath $GuiExe | Select-Object FullName, Length, LastWriteTime
}

if ($Package) {
    New-ReleasePackage -VersionText $versionText
}
