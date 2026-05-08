import { WidthDot } from "./WidthDot";
import { WIDTHS } from "../../types";
import type { StrokeWidth } from "../../types";

interface Props {
  active: StrokeWidth;
  onSelect: (w: StrokeWidth) => void;
}

export function WidthGroup({ active, onSelect }: Props) {
  return (
    <div
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 4,
        padding: "2px 8px",
        borderRadius: 100,
        background: "rgba(255,255,255,0.03)",
      }}
    >
      {WIDTHS.map(({ value, sizePx }) => (
        <WidthDot
          key={value}
          sizePx={sizePx}
          active={active === value}
          onSelect={() => onSelect(value)}
        />
      ))}
    </div>
  );
}
