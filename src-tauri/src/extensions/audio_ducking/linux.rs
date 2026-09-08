// pactl talks to PulseAudio and to PipeWire's PulseAudio compatibility server.
use super::engine::{Backend, Device, Levels, Result};
use std::{
    io::Read,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
fn pactl(args: &[&str]) -> Result<String> {
    let mut child = Command::new("pactl")
        .args(args)
        .env("LC_ALL", "C")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("pactl unavailable: {e}"))?;
    let stdout = child.stdout.take().ok_or("pactl stdout unavailable")?;
    let reader = std::thread::spawn(move || {
        let mut text = String::new();
        stdout
            .take(1024 * 1024)
            .read_to_string(&mut text)
            .map(|_| text)
    });
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => {
                return if status.success() {
                    reader
                        .join()
                        .map_err(|_| "pactl reader failed")?
                        .map_err(|e| e.to_string())
                } else {
                    Err("pactl command failed".into())
                }
            }
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("pactl timed out".into());
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}
pub struct Native;
impl Backend for Native {
    fn default_device(&mut self) -> Result<String> {
        Ok(pactl(&["get-default-sink"])?.trim().to_string())
    }
    fn read(&mut self, id: &str) -> Result<Device> {
        let data: serde_json::Value =
            serde_json::from_str(&pactl(&["--format=json", "list", "sinks"])?)
                .map_err(|e| e.to_string())?;
        let sink = data
            .as_array()
            .ok_or("Invalid sink list")?
            .iter()
            .find(|v| v["name"].as_str() == Some(id))
            .ok_or("Output device disconnected")?;
        // Preserve channel order from channel_map, not alphabetically sorted JSON keys.
        let map = sink["channel_map"].as_str().ok_or("Missing channel map")?;
        let volume = map
            .split(',')
            .map(|name| {
                sink["volume"][name.trim()]["value"]
                    .as_u64()
                    .map(|v| v as f32 / 65536.0)
                    .ok_or_else(|| "Missing channel volume".into())
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Device {
            levels: Levels {
                volume,
                muted: sink["mute"].as_bool().ok_or("Missing mute state")?,
            },
            can_mute: true,
        })
    }
    fn write(&mut self, id: &str, from: &Levels, to: &Levels) -> Result<()> {
        if from.volume != to.volume {
            let values: Vec<_> = to
                .volume
                .iter()
                .map(|v| format!("{:.0}", v * 65536.0))
                .collect();
            let mut args = vec!["set-sink-volume", id];
            args.extend(values.iter().map(String::as_str));
            pactl(&args)?;
        }
        if from.muted != to.muted {
            pactl(&["set-sink-mute", id, if to.muted { "1" } else { "0" }])?;
        }
        Ok(())
    }
}
