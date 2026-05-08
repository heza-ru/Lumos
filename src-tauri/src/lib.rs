pub mod state;
pub mod overlay;
pub mod renderer;

use state::new_shared_state;

pub fn run() {
    let app_state = new_shared_state();

    #[cfg(target_os = "macos")]
    let app_state_for_setup = app_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(app_state)
        .setup(move |_app| {
            #[cfg(target_os = "macos")]
            {
                let panel = unsafe { overlay::create_overlay() };
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
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lumos");
}
