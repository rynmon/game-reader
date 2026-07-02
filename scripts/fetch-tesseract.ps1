#Requires -Version 5.1
<#
.SYNOPSIS
  Download and extract Tesseract-OCR portable binaries for bundling.

.DESCRIPTION
  Downloads the UB Mannheim Tesseract Windows installer assets and extracts
  tesseract.exe + eng.traineddata into assets/tesseract/ for Tauri bundling.

  Run once before building the installer on Windows.
#>
param(
    [string]$Version = "5.5.0.20241111"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Dest = Join-Path $Root "assets/tesseract"
New-Item -ItemType Directory -Force -Path $Dest | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $Dest "tessdata") | Out-Null

$Url = "https://digi.bib.uni-mannheim.de/tesseract/tesseract-ocr-w64-setup-$Version.exe"
$Installer = Join-Path $env:TEMP "tesseract-setup.exe"

Write-Host "==> Downloading Tesseract $Version..."
Invoke-WebRequest -Uri $Url -OutFile $Installer

Write-Host "==> Extracting with 7-Zip (install 7-Zip if missing)..."
$SevenZip = "${env:ProgramFiles}\7-Zip\7z.exe"
if (-not (Test-Path $SevenZip)) {
    Write-Error "7-Zip not found at $SevenZip. Install 7-Zip or copy Tesseract manually to assets/tesseract/"
}

$ExtractDir = Join-Path $env:TEMP "tesseract-extract"
Remove-Item -Recurse -Force $ExtractDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $ExtractDir | Out-Null
& $SevenZip x $Installer "-o$ExtractDir" -y | Out-Null

$TessExe = Get-ChildItem -Path $ExtractDir -Recurse -Filter "tesseract.exe" | Select-Object -First 1
$EngData = Get-ChildItem -Path $ExtractDir -Recurse -Filter "eng.traineddata" | Select-Object -First 1

if (-not $TessExe -or -not $EngData) {
    Write-Error "Could not find tesseract.exe or eng.traineddata in extracted installer."
}

Copy-Item $TessExe.FullName (Join-Path $Dest "tesseract.exe") -Force
Copy-Item $EngData.FullName (Join-Path $Dest "tessdata/eng.traineddata") -Force

Write-Host "==> Tesseract bundled to assets/tesseract/"
