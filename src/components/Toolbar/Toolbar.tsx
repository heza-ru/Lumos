import { useEffect, useState } from "react";
import { ToolButton } from "./ToolButton";
import { ColorGroup } from "./ColorGroup";
import { WidthGroup } from "./WidthGroup";
import { ActionButton } from "./ActionButton";
import { ModeChip } from "./ModeChip";
import { Divider } from "./Divider";
import { Undo2, Trash2 } from "lucide-react";
import { useToolState } from "../../hooks/useToolState";
import { TOOLS } from "../../types";
import styles from "./Toolbar.module.css";
import { GlassMaterialVariant } from "tauri-plugin-liquid-glass-api";

export function Toolbar() {
  const { activeTool, activeColor, activeWidth, selectTool, selectColor, selectWidth, undo, clear } = useToolState();
  const [isNativeGlass, setIsNativeGlass] = useState(false);

  useEffect(() => {
    applyGlass();
  }, []);

  async function applyGlass() {
    try {
      const { setLiquidGlassEffect } = await import("tauri-plugin-liquid-glass-api");
      await setLiquidGlassEffect({ cornerRadius: 100, variant: GlassMaterialVariant.Regular });
      setIsNativeGlass(true);
    } catch {
      setIsNativeGlass(false);
    }
  }

  return (
    <div
      data-tauri-drag-region
      className={`${styles.pill} ${isNativeGlass ? "" : styles.fallback}`}
    >
      <div className={styles.groupTools}>
        {TOOLS.map((t) => (
          <ToolButton
            key={t.kind}
            kind={t.kind}
            label={t.label}
            hotkey={t.hotkey}
            active={activeTool === t.kind}
            onClick={() => selectTool(t.kind)}
          />
        ))}
      </div>

      <Divider />
      <ColorGroup active={activeColor} onSelect={selectColor} />
      <Divider />
      <WidthGroup active={activeWidth} onSelect={selectWidth} />
      <Divider />

      <div className={styles.groupActions}>
        <ActionButton title="Undo (⌘Z)" onClick={undo}>
          <Undo2 size={15} strokeWidth={2} />
        </ActionButton>
        <ActionButton title="Clear all (⌘K)" onClick={clear}>
          <Trash2 size={15} strokeWidth={2} />
        </ActionButton>
      </div>

      <Divider />
      <ModeChip />
    </div>
  );
}
