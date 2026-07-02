#Requires -Version 5.1
param(
    [string]$Python = "python",
    [switch]$SkipTorch
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

function Invoke-Python {
    param([string[]]$Args)
    & $Python @Args
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed: python $($Args -join ' ')"
    }
}

Write-Host "==> Installing engine dependencies..."
Invoke-Python -Args @("-m", "pip", "install", "--upgrade", "pip<24.1")
Invoke-Python -Args @("-m", "pip", "install", "-r", "engine/requirements.txt")

if (-not $SkipTorch) {
    Write-Host "==> Installing PyTorch CUDA (this may take a while)..."
    try {
        Invoke-Python -Args @(
            "-m", "pip", "install",
            "torch==2.11.0+cu128", "torchvision==0.26.0+cu128", "torchaudio==2.11.0+cu128",
            "--index-url", "https://download.pytorch.org/whl/cu128"
        )
    } catch {
        Write-Warning "cu128 wheels unavailable, falling back to cu124..."
        Invoke-Python -Args @(
            "-m", "pip", "install", "torch", "torchvision", "torchaudio",
            "--index-url", "https://download.pytorch.org/whl/cu124"
        )
    }
}

Write-Host "==> Verifying rvc-python on current Python..."
Invoke-Python -Args @("-c", "from rvc_python.infer import RVCInference; print('rvc-python OK')")

Write-Host "==> Building PyInstaller bundles..."
Invoke-Python -Args @("-m", "PyInstaller", "--noconfirm", "--clean", "engine/game-reader-engine.spec")

$BinDir = Join-Path $Root "src-tauri/binaries"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

$Triple = "x86_64-pc-windows-msvc"
Copy-Item (Join-Path $Root "dist/game-reader-engine.exe") (Join-Path $BinDir "game-reader-engine-$Triple.exe") -Force
Copy-Item (Join-Path $Root "dist/rvc-worker.exe") (Join-Path $BinDir "rvc-worker-$Triple.exe") -Force

Write-Host "==> Done."
Write-Host "    Engine: src-tauri/binaries/game-reader-engine-$Triple.exe"
Write-Host "    Worker: src-tauri/binaries/rvc-worker-$Triple.exe"
