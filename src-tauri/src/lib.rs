pub mod state;

use state::new_shared_state;

pub fn run() {
    let app_state = new_shared_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(app_state)
        .run(tauri::generate_context!())
        .expect("error while running Lumos");
}
