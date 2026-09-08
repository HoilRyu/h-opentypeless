// AppHandle dispatches activation policy changes through the runtime main thread.
pub fn visible(app: &tauri::AppHandle, visible: bool) {
    #[cfg(target_os = "macos")]
    if super::enabled(app, "menu_bar_when_closed") {
        let policy = if visible {
            tauri::ActivationPolicy::Regular
        } else {
            tauri::ActivationPolicy::Accessory
        };
        if let Err(error) = app.set_activation_policy(policy) {
            tracing::warn!("H Dock policy failed: {error}");
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, visible);
}
