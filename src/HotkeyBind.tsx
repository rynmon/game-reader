import { useEffect, useState } from "react";
import { formatHotkeyDisplay, eventToHotkey } from "./utils";

interface Props {
  label: string;
  description?: string;
  value: string;
  onChange: (value: string) => void;
}

export default function HotkeyBind({ label, description, value, onChange }: Props) {
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!recording) return;

    const onKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      if (e.key === "Escape") {
        setRecording(false);
        setError(null);
        return;
      }

      const hotkey = eventToHotkey(e);
      if (!hotkey) {
        setError("Use at least one modifier (Ctrl, Shift, Alt, Cmd) plus a key.");
        return;
      }

      onChange(hotkey);
      setRecording(false);
      setError(null);
    };

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [recording, onChange]);

  return (
    <div className="hotkey-bind">
      <div className="hotkey-bind-info">
        <span className="hotkey-bind-label">{label}</span>
        {description && <span className="hotkey-bind-desc">{description}</span>}
      </div>
      <button
        type="button"
        className={`hotkey-chip${recording ? " recording" : ""}`}
        onClick={() => {
          setRecording(true);
          setError(null);
        }}
        onBlur={() => setRecording(false)}
      >
        {recording ? "Press shortcut…" : formatHotkeyDisplay(value)}
      </button>
      {error && recording && <p className="text-error hotkey-bind-error">{error}</p>}
    </div>
  );
}
