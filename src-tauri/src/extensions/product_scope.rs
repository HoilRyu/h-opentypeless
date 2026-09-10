//! H product scope; preserve upstream Ask implementation without exposing it.
pub fn apply(config: &mut crate::storage::AppConfig) {
    config.ask_hotkey.clear();
    config.hotkeys.ask = None;
    config.hotkeys.ask_bindings.clear();
}

pub fn require_ask() -> Result<(), String> {
    Err("H-OpenTypeless에서는 Ask Anything을 제공하지 않습니다.".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn removes_legacy_and_multiple_ask_bindings_only() {
        let mut config = crate::storage::AppConfig::default();
        let dictation = config.hotkeys.dictation.clone();
        super::apply(&mut config);
        assert!(config.ask_hotkey.is_empty());
        assert!(config.hotkeys.ask.is_none());
        assert!(config.hotkeys.ask_bindings.is_empty());
        assert_eq!(config.hotkeys.dictation, dictation);
        assert!(super::require_ask().is_err());
    }
}
