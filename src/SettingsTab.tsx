import { useEffect, useState } from "react";
import type { AppSettings, EngineStatus } from "./types";
import * as api from "./api";

interface Props {
  status: EngineStatus | null;
  onRefresh: () => void;
}

export default function SettingsTab({ status, onRefresh }: Props) {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    api.getSettings().then(setSettings);
  }, [status]);

  if (!settings) {
    return (
      <>
        <h1 className="page-title">Settings</h1>
        <div className="empty">Loading…</div>
      </>
    );
  }

  function update<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    setSettings((s) => (s ? { ...s, [key]: value } : s));
    setSaved(false);
  }

  async function save() {
    if (!settings) return;
    await api.saveSettings(settings);
    setSaved(true);
    onRefresh();
  }

  const installed = status?.installed_voices ?? [];

  return (
    <>
      <h1 className="page-title">Settings</h1>
      <p className="page-desc">Voice, performance, and keyboard shortcuts.</p>

      <section className="section">
        <h2 className="section-title">Voice</h2>
        <div className="panel">
          <div className="form-group">
            <label htmlFor="active-voice">Active voice</label>
            <select
              id="active-voice"
              value={settings.active_voice}
              onChange={(e) => {
                update("active_voice", e.target.value);
                api.setActiveVoice(e.target.value);
              }}
            >
              {installed.length === 0 && <option value="">No voices installed</option>}
              {installed.map((id) => (
                <option key={id} value={id}>
                  {status?.voice_labels[id] ?? id}
                </option>
              ))}
            </select>
          </div>
          <div className="form-group">
            <label htmlFor="tts-speed">Speech speed</label>
            <input
              id="tts-speed"
              type="number"
              min={0.5}
              max={2}
              step={0.1}
              value={settings.tts_speed}
              onChange={(e) => update("tts_speed", parseFloat(e.target.value))}
            />
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section-title">Performance</h2>
        <div className="panel">
          <div className="form-group">
            <div className="toggle-row">
              <div>
                <label htmlFor="prefetch">Background OCR prefetch</label>
                <p className="hint" style={{ margin: "4px 0 0" }}>
                  Pre-reads static text for faster response.
                </p>
              </div>
              <input
                id="prefetch"
                type="checkbox"
                checked={settings.prefetch_ocr}
                onChange={(e) => update("prefetch_ocr", e.target.checked)}
              />
            </div>
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section-title">Shortcuts</h2>
        <div className="panel">
          <p className="hint" style={{ padding: "14px 16px 0", margin: 0 }}>
            Changes apply after saving and restarting the app.
          </p>
          {(
            [
              ["hotkey_select", "Select region"],
              ["hotkey_read", "Read region"],
              ["hotkey_stop", "Stop speech"],
              ["hotkey_cycle", "Cycle voice"],
              ["hotkey_quit", "Quit"],
            ] as const
          ).map(([key, label]) => (
            <div className="form-group" key={key}>
              <label htmlFor={key}>{label}</label>
              <input
                id={key}
                type="text"
                value={settings[key]}
                onChange={(e) => update(key, e.target.value)}
              />
            </div>
          ))}
        </div>
      </section>

      <div className="save-row">
        <button className="btn btn-primary" onClick={save}>
          Save settings
        </button>
        {saved && <span className="saved-msg">Saved</span>}
      </div>
    </>
  );
}
