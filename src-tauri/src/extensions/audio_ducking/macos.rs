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
    Ok(v)
}
fn write<T>(o: u32, a: &Address, v: &T) -> Result<()> {
    unsafe {
        check(AudioObjectSetPropertyData(
            o,
            a,
            0,
            ptr::null(),
            std::mem::size_of::<T>() as u32,
            v as *const _ as *const c_void,
        ))
    }
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
    if size > 65536 {
        return Err("Invalid audio device list".into());
    }
    let mut ids = vec![0u32; size as usize / 4];
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
    fn default_device(&mut self) -> Result<String> {
        uid(read(1, &addr(b"dOut", false, 0))?)
    }
    fn read(&mut self, id: &str) -> Result<Device> {
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
        let o = resolve(id)?;
        let channels = channels(o)?;
        if channels.len() != to.volume.len() || channels.len() != from.volume.len() {
            return Err("Output channel layout changed".into());
        }
        for (i, e) in channels.iter().enumerate() {
            if from.volume[i] != to.volume[i] {
                if let Err(error) = write(o, &addr(b"volm", true, *e), &to.volume[i]) {
                    for (j, prior) in channels.iter().enumerate().take(i) {
                        let _ = write(o, &addr(b"volm", true, *prior), &from.volume[j]);
                    }
                    return Err(error);
                }
            }
        }
        if from.muted != to.muted {
            write(o, &addr(b"mute", true, 0), &u32::from(to.muted))?;
        }
        Ok(())
    }
}
