import React, { useCallback, useEffect, useRef, useState } from 'react';
import './ToolBar.scss';
import { colorList, widthList } from '../constants.js';

const ZONE_BORDER = 10;
const STICKY = 15;

const ToolBar = ({
  position,
  setPosition,
  isCollapsed,
  setIsCollapsed,
  activeTool,
  activeColorIndex,
  activeWidthIndex,
  handleChangeTool,
  handleChangeColor,
  handleChangeWidth,
  handleClearDesk,
  isPointerMode,
  handleTogglePointerMode,
  spotlightActive,
  handleToggleSpotlight,
  zoomActive,
  handleToggleZoom,
  Icons,
}) => {
  const [dragging, setDragging] = useState(false);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const pillRef = useRef(null);

  /* ── Drag ──────────────────────────────────────────────── */
  const clamp = useCallback((x, y) => {
    if (!pillRef.current) return { x: Math.trunc(x), y: Math.trunc(y) };
    const { offsetWidth: w, offsetHeight: h } = pillRef.current;
    const maxX = Math.max(ZONE_BORDER, window.innerWidth  - ZONE_BORDER - w);
    const maxY = Math.max(ZONE_BORDER, window.innerHeight - ZONE_BORDER - h);
    let nx = Math.min(Math.max(x, ZONE_BORDER), maxX);
    let ny = Math.min(Math.max(y, ZONE_BORDER), maxY);
    // Sticky edges
    if (nx < ZONE_BORDER + STICKY) nx = ZONE_BORDER;
    else if (nx > maxX - STICKY)   nx = maxX;
    if (ny < ZONE_BORDER + STICKY) ny = ZONE_BORDER;
    else if (ny > maxY - STICKY)   ny = maxY;
    return { x: Math.trunc(nx), y: Math.trunc(ny) };
  }, []);

  const onDragStart = (e) => {
    // Only start drag from the pill background — not from buttons/dots
    if (e.target !== pillRef.current && e.target.closest('button, .cdot, .wdot')) return;
    setDragging(true);
    setOffset({ x: e.clientX - position.x, y: e.clientY - position.y });
    e.currentTarget.setPointerCapture(e.pointerId);
  };

  const onDragMove = useCallback((e) => {
    if (!dragging) return;
    setPosition(clamp(e.clientX - offset.x, e.clientY - offset.y));
  }, [dragging, offset, clamp, setPosition]);

  const onDragEnd = useCallback(() => setDragging(false), []);

  useEffect(() => {
    setPosition(prev => clamp(prev.x, prev.y));
  }, [clamp, setPosition]);

  /* ── Icon map ─────────────────────────────────────────── */
  const toolIcons = {
    pen:         <Icons.Brush />,
    fadepen:     <Icons.MagicBrush />,
    highlighter: <Icons.Highlighter />,
    arrow:       <Icons.Arrow />,
    flat_arrow:  <Icons.FlatArrow />,
    rectangle:   <Icons.Rectangle />,
    oval:        <Icons.Oval />,
    line:        <Icons.Line />,
    text:        <Icons.Text />,
    laser:       <Icons.Laser />,
    eraser:      <Icons.Eraser />,
  };

  /* ── Tool list shown in toolbar ───────────────────────── */
  const TOOLS = [
    { key: 'pen',         label: 'Pen (F)' },
    { key: 'highlighter', label: 'Highlighter (H)' },
    { key: 'arrow',       label: 'Arrow (A)' },
    { key: 'rectangle',   label: 'Rectangle (R)' },
    { key: 'oval',        label: 'Circle (C)' },
    { key: 'text',        label: 'Text (T)' },
    { key: 'laser',       label: 'Laser (L)' },
    { key: 'eraser',      label: 'Eraser (X)' },
  ];

  const isToolActive = (key) => {
    if (key === 'pen')   return activeTool === 'pen' || activeTool === 'fadepen';
    if (key === 'arrow') return activeTool === 'arrow' || activeTool === 'flat_arrow';
    return activeTool === key;
  };

  const stop = (e) => e.stopPropagation();

  /* ── MINIMISED STATE ──────────────────────────────────── */
  if (isCollapsed) {
    return (
      <aside
        id="toolbar"
        style={{ left: position.x, top: position.y }}
      >
        <div
          className="tb-mini"
          ref={pillRef}
          onPointerDown={onDragStart}
          onPointerMove={onDragMove}
          onPointerUp={onDragEnd}
          onPointerCancel={onDragEnd}
        >
          {/* Active tool */}
          <div className="mini-icon">
            {toolIcons[activeTool]}
          </div>

          {/* Active color dot */}
          <div
            className="mini-color"
            style={{ background: colorList[activeColorIndex]?.color || '#529BE0' }}
          />

          {/* Active width dot */}
          <div
            className="mini-width"
            style={{ width: 4 + activeWidthIndex * 2, height: 4 + activeWidthIndex * 2 }}
          />

          {/* Expand */}
          <button
            className="mini-expand"
            title="Expand toolbar"
            onClick={() => setIsCollapsed(false)}
            onPointerDown={stop}
          >
            <Icons.AngleRight />
          </button>
        </div>
      </aside>
    );
  }

  /* ── FULL STATE ───────────────────────────────────────── */
  return (
    <aside
      id="toolbar"
      style={{ left: position.x, top: position.y }}
    >
      <div
        className={`tb-pill${isPointerMode ? ' tb-pill--pointer' : ''}`}
        ref={pillRef}
        onPointerDown={onDragStart}
        onPointerMove={onDragMove}
        onPointerUp={onDragEnd}
        onPointerCancel={onDragEnd}
      >

        {/* ── TOOLS ──────────────────────────────────── */}
        <div className="tb-group tb-group--tools">
          {TOOLS.map(({ key, label }) => (
            <button
              key={key}
              className={`tb-btn${isToolActive(key) ? ' tb-btn--on' : ''}`}
              title={label}
              onClick={() => handleChangeTool(key)}
              onPointerDown={stop}
            >
              {toolIcons[key]}
            </button>
          ))}
        </div>

        <div className="tb-sep" />

        {/* ── COLORS ─────────────────────────────────── */}
        <div className="tb-group tb-group--colors">
          {colorList.map((c, i) => (
            <div
              key={i}
              className={`cdot${i === activeColorIndex ? ' cdot--on' : ''}`}
              title={c.title}
              style={{
                background: c.name === 'color_rainbow'
                  ? 'conic-gradient(red,yellow,lime,aqua,blue,magenta,red)'
                  : c.color
              }}
              onClick={() => handleChangeColor(i)}
              onPointerDown={stop}
            />
          ))}
        </div>

        <div className="tb-sep" />

        {/* ── WIDTHS ─────────────────────────────────── */}
        <div className="tb-group tb-group--widths">
          {widthList.map((w, i) => (
            <div
              key={i}
              className={`wdot${i === activeWidthIndex ? ' wdot--on' : ''}`}
              title={`${w.title} ([/])`}
              style={{ width: 5 + i * 2, height: 5 + i * 2 }}
              onClick={() => handleChangeWidth(i)}
              onPointerDown={stop}
            />
          ))}
        </div>

        <div className="tb-sep" />

        {/* ── EFFECTS ────────────────────────────────── */}
        <div className="tb-group tb-group--fx">
          {/* Spotlight */}
          <button
            className={`tb-btn tb-btn--sm${spotlightActive ? ' tb-btn--on' : ''}`}
            title="Spotlight (⇧S)"
            onClick={handleToggleSpotlight}
            onPointerDown={stop}
          >
            <Icons.Spotlight />
          </button>
          {/* Zoom */}
          <button
            className={`tb-btn tb-btn--sm${zoomActive ? ' tb-btn--on' : ''}`}
            title="Zoom lens (⇧Z)"
            onClick={handleToggleZoom}
            onPointerDown={stop}
          >
            <Icons.Zoom />
          </button>
        </div>

        <div className="tb-sep" />

        {/* ── ACTIONS ────────────────────────────────── */}
        <div className="tb-group tb-group--util">
          <button
            className="tb-btn tb-btn--dim"
            title="Undo (⌘Z)"
            onPointerDown={stop}
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{width:16,height:16}}><path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"/></svg>
          </button>
          <button
            className="tb-btn tb-btn--dim"
            title="Clear all (⌘K)"
            onClick={handleClearDesk}
            onPointerDown={stop}
          >
            <Icons.Trash />
          </button>
        </div>

        <div className="tb-sep" />

        {/* ── POINTER MODE CHIP ──────────────────────── */}
        <button
          className={`tb-ptr${isPointerMode ? ' tb-ptr--active' : ''}`}
          title={isPointerMode ? 'Resume drawing (⌘D)' : 'Pointer mode — canvas click-through (⌘D)'}
          onClick={handleTogglePointerMode}
          onPointerDown={stop}
        >
          <Icons.DrawModeEnabled style={{ width: 12, height: 12 }} />
          {isPointerMode ? 'Draw' : 'Pointer'}
        </button>

        {/* ── COLLAPSE ───────────────────────────────── */}
        <button
          className="tb-collapse"
          title="Minimise toolbar"
          onClick={() => setIsCollapsed(true)}
          onPointerDown={stop}
        >
          <Icons.AngleLeft />
        </button>

      </div>
    </aside>
  );
};

export default ToolBar;
