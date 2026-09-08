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
            && self
                .volume
                .iter()
                .zip(&other.volume)
                .all(|(a, b)| (a - b).abs() < 0.002)
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
    fn default_device(&mut self) -> Result<String>;
    fn read(&mut self, id: &str) -> Result<Device>;
    fn write(&mut self, id: &str, from: &Levels, to: &Levels) -> Result<()>;
}
#[derive(Clone, Serialize, Deserialize)]
struct Journal {
    id: String,
    before: Levels,
    applied: Levels,
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
    pending: Option<Journal>,
    mode: Mode,
    volume_percent: u8,
    seen: HashSet<String>,
}
impl<B: Backend> Engine<B> {
    pub fn new(backend: B, journal: PathBuf) -> Self {
        Self {
            backend,
            journal,
            pending: None,
            mode: Mode::Off,
            volume_percent: 20,
            seen: HashSet::new(),
        }
    }
    pub fn recover(&mut self) -> Result<()> {
        if self.pending.is_none() && self.journal.exists() {
            self.pending = Some(
                serde_json::from_slice(&fs::read(&self.journal).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?,
            );
        }
        self.restore()
    }
    fn restore(&mut self) -> Result<()> {
        if let Some(j) = self.pending.clone() {
            if !j.before.valid() || !j.applied.valid() {
                return Err("Invalid audio recovery record".into());
            }
            let current = self.backend.read(&j.id)?.levels;
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
                self.backend.write(&j.id, &current, &restored)?;
            }
            if self.journal.exists() {
                fs::remove_file(&self.journal).map_err(|e| e.to_string())?;
            }
            self.pending = None;
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
        self.recover()?;
        self.seen.clear();
        self.mode = mode;
        self.volume_percent = volume_percent;
        self.tick()
    }
    pub fn end(&mut self) -> Result<()> {
        self.mode = Mode::Off;
        self.restore()
    }
    pub fn tick(&mut self) -> Result<()> {
        if self.mode == Mode::Off {
            return Ok(());
        }
        let id = self.backend.default_device()?;
        if self.pending.as_ref().is_some_and(|j| j.id != id) {
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
            Mode::Mute if device.can_mute => applied.muted = true,
            Mode::Mute => applied.volume.fill(0.0),
        }
        if applied.matches(&device.levels) {
            return Ok(());
        }
        let j = Journal {
            id,
            before: device.levels,
            applied,
        };
        save_json(&self.journal, &j)?;
        self.pending = Some(j.clone());
        self.backend.write(&j.id, &j.before, &j.applied)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Fake {
        id: String,
        values: std::collections::HashMap<String, Levels>,
        fail: bool,
    }
    impl Backend for Fake {
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
    fn engine() -> Engine<Fake> {
        Engine::new(
            Fake {
                id: "a".into(),
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
        let mut restarted = Engine::new(e.backend, e.journal);
        restarted.recover().unwrap();
        assert_eq!(restarted.backend.values["a"].volume, vec![0.8, 0.4]);
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
        let mut restarted = Engine::new(e.backend, e.journal);
        restarted.recover().unwrap();
        assert_eq!(restarted.backend.values["a"].volume, vec![0.5, 0.25]);
        assert!(!restarted.journal.exists());
    }
    #[test]
    fn journal_failure_prevents_volume_change() {
        let mut e = engine();
        e.journal = e.journal.join("missing-parent.json");
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
