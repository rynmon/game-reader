import { useEffect, useState } from "react";
import type { DownloadProgress, EngineRuntimeManifest, EngineStatus, VoiceCatalogEntry } from "./types";
import { formatBytes } from "./utils";
import * as api from "./api";

interface Props {
  status: EngineStatus | null;
  onRefresh: () => void;
}

function ProgressBar({ progress }: { progress: DownloadProgress }) {
  const pct = (progress.downloaded / Math.max(progress.total, 1)) * 100;
  return (
    <div className="progress">
      <div className="progress-fill" style={{ width: `${pct}%` }} />
    </div>
  );
}

export default function VoicesTab({ status, onRefresh }: Props) {
  const [catalog, setCatalog] = useState<VoiceCatalogEntry[]>([]);
  const [runtimeManifest, setRuntimeManifest] = useState<EngineRuntimeManifest | null>(null);
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({});
  const [kokoroProgress, setKokoroProgress] = useState<DownloadProgress | null>(null);
  const [runtimeProgress, setRuntimeProgress] = useState<DownloadProgress | null>(null);
  const [storage, setStorage] = useState<Record<string, number>>({});

  const runtimeInstalled = status?.engine_runtime_installed ?? false;

  useEffect(() => {
    api.getVoiceCatalog().then(setCatalog);
    api.getEngineRuntimeManifest().then(setRuntimeManifest).catch(console.error);
    api.getStorageUsage().then(setStorage);
    const unlisten = api.onDownloadProgress((p) => {
      if (p.kind === "kokoro") {
        setKokoroProgress(p);
      } else if (p.kind === "engine-runtime") {
        setRuntimeProgress(p);
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
    <>
      <h1 className="page-title">Voices</h1>
      <p className="page-desc">Download components on demand to keep the installer small.</p>

      <section className="section">
        <h2 className="section-title">AI runtime</h2>
        <div className="panel">
          <div className="list-item">
            <div className="list-body">
              <p className="list-title">{runtimeManifest?.label ?? "CUDA AI runtime"}</p>
              <p className="list-sub">
                {runtimeManifest?.description ?? "PyTorch engine with NVIDIA CUDA"}{" "}
                {runtimeManifest?.size_bytes
                  ? `· ${formatBytes(runtimeManifest.size_bytes)}`
                  : null}{" "}
                · {runtimeInstalled ? "Installed" : "Required first"}
              </p>
              {runtimeProgress?.status === "downloading" && (
                <ProgressBar progress={runtimeProgress} />
              )}
              {runtimeProgress?.status === "error" && (
                <p className="text-error">{runtimeProgress.error}</p>
              )}
            </div>
            {runtimeInstalled ? (
              <button
                className="btn btn-sm btn-ghost"
                onClick={() => api.deleteEngineRuntime().then(onRefresh)}
              >
                Remove
              </button>
            ) : (
              <button
                className="btn btn-primary btn-sm"
                disabled={runtimeProgress?.status === "downloading"}
                onClick={() => api.downloadEngineRuntime()}
              >
                Download
              </button>
            )}
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section-title">Speech engine</h2>
        <div className="panel">
          <div className="list-item">
            <div className="list-body">
              <p className="list-title">Kokoro TTS</p>
              <p className="list-sub">
                Base model · ~330 MB ·{" "}
                {status?.kokoro_installed ? "Installed" : runtimeInstalled ? "Required" : "Needs AI runtime"}
              </p>
              {kokoroProgress?.status === "downloading" && (
                <ProgressBar progress={kokoroProgress} />
              )}
            </div>
            {status?.kokoro_installed ? (
              <span className="badge ok">Installed</span>
            ) : (
              <button
                className="btn btn-primary btn-sm"
                disabled={!runtimeInstalled || kokoroProgress?.status === "downloading"}
                onClick={() => api.downloadKokoro()}
              >
                Download
              </button>
            )}
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section-title">Character voices</h2>
        <div className="panel">
          {catalog.length === 0 ? (
            <div className="empty">No voices in catalog.</div>
          ) : (
            catalog.map((voice) => {
              const isInstalled = installed.has(voice.id);
              const dl = progress[voice.id];
              return (
                <div className="list-item" key={voice.id}>
                  <div className="list-body">
                    <p className="list-title">
                      {voice.label}
                      {voice.recommended && <span className="tag">Recommended</span>}
                    </p>
                    <p className="list-sub">
                      {voice.description} · {formatBytes(voice.size_bytes)}
                    </p>
                    {dl?.status === "downloading" && <ProgressBar progress={dl} />}
                    {dl?.status === "error" && (
                      <p className="text-error">{dl.error}</p>
                    )}
                  </div>
                  {isInstalled ? (
                    <button
                      className="btn btn-sm btn-ghost"
                      onClick={() => api.deleteVoice(voice.id).then(onRefresh)}
                    >
                      Remove
                    </button>
                  ) : (
                    <button
                      className="btn btn-primary btn-sm"
                      disabled={!runtimeInstalled || dl?.status === "downloading"}
                      onClick={() => api.downloadVoice(voice.id)}
                    >
                      Download
                    </button>
                  )}
                </div>
              );
            })
          )}
        </div>
      </section>

      {Object.keys(storage).length > 0 && (
        <section className="section">
          <h2 className="section-title">Storage</h2>
          <div className="panel">
            {Object.entries(storage).map(([key, bytes]) => (
              <div className="row" key={key}>
                <span className="row-label">
                  {key === "engine-runtime"
                    ? "AI runtime"
                    : key === "kokoro"
                      ? "Kokoro"
                      : key.replace("voice:", "")}
                </span>
                <span className="row-value">{formatBytes(bytes)}</span>
              </div>
            ))}
          </div>
        </section>
      )}
    </>
  );
}
