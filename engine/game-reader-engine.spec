# PyInstaller build for the Game Reader Python engine sidecar.
# Run from repo root: pyinstaller engine/game-reader-engine.spec

import os
import sys

block_cipher = None
root = os.path.abspath(os.path.join(SPECPATH, ".."))

a = Analysis(
    [os.path.join(SPECPATH, "service_entry.py")],
    pathex=[root],
    binaries=[],
    datas=[],
    hiddenimports=[
        "engine",
        "engine.service",
        "engine.tts",
        "engine.ocr",
        "engine.capture",
        "engine.config",
        "engine.prefetch",
        "engine.gpu",
        "engine.voices",
        "engine.paths",
        "engine.rvc_worker",
        "kokoro",
        "sounddevice",
        "soundfile",
        "mss",
        "pytesseract",
        "transformers",
        "rvc_python",
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=["tkinter", "matplotlib"],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name="game-reader-engine",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
