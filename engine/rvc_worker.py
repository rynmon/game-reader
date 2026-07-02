"""
Persistent RVC worker — spawned by the engine sidecar.
Reads "model_key|input_wav|output_wav" lines from stdin, writes "ok" or "err:<msg>".
Models are loaded from manifest.json in the user data directory.
"""
import json
import os
import sys

_proto = sys.stdout


def _load_models(manifest_path: str) -> dict:
    if not os.path.isfile(manifest_path):
        return {}
    try:
        with open(manifest_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data.get("voices", {})
    except Exception:
        return {}


def _reply(msg):
    _proto.write(msg + "\n")
    _proto.flush()


def main():
    global _proto
    sys.stdout = sys.stderr
    from engine import paths

    manifest_path = paths.manifest_path()
    models = _load_models(manifest_path)

    if not models:
        _reply("err:No voice models installed. Download a voice from Settings.")
        sys.exit(1)

    from rvc_python.infer import RVCInference

    rvc = RVCInference(device="cuda:0")
    loaded_key = None

    def ensure_model(key):
        nonlocal loaded_key, models
        if key not in models:
            models = _load_models(manifest_path)
        if key not in models:
            raise KeyError(f"Voice model '{key}' is not installed")
        if key == loaded_key:
            return
        cfg = models[key]
        model_path = cfg["model"]
        index_path = cfg.get("index")
        if index_path and not os.path.isfile(index_path):
            index_path = None
        rvc.load_model(model_path, version="v2", index_path=index_path)
        rvc.set_params(
            f0method="rmvpe",
            f0up_key=cfg.get("f0up_key", 0),
            index_rate=0.0,
            filter_radius=3,
            resample_sr=0,
            rms_mix_rate=1,
            protect=0.33,
        )
        loaded_key = key

    default_key = next(iter(models))
    ensure_model(default_key)
    _reply("ready")

    for line in sys.stdin:
        line = line.strip().lstrip("\ufeff").strip()
        if not line:
            continue
        if line == "reload":
            models = _load_models(manifest_path)
            loaded_key = None
            _reply("ok")
            continue
        try:
            model_key, input_wav, output_wav = line.split("|", 2)
            ensure_model(model_key)
            rvc.infer_file(input_wav, output_wav)
            _reply("ok")
        except Exception as e:
            _reply(f"err:{e}")


if __name__ == "__main__":
    main()
