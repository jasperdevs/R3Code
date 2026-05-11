param(
  [string]$ExePath = "target\debug\r3code.exe",
  [string]$OutputPath = "reference\screenshots\r3code-window.png",
  [int]$StartupDelaySeconds = 6
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$resolvedExe = Resolve-Path (Join-Path $repoRoot $ExePath)
$resolvedOutput = Join-Path $repoRoot $OutputPath
$outputDir = Split-Path -Parent $resolvedOutput
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

$process = Start-Process -FilePath $resolvedExe -PassThru
try {
  Start-Sleep -Seconds $StartupDelaySeconds
  $liveProcess = Get-Process -Id $process.Id -ErrorAction Stop
  $hwnd = $liveProcess.MainWindowHandle
  if ($hwnd -eq 0) {
    throw "R3Code did not expose a main window handle."
  }

  Add-Type -AssemblyName System.Drawing
  if (-not ([System.Management.Automation.PSTypeName]"Win32Capture").Type) {
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
public class Win32Capture { [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect); }
'@
  }

  $rect = New-Object RECT
  [Win32Capture]::GetWindowRect($hwnd, [ref]$rect) | Out-Null
  $width = $rect.Right - $rect.Left
  $height = $rect.Bottom - $rect.Top
  if ($width -le 0 -or $height -le 0) {
    throw "Invalid R3Code window bounds ${width}x${height}."
  }

  $bitmap = New-Object System.Drawing.Bitmap $width, $height
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  try {
    $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
    $bitmap.Save($resolvedOutput, [System.Drawing.Imaging.ImageFormat]::Png)
  } finally {
    $graphics.Dispose()
    $bitmap.Dispose()
  }

  Write-Host $resolvedOutput
} finally {
  Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
}
