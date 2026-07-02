export interface VoiceCatalogEntry {
  id: string;
  label: string;
  description?: string;
  release_tag: string;
  asset: string;
  size_bytes: number;
  f0up_key: number;
  speed: number;
  recommended?: boolean;
}

export interface GpuInfo {
  available: boolean;
  device_name: string | null;
  cuda_version: string | null;
  error: string | null;
}

export interface EngineStatus {
  initialized: boolean;
  init_error: string | null;
  active_voice: string | null;
  active_label: string;
  installed_voices: string[];
  voice_labels: Record<string, string>;
  catalog: VoiceCatalogEntry[];
  kokoro_installed: boolean;
  gpu?: GpuInfo;
  region?: { x: number; y: number; w: number; h: number } | null;
  settings?: Record<string, unknown>;
}

export interface DownloadProgress {
  id: string;
  kind: "voice" | "kokoro";
  downloaded: number;
  total: number;
  status: "downloading" | "complete" | "error";
  error?: string;
}

export interface AppSettings {
  region: { x: number; y: number; w: number; h: number } | null;
  hotkey_select: string;
  hotkey_read: string;
  hotkey_stop: string;
  hotkey_quit: string;
  hotkey_cycle: string;
  tts_voice: string;
  tts_speed: number;
  active_voice: string;
  prefetch_ocr: boolean;
}
