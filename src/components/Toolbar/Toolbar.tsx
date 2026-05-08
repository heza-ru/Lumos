import { useToolState } from "../../hooks/useToolState";
import { ToolButton } from "./ToolButton";
import { ColorPicker } from "./ColorPicker";
import { TOOLS } from "../../types";

const DIVIDER = (
  <div style={{
    width: 1,
    height: 20,
    background: "rgba(255,255,255,0.15)",
    margin: "0 2px",
    flexShrink: 0,
  }} />
);

export function Toolbar() {
  const { activeTool, activeColor, selectTool, selectColor, undo, clear } = useToolState();

  return (
    <div
      data-tauri-drag-region
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 3,
        padding: "7px 10px",
        borderRadius: 14,
        background: "rgba(18, 18, 18, 0.85)",
        backdropFilter: "blur(24px)",
        WebkitBackdropFilter: "blur(24px)",
        boxShadow: "0 4px 28px rgba(0,0,0,0.45), inset 0 0 0 0.5px rgba(255,255,255,0.08)",
        userSelect: "none",
        WebkitUserSelect: "none",
        // Entire bar is a drag region; individual controls override with no-drag
        WebkitAppRegion: "drag",
      } as React.CSSProperties}
    >
      {TOOLS.map((t) => (
        <ToolButton
          key={t.kind}
          kind={t.kind}
          emoji={t.emoji}
          label={t.label}
          hotkey={t.hotkey}
          active={activeTool === t.kind}
          onClick={() => selectTool(t.kind)}
        />
      ))}

      {DIVIDER}

      <ColorPicker active={activeColor} onSelect={selectColor} />

      {DIVIDER}

      {/* Undo */}
      <button
        title="Undo (⌘Z)"
        onClick={undo}
        style={{
          background: "transparent",
          border: "none",
          color: "rgba(255,255,255,0.55)",
          cursor: "pointer",
          fontSize: 16,
          padding: "0 4px",
          lineHeight: 1,
          WebkitAppRegion: "no-drag",
        } as React.CSSProperties}
      >
        ↩
      </button>

      {/* Clear */}
      <button
        title="Clear all (⌘K)"
        onClick={clear}
        style={{
          background: "transparent",
          border: "none",
          color: "rgba(255,255,255,0.55)",
          cursor: "pointer",
          fontSize: 13,
          padding: "0 4px",
          lineHeight: 1,
          WebkitAppRegion: "no-drag",
        } as React.CSSProperties}
      >
        ✕
      </button>
    </div>
  );
}
