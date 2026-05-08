import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ToolKind, StrokeWidth, Color } from "../types";
import { PRESET_COLORS } from "../types";

export function useToolState() {
  const [activeTool, setActiveTool]   = useState<ToolKind>("pen");
  const [activeWidth, setActiveWidth] = useState<StrokeWidth>("medium");
  const [activeColor, setActiveColor] = useState<Color>(PRESET_COLORS[0].color);

  const selectTool = async (tool: ToolKind) => {
    try {
      await invoke("set_tool", { tool });
      setActiveTool(tool);
    } catch (e) {
      console.error("set_tool failed:", e);
    }
  };

  const selectWidth = async (width: StrokeWidth) => {
    try {
      await invoke("set_width", { width });
      setActiveWidth(width);
    } catch (e) {
      console.error("set_width failed:", e);
    }
  };

  const selectColor = async (color: Color) => {
    try {
      await invoke("set_color", { r: color.r, g: color.g, b: color.b });
      setActiveColor(color);
    } catch (e) {
      console.error("set_color failed:", e);
    }
  };

  const undo  = () => invoke("undo").catch(console.error);
  const clear = () => invoke("clear_all").catch(console.error);

  return {
    activeTool,
    activeWidth,
    activeColor,
    selectTool,
    selectWidth,
    selectColor,
    undo,
    clear,
  };
}
