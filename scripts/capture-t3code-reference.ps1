param(
  [string]$T3CodeRepo = "$env:TEMP\t3code-inspect",
  [string]$OutputDir = "reference\screenshots"
)

$ErrorActionPreference = "Stop"

if (!(Test-Path $T3CodeRepo)) {
  git clone --depth=1 https://github.com/pingdotgg/t3code.git $T3CodeRepo
}

$commit = git -C $T3CodeRepo rev-parse HEAD
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$resolvedOutput = Join-Path $repoRoot $OutputDir
New-Item -ItemType Directory -Force -Path $resolvedOutput | Out-Null

@"
T3Code reference repository: $T3CodeRepo
Reference commit: $commit
Output directory: $resolvedOutput

Next manual capture states:
- empty
- sidebar
- active-chat
- running-turn
- pending-approval
- composer-focused
- command-palette
- settings
- terminal-drawer
- diff-panel
"@ | Set-Content -Encoding UTF8 (Join-Path $resolvedOutput "CAPTURE_MANIFEST.txt")

Write-Host "Prepared reference capture manifest for T3Code commit $commit"
