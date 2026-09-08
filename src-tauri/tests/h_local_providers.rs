//! Opt-in integration check. Uses real local engines; never runs in ordinary CI.
use opentypeless_lib::{
    app_detector::types::ContextProfile,
    llm::{LlmConfig, LlmProvider, PolishRequest},
    stt::{SttConfig, SttProvider},
    voice_intent::{VoiceIntent, VoiceIntentKind, VoiceOutputPlacement},
};

#[tokio::test]
#[ignore = "requires H_SETTINGS and H_PCM (16 kHz mono signed 16-bit PCM), and local engines"]
async fn direct_local_audio_and_polish() {
    let path = std::env::var("H_SETTINGS").expect("H_SETTINGS");
    let stored: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let value = &stored["app_config"];
    let value = if let Some(s) = value.as_str() {
        serde_json::from_str(s).unwrap()
    } else {
        value.clone()
    };
    let cfg = opentypeless_lib::storage::AppConfig::from_stored_value(value).unwrap();
    assert_eq!(cfg.llm_provider, "ollama");
    assert_eq!(cfg.llm_base_url, "http://127.0.0.1:11434/v1");
    let key = opentypeless_lib::credentials::resolve_config_secret(
        &cfg.stt_custom_api_key,
        "stt",
        "custom-whisper",
        &opentypeless_lib::credentials::SystemCredentialVault,
    )
    .unwrap();
    let mut stt = opentypeless_lib::stt::whisper_compat::WhisperCompatProvider::new(
        opentypeless_lib::stt::config::build_custom_whisper_config(
            &cfg.stt_custom_base_url,
            &cfg.stt_custom_model,
        )
        .unwrap(),
    );
    stt.connect(&SttConfig {
        api_key: key,
        ..Default::default()
    })
    .await
    .unwrap();
    stt.send_audio(&std::fs::read(std::env::var("H_PCM").expect("H_PCM")).unwrap())
        .await
        .unwrap();
    let text = stt.disconnect().await.unwrap().unwrap();
    assert!(!text.trim().is_empty());
    let req = PolishRequest {
        raw_text: text.clone(),
        context: ContextProfile::general_native().summary(),
        dictionary: vec![],
        correction_rules: vec![],
        polish_style: cfg.polish_style,
        mapped_scene_prompt: String::new(),
        active_scene_prompt: String::new(),
        polish_custom_prompt: cfg.polish_custom_prompt,
        translate_enabled: false,
        target_lang: "ko".into(),
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
        .unwrap(),
    };
    let llm = opentypeless_lib::llm::openai::OpenAiProvider::new();
    let config = LlmConfig {
        provider: cfg.llm_provider,
        model: cfg.llm_model,
        base_url: cfg.llm_base_url,
        api_key: String::new(),
        ..Default::default()
    };
    let result = llm.polish(&config, &req, None).await.unwrap();
    assert!(!result.polished_text.trim().is_empty());
    println!("STT: {text}\nPolished: {}", result.polished_text);
    let callback: opentypeless_lib::llm::ChunkCallback = Box::new(|_| {});
    let streamed = llm.polish(&config, &req, Some(&callback)).await.unwrap();
    assert!(!streamed.polished_text.trim().is_empty());
    println!("Streamed: {}", streamed.polished_text);
}
