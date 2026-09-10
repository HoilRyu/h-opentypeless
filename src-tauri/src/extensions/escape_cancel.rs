//! Reserve plain Escape only while a cancellable voice operation is active.
use crate::pipeline::{PipelineHandle, PipelineState};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Default)]
pub struct Service {
    active: AtomicBool,
    generation: AtomicU64,
}
pub fn shortcut() -> Shortcut {
    Shortcut::new(None, Code::Escape)
}
pub fn matches(key: &Shortcut) -> bool {
    key.id() == shortcut().id()
}
fn cancellable(state: PipelineState) -> bool {
    matches!(
        state,
        PipelineState::Preparing
            | PipelineState::Recording
            | PipelineState::Transcribing
            | PipelineState::Polishing
            | PipelineState::AskRecording
            | PipelineState::AskThinking
    )
}
pub fn transition(app: &tauri::AppHandle, state: PipelineState) {
    let Some(service) = app.try_state::<Service>() else {
        return;
    };
    let active = cancellable(state);
    if active && !service.active.load(Ordering::SeqCst) {
        service.generation.fetch_add(1, Ordering::SeqCst);
    }
    service.active.store(active, Ordering::SeqCst);
    refresh(app);
}
pub fn refresh(app: &tauri::AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let Some(service) = handle.try_state::<Service>() else {
            return;
        };
        let active = service.active.load(Ordering::SeqCst);
        let shortcuts = handle.global_shortcut();
        let registered = shortcuts.is_registered(shortcut());
        let result = if active && !registered {
            shortcuts.register(shortcut())
        } else if !active && registered {
            shortcuts.unregister(shortcut())
        } else {
            return;
        };
        if let Err(error) = result {
            tracing::warn!("Escape cancellation shortcut unavailable: {error}");
        }
    });
}
pub fn dispatch(app: &tauri::AppHandle, event: ShortcutState) {
    let Some(service) = app.try_state::<Service>() else {
        return;
    };
    let generation = service.generation.load(Ordering::SeqCst);
    let app = app.clone();
    // The shortcut plugin invokes callbacks while holding its registry mutex.
    // Cancellation unregisters Escape; never re-enter that mutex in the callback.
    tauri::async_runtime::spawn(async move {
        handle(&app, event, generation);
    });
}
fn handle(app: &tauri::AppHandle, event: ShortcutState, generation: u64) {
    if event != ShortcutState::Pressed {
        return;
    }
    let Some(service) = app.try_state::<Service>() else {
        return;
    };
    if !service.active.load(Ordering::SeqCst)
        || service.generation.load(Ordering::SeqCst) != generation
    {
        return;
    }
    crate::audio::lifecycle::event("escape_cancel_requested");
    let ask = app.state::<crate::commands::ask::AskDictationState>();
    if ask.is_busy() {
        let _ = crate::commands::ask::abort_ask_dictation(app.clone(), ask);
    } else if let Some(pipeline) = app.try_state::<PipelineHandle>() {
        if cancellable(pipeline.current_state()) {
            pipeline.abort();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_processing_phase_can_cancel_but_idle_and_completed_output_cannot() {
        for state in [
            PipelineState::Preparing,
            PipelineState::Recording,
            PipelineState::Transcribing,
            PipelineState::Polishing,
            PipelineState::AskRecording,
            PipelineState::AskThinking,
        ] {
            assert!(cancellable(state));
        }
        assert!(!cancellable(PipelineState::Idle));
        assert!(!cancellable(PipelineState::Outputting));
    }
    #[test]
    fn only_plain_escape_is_reserved() {
        assert!(matches(&shortcut()));
        assert!(!matches(&Shortcut::new(
            Some(tauri_plugin_global_shortcut::Modifiers::CONTROL),
            Code::Escape
        )));
        assert!(!matches(&Shortcut::new(None, Code::Space)));
    }
}
