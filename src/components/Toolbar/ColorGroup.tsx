import { ColorDot } from "./ColorDot";
import { PRESET_COLORS } from "../../types";
import type { Color } from "../../types";

interface Props {
  active: Color;
  onSelect: (c: Color) => void;
}

function colorsMatch(a: Color, b: Color) {
  return a.r === b.r && a.g === b.g && a.b === b.b;
}

export function ColorGroup({ active, onSelect }: Props) {
  return (
    <div
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 5,
        padding: "2px 7px",
        borderRadius: 100,
        background: "rgba(255,255,255,0.03)",
      }}
    >
      {PRESET_COLORS.map(({ label, color }) => (
        <ColorDot
          key={label}
          color={color}
          label={label}
          active={colorsMatch(color, active)}
          onSelect={() => onSelect(color)}
        />
      ))}
    </div>
  );
}
