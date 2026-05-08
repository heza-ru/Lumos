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

                // Store the raw panel pointer in AppState so hotkeys can sync ignoresMouseEvents.
                app_state_for_setup.lock().overlay_panel_ptr = panel as usize;

                // Manage OverlayRef in Tauri state so Tauri commands can access the panel.
                app.manage(overlay::OverlayRef::new(panel));
                unsafe { overlay::show_overlay(panel) };

                // Install CGEventTap for mouse capture in draw mode.
                // The tap starts inactive; it will be enabled when the overlay
                // switches from pointer mode to draw mode.
                let tap = overlay::EventTap::install(app_state_for_setup.clone());
                std::mem::forget(tap); // keep tap alive for app lifetime

                // Register global hotkeys.
                hotkeys::register_all(&app.handle(), app_state_for_setup.clone());

                // Raise the toolbar window above the NSPanel (level 1001) so it
                // is always visible on top of the annotation overlay.
                if let Some(toolbar_win) = app.get_webview_window("toolbar") {
                    unsafe {
                        use objc::{msg_send, sel, sel_impl};
                        // ns_window() returns *mut c_void; cast to id for msg_send
                        let raw = toolbar_win.ns_window().unwrap();
                        let ns_win = raw as cocoa::base::id;
                        // Level 1002 = above our NSPanel at 1001
                        let _: () = msg_send![ns_win, setLevel: 1002i64];
                    }
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lumos");
}
