import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface Point {
  x: number;
  y: number;
}

export default function RegionOverlay() {
  const [start, setStart] = useState<Point | null>(null);
  const [current, setCurrent] = useState<Point | null>(null);
  const dragging = useRef(false);

  useEffect(() => {
    document.body.classList.add("overlay-root");
    return () => document.body.classList.remove("overlay-root");
  }, []);

  const rect = start && current ? normalizeRect(start, current) : null;

  async function finish(region: { x: number; y: number; w: number; h: number } | null) {
    await invoke("complete_region_selection", { region });
    await getCurrentWindow().close();
  }

  function onMouseDown(e: React.MouseEvent) {
    dragging.current = true;
    setStart({ x: e.clientX, y: e.clientY });
    setCurrent({ x: e.clientX, y: e.clientY });
  }

  function onMouseMove(e: React.MouseEvent) {
    if (!dragging.current) return;
    setCurrent({ x: e.clientX, y: e.clientY });
  }

  async function onMouseUp(e: React.MouseEvent) {
    if (!dragging.current || !start) return;
    dragging.current = false;
    const r = normalizeRect(start, { x: e.clientX, y: e.clientY });
    if (r.w > 5 && r.h > 5) {
      const scale = await invoke<number>("get_dpi_scale");
      await finish({
        x: Math.round(r.x * scale),
        y: Math.round(r.y * scale),
        w: Math.round(r.w * scale),
        h: Math.round(r.h * scale),
      });
    } else {
      await finish(null);
    }
  }

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        void finish(null);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div
      className="overlay-root"
      onMouseDown={onMouseDown}
      onMouseMove={onMouseMove}
      onMouseUp={onMouseUp}
    >
      <div className="overlay-hint">Drag to select region | Escape to cancel</div>
      {rect && (
        <div
          className="overlay-rect"
          style={{ left: rect.x, top: rect.y, width: rect.w, height: rect.h }}
        />
      )}
    </div>
  );
}

function normalizeRect(a: Point, b: Point) {
  const x = Math.min(a.x, b.x);
  const y = Math.min(a.y, b.y);
  const w = Math.abs(a.x - b.x);
  const h = Math.abs(a.y - b.y);
  return { x, y, w, h };
}
