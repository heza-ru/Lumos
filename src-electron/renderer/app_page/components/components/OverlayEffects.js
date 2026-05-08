import React, { useEffect, useRef } from 'react';

/**
 * OverlayEffects — renders spotlight, zoom lens, and cursor glow
 * on a transparent canvas layered above the main drawing canvas.
 */
const OverlayEffects = ({
  mouseCoordinates,
  spotlightActive,
  spotlightShape,    // 'circle' | 'rectangle'
  spotlightRadius,   // number (logical px)
  cursorHighlight,   // 'none' | 'glow' | 'ring'
  zoomActive,
  zoomFactor,
  zoomRadius,
  screenshotCanvas,  // ref to main canvas (for zoom source)
}) => {
  const canvasRef = useRef(null);
  const dpr = window.devicePixelRatio || 1;
  const animFrameRef = useRef(null);
  const timeRef = useRef(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    canvas.width  = Math.floor(window.innerWidth  * dpr);
    canvas.height = Math.floor(window.innerHeight * dpr);
    const ctx = canvas.getContext('2d');
    ctx.scale(dpr, dpr);
  }, []);

  useEffect(() => {
    const render = (ts) => {
      timeRef.current = ts / 1000; // seconds
      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext('2d');
      const W = window.innerWidth;
      const H = window.innerHeight;
      const { x, y } = mouseCoordinates;

      ctx.clearRect(0, 0, W, H);

      // ── Spotlight ──────────────────────────────────────────────
      if (spotlightActive) {
        ctx.save();
        // Dark dim overlay
        ctx.fillStyle = 'rgba(0, 0, 0, 0.65)';
        ctx.fillRect(0, 0, W, H);

        // Cut hole using destination-out
        ctx.globalCompositeOperation = 'destination-out';
        ctx.fillStyle = 'rgba(0,0,0,1)';

        if (spotlightShape === 'circle') {
          ctx.beginPath();
          ctx.arc(x, y, spotlightRadius, 0, Math.PI * 2);
          ctx.fill();
        } else {
          // Rectangle spotlight
          const hw = spotlightRadius * 1.6;
          const hh = spotlightRadius;
          ctx.beginPath();
          roundRect(ctx, x - hw, y - hh, hw * 2, hh * 2, 16);
          ctx.fill();
        }

        ctx.globalCompositeOperation = 'source-over';

        // Soft edge glow around the hole
        const gradient = ctx.createRadialGradient(
          x, y, spotlightRadius * 0.85,
          x, y, spotlightRadius * 1.1
        );
        gradient.addColorStop(0, 'rgba(0,0,0,0)');
        gradient.addColorStop(1, 'rgba(0,0,0,0.4)');
        ctx.fillStyle = gradient;
        ctx.beginPath();
        ctx.arc(x, y, spotlightRadius * 1.1, 0, Math.PI * 2);
        ctx.fill();

        ctx.restore();
      }

      // ── Cursor highlight (Presentify-style: big pink hollow ring) ─────────────
      if (cursorHighlight !== 'none') {
        ctx.save();
        if (cursorHighlight === 'ring') {
          // Presentify default: large hollow ring with soft glow
          // Outer glow
          ctx.beginPath();
          ctx.arc(x, y, 26, 0, Math.PI * 2);
          ctx.strokeStyle = 'rgba(255, 105, 180, 0.25)'; // pink glow
          ctx.lineWidth = 8;
          ctx.stroke();
          // Main ring — big, hot pink, hollow
          ctx.beginPath();
          ctx.arc(x, y, 22, 0, Math.PI * 2);
          ctx.strokeStyle = 'rgba(255, 80, 160, 0.85)'; // hot pink like Presentify
          ctx.lineWidth = 2.5;
          ctx.stroke();
        } else if (cursorHighlight === 'glow') {
          // Radial glow variant
          for (let i = 0; i < 3; i++) {
            const r = 16 + i * 10;
            const alpha = 0.16 - i * 0.04;
            ctx.beginPath();
            ctx.arc(x, y, r, 0, Math.PI * 2);
            ctx.fillStyle = `rgba(82, 155, 224, ${alpha})`;
            ctx.fill();
          }
        }
        ctx.restore();
      }

      // ── Zoom lens ──────────────────────────────────────────────
      if (zoomActive && screenshotCanvas && screenshotCanvas.current) {
        const src = screenshotCanvas.current;
        const lensR = zoomRadius;
        const srcSize = (lensR * 2) / zoomFactor;
        const srcX = x - srcSize / 2;
        const srcY = y - srcSize / 2;

        // Position lens above-right of cursor
        const lensX = Math.min(x + 30, W - lensR * 2 - 20);
        const lensY = Math.max(y - lensR * 2 - 30, 20);

        ctx.save();
        ctx.beginPath();
        ctx.arc(lensX + lensR, lensY + lensR, lensR, 0, Math.PI * 2);
        ctx.clip();

        // Draw zoomed region from main annotation canvas
        ctx.drawImage(
          src,
          srcX * dpr, srcY * dpr, srcSize * dpr, srcSize * dpr,
          lensX, lensY, lensR * 2, lensR * 2
        );

        ctx.restore();

        // Lens border
        ctx.beginPath();
        ctx.arc(lensX + lensR, lensY + lensR, lensR, 0, Math.PI * 2);
        ctx.strokeStyle = 'rgba(255,255,255,0.55)';
        ctx.lineWidth = 2;
        ctx.stroke();

        // Crosshair at center
        ctx.strokeStyle = 'rgba(255,255,255,0.4)';
        ctx.lineWidth = 1;
        const cx = lensX + lensR, cy = lensY + lensR;
        ctx.beginPath();
        ctx.moveTo(cx - 8, cy); ctx.lineTo(cx + 8, cy);
        ctx.moveTo(cx, cy - 8); ctx.lineTo(cx, cy + 8);
        ctx.stroke();
      }

      animFrameRef.current = requestAnimationFrame(render);
    };

    animFrameRef.current = requestAnimationFrame(render);
    return () => cancelAnimationFrame(animFrameRef.current);
  }, [mouseCoordinates, spotlightActive, spotlightShape, spotlightRadius,
      cursorHighlight, zoomActive, zoomFactor, zoomRadius, screenshotCanvas]);

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: 'fixed',
        top: 0, left: 0,
        width: '100vw', height: '100vh',
        pointerEvents: 'none',
        zIndex: 10,
      }}
    />
  );
};

// Helper: rounded rect path
function roundRect(ctx, x, y, w, h, r) {
  ctx.moveTo(x + r, y);
  ctx.lineTo(x + w - r, y);
  ctx.quadraticCurveTo(x + w, y, x + w, y + r);
  ctx.lineTo(x + w, y + h - r);
  ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
  ctx.lineTo(x + r, y + h);
  ctx.quadraticCurveTo(x, y + h, x, y + h - r);
  ctx.lineTo(x, y + r);
  ctx.quadraticCurveTo(x, y, x + r, y);
  ctx.closePath();
}

export default OverlayEffects;
