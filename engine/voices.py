"""Voice catalog and installed-model registry."""

import json
import os

from engine import paths

# Static catalog — download URLs resolved at runtime from GitHub Releases.
VOICE_CATALOG = [
    {
        "id": "dagoth",
        "label": "Dagoth Ur",
        "release_tag": "v1.0",
        "asset": "dagoth_ur_v2.pth",
        "size_bytes": 55220472,
        "f0up_key": -12,
        "speed": 1.0,
    },
    {
        "id": "narrator",
        "label": "Narrator",
        "release_tag": "v1.1",
        "asset": "bg3_narrator_v2.pth",
        "size_bytes": 55232492,
        "f0up_key": -6,
        "speed": 0.8,
    },
]

_CATALOG_BY_ID = {v["id"]: v for v in VOICE_CATALOG}


def catalog_entry(voice_id: str) -> dict | None:
    return _CATALOG_BY_ID.get(voice_id)


def model_file(voice_id: str) -> str | None:
    entry = catalog_entry(voice_id)
    if not entry:
        return None
    path = os.path.join(paths.voice_model_dir(voice_id), entry["asset"])
    return path if os.path.isfile(path) else None


def is_installed(voice_id: str) -> bool:
    return model_file(voice_id) is not None


def installed_voices() -> list[str]:
    return [v["id"] for v in VOICE_CATALOG if is_installed(v["id"])]


def load_manifest() -> dict:
    path = paths.manifest_path()
    if not os.path.isfile(path):
        return {"voices": {}}
    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except Exception:
        return {"voices": {}}


def save_manifest(manifest: dict):
    os.makedirs(paths.models_dir(), exist_ok=True)
    with open(paths.manifest_path(), "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)


def sync_manifest():
    """Rebuild manifest.json from installed voice files on disk."""
    voices = {}
    for entry in VOICE_CATALOG:
        model = model_file(entry["id"])
        if model:
            voices[entry["id"]] = {
                "model": model,
                "f0up_key": entry["f0up_key"],
            }
    save_manifest({"voices": voices})
    return voices


def rvc_models() -> dict[str, str]:
    """Map selectable voice id -> worker model key (same id when installed)."""
    return {vid: vid for vid in installed_voices()}


def voice_labels() -> dict[str, str]:
    return {v["id"]: v["label"] for v in VOICE_CATALOG if is_installed(v["id"])}


def voice_speed(voice_id: str) -> float:
    entry = catalog_entry(voice_id)
    return entry["speed"] if entry else 1.0


def register_download(voice_id: str):
    sync_manifest()
