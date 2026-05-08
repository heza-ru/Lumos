use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolKind {
    #[default]
    Pen,
    Highlighter,
    Arrow,
    Rectangle,
    Ellipse,
    Line,
    Text,
    Laser,
    Eraser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StrokeWidth {
    Thin,
    #[default]
    Medium,
    Bold,
    ExtraBold,
}

impl StrokeWidth {
    pub fn pen_px(self) -> f32 {
        match self {
            Self::Thin      => 4.0,
            Self::Medium    => 8.0,
            Self::Bold      => 12.0,
            Self::ExtraBold => 16.0,
        }
    }
    pub fn highlighter_px(self) -> f32 {
        match self {
            Self::Thin      => 8.0,
            Self::Medium    => 16.0,
            Self::Bold      => 24.0,
            Self::ExtraBold => 32.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLUE:  Self = Self { r: 82,  g: 155, b: 224, a: 255 };
    pub const RED:   Self = Self { r: 224, g: 82,  b: 82,  a: 255 };
    pub const GREEN: Self = Self { r: 82,  g: 224, b: 108, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 30,  g: 30,  b: 30,  a: 255 };
}

impl Default for Color {
    fn default() -> Self { Self::BLUE }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub id: u64,
    pub tool: ToolKind,
    pub color: Color,
    pub width: StrokeWidth,
    pub points: Vec<Point>,
    /// milliseconds since UNIX epoch when stroke was created (used for laser fade)
    pub created_at_ms: u64,
}

/// All persistent drawing content on the overlay.
#[derive(Debug, Clone, Default)]
pub struct DrawingState {
    pub active_tool:  ToolKind,
    pub active_color: Color,
    pub active_width: StrokeWidth,
    pub strokes:      Vec<Stroke>,
    pub next_id:      u64,
    /// Stroke currently being drawn (not yet committed to `strokes`)
    pub live_stroke:  Option<Stroke>,
}

impl DrawingState {
    pub fn begin_stroke(&mut self, x: f32, y: f32, ts: u64) {
        self.live_stroke = Some(Stroke {
            id:            self.next_id,
            tool:          self.active_tool,
            color:         self.active_color.clone(),
            width:         self.active_width,
            points:        vec![Point { x, y }],
            created_at_ms: ts,
        });
        self.next_id += 1;
    }

    pub fn extend_stroke(&mut self, x: f32, y: f32) {
        if let Some(s) = &mut self.live_stroke {
            s.points.push(Point { x, y });
        }
    }

    pub fn commit_stroke(&mut self) {
        if let Some(s) = self.live_stroke.take() {
            if s.points.len() >= 2 {
                self.strokes.push(s);
            }
        }
    }

    pub fn undo(&mut self) {
        self.strokes.pop();
    }

    pub fn clear(&mut self) {
        self.strokes.clear();
        self.live_stroke = None;
    }
}

/// Top-level app state shared between Tauri commands and the render loop.
#[derive(Debug)]
pub struct AppState {
    pub overlay_visible:  bool,
    pub click_through:    bool,
    pub drawing:          DrawingState,
    /// Current cursor position (updated by EventTap on every mouse move).
    pub cursor_pos:       Point,
    pub spotlight_active: bool,
    pub zoom_active:      bool,
    /// Raw NSPanel pointer as usize for click-through sync from hotkeys.
    /// Only set on macOS after overlay creation; 0 means unset.
    pub overlay_panel_ptr: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            overlay_visible:   false,
            click_through:     true,
            drawing:           DrawingState::default(),
            cursor_pos:        Point { x: 0.0, y: 0.0 },
            spotlight_active:  false,
            zoom_active:       false,
            overlay_panel_ptr: 0,
        }
    }
}

pub type SharedState = Arc<Mutex<AppState>>;

pub fn new_shared_state() -> SharedState {
    Arc::new(Mutex::new(AppState {
        overlay_visible:   false,
        click_through:     true,
        drawing:           DrawingState::default(),
        cursor_pos:        Point { x: 0.0, y: 0.0 },
        spotlight_active:  false,
        zoom_active:       false,
        overlay_panel_ptr: 0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tool_is_pen() {
        let state = DrawingState::default();
        assert_eq!(state.active_tool, ToolKind::Pen);
    }

    #[test]
    fn overlay_starts_hidden() {
        let state = AppState::default();
        assert!(!state.overlay_visible);
    }

    #[test]
    fn passthrough_starts_true() {
        let state = AppState::default();
        assert!(state.click_through);
    }

    #[test]
    fn begin_extend_commit_stroke() {
        let mut ds = DrawingState::default();
        ds.begin_stroke(10.0, 20.0, 0);
        ds.extend_stroke(30.0, 40.0);
        assert!(ds.live_stroke.is_some());
        assert_eq!(ds.live_stroke.as_ref().unwrap().points.len(), 2);
        ds.commit_stroke();
        assert_eq!(ds.strokes.len(), 1);
        assert!(ds.live_stroke.is_none());
    }

    #[test]
    fn stroke_with_single_point_not_committed() {
        let mut ds = DrawingState::default();
        ds.begin_stroke(10.0, 20.0, 0);
        ds.commit_stroke();
        assert_eq!(ds.strokes.len(), 0);
    }

    #[test]
    fn undo_removes_last_stroke() {
        let mut ds = DrawingState::default();
        ds.begin_stroke(0.0, 0.0, 0);
        ds.extend_stroke(10.0, 10.0);
        ds.commit_stroke();
        assert_eq!(ds.strokes.len(), 1);
        ds.undo();
        assert_eq!(ds.strokes.len(), 0);
    }

    #[test]
    fn clear_removes_all_strokes_and_live() {
        let mut ds = DrawingState::default();
        ds.begin_stroke(0.0, 0.0, 0);
        ds.extend_stroke(10.0, 10.0);
        ds.commit_stroke();
        ds.begin_stroke(5.0, 5.0, 1);
        ds.clear();
        assert_eq!(ds.strokes.len(), 0);
        assert!(ds.live_stroke.is_none());
    }

    #[test]
    fn stroke_width_pen_px_values() {
        assert_eq!(StrokeWidth::Thin.pen_px(), 4.0);
        assert_eq!(StrokeWidth::Medium.pen_px(), 8.0);
        assert_eq!(StrokeWidth::Bold.pen_px(), 12.0);
        assert_eq!(StrokeWidth::ExtraBold.pen_px(), 16.0);
    }

    #[test]
    fn new_shared_state_defaults() {
        let shared = new_shared_state();
        let s = shared.lock();
        assert!(!s.overlay_visible);
        assert!(s.click_through);
        assert_eq!(s.drawing.active_tool, ToolKind::Pen);
    }
}
