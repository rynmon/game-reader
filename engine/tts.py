import os
import queue
import re
import subprocess
import sys
import tempfile
import threading
import time

import numpy as np
import sounddevice as sd
import soundfile as sf

from engine import config, paths, voices

_stop_event = threading.Event()
_kokoro_pipeline = None
_active_voice: str | None = None
_initialized = False
_init_error: str | None = None

_rvc_proc = None
_rvc_lock = threading.Lock()
_rvc_ready = threading.Event()
_use_worker_exe = False


def _voice_list() -> list[str]:
    installed = voices.installed_voices()
    return installed if installed else []


def _ensure_active_voice():
    global _active_voice
    available = _voice_list()
    if not available:
        _active_voice = None
        return
    preferred = config.get("active_voice", "narrator")
    if _active_voice and _active_voice in available:
        return
    if preferred in available:
        _active_voice = preferred
    else:
        _active_voice = available[0]


def _rvc_python() -> str:
    return paths.rvc_python()


def _rvc_worker() -> str:
    return paths.rvc_worker_script()


def _start_rvc_worker():
    global _rvc_proc, _use_worker_exe
    with _rvc_lock:
        if _rvc_proc is not None and _rvc_proc.poll() is None:
            return
        _rvc_ready.clear()
        worker = _rvc_worker()
        python = _rvc_python()
        if worker.endswith(".exe") and os.path.isfile(worker):
            cmd = [worker]
            _use_worker_exe = True
        else:
            cmd = [python, "-m", "engine.rvc_worker"]
            _use_worker_exe = False
        proc = subprocess.Popen(
            cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            bufsize=1,
            cwd=os.path.dirname(worker) if not _use_worker_exe else None,
        )
        for _ in range(50):
            line = proc.stdout.readline().strip()
            if line == "ready":
                break
            if line.startswith("err:"):
                raise RuntimeError(line[4:])
            if proc.poll() is not None:
                raise RuntimeError("RVC worker exited before becoming ready")
        else:
            proc.kill()
            raise RuntimeError("RVC worker never sent 'ready'")
        _rvc_proc = proc
        _rvc_ready.set()


def _reload_rvc_worker():
    global _rvc_proc
    with _rvc_lock:
        if _rvc_proc is not None and _rvc_proc.poll() is None:
            try:
                _rvc_proc.stdin.write("reload\n")
                _rvc_proc.stdin.flush()
                _rvc_proc.stdout.readline()
            except Exception:
                _rvc_proc.kill()
                _rvc_proc = None
    if _rvc_proc is None or _rvc_proc.poll() is not None:
        _start_rvc_worker()


def _warm_rvc():
    try:
        _ensure_active_voice()
        if not _active_voice:
            return
        dummy = np.random.randn(int(24000 * 1.0)).astype(np.float32) * 0.01
        _rvc_convert(dummy, _active_voice)
    except Exception as e:
        print(f"RVC warm-up skipped: {e}", file=sys.stderr)


def _start_and_warm():
    try:
        if _voice_list():
            _start_rvc_worker()
            _warm_rvc()
    except Exception as e:
        print(f"RVC worker start failed: {e}", file=sys.stderr)


def reset():
    global _kokoro_pipeline, _initialized, _init_error, _rvc_proc
    stop()
    _kokoro_pipeline = None
    _initialized = False
    _init_error = None
    if _rvc_proc is not None:
        try:
            _rvc_proc.kill()
        except Exception:
            pass
        _rvc_proc = None


def init():
    global _kokoro_pipeline, _initialized, _init_error
    if _initialized and _kokoro_pipeline is not None:
        return {"ok": True}
    from engine.gpu import require_gpu

    gpu_err = require_gpu()
    if gpu_err:
        _init_error = gpu_err["error"]
        _initialized = False
        return {"ok": False, "error": _init_error}

    voices.sync_manifest()
    _ensure_active_voice()

    kokoro_dir = paths.kokoro_cache_dir()
    os.makedirs(kokoro_dir, exist_ok=True)
    os.environ.setdefault("HF_HOME", kokoro_dir)
    os.environ.setdefault("HUGGINGFACE_HUB_CACHE", kokoro_dir)

    if not os.environ.get("GAME_READER_ALLOW_HF_DOWNLOAD"):
        os.environ["HF_HUB_OFFLINE"] = "1"

    try:
        from kokoro import KPipeline

        _kokoro_pipeline = KPipeline(lang_code="b")
    except Exception as e:
        _init_error = f"Failed to load Kokoro TTS: {e}"
        _initialized = False
        return {"ok": False, "error": _init_error, "needs_kokoro_download": True}

    _initialized = True
    _init_error = None
    threading.Thread(target=_start_and_warm, daemon=True).start()
    return {"ok": True}


def reload_voices():
    voices.sync_manifest()
    _ensure_active_voice()
    if _voice_list():
        threading.Thread(target=_reload_rvc_worker, daemon=True).start()
    return get_status()


def cycle_voice() -> str:
    global _active_voice
    available = _voice_list()
    if not available:
        raise RuntimeError("No voice models installed")
    if _active_voice not in available:
        _active_voice = available[0]
    else:
        idx = available.index(_active_voice)
        _active_voice = available[(idx + 1) % len(available)]
    config.set("active_voice", _active_voice)
    labels = voices.voice_labels()
    return labels.get(_active_voice, _active_voice)


def set_active_voice(voice_id: str) -> str:
    global _active_voice
    if voice_id not in _voice_list():
        raise ValueError(f"Voice '{voice_id}' is not installed")
    _active_voice = voice_id
    config.set("active_voice", voice_id)
    return voices.voice_labels().get(voice_id, voice_id)


def get_active_label() -> str:
    if not _active_voice:
        return "None"
    return voices.voice_labels().get(_active_voice, _active_voice)


def get_status() -> dict:
    labels = voices.voice_labels()
    installed = voices.installed_voices()
    return {
        "initialized": _initialized,
        "init_error": _init_error,
        "active_voice": _active_voice,
        "active_label": get_active_label(),
        "installed_voices": installed,
        "voice_labels": labels,
        "catalog": voices.VOICE_CATALOG,
        "kokoro_installed": _kokoro_pipeline is not None,
    }


_SPEAKER_RE = re.compile(r"^[A-Z][\w''.\-]*(?:\s+[A-Z][\w''.\-]*){0,3}\s*:\s*(?=\S)")


def _strip_speaker(text: str) -> str:
    m = _SPEAKER_RE.match(text)
    if m and m.end() < len(text):
        return text[m.end() :]
    return text


def _clean(text: str) -> str:
    text = re.sub(r"\n+", " ", text)
    text = re.sub(r" +", " ", text).strip()
    return _strip_speaker(text)


def _play(audio: np.ndarray, sr: int, tail: float = 0.5):
    sd.play(audio, samplerate=sr)
    duration = len(audio) / sr
    start = time.time()
    while time.time() - start < duration + tail:
        if _stop_event.is_set():
            sd.stop()
            return
        time.sleep(0.05)


def _rvc_convert(audio: np.ndarray, model_key: str) -> tuple[np.ndarray, int] | None:
    global _rvc_proc
    tmp_in = os.path.join(tempfile.gettempdir(), "rvc_in.wav")
    tmp_out = os.path.join(tempfile.gettempdir(), "rvc_out.wav")
    sf.write(tmp_in, audio, 24000)

    _rvc_ready.wait(timeout=120)
    with _rvc_lock:
        if _rvc_proc is None or _rvc_proc.poll() is not None:
            _start_rvc_worker()
        _rvc_proc.stdin.write(f"{model_key}|{tmp_in}|{tmp_out}\n")
        _rvc_proc.stdin.flush()
        response = _rvc_proc.stdout.readline().strip()

    if response != "ok":
        print(f"RVC worker error: {response}", file=sys.stderr)
        return None
    out_audio, out_sr = sf.read(tmp_out)
    return out_audio.astype(np.float32), out_sr


def _speak_rvc(text: str, voice: str, speed: float, model_key: str):
    audio_q: queue.Queue = queue.Queue(maxsize=4)

    def _put(item) -> bool:
        while not _stop_event.is_set():
            try:
                audio_q.put(item, timeout=0.1)
                return True
            except queue.Full:
                continue
        return False

    def produce():
        try:
            for _, _, audio in _kokoro_pipeline(text, voice=voice, speed=speed):
                if _stop_event.is_set():
                    return
                if audio is None:
                    continue
                seg = audio.cpu().numpy()
                result = _rvc_convert(seg, model_key)
                if result is None:
                    result = (seg, 24000)
                if not _put(result):
                    return
        finally:
            _put(None)

    threading.Thread(target=produce, daemon=True).start()

    while not _stop_event.is_set():
        item = audio_q.get()
        if item is None:
            return
        out_audio, out_sr = item
        _play(out_audio, out_sr, tail=0.1)


def _speak_kokoro_only(text: str, voice: str, speed: float):
    for _, _, audio in _kokoro_pipeline(text, voice=voice, speed=speed):
        if _stop_event.is_set():
            return
        if audio is None:
            continue
        seg = audio.cpu().numpy()
        _play(seg, 24000, tail=0.1)


def speak(text: str, voice: str = "bf_emma", speed: float = 1.0):
    if not _initialized or _kokoro_pipeline is None:
        raise RuntimeError(_init_error or "TTS engine not initialized")
    _stop_event.clear()
    text = _clean(text)
    _ensure_active_voice()
    if _active_voice and _active_voice in voices.rvc_models():
        speed = voices.voice_speed(_active_voice) if speed == config.get("tts_speed", 1.0) else speed
        _speak_rvc(text, voice, speed, _active_voice)
    else:
        _speak_kokoro_only(text, voice, speed)


def stop():
    _stop_event.set()
    try:
        sd.stop()
    except Exception:
        pass
