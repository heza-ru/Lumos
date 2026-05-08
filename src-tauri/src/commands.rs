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

#[tauri::command]
pub fn toggle_spotlight(state: State<SharedState>) -> bool {
    let mut s = state.lock();
    s.spotlight_active = !s.spotlight_active;
    s.spotlight_active
}

#[tauri::command]
pub fn set_spotlight_shape(shape: String, state: State<SharedState>) -> Result<(), String> {
    use crate::effects::spotlight::SpotlightShape;
    let s_shape = match shape.as_str() {
        "circle"    => SpotlightShape::Circle { radius: 120.0 },
        "rectangle" => SpotlightShape::Rectangle { width: 400.0, height: 250.0 },
        other => return Err(format!("unknown shape: {other}")),
    };
    state.lock().spotlight_shape = s_shape;
    Ok(())
}

#[tauri::command]
pub fn toggle_zoom(state: State<SharedState>) -> bool {
    let mut s = state.lock();
    s.zoom_active = !s.zoom_active;
    s.zoom_active
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> crate::settings::Settings {
    use tauri_plugin_store::StoreExt;
    app.store("settings.json")
        .ok()
        .and_then(|store| store.get("settings"))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(
    settings: crate::settings::Settings,
    state: State<SharedState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Apply relevant settings to live state
    {
        let mut s = state.lock();
        s.spotlight_dim_alpha = settings.spotlight_dim_alpha;
        s.zoom_factor = settings.zoom_factor;
    }
    // Persist to store
    use tauri_plugin_store::StoreExt;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("settings", serde_json::to_value(&settings).map_err(|e| e.to_string())?);
    store.save().map_err(|e| e.to_string())
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
        assert!(state.lock().overlay_visible);   // starts true
        toggle_overlay_inner(&state);
        assert!(!state.lock().overlay_visible);
        toggle_overlay_inner(&state);
        assert!(state.lock().overlay_visible);
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
