import { useEffect, useState } from "react";
import type { DownloadProgress, EngineStatus, VoiceCatalogEntry } from "./types";
import { formatBytes } from "./utils";
import * as api from "./api";

interface Props {
  status: EngineStatus | null;
  onRefresh: () => void;
}

export default function VoicesTab({ status, onRefresh }: Props) {
  const [catalog, setCatalog] = useState<VoiceCatalogEntry[]>([]);
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({});
  const [kokoroProgress, setKokoroProgress] = useState<DownloadProgress | null>(null);
  const [storage, setStorage] = useState<Record<string, number>>({});

  useEffect(() => {
    api.getVoiceCatalog().then(setCatalog);
    api.getStorageUsage().then(setStorage);
    const unlisten = api.onDownloadProgress((p) => {
      if (p.kind === "kokoro") {
        setKokoroProgress(p);
      } else {
        setProgress((prev) => ({ ...prev, [p.id]: p }));
      }
      if (p.status === "complete") {
        onRefresh();
        api.getStorageUsage().then(setStorage);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [onRefresh]);

  const installed = new Set(status?.installed_voices ?? []);

  return (
    <div>
      <div className="card">
        <h2>Speech Engine (Kokoro)</h2>
        <p style={{ fontSize: "0.85rem", color: "#8ba3b0", margin: "0 0 0.75rem" }}>
          Required base TTS model (~330 MB). Downloaded once and cached locally.
        </p>
        <div className="voice-item">
          <div className="voice-meta">
            <h3>Kokoro TTS</h3>
            <p>{status?.kokoro_installed ? "Installed" : "Not installed"}</p>
            {kokoroProgress && kokoroProgress.status === "downloading" && (
              <div className="progress-bar">
                <div
                  className="progress-bar-fill"
                  style={{
                    width: `${(kokoroProgress.downloaded / Math.max(kokoroProgress.total, 1)) * 100}%`,
                  }}
                />
              </div>
            )}
          </div>
          {!status?.kokoro_installed && (
            <button className="primary" onClick={() => api.downloadKokoro()}>
              Download
            </button>
          )}
        </div>
      </div>

      <div className="card">
        <h2>Character Voices</h2>
        <p style={{ fontSize: "0.85rem", color: "#8ba3b0", margin: "0 0 0.75rem" }}>
          Optional downloads (~53 MB each). Only the model weights are needed.
        </p>
        {catalog.map((voice) => {
          const isInstalled = installed.has(voice.id);
          const dl = progress[voice.id];
          return (
            <div className="voice-item" key={voice.id}>
              <div className="voice-meta">
                <h3>
                  {voice.label}
                  {voice.recommended && " (recommended)"}
                </h3>
                <p>
                  {voice.description} — {formatBytes(voice.size_bytes)}
                </p>
                {dl && dl.status === "downloading" && (
                  <div className="progress-bar">
                    <div
                      className="progress-bar-fill"
                      style={{
                        width: `${(dl.downloaded / Math.max(dl.total, 1)) * 100}%`,
                      }}
                    />
                  </div>
                )}
                {dl?.status === "error" && (
                  <p className="status-error">{dl.error}</p>
                )}
              </div>
              {isInstalled ? (
                <button className="danger" onClick={() => api.deleteVoice(voice.id).then(onRefresh)}>
                  Delete
                </button>
              ) : (
                <button
                  className="primary"
                  disabled={dl?.status === "downloading"}
                  onClick={() => api.downloadVoice(voice.id)}
                >
                  Download
                </button>
              )}
            </div>
          );
        })}
      </div>

      {Object.keys(storage).length > 0 && (
        <div className="card">
          <h2>Storage</h2>
          {Object.entries(storage).map(([key, bytes]) => (
            <div className="status-row" key={key}>
              <span>{key}</span>
              <span>{formatBytes(bytes)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
