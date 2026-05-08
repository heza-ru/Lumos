pub mod state;
pub mod overlay;

use state::new_shared_state;

pub fn run() {
    let app_state = new_shared_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(app_state)
        .setup(|_app| {
            #[cfg(target_os = "macos")]
            {
                let panel = unsafe { overlay::create_overlay() };
                let overlay_ref = overlay::OverlayRef::new(panel);
                // Store in app state so commands can access it later
                // (In Task 10 we'll wire it into the Tauri managed state properly)
                std::mem::forget(overlay_ref); // keep panel alive, wired properly in Task 10
                unsafe { overlay::show_overlay(panel) };
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lumos");
}
