export type ToolKind =
  | "pen" | "highlighter" | "arrow" | "rectangle"
  | "ellipse" | "line" | "text" | "laser" | "eraser";

export type StrokeWidth = "thin" | "medium" | "bold" | "extra_bold";

export type CursorEffect = "none" | "glow" | "ring" | "pulse";

export interface Color {
  r: number;
  g: number;
  b: number;
}

export interface AppSnapshot {
  overlay_visible: boolean;
  click_through: boolean;
  active_tool: ToolKind;
}

// Orange replaces Black — matches the design spec palette
export const PRESET_COLORS: { label: string; color: Color }[] = [
  { label: "Blue",   color: { r: 82,  g: 155, b: 224 } },
  { label: "Red",    color: { r: 224, g: 82,  b: 82  } },
  { label: "Green",  color: { r: 82,  g: 224, b: 108 } },
  { label: "Orange", color: { r: 224, g: 165, b: 82  } },
  { label: "White",  color: { r: 255, g: 255, b: 255 } },
];

// 7 toolbar tools — no emoji field (Lucide icons used in ToolButton)
export const TOOLS: { kind: ToolKind; label: string; hotkey: string }[] = [
  { kind: "pen",         label: "Pen",         hotkey: "P" },
  { kind: "highlighter", label: "Highlighter", hotkey: "H" },
  { kind: "arrow",       label: "Arrow",       hotkey: "A" },
  { kind: "rectangle",   label: "Rectangle",   hotkey: "R" },
  { kind: "ellipse",     label: "Ellipse",     hotkey: "E" },
  { kind: "laser",       label: "Laser",       hotkey: "L" },
  { kind: "eraser",      label: "Eraser",      hotkey: "X" },
];

export const WIDTHS: { value: StrokeWidth; label: string; sizePx: number }[] = [
  { value: "thin",       label: "Thin",       sizePx: 5  },
  { value: "medium",     label: "Medium",     sizePx: 7  },
  { value: "bold",       label: "Bold",       sizePx: 9  },
  { value: "extra_bold", label: "Extra Bold", sizePx: 11 },
];
