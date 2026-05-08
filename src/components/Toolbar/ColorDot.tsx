import type { Color } from "../../types";

interface Props {
  color: Color;
  label: string;
  active: boolean;
  onSelect: () => void;
}

export function ColorDot({ color, label, active, onSelect }: Props) {
  return (
    <button
      title={label}
      onClick={onSelect}
      style={{
        width: 12,
        height: 12,
        borderRadius: "50%",
        border: `1.5px solid ${active ? "rgba(255,255,255,0.80)" : "transparent"}`,
        background: `rgb(${color.r},${color.g},${color.b})`,
        cursor: "pointer",
        padding: 0,
        flexShrink: 0,
        transition: "transform 0.14s cubic-bezier(0.34,1.56,0.64,1), border-color 0.12s",
        transform: active ? "scale(1.20)" : "scale(1)",
        WebkitAppRegion: "no-drag",
      } as React.CSSProperties}
    />
  );
}
