interface Props {
  sizePx: number;
  active: boolean;
  onSelect: () => void;
}

export function WidthDot({ sizePx, active, onSelect }: Props) {
  return (
    <button
      onClick={onSelect}
      title={`${sizePx}px`}
      style={{
        width: sizePx,
        height: sizePx,
        borderRadius: "50%",
        border: "none",
        cursor: "pointer",
        flexShrink: 0,
        padding: 0,
        background: active ? "#fff" : "rgba(255,255,255,0.40)",
        boxShadow: active ? "0 0 6px rgba(255,255,255,0.45)" : "none",
        transform: active ? "scale(1.25)" : "scale(1)",
        transition: "background 0.1s, transform 0.14s cubic-bezier(0.34,1.56,0.64,1), box-shadow 0.1s",
        WebkitAppRegion: "no-drag",
      } as React.CSSProperties}
    />
  );
}
