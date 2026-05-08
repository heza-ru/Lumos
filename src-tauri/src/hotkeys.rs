use crate::state::{SharedState, ToolKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    ToggleOverlay,
    ClearAll,
    Undo,
    ToggleClickThrough,
    ToolPen,
    ToolHighlighter,
    ToolArrow,
    ToolRectangle,
    ToolEllipse,
    ToolLaser,
    ToolEraser,
    ToggleSpotlight,
    ToggleZoom,
}

#[derive(Debug, Clone)]
pub struct HotkeyDef {
    pub accelerator: String,
    pub action: HotkeyAction,
}

pub fn default_shortcuts() -> Vec<HotkeyDef> {
    vec![
        HotkeyDef { accelerator: "CmdOrCtrl+Shift+A".into(), action: HotkeyAction::ToggleOverlay },
        HotkeyDef { accelerator: "CmdOrCtrl+K".into(),       action: HotkeyAction::ClearAll },
        HotkeyDef { accelerator: "CmdOrCtrl+Z".into(),       action: HotkeyAction::Undo },
        HotkeyDef { accelerator: "CmdOrCtrl+D".into(),       action: HotkeyAction::ToggleClickThrough },
        HotkeyDef { accelerator: "KeyP".into(),               action: HotkeyAction::ToolPen },
        HotkeyDef { accelerator: "KeyH".into(),               action: HotkeyAction::ToolHighlighter },
        HotkeyDef { accelerator: "KeyA".into(),               action: HotkeyAction::ToolArrow },
        HotkeyDef { accelerator: "KeyR".into(),               action: HotkeyAction::ToolRectangle },
        HotkeyDef { accelerator: "KeyE".into(),               action: HotkeyAction::ToolEllipse },
        HotkeyDef { accelerator: "KeyL".into(),               action: HotkeyAction::ToolLaser },
        HotkeyDef { accelerator: "KeyX".into(),               action: HotkeyAction::ToolEraser },
        HotkeyDef { accelerator: "Shift+KeyS".into(),         action: HotkeyAction::ToggleSpotlight },
        HotkeyDef { accelerator: "Shift+KeyZ".into(),         action: HotkeyAction::ToggleZoom },
    ]
}

/// Dispatch a hotkey action directly against shared state.
/// Called both from the Tauri shortcut handler and in tests.
pub fn dispatch_action(action: HotkeyAction, state: &SharedState) {
    let mut s = state.lock();
    match action {
        HotkeyAction::ToggleOverlay      => { s.overlay_visible = !s.overlay_visible; }
        HotkeyAction::ClearAll           => s.drawing.clear(),
        HotkeyAction::Undo               => s.drawing.undo(),
        HotkeyAction::ToggleClickThrough => { s.click_through = !s.click_through; }
        HotkeyAction::ToolPen            => s.drawing.active_tool = ToolKind::Pen,
        HotkeyAction::ToolHighlighter    => s.drawing.active_tool = ToolKind::Highlighter,
        HotkeyAction::ToolArrow          => s.drawing.active_tool = ToolKind::Arrow,
        HotkeyAction::ToolRectangle      => s.drawing.active_tool = ToolKind::Rectangle,
        HotkeyAction::ToolEllipse        => s.drawing.active_tool = ToolKind::Ellipse,
        HotkeyAction::ToolLaser          => s.drawing.active_tool = ToolKind::Laser,
        HotkeyAction::ToolEraser         => s.drawing.active_tool = ToolKind::Eraser,
        HotkeyAction::ToggleSpotlight    => { s.spotlight_active = !s.spotlight_active; }
        HotkeyAction::ToggleZoom         => { s.zoom_active = !s.zoom_active; }
    }
}

/// Register all default global shortcuts with the Tauri app.
/// Call once from the `setup` closure.
pub fn register_all(app: &tauri::AppHandle, state: SharedState) {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    let shortcuts = default_shortcuts();

    for def in shortcuts {
        let state_clone = state.clone();
        let action = def.action;
        let _ = app.global_shortcut().on_shortcut(
            def.accelerator.as_str(),
            move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    dispatch_action(action, &state_clone);
                }
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shortcuts_not_empty() {
        assert!(!default_shortcuts().is_empty());
    }

    #[test]
    fn all_shortcuts_have_non_empty_accelerator() {
        for s in default_shortcuts() {
            assert!(!s.accelerator.is_empty(), "empty accelerator for {:?}", s.action);
        }
    }

    #[test]
    fn toggle_overlay_shortcut_exists() {
        let shortcuts = default_shortcuts();
        assert!(shortcuts.iter().any(|s| s.action == HotkeyAction::ToggleOverlay));
    }

    #[test]
    fn dispatch_action_toggle_overlay() {
        let state = crate::state::new_shared_state();
        assert!(!state.lock().overlay_visible);
        dispatch_action(HotkeyAction::ToggleOverlay, &state);
        assert!(state.lock().overlay_visible);
    }

    #[test]
    fn dispatch_action_clear_all() {
        let state = crate::state::new_shared_state();
        {
            let mut s = state.lock();
            s.drawing.begin_stroke(0.0, 0.0, 0);
            s.drawing.extend_stroke(10.0, 10.0);
            s.drawing.commit_stroke();
        }
        assert_eq!(state.lock().drawing.strokes.len(), 1);
        dispatch_action(HotkeyAction::ClearAll, &state);
        assert_eq!(state.lock().drawing.strokes.len(), 0);
    }

    #[test]
    fn dispatch_action_tool_switch() {
        let state = crate::state::new_shared_state();
        dispatch_action(HotkeyAction::ToolHighlighter, &state);
        assert_eq!(state.lock().drawing.active_tool, ToolKind::Highlighter);
        dispatch_action(HotkeyAction::ToolPen, &state);
        assert_eq!(state.lock().drawing.active_tool, ToolKind::Pen);
    }
}
