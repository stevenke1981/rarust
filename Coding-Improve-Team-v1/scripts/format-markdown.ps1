# Lightweight markdown cleanup: remove trailing whitespace.
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

Get-ChildItem -Path $Root -Filter *.md -Recurse | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    $lines = $content -split "`r?`n"
    $clean = ($lines | ForEach-Object { $_.TrimEnd() }) -join "`n"
    if (-not $clean.EndsWith("`n")) {
        $clean += "`n"
    }
    Set-Content -Path $_.FullName -Value $clean -NoNewline -Encoding UTF8
}

Write-Host "Markdown cleanup complete."
