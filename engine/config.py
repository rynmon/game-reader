import json
import os

from engine import paths

DEFAULT_CONFIG = {
    "region": None,
    "hotkey_select": "ctrl+shift+r",
    "hotkey_read": "ctrl+shift+t",
    "hotkey_stop": "ctrl+shift+s",
    "hotkey_quit": "ctrl+shift+q",
    "hotkey_cycle": "ctrl+shift+v",
    "tts_voice": "bf_emma",
    "tts_speed": 1.0,
    "active_voice": "narrator",
    "prefetch_ocr": True,
}

_config = dict(DEFAULT_CONFIG)


def load():
    global _config
    os.makedirs(paths.config_dir(), exist_ok=True)
    cfg_path = paths.config_path()
    if os.path.exists(cfg_path):
        try:
            with open(cfg_path, "r", encoding="utf-8") as f:
                saved = json.load(f)
            _config = {**DEFAULT_CONFIG, **saved}
        except Exception:
            _config = dict(DEFAULT_CONFIG)
    else:
        _config = dict(DEFAULT_CONFIG)


def save():
    os.makedirs(paths.config_dir(), exist_ok=True)
    with open(paths.config_path(), "w", encoding="utf-8") as f:
        json.dump(_config, f, indent=2)


def get_region():
    return _config.get("region")


def save_region(x, y, w, h):
    _config["region"] = {"x": x, "y": y, "w": w, "h": h}
    save()


def get(key, default=None):
    return _config.get(key, default)


def set(key, value):
    _config[key] = value
    save()


def all_settings() -> dict:
    return dict(_config)
