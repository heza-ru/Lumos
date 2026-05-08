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

/// Shortcuts that are ALWAYS registered.
/// All use modifier keys — safe to intercept globally without affecting typing.
pub fn global_shortcuts() -> Vec<HotkeyDef> {
    vec![
        HotkeyDef { accelerator: "CmdOrCtrl+Shift+A".into(), action: HotkeyAction::ToggleOverlay },
        HotkeyDef { accelerator: "CmdOrCtrl+K".into(),       action: HotkeyAction::ClearAll },
        HotkeyDef { accelerator: "CmdOrCtrl+Z".into(),       action: HotkeyAction::Undo },
        HotkeyDef { accelerator: "CmdOrCtrl+D".into(),       action: HotkeyAction::ToggleClickThrough },
        HotkeyDef { accelerator: "Shift+KeyS".into(),        action: HotkeyAction::ToggleSpotlight },
        HotkeyDef { accelerator: "Shift+KeyZ".into(),        action: HotkeyAction::ToggleZoom },
    ]
}

/// Shortcuts registered ONLY while the overlay is visible.
/// Single letters — would intercept normal typing if always active.
pub fn contextual_shortcuts() -> Vec<HotkeyDef> {
    vec![
        HotkeyDef { accelerator: "KeyP".into(), action: HotkeyAction::ToolPen },
        HotkeyDef { accelerator: "KeyH".into(), action: HotkeyAction::ToolHighlighter },
        HotkeyDef { accelerator: "KeyA".into(), action: HotkeyAction::ToolArrow },
        HotkeyDef { accelerator: "KeyR".into(), action: HotkeyAction::ToolRectangle },
        HotkeyDef { accelerator: "KeyE".into(), action: HotkeyAction::ToolEllipse },
        HotkeyDef { accelerator: "KeyL".into(), action: HotkeyAction::ToolLaser },
        HotkeyDef { accelerator: "KeyX".into(), action: HotkeyAction::ToolEraser },
    ]
}

/// For tests only — kept for backwards compat.
pub fn default_shortcuts() -> Vec<HotkeyDef> {
    let mut all = global_shortcuts();
    all.extend(contextual_shortcuts());
    all
}

/// Dispatch a hotkey action against shared state.
/// Does NOT handle ToggleOverlay (that's managed by register_all because it
/// needs AppHandle access to register/unregister contextual shortcuts).
pub fn dispatch_action(action: HotkeyAction, state: &SharedState) {
    let mut s = state.lock();
    match action {
        HotkeyAction::ToggleOverlay => {
            // Handled separately in register_all — should not reach here
            s.overlay_visible = !s.overlay_visible;
        }
        HotkeyAction::ClearAll           => s.drawing.clear(),
        HotkeyAction::Undo               => s.drawing.undo(),
        HotkeyAction::ToggleClickThrough => {
            s.click_through = !s.click_through;
            let enabled = s.click_through;
            let ptr = s.overlay_panel_ptr;
            drop(s);
            #[cfg(target_os = "macos")]
            if ptr != 0 {
                let panel = ptr as *mut objc::runtime::Object;
                unsafe { crate::overlay::set_click_through(panel, enabled) };
            }
            return;
        }
        HotkeyAction::ToolPen         => s.drawing.active_tool = ToolKind::Pen,
        HotkeyAction::ToolHighlighter => s.drawing.active_tool = ToolKind::Highlighter,
        HotkeyAction::ToolArrow       => s.drawing.active_tool = ToolKind::Arrow,
        HotkeyAction::ToolRectangle   => s.drawing.active_tool = ToolKind::Rectangle,
        HotkeyAction::ToolEllipse     => s.drawing.active_tool = ToolKind::Ellipse,
        HotkeyAction::ToolLaser       => s.drawing.active_tool = ToolKind::Laser,
        HotkeyAction::ToolEraser      => s.drawing.active_tool = ToolKind::Eraser,
        HotkeyAction::ToggleSpotlight => { s.spotlight_active = !s.spotlight_active; }
        HotkeyAction::ToggleZoom      => { s.zoom_active = !s.zoom_active; }
    }
}

/// Register contextual (single-letter) shortcuts.
/// Call when overlay becomes visible.
pub fn register_contextual(app: &tauri::AppHandle, state: SharedState) {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
    for def in contextual_shortcuts() {
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

/// Unregister contextual shortcuts.
/// Call when overlay becomes hidden.
pub fn unregister_contextual(app: &tauri::AppHandle) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    for def in contextual_shortcuts() {
        let _ = app.global_shortcut().unregister(def.accelerator.as_str());
    }
}

/// Register global (always-on) shortcuts. Call once from setup.
/// ToggleOverlay is handled specially here — it also manages contextual shortcuts.
pub fn register_all(app: &tauri::AppHandle, state: SharedState) {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    for def in global_shortcuts() {
        let state_clone = state.clone();
        let action = def.action;
        let handle = app.clone();

        let _ = app.global_shortcut().on_shortcut(
            def.accelerator.as_str(),
            move |_app, _shortcut, event| {
                if event.state() != ShortcutState::Pressed { return; }

                if action == HotkeyAction::ToggleOverlay {
                    // Flip visibility
                    let mut s = state_clone.lock();
                    s.overlay_visible = !s.overlay_visible;
                    let visible = s.overlay_visible;
                    drop(s);

                    // Register single-letter shortcuts only while overlay is visible
                    if visible {
                        register_contextual(&handle, state_clone.clone());
                    } else {
                        unregister_contextual(&handle);
                    }
                } else {
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
        let shortcuts = global_shortcuts();
        assert!(shortcuts.iter().any(|s| s.action == HotkeyAction::ToggleOverlay));
    }

    #[test]
    fn contextual_shortcuts_are_single_letters() {
        for s in contextual_shortcuts() {
            // All contextual shortcuts start with "Key" (single letter, no modifiers)
            assert!(s.accelerator.starts_with("Key"), "{} should be a bare Key shortcut", s.accelerator);
        }
    }

    #[test]
    fn global_shortcuts_have_modifiers() {
        for s in global_shortcuts() {
            // All global shortcuts should have at least one modifier
            let has_modifier = s.accelerator.contains("Ctrl")
                || s.accelerator.contains("Cmd")
                || s.accelerator.contains("Shift")
                || s.accelerator.contains("Alt");
            assert!(has_modifier, "{} should have a modifier key", s.accelerator);
        }
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
