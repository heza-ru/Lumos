import {
  Pencil,
  Highlighter,
  ArrowUpRight,
  RectangleHorizontal,
  Zap,
  Eraser,
  type LucideIcon,
} from "lucide-react";
import type { ToolKind } from "../../types";

// Custom ellipse icon — Lucide has no horizontal ellipse
function EllipseIcon({ size = 16, strokeWidth = 2 }: { size?: number; strokeWidth?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round">
      <ellipse cx="12" cy="12" rx="10" ry="7" />
    </svg>
  );
}

const ICON_MAP: Record<ToolKind, LucideIcon | typeof EllipseIcon> = {
  pen:         Pencil,
  highlighter: Highlighter,
  arrow:       ArrowUpRight,
  rectangle:   RectangleHorizontal,
  ellipse:     EllipseIcon,
  line:        ArrowUpRight,
  text:        Pencil,
  laser:       Zap,
  eraser:      Eraser,
};

interface Props {
  kind: ToolKind;
  label: string;
  hotkey: string;
  active: boolean;
  onClick: () => void;
}

export function ToolButton({ kind, label, hotkey, active, onClick }: Props) {
  const Icon = ICON_MAP[kind];

  return (
    <button
      title={`${label} (${hotkey})`}
      onClick={onClick}
      onMouseEnter={e => {
        if (!active) {
          const btn = e.currentTarget as HTMLButtonElement;
          btn.style.background = "rgba(255,255,255,0.11)";
          btn.style.color = "rgba(255,255,255,0.92)";
          btn.style.transform = "scale(1.10)";
        }
      }}
      onMouseLeave={e => {
        if (!active) {
          const btn = e.currentTarget as HTMLButtonElement;
          btn.style.background = "transparent";
          btn.style.color = "rgba(255,255,255,0.55)";
          btn.style.transform = "scale(1)";
        }
      }}
      style={{
        width: 34,
        height: 34,
        borderRadius: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        border: "none",
        cursor: "pointer",
        flexShrink: 0,
        background: active ? "rgba(255,255,255,0.17)" : "transparent",
        color: active ? "#fff" : "rgba(255,255,255,0.55)",
        boxShadow: active
          ? "inset 0 1px 2px rgba(0,0,0,0.22), inset 0 0 0 0.5px rgba(255,255,255,0.26)"
          : "none",
        transform: active ? "scale(0.96)" : "scale(1)",
        transition: "background 0.12s, color 0.12s, transform 0.16s cubic-bezier(0.34, 1.56, 0.64, 1)",
        WebkitAppRegion: "no-drag",
      } as React.CSSProperties}
    >
      <Icon size={16} strokeWidth={2} />
    </button>
  );
}
