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

  return (
    <div>
      {gpu && !gpu.available && (
        <div className="banner error">
          {gpu.error ?? "NVIDIA GPU with CUDA is required."}
        </div>
      )}
      {(needsKokoro || needsVoice) && (
        <div className="banner warn">
          {needsKokoro && "Download the Kokoro speech engine from the Voices tab before reading."}
          {needsKokoro && needsVoice && " "}
          {needsVoice && "Install at least one character voice from the Voices tab."}
        </div>
      )}

      <div className="card">
        <h2>Status</h2>
        <div className="status-row">
          <span>Engine</span>
          <span className={status?.initialized ? "status-ok" : "status-error"}>
            {status?.initialized ? "Ready" : status?.init_error ?? "Not initialized"}
          </span>
        </div>
        <div className="status-row">
          <span>GPU</span>
          <span className={gpu?.available ? "status-ok" : "status-error"}>
            {gpu?.available ? gpu.device_name : gpu?.error ?? "Unknown"}
          </span>
        </div>
        <div className="status-row">
          <span>Active voice</span>
          <span>{status?.active_label ?? "None"}</span>
        </div>
        <div className="status-row">
          <span>Region</span>
          <span>
            {status?.region
              ? `${status.region.w}×${status.region.h} at (${status.region.x}, ${status.region.y})`
              : "Not set"}
          </span>
        </div>
        <div style={{ marginTop: "0.75rem", display: "flex", gap: "0.5rem" }}>
          <button className="primary" onClick={() => api.openRegionSelector()}>
            Select Region
          </button>
          <button onClick={() => api.readRegion()} disabled={!status?.initialized}>
            Read Now
          </button>
          <button onClick={onRefresh}>Refresh</button>
        </div>
      </div>

      <div className="card hotkeys">
        <h2>Hotkeys</h2>
        <p>
          <kbd>Ctrl+Shift+R</kbd> Select region &nbsp;·&nbsp;
          <kbd>Ctrl+Shift+T</kbd> Read region &nbsp;·&nbsp;
          <kbd>Ctrl+Shift+S</kbd> Stop &nbsp;·&nbsp;
          <kbd>Ctrl+Shift+V</kbd> Cycle voice &nbsp;·&nbsp;
          <kbd>Ctrl+Shift+Q</kbd> Quit
        </p>
      </div>
    </div>
  );
}
