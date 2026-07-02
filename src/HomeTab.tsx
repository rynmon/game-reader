import { useEffect, useState } from "react";
import type { AppSettings, EngineStatus } from "./types";
import { DEFAULT_SETTINGS } from "./utils";
import ShortcutsPanel from "./ShortcutsPanel";
import * as api from "./api";

interface Props {
  status: EngineStatus | null;
  onRefresh: () => void;
}

export default function HomeTab({ status, onRefresh }: Props) {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const gpu = status?.gpu;
  const needsRuntime = status && !status.engine_runtime_installed;
  const needsKokoro = status && status.engine_runtime_installed && !status.kokoro_installed;
  const needsVoice = status && status.engine_runtime_installed && status.installed_voices.length === 0;

  const engineState = status?.initialized
    ? "ok"
    : status?.init_error
      ? "error"
      : "warn";

  useEffect(() => {
    api.getSettings().then((s) => setSettings({ ...DEFAULT_SETTINGS, ...s }));
  }, []);

  const shortcutSettings = settings ?? DEFAULT_SETTINGS;

  return (
    <>
      <h1 className="page-title">Overview</h1>
      <p className="page-desc">Screen region OCR with local text-to-speech.</p>

      {gpu && !gpu.available && (
        <div className="alert alert-error">
          {gpu.error ?? "NVIDIA GPU with CUDA is required."}
        </div>
      )}

      {(needsRuntime || needsKokoro || needsVoice) && (
        <div className="alert alert-warn">
          {needsRuntime
            ? "Download the CUDA AI runtime from Voices before continuing."
            : needsKokoro && needsVoice
              ? "Download the speech engine and at least one voice before reading."
              : needsKokoro
                ? "Download the Kokoro speech engine from Voices."
                : "Install a character voice from Voices."}
        </div>
      )}

      <section className="section">
        <h2 className="section-title">Status</h2>
        <div className="panel">
          <div className="row">
            <span className="row-label">Engine</span>
            <span className={`badge ${engineState}`}>
              {status?.initialized ? "Ready" : status?.init_error ?? "Starting…"}
            </span>
          </div>
          <div className="row">
            <span className="row-label">GPU</span>
            <span className={`badge ${gpu?.available ? "ok" : "error"}`}>
              {gpu?.available ? gpu.device_name : gpu?.error ?? "Unknown"}
            </span>
          </div>
          <div className="row">
            <span className="row-label">Voice</span>
            <span className="row-value">{status?.active_label ?? "None"}</span>
          </div>
          <div className="row">
            <span className="row-label">Region</span>
            <span className="row-value">
              {status?.region
                ? `${status.region.w} × ${status.region.h}`
                : "Not selected"}
            </span>
          </div>
        </div>

        <div className="actions">
          <button className="btn btn-primary" onClick={() => api.openRegionSelector()}>
            Select region
          </button>
          <button
            className="btn"
            onClick={() => api.readRegion()}
            disabled={!status?.initialized}
          >
            Read now
          </button>
          <button className="btn btn-ghost" onClick={onRefresh}>
            Refresh
          </button>
        </div>
      </section>

      <section className="section">
        <h2 className="section-title">Shortcuts</h2>
        <ShortcutsPanel settings={shortcutSettings} />
        <p className="section-hint">Customize bindings in Settings.</p>
      </section>
    </>
  );
}
