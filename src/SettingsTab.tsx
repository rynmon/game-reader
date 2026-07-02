import { useEffect, useState } from "react";
import type { AppSettings, EngineStatus } from "./types";
import { DEFAULT_SETTINGS } from "./utils";
import HotkeyBind from "./HotkeyBind";
import * as api from "./api";

interface Props {
  status: EngineStatus | null;
  onRefresh: () => void;
}

const HOTKEY_FIELDS: {
  key: keyof AppSettings;
  label: string;
  description: string;
}[] = [
  { key: "hotkey_select", label: "Select region", description: "Open the screen region picker" },
  { key: "hotkey_read", label: "Read region", description: "OCR and read the selected area" },
  { key: "hotkey_stop", label: "Stop speech", description: "Stop current playback" },
  { key: "hotkey_cycle", label: "Cycle voice", description: "Switch between installed voices" },
  { key: "hotkey_quit", label: "Quit", description: "Exit Game Reader" },
];

export default function SettingsTab({ status, onRefresh }: Props) {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    setError(null);
    api
      .getSettings()
      .then((s) => setSettings({ ...DEFAULT_SETTINGS, ...s }))
      .catch((e) => {
        setError(String(e));
        setSettings({ ...DEFAULT_SETTINGS });
      })
      .finally(() => setLoading(false));
  }, []);

  if (loading || !settings) {
    return (
      <>
        <h1 className="page-title">Settings</h1>
        <div className="empty">{error ? `Failed to load: ${error}` : "Loading…"}</div>
      </>
    );
  }

  function update<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    setSettings((s) => (s ? { ...s, [key]: value } : s));
    setSaved(false);
    setError(null);
  }

  async function save() {
    if (!settings) return;
    try {
      const updated = await api.saveSettings(settings);
      setSettings({ ...DEFAULT_SETTINGS, ...updated });
      setSaved(true);
      setError(null);
      onRefresh();
    } catch (e) {
      setError(String(e));
      setSaved(false);
    }
  }

  const installed = status?.installed_voices ?? [];

  return (
    <>
      <h1 className="page-title">Settings</h1>
      <p className="page-desc">Voice, performance, and keyboard shortcuts.</p>

      {error && <div className="alert alert-error">{error}</div>}

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
                if (e.target.value) api.setActiveVoice(e.target.value);
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
        <p className="section-hint">Click a shortcut to rebind. Press Escape to cancel.</p>
        <div className="panel">
          {HOTKEY_FIELDS.map(({ key, label, description }) => (
            <HotkeyBind
              key={key}
              label={label}
              description={description}
              value={settings[key] as string}
              onChange={(value) => update(key, value)}
            />
          ))}
        </div>
      </section>

      <div className="save-row">
        <button className="btn btn-primary" onClick={save}>
          Save settings
        </button>
        {saved && <span className="saved-msg">Saved — shortcuts updated</span>}
      </div>
    </>
  );
}
