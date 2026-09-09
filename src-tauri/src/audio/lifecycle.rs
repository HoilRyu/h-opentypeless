//! Serialize device transitions and stop issuing commands after uncertain completion.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, MutexGuard, TryLockError,
};
use std::time::{Duration, Instant};
static TRANSITION: Mutex<()> = Mutex::new(());
static FAILED: AtomicBool = AtomicBool::new(false);
pub fn fail() {
    event("audio_transitions_disabled");
    FAILED.store(true, Ordering::SeqCst);
}
pub fn check() -> Result<(), String> {
    if FAILED.load(Ordering::SeqCst) {
        Err(
            "Audio device completion is uncertain. Restart H after checking the audio device."
                .into(),
        )
    } else {
        Ok(())
    }
}
pub fn enter() -> Result<MutexGuard<'static, ()>, String> {
    check()?;
    let until = Instant::now() + Duration::from_secs(3);
    loop {
        match TRANSITION.try_lock() {
            Ok(g) => {
                check()?;
                return Ok(g);
            }
            Err(TryLockError::Poisoned(_)) => {
                fail();
                return Err("Audio transition lock poisoned".into());
            }
            Err(TryLockError::WouldBlock) if Instant::now() < until => {
                std::thread::sleep(Duration::from_millis(10))
            }
            _ => {
                fail();
                return Err("Audio device transition timed out".into());
            }
        }
    }
}

// Teardown must wait even after a fault: never race an in-flight device call.
pub fn teardown() -> MutexGuard<'static, ()> {
    TRANSITION.lock().unwrap_or_else(|e| e.into_inner())
}

static LOG: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
static LOG_LOCK: Mutex<()> = Mutex::new(());
pub fn init_log(path: std::path::PathBuf) {
    let _ = LOG.set(path);
    event("audio_service_started");
}
pub fn event(name: &'static str) {
    let Some(path) = LOG.get() else {
        return;
    };
    let Ok(_guard) = LOG_LOCK.lock() else {
        return;
    };
    // Two bounded files, metadata only. No device identifiers or speech content.
    if std::fs::metadata(path).is_ok_and(|m| m.len() > 256 * 1024) {
        let previous = path.with_extension("previous.log");
        if previous.exists() && std::fs::remove_file(&previous).is_err() {
            return;
        }
        if std::fs::rename(path, previous).is_err() {
            return;
        }
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        use std::io::Write;
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let _ = writeln!(file, "{time} {name}");
    }
}

#[cfg(any(test, target_os = "macos"))]
pub fn wait_confirmation(
    mut confirmed: impl FnMut() -> Result<bool, String>,
    timeout: Duration,
) -> Result<(), String> {
    let until = Instant::now() + timeout;
    loop {
        if confirmed()? {
            return Ok(());
        }
        if Instant::now() >= until {
            return Err("Audio property completion timed out".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notification_without_matching_readback_does_not_complete() {
        let mut reads = 0;
        wait_confirmation(
            || {
                reads += 1;
                Ok(reads >= 3)
            },
            Duration::from_millis(100),
        )
        .unwrap();
        assert_eq!(reads, 3);
    }
    #[test]
    fn missing_completion_is_bounded_and_errors_propagate() {
        assert!(wait_confirmation(|| Ok(false), Duration::from_millis(15)).is_err());
        assert_eq!(
            wait_confirmation(|| Err("disconnected".into()), Duration::from_secs(1)),
            Err("disconnected".into())
        );
    }
}
