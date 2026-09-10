//! H-specific Mac recording feedback. Only desktop pipeline entry points call transition.
#[cfg(target_os = "macos")]
mod native;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
use tauri::{Emitter, Listener, Manager};
use tauri_plugin_store::StoreExt;
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub enabled: bool,
    pub brightness: f64,
    pub reactive: bool,
    pub near_caret: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            brightness: 0.55,
            reactive: true,
            near_caret: true,
        }
    }
}
#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub phase: String,
    pub settings: Settings,
    pub revision: u64,
    pub failed: bool,
    pub preview: bool,
}
struct Inner {
    snapshot: Snapshot,
    #[cfg(target_os = "macos")]
    anchor: Option<(native::Rect, native::Rect, native::Rect)>,
    width: f64,
    height: f64,
    preview: bool,
    suspended: bool,
    #[cfg(target_os = "macos")]
    last_frame: Option<native::Rect>,
    #[cfg(target_os = "macos")]
    dragged: Option<native::Point>,
    #[cfg(target_os = "macos")]
    observers: Vec<(usize, usize)>,
}
pub struct Service {
    inner: Mutex<Inner>,
    generation: AtomicU64,
}
impl Service {
    fn new(settings: Settings) -> Self {
        Self {
            inner: Mutex::new(Inner {
                snapshot: Snapshot {
                    phase: "idle".into(),
                    settings,
                    revision: 0,
                    failed: false,
                    preview: false,
                },
                #[cfg(target_os = "macos")]
                anchor: None,
                width: 224.,
                height: 60.,
                preview: false,
                suspended: false,
                #[cfg(target_os = "macos")]
                last_frame: None,
                #[cfg(target_os = "macos")]
                dragged: None,
                #[cfg(target_os = "macos")]
                observers: Vec::new(),
            }),
            generation: AtomicU64::new(0),
        }
    }
}
#[cfg(target_os = "macos")]
impl Inner {
    fn advance(&mut self, phase: String, epoch: &AtomicU64) -> (bool, u64) {
        let start = (self.snapshot.phase == "idle" || self.preview) && phase != "idle";
        self.preview = false;
        self.snapshot.preview = false;
        if start || phase == "idle" {
            self.snapshot.failed = false;
        }
        if start {
            self.last_frame = None;
            self.dragged = None;
            self.anchor = None;
        }
        self.snapshot.revision += 1;
        self.snapshot.phase = phase.clone();
        if phase == "idle" {
            epoch.fetch_add(1, Ordering::SeqCst);
            self.anchor = None;
        }
        let generation = if start {
            epoch.fetch_add(1, Ordering::SeqCst) + 1
        } else {
            epoch.load(Ordering::SeqCst)
        };
        (start, generation)
    }
    fn finish_preview(&mut self, epoch: &AtomicU64, generation: u64) -> bool {
        if epoch.load(Ordering::SeqCst) != generation || !self.preview {
            return false;
        }
        self.preview = false;
        self.snapshot.preview = false;
        self.snapshot.phase = "idle".into();
        self.snapshot.revision += 1;
        self.anchor = None;
        true
    }
}
pub fn setup(app: &tauri::AppHandle) {
    let settings = app
        .store("h-features.json")
        .ok()
        .and_then(|s| s.get("voice_feedback"))
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    app.manage(Service::new(settings));
    #[cfg(target_os = "macos")]
    {
        let handle = app.clone();
        app.listen("pipeline:error", move |_| {
            if let Some(service) = handle.try_state::<Service>() {
                let mut inner = service.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.snapshot.failed = true;
                inner.snapshot.revision += 1;
                drop(inner);
                render(&handle);
            }
        });
        let observers = native::observe(app);
        app.state::<Service>()
            .inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .observers = observers;
        match tauri::WebviewWindowBuilder::new(
            app,
            "voice-edge",
            tauri::WebviewUrl::App("index.html#voice-edge".into()),
        )
        .title("H Voice Feedback")
        .transparent(true)
        .decorations(false)
        .shadow(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .focusable(false)
        .visible(false)
        .visible_on_all_workspaces(true)
        .build()
        {
            Ok(w) => {
                let _ = w.set_ignore_cursor_events(true);
            }
            Err(e) => tracing::warn!("Voice edge unavailable: {e}"),
        }
    }
}
#[tauri::command]
pub fn get_voice_feedback(app: tauri::AppHandle) -> Snapshot {
    app.state::<Service>()
        .inner
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .snapshot
        .clone()
}
#[tauri::command]
pub fn set_voice_feedback(app: tauri::AppHandle, settings: Settings) -> Result<Snapshot, String> {
    if !settings.brightness.is_finite() || !(0.1..=1.0).contains(&settings.brightness) {
        return Err("Brightness must be between 0.1 and 1".into());
    }
    let store = app.store("h-features.json").map_err(|e| e.to_string())?;
    store.set(
        "voice_feedback",
        serde_json::to_value(&settings).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;
    {
        let service = app.state::<Service>();
        let mut inner = service.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.snapshot.settings = settings;
        inner.snapshot.revision += 1;
        #[cfg(target_os = "macos")]
        {
            inner.last_frame = None;
            inner.dragged = None;
        }
    }
    render(&app);
    Ok(get_voice_feedback(app))
}
pub fn transition(app: &tauri::AppHandle, state: crate::pipeline::PipelineState) {
    crate::extensions::escape_cancel::transition(app, state);
    #[cfg(target_os = "macos")]
    {
        let Some(service) = app.try_state::<Service>() else {
            return;
        };
        let phase = serde_json::to_value(state)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or("idle".into());
        let mut inner = service.inner.lock().unwrap_or_else(|e| e.into_inner());
        let (start, generation) = inner.advance(phase, &service.generation);
        drop(inner);
        if start {
            let app = app.clone();
            std::thread::spawn(move || {
                let input = native::input_area();
                let handle = app.clone();
                let _ = app.run_on_main_thread(move || {
                    let service = handle.state::<Service>();
                    let mut inner = service.inner.lock().unwrap_or_else(|e| e.into_inner());
                    if service.generation.load(Ordering::SeqCst) != generation {
                        return;
                    }
                    tracing::debug!(
                        "Voice feedback anchor resolved (input_area={})",
                        input.is_some()
                    );
                    inner.anchor = Some(native::anchor(input));
                    drop(inner);
                    render_now(&handle);
                });
            });
        }
        render(app);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, state);
}
fn render(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    {
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || render_now(&handle));
    }
    #[cfg(not(target_os = "macos"))]
    let _ = app;
}
#[cfg(target_os = "macos")]
fn render_now(app: &tauri::AppHandle) {
    let service = app.state::<Service>();
    let mut inner = service.inner.lock().unwrap_or_else(|e| e.into_inner());
    let _ = app.emit("voice-feedback:state", &inner.snapshot);
    if let Some(w) = app.get_webview_window("voice-edge") {
        if inner.suspended
            || inner.snapshot.failed
            || inner.snapshot.phase == "idle"
            || !inner.snapshot.settings.enabled
        {
            let _ = w.hide();
        } else if let Some((_, screen, _)) = inner.anchor {
            native::frame(&w, screen, true);
            let _ = w.show();
        }
    }
    if !inner.suspended && !inner.preview {
        if let Some((anchor, _, work)) = inner.anchor {
            if let Some(w) = app.get_webview_window("capsule") {
                let (width, height, near) = (
                    inner.width,
                    inner.height,
                    inner.snapshot.settings.near_caret,
                );
                let r = capsule_frame(&mut inner, &w, anchor, work, width, height, near);
                native::frame(&w, r, false);
                inner.last_frame = Some(r);
                if inner.snapshot.phase != "idle" {
                    let _ = w.show();
                }
            }
        }
    }
}
#[tauri::command]
pub async fn layout_voice_capsule(
    app: tauri::AppHandle,
    width: f64,
    height: f64,
    visible: bool,
    phase: String,
) -> Result<(), String> {
    if !(36.0..=800.0).contains(&width) || !(36.0..=800.0).contains(&height) {
        return Err("Invalid capsule size".into());
    }
    #[cfg(target_os = "macos")]
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let handle = app.clone();
        app.run_on_main_thread(move || {
            let service = handle.state::<Service>();
            let mut inner = service.inner.lock().unwrap_or_else(|e| e.into_inner());
            if !inner.preview && inner.snapshot.phase != phase {
                let _ = tx.send(());
                return;
            }
            inner.width = width;
            inner.height = height;
            if let Some(w) = handle.get_webview_window("capsule") {
                let (a, _, work) = inner.anchor.unwrap_or_else(|| native::anchor(None));
                let near = inner.snapshot.phase != "idle" && inner.snapshot.settings.near_caret;
                let r = capsule_frame(&mut inner, &w, a, work, width, height, near);
                native::frame(&w, r, false);
                inner.last_frame = Some(r);
                if visible && !inner.suspended {
                    let _ = w.show();
                } else {
                    let _ = w.hide();
                }
            }
            let _ = tx.send(());
        })
        .map_err(|e| e.to_string())?;
        rx.await.map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
pub fn preview_voice_feedback(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let service = app.state::<Service>();
        let mut inner = service.inner.lock().unwrap_or_else(|e| e.into_inner());
        if inner.snapshot.phase != "idle" {
            return Err("Finish recording before previewing".into());
        }
        inner.snapshot.failed = false;
        inner.preview = true;
        inner.snapshot.preview = true;
        inner.snapshot.phase = "recording".into();
        inner.snapshot.revision += 1;
        let generation = service.generation.fetch_add(1, Ordering::SeqCst) + 1;
        drop(inner);
        let handle = app.clone();
        app.run_on_main_thread(move || {
            let service = handle.state::<Service>();
            let mut inner = service.inner.lock().unwrap_or_else(|e| e.into_inner());
            if service.generation.load(Ordering::SeqCst) != generation {
                return;
            }
            inner.anchor = Some(native::anchor(None));
            drop(inner);
            render_now(&handle);
        })
        .map_err(|e| e.to_string())?;
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let service = app.state::<Service>();
            let mut inner = service.inner.lock().unwrap_or_else(|e| e.into_inner());
            if inner.finish_preview(&service.generation, generation) {
                drop(inner);
                render(&app);
            }
        });
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn capsule_frame(
    inner: &mut Inner,
    w: &tauri::WebviewWindow,
    a: native::Rect,
    work: native::Rect,
    width: f64,
    height: f64,
    near: bool,
) -> native::Rect {
    let mut r = native::place(a, work, width, height, near);
    // Retain a user's drag across phase/size changes; clamp to the current work area.
    if let (Some(last), Some(current)) = (inner.last_frame, native::current_frame(w)) {
        if (current.origin.x - last.origin.x).abs() > 1.0
            || (current.origin.y - last.origin.y).abs() > 1.0
        {
            inner.dragged = Some(current.origin);
        }
    }
    if let Some(current) = inner.dragged {
        r.origin.x = current.x.clamp(
            work.origin.x + 12.0,
            (work.origin.x + work.size.width - width - 12.0).max(work.origin.x + 12.0),
        );
        r.origin.y = current.y.clamp(
            work.origin.y + 12.0,
            (work.origin.y + work.size.height - height - 12.0).max(work.origin.y + 12.0),
        );
    }
    r
}
#[cfg(target_os = "macos")]
fn environment_changed(app: &tauri::AppHandle, suspend: Option<bool>) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let service = handle.state::<Service>();
        let mut inner = service.inner.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(suspend) = suspend {
            inner.suspended = suspend;
        }
        if let Some(saved) = inner.anchor {
            inner.anchor = Some(native::reconcile(saved));
            inner.last_frame = None;
            inner.dragged = None;
        }
        if inner.suspended {
            if let Some(w) = handle.get_webview_window("capsule") {
                let _ = w.hide();
            }
        } else if inner.snapshot.phase != "idle" && !inner.preview {
            if let Some(w) = handle.get_webview_window("capsule") {
                let _ = w.show();
            }
        }
        drop(inner);
        render_now(&handle);
    });
}

pub fn shutdown(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    if let Some(service) = app.try_state::<Service>() {
        let mut inner = service.inner.lock().unwrap_or_else(|e| e.into_inner());
        service.generation.fetch_add(1, Ordering::SeqCst);
        native::remove_observers(std::mem::take(&mut inner.observers));
        if let Some(w) = app.get_webview_window("voice-edge") {
            let _ = w.hide();
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    #[test]
    fn old_caret_result_cannot_survive_cancel_and_restart() {
        let service = Service::new(Settings::default());
        let mut inner = service.inner.lock().unwrap();
        let (start, old) = inner.advance("preparing".into(), &service.generation);
        assert!(start);
        assert!(!inner.advance("recording".into(), &service.generation).0);
        inner.advance("idle".into(), &service.generation);
        let (_, new) = inner.advance("preparing".into(), &service.generation);
        assert_ne!(old, new);
        assert_eq!(new, service.generation.load(Ordering::SeqCst));
    }
    #[test]
    fn preview_timeout_cannot_hide_real_recording() {
        let service = Service::new(Settings::default());
        let mut inner = service.inner.lock().unwrap();
        inner.preview = true;
        inner.snapshot.preview = true;
        inner.snapshot.phase = "recording".into();
        let old = service.generation.fetch_add(1, Ordering::SeqCst) + 1;
        assert!(inner.advance("preparing".into(), &service.generation).0);
        assert!(!inner.finish_preview(&service.generation, old));
        assert_eq!(inner.snapshot.phase, "preparing");
    }
    #[test]
    fn preview_expires_without_changing_real_pipeline() {
        let service = Service::new(Settings::default());
        let mut inner = service.inner.lock().unwrap();
        inner.preview = true;
        inner.snapshot.preview = true;
        inner.snapshot.phase = "recording".into();
        assert!(inner.finish_preview(&service.generation, 0));
        assert_eq!(inner.snapshot.phase, "idle");
        assert!(!inner.finish_preview(&service.generation, 0));
    }
}
