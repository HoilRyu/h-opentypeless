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
    recovery_pending: bool,
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
    Disable,
    Shutdown(mpsc::Sender<()>),
}
pub struct Service {
    tx: mpsc::SyncSender<Message>,
    status: Arc<Mutex<Status>>,
    config: PathBuf,
}
fn update_status(status: &Mutex<Status>, active: bool, result: Result<(), String>) {
    let mut status = status.lock().unwrap_or_else(|e| e.into_inner());
    status.active = active;
    if result.is_ok() {
        status.warning = None;
    }
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
        crate::audio::lifecycle::init_log(dir.join("h-audio-events.log"));
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
            recovery_pending: false,
        }));
        let state = status.clone();
        let (tx, rx) = mpsc::sync_channel(64);
        std::thread::Builder::new()
            .name("h-audio-ducking".into())
            .spawn(move || {
                let mut engine = Engine::new(Native, dir.join("h-audio-recovery.json"));
                // Never touch hardware on startup while the feature is disabled.
                if loaded.mode != Mode::Off {
                    update_status(&state, false, engine.recover());
                }
                let mut owners = std::collections::HashSet::new();
                loop {
                    state
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .recovery_pending = engine.recovery_pending();
                    match rx.recv_timeout(Duration::from_millis(250)) {
                        Ok(Message::Begin(owner, mode, volume_percent, done)) => {
                            if done.is_closed() {
                                continue;
                            }
                            let first = owners.is_empty();
                            owners.insert(owner);
                            if first {
                                let result = engine.begin(mode, volume_percent);
                                update_status(&state, mode != Mode::Off && result.is_ok(), result);
                            }
                            let _ = done.send(());
                        }
                        Ok(Message::Disable) => {
                            update_status(&state, false, engine.end());
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
                            // An idle tick is not a successful recovery. Keep the
                            // failure visible after the bounded retries expire.
                            if engine.has_tick_work() {
                                let active = state.lock().unwrap_or_else(|e| e.into_inner()).active;
                                let result = engine.tick();
                                update_status(&state, active && result.is_ok(), result);
                            }
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
            .try_send(Message::Begin(owner, mode, volume_percent, tx))
            .is_err()
        {
            crate::audio::lifecycle::fail();
            return;
        }
        if !matches!(
            tokio::time::timeout(Duration::from_secs(3), rx).await,
            Ok(Ok(()))
        ) {
            crate::audio::lifecycle::fail();
        }
    }

    pub fn end(&self, owner: &'static str) {
        if self.tx.try_send(Message::End(owner)).is_err() {
            crate::audio::lifecycle::fail();
        }
    }
    pub fn shutdown(&self) {
        let (tx, rx) = mpsc::channel();
        if self.tx.try_send(Message::Shutdown(tx)).is_ok() {
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
    if mode == Mode::Off && state.tx.try_send(Message::Disable).is_err() {
        crate::audio::lifecycle::fail();
        return Err("Audio control queue unavailable".into());
    }
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
        // Reproduce the user's stale, unconfirmed Bluetooth record while
        // the actual default output is a different device.
        engine::save_json(
            &path,
            &serde_json::json!({
                "id": "h-test-disconnected-output",
                "before": { "volume": [0.007874, 0.007874], "muted": false },
                "applied": { "volume": [0.000157, 0.000157], "muted": false },
                "confirmed": false
            }),
        )
        .unwrap();
        let mut engine = Engine::new(native, path.clone());
        for (mode, percent) in [(Mode::Reduce, 5), (Mode::Reduce, 35), (Mode::Mute, 35)] {
            let can_mute = engine.backend.read(&id).unwrap().can_mute
                && !engine.backend.prefer_volume_for_mute();
            let start = engine.begin(mode, percent);
            let during = engine.backend.read(&id).map(|v| v.levels);
            let stop = engine.end();
            start.unwrap();
            stop.unwrap();
            let during = during.unwrap();
            let after = engine.backend.read(&id).unwrap().levels;
            eprintln!("mode={mode:?} before={before:?} during={during:?} after={after:?}");
            assert_eq!(before.muted, after.muted);
            for ((before, during), after) in
                before.volume.iter().zip(during.volume).zip(after.volume)
            {
                // USB hardware quantizes the requested percentage. The actual
                // confirmed step must attenuate and then restore the original.
                if mode == Mode::Reduce {
                    assert!(during <= *before && during >= 0.0);
                } else if !can_mute {
                    assert_eq!(during, 0.0);
                }
                assert!((after - before).abs() < 0.002);
            }
            if mode == Mode::Mute && can_mute {
                assert!(during.muted);
            } else if mode == Mode::Mute {
                assert_eq!(during.muted, before.muted);
            }
        }
        use sha2::{Digest, Sha256};
        let deferred = path.with_extension(format!(
            "{:x}.json",
            Sha256::digest(b"h-test-disconnected-output")
        ));
        assert!(deferred.exists());
        std::fs::remove_file(deferred).unwrap();
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

#[cfg(test)]
mod status_tests {
    use super::*;
    #[test]
    fn confirmed_success_clears_previous_warning() {
        let status = Mutex::new(Status {
            mode: Mode::Reduce,
            volume_percent: 20,
            active: false,
            warning: None,
            recovery_pending: false,
        });
        update_status(&status, false, Err("device disconnected".into()));
        assert!(status.lock().unwrap().warning.is_some());
        update_status(&status, false, Ok(()));
        assert!(status.lock().unwrap().warning.is_none());
    }
}
