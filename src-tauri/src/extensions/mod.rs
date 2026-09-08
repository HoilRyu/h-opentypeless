pub mod audio_ducking;
pub mod mac_window;

use tauri_plugin_store::StoreExt;

pub fn enabled(app: &tauri::AppHandle, key: &str) -> bool {
    app.store("h-features.json")
        .ok()
        .and_then(|store| store.get(key))
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
}
