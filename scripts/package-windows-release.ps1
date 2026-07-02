#Requires -Version 5.1
param(
    [string]$Tag = "v2.0.0"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$ReleaseName = "Game-Reader-$Tag-windows-x64"
$Stage = Join-Path $Root "release-stage/$ReleaseName"
$Resources = Join-Path $Stage "resources"
$Binaries = Join-Path $Resources "binaries"

Remove-Item -Recurse -Force (Join-Path $Root "release-stage") -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Resources, $Binaries | Out-Null

$MainExe = Join-Path $Root "src-tauri/target/release/game-reader.exe"
if (-not (Test-Path $MainExe)) {
    throw "Missing $MainExe — run npm run tauri build -- --no-bundle first"
}
Copy-Item $MainExe $Stage

function Copy-Sidecar {
    param([string]$BaseName)
    $Triple = "x86_64-pc-windows-msvc"
    $candidates = @(
        (Join-Path $Root "src-tauri/target/release/$BaseName-$Triple.exe"),
        (Join-Path $Root "src-tauri/binaries/$BaseName-$Triple.exe"),
        (Join-Path $Root "dist/$BaseName.exe")
    )
    foreach ($src in $candidates) {
        if (Test-Path $src) {
            $plain = "$BaseName.exe"
            Copy-Item $src (Join-Path $Stage $plain)
            Copy-Item $src (Join-Path $Binaries $plain)
            $sizeGb = [math]::Round((Get-Item $src).Length / 1GB, 2)
            Write-Host "==> $plain ($sizeGb GB) from $src"
            return
        }
    }
    throw "Sidecar not found: $BaseName"
}

Copy-Sidecar "game-reader-engine"
Copy-Sidecar "rvc-worker"

if (Test-Path (Join-Path $Root "assets/voices.json")) {
    Copy-Item (Join-Path $Root "assets/voices.json") $Resources
}
if (Test-Path (Join-Path $Root "assets/tesseract")) {
    Copy-Item -Recurse (Join-Path $Root "assets/tesseract") (Join-Path $Resources "tesseract")
}

$ArchiveBase = Join-Path $Root "$ReleaseName.7z"
if (Get-Command 7z -ErrorAction SilentlyContinue) {
    Remove-Item "$ArchiveBase.*" -Force -ErrorAction SilentlyContinue
    # Split into <2 GB parts for GitHub Releases.
    & 7z a -t7z -mx=5 -v1900m $ArchiveBase (Join-Path $Root "release-stage/$ReleaseName")
    if ($LASTEXITCODE -ne 0) { throw "7z failed with exit code $LASTEXITCODE" }
    Get-ChildItem "$ArchiveBase.*" | ForEach-Object {
        $sizeGb = [math]::Round($_.Length / 1GB, 2)
        Write-Host "==> Archive part: $($_.Name) ($sizeGb GB)"
    }
} else {
    throw "7z not found on PATH"
}

Write-Host "==> Release package ready: $ArchiveBase.*"
