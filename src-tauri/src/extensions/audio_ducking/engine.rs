use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf};
pub type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Off,
    Reduce,
    Mute,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Levels {
    pub volume: Vec<f32>,
    pub muted: bool,
}
impl Levels {
    fn matches(&self, other: &Self) -> bool {
        self.muted == other.muted
            && self.volume.len() == other.volume.len()
            && self.volume.iter().zip(&other.volume).all(|(a, b)| {
                if *a == 0.0 || *b == 0.0 {
                    a == b
                } else {
                    (a - b).abs() < 0.002
                }
            })
    }
    fn valid(&self) -> bool {
        !self.volume.is_empty()
            && self
                .volume
                .iter()
                .all(|v| v.is_finite() && (0.0..=4.0).contains(v))
    }
}
pub struct Device {
    pub levels: Levels,
    pub can_mute: bool,
}
pub trait Backend: Send {
    // Some backends avoid the hardware mute switch even when it is writable.
    fn prefer_volume_for_mute(&self) -> bool {
        false
    }
    fn default_device(&mut self) -> Result<String>;
    fn read(&mut self, id: &str) -> Result<Device>;
    fn write(&mut self, id: &str, from: &Levels, to: &Levels) -> Result<()>;
    fn write_observed(&mut self, id: &str, from: &Levels, to: &Levels) -> Result<Levels> {
        self.write(id, from, to)?;
        Ok(to.clone())
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Journal {
    id: String,
    before: Levels,
    applied: Levels,
    #[serde(default = "confirmed_default")]
    confirmed: bool,
}
fn confirmed_default() -> bool {
    true
}
// Write the recovery record before touching the device. A failed disk write
// disables ducking; it must never leave an unrecorded volume change behind.
pub fn save_json(path: &std::path::Path, value: &impl Serialize) -> Result<()> {
    let data = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut f = fs::File::create(&temporary).map_err(|e| e.to_string())?;
        use std::io::Write;
        f.write_all(&data)
            .and_then(|_| f.sync_all())
            .map_err(|e| e.to_string())?;
        drop(f);
        fs::rename(&temporary, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
pub struct Engine<B> {
    pub backend: B,
    journal: PathBuf,
    legacy_journal: PathBuf,
    pending: Option<Journal>,
    mode: Mode,
    volume_percent: u8,
    seen: HashSet<String>,
    restore_retries: u8,
}
impl<B: Backend> Engine<B> {
    pub fn new(backend: B, journal: PathBuf) -> Self {
        Self {
            backend,
            legacy_journal: journal.clone(),
            journal,
            pending: None,
            mode: Mode::Off,
            volume_percent: 20,
            seen: HashSet::new(),
            restore_retries: 0,
        }
    }
    fn device_journal(&self, id: &str) -> PathBuf {
        use sha2::{Digest, Sha256};
        self.legacy_journal
            .with_extension(format!("{:x}.json", Sha256::digest(id.as_bytes())))
    }
    fn select_device(&mut self, id: &str) -> Result<()> {
        // Migrate the old singleton journal without touching its device. A
        // disconnected headset must not block a different output indefinitely.
        if self.legacy_journal.exists() {
            let old: Journal =
                serde_json::from_slice(&fs::read(&self.legacy_journal).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let target = self.device_journal(&old.id);
            if target.exists() {
                return Err("Conflicting audio recovery records".into());
            }
            fs::rename(&self.legacy_journal, target).map_err(|e| e.to_string())?;
        }
        let target = self.device_journal(id);
        if self.journal != target {
            if self.pending.is_some() {
                // Best effort for the previous output. Its durable record stays
                // available until that device becomes the default output again.
                if self.restore().is_err() {
                    crate::audio::lifecycle::event("volume_recovery_deferred");
                }
            }
            self.pending = None;
            self.restore_retries = 0;
            self.journal = target;
        }
        if self.pending.is_none() && self.journal.exists() {
            let record: Journal =
                serde_json::from_slice(&fs::read(&self.journal).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            if record.id != id {
                return Err("Audio recovery device mismatch".into());
            }
            self.pending = Some(record);
        }
        Ok(())
    }
    pub fn recover(&mut self) -> Result<()> {
        let id = self.backend.default_device()?;
        self.select_device(&id)?;
        self.restore_retries = 4;
        self.restore()
    }
    fn restore(&mut self) -> Result<()> {
        if let Some(j) = self.pending.clone() {
            if !j.before.valid() || !j.applied.valid() {
                return Err("Invalid audio recovery record".into());
            }
            let current = self.backend.read(&j.id)?.levels;
            if !current.valid() {
                return Err("Invalid device levels during recovery".into());
            }
            if !j.confirmed && !current.matches(&j.applied) {
                return Err("Unconfirmed audio change; recovery record retained".into());
            }
            if current.volume.len() != j.before.volume.len()
                || current.volume.len() != j.applied.volume.len()
            {
                return Err("Output channel layout changed during recovery".into());
            }
            // Restore only properties we changed, and only if they still have
            // our value. A manual mute must not strand an attenuated volume,
            // and a manual volume change must survive an automatic unmute.
            let mut restored = current.clone();
            for (i, value) in restored.volume.iter_mut().enumerate() {
                if j.before.volume[i] != j.applied.volume[i]
                    && (*value - j.applied.volume[i]).abs() < 0.002
                {
                    *value = j.before.volume[i];
                }
            }
            if j.before.muted != j.applied.muted && current.muted == j.applied.muted {
                restored.muted = j.before.muted;
            }
            if !current.matches(&restored) {
                crate::audio::lifecycle::event("volume_restore_requested");
                self.backend.write(&j.id, &current, &restored)?;
            }
            if self.journal.exists() {
                fs::remove_file(&self.journal).map_err(|e| e.to_string())?;
            }
            self.pending = None;
            self.restore_retries = 0;
            crate::audio::lifecycle::event("volume_restore_completed");
        }
        Ok(())
    }
    pub fn begin(&mut self, mode: Mode, volume_percent: u8) -> Result<()> {
        if volume_percent > 100 {
            return Err("Volume percentage must be between 0 and 100".into());
        }
        if self.mode != Mode::Off {
            return Ok(());
        }
        if mode == Mode::Off {
            return Ok(());
        }
        self.recover()?;
        self.seen.clear();
        self.mode = mode;
        self.volume_percent = volume_percent;
        self.tick()
    }
    pub fn end(&mut self) -> Result<()> {
        self.mode = Mode::Off;
        self.restore_retries = 4;
        self.restore()
    }
    pub fn has_tick_work(&self) -> bool {
        self.mode != Mode::Off || (self.pending.is_some() && self.restore_retries > 0)
    }
    pub fn recovery_pending(&self) -> bool {
        self.mode == Mode::Off && self.pending.is_some()
    }
    pub fn tick(&mut self) -> Result<()> {
        let result = self.tick_inner();
        if result.is_err() {
            self.mode = Mode::Off;
        }
        result
    }
    fn tick_inner(&mut self) -> Result<()> {
        if self.mode == Mode::Off {
            if self.pending.is_some() && self.restore_retries > 0 {
                self.restore_retries -= 1;
                crate::audio::lifecycle::event("volume_restore_retry");
                return self.restore();
            }
            return Ok(());
        }
        let id = self.backend.default_device()?;
        if self.journal != self.device_journal(&id) {
            self.select_device(&id)?;
            self.restore()?;
        }
        // A device is changed at most once per recording, including after a
        // manual adjustment or a device switch away and back.
        if !self.seen.insert(id.clone()) {
            return Ok(());
        }
        let device = self.backend.read(&id)?;
        if !device.levels.valid() {
            return Err("Unsupported output volume".into());
        }
        let mut applied = device.levels.clone();
        match self.mode {
            Mode::Off => return Ok(()),
            Mode::Reduce => applied
                .volume
                .iter_mut()
                .for_each(|v| *v *= self.volume_percent as f32 / 100.0),
            Mode::Mute if device.can_mute && !self.backend.prefer_volume_for_mute() => {
                applied.muted = true
            }
            Mode::Mute => applied.volume.fill(0.0),
        }
        if applied.matches(&device.levels) {
            return Ok(());
        }
        let j = Journal {
            id,
            before: device.levels,
            applied,
            confirmed: false,
        };
        save_json(&self.journal, &j)?;
        self.pending = Some(j.clone());
        let applied = self.backend.write_observed(&j.id, &j.before, &j.applied)?;
        if !applied.valid() || applied.volume.len() != j.applied.volume.len() {
            return Err("Invalid confirmed output levels".into());
        }
        let mut confirmed = j;
        confirmed.applied = applied;
        confirmed.confirmed = true;
        save_json(&self.journal, &confirmed)?;
        self.pending = Some(confirmed);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Fake {
        id: String,
        values: std::collections::HashMap<String, Levels>,
        fail: bool,
        volume_mute: bool,
    }
    impl Backend for Fake {
        fn prefer_volume_for_mute(&self) -> bool {
            self.volume_mute
        }
        fn default_device(&mut self) -> Result<String> {
            Ok(self.id.clone())
        }
        fn read(&mut self, id: &str) -> Result<Device> {
            Ok(Device {
                levels: self.values.get(id).ok_or("missing")?.clone(),
                can_mute: true,
            })
        }
        fn write(&mut self, id: &str, _: &Levels, to: &Levels) -> Result<()> {
            if self.fail {
                return Err("failed".into());
            }
            self.values.insert(id.into(), to.clone());
            Ok(())
        }
    }
    #[test]
    fn exhausted_retry_is_not_a_successful_idle_tick() {
        let mut e = engine();
        e.begin(Mode::Reduce, 20).unwrap();
        e.backend.fail = true;
        assert!(e.end().is_err());
        for _ in 0..4 {
            assert!(e.has_tick_work());
            assert!(e.tick().is_err());
        }
        assert!(!e.has_tick_work());
        assert!(e.recovery_pending());
        e.backend.fail = false;
        e.recover().unwrap();
        assert!(!e.recovery_pending());
        assert!(!e.has_tick_work());
        assert_eq!(e.backend.values["a"].volume, vec![0.8, 0.4]);
    }
    fn engine() -> Engine<Fake> {
        Engine::new(
            Fake {
                id: "a".into(),
                volume_mute: false,
                values: [
                    (
                        "a".into(),
                        Levels {
                            volume: vec![0.8, 0.4],
                            muted: false,
                        },
                    ),
                    (
                        "b".into(),
                        Levels {
                            volume: vec![0.6],
                            muted: false,
                        },
                    ),
                ]
                .into(),
                fail: false,
            },
            std::env::temp_dir().join(format!("h-audio-test-{}.json", uuid::Uuid::new_v4())),
        )
    }
    #[test]
    fn recovery_uses_confirmed_hardware_step_and_respects_manual_change() {
        struct Quantized(Fake);
        impl Backend for Quantized {
            fn default_device(&mut self) -> Result<String> {
                self.0.default_device()
            }
            fn read(&mut self, id: &str) -> Result<Device> {
                self.0.read(id)
            }
            fn write(&mut self, id: &str, from: &Levels, to: &Levels) -> Result<()> {
                self.0.write(id, from, to)
            }
            fn write_observed(&mut self, id: &str, from: &Levels, to: &Levels) -> Result<Levels> {
                let mut actual = to.clone();
                for value in &mut actual.volume {
                    *value = (*value * 20.0).round() / 20.0;
                }
                self.0.write(id, from, &actual)?;
                Ok(actual)
            }
        }
        let fake = engine();
        let mut e = Engine::new(Quantized(fake.backend), fake.legacy_journal);
        e.begin(Mode::Reduce, 35).unwrap();
        assert_eq!(e.pending.as_ref().unwrap().applied.volume, vec![0.3, 0.15]);
        e.end().unwrap();
        assert_eq!(e.backend.0.values["a"].volume, vec![0.8, 0.4]);
        e.begin(Mode::Reduce, 35).unwrap();
        e.backend.0.values.get_mut("a").unwrap().volume = vec![0.6, 0.2];
        e.end().unwrap();
        assert_eq!(e.backend.0.values["a"].volume, vec![0.6, 0.2]);
    }
    #[test]
    fn disconnected_previous_output_does_not_block_new_output() {
        let mut e = engine();
        e.begin(Mode::Reduce, 20).unwrap();
        let old_path = e.journal.clone();
        let old = e.backend.values.remove("a").unwrap();
        e.backend.id = "b".into();
        e.tick().unwrap();
        assert!(old_path.exists());
        assert!((e.backend.values["b"].volume[0] - 0.12).abs() < 0.002);
        e.end().unwrap();
        assert_eq!(e.backend.values["b"].volume, vec![0.6]);
        e.backend.values.insert("a".into(), old);
        e.backend.id = "a".into();
        e.recover().unwrap();
        assert_eq!(e.backend.values["a"].volume, vec![0.8, 0.4]);
        assert!(!old_path.exists());
    }
    #[test]
    fn legacy_unconfirmed_other_device_is_preserved_without_blocking_mute() {
        let mut e = engine();
        let legacy = Journal {
            id: "disconnected-headset".into(),
            before: Levels {
                volume: vec![0.007874; 2],
                muted: false,
            },
            applied: Levels {
                volume: vec![0.000157; 2],
                muted: false,
            },
            confirmed: false,
        };
        save_json(&e.legacy_journal, &legacy).unwrap();
        let saved = e.device_journal(&legacy.id);
        e.begin(Mode::Mute, 2).unwrap();
        assert!(e.backend.values["a"].muted);
        assert!(saved.exists());
        assert!(!e.legacy_journal.exists());
        e.end().unwrap();
        assert!(!e.backend.values["a"].muted);
        let mut restarted = Engine::new(e.backend, e.legacy_journal);
        restarted.begin(Mode::Mute, 2).unwrap();
        restarted.end().unwrap();
        assert!(saved.exists());
        fs::remove_file(saved).unwrap();
    }
    #[test]
    fn uncertain_delayed_write_preserves_journal_until_observed() {
        let mut e = engine();
        let j = Journal {
            id: "a".into(),
            before: e.backend.values["a"].clone(),
            applied: Levels {
                volume: vec![0.16, 0.08],
                muted: false,
            },
            confirmed: false,
        };
        save_json(&e.journal, &j).unwrap();
        assert!(e.recover().is_err());
        assert!(e.journal.exists());
        e.backend.values.insert("a".into(), j.applied);
        e.end().unwrap();
        assert_eq!(e.backend.values["a"].volume, vec![0.8, 0.4]);
        assert!(!e.journal.exists());
    }
    #[test]
    fn disabled_begin_does_not_recover_or_write_device() {
        let mut e = engine();
        let j = Journal {
            id: "a".into(),
            before: e.backend.values["a"].clone(),
            applied: Levels {
                volume: vec![0.16, 0.08],
                muted: false,
            },
            confirmed: true,
        };
        save_json(&e.journal, &j).unwrap();
        e.backend.values.insert("a".into(), j.applied.clone());
        e.begin(Mode::Off, 20).unwrap();
        assert_eq!(e.backend.values["a"], j.applied);
        assert!(e.journal.exists());
        e.recover().unwrap();
    }
    #[test]
    fn reduce_restores_balance_and_duplicate_start_does_not_compound() {
        let mut e = engine();
        e.begin(Mode::Reduce, 20).unwrap();
        e.begin(Mode::Reduce, 20).unwrap();
        assert!(e.backend.values["a"].matches(&Levels {
            volume: vec![0.16, 0.08],
            muted: false
        }));
        e.end().unwrap();
        assert_eq!(e.backend.values["a"].volume, vec![0.8, 0.4]);
        assert!(!e.journal.exists());
    }
    #[test]
    fn volume_mute_restores_channels_without_toggling_existing_mute() {
        for muted in [false, true] {
            let mut e = engine();
            e.backend.volume_mute = true;
            e.backend.values.get_mut("a").unwrap().muted = muted;
            let original = e.backend.values["a"].clone();
            e.begin(Mode::Mute, 20).unwrap();
            assert_eq!(e.backend.values["a"].volume, vec![0.0, 0.0]);
            assert_eq!(e.backend.values["a"].muted, muted);
            let journal = e.pending.as_ref().unwrap();
            assert_eq!(journal.before.muted, journal.applied.muted);
            e.end().unwrap();
            assert_eq!(e.backend.values["a"], original);
            assert!(!e.journal.exists());
        }
    }
    #[test]
    fn low_volume_is_still_muted_to_exact_zero_and_restored() {
        let mut e = engine();
        e.backend.volume_mute = true;
        e.backend.values.get_mut("a").unwrap().volume = vec![0.001, 0.000157];
        e.begin(Mode::Mute, 2).unwrap();
        assert_eq!(e.backend.values["a"].volume, vec![0.0, 0.0]);
        e.end().unwrap();
        assert_eq!(e.backend.values["a"].volume, vec![0.001, 0.000157]);
    }
    #[test]
    fn volume_mute_preserves_user_volume_and_mute_changes() {
        let mut e = engine();
        e.backend.volume_mute = true;
        e.begin(Mode::Mute, 20).unwrap();
        e.backend.values.get_mut("a").unwrap().volume[0] = 0.3;
        e.backend.values.get_mut("a").unwrap().muted = true;
        e.end().unwrap();
        assert_eq!(e.backend.values["a"].volume, vec![0.3, 0.4]);
        assert!(e.backend.values["a"].muted);
    }
    #[test]
    fn volume_mute_policy_still_recovers_legacy_hardware_mute() {
        let mut e = engine();
        e.begin(Mode::Mute, 20).unwrap();
        assert!(e.backend.values["a"].muted);
        let mut restarted = Engine::new(e.backend, e.legacy_journal);
        restarted.backend.volume_mute = true;
        restarted.recover().unwrap();
        assert!(!restarted.backend.values["a"].muted);
        assert_eq!(restarted.backend.values["a"].volume, vec![0.8, 0.4]);
    }
    #[test]
    fn mute_and_cancel_restore_original_mute_state() {
        let mut e = engine();
        e.begin(Mode::Mute, 20).unwrap();
        assert!(e.backend.values["a"].muted);
        e.end().unwrap();
        assert!(!e.backend.values["a"].muted);
        e.backend.values.get_mut("a").unwrap().muted = true;
        e.begin(Mode::Mute, 20).unwrap();
        e.end().unwrap();
        assert!(e.backend.values["a"].muted);
    }
    #[test]
    fn manual_change_wins_over_restore() {
        let mut e = engine();
        e.begin(Mode::Reduce, 20).unwrap();
        e.backend.values.get_mut("a").unwrap().volume = vec![0.3, 0.2];
        e.end().unwrap();
        assert_eq!(e.backend.values["a"].volume, vec![0.3, 0.2]);
    }
    #[test]
    fn switched_device_restores_old_and_ducks_new() {
        let mut e = engine();
        e.begin(Mode::Reduce, 20).unwrap();
        e.backend.id = "b".into();
        e.tick().unwrap();
        assert_eq!(e.backend.values["a"].volume, vec![0.8, 0.4]);
        assert!((e.backend.values["b"].volume[0] - 0.12).abs() < 0.001);
        e.end().unwrap();
        assert_eq!(e.backend.values["b"].volume, vec![0.6]);
    }
    #[test]
    fn restart_recovers_only_unchanged_applied_volume() {
        let mut e = engine();
        e.begin(Mode::Reduce, 20).unwrap();
        let mut restarted = Engine::new(e.backend, e.legacy_journal);
        restarted.recover().unwrap();
        assert_eq!(restarted.backend.values["a"].volume, vec![0.8, 0.4]);
    }
    #[test]
    fn failed_restore_is_retried_without_another_recording() {
        let mut e = engine();
        e.begin(Mode::Mute, 20).unwrap();
        e.backend.fail = true;
        assert!(e.end().is_err());
        e.backend.fail = false;
        e.tick().unwrap();
        assert!(!e.backend.values["a"].muted);
        assert!(!e.journal.exists());
    }
    #[test]
    fn recovery_retries_are_bounded_and_preserve_manual_adjustment() {
        let mut e = engine();
        e.begin(Mode::Reduce, 20).unwrap();
        e.backend.fail = true;
        assert!(e.end().is_err());
        for _ in 0..4 {
            assert!(e.tick().is_err());
        }
        assert!(e.tick().is_ok());
        assert!(e.journal.exists());
        e.backend.fail = false;
        e.backend.values.get_mut("a").unwrap().volume = vec![0.3, 0.2];
        e.end().unwrap();
        assert_eq!(e.backend.values["a"].volume, vec![0.3, 0.2]);
    }
    #[test]
    fn failure_keeps_recovery_and_does_not_block_end_retry() {
        let mut e = engine();
        e.begin(Mode::Reduce, 20).unwrap();
        e.backend.fail = true;
        assert!(e.end().is_err());
        assert!(e.journal.exists());
        e.backend.fail = false;
        e.end().unwrap();
        assert!(!e.journal.exists());
    }
    #[test]
    fn restart_respects_manual_adjustment() {
        let mut e = engine();
        e.begin(Mode::Reduce, 20).unwrap();
        e.backend.values.get_mut("a").unwrap().volume = vec![0.5, 0.25];
        let mut restarted = Engine::new(e.backend, e.legacy_journal);
        restarted.recover().unwrap();
        assert_eq!(restarted.backend.values["a"].volume, vec![0.5, 0.25]);
        assert!(!restarted.journal.exists());
    }
    #[test]
    fn journal_failure_prevents_volume_change() {
        let mut e = engine();
        e.journal = e.journal.join("missing-parent.json");
        e.legacy_journal = e.journal.clone();
        assert!(e.begin(Mode::Reduce, 20).is_err());
        assert_eq!(e.backend.values["a"].volume, vec![0.8, 0.4]);
    }
    #[test]
    fn disconnected_device_keeps_recovery_until_available() {
        let mut e = engine();
        e.begin(Mode::Reduce, 20).unwrap();
        let disconnected = e.backend.values.remove("a").unwrap();
        assert!(e.end().is_err());
        assert!(e.journal.exists());
        e.backend.values.insert("a".into(), disconnected);
        e.recover().unwrap();
        assert_eq!(e.backend.values["a"].volume, vec![0.8, 0.4]);
    }
    #[test]
    fn manual_mute_is_preserved_while_our_volume_is_restored() {
        let mut e = engine();
        e.begin(Mode::Reduce, 20).unwrap();
        e.backend.values.get_mut("a").unwrap().muted = true;
        e.end().unwrap();
        assert!(e.backend.values["a"].muted);
        assert_eq!(e.backend.values["a"].volume, vec![0.8, 0.4]);
    }
    #[test]
    fn custom_percentage_is_used_and_restored() {
        for percent in [0, 35, 50, 100] {
            let mut e = engine();
            e.begin(Mode::Reduce, percent).unwrap();
            assert!((e.backend.values["a"].volume[0] - 0.8 * percent as f32 / 100.0).abs() < 0.002);
            e.end().unwrap();
            assert_eq!(e.backend.values["a"].volume, vec![0.8, 0.4]);
        }
    }
    #[test]
    fn invalid_percentage_never_changes_volume() {
        let mut e = engine();
        assert!(e.begin(Mode::Reduce, 101).is_err());
        assert_eq!(e.backend.values["a"].volume, vec![0.8, 0.4]);
    }
    #[test]
    fn disabled_never_changes_output() {
        let mut e = engine();
        e.begin(Mode::Off, 20).unwrap();
        assert_eq!(e.backend.values["a"].volume, vec![0.8, 0.4]);
        assert!(!e.journal.exists());
    }
}
