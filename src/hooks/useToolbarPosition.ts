import { useEffect } from "react";
import {
  getCurrentWindow,
  LogicalPosition,
  currentMonitor,
} from "@tauri-apps/api/window";
import { load } from "@tauri-apps/plugin-store";

const STORE_KEY = "toolbar_position";
const WINDOW_W = 560;
const WINDOW_H = 72;
const BOTTOM_OFFSET = 80;

async function getBottomCenterPosition(): Promise<LogicalPosition> {
  const monitor = await currentMonitor();
  if (!monitor) return new LogicalPosition(100, 100);

  const scaleFactor = monitor.scaleFactor;
  const screenW = monitor.size.width / scaleFactor;
  const screenH = monitor.size.height / scaleFactor;

  const x = Math.round((screenW - WINDOW_W) / 2);
  const y = Math.round(screenH - WINDOW_H - BOTTOM_OFFSET);
  return new LogicalPosition(x, y);
}

export function useToolbarPosition() {
  useEffect(() => {
    positionOnMount();
  }, []);

  async function positionOnMount() {
    const win = getCurrentWindow();

    try {
      const store = await load("settings.json", {
        defaults: {},
        autoSave: false,
      });
      const saved = await store.get<{ x: number; y: number }>(STORE_KEY);

      if (saved) {
        await win.setPosition(new LogicalPosition(saved.x, saved.y));
      } else {
        const pos = await getBottomCenterPosition();
        await win.setPosition(pos);
      }
    } catch {
      const pos = await getBottomCenterPosition().catch(
        () => new LogicalPosition(100, 100)
      );
      await win.setPosition(pos).catch(() => {});
    }

    await win.show().catch(() => {});
  }

  async function savePosition() {
    try {
      const win = getCurrentWindow();
      const pos = await win.outerPosition();
      const monitor = await currentMonitor();
      const scale = monitor?.scaleFactor ?? 1;

      const store = await load("settings.json", {
        defaults: {},
        autoSave: false,
      });
      await store.set(STORE_KEY, { x: pos.x / scale, y: pos.y / scale });
      await store.save();
    } catch {
      // Non-critical — silently ignore if outside Tauri context
    }
  }

  return { savePosition };
}
