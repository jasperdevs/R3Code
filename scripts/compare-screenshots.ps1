param(
  [Parameter(Mandatory = $true)]
  [string]$Expected,
  [Parameter(Mandatory = $true)]
  [string]$Actual,
  [double]$MaxDifferentPixelsPercent = 1.0
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

$expectedPath = Resolve-Path $Expected
$actualPath = Resolve-Path $Actual
$expectedImage = [System.Drawing.Bitmap]::FromFile($expectedPath)
$actualImage = [System.Drawing.Bitmap]::FromFile($actualPath)

try {
  if ($expectedImage.Width -ne $actualImage.Width -or $expectedImage.Height -ne $actualImage.Height) {
    throw "Image dimensions differ. Expected $($expectedImage.Width)x$($expectedImage.Height), actual $($actualImage.Width)x$($actualImage.Height)."
  }

  $differentPixels = 0
  $totalPixels = $expectedImage.Width * $expectedImage.Height

  for ($y = 0; $y -lt $expectedImage.Height; $y++) {
    for ($x = 0; $x -lt $expectedImage.Width; $x++) {
      if ($expectedImage.GetPixel($x, $y).ToArgb() -ne $actualImage.GetPixel($x, $y).ToArgb()) {
        $differentPixels++
      }
    }
  }

  $differentPercent = ($differentPixels / $totalPixels) * 100
  $summary = "Different pixels: {0:N0}/{1:N0} ({2:N3}%). Limit: {3:N3}%." -f $differentPixels, $totalPixels, $differentPercent, $MaxDifferentPixelsPercent
  Write-Host $summary

  if ($differentPercent -gt $MaxDifferentPixelsPercent) {
    throw "Screenshot comparison failed. $summary"
  }
} finally {
  $expectedImage.Dispose()
  $actualImage.Dispose()
}
