//! AppKit geometry stays in desktop points, including mixed-DPI/negative-origin displays.
use objc2::{class, msg_send, runtime::AnyObject, Encode, Encoding};
use std::{ffi::c_void, ptr};
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}
unsafe impl Encode for Point {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[Encoding::Double, Encoding::Double]);
}
unsafe impl Encode for Size {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[Encoding::Double, Encoding::Double]);
}
unsafe impl Encode for Rect {
    const ENCODING: Encoding = Encoding::Struct("CGRect", &[Point::ENCODING, Size::ENCODING]);
}
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> u8;
    fn AXUIElementCreateSystemWide() -> *const c_void;
    fn AXUIElementSetMessagingTimeout(e: *const c_void, timeout: f32) -> i32;
    fn AXUIElementCopyAttributeValue(
        e: *const c_void,
        n: *const c_void,
        v: *mut *const c_void,
    ) -> i32;
    fn AXValueGetValue(v: *const c_void, t: u32, out: *mut c_void) -> bool;
}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(v: *const c_void);
    fn CFEqual(a: *const c_void, b: *const c_void) -> u8;
    fn CFArrayGetCount(array: *const c_void) -> isize;
    fn CFArrayGetValueAtIndex(array: *const c_void, index: isize) -> *mut AnyObject;
    fn CFStringCreateWithCString(a: *const c_void, s: *const i8, e: u32) -> *const c_void;
}
struct Owned(*const c_void);
impl Drop for Owned {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0) }
        }
    }
}
unsafe fn string(s: &str) -> Owned {
    let s = std::ffi::CString::new(s).unwrap();
    Owned(CFStringCreateWithCString(
        ptr::null(),
        s.as_ptr(),
        0x08000100,
    ))
}
unsafe fn attribute(element: &Owned, name: &str) -> Option<Owned> {
    let mut raw = ptr::null();
    let status = AXUIElementCopyAttributeValue(element.0, string(name).0, &mut raw);
    let value = Owned(raw);
    (status == 0 && !raw.is_null()).then_some(value)
}

// Read only geometry and roles, never the contents of the input field.
// Some editors focus a child inside the actual text control; bound the parent walk.
pub fn input_area() -> Option<Rect> {
    unsafe {
        if AXIsProcessTrusted() == 0 {
            return None;
        }
        let root = Owned(AXUIElementCreateSystemWide());
        AXUIElementSetMessagingTimeout(root.0, 0.12);
        let mut element = attribute(&root, "AXFocusedUIElement")?;
        for _ in 0..4 {
            AXUIElementSetMessagingTimeout(element.0, 0.12);
            let role = attribute(&element, "AXRole")?;
            let text_control = ["AXTextArea", "AXTextField", "AXComboBox", "AXSearchField"]
                .iter()
                .any(|name| CFEqual(role.0, string(name).0) != 0);
            if text_control {
                let position = attribute(&element, "AXPosition")?;
                let size = attribute(&element, "AXSize")?;
                let mut rect = Rect::default();
                if AXValueGetValue(position.0, 1, &mut rect.origin as *mut _ as *mut c_void)
                    && AXValueGetValue(size.0, 2, &mut rect.size as *mut _ as *mut c_void)
                    && valid_input_area(rect)
                {
                    return Some(rect);
                }
                return None;
            }
            element = attribute(&element, "AXParent")?;
        }
        None
    }
}
fn valid_input_area(r: Rect) -> bool {
    [r.origin.x, r.origin.y, r.size.width, r.size.height]
        .iter()
        .all(|v| v.is_finite())
        && r.size.width > 0.
        && r.size.height > 0.
}
// Main-thread only. Convert AX top-left coordinates using the primary screen's top edge.
pub fn anchor(input: Option<Rect>) -> (Rect, Rect, Rect) {
    unsafe {
        let screens: *mut AnyObject = msg_send![class!(NSScreen), screens];
        let primary: *mut AnyObject = CFArrayGetValueAtIndex(screens.cast(), 0);
        let first: Rect = msg_send![primary, frame];
        let target = input
            .map(|r| Rect {
                origin: Point {
                    x: r.origin.x,
                    y: first.origin.y + first.size.height - r.origin.y - r.size.height,
                },
                size: r.size,
            })
            .unwrap_or(Rect {
                origin: Point::default(),
                size: Size::default(),
            });
        let count = CFArrayGetCount(screens.cast());
        let main: *mut AnyObject = msg_send![class!(NSScreen), mainScreen];
        let mut chosen = if main.is_null() { primary } else { main };
        for i in 0..count {
            let screen: *mut AnyObject = CFArrayGetValueAtIndex(screens.cast(), i);
            let r: Rect = msg_send![screen, frame];
            if input.is_some() && contains(r, target.origin) {
                chosen = screen;
                break;
            }
        }
        let frame: Rect = msg_send![chosen, frame];
        let work: Rect = msg_send![chosen, visibleFrame];
        (target, frame, work)
    }
}
fn contains(r: Rect, p: Point) -> bool {
    p.x >= r.origin.x
        && p.y >= r.origin.y
        && p.x < r.origin.x + r.size.width
        && p.y < r.origin.y + r.size.height
}
pub fn place(anchor: Rect, work: Rect, width: f64, height: f64, near: bool) -> Rect {
    let margin = 12.0;
    let near = near && valid_input_area(anchor);
    let x = if near {
        anchor.origin.x + (anchor.size.width - width) / 2.0
    } else {
        work.origin.x + (work.size.width - width) / 2.0
    };
    let below = anchor.origin.y - height - 12.0;
    let y = if !near {
        work.origin.y + 60.0
    } else if below >= work.origin.y + margin {
        below
    } else {
        anchor.origin.y + anchor.size.height + 12.0
    };
    Rect {
        origin: Point {
            x: x.clamp(
                work.origin.x + margin,
                (work.origin.x + work.size.width - width - margin).max(work.origin.x + margin),
            ),
            y: y.clamp(
                work.origin.y + margin,
                (work.origin.y + work.size.height - height - margin).max(work.origin.y + margin),
            ),
        },
        size: Size { width, height },
    }
}
pub fn frame(window: &tauri::WebviewWindow, r: Rect, overlay: bool) {
    unsafe {
        if let Ok(raw) = window.ns_window() {
            let w = raw as *mut AnyObject;
            let _: () = msg_send![w,setFrame:r display:true];
            let _: () = msg_send![w,setCollectionBehavior: ((1usize<<0)|(1usize<<8)|(1usize<<4))];
            if overlay {
                let _: () = msg_send![w,setIgnoresMouseEvents:true];
            }
        }
    }
}
// Re-resolve only when display configuration changes; ordinary pointer movement is ignored.
pub fn reconcile(saved: (Rect, Rect, Rect)) -> (Rect, Rect, Rect) {
    unsafe {
        let screens: *mut AnyObject = msg_send![class!(NSScreen), screens];
        let count = CFArrayGetCount(screens.cast());
        for i in 0..count {
            let screen: *mut AnyObject = CFArrayGetValueAtIndex(screens.cast(), i);
            let frame: Rect = msg_send![screen, frame];
            if frame.origin.x == saved.1.origin.x
                && frame.origin.y == saved.1.origin.y
                && frame.size.width == saved.1.size.width
                && frame.size.height == saved.1.size.height
            {
                let work: Rect = msg_send![screen, visibleFrame];
                return (saved.0, frame, work);
            }
        }
        anchor(None)
    }
}
pub fn current_frame(w: &tauri::WebviewWindow) -> Option<Rect> {
    unsafe {
        let raw = w.ns_window().ok()?;
        let w = raw as *mut AnyObject;
        Some(msg_send![w, frame])
    }
}
// Notification centers own these observers for the application lifetime. No polling/timers.
pub fn observe(app: &tauri::AppHandle) -> Vec<(usize, usize)> {
    unsafe {
        let mut observers = Vec::new();
        let workspace: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
        let workspace_center: *mut AnyObject = msg_send![workspace, notificationCenter];
        let center: *mut AnyObject = msg_send![class!(NSNotificationCenter), defaultCenter];
        let distributed: *mut AnyObject =
            msg_send![class!(NSDistributedNotificationCenter), defaultCenter];
        for (center, name, suspend) in [
            (
                workspace_center,
                "NSWorkspaceWillSleepNotification",
                Some(true),
            ),
            (
                workspace_center,
                "NSWorkspaceDidWakeNotification",
                Some(false),
            ),
            (
                workspace_center,
                "NSWorkspaceSessionDidResignActiveNotification",
                Some(true),
            ),
            (
                workspace_center,
                "NSWorkspaceSessionDidBecomeActiveNotification",
                Some(false),
            ),
            (distributed, "com.apple.screenIsLocked", Some(true)),
            (distributed, "com.apple.screenIsUnlocked", Some(false)),
            (
                center,
                "NSApplicationDidChangeScreenParametersNotification",
                None,
            ),
        ] {
            let handle = app.clone();
            let callback = block2::RcBlock::new(move |_notification: *mut AnyObject| {
                super::environment_changed(&handle, suspend);
            });
            let token: *mut AnyObject = msg_send![center,addObserverForName:string(name).0 as *mut AnyObject object:ptr::null::<AnyObject>() queue:ptr::null::<AnyObject>() usingBlock:&*callback];
            observers.push((center as usize, token as usize));
        }
        observers
    }
}

pub fn remove_observers(observers: Vec<(usize, usize)>) {
    unsafe {
        for (center, token) in observers {
            let center = center as *mut AnyObject;
            let _: () = msg_send![center,removeObserver:token as *mut AnyObject];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn negative_screen_and_bottom_input_stay_visible() {
        let w = Rect {
            origin: Point {
                x: -1600.,
                y: -400.,
            },
            size: Size {
                width: 1600.,
                height: 1000.,
            },
        };
        let a = Rect {
            origin: Point { x: -5., y: -395. },
            size: Size {
                width: 400.,
                height: 20.,
            },
        };
        let r = place(a, w, 260., 68., true);
        assert!(r.origin.x + r.size.width <= -12.);
        assert!(r.origin.y > a.origin.y);
    }
    #[test]
    fn below_input_has_gap() {
        let w = Rect {
            origin: Point { x: 0., y: 0. },
            size: Size {
                width: 1440.,
                height: 900.,
            },
        };
        let a = Rect {
            origin: Point { x: 400., y: 500. },
            size: Size {
                width: 400.,
                height: 20.,
            },
        };
        let r = place(a, w, 260., 68., true);
        assert_eq!(r.origin.y + r.size.height, 488.);
        assert_eq!(r.origin.x, 470.);
    }

    #[test]
    fn unavailable_input_uses_screen_bottom_not_pointer_geometry() {
        let work = Rect {
            origin: Point { x: -1440., y: 0. },
            size: Size {
                width: 1440.,
                height: 900.,
            },
        };
        let result = place(Rect::default(), work, 260., 68., true);
        assert_eq!(result.origin.x, -850.);
        assert_eq!(result.origin.y, 60.);
        assert!(!valid_input_area(Rect {
            origin: Point { x: f64::NAN, y: 0. },
            size: Size {
                width: 400.,
                height: 100.
            },
        }));
    }
}
