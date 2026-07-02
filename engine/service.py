"""
JSON-RPC 2.0 service over stdio for the Game Reader engine sidecar.
Protocol: one JSON object per line on stdin; one JSON response per line on stdout.
Logs and debug output go to stderr.
"""

import json
import os
import sys
import threading
import traceback
from concurrent.futures import ThreadPoolExecutor

from engine import config, capture, ocr, prefetch, tts, voices
from engine.gpu import check_gpu

_executor = ThreadPoolExecutor(max_workers=1)
_prefetch_started = False


def _reply(id_, result=None, error=None):
    msg = {"jsonrpc": "2.0", "id": id_}
    if error is not None:
        msg["error"] = error
    else:
        msg["result"] = result
    sys.stdout.write(json.dumps(msg) + "\n")
    sys.stdout.flush()


def _error(code: int, message: str) -> dict:
    return {"code": code, "message": message}


def _ensure_prefetch():
    global _prefetch_started
    if not _prefetch_started and config.get("prefetch_ocr", True):
        prefetch.start(config.get_region)
        _prefetch_started = True


def _read_region():
    region = config.get_region()
    if not region:
        tts.speak(
            "No region selected. Press Control Shift R to select a region.",
            config.get("tts_voice"),
            config.get("tts_speed", 1.0),
        )
        return {"spoken": True, "text": None}
    try:
        img = capture.capture_region(region)
    except Exception as e:
        return {"spoken": False, "error": str(e)}
    text = prefetch.lookup(prefetch.frame_hash(img))
    if text is None:
        if capture.is_black_frame(img):
            tts.speak(
                "Screen capture failed. Switch your game to borderless windowed mode.",
                config.get("tts_voice"),
                config.get("tts_speed", 1.0),
            )
            return {"spoken": True, "text": None}
        text = ocr.run_ocr(img)
        prefetch.note(prefetch.frame_hash(img), text)
    if not text:
        tts.speak(
            "No text detected.",
            config.get("tts_voice"),
            config.get("tts_speed", 1.0),
        )
        return {"spoken": True, "text": ""}
    tts.speak(text, config.get("tts_voice"), config.get("tts_speed", 1.0))
    return {"spoken": True, "text": text}


def _init_engine():
    config.load()
    result = tts.init()
    _ensure_prefetch()
    status = tts.get_status()
    return {**result, **status}


def handle(method: str, params: dict | None):
    params = params or {}

    if method == "ping":
        return {"pong": True}

    if method == "get_gpu_info":
        return check_gpu()

    if method == "load_config":
        config.load()
        return config.all_settings()

    if method == "save_config":
        data = params.get("settings", params)
        if isinstance(data, dict):
            for key, value in data.items():
                config.set(key, value)
        return config.all_settings()

    if method == "save_region":
        config.save_region(params["x"], params["y"], params["w"], params["h"])
        return config.get_region()

    if method == "init":
        return _init_engine()

    if method == "download_kokoro":
        os.environ["GAME_READER_ALLOW_HF_DOWNLOAD"] = "1"
        os.environ.pop("HF_HUB_OFFLINE", None)
        config.load()
        tts.reset()
        result = tts.init()
        status = tts.get_status()
        return {**result, **status}

    if method == "get_status":
        return {
            **tts.get_status(),
            "gpu": check_gpu(),
            "region": config.get_region(),
            "settings": config.all_settings(),
        }

    if method == "reload_voices":
        voices.register_download(params.get("voice_id", ""))
        return tts.reload_voices()

    if method == "read_region":
        return _read_region()

    if method == "speak":
        tts.speak(params.get("text", ""), params.get("voice", config.get("tts_voice")), params.get("speed", config.get("tts_speed", 1.0)))
        return {"ok": True}

    if method == "stop":
        tts.stop()
        return {"ok": True}

    if method == "cycle_voice":
        label = tts.cycle_voice()
        tts.speak(f"Voice switched to {label}.", config.get("tts_voice"), config.get("tts_speed", 1.0))
        return {"label": label}

    if method == "set_voice":
        label = tts.set_active_voice(params["voice_id"])
        return {"label": label}

    if method == "shutdown":
        prefetch.stop()
        tts.stop()
        return {"ok": True}

    raise ValueError(f"Unknown method: {method}")


def _dispatch(req: dict):
    id_ = req.get("id")
    method = req.get("method")
    params = req.get("params")

    def work():
        try:
            if method in ("read_region", "speak", "cycle_voice"):
                future = _executor.submit(handle, method, params)
                result = future.result()
            else:
                result = handle(method, params)
            if id_ is not None:
                _reply(id_, result=result)
        except Exception as e:
            traceback.print_exc(file=sys.stderr)
            if id_ is not None:
                _reply(id_, error=_error(-32000, str(e)))

    threading.Thread(target=work, daemon=True).start()


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except json.JSONDecodeError as e:
            _reply(None, error=_error(-32700, f"Parse error: {e}"))
            continue
        if req.get("method") == "notify":
            threading.Thread(target=_dispatch, args=(req,), daemon=True).start()
        else:
            _dispatch(req)


if __name__ == "__main__":
    main()
