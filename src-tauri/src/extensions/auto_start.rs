//! Register the main app bundle on modern macOS. Keep the existing plugin on other platforms.
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub fn is_enabled(app: &AppHandle) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    if let Some(status) = macos::status() {
        match status? {
            2 => {} // Awaiting approval is not enabled; still account for an older agent.
            status if registered(status)? => return Ok(true),
            _ => {}
        }
    }
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

pub fn enable(app: &AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    if let Some(result) = macos::set_enabled(true) {
        result?;
        // Registration must succeed before removing the older LaunchAgent.
        return app.autolaunch().disable().map_err(|e| e.to_string());
    }
    app.autolaunch().enable().map_err(|e| e.to_string())
}

pub fn disable(app: &AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    if let Some(result) = macos::set_enabled(false) {
        result?;
    }
    app.autolaunch().disable().map_err(|e| e.to_string())
}

/// Migrate an enabled legacy registration, never create one for a disabled preference.
pub fn repair(app: &AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    if app.autolaunch().is_enabled().map_err(|e| e.to_string())? {
        enable(app)?;
    }
    let _ = app;
    Ok(())
}

#[cfg(any(target_os = "macos", test))]
fn registered(status: isize) -> Result<bool, String> {
    match status {
        0 | 3 => Ok(false), // notRegistered / notFound
        1 => Ok(true),
        2 => Err("Allow H-OpenTypeless in System Settings → General → Login Items to enable launch at startup".into()),
        _ => Err("Unknown macOS login item status".into()),
    }
}

#[cfg(any(target_os = "macos", test))]
fn needs_change(status: isize, enabled: bool) -> Result<bool, String> {
    if enabled {
        registered(status).map(|current| !current)
    } else {
        match status {
            0 | 3 => Ok(false),
            1 | 2 => Ok(true),
            _ => Err("Unknown macOS login item status".into()),
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use objc2::{
        msg_send,
        rc::autoreleasepool,
        runtime::{AnyClass, AnyObject, Bool},
    };
    use std::{ffi::CStr, ptr};

    #[link(name = "ServiceManagement", kind = "framework")]
    extern "C" {}

    fn with_service<T>(
        operation: impl FnOnce(*mut AnyObject) -> Result<T, String>,
    ) -> Option<Result<T, String>> {
        // SMAppService is available on macOS 13+. A development executable is not an app bundle.
        let executable = std::env::current_exe().ok()?;
        if !executable
            .ancestors()
            .any(|p| p.extension().is_some_and(|e| e == "app"))
        {
            return None;
        }
        let class = AnyClass::get("SMAppService")?;
        Some(autoreleasepool(|_| {
            // The returned service is borrowed only for this autorelease pool's lifetime.
            let service: *mut AnyObject = unsafe { msg_send![class, mainAppService] };
            if service.is_null() {
                return Err("macOS main app login service unavailable".into());
            }
            operation(service)
        }))
    }

    pub fn status() -> Option<Result<isize, String>> {
        with_service(|service| Ok(unsafe { msg_send![service, status] }))
    }

    pub fn set_enabled(enabled: bool) -> Option<Result<(), String>> {
        with_service(|service| {
            let status: isize = unsafe { msg_send![service, status] };
            if !needs_change(status, enabled)? {
                return Ok(());
            }
            let mut error: *mut AnyObject = ptr::null_mut();
            let success: Bool = unsafe {
                if enabled {
                    msg_send![service, registerAndReturnError: &mut error]
                } else {
                    msg_send![service, unregisterAndReturnError: &mut error]
                }
            };
            if !success.as_bool() {
                return Err(error_message(error));
            }
            // ServiceManagement publishes status changes asynchronously. A successful
            // unregister can still read as Enabled here; treating that as failure leaves
            // the saved preference out of sync and makes the user save twice.
            if enabled {
                let status: isize = unsafe { msg_send![service, status] };
                registered(status)?; // Surface explicit approval requirements, not propagation delay.
            }
            Ok(())
        })
    }

    fn error_message(error: *mut AnyObject) -> String {
        if !error.is_null() {
            unsafe {
                let description: *mut AnyObject = msg_send![error, localizedDescription];
                if !description.is_null() {
                    let text: *const std::ffi::c_char = msg_send![description, UTF8String];
                    if !text.is_null() {
                        return CStr::from_ptr(text).to_string_lossy().into_owned();
                    }
                }
            }
        }
        "macOS login item registration failed".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_registration_does_not_register_twice() {
        assert!(!needs_change(1, true).unwrap());
        assert!(!needs_change(0, false).unwrap());
        assert!(!needs_change(3, false).unwrap());
        assert!(needs_change(0, true).unwrap());
        assert!(needs_change(1, false).unwrap());
    }

    #[test]
    fn approval_required_is_not_reported_as_enabled_or_overridden() {
        assert!(registered(2).is_err());
        assert!(needs_change(2, true).is_err());
        assert!(needs_change(2, false).unwrap());
        assert!(registered(99).is_err());
    }
}
