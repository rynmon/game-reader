import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import HomeTab from "./HomeTab";
import VoicesTab from "./VoicesTab";
import SettingsTab from "./SettingsTab";
import RegionOverlay from "./RegionOverlay";
import type { EngineStatus } from "./types";
import * as api from "./api";

type Tab = "home" | "voices" | "settings";

export default function App() {
  const [tab, setTab] = useState<Tab>("home");
  const [status, setStatus] = useState<EngineStatus | null>(null);
  const [isOverlay, setIsOverlay] = useState(false);

  async function refresh() {
    try {
      const s = await api.getEngineStatus();
      setStatus(s);
    } catch {
      try {
        const s = await api.initEngine();
        setStatus(s);
      } catch (e) {
        console.error(e);
      }
    }
  }

  useEffect(() => {
    const label = getCurrentWebviewWindow().label;
    if (label === "overlay") {
      setIsOverlay(true);
    } else {
      refresh();
    }
  }, []);

  if (isOverlay) {
    return <RegionOverlay />;
  }

  return (
    <div className="app">
      <header>
        <h1>Game Reader</h1>
        <span className="hotkeys">v2.0</span>
      </header>

      <div className="tabs">
        <button className={tab === "home" ? "active" : ""} onClick={() => setTab("home")}>
          Home
        </button>
        <button className={tab === "voices" ? "active" : ""} onClick={() => setTab("voices")}>
          Voices
        </button>
        <button className={tab === "settings" ? "active" : ""} onClick={() => setTab("settings")}>
          Settings
        </button>
      </div>

      {tab === "home" && <HomeTab status={status} onRefresh={refresh} />}
      {tab === "voices" && <VoicesTab status={status} onRefresh={refresh} />}
      {tab === "settings" && <SettingsTab status={status} onRefresh={refresh} />}
    </div>
  );
}
