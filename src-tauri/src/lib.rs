pub mod state;
pub mod overlay;
pub mod renderer;
pub mod commands;
pub mod hotkeys;
pub mod tools;
pub mod effects;
pub mod settings;
pub mod display;

use state::new_shared_state;
use tauri::Manager;

pub fn run() {
    let app_state = new_shared_state();

    let app_state_for_setup = app_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_liquid_glass::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::set_tool,
            commands::set_color,
            commands::set_width,
            commands::undo,
            commands::clear_all,
            commands::toggle_overlay,
            commands::toggle_click_through,
            commands::get_app_state,
            commands::toggle_spotlight,
            commands::set_spotlight_shape,
            commands::toggle_zoom,
            commands::get_settings,
            commands::save_settings,
            commands::set_draw_mode,
        ])
        .setup(move |app| {
            // Load persisted settings and apply to state
            {
                use tauri_plugin_store::StoreExt;
                if let Ok(store) = app.store("settings.json") {
                    if let Some(val) = store.get("settings") {
                        if let Ok(loaded) = serde_json::from_value::<crate::settings::Settings>(val.clone()) {
                            let mut s = app_state_for_setup.lock();
                            s.spotlight_dim_alpha = loaded.spotlight_dim_alpha;
                            s.zoom_factor = loaded.zoom_factor;
                        }
                    }
                }
            }
            #[cfg(target_os = "macos")]
            {
                // Set up the canvas window: all Spaces, above fullscreen, correct size
                if let Some(canvas_win) = app.get_webview_window("canvas") {
                    unsafe {
                        use objc::{msg_send, sel, sel_impl};
                        use cocoa::foundation::NSRect;

                        let screen: cocoa::base::id = msg_send![objc::class!(NSScreen), mainScreen];

                        // Use full screen bounds so the canvas covers everything
                        let full_frame: NSRect = msg_send![screen, frame];
                        let _ = canvas_win.set_position(
                            tauri::Position::Logical(tauri::LogicalPosition::new(0.0_f64, 0.0_f64))
                        );
                        let _ = canvas_win.set_size(
                            tauri::Size::Logical(tauri::LogicalSize::new(
                                full_frame.size.width,
                                full_frame.size.height,
                            ))
                        );

                        // Set window level and collection behavior
                        let raw = canvas_win.ns_window().unwrap();
                        let ns_win = raw as cocoa::base::id;

                        // Level just below toolbar (1001) but above all normal windows
                        let _: () = msg_send![ns_win, setLevel: 1000i64];

                        // CanJoinAllSpaces(1) + Stationary(16) + IgnoresCycle(64) + FullScreenAuxiliary(256)
                        let _: () = msg_send![ns_win, setCollectionBehavior: 337u64];

                        // Start click-through (pointer mode)
                        let _ = canvas_win.set_ignore_cursor_events(true);
                    }
                    let _ = canvas_win.show();
                }

                // Position and show toolbar window
                if let Some(toolbar_win) = app.get_webview_window("toolbar") {
                    unsafe {
                        use objc::{msg_send, sel, sel_impl};
                        use cocoa::foundation::NSRect;

                        let screen: cocoa::base::id = msg_send![objc::class!(NSScreen), mainScreen];
                        let frame: NSRect = msg_send![screen, frame];
                        let screen_w = frame.size.width;
                        let screen_h = frame.size.height;

                        let win_w = 640.0_f64;
                        let win_h = 72.0_f64;
                        let x = (screen_w - win_w) / 2.0;
                        let y = screen_h - win_h - 80.0;

                        let _ = toolbar_win.set_position(
                            tauri::Position::Logical(tauri::LogicalPosition::new(x, y))
                        );

                        let raw = toolbar_win.ns_window().unwrap();
                        let ns_win = raw as cocoa::base::id;
                        let _: () = msg_send![ns_win, setLevel: 1002i64];
                        let _: () = msg_send![ns_win, setCollectionBehavior: 337u64];
                    }
                    let _ = toolbar_win.show();
                }

                // Register global hotkeys (no NSPanel or CGEventTap needed)
                hotkeys::register_all(&app.handle(), app_state_for_setup.clone());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lumos");
}
