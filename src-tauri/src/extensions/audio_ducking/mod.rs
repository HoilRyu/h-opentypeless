//! Fork-only output attenuation; no server or provider dependencies.
mod engine;
pub(crate) use engine::save_json as save_config_json;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
use engine::{Engine, Mode};
#[cfg(target_os = "linux")]
use linux::Native;
#[cfg(target_os = "macos")]
use macos::Native;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};
use tauri::Manager;
#[cfg(target_os = "windows")]
use windows::Native;
#[derive(Clone, Serialize, Deserialize)]
pub struct Status {
    mode: Mode,
    volume_percent: u8,
    active: bool,
    warning: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(default)]
    mode: Mode,
    #[serde(default = "default_volume_percent")]
    volume_percent: u8,
}
fn default_volume_percent() -> u8 {
    20
}
enum Message {
    Begin(&'static str, Mode, u8, tokio::sync::oneshot::Sender<()>),
    End(&'static str),
    Shutdown(mpsc::Sender<()>),
}
pub struct Service {
    tx: mpsc::Sender<Message>,
    status: Arc<Mutex<Status>>,
    config: PathBuf,
}
fn update_status(status: &Mutex<Status>, active: bool, result: Result<(), String>) {
    let mut status = status.lock().unwrap_or_else(|e| e.into_inner());
    status.active = active;
    if let Err(error) = result {
        if status.warning.as_ref() != Some(&error) {
            tracing::warn!("Audio attenuation: {error}");
        }
        status.warning = Some(error);
    }
}
impl Service {
    pub fn new(dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let config = dir.join("h-audio-ducking.json");
        let loaded = std::fs::read(&config)
            .ok()
            .and_then(|v| serde_json::from_slice::<Config>(&v).ok())
            .unwrap_or(Config {
                mode: Mode::Off,
                volume_percent: 20,
            });
        let status = Arc::new(Mutex::new(Status {
            mode: loaded.mode,
            volume_percent: loaded.volume_percent.min(100),
            active: false,
            warning: None,
        }));
        let state = status.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("h-audio-ducking".into())
            .spawn(move || {
                let mut engine = Engine::new(Native, dir.join("h-audio-recovery.json"));
                update_status(&state, false, engine.recover());
                let mut owners = std::collections::HashSet::new();
                loop {
                    match rx.recv_timeout(Duration::from_millis(250)) {
                        Ok(Message::Begin(owner, mode, volume_percent, done)) => {
                            let first = owners.is_empty();
                            owners.insert(owner);
                            state.lock().unwrap_or_else(|e| e.into_inner()).warning = None;
                            let result = if first {
                                engine.begin(mode, volume_percent)
                            } else {
                                Ok(())
                            };
                            update_status(&state, mode != Mode::Off && result.is_ok(), result);
                            let _ = done.send(());
                        }
                        Ok(Message::End(owner)) => {
                            owners.remove(owner);
                            if owners.is_empty() {
                                update_status(&state, false, engine.end());
                            }
                        }
                        Ok(Message::Shutdown(done)) => {
                            update_status(&state, false, engine.end());
                            let _ = done.send(());
                            break;
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            let active = state.lock().unwrap_or_else(|e| e.into_inner()).active;
                            update_status(&state, active, engine.tick());
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            let _ = engine.end();
                            break;
                        }
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self { tx, status, config })
    }
    pub async fn begin(&self, owner: &'static str) {
        let (mode, volume_percent) = {
            let status = self.status.lock().unwrap_or_else(|e| e.into_inner());
            (status.mode, status.volume_percent)
        };
        let (tx, rx) = tokio::sync::oneshot::channel();
        if self
            .tx
            .send(Message::Begin(owner, mode, volume_percent, tx))
            .is_ok()
        {
            let _ = rx.await;
        }
    }
    pub fn end(&self, owner: &'static str) {
        let _ = self.tx.send(Message::End(owner));
    }
    pub fn shutdown(&self) {
        let (tx, rx) = mpsc::channel();
        if self.tx.send(Message::Shutdown(tx)).is_ok() {
            let _ = rx.recv_timeout(Duration::from_secs(5));
        }
    }
}
#[tauri::command]
pub fn get_audio_ducking(app: tauri::AppHandle) -> Result<Status, String> {
    let state = app
        .try_state::<Service>()
        .ok_or("Audio control unavailable")?;
    let value = state
        .status
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    Ok(value)
}
#[tauri::command]
pub fn set_audio_ducking(
    app: tauri::AppHandle,
    mode: Mode,
    volume_percent: Option<u8>,
) -> Result<Status, String> {
    let state = app
        .try_state::<Service>()
        .ok_or("Audio control unavailable")?;
    let mut status = state.status.lock().unwrap_or_else(|e| e.into_inner());
    let volume_percent = volume_percent.unwrap_or(status.volume_percent);
    if volume_percent > 100 {
        return Err("Volume percentage must be between 0 and 100".into());
    }
    engine::save_json(
        &state.config,
        &Config {
            mode,
            volume_percent,
        },
    )?;
    status.mode = mode;
    status.volume_percent = volume_percent;
    Ok(status.clone())
}

#[cfg(all(test, target_os = "macos"))]
mod native_tests {
    use super::engine::Backend;
    use super::*;
    #[test]
    #[ignore = "Changes the current Mac output volume briefly; opt-in local verification"]
    fn actual_output_reduces_and_restores() {
        let mut native = Native;
        let id = native.default_device().unwrap();
        let before = native.read(&id).unwrap().levels;
        let path =
            std::env::temp_dir().join(format!("h-native-audio-{}.json", uuid::Uuid::new_v4()));
        let mut engine = Engine::new(native, path);
        for mode in [Mode::Reduce, Mode::Mute] {
            let can_mute = engine.backend.read(&id).unwrap().can_mute;
            let start = engine.begin(mode, 35);
            let during = engine.backend.read(&id).map(|v| v.levels);
            let stop = engine.end();
            start.unwrap();
            stop.unwrap();
            let during = during.unwrap();
            let after = engine.backend.read(&id).unwrap().levels;
            assert_eq!(before.muted, after.muted);
            for ((before, during), after) in
                before.volume.iter().zip(during.volume).zip(after.volume)
            {
                let expected = if mode == Mode::Reduce {
                    before * 0.35
                } else if can_mute {
                    *before
                } else {
                    0.0
                };
                assert!((during - expected).abs() < 0.002, "mode={mode:?} before={before} during={during} expected={expected} after={after}");
                assert!((after - before).abs() < 0.002);
            }
            if mode == Mode::Mute && can_mute {
                assert!(during.muted);
            }
        }
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    #[test]
    fn old_settings_keep_twenty_percent_default() {
        let config: Config = serde_json::from_str(r#"{"mode":"reduce"}"#).unwrap();
        assert_eq!(config.volume_percent, 20);
        assert_eq!(config.mode, Mode::Reduce);
    }
}
