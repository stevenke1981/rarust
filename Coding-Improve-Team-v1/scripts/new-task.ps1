param(
    [Parameter(Mandatory = $true)]
    [string]$Name
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$SafeName = $Name -replace '[^a-zA-Z0-9._-]', '-'
$TaskDir = Join-Path $Root "worklog\tasks\$SafeName"
$TemplateDir = Join-Path $Root "templates"

New-Item -ItemType Directory -Force -Path $TaskDir | Out-Null

foreach ($file in @("spec.md", "plan.md", "todos.md", "test.md", "final.md")) {
    $src = Join-Path $TemplateDir $file
    $dst = Join-Path $TaskDir $file
    if (-not (Test-Path $dst)) {
        Copy-Item $src $dst
    }
}

Write-Host "Created task workspace: $TaskDir"
