//! H's bounded practice session: microphone -> existing providers -> preview only.
//! Never invokes desktop typing, clipboard, history, or the public mobile listener.
use crate::audio::{AudioCaptureHandle, AudioConfig};
use serde::Serialize;
use std::{
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tauri::{Emitter, Manager};
use tokio::sync::watch;

struct Session {
    id: String,
    signal: watch::Sender<u8>,
}
static SESSION: OnceLock<Mutex<Option<Session>>> = OnceLock::new();
fn slot() -> &'static Mutex<Option<Session>> {
    SESSION.get_or_init(|| Mutex::new(None))
}
struct Lease;
impl Drop for Lease {
    fn drop(&mut self) {
        *slot().lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}
fn require_main(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() == "main" {
        Ok(())
    } else {
        Err("Practice is only available in the main window".into())
    }
}
fn signal(id: &str, cancel: bool) {
    if let Some(session) = slot().lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        if session.id == id {
            session.signal.send_replace(if cancel { 2 } else { 1 });
        }
    }
}
#[tauri::command]
pub fn control_tutorial_recording(
    window: tauri::WebviewWindow,
    id: String,
    cancel: bool,
) -> Result<(), String> {
    require_main(&window)?;
    signal(&id, cancel);
    Ok(())
}
// Cancellation can drop the recording future while the native stream is starting.
// Keep its bounded shutdown off Tokio's worker threads even on that path.
struct PracticeCapture(Option<AudioCaptureHandle>);
impl Drop for PracticeCapture {
    fn drop(&mut self) {
        if let Some(mut capture) = self.0.take() {
            tokio::task::spawn_blocking(move || capture.stop());
        }
    }
}
#[derive(Serialize)]
pub struct Preview {
    raw_text: String,
    polished_text: String,
    warning: Option<String>,
    processing_ms: u128,
}
#[tauri::command]
pub async fn run_tutorial_recording(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    id: String,
) -> Result<Preview, String> {
    require_main(&window)?;
    if id.is_empty() || id.len() > 64 {
        return Err("Invalid practice session".into());
    }
    let (tx, mut stop) = watch::channel(0u8);
    {
        let mut current = slot().lock().unwrap_or_else(|e| e.into_inner());
        if current.is_some() {
            return Err("A practice session is already running".into());
        }
        *current = Some(Session {
            id: id.clone(),
            signal: tx,
        });
    }
    let _lease = Lease;
    let mut cancel = stop.clone();
    let _ = app.emit_to(
        "main",
        "h-tutorial:phase",
        serde_json::json!({"id": id, "phase":"preparing"}),
    );
    let operation = async {
        let config = app
            .state::<crate::storage::ConfigManager>()
            .load()
            .await
            .map_err(|e| e.to_string())?;
        if !super::mobile::processing::supported(&config.stt_provider) {
            return Err("This preview supports built-in STT and Whisper-compatible APIs. You can skip practice and use your configured provider normally.".into());
        }
        let (capture, mut audio) =
            AudioCaptureHandle::start(AudioConfig::default()).map_err(|e| e.to_string())?;
        let mut capture = PracticeCapture(Some(capture));
        tokio::time::timeout(
            Duration::from_secs(30),
            capture.0.as_mut().unwrap().wait_until_ready(),
        )
        .await
        .map_err(|_| "Microphone preparation timed out")?
        .map_err(|e| e.to_string())?;
        let _ = app.emit_to(
            "main",
            "h-tutorial:phase",
            serde_json::json!({"id": id, "phase":"recording"}),
        );
        let deadline = tokio::time::sleep(Duration::from_secs(15));
        tokio::pin!(deadline);
        let mut pcm = Vec::new();
        loop {
            tokio::select! {
                biased;
                _ = stop.wait_for(|v| *v != 0) => break,
                _ = &mut deadline => break,
                chunk = audio.recv() => match chunk {
                    Some(chunk) => {
                        if pcm.len() + chunk.len() > 15 * 16000 * 2 { break; }
                        pcm.extend_from_slice(&chunk);
                    },
                    None => return Err("Microphone stream ended unexpectedly".into()),
                }
            }
        }
        // Teardown may wait for native audio; never block the async worker pool.
        let mut native_capture = capture.0.take().unwrap();
        tokio::task::spawn_blocking(move || native_capture.stop())
            .await
            .map_err(|e| e.to_string())?;
        drop(audio);
        if pcm.len() < 3200 {
            return Err("Record a short sentence before stopping".into());
        }
        let _ = app.emit_to(
            "main",
            "h-tutorial:phase",
            serde_json::json!({"id":id, "phase":"processing"}),
        );
        let started = std::time::Instant::now();
        let text = super::mobile::processing::process(&app, &pcm).await?;
        Ok(Preview {
            raw_text: text.raw_text,
            polished_text: text.polished_text,
            warning: text.warning,
            processing_ms: started.elapsed().as_millis(),
        })
    };
    tokio::select! {
        biased;
        _ = cancel.wait_for(|v| *v == 2) => Err("Practice cancelled".into()),
        result = tokio::time::timeout(Duration::from_secs(210), operation) => result.unwrap_or_else(|_| Err("Practice timed out; check your model connection".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_cancel_cannot_interrupt_another_session() {
        let (tx, rx) = watch::channel(0);
        *slot().lock().unwrap() = Some(Session {
            id: "current".into(),
            signal: tx,
        });
        let _lease = Lease;
        signal("old", true);
        assert_eq!(*rx.borrow(), 0);
        signal("current", false);
        assert_eq!(*rx.borrow(), 1);
        signal("current", true);
        assert_eq!(*rx.borrow(), 2);
    }
}
