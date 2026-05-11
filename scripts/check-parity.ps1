param(
  [switch]$RefreshT3CodeReference
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $repoRoot
try {
  cargo fmt --all -- --check
  cargo check --workspace
  cargo build -p r3_app

  if ($RefreshT3CodeReference) {
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts\capture-t3code-browser.ps1
  }

  powershell -NoProfile -ExecutionPolicy Bypass -File scripts\capture-r3code-window.ps1 `
    -Theme light `
    -OutputPath reference\screenshots\r3code-window.png
  powershell -NoProfile -ExecutionPolicy Bypass -File scripts\compare-screenshots.ps1 `
    -Expected reference\screenshots\t3code-empty-reference.png `
    -Actual reference\screenshots\r3code-window.png `
    -ChannelTolerance 8 `
    -IgnoreRect 0,0,120,45 `
    -MaxDifferentPixelsPercent 2

  powershell -NoProfile -ExecutionPolicy Bypass -File scripts\capture-r3code-window.ps1 `
    -Theme light `
    -Screen settings `
    -OutputPath reference\screenshots\r3code-settings-window.png
  powershell -NoProfile -ExecutionPolicy Bypass -File scripts\compare-screenshots.ps1 `
    -Expected reference\screenshots\t3code-settings-reference.png `
    -Actual reference\screenshots\r3code-settings-window.png `
    -ChannelTolerance 8 `
    -IgnoreRect 0,0,120,45 `
    -MaxDifferentPixelsPercent 6

  powershell -NoProfile -ExecutionPolicy Bypass -File scripts\capture-r3code-window.ps1 `
    -Theme dark `
    -OutputPath reference\screenshots\r3code-dark-window.png

  Write-Host "R3Code parity checks passed."
} finally {
  Pop-Location
}
