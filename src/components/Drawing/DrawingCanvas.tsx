import { useEffect, useRef, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ToolKind, Color, StrokeWidth } from "../../types";
import { PRESET_COLORS } from "../../types";

interface Stroke {
  tool: ToolKind;
  color: Color;
  widthPx: number;
  points: { x: number; y: number }[];
  startTime: number; // for laser fade
}

const PEN_WIDTHS: Record<StrokeWidth, number> = {
  thin: 4, medium: 8, bold: 12, extra_bold: 16,
};
const HIGHLIGHTER_WIDTHS: Record<StrokeWidth, number> = {
  thin: 8, medium: 16, bold: 24, extra_bold: 32,
};

export function DrawingCanvas() {
  const canvasRef  = useRef<HTMLCanvasElement>(null);
  const stateRef   = useRef({
    tool:      "pen" as ToolKind,
    color:     PRESET_COLORS[0].color,
    widthKey:  "medium" as StrokeWidth,
    strokes:   [] as Stroke[],
    liveStroke: null as Stroke | null,
    isDrawing: false,
  });
  const rafRef = useRef<number>(0);

  // ── Draw all strokes onto the canvas ──────────────────────────────────────
  const render = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    const now = Date.now();
    const allStrokes = [
      ...stateRef.current.strokes,
      ...(stateRef.current.liveStroke ? [stateRef.current.liveStroke] : []),
    ];

    for (const stroke of allStrokes) {
      drawStroke(ctx, stroke, now);
    }

    rafRef.current = requestAnimationFrame(render);
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    // Match canvas pixel size to actual display pixels (Retina)
    const dpr = window.devicePixelRatio || 1;
    canvas.width  = window.innerWidth  * dpr;
    canvas.height = window.innerHeight * dpr;
    const ctx = canvas.getContext("2d");
    if (ctx) ctx.scale(dpr, dpr);
    rafRef.current = requestAnimationFrame(render);
    return () => cancelAnimationFrame(rafRef.current);
  }, [render]);

  // ── Listen for toolbar events ─────────────────────────────────────────────
  useEffect(() => {
    const unlisteners: Promise<() => void>[] = [
      listen<{ tool: ToolKind }>("tool-changed",   e => { stateRef.current.tool = e.payload.tool; }),
      listen<{ r: number; g: number; b: number }>("color-changed", e => { stateRef.current.color = e.payload; }),
      listen<{ width: StrokeWidth }>("width-changed", e => { stateRef.current.widthKey = e.payload.width; }),
      listen("clear-all", () => { stateRef.current.strokes = []; stateRef.current.liveStroke = null; }),
      listen("undo",      () => { stateRef.current.strokes = stateRef.current.strokes.slice(0, -1); }),
    ];
    return () => { unlisteners.forEach(p => p.then(f => f())); };
  }, []);

  // ── Mouse event handlers ──────────────────────────────────────────────────
  const getPos = (e: React.MouseEvent) => ({ x: e.clientX, y: e.clientY });

  const onMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    const s = stateRef.current;
    s.isDrawing = true;
    const widthPx = s.tool === "highlighter"
      ? HIGHLIGHTER_WIDTHS[s.widthKey]
      : PEN_WIDTHS[s.widthKey];
    s.liveStroke = {
      tool: s.tool,
      color: s.color,
      widthPx,
      points: [getPos(e)],
      startTime: Date.now(),
    };
  };

  const onMouseMove = (e: React.MouseEvent) => {
    if (!stateRef.current.isDrawing || !stateRef.current.liveStroke) return;
    stateRef.current.liveStroke.points.push(getPos(e));
  };

  const onMouseUp = () => {
    const s = stateRef.current;
    if (!s.isDrawing) return;
    s.isDrawing = false;
    if (s.liveStroke && s.liveStroke.points.length >= 2) {
      s.strokes.push(s.liveStroke);
    }
    s.liveStroke = null;
  };

  return (
    <canvas
      ref={canvasRef}
      onMouseDown={onMouseDown}
      onMouseMove={onMouseMove}
      onMouseUp={onMouseUp}
      onMouseLeave={onMouseUp}
      style={{
        position: "fixed",
        top: 0, left: 0,
        width: "100vw", height: "100vh",
        cursor: "crosshair",
        touchAction: "none",
      }}
    />
  );
}

// ── Stroke rendering ──────────────────────────────────────────────────────────
function drawStroke(ctx: CanvasRenderingContext2D, stroke: Stroke, now: number) {
  if (stroke.points.length < 2) return;

  const { tool, color, widthPx, points, startTime } = stroke;

  let alpha = 1;
  if (tool === "laser") {
    const age = now - startTime;
    const FADE_MS = 2000;
    if (age >= FADE_MS) return;
    alpha = 1 - age / FADE_MS;
  }
  if (tool === "highlighter") alpha = 0.35;

  const rgba = `rgba(${color.r},${color.g},${color.b},${alpha})`;

  ctx.save();
  ctx.strokeStyle = rgba;
  ctx.lineWidth   = widthPx;
  ctx.lineCap     = "round";
  ctx.lineJoin    = "round";

  if (tool === "pen" || tool === "highlighter" || tool === "laser") {
    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);
    // Smooth freehand via quadratic bezier through midpoints
    for (let i = 1; i < points.length - 1; i++) {
      const mx = (points[i].x + points[i + 1].x) / 2;
      const my = (points[i].y + points[i + 1].y) / 2;
      ctx.quadraticCurveTo(points[i].x, points[i].y, mx, my);
    }
    ctx.lineTo(points[points.length - 1].x, points[points.length - 1].y);
    ctx.stroke();

  } else if (tool === "arrow") {
    const from = points[0];
    const to   = points[points.length - 1];
    ctx.fillStyle = rgba;
    ctx.beginPath();
    ctx.moveTo(from.x, from.y);
    ctx.lineTo(to.x, to.y);
    ctx.stroke();
    // Arrowhead
    const angle = Math.atan2(to.y - from.y, to.x - from.x);
    const headLen = widthPx * 4;
    const spread = Math.PI / 6;
    ctx.beginPath();
    ctx.moveTo(to.x, to.y);
    ctx.lineTo(to.x - headLen * Math.cos(angle - spread), to.y - headLen * Math.sin(angle - spread));
    ctx.lineTo(to.x - headLen * Math.cos(angle + spread), to.y - headLen * Math.sin(angle + spread));
    ctx.closePath();
    ctx.fill();

  } else if (tool === "rectangle") {
    const [p1, p2] = [points[0], points[points.length - 1]];
    ctx.strokeRect(
      Math.min(p1.x, p2.x), Math.min(p1.y, p2.y),
      Math.abs(p2.x - p1.x), Math.abs(p2.y - p1.y)
    );

  } else if (tool === "ellipse") {
    const [p1, p2] = [points[0], points[points.length - 1]];
    const cx = (p1.x + p2.x) / 2, cy = (p1.y + p2.y) / 2;
    const rx = Math.abs(p2.x - p1.x) / 2, ry = Math.abs(p2.y - p1.y) / 2;
    ctx.beginPath();
    ctx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
    ctx.stroke();

  } else if (tool === "eraser") {
    ctx.globalCompositeOperation = "destination-out";
    ctx.strokeStyle = "rgba(0,0,0,1)";
    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);
    for (const p of points.slice(1)) ctx.lineTo(p.x, p.y);
    ctx.stroke();
    ctx.globalCompositeOperation = "source-over";
  }

  ctx.restore();
}
