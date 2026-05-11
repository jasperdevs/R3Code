param(
  [string]$ExePath = "target\debug\r3code.exe",
  [string]$OutputPath = "reference\screenshots\r3code-window.png",
  [string]$Screen = "",
  [int]$StartupDelaySeconds = 6
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$resolvedExe = Resolve-Path (Join-Path $repoRoot $ExePath)
$resolvedOutput = Join-Path $repoRoot $OutputPath
$outputDir = Split-Path -Parent $resolvedOutput
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

if ($Screen.Trim()) {
  $oldScreen = $env:R3CODE_SCREEN
  $env:R3CODE_SCREEN = $Screen.Trim()
}

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
public struct POINT { public int X; public int Y; }
public class Win32Capture {
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hWnd, out RECT rect);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr hWnd, ref POINT point);
}
'@
  }

  $rect = New-Object RECT
  [Win32Capture]::GetClientRect($hwnd, [ref]$rect) | Out-Null
  $point = New-Object POINT
  $point.X = 0
  $point.Y = 0
  [Win32Capture]::ClientToScreen($hwnd, [ref]$point) | Out-Null
  $width = $rect.Right - $rect.Left
  $height = $rect.Bottom - $rect.Top
  if ($width -le 0 -or $height -le 0) {
    throw "Invalid R3Code client bounds ${width}x${height}."
  }

  $bitmap = New-Object System.Drawing.Bitmap $width, $height
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  try {
    $graphics.CopyFromScreen($point.X, $point.Y, 0, 0, $bitmap.Size)
    $bitmap.Save($resolvedOutput, [System.Drawing.Imaging.ImageFormat]::Png)
  } finally {
    $graphics.Dispose()
    $bitmap.Dispose()
  }

  Write-Host $resolvedOutput
} finally {
  Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
  if (Get-Variable -Name oldScreen -ErrorAction SilentlyContinue) {
    if ($oldScreen) {
      $env:R3CODE_SCREEN = $oldScreen
    } else {
      Remove-Item Env:R3CODE_SCREEN -ErrorAction SilentlyContinue
    }
  }
}
