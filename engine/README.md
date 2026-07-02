# Game Reader Engine

Python sidecar for the Game Reader desktop app. Exposes a JSON-RPC 2.0 service over stdio.

## Dev mode

From the repo root:

```powershell
set PYTHONPATH=.
set GAME_READER_ALLOW_HF_DOWNLOAD=1
python -m engine
```

Send requests as one JSON object per line on stdin:

```json
{"jsonrpc":"2.0","method":"ping","id":1}
{"jsonrpc":"2.0","method":"init","id":2}
```

## Unified Python environment

The desktop app uses a **single Python 3.11** environment with `rvc-python` bundled alongside Kokoro.
The legacy `rvc_env` (Python 3.10) is no longer required for the packaged app.

If `rvc-python` fails on 3.11, the build script produces a separate `rvc-worker.exe` fallback.

## Model paths

| Asset | Location |
|-------|----------|
| Config | `%APPDATA%\GameReader\config.json` |
| Voice models | `%LOCALAPPDATA%\GameReader\models\{voice_id}\` |
| Kokoro cache | `%LOCALAPPDATA%\GameReader\models\kokoro\` |
| Manifest | `%LOCALAPPDATA%\GameReader\models\manifest.json` |

## Build

```powershell
.\scripts\build-engine.ps1
```

Output is copied to `src-tauri/binaries/` for Tauri bundling.
