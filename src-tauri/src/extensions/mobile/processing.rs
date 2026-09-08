use crate::{
    app_detector::types::ContextProfile,
    credentials::{resolve_llm_config_secret, resolve_stt_config_secret, SystemCredentialVault},
    llm::{self, LlmConfig, PolishRequest},
    storage,
    stt::{self, SttConfig},
    voice_intent::{VoiceIntent, VoiceIntentKind, VoiceOutputPlacement},
};
use serde::Serialize;
use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::sync::Semaphore;
static SECRET_GATE: LazyLock<Arc<Semaphore>> = LazyLock::new(|| Arc::new(Semaphore::new(1)));

async fn read_secret(config: &storage::AppConfig, stt: bool) -> Result<String, String> {
    let permit = SECRET_GATE
        .clone()
        .try_acquire_owned()
        .map_err(|_| "컴퓨터에서 이전 키체인 접근 승인을 기다리고 있습니다.")?;
    let config = config.clone();
    let task = tokio::task::spawn_blocking(move || {
        // Retain this permit if a native keychain dialog outlives the HTTP request.
        let _permit = permit;
        if stt {
            resolve_stt_config_secret(&config, &SystemCredentialVault)
        } else {
            resolve_llm_config_secret(&config, &SystemCredentialVault)
        }
    });
    tokio::time::timeout(Duration::from_secs(20), task)
        .await
        .map_err(|_| "컴퓨터에서 H 앱의 키체인 접근을 허용한 뒤 다시 시도하세요.")?
        .map_err(|_| "인증 정보를 읽는 작업이 중단되었습니다.")?
        .map_err(|_| "컴퓨터에서 H 앱의 인증 정보와 키체인 접근 권한을 확인하세요.".into())
}

use tauri::Manager;
#[derive(Serialize)]
pub struct ResultText {
    pub raw_text: String,
    pub polished_text: String,
    pub warning: Option<String>,
}
pub fn supported(provider: &str) -> bool {
    provider == stt::config::CUSTOM_WHISPER_PROVIDER
        || stt::config::get_whisper_config(provider).is_some()
}
pub async fn process(app: &tauri::AppHandle, audio: &[u8]) -> Result<ResultText, String> {
    let config = app
        .state::<storage::ConfigManager>()
        .load()
        .await
        .map_err(|_| "앱 설정을 읽지 못했습니다.")?;
    if !supported(&config.stt_provider) {
        return Err("모바일 입력은 현재 Local / Custom Whisper 및 Whisper 호환 STT를 지원합니다. H 앱의 음성 인식 설정을 확인하세요.".into());
    }
    let custom = if config.stt_provider == stt::config::CUSTOM_WHISPER_PROVIDER {
        Some(
            stt::config::build_custom_whisper_config(
                &config.stt_custom_base_url,
                &config.stt_custom_model,
            )
            .map_err(|_| "STT 주소와 모델 설정을 확인하세요.")?,
        )
    } else {
        None
    };
    let client = app.state::<reqwest::Client>().inner().clone();
    let key = read_secret(&config, true).await?;
    let mut provider = stt::create_provider(&config.stt_provider, custom, Some(client.clone()))
        .map_err(|_| "STT 제공자를 시작하지 못했습니다.")?;
    provider
        .connect(&SttConfig {
            api_key: key,
            language: (config.stt_language != "multi").then(|| config.stt_language.clone()),
            ..Default::default()
        })
        .await
        .map_err(|_| "STT에 연결하지 못했습니다. H 앱의 모델 엔진과 설정을 확인하세요.")?;
    provider
        .send_audio(audio)
        .await
        .map_err(|_| "STT에 음성을 전달하지 못했습니다.")?;
    let raw = tokio::time::timeout(Duration::from_secs(100), provider.disconnect())
        .await
        .map_err(|_| "음성 인식 시간이 초과되었습니다. STT 연결을 확인하세요.")?
        .map_err(|_| "음성 인식에 실패했습니다. STT 연결과 모델을 확인하세요.")?
        .unwrap_or_default();
    polish(app, &config, client, raw).await
}
async fn polish(
    app: &tauri::AppHandle,
    config: &storage::AppConfig,
    client: reqwest::Client,
    raw: String,
) -> Result<ResultText, String> {
    let mut result = ResultText {
        polished_text: raw.clone(),
        raw_text: raw.clone(),
        warning: None,
    };
    if raw.trim().is_empty() || !config.polish_enabled {
        return Ok(result);
    }
    // Mobile uses desktop providers/prompts but never desktop focus, selection, or keyboard output.
    let key = match read_secret(config, false).await {
        Ok(key) => key,
        Err(error) => {
            result.warning = Some(format!("{error} AI 교정 대신 원문을 표시합니다."));
            return Ok(result);
        }
    };
    if config.llm_provider == "cloud"
        || !llm::has_usable_provider_credentials(&config.llm_provider, &key)
    {
        result.warning = Some(
            "모바일 AI 다듬기에는 Ollama 또는 직접 API 제공자를 설정하세요. 원문을 표시합니다."
                .into(),
        );
        return Ok(result);
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
    let request = PolishRequest {
        raw_text: raw,
        context: ContextProfile::general_native().summary(),
        dictionary: dictionary.words().await,
        correction_rules: rules,
        polish_style: config.polish_style.clone(),
        mapped_scene_prompt: String::new(),
        active_scene_prompt: config
            .active_scene
            .as_ref()
            .map(|s| s.prompt_template.clone())
            .unwrap_or_default(),
        polish_custom_prompt: config.polish_custom_prompt.clone(),
        translate_enabled: config.translate_enabled,
        target_lang: config.translation.active_target.clone(),
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
        .map_err(|_| "모바일 받아쓰기 요청을 만들지 못했습니다.")?,
    };
    let provider = llm::create_provider(&config.llm_provider, Some(client));
    let llm_config = LlmConfig {
        provider: config.llm_provider.clone(),
        api_key: key,
        model: config.llm_model.clone(),
        base_url: config.llm_base_url.clone(),
        max_tokens: 4096,
        temperature: 0.3,
    };
    match tokio::time::timeout(
        std::time::Duration::from_secs(150),
        provider.polish(&llm_config, &request, None),
    )
    .await
    {
        Ok(Ok(response)) if !response.polished_text.trim().is_empty() => {
            result.polished_text = response.polished_text
        }
        _ => result.warning = Some("AI 다듬기에 실패하여 음성 인식 원문을 표시합니다.".into()),
    }
    Ok(result)
}
