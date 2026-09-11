pub mod audio_ducking;
pub mod local_stt;
pub mod mac_window;
pub mod mobile;

use tauri_plugin_store::StoreExt;

pub fn enabled(app: &tauri::AppHandle, key: &str) -> bool {
    app.store("h-features.json")
        .inspect_err(|error| tracing::warn!("Could not load H feature settings: {error}"))
        .ok()
        .and_then(|store| store.get(key))
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
}

pub mod voice_feedback;

pub mod result_window;

pub mod tutorial;

pub mod escape_cancel;

pub mod fn_event;
pub mod product_scope;

pub mod auto_start;

pub mod local_llm;
