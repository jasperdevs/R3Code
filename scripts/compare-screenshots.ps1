param(
  [Parameter(Mandatory = $true)]
  [string]$Expected,
  [Parameter(Mandatory = $true)]
  [string]$Actual,
  [double]$MaxDifferentPixelsPercent = 1.0,
  [int]$ChannelTolerance = 0,
  [string[]]$IgnoreRect = @()
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$expectedPath = Resolve-Path $Expected
$actualPath = Resolve-Path $Actual
$expectedImage = [System.Drawing.Bitmap]::FromFile($expectedPath)
$actualImage = [System.Drawing.Bitmap]::FromFile($actualPath)

function ConvertTo-Rect {
  param([string]$Value)

  $parts = $Value.Split(",") | ForEach-Object { [int]$_.Trim() }
  if ($parts.Count -ne 4) {
    throw "Invalid ignore rectangle '$Value'. Expected x,y,width,height."
  }

  [PSCustomObject]@{
    X = $parts[0]
    Y = $parts[1]
    Width = $parts[2]
    Height = $parts[3]
  }
}

function Test-InIgnoredRect {
  param(
    [int]$X,
    [int]$Y,
    [object[]]$Rectangles
  )

  foreach ($rect in $Rectangles) {
    if (
      $X -ge $rect.X -and
      $X -lt ($rect.X + $rect.Width) -and
      $Y -ge $rect.Y -and
      $Y -lt ($rect.Y + $rect.Height)
    ) {
      return $true
    }
  }

  return $false
}

function Test-PixelDifferent {
  param(
    [System.Drawing.Color]$ExpectedPixel,
    [System.Drawing.Color]$ActualPixel,
    [int]$Tolerance
  )

  return (
    [Math]::Abs($ExpectedPixel.A - $ActualPixel.A) -gt $Tolerance -or
    [Math]::Abs($ExpectedPixel.R - $ActualPixel.R) -gt $Tolerance -or
    [Math]::Abs($ExpectedPixel.G - $ActualPixel.G) -gt $Tolerance -or
    [Math]::Abs($ExpectedPixel.B - $ActualPixel.B) -gt $Tolerance
  )
}

try {
  if ($expectedImage.Width -ne $actualImage.Width -or $expectedImage.Height -ne $actualImage.Height) {
    throw "Image dimensions differ. Expected $($expectedImage.Width)x$($expectedImage.Height), actual $($actualImage.Width)x$($actualImage.Height)."
  }

  $differentPixels = 0
  $totalPixels = $expectedImage.Width * $expectedImage.Height
  $ignoredPixels = 0
  $ignoreRectangles = @($IgnoreRect | ForEach-Object { ConvertTo-Rect $_ })

  for ($y = 0; $y -lt $expectedImage.Height; $y++) {
    for ($x = 0; $x -lt $expectedImage.Width; $x++) {
      if (Test-InIgnoredRect -X $x -Y $y -Rectangles $ignoreRectangles) {
        $ignoredPixels++
        continue
      }

      $expectedPixel = $expectedImage.GetPixel($x, $y)
      $actualPixel = $actualImage.GetPixel($x, $y)
      if (Test-PixelDifferent -ExpectedPixel $expectedPixel -ActualPixel $actualPixel -Tolerance $ChannelTolerance) {
        $differentPixels++
      }
    }
  }

  $comparedPixels = $totalPixels - $ignoredPixels
  $differentPercent = ($differentPixels / $comparedPixels) * 100
  $summary = "Different pixels: {0:N0}/{1:N0} ({2:N3}%). Ignored: {3:N0}. Channel tolerance: {4}. Limit: {5:N3}%." -f $differentPixels, $comparedPixels, $differentPercent, $ignoredPixels, $ChannelTolerance, $MaxDifferentPixelsPercent
  Write-Host $summary

  if ($differentPercent -gt $MaxDifferentPixelsPercent) {
    throw "Screenshot comparison failed. $summary"
  }
} finally {
  $expectedImage.Dispose()
  $actualImage.Dispose()
}
