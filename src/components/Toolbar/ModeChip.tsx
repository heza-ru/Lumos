import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

export function ModeChip() {
  const [isDrawMode, setIsDrawMode] = useState(true);

  const toggle = async () => {
    await invoke("toggle_click_through").catch(console.error);
    setIsDrawMode(prev => !prev);
  };

  return (
    <button
      onClick={toggle}
      title={isDrawMode ? "Switch to pointer mode (⌘D)" : "Switch to draw mode (⌘D)"}
      style={{
        height: 26,
        padding: "0 12px",
        borderRadius: 100,
        display: "inline-flex",
        alignItems: "center",
        fontSize: 10,
        fontWeight: 700,
        letterSpacing: "0.09em",
        textTransform: "uppercase",
        cursor: "pointer",
        marginLeft: 2,
        flexShrink: 0,
        border: `0.5px solid ${isDrawMode ? "rgba(255,255,255,0.16)" : "rgba(82,224,108,0.22)"}`,
        background: isDrawMode ? "rgba(255,255,255,0.09)" : "rgba(82,224,108,0.12)",
        color: isDrawMode ? "rgba(255,255,255,0.52)" : "rgba(120,220,140,0.90)",
        boxShadow: "inset 0 0.5px 0 rgba(255,255,255,0.20)",
        transition: "background 0.15s, color 0.15s, border-color 0.15s",
        WebkitAppRegion: "no-drag",
      } as React.CSSProperties}
    >
      {isDrawMode ? "Draw" : "Point"}
    </button>
  );
}
