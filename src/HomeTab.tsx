import type { EngineStatus } from "./types";
import * as api from "./api";

interface Props {
  status: EngineStatus | null;
  onRefresh: () => void;
}

export default function HomeTab({ status, onRefresh }: Props) {
  const gpu = status?.gpu;
  const needsKokoro = status && !status.kokoro_installed;
  const needsVoice = status && status.installed_voices.length === 0;

  const engineState = status?.initialized
    ? "ok"
    : status?.init_error
      ? "error"
      : "warn";

  return (
    <>
      <h1 className="page-title">Overview</h1>
      <p className="page-desc">Screen region OCR with local text-to-speech.</p>

      {gpu && !gpu.available && (
        <div className="alert alert-error">
          {gpu.error ?? "NVIDIA GPU with CUDA is required."}
        </div>
      )}

      {(needsKokoro || needsVoice) && (
        <div className="alert alert-warn">
          {needsKokoro && needsVoice
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
        <div className="panel" style={{ padding: "16px" }}>
          <div className="hotkey-grid">
            <kbd>Ctrl+Shift+R</kbd>
            <span>Select region</span>
            <kbd>Ctrl+Shift+T</kbd>
            <span>Read region</span>
            <kbd>Ctrl+Shift+S</kbd>
            <span>Stop speech</span>
            <kbd>Ctrl+Shift+V</kbd>
            <span>Cycle voice</span>
            <kbd>Ctrl+Shift+Q</kbd>
            <span>Quit</span>
          </div>
        </div>
      </section>
    </>
  );
}
