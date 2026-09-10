use super::engine::{Backend, Device, Levels, Result};
use std::{ffi::c_void, ptr};
#[repr(C)]
struct Address {
    selector: u32,
    scope: u32,
    element: u32,
}
const fn four(s: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*s)
}
fn addr(s: &[u8; 4], output: bool, e: u32) -> Address {
    Address {
        selector: four(s),
        scope: four(if output { b"outp" } else { b"glob" }),
        element: e,
    }
}
#[link(name = "CoreAudio", kind = "framework")]
extern "C" {
    fn AudioObjectGetPropertyData(
        o: u32,
        a: *const Address,
        q: u32,
        qp: *const c_void,
        size: *mut u32,
        data: *mut c_void,
    ) -> i32;
    fn AudioObjectGetPropertyDataSize(
        o: u32,
        a: *const Address,
        q: u32,
        qp: *const c_void,
        size: *mut u32,
    ) -> i32;
    fn AudioObjectSetPropertyData(
        o: u32,
        a: *const Address,
        q: u32,
        qp: *const c_void,
        size: u32,
        data: *const c_void,
    ) -> i32;
    fn AudioObjectIsPropertySettable(o: u32, a: *const Address, settable: *mut u8) -> i32;
}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFStringGetCString(s: *const c_void, out: *mut u8, size: isize, encoding: u32) -> bool;
    fn CFRelease(s: *const c_void);
}
fn check(status: i32) -> Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(format!("Core Audio error {status}"))
    }
}
fn read<T: Default>(o: u32, a: &Address) -> Result<T> {
    let mut v = T::default();
    let mut n = std::mem::size_of::<T>() as u32;
    unsafe {
        check(AudioObjectGetPropertyData(
            o,
            a,
            0,
            ptr::null(),
            &mut n,
            &mut v as *mut _ as *mut c_void,
        ))?;
    }
    if n as usize != std::mem::size_of::<T>() {
        return Err("Invalid audio property size".into());
    }
    Ok(v)
}
trait PropertyValue: Copy + Default + PartialEq {
    fn valid(self) -> bool;
    fn matches(self, desired: Self) -> bool;
}
impl PropertyValue for f32 {
    fn valid(self) -> bool {
        self.is_finite() && (0.0..=1.0).contains(&self)
    }
    fn matches(self, desired: Self) -> bool {
        self.is_finite()
            && if desired == 0.0 {
                self == 0.0
            } else {
                (self - desired).abs() < 0.002
            }
    }
}
impl PropertyValue for u32 {
    fn valid(self) -> bool {
        self <= 1
    }
    fn matches(self, desired: Self) -> bool {
        self == desired
    }
}
static WRITE_FAILED: std::sync::Mutex<Vec<u32>> = std::sync::Mutex::new(Vec::new());

// HAL reports the hardware's supported step, which can differ substantially
// from the requested scalar on USB devices. Keep that observed value for recovery.
fn completed_value<T: PropertyValue>(observed: T, previous: T, notified: bool) -> Option<T> {
    // HAL may notify before readback stops returning its cached previous value.
    // Recording that value as "applied" would lose ownership of the later mute.
    (observed.valid() && notified && observed != previous).then_some(observed)
}
fn write<T: PropertyValue>(o: u32, a: &Address, v: &T) -> Result<T> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;
    static NOTIFICATIONS: AtomicU64 = AtomicU64::new(0);
    unsafe extern "C" fn changed(_: u32, _: u32, _: *const Address, _: *mut c_void) -> i32 {
        NOTIFICATIONS.fetch_add(1, Ordering::SeqCst);
        0
    }
    type Listener = unsafe extern "C" fn(u32, u32, *const Address, *mut c_void) -> i32;
    extern "C" {
        fn AudioObjectAddPropertyListener(
            o: u32,
            a: *const Address,
            listener: Listener,
            context: *mut c_void,
        ) -> i32;
        fn AudioObjectRemovePropertyListener(
            o: u32,
            a: *const Address,
            listener: Listener,
            context: *mut c_void,
        ) -> i32;
    }
    if WRITE_FAILED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .contains(&o)
    {
        return Err("Output volume control suspended; restart H to retry".into());
    }
    let current = read::<T>(o, a)?;
    // A no-op need not emit a HAL notification.
    if current.valid() && current.matches(*v) {
        return Ok(current);
    }
    // Callback uses static state, so a delayed notification cannot access freed memory.
    unsafe {
        check(AudioObjectAddPropertyListener(
            o,
            a,
            changed,
            ptr::null_mut(),
        ))?;
    }
    let before = NOTIFICATIONS.load(Ordering::SeqCst);
    crate::audio::lifecycle::event("volume_change_requested");
    let mut observed = None;
    let result = (|| {
        unsafe {
            check(AudioObjectSetPropertyData(
                o,
                a,
                0,
                ptr::null(),
                std::mem::size_of::<T>() as u32,
                v as *const _ as *const c_void,
            ))?;
        }
        crate::audio::lifecycle::wait_confirmation(
            || {
                observed = completed_value(
                    read::<T>(o, a)?,
                    current,
                    NOTIFICATIONS.load(Ordering::SeqCst) != before,
                );
                Ok(observed.is_some())
            },
            Duration::from_secs(1),
        )
    })();
    let removed = unsafe {
        check(AudioObjectRemovePropertyListener(
            o,
            a,
            changed,
            ptr::null_mut(),
        ))
    };
    if result.is_err() || removed.is_err() {
        // A returned volume API call with an unconfirmed value disables only
        // further volume writes. Capture has its own teardown/transition guard.
        let mut failed = WRITE_FAILED.lock().unwrap_or_else(|e| e.into_inner());
        if !failed.contains(&o) {
            failed.push(o);
        }
    }
    let result = result.and(removed);
    crate::audio::lifecycle::event(if result.is_ok() {
        "volume_change_confirmed"
    } else {
        "volume_change_failed"
    });
    result?;
    observed.ok_or_else(|| "Missing confirmed output value".into())
}

fn writable(o: u32, a: &Address) -> bool {
    let mut yes = 0;
    unsafe { AudioObjectIsPropertySettable(o, a, &mut yes) == 0 && yes != 0 }
}
fn uid(o: u32) -> Result<String> {
    let raw: usize = read(o, &addr(b"uid ", false, 0))?;
    if raw == 0 {
        return Err("Missing output device UID".into());
    }
    let mut buf = vec![0u8; 4096];
    let ok = unsafe {
        let ok = CFStringGetCString(
            raw as *const c_void,
            buf.as_mut_ptr(),
            buf.len() as isize,
            0x08000100,
        );
        CFRelease(raw as *const c_void);
        ok
    };
    if !ok {
        return Err("Invalid output device UID".into());
    }
    let n = buf.iter().position(|v| *v == 0).unwrap_or(buf.len());
    String::from_utf8(buf[..n].to_vec()).map_err(|e| e.to_string())
}
fn resolve(id: &str) -> Result<u32> {
    let a = addr(b"dev#", false, 0);
    let mut size = 0;
    unsafe {
        check(AudioObjectGetPropertyDataSize(
            1,
            &a,
            0,
            ptr::null(),
            &mut size,
        ))?;
    }
    if size > 65536 || size % 4 != 0 {
        return Err("Invalid audio device list".into());
    }
    let mut ids = vec![0u32; size as usize / 4];
    let capacity_bytes = size;
    unsafe {
        check(AudioObjectGetPropertyData(
            1,
            &a,
            0,
            ptr::null(),
            &mut size,
            ids.as_mut_ptr().cast(),
        ))?;
    }
    if size > capacity_bytes || size % 4 != 0 {
        return Err("Invalid returned audio device list size".into());
    }
    ids.truncate(size as usize / 4);
    ids.into_iter()
        .find(|o| uid(*o).as_deref() == Ok(id))
        .ok_or("Output device disconnected".into())
}
fn channels(o: u32) -> Result<Vec<u32>> {
    if writable(o, &addr(b"volm", true, 0)) {
        return Ok(vec![0]);
    }
    let channels: Vec<_> = (1..=32)
        .filter(|e| writable(o, &addr(b"volm", true, *e)))
        .collect();
    if channels.is_empty() {
        Err("This output device does not support software volume control".into())
    } else {
        Ok(channels)
    }
}
pub struct Native;
impl Backend for Native {
    fn prefer_volume_for_mute(&self) -> bool {
        // EDIFIER USB output remained inaudible after HAL confirmed unmute,
        // then recovered on another recording cycle. Avoid that switch for
        // new sessions; retain its read/write support for legacy recovery.
        // This is a mitigation, not proof of a driver or firmware root cause.
        true
    }
    fn default_device(&mut self) -> Result<String> {
        crate::audio::lifecycle::check()?;
        uid(read(1, &addr(b"dOut", false, 0))?)
    }
    fn read(&mut self, id: &str) -> Result<Device> {
        crate::audio::lifecycle::check()?;
        let o = resolve(id)?;
        let volume = channels(o)?
            .iter()
            .map(|e| read::<f32>(o, &addr(b"volm", true, *e)))
            .collect::<Result<Vec<_>>>()?;
        let can_mute = writable(o, &addr(b"mute", true, 0));
        let muted = if can_mute {
            read::<u32>(o, &addr(b"mute", true, 0))? != 0
        } else {
            false
        };
        Ok(Device {
            levels: Levels { volume, muted },
            can_mute,
        })
    }
    fn write(&mut self, id: &str, from: &Levels, to: &Levels) -> Result<()> {
        self.write_observed(id, from, to).map(|_| ())
    }
    fn write_observed(&mut self, id: &str, from: &Levels, to: &Levels) -> Result<Levels> {
        let _transition = crate::audio::lifecycle::enter()?;
        let o = resolve(id)?;
        let channels = channels(o)?;
        if channels.len() != to.volume.len() || channels.len() != from.volume.len() {
            return Err("Output channel layout changed".into());
        }
        let mut applied = to.clone();
        for (i, e) in channels.iter().enumerate() {
            if from.volume[i] != to.volume[i] {
                // Do not issue rollback writes after uncertain completion.
                applied.volume[i] = write(o, &addr(b"volm", true, *e), &to.volume[i])?;
            }
        }
        if from.muted != to.muted {
            applied.muted = write(o, &addr(b"mute", true, 0), &u32::from(to.muted))? != 0;
        }
        Ok(applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_mac_mute_sessions_use_volume_instead_of_hardware_mute() {
        assert!(Native.prefer_volume_for_mute());
    }
    #[test]
    fn usb_quantized_value_completes_after_notification() {
        assert_eq!(completed_value(0.12543403f32, 0.4, true), Some(0.12543403));
        assert_eq!(completed_value(0.34f32, 0.8, false), None);
    }
    #[test]
    fn notification_with_old_mute_or_volume_is_not_completion() {
        assert_eq!(completed_value(0u32, 0, true), None);
        assert_eq!(completed_value(1u32, 0, true), Some(1));
        assert_eq!(completed_value(0.8f32, 0.8, true), None);
    }
    #[test]
    fn small_confirmed_changes_and_exact_silence_are_not_noops() {
        assert_eq!(completed_value(0.000157f32, 0.001, true), Some(0.000157));
        assert!(!0.001f32.matches(0.0));
        assert!(0.0f32.matches(0.0));
    }
    #[test]
    fn cached_readback_alone_does_not_confirm_a_write() {
        assert_eq!(completed_value(0.125f32, 0.8, false), None);
        assert_eq!(completed_value(f32::NAN, 0.8, true), None);
        assert_eq!(completed_value(0u32, 1, false), None);
    }
}
