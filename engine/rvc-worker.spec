# PyInstaller build for the RVC worker subprocess.
# Run from repo root: pyinstaller engine/rvc-worker.spec

import os

block_cipher = None
root = os.path.abspath(os.path.join(SPECPATH, ".."))

worker = Analysis(
    [os.path.join(SPECPATH, "rvc_worker_entry.py")],
    pathex=[root],
    binaries=[],
    datas=[],
    hiddenimports=["engine", "engine.rvc_worker", "engine.paths", "engine.voices", "rvc_python"],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=["tkinter"],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

worker_pyz = PYZ(worker.pure, worker.zipped_data, cipher=block_cipher)

worker_exe = EXE(
    worker_pyz,
    worker.scripts,
    worker.binaries,
    worker.zipfiles,
    worker.datas,
    [],
    name="rvc-worker",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,
    console=True,
)
