#Requires -Version 5.1
param(
    [string]$Python = "python",
    [switch]$SkipTorch
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

function Invoke-Python {
    param([string[]]$PythonArgs)
    Write-Host ">>> python $($PythonArgs -join ' ')"
    $output = & $Python @PythonArgs 2>&1
    if ($output) { $output | ForEach-Object { Write-Host $_ } }
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed (exit $LASTEXITCODE): python $($PythonArgs -join ' ')"
    }
}

Write-Host "==> Installing engine dependencies..."
Invoke-Python -PythonArgs @("-m", "pip", "install", "--upgrade", "pip<24.1")
Invoke-Python -PythonArgs @("-m", "pip", "install", "-r", "engine/requirements.txt")

if (-not $SkipTorch) {
    Write-Host "==> Installing PyTorch CUDA (this may take a while)..."
    try {
        Invoke-Python -PythonArgs @(
            "-m", "pip", "install",
            "torch==2.11.0+cu128", "torchvision==0.26.0+cu128", "torchaudio==2.11.0+cu128",
            "--index-url", "https://download.pytorch.org/whl/cu128"
        )
    } catch {
        Write-Warning "cu128 wheels unavailable, falling back to cu124..."
        Invoke-Python -PythonArgs @(
            "-m", "pip", "install", "torch", "torchvision", "torchaudio",
            "--index-url", "https://download.pytorch.org/whl/cu124"
        )
    }
}

Write-Host "==> Verifying torch..."
Invoke-Python -PythonArgs @("-c", "import torch; print('torch', torch.__version__)")

Write-Host "==> Verifying rvc-python on current Python..."
Invoke-Python -PythonArgs @("-c", "from rvc_python.infer import RVCInference; print('rvc-python OK')")

Write-Host "==> Building PyInstaller bundles..."
$DistPath = Join-Path $Root "dist"
$WorkPath = Join-Path $Root "build/pyinstaller"
New-Item -ItemType Directory -Force -Path $DistPath, $WorkPath | Out-Null

$PyInstallerArgs = @(
    "-m", "PyInstaller", "--noconfirm", "--clean",
    "--distpath", $DistPath,
    "--workpath", $WorkPath,
    "--log-level", "INFO"
)

$EngineSpec = Join-Path $Root "engine/game-reader-engine.spec"
$WorkerSpec = Join-Path $Root "engine/rvc-worker.spec"

Invoke-Python -PythonArgs ($PyInstallerArgs + @($EngineSpec))
Invoke-Python -PythonArgs ($PyInstallerArgs + @($WorkerSpec))

function Resolve-BuiltExe {
    param([string]$Name)
    $candidates = @(
        (Join-Path $DistPath "$Name.exe"),
        (Join-Path $Root "engine/dist/$Name.exe"),
        (Join-Path $DistPath $Name)
    )
    foreach ($path in $candidates) {
        if (Test-Path $path) { return $path }
    }
    Write-Host "==> Build artifacts under dist/:"
    if (Test-Path $DistPath) { Get-ChildItem -Recurse $DistPath | ForEach-Object { Write-Host $_.FullName } }
    Write-Host "==> Build artifacts under engine/dist/:"
    $engineDist = Join-Path $Root "engine/dist"
    if (Test-Path $engineDist) { Get-ChildItem -Recurse $engineDist | ForEach-Object { Write-Host $_.FullName } }
    throw "PyInstaller did not produce $Name.exe"
}

$EngineExe = Resolve-BuiltExe -Name "game-reader-engine"
$WorkerExe = Resolve-BuiltExe -Name "rvc-worker"

$BinDir = Join-Path $Root "src-tauri/binaries"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

$Triple = "x86_64-pc-windows-msvc"
Copy-Item $EngineExe (Join-Path $BinDir "game-reader-engine-$Triple.exe") -Force
Copy-Item $WorkerExe (Join-Path $BinDir "rvc-worker-$Triple.exe") -Force

Write-Host "==> Done."
Write-Host "    Engine: src-tauri/binaries/game-reader-engine-$Triple.exe"
Write-Host "    Worker: src-tauri/binaries/rvc-worker-$Triple.exe"
