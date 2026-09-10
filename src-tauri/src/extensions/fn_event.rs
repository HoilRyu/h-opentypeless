//! Software-generated Fn transitions can accompany remapped navigation keys.
//! Only the native Fn base edge is filtered; other shortcuts are unaffected.
pub fn is_synthetic_fn(keycode: i64, source_pid: i64) -> bool {
    keycode == 63 && source_pid > 0
}

#[cfg(test)]
mod tests {
    #[test]
    fn remapped_arrows_cannot_generate_a_recording_fn_edge() {
        for arrow in [123, 124, 125, 126] {
            // Observed BTT sequence: arrow down/up, synthetic Fn down/up.
            assert!(!super::is_synthetic_fn(arrow, 66467));
            assert!(super::is_synthetic_fn(63, 66467));
        }
        assert!(!super::is_synthetic_fn(63, 0));
        assert!(!super::is_synthetic_fn(49, 66467));
    }
}
