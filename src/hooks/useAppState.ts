import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSnapshot } from "../types";

export function useAppState() {
  const [snapshot, setSnapshot] = useState<AppSnapshot>({
    overlay_visible: false,
    click_through: true,
    active_tool: "pen",
  });

  const refresh = async () => {
    try {
      const s = await invoke<AppSnapshot>("get_app_state");
      setSnapshot(s);
    } catch (e) {
      console.error("Failed to get app state:", e);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  return { snapshot, refresh };
}
