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

export const PRESET_COLORS: { label: string; color: Color }[] = [
  { label: "Blue",   color: { r: 82,  g: 155, b: 224 } },
  { label: "Red",    color: { r: 224, g: 82,  b: 82  } },
  { label: "Green",  color: { r: 82,  g: 224, b: 108 } },
  { label: "White",  color: { r: 255, g: 255, b: 255 } },
  { label: "Black",  color: { r: 30,  g: 30,  b: 30  } },
];

export const TOOLS: { kind: ToolKind; label: string; hotkey: string; emoji: string }[] = [
  { kind: "pen",         label: "Pen",         hotkey: "P", emoji: "✏️" },
  { kind: "highlighter", label: "Highlighter", hotkey: "H", emoji: "🖊" },
  { kind: "arrow",       label: "Arrow",       hotkey: "A", emoji: "↗" },
  { kind: "rectangle",   label: "Rectangle",   hotkey: "R", emoji: "▭" },
  { kind: "ellipse",     label: "Ellipse",     hotkey: "E", emoji: "⬭" },
  { kind: "line",        label: "Line",        hotkey: "/", emoji: "╱" },
  { kind: "laser",       label: "Laser",       hotkey: "L", emoji: "⚡" },
  { kind: "eraser",      label: "Eraser",      hotkey: "X", emoji: "⌫" },
];

export const WIDTHS: { value: StrokeWidth; label: string }[] = [
  { value: "thin",       label: "Thin" },
  { value: "medium",     label: "Medium" },
  { value: "bold",       label: "Bold" },
  { value: "extra_bold", label: "X-Bold" },
];
