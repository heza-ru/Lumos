console.log('[LUMOS]: Extended toolbar page loading...');

// ── Color dots ─────────────────────────────────────────────────────────────
const colorDots = document.querySelectorAll('.color-dot');
colorDots.forEach(dot => {
  dot.addEventListener('click', () => {
    const idx = parseInt(dot.dataset.index, 10);
    // Update active state visually
    colorDots.forEach(d => d.classList.remove('active'));
    dot.classList.add('active');
    // Notify main process (stores color preference)
    window.electronAPI.invokeSetPointerColor(idx);
  });
});

// ── Tool buttons ────────────────────────────────────────────────────────────
const toolBtns = document.querySelectorAll('.tool-btn[data-tool]');
toolBtns.forEach(btn => {
  btn.addEventListener('click', () => {
    // Update active state visually
    toolBtns.forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    // Enter draw mode with this specific tool
    window.electronAPI.invokeDrawModeWithTool(btn.dataset.tool);
  });
});

// ── Clear button ─────────────────────────────────────────────────────────────
// Clear enters draw mode then immediately clears — handled via entering draw mode
// (clearing in pointer mode isn't meaningful without the canvas visible)
document.getElementById('clearBtn')?.addEventListener('click', () => {
  window.electronAPI.invokeDrawMode();
});

// ── Close button ─────────────────────────────────────────────────────────────
document.getElementById('closeBtn')?.addEventListener('click', () => {
  window.electronAPI.invokeCloseApp();
});
