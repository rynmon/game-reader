"""Background OCR cache."""

import threading
import time
import zlib

from engine import capture, ocr

_INTERVAL = 0.25

_lock = threading.Lock()
_last_hash: int | None = None
_last_text: str | None = None

_get_region = None
_stop = threading.Event()


def frame_hash(img) -> int:
    return zlib.crc32(img.tobytes())


def note(h: int, text: str):
    global _last_hash, _last_text
    with _lock:
        _last_hash = h
        _last_text = text


def lookup(h: int) -> str | None:
    with _lock:
        if h == _last_hash:
            return _last_text
    return None


def _loop():
    while not _stop.is_set():
        region = _get_region() if _get_region else None
        if region:
            try:
                img = capture.capture_region(region)
                h = frame_hash(img)
                with _lock:
                    unchanged = h == _last_hash
                if not unchanged and not capture.is_black_frame(img):
                    note(h, ocr.run_ocr(img))
            except Exception:
                pass
        _stop.wait(_INTERVAL)


def start(get_region):
    global _get_region
    _get_region = get_region
    _stop.clear()
    threading.Thread(target=_loop, daemon=True).start()


def stop():
    _stop.set()
