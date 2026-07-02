"""Central path resolution for bundled and user data directories."""

import os
import sys


def _default_data_dir() -> str:
    override = os.environ.get("GAME_READER_DATA_DIR")
    if override:
        return override
    if sys.platform == "win32":
        base = os.environ.get("LOCALAPPDATA") or os.environ.get("APPDATA") or os.path.expanduser("~")
        return os.path.join(base, "GameReader")
    return os.path.join(os.path.expanduser("~"), ".game-reader")


def _default_config_dir() -> str:
    override = os.environ.get("GAME_READER_CONFIG_DIR")
    if override:
        return override
    if sys.platform == "win32":
        base = os.environ.get("APPDATA") or os.path.expanduser("~")
        return os.path.join(base, "GameReader")
    return os.path.join(os.path.expanduser("~"), ".game-reader")


def app_dir() -> str:
    """Directory containing bundled app resources (tesseract, sidecar assets)."""
    override = os.environ.get("GAME_READER_APP_DIR")
    if override:
        return override
    if getattr(sys, "frozen", False):
        return os.path.dirname(sys.executable)
    return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def data_dir() -> str:
    """User data: models, manifests, caches."""
    return _default_data_dir()


def config_dir() -> str:
    """User config: config.json."""
    return _default_config_dir()


def config_path() -> str:
    return os.path.join(config_dir(), "config.json")


def models_dir() -> str:
    return os.path.join(data_dir(), "models")


def voice_model_dir(voice_id: str) -> str:
    return os.path.join(models_dir(), voice_id)


def manifest_path() -> str:
    return os.path.join(models_dir(), "manifest.json")


def kokoro_cache_dir() -> str:
    return os.path.join(models_dir(), "kokoro")


def tesseract_exe() -> str:
    override = os.environ.get("GAME_READER_TESSERACT")
    if override:
        return override
    bundled = os.path.join(app_dir(), "tesseract", "tesseract.exe")
    if os.path.isfile(bundled):
        return bundled
    if sys.platform == "win32":
        return r"C:\Program Files\Tesseract-OCR\tesseract.exe"
    return "tesseract"


def tessdata_dir() -> str:
    override = os.environ.get("TESSDATA_PREFIX")
    if override:
        return override
    bundled = os.path.join(app_dir(), "tesseract", "tessdata")
    if os.path.isdir(bundled):
        return bundled
    if sys.platform == "win32":
        return r"C:\Program Files\Tesseract-OCR\tessdata"
    return ""


def rvc_worker_script() -> str:
    if getattr(sys, "frozen", False):
        worker = os.path.join(app_dir(), "rvc-worker.exe")
        if os.path.isfile(worker):
            return worker
    bundled = os.path.join(app_dir(), "binaries", "rvc-worker.exe")
    if os.path.isfile(bundled):
        return bundled
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "rvc_worker.py")


def rvc_python() -> str:
    """Interpreter or executable used to spawn the RVC worker."""
    override = os.environ.get("GAME_READER_RVC_PYTHON")
    if override:
        return override
    worker = rvc_worker_script()
    if worker.endswith(".exe") and os.path.isfile(worker):
        return worker
    legacy = os.path.join(app_dir(), "rvc_env", "Scripts", "python.exe")
    if os.path.isfile(legacy):
        return legacy
    return sys.executable
