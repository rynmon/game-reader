#Requires -Version 5.1
<#
.SYNOPSIS
  Bundle Tesseract OCR for the Game Reader installer.

.DESCRIPTION
  Prefers an existing Chocolatey install on CI/dev machines, otherwise downloads
  the UB Mannheim Windows installer.
#>
param(
    [string]$Version = "5.5.0.20241111"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Dest = Join-Path $Root "assets/tesseract"
New-Item -ItemType Directory -Force -Path $Dest | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $Dest "tessdata") | Out-Null

function Copy-BundledTesseract {
    param([string]$SourceDir)
    $TessExe = Join-Path $SourceDir "tesseract.exe"
    $EngData = Join-Path $SourceDir "tessdata/eng.traineddata"
    if (-not (Test-Path $TessExe)) {
        throw "tesseract.exe not found at $TessExe"
    }
    if (-not (Test-Path $EngData)) {
        throw "eng.traineddata not found at $EngData"
    }
    Copy-Item $TessExe (Join-Path $Dest "tesseract.exe") -Force
    Copy-Item $EngData (Join-Path $Dest "tessdata/eng.traineddata") -Force
    Write-Host "==> Tesseract bundled to assets/tesseract/ from $SourceDir"
}

$DefaultInstall = "C:\Program Files\Tesseract-OCR"
if (Test-Path (Join-Path $DefaultInstall "tesseract.exe")) {
    Copy-BundledTesseract -SourceDir $DefaultInstall
    exit 0
}

if (Get-Command choco -ErrorAction SilentlyContinue) {
    Write-Host "==> Installing Tesseract via Chocolatey..."
    choco install tesseract -y --no-progress
    if (Test-Path (Join-Path $DefaultInstall "tesseract.exe")) {
        Copy-BundledTesseract -SourceDir $DefaultInstall
        exit 0
    }
}

$Urls = @(
    "https://digi.bib.uni-mannheim.de/tesseract/tesseract-ocr-w64-setup-$Version.exe",
    "https://github.com/UB-Mannheim/tesseract/releases/download/v$Version/tesseract-ocr-w64-setup-$Version.exe"
)

$Installer = Join-Path $env:TEMP "tesseract-setup.exe"
$SevenZip = "${env:ProgramFiles}\7-Zip\7z.exe"
if (-not (Test-Path $SevenZip)) {
    throw "7-Zip not found at $SevenZip. Install 7-Zip or Tesseract manually."
}

$Downloaded = $false
foreach ($Url in $Urls) {
    Write-Host "==> Trying download: $Url"
    try {
        Invoke-WebRequest -Uri $Url -OutFile $Installer -UseBasicParsing
        $Downloaded = $true
        break
    } catch {
        Write-Warning "Download failed: $_"
    }
}

if (-not $Downloaded) {
    throw "Could not download Tesseract installer from any known URL."
}

$ExtractDir = Join-Path $env:TEMP "tesseract-extract"
Remove-Item -Recurse -Force $ExtractDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $ExtractDir | Out-Null
& $SevenZip x $Installer "-o$ExtractDir" -y | Out-Null

$TessExe = Get-ChildItem -Path $ExtractDir -Recurse -Filter "tesseract.exe" | Select-Object -First 1
$EngData = Get-ChildItem -Path $ExtractDir -Recurse -Filter "eng.traineddata" | Select-Object -First 1
if (-not $TessExe -or -not $EngData) {
    throw "Could not find tesseract.exe or eng.traineddata in extracted installer."
}

Copy-Item $TessExe.FullName (Join-Path $Dest "tesseract.exe") -Force
Copy-Item $EngData.FullName (Join-Path $Dest "tessdata/eng.traineddata") -Force
Write-Host "==> Tesseract bundled to assets/tesseract/"
