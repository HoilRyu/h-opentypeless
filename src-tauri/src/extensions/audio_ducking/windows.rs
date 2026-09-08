use super::engine::{Backend, Device, Levels, Result};
use windows::{
    core::PCWSTR,
    Win32::{
        Media::Audio::{
            eConsole, eRender, Endpoints::IAudioEndpointVolume, IMMDevice, IMMDeviceEnumerator,
            MMDeviceEnumerator,
        },
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
            COINIT_MULTITHREADED,
        },
    },
};
struct Com;
impl Com {
    fn init() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .map_err(|e| e.to_string())?;
        }
        Ok(Self)
    }
}
impl Drop for Com {
    fn drop(&mut self) {
        unsafe { CoUninitialize() }
    }
}
fn enumerator() -> Result<IMMDeviceEnumerator> {
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| e.to_string()) }
}
fn device(id: &str) -> Result<IMMDevice> {
    let wide: Vec<_> = id.encode_utf16().chain(Some(0)).collect();
    unsafe {
        enumerator()?
            .GetDevice(PCWSTR(wide.as_ptr()))
            .map_err(|e| e.to_string())
    }
}
fn endpoint(id: &str) -> Result<IAudioEndpointVolume> {
    unsafe {
        device(id)?
            .Activate(CLSCTX_ALL, None)
            .map_err(|e| e.to_string())
    }
}
pub struct Native;
impl Backend for Native {
    fn default_device(&mut self) -> Result<String> {
        let _com = Com::init()?;
        unsafe {
            let d = enumerator()?
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| e.to_string())?;
            let id = d.GetId().map_err(|e| e.to_string())?;
            let text = id.to_string().map_err(|e| e.to_string());
            CoTaskMemFree(Some(id.0.cast()));
            text
        }
    }
    fn read(&mut self, id: &str) -> Result<Device> {
        let _com = Com::init()?;
        let e = endpoint(id)?;
        unsafe {
            Ok(Device {
                levels: Levels {
                    volume: vec![e.GetMasterVolumeLevelScalar().map_err(|e| e.to_string())?],
                    muted: e.GetMute().map_err(|e| e.to_string())?.as_bool(),
                },
                can_mute: true,
            })
        }
    }
    fn write(&mut self, id: &str, from: &Levels, to: &Levels) -> Result<()> {
        let _com = Com::init()?;
        let e = endpoint(id)?;
        unsafe {
            if from.volume != to.volume {
                e.SetMasterVolumeLevelScalar(to.volume[0], std::ptr::null())
                    .map_err(|e| e.to_string())?;
            }
            if from.muted != to.muted {
                e.SetMute(to.muted, std::ptr::null())
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}
