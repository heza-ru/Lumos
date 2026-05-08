import type { Color } from "../../types";
import { PRESET_COLORS } from "../../types";

interface Props {
  active: Color;
  onSelect: (c: Color) => void;
}

function colorsMatch(a: Color, b: Color) {
  return a.r === b.r && a.g === b.g && a.b === b.b;
}

export function ColorPicker({ active, onSelect }: Props) {
  return (
    <div style={{
      display: "flex",
      gap: 5,
      alignItems: "center",
      WebkitAppRegion: "no-drag",
    } as React.CSSProperties}>
      {PRESET_COLORS.map(({ label, color }) => (
        <button
          key={label}
          title={label}
          onClick={() => onSelect(color)}
          style={{
            width: 14,
            height: 14,
            borderRadius: "50%",
            border: colorsMatch(color, active)
              ? "2px solid white"
              : "2px solid rgba(255,255,255,0.3)",
            background: `rgb(${color.r},${color.g},${color.b})`,
            cursor: "pointer",
            padding: 0,
            flexShrink: 0,
          }}
        />
      ))}
    </div>
  );
}
