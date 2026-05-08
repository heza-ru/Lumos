import type { ReactNode } from "react";

interface Props {
  title: string;
  onClick: () => void;
  children: ReactNode;
}

export function ActionButton({ title, onClick, children }: Props) {
  return (
    <button
      title={title}
      onClick={onClick}
      onMouseEnter={e => {
        const btn = e.currentTarget as HTMLButtonElement;
        btn.style.background = "rgba(255,255,255,0.09)";
        btn.style.color = "rgba(255,255,255,0.80)";
      }}
      onMouseLeave={e => {
        const btn = e.currentTarget as HTMLButtonElement;
        btn.style.background = "transparent";
        btn.style.color = "rgba(255,255,255,0.40)";
      }}
      style={{
        width: 32,
        height: 32,
        borderRadius: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        border: "none",
        cursor: "pointer",
        background: "transparent",
        color: "rgba(255,255,255,0.40)",
        transition: "background 0.12s, color 0.12s, transform 0.14s cubic-bezier(0.34,1.56,0.64,1)",
        WebkitAppRegion: "no-drag",
      } as React.CSSProperties}
    >
      {children}
    </button>
  );
}
