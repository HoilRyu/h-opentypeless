//! Filter incidental software Fn edges while supporting intentional Stream Deck keys.
//! This is an input compatibility filter, not a security boundary.
pub fn should_ignore_fn(keycode: i64, source_pid: i64) -> bool {
    if keycode != 63 || source_pid <= 0 {
        return false;
    }
    ignore_source(
        keycode,
        source_pid,
        source_executable(source_pid).as_deref(),
    )
}

fn ignore_source(keycode: i64, source_pid: i64, executable: Option<&str>) -> bool {
    keycode == 63
        && source_pid > 0
        && !executable.is_some_and(|path| {
            path.ends_with("/Elgato Stream Deck.app/Contents/MacOS/Stream Deck")
        })
}

#[cfg(target_os = "macos")]
fn source_executable(source_pid: i64) -> Option<String> {
    use std::ffi::{c_int, c_void, CStr};
    #[link(name = "proc")]
    extern "C" {
        fn proc_pidpath(pid: c_int, buffer: *mut c_void, buffersize: u32) -> c_int;
    }
    let pid = i32::try_from(source_pid).ok()?;
    // PROC_PIDPATHINFO_MAXSIZE. Resolve on each Fn edge, avoiding stale PID caches.
    let mut buffer = [0u8; 4096];
    let len = unsafe { proc_pidpath(pid, buffer.as_mut_ptr().cast(), buffer.len() as u32) };
    if len <= 0 {
        return None;
    }
    CStr::from_bytes_until_nul(&buffer)
        .ok()?
        .to_str()
        .ok()
        .map(str::to_owned)
}

#[cfg(not(target_os = "macos"))]
fn source_executable(_source_pid: i64) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::ignore_source;
    const DECK: &str = "/Applications/Elgato Stream Deck.app/Contents/MacOS/Stream Deck";
    const BTT: &str = "/Applications/BetterTouchTool.app/Contents/MacOS/BetterTouchTool";

    #[test]
    fn stream_deck_fn_survives_process_restarts_and_both_edges() {
        // Both press and release go through this same source policy.
        for pid in [1012, 56789] {
            for _edge in [true, false] {
                assert!(!ignore_source(63, pid, Some(DECK)));
            }
        }
        assert!(!ignore_source(
            63,
            1012,
            Some("/Users/example/Applications/Elgato Stream Deck.app/Contents/MacOS/Stream Deck")
        ));
    }

    #[test]
    fn btt_incidental_fn_is_still_ignored() {
        for arrow in [123, 124, 125, 126] {
            assert!(!ignore_source(arrow, 225, Some(BTT)));
        }
        assert!(ignore_source(63, 225, Some(BTT)));
    }

    #[test]
    fn hardware_fn_is_accepted_without_process_lookup() {
        assert!(!super::should_ignore_fn(63, 0));
        assert!(!super::should_ignore_fn(49, 1012));
    }

    #[test]
    fn unknown_or_unresolvable_software_remains_filtered() {
        for path in [
            None,
            Some("/tmp/Stream Deck"),
            Some("/tmp/Elgato Stream Deck.app/Contents/MacOS/Stream Deck Helper"),
        ] {
            assert!(ignore_source(63, 1012, path));
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn resolves_live_process_and_handles_missing_process() {
        let executable = super::source_executable(i64::from(std::process::id())).unwrap();
        assert_eq!(
            std::path::Path::new(&executable).canonicalize().unwrap(),
            std::env::current_exe().unwrap().canonicalize().unwrap()
        );
        assert!(super::source_executable(i64::MAX).is_none());
    }
}
