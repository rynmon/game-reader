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
    return <div className="card">Loading settings…</div>;
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
    <div>
      <div className="card">
        <h2>Voice</h2>
        <div className="form-row">
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
        <div className="form-row">
          <label htmlFor="tts-speed">TTS speed</label>
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

      <div className="card">
        <h2>Performance</h2>
        <div className="form-row">
          <label htmlFor="prefetch">Background OCR prefetch</label>
          <input
            id="prefetch"
            type="checkbox"
            checked={settings.prefetch_ocr}
            onChange={(e) => update("prefetch_ocr", e.target.checked)}
          />
        </div>
      </div>

      <div className="card">
        <h2>Hotkeys</h2>
        <p style={{ fontSize: "0.8rem", color: "#8ba3b0", margin: "0 0 0.75rem" }}>
          Hotkey changes take effect after saving and restarting the app.
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
          <div className="form-row" key={key}>
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

      <button className="primary" onClick={save}>
        Save Settings
      </button>
      {saved && <span style={{ marginLeft: "0.75rem", color: "#5eead4" }}>Saved</span>}
    </div>
  );
}
