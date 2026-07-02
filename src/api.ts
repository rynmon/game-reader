import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppSettings, DownloadProgress, EngineStatus, GpuInfo, VoiceCatalogEntry } from "./types";

export async function getEngineStatus(): Promise<EngineStatus> {
  return invoke("get_engine_status");
}

export async function initEngine(): Promise<EngineStatus> {
  return invoke("init_engine");
}

export async function readRegion(): Promise<void> {
  return invoke("read_region");
}

export async function stopSpeech(): Promise<void> {
  return invoke("stop_speech");
}

export async function cycleVoice(): Promise<void> {
  return invoke("cycle_voice");
}

export async function setActiveVoice(voiceId: string): Promise<void> {
  return invoke("set_active_voice", { voiceId });
}

export async function openRegionSelector(): Promise<void> {
  return invoke("open_region_selector");
}

export async function getVoiceCatalog(): Promise<VoiceCatalogEntry[]> {
  return invoke("get_voice_catalog");
}

export async function downloadVoice(voiceId: string): Promise<void> {
  return invoke("download_voice", { voiceId });
}

export async function deleteVoice(voiceId: string): Promise<void> {
  return invoke("delete_voice", { voiceId });
}

export async function downloadKokoro(): Promise<void> {
  return invoke("download_kokoro");
}

export async function getGpuInfo(): Promise<GpuInfo> {
  return invoke("get_gpu_info");
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: Partial<AppSettings>): Promise<AppSettings> {
  const result = await invoke<AppSettings>("save_settings", { settings });
  return result;
}

export async function getStorageUsage(): Promise<Record<string, number>> {
  return invoke("get_storage_usage");
}

export function onDownloadProgress(callback: (p: DownloadProgress) => void) {
  return listen<DownloadProgress>("download-progress", (event) => callback(event.payload));
}

export function onEngineEvent(callback: (event: string) => void) {
  return listen<string>("engine-event", (event) => callback(event.payload));
}
