import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import HomeTab from "./HomeTab";
import VoicesTab from "./VoicesTab";
import SettingsTab from "./SettingsTab";
import RegionOverlay from "./RegionOverlay";
import type { EngineStatus } from "./types";
import * as api from "./api";

type Tab = "home" | "voices" | "settings";

const TABS: { id: Tab; label: string }[] = [
  { id: "home", label: "Overview" },
  { id: "voices", label: "Voices" },
  { id: "settings", label: "Settings" },
];

export default function App() {
  const [tab, setTab] = useState<Tab>("home");
  const [status, setStatus] = useState<EngineStatus | null>(null);
  const [isOverlay, setIsOverlay] = useState(false);

  async function refresh() {
    try {
      setStatus(await api.getEngineStatus());
    } catch {
      try {
        setStatus(await api.initEngine());
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
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">Game Reader</div>
        <nav className="nav">
          {TABS.map(({ id, label }) => (
            <button
              key={id}
              className={`nav-item${tab === id ? " active" : ""}`}
              onClick={() => setTab(id)}
            >
              {label}
            </button>
          ))}
        </nav>
      </aside>

      <main className="content">
        {tab === "home" && <HomeTab status={status} onRefresh={refresh} />}
        {tab === "voices" && <VoicesTab status={status} onRefresh={refresh} />}
        {tab === "settings" && <SettingsTab status={status} onRefresh={refresh} />}
      </main>
    </div>
  );
}
