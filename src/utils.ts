export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/** Display "ctrl+shift+r" as "Ctrl + Shift + R" */
export function formatHotkeyDisplay(raw: string): string {
  if (!raw) return "Unbound";
  return raw
    .split("+")
    .filter(Boolean)
    .map((part) => {
      switch (part.toLowerCase()) {
        case "ctrl":
        case "control":
          return "Ctrl";
        case "shift":
          return "Shift";
        case "alt":
        case "option":
          return "Alt";
        case "cmd":
        case "command":
        case "meta":
        case "super":
        case "win":
          return "Cmd";
        default:
          return part.length === 1 ? part.toUpperCase() : part.toUpperCase();
      }
    })
    .join(" + ");
}

/** Capture keyboard event into hotkey string like "ctrl+shift+r" */
export function eventToHotkey(e: KeyboardEvent): string | null {
  if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) {
    return null;
  }

  const parts: string[] = [];
  if (e.ctrlKey) parts.push("ctrl");
  if (e.shiftKey) parts.push("shift");
  if (e.altKey) parts.push("alt");
  if (e.metaKey) parts.push("cmd");

  let key = e.key.toLowerCase();
  if (key === " ") key = "space";
  if (key.length === 1) {
    parts.push(key);
  } else if (key.startsWith("arrow")) {
    parts.push(key.replace("arrow", ""));
  } else if (key === "escape") {
    parts.push("esc");
  } else {
    parts.push(key);
  }

  if (parts.length < 2) return null;
  return parts.join("+");
}

export const DEFAULT_SETTINGS = {
  region: null,
  hotkey_select: "ctrl+shift+r",
  hotkey_read: "ctrl+shift+t",
  hotkey_stop: "ctrl+shift+s",
  hotkey_quit: "ctrl+shift+q",
  hotkey_cycle: "ctrl+shift+v",
  tts_voice: "bf_emma",
  tts_speed: 1.0,
  active_voice: "narrator",
  prefetch_ocr: true,
} as const;
