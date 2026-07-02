#Requires -Version 5.1
<#
.SYNOPSIS
  Build the Game Reader Python engine sidecars with PyInstaller.

.DESCRIPTION
  Creates:
    dist/game-reader-engine.exe  — main JSON-RPC sidecar
    dist/rvc-worker.exe          — RVC worker fallback
  Copies to src-tauri/binaries/ with Tauri target-triple suffix.
#>
param(
    [string]$Python = "python",
    [switch]$SkipTorch
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

Write-Host "==> Installing engine dependencies..."
& $Python -m pip install --upgrade pip
& $Python -m pip install -r engine/requirements.txt pyinstaller

if (-not $SkipTorch) {
    Write-Host "==> Installing PyTorch CUDA (this may take a while)..."
    try {
        & $Python -m pip install torch==2.11.0+cu128 torchvision==0.26.0+cu128 torchaudio==2.11.0+cu128 `
            --index-url https://download.pytorch.org/whl/cu128
    } catch {
        Write-Warning "cu128 wheels unavailable, falling back to cu124..."
        & $Python -m pip install torch torchvision torchaudio `
            --index-url https://download.pytorch.org/whl/cu124
    }
}

Write-Host "==> Verifying rvc-python on current Python..."
& $Python -c "from rvc_python.infer import RVCInference; print('rvc-python OK')"

Write-Host "==> Building PyInstaller bundles..."
& $Python -m PyInstaller --noconfirm --clean engine/game-reader-engine.spec

$BinDir = Join-Path $Root "src-tauri/binaries"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

$Triple = "x86_64-pc-windows-msvc"
Copy-Item (Join-Path $Root "dist/game-reader-engine.exe") (Join-Path $BinDir "game-reader-engine-$Triple.exe") -Force
Copy-Item (Join-Path $Root "dist/rvc-worker.exe") (Join-Path $BinDir "rvc-worker-$Triple.exe") -Force

Write-Host "==> Done."
Write-Host "    Engine: src-tauri/binaries/game-reader-engine-$Triple.exe"
Write-Host "    Worker: src-tauri/binaries/rvc-worker-$Triple.exe"
