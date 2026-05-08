import type { ToolKind } from "../../types";

interface Props {
  kind: ToolKind;
  emoji: string;
  label: string;
  hotkey: string;
  active: boolean;
  onClick: () => void;
}

export function ToolButton({ emoji, label, hotkey, active, onClick }: Props) {
  return (
    <button
      title={`${label} (${hotkey})`}
      onClick={onClick}
      style={{
        width: 34,
        height: 34,
        borderRadius: 8,
        border: "none",
        cursor: "pointer",
        fontSize: 14,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: active ? "rgba(255,255,255,0.22)" : "transparent",
        color: "white",
        transition: "background 0.1s ease",
        WebkitAppRegion: "no-drag",
      } as React.CSSProperties}
    >
      {emoji}
    </button>
  );
}
