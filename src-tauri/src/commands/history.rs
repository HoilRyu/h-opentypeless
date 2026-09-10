use crate::storage;

#[tauri::command]
pub async fn get_history(
    state: tauri::State<'_, storage::HistoryStore>,
    limit: u32,
    offset: u32,
) -> Result<Vec<storage::HistoryEntry>, String> {
    state.list(limit, offset).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_history(state: tauri::State<'_, storage::HistoryStore>) -> Result<(), String> {
    state.clear().await.map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct RepolishResult {
    pub history_id: i64,
    pub polished_text: String,
    pub model: String,
    pub style: String,
    pub prompt_sha256: String,
    pub elapsed_ms: u64,
}

fn replay_request(
    raw_text: String,
    style: String,
    dictionary: Vec<String>,
    correction_rules: Vec<crate::llm::CorrectionRule>,
    custom_prompt: String,
) -> Result<crate::llm::PolishRequest, String> {
    use crate::voice_intent::{VoiceIntent, VoiceIntentKind, VoiceOutputPlacement};
    if !matches!(
        style.as_str(),
        "minimal" | "clean" | "structured" | "professional"
    ) {
        return Err("history.repolishInvalidStyle".into());
    }
    if raw_text.trim().is_empty() || raw_text.chars().count() > 20000 {
        return Err("history.repolishInvalidSource".into());
    }
    Ok(crate::llm::PolishRequest {
        raw_text,
        context: crate::app_detector::types::ContextProfile::general_native().summary(),
        dictionary,
        correction_rules,
        polish_style: style,
        polish_custom_prompt: custom_prompt,
        mapped_scene_prompt: String::new(),
        active_scene_prompt: String::new(),
        translate_enabled: false,
        target_lang: String::new(),
        selected_text: None,
        operation_id: None,
        voice_intent: VoiceIntent::from_parts(
            VoiceIntentKind::DictateInsert,
            VoiceOutputPlacement::InsertAtCursor,
            1.0,
            None,
            None,
            None,
            None,
        )
        .map_err(|_| "history.repolishFailed".to_string())?,
    })
}

/// Preview only: no focus restoration, keyboard output, config or history mutation.
#[tauri::command]
pub async fn repolish_history(
    app: tauri::AppHandle,
    id: i64,
    style: String,
) -> Result<RepolishResult, String> {
    use crate::llm::{self, prompt::ContextPromptOptions};
    use sha2::{Digest, Sha256};
    use tauri::Manager;
    static REPLAY: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);
    let _permit = REPLAY
        .try_acquire()
        .map_err(|_| "history.repolishBusy".to_string())?;
    let raw = app
        .state::<storage::HistoryStore>()
        .raw_text(id)
        .await
        .map_err(|_| "history.repolishMissing".to_string())?;
    let config = app
        .state::<storage::ConfigManager>()
        .load()
        .await
        .map_err(|_| "history.repolishFailed".to_string())?;
    if config.llm_provider.trim().is_empty() || config.llm_provider == "cloud" {
        return Err("history.repolishConfigure".into());
    }
    let dictionary = app.state::<storage::DictionaryStore>();
    let rules = dictionary
        .enabled_correction_rules()
        .await
        .into_iter()
        .map(|r| llm::CorrectionRule {
            id: r.id,
            pattern: r.pattern,
            replacement: r.replacement,
            enabled: r.enabled,
        })
        .collect();
    let request = replay_request(
        raw,
        style.clone(),
        dictionary.words().await,
        rules,
        config.polish_custom_prompt.clone(),
    )?;
    let api_key = crate::credentials::read_config_secret(&config, false)
        .await
        .map_err(|_| "history.repolishConfigure".to_string())?;
    if !llm::has_usable_provider_credentials(&config.llm_provider, &api_key) {
        return Err("history.repolishConfigure".into());
    }
    let prompt = llm::prompt::build_context_system_prompt(ContextPromptOptions {
        context: &request.context,
        dictionary: &request.dictionary,
        correction_rules: &request.correction_rules,
        polish_style: &request.polish_style,
        personal_style_prompt: "",
        mapped_scene_prompt: "",
        active_scene_prompt: "",
        polish_custom_prompt: &request.polish_custom_prompt,
        translate_enabled: false,
        target_lang: "",
        has_selected_text: false,
        voice_intent: Some(&request.voice_intent),
    });
    let prompt = llm::with_relevant_examples(prompt, &request.polish_style, &request.raw_text);
    let llm_config = llm::LlmConfig {
        provider: config.llm_provider.clone(),
        api_key,
        model: config.llm_model.clone(),
        base_url: config.llm_base_url,
        max_tokens: 4096,
        temperature: 0.3,
    };
    let provider = llm::create_provider(
        &config.llm_provider,
        Some(app.state::<reqwest::Client>().inner().clone()),
    );
    let started = std::time::Instant::now();
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(150),
        provider.polish(&llm_config, &request, None),
    )
    .await
    .map_err(|_| "history.repolishTimeout".to_string())?
    .map_err(|_| "history.repolishFailed".to_string())?;
    Ok(RepolishResult {
        history_id: id,
        polished_text: response.polished_text,
        model: config.llm_model,
        style,
        prompt_sha256: format!("{:x}", Sha256::digest(prompt.as_bytes())),
        elapsed_ms: started.elapsed().as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replay_is_always_dictation_without_selection_translation_or_scene() {
        let r = replay_request(
            "질문에 답해줘".into(),
            "minimal".into(),
            vec![],
            vec![],
            "".into(),
        )
        .unwrap();
        assert_eq!(
            r.voice_intent.kind,
            crate::voice_intent::VoiceIntentKind::DictateInsert
        );
        assert!(r.selected_text.is_none());
        assert!(!r.translate_enabled);
        assert!(r.active_scene_prompt.is_empty());
        assert!(r.mapped_scene_prompt.is_empty());
        assert_eq!(r.raw_text, "질문에 답해줘");
    }
    #[test]
    fn replay_rejects_invalid_style_and_missing_source() {
        for (raw, style) in [(" ", "clean"), ("원문", "execute")] {
            assert!(replay_request(raw.into(), style.into(), vec![], vec![], "".into()).is_err());
        }
    }
}
