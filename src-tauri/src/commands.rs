use serde::Serialize;
use tauri::State;
use crate::state::{SharedState, ToolKind, StrokeWidth, Color as LColor};

// ── Pure testable logic ──────────────────────────────────────────────────────

pub fn set_tool_inner(state: &SharedState, tool: &str) -> Result<(), String> {
    let kind = match tool {
        "pen"         => ToolKind::Pen,
        "highlighter" => ToolKind::Highlighter,
        "arrow"       => ToolKind::Arrow,
        "rectangle"   => ToolKind::Rectangle,
        "ellipse"     => ToolKind::Ellipse,
        "line"        => ToolKind::Line,
        "text"        => ToolKind::Text,
        "laser"       => ToolKind::Laser,
        "eraser"      => ToolKind::Eraser,
        other         => return Err(format!("unknown tool: {other}")),
    };
    state.lock().drawing.active_tool = kind;
    Ok(())
}

pub fn toggle_overlay_inner(state: &SharedState) -> bool {
    let mut s = state.lock();
    s.overlay_visible = !s.overlay_visible;
    s.overlay_visible
}

pub fn toggle_click_through_inner(state: &SharedState) -> bool {
    let mut s = state.lock();
    s.click_through = !s.click_through;
    s.click_through
}

// ── Tauri commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn set_tool(tool: String, state: State<SharedState>) -> Result<(), String> {
    set_tool_inner(&state, &tool)
}

#[tauri::command]
pub fn set_color(r: u8, g: u8, b: u8, state: State<SharedState>) {
    state.lock().drawing.active_color = LColor { r, g, b, a: 255 };
}

#[tauri::command]
pub fn set_width(width: String, state: State<SharedState>) -> Result<(), String> {
    let w = match width.as_str() {
        "thin"       => StrokeWidth::Thin,
        "medium"     => StrokeWidth::Medium,
        "bold"       => StrokeWidth::Bold,
        "extra_bold" => StrokeWidth::ExtraBold,
        other        => return Err(format!("unknown width: {other}")),
    };
    state.lock().drawing.active_width = w;
    Ok(())
}

#[tauri::command]
pub fn undo(state: State<SharedState>) {
    state.lock().drawing.undo();
}

#[tauri::command]
pub fn clear_all(state: State<SharedState>) {
    state.lock().drawing.clear();
}

#[tauri::command]
pub fn toggle_overlay(state: State<SharedState>) -> bool {
    toggle_overlay_inner(&state)
}

#[tauri::command]
pub fn toggle_click_through(
    state: State<SharedState>,
    #[cfg(target_os = "macos")]
    overlay: State<crate::overlay::OverlayRef>,
) -> bool {
    let enabled = toggle_click_through_inner(&state);
    #[cfg(target_os = "macos")]
    {
        let panel = *overlay.0.lock().unwrap();
        unsafe { crate::overlay::set_click_through(panel, enabled) };
    }
    enabled
}

#[derive(Serialize)]
pub struct AppSnapshot {
    pub overlay_visible: bool,
    pub click_through:   bool,
    pub active_tool:     ToolKind,
}

#[tauri::command]
pub fn get_app_state(state: State<SharedState>) -> AppSnapshot {
    let s = state.lock();
    AppSnapshot {
        overlay_visible: s.overlay_visible,
        click_through:   s.click_through,
        active_tool:     s.drawing.active_tool,
    }
}

// Tests at bottom
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::new_shared_state;

    #[test]
    fn set_tool_inner_updates_state() {
        let state = new_shared_state();
        assert!(set_tool_inner(&state, "highlighter").is_ok());
        assert_eq!(state.lock().drawing.active_tool, ToolKind::Highlighter);
    }

    #[test]
    fn set_tool_inner_rejects_unknown_tool() {
        let state = new_shared_state();
        assert!(set_tool_inner(&state, "magic_wand").is_err());
    }

    #[test]
    fn toggle_overlay_flips_visibility() {
        let state = new_shared_state();
        assert!(!state.lock().overlay_visible);
        toggle_overlay_inner(&state);
        assert!(state.lock().overlay_visible);
        toggle_overlay_inner(&state);
        assert!(!state.lock().overlay_visible);
    }

    #[test]
    fn toggle_click_through_sequence() {
        let state = new_shared_state();
        assert!(state.lock().click_through);
        let r1 = toggle_click_through_inner(&state);
        assert!(!r1);
        let r2 = toggle_click_through_inner(&state);
        assert!(r2);
    }

    #[test]
    fn set_tool_inner_all_variants() {
        let state = new_shared_state();
        for (name, expected) in [
            ("pen",         ToolKind::Pen),
            ("highlighter", ToolKind::Highlighter),
            ("arrow",       ToolKind::Arrow),
            ("rectangle",   ToolKind::Rectangle),
            ("ellipse",     ToolKind::Ellipse),
            ("line",        ToolKind::Line),
            ("text",        ToolKind::Text),
            ("laser",       ToolKind::Laser),
            ("eraser",      ToolKind::Eraser),
        ] {
            assert!(set_tool_inner(&state, name).is_ok());
            assert_eq!(state.lock().drawing.active_tool, expected);
        }
    }
}
