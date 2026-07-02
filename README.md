# Game Reader

A Windows desktop tool that lets you select a region of your screen, then press a hotkey to have the text read aloud. Built for games — works in borderless windowed mode, remembers your selected region across sessions, and runs from the system tray.

Character voices run fully locally on GPU (Kokoro TTS generates speech, then an RVC model converts the voice):

- **Dagoth Ur** — trained RVC voice model from Morrowind
- **Narrator** — trained RVC voice model of the Baldur's Gate 3 narrator *(recommended default)*

Everything runs locally. No API keys, no manual Python setup.

---

## What's new in v2

Game Reader is now a **standalone desktop app** (Tauri + bundled Python engine):

- Single installer — no Python, Tesseract, or pip steps
- In-app downloads for Kokoro (~330 MB) and character voices (~53 MB each)
- Settings UI for hotkeys, voice selection, and storage
- Global hotkeys without running as Administrator

---

## Requirements

- Windows 10/11
- **NVIDIA GPU with CUDA** (required for voice inference)

---

## Install (end users)

1. Download the latest `.msi` or `.exe` installer from [Releases](https://github.com/baylic/game-reader/releases)
2. Run the installer
3. Open **Game Reader** from the Start menu
4. Go to **Voices** → download **Kokoro TTS** and at least one character voice
5. Press **Ctrl+Shift+R** to select a screen region, then **Ctrl+Shift+T** to read

---

## Hotkeys

| Hotkey | Action |
|--------|--------|
| Ctrl+Shift+R | Draw selection region on screen |
| Ctrl+Shift+T | Read selected region aloud |
| Ctrl+Shift+S | Stop playback |
| Ctrl+Shift+V | Cycle between installed voices |
| Ctrl+Shift+Q | Quit |

---

## Development

### Prerequisites

- Node.js 20+
- Rust (for Tauri)
- Python 3.11 with CUDA PyTorch
- Optional: 7-Zip (for bundling Tesseract on Windows)

### Setup

```powershell
git clone https://github.com/baylic/game-reader.git
cd game-reader

npm install
pip install -r engine/requirements.txt
pip install torch==2.11.0+cu128 torchvision==0.26.0+cu128 torchaudio==2.11.0+cu128 --index-url https://download.pytorch.org/whl/cu128

# Generate icons
python scripts/generate-icons.py

# Optional: bundle Tesseract for OCR (Windows)
.\scripts\fetch-tesseract.ps1
```

### Run in dev mode

Terminal 1 — frontend:

```powershell
npm run dev
```

Terminal 2 — Tauri shell (uses Python engine automatically):

```powershell
$env:PYTHONPATH = "."
$env:GAME_READER_ALLOW_HF_DOWNLOAD = "1"
npm run tauri dev
```

Or test the engine alone:

```powershell
$env:PYTHONPATH = "."
python -m engine
```

### Release build (Windows)

```powershell
.\scripts\fetch-tesseract.ps1
.\scripts\build-engine.ps1
npm run tauri build
```

Installers are written to `src-tauri/target/release/bundle/`.

---

## Project structure

```
game-reader/
  src/                 React UI (settings, voice manager)
  src-tauri/           Tauri shell (tray, hotkeys, overlay, downloads)
  engine/              Python ML sidecar (OCR, Kokoro, RVC)
  assets/              Voice catalog, bundled Tesseract
  scripts/             Build helpers
```

---

## Gaming tips

- Use **Borderless Windowed** mode — overlays don't work over exclusive fullscreen
- Select your region over an in-game text area (quest log, dialogue box, tooltip)
- If OCR misreads text, try selecting a tighter region around just the text

---

## Training your own RVC voice

See the [Applio](https://github.com/IAHispano/Applio) workflow in the previous README section. After exporting a `.pth` model, add it to `assets/voices.json` and publish via GitHub Releases.

---

## Legacy Python-only setup

The pre-v2 manual setup (`python main.py`, separate `rvc_env`, etc.) is deprecated. Use the desktop app or `python -m engine` for development.
