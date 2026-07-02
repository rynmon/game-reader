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
$DistPath = Join-Path $Root "dist"
$WorkPath = Join-Path $Root "build/pyinstaller"
New-Item -ItemType Directory -Force -Path $DistPath, $WorkPath | Out-Null

$PyInstallerArgs = @(
    "-m", "PyInstaller", "--noconfirm", "--clean",
    "--distpath", $DistPath,
    "--workpath", $WorkPath,
    "--specpath", (Join-Path $Root "engine")
)

Invoke-Python -Args ($PyInstallerArgs + @("engine/game-reader-engine.spec"))
Invoke-Python -Args ($PyInstallerArgs + @("engine/rvc-worker.spec"))

$EngineExe = Join-Path $DistPath "game-reader-engine.exe"
$WorkerExe = Join-Path $DistPath "rvc-worker.exe"
if (-not (Test-Path $EngineExe)) {
    throw "PyInstaller did not produce $EngineExe"
}
if (-not (Test-Path $WorkerExe)) {
    throw "PyInstaller did not produce $WorkerExe"
}

$BinDir = Join-Path $Root "src-tauri/binaries"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

$Triple = "x86_64-pc-windows-msvc"
Copy-Item $EngineExe (Join-Path $BinDir "game-reader-engine-$Triple.exe") -Force
Copy-Item $WorkerExe (Join-Path $BinDir "rvc-worker-$Triple.exe") -Force

Write-Host "==> Done."
Write-Host "    Engine: src-tauri/binaries/game-reader-engine-$Triple.exe"
Write-Host "    Worker: src-tauri/binaries/rvc-worker-$Triple.exe"
