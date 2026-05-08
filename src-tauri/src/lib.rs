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
                let panel = unsafe { overlay::create_overlay() };

                // Get screen dimensions and scale for the render loop
                let (screen_w, screen_h, scale) = unsafe {
                    use objc::{class, msg_send, sel, sel_impl};
                    use cocoa::foundation::NSRect;

                    let screen: cocoa::base::id = msg_send![objc::class!(NSScreen), mainScreen];
                    let frame: NSRect = msg_send![screen, frame];
                    let scale: f64 = msg_send![screen, backingScaleFactor];
                    (frame.size.width as i32, frame.size.height as i32, scale as f32)
                };

                // Start the 120fps render loop on a dedicated background thread.
                renderer::start_render_loop(panel, app_state_for_setup.clone(), screen_w, screen_h, scale);

                // Store screen_height (logical points) in AppState.
                // EventTap needs it to flip CGEvent y (bottom-origin) → Skia y (top-origin).
                {
                    let mut s = app_state_for_setup.lock();
                    s.screen_height = screen_h as f32;
                    s.overlay_panel_ptr = panel as usize;
                }

                // Manage OverlayRef in Tauri state so Tauri commands can access the panel.
                app.manage(overlay::OverlayRef::new(panel));
                unsafe { overlay::show_overlay(panel) };

                // Install CGEventTap. Always-on: click_through flag in AppState
                // controls whether events are consumed (draw) or passed through (pointer).
                // mem::forget keeps the thread alive; EventTap no longer needs a handle.
                std::mem::forget(overlay::EventTap::install(app_state_for_setup.clone()));

                // Register global hotkeys.
                hotkeys::register_all(&app.handle(), app_state_for_setup.clone());

                // Position toolbar at bottom-center and show it from Rust.
                // This is authoritative — we do NOT rely on the JS useEffect
                // to show the window because async JS can fail silently.
                if let Some(toolbar_win) = app.get_webview_window("toolbar") {
                    unsafe {
                        use objc::{msg_send, sel, sel_impl};
                        use cocoa::foundation::NSRect;

                        let screen: cocoa::base::id = msg_send![objc::class!(NSScreen), mainScreen];
                        let frame: NSRect = msg_send![screen, frame];
                        let scale: f64   = msg_send![screen, backingScaleFactor];
                        let screen_w = frame.size.width;
                        let screen_h = frame.size.height;

                        // Bottom-center: 80 logical px above the Dock
                        let win_w = 640.0_f64;
                        let win_h = 72.0_f64;
                        let x = (screen_w - win_w) / 2.0;
                        let y = screen_h - win_h - 80.0;

                        let _ = toolbar_win.set_position(
                            tauri::Position::Logical(tauri::LogicalPosition::new(x, y))
                        );

                        // Raise above the NSPanel overlay (level 1001) and join all Spaces
                        let raw    = toolbar_win.ns_window().unwrap();
                        let ns_win = raw as cocoa::base::id;
                        let _: () = msg_send![ns_win, setLevel: 1002i64];

                        // NSWindowCollectionBehaviorCanJoinAllSpaces    = 1 << 0 =   1
                        // NSWindowCollectionBehaviorStationary           = 1 << 4 =  16
                        // NSWindowCollectionBehaviorFullScreenAuxiliary  = 1 << 8 = 256
                        // Combined: toolbar appears on every Space including fullscreen apps
                        let _: () = msg_send![ns_win, setCollectionBehavior: 273u64];
                    }
                    let _ = toolbar_win.show();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lumos");
}
