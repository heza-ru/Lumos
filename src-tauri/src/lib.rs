pub mod state;
pub mod overlay;
pub mod renderer;
pub mod commands;
pub mod hotkeys;

use state::new_shared_state;

pub fn run() {
    let app_state = new_shared_state();

    #[cfg(target_os = "macos")]
    let app_state_for_setup = app_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
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
        ])
        .setup(move |app| {
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

                let overlay_ref = overlay::OverlayRef::new(panel);
                // Store in app state so commands can access it later
                // (In Task 10 we'll wire it into the Tauri managed state properly)
                std::mem::forget(overlay_ref); // keep panel alive, wired properly in Task 10
                unsafe { overlay::show_overlay(panel) };

                // Install CGEventTap for mouse capture in draw mode.
                // The tap starts inactive; it will be enabled when the overlay
                // switches from pointer mode to draw mode.
                let tap = overlay::EventTap::install(app_state_for_setup.clone());
                std::mem::forget(tap); // keep tap alive for app lifetime

                // Register global hotkeys.
                hotkeys::register_all(&app.handle(), app_state_for_setup.clone());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lumos");
}
