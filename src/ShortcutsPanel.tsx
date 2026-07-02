import { formatHotkeyDisplay } from "./utils";
import type { AppSettings } from "./types";

const ACTIONS: {
  key: "hotkey_select" | "hotkey_read" | "hotkey_stop" | "hotkey_cycle" | "hotkey_quit";
  label: string;
}[] = [
  { key: "hotkey_select", label: "Select region" },
  { key: "hotkey_read", label: "Read region" },
  { key: "hotkey_stop", label: "Stop speech" },
  { key: "hotkey_cycle", label: "Cycle voice" },
  { key: "hotkey_quit", label: "Quit" },
];

interface Props {
  settings: Pick<
    AppSettings,
    "hotkey_select" | "hotkey_read" | "hotkey_stop" | "hotkey_cycle" | "hotkey_quit"
  >;
}

export default function ShortcutsPanel({ settings }: Props) {
  return (
    <div className="panel shortcuts-panel">
      {ACTIONS.map(({ key, label }) => (
        <div className="shortcut-row" key={key}>
          <span className="shortcut-label">{label}</span>
          <kbd className="shortcut-keys">{formatHotkeyDisplay(settings[key])}</kbd>
        </div>
      ))}
    </div>
  );
}
