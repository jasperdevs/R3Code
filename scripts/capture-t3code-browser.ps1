param(
  [string]$T3CodeRepo = "$env:TEMP\t3code-inspect",
  [string]$T3CodeHome = "$env:TEMP\t3code-reference-home",
  [string]$OutputDir = "reference\screenshots",
  [int]$StartupTimeoutSeconds = 90
)

$ErrorActionPreference = "Stop"

function Stop-T3CodeReferenceProcesses {
  param([string]$RepoPath)

  $processes = Get-CimInstance Win32_Process | Where-Object {
    $_.CommandLine -and $_.CommandLine.Contains($RepoPath)
  }

  foreach ($process in $processes) {
    if ($process.ProcessId -ne $PID) {
      Stop-Process -Id $process.ProcessId -Force -ErrorAction SilentlyContinue
    }
  }
}

if (!(Test-Path $T3CodeRepo)) {
  git clone --depth=1 https://github.com/pingdotgg/t3code.git $T3CodeRepo
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$resolvedRepo = Resolve-Path $T3CodeRepo
$resolvedHome = New-Item -ItemType Directory -Force -Path $T3CodeHome
$resolvedOutput = Join-Path $repoRoot $OutputDir
$stdoutPath = Join-Path $env:TEMP "t3code-reference.out.log"
$stderrPath = Join-Path $env:TEMP "t3code-reference.err.log"
New-Item -ItemType Directory -Force -Path $resolvedOutput | Out-Null
Remove-Item -LiteralPath $stdoutPath, $stderrPath -Force -ErrorAction SilentlyContinue

$commit = git -C $resolvedRepo rev-parse HEAD

try {
  Stop-T3CodeReferenceProcesses -RepoPath $resolvedRepo

  $oldT3CodeHome = $env:T3CODE_HOME
  $oldT3CodeNoBrowser = $env:T3CODE_NO_BROWSER
  $env:T3CODE_HOME = $resolvedHome.FullName
  $env:T3CODE_NO_BROWSER = "1"
  $process = Start-Process `
    -FilePath "bun" `
    -ArgumentList @("run", "dev", "--no-browser") `
    -WorkingDirectory $resolvedRepo.Path `
    -RedirectStandardOutput $stdoutPath `
    -RedirectStandardError $stderrPath `
    -PassThru `
    -WindowStyle Hidden

  $deadline = (Get-Date).AddSeconds($StartupTimeoutSeconds)
  $pairingUrl = $null
  while ((Get-Date) -lt $deadline) {
    if ($process.HasExited) {
      $stdout = if (Test-Path $stdoutPath) { Get-Content $stdoutPath -Raw } else { "" }
      $stderr = if (Test-Path $stderrPath) { Get-Content $stderrPath -Raw } else { "" }
      throw "T3Code dev process exited before a pairing URL was available. Exit=$($process.ExitCode)`nSTDOUT:`n$stdout`nSTDERR:`n$stderr"
    }

    if (Test-Path $stdoutPath) {
      $match = Select-String -Path $stdoutPath -Pattern "pairingUrl: (http://[^ ]+)" | Select-Object -Last 1
      if ($match) {
        $pairingUrl = $match.Matches.Groups[1].Value
        break
      }
    }

    Start-Sleep -Milliseconds 500
  }

  if (!$pairingUrl) {
    throw "Timed out waiting for T3Code pairing URL."
  }

  $playwrightPath = Join-Path $resolvedRepo "node_modules\.bun\node_modules\playwright"
  if (!(Test-Path $playwrightPath)) {
    throw "Playwright was not found at $playwrightPath. Run bun install in $resolvedRepo."
  }

  $captureScript = @"
const { chromium } = require(process.env.PLAYWRIGHT_PATH);
const path = require("path");

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 });
  await page.goto(process.env.PAIRING_URL, { waitUntil: "networkidle", timeout: 30000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "t3code-empty-reference.png"), fullPage: true });
  await page.goto("http://localhost:5733/settings", { waitUntil: "networkidle", timeout: 30000 });
  await page.screenshot({ path: path.join(process.env.OUTPUT_DIR, "t3code-settings-reference.png"), fullPage: true });
  await browser.close();
})().catch((error) => {
  console.error(error);
  process.exit(1);
});
"@

  $captureScriptPath = Join-Path $env:TEMP "capture-t3code-browser.cjs"
  Set-Content -Encoding UTF8 -Path $captureScriptPath -Value $captureScript
  $env:PLAYWRIGHT_PATH = $playwrightPath
  $env:PAIRING_URL = $pairingUrl
  $env:OUTPUT_DIR = $resolvedOutput
  node $captureScriptPath

  @"
T3Code reference repository: $resolvedRepo
Reference commit: $commit
Isolated T3CODE_HOME: $($resolvedHome.FullName)
Output directory: $resolvedOutput
Captured:
- t3code-empty-reference.png
- t3code-settings-reference.png
"@ | Set-Content -Encoding UTF8 (Join-Path $resolvedOutput "CAPTURE_MANIFEST.txt")

  Write-Host "Captured T3Code reference screenshots from $commit"
} finally {
  if (Get-Variable -Name oldT3CodeHome -ErrorAction SilentlyContinue) {
    $env:T3CODE_HOME = $oldT3CodeHome
  }
  if (Get-Variable -Name oldT3CodeNoBrowser -ErrorAction SilentlyContinue) {
    $env:T3CODE_NO_BROWSER = $oldT3CodeNoBrowser
  }
  if ($process -and !$process.HasExited) {
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
  }
  Stop-T3CodeReferenceProcesses -RepoPath $resolvedRepo
}
